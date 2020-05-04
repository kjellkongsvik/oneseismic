#[macro_use]
extern crate lazy_static;
use crate::config::CONFIG;
use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use jsonwebtoken::{Algorithm, Validation};

mod auth;
mod config;
mod errors;
mod openid;
mod state;
mod store;

#[actix_rt::main]
async fn main() -> Result<(), errors::Error> {
    dotenv::dotenv().ok();

    let mut aud = std::collections::HashSet::new();
    aud.insert(CONFIG.audience.clone());

    let oidc = openid::get_config(CONFIG.authserver.as_ref()).await?;

    let mut validation = Validation::default();
    validation.algorithms = vec![Algorithm::RS256, Algorithm::RS384, Algorithm::RS512];
    validation.iss = Some(oidc.issuer.clone());
    validation.aud = Some(aud);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .data(state::AppState {
                oidc: oidc.clone(),
                validation: validation.clone(),
            })
            .wrap(HttpAuthentication::bearer(auth::obo))
            .wrap(HttpAuthentication::bearer(auth::validator))
            .service(store::list)
            .service(store::dimensions)
            .service(store::lines)
    })
    .bind(&CONFIG.host_addr)?
    .run()
    .await
    .map_err(Into::into)
}
