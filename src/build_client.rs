use core::panic;
use reqwest::cookie::Jar;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Client;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::lookup_host;
use url::Url;

#[derive(Debug, Clone, Default)]
pub struct SpecialHeaders {
    pub user_agent: Option<String>,
    pub gzip: bool,
    pub deflate: bool,
    pub cookie: Option<String>,
}

pub fn parse_header(headers: Vec<String>) -> anyhow::Result<(HeaderMap, SpecialHeaders)> {
    let mut header_map = HeaderMap::new();
    let mut special_headers = SpecialHeaders::default();

    for header in headers {
        if let Some(pos) = header.find(':') {
            let (name, value) = header.split_at(pos);
            let name = name.trim().to_lowercase();
            let value = value[1..].trim();

            match name.as_str() {
                "user-agent" => {
                    special_headers.user_agent = Some(value.to_string());
                }
                "accept-encoding" => {
                    special_headers.gzip = value.contains("gzip");
                    special_headers.deflate = value.contains("deflate");
                }
                "cookie" => {
                    special_headers.cookie = Some(value.to_string());
                }
                _ => {
                    if let Ok(header_name) = HeaderName::from_str(&name) {
                        if let Ok(header_value) = HeaderValue::from_str(value) {
                            header_map.insert(header_name, header_value);
                        }
                    }
                }
            }
        } else {
            eprintln!(
                "Invalid header format: '{}'. Expected 'Name: Value'.",
                header
            );
            return Err(anyhow::anyhow!("Invalid header format"));
        }
    }
    Ok((header_map, special_headers))
}

pub async fn build_client(
    parsed_url: &Url,
    ip: &String,
    special_headers: &SpecialHeaders,
) -> anyhow::Result<Client> {
    let domain = parsed_url.host_str().unwrap_or_default();
    println!("the domain is: {}", domain);

    let client_ip_addr = if ip.is_empty() {
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
        println!("For domain '{}', using the provided IP: {}.", domain, ip);
        format!("{}:{}", ip, parsed_url.port_or_known_default().unwrap_or(0))
            .parse::<SocketAddr>()?
    };

    let mut client = Client::builder().resolve(domain, client_ip_addr);

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
