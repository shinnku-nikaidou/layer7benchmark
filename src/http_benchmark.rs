use crate::args::Args;
use crate::core_request;
use crate::parse_header::HeadersConfig;
use crate::{build_client, shutdown, terminal};
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
        ..
    } = args;

    let mut handles = JoinSet::new();
    let timeout = Duration::from_secs(args.timeout);
    let time = Duration::from_secs(args.time);
    let headers_config: HeadersConfig = args.header.into();

    info!("Method is: {}", method);
    headers_config.log_detail();

    let url_t =
        reqwest::Url::parse(&url).map_err(|e| anyhow::anyhow!("Failed to parse URL: {}", e))?;

    let client = build_client::build_client(&url_t, &args.ip, &headers_config).await?;
    let headers = headers_config.other_headers;

    if args.test {
        test_request(client, url_t, method, headers).await?;
        return Ok(());
    }

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let req = core_request::FullRequest {
        url,
        client,
        headers,
        method: method.clone(),
        timeout,
        random,
        body: (!args.body.is_empty()).then(|| args.body.clone()),
    };

    for _ in 0..args.concurrent_count {
        let handle = tokio::spawn(core_request::send_requests(
            req.clone(),
            shutdown_rx.clone(),
        ));
        handles.spawn(handle);
    }

    if args.normal_output {
        tokio::spawn(terminal::normal_output(method.clone(), shutdown_rx.clone()));
    } else {
        tokio::spawn(terminal::terminal_output(
            method.clone(),
            shutdown_rx.clone(),
        ));
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
