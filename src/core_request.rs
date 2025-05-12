use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct FullRequest {
    pub url: String,
    pub client: reqwest::Client,
    pub headers: reqwest::header::HeaderMap,
    pub method: String,
    pub has_header: bool,
}

pub async fn send_requests(
    fr: FullRequest,
    mut shutdown_rx: broadcast::Receiver<()>,
    counter: Arc<AtomicU64>,
) {
    loop {
        let mut request_builder = fr.client.request(fr.method.parse().unwrap(), &fr.url);
        if fr.has_header {
            request_builder = request_builder.headers(fr.headers.clone());
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
