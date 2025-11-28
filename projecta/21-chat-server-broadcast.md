# Project 2: Chat Server with Broadcast and Backpressure

## Problem Statement

Build a **multi-client chat server** that broadcasts messages to all connected clients while handling backpressure, timeouts, and graceful disconnections. The server must prevent fast senders from overwhelming slow receivers and support commands like `/name`, `/list`, `/whisper`, and `/quit`.

**Use Cases**:
- Real-time chat applications with hundreds of concurrent users
- WebSocket-based notification systems
- Pub/sub message brokers with heterogeneous subscriber speeds
- Multiplayer game lobbies with chat features

## Why It Matters

Chat servers demonstrate **async broadcasting patterns** where you must:
- Handle clients at different speeds (slow clients can't block fast ones)
- Implement backpressure to prevent memory exhaustion
- Detect and handle idle/disconnected clients
- Maintain shared state (username registry) across async tasks

**Performance Impact**:
- **Without backpressure**: A single slow client (1 msg/sec) blocks all clients, causing 10+ second delays
- **With bounded channels**: Fast clients maintain <10ms latency even when slow clients lag
- **Without timeouts**: Zombie connections waste resources (file descriptors, memory)
- **With idle detection**: 30-second timeout reclaims resources automatically

These patterns apply to **WebSocket servers**, **pub/sub systems**, and **real-time applications** where clients have varying network conditions.

---

## Milestone 1: Basic TCP Echo Server

**Goal**: Accept multiple TCP connections concurrently and echo received lines back to the sender.

**Concepts**:
- `TcpListener::accept()` in a loop
- Spawning tasks with `tokio::spawn`
- Line-based protocol with `BufReader::lines()`
- Handling client disconnections

**Implementation Steps**:

1. **Create `TcpListener` and bind to address**:
   - Use `TcpListener::bind("127.0.0.1:8080").await?`
   - Print "Server listening on ..." when ready

2. **Accept connections in a loop**:
   - Use `listener.accept().await?` to get `(TcpStream, SocketAddr)`
   - Spawn a new task with `tokio::spawn` for each connection
   - Pass ownership of `TcpStream` to the spawned task

3. **Implement `handle_client` function**:
   - Split the stream: `let (reader, mut writer) = stream.split()`
   - Wrap reader in `BufReader::new(reader)`
   - Use `.lines()` to iterate over lines asynchronously
   - For each line, write it back: `writer.write_all(line.as_bytes()).await?`
   - Write a newline: `writer.write_all(b"\n").await?`

4. **Handle disconnections**:
   - When `.lines()` returns `None` or an error, the client disconnected
   - Print "Client disconnected: {addr}" and exit the task

**Starter Code**:

```rust
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    // TODO: Bind TcpListener to 127.0.0.1:8080

    println!("Server listening on 127.0.0.1:8080");

    loop {
        // TODO: Accept a connection

        // TODO: Spawn a task to handle the client
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, addr).await {
                eprintln!("Error handling client {}: {}", addr, e);
            }
        });
    }
}

async fn handle_client(stream: TcpStream, addr: std::net::SocketAddr) -> io::Result<()> {
    println!("Client connected: {}", addr);

    // TODO: Split stream into reader and writer

    // TODO: Wrap reader in BufReader

    // TODO: Create lines() stream

    // TODO: Loop over lines
    while let Some(line) = lines.next_line().await? {
        // TODO: Echo line back to client (with newline)
    }

    println!("Client disconnected: {}", addr);
    Ok(())
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    #[tokio::test]
    async fn test_echo_single_line() {
        // Start server in background
        tokio::spawn(async {
            main().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut client = TcpStream::connect("127.0.0.1:8080").await.unwrap();
        client.write_all(b"Hello\n").await.unwrap();

        let mut buf = vec![0u8; 6];
        client.read_exact(&mut buf).await.unwrap();

        assert_eq!(&buf, b"Hello\n");
    }

    #[tokio::test]
    async fn test_echo_multiple_lines() {
        tokio::spawn(async {
            main().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut client = TcpStream::connect("127.0.0.1:8080").await.unwrap();
        client.write_all(b"Line1\nLine2\n").await.unwrap();

        let mut buf = vec![0u8; 12];
        client.read_exact(&mut buf).await.unwrap();

        assert_eq!(&buf, b"Line1\nLine2\n");
    }
}
```

**Check Your Understanding**:
1. Why do we spawn a new task for each client instead of handling them sequentially?
2. What happens if a client sends partial data without a newline and disconnects?
3. Why do we split the `TcpStream` into reader and writer?

---

## Milestone 2: Broadcast Channel with Manual Distribution

**Goal**: Broadcast messages from any client to all other connected clients using a shared broadcast channel.

**Concepts**:
- `tokio::sync::broadcast` channel for message distribution
- Shared state with `Arc<Mutex<HashMap>>`
- Cloning the broadcast sender for each client
- Filtering out the sender from broadcast recipients

**Implementation Steps**:

1. **Create a broadcast channel**:
   - Use `tokio::sync::broadcast::channel::<String>(100)` (capacity 100)
   - Wrap the sender in `Arc` to share across tasks

2. **Store client information**:
   - Create a struct `ClientInfo { addr: SocketAddr, tx: mpsc::Sender<String> }`
   - Use `Arc<Mutex<HashMap<SocketAddr, ClientInfo>>>` to store all clients
   - When a client connects, create an `mpsc::channel` for that client
   - Insert the client into the HashMap

3. **Spawn a broadcast listener task for each client**:
   - Call `broadcast_rx.subscribe()` to get a receiver
   - In a loop, receive messages from broadcast channel
   - Filter out messages sent by this client (by address)
   - Send to the client's mpsc sender

4. **Spawn a writer task for each client**:
   - Receive messages from the client's mpsc receiver
   - Write them to the TcpStream writer

5. **Handle client messages**:
   - When a client sends a line, broadcast it via `broadcast_tx.send(message)?`
   - The broadcast channel will distribute to all subscribers

6. **Clean up on disconnect**:
   - Remove client from the HashMap when they disconnect

**Starter Code**:

```rust
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

type ClientMap = Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<String>>>>;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");

    // TODO: Create broadcast channel (capacity 100)
    let (broadcast_tx, _) = broadcast::channel(100);
    let broadcast_tx = Arc::new(broadcast_tx);

    // TODO: Create client map
    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, addr) = listener.accept().await?;
        let broadcast_tx = Arc::clone(&broadcast_tx);
        let clients = Arc::clone(&clients);

        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, addr, broadcast_tx, clients).await {
                eprintln!("Error handling client {}: {}", addr, e);
            }
        });
    }
}

async fn handle_client(
    stream: TcpStream,
    addr: SocketAddr,
    broadcast_tx: Arc<broadcast::Sender<String>>,
    clients: ClientMap,
) -> io::Result<()> {
    println!("Client connected: {}", addr);

    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut writer = writer;

    // TODO: Create mpsc channel for this client (capacity 10)
    let (client_tx, mut client_rx) = mpsc::channel::<String>(10);

    // TODO: Insert client into the map
    {
        let mut clients = clients.lock().await;
        clients.insert(addr, client_tx);
    }

    // TODO: Subscribe to broadcast channel
    let mut broadcast_rx = broadcast_tx.subscribe();

    // Spawn task to forward broadcast messages to this client
    let addr_clone = addr;
    tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            // TODO: Send to client's mpsc channel
            // Hint: Use client_tx.send(msg).await
            // Handle channel full by logging a warning
        }
    });

    // Spawn task to write messages to the client
    tokio::spawn(async move {
        while let Some(msg) = client_rx.recv().await {
            // TODO: Write message to TCP stream
            if let Err(e) = writer.write_all(msg.as_bytes()).await {
                eprintln!("Error writing to {}: {}", addr_clone, e);
                break;
            }
            if let Err(e) = writer.write_all(b"\n").await {
                eprintln!("Error writing newline to {}: {}", addr_clone, e);
                break;
            }
        }
    });

    // Read lines from client and broadcast
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        // TODO: Format message as "[addr]: message"
        let message = format!("[{}]: {}", addr, line);

        // TODO: Broadcast to all clients
        let _ = broadcast_tx.send(message);
    }

    // TODO: Remove client from map on disconnect
    {
        let mut clients = clients.lock().await;
        clients.remove(&addr);
    }

    println!("Client disconnected: {}", addr);
    Ok(())
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    #[tokio::test]
    async fn test_broadcast_to_multiple_clients() {
        tokio::spawn(async {
            main().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut client1 = TcpStream::connect("127.0.0.1:8080").await.unwrap();
        let mut client2 = TcpStream::connect("127.0.0.1:8080").await.unwrap();

        // Client1 sends a message
        client1.write_all(b"Hello from client1\n").await.unwrap();

        // Client2 should receive the broadcast
        let mut buf = vec![0u8; 50];
        let n = tokio::time::timeout(
            tokio::time::Duration::from_secs(1),
            client2.read(&mut buf),
        )
        .await
        .unwrap()
        .unwrap();

        let received = String::from_utf8_lossy(&buf[..n]);
        assert!(received.contains("Hello from client1"));
    }

    #[tokio::test]
    async fn test_client_does_not_receive_own_message() {
        tokio::spawn(async {
            main().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut client = TcpStream::connect("127.0.0.1:8080").await.unwrap();
        client.write_all(b"Test\n").await.unwrap();

        // Try to read with timeout - should timeout since client shouldn't receive own message
        let mut buf = vec![0u8; 50];
        let result = tokio::time::timeout(
            tokio::time::Duration::from_millis(500),
            client.read(&mut buf),
        )
        .await;

        // For now, this test will fail because we haven't implemented filtering yet
        // We'll fix this in the next step
    }
}
```

**Check Your Understanding**:
1. Why do we use both a broadcast channel and per-client mpsc channels?
2. What happens if the mpsc channel is full when we try to send a message?
3. Why do we need `Arc<Mutex<HashMap>>` instead of just `Arc<HashMap>`?

---

## Milestone 3: Username Registry and Message Filtering

**Goal**: Add username support, prevent duplicates, filter out own messages, and implement `/name` command.

**Concepts**:
- Username registry with `HashMap<String, SocketAddr>`
- Message tagging with sender address
- Command parsing (`/name username`)
- Filtering broadcast messages by sender

**Implementation Steps**:

1. **Create a message structure**:
   - Define `struct Message { sender: SocketAddr, content: String }`
   - Change broadcast channel to `broadcast::channel::<Message>(100)`

2. **Create username registry**:
   - Add `Arc<Mutex<HashMap<String, SocketAddr>>>` for usernames
   - Add reverse map `Arc<Mutex<HashMap<SocketAddr, String>>>` for addr->username lookup

3. **Implement `/name` command**:
   - Parse lines starting with `/name `
   - Extract the username after the command
   - Check if username is already taken (search username registry)
   - If available, insert into both maps
   - Send confirmation to the client

4. **Filter own messages in broadcast receiver**:
   - When receiving from broadcast, check `if msg.sender == addr { continue; }`
   - Only forward messages from other clients

5. **Format messages with username**:
   - Look up username from addr->username map
   - Format as `[username]: content` or `[addr]: content` if no username set

**Starter Code**:

```rust
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
struct Message {
    sender: SocketAddr,
    content: String,
}

type ClientMap = Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<String>>>>;
type UsernameMap = Arc<Mutex<HashMap<String, SocketAddr>>>;
type AddrToUsername = Arc<Mutex<HashMap<SocketAddr, String>>>;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");

    let (broadcast_tx, _) = broadcast::channel::<Message>(100);
    let broadcast_tx = Arc::new(broadcast_tx);

    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));

    // TODO: Create username registries
    let usernames: UsernameMap = Arc::new(Mutex::new(HashMap::new()));
    let addr_to_username: AddrToUsername = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, addr) = listener.accept().await?;
        let broadcast_tx = Arc::clone(&broadcast_tx);
        let clients = Arc::clone(&clients);
        let usernames = Arc::clone(&usernames);
        let addr_to_username = Arc::clone(&addr_to_username);

        tokio::spawn(async move {
            if let Err(e) = handle_client(
                stream,
                addr,
                broadcast_tx,
                clients,
                usernames,
                addr_to_username,
            )
            .await
            {
                eprintln!("Error handling client {}: {}", addr, e);
            }
        });
    }
}

async fn handle_client(
    stream: TcpStream,
    addr: SocketAddr,
    broadcast_tx: Arc<broadcast::Sender<Message>>,
    clients: ClientMap,
    usernames: UsernameMap,
    addr_to_username: AddrToUsername,
) -> io::Result<()> {
    println!("Client connected: {}", addr);

    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut writer = writer;

    let (client_tx, mut client_rx) = mpsc::channel::<String>(10);

    {
        let mut clients = clients.lock().await;
        clients.insert(addr, client_tx.clone());
    }

    let mut broadcast_rx = broadcast_tx.subscribe();

    // Forward broadcast messages (filter out own messages)
    let client_tx_clone = client_tx.clone();
    tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            // TODO: Filter out messages sent by this client
            if msg.sender == addr {
                continue;
            }

            // Forward to client
            let _ = client_tx_clone.send(msg.content).await;
        }
    });

    // Write messages to TCP stream
    tokio::spawn(async move {
        while let Some(msg) = client_rx.recv().await {
            if let Err(e) = writer.write_all(msg.as_bytes()).await {
                eprintln!("Error writing to {}: {}", addr, e);
                break;
            }
            if let Err(e) = writer.write_all(b"\n").await {
                eprintln!("Error writing newline to {}: {}", addr, e);
                break;
            }
        }
    });

    // Read and process commands/messages
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        // TODO: Check if line starts with /name
        if line.starts_with("/name ") {
            let username = line[6..].trim().to_string();

            // TODO: Check if username is taken
            let mut usernames_guard = usernames.lock().await;
            if usernames_guard.contains_key(&username) {
                // Send error to client
                let _ = client_tx.send(format!("Error: Username '{}' is already taken", username)).await;
                continue;
            }

            // TODO: Register username
            usernames_guard.insert(username.clone(), addr);
            drop(usernames_guard);

            let mut addr_to_username_guard = addr_to_username.lock().await;
            addr_to_username_guard.insert(addr, username.clone());
            drop(addr_to_username_guard);

            // Send confirmation
            let _ = client_tx.send(format!("Username set to '{}'", username)).await;
            continue;
        }

        // TODO: Format message with username or addr
        let sender_name = {
            let addr_to_username_guard = addr_to_username.lock().await;
            addr_to_username_guard
                .get(&addr)
                .cloned()
                .unwrap_or_else(|| addr.to_string())
        };

        let message = Message {
            sender: addr,
            content: format!("[{}]: {}", sender_name, line),
        };

        // Broadcast to all clients
        let _ = broadcast_tx.send(message);
    }

    // Cleanup on disconnect
    {
        let mut clients = clients.lock().await;
        clients.remove(&addr);
    }

    {
        let mut addr_to_username_guard = addr_to_username.lock().await;
        if let Some(username) = addr_to_username_guard.remove(&addr) {
            let mut usernames_guard = usernames.lock().await;
            usernames_guard.remove(&username);
        }
    }

    println!("Client disconnected: {}", addr);
    Ok(())
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpStream;

    #[tokio::test]
    async fn test_set_username() {
        tokio::spawn(async {
            main().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut client = TcpStream::connect("127.0.0.1:8080").await.unwrap();
        client.write_all(b"/name Alice\n").await.unwrap();

        let (reader, _) = client.into_split();
        let mut reader = BufReader::new(reader).lines();

        let response = reader.next_line().await.unwrap().unwrap();
        assert_eq!(response, "Username set to 'Alice'");
    }

    #[tokio::test]
    async fn test_duplicate_username_rejected() {
        tokio::spawn(async {
            main().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut client1 = TcpStream::connect("127.0.0.1:8080").await.unwrap();
        let mut client2 = TcpStream::connect("127.0.0.1:8080").await.unwrap();

        client1.write_all(b"/name Bob\n").await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        client2.write_all(b"/name Bob\n").await.unwrap();

        let (reader, _) = client2.into_split();
        let mut reader = BufReader::new(reader).lines();

        let response = reader.next_line().await.unwrap().unwrap();
        assert!(response.contains("already taken"));
    }

    #[tokio::test]
    async fn test_message_with_username() {
        tokio::spawn(async {
            main().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut client1 = TcpStream::connect("127.0.0.1:8080").await.unwrap();
        let mut client2 = TcpStream::connect("127.0.0.1:8080").await.unwrap();

        client1.write_all(b"/name Charlie\n").await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        client1.write_all(b"Hello everyone!\n").await.unwrap();

        let (reader, _) = client2.into_split();
        let mut reader = BufReader::new(reader).lines();

        let message = reader.next_line().await.unwrap().unwrap();
        assert_eq!(message, "[Charlie]: Hello everyone!");
    }
}
```

**Check Your Understanding**:
1. Why do we need both a `username -> addr` map and an `addr -> username` map?
2. What happens if two clients try to register the same username simultaneously?
3. Why do we filter messages in the broadcast receiver instead of before sending?

---

## Milestone 4: Backpressure with Bounded Channels and Timeouts

**Goal**: Implement bounded channels to prevent slow clients from blocking fast ones, and add idle timeout detection.

**Concepts**:
- Bounded `mpsc::channel` with explicit capacity
- Handling `try_send` errors for full channels
- Idle timeout with `tokio::time::timeout`
- Graceful client disconnection on timeout

**Implementation Steps**:

1. **Use bounded channels with small capacity**:
   - Change `mpsc::channel(10)` to `mpsc::channel(5)` to test backpressure
   - This makes it easier to trigger the "channel full" condition

2. **Handle channel full errors**:
   - When forwarding broadcast messages, use `try_send` instead of `send`
   - If `try_send` returns `Err(TrySendError::Full(_))`, log a warning
   - Count dropped messages per client and log periodically

3. **Implement idle timeout**:
   - Use `tokio::time::timeout(Duration::from_secs(30), lines.next_line())`
   - If timeout occurs, send a message to the client and disconnect
   - Reset timeout on each received message

4. **Add heartbeat/ping mechanism** (optional):
   - Send a ping message every 10 seconds to idle clients
   - Expect a response within 5 seconds
   - Disconnect if no response

**Starter Code**:

```rust
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio::time::{timeout, Duration};
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
struct Message {
    sender: SocketAddr,
    content: String,
}

type ClientMap = Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<String>>>>;
type UsernameMap = Arc<Mutex<HashMap<String, SocketAddr>>>;
type AddrToUsername = Arc<Mutex<HashMap<SocketAddr, String>>>;

const IDLE_TIMEOUT: Duration = Duration::from_secs(30);
const CHANNEL_CAPACITY: usize = 5;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");

    let (broadcast_tx, _) = broadcast::channel::<Message>(100);
    let broadcast_tx = Arc::new(broadcast_tx);

    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));
    let usernames: UsernameMap = Arc::new(Mutex::new(HashMap::new()));
    let addr_to_username: AddrToUsername = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, addr) = listener.accept().await?;
        let broadcast_tx = Arc::clone(&broadcast_tx);
        let clients = Arc::clone(&clients);
        let usernames = Arc::clone(&usernames);
        let addr_to_username = Arc::clone(&addr_to_username);

        tokio::spawn(async move {
            if let Err(e) = handle_client(
                stream,
                addr,
                broadcast_tx,
                clients,
                usernames,
                addr_to_username,
            )
            .await
            {
                eprintln!("Error handling client {}: {}", addr, e);
            }
        });
    }
}

async fn handle_client(
    stream: TcpStream,
    addr: SocketAddr,
    broadcast_tx: Arc<broadcast::Sender<Message>>,
    clients: ClientMap,
    usernames: UsernameMap,
    addr_to_username: AddrToUsername,
) -> io::Result<()> {
    println!("Client connected: {}", addr);

    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut writer = writer;

    // TODO: Use bounded channel with CHANNEL_CAPACITY
    let (client_tx, mut client_rx) = mpsc::channel::<String>(CHANNEL_CAPACITY);

    {
        let mut clients = clients.lock().await;
        clients.insert(addr, client_tx.clone());
    }

    let mut broadcast_rx = broadcast_tx.subscribe();

    // Forward broadcast messages with backpressure handling
    let client_tx_clone = client_tx.clone();
    tokio::spawn(async move {
        let mut dropped_count = 0;

        while let Ok(msg) = broadcast_rx.recv().await {
            if msg.sender == addr {
                continue;
            }

            // TODO: Use try_send to handle full channel
            match client_tx_clone.try_send(msg.content) {
                Ok(_) => {
                    // Successfully sent
                    if dropped_count > 0 {
                        println!("[{}] Recovered, dropped {} messages", addr, dropped_count);
                        dropped_count = 0;
                    }
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    // Channel full - client is slow
                    dropped_count += 1;
                    if dropped_count % 10 == 0 {
                        eprintln!("[{}] Slow client, dropped {} messages", addr, dropped_count);
                    }
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    // Client disconnected
                    break;
                }
            }
        }
    });

    // Write messages to TCP stream
    tokio::spawn(async move {
        while let Some(msg) = client_rx.recv().await {
            if let Err(e) = writer.write_all(msg.as_bytes()).await {
                eprintln!("Error writing to {}: {}", addr, e);
                break;
            }
            if let Err(e) = writer.write_all(b"\n").await {
                eprintln!("Error writing newline to {}: {}", addr, e);
                break;
            }
        }
    });

    // Read and process commands/messages with idle timeout
    let mut lines = reader.lines();
    loop {
        // TODO: Wrap next_line with timeout
        match timeout(IDLE_TIMEOUT, lines.next_line()).await {
            Ok(Ok(Some(line))) => {
                // Process message
                if line.starts_with("/name ") {
                    let username = line[6..].trim().to_string();

                    let mut usernames_guard = usernames.lock().await;
                    if usernames_guard.contains_key(&username) {
                        let _ = client_tx.send(format!("Error: Username '{}' is already taken", username)).await;
                        continue;
                    }

                    usernames_guard.insert(username.clone(), addr);
                    drop(usernames_guard);

                    let mut addr_to_username_guard = addr_to_username.lock().await;
                    addr_to_username_guard.insert(addr, username.clone());
                    drop(addr_to_username_guard);

                    let _ = client_tx.send(format!("Username set to '{}'", username)).await;
                    continue;
                }

                let sender_name = {
                    let addr_to_username_guard = addr_to_username.lock().await;
                    addr_to_username_guard
                        .get(&addr)
                        .cloned()
                        .unwrap_or_else(|| addr.to_string())
                };

                let message = Message {
                    sender: addr,
                    content: format!("[{}]: {}", sender_name, line),
                };

                let _ = broadcast_tx.send(message);
            }
            Ok(Ok(None)) => {
                // Client disconnected
                break;
            }
            Ok(Err(e)) => {
                // Read error
                eprintln!("Error reading from {}: {}", addr, e);
                break;
            }
            Err(_) => {
                // TODO: Timeout - client is idle
                println!("[{}] Idle timeout, disconnecting", addr);
                let _ = client_tx.send("Disconnected due to inactivity".to_string()).await;
                break;
            }
        }
    }

    // Cleanup
    {
        let mut clients = clients.lock().await;
        clients.remove(&addr);
    }

    {
        let mut addr_to_username_guard = addr_to_username.lock().await;
        if let Some(username) = addr_to_username_guard.remove(&addr) {
            let mut usernames_guard = usernames.lock().await;
            usernames_guard.remove(&username);
        }
    }

    println!("Client disconnected: {}", addr);
    Ok(())
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpStream;

    #[tokio::test]
    async fn test_slow_client_backpressure() {
        tokio::spawn(async {
            main().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut fast_client = TcpStream::connect("127.0.0.1:8080").await.unwrap();
        let slow_client = TcpStream::connect("127.0.0.1:8080").await.unwrap();

        // Slow client doesn't read messages
        // Fast client sends many messages
        for i in 0..20 {
            fast_client
                .write_all(format!("Message {}\n", i).as_bytes())
                .await
                .unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Check that server logs indicate dropped messages for slow client
        // (This test mainly checks that the server doesn't crash)
    }

    #[tokio::test]
    async fn test_idle_timeout() {
        tokio::spawn(async {
            main().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let client = TcpStream::connect("127.0.0.1:8080").await.unwrap();

        // Don't send anything for 31 seconds
        tokio::time::sleep(tokio::time::Duration::from_secs(31)).await;

        // Try to read - should get disconnect message or EOF
        let (reader, _) = client.into_split();
        let mut reader = BufReader::new(reader).lines();

        let result = reader.next_line().await;
        // Should either get disconnect message or None (EOF)
        assert!(
            result.is_ok() && (result.as_ref().unwrap().is_none()
                || result.as_ref().unwrap().as_ref().unwrap().contains("inactivity"))
        );
    }

    #[tokio::test]
    async fn test_activity_resets_timeout() {
        tokio::spawn(async {
            main().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut client = TcpStream::connect("127.0.0.1:8080").await.unwrap();

        // Send a message every 20 seconds (within timeout)
        for _ in 0..3 {
            client.write_all(b"Ping\n").await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
        }

        // Client should still be connected
        client.write_all(b"Still here\n").await.unwrap();
    }
}
```

**Check Your Understanding**:
1. Why do we use `try_send` instead of `send` for broadcast messages?
2. What happens to messages when a client's channel is full?
3. How does the idle timeout prevent resource exhaustion from zombie connections?

---

## Milestone 5: Commands (/list, /whisper, /quit) and Production Features

**Goal**: Implement chat commands, graceful shutdown, and connection metrics.

**Concepts**:
- Command parsing and routing
- Private messages (whisper)
- Listing online users
- Graceful shutdown with `CancellationToken`
- Connection metrics (total connections, active users)

**Implementation Steps**:

1. **Implement `/list` command**:
   - Lock the `addr_to_username` map
   - Collect all usernames (or addresses if no username)
   - Format as a list and send to the requesting client only

2. **Implement `/whisper` command**:
   - Parse `/whisper <username> <message>`
   - Look up the target username in the username registry
   - Get their `mpsc::Sender` from the clients map
   - Send the message directly (not via broadcast)
   - Format as `[Whisper from <sender>]: <message>`

3. **Implement `/quit` command**:
   - Send a goodbye message to the client
   - Break out of the read loop to trigger cleanup

4. **Add graceful shutdown**:
   - Create a `CancellationToken` and clone it for each client task
   - On SIGINT/SIGTERM, cancel the token
   - In each client task, select between `token.cancelled()` and reading lines
   - When cancelled, send a shutdown message and disconnect clients

5. **Track connection metrics**:
   - Add `Arc<Mutex<ConnectionMetrics>>` with fields: `total_connections`, `active_connections`, `total_messages`
   - Increment counters appropriately
   - Add a `/stats` command to display metrics

**Starter Code**:

```rust
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio::time::{timeout, Duration};
use tokio_util::sync::CancellationToken;
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
struct Message {
    sender: SocketAddr,
    content: String,
}

#[derive(Default)]
struct ConnectionMetrics {
    total_connections: u64,
    active_connections: u64,
    total_messages: u64,
}

type ClientMap = Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<String>>>>;
type UsernameMap = Arc<Mutex<HashMap<String, SocketAddr>>>;
type AddrToUsername = Arc<Mutex<HashMap<SocketAddr, String>>>;
type Metrics = Arc<Mutex<ConnectionMetrics>>;

const IDLE_TIMEOUT: Duration = Duration::from_secs(30);
const CHANNEL_CAPACITY: usize = 5;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");

    let (broadcast_tx, _) = broadcast::channel::<Message>(100);
    let broadcast_tx = Arc::new(broadcast_tx);

    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));
    let usernames: UsernameMap = Arc::new(Mutex::new(HashMap::new()));
    let addr_to_username: AddrToUsername = Arc::new(Mutex::new(HashMap::new()));

    // TODO: Create metrics and cancellation token
    let metrics: Metrics = Arc::new(Mutex::new(ConnectionMetrics::default()));
    let cancel_token = CancellationToken::new();

    // TODO: Spawn shutdown handler
    let cancel_token_clone = cancel_token.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        println!("\nShutting down server...");
        cancel_token_clone.cancel();
    });

    loop {
        tokio::select! {
            result = listener.accept() => {
                let (stream, addr) = result?;

                // Update metrics
                {
                    let mut metrics_guard = metrics.lock().await;
                    metrics_guard.total_connections += 1;
                    metrics_guard.active_connections += 1;
                }

                let broadcast_tx = Arc::clone(&broadcast_tx);
                let clients = Arc::clone(&clients);
                let usernames = Arc::clone(&usernames);
                let addr_to_username = Arc::clone(&addr_to_username);
                let metrics = Arc::clone(&metrics);
                let cancel_token = cancel_token.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_client(
                        stream,
                        addr,
                        broadcast_tx,
                        clients,
                        usernames,
                        addr_to_username,
                        metrics,
                        cancel_token,
                    )
                    .await
                    {
                        eprintln!("Error handling client {}: {}", addr, e);
                    }
                });
            }
            _ = cancel_token.cancelled() => {
                println!("Server shutdown complete");
                break;
            }
        }
    }

    Ok(())
}

async fn handle_client(
    stream: TcpStream,
    addr: SocketAddr,
    broadcast_tx: Arc<broadcast::Sender<Message>>,
    clients: ClientMap,
    usernames: UsernameMap,
    addr_to_username: AddrToUsername,
    metrics: Metrics,
    cancel_token: CancellationToken,
) -> io::Result<()> {
    println!("Client connected: {}", addr);

    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut writer = writer;

    let (client_tx, mut client_rx) = mpsc::channel::<String>(CHANNEL_CAPACITY);

    {
        let mut clients = clients.lock().await;
        clients.insert(addr, client_tx.clone());
    }

    let mut broadcast_rx = broadcast_tx.subscribe();

    // Forward broadcast messages
    let client_tx_clone = client_tx.clone();
    tokio::spawn(async move {
        let mut dropped_count = 0;

        while let Ok(msg) = broadcast_rx.recv().await {
            if msg.sender == addr {
                continue;
            }

            match client_tx_clone.try_send(msg.content) {
                Ok(_) => {
                    if dropped_count > 0 {
                        println!("[{}] Recovered, dropped {} messages", addr, dropped_count);
                        dropped_count = 0;
                    }
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    dropped_count += 1;
                    if dropped_count % 10 == 0 {
                        eprintln!("[{}] Slow client, dropped {} messages", addr, dropped_count);
                    }
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    break;
                }
            }
        }
    });

    // Write messages to TCP stream
    tokio::spawn(async move {
        while let Some(msg) = client_rx.recv().await {
            if let Err(e) = writer.write_all(msg.as_bytes()).await {
                eprintln!("Error writing to {}: {}", addr, e);
                break;
            }
            if let Err(e) = writer.write_all(b"\n").await {
                eprintln!("Error writing newline to {}: {}", addr, e);
                break;
            }
        }
    });

    // Read and process commands/messages
    let mut lines = reader.lines();
    loop {
        tokio::select! {
            result = timeout(IDLE_TIMEOUT, lines.next_line()) => {
                match result {
                    Ok(Ok(Some(line))) => {
                        // Update metrics
                        {
                            let mut metrics_guard = metrics.lock().await;
                            metrics_guard.total_messages += 1;
                        }

                        // TODO: Handle /list command
                        if line == "/list" {
                            let user_list = {
                                let addr_to_username_guard = addr_to_username.lock().await;
                                let mut users: Vec<String> = addr_to_username_guard
                                    .iter()
                                    .map(|(addr, name)| format!("{} ({})", name, addr))
                                    .collect();

                                if users.is_empty() {
                                    "No users online".to_string()
                                } else {
                                    format!("Users online:\n{}", users.join("\n"))
                                }
                            };
                            let _ = client_tx.send(user_list).await;
                            continue;
                        }

                        // TODO: Handle /whisper command
                        if line.starts_with("/whisper ") {
                            let parts: Vec<&str> = line[9..].splitn(2, ' ').collect();
                            if parts.len() < 2 {
                                let _ = client_tx.send("Usage: /whisper <username> <message>".to_string()).await;
                                continue;
                            }

                            let target_username = parts[0];
                            let message = parts[1];

                            // Look up target
                            let target_addr = {
                                let usernames_guard = usernames.lock().await;
                                usernames_guard.get(target_username).copied()
                            };

                            match target_addr {
                                Some(target_addr) => {
                                    let sender_name = {
                                        let addr_to_username_guard = addr_to_username.lock().await;
                                        addr_to_username_guard
                                            .get(&addr)
                                            .cloned()
                                            .unwrap_or_else(|| addr.to_string())
                                    };

                                    let whisper_msg = format!("[Whisper from {}]: {}", sender_name, message);

                                    // Send to target
                                    let clients_guard = clients.lock().await;
                                    if let Some(target_tx) = clients_guard.get(&target_addr) {
                                        let _ = target_tx.send(whisper_msg.clone()).await;
                                        let _ = client_tx.send(format!("[Whisper to {}]: {}", target_username, message)).await;
                                    }
                                }
                                None => {
                                    let _ = client_tx.send(format!("User '{}' not found", target_username)).await;
                                }
                            }
                            continue;
                        }

                        // TODO: Handle /stats command
                        if line == "/stats" {
                            let stats = {
                                let metrics_guard = metrics.lock().await;
                                format!(
                                    "Server Statistics:\nTotal connections: {}\nActive connections: {}\nTotal messages: {}",
                                    metrics_guard.total_connections,
                                    metrics_guard.active_connections,
                                    metrics_guard.total_messages
                                )
                            };
                            let _ = client_tx.send(stats).await;
                            continue;
                        }

                        // TODO: Handle /quit command
                        if line == "/quit" {
                            let _ = client_tx.send("Goodbye!".to_string()).await;
                            break;
                        }

                        // Handle /name command
                        if line.starts_with("/name ") {
                            let username = line[6..].trim().to_string();

                            let mut usernames_guard = usernames.lock().await;
                            if usernames_guard.contains_key(&username) {
                                let _ = client_tx.send(format!("Error: Username '{}' is already taken", username)).await;
                                continue;
                            }

                            usernames_guard.insert(username.clone(), addr);
                            drop(usernames_guard);

                            let mut addr_to_username_guard = addr_to_username.lock().await;
                            addr_to_username_guard.insert(addr, username.clone());
                            drop(addr_to_username_guard);

                            let _ = client_tx.send(format!("Username set to '{}'", username)).await;
                            continue;
                        }

                        // Regular message - broadcast
                        let sender_name = {
                            let addr_to_username_guard = addr_to_username.lock().await;
                            addr_to_username_guard
                                .get(&addr)
                                .cloned()
                                .unwrap_or_else(|| addr.to_string())
                        };

                        let message = Message {
                            sender: addr,
                            content: format!("[{}]: {}", sender_name, line),
                        };

                        let _ = broadcast_tx.send(message);
                    }
                    Ok(Ok(None)) => {
                        break;
                    }
                    Ok(Err(e)) => {
                        eprintln!("Error reading from {}: {}", addr, e);
                        break;
                    }
                    Err(_) => {
                        println!("[{}] Idle timeout, disconnecting", addr);
                        let _ = client_tx.send("Disconnected due to inactivity".to_string()).await;
                        break;
                    }
                }
            }
            _ = cancel_token.cancelled() => {
                println!("[{}] Server shutting down", addr);
                let _ = client_tx.send("Server is shutting down".to_string()).await;
                break;
            }
        }
    }

    // Cleanup
    {
        let mut clients = clients.lock().await;
        clients.remove(&addr);
    }

    {
        let mut addr_to_username_guard = addr_to_username.lock().await;
        if let Some(username) = addr_to_username_guard.remove(&addr) {
            let mut usernames_guard = usernames.lock().await;
            usernames_guard.remove(&username);
        }
    }

    {
        let mut metrics_guard = metrics.lock().await;
        metrics_guard.active_connections -= 1;
    }

    println!("Client disconnected: {}", addr);
    Ok(())
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpStream;

    #[tokio::test]
    async fn test_list_command() {
        tokio::spawn(async {
            main().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut client1 = TcpStream::connect("127.0.0.1:8080").await.unwrap();
        let mut client2 = TcpStream::connect("127.0.0.1:8080").await.unwrap();

        client1.write_all(b"/name Alice\n").await.unwrap();
        client2.write_all(b"/name Bob\n").await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        client1.write_all(b"/list\n").await.unwrap();

        let (reader, _) = client1.into_split();
        let mut reader = BufReader::new(reader).lines();

        // Skip the username confirmation
        reader.next_line().await.unwrap();

        let response = reader.next_line().await.unwrap().unwrap();
        assert!(response.contains("Alice"));
        assert!(response.contains("Bob"));
    }

    #[tokio::test]
    async fn test_whisper_command() {
        tokio::spawn(async {
            main().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut client1 = TcpStream::connect("127.0.0.1:8080").await.unwrap();
        let mut client2 = TcpStream::connect("127.0.0.1:8080").await.unwrap();

        client1.write_all(b"/name Alice\n").await.unwrap();
        client2.write_all(b"/name Bob\n").await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        client1.write_all(b"/whisper Bob Secret message\n").await.unwrap();

        let (reader, _) = client2.into_split();
        let mut reader = BufReader::new(reader).lines();

        // Skip username confirmation
        reader.next_line().await.unwrap();

        let whisper = reader.next_line().await.unwrap().unwrap();
        assert!(whisper.contains("Whisper from Alice"));
        assert!(whisper.contains("Secret message"));
    }

    #[tokio::test]
    async fn test_quit_command() {
        tokio::spawn(async {
            main().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut client = TcpStream::connect("127.0.0.1:8080").await.unwrap();
        client.write_all(b"/quit\n").await.unwrap();

        let (reader, _) = client.into_split();
        let mut reader = BufReader::new(reader).lines();

        let goodbye = reader.next_line().await.unwrap().unwrap();
        assert_eq!(goodbye, "Goodbye!");
    }

    #[tokio::test]
    async fn test_stats_command() {
        tokio::spawn(async {
            main().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut client = TcpStream::connect("127.0.0.1:8080").await.unwrap();
        client.write_all(b"Hello\n").await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        client.write_all(b"/stats\n").await.unwrap();

        let (reader, _) = client.into_split();
        let mut reader = BufReader::new(reader).lines();

        let stats = reader.next_line().await.unwrap().unwrap();
        assert!(stats.contains("Server Statistics"));
        assert!(stats.contains("Total connections"));
        assert!(stats.contains("Active connections"));
    }
}
```

**Check Your Understanding**:
1. Why do we use `CancellationToken` instead of a simple `Arc<AtomicBool>` for shutdown?
2. How does the `/whisper` command differ from broadcast messages in terms of delivery?
3. What happens to active connections when the server shuts down gracefully?

---

## Summary

You've built a **production-grade chat server** with:

1. **Concurrent client handling** with `tokio::spawn`
2. **Broadcast messaging** with filtering to prevent echo
3. **Username registry** with duplicate prevention
4. **Backpressure handling** with bounded channels and `try_send`
5. **Idle timeout detection** to reclaim resources
6. **Commands**: `/name`, `/list`, `/whisper`, `/quit`, `/stats`
7. **Graceful shutdown** with `CancellationToken`
8. **Connection metrics** for monitoring

**Key Patterns Learned**:
- **Async broadcasting** with `tokio::sync::broadcast`
- **Per-client channels** for heterogeneous speeds
- **RAII cleanup** on client disconnect
- **Command routing** with string parsing
- **Timeout handling** with `tokio::select!`

**Performance Characteristics**:
- **Without backpressure**: Slow clients block everyone (10+ sec delays)
- **With bounded channels**: Fast clients maintain <10ms latency
- **Memory usage**: O(clients × channel_capacity) for buffered messages
- **CPU usage**: Minimal - event-driven architecture scales to 10k+ connections

These patterns apply to **WebSocket servers**, **pub/sub systems**, **multiplayer game servers**, and any **real-time application** with multiple concurrent clients.

**Next Steps**:
- Add TLS encryption with `tokio-rustls`
- Implement rate limiting per client
- Add persistent message history with SQLite
- Extend to WebSocket protocol for browser clients (Milestone 6)
- Implement rooms/channels for topic-based chat

---

## Milestone 6: WebSocket Protocol Support

**Goal**: Add WebSocket protocol support alongside TCP, enabling browser clients to connect to the chat server.

**Concepts**:
- WebSocket handshake (HTTP Upgrade)
- Frame parsing (opcodes, masking, fragmentation)
- `tokio-tungstenite` for WebSocket implementation
- Protocol negotiation (TCP vs WebSocket on same port)
- Building a browser-based WebSocket client

**Implementation Steps**:

1. **Add WebSocket dependencies**:
   - Add `tokio-tungstenite` and `futures-util` to `Cargo.toml`
   - `tokio-tungstenite` handles WebSocket handshake and framing

2. **Detect protocol on connection**:
   - Peek at first bytes of connection to detect HTTP GET (WebSocket handshake)
   - If HTTP GET detected, upgrade to WebSocket
   - Otherwise, treat as raw TCP (line-based protocol)

3. **Handle WebSocket handshake**:
   - Use `tokio_tungstenite::accept_async(stream)` to upgrade connection
   - This performs the HTTP 101 Switching Protocols handshake automatically

4. **Adapt message handling for WebSocket frames**:
   - WebSocket uses `Text` and `Binary` frames instead of lines
   - Extract text from `Message::Text` frames
   - Send responses as `Message::Text` frames

5. **Implement unified client handler**:
   - Create `enum ClientStream { Tcp(TcpStream), WebSocket(WebSocketStream)}`
   - Abstract reading/writing over both protocols
   - Reuse all existing chat logic (broadcast, commands, etc.)

6. **Create browser WebSocket client**:
   - Write an HTML/JavaScript client using the WebSocket API
   - Connect to `ws://127.0.0.1:8080`
   - Send/receive chat messages
   - Handle commands via UI buttons

**Starter Code**:

Add to `Cargo.toml`:
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.20"
futures-util = "0.3"
tokio-util = { version = "0.7", features = ["codec"] }
```

Server code:
```rust
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, AsyncReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio::time::{timeout, Duration};
use tokio_util::sync::CancellationToken;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message as WsMessage};
use futures_util::{StreamExt, SinkExt};
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
struct Message {
    sender: SocketAddr,
    content: String,
}

