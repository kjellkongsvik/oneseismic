use futures::SinkExt;
use futures::StreamExt;

use bytes::{Bytes, BytesMut};
use log::warn;
use std::collections::HashMap;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
pub struct Job {
    pub job_id: String,
    pub request: Bytes,
    pub tx: Sender<Bytes>,
}

pub fn start(
    ctx: &tmq::Context,
    pusher_addr: &str,
    dealer_addr: &str,
    rx_job: Receiver<Job>,
) -> Result<(), tmq::TmqError> {
    let zmq_identity = &uuid::Uuid::new_v4().to_string();

    let pusher = tmq::push(&ctx).bind(pusher_addr)?;
    let dealer = tmq::dealer(&ctx)
        .set_identity(zmq_identity.as_bytes())
        .bind(dealer_addr)?;
    tokio::spawn(multiplexer(rx_job, zmq_identity.into(), pusher, dealer));
    Ok(())
}

fn listen_core(mut dealer: tmq::dealer::Dealer) -> Receiver<Result<tmq::Multipart, tmq::TmqError>> {
    let (mut tx, rx) = channel(1);
    tokio::spawn(async move {
        while let Some(msg) = dealer.next().await {
            if let Err(_) = tx.send(msg).await {
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
                let msg = vec![zmq_identity.as_bytes(), job.job_id.as_bytes(), &job.request[..]];
                if let Err(e) = pusher.send(msg).await{
                    warn!("{}", e);
                    // TODO handle error
                    break
                }
                jobs.insert(job.job_id, job.tx);
            },
            Some(Ok(msg)) = core.recv() => {
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
