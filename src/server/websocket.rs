use std::sync::Arc;

use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde_json::to_string;
use tokio::{select, spawn};

use super::state::{AppState, WsMessage};

pub async fn upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut tx, mut rx) = socket.split();
    let mut broadcast = state.tx.subscribe();

    let content = state.content.read().await.clone();
    let path = state.file_path.display().to_string();
    let msg = to_string(&WsMessage { msg_type: "update", path: &path, content: &content }).unwrap();

    if tx.send(Message::Text(msg.into())).await.is_err() {
        return;
    }

    let mut send_task = spawn(async move {
        while let Ok(msg) = broadcast.recv().await {
            if tx.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = spawn(async move {
        while let Some(Ok(msg)) = rx.next().await {
            if matches!(msg, Message::Close(_)) {
                break;
            }
        }
    });

    select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}
