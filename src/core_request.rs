use futures_util::StreamExt;
use std::sync::atomic::Ordering;
use tokio::{sync::watch, time::timeout};

use crate::statistic::STATISTIC;
use std::time::Duration;

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
    let network_traffics = &s.network_traffics.clone();

    loop {
        let request_builder = req
            .client
            .request(req.method.clone(), &req.url)
            .headers(req.headers.clone())
            .timeout(Duration::from_secs(10));

        let mut stream_byte = 0;

        tokio::select! {
            result = request_builder.send() => {
                if let Ok(resp) = result {
                    let status = resp.status().as_u16();

                    match timeout(Duration::from_secs(10), async {
                        let mut stream = resp.bytes_stream();
                        let mut bytes = 0;

                        while let Some(chunk_res) = stream.next().await {
                            match chunk_res {
                                Ok(chunk) => {
                                    bytes += chunk.len() as u64;
                                }
                                Err(_) => {
                                    break;
                                }
                            }
                        }
                        bytes
                    }).await {
                        Ok(bytes) => {
                            stream_byte = bytes;
                        }
                        Err(_) => {
                        }
                    }
                     counter.fetch_add(1, Ordering::Relaxed);
                     network_traffics.fetch_add(stream_byte, Ordering::Relaxed);

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
