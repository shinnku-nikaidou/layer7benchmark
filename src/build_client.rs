use core::panic;
use reqwest::Client;
use std::net::SocketAddr;
use tokio::net::lookup_host;
use url::Url;

pub async fn build_client(parsed_url: &Url, ip: &String) -> anyhow::Result<Client> {
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

    let client = Client::builder().resolve(domain, client_ip_addr).build()?;
    Ok(client)
}
