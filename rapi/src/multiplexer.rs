use bytes::{Bytes, BytesMut};
use futures::SinkExt;
use futures::StreamExt;
use log::{trace, warn};
use std::collections::HashMap;
use std::convert::TryFrom;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
pub struct Job {
    pub job_id: String,
    pub request: Bytes,
    pub tx: Sender<Bytes>,
}

struct PartialResult {
    pid: String,
    n: i32,
    m: i32,
    payload: BytesMut,
}

impl TryFrom<tmq::Multipart> for PartialResult {
    type Error = &'static str;

    fn try_from(item: tmq::Multipart) -> Result<Self, Self::Error> {
        match item.len() {
            3 => Ok(PartialResult {
                pid: item[0].as_str().unwrap().into(),
                n: std::str::FromStr::from_str(
                    item[1].as_str().unwrap().split("/").nth(0).unwrap(),
                )
                .unwrap(),
                m: std::str::FromStr::from_str(
                    item[1].as_str().unwrap().split("/").nth(1).unwrap(),
                )
                .unwrap(),
                payload: BytesMut::from(&item[2][..]),
            }),
            _ => Err("len"),
        }
    }
}

pub fn start(
    ctx: &tmq::Context,
    rep_addr: &str,
    req_addr: &str,
    rx_job: Receiver<Job>,
) -> Result<(), tmq::TmqError> {
    let zmq_identity = &uuid::Uuid::new_v4().to_string();

    let pusher = tmq::push(&ctx).bind(req_addr)?;
    let dealer = tmq::dealer(&ctx)
        .set_identity(zmq_identity.as_bytes())
        .bind(rep_addr)?;
    tokio::spawn(multiplexer(rx_job, zmq_identity.into(), pusher, dealer));
    Ok(())
}

fn listen_core(mut dealer: tmq::dealer::Dealer) -> Receiver<PartialResult> {
    let (mut tx, rx) = channel(1);
    tokio::spawn(async move {
        while let Some(msg) = dealer.next().await {
            let partial = PartialResult::try_from(msg.unwrap()).unwrap();
            if let Err(_) = tx.send(partial).await {
                break;
            }
        }
    });
    rx
}

async fn multiplexer(
    mut rx_job: Receiver<Job>,
    zmq_identity: String,
    mut pusher: tmq::push::Push,
    dealer: tmq::dealer::Dealer,
) {
    let mut core = listen_core(dealer);

    let mut jobs = HashMap::new();
    loop {
        tokio::select! {
            Some(job) = rx_job.recv() => {
                trace!("sending job_id: {:?}", job.job_id);
                let msg = vec![zmq_identity.as_bytes(), job.job_id.as_bytes(), &job.request[..]];
                if let Err(e) = pusher.send(msg).await{
                    warn!("{}", e);
                    // TODO handle error
                    break
                }
                jobs.insert(job.job_id, job.tx);
            },
            Some(partial) = core.recv() => {
                if let Some(id) = msg[0].as_str(){
                    if let Some(mut job) = jobs.remove(id) {
                        if let Err(e) = job.send(BytesMut::from(&msg[1][..]).into()).await {
                            // TODO handle error
                            warn!("{}", e);
                            break
                        }
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn mock_core(ctx: &tmq::Context, req_addr: &str, rep_addr: &str) {
        let mut core_pull = tmq::pull(&ctx).connect(&req_addr).unwrap();
        let mut core_router = tmq::router(&ctx).connect(&rep_addr).unwrap();
        tokio::spawn(async move {
            while let Some(Ok(msg)) = core_pull.next().await {
                core_router.send(msg).await.unwrap();
            }
        });
    }

    #[tokio::test]
    async fn ping_pong() {
        let pusher_addr = "inproc://".to_string() + &uuid::Uuid::new_v4().to_string();
        let dealer_addr = "inproc://".to_string() + &uuid::Uuid::new_v4().to_string();

        let ctx = tmq::Context::new();

        mock_core(&ctx, &pusher_addr, &dealer_addr).await;
        let (mut job_multiplexer, rx_job) = tokio::sync::mpsc::channel(1);

        start(&ctx, &pusher_addr, &dealer_addr, rx_job).unwrap();

        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let data = b"data";
        let job = Job {
            job_id: "id".into(),
            request: BytesMut::from(&data[..]).into(),
            tx,
        };
        job_multiplexer.send(job).await.unwrap();

        assert_eq!(rx.recv().await.unwrap(), data.as_ref());
    }
}
