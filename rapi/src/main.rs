#[macro_use]
extern crate lazy_static;
use crate::config::CONFIG;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use actix_web::{HttpResponse, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use jsonwebtoken::{Algorithm, Validation};

mod auth;
mod config;
mod errors;
mod openid;
mod state;

async fn index() -> impl Responder {
    HttpResponse::Ok().json(vec!["hello", "world"])
}

#[actix_rt::main]
async fn main() -> Result<(), errors::Error> {
    dotenv::dotenv().ok();
    env_logger::init();

    let mut validation = Validation::default();
    validation.algorithms = vec![Algorithm::RS256, Algorithm::RS384, Algorithm::RS512];
    validation.iss = Some(CONFIG.issuer.clone());

    let jwks = openid::jwks(CONFIG.authserver.as_ref()).await?;
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .data(state::AppState {
                jwks: jwks.clone(),
                validation: validation.clone(),
            })
            .wrap(HttpAuthentication::bearer(auth::validator))
            .service(web::scope("/").route("", web::get().to(index)))
    })
    .bind(&CONFIG.host_addr)?
    .run()
    .await
    .map_err(Into::into)
}