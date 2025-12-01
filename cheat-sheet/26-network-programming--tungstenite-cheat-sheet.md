### Tungstenite Cheat Sheet

```rust
// Cargo.toml:
/*
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.21"
tungstenite = "0.21"
futures-util = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
*/

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_async, connect_async, tungstenite::protocol::Message, WebSocketStream,
    MaybeTlsStream,
};
use futures_util::{StreamExt, SinkExt, stream::{SplitSink, SplitStream}};
use std::net::SocketAddr;

// ===== WEBSOCKET SERVER =====
// Basic WebSocket server
#[tokio::main]
async fn ws_server_basic() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("WebSocket server listening on ws://127.0.0.1:8080");
    
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }
    
    Ok(())
}

async fn handle_connection(stream: TcpStream, addr: SocketAddr) {
    println!("New connection from: {}", addr);
    
    // Upgrade to WebSocket
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("WebSocket handshake failed: {}", e);
            return;
        }
    };
    
    handle_websocket(ws_stream, addr).await;
}

async fn handle_websocket(ws_stream: WebSocketStream<TcpStream>, addr: SocketAddr) {
    let (mut write, mut read) = ws_stream.split();
    
    while let Some(msg) = read.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
        };
        
        match msg {
            Message::Text(text) => {
                println!("Received from {}: {}", addr, text);
                
                // Echo back
                if write.send(Message::Text(text)).await.is_err() {
                    break;
                }
            }
            Message::Binary(data) => {
                println!("Received {} bytes from {}", data.len(), addr);
                if write.send(Message::Binary(data)).await.is_err() {
                    break;
                }
            }
            Message::Close(frame) => {
                println!("Connection closed: {:?}", frame);
                break;
            }
            Message::Ping(data) => {
                if write.send(Message::Pong(data)).await.is_err() {
                    break;
                }
            }
            _ => {}
        }
    }
    
    println!("Connection closed: {}", addr);
}

// ===== WEBSOCKET CLIENT =====
// Basic WebSocket client
#[tokio::main]
async fn ws_client_basic() -> Result<(), Box<dyn std::error::Error>> {
    let (ws_stream, _) = connect_async("ws://127.0.0.1:8080").await?;
    println!("Connected to server");
    
    let (mut write, mut read) = ws_stream.split();
    
    // Send message
    write.send(Message::Text("Hello, Server!".to_string())).await?;
    
    // Receive message
    if let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            println!("Received: {}", text);
        }
    }
    
    // Close connection
    write.send(Message::Close(None)).await?;
    
    Ok(())
}

// ===== MESSAGE TYPES =====
async fn message_types(
    write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Text message
    write.send(Message::Text("Hello".to_string())).await?;
    
    // Binary message
    write.send(Message::Binary(vec![1, 2, 3, 4])).await?;
    
    // Ping
    write.send(Message::Ping(vec![1, 2, 3])).await?;
    
    // Pong
    write.send(Message::Pong(vec![1, 2, 3])).await?;
    
    // Close with reason
    use tungstenite::protocol::CloseFrame;
    let frame = CloseFrame {
        code: tungstenite::protocol::frame::coding::CloseCode::Normal,
        reason: "Goodbye".into(),
    };
    write.send(Message::Close(Some(frame))).await?;
    
    Ok(())
}

// ===== HANDLING DIFFERENT MESSAGE TYPES =====
async fn handle_message(msg: Message) -> Option<Message> {
    match msg {
        Message::Text(text) => {
            println!("Text: {}", text);
            Some(Message::Text(format!("Echo: {}", text)))
        }
        Message::Binary(data) => {
            println!("Binary: {} bytes", data.len());
            Some(Message::Binary(data))
        }
        Message::Ping(data) => {
            println!("Ping received");
            Some(Message::Pong(data))
        }
        Message::Pong(_) => {
            println!("Pong received");
            None
        }
        Message::Close(frame) => {
            println!("Close frame: {:?}", frame);
            None
        }
        Message::Frame(_) => {
            // Raw frame (rarely used)
            None
        }
    }
}

// ===== JSON MESSAGES =====
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    user: String,
    text: String,
    timestamp: u64,
}

async fn send_json(
    write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    msg: &ChatMessage,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(msg)?;
    write.send(Message::Text(json)).await?;
    Ok(())
}

async fn receive_json(
    msg: Message,
) -> Result<ChatMessage, Box<dyn std::error::Error>> {
    if let Message::Text(text) = msg {
        let chat_msg: ChatMessage = serde_json::from_str(&text)?;
        Ok(chat_msg)
    } else {
        Err("Not a text message".into())
    }
}

// ===== BROADCAST SERVER =====
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

type Tx = broadcast::Sender<String>;
type Rx = broadcast::Receiver<String>;

#[tokio::main]
async fn broadcast_server() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, _rx) = broadcast::channel::<String>(100);
    let tx = Arc::new(tx);
    
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Broadcast server listening on ws://127.0.0.1:8080");
    
    while let Ok((stream, addr)) = listener.accept().await {
        let tx = Arc::clone(&tx);
        tokio::spawn(handle_broadcast_client(stream, addr, tx));
    }
    
    Ok(())
}

async fn handle_broadcast_client(
    stream: TcpStream,
    addr: SocketAddr,
    tx: Arc<Tx>,
) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("WebSocket handshake failed: {}", e);
            return;
        }
    };
    
    let (mut ws_write, mut ws_read) = ws_stream.split();
    let mut rx = tx.subscribe();
    
    // Spawn task to receive broadcasts and send to client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_write.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });
    
    // Receive from client and broadcast
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = ws_read.next().await {
            let msg = match msg {
                Ok(msg) => msg,
                Err(_) => break,
            };
            
            if let Message::Text(text) = msg {
                let broadcast_msg = format!("{}: {}", addr, text);
                let _ = tx.send(broadcast_msg);
            } else if let Message::Close(_) = msg {
                break;
            }
        }
    });
    
    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
    
    println!("Connection closed: {}", addr);
}

// ===== CLIENT WITH RECONNECTION =====
use std::time::Duration;

#[tokio::main]
async fn client_with_reconnect() -> Result<(), Box<dyn std::error::Error>> {
    let url = "ws://127.0.0.1:8080";
    let mut retry_count = 0;
    let max_retries = 5;
    
    loop {
        match connect_and_run(url).await {
            Ok(_) => {
                println!("Connection closed normally");
                break;
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
                retry_count += 1;
                
                if retry_count >= max_retries {
                    eprintln!("Max retries reached");
                    break;
                }
                
                let delay = Duration::from_secs(2u64.pow(retry_count));
                println!("Reconnecting in {:?}...", delay);
                tokio::time::sleep(delay).await;
            }
        }
    }
    
    Ok(())
}

async fn connect_and_run(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (ws_stream, _) = connect_async(url).await?;
    println!("Connected to {}", url);
    
    let (mut write, mut read) = ws_stream.split();
    
    // Send periodic messages
    let mut send_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            if write.send(Message::Text("ping".to_string())).await.is_err() {
                break;
            }
        }
    });
    
    // Receive messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => println!("Received: {}", text),
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });
    
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
    
    Ok(())
}

// ===== SECURE WEBSOCKET (WSS) =====
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

#[tokio::main]
async fn wss_client() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to secure WebSocket
    let url = "wss://echo.websocket.org";
    let (ws_stream, _) = connect_async(url).await?;
    println!("Connected to {}", url);
    
    let (mut write, mut read) = ws_stream.split();
    
    write.send(Message::Text("Hello, WSS!".to_string())).await?;
    
    if let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            println!("Received: {}", text);
        }
    }
    
    Ok(())
}

// WSS with custom headers
async fn wss_with_headers() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = "wss://echo.websocket.org".into_client_request()?;
    request.headers_mut().insert(
        "Authorization",
        "Bearer token123".parse().unwrap(),
    );
    
    let (ws_stream, _) = connect_async(request).await?;
    
    // Use ws_stream...
    
    Ok(())
}

// ===== PING/PONG HEARTBEAT =====
async fn client_with_heartbeat(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();
    
    // Heartbeat task
    let mut ping_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            if write.send(Message::Ping(vec![])).await.is_err() {
                break;
            }
        }
    });
    
    // Receive task
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => println!("Received: {}", text),
                Ok(Message::Pong(_)) => println!("Pong received"),
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });
    
    tokio::select! {
        _ = (&mut ping_task) => recv_task.abort(),
        _ = (&mut recv_task) => ping_task.abort(),
    }
    
    Ok(())
}

// ===== ROOM-BASED CHAT SERVER =====
use std::collections::HashMap;

type RoomId = String;
type ClientId = usize;
type ClientSender = tokio::sync::mpsc::UnboundedSender<Message>;

struct ChatServer {
    rooms: Arc<Mutex<HashMap<RoomId, HashMap<ClientId, ClientSender>>>>,
    next_client_id: Arc<Mutex<ClientId>>,
}

impl ChatServer {
    fn new() -> Self {
        ChatServer {
            rooms: Arc::new(Mutex::new(HashMap::new())),
            next_client_id: Arc::new(Mutex::new(0)),
        }
    }
    
    async fn handle_client(
        &self,
        stream: TcpStream,
        addr: SocketAddr,
    ) {
        let ws_stream = match accept_async(stream).await {
            Ok(ws) => ws,
            Err(e) => {
                eprintln!("WebSocket handshake failed: {}", e);
                return;
            }
        };
        
        let (mut ws_write, mut ws_read) = ws_stream.split();
        
        // Get client ID
        let client_id = {
            let mut id = self.next_client_id.lock().await;
            let current = *id;
            *id += 1;
            current
        };
        
        // Create channel for this client
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Task to send messages to client
        let mut send_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if ws_write.send(msg).await.is_err() {
                    break;
                }
            }
        });
        
        // Task to receive messages from client
        let rooms = Arc::clone(&self.rooms);
        let mut recv_task = tokio::spawn(async move {
            let mut current_room: Option<RoomId> = None;
            
            while let Some(msg) = ws_read.next().await {
                let msg = match msg {
                    Ok(msg) => msg,
                    Err(_) => break,
                };
                
                match msg {
                    Message::Text(text) => {
                        // Parse commands: JOIN room, LEAVE, MSG text
                        if text.starts_with("JOIN ") {
                            let room = text[5..].to_string();
                            
                            // Leave current room if any
                            if let Some(old_room) = current_room.take() {
                                let mut rooms = rooms.lock().await;
                                if let Some(clients) = rooms.get_mut(&old_room) {
                                    clients.remove(&client_id);
                                }
                            }
                            
                            // Join new room
                            let mut rooms = rooms.lock().await;
                            rooms
                                .entry(room.clone())
                                .or_insert_with(HashMap::new)
                                .insert(client_id, tx.clone());
                            current_room = Some(room);
                            
                        } else if text == "LEAVE" {
                            if let Some(room) = current_room.take() {
                                let mut rooms = rooms.lock().await;
                                if let Some(clients) = rooms.get_mut(&room) {
                                    clients.remove(&client_id);
                                }
                            }
                            
                        } else if let Some(room) = &current_room {
                            // Broadcast to room
                            let rooms = rooms.lock().await;
                            if let Some(clients) = rooms.get(room) {
                                let broadcast_msg = format!("{}: {}", addr, text);
                                for (id, client_tx) in clients.iter() {
                                    if *id != client_id {
                                        let _ = client_tx.send(Message::Text(broadcast_msg.clone()));
                                    }
                                }
                            }
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            
            // Cleanup: remove from room
            if let Some(room) = current_room {
                let mut rooms = rooms.lock().await;
                if let Some(clients) = rooms.get_mut(&room) {
                    clients.remove(&client_id);
                }
            }
        });
        
        tokio::select! {
            _ = (&mut send_task) => recv_task.abort(),
            _ = (&mut recv_task) => send_task.abort(),
        }
    }
}

// ===== COMMON PATTERNS =====

// Pattern 1: Echo server
async fn echo_server() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            if let Ok(ws_stream) = accept_async(stream).await {
                let (write, read) = ws_stream.split();
                let _ = read.forward(write).await;
            }
        });
    }
    
    Ok(())
}

// Pattern 2: Message rate limiting
use std::collections::VecDeque;

struct RateLimiter {
    timestamps: VecDeque<tokio::time::Instant>,
    max_messages: usize,
    window: Duration,
}

impl RateLimiter {
    fn new(max_messages: usize, window: Duration) -> Self {
        RateLimiter {
            timestamps: VecDeque::new(),
            max_messages,
            window,
        }
    }
    
    fn check(&mut self) -> bool {
        let now = tokio::time::Instant::now();
        
        // Remove old timestamps
        while let Some(&first) = self.timestamps.front() {
            if now.duration_since(first) > self.window {
                self.timestamps.pop_front();
            } else {
                break;
            }
        }
        
        // Check limit
        if self.timestamps.len() < self.max_messages {
            self.timestamps.push_back(now);
            true
        } else {
            false
        }
    }
}

// Pattern 3: Authentication handshake
async fn authenticated_handler(stream: TcpStream) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(_) => return,
    };
    
    let (mut write, mut read) = ws_stream.split();
    
    // Wait for auth message
    let auth_msg = match read.next().await {
        Some(Ok(Message::Text(text))) => text,
        _ => {
            let _ = write.send(Message::Close(None)).await;
            return;
        }
    };
    
    if !verify_token(&auth_msg) {
        let _ = write.send(Message::Text("AUTH_FAILED".to_string())).await;
        let _ = write.send(Message::Close(None)).await;
        return;
    }
    
    let _ = write.send(Message::Text("AUTH_SUCCESS".to_string())).await;
    
    // Continue with authenticated connection...
}

fn verify_token(_token: &str) -> bool {
    // Verify authentication token
    true
}

// Pattern 4: Binary protocol
async fn send_binary_frame(
    write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    message_type: u8,
    data: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut frame = Vec::with_capacity(1 + data.len());
    frame.push(message_type);
    frame.extend_from_slice(data);
    
    write.send(Message::Binary(frame)).await?;
    Ok(())
}

async fn parse_binary_frame(data: Vec<u8>) -> Option<(u8, Vec<u8>)> {
    if data.is_empty() {
        return None;
    }
    
    let message_type = data[0];
    let payload = data[1..].to_vec();
    
    Some((message_type, payload))
}
```