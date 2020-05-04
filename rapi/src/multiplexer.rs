use bytes::Bytes;
use futures::SinkExt;
use futures::StreamExt;
use log::{error, trace};
use std::collections::HashMap;
use std::convert::TryFrom;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
pub struct Job {
    pub job_id: String,
    pub request: Bytes,
    pub tx_response: Sender<Result<bytes::Bytes, ()>>,
}

#[derive(PartialEq, Debug)]
pub struct PartialResult {
    pub pid: String,
    pub m: i32,
    pub n: i32,
    pub payload: bytes::Bytes,
}

impl TryFrom<tmq::Multipart> for PartialResult {
    type Error = &'static str;

    fn try_from(item: tmq::Multipart) -> Result<Self, Self::Error> {
        match item.len() {
            3 => {
                let pid = item[0].as_str().unwrap().to_string();
                let mn: Vec<_> = item[1]
                    .as_str()
                    .ok_or("bytes are not a valid string")?
                    .split("/")
                    .collect();
                if mn.len() != 2 {
                    return Err("malformed m/n");
                }
                let m = std::str::FromStr::from_str(mn[0]).expect("m is not an i32");
                let n = std::str::FromStr::from_str(mn[1]).expect("n is not an i32");
                let payload: bytes::BytesMut = item[2][..].into();
                Ok(PartialResult {
                    pid,
                    n,
                    m,
                    payload: bytes::Bytes::from(payload),
                })
            }
            _ => Err("Multipart length != 3"),
        }
    }
}

pub fn start(
    ctx: &tmq::Context,
    req_addr: &str,
    rep_addr: &str,
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
        while let Some(Ok(msg)) = dealer.next().await {
            if let Ok(partial) = PartialResult::try_from(msg) {
                if let Err(_) = tx.send(partial).await {
                    break;
                }
            };
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

    struct ProcStatus {
        tx_response: Sender<Result<bytes::Bytes, ()>>,
        incomplete: i32,
    }

    let mut jobs = HashMap::new();
    loop {
        tokio::select! {
            Some(job) = rx_job.recv() => {
                let msg = vec![zmq_identity.as_bytes(), job.job_id.as_bytes(), &job.request[..]];
                trace!("{}: sending req", job.job_id);
                if let Err(e) = pusher.send(msg).await{
                    error!("{}", e);
                }
                jobs.insert(job.job_id, ProcStatus{tx_response: job.tx_response, incomplete: -1});
            },
            Some(partial) = core.recv() => {
                // TODO dont remove and re./insert
                if let Some(mut ps) = jobs.remove(&partial.pid) {
                    trace!("{}: got {:?}/{:?}", partial.pid, partial.m, partial.n);
                    if ps.incomplete < 0 {
                        ps.incomplete = partial.n;
                    }
                    ps.incomplete = ps.incomplete - 1;
                    let b: Result<_, ()> = Ok(partial.payload);
                    if let Err(e) = ps.tx_response.send(b).await {
                        error!("{}", e);
                    }
                    trace!("{}: sent {:?}/{:?}", partial.pid, partial.m, partial.n);

                    if ps.incomplete > 0 {
                        jobs.insert(partial.pid, ps);
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use env_logger;
    use log::error;

    async fn mock_core(ctx: &tmq::Context, req_addr: &str, rep_addr: &str, n: i32) {
        let mut core_pull = tmq::pull(&ctx).connect(&req_addr).unwrap();
        let mut core_router = tmq::router(&ctx).connect(&rep_addr).unwrap();
        tokio::spawn(async move {
            while let Some(Ok(msg)) = core_pull.next().await {
                if msg.len() != 3 {
                    error!("msg.len() != 3");
                    break;
                }
                for i in 0..n {
                    let p = tmq::Multipart::from(vec![
                        msg[0].as_str().unwrap().as_bytes(),
                        msg[1].as_str().unwrap().as_bytes(),
                        "0/2".as_bytes(),
                        msg[2].as_str().unwrap().as_bytes(),
                    ]);

                    trace!("core replying: {:?}", i);
                    if let Err(e) = core_router.send(p).await {
                        error!("{:?}", e)
                    }
                }
            }
        });
    }

    #[tokio::test]
    #[ignore]
    async fn ping_pong() {
        env_logger::init();
        let pusher_addr = "inproc://".to_string() + &uuid::Uuid::new_v4().to_string();
        let dealer_addr = "inproc://".to_string() + &uuid::Uuid::new_v4().to_string();

        let ctx = tmq::Context::new();
        let n = 2;
        mock_core(&ctx, &pusher_addr, &dealer_addr, n).await;
        let (mut tx_job, rx_job) = tokio::sync::mpsc::channel(1);

        start(&ctx, &pusher_addr, &dealer_addr, rx_job).unwrap();

        let (tx_response, mut rx_response) = tokio::sync::mpsc::channel(1);
        let payload = uuid::Uuid::new_v4().to_string();
        let job = Job {
            job_id: uuid::Uuid::new_v4().to_string(),
            request: payload.clone().into(),
            tx_response,
        };

        tx_job.send(job).await.unwrap();
        let mut count = 0;
        while let Some(r) = rx_response.recv().await {
            count = count + 1;
            assert_eq!(r, Ok(bytes::Bytes::from(payload.clone())));
        }
        assert_eq!(n, count);
    }
}
