use crate::args::Args;
use crate::components::{output, shutdown};

use anyhow::Result;
use log::info;
use std::time::Duration;
use tokio::sync::watch;
use tokio::task::JoinSet;
use crate::components::client::client_builder::ClientBuilder;
use crate::components::client::header::HeadersConfig;

pub async fn run(args: Args) -> Result<()> {
    let Args {
        url,
        method,
        random,
        ip_files,
        ..
    } = args;
    
    let headers_config = HeadersConfig::from(args.header);
    
    let mut executor_builder = ClientBuilder::new()
        .url(url)
        .resolve_dns()
        .headers_config(headers_config.clone());

    if let Some(ip_files) = ip_files {
        executor_builder = executor_builder.random_ip_from_file(ip_files).await?;
    }
    let executor = executor_builder.build().await?;

    let mut handles = JoinSet::new();
    let timeout = Duration::from_secs(args.timeout);
    let time = Duration::from_secs(args.time);

    info!("Method is: {}", method);
    headers_config.log_detail();

    if args.test {
        let response = executor.single_request().await?;
        info!("Response status: {:?}", response.status());
        info!("Response is: {:?}", response.text().await?);
        return Ok(());
    }

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let request_ready = executor.request_ready(
        args.concurrent_count as u32, timeout, args.body, random
    );
    for request in request_ready {
        let handle = tokio::spawn(super::request::send_requests(request, shutdown_rx.clone()));
        handles.spawn(handle);
    }

    if args.normal_output {
        tokio::spawn(output::normal_output(method.clone(), shutdown_rx.clone()));
    } else {
        tokio::spawn(output::terminal_output(method.clone(), shutdown_rx.clone()));
    }

    tokio::spawn(shutdown::handle_shutdown_signals(shutdown_tx.clone()));
    wait_for_completion(time, shutdown_tx.clone(), shutdown_rx.clone()).await?;
    Ok(())
}

async fn wait_for_completion(
    time: Duration,
    shutdown_tx: watch::Sender<bool>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<()> {
    tokio::select! {
        _ = tokio::time::sleep(time) => {
            info!("Time limit reached");
        }
        Ok(_) = shutdown_rx.changed() => {
            info!("Received shutdown signal");
        }
    }
    shutdown_tx
        .send(true)
        .map_err(|e| anyhow::anyhow!("Failed to send shutdown signal: {}", e))?;
    Ok(())
}
