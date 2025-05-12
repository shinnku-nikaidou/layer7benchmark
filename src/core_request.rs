use std::{cell::Cell, sync::atomic::Ordering};

use anyhow::Result;
use reqwest::{Client, Request};
use tokio::sync::watch;

use crate::COUNTER;

#[derive(Default)]
struct ThreadCounter {
    counter: Cell<u64>,
}

impl ThreadCounter {
    fn inc(&self) {
        // TODO: re-design publish strategy
        // self.counter.update(|c| c + 1);

        let current = self.counter.get();
        self.counter.set(current + 1);

        if (current + 1) % 100 == 0 {
            self.publish();
        }
    }

    fn publish(&self) {
        let value = self.counter.replace(0);
        COUNTER.fetch_add(value, Ordering::AcqRel);
    }
}

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
    thread_local! {
        static LOCAL_COUNTER: ThreadCounter = ThreadCounter::default();
    }

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

                LOCAL_COUNTER.with(|c| c.inc());
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
