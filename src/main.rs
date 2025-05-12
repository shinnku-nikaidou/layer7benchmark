mod args;
mod build_client;
mod parse_header;
use core::panic;
use std::time::Duration;

use args::Args;
use clap::Parser;
use tokio::sync::broadcast;
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut handles = Vec::new();
    let url = args.url.clone();
    let method = args.method;
    let parsed_url = Url::parse(&args.url)?;

    if method != "GET" && method != "POST" && method != "PUT" && method != "DELETE" {
        panic!("Method must be GET or POST or PUT or DELETE");
    } else {
        println!("Method is: {}", method);
    }

    println!("Headers is: {:?}", args.header);

    let mut have_header = false;
    let headers = if args.header.len() > 0 {
        have_header = true;
        parse_header::parse_header(args.header)?
    } else {
        reqwest::header::HeaderMap::new()
    };

    let client = build_client::build_client(&parsed_url, &args.ip).await?;
    let (shutdown_tx, _) = broadcast::channel(1);

    for _ in 0..args.concurrent_count {
        let url = url.clone();
        let client = client.clone();
        let mut shutdown_rx = shutdown_tx.subscribe();
        let method = method.clone();
        let headers = headers.clone();

        let handle = tokio::spawn(async move {
            loop {
                let mut request_builder = client.request(method.parse().unwrap(), &url);
                if have_header {
                    request_builder = request_builder.headers(headers.clone());
                }
                tokio::select! {
                    biased;
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                    result = request_builder.send() => {
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
