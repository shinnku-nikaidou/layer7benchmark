use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::watch;

pub struct FullRequest {
    pub url: String,
    pub client: reqwest::Client,
    pub headers: reqwest::header::HeaderMap,
    pub method: reqwest::Method,
}

pub async fn send_requests(
    req: FullRequest,
    counter: Arc<AtomicU64>,
    mut shutdown: watch::Receiver<bool>,
) {
    loop {
        let request_builder = req
            .client
            .request(req.method.clone(), &req.url)
            .headers(req.headers.clone());

        tokio::select! {
            biased;
            result = request_builder.send() => {
                if let Ok(resp) = result {
                    let _ = resp.bytes().await;
                     counter.fetch_add(1, Ordering::Relaxed);
                }
                tokio::task::yield_now().await;
            }

            _ = shutdown.changed() => {
                let shutdown = shutdown.borrow_and_update();

                if *shutdown {
                    break;
                }
            }
        }
    }
}
