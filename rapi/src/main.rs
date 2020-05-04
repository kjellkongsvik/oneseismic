#[macro_use]
extern crate lazy_static;
use crate::config::CONFIG;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use jsonwebtoken::{Algorithm, Validation};
use tmq::Context;

mod auth;
mod config;
mod errors;
mod multiplexer;
mod openid;
mod slice_handler;
mod slice_model;
mod state;

pub mod oneseismic {
    include!(concat!(env!("OUT_DIR"), "/oneseismic.rs"));
}

#[actix_rt::main]
async fn main() -> Result<(), errors::Error> {
    dotenv::dotenv().ok();
    env_logger::init();

    let (tx_job, rx_job) = tokio::sync::mpsc::channel(1);
    multiplexer::start(
        &Context::new(),
        &CONFIG.zmq_push_addr,
        &CONFIG.zmq_deal_addr,
        rx_job,
    )?;

    let mut validation = Validation::default();
    validation.algorithms = vec![Algorithm::RS256, Algorithm::RS384, Algorithm::RS512];
    validation.iss = Some(CONFIG.issuer.clone());

    let jwks = openid::jwks(CONFIG.authserver.as_ref()).await?;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .data(state::AppState {
                sender: tx_job.clone(),
                jwks: jwks.clone(),
                validation: validation.clone(),
                root: CONFIG.account.clone(),
            })
            .wrap(HttpAuthentication::bearer(auth::validator))
            .service(web::scope("/").configure(slice_handler::service))
    })
    .bind(&CONFIG.host_addr)?
    .run()
    .await
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multiplexer;
    use crate::oneseismic;
    use actix_web::dev::Service;
    use actix_web::{test, App};
    use bytes::BytesMut;
    use futures::SinkExt;
    use futures::StreamExt;
    use prost::Message;

    async fn mock_core(ctx: &tmq::Context, req_addr: &str, rep_addr: &str) {
        let mut core_pull = tmq::pull(&ctx).connect(&req_addr).unwrap();
        let mut core_router = tmq::router(&ctx).connect(&rep_addr).unwrap();
        tokio::spawn(async move {
            while let Some(Ok(mut msg)) = core_pull.next().await {
                let request_id = std::str::from_utf8(&msg[1][..]).unwrap();

                let fr = oneseismic::FetchResponse {
                    requestid: request_id.into(),
                    function: Some(oneseismic::fetch_response::Function::Slice(
                        oneseismic::SliceResponse { tiles: vec![] },
                    )),
                };
                let mut response = BytesMut::with_capacity(10);
                fr.encode(&mut response).unwrap();

                msg[2] = tmq::Message::from(&response[..]);
                core_router.send(msg).await.unwrap();
            }
        });
    }

    #[actix_rt::test]
    async fn test_int() {
        let pusher_addr = "inproc://".to_string() + &uuid::Uuid::new_v4().to_string();
        let dealer_addr = "inproc://".to_string() + &uuid::Uuid::new_v4().to_string();
        let ctx = Context::new();
        let (tx_job, rx_job) = tokio::sync::mpsc::channel(1);
        multiplexer::start(&ctx, &pusher_addr, &dealer_addr, rx_job).unwrap();
        mock_core(&ctx, &pusher_addr, &dealer_addr).await;

        let mut app = test::init_service(
            App::new()
                .data(state::AppState {
                    sender: tx_job.clone(),
                    jwks: std::collections::HashMap::new(),
                    validation: Validation::default(),
                    root: "acc".into(),
                })
                .configure(slice_handler::service),
        )
        .await;

        let req = test::TestRequest::with_uri("/name/slice/0/0").to_request();

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), 200);
    }
}
