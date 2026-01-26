//! Pattern 5: WebSocket Client
//!
//! Demonstrates connecting to a WebSocket server using tokio-tungstenite.

use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pattern 5: WebSocket Client ===\n");
    println!("Make sure the WebSocket server is running:");
    println!("  cargo run --bin p5_websocket_broadcast\n");

    let url = "ws://127.0.0.1:3000/ws";

    // Connect to the server
    println!("Connecting to {}...", url);
    let (ws_stream, _response) = connect_async(url).await?;
    println!("Connected!\n");

    let (mut write, mut read) = ws_stream.split();

    // Spawn a task to handle incoming messages
    let read_handle = tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received: {}", text);
                }
                Ok(Message::Ping(data)) => {
                    println!("Received ping: {:?}", data);
                }
                Ok(Message::Pong(data)) => {
                    println!("Received pong: {:?}", data);
                }
                Ok(Message::Close(reason)) => {
                    println!("Server closed connection: {:?}", reason);
                    break;
                }
                Ok(Message::Binary(data)) => {
                    println!("Received binary: {} bytes", data.len());
                }
                Ok(Message::Frame(_)) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }
        }
    });

    // Send some test messages
    println!("Sending test messages...\n");

    for i in 1..=5 {
        let msg = format!("Hello from Rust client! Message #{}", i);
        println!("Sending: {}", msg);
        write.send(Message::Text(msg)).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // Close the connection gracefully
    println!("\nClosing connection...");
    write.send(Message::Close(None)).await?;

    // Wait a bit for the read task to receive the close
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    read_handle.abort();

    println!("WebSocket client demo completed!");

    Ok(())
}
