use crate::components::controlled_mode::server::commands::CommandResultItem;
use crate::components::controlled_mode::server::heartbeat::ServerResponse;
use crate::components::controlled_mode::server::heartbeat::heartbeat_service_client::HeartbeatServiceClient;
use crate::components::controlled_mode::server::{commands, heartbeat, send_heartbeat};
use crate::components::controlled_mode::server_command::{ParallelCommands, RemoteCommand};
use chrono::NaiveDateTime;
use log::{info, warn};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinSet;
use tonic::transport::Channel;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ClientStatus {
    Idle,
    Executing {
        id: u64,
    },
    Waiting {
        id: u64,
        waiting_until: NaiveDateTime,
    },
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

impl From<&ClientStatus> for heartbeat::ClientStatus {
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

    /// worker threads
    worker_spawns: JoinSet<()>,

    /// statistic
    statistic: Arc<crate::statistic::Statistic>,

    shutdown_tx: tokio::sync::watch::Sender<bool>,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,

    output_sender: mpsc::Sender<CommandResultItem>,
    output_receiver: mpsc::Receiver<CommandResultItem>,
}

impl ServerCommandExecutor {
    pub fn new() -> Self {
        let (output_tx, output_rx) = mpsc::channel(100);
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        Self {
            status: ClientStatus::Idle,
            time_diff: Arc::new(Mutex::new(0)),
            worker_spawns: JoinSet::new(),
            statistic: Arc::new(crate::statistic::Statistic::default()),
            output_sender: output_tx,
            output_receiver: output_rx,
            shutdown_tx,
            shutdown_rx,
        }
    }

    pub async fn pop_results(&mut self) -> Box<[CommandResultItem]> {
        let mut results = Vec::new();
        while let Ok(result) = self.output_receiver.try_recv() {
            results.push(result);
        }
        results.push(CommandResultItem {
            command_result: Some(commands::command_result_item::CommandResult::Request(
                self.statistic.status_counter.get_grpc(),
            )),
        });
        results.into_boxed_slice()
    }

    pub async fn clock_sync(&self, server_time: NaiveDateTime, client_time: NaiveDateTime) {
        *self.time_diff.lock().await =
            server_time.and_utc().timestamp() - client_time.and_utc().timestamp();
    }

    pub async fn shutdown_workers(&mut self) -> anyhow::Result<()> {
        self.worker_spawns.abort_all();
        self.status = ClientStatus::Idle;
        self.shutdown_tx.send(true)?;
        info!("shutdown_workers done, killed");
        Ok(())
    }

    pub async fn execute(&mut self, commands: ParallelCommands, id: u64) {
        log::debug!("Executing commands: {:?}", commands);

        let now = chrono::Utc::now().naive_utc();
        // if self.shutdown_tx.send(false).is_err() {
        //     self.status = ClientStatus::Idle;
        //     log::error!("Failed to send shutdown signal as false to workers");
        //     return;
        // }
        self.status = ClientStatus::Executing { id };
        let commands = commands
            .commands
            .into_iter()
            .filter(|c| c.start_at().is_none())
            .filter(|c| match c.abort_if_after() {
                Some(t) => t > now,
                None => true,
            })
            .collect::<Vec<_>>();
        for command in commands {
            match command {
                RemoteCommand::Request(request) => {
                    if let Err(e) = request
                        .execute(
                            &mut self.worker_spawns,
                            self.statistic.clone(),
                            self.shutdown_tx.clone(),
                            self.shutdown_rx.clone(),
                            self.output_sender.clone(),
                        )
                        .await
                    {
                        log::error!("Failed to execute request: {}", e);
                    }
                }
                RemoteCommand::Shell(_) => {
                    warn!("Not implemented");
                }
            }
        }
        
    }

    pub async fn check_idle(&mut self) {
        if self.worker_spawns.is_empty() {
            self.status = ClientStatus::Idle;
        }
    }

    pub async fn send_heartbeat(
        &mut self,
        client: &mut HeartbeatServiceClient<Channel>,
        self_ip: &str,
        now: NaiveDateTime,
    ) -> anyhow::Result<ServerResponse> {
        self.check_idle().await;
        let command_result = self.pop_results().await;
        let status = self.status;
        send_heartbeat(client, self_ip, command_result, status, now).await
    }
}