#[derive(Default)]
struct ConnectionMetrics {
    total_connections: u64,
    active_connections: u64,
    total_messages: u64,
}

type ClientMap = Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<String>>>>;
type UsernameMap = Arc<Mutex<HashMap<String, SocketAddr>>>;
type AddrToUsername = Arc<Mutex<HashMap<SocketAddr, String>>>;
type Metrics = Arc<Mutex<ConnectionMetrics>>;

const IDLE_TIMEOUT: Duration = Duration::from_secs(30);
const CHANNEL_CAPACITY: usize = 5;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");
    println!("WebSocket clients: ws://127.0.0.1:8080");
    println!("TCP clients: telnet 127.0.0.1 8080");

    let (broadcast_tx, _) = broadcast::channel::<Message>(100);
    let broadcast_tx = Arc::new(broadcast_tx);

    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));
    let usernames: UsernameMap = Arc::new(Mutex::new(HashMap::new()));
    let addr_to_username: AddrToUsername = Arc::new(Mutex::new(HashMap::new()));
    let metrics: Metrics = Arc::new(Mutex::new(ConnectionMetrics::default()));
    let cancel_token = CancellationToken::new();

    let cancel_token_clone = cancel_token.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        println!("\nShutting down server...");
        cancel_token_clone.cancel();
    });

    loop {
        tokio::select! {
            result = listener.accept() => {
                let (stream, addr) = result?;

                {
                    let mut metrics_guard = metrics.lock().await;
                    metrics_guard.total_connections += 1;
                    metrics_guard.active_connections += 1;
                }

                let broadcast_tx = Arc::clone(&broadcast_tx);
                let clients = Arc::clone(&clients);
                let usernames = Arc::clone(&usernames);
                let addr_to_username = Arc::clone(&addr_to_username);
                let metrics = Arc::clone(&metrics);
                let cancel_token = cancel_token.clone();

                tokio::spawn(async move {
                    // TODO: Detect protocol (WebSocket vs TCP)
                    if let Err(e) = handle_connection(
                        stream,
                        addr,
                        broadcast_tx,
                        clients,
                        usernames,
                        addr_to_username,
                        metrics,
                        cancel_token,
                    )
                    .await
                    {
                        eprintln!("Error handling connection {}: {}", addr, e);
                    }
                });
            }
            _ = cancel_token.cancelled() => {
                println!("Server shutdown complete");
                break;
            }
        }
    }

    Ok(())
}

