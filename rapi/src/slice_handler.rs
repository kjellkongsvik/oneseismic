use crate::oneseismic;
use crate::slice_model;
use crate::state::AppState;
use actix_http::ResponseBuilder;
use actix_web::{error, http::header, http::StatusCode, web, HttpResponse, Result};
use log::trace;
use uuid::Uuid;

pub fn service(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/{guid}/slice/{dim}/{ord}").route(web::get().to(slice)));
}

async fn slice<'a>(
    p: web::Path<(String, i32, i32)>,
    state: web::Data<AppState<'a>>,
) -> Result<HttpResponse> {
    let ar = oneseismic::ApiRequest {
        guid: p.0.clone(),
        requestid: Uuid::new_v4().to_string(),
        root: state.root.clone(),
        shape: None,
        function: None,
    };
    trace!("ApiRequest: {:?}", &ar);
    match slice_model::fetch(state.sender.clone(), ar).await?.function {
        Some(sr) => Ok(HttpResponse::Ok().json(sr)),
        _ => Err(error::ErrorNotFound("not found")),
    }
}

impl error::ResponseError for slice_model::FetchError {
    fn error_response(&self) -> HttpResponse {
        ResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multiplexer;
    use actix_web::{http, web};
    use prost::bytes::BytesMut;
    use prost::Message;
    use tokio::sync::mpsc::Receiver;

    async fn mock_multiplexer_200(mut rx_job: Receiver<multiplexer::Job>) {
        let mut job = rx_job.recv().await.unwrap();
        let mut response = BytesMut::with_capacity(10);
        oneseismic::FetchResponse {
            requestid: job.job_id.clone(),
            function: Some(oneseismic::fetch_response::Function::Slice(
                oneseismic::SliceResponse { tiles: vec![] },
            )),
        }
        .encode(&mut response)
        .unwrap();
        job.tx.send(response.into()).await.unwrap();
    }

    #[actix_rt::test]
    async fn test_slice_ok() {
        let (tx_job, rx_job) = tokio::sync::mpsc::channel(1);
        tokio::spawn(mock_multiplexer_200(rx_job));

        let resp = slice(
            web::Path::from(("".into(), 0i32, 0i32)),
            web::Data::new(AppState {
                sender: tx_job,
                jwks: std::collections::HashMap::new(),
                validation: jsonwebtoken::Validation::default(),
                root: "".into(),
            }),
        );
        assert_eq!(resp.await.unwrap().status(), http::StatusCode::OK);
    }

    async fn mock_multiplexer_404(mut rx_job: Receiver<multiplexer::Job>) {
        let mut job = rx_job.recv().await.unwrap();
        let mut response = BytesMut::with_capacity(10);
        oneseismic::FetchResponse {
            requestid: job.job_id.clone(),
            function: None,
        }
        .encode(&mut response)
        .unwrap();
        job.tx.send(response.into()).await.unwrap();
    }

    #[actix_rt::test]
    async fn test_slice_not_found() {
        let (tx_job, rx_job) = tokio::sync::mpsc::channel(1);
        tokio::spawn(mock_multiplexer_404(rx_job));

        let resp = slice(
            web::Path::from(("".into(), 0i32, 0i32)),
            web::Data::new(AppState {
                sender: tx_job,
                jwks: std::collections::HashMap::new(),
                validation: jsonwebtoken::Validation::default(),
                root: "".into(),
            }),
        );
        assert!(resp.await.is_err());
    }
}
