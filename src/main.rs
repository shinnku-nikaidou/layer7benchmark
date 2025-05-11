mod args;
use args::Args;
use clap::Parser;
use reqwest::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut handles = Vec::new();
    let client = Client::new();

    for _ in 0..args.concurrent_count {
        let url = args.url.clone();
        let client = client.clone();
        let handle = tokio::spawn(async move {
            if let Ok(resp) = client.get(&url).send().await {
                let _ = resp.bytes().await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}