// TODO: Implement protocol detection
async fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    broadcast_tx: Arc<broadcast::Sender<Message>>,
    clients: ClientMap,
    usernames: UsernameMap,
    addr_to_username: AddrToUsername,
    metrics: Metrics,
    cancel_token: CancellationToken,
) -> io::Result<()> {
    // Peek at first bytes to detect protocol
    let mut peek_buf = [0u8; 4];
    stream.peek(&mut peek_buf).await?;

    // Check for HTTP GET (WebSocket handshake starts with "GET ")
    if &peek_buf == b"GET " {
        println!("[{}] WebSocket connection detected", addr);

        // TODO: Accept WebSocket handshake
        match accept_async(stream).await {
            Ok(ws_stream) => {
                handle_websocket_client(
                    ws_stream,
                    addr,
                    broadcast_tx,
                    clients,
                    usernames,
                    addr_to_username,
                    metrics,
                    cancel_token,
                )
                .await
            }
            Err(e) => {
                eprintln!("[{}] WebSocket handshake failed: {}", addr, e);
                Ok(())
            }
        }
    } else {
        println!("[{}] TCP connection detected", addr);

        // Handle as regular TCP client (existing implementation)
        handle_tcp_client(
            stream,
            addr,
            broadcast_tx,
            clients,
            usernames,
            addr_to_username,
            metrics,
            cancel_token,
        )
        .await
    }
}

