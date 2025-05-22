use anyhow::anyhow;
use chrono::{DateTime, NaiveDateTime};
use log::debug;
use serde::Deserialize;
use tonic::transport::Channel;
use url::Url;

pub mod l7b {
    pub mod commands {
        tonic::include_proto!("l7b.commands");
    }
    pub mod heartbeat {
        tonic::include_proto!("l7b.heartbeat");
    }
}
use crate::components::controlled_mode::server::heartbeat::server_response::NextOperation;
use crate::components::controlled_mode::server_command_executor::ServerCommandExecutor;
use l7b::commands::CommandResultItem;
use l7b::heartbeat::heartbeat_service_client::HeartbeatServiceClient;
use l7b::heartbeat::*;
pub use l7b::*;

async fn get_self_ip() -> anyhow::Result<String> {
    #[derive(Deserialize)]
    struct IpInfoResponse {
        pub ip: String,
    }

    reqwest::get("https://ipinfo.io")
        .await?
        .json::<IpInfoResponse>()
        .await
        .map(|ip_info| ip_info.ip)
        .map_err(|e| e.into())
}

pub async fn send_heartbeat(
    client: &mut HeartbeatServiceClient<Channel>,
    self_ip: &str,
    command_result: Box<[CommandResultItem]>,
    client_status: super::server_command_executor::ClientStatus,
    now: NaiveDateTime,
) -> anyhow::Result<ServerResponse> {
    let status: ClientStatus = (&client_status).into();
    let heartbeat = HeartBeat {
        timestamp: now.and_utc().timestamp() as u64,
        status: status as i32,
        current_command_id: client_status.current_command_id(),
        command_result: command_result.clone().into(),
        ip: self_ip.to_owned(),
    };
    debug!("Sending heartbeat: {:?}", heartbeat);
    let response = client.heartbeat(heartbeat).await?;
    Ok(response.into_inner())
}

pub async fn connect_to_server(url: Url) -> anyhow::Result<()> {
    let self_ip = get_self_ip().await?;
    let mut grpc_client = HeartbeatServiceClient::connect(url.to_string()).await?;
    let mut executor = ServerCommandExecutor::new();
    let mut stop_time = chrono::Utc::now().naive_utc() + chrono::Duration::days(30);

    loop {
        let now = chrono::Utc::now().naive_utc();
        if now > stop_time {
            log::info!("Stopping execution as the time limit has been reached.");
            executor.shutdown_workers().await?;
            stop_time = chrono::Utc::now().naive_utc() + chrono::Duration::days(30);
        }

        let ServerResponse {
            server_timestamp,
            next_operation,
            command_id,
        } = match executor
            .send_heartbeat(&mut grpc_client, &self_ip, now)
            .await
        {
            Ok(heartbeat) => {
                log::debug!("Received heartbeat: {:?}", heartbeat);
                heartbeat
            }
            Err(e) => {
                log::error!("Failed to send heartbeat: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                continue;
            }
        };
        let server_time = DateTime::from_timestamp(server_timestamp as i64, 0)
            .ok_or(anyhow!("Invalid server timestamp"))?
            .naive_utc();

        executor.clock_sync(server_time, now).await;

        if let Some(next_operation) = next_operation {
            match next_operation {
                NextOperation::KeepIdle(Empty {}) => {}
                NextOperation::ContinueCurrent(Empty {}) => {}
                NextOperation::StopCurrent(Empty {}) => {
                    executor.shutdown_workers().await?;
                    stop_time = chrono::Utc::now().naive_utc() + chrono::Duration::days(30);
                }
                NextOperation::StopAndExecute(commands) => {
                    executor.shutdown_workers().await?;
                    stop_time = chrono::Utc::now().naive_utc() + chrono::Duration::days(30);
                    let max_time = executor
                        .execute(commands.try_into()?, command_id.unwrap_or(0))
                        .await;
                    match max_time {
                        Ok(max_time) => {
                            stop_time = now + chrono::Duration::seconds(max_time as i64);
                        }
                        Err(e) => {
                            log::error!("Failed to execute commands: {}", e);
                            continue;
                        }
                    }
                }
                NextOperation::Execute(commands) => {
                    stop_time = chrono::Utc::now().naive_utc() + chrono::Duration::days(30);
                    let max_time = executor
                        .execute(commands.try_into()?, command_id.unwrap_or(0))
                        .await;
                    match max_time {
                        Ok(max_time) => {
                            stop_time = now + chrono::Duration::seconds(max_time as i64);
                        }
                        Err(e) => {
                            log::error!("Failed to execute commands: {}", e);
                            continue;
                        }
                    }
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(20)).await;
    }
}
