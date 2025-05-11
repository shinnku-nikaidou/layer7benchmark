mod args;
use core::panic;
use std::time::Duration;

use args::Args;
use clap::Parser;
use reqwest::Client;
use std::net::SocketAddr;
use tokio::net::lookup_host;
use tokio::sync::broadcast;
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut handles = Vec::new();
    let url = args.url.clone();
    let parsed_url = Url::parse(&args.url)?;
    let domain = parsed_url.host_str().unwrap_or_default();

    println!("the download url is: {}", url);
    println!("the domain is: {}", domain);
    let mut addrs = lookup_host((domain, 0)).await?;
    let ip;

    match addrs.next() {
        Some(socket_addr) => {
            ip = socket_addr.ip();
            println!(
                "For domain '{}', no specific IP provided. System DNS lookup suggests: {}.",
                domain, ip
            );
        }
        None => {
            eprintln!(
                "Could not resolve IP for domain '{}' via system DNS lookup.",
                domain
            );
            panic!("Please check your command or computer.");
        }
    }

    let client = if args.ip.is_empty() {
        // Client::new()
        Client::builder()
            .resolve(domain, SocketAddr::new(ip, 0))
            .build()?
    } else {
        Client::builder()
            .resolve(domain, format!("{}:0", args.ip).parse::<SocketAddr>()?)
            .build()?
    };
    let (shutdown_tx, _) = broadcast::channel(1);

    for _ in 0..args.concurrent_count {
        let url = url.clone();
        let client = client.clone();
        let mut shutdown_rx = shutdown_tx.subscribe();

        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                    result = client.get(&url).send() => {
                        if let Ok(resp) = result {
                            let _ = resp.bytes().await;
                        }
                        tokio::task::yield_now().await;
                    }
                }
            }
        });
        handles.push(handle);
    }

    tokio::time::sleep(Duration::from_secs(args.time)).await;
    let _ = shutdown_tx.send(());

    for handle in handles {
        if let Err(e) = handle.await {
            eprintln!("Task exited with error: {:?}", e);
        }
    }

    Ok(())
}
