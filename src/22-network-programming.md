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

## Overview
This chapter covers network programming patterns—TCP/UDP for low-level protocols, HTTP client/server for web services, WebSocket for real-time bidirectional communication. Rust's async ecosystem enables high-performance, concurrent network applications with safety guarantees.



## Pattern 1: TCP Server/Client Patterns

**Problem**: Need reliable bidirectional communication between client and server. Simple echo server handles one client at time (blocks). Thread-per-connection model doesn't scale—1K threads hits OS limits, each consumes 2MB stack. Need concurrent handling of 10K+ connections. Graceful shutdown complex. Line-based protocols need buffering. Connection errors must be handled gracefully.

**Solution**: Use tokio's async TcpListener and TcpStream. tokio::spawn() spawns task per connection. BufReader for line-based protocols (efficient buffering). select! for graceful shutdown. Connection pooling on client side. Non-blocking I/O allows single thread to handle many connections. CancellationToken coordinates shutdown. Error handling with Result propagation.

**Why It Matters**: TCP foundation for HTTP, SSH, FTP, databases—essential protocol. Async I/O solves C10K problem (10K concurrent connections). Thread-per-connection limited to ~1K clients (stack memory exhaustion). Production servers need 10K+ concurrent connections. Incorrect shutdown causes connection leaks. Buffering critical for performance (avoid byte-by-byte reads).

**Use Cases**: Chat servers (persistent connections per user), game servers (player connections), database protocols (Postgres, Redis), custom TCP protocols, proxy servers, load balancers, monitoring agents, message brokers, SSH servers.

### Example: Async TCP Server Pattern

Handle thousands of concurrent TCP connections efficiently.

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

### Example: Multi-threaded TCP Server

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

### Example: Async TCP Server with Tokio

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

### Example: Line-based Protocol Server

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

### Example: TCP Client

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

### Example: Connection Pooling

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

### Example: UDP Echo Server Pattern

 Build simple UDP server that receives and responds to datagrams.

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

### Example: UDP Client

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

### Example: Broadcast and Multicast

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

### Example: Reliable UDP Pattern

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

### Example: Basic HTTP Client Pattern

Make HTTP requests with JSON payloads efficiently.

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

### Example: POST Requests and Request Building

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

### Example: Request Headers and Authentication

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

### Example: Error Handling and Retries

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

### Example: Downloading Files

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

### Example: axum Server Pattern

Build REST API with routing, JSON, and shared state.

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

### Example: Shared State in axum

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

### Example: Middleware in axum

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

### Example: actix-web Server

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

### Example: WebSocket Broadcast Pattern

 Broadcast messages from any client to all connected WebSocket clients.

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

### Example: WebSocket Client

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

### Example: Room-based WebSocket Pattern

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

### Example: Handling Ping/Pong for Keep-Alive

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


## TCP Cheat Sheet
```rust
use std::net::{TcpListener, TcpStream, UdpSocket, SocketAddr, IpAddr, Ipv4Addr, Ipv6Addr};
use std::io::{Read, Write, BufReader, BufRead, Result, Error, ErrorKind};
use std::time::Duration;

// ===== TCP SERVER =====
// Basic TCP server
fn tcp_server_basic() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;    // Bind to address
    println!("Server listening on port 8080");
    
    for stream in listener.incoming() {                      // Accept connections
        let stream = stream?;
        println!("New connection: {}", stream.peer_addr()?);
        handle_client(stream)?;
    }
    
    Ok(())
}

fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer)?;                       // Read data
    println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));
    
    stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n")?;          // Write response
    stream.flush()?;                                         // Flush buffer
    
    Ok(())
}

// ===== TCP SERVER - MULTI-THREADED =====
use std::thread;

fn tcp_server_threaded() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    
    for stream in listener.incoming() {
        let stream = stream?;
        thread::spawn(move || {                              // Spawn thread per connection
            if let Err(e) = handle_client_thread(stream) {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
    
    Ok(())
}

fn handle_client_thread(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];
    loop {
        let n = stream.read(&mut buffer)?;
        if n == 0 {
            break;                                           // Connection closed
        }
        stream.write_all(&buffer[..n])?;                    // Echo back
    }
    Ok(())
}

// ===== TCP SERVER - THREAD POOL =====
use std::sync::{Arc, Mutex, mpsc};
use std::sync::mpsc::{Sender, Receiver};

struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    fn new(size: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        
        ThreadPool { workers, sender }
    }
    
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();
            
            match job {
                Ok(job) => {
                    println!("Worker {} executing job", id);
                    job();
                }
                Err(_) => break,
            }
        });
        
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

fn tcp_server_with_pool() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    let pool = ThreadPool::new(4);
    
    for stream in listener.incoming() {
        let stream = stream?;
        pool.execute(move || {
            handle_client(stream).unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
            });
        });
    }
    
    Ok(())
}

// ===== TCP CLIENT =====
// Basic TCP client
fn tcp_client_basic() -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;  // Connect to server
    
    stream.write_all(b"Hello, server!")?;                    // Send data
    stream.flush()?;
    
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer)?;                       // Read response
    println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));
    
    Ok(())
}

// TCP client with timeout
fn tcp_client_with_timeout() -> Result<()> {
    let mut stream = TcpStream::connect_timeout(
        &"127.0.0.1:8080".parse().unwrap(),
        Duration::from_secs(5)
    )?;
    
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;  // Read timeout
    stream.set_write_timeout(Some(Duration::from_secs(5)))?; // Write timeout
    
    stream.write_all(b"Hello")?;
    
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(n) => println!("Read {} bytes", n),
        Err(e) if e.kind() == ErrorKind::TimedOut => {
            println!("Read timeout");
        }
        Err(e) => return Err(e),
    }
    
    Ok(())
}

// ===== TCP SOCKET CONFIGURATION =====
fn tcp_configure_socket(stream: &TcpStream) -> Result<()> {
    // Get local/remote addresses
    let local = stream.local_addr()?;
    let peer = stream.peer_addr()?;
    println!("Local: {}, Peer: {}", local, peer);
    
    // TCP options
    stream.set_nodelay(true)?;                               // Disable Nagle's algorithm
    stream.set_ttl(64)?;                                     // Set TTL
    
    // Timeouts
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;
    
    // Get options
    let nodelay = stream.nodelay()?;
    let ttl = stream.ttl()?;
    println!("Nodelay: {}, TTL: {}", nodelay, ttl);
    
    // Linger
    stream.set_linger(Some(Duration::from_secs(5)))?;       // SO_LINGER
    
    // Clone stream (shares underlying socket)
    let stream2 = stream.try_clone()?;
    
    Ok(())
}

// ===== TCP BUFFERED I/O =====
use std::io::BufWriter;

fn tcp_buffered_io() -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080")?;
    
    // Buffered reading
    let reader = BufReader::new(&stream);
    for line in reader.lines() {
        let line = line?;
        println!("Line: {}", line);
    }
    
    // Buffered writing
    let mut writer = BufWriter::new(&stream);
    writer.write_all(b"Line 1\n")?;
    writer.write_all(b"Line 2\n")?;
    writer.flush()?;                                         // Flush buffer
    
    Ok(())
}

// ===== TCP SHUTDOWN =====
use std::net::Shutdown;

fn tcp_shutdown(stream: TcpStream) -> Result<()> {
    stream.shutdown(Shutdown::Write)?;                       // Close write half
    stream.shutdown(Shutdown::Read)?;                        // Close read half
    stream.shutdown(Shutdown::Both)?;                        // Close both halves
    
    Ok(())
}

```

