use crate::state;
use actix_web::{dev::ServiceRequest, error, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, decode_header};
use log::trace;
use serde::{Deserialize, Serialize};

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    let state = req
        .app_data::<state::AppState>()
        .ok_or(error::ErrorInternalServerError("state"))?;

    let kid = decode_header(credentials.token())
        .map_err(|_| error::ErrorBadRequest("bad token"))?
        .kid
        .ok_or(error::ErrorBadRequest("token missing kid"))?;
    trace!("kid: {:?}", kid);

    let key = state
        .jwks
        .get(&kid)
        .ok_or(error::ErrorBadRequest("invalid kid in token"))?;
    trace!("key: {:?}", key);

    let t = decode::<Claims>(credentials.token(), key, &state.validation);
    trace!("claims: {:?}", t);
    t.map_err(|_| error::ErrorUnauthorized("invalid token"))?;
    Ok(req)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub nbf: usize,
    pub iss: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use actix_web_httpauth::middleware::HttpAuthentication;
    use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
    use openssl::rsa::Rsa;
    use std::time::SystemTime;

    #[actix_rt::test]
    async fn test_no_auth() {
        let mut app =
            test::init_service(App::new().route("/", web::get().to(|| async { "" }))).await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_auth() {
        lazy_static! {
            static ref RSA: Rsa<openssl::pkey::Private> = Rsa::generate(2048).unwrap();
            static ref PRIVATE_KEY: Vec<u8> = RSA.private_key_to_pem().unwrap();
            static ref PUBLIC_KEY: Vec<u8> = RSA.public_key_to_pem().unwrap();
        }

        let mut jwks = std::collections::HashMap::new();
        let kid = "0";
        jwks.insert(kid.into(), DecodingKey::from_rsa_pem(&PUBLIC_KEY).unwrap());

        let mut app = test::init_service(
            App::new()
                .data(state::AppState {
                    jwks,
                    sender: tokio::sync::mpsc::channel(1).0,
                    validation: Validation::new(Algorithm::RS256),
                    root: "".into(),
                })
                .wrap(HttpAuthentication::bearer(validator))
                .route("/", web::get().to(|| async { "" })),
        )
        .await;

        let mut h = Header::new(Algorithm::RS256);
        h.kid = Some(kid.into());
        let req = test::TestRequest::get()
            .header(
                "Authorization",
                "Bearer ".to_string()
                    + &encode(
                        &h,
                        &Claims {
                            exp: SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as usize
                                + 3600,
                            nbf: 0,
                            iss: "".into(),
                        },
                        &EncodingKey::from_rsa_pem(&PRIVATE_KEY).unwrap(),
                    )
                    .unwrap(),
            )
            .uri("/")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }
}
