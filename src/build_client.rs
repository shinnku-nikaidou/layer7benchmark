use log::info;
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

pub async fn build_client(
    parsed_url: &Url,
    ip: &Option<IpAddr>,
    ip_lists: &Option<Vec<IpAddr>>,
    headers_config: &HeadersConfig,
) -> Result<Client, ClientBuildError> {
    let domain = parsed_url
        .host_str()
        .ok_or(ClientBuildError::URLMissingHost)?;
    info!("Domain: {}", domain);

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
