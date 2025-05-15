use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub async fn connect_to_server() -> anyhow::Result<()> {
    let url = url::Url::parse("ws://127.0.0.1:3000/ws").unwrap();
    let (mut ws_stream, _resp) = connect_async(url.as_str()).await.unwrap();
    ws_stream.send(Message::Text("Hello".into())).await.unwrap();
    if let Some(msg) = ws_stream.next().await {
        println!("Received: {:?}", msg.unwrap());
    }
    Ok(())
}
