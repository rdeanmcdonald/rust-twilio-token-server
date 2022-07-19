mod enterprise;
mod jwt;
mod settings;

use actix_cors::Cors;
use actix_web::dev::ServiceRequest;
use actix_web::middleware::Logger;
use actix_web::{
    get, http::header::ORIGIN, post, web, web::Data, App, Error, HttpResponse, HttpServer,
    Responder, Result,
};
use enterprise::Enterprises;
use env_logger::Env;
use log::{debug, info};
use regex::Regex;
use settings::Settings;

use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::middleware::HttpAuthentication;

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

fn auth_err(req: &ServiceRequest) -> Error {
    let config = req
        .app_data::<Config>()
        .map(|data| data.clone())
        .unwrap_or_else(Default::default);
    // .scope("urn:example:channel=HBO&urn:example:rating=G,PG-13");

    AuthenticationError::from(config).into()
}

async fn bearer_auth_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    debug!("VALIDATING TOKEN: {}", credentials.token());
    let enterprise = req
        .app_data::<web::Data<AppState>>()
        .ok_or(auth_err(&req))?
        .enterprises
        .find(credentials.token())
        .ok_or(auth_err(&req))?;

    debug!("FOUND ENTERPRISE: {:?}", enterprise);
    let origin = req
        .headers()
        .get(ORIGIN)
        .ok_or(auth_err(&req))?
        .to_str()
        .map_err(|_e| auth_err(&req))?;

    debug!("ORIGIN: {}", origin);
    if credentials.token() == &enterprise.id[..] && &enterprise.origin[..] == origin {
        debug!("AUTHORIZED");
        Ok(req)
    } else {
        debug!("NOT AUTHORIZED");
        Err(auth_err(&req))
    }
}

#[derive(Debug)]
struct AppState {
    enterprises: Enterprises,
    settings: Settings,
    jwt: jwt::Jwt,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let settings = Settings::new().expect("INVALID SETTINGS");
    let port = settings.server.port;
    info!("Starting server: {:?}", settings);

    let app_data = web::Data::new(AppState {
        enterprises: Enterprises::new(),
        settings,
        jwt: jwt::Jwt::new(),
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_method()
            .allow_any_header()
            // .allowed_origin("http://localhost:8081")
            .allowed_origin_fn(|origin, _req_head| {
                debug!("CHECKING ORIGIN: {}", origin.to_str().unwrap());
                let re = Regex::new(r"http://localhost:*").unwrap();
                let allowed = re.is_match(std::str::from_utf8(origin.as_bytes()).unwrap());
                debug!("IS ALLOWED: {}", allowed);
                allowed
            })
            // .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .max_age(3600);

        let auth = HttpAuthentication::bearer(bearer_auth_validator);

        App::new().service(
            web::scope("/visitor")
                // wrapps called from inside out, so this is the last call
                // .wrap_fn(|req, srv| {
                //     println!("Hi from start. You requested: {}", req.path());
                //     srv.call(req)
                // })
                // this wrap is called first
                .wrap(auth)
                .wrap(cors)
                .wrap(Logger::default())
                .app_data(Data::clone(&app_data))
                .service(hello)
                .service(echo)
                .service(token)
                .route("/hey", web::get().to(manual_hello))
                .service(token),
        )
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
