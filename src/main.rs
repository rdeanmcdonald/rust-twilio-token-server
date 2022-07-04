mod jwt;
mod settings;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use env_logger::Env;
use regex::Regex;
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
async fn token(data: web::Data<AppState>, id: web::Path<String>) -> Result<impl Responder> {
    let token = data.jwt.gen_token(&data.settings, format!("{}", id));
    Ok(web::Json(token))
}

struct AppState {
    settings: Settings,
    jwt: jwt::Jwt,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let settings = Settings::new().expect("INVALID SETTINGS");
    println!("Starting server: {:?}", settings);

    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req_head| {
                println!("{:?}", origin);
                let re = Regex::new(r"http://localhost:*").unwrap();

                let allowed = re.is_match(std::str::from_utf8(origin.as_bytes()).unwrap());
                println!("HIIIII");
                println!("{:?}", allowed);

                allowed
            })
            .allowed_methods(vec!["GET", "POST"])
            .max_age(3600);

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
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
