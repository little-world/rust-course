# 22. Network Programming

Network programming is fundamental to building distributed systems, web services, and real-time applications. Rust's async ecosystem, combined with its safety guarantees, makes it an excellent choice for building reliable network applications. This guide explores common network programming patterns in Rust, from low-level TCP/UDP to high-level HTTP and WebSocket protocols.

## TCP Server/Client Patterns

TCP (Transmission Control Protocol) provides reliable, ordered, and error-checked delivery of data between applications. It's the foundation for most internet protocols including HTTP, SSH, and FTP. Understanding TCP patterns is essential for building robust network applications.

### Understanding TCP Fundamentals

TCP establishes a connection between a client and server before exchanging data. This connection-oriented approach ensures that all data arrives in order and without corruption. The server listens on a specific port, accepts incoming connections, and then handles each connection independently—often in a separate task or thread.

The basic flow works like this: The server binds to a port and listens for connections. When a client connects, the server accepts the connection, creating a bidirectional stream. Both sides can then read and write data until one side closes the connection.

### Basic Synchronous TCP Server

Let's start with a simple synchronous TCP server that echoes back whatever it receives. This example uses the standard library's TCP facilities:

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

## UDP Patterns

UDP (User Datagram Protocol) is connectionless and doesn't guarantee delivery or ordering. However, it's much faster and has lower overhead than TCP. UDP is ideal for applications where occasional packet loss is acceptable, such as video streaming, gaming, or DNS queries.

### Understanding UDP Characteristics

Unlike TCP, UDP doesn't establish connections. You simply send datagrams to an address and port. This means:
- **No connection overhead**: You can immediately start sending data
- **No reliability**: Packets may be lost, duplicated, or arrive out of order
- **Lower latency**: No handshakes or acknowledgments
- **Message boundaries**: Each send operation creates a discrete datagram

These characteristics make UDP perfect for real-time applications where the latest data is more important than historical data.

### Basic UDP Server

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

## HTTP Client (reqwest)

The `reqwest` library is the de facto standard for HTTP clients in Rust. It provides both async and blocking APIs, with support for JSON, cookies, redirects, and more.

### Basic HTTP Requests

Making HTTP requests with reqwest is straightforward and ergonomic:

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

## HTTP Server (axum, actix-web)

Building HTTP servers in Rust typically involves using one of the major web frameworks. We'll cover both axum (newer, from the Tokio team) and actix-web (mature, high performance).

### Why Choose axum?

axum is built on top of hyper and tower, focusing on type safety and ergonomics. It uses Rust's type system to ensure correctness at compile time. If you value compiler-checked correctness and are already using Tokio, axum is an excellent choice.

### Basic axum Server

Let's build a simple REST API with axum:

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

// Handler for GET /
async fn root_handler() -> &'static str {
    "Hello, World!"
}

// Handler for GET /users
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

// Handler for GET /users/:id
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

// Handler for POST /users
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

## WebSocket Patterns

WebSockets provide full-duplex communication between client and server. Unlike HTTP's request-response model, WebSockets allow the server to push data to clients in real-time. This is perfect for chat applications, live updates, multiplayer games, and collaborative editing.

### Understanding WebSockets

A WebSocket connection starts as an HTTP request with an "Upgrade" header. Once upgraded, the connection stays open and either party can send messages at any time. Messages can be text or binary.

The lifecycle looks like this:
1. Client sends HTTP upgrade request
2. Server accepts upgrade (or rejects)
3. Both parties exchange messages freely
4. Either party can close the connection

### WebSocket Server with axum

Here's a complete WebSocket server that broadcasts messages to all connected clients:

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

## Conclusion

Network programming in Rust offers both safety and performance. The async ecosystem, particularly Tokio, provides excellent primitives for building scalable network applications. Whether you're building low-level TCP servers, HTTP APIs, or real-time WebSocket applications, Rust's type system and ownership model help prevent common networking bugs like race conditions and use-after-free errors.

Key takeaways:
- **Use async/await** for I/O-bound network code—it's more efficient than threads
- **TCP is reliable** but has overhead; use it when you need guaranteed delivery
- **UDP is fast** but unreliable; perfect for real-time applications
- **reqwest** is the go-to for HTTP clients; it's mature and ergonomic
- **axum and actix-web** are both excellent server frameworks—choose based on your preferences
- **WebSockets** enable real-time bidirectional communication; remember to handle disconnections gracefully

As you build network applications, remember that errors are common in networking. Networks are unreliable, clients disconnect unexpectedly, and timeouts happen. Design your applications to handle these gracefully, and you'll build robust systems that users can rely on.
