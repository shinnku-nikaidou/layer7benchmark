use anyhow::Result;
use log::{debug, info};
use rand::Rng;
use reqwest::{cookie::Jar, Client};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::net::lookup_host;
use url::Url;

use crate::parse_header::HeadersConfig;

#[derive(thiserror::Error, Debug)]
pub enum ClientBuildError {
    #[error("URL is missing host component")]
    URLMissingHost,

    #[error("Failed to build HTTP client: \n{0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Failed to resolve IP address for domain {0}")]
    DNSLookupFailed(String),

    #[error("No IP addresses found for domain {0}")]
    NoIpAddressesFound(String),
}

pub fn generate_ip_list(ip_lists: &String) -> Result<Vec<std::net::IpAddr>> {
    let mut ip_list = Vec::new();
    let file_text = std::fs::read_to_string(ip_lists)
        .map_err(|e| anyhow::anyhow!("Failed to read IP list file: {}", e))?;
    for line in file_text.lines() {
        let ip = line.trim();
        if !ip.is_empty() {
            let ip_addr = ip
                .parse::<std::net::IpAddr>()
                .map_err(|e| anyhow::anyhow!("Failed to parse IP address: {}", e))?;
            info!("Get new IP address: {}", ip_addr);
            ip_list.push(ip_addr);
        }
    }
    Ok(ip_list)
}

pub async fn generate_clients(
    url_t: &reqwest::Url,
    ip: &Option<std::net::IpAddr>,
    ip_lists: &Option<Vec<std::net::IpAddr>>,
    headers_config: &HeadersConfig,
) -> Result<Vec<Client>, ClientBuildError> {
    let mut clients = Vec::new();
    if ip_lists.is_some() {
        for ip in ip_lists.clone().unwrap() {
            debug!("Build client with IP address: {}", ip);
            clients.push(build_client(url_t, &Some(ip), headers_config).await?);
        }
    } else {
        clients.push(build_client(url_t, ip, headers_config).await?);
    }
    Ok(clients)
}

pub async fn generate_client(clients: &Vec<Client>) -> Result<Client, ClientBuildError> {
    let random_index = rand::rng().random_range(0..clients.len());
    Ok(clients[random_index].clone())
}

pub async fn build_client(
    parsed_url: &Url,
    ip: &Option<IpAddr>,
    headers_config: &HeadersConfig,
) -> Result<Client, ClientBuildError> {
    let domain = parsed_url
        .host_str()
        .ok_or(ClientBuildError::URLMissingHost)?;
    debug!("Domain: {}", domain);

    let socket_addr = resolve_socket_addr(parsed_url, ip, domain).await?;

    let client_builder = Client::builder()
        .resolve(domain, socket_addr)
        .use_native_tls()
        .gzip(headers_config.gzip)
        .deflate(headers_config.deflate);

    let client_builder = if let Some(user_agent) = &headers_config.user_agent {
        client_builder.user_agent(user_agent)
    } else {
        client_builder
    };

    let client_builder = if let Some(cookie) = &headers_config.cookie {
        let jar = Arc::new(Jar::default());
        jar.add_cookie_str(cookie, parsed_url);
        client_builder.cookie_provider(jar)
    } else {
        client_builder
    };

    Ok(client_builder.build()?)
}

async fn resolve_socket_addr(
    parsed_url: &Url,
    ip: &Option<IpAddr>,
    domain: &str,
) -> Result<SocketAddr, ClientBuildError> {
    let port = parsed_url.port_or_known_default().unwrap_or(0);

    match ip {
        Some(specified_ip) => {
            info!("Using provided IP {} for domain '{}'", specified_ip, domain);
            Ok(SocketAddr::new(*specified_ip, port))
        }
        None => {
            let socket_addr = lookup_host((domain, 0))
                .await
                .map_err(|e| ClientBuildError::DNSLookupFailed(e.to_string()))?
                .next()
                .ok_or(ClientBuildError::NoIpAddressesFound(domain.to_string()))?;

            info!("Resolved domain '{}' to IP: {}", domain, socket_addr.ip());
            Ok(SocketAddr::new(socket_addr.ip(), port))
        }
    }
}
