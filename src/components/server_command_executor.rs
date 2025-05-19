use std::collections::VecDeque;
use std::sync::Arc;
use chrono::NaiveDateTime;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::task::{JoinHandle, JoinSet};
use crate::components::server_command::ParallelCommands;
use crate::server::commands::CommandResultItem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientStatus {
    Idle,
    Executing {
        id: u64,
    },
    Waiting{
        id: u64,
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

pub struct ServerCommandExecutor {
    /// state machine
    status: ClientStatus,
    
    /// sync the clock between server and client
    time_diff: Arc<Mutex<i64>>,
    
    /// a command result collector
    command_result_sender: Arc<Mutex<VecDeque<CommandResultItem>>>,
    
    /// worker threads
    worker_spawns: JoinSet<()>,
    
    /// statistic
    statistic: &'static crate::statistic::Statistic,
    
    /// shell command result sender
    shell_command_result_sender: mpsc::Sender<CommandResultItem>,
    
    /// shell command result receiver
    shell_command_result_receiver: mpsc::Receiver<CommandResultItem>,
    
    /// single request result sender
    single_req_result_sender: mpsc::Sender<CommandResultItem>,
    
    /// single request result receiver
    single_req_result_receiver: mpsc::Receiver<CommandResultItem>,
}

impl ServerCommandExecutor {
    pub fn new() -> Self {
        let (shell_tx, shell_rx) = mpsc::channel(100);
        let (single_req_tx, single_req_rx) = mpsc::channel(100);
        Self {
            status: ClientStatus::Idle,
            time_diff: Arc::new(Mutex::new(0)),
            command_result_sender: Arc::new(Mutex::new(VecDeque::new())),
            worker_spawns: JoinSet::new(),
            statistic: &crate::statistic::STATISTIC.get().unwrap(),
            shell_command_result_sender: shell_tx,
            shell_command_result_receiver: shell_rx,
            single_req_result_sender: single_req_tx,
            single_req_result_receiver: single_req_rx,
        }
    }
    
    pub async fn pop_results(&self) -> Box<[CommandResultItem]> {
        self.command_result_sender.lock().await.drain(..).collect()
    }
    
    pub async fn clock_sync(&self, server_time: NaiveDateTime, client_time: NaiveDateTime) {
        *self.time_diff.lock().await = server_time.and_utc().timestamp() - client_time.and_utc().timestamp();
    }
    
    pub async fn shutdown_workers(&mut self) -> anyhow::Result<()> {
        self.worker_spawns.abort_all();
        self.status = ClientStatus::Idle;
        Ok(())
    }
    
    pub async fn execute(&mut self, commands: ParallelCommands, id: u64) {
        self.status = ClientStatus::Executing {
            id,
        };
    }
}