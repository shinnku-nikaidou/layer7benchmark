use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::watch;

pub struct FullRequest {
    pub url: String,
    pub client: reqwest::Client,
    pub headers: reqwest::header::HeaderMap,
    pub method: String,
    pub shutdown: watch::Receiver<bool>,
}

pub async fn send_requests(
    mut fr: FullRequest,
    counter: Arc<AtomicU64>,
) {
    loop {
        let request_builder = fr
            .client
            .request(fr.method.parse().unwrap(), &fr.url)
            .headers(fr.headers.clone());

        tokio::select! {
            biased;
            result = request_builder.send() => {
                if let Ok(resp) = result {
                    let _ = resp.bytes().await;
                     counter.fetch_add(1, Ordering::Relaxed);
                }
                tokio::task::yield_now().await;
            }

            _ = fr.shutdown.changed() => {
                let shutdown = fr.shutdown.borrow_and_update();

                if *shutdown {
                    break;
                }
            }
        }
    }
}
