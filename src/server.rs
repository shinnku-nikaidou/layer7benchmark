use futures_util::{SinkExt, StreamExt};
use reqwest_websocket::RequestBuilderExt;

pub async fn connect_to_server() -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let resp = client
        .get("ws://127.0.0.1:3000/ws")
        .upgrade()
        .send()
        .await?;

    let mut ws = resp.into_websocket().await?;
    ws.send(reqwest_websocket::Message::Text("Hello".into()))
        .await?;
    while let Some(msg) = ws.next().await {
        let msg = msg?;
        if let reqwest_websocket::Message::Text(txt) = msg {
            println!("收到广播: {}", txt);
        }
    }
    Ok(())
}