## UDP Cheat Sheet
```rust
// ===== UDP SOCKET =====
// Basic UDP server
fn udp_server_basic() -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:8080")?;        // Bind to address
    println!("UDP server listening on port 8080");
    
    let mut buffer = [0; 1024];
    loop {
        let (n, src) = socket.recv_from(&mut buffer)?;      // Receive datagram
        println!("Received {} bytes from {}", n, src);
        
        socket.send_to(&buffer[..n], src)?;                 // Send response
    }
}

// UDP client
fn udp_client_basic() -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;           // Bind to any port
    
    socket.send_to(b"Hello, UDP!", "127.0.0.1:8080")?;      // Send datagram
    
    let mut buffer = [0; 1024];
    let (n, src) = socket.recv_from(&mut buffer)?;          // Receive response
    println!("Received {} bytes from {}", n, src);
    
    Ok(())
}

// ===== UDP CONNECTED MODE =====
fn udp_connected() -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.connect("127.0.0.1:8080")?;                       // Connect to remote
    
    // Now can use send/recv instead of send_to/recv_from
    socket.send(b"Hello")?;                                  // Send to connected address
    
    let mut buffer = [0; 1024];
    let n = socket.recv(&mut buffer)?;                       // Receive from connected
    println!("Received {} bytes", n);
    
    Ok(())
}

// ===== UDP SOCKET CONFIGURATION =====
fn udp_configure_socket(socket: &UdpSocket) -> Result<()> {
    // Get addresses
    let local = socket.local_addr()?;
    println!("Local address: {}", local);
    
    if let Ok(peer) = socket.peer_addr() {                  // Only if connected
        println!("Peer address: {}", peer);
    }
    
    // Set options
    socket.set_broadcast(true)?;                             // Enable broadcast
    socket.set_ttl(64)?;                                     // Set TTL
    socket.set_read_timeout(Some(Duration::from_secs(5)))?; // Read timeout
    socket.set_write_timeout(Some(Duration::from_secs(5)))?;// Write timeout
    
    // Get options
    let broadcast = socket.broadcast()?;
    let ttl = socket.ttl()?;
    println!("Broadcast: {}, TTL: {}", broadcast, ttl);
    
    // Clone socket
    let socket2 = socket.try_clone()?;
    
    Ok(())
}

// ===== UDP BROADCAST =====
fn udp_broadcast_send() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;                             // Enable broadcast
    
    socket.send_to(
        b"Broadcast message",
        "255.255.255.255:8080"                               // Broadcast address
    )?;
    
    Ok(())
}

fn udp_broadcast_receive() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:8080")?;          // Bind to all interfaces
    socket.set_broadcast(true)?;
    
    let mut buffer = [0; 1024];
    loop {
        let (n, src) = socket.recv_from(&mut buffer)?;
        println!("Broadcast from {}: {}", src, 
                 String::from_utf8_lossy(&buffer[..n]));
    }
}

// ===== UDP MULTICAST =====
fn udp_multicast_send() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    
    let multicast_addr: Ipv4Addr = "224.0.0.1".parse().unwrap();
    socket.send_to(
        b"Multicast message",
        (multicast_addr, 8080)
    )?;
    
    Ok(())
}

fn udp_multicast_receive() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:8080")?;
    
    let multicast_addr: Ipv4Addr = "224.0.0.1".parse().unwrap();
    let interface = Ipv4Addr::new(0, 0, 0, 0);              // Any interface
    
    socket.join_multicast_v4(&multicast_addr, &interface)?; // Join multicast group
    
    let mut buffer = [0; 1024];
    loop {
        let (n, src) = socket.recv_from(&mut buffer)?;
        println!("Multicast from {}: {}",
                 src, String::from_utf8_lossy(&buffer[..n]));
    }
    
    // Leave multicast group
    // socket.leave_multicast_v4(&multicast_addr, &interface)?;
}

// IPv6 multicast
fn udp_multicast_v6() -> Result<()> {
    let socket = UdpSocket::bind("[::]:8080")?;
    
    let multicast_addr: Ipv6Addr = "ff02::1".parse().unwrap();
    socket.join_multicast_v6(&multicast_addr, 0)?;          // 0 = default interface
    
    let mut buffer = [0; 1024];
    let (n, src) = socket.recv_from(&mut buffer)?;
    
    socket.leave_multicast_v6(&multicast_addr, 0)?;
    
    Ok(())
}

// ===== SOCKET ADDRESSES =====
fn working_with_addresses() -> Result<()> {
    // Parse SocketAddr
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let addr: SocketAddr = "[::1]:8080".parse().unwrap();   // IPv6
    
    // Create SocketAddr
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let addr = SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], 8080)); // IPv6
    
    // Get IP and port
    let ip = addr.ip();
    let port = addr.port();
    println!("IP: {}, Port: {}", ip, port);
    
    // Check IPv4/IPv6
    if addr.is_ipv4() {
        println!("IPv4 address");
    }
    if addr.is_ipv6() {
        println!("IPv6 address");
    }
    
    // Multiple addresses (try each)
    let addrs = "example.com:80".to_socket_addrs()?;
    for addr in addrs {
        println!("Resolved: {}", addr);
    }
    
    Ok(())
}

// ===== COMMON PATTERNS =====

// Pattern 1: Echo server
fn tcp_echo_server() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    
    for stream in listener.incoming() {
        let mut stream = stream?;
        thread::spawn(move || {
            let mut buffer = [0; 1024];
            loop {
                match stream.read(&mut buffer) {
                    Ok(0) => break,                          // Connection closed
                    Ok(n) => {
                        stream.write_all(&buffer[..n]).unwrap();
                    }
                    Err(_) => break,
                }
            }
        });
    }
    
    Ok(())
}

// Pattern 2: HTTP-like protocol
fn tcp_http_style() -> Result<()> {
    let mut stream = TcpStream::connect("example.com:80")?;
    
    // Send HTTP request
    stream.write_all(b"GET / HTTP/1.1\r\n")?;
    stream.write_all(b"Host: example.com\r\n")?;
    stream.write_all(b"\r\n")?;
    stream.flush()?;
    
    // Read response
    let reader = BufReader::new(&stream);
    for line in reader.lines() {
        println!("{}", line?);
    }
    
    Ok(())
}

// Pattern 3: Read exact amount
fn tcp_read_exact(mut stream: &TcpStream) -> Result<Vec<u8>> {
    let mut length_bytes = [0u8; 4];
    stream.read_exact(&mut length_bytes)?;                   // Read exact 4 bytes
    
    let length = u32::from_be_bytes(length_bytes) as usize;
    
    let mut data = vec![0u8; length];
    stream.read_exact(&mut data)?;                           // Read exact length
    
    Ok(data)
}

// Pattern 4: Write with length prefix
fn tcp_write_with_length(mut stream: &TcpStream, data: &[u8]) -> Result<()> {
    let length = data.len() as u32;
    stream.write_all(&length.to_be_bytes())?;               // Write length
    stream.write_all(data)?;                                 // Write data
    stream.flush()?;
    Ok(())
}

// Pattern 5: Non-blocking I/O
fn tcp_nonblocking() -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080")?;
    stream.set_nonblocking(true)?;                           // Set non-blocking
    
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(n) => {
                println!("Read {} bytes", n);
                break;
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                // No data available, do other work
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    
    Ok(())
}

// Pattern 6: UDP request-response with timeout
fn udp_request_with_retry() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_secs(2)))?;
    
    let server_addr = "127.0.0.1:8080";
    let mut buffer = [0; 1024];
    
    for attempt in 0..3 {
        socket.send_to(b"Request", server_addr)?;
        
        match socket.recv_from(&mut buffer) {
            Ok((n, src)) => {
                println!("Response from {}: {}", 
                         src, String::from_utf8_lossy(&buffer[..n]));
                return Ok(());
            }
            Err(e) if e.kind() == ErrorKind::TimedOut => {
                println!("Attempt {} timed out", attempt + 1);
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    
    Err(Error::new(ErrorKind::TimedOut, "All retries failed"))
}

// Pattern 7: Connection pool
use std::collections::VecDeque;

struct ConnectionPool {
    connections: Arc<Mutex<VecDeque<TcpStream>>>,
    addr: String,
    max_size: usize,
}

impl ConnectionPool {
    fn new(addr: String, max_size: usize) -> Self {
        ConnectionPool {
            connections: Arc::new(Mutex::new(VecDeque::new())),
            addr,
            max_size,
        }
    }
    
    fn get(&self) -> Result<TcpStream> {
        let mut pool = self.connections.lock().unwrap();
        
        if let Some(stream) = pool.pop_front() {
            Ok(stream)
        } else {
            TcpStream::connect(&self.addr)
        }
    }
    
    fn put(&self, stream: TcpStream) {
        let mut pool = self.connections.lock().unwrap();
        if pool.len() < self.max_size {
            pool.push_back(stream);
        }
    }
}

// Pattern 8: Keep-alive
fn tcp_keepalive(stream: &TcpStream) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        use libc::{setsockopt, SOL_SOCKET, SO_KEEPALIVE};
        
        let fd = stream.as_raw_fd();
        let optval: libc::c_int = 1;
        unsafe {
            setsockopt(
                fd,
                SOL_SOCKET,
                SO_KEEPALIVE,
                &optval as *const _ as *const libc::c_void,
                std::mem::size_of_val(&optval) as libc::socklen_t,
            );
        }
    }
    
    Ok(())
}

// Pattern 9: Graceful shutdown
fn tcp_graceful_shutdown() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    listener.set_nonblocking(true)?;
    
    let running = Arc::new(Mutex::new(true));
    let running_clone = Arc::clone(&running);
    
    // Spawn signal handler thread
    thread::spawn(move || {
        // Wait for signal (simplified)
        std::thread::sleep(Duration::from_secs(10));
        *running_clone.lock().unwrap() = false;
    });
    
    // Accept loop
    while *running.lock().unwrap() {
        match listener.accept() {
            Ok((stream, _)) => {
                // Handle connection
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(e),
        }
    }
    
    println!("Server shutting down gracefully");
    Ok(())
}
```

