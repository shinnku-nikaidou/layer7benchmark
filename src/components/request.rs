use anyhow::{Context, Result};
use futures_util::StreamExt;
use log::debug;
use std::sync::atomic::Ordering;
use tokio::{sync::watch, time::timeout};

use crate::statistic::STATISTIC;
use std::time::Duration;
use url::Url;

#[derive(Debug, Clone)]
pub struct FullRequest {
    pub url: String,
    pub client: reqwest::Client,
    pub headers: reqwest::header::HeaderMap,
    pub method: reqwest::Method,
    pub timeout: Duration,
    pub body: Option<String>,
    pub random: bool,
}

impl FullRequest {
    fn get_url(&self, generator: &Option<impl Fn() -> String>) -> Result<Url> {
        if self.random {
            let generator = generator
                .as_ref()
                .context("Random generator not initialized")?;
            let random_url = generator();
            debug!("Random URL generated: {}", random_url);
            Url::parse(&random_url).context("Failed to parse random URL")
        } else {
            Url::parse(&self.url).context("Failed to parse URL")
        }
    }

    fn build_request(
        &self,
        generator: &Option<impl Fn() -> String>,
    ) -> Result<reqwest::RequestBuilder> {
        let url = self.get_url(generator)?;
        let request = self
            .client
            .request(self.method.clone(), url)
            .headers(self.headers.clone())
            .timeout(self.timeout);

        if let Some(ref body) = self.body {
            Ok(request.body(body.clone()))
        } else {
            Ok(request)
        }
    }
}

#[inline]
fn update_status_counter(status: u16, sc: &crate::statistic::StatusCounter) {
    match status {
        200..=299 => sc.status_2xx.fetch_add(1, Ordering::Relaxed),
        300..=399 => sc.status_3xx.fetch_add(1, Ordering::Relaxed),
        400..=499 => sc.status_4xx.fetch_add(1, Ordering::Relaxed),
        500..=599 => sc.status_5xx.fetch_add(1, Ordering::Relaxed),
        _ => sc.status_other.fetch_add(1, Ordering::Relaxed),
    };
}

pub async fn send_requests(req: FullRequest, mut shutdown: watch::Receiver<bool>) {
    let s = STATISTIC.get().unwrap();
    let counter = &s.request_counter;
    let sc = &s.status_counter;
    let network_traffics = &s.network_traffics;
    let generator = req.random.then({
        let template = req.url.to_string();
        move || crate::components::randomization::make_template_generator(&template)
    });

    loop {
        let request_builder = req.build_request(&generator).unwrap();

        tokio::select! {
            biased;

            result = request_builder.send() => {
                if let Ok(resp) = result {
                    let status = resp.status().as_u16();

                    let stream_byte = process_response(resp, sc).await;
                    counter.fetch_add(1, Ordering::Relaxed);
                    network_traffics.fetch_add(stream_byte, Ordering::Relaxed);

                    update_status_counter(status, sc);
                } else {
                    sc.status_other.fetch_add(1, Ordering::Relaxed);
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

async fn process_response(resp: reqwest::Response, sc: &crate::statistic::StatusCounter) -> u64 {
    let mut bytes = 0;
    match timeout(Duration::from_secs(60), async {
        let mut stream = resp.bytes_stream();
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
    })
    .await
    {
        Ok(bytes) => bytes,
        Err(_) => {
            sc.status_other.fetch_add(1, Ordering::Relaxed);
            bytes
        }
    }
}