// TODO: Implement WebSocket client handler
async fn handle_websocket_client(
    ws_stream: tokio_tungstenite::WebSocketStream<TcpStream>,
    addr: SocketAddr,
    broadcast_tx: Arc<broadcast::Sender<Message>>,
    clients: ClientMap,
    usernames: UsernameMap,
    addr_to_username: AddrToUsername,
    metrics: Metrics,
    cancel_token: CancellationToken,
) -> io::Result<()> {
    println!("WebSocket client connected: {}", addr);

    let (mut ws_writer, mut ws_reader) = ws_stream.split();

    let (client_tx, mut client_rx) = mpsc::channel::<String>(CHANNEL_CAPACITY);

    {
        let mut clients = clients.lock().await;
        clients.insert(addr, client_tx.clone());
    }

    let mut broadcast_rx = broadcast_tx.subscribe();

    // Forward broadcast messages to WebSocket
    let client_tx_clone = client_tx.clone();
    tokio::spawn(async move {
        let mut dropped_count = 0;

        while let Ok(msg) = broadcast_rx.recv().await {
            if msg.sender == addr {
                continue;
            }

            match client_tx_clone.try_send(msg.content) {
                Ok(_) => {
                    if dropped_count > 0 {
                        println!("[{}] Recovered, dropped {} messages", addr, dropped_count);
                        dropped_count = 0;
                    }
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    dropped_count += 1;
                    if dropped_count % 10 == 0 {
                        eprintln!("[{}] Slow client, dropped {} messages", addr, dropped_count);
                    }
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    break;
                }
            }
        }
    });

    // Write messages to WebSocket
    tokio::spawn(async move {
        while let Some(msg) = client_rx.recv().await {
            // TODO: Send as WebSocket text frame
            if let Err(e) = ws_writer.send(WsMessage::Text(msg)).await {
                eprintln!("Error writing to WebSocket {}: {}", addr, e);
                break;
            }
        }
    });

    // Read WebSocket frames and process commands/messages
    loop {
        tokio::select! {
            result = timeout(IDLE_TIMEOUT, ws_reader.next()) => {
                match result {
                    Ok(Some(Ok(msg))) => {
                        // Update metrics
                        {
                            let mut metrics_guard = metrics.lock().await;
                            metrics_guard.total_messages += 1;
                        }

                        // TODO: Extract text from WebSocket message
                        let line = match msg {
                            WsMessage::Text(text) => text,
                            WsMessage::Close(_) => {
                                println!("[{}] WebSocket close frame received", addr);
                                break;
                            }
                            WsMessage::Ping(data) => {
                                // Respond to ping with pong
                                continue;
                            }
                            _ => continue, // Ignore other frame types
                        };

                        // Process commands (same as TCP)
                        if line == "/list" {
                            let user_list = {
                                let addr_to_username_guard = addr_to_username.lock().await;
                                let users: Vec<String> = addr_to_username_guard
                                    .iter()
                                    .map(|(addr, name)| format!("{} ({})", name, addr))
                                    .collect();

                                if users.is_empty() {
                                    "No users online".to_string()
                                } else {
                                    format!("Users online:\n{}", users.join("\n"))
                                }
                            };
                            let _ = client_tx.send(user_list).await;
                            continue;
                        }

                        if line.starts_with("/whisper ") {
                            let parts: Vec<&str> = line[9..].splitn(2, ' ').collect();
                            if parts.len() < 2 {
                                let _ = client_tx.send("Usage: /whisper <username> <message>".to_string()).await;
                                continue;
                            }

                            let target_username = parts[0];
                            let message = parts[1];

                            let target_addr = {
                                let usernames_guard = usernames.lock().await;
                                usernames_guard.get(target_username).copied()
                            };

                            match target_addr {
                                Some(target_addr) => {
                                    let sender_name = {
                                        let addr_to_username_guard = addr_to_username.lock().await;
                                        addr_to_username_guard
                                            .get(&addr)
                                            .cloned()
                                            .unwrap_or_else(|| addr.to_string())
                                    };

                                    let whisper_msg = format!("[Whisper from {}]: {}", sender_name, message);

                                    let clients_guard = clients.lock().await;
                                    if let Some(target_tx) = clients_guard.get(&target_addr) {
                                        let _ = target_tx.send(whisper_msg).await;
                                        let _ = client_tx.send(format!("[Whisper to {}]: {}", target_username, message)).await;
                                    }
                                }
                                None => {
                                    let _ = client_tx.send(format!("User '{}' not found", target_username)).await;
                                }
                            }
                            continue;
                        }

                        if line == "/stats" {
                            let stats = {
                                let metrics_guard = metrics.lock().await;
                                format!(
                                    "Server Statistics:\nTotal connections: {}\nActive connections: {}\nTotal messages: {}",
                                    metrics_guard.total_connections,
                                    metrics_guard.active_connections,
                                    metrics_guard.total_messages
                                )
                            };
                            let _ = client_tx.send(stats).await;
                            continue;
                        }

                        if line == "/quit" {
                            let _ = client_tx.send("Goodbye!".to_string()).await;
                            break;
                        }

                        if line.starts_with("/name ") {
                            let username = line[6..].trim().to_string();

                            let mut usernames_guard = usernames.lock().await;
                            if usernames_guard.contains_key(&username) {
                                let _ = client_tx.send(format!("Error: Username '{}' is already taken", username)).await;
                                continue;
                            }

                            usernames_guard.insert(username.clone(), addr);
                            drop(usernames_guard);

                            let mut addr_to_username_guard = addr_to_username.lock().await;
                            addr_to_username_guard.insert(addr, username.clone());
                            drop(addr_to_username_guard);

                            let _ = client_tx.send(format!("Username set to '{}'", username)).await;
                            continue;
                        }

                        // Regular message - broadcast
                        let sender_name = {
                            let addr_to_username_guard = addr_to_username.lock().await;
                            addr_to_username_guard
                                .get(&addr)
                                .cloned()
                                .unwrap_or_else(|| addr.to_string())
                        };

                        let message = Message {
                            sender: addr,
                            content: format!("[{}]: {}", sender_name, line),
                        };

                        let _ = broadcast_tx.send(message);
                    }
                    Ok(Some(Err(e))) => {
                        eprintln!("WebSocket error from {}: {}", addr, e);
                        break;
                    }
                    Ok(None) => {
                        // WebSocket stream closed
                        break;
                    }
                    Err(_) => {
                        println!("[{}] Idle timeout, disconnecting", addr);
                        let _ = client_tx.send("Disconnected due to inactivity".to_string()).await;
                        break;
                    }
                }
            }
            _ = cancel_token.cancelled() => {
                println!("[{}] Server shutting down", addr);
                let _ = client_tx.send("Server is shutting down".to_string()).await;
                break;
            }
        }
    }

    // Cleanup
    {
        let mut clients = clients.lock().await;
        clients.remove(&addr);
    }

    {
        let mut addr_to_username_guard = addr_to_username.lock().await;
        if let Some(username) = addr_to_username_guard.remove(&addr) {
            let mut usernames_guard = usernames.lock().await;
            usernames_guard.remove(&username);
        }
    }

    {
        let mut metrics_guard = metrics.lock().await;
        metrics_guard.active_connections -= 1;
    }

    println!("WebSocket client disconnected: {}", addr);
    Ok(())
}

