# Network Programming

TCP Server/Client Patterns

- Problem: Need reliable bidirectional communication; handle multiple concurrent connections; echo server blocks on one client; thread-per-connection doesn't scale
- Solution: Async TcpListener/TcpStream with tokio; spawn task per connection; BufReader for line-based protocols; graceful shutdown with CancellationToken
- Why It Matters: TCP foundation for HTTP/SSH/FTP; async handles 10K+ connections; thread-per-connection hits OS limits (~1K threads); essential for servers
- Use Cases: Chat servers, game servers, database protocols, custom TCP protocols, proxy servers, load balancers, monitoring agents

UDP Patterns

- Problem: Need low-latency connectionless communication; broadcasting; multicast; TCP overhead too high; no connection setup
- Solution: UdpSocket::bind() for server, send_to()/recv_from() for datagrams; set_broadcast()/join_multicast() for group communication; handle out-of-order/loss
- Why It Matters: Lower latency than TCP (no handshake/ack); essential for gaming, VoIP, video streaming; multicast for discovery; DNS uses UDP
- Use Cases: Gaming (position updates), VoIP, video streaming, DNS queries, service discovery, IoT sensors, time sync (NTP), multicast notifications

HTTP Client (reqwest)

- Problem: Need HTTP requests with async; handle cookies/headers/redirects; connection pooling; timeout management; JSON/form data; TLS verification
- Solution: reqwest::Client with connection pool; async/await API; automatic JSON (de)serialization with serde; cookie_store() for sessions; timeout()
- Why It Matters: HTTP ubiquitous for APIs; reqwest production-ready (connection pooling, retries); 10x easier than manual HTTP; essential for microservices
- Use Cases: REST API clients, web scraping, microservice communication, webhook consumers, OAuth flows, file downloads, GraphQL clients, API testing

HTTP Server (axum, actix-web)

- Problem: Need HTTP server with routing, middleware, state; handle concurrent requests; parse JSON/forms; websocket upgrade; graceful shutdown
- Solution: axum Router with handlers; State for shared data; extractors (Json, Query, Path); middleware (auth, logging); tower-http for CORS/compression
- Why It Matters: Web servers core infrastructure; axum built on tokio (hyper); handles 100K+ req/s; type-safe extractors prevent bugs; ecosystem mature
- Use Cases: REST APIs, web applications, microservices, GraphQL servers, webhook receivers, admin dashboards, file servers, proxy/gateway

WebSocket Patterns

- Problem: Need full-duplex real-time communication; HTTP request/response inadequate; long-polling wasteful; need bidirectional push; low latency
- Solution: tokio-tungstenite for WebSocket; split into read/write halves; async message handling; ping/pong for keepalive; graceful close
- Why It Matters: Real-time apps require bidirectional push; WebSocket persistent connection avoids HTTP overhead; 1 connection vs polling every 100ms
- Use Cases: Chat applications, live notifications, collaborative editing, stock tickers, gaming, dashboard updates, IoT control, video streaming signaling


This chapter covers network programming patterns—TCP/UDP for low-level protocols, HTTP client/server for web services, WebSocket for real-time bidirectional communication. Rust's async ecosystem enables high-performance, concurrent network applications with safety guarantees.

## Table of Contents

