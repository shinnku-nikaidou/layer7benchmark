use std::net::IpAddr;
use anyhow::anyhow;
use chrono::NaiveDateTime;
use chrono::DateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestCommand {
    concurrent_count: u32,
    url: String,
    time: Option<u64>,
    ip: Option<IpAddr>,
    header: Vec<HttpHeader>,
    method: RequestMethod,
    body: Option<String>,
    timeout: Option<u64>,
    start_at: Option<NaiveDateTime>,
    abort_if_after: Option<NaiveDateTime>,
    enable_random: bool,
    single_request: bool,
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


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellCommand {
    pub shell: Option<String>,
    pub command: String,
    pub work_dir: Option<String>,
    pub timeout: Option<u64>,
    pub start_at: Option<NaiveDateTime>,
    pub abort_if_after: Option<NaiveDateTime>,
}

impl From<ShellCommand> for crate::server::commands::ShellCommand {
    fn from(value: ShellCommand) -> Self {
        Self {
            shell: value.shell,
            command: value.command,
            work_dir: value.work_dir,
            timeout: value.timeout,
            start_at: value.start_at.map(|t| t.and_utc().timestamp() as u64),
            abort_if_after: value.abort_if_after.map(|t| t.and_utc().timestamp() as u64),
        }
    }
}

impl TryFrom<crate::server::commands::ShellCommand> for ShellCommand {
    type Error = anyhow::Error;

    fn try_from(value: crate::server::commands::ShellCommand) -> Result<Self, Self::Error> {
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
            shell: value.shell,
            command: value.command,
            work_dir: value.work_dir,
            timeout: value.timeout,
            start_at,
            abort_if_after,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteCommand {
    Request(RequestCommand),
    Shell(ShellCommand),
}

impl From<RemoteCommand> for crate::server::commands::Command {
    fn from(value: RemoteCommand) -> Self {
        match value {
            RemoteCommand::Request(request) => Self {
                command: Some(crate::server::commands::command::Command::Request(request.into())),
            },
            RemoteCommand::Shell(shell) => Self {
                command: Some(crate::server::commands::command::Command::Shell(shell.into())),
            },
        }
    }
}

impl TryFrom<crate::server::commands::Command> for RemoteCommand {
    type Error = anyhow::Error;

    fn try_from(value: crate::server::commands::Command) -> Result<Self, Self::Error> {
        match value.command {
            Some(crate::server::commands::command::Command::Request(request)) => {
                Ok(Self::Request(request.try_into()?))
            },
            Some(crate::server::commands::command::Command::SingleRequest(request)) => {
                Ok(Self::Request(request.try_into()?))
            },
            Some(crate::server::commands::command::Command::Shell(shell)) => {
                Ok(Self::Shell(shell.try_into()?))
            },
            None => Err(anyhow!("No command")),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParallelCommands {
    pub commands: Vec<RemoteCommand>,
}

impl From<ParallelCommands> for crate::server::commands::ExecuteGroup {
    fn from(value: ParallelCommands) -> Self {
        Self {
            commands: value.commands.into_iter().map(|c| c.into()).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientStatus {
    Idle,
    Executing {
        id: u64,
        commands: ParallelCommands,
    },
    Waiting{
        id: u64,
        commands: ParallelCommands,
        waiting_until: NaiveDateTime,
    }
}

impl ClientStatus {
    pub fn current_command_id(&self) -> Option<u64> {
        match self {
            ClientStatus::Idle => None,
            ClientStatus::Executing { id, .. } => Some(*id),
            ClientStatus::Waiting { id, .. } => Some(*id),
        }
    }
}

impl From<&ClientStatus> for crate::server::heartbeat::ClientStatus {
    fn from(value: &ClientStatus) -> Self {
        match value {
            ClientStatus::Idle => Self::Idle,
            ClientStatus::Executing { .. } => Self::Requesting,
            ClientStatus::Waiting { .. } => Self::RequestPreparing,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandScheduler {
    pub time_diff: i64,
    pub execute_at: Option<NaiveDateTime>,
}
