use anyhow::anyhow;
use chrono::NaiveDateTime;
use chrono::DateTime;
use crate::components::server_request_command::RequestCommand;

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