// Existing TCP client handler (from Milestone 5)
async fn handle_tcp_client(
    stream: TcpStream,
    addr: SocketAddr,
    broadcast_tx: Arc<broadcast::Sender<Message>>,
    clients: ClientMap,
    usernames: UsernameMap,
    addr_to_username: AddrToUsername,
    metrics: Metrics,
    cancel_token: CancellationToken,
) -> io::Result<()> {
    // Same implementation as Milestone 5's handle_client function
    // (Copy the entire handle_client function from Milestone 5 here)
    println!("TCP client connected: {}", addr);

    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut writer = writer;

    let (client_tx, mut client_rx) = mpsc::channel::<String>(CHANNEL_CAPACITY);

    {
        let mut clients = clients.lock().await;
        clients.insert(addr, client_tx.clone());
    }

    let mut broadcast_rx = broadcast_tx.subscribe();

    let client_tx_clone = client_tx.clone();
    tokio::spawn(async move {
        let mut dropped_count = 0;

        while let Ok(msg) = broadcast_rx.recv().await {
            if msg.sender == addr {
                continue;
            }

            match client_tx_clone.try_send(msg.content) {
                Ok(_) => {
                    if dropped_count > 0 {
                        println!("[{}] Recovered, dropped {} messages", addr, dropped_count);
                        dropped_count = 0;
                    }
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    dropped_count += 1;
                    if dropped_count % 10 == 0 {
                        eprintln!("[{}] Slow client, dropped {} messages", addr, dropped_count);
                    }
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    break;
                }
            }
        }
    });

    tokio::spawn(async move {
        while let Some(msg) = client_rx.recv().await {
            if let Err(e) = writer.write_all(msg.as_bytes()).await {
                eprintln!("Error writing to {}: {}", addr, e);
                break;
            }
            if let Err(e) = writer.write_all(b"\n").await {
                eprintln!("Error writing newline to {}: {}", addr, e);
                break;
            }
        }
    });

    let mut lines = reader.lines();
    loop {
        tokio::select! {
            result = timeout(IDLE_TIMEOUT, lines.next_line()) => {
                match result {
                    Ok(Ok(Some(line))) => {
                        {
                            let mut metrics_guard = metrics.lock().await;
                            metrics_guard.total_messages += 1;
                        }

                        if line == "/list" {
                            let user_list = {
                                let addr_to_username_guard = addr_to_username.lock().await;
                                let users: Vec<String> = addr_to_username_guard
                                    .iter()
                                    .map(|(addr, name)| format!("{} ({})", name, addr))
                                    .collect();

                                if users.is_empty() {
                                    "No users online".to_string()
                                } else {
                                    format!("Users online:\n{}", users.join("\n"))
                                }
                            };
                            let _ = client_tx.send(user_list).await;
                            continue;
                        }

                        if line.starts_with("/whisper ") {
                            let parts: Vec<&str> = line[9..].splitn(2, ' ').collect();
                            if parts.len() < 2 {
                                let _ = client_tx.send("Usage: /whisper <username> <message>".to_string()).await;
                                continue;
                            }

                            let target_username = parts[0];
                            let message = parts[1];

                            let target_addr = {
                                let usernames_guard = usernames.lock().await;
                                usernames_guard.get(target_username).copied()
                            };

                            match target_addr {
                                Some(target_addr) => {
                                    let sender_name = {
                                        let addr_to_username_guard = addr_to_username.lock().await;
                                        addr_to_username_guard
                                            .get(&addr)
                                            .cloned()
                                            .unwrap_or_else(|| addr.to_string())
                                    };

                                    let whisper_msg = format!("[Whisper from {}]: {}", sender_name, message);

                                    let clients_guard = clients.lock().await;
                                    if let Some(target_tx) = clients_guard.get(&target_addr) {
                                        let _ = target_tx.send(whisper_msg).await;
                                        let _ = client_tx.send(format!("[Whisper to {}]: {}", target_username, message)).await;
                                    }
                                }
                                None => {
                                    let _ = client_tx.send(format!("User '{}' not found", target_username)).await;
                                }
                            }
                            continue;
                        }

                        if line == "/stats" {
                            let stats = {
                                let metrics_guard = metrics.lock().await;
                                format!(
                                    "Server Statistics:\nTotal connections: {}\nActive connections: {}\nTotal messages: {}",
                                    metrics_guard.total_connections,
                                    metrics_guard.active_connections,
                                    metrics_guard.total_messages
                                )
                            };
                            let _ = client_tx.send(stats).await;
                            continue;
                        }

                        if line == "/quit" {
                            let _ = client_tx.send("Goodbye!".to_string()).await;
                            break;
                        }

                        if line.starts_with("/name ") {
                            let username = line[6..].trim().to_string();

                            let mut usernames_guard = usernames.lock().await;
                            if usernames_guard.contains_key(&username) {
                                let _ = client_tx.send(format!("Error: Username '{}' is already taken", username)).await;
                                continue;
                            }

                            usernames_guard.insert(username.clone(), addr);
                            drop(usernames_guard);

                            let mut addr_to_username_guard = addr_to_username.lock().await;
                            addr_to_username_guard.insert(addr, username.clone());
                            drop(addr_to_username_guard);

                            let _ = client_tx.send(format!("Username set to '{}'", username)).await;
                            continue;
                        }

                        let sender_name = {
                            let addr_to_username_guard = addr_to_username.lock().await;
                            addr_to_username_guard
                                .get(&addr)
                                .cloned()
                                .unwrap_or_else(|| addr.to_string())
                        };

                        let message = Message {
                            sender: addr,
                            content: format!("[{}]: {}", sender_name, line),
                        };

                        let _ = broadcast_tx.send(message);
                    }
                    Ok(Ok(None)) => {
                        break;
                    }
                    Ok(Err(e)) => {
                        eprintln!("Error reading from {}: {}", addr, e);
                        break;
                    }
                    Err(_) => {
                        println!("[{}] Idle timeout, disconnecting", addr);
                        let _ = client_tx.send("Disconnected due to inactivity".to_string()).await;
                        break;
                    }
                }
            }
            _ = cancel_token.cancelled() => {
                println!("[{}] Server shutting down", addr);
                let _ = client_tx.send("Server is shutting down".to_string()).await;
                break;
            }
        }
    }

    // Cleanup
    {
        let mut clients = clients.lock().await;
        clients.remove(&addr);
    }

    {
        let mut addr_to_username_guard = addr_to_username.lock().await;
        if let Some(username) = addr_to_username_guard.remove(&addr) {
            let mut usernames_guard = usernames.lock().await;
            usernames_guard.remove(&username);
        }
    }

    {
        let mut metrics_guard = metrics.lock().await;
        metrics_guard.active_connections -= 1;
    }

    println!("TCP client disconnected: {}", addr);
    Ok(())
}
```

**Browser WebSocket Client** (`chat-client.html`):

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Chat Client</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
        }
        #messages {
            border: 1px solid #ccc;
            height: 400px;
            overflow-y: scroll;
            padding: 10px;
            margin-bottom: 10px;
            background-color: #f9f9f9;
        }
        .message {
            margin: 5px 0;
        }
        .system {
            color: #666;
            font-style: italic;
        }
        #input-area {
            display: flex;
            gap: 10px;
        }
        #messageInput {
            flex: 1;
            padding: 10px;
        }
        button {
            padding: 10px 20px;
            cursor: pointer;
        }
        #controls {
            margin-bottom: 10px;
            display: flex;
            gap: 10px;
        }
        #controls input {
            padding: 5px;
        }
    </style>
</head>
<body>
    <h1>WebSocket Chat Client</h1>

    <div id="controls">
        <input type="text" id="usernameInput" placeholder="Enter username">
        <button onclick="setUsername()">Set Username</button>
        <button onclick="listUsers()">List Users</button>
        <button onclick="showStats()">Stats</button>
    </div>

    <div id="messages"></div>

    <div id="input-area">
        <input type="text" id="messageInput" placeholder="Type a message..." onkeypress="handleKeyPress(event)">
        <button onclick="sendMessage()">Send</button>
        <button onclick="disconnect()">Disconnect</button>
    </div>

    <script>
        // TODO: Connect to WebSocket server
        const ws = new WebSocket('ws://127.0.0.1:8080');

        ws.onopen = () => {
            addMessage('Connected to server', 'system');
        };

        ws.onmessage = (event) => {
            // TODO: Display received message
            addMessage(event.data, 'received');
        };

        ws.onerror = (error) => {
            addMessage('WebSocket error: ' + error, 'system');
        };

        ws.onclose = () => {
            addMessage('Disconnected from server', 'system');
        };

        function addMessage(text, className) {
            const messagesDiv = document.getElementById('messages');
            const messageDiv = document.createElement('div');
            messageDiv.className = `message ${className}`;
            messageDiv.textContent = text;
            messagesDiv.appendChild(messageDiv);
            messagesDiv.scrollTop = messagesDiv.scrollHeight;
        }

        function sendMessage() {
            const input = document.getElementById('messageInput');
            const message = input.value.trim();

            if (message && ws.readyState === WebSocket.OPEN) {
                // TODO: Send message via WebSocket
                ws.send(message);
                input.value = '';
            }
        }

        function setUsername() {
            const username = document.getElementById('usernameInput').value.trim();
            if (username) {
                ws.send('/name ' + username);
            }
        }

        function listUsers() {
            ws.send('/list');
        }

        function showStats() {
            ws.send('/stats');
        }

        function disconnect() {
            ws.send('/quit');
            ws.close();
        }

        function handleKeyPress(event) {
            if (event.key === 'Enter') {
                sendMessage();
            }
        }
    </script>
</body>
</html>
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
    use futures_util::{StreamExt, SinkExt};

    #[tokio::test]
    async fn test_websocket_connection() {
        tokio::spawn(async {
            main().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let (ws_stream, _) = connect_async("ws://127.0.0.1:8080")
            .await
            .expect("Failed to connect");

        let (mut write, mut read) = ws_stream.split();

        // Set username
        write.send(Message::Text("/name TestUser".to_string())).await.unwrap();

        let response = read.next().await.unwrap().unwrap();
        if let Message::Text(text) = response {
            assert!(text.contains("Username set"));
        }
    }

    #[tokio::test]
    async fn test_websocket_broadcast() {
        tokio::spawn(async {
            main().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let (ws1, _) = connect_async("ws://127.0.0.1:8080").await.unwrap();
        let (ws2, _) = connect_async("ws://127.0.0.1:8080").await.unwrap();

        let (mut write1, mut read1) = ws1.split();
        let (mut write2, mut read2) = ws2.split();

        // Client 1 sends message
        write1.send(Message::Text("Hello from WS1".to_string())).await.unwrap();

        // Client 2 should receive it
        let response = tokio::time::timeout(
            tokio::time::Duration::from_secs(1),
            read2.next()
        ).await.unwrap().unwrap().unwrap();

        if let Message::Text(text) = response {
            assert!(text.contains("Hello from WS1"));
        }
    }

    #[tokio::test]
    async fn test_mixed_tcp_websocket() {
        tokio::spawn(async {
            main().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Connect TCP client
        let mut tcp_client = TcpStream::connect("127.0.0.1:8080").await.unwrap();

        // Connect WebSocket client
        let (ws, _) = connect_async("ws://127.0.0.1:8080").await.unwrap();
        let (mut ws_write, mut ws_read) = ws.split();

        // TCP client sends message
        tcp_client.write_all(b"Hello from TCP\n").await.unwrap();

        // WebSocket client should receive it
        let response = tokio::time::timeout(
            tokio::time::Duration::from_secs(1),
            ws_read.next()
        ).await.unwrap().unwrap().unwrap();

        if let Message::Text(text) = response {
            assert!(text.contains("Hello from TCP"));
        }

        // WebSocket client sends message
        ws_write.send(Message::Text("Hello from WS".to_string())).await.unwrap();

        // TCP client should receive it
        let mut buf = vec![0u8; 100];
        let n = tokio::time::timeout(
            tokio::time::Duration::from_secs(1),
            tcp_client.read(&mut buf)
        ).await.unwrap().unwrap();

        let received = String::from_utf8_lossy(&buf[..n]);
        assert!(received.contains("Hello from WS"));
    }
}
```

