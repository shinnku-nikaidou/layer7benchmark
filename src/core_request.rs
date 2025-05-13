use std::sync::atomic::Ordering;
use tokio::sync::watch;

use crate::statistic::STATISTIC;

pub struct FullRequest {
    pub url: String,
    pub client: reqwest::Client,
    pub headers: reqwest::header::HeaderMap,
    pub method: reqwest::Method,
}

pub async fn send_requests(req: FullRequest, mut shutdown: watch::Receiver<bool>) {
    let s = STATISTIC.get().unwrap().clone();
    let counter = &s.request_counter.clone();
    let sc = &s.status_counter.clone();

    loop {
        let request_builder = req
            .client
            .request(req.method.clone(), &req.url)
            .headers(req.headers.clone());

        tokio::select! {
            biased;
            result = request_builder.send() => {
                if let Ok(resp) = result {
                    let status = resp.status().as_u16();
                    let _ = resp.bytes().await;

                     counter.fetch_add(1, Ordering::Relaxed);

                     match status {
                        200..=299 => sc.status_2xx.fetch_add(1, Ordering::Relaxed),
                        300..=399 => sc.status_3xx.fetch_add(1, Ordering::Relaxed),
                        400..=499 => sc.status_4xx.fetch_add(1, Ordering::Relaxed),
                        500..=599 => sc.status_5xx.fetch_add(1, Ordering::Relaxed),
                        _ => sc.status_other.fetch_add(1, Ordering::Relaxed),
                    };
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
