use crate::args::Args;
use crate::components::{client, header::HeadersConfig, output, request, shutdown};

use anyhow::Result;
use log::info;
use std::time::Duration;
use tokio::sync::watch;
use tokio::task::JoinSet;

pub async fn run(args: Args) -> Result<()> {
    let Args {
        url,
        method,
        random,
        ip_files,
        ..
    } = args;

    let ip_lists = if !ip_files.is_empty() {
        Some(client::read_ip_files(&ip_files)?)
    } else {
        None
    };

    let mut handles = JoinSet::new();
    let timeout = Duration::from_secs(args.timeout);
    let time = Duration::from_secs(args.time);
    let headers_config: HeadersConfig = args.header.into();

    info!("Method is: {}", method);
    headers_config.log_detail();

    // url_t is only for getting the ip address from the host, not for sending the request.
    let url_t =
        reqwest::Url::parse(&url).map_err(|e| anyhow::anyhow!("Failed to parse URL: {}", e))?;

    let headers = headers_config.other_headers.clone();

    // Why need clients, why not client
    // The client will pre compile DNS result into itself so that no more redundant ip lookup.
    // And the user may provide multiple ip through --ip-files, it needs multiple clients.
    // if the user don't provide --ip-files, then `assert_eq!(clients.len(), 1)` will hold.
    let clients: Vec<reqwest::Client> =
        client::build(&url_t, &args.ip, &ip_lists, &headers_config).await?;

    if args.test {
        test_request(
            client::generate_client(&clients).await?,
            url_t,
            method,
            headers,
        )
        .await?;
        return Ok(());
    }

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    for _ in 0..args.concurrent_count {
        let req = request::FullRequest {
            url: url.clone(),
            client: client::generate_client(&clients).await?,
            headers: headers.clone(),
            method: method.clone(),
            timeout,
            random,
            body: (!args.body.is_empty()).then(|| args.body.clone()),
        };
        let handle = tokio::spawn(request::send_requests(req, shutdown_rx.clone()));
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

async fn test_request(
    client: reqwest::Client,
    url: reqwest::Url,
    method: reqwest::Method,
    headers: reqwest::header::HeaderMap,
) -> Result<()> {
    info!("Test mode enabled, sending a single request...");
    let request_builder = client.request(method, url);
    let response = request_builder.headers(headers).send().await?;
    info!("Response status: {:?}", response.status());
    info!("Response is: {:?}", response.text().await?);
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