## Reqwest Cheat Sheet
```rust
// Cargo.toml dependencies:
/*
[dependencies]
reqwest = { version = "0.11", features = ["json", "blocking", "cookies", "multipart"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
*/

use reqwest::{Client, Response, Error, StatusCode, Method, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// ===== BASIC GET REQUEST =====
// Async GET request
#[tokio::main]
async fn basic_get() -> Result<(), Error> {
    let response = reqwest::get("https://api.example.com/data").await?;
    
    // Get response status
    let status = response.status();
    println!("Status: {}", status);
    
    // Get response text
    let body = response.text().await?;
    println!("Body: {}", body);
    
    Ok(())
}

// Blocking GET request
fn blocking_get() -> Result<(), Error> {
    let response = reqwest::blocking::get("https://api.example.com/data")?;
    let body = response.text()?;
    println!("Body: {}", body);
    Ok(())
}

// ===== CLIENT CREATION =====
// Create reusable client
async fn create_client() -> Result<(), Error> {
    let client = Client::new();                              // Basic client
    
    let response = client
        .get("https://api.example.com/data")
        .send()
        .await?;
    
    Ok(())
}

// Client with configuration
async fn configured_client() -> Result<(), Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))                    // Request timeout
        .connect_timeout(Duration::from_secs(10))            // Connect timeout
        .user_agent("MyApp/1.0")                             // User agent
        .gzip(true)                                          // Enable gzip
        .brotli(true)                                        // Enable brotli
        .deflate(true)                                       // Enable deflate
        .redirect(reqwest::redirect::Policy::limited(10))   // Max redirects
        .pool_max_idle_per_host(10)                         // Connection pool
        .pool_idle_timeout(Duration::from_secs(90))         // Idle timeout
        .tcp_keepalive(Duration::from_secs(60))             // Keep-alive
        .build()?;
    
    Ok(())
}

// ===== HTTP METHODS =====
async fn http_methods(client: &Client) -> Result<(), Error> {
    // GET
    let response = client.get("https://api.example.com/resource").send().await?;
    
    // POST
    let response = client.post("https://api.example.com/resource")
        .body("request body")
        .send()
        .await?;
    
    // PUT
    let response = client.put("https://api.example.com/resource")
        .body("updated data")
        .send()
        .await?;
    
    // PATCH
    let response = client.patch("https://api.example.com/resource")
        .body("partial update")
        .send()
        .await?;
    
    // DELETE
    let response = client.delete("https://api.example.com/resource").send().await?;
    
    // HEAD
    let response = client.head("https://api.example.com/resource").send().await?;
    
    // Custom method
    let response = client.request(Method::OPTIONS, "https://api.example.com")
        .send()
        .await?;
    
    Ok(())
}

// ===== REQUEST HEADERS =====
async fn request_headers(client: &Client) -> Result<(), Error> {
    let response = client.get("https://api.example.com/data")
        .header("Authorization", "Bearer token123")          // Single header
        .header("X-Custom-Header", "value")
        .header(header::ACCEPT, "application/json")         // Typed header
        .header(header::CONTENT_TYPE, "application/json")
        .send()
        .await?;
    
    // Multiple headers with HeaderMap
    let mut headers = header::HeaderMap::new();
    headers.insert(header::AUTHORIZATION, "Bearer token".parse().unwrap());
    headers.insert(header::ACCEPT, "application/json".parse().unwrap());
    
    let response = client.get("https://api.example.com/data")
        .headers(headers)
        .send()
        .await?;
    
    Ok(())
}

// ===== RESPONSE HEADERS =====
async fn response_headers() -> Result<(), Error> {
    let response = reqwest::get("https://api.example.com/data").await?;
    
    // Access headers
    let headers = response.headers();
    
    // Get specific header
    if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        println!("Content-Type: {:?}", content_type);
    }
    
    // Iterate headers
    for (name, value) in headers.iter() {
        println!("{}: {:?}", name, value);
    }
    
    // Get header as string
    if let Some(value) = headers.get("X-Custom").and_then(|v| v.to_str().ok()) {
        println!("Custom header: {}", value);
    }
    
    Ok(())
}

// ===== QUERY PARAMETERS =====
async fn query_parameters(client: &Client) -> Result<(), Error> {
    // Using tuples
    let response = client.get("https://api.example.com/search")
        .query(&[("q", "rust"), ("page", "1")])
        .send()
        .await?;
    
    // Using HashMap
    let mut params = HashMap::new();
    params.insert("q", "rust");
    params.insert("limit", "10");
    
    let response = client.get("https://api.example.com/search")
        .query(&params)
        .send()
        .await?;
    
    // Using struct with serde
    #[derive(Serialize)]
    struct SearchQuery {
        q: String,
        page: u32,
        limit: u32,
    }
    
    let query = SearchQuery {
        q: "rust".to_string(),
        page: 1,
        limit: 10,
    };
    
    let response = client.get("https://api.example.com/search")
        .query(&query)
        .send()
        .await?;
    
    Ok(())
}

// ===== JSON REQUEST/RESPONSE =====
#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

async fn json_requests(client: &Client) -> Result<(), Error> {
    // Send JSON
    let new_user = User {
        id: 0,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    
    let response = client.post("https://api.example.com/users")
        .json(&new_user)                                     // Serialize to JSON
        .send()
        .await?;
    
    // Receive JSON
    let user: User = response.json().await?;                 // Deserialize from JSON
    println!("Created user: {:?}", user);
    
    Ok(())
}

// Get JSON directly
async fn get_json() -> Result<User, Error> {
    let user = reqwest::get("https://api.example.com/users/1")
        .await?
        .json::<User>()
        .await?;
    
    Ok(user)
}

// ===== FORM DATA =====
async fn form_data(client: &Client) -> Result<(), Error> {
    // URL-encoded form
    let params = [("username", "alice"), ("password", "secret")];
    
    let response = client.post("https://api.example.com/login")
        .form(&params)                                       // application/x-www-form-urlencoded
        .send()
        .await?;
    
    // Using HashMap
    let mut form = HashMap::new();
    form.insert("username", "alice");
    form.insert("password", "secret");
    
    let response = client.post("https://api.example.com/login")
        .form(&form)
        .send()
        .await?;
    
    Ok(())
}

// ===== MULTIPART FORM =====
use reqwest::multipart;

async fn multipart_form(client: &Client) -> Result<(), Error> {
    // Create multipart form
    let form = multipart::Form::new()
        .text("name", "Alice")                               // Text field
        .text("email", "alice@example.com")
        .file("avatar", "/path/to/avatar.png")              // File field
        .await?;
    
    let response = client.post("https://api.example.com/upload")
        .multipart(form)
        .send()
        .await?;
    
    Ok(())
}

// Upload file from bytes
async fn upload_bytes(client: &Client) -> Result<(), Error> {
    let file_data = std::fs::read("file.txt")?;
    
    let part = multipart::Part::bytes(file_data)
        .file_name("file.txt")
        .mime_str("text/plain")?;
    
    let form = multipart::Form::new()
        .part("file", part);
    
    let response = client.post("https://api.example.com/upload")
        .multipart(form)
        .send()
        .await?;
    
    Ok(())
}

// ===== RESPONSE HANDLING =====
async fn response_handling() -> Result<(), Error> {
    let response = reqwest::get("https://api.example.com/data").await?;
    
    // Get status code
    let status = response.status();
    println!("Status: {}", status);
    
    // Check status
    if status.is_success() {
        println!("Success!");
    } else if status.is_client_error() {
        println!("Client error: {}", status);
    } else if status.is_server_error() {
        println!("Server error: {}", status);
    }
    
    // Specific status codes
    match status {
        StatusCode::OK => println!("OK"),
        StatusCode::NOT_FOUND => println!("Not found"),
        StatusCode::UNAUTHORIZED => println!("Unauthorized"),
        _ => println!("Other status: {}", status),
    }
    
    // Error for status
    let response = response.error_for_status()?;            // Error if not 2xx
    
    // Get response as different types
    let text = response.text().await?;                       // As text
    // let json: MyType = response.json().await?;            // As JSON
    // let bytes = response.bytes().await?;                  // As bytes
    
    Ok(())
}

// ===== STREAMING RESPONSES =====
use futures_util::StreamExt;

async fn stream_response() -> Result<(), Error> {
    let response = reqwest::get("https://api.example.com/large-file").await?;
    
    let mut stream = response.bytes_stream();
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        println!("Received {} bytes", chunk.len());
        // Process chunk
    }
    
    Ok(())
}

// Download file with progress
async fn download_file() -> Result<(), Error> {
    let response = reqwest::get("https://example.com/file.zip").await?;
    let total_size = response.content_length().unwrap_or(0);
    
    let mut downloaded = 0u64;
    let mut stream = response.bytes_stream();
    let mut file = tokio::fs::File::create("output.zip").await?;
    
    use tokio::io::AsyncWriteExt;
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        
        let progress = (downloaded as f64 / total_size as f64) * 100.0;
        println!("Downloaded: {:.2}%", progress);
    }
    
    Ok(())
}

// ===== AUTHENTICATION =====
async fn basic_auth(client: &Client) -> Result<(), Error> {
    // Basic authentication
    let response = client.get("https://api.example.com/protected")
        .basic_auth("username", Some("password"))
        .send()
        .await?;
    
    Ok(())
}

async fn bearer_token(client: &Client) -> Result<(), Error> {
    // Bearer token
    let response = client.get("https://api.example.com/protected")
        .bearer_auth("your-token-here")
        .send()
        .await?;
    
    Ok(())
}

// ===== COOKIES =====
async fn cookies_example() -> Result<(), Error> {
    // Client with cookie store
    let client = Client::builder()
        .cookie_store(true)                                  // Enable cookie store
        .build()?;
    
    // First request sets cookies
    let response = client.get("https://api.example.com/login")
        .send()
        .await?;
    
    // Second request uses stored cookies
    let response = client.get("https://api.example.com/profile")
        .send()
        .await?;
    
    Ok(())
}

// Custom cookie jar
use reqwest::cookie::Jar;
use std::sync::Arc;

async fn custom_cookie_jar() -> Result<(), Error> {
    let jar = Arc::new(Jar::default());
    
    // Add cookies manually
    let url = "https://api.example.com".parse().unwrap();
    jar.add_cookie_str("session=abc123", &url);
    
    let client = Client::builder()
        .cookie_provider(Arc::clone(&jar))
        .build()?;
    
    let response = client.get("https://api.example.com/data")
        .send()
        .await?;
    
    Ok(())
}

// ===== PROXY =====
async fn proxy_example() -> Result<(), Error> {
    // HTTP proxy
    let proxy = reqwest::Proxy::http("http://proxy.example.com:8080")?;
    
    let client = Client::builder()
        .proxy(proxy)
        .build()?;
    
    // HTTPS proxy
    let proxy = reqwest::Proxy::https("https://proxy.example.com:8080")?;
    
    // All protocols
    let proxy = reqwest::Proxy::all("http://proxy.example.com:8080")?;
    
    // Proxy with authentication
    let proxy = reqwest::Proxy::http("http://proxy.example.com:8080")?
        .basic_auth("username", "password");
    
    let client = Client::builder()
        .proxy(proxy)
        .build()?;
    
    Ok(())
}

// ===== TIMEOUTS =====
async fn timeout_example() -> Result<(), Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))                    // Overall timeout
        .connect_timeout(Duration::from_secs(10))            // Connect timeout
        .build()?;
    
    // Per-request timeout
    let response = client.get("https://api.example.com/slow")
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    
    Ok(())
}

// ===== RETRIES =====
async fn retry_request() -> Result<(), Error> {
    let client = Client::new();
    let mut attempts = 0;
    let max_attempts = 3;
    
    loop {
        attempts += 1;
        
        match client.get("https://api.example.com/data").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    return Ok(());
                }
            }
            Err(e) if attempts < max_attempts => {
                println!("Attempt {} failed: {}", attempts, e);
                tokio::time::sleep(Duration::from_secs(2u64.pow(attempts))).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}

// ===== ERROR HANDLING =====
async fn error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let result = reqwest::get("https://api.example.com/data").await;
    
    match result {
        Ok(response) => {
            // Check status
            if !response.status().is_success() {
                return Err(format!("HTTP error: {}", response.status()).into());
            }
            
            let body = response.text().await?;
            println!("Body: {}", body);
        }
        Err(e) => {
            // Check error type
            if e.is_timeout() {
                println!("Request timed out");
            } else if e.is_connect() {
                println!("Connection error");
            } else if e.is_redirect() {
                println!("Redirect error");
            } else if e.is_status() {
                println!("Status error: {:?}", e.status());
            } else if e.is_request() {
                println!("Request error");
            }
            
            return Err(e.into());
        }
    }
    
    Ok(())
}

// ===== TLS/SSL CONFIGURATION =====
async fn tls_configuration() -> Result<(), Error> {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)                   // Accept invalid certs (DANGER!)
        .danger_accept_invalid_hostnames(true)               // Accept invalid hostnames (DANGER!)
        .min_tls_version(reqwest::tls::Version::TLS_1_2)    // Minimum TLS version
        .build()?;
    
    Ok(())
}

// Custom TLS with certificate
async fn custom_certificate() -> Result<(), Error> {
    let cert = std::fs::read("cert.pem")?;
    let cert = reqwest::Certificate::from_pem(&cert)?;
    
    let client = Client::builder()
        .add_root_certificate(cert)
        .build()?;
    
    Ok(())
}

// ===== COMMON PATTERNS =====

// Pattern 1: API client wrapper
struct ApiClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl ApiClient {
    fn new(base_url: String, api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
        
        ApiClient { client, base_url, api_key }
    }
    
    async fn get_user(&self, id: u32) -> Result<User, Error> {
        let url = format!("{}/users/{}", self.base_url, id);
        
        let user = self.client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(user)
    }
    
    async fn create_user(&self, user: &User) -> Result<User, Error> {
        let url = format!("{}/users", self.base_url);
        
        let created = self.client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(user)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(created)
    }
}

// Pattern 2: Pagination
async fn fetch_all_pages(client: &Client) -> Result<Vec<User>, Error> {
    let mut all_users = Vec::new();
    let mut page = 1;
    
    loop {
        let response: Vec<User> = client
            .get("https://api.example.com/users")
            .query(&[("page", page), ("limit", 100)])
            .send()
            .await?
            .json()
            .await?;
        
        if response.is_empty() {
            break;
        }
        
        all_users.extend(response);
        page += 1;
    }
    
    Ok(all_users)
}

// Pattern 3: Parallel requests
async fn parallel_requests(client: &Client, ids: Vec<u32>) -> Result<Vec<User>, Error> {
    let futures: Vec<_> = ids
        .into_iter()
        .map(|id| {
            let client = client.clone();
            async move {
                let url = format!("https://api.example.com/users/{}", id);
                client.get(&url).send().await?.json::<User>().await
            }
        })
        .collect();
    
    let results = futures::future::join_all(futures).await;
    
    let users: Result<Vec<_>, _> = results.into_iter().collect();
    users
}

// Pattern 4: Rate limiting
use std::time::Instant;

async fn rate_limited_requests(client: &Client, urls: Vec<String>) -> Result<(), Error> {
    let rate_limit = Duration::from_millis(100);             // 10 requests per second
    let mut last_request = Instant::now();
    
    for url in urls {
        let elapsed = last_request.elapsed();
        if elapsed < rate_limit {
            tokio::time::sleep(rate_limit - elapsed).await;
        }
        
        client.get(&url).send().await?;
        last_request = Instant::now();
    }
    
    Ok(())
}

// Pattern 5: Conditional requests (ETag)
async fn conditional_request(client: &Client, etag: Option<&str>) -> Result<(), Error> {
    let mut request = client.get("https://api.example.com/data");
    
    if let Some(etag) = etag {
        request = request.header(header::IF_NONE_MATCH, etag);
    }
    
    let response = request.send().await?;
    
    match response.status() {
        StatusCode::NOT_MODIFIED => {
            println!("Data not modified");
        }
        StatusCode::OK => {
            let new_etag = response.headers()
                .get(header::ETAG)
                .and_then(|v| v.to_str().ok());
            
            let body = response.text().await?;
            println!("New data, ETag: {:?}", new_etag);
        }
        _ => {}
    }
    
    Ok(())
}
```

