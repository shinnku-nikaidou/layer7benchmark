use anyhow::{Context, Result};
use log::info;
use reqwest::{cookie::Jar, Client};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::net::lookup_host;
use url::Url;

use crate::parse_header::SpecialHeaders;

pub async fn build_client(
    parsed_url: &Url,
    ip: &Option<IpAddr>,
    special_headers: &SpecialHeaders,
) -> Result<Client> {
    let domain = parsed_url
        .host_str()
        .context("URL is missing host component")?;
    info!("Domain: {}", domain);

    let socket_addr = resolve_socket_addr(parsed_url, ip, domain).await?;

    let client_builder = Client::builder()
        .resolve(domain, socket_addr)
        .use_native_tls()
        .gzip(special_headers.gzip)
        .deflate(special_headers.deflate);

    let client_builder = if let Some(user_agent) = &special_headers.user_agent {
        client_builder.user_agent(user_agent)
    } else {
        client_builder
    };

    let client_builder = if let Some(cookie) = &special_headers.cookie {
        let jar = Arc::new(Jar::default());
        jar.add_cookie_str(cookie, parsed_url);
        client_builder.cookie_provider(jar)
    } else {
        client_builder
    };

    client_builder
        .build()
        .context("Failed to build HTTP client")
}

async fn resolve_socket_addr(
    parsed_url: &Url,
    ip: &Option<IpAddr>,
    domain: &str,
) -> Result<SocketAddr> {
    let port = parsed_url.port_or_known_default().unwrap_or(0);

    match ip {
        Some(specified_ip) => {
            info!("Using provided IP {} for domain '{}'", specified_ip, domain);
            Ok(SocketAddr::new(*specified_ip, port))
        }
        None => {
            let socket_addr = lookup_host((domain, 0))
                .await
                .context("DNS lookup failed")?
                .next()
                .context("No IP addresses found for domain")?;

            info!("Resolved domain '{}' to IP: {}", domain, socket_addr.ip());
            Ok(SocketAddr::new(socket_addr.ip(), port))
        }
    }
}
