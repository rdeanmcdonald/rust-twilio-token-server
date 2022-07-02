mod jwt;
mod settings;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use settings::Settings;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[get("/token/{id}")]
async fn token(data: web::Data<AppState>, id: web::Path<String>) -> impl Responder {
    let token = data.jwt.gen_token(&data.settings, format!("{}", id));
    HttpResponse::Ok().body(token)
}

struct AppState {
    settings: Settings,
    jwt: jwt::Jwt,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = Settings::new().expect("INVALID SETTINGS");
    println!("Starting server: {:?}", settings);

    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(AppState {
                settings: Settings::new().expect("INVALID SETTINGS"),
                jwt: jwt::Jwt::new(),
            }))
            .service(hello)
            .service(echo)
            .service(token)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", settings.server.port))?
    .run()
    .await
}