**Check Your Understanding**:
1. How does the server detect whether a connection is WebSocket or TCP?
2. Why can WebSocket and TCP clients communicate seamlessly in the same chat room?
3. What are the advantages of WebSocket over raw TCP for browser-based clients?
4. How does the WebSocket handshake work (HTTP Upgrade process)?

**Key Concepts Learned**:
- **Protocol detection** by peeking at connection bytes
- **WebSocket handshake** (HTTP 101 Switching Protocols)
- **Frame-based messaging** vs line-based protocols
- **Unified message handling** across different transports
- **Browser WebSocket API** for real-time communication

**Performance Comparison**:
- **TCP (telnet)**: Requires line buffering, manual newlines
- **WebSocket**: Frame-based, automatic message boundaries
- **Browser support**: WebSocket only (no raw TCP in browsers)
- **Latency**: Similar for both (< 1ms local overhead)

This milestone demonstrates how to build **protocol-agnostic servers** that support multiple client types while reusing the same business logic!

---

## Bonus: Angular WebSocket Client

In addition to the vanilla HTML/JavaScript client, we've provided a full **Angular** implementation that demonstrates modern frontend architecture with TypeScript and reactive programming.

**Location**: `angular-chat-client/`

**Features**:
- **TypeScript** type safety with interfaces and enums
- **RxJS** reactive programming with Observables
- **Service-based architecture** (WebSocketService)
- **Dependency injection** for clean component design
- **Standalone components** (Angular 17+ pattern)
- **Automatic reconnection** with exponential backoff
- **Reactive connection status** with live updates
- **Message type detection** (system, received, sent, whisper, error)
- **Responsive design** with mobile support

