use core::panic;
use reqwest::cookie::Jar;
use reqwest::Client;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::net::lookup_host;
use url::Url;

use crate::parse_header::SpecialHeaders;

pub async fn build_client(
    parsed_url: &Url,
    ip: &Option<IpAddr>,
    special_headers: &SpecialHeaders,
) -> anyhow::Result<Client> {
    let domain = parsed_url.host_str().unwrap_or_default();
    println!("the domain is: {}", domain);

    let client_ip_addr = if ip.is_none() {
        let mut addrs = lookup_host((domain, 0)).await?;
        match addrs.next() {
            Some(socket_addr) => {
                let ip = socket_addr.ip();
                println!(
                    "For domain '{}', no specific IP provided. System DNS lookup suggests: {}.",
                    domain, ip
                );
                SocketAddr::new(ip, parsed_url.port_or_known_default().unwrap_or(0))
            }
            None => {
                eprintln!(
                    "Could not resolve IP for domain '{}' via system DNS lookup.",
                    domain
                );
                panic!("Please check your command or computer.");
            }
        }
    } else {
        println!(
            "For domain '{}', using the provided IP: {}.",
            domain,
            ip.unwrap()
        );
        format!(
            "{}:{}",
            ip.unwrap(),
            parsed_url.port_or_known_default().unwrap_or(0)
        )
        .parse::<SocketAddr>()?
    };

    let mut client = Client::builder()
        .resolve(domain, client_ip_addr)
        .use_native_tls();

    if let Some(user_agent) = &special_headers.user_agent {
        client = client.user_agent(user_agent);
    }
    if special_headers.gzip {
        client = client.gzip(true);
    } else {
        client = client.gzip(false);
    }

    if special_headers.deflate {
        client = client.deflate(true);
    } else {
        client = client.deflate(false);
    }
    if let Some(cookie) = &special_headers.cookie {
        let jar = Arc::new(Jar::default());
        jar.add_cookie_str(cookie, parsed_url);
        client = client.cookie_provider(jar);
    }
    Ok(client.build()?)
}
