use std::sync::atomic::{AtomicU64, Ordering};
use std::{sync::Arc, time::Duration};
use tokio::sync::broadcast;

pub struct FullRequest {
    pub url: String,
    pub client: reqwest::Client,
    pub headers: reqwest::header::HeaderMap,
    pub method: String,
    pub has_header: bool,
}

pub async fn send_requests(
    full_request: FullRequest,
    mut shutdown_rx: broadcast::Receiver<()>,
    counter: Arc<AtomicU64>,
) {
    loop {
        let mut request_builder = full_request
            .client
            .request(full_request.method.parse().unwrap(), &full_request.url);
        if full_request.has_header {
            request_builder = request_builder.headers(full_request.headers.clone());
        }
        tokio::select! {
            biased;
            _ = shutdown_rx.recv() => {
                break;
            }
            result = request_builder.send() => {
                if let Ok(resp) = result {
                    let _ = resp.bytes().await;
                     counter.fetch_add(1, Ordering::Relaxed);
                }
                tokio::task::yield_now().await;
            }
        }
    }
}
