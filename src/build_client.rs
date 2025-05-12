use std::net::{IpAddr, SocketAddr};

use anyhow::{anyhow, Result};
use hickory_resolver::Resolver;
use reqwest::Client;
use url::Url;

pub async fn build_client(url: &Url, ip: Option<IpAddr>) -> Result<Client> {
    let domain = url
        .host_str()
        .ok_or(anyhow!("The URL does not have a host"))?;

    let ip = match ip {
        Some(ip) => ip,
        None => {
            let resolver = Resolver::builder_tokio()?.build();
            let response = resolver.lookup_ip(domain).await?;
            let ip = response
                .iter()
                .next()
                .ok_or(anyhow!("Could not resolve IP for domain `{domain}`"))?;
            println!("For domain `{domain}`, no specific IP provided. DNS lookup suggests: {ip}.");
            ip
        }
    };

    let port = url.port_or_known_default().unwrap_or(0);

    Ok(Client::builder()
        .resolve(domain, SocketAddr::new(ip, port))
        .build()?)
}