1. [TCP Server/Client Patterns](#pattern-1-tcp-serverclient-patterns)
2. [UDP Patterns](#pattern-2-udp-patterns)
3. [HTTP Client (reqwest)](#pattern-3-http-client-reqwest)
4. [HTTP Server (axum, actix-web)](#pattern-4-http-server-axum-actix-web)
5. [WebSocket Patterns](#pattern-5-websocket-patterns)

---

## Pattern 1: TCP Server/Client Patterns

**Problem**: Need reliable bidirectional communication between client and server. Simple echo server handles one client at time (blocks). Thread-per-connection model doesn't scale—1K threads hits OS limits, each consumes 2MB stack. Need concurrent handling of 10K+ connections. Graceful shutdown complex. Line-based protocols need buffering. Connection errors must be handled gracefully.

**Solution**: Use tokio's async TcpListener and TcpStream. tokio::spawn() spawns task per connection. BufReader for line-based protocols (efficient buffering). select! for graceful shutdown. Connection pooling on client side. Non-blocking I/O allows single thread to handle many connections. CancellationToken coordinates shutdown. Error handling with Result propagation.

**Why It Matters**: TCP foundation for HTTP, SSH, FTP, databases—essential protocol. Async I/O solves C10K problem (10K concurrent connections). Thread-per-connection limited to ~1K clients (stack memory exhaustion). Production servers need 10K+ concurrent connections. Incorrect shutdown causes connection leaks. Buffering critical for performance (avoid byte-by-byte reads).

**Use Cases**: Chat servers (persistent connections per user), game servers (player connections), database protocols (Postgres, Redis), custom TCP protocols, proxy servers, load balancers, monitoring agents, message brokers, SSH servers.

### Async TCP Server Pattern

**Problem**: Handle thousands of concurrent TCP connections efficiently.

```rust
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, BufReader, BufRead};
use std::thread;

/// A basic echo server that handles one client at a time
/// This is synchronous and will block on each operation
fn simple_echo_server() -> std::io::Result<()> {
    // Bind to localhost on port 8080
    // The "0.0.0.0:8080" address means "listen on all network interfaces"
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server listening on port 8080");

    // Accept connections in a loop
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection from: {}", stream.peer_addr()?);
                handle_client(stream)?;
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024];

    loop {
        // Read data from the client
        let bytes_read = stream.read(&mut buffer)?;

        // If bytes_read is 0, the client has disconnected
        if bytes_read == 0 {
            println!("Client disconnected");
            break;
        }

        // Echo the data back to the client
        stream.write_all(&buffer[..bytes_read])?;
        stream.flush()?;
    }

    Ok(())
}
```

This simple server has a critical limitation: it can only handle one client at a time. While one client is connected, other clients attempting to connect will have to wait. For production use, we need concurrent handling.

### Multi-threaded TCP Server

To handle multiple clients simultaneously, we can spawn a thread for each connection. This allows the server to accept new connections while existing connections are being serviced:

```rust
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

/// Multi-threaded server that spawns a thread per client
/// This scales better but can exhaust system resources with many connections
fn multithreaded_server() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Multi-threaded server listening on port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Spawn a new thread for each connection
                thread::spawn(move || {
                    if let Err(e) = handle_client_thread(stream) {
                        eprintln!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client_thread(mut stream: TcpStream) -> std::io::Result<()> {
    let addr = stream.peer_addr()?;
    println!("Thread handling client: {}", addr);

    let mut buffer = [0; 1024];

    loop {
        let bytes_read = stream.read(&mut buffer)?;

        if bytes_read == 0 {
            println!("Client {} disconnected", addr);
            break;
        }

        // Echo back
        stream.write_all(&buffer[..bytes_read])?;
    }

    Ok(())
}
```

While this works well for moderate numbers of clients, each thread consumes system resources. For high-concurrency scenarios, async I/O is more efficient.

### Async TCP Server with Tokio

Modern Rust networking typically uses async/await with Tokio. This allows handling thousands of concurrent connections efficiently because tasks are much lighter than threads:

```rust
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Async echo server using Tokio
/// Can handle thousands of concurrent connections efficiently
#[tokio::main]
async fn async_echo_server() -> tokio::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Async server listening on port 8080");

    loop {
        // Accept is async - other tasks can run while waiting
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);

        // Spawn an async task for this connection
        // Tasks are much cheaper than threads
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket).await {
                eprintln!("Error handling {}: {}", addr, e);
            }
        });
    }
}

async fn handle_connection(mut socket: TcpStream) -> tokio::io::Result<()> {
    let mut buffer = vec![0; 1024];

    loop {
        // Async read - yields to other tasks while waiting for data
        let n = socket.read(&mut buffer).await?;

        if n == 0 {
            // Connection closed
            return Ok(());
        }

        // Echo the data back
        socket.write_all(&buffer[..n]).await?;
    }
}
```

The key advantage here is that `await` points yield control to the runtime, allowing other tasks to make progress. This means one slow client doesn't block others.

### Line-based Protocol Server

Many protocols are line-based (like HTTP, SMTP, FTP). Here's a server that reads line by line, which is more realistic than raw bytes:

```rust
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// A line-based protocol server
/// Useful for protocols like SMTP, FTP, or custom text protocols
async fn line_based_server() -> tokio::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Line-based server listening on port 8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Connection from {}", addr);

        tokio::spawn(async move {
            // BufReader provides efficient buffered reading
            let (reader, mut writer) = socket.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                line.clear();

                // Read until we get a newline
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        // EOF - connection closed
                        println!("Client {} disconnected", addr);
                        break;
                    }
                    Ok(_) => {
                        // Process the line
                        let response = process_command(&line);

                        // Send response
                        if let Err(e) = writer.write_all(response.as_bytes()).await {
                            eprintln!("Failed to write to {}: {}", addr, e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading from {}: {}", addr, e);
                        break;
                    }
                }
            }
        });
    }
}

fn process_command(line: &str) -> String {
    let line = line.trim();

    match line.to_uppercase().as_str() {
        "HELLO" => "WORLD\n".to_string(),
        "QUIT" => "BYE\n".to_string(),
        _ => format!("ECHO: {}\n", line),
    }
}
```

This pattern is extremely common in network programming. By reading line-by-line, you can implement simple command-response protocols efficiently.

### TCP Client

Now let's look at the client side. A TCP client connects to a server and exchanges data:

```rust
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Connect to a server and exchange messages
async fn tcp_client_example() -> tokio::io::Result<()> {
    // Connect to the server
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Connected to server");

    // Send a message
    let message = "Hello, Server!\n";
    stream.write_all(message.as_bytes()).await?;

    // Read the response
    let mut buffer = vec![0; 1024];
    let n = stream.read(&mut buffer).await?;

    println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));

    Ok(())
}
```

For more complex clients, you might want to separate reading and writing:

```rust
use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Interactive client that can send and receive concurrently
async fn interactive_client() -> tokio::io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Spawn a task to handle incoming messages
    let read_handle = tokio::spawn(async move {
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // Connection closed
                Ok(_) => print!("Server: {}", line),
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    break;
                }
            }
        }
    });

    // Main task handles user input and sending
    let write_handle = tokio::spawn(async move {
        use tokio::io::{stdin, AsyncBufReadExt, BufReader};

        let stdin = BufReader::new(stdin());
        let mut lines = stdin.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if writer.write_all(format!("{}\n", line).as_bytes()).await.is_err() {
                break;
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = read_handle => println!("Read task finished"),
        _ = write_handle => println!("Write task finished"),
    }

    Ok(())
}
```

This pattern—splitting reading and writing into separate tasks—is very powerful for building responsive network clients.

### Connection Pooling

For clients that make many connections to the same server, connection pooling can significantly improve performance:

```rust
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::VecDeque;

/// Simple connection pool for reusing TCP connections
/// In production, use libraries like deadpool or bb8
struct ConnectionPool {
    available: Arc<Mutex<VecDeque<TcpStream>>>,
    address: String,
    max_size: usize,
}

impl ConnectionPool {
    fn new(address: String, max_size: usize) -> Self {
        ConnectionPool {
            available: Arc::new(Mutex::new(VecDeque::new())),
            address,
            max_size,
        }
    }

    async fn acquire(&self) -> tokio::io::Result<PooledConnection> {
        let mut pool = self.available.lock().await;

        // Try to reuse an existing connection
        if let Some(stream) = pool.pop_front() {
            return Ok(PooledConnection {
                stream: Some(stream),
                pool: self.available.clone(),
            });
        }

        // Otherwise create a new connection
        drop(pool); // Release lock before async operation
        let stream = TcpStream::connect(&self.address).await?;

        Ok(PooledConnection {
            stream: Some(stream),
            pool: self.available.clone(),
        })
    }
}

/// RAII wrapper that returns connection to pool on drop
struct PooledConnection {
    stream: Option<TcpStream>,
    pool: Arc<Mutex<VecDeque<TcpStream>>>,
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(stream) = self.stream.take() {
            let pool = self.pool.clone();
            tokio::spawn(async move {
                pool.lock().await.push_back(stream);
            });
        }
    }
}

impl std::ops::Deref for PooledConnection {
    type Target = TcpStream;

    fn deref(&self) -> &Self::Target {
        self.stream.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for PooledConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.stream.as_mut().unwrap()
    }
}
```

## Pattern 2: UDP Patterns

**Problem**: Need low-latency connectionless communication where packet loss acceptable. TCP handshake/ack overhead too high for real-time data. Broadcasting to multiple receivers simultaneously. Multicast group communication. Service discovery. No connection setup needed—just send datagrams. Message boundaries important (TCP is stream, UDP is messages).

**Solution**: Use tokio::net::UdpSocket. bind() on server. send_to()/recv_from() for datagrams (includes sender address). set_broadcast(true) for broadcast. join_multicast_v4()/v6() for multicast groups. Handle out-of-order delivery and loss at application layer. Fixed-size buffers for receiving. No connection state to manage.

**Why It Matters**: Lower latency than TCP (no handshake, no acks)—critical for gaming, VoIP. Essential for real-time where latest data > old data (position updates). Multicast for efficient group communication (service discovery). DNS uses UDP (query/response). Connectionless simplifies some protocols. Broadcast for local network discovery. When reliability not needed, UDP more efficient.

**Use Cases**: Gaming (player position/state updates), VoIP (audio packets), video streaming (RTP), DNS queries, service discovery (mDNS, SSDP), IoT sensor data, time synchronization (NTP), multicast notifications, DHCP, TFTP.

### UDP Echo Server Pattern

**Problem**: Build simple UDP server that receives and responds to datagrams.

```rust
use tokio::net::UdpSocket;
use std::io;

/// UDP echo server
/// Receives datagrams and echoes them back to the sender
async fn udp_echo_server() -> io::Result<()> {
    // Bind to a port
    let socket = UdpSocket::bind("127.0.0.1:8080").await?;
    println!("UDP server listening on port 8080");

    let mut buffer = vec![0u8; 1024];

    loop {
        // Receive a datagram
        // recv_from returns the number of bytes and the sender's address
        let (len, addr) = socket.recv_from(&mut buffer).await?;

        println!("Received {} bytes from {}", len, addr);

        // Echo it back
        socket.send_to(&buffer[..len], addr).await?;
    }
}
```

Notice how much simpler this is than TCP—no connection management, no accept loop. Each packet is independent.

### UDP Client

```rust
use tokio::net::UdpSocket;

/// Send a UDP message and wait for a response
async fn udp_client_example() -> tokio::io::Result<()> {
    // Bind to any available port
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    // Connect sets the default destination
    // This doesn't establish a connection (UDP is connectionless)
    // but allows using send/recv instead of send_to/recv_from
    socket.connect("127.0.0.1:8080").await?;

    // Send a message
    let message = b"Hello, UDP Server!";
    socket.send(message).await?;

    // Wait for a response
    let mut buffer = vec![0u8; 1024];
    let len = socket.recv(&mut buffer).await?;

    println!("Received: {}", String::from_utf8_lossy(&buffer[..len]));

    Ok(())
}
```

### Broadcast and Multicast

UDP supports broadcasting to multiple recipients. This is useful for service discovery and distributed systems:

```rust
use tokio::net::UdpSocket;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Broadcast a message to all hosts on the local network
async fn udp_broadcast() -> tokio::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    // Enable broadcast
    socket.set_broadcast(true)?;

    // Broadcast address (255.255.255.255 reaches all hosts on local network)
    let broadcast_addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)),
        8080
    );

    let message = b"Service Discovery Request";
    socket.send_to(message, broadcast_addr).await?;

    println!("Broadcast sent");
    Ok(())
}

/// Listen for broadcast messages
async fn udp_broadcast_listener() -> tokio::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:8080").await?;
    socket.set_broadcast(true)?;

    let mut buffer = vec![0u8; 1024];

    loop {
        let (len, addr) = socket.recv_from(&mut buffer).await?;
        println!("Broadcast from {}: {}",
            addr,
            String::from_utf8_lossy(&buffer[..len])
        );
    }
}
```

### Reliable UDP Pattern

Sometimes you want UDP's low latency but need some reliability. Here's a simple request-response pattern with retries:

```rust
use tokio::net::UdpSocket;
use tokio::time::{timeout, Duration};

/// Send a UDP request with retries
async fn reliable_udp_request(
    socket: &UdpSocket,
    message: &[u8],
    server_addr: &str,
    retries: usize,
) -> tokio::io::Result<Vec<u8>> {
    let mut buffer = vec![0u8; 1024];

    for attempt in 0..retries {
        // Send the request
        socket.send_to(message, server_addr).await?;

        // Wait for response with timeout
        match timeout(Duration::from_secs(2), socket.recv_from(&mut buffer)).await {
            Ok(Ok((len, _addr))) => {
                // Success! Return the response
                return Ok(buffer[..len].to_vec());
            }
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                // Timeout - retry
                println!("Attempt {} timed out, retrying...", attempt + 1);
                continue;
            }
        }
    }

    Err(tokio::io::Error::new(
        tokio::io::ErrorKind::TimedOut,
        "All retry attempts failed"
    ))
}
```

This pattern is the basis for protocols like QUIC and helps bridge the gap between UDP's speed and TCP's reliability.

## Pattern 3: HTTP Client (reqwest)

**Problem**: Need to make HTTP requests with async I/O. Handle cookies, headers, redirects automatically. Connection pooling for performance. Timeout management. JSON/form data serialization. TLS certificate verification. OAuth flows. Rate limiting. Retry logic. Progress tracking for large downloads. Authentication schemes.

**Solution**: Use reqwest::Client with built-in connection pool. Async/await API. Automatic JSON (de)serialization with serde (json() method). cookie_store() for session management. timeout() for request deadlines. Client::builder() for configuration. multipart for file uploads. Middleware for auth/retries. Error handling with Result.

**Why It Matters**: HTTP ubiquitous for APIs—essential for microservices. reqwest production-ready (connection pooling, retries, cookie management). 10x easier than manual HTTP parsing. Connection pooling critical for performance (reuse connections). Async enables concurrent requests. TLS verification prevents MITM attacks. Industry standard crate.

**Use Cases**: REST API clients, web scraping, microservice communication, webhook consumers, OAuth authentication flows, file downloads, GraphQL clients, API testing/monitoring, service health checks, data synchronization.

### Basic HTTP Client Pattern

**Problem**: Make HTTP requests with JSON payloads efficiently.

```rust
use reqwest;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    message: String,
    status: String,
}

/// Simple GET request
async fn simple_get_request() -> Result<(), Box<dyn std::error::Error>> {
    // GET request returns a Response
    let response = reqwest::get("https://httpbin.org/get").await?;

    println!("Status: {}", response.status());
    println!("Headers: {:#?}", response.headers());

    // Read the response body as text
    let body = response.text().await?;
    println!("Body: {}", body);

    Ok(())
}

/// GET request with JSON deserialization
async fn get_json() -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::get("https://api.example.com/data")
        .await?
        .json::<ApiResponse>()
        .await?;

    println!("Response: {:?}", response);
    Ok(())
}
```

The `.json()` method automatically deserializes the response body using serde, making it very convenient for API interactions.

### POST Requests and Request Building

For more complex requests, use the `Client` and `RequestBuilder`:

```rust
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
struct CreateUser {
    username: String,
    email: String,
}

#[derive(Deserialize, Debug)]
struct User {
    id: u64,
    username: String,
    email: String,
}

/// POST request with JSON body
async fn create_user() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let new_user = CreateUser {
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    let response = client
        .post("https://api.example.com/users")
        .json(&new_user)
        .send()
        .await?;

    let created_user: User = response.json().await?;
    println!("Created user: {:?}", created_user);

    Ok(())
}

/// POST with form data
async fn post_form() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let mut form_data = HashMap::new();
    form_data.insert("username", "bob");
    form_data.insert("password", "secret123");

    let response = client
        .post("https://example.com/login")
        .form(&form_data)
        .send()
        .await?;

    println!("Login status: {}", response.status());
    Ok(())
}
```

### Request Headers and Authentication

Many APIs require authentication headers. Here's how to add them:

```rust
use reqwest::{Client, header};

/// Request with custom headers
async fn request_with_auth() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let response = client
        .get("https://api.example.com/protected")
        .header(header::AUTHORIZATION, "Bearer YOUR_API_TOKEN")
        .header(header::USER_AGENT, "MyApp/1.0")
        .send()
        .await?;

    println!("Response: {}", response.text().await?);
    Ok(())
}

/// Client with default headers
async fn client_with_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_static("Bearer YOUR_API_TOKEN")
    );
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json")
    );

    // Create a client with default headers
    let client = Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    // All requests with this client will include the default headers
    let response = client
        .get("https://api.example.com/data")
        .send()
        .await?;

    Ok(())
}
```

### Error Handling and Retries

Robust HTTP clients need proper error handling and retry logic:

```rust
use reqwest::{Client, StatusCode};
use tokio::time::{sleep, Duration};

/// Retry a request on failure
async fn request_with_retry(
    client: &Client,
    url: &str,
    max_retries: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut attempts = 0;

    loop {
        attempts += 1;

        match client.get(url).send().await {
            Ok(response) => {
                match response.status() {
                    StatusCode::OK => {
                        return Ok(response.text().await?);
                    }
                    StatusCode::TOO_MANY_REQUESTS => {
                        // Rate limited - wait and retry
                        if attempts >= max_retries {
                            return Err("Max retries exceeded".into());
                        }
                        println!("Rate limited, waiting...");
                        sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                    status if status.is_server_error() => {
                        // Server error - retry with backoff
                        if attempts >= max_retries {
                            return Err(format!("Server error: {}", status).into());
                        }
                        let backoff = Duration::from_secs(2u64.pow(attempts));
                        println!("Server error, retrying in {:?}", backoff);
                        sleep(backoff).await;
                        continue;
                    }
                    status => {
                        // Client error - don't retry
                        return Err(format!("HTTP error: {}", status).into());
                    }
                }
            }
            Err(e) => {
                if attempts >= max_retries {
                    return Err(e.into());
                }
                println!("Request failed: {}, retrying...", e);
                sleep(Duration::from_secs(2)).await;
                continue;
            }
        }
    }
}
```

This pattern—exponential backoff with retry limits—is essential for building resilient network clients.

### Downloading Files

For downloading large files, streaming the response is more memory-efficient than loading everything into memory:

```rust
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;

/// Download a file with progress tracking
async fn download_file(
    url: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client.get(url).send().await?;

    // Get the total file size
    let total_size = response.content_length().unwrap_or(0);
    println!("Downloading {} bytes", total_size);

    // Create output file
    let mut file = File::create(output_path).await?;
    let mut downloaded = 0u64;

    // Stream the response body
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;

        downloaded += chunk.len() as u64;
        if total_size > 0 {
            let percent = (downloaded as f64 / total_size as f64) * 100.0;
            print!("\rProgress: {:.2}%", percent);
        }
    }

    println!("\nDownload complete!");
    Ok(())
}
```

## Pattern 4: HTTP Server (axum, actix-web)

**Problem**: Need HTTP server with routing, middleware, shared state. Handle concurrent requests safely. Parse JSON/query/path parameters. Serve static files. WebSocket upgrade. CORS, authentication, logging middleware. Graceful shutdown. Form data handling. Error responses. Type-safe extractors to prevent runtime bugs.

**Solution**: Use axum Router for routing. State extractor for shared data (Arc for thread-safety). Extractors (Json<T>, Query<T>, Path<T>) with serde. Middleware tower layers (CORS, compression, logging). axum::serve() for async server. Typed errors with IntoResponse. Extension for request-scoped data. tower-http for common middleware.

**Why It Matters**: Web servers are core infrastructure—REST APIs, microservices, dashboards. axum built on tokio/hyper (100K+ req/s). Type-safe extractors catch errors at compile-time (wrong JSON schema = compile error). Middleware composable. Production-ready (graceful shutdown, backpressure). Ecosystem mature. Essential for modern web services.

**Use Cases**: REST APIs, web applications, microservices, GraphQL servers (with async-graphql), webhook receivers, admin dashboards, file upload services, proxy/API gateway, authentication services, monitoring endpoints.

### axum Server Pattern

**Problem**: Build REST API with routing, JSON, and shared state.

```rust
use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::{Path, Query},
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    username: String,
    email: String,
}

#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
    email: String,
}

#[derive(Deserialize)]
struct ListQuery {
    page: Option<u32>,
    per_page: Option<u32>,
}

#[tokio::main]
async fn basic_axum_server() {
    // Build our application with routes
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", get(get_user));

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

//==================
// Handler for GET /
//==================
async fn root_handler() -> &'static str {
    "Hello, World!"
}

//=======================
// Handler for GET /users
//=======================
async fn list_users(Query(params): Query<ListQuery>) -> Json<Vec<User>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(10);

    println!("Listing users: page={}, per_page={}", page, per_page);

    // In a real app, fetch from database
    let users = vec![
        User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        },
        User {
            id: 2,
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
        },
    ];

    Json(users)
}

//===========================
// Handler for GET /users/:id
//===========================
async fn get_user(Path(user_id): Path<u64>) -> Result<Json<User>, StatusCode> {
    println!("Getting user {}", user_id);

    // Simulate database lookup
    if user_id == 1 {
        Ok(Json(User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

//========================
// Handler for POST /users
//========================
async fn create_user(
    Json(payload): Json<CreateUserRequest>,
) -> (StatusCode, Json<User>) {
    println!("Creating user: {}", payload.username);

    // In a real app, save to database and return the created user
    let user = User {
        id: 42, // Would come from database
        username: payload.username,
        email: payload.email,
    };

    (StatusCode::CREATED, Json(user))
}
```

Notice how axum uses extractors (like `Path`, `Query`, `Json`) to parse and validate request data at compile time. If the types don't match, you get a compile error.

### Shared State in axum

Most applications need shared state (database connections, caches, etc.). axum makes this easy with the `State` extractor:

```rust
use axum::{
    routing::get,
    Router,
    extract::State,
    Json,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::Serialize;

#[derive(Clone)]
struct AppState {
    // Use Arc for shared ownership across tasks
    // RwLock allows multiple readers or one writer
    db: Arc<RwLock<Database>>,
    config: Arc<Config>,
}

struct Database {
    users: Vec<User>,
}

struct Config {
    max_users: usize,
}

#[derive(Serialize, Clone)]
struct User {
    id: u64,
    name: String,
}

#[tokio::main]
async fn stateful_server() {
    let state = AppState {
        db: Arc::new(RwLock::new(Database {
            users: vec![],
        })),
        config: Arc::new(Config {
            max_users: 1000,
        }),
    };

    let app = Router::new()
        .route("/users", get(get_all_users))
        .with_state(state);

    // Run server...
}

async fn get_all_users(
    State(state): State<AppState>,
) -> Json<Vec<User>> {
    // Acquire read lock
    let db = state.db.read().await;

    // Clone the users (in real app, might want to use pagination)
    Json(db.users.clone())
}
```

The `State` extractor ensures every handler has access to the application state without global variables.

### Middleware in axum

Middleware allows you to add cross-cutting concerns like logging, authentication, or CORS:

```rust
use axum::{
    Router,
    routing::get,
    middleware::{self, Next},
    response::Response,
    http::Request,
};
use std::time::Instant;

#[tokio::main]
async fn middleware_example() {
    let app = Router::new()
        .route("/", get(|| async { "Hello!" }))
        // Add middleware to all routes
        .layer(middleware::from_fn(timing_middleware))
        .layer(middleware::from_fn(auth_middleware));

    // Run server...
}

/// Middleware that logs request timing
async fn timing_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Response {
    let start = Instant::now();
    let uri = request.uri().clone();

    let response = next.run(request).await;

    let elapsed = start.elapsed();
    println!("{} took {:?}", uri, elapsed);

    response
}

/// Middleware that checks authentication
async fn auth_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Response {
    // Check for auth header
    if let Some(auth_header) = request.headers().get("authorization") {
        if auth_header.to_str().unwrap_or("").starts_with("Bearer ") {
            // Valid auth, continue
            return next.run(request).await;
        }
    }

    // No valid auth
    Response::builder()
        .status(401)
        .body("Unauthorized".into())
        .unwrap()
}
```

Middleware composes nicely, allowing you to build complex request processing pipelines.

### actix-web Server

actix-web is known for being extremely fast and feature-rich. It uses the actor model internally:

```rust
use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
}

#[actix_web::main]
async fn actix_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/users", web::get().to(get_users))
            .route("/users", web::post().to(create_user))
            .route("/users/{id}", web::get().to(get_user))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello from actix-web!")
}

async fn get_users() -> impl Responder {
    let users = vec![
        User { id: 1, name: "Alice".to_string() },
        User { id: 2, name: "Bob".to_string() },
    ];

    HttpResponse::Ok().json(users)
}

async fn create_user(user: web::Json<User>) -> impl Responder {
    println!("Creating user: {}", user.name);
    HttpResponse::Created().json(user.into_inner())
}

async fn get_user(path: web::Path<u64>) -> impl Responder {
    let user_id = path.into_inner();

    if user_id == 1 {
        HttpResponse::Ok().json(User {
            id: 1,
            name: "Alice".to_string(),
        })
    } else {
        HttpResponse::NotFound().finish()
    }
}
```

actix-web is slightly more imperative in style compared to axum's declarative approach, but both are excellent choices.

## Pattern 5: WebSocket Patterns

**Problem**: Need full-duplex real-time bidirectional communication. HTTP request-response inadequate for live updates. Long-polling wasteful (poll every 100ms). Server needs to push to clients without request. Chat, notifications, live dashboards require bidirectional push. Low-latency streaming. Persistent connection more efficient than repeated HTTP requests.

**Solution**: Use tokio-tungstenite for WebSocket (or axum/actix WebSocket support). HTTP upgrade to WebSocket. Split into read/write halves for concurrent send/receive. async message handling. Ping/pong for keepalive (detect dead connections). Graceful close with close frame. Broadcast pattern with tokio::sync::broadcast channel. Per-client state management.

**Why It Matters**: Real-time applications require bidirectional push—can't rely on polling. WebSocket single persistent connection vs HTTP polling overhead (100 req/s vs 1 connection). Essential for chat, live dashboards, collaborative editing. Lower latency than HTTP polling. Server-initiated push critical for notifications. Efficient for streaming data (stock prices, game state).

**Use Cases**: Chat applications (real-time messages), live notifications (alerts, updates), collaborative editing (Google Docs-style), stock tickers (price updates), gaming (multiplayer state sync), dashboard updates (metrics, logs), IoT device control, video streaming signaling.

### WebSocket Broadcast Pattern

**Problem**: Broadcast messages from any client to all connected WebSocket clients.

```rust
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

#[derive(Clone)]
struct AppState {
    // Broadcast channel for sending messages to all clients
    tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn websocket_server() {
    // Create broadcast channel
    let (tx, _rx) = broadcast::channel(100);

    let state = AppState { tx };

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(state);

    println!("WebSocket server running on http://127.0.0.1:3000");

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
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
                // Broadcast the message to all clients
                let _ = tx.send(text);
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
```

This creates a simple chat server where all messages are broadcast to all connected clients. Each client gets two tasks: one for receiving broadcasts and one for sending messages.

### WebSocket Client

Here's a WebSocket client using tokio-tungstenite:

```rust
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};

async fn websocket_client() -> Result<(), Box<dyn std::error::Error>> {
    let url = "ws://127.0.0.1:3000/ws";

    // Connect to the server
    let (ws_stream, _) = connect_async(url).await?;
    println!("Connected to {}", url);

    let (mut write, mut read) = ws_stream.split();

    // Spawn a task to handle incoming messages
    let read_handle = tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received: {}", text);
                }
                Ok(Message::Close(_)) => {
                    println!("Server closed connection");
                    break;
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Send some messages
    for i in 0..5 {
        let msg = format!("Message {}", i);
        write.send(Message::Text(msg)).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // Close the connection
    write.send(Message::Close(None)).await?;

    read_handle.await?;

    Ok(())
}
```

### Room-based WebSocket Pattern

For more complex applications like chat rooms or game lobbies, you need to manage multiple rooms:

```rust
use axum::extract::ws::{WebSocket, Message};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

type RoomId = String;
type UserId = String;

struct ChatServer {
    rooms: Arc<RwLock<HashMap<RoomId, Room>>>,
}

struct Room {
    // Broadcast channel for this room
    tx: broadcast::Sender<ChatMessage>,
    // Connected users
    users: HashMap<UserId, UserInfo>,
}

struct UserInfo {
    username: String,
}

#[derive(Clone)]
struct ChatMessage {
    user_id: UserId,
    username: String,
    content: String,
}

impl ChatServer {
    fn new() -> Self {
        ChatServer {
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn join_room(&self, room_id: RoomId, user_id: UserId, username: String) -> broadcast::Receiver<ChatMessage> {
        let mut rooms = self.rooms.write().await;

        let room = rooms.entry(room_id.clone()).or_insert_with(|| {
            let (tx, _) = broadcast::channel(100);
            Room {
                tx,
                users: HashMap::new(),
            }
        });

        room.users.insert(user_id, UserInfo { username });
        room.tx.subscribe()
    }

    async fn send_message(&self, room_id: &RoomId, msg: ChatMessage) {
        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(room_id) {
            let _ = room.tx.send(msg);
        }
    }

    async fn leave_room(&self, room_id: &RoomId, user_id: &UserId) {
        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.users.remove(user_id);

            // Clean up empty rooms
            if room.users.is_empty() {
                rooms.remove(room_id);
            }
        }
    }
}
```

This pattern allows users to join specific rooms and only receive messages from those rooms, which is much more scalable than broadcasting everything to everyone.

### Handling Ping/Pong for Keep-Alive

WebSocket connections can silently die (network issues, etc.). Use ping/pong to detect dead connections:

```rust
use tokio::time::{interval, Duration};
use axum::extract::ws::{WebSocket, Message};

async fn websocket_with_keepalive(mut socket: WebSocket) {
    let mut ping_interval = interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = ping_interval.tick() => {
                // Send ping
                if socket.send(Message::Ping(vec![])).await.is_err() {
                    break;
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Pong(_))) => {
                        // Client is alive
                    }
                    Some(Ok(Message::Text(text))) => {
                        // Handle message
                        println!("Received: {}", text);
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    Some(Err(e)) => {
                        eprintln!("Error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    println!("WebSocket connection closed");
}
```

This ensures you detect and clean up dead connections promptly.

---

## Summary

This chapter covered network programming patterns:

1. **TCP Server/Client**: Async TcpListener/TcpStream, tokio::spawn per connection, BufReader for protocols
2. **UDP Patterns**: UdpSocket, send_to/recv_from, broadcast/multicast, connectionless communication
3. **HTTP Client (reqwest)**: Client with connection pool, JSON serialization, cookies, timeouts, retries
4. **HTTP Server (axum)**: Router, extractors (Json, Query, Path), middleware, shared State, type-safe
5. **WebSocket Patterns**: Full-duplex real-time, split read/write, broadcast channel, ping/pong keepalive

**Key Takeaways**:
- Async I/O solves C10K problem—single thread handles 10K+ connections
- TCP reliable but higher latency; UDP fast but lossy
- reqwest industry standard for HTTP clients (connection pooling, retries)
- axum type-safe extractors prevent runtime bugs (compile-time validation)
- WebSocket enables server-initiated push (vs HTTP polling overhead)

**Protocol Selection**:
- **TCP**: Reliable ordered delivery (HTTP, SSH, databases)
- **UDP**: Low-latency real-time (gaming, VoIP, DNS)
- **HTTP**: Request-response APIs (REST, microservices)
- **WebSocket**: Bidirectional real-time (chat, live updates)

**Performance Patterns**:
- Connection pooling (reqwest Client, database pools)
- Buffering (BufReader for line protocols)
- Async spawning (tokio::spawn per connection)
- Graceful shutdown (CancellationToken)
- Backpressure (bounded channels)

**Common Patterns**:
```rust
// TCP async server
let listener = TcpListener::bind("127.0.0.1:8080").await?;
loop {
    let (stream, _) = listener.accept().await?;
    tokio::spawn(handle_connection(stream));
}

// HTTP client
let client = reqwest::Client::new();
let response = client.get("https://api.example.com")
    .timeout(Duration::from_secs(5))
    .send().await?;
let data: MyType = response.json().await?;

// axum server
let app = Router::new()
    .route("/", get(handler))
    .with_state(state);
axum::serve(listener, app).await?;
```

**Error Handling**:
- Network errors common—design for failure
- Timeouts essential (prevent hanging on dead connections)
- Graceful degradation (retry with backoff)
- Connection cleanup (tokio::select! for cancellation)
- Validate input (malicious clients)

**Best Practices**:
- Always set timeouts on network operations
- Handle connection errors gracefully
- Use connection pooling for performance
- Implement graceful shutdown
- Buffer I/O operations (avoid byte-by-byte)
- Validate and sanitize all input
- Use TLS for sensitive data

**Common Use Cases**:
- **Chat server**: WebSocket with broadcast channel
- **REST API**: axum with JSON extractors
- **Game server**: UDP for position updates + TCP for critical events
- **Microservices**: reqwest client + axum server
- **Proxy**: TCP forwarding with tokio
- **Live dashboard**: WebSocket for real-time metrics

**Tools**:
- **tokio**: Async runtime, TcpListener/TcpStream/UdpSocket
- **reqwest**: HTTP client with connection pooling
- **axum**: HTTP server framework (type-safe, tokio-based)
- **tokio-tungstenite**: WebSocket implementation
- **tower**: Middleware for HTTP services
- **hyper**: Low-level HTTP library (foundation for axum)
