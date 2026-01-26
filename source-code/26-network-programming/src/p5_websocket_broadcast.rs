//! Pattern 5: WebSocket Broadcast Server
//!
//! Demonstrates a chat-like WebSocket server that broadcasts messages
//! to all connected clients using Tokio's broadcast channel.

use axum::{
    routing::get,
    Router,
    extract::{
        ws::{WebSocket, WebSocketUpgrade, Message},
        State,
    },
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use futures_util::{StreamExt, SinkExt};
use std::net::SocketAddr;

#[derive(Clone)]
struct AppState {
    // Broadcast channel for sending messages to all clients
    tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    println!("=== Pattern 5: WebSocket Broadcast Server ===\n");

    // Create broadcast channel with capacity 100
    let (tx, _rx) = broadcast::channel(100);

    let state = AppState { tx };

    let app = Router::new()
        .route("/", get(|| async {
            r#"<!DOCTYPE html>
<html>
<head><title>WebSocket Chat</title></head>
<body>
<h1>WebSocket Chat Demo</h1>
<div id="messages" style="height:300px;overflow-y:scroll;border:1px solid #ccc;padding:10px;"></div>
<input type="text" id="input" placeholder="Type a message..." style="width:300px;">
<button onclick="send()">Send</button>
<script>
const ws = new WebSocket('ws://' + location.host + '/ws');
const messages = document.getElementById('messages');
const input = document.getElementById('input');
ws.onmessage = (e) => {
    messages.innerHTML += '<div>' + e.data + '</div>';
    messages.scrollTop = messages.scrollHeight;
};
ws.onopen = () => messages.innerHTML += '<div><em>Connected!</em></div>';
ws.onclose = () => messages.innerHTML += '<div><em>Disconnected</em></div>';
function send() {
    if (input.value) {
        ws.send(input.value);
        input.value = '';
    }
}
input.onkeypress = (e) => { if (e.key === 'Enter') send(); };
</script>
</body>
</html>"#
        }))
        .route("/ws", get(websocket_handler))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("WebSocket server running on http://{}", addr);
    println!("Open http://localhost:3000 in your browser");
    println!("Or test with: cargo run --bin p5_websocket_client");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/// Handle WebSocket upgrade requests
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Complete the WebSocket upgrade
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle an individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: AppState) {
    // Split the socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast channel
    let mut rx = state.tx.subscribe();

    // Spawn a task to send broadcast messages to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // Send message to this client
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Spawn a task to receive messages from this client
    let tx = state.tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                println!("Received: {}", text);
                // Broadcast the message to all clients
                let _ = tx.send(format!("[User]: {}", text));
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    println!("Client disconnected");
}
