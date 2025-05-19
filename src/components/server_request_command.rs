use std::net::IpAddr;
use anyhow::anyhow;
use chrono::{DateTime, NaiveDateTime};

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

impl TryFrom<crate::server::commands::RequestCommand> for RequestCommand {
    type Error = anyhow::Error;

    fn try_from(value: crate::server::commands::RequestCommand) -> Result<Self, Self::Error> {
        let ip = if let Some(ip) = value.ip {
            Some(ip.parse::<IpAddr>().map_err(|e| anyhow!("Invalid IP address: {}", e))?)
        } else {
            None
        };

        let method = match value.method {
            v if v == crate::server::commands::RequestMethod::Get as i32 => RequestMethod::Get,
            v if v == crate::server::commands::RequestMethod::Post as i32 => RequestMethod::Post,
            _ => return Err(anyhow!("Invalid request method")),
        };

        let start_at = if let Some(t) = value.start_at {
            Some(DateTime::from_timestamp(t as i64, 0)
                .ok_or(anyhow!("Invalid start time"))?
                .naive_utc()
            )
        } else {
            None
        };

        let abort_if_after = if let Some(t) = value.abort_if_after {
            Some(DateTime::from_timestamp(t as i64, 0)
                .ok_or(anyhow!("Invalid abort time"))?
                .naive_utc()
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
            enable_random: value.enable_random,
            single_request: value.single_request,
        })
    }
}

impl From<RequestCommand> for crate::server::commands::RequestCommand {
    fn from(value: RequestCommand) -> Self {
        Self {
            concurrent_count: value.concurrent_count,
            url: value.url,
            time: value.time,
            ip: value.ip.map(|ip| ip.to_string()),
            header: value.header.into_iter().map(|h| h.into()).collect(),
            method: Into::<crate::server::commands::RequestMethod>::into(value.method) as i32,
            body: value.body,
            timeout: value.timeout,
            enable_random: value.enable_random,
            single_request: value.single_request,
            start_at: value.start_at.map(|t| t.and_utc().timestamp() as u64),
            abort_if_after: value.abort_if_after.map(|t| t.and_utc().timestamp() as u64),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpHeader(pub String, pub String);

impl From<HttpHeader> for crate::server::commands::HttpHeader {
    fn from(value: HttpHeader) -> Self {
        Self {
            key: value.0,
            value: value.1,
        }
    }
}

impl From<crate::server::commands::HttpHeader> for HttpHeader {
    fn from(value: crate::server::commands::HttpHeader) -> Self  {
        Self(value.key, value.value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestMethod {
    Get,
    Post,
}

impl From<RequestMethod> for crate::server::commands::RequestMethod {
    fn from(value: RequestMethod) -> Self {
        match value {
            RequestMethod::Get => Self::Get,
            RequestMethod::Post => Self::Post,
        }
    }
}

impl From<crate::server::commands::RequestMethod> for RequestMethod {
    fn from(value: crate::server::commands::RequestMethod) -> Self {
        match value {
            crate::server::commands::RequestMethod::Get => RequestMethod::Get,
            crate::server::commands::RequestMethod::Post => RequestMethod::Post,
        }
    }
}