use crate::args::Args;
use crate::build_client::ClientBuildError;
use crate::core_request;
use crate::parse_header::HeadersConfig;
use crate::{build_client, shutdown, terminal};
use anyhow::Result;
use log::{debug, info};
use rand::{rng, Rng};
use reqwest::Client;
use std::time::Duration;
use tokio::sync::watch;
use tokio::task::JoinSet;

fn generate_ip_list(ip_lists: &String) -> Result<Vec<std::net::IpAddr>> {
    let mut ip_list = Vec::new();
    let file_text = std::fs::read_to_string(ip_lists)
        .map_err(|e| anyhow::anyhow!("Failed to read IP list file: {}", e))?;
    for line in file_text.lines() {
        let ip = line.trim();
        if !ip.is_empty() {
            let ip_addr = ip
                .parse::<std::net::IpAddr>()
                .map_err(|e| anyhow::anyhow!("Failed to parse IP address: {}", e))?;
            info!("Get new IP address: {}", ip_addr);
            ip_list.push(ip_addr);
        }
    }
    Ok(ip_list)
}

async fn generate_client(
    url_t: &reqwest::Url,
    ip: &Option<std::net::IpAddr>,
    ip_lists: &Option<Vec<std::net::IpAddr>>,
    headers_config: &HeadersConfig,
    ip_files: &str,
) -> Result<Client, ClientBuildError> {
    if !ip_files.is_empty() {
        let random_ip = generate_random_ip(ip_lists.clone());
        debug!("Build client with Random IP address: {}", random_ip);
        build_client::build_client(url_t, &Some(random_ip), ip_lists, headers_config).await
    } else {
        build_client::build_client(url_t, ip, ip_lists, headers_config).await
    }
}

fn generate_random_ip(ip_lists: Option<Vec<std::net::IpAddr>>) -> std::net::IpAddr {
    let ip_lists = ip_lists.expect("IP list should not be None here");
    let mut rng = rand::rng();
    let random_index = rng.random_range(0..ip_lists.len());
    ip_lists[random_index]
}

pub async fn run(args: Args) -> Result<()> {
    let Args {
        url,
        method,
        random,
        ip_files,
        ..
    } = args;

    let mut ip_lists = None;
    if !ip_files.is_empty() {
        ip_lists = Some(generate_ip_list(&ip_files)?);
    }

    let mut handles = JoinSet::new();
    let timeout = Duration::from_secs(args.timeout);
    let time = Duration::from_secs(args.time);
    let headers_config: HeadersConfig = args.header.into();

    info!("Method is: {}", method);
    headers_config.log_detail();

    let url_t =
        reqwest::Url::parse(&url).map_err(|e| anyhow::anyhow!("Failed to parse URL: {}", e))?;

    let headers = headers_config.other_headers.clone();

    if args.test {
        test_request(
            generate_client(&url_t, &args.ip, &ip_lists, &headers_config, &ip_files).await?,
            url_t,
            method,
            headers,
        )
        .await?;
        return Ok(());
    }

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    for _ in 0..args.concurrent_count {
        let req = core_request::FullRequest {
            url: url.clone(),
            client: generate_client(&url_t, &args.ip, &ip_lists, &headers_config, &ip_files)
                .await?,
            headers: headers.clone(),
            method: method.clone(),
            timeout,
            random,
            body: (!args.body.is_empty()).then(|| args.body.clone()),
        };
        let handle = tokio::spawn(core_request::send_requests(req, shutdown_rx.clone()));
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
