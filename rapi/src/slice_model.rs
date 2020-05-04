use crate::multiplexer;
use crate::oneseismic;
use prost::bytes::BytesMut;
use prost::Message;
use std::fmt;
use tokio::sync::mpsc;

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
}
