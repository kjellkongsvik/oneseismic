use crate::errors::FetchError;
use crate::multiplexer;
use crate::oneseismic;
use crate::state::AppState;
use actix_http::ResponseBuilder;
use actix_web::{
    error, get, http::header, http::StatusCode, web, HttpRequest, HttpResponse, Result,
};
use log::{error, trace};
use prost::bytes::{Bytes, BytesMut};
use prost::Message;
use tokio::sync::mpsc;

fn tok(req: HttpRequest) -> String {
    req.headers()
        .get("Authorization")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

#[derive(serde::Deserialize)]
struct SliceParams {
    guid: String,
    dim: i32,
    ord: i32,
}

#[get("/{guid}/slice/{dim}/{ord}")]
async fn slice<'a>(
    p: web::Path<SliceParams>,
    state: web::Data<AppState<'a>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    trace!("slice");
    let (mut tx, rx) = mpsc::channel(1);
    let mut out = fetch(state.sender.clone(), &p.guid, p.dim, p.ord, &tok(req)).await?;
    tokio::spawn(async move {
        while let Some(Ok(bytes)) = out.recv().await {
            let fr = oneseismic::FetchResponse::decode(bytes)
                .map_err(FetchError::DecodeError)
                .expect("not a FetchResponse");
            trace!("{:?} got", fr.requestid);
            if let Some(oneseismic::fetch_response::Function::Slice(sr)) = fr.function {
                let mut response = bytes::BytesMut::with_capacity(10);
                if let Err(e) = sr.encode(&mut response) {
                    panic!("{}", e);
                }
                let bl: Result<Bytes, ()> = Ok(bytes::Bytes::copy_from_slice(
                    &(response.len() as i32).to_le_bytes(),
                ));
                if let Err(e) = tx.send(bl).await {
                    error!("{}", e);
                }
                if let Err(e) = tx.send(Ok(bytes::Bytes::from(response))).await {
                    error!("{}", e);
                }
            };
        }
    });
    HttpResponse::Ok().streaming(rx).await
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

async fn fetch(
    mut tx_job: mpsc::Sender<multiplexer::Job>,
    guid: &str,
    dim: i32,
    lineno: i32,
    token: &str,
) -> Result<mpsc::Receiver<Result<bytes::Bytes, ()>>, FetchError> {
    let api_request = oneseismic::ApiRequest {
        storage_endpoint: "https://oneseismicdev.blob.core.windows.net".into(),
        token: token.into(),
        guid: guid.into(),
        requestid: uuid::Uuid::new_v4().to_string(),
        shape: Some(oneseismic::FragmentShape {
            dim0: 64,
            dim1: 64,
            dim2: 64,
        }),
        function: Some(oneseismic::api_request::Function::Slice(
            oneseismic::ApiSlice { dim, lineno },
        )),
    };
    let _rid = api_request.requestid.clone();
    trace!("{:?} created", &_rid);
    let (tx_response, rx_response) = mpsc::channel(1);
    let mut request = BytesMut::with_capacity(10);
    api_request.encode(&mut request)?;
    let job = multiplexer::Job {
        job_id: api_request.requestid,
        request: request.into(),
        tx_response,
    };
    tx_job.send(job).await?;
    trace!("{:?}: requested", &_rid);

    Ok(rx_response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multiplexer;
    use tokio::sync::mpsc::Receiver;

    async fn mock_multiplexer(mut rx_job: Receiver<multiplexer::Job>) {
        let mut job = rx_job.recv().await.unwrap();
        let fr = oneseismic::FetchResponse {
            requestid: job.job_id.clone(),
            function: Some(oneseismic::fetch_response::Function::Slice(
                oneseismic::SliceResponse {
                    slice_shape: None,
                    tiles: vec![],
                },
            )),
        };
        let mut response = bytes::BytesMut::with_capacity(10);
        fr.encode(&mut response).unwrap();

        job.tx_response
            .send(Ok(bytes::Bytes::from(response)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn fetch_ok() {
        let (tx_job, rx_job) = mpsc::channel(1);
        tokio::spawn(mock_multiplexer(rx_job));
        let mut rx_response = fetch(tx_job, "", 0, 0, "").await.unwrap();
        while let Some(Ok(r)) = rx_response.recv().await {
            let sr = oneseismic::FetchResponse::decode(r)
                .map_err(FetchError::DecodeError)
                .unwrap()
                .function;
            if let Some(oneseismic::fetch_response::Function::Slice(t)) = sr {
                assert_eq!(t.slice_shape, None);
                assert_eq!(t.tiles, vec![]);
            }
        }
    }
}
