mod args;
mod build_client;
mod core_request;
mod parse_header;
mod terminal;
use core::panic;
use std::{sync::Arc, time::Duration};

use args::Args;
use clap::Parser;
use std::sync::atomic::AtomicU64;
use tokio::sync::broadcast;
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut handles = Vec::new();
    let url = args.url.clone();
    let method = args.method;
    let parsed_url = Url::parse(&args.url)?;
    let request_counter = Arc::new(AtomicU64::new(0));

    if method != "GET"
        && method != "POST"
        && method != "PUT"
        && method != "DELETE"
        && method != "OPTIONS"
    {
        panic!("Method must be GET or POST or PUT or DELETE or OPTIONS");
    } else {
        println!("Method is: {}", method);
    }

    let mut have_header = false;
    let headers = if args.header.len() > 0 {
        have_header = true;
        parse_header::parse_header(args.header)?
    } else {
        reqwest::header::HeaderMap::new()
    };

    println!("Headers is: {:?}", headers);

    let client = build_client::build_client(&parsed_url, &args.ip).await?;

    if args.test {
        println!("Test mode enabled. Only send one single request.");
        let request_builder = client.request(method.parse().unwrap(), &url);
        let response = request_builder.headers(headers).send().await?;
        println!("Response status: {:?}", response.status());
        println!("Response is: {:?}", response.text().await?);
        return Ok(());
    }

    let (shutdown_tx, _) = broadcast::channel(1);

    for _ in 0..args.concurrent_count {
        let full_request = core_request::FullRequest {
            url: url.clone(),
            client: client.clone(),
            headers: headers.clone(),
            method: method.clone(),
            has_header: have_header,
        };
        let shutdown_rx = shutdown_tx.subscribe();
        let counter_clone = Arc::clone(&request_counter);

        let handle = tokio::spawn(core_request::send_requests(
            full_request,
            shutdown_rx,
            counter_clone,
        ));
        handles.push(handle);
    }

    let _terminal_handle = tokio::spawn(terminal::terminal_output(
        Arc::clone(&request_counter),
        method.clone(),
    ));

    tokio::time::sleep(Duration::from_secs(args.time)).await;
    let _ = shutdown_tx.send(());

    for handle in handles {
        if let Err(e) = handle.await {
            eprintln!("Task exited with error: {:?}", e);
        }
    }

    Ok(())
}
