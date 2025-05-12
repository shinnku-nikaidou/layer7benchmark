use std::sync::atomic::Ordering;

use anyhow::Result;
use reqwest::{Client, Request};
use tokio::sync::watch;

use crate::COUNTER;

pub struct FullRequest {
    client: Client,
    req: Request,
    shutdown: watch::Receiver<bool>,
}

impl FullRequest {
    pub fn new(client: Client, req: Request, shutdown: watch::Receiver<bool>) -> Self {
        Self {
            client,
            req,
            shutdown,
        }
    }
}

pub async fn send_requests(mut req: FullRequest) -> Result<()> {
    loop {
        let request = req
            .req
            .try_clone()
            .expect("The request can not be cloned, maybe the body is a stream");

        tokio::select! {
            response = req.client.execute(request) => {
                let Ok(response) = response else {
                    continue;
                };

                if response.bytes().await.is_err() {
                    continue;
                }

                COUNTER.fetch_add(1, Ordering::AcqRel);
            }

            _ = req.shutdown.changed() => {
                let shutdown = req.shutdown.borrow_and_update();

                if *shutdown {
                    break;
                }
            }
        }
    }

    Ok(())
}
