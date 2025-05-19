use std::collections::VecDeque;
use chrono::NaiveDateTime;
use url::Url;
use serde::Deserialize;
use tonic::transport::Channel;

pub mod l7b {
    pub mod commands {
        tonic::include_proto!("l7b.commands");
    }
    pub mod heartbeat {
        tonic::include_proto!("l7b.heartbeat");
    }
}
pub use l7b::*;
use l7b::heartbeat::*;
use crate::server::commands::CommandResultItem;
use crate::server::l7b::heartbeat::heartbeat_service_client::HeartbeatServiceClient;

async fn get_self_ip() -> anyhow::Result<String> {
    #[derive(Deserialize)]
    struct IpInfoResponse {
        pub ip: String
    }

    reqwest::get("https://ipinfo.io")
        .await?
        .json::<IpInfoResponse>()
        .await
        .map(|ip_info| ip_info.ip)
        .map_err(|e| e.into())
}

async fn send_heartbeat(
    client: &mut HeartbeatServiceClient<Channel>,
    self_ip: &str,
    command_result: &VecDeque<CommandResultItem>,
    client_status: &crate::components::server_command_executor::ClientStatus,
    now: NaiveDateTime
) -> anyhow::Result<ServerResponse> {
    let status: ClientStatus = client_status.into();
    let heartbeat = HeartBeat {
        timestamp: now.and_utc().timestamp() as u64,
        status: status as i32,
        current_command_id: client_status.current_command_id(),
        command_result: command_result.clone().into(),
        ip: self_ip.to_owned(),
    };
    let response = client.heartbeat(heartbeat).await?;
    Ok(response.into_inner())
}

pub async fn connect_to_server(url: Url) -> anyhow::Result<()> {
    let self_ip = get_self_ip().await?;
    let grpc_client = HeartbeatServiceClient::connect(
        url.to_string()
    ).await?;

    Ok(())
}

