use crate::components::client::client_builder::{BenchmarkBuilder, ClientBuildError};
use crate::components::client::executor::BenchmarkReady;
use crate::components::client::request::send_requests;
use crate::components::controlled_mode::server::commands;
use crate::components::controlled_mode::server::commands::CommandResultItem;
use crate::components::shutdown;
use anyhow::anyhow;
use chrono::{DateTime, NaiveDateTime};
use std::net::IpAddr;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestCommand {
    pub concurrent_count: u32,
    pub url: String,
    pub time: Option<u64>,
    pub ip: Option<IpAddr>,
    pub header: Vec<HttpHeader>,
    pub method: RequestMethod,
    pub body: Option<String>,
    pub timeout: Option<u64>,
    pub start_at: Option<NaiveDateTime>,
    pub abort_if_after: Option<NaiveDateTime>,
    pub enable_random: bool,
    pub single_request: bool,
}

impl RequestCommand {
    pub async fn ready(&self) -> Result<BenchmarkReady, ClientBuildError> {
        let mut builder = BenchmarkBuilder::new()
            .url(self.url.clone())
            .method(self.method.into());

        if let Some(ip) = self.ip {
            builder = builder.fixed_ip(ip);
        }

        builder.build().await
    }
    pub async fn execute_single(&self) -> anyhow::Result<reqwest::Response> {
        let ready = self.ready().await?;
        ready.single_request().await.map_err(|e| e.into())
    }

    pub async fn execute_multi(
        &self,
        join_set: &mut tokio::task::JoinSet<()>,
        statistic: Arc<crate::statistic::Statistic>,
        shutdown_rx: tokio::sync::watch::Receiver<bool>,
    ) -> anyhow::Result<()> {
        log::debug!("Executing multi request");
        let ready = self.ready().await?;
        let requests = ready.build_full_requests(
            self.concurrent_count,
            self.timeout
                .map(std::time::Duration::from_secs)
                .unwrap_or(std::time::Duration::from_secs(10)),
            self.body.clone(),
            self.enable_random,
        );
        for request in requests {
            join_set.spawn(send_requests(
                request,
                shutdown_rx.clone(),
                statistic.clone(),
            ));
        }
        Ok(())
    }

    pub async fn execute(
        &self,
        join_set: &mut tokio::task::JoinSet<()>,
        statistic: Arc<crate::statistic::Statistic>,
        shutdown_tx: tokio::sync::watch::Sender<bool>,
        shutdown_rx: tokio::sync::watch::Receiver<bool>,
        output_stream: tokio::sync::mpsc::Sender<CommandResultItem>,
    ) -> anyhow::Result<()> {
        let time = self.time.unwrap_or(0);
        log::debug!("Executing request command: {:?}", self);

        if self.single_request {
            let response = self.execute_single().await?;
            output_stream
                .send(CommandResultItem {
                    command_result: Some(
                        commands::command_result_item::CommandResult::SingleRequest(
                            commands::SingleRequestResultItem {
                                code: response.status().as_u16() as u32,
                                content: response.text().await?,
                                timestamp: chrono::Utc::now().timestamp() as u64,
                            },
                        ),
                    ),
                })
                .await
                .map_err(|e| anyhow!("Failed to send command result: {}", e))?;
        } else {
            tokio::spawn(shutdown::wait_for_completion(
                std::time::Duration::from_secs(time),
                shutdown_tx.clone(),
                shutdown_rx.clone(),
            ));
            println!("in side this, tag");
            self.execute_multi(join_set, statistic, shutdown_rx.clone())
                .await?;
        }
        Ok(())
    }
}

impl TryFrom<commands::RequestCommand> for RequestCommand {
    type Error = anyhow::Error;

    fn try_from(value: commands::RequestCommand) -> Result<Self, Self::Error> {
        let ip = if let Some(ip) = value.ip {
            Some(
                ip.parse::<IpAddr>()
                    .map_err(|e| anyhow!("Invalid IP address: {}", e))?,
            )
        } else {
            None
        };

        let method = match value.method {
            v if v == commands::RequestMethod::Get as i32 => RequestMethod::Get,
            v if v == commands::RequestMethod::Post as i32 => RequestMethod::Post,
            _ => return Err(anyhow!("Invalid request method")),
        };

        let start_at = if let Some(t) = value.start_at {
            Some(
                DateTime::from_timestamp(t as i64, 0)
                    .ok_or(anyhow!("Invalid start time"))?
                    .naive_utc(),
            )
        } else {
            None
        };

        let abort_if_after = if let Some(t) = value.abort_if_after {
            Some(
                DateTime::from_timestamp(t as i64, 0)
                    .ok_or(anyhow!("Invalid abort time"))?
                    .naive_utc(),
            )
        } else {
            None
        };

        Ok(Self {
            concurrent_count: value.concurrent_count,
            url: value.url,
            time: value.time,
            ip,
            header: value.header.into_iter().map(|h| h.into()).collect(),
            method,
            body: value.body,
            timeout: value.timeout,
            start_at,
            abort_if_after,
            single_request: false,
            enable_random: value.enable_random,
        })
    }
}

impl From<RequestCommand> for commands::RequestCommand {
    fn from(value: RequestCommand) -> Self {
        Self {
            concurrent_count: value.concurrent_count,
            url: value.url.to_string(),
            time: value.time,
            ip: value.ip.map(|ip| ip.to_string()),
            header: value.header.into_iter().map(|h| h.into()).collect(),
            method: Into::<commands::RequestMethod>::into(value.method) as i32,
            body: value.body,
            timeout: value.timeout,
            enable_random: value.enable_random,
            start_at: value.start_at.map(|t| t.and_utc().timestamp() as u64),
            abort_if_after: value.abort_if_after.map(|t| t.and_utc().timestamp() as u64),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpHeader(pub String, pub String);

impl From<HttpHeader> for commands::HttpHeader {
    fn from(value: HttpHeader) -> Self {
        Self {
            key: value.0,
            value: value.1,
        }
    }
}

impl From<commands::HttpHeader> for HttpHeader {
    fn from(value: commands::HttpHeader) -> Self {
        Self(value.key, value.value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestMethod {
    Get,
    Post,
}

impl From<RequestMethod> for commands::RequestMethod {
    fn from(value: RequestMethod) -> Self {
        match value {
            RequestMethod::Get => Self::Get,
            RequestMethod::Post => Self::Post,
        }
    }
}

impl From<commands::RequestMethod> for RequestMethod {
    fn from(value: commands::RequestMethod) -> Self {
        match value {
            commands::RequestMethod::Get => RequestMethod::Get,
            commands::RequestMethod::Post => RequestMethod::Post,
        }
    }
}

impl From<commands::RequestMethod> for reqwest::Method {
    fn from(value: commands::RequestMethod) -> Self {
        match value {
            commands::RequestMethod::Get => reqwest::Method::GET,
            commands::RequestMethod::Post => reqwest::Method::POST,
        }
    }
}

impl From<RequestMethod> for reqwest::Method {
    fn from(value: RequestMethod) -> Self {
        match value {
            RequestMethod::Get => reqwest::Method::GET,
            RequestMethod::Post => reqwest::Method::POST,
        }
    }
}
