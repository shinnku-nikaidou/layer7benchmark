mod args;
mod build_client;
mod core_request;
mod parse_header;
mod statistic;
mod terminal;

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use log::{error, info};
use std::time::Duration;
use tokio::{runtime::Runtime, sync::watch};
use url::Url;

async fn run(args: Args) -> Result<()> {
    let mut handles = Vec::new();
    let url = args.url.clone();
    let method = args.method;
    let parsed_url = Url::parse(&args.url).context("Failed to parse URL")?;
    let timeout = Duration::from_secs(args.timeout);
    let (headers, special_headers) = parse_header::parse_header(args.header.clone())?;

    info!("Method is: {}", method);
    info!("Headers is: {:?}", headers);
    info!("enabled gzip: {:?}", special_headers.gzip);
    info!("enabled deflate: {:?}", special_headers.deflate);
    info!("cookie is: {:?}", special_headers.cookie);
    info!("user agent is: {:?}", special_headers.user_agent);

    let client = build_client::build_client(&parsed_url, &args.ip, &special_headers).await?;

    if args.test {
        info!("Test mode enabled. Only send one single request.");
        let request_builder = client.request(method, &url);
        let response = request_builder.headers(headers).send().await?;
        info!("Response status: {:?}", response.status());
        info!("Response is: {:?}", response.text().await?);
        return Ok(());
    }

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    for _ in 0..args.concurrent_count {
        let full_request = core_request::FullRequest {
            url: url.clone(),
            client: client.clone(),
            headers: headers.clone(),
            method: method.clone(),
            timeout: timeout.clone(),
        };

        let handle = tokio::spawn(core_request::send_requests(
            full_request,
            shutdown_rx.clone(),
        ));
        handles.push(handle);
    }

    let _terminal_handle = tokio::spawn(terminal::terminal_output(method.clone()));

    tokio::time::sleep(Duration::from_secs(args.time)).await;
    let _ = shutdown_tx.send(true);

    for handle in handles {
        if let Err(e) = handle.await {
            error!("Task exited with error: {:?}", e);
        }
    }

    Ok(())
}

fn main() {
    let args = Args::parse();
    let _ = statistic::STATISTIC.set(statistic::Statistic::default());
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    info!("l7_flood started");
    let runtime = Runtime::new().expect("Could not build the tokio runtime");

    if let Err(error) = runtime.block_on(run(args)) {
        error!("Exited with error: {}", error);
    } else {
        info!("Finished.");
    }
}
