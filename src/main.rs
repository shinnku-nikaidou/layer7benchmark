mod args;
mod build_client;
mod core_request;
mod parse_header;
mod statistic;
mod terminal;

use anyhow::Result;
use args::Args;
use clap::Parser;
use log::{debug, error, info};
use std::time::Duration;
use tokio::{runtime::Runtime, sync::watch};
use tokio::task::JoinSet;
use tokio::signal;
use crate::parse_header::HeadersConfig;

async fn run(args: Args) -> Result<()> {
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

    for _ in 0..concurrent_count {
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
        handles.spawn(handle);
    }

    tokio::spawn(terminal::terminal_output(method.clone(), shutdown_rx.clone()));

    tokio::time::sleep(Duration::from_secs(time)).await;
    let _ = shutdown_tx.send(true);

    handle_shutdown_signals(shutdown_tx).await;
    Ok(())
}

async fn handle_shutdown_signals(shutdown_tx: watch::Sender<bool>)  {
    let ctrl_c = async {
        #[allow(clippy::expect_used)]
        signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        #[allow(clippy::expect_used)]
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install terminate signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    #[cfg(windows)]
    #[allow(clippy::expect_used)]
    let ctrl_shutdown = async {
        signal::windows::ctrl_shutdown()
            .expect("Failed to install Ctrl+Shutdown handler")
            .recv()
            .await;
    };

    #[cfg(not(windows))]
    let ctrl_shutdown = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            debug!("Received Ctrl+C signal");
        }
        _ = terminate => {
            debug!("Received terminate signal");
        }
        _ = ctrl_shutdown => {
            debug!("Received shutdown signal");
        }
    }

    // Send shutdown signal to all tasks
    let _ = shutdown_tx.send(true);

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