**Project Structure**:
```
angular-chat-client/
├── src/
│   ├── app/
│   │   ├── models/
│   │   │   └── message.model.ts          # TypeScript interfaces and enums
│   │   ├── services/
│   │   │   └── websocket.service.ts      # WebSocket management service
│   │   ├── app.component.ts              # Main component logic
│   │   ├── app.component.html            # Component template
│   │   └── app.component.css             # Component styles
│   ├── main.ts                            # Application bootstrap
│   ├── index.html                         # HTML entry point
│   └── styles.css                         # Global styles
├── angular.json                           # Angular CLI configuration
├── tsconfig.json                          # TypeScript configuration
├── package.json                           # Dependencies
└── README.md                              # Setup instructions
```

**Setup and Run**:

```bash
# Navigate to Angular project
cd angular-chat-client

# Install dependencies
npm install

# Start development server
ng serve

# Open browser to http://localhost:4200
```

**Key Implementation Highlights**:

**1. WebSocket Service** (`websocket.service.ts`):
```typescript
@Injectable({ providedIn: 'root' })
export class WebSocketService {
  private socket: WebSocket | null = null;
  private messagesSubject = new Subject<ChatMessage>();
  private connectionStatusSubject = new BehaviorSubject<ConnectionStatus>(
    ConnectionStatus.Disconnected
  );

  public messages$: Observable<ChatMessage> = this.messagesSubject.asObservable();
  public connectionStatus$: Observable<ConnectionStatus> =
    this.connectionStatusSubject.asObservable();

  connect(): void {
    this.socket = new WebSocket('ws://127.0.0.1:8080');
    // ... handle onopen, onmessage, onerror, onclose
  }

  send(message: string): void {
    if (this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(message);
    }
  }
}
```

