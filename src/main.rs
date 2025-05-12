mod args;
mod build_client;
mod core_request;
mod terminal;

use std::{sync::atomic::AtomicU64, time::Duration};

use anyhow::Result;
use clap::Parser;
use tokio::{runtime::Runtime, sync::watch};

use args::Args;
use build_client::build_client;
use core_request::{send_requests, FullRequest};
use terminal::terminal_output;

static COUNTER: AtomicU64 = AtomicU64::new(0);

async fn run(args: Args) -> Result<()> {
    let client = build_client(&args.url, args.ip).await?;

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let headers = args.header.into_iter().collect();

    let request = client
        .request(args.method.clone(), args.url)
        .headers(headers)
        .build()?;

    let handles: Vec<_> = (0..args.concurrent_count)
        .map(|_| {
            let full_request = FullRequest::new(
                client.clone(),
                request
                    .try_clone()
                    .expect("The request can not be cloned, maybe the body is a stream"),
                shutdown_rx.clone(),
            );

            tokio::spawn(send_requests(full_request))
        })
        .collect();

    let terminal_handle = tokio::spawn(terminal_output(args.method, shutdown_rx));

    tokio::time::sleep(Duration::from_secs(args.time)).await;

    shutdown_tx.send(true)?;

    terminal_handle.await??;

    for handle in handles {
        handle.await??;
    }

    Ok(())
}

fn main() {
    let args = Args::parse();

    println!("Method is: {}", args.method);
    println!("Headers are: {:?}", args.header);

    let runtime = Runtime::new().expect("Could not build the tokio runtime");

    if let Err(error) = runtime.block_on(run(args)) {
        eprintln!("Exited with error: {error}");
    } else {
        println!("Finished.");
    }
}
