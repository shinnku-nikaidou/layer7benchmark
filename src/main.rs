mod args;
use std::time::Duration;

use args::Args;
use clap::Parser;
use reqwest::Client;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut handles = Vec::new();
    let client = Client::new();
    let (shutdown_tx, _) = broadcast::channel(1);

    for _ in 0..args.concurrent_count {
        let url = args.url.clone();
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