## Axum Cheat Sheet

```rust
// ===== AXUM =====
// Cargo.toml:
/*
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tower = "0.4"
tower-http = { version = "0.5", features = ["fs", "trace", "cors"] }
*/

use axum::{
    routing::{get, post, put, delete},
    Router, Json, extract::{Path, Query, State},
    response::{IntoResponse, Response, Html},
    http::{StatusCode, HeaderMap, Method},
    middleware,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// ===== AXUM BASIC SERVER =====
#[tokio::main]
async fn axum_basic() {
    let app = Router::new()
        .route("/", get(root))
        .route("/hello/:name", get(hello));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    println!("Server running on http://localhost:3000");
    
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn hello(Path(name): Path<String>) -> String {
    format!("Hello, {}!", name)
}

// ===== AXUM ROUTES =====
fn axum_routes() -> Router {
    Router::new()
        .route("/", get(root))                               // GET /
        .route("/users", get(get_users).post(create_user))  // GET, POST /users
        .route("/users/:id", get(get_user).put(update_user).delete(delete_user))
        .route("/static/*path", get(serve_static))          // Wildcard route
}

// ===== AXUM HANDLERS =====
// Path parameters
async fn get_user(Path(id): Path<u32>) -> String {
    format!("User ID: {}", id)
}

// Multiple path parameters
async fn get_post(Path((user_id, post_id)): Path<(u32, u32)>) -> String {
    format!("User: {}, Post: {}", user_id, post_id)
}

// Query parameters
#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn get_users(Query(pagination): Query<Pagination>) -> String {
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(10);
    format!("Page: {}, Limit: {}", page, limit)
}

// ===== AXUM JSON =====
#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

// Return JSON
async fn get_user_json(Path(id): Path<u32>) -> Json<User> {
    let user = User {
        id,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    
    Json(user)
}

// Accept JSON
async fn create_user(Json(user): Json<User>) -> (StatusCode, Json<User>) {
    println!("Creating user: {:?}", user);
    (StatusCode::CREATED, Json(user))
}

// ===== AXUM STATE =====
// Shared application state
#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Vec<User>>>,
}

fn axum_with_state() -> Router {
    let state = AppState {
        db: Arc::new(Mutex::new(Vec::new())),
    };
    
    Router::new()
        .route("/users", get(list_users).post(add_user))
        .with_state(state)
}

async fn list_users(State(state): State<AppState>) -> Json<Vec<User>> {
    let db = state.db.lock().await;
    Json(db.clone())
}

async fn add_user(
    State(state): State<AppState>,
    Json(user): Json<User>,
) -> StatusCode {
    let mut db = state.db.lock().await;
    db.push(user);
    StatusCode::CREATED
}

// ===== AXUM HEADERS =====
async fn read_headers(headers: HeaderMap) -> String {
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    
    format!("User-Agent: {}", user_agent)
}

// Return custom headers
async fn with_headers() -> (HeaderMap, &'static str) {
    let mut headers = HeaderMap::new();
    headers.insert("X-Custom-Header", "value".parse().unwrap());
    (headers, "Response with headers")
}

// ===== AXUM RESPONSE TYPES =====
use axum::response::Redirect;

// HTML response
async fn html_response() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

// Redirect
async fn redirect_handler() -> Redirect {
    Redirect::to("/")
}

// Status code with body
async fn not_found() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found")
}

// Custom response
async fn custom_response() -> Response {
    (
        StatusCode::OK,
        [("X-Custom", "value")],
        "Custom response",
    )
        .into_response()
}

// ===== AXUM ERROR HANDLING =====
use axum::http::header;

#[derive(Debug)]
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

async fn fallible_handler() -> Result<String, AppError> {
    // May return error
    Ok("Success".to_string())
}

// ===== AXUM MIDDLEWARE =====
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tower_http::cors::{CorsLayer, Any};

fn axum_with_middleware() -> Router {
    let app = Router::new()
        .route("/", get(root))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())           // Logging
                .layer(CorsLayer::permissive())              // CORS
        );
    
    app
}

// Custom middleware
use axum::middleware::Next;
use axum::extract::Request;

async fn auth_middleware(
    headers: HeaderMap,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    
    if let Some(auth) = auth_header {
        if auth.starts_with("Bearer ") {
            return Ok(next.run(req).await);
        }
    }
    
    Err(StatusCode::UNAUTHORIZED)
}

fn with_auth_middleware() -> Router {
    Router::new()
        .route("/protected", get(protected_route))
        .layer(middleware::from_fn(auth_middleware))
}

async fn protected_route() -> &'static str {
    "Protected resource"
}

// ===== AXUM FILE UPLOADS =====
use axum::extract::Multipart;

async fn upload_file(mut multipart: Multipart) -> Result<String, StatusCode> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        
        println!("Field: {}, Size: {} bytes", name, data.len());
        
        // Save file
        tokio::fs::write(format!("uploads/{}", name), data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    
    Ok("Files uploaded".to_string())
}

// ===== AXUM WEBSOCKETS =====
use axum::extract::ws::{WebSocket, WebSocketUpgrade, Message};

async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(_) => break,
        };
        
        match msg {
            Message::Text(text) => {
                println!("Received: {}", text);
                socket.send(Message::Text(format!("Echo: {}", text)))
                    .await
                    .unwrap();
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

// ===== AXUM NESTED ROUTES =====
fn axum_nested() -> Router {
    let api_routes = Router::new()
        .route("/users", get(get_users))
        .route("/posts", get(get_posts));
    
    Router::new()
        .route("/", get(root))
        .nest("/api", api_routes)                            // /api/users, /api/posts
        .nest("/admin", admin_routes())
}

fn admin_routes() -> Router {
    Router::new()
        .route("/dashboard", get(admin_dashboard))
        .route("/settings", get(admin_settings))
}

async fn get_posts() -> &'static str { "Posts" }
async fn admin_dashboard() -> &'static str { "Dashboard" }
async fn admin_settings() -> &'static str { "Settings" }

// ===== AXUM STATIC FILES =====
use tower_http::services::ServeDir;

fn axum_static_files() -> Router {
    Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .route("/", get(root))
}

async fn serve_static(Path(path): Path<String>) -> Response {
    // Custom static file handling
    (StatusCode::OK, format!("Serving: {}", path)).into_response()
}
```
## Actix-web Cheat Sheet
```rust
// ===== ACTIX-WEB =====
// Cargo.toml:
/*
[dependencies]
actix-web = "4"
actix-files = "0.6"
actix-multipart = "0.6"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
*/

use actix_web::{
    web, App, HttpServer, HttpRequest, HttpResponse, Responder,
    middleware as actix_middleware,
};
use actix_files as fs;

// ===== ACTIX BASIC SERVER =====
#[actix_web::main]
async fn actix_basic() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(actix_root))
            .route("/hello/{name}", web::get().to(actix_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn actix_root() -> impl Responder {
    "Hello, World!"
}

async fn actix_hello(path: web::Path<String>) -> impl Responder {
    format!("Hello, {}!", path.into_inner())
}

// ===== ACTIX ROUTES =====
fn actix_routes() -> App
    impl actix_web::dev::ServiceFactory
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .route("/", web::get().to(actix_root))
        .service(
            web::scope("/users")
                .route("", web::get().to(actix_get_users))
                .route("", web::post().to(actix_create_user))
                .route("/{id}", web::get().to(actix_get_user))
                .route("/{id}", web::put().to(actix_update_user))
                .route("/{id}", web::delete().to(actix_delete_user))
        )
}

// ===== ACTIX HANDLERS =====
// Path parameters
async fn actix_get_user(path: web::Path<u32>) -> impl Responder {
    format!("User ID: {}", path.into_inner())
}

// Multiple path parameters
async fn actix_get_post(path: web::Path<(u32, u32)>) -> impl Responder {
    let (user_id, post_id) = path.into_inner();
    format!("User: {}, Post: {}", user_id, post_id)
}

// Query parameters
async fn actix_get_users(query: web::Query<Pagination>) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    format!("Page: {}, Limit: {}", page, limit)
}

// ===== ACTIX JSON =====
// Return JSON
async fn actix_get_user_json(path: web::Path<u32>) -> impl Responder {
    let user = User {
        id: path.into_inner(),
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    
    web::Json(user)
}

// Accept JSON
async fn actix_create_user(user: web::Json<User>) -> impl Responder {
    println!("Creating user: {:?}", user);
    HttpResponse::Created().json(user.into_inner())
}

// ===== ACTIX STATE =====
use actix_web::web::Data;

struct ActixAppState {
    db: Arc<Mutex<Vec<User>>>,
}

async fn actix_list_users(data: Data<ActixAppState>) -> impl Responder {
    let db = data.db.lock().await;
    web::Json(db.clone())
}

async fn actix_add_user(
    data: Data<ActixAppState>,
    user: web::Json<User>,
) -> impl Responder {
    let mut db = data.db.lock().await;
    db.push(user.into_inner());
    HttpResponse::Created()
}

fn actix_with_state() -> App
    impl actix_web::dev::ServiceFactory
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let state = Data::new(ActixAppState {
        db: Arc::new(Mutex::new(Vec::new())),
    });
    
    App::new()
        .app_data(state)
        .route("/users", web::get().to(actix_list_users))
        .route("/users", web::post().to(actix_add_user))
}

// ===== ACTIX HEADERS =====
async fn actix_read_headers(req: HttpRequest) -> impl Responder {
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    
    format!("User-Agent: {}", user_agent)
}

// Return custom headers
async fn actix_with_headers() -> impl Responder {
    HttpResponse::Ok()
        .insert_header(("X-Custom-Header", "value"))
        .body("Response with headers")
}

// ===== ACTIX RESPONSE TYPES =====
// HTML response
async fn actix_html() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body("<h1>Hello, World!</h1>")
}

// Redirect
async fn actix_redirect() -> impl Responder {
    HttpResponse::Found()
        .insert_header(("Location", "/"))
        .finish()
}

// Custom status
async fn actix_not_found() -> impl Responder {
    HttpResponse::NotFound().body("Not found")
}

// ===== ACTIX ERROR HANDLING =====
use actix_web::error::ResponseError;
use std::fmt;

#[derive(Debug)]
struct ActixAppError {
    message: String,
}

impl fmt::Display for ActixAppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ResponseError for ActixAppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().json(serde_json::json!({
            "error": self.message
        }))
    }
}

async fn actix_fallible() -> Result<String, ActixAppError> {
    Ok("Success".to_string())
}

// ===== ACTIX MIDDLEWARE =====
use actix_web::middleware::{Logger, Compress};

fn actix_with_middleware() -> App
    impl actix_web::dev::ServiceFactory
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .wrap(Logger::default())                             // Logging
        .wrap(Compress::default())                           // Compression
        .route("/", web::get().to(actix_root))
}

// Custom middleware
use actix_web::dev::{Service, Transform, ServiceRequest, ServiceResponse};
use actix_web::Error as ActixError;
use futures::future::{ready, Ready, LocalBoxFuture};

struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;
    
    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }
    
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req.headers().get("Authorization");
        
        if let Some(auth) = auth_header {
            if auth.to_str().unwrap_or("").starts_with("Bearer ") {
                let fut = self.service.call(req);
                return Box::pin(async move { fut.await });
            }
        }
        
        Box::pin(async move {
            Err(actix_web::error::ErrorUnauthorized("Unauthorized"))
        })
    }
}

// ===== ACTIX FILE UPLOADS =====
use actix_multipart::Multipart;
use futures::StreamExt;

async fn actix_upload(mut payload: Multipart) -> Result<HttpResponse, ActixError> {
    while let Some(item) = payload.next().await {
        let mut field = item?;
        
        let content_disposition = field.content_disposition();
        let filename = content_disposition.get_filename().unwrap();
        
        let mut data = Vec::new();
        while let Some(chunk) = field.next().await {
            data.extend_from_slice(&chunk?);
        }
        
        tokio::fs::write(format!("uploads/{}", filename), data).await?;
    }
    
    Ok(HttpResponse::Ok().body("Files uploaded"))
}

// ===== ACTIX WEBSOCKETS =====
use actix_web_actors::ws;
use actix::{Actor, StreamHandler};

struct MyWebSocket;

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                println!("Received: {}", text);
                ctx.text(format!("Echo: {}", text));
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
            }
            _ => {}
        }
    }
}

async fn actix_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, ActixError> {
    ws::start(MyWebSocket {}, &req, stream)
}

// ===== ACTIX STATIC FILES =====
fn actix_static_files() -> App
    impl actix_web::dev::ServiceFactory
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .service(fs::Files::new("/static", "static").show_files_listing())
        .route("/", web::get().to(actix_root))
}

// ===== ACTIX GUARDS =====
use actix_web::guard;

fn actix_with_guards() -> App
    impl actix_web::dev::ServiceFactory
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .service(
            web::resource("/")
                .guard(guard::Get())
                .to(actix_root)
        )
        .service(
            web::resource("/api")
                .guard(guard::Header("content-type", "application/json"))
                .to(actix_api)
        )
}

async fn actix_api() -> impl Responder {
    "API endpoint"
}

// Stub implementations for missing functions
async fn update_user() -> &'static str { "Updated" }
async fn delete_user() -> &'static str { "Deleted" }
async fn actix_update_user() -> impl Responder { "Updated" }
async fn actix_delete_user() -> impl Responder { "Deleted" }
```

## Tungstenite Cheat Sheet

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