**2. Reactive Component** (`app.component.ts`):
```typescript
export class AppComponent implements OnInit, OnDestroy {
  messages: ChatMessage[] = [];
  connectionStatus = ConnectionStatus.Disconnected;

  ngOnInit(): void {
    // Subscribe to messages stream
    this.wsService.messages$.subscribe(message => {
      this.messages.push(message);
    });

    // Subscribe to connection status stream
    this.wsService.connectionStatus$.subscribe(status => {
      this.connectionStatus = status;
    });
  }
}
```

**3. Type-Safe Models** (`message.model.ts`):
```typescript
export interface ChatMessage {
  content: string;
  type: MessageType;
  timestamp: Date;
}

export enum MessageType {
  System = 'system',
  Received = 'received',
  Sent = 'sent',
  Whisper = 'whisper',
  Error = 'error'
}
```

**Architecture Benefits**:
- **Separation of Concerns**: Service handles WebSocket logic, component handles UI
- **Reactive Streams**: Observable-based state management with RxJS
- **Type Safety**: Compile-time checks prevent runtime errors
- **Testability**: Injectable services make unit testing straightforward
- **Scalability**: Easy to extend with additional features (authentication, rooms, etc.)

**Comparison: Vanilla JS vs Angular**:

| Feature | Vanilla JS | Angular |
|---------|-----------|---------|
| **Type Safety** | Runtime only | Compile-time |
| **State Management** | Manual | RxJS Observables |
| **Code Organization** | Single file | Service/Component pattern |
| **Dependency Injection** | Manual | Built-in |
| **Auto-reconnect** | Basic | Exponential backoff |
| **Testability** | Limited | Full unit testing |
| **Bundle Size** | ~5KB | ~150KB (minified) |
| **Learning Curve** | Low | Medium-High |

**When to Use Each**:
- **Vanilla JS**: Quick prototypes, minimal dependencies, simple use cases
- **Angular**: Production apps, team collaboration, complex state management, enterprise features

This demonstrates how the **same WebSocket protocol** works seamlessly with both simple and complex frontend architectures!
