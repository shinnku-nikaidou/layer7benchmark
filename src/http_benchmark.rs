use crate::core_request;
use crate::args::Args;
use crate::parse_header::HeadersConfig;
use crate::{build_client, shutdown, terminal};
use anyhow::Result;
use log::info;
use std::time::Duration;
use tokio::sync::watch;
use tokio::task::JoinSet;

pub async fn run(args: Args) -> Result<()> {
    let Args {
        concurrent_count,
        url,
        time,
        ip,
        header,
        body,
        method,
        test,
        timeout,
    } = args;
    let mut handles = JoinSet::new();
    let timeout = Duration::from_secs(timeout);
    let headers_config: HeadersConfig = header.into();

    info!("Method is: {}", method);
    headers_config.log_detail();

    let client = build_client::build_client(&url, &ip, &headers_config).await?;
    let headers = headers_config.other_headers;

    if test {
        info!("Test mode enabled. Only send one single request.");
        let request_builder = client.request(method, url);
        let response = request_builder.headers(headers).send().await?;
        info!("Response status: {:?}", response.status());
        info!("Response is: {:?}", response.text().await?);
        return Ok(());
    }

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let full_request = core_request::FullRequest {
        url: url.clone(),
        client: client.clone(),
        headers: headers.clone(),
        method: method.clone(),
        timeout: timeout.clone(),
        body: if body.is_empty() {
            None
        } else {
            Some(body.clone())
        },
    };

    for _ in 0..concurrent_count {
        let handle = tokio::spawn(core_request::send_requests(
            full_request.clone(),
            shutdown_rx.clone(),
        ));
        handles.spawn(handle);
    }

    tokio::spawn(terminal::terminal_output(
        method.clone(),
        shutdown_rx.clone(),
    ));

    tokio::spawn(shutdown::handle_shutdown_signals(shutdown_tx.clone()));

    let mut sr_clone = shutdown_rx.clone();
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(time)) => {
            info!("Time limit reached");
        }
        Ok(_) = sr_clone.changed() => {
            info!("Received shutdown signal");
        }
    }
    shutdown_tx
        .send(true)
        .map_err(|e| anyhow::anyhow!("Failed to send shutdown signal: {}", e))?;
    Ok(())
}
