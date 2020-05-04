use crate::multiplexer;
use crate::oneseismic;
use crate::state::AppState;
use actix_http::ResponseBuilder;
use actix_web::{error, get, http::header, http::StatusCode, web, HttpResponse, Result};
use log::trace;
use prost::bytes::BytesMut;
use prost::Message;
use std::fmt;
use tokio::sync::mpsc;
use uuid::Uuid;

#[get("/{account}/{guid}/slice/{dim}/{ord}")]
async fn slice<'a>(
    p: web::Path<(String, String, i32, i32)>,
    state: web::Data<AppState<'a>>,
) -> Result<HttpResponse> {
    let ar = oneseismic::ApiRequest {
        guid: p.1.clone(),
        requestid: Uuid::new_v4().to_string(),
        root: p.0.clone(),
        shape: None,
        function: None,
    };
    trace!("ApiRequest: {:?}", &ar);
    match fetch(state.sender.clone(), ar).await?.function {
        Some(sr) => Ok(HttpResponse::Ok().json(sr)),
        _ => Err(error::ErrorNotFound("not found")),
    }
}

impl error::ResponseError for FetchError {
    fn error_response(&self) -> HttpResponse {
        ResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub async fn fetch(
    tx_job: mpsc::Sender<multiplexer::Job>,
    ar: oneseismic::ApiRequest,
) -> Result<oneseismic::FetchResponse, FetchError> {
    let (tx_fr, mut rx_fr) = mpsc::channel(1);

    let mut request = BytesMut::with_capacity(10);
    ar.encode(&mut request)?;
    let job = multiplexer::Job {
        job_id: ar.requestid,
        request: request.into(),
        tx: tx_fr,
    };

    tx_job.clone().send(job).await?;
    let bytes = rx_fr.recv().await.ok_or(FetchError::RecvError)?;
    oneseismic::FetchResponse::decode(&bytes[..]).map_err(FetchError::DecodeError)
}

#[derive(Debug)]
pub enum FetchError {
    RecvError,
    SendError(mpsc::error::SendError<multiplexer::Job>),
    DecodeError(prost::DecodeError),
    EncodeError(prost::EncodeError),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "internal error")
    }
}

impl From<prost::DecodeError> for FetchError {
    fn from(err: prost::DecodeError) -> FetchError {
        FetchError::DecodeError(err)
    }
}

impl From<prost::EncodeError> for FetchError {
    fn from(err: prost::EncodeError) -> FetchError {
        FetchError::EncodeError(err)
    }
}

impl From<mpsc::error::SendError<multiplexer::Job>> for FetchError {
    fn from(err: mpsc::error::SendError<multiplexer::Job>) -> FetchError {
        FetchError::SendError(err)
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

    async fn mock_multiplexer(mut rx_job: Receiver<multiplexer::Job>) {
        let mut job = rx_job.recv().await.unwrap();
        let fr = oneseismic::FetchResponse {
            requestid: job.job_id.clone(),
            function: None,
        };
        let mut response = BytesMut::with_capacity(10);
        fr.encode(&mut response).unwrap();
        job.tx.send(response.into()).await.unwrap();
    }

    #[tokio::test]
    async fn fetch_ok() {
        let (tx_job, rx_job) = mpsc::channel(1);
        let ar = oneseismic::ApiRequest {
            requestid: "0".into(),
            root: "".into(),
            guid: "".into(),
            shape: None,
            function: None,
        };

        tokio::spawn(mock_multiplexer(rx_job));

        let fr = fetch(tx_job.clone(), ar).await;
        assert_eq!(fr.unwrap().requestid, "0");
    }

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
            web::Path::from(("".into(), "".into(), 0i32, 0i32)),
            web::Data::new(AppState {
                sender: tx_job,
                jwks: std::collections::HashMap::new(),
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
            web::Path::from(("".into(), "".into(), 0i32, 0i32)),
            web::Data::new(AppState {
                sender: tx_job,
                jwks: std::collections::HashMap::new(),
            }),
        );
        assert!(resp.await.is_err());
    }
}
