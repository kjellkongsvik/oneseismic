use crate::errors::FetchError;
use crate::multiplexer;
use crate::oneseismic;
use crate::state::AppState;
use crate::CONFIG;
use actix_http::ResponseBuilder;
use actix_web::{error, get, http::header, http::StatusCode, web, HttpResponse, Result};
use log::trace;
use prost::bytes::BytesMut;
use prost::Message;
use tokio::sync::mpsc;
use uuid::Uuid;

#[get("/{account}/{guid}/slice/{dim}/{ord}")]
async fn slice<'a>(
    p: web::Path<(String, i32, i32)>,
    state: web::Data<AppState<'a>>,
) -> Result<HttpResponse> {
    let ar = oneseismic::ApiRequest {
        storage_endpoint: "".into(),
        token: "".into(),
        guid: p.0.clone(),
        requestid: Uuid::new_v4().to_string(),
        root: CONFIG.azure_storage_account.clone(),
        shape: Some(oneseismic::FragmentShape {
            dim0: 64,
            dim1: 64,
            dim2: 64,
        }),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multiplexer;
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
            storage_endpoint: "".into(),
            token: "".into(),
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
}
