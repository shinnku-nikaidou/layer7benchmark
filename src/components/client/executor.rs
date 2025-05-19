use std::net::{SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use log::info;
use rand::seq::IndexedRandom;
use reqwest::{Client, Method, Response};
use crate::components::client::client_builder::{ClientBuildError, ClientBuilder};
use crate::components::client::request::FullRequest;

pub struct BenchmarkExecutor {
    reqwest_clients: Box<[Client]>,
    method: Method,
    headers: reqwest::header::HeaderMap,
    url: url::Url,
}

impl BenchmarkExecutor {
    pub async fn from_builder(builder: ClientBuilder) -> Result<Self, ClientBuildError> {
        let ClientBuilder {
            url,
            ip_mode,
            headers_config,
            method
        } = builder;
        let url = url.ok_or(ClientBuildError::UrlIsRequired)?;
        let domain = url.host_str().ok_or(ClientBuildError::URLMissingHost)?;
        let ip = ip_mode.resolve(domain).await?;
        let port = url.port_or_known_default().unwrap_or(443);
        
        let socket_addrs: Box<[_]> = ip
            .into_iter()
            .map(|ip| SocketAddr::new(ip, port))
            .collect();
        let cookie_jar = headers_config.get_cookie_jar(&url).map(Arc::new);
        let reqwest_clients = socket_addrs
            .into_iter()
            .map(|socket_addr| {
                Client::builder()
                    .resolve(domain, socket_addr)
                    .use_native_tls()
            })
            .map(|client_builder| {
                headers_config.set_compress_header_for_client(client_builder)
            })
            .map(|client_builder| {
                headers_config.set_user_agent_for_client(client_builder)
            });
        let reqwest_clients: Box<[_]> = if let Some(cookie_jar) = cookie_jar {
            reqwest_clients
                .map(|client_builder| client_builder
                    .cookie_provider(cookie_jar.clone())
                    .cookie_store(true)
                )
                .filter_map(|client_builder| client_builder.build().ok())
                .collect()
        } else {
            reqwest_clients
                .filter_map(|client_builder| client_builder.build().ok())
                .collect()
        };
        
        let headers = headers_config.other_headers;
        
        Ok(Self {
            reqwest_clients,
            method,
            headers,
            url,
        })
    }
    
    pub async fn single_request(
        &self
    ) -> Result<Response, reqwest::Error> {
        info!("Test mode enabled, sending a single request...");
        let client = self
            .reqwest_clients[0]
            .request(self.method.clone(), self.url.clone())
            .headers(self.headers.clone())
            .send()
            .await?;
        Ok(client)
    }

    pub fn request_ready(
        &self,
        concurrent_count: u32,
        timeout: Duration,
        body: Option<String>,
        random: bool,
    ) -> Box<[FullRequest]> {
        let mut rng = rand::rng();
        (0..concurrent_count)
            .filter_map(|concurrent_count| {
                self.reqwest_clients.choose(&mut rng)
            })
            .map(|client| {
                FullRequest {
                    url: self.url.to_string(),
                    client: client.clone(),
                    headers: self.headers.clone(),
                    method: self.method.clone(),
                    timeout,
                    body: body.clone(),
                    random,
                }
            })
            .collect()
    }
}