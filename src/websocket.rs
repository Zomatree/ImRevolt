use std::sync::mpsc::Sender;
use futures::{SinkExt, StreamExt};
use revolt_database::events::client::{EventV1, Ping};
use serde::Serialize;
use tokio_tungstenite::connect_async;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Authenticate { token: String },
    BeginTyping { channel: String },
    EndTyping { channel: String },
    Subscribe { server_id: String },
    Ping { data: Ping, responded: Option<()> },
}

use crate::http::RevoltConfig;

pub async fn run(event_sender: Sender<EventV1>, token: String, api_info: RevoltConfig) {
    let (ws, _) = connect_async(&api_info.ws).await.unwrap();

    let (mut ws_send, mut ws_receive) = ws.split();

    let send = |e: ClientMessage| async move {
        ws_send.send(tungstenite::Message::Text(serde_json::to_string(&e).unwrap())).await
    };

    send(ClientMessage::Authenticate { token }).await.unwrap();

    tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receive.next().await {
            if let Ok(data) = msg.to_text() {
                event_sender.send(serde_json::from_str(data).unwrap()).unwrap();
            }
        }
    });
}