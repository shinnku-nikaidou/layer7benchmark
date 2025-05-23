use super::header::HeadersConfig;
use std::collections::HashSet;
use tokio::io;

use crate::components::client::executor::BenchmarkReady;
use anyhow::Result;
use log::error;
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::io::AsyncWriteExt;
use tokio::net::lookup_host;

#[derive(thiserror::Error, Debug)]
pub enum ClientBuildError {
    #[error("URL is missing host component")]
    URLMissingHost,

    #[allow(dead_code)]
    #[error("URL is required")]
    UrlIsRequired,

    #[error("Failed to build HTTP client: \n{0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Failed to resolve IP address for domain {0}")]
    DNSLookupFailed(String),

    #[error("Failed to read IP list file: {0}")]
    FailedToReadIpListFile(io::Error),

    #[error("No valid IP addresses found in file")]
    NoValidIpInFile,
}

async fn rebuild_ip_list_file(ip_list: &HashSet<IpAddr>, path: &PathBuf) -> io::Result<()> {
    let mut file = tokio::fs::File::create(path).await?;
    let file_string = ip_list
        .iter()
        .map(|ip| ip.to_string())
        .collect::<Vec<String>>()
        .join("\n");
    file.write_all(file_string.as_bytes()).await?;
    Ok(())
}

pub async fn read_ip_files(ip_lists: PathBuf) -> Result<Vec<IpAddr>, ClientBuildError> {
    let file_text = tokio::fs::read_to_string(&ip_lists)
        .await
        .map_err(ClientBuildError::FailedToReadIpListFile)?;

    let ips: HashSet<_> = file_text
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .filter_map(|line| IpAddr::from_str(line).ok())
        .collect();

    if ips.is_empty() {
        return Err(ClientBuildError::NoValidIpInFile);
    }

    if let Err(e) = rebuild_ip_list_file(&ips, &ip_lists).await {
        error!("Failed to rebuild ip list: {}", e);
    };
    let ip_list = ips.into_iter().collect();
    Ok(ip_list)
}

#[derive(Debug, Clone, Default)]
pub struct BenchmarkBuilder {
    pub url: Option<String>,
    pub ip_mode: ClientIpSelectMode,
    pub headers_config: HeadersConfig,
    pub method: reqwest::Method,
}

impl BenchmarkBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn resolve_dns(mut self) -> Self {
        self.ip_mode = ClientIpSelectMode::Resolve;
        self
    }

    pub fn fixed_ip(mut self, ip: IpAddr) -> Self {
        self.ip_mode = ClientIpSelectMode::Locked(ip);
        self
    }

    pub async fn random_ip_from_file(mut self, path: PathBuf) -> Result<Self, ClientBuildError> {
        let ips = read_ip_files(path).await?;
        self.ip_mode = ClientIpSelectMode::Random(ips);
        Ok(self)
    }

    pub fn headers_config(mut self, config: HeadersConfig) -> Self {
        self.headers_config = config;
        self
    }

    pub async fn build(self) -> Result<BenchmarkReady, ClientBuildError> {
        BenchmarkReady::from_builder(self).await
    }

    pub fn method(mut self, method: reqwest::Method) -> Self {
        self.method = method;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum ClientIpSelectMode {
    #[default]
    Resolve,
    Locked(IpAddr),
    Random(Vec<IpAddr>),
}

impl ClientIpSelectMode {
    pub async fn resolve(self, domain: &str) -> Result<Box<[IpAddr]>, ClientBuildError> {
        match self {
            ClientIpSelectMode::Resolve => {
                let ips = lookup_host((domain, 0))
                    .await
                    .map_err(|e| ClientBuildError::DNSLookupFailed(e.to_string()))?
                    .map(|socket_addr| socket_addr.ip())
                    .collect();
                log::info!("Resolved IPs: {:?}", ips);
                Ok(ips)
            }
            ClientIpSelectMode::Locked(ip) => {
                log::info!("Using fixed IP: {}", ip);
                Ok(Box::new([ip]))
            }
            ClientIpSelectMode::Random(ips) => Ok(ips.into_boxed_slice()),
        }
    }
}
