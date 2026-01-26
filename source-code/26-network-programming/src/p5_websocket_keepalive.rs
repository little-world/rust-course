//! Pattern 5: WebSocket with Ping/Pong Keep-Alive
//!
//! Demonstrates using ping/pong frames to detect dead connections.

use axum::{
    routing::get,
    Router,
    extract::ws::{WebSocket, WebSocketUpgrade, Message},
    response::IntoResponse,
};
use tokio::time::{interval, Duration, timeout};
use futures_util::{StreamExt, SinkExt};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    println!("=== Pattern 5: WebSocket with Ping/Pong Keep-Alive ===\n");

    let app = Router::new()
        .route("/", get(|| async {
            r#"<!DOCTYPE html>
<html>
<head><title>WebSocket Keepalive Demo</title></head>
<body>
<h1>WebSocket Keepalive Demo</h1>
<div id="log" style="height:400px;overflow-y:scroll;border:1px solid #ccc;padding:10px;font-family:monospace;"></div>
<script>
const log = document.getElementById('log');
function addLog(msg) {
    log.innerHTML += new Date().toISOString().substr(11,8) + ' ' + msg + '<br>';
    log.scrollTop = log.scrollHeight;
}
const ws = new WebSocket('ws://' + location.host + '/ws');
ws.onopen = () => addLog('Connected! Server will ping every 5s');
ws.onclose = () => addLog('Disconnected');
ws.onmessage = (e) => addLog('Message: ' + e.data);
// Note: Browser automatically responds to pings with pongs
// We can't see ping/pong frames directly in JavaScript
</script>
</body>
</html>"#
        }))
        .route("/ws", get(websocket_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("WebSocket server with keepalive running on http://{}", addr);
    println!("Open http://localhost:3000 in your browser");
    println!("\nThe server will:");
    println!("  - Send a ping every 5 seconds");
    println!("  - Disconnect clients that don't respond within 10 seconds");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket_with_keepalive)
}

async fn handle_socket_with_keepalive(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    // Send welcome message
    let _ = sender.send(Message::Text("Connected! Pings every 5s".to_string())).await;

    let mut ping_interval = interval(Duration::from_secs(5));
    let mut last_pong = std::time::Instant::now();

    loop {
        tokio::select! {
            // Send ping periodically
            _ = ping_interval.tick() => {
                // Check if we've received a pong recently
                if last_pong.elapsed() > Duration::from_secs(10) {
                    println!("Client unresponsive (no pong in 10s), disconnecting");
                    let _ = sender.send(Message::Text("Disconnecting: no pong received".to_string())).await;
                    break;
                }

                println!("Sending ping...");
                if sender.send(Message::Ping(vec![1, 2, 3, 4])).await.is_err() {
                    println!("Failed to send ping, client disconnected");
                    break;
                }
            }

            // Handle incoming messages with timeout
            result = timeout(Duration::from_secs(15), receiver.next()) => {
                match result {
                    Ok(Some(Ok(msg))) => {
                        match msg {
                            Message::Pong(data) => {
                                println!("Received pong: {:?}", data);
                                last_pong = std::time::Instant::now();
                            }
                            Message::Ping(data) => {
                                println!("Received ping, sending pong");
                                if sender.send(Message::Pong(data)).await.is_err() {
                                    break;
                                }
                            }
                            Message::Text(text) => {
                                println!("Received text: {}", text);
                                // Echo back
                                let response = format!("Echo: {}", text);
                                if sender.send(Message::Text(response)).await.is_err() {
                                    break;
                                }
                            }
                            Message::Binary(data) => {
                                println!("Received {} bytes of binary data", data.len());
                            }
                            Message::Close(reason) => {
                                println!("Client sent close: {:?}", reason);
                                break;
                            }
                        }
                    }
                    Ok(Some(Err(e))) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                    Ok(None) => {
                        println!("Client disconnected");
                        break;
                    }
                    Err(_) => {
                        // Timeout waiting for message - this is fine, just continue
                        // The ping interval will handle keepalive checks
                    }
                }
            }
        }
    }

    println!("WebSocket connection closed");
}
