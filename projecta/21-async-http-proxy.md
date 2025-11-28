
## Project: Async HTTP Proxy Server with Connection Pooling

### Problem Statement

Build a production-ready HTTP proxy server that forwards client requests to backend servers, implementing connection pooling for efficiency, backpressure to prevent memory exhaustion, timeout handling, health checks, and graceful shutdown. You'll learn how reverse proxies, load balancers, and API gateways work internally while mastering async I/O patterns.

### Use Cases

**When you need this pattern**:
1. **Reverse proxies**: Route requests to backend servers (NGINX, HAProxy)
2. **API gateways**: Single entry point for microservices
3. **Load balancers**: Distribute traffic across multiple backends
4. **Service mesh**: Sidecar proxies for service-to-service communication
5. **Caching proxies**: Cache responses from slow backends
6. **Protocol translation**: HTTP to gRPC, REST to GraphQL

### Why It Matters

**Real-World Impact**: Proxies are fundamental to modern web architecture:

**The Direct Connection Problem**:
```rust
// Naive approach - creates new connection per request
async fn handle_request(client_req: Request) -> Response {
    // Problem 1: TCP handshake + TLS = 100ms overhead per request
    let backend = TcpStream::connect("backend:8080").await?;

    // Problem 2: No timeout - hangs forever if backend is down
    backend.write_all(client_req.as_bytes()).await?;

    // Problem 3: Unbounded memory - fast clients overwhelm slow backends
    let response = read_response(backend).await?;

    // Problem 4: Leaked connections - backend hits connection limit
    // (connection closed here but backend maintains TIME_WAIT)
}
```

**Proxy with Connection Pooling Benefits**:
```rust
// Production approach - reuse connections
async fn handle_request(client_req: Request, pool: &ConnectionPool) -> Response {
    // ✓ Connection reuse: 1ms vs 100ms for new connection
    let conn = pool.acquire().await?;

    // ✓ Timeout handling: fail fast if backend is slow
    let response = timeout(Duration::from_secs(30),
        forward_request(&conn, client_req)
    ).await??;

    // ✓ Backpressure: bounded queue prevents memory explosion
    // ✓ Connection pooling: returns conn to pool for reuse
    pool.release(conn);

    response
}
```

**Performance Impact**:
- **Without pooling**: 100ms TCP handshake + 50ms TLS = 150ms overhead per request
- **With pooling**: Connection reuse = ~1ms overhead
- **150x improvement** in latency

**Connection Pool Benefits**:
1. **Reduced latency**: Reuse connections (avoid handshake)
2. **Backend protection**: Limit concurrent connections
3. **Resource efficiency**: Fewer file descriptors, less memory
4. **Health checking**: Remove dead connections automatically
5. **Load balancing**: Distribute requests across healthy backends

**Architecture**:
```
Client → Proxy Server → Connection Pool → Backend Server
  ↓                          ↓                    ↓
Accept                   Acquire Conn         Process
 ↓                          ↓                    ↓
Parse                    Forward Req         Generate Resp
 ↓                          ↓                    ↓
Forward → [Bounded Queue] → Pool → [Conn1, Conn2, Conn3]
 ↓                                      ↓
Return ← [Backpressure] ← Release ← Health Check
```

### Learning Goals

By completing this project, you will:

1. **Master async networking**: `TcpListener`, `TcpStream`, `tokio::spawn`
2. **Implement connection pooling**: Reuse expensive resources
3. **Handle backpressure**: Prevent memory exhaustion with bounded channels
4. **Parse HTTP**: Simple HTTP/1.1 request/response parsing
5. **Timeout handling**: Fail fast on slow backends
6. **Health checks**: Detect and remove stale connections
7. **Graceful shutdown**: Clean up resources on SIGTERM

---

### Milestone 1: Basic TCP Proxy (Echo Server)

**Goal**: Accept TCP connections and forward data bidirectionally.

**Implementation Steps**:

1. **Create TCP listener**:
   - Bind to address with `TcpListener::bind()`
   - Accept connections in loop with `.accept()`
   - Spawn task for each connection with `tokio::spawn`

2. **Implement bidirectional forwarding**:
   - Connect to backend server
   - Copy client → backend and backend → client concurrently
   - Use `tokio::io::copy()` for efficient copying
   - Handle connection close from either side

3. **Basic error handling**:
   - Connection refused (backend down)
   - Connection reset
   - Broken pipe

4. **Logging**:
   - Log accepted connections
   - Log forwarded bytes
   - Log errors

**Starter Code**:

```rust
// Cargo.toml dependencies
// [dependencies]
// tokio = { version = "1", features = ["full"] }
// tracing = "0.1"
// tracing-subscriber = "0.3"

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use std::error::Error;

/// Start proxy server
pub async fn start_proxy(listen_addr: &str, backend_addr: &str) -> Result<(), Box<dyn Error>> {
    // TODO: Bind to listen address
    let listener = TcpListener::bind(listen_addr).await?;
    println!("Proxy listening on {}", listen_addr);
    println!("Forwarding to backend {}", backend_addr);

    // TODO: Accept connections in loop
    loop {
        let (client_stream, client_addr) = listener.accept().await?;
        println!("Accepted connection from {}", client_addr);

        // TODO: Spawn task to handle connection
        let backend = backend_addr.to_string();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(client_stream, &backend).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
}

/// Handle single client connection
async fn handle_connection(
    mut client: TcpStream,
    backend_addr: &str,
) -> Result<(), Box<dyn Error>> {
    // TODO: Connect to backend
    let mut backend = TcpStream::connect(backend_addr).await?;
    println!("Connected to backend {}", backend_addr);

    // TODO: Forward data bidirectionally
    // Hint: Use tokio::io::copy_bidirectional
    let (client_to_backend, backend_to_client) =
        tokio::io::copy_bidirectional(&mut client, &mut backend).await?;

    println!("Connection closed: {}B → backend, {}B ← backend",
        client_to_backend, backend_to_client);

    Ok(())
}
```

**Checkpoint Tests**:

```rust
#[tokio::test]
async fn test_basic_proxy() {
    // Start echo server as backend
    tokio::spawn(async {
        let listener = TcpListener::bind("127.0.0.1:9001").await.unwrap();
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                loop {
                    let n = socket.read(&mut buf).await.unwrap();
                    if n == 0 { break; }
                    socket.write_all(&buf[..n]).await.unwrap();
                }
            });
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Start proxy
    tokio::spawn(async {
        start_proxy("127.0.0.1:8001", "127.0.0.1:9001").await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Test proxy
    let mut client = TcpStream::connect("127.0.0.1:8001").await.unwrap();
    client.write_all(b"Hello, proxy!").await.unwrap();

    let mut buf = [0u8; 1024];
    let n = client.read(&mut buf).await.unwrap();
    assert_eq!(&buf[..n], b"Hello, proxy!");
}

#[tokio::test]
async fn test_multiple_connections() {
    // Start proxy and backend (same as above)
    // ...

    // Create multiple concurrent connections
    let handles: Vec<_> = (0..10).map(|i| {
        tokio::spawn(async move {
            let mut client = TcpStream::connect("127.0.0.1:8001").await.unwrap();
            let msg = format!("Message {}", i);
            client.write_all(msg.as_bytes()).await.unwrap();

            let mut buf = [0u8; 1024];
            let n = client.read(&mut buf).await.unwrap();
            assert_eq!(&buf[..n], msg.as_bytes());
        })
    }).collect();

    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_backend_connection_refused() {
    // Start proxy pointing to non-existent backend
    tokio::spawn(async {
        start_proxy("127.0.0.1:8002", "127.0.0.1:9999").await.ok();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Connection should be accepted but then fail
    let result = TcpStream::connect("127.0.0.1:8002").await;
    // Proxy accepts connection, but backend connection fails
}
```

**Check Your Understanding**:
- Why spawn a new task for each connection?
- What happens if backend is down when client connects?
- How does `copy_bidirectional` work internally?
- Why use async instead of threads?

---

### Milestone 2: HTTP Request/Response Parsing

**Goal**: Parse HTTP/1.1 requests and responses.

**Implementation Steps**:

1. **Parse HTTP request**:
   - Read request line: `GET /path HTTP/1.1`
   - Parse headers: `Host: example.com`
   - Handle chunked transfer encoding
   - Read request body if present

2. **Parse HTTP response**:
   - Read status line: `HTTP/1.1 200 OK`
   - Parse response headers
   - Read response body

3. **Use buffered I/O**:
   - Wrap streams in `BufReader`/`BufWriter`
   - Read headers line-by-line with `.read_line()`
   - Efficient parsing with minimal allocations

4. **Reconstruct HTTP messages**:
   - Serialize requests to send to backend
   - Serialize responses to send to client
   - Preserve header order and formatting

**Starter Code Extension**:

```rust
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    /// Parse HTTP request from stream
    pub async fn parse<R: AsyncBufReadExt + Unpin>(
        reader: &mut R,
    ) -> io::Result<Self> {
        // TODO: Read request line
        let mut request_line = String::new();
        reader.read_line(&mut request_line).await?;

        // TODO: Parse method, path, version
        // Example: "GET /index.html HTTP/1.1\r\n"
        let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
        if parts.len() != 3 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid request line"
            ));
        }

        let method = parts[0].to_string();
        let path = parts[1].to_string();
        let version = parts[2].to_string();

        // TODO: Parse headers
        let mut headers = HashMap::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).await?;

            if line.trim().is_empty() {
                break; // Empty line marks end of headers
            }

            // Parse "Key: Value"
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.insert(key, value);
            }
        }

        // TODO: Read body if Content-Length present
        let body = if let Some(content_length) = headers.get("Content-Length") {
            let len: usize = content_length.parse().unwrap_or(0);
            let mut body = vec![0u8; len];
            reader.read_exact(&mut body).await?;
            body
        } else {
            Vec::new()
        };

        Ok(HttpRequest {
            method,
            path,
            version,
            headers,
            body,
        })
    }

    /// Serialize request to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Reconstruct HTTP request
        let mut bytes = Vec::new();

        // Request line
        bytes.extend_from_slice(
            format!("{} {} {}\r\n", self.method, self.path, self.version).as_bytes()
        );

        // Headers
        for (key, value) in &self.headers {
            bytes.extend_from_slice(format!("{}: {}\r\n", key, value).as_bytes());
        }

        // Empty line
        bytes.extend_from_slice(b"\r\n");

        // Body
        bytes.extend_from_slice(&self.body);

        bytes
    }
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub version: String,
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Parse HTTP response from stream
    pub async fn parse<R: AsyncBufReadExt + Unpin>(
        reader: &mut R,
    ) -> io::Result<Self> {
        // TODO: Similar to request parsing
        // Status line: "HTTP/1.1 200 OK\r\n"

        todo!()
    }

    /// Serialize response to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Similar to request serialization
        todo!()
    }
}
```

**Checkpoint Tests**:

```rust
#[tokio::test]
async fn test_parse_http_request() {
    let request_bytes = b"GET /index.html HTTP/1.1\r\n\
                          Host: example.com\r\n\
                          User-Agent: test\r\n\
                          \r\n";

    let mut reader = BufReader::new(&request_bytes[..]);
    let request = HttpRequest::parse(&mut reader).await.unwrap();

    assert_eq!(request.method, "GET");
    assert_eq!(request.path, "/index.html");
    assert_eq!(request.version, "HTTP/1.1");
    assert_eq!(request.headers.get("Host").unwrap(), "example.com");
}

#[tokio::test]
async fn test_parse_request_with_body() {
    let request_bytes = b"POST /api/data HTTP/1.1\r\n\
                          Host: example.com\r\n\
                          Content-Length: 13\r\n\
                          \r\n\
                          Hello, World!";

    let mut reader = BufReader::new(&request_bytes[..]);
    let request = HttpRequest::parse(&mut reader).await.unwrap();

    assert_eq!(request.method, "POST");
    assert_eq!(request.body, b"Hello, World!");
}

#[tokio::test]
async fn test_serialize_request() {
    let request = HttpRequest {
        method: "GET".to_string(),
        path: "/test".to_string(),
        version: "HTTP/1.1".to_string(),
        headers: {
            let mut h = HashMap::new();
            h.insert("Host".to_string(), "example.com".to_string());
            h
        },
        body: Vec::new(),
    };

    let bytes = request.to_bytes();
    let text = String::from_utf8_lossy(&bytes);

    assert!(text.contains("GET /test HTTP/1.1"));
    assert!(text.contains("Host: example.com"));
}
```

**Check Your Understanding**:
- Why use `BufReader` for HTTP parsing?
- How do we know when headers end?
- What's the difference between Content-Length and chunked encoding?
- Why use `HashMap` for headers instead of `Vec`?

---

### Milestone 3: Connection Pool Implementation

**Goal**: Implement connection pool for backend servers.

**Implementation Steps**:

1. **Design connection pool**:
   - Store idle connections in `Vec<TcpStream>`
   - Track in-use connections
   - Limit maximum pool size
   - Use `Arc<Mutex<PoolState>>` for thread-safe sharing

2. **Implement acquire/release**:
   - `acquire()`: Get connection from pool or create new
   - `release()`: Return connection to pool
   - Handle pool full condition
   - Close excess connections when pool is full

3. **Connection health checking**:
   - Detect dead connections before returning
   - Remove stale connections (idle timeout)
   - Periodic cleanup of old connections

4. **Pooled connection wrapper**:
   - RAII pattern: auto-return to pool on drop
   - Prevent connection leaks
   - Track connection usage

**Starter Code Extension**:

```rust
use tokio::sync::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct ConnectionPool {
    inner: Arc<Mutex<PoolState>>,
    backend_addr: String,
    max_size: usize,
    idle_timeout: Duration,
}

struct PoolState {
    idle: Vec<PooledConn>,
    active_count: usize,
}

struct PooledConn {
    stream: TcpStream,
    created_at: Instant,
    last_used: Instant,
}

impl ConnectionPool {
    pub fn new(backend_addr: String, max_size: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(PoolState {
                idle: Vec::new(),
                active_count: 0,
            })),
            backend_addr,
            max_size,
            idle_timeout: Duration::from_secs(30),
        }
    }

    /// Acquire connection from pool
    pub async fn acquire(&self) -> io::Result<PooledConnection> {
        let mut state = self.inner.lock().await;

        // TODO: Try to get idle connection
        while let Some(mut conn) = state.idle.pop() {
            // Check if connection is still alive
            if conn.is_alive().await {
                state.active_count += 1;
                drop(state); // Release lock

                return Ok(PooledConnection {
                    stream: Some(conn.stream),
                    pool: self.clone(),
                });
            }
            // Connection dead, try next
        }

        // TODO: Check if can create new connection
        if state.active_count >= self.max_size {
            // Pool exhausted, wait or error
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "Connection pool exhausted"
            ));
        }

        // TODO: Create new connection
        state.active_count += 1;
        drop(state); // Release lock before connecting

        let stream = TcpStream::connect(&self.backend_addr).await?;

        Ok(PooledConnection {
            stream: Some(stream),
            pool: self.clone(),
        })
    }

    /// Return connection to pool
    async fn release(&self, stream: TcpStream) {
        let mut state = self.inner.lock().await;
        state.active_count -= 1;

        // TODO: Check if pool has space
        if state.idle.len() < self.max_size {
            state.idle.push(PooledConn {
                stream,
                created_at: Instant::now(),
                last_used: Instant::now(),
            });
        } else {
            // Pool full, drop connection
            drop(stream);
        }
    }

    /// Remove stale connections
    pub async fn cleanup_stale(&self) {
        let mut state = self.inner.lock().await;
        let now = Instant::now();

        state.idle.retain(|conn| {
            now.duration_since(conn.last_used) < self.idle_timeout
        });
    }
}

/// RAII wrapper that returns connection to pool on drop
pub struct PooledConnection {
    stream: Option<TcpStream>,
    pool: ConnectionPool,
}

impl PooledConnection {
    pub fn stream(&mut self) -> &mut TcpStream {
        self.stream.as_mut().unwrap()
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(stream) = self.stream.take() {
            let pool = self.pool.clone();
            tokio::spawn(async move {
                pool.release(stream).await;
            });
        }
    }
}

impl PooledConn {
    async fn is_alive(&mut self) -> bool {
        // TODO: Check if connection is still alive
        // Try to peek at socket to see if it's readable
        // If readable with 0 bytes, connection closed

        // Simple check: try to set nodelay (will fail if closed)
        self.stream.set_nodelay(true).is_ok()
    }
}
```

**Checkpoint Tests**:

```rust
#[tokio::test]
async fn test_pool_acquire_release() {
    // Start backend
    tokio::spawn(async {
        let listener = TcpListener::bind("127.0.0.1:9002").await.unwrap();
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            // Keep connection open
            tokio::time::sleep(Duration::from_secs(100)).await;
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let pool = ConnectionPool::new("127.0.0.1:9002".to_string(), 5);

    // Acquire connection
    let conn1 = pool.acquire().await.unwrap();

    // Connection count should be 1
    let state = pool.inner.lock().await;
    assert_eq!(state.active_count, 1);
    drop(state);

    // Release connection (via drop)
    drop(conn1);

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Should be back in pool
    let state = pool.inner.lock().await;
    assert_eq!(state.idle.len(), 1);
    assert_eq!(state.active_count, 0);
}

#[tokio::test]
async fn test_pool_reuse() {
    // Start backend
    // ...

    let pool = ConnectionPool::new("127.0.0.1:9002".to_string(), 5);

    // Acquire and release
    let conn1 = pool.acquire().await.unwrap();
    drop(conn1);

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Second acquire should reuse connection
    let conn2 = pool.acquire().await.unwrap();

    // Should still only have created 1 connection total
}

#[tokio::test]
async fn test_pool_max_size() {
    // Start backend
    // ...

    let pool = ConnectionPool::new("127.0.0.1:9002".to_string(), 2);

    let conn1 = pool.acquire().await.unwrap();
    let conn2 = pool.acquire().await.unwrap();

    // Pool exhausted
    let result = pool.acquire().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cleanup_stale_connections() {
    // Start backend
    // ...

    let mut pool = ConnectionPool::new("127.0.0.1:9002".to_string(), 5);
    pool.idle_timeout = Duration::from_millis(100);

    let conn = pool.acquire().await.unwrap();
    drop(conn);

    tokio::time::sleep(Duration::from_millis(200)).await;

    pool.cleanup_stale().await;

    let state = pool.inner.lock().await;
    assert_eq!(state.idle.len(), 0); // Stale connection removed
}
```

**Check Your Understanding**:
- Why use `Arc<Mutex<>>` for the pool?
- How does RAII pattern prevent connection leaks?
- Why check if connection is alive before returning?
- What happens if we don't limit pool size?

---

### Milestone 4: Backpressure and Timeout Handling

**Goal**: Prevent memory exhaustion and handle slow clients/backends.

**Implementation Steps**:

1. **Implement backpressure with bounded channels**:
   - Use `tokio::sync::mpsc::channel(capacity)`
   - Limit pending requests in flight
   - Apply backpressure when queue is full
   - Return 503 Service Unavailable to client

2. **Client timeout handling**:
   - Timeout on reading client request
   - Close connection if client is too slow
   - Prevent slow loris attacks

3. **Backend timeout handling**:
   - Timeout on backend connection
   - Timeout on backend response
   - Return 504 Gateway Timeout to client

4. **Request queueing**:
   - Queue requests when all connections busy
   - Process requests in order
   - Shed load when queue is full

**Starter Code Extension**:

```rust
use tokio::time::{timeout, Duration};
use tokio::sync::mpsc;

const MAX_PENDING_REQUESTS: usize = 100;
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);
const BACKEND_TIMEOUT: Duration = Duration::from_secs(30);

/// Handle HTTP request with timeouts and backpressure
pub async fn handle_http_request(
    mut client: TcpStream,
    pool: ConnectionPool,
    request_queue: mpsc::Sender<()>,
) -> io::Result<()> {
    // TODO: Apply backpressure - try to acquire queue slot
    let _permit = match request_queue.try_send(()) {
        Ok(_) => (),
        Err(_) => {
            // Queue full, return 503
            send_503_response(&mut client).await?;
            return Ok(());
        }
    };

    // TODO: Read request with timeout
    let request = match timeout(
        CLIENT_TIMEOUT,
        read_http_request(&mut client)
    ).await {
        Ok(Ok(req)) => req,
        Ok(Err(e)) => {
            eprintln!("Error reading request: {}", e);
            return Err(e);
        }
        Err(_) => {
            eprintln!("Client timeout");
            send_408_response(&mut client).await?;
            return Ok(());
        }
    };

    // TODO: Acquire connection from pool
    let mut backend_conn = pool.acquire().await?;

    // TODO: Forward request to backend with timeout
    let response = match timeout(
        BACKEND_TIMEOUT,
        forward_to_backend(&mut backend_conn, &request)
    ).await {
        Ok(Ok(resp)) => resp,
        Ok(Err(e)) => {
            eprintln!("Backend error: {}", e);
            send_502_response(&mut client).await?;
            return Ok(());
        }
        Err(_) => {
            eprintln!("Backend timeout");
            send_504_response(&mut client).await?;
            return Ok(());
        }
    };

    // TODO: Send response to client
    client.write_all(&response.to_bytes()).await?;

    Ok(())
}

async fn send_503_response(client: &mut TcpStream) -> io::Result<()> {
    let response = b"HTTP/1.1 503 Service Unavailable\r\n\
                     Content-Length: 0\r\n\
                     \r\n";
    client.write_all(response).await
}

async fn send_408_response(client: &mut TcpStream) -> io::Result<()> {
    let response = b"HTTP/1.1 408 Request Timeout\r\n\
                     Content-Length: 0\r\n\
                     \r\n";
    client.write_all(response).await
}

async fn send_502_response(client: &mut TcpStream) -> io::Result<()> {
    let response = b"HTTP/1.1 502 Bad Gateway\r\n\
                     Content-Length: 0\r\n\
                     \r\n";
    client.write_all(response).await
}

async fn send_504_response(client: &mut TcpStream) -> io::Result<()> {
    let response = b"HTTP/1.1 504 Gateway Timeout\r\n\
                     Content-Length: 0\r\n\
                     \r\n";
    client.write_all(response).await
}

async fn read_http_request(client: &mut TcpStream) -> io::Result<HttpRequest> {
    let mut reader = BufReader::new(client);
    HttpRequest::parse(&mut reader).await
}

async fn forward_to_backend(
    backend: &mut PooledConnection,
    request: &HttpRequest,
) -> io::Result<HttpResponse> {
    // Write request
    backend.stream().write_all(&request.to_bytes()).await?;

    // Read response
    let mut reader = BufReader::new(backend.stream());
    HttpResponse::parse(&mut reader).await
}
```

**Checkpoint Tests**:

```rust
#[tokio::test]
async fn test_client_timeout() {
    // Client that sends request slowly
    // Should receive 408 timeout
}

#[tokio::test]
async fn test_backend_timeout() {
    // Backend that responds slowly
    // Should receive 504 gateway timeout
}

#[tokio::test]
async fn test_backpressure_503() {
    // Fill request queue
    // Next request should get 503
}

#[tokio::test]
async fn test_successful_request_with_timeouts() {
    // Normal request within timeouts
    // Should succeed
}
```

**Check Your Understanding**:
- Why use bounded channels for backpressure?
- What happens if we don't timeout slow clients?
- How does `timeout()` work internally?
- Why different timeouts for client vs backend?

---

### Milestone 5: Health Checks and Graceful Shutdown

**Goal**: Complete production-ready proxy with health checks and clean shutdown.

**Implementation Steps**:

1. **Periodic health checks**:
   - Spawn background task that checks backend health
   - Mark backends as healthy/unhealthy
   - Remove unhealthy backends from rotation
   - Re-add when they recover

2. **Graceful shutdown**:
   - Listen for SIGTERM/SIGINT
   - Stop accepting new connections
   - Drain in-flight requests
   - Close all connections cleanly
   - Use `CancellationToken`

3. **Metrics and monitoring**:
   - Track active connections
   - Track request rate
   - Track error rate
   - Track backend latency

4. **Multiple backend support**:
   - Round-robin load balancing
   - Health-based routing
   - Connection pool per backend

**Final Implementation**:

```rust
use tokio_util::sync::CancellationToken;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct ProxyServer {
    listen_addr: String,
    backends: Vec<Backend>,
    shutdown_token: CancellationToken,
    metrics: Arc<ProxyMetrics>,
}

struct Backend {
    addr: String,
    pool: ConnectionPool,
    healthy: Arc<Mutex<bool>>,
}

#[derive(Default)]
struct ProxyMetrics {
    requests_total: AtomicU64,
    requests_failed: AtomicU64,
    active_connections: AtomicU64,
}

impl ProxyServer {
    pub fn new(listen_addr: String, backend_addrs: Vec<String>) -> Self {
        let backends = backend_addrs.into_iter().map(|addr| {
            Backend {
                pool: ConnectionPool::new(addr.clone(), 10),
                addr,
                healthy: Arc::new(Mutex::new(true)),
            }
        }).collect();

        Self {
            listen_addr,
            backends,
            shutdown_token: CancellationToken::new(),
            metrics: Arc::new(ProxyMetrics::default()),
        }
    }

    pub async fn run(self) -> io::Result<()> {
        let listener = TcpListener::bind(&self.listen_addr).await?;
        println!("Proxy listening on {}", self.listen_addr);

        // Start health check task
        let health_check_token = self.shutdown_token.clone();
        let backends = self.backends.clone();
        tokio::spawn(async move {
            Self::health_check_loop(backends, health_check_token).await;
        });

        // Start metrics task
        let metrics_token = self.shutdown_token.clone();
        let metrics = self.metrics.clone();
        tokio::spawn(async move {
            Self::metrics_loop(metrics, metrics_token).await;
        });

        // Accept connections
        loop {
            tokio::select! {
                result = listener.accept() => {
                    let (client, addr) = result?;
                    println!("Accepted connection from {}", addr);

                    self.metrics.active_connections.fetch_add(1, Ordering::Relaxed);
                    self.metrics.requests_total.fetch_add(1, Ordering::Relaxed);

                    let backend = self.select_backend().await;
                    let metrics = self.metrics.clone();

                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(client, backend).await {
                            eprintln!("Connection error: {}", e);
                            metrics.requests_failed.fetch_add(1, Ordering::Relaxed);
                        }
                        metrics.active_connections.fetch_sub(1, Ordering::Relaxed);
                    });
                }
                _ = self.shutdown_token.cancelled() => {
                    println!("Shutdown signal received");
                    break;
                }
            }
        }

        println!("Proxy shutdown complete");
        Ok(())
    }

    async fn select_backend(&self) -> Backend {
        // TODO: Round-robin or random selection
        // TODO: Skip unhealthy backends
        // For now, simple round-robin

        // Filter healthy backends
        let mut healthy_backends = Vec::new();
        for backend in &self.backends {
            if *backend.healthy.lock().await {
                healthy_backends.push(backend.clone());
            }
        }

        if healthy_backends.is_empty() {
            // All backends unhealthy, use first one anyway
            self.backends[0].clone()
        } else {
            // Simple random selection
            use rand::Rng;
            let idx = rand::thread_rng().gen_range(0..healthy_backends.len());
            healthy_backends[idx].clone()
        }
    }

    async fn health_check_loop(
        backends: Vec<Backend>,
        cancel_token: CancellationToken,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    for backend in &backends {
                        let is_healthy = Self::check_backend_health(&backend.addr).await;
                        *backend.healthy.lock().await = is_healthy;

                        println!("Backend {} health: {}",
                            backend.addr,
                            if is_healthy { "healthy" } else { "unhealthy" }
                        );
                    }
                }
                _ = cancel_token.cancelled() => {
                    println!("Health check shutting down");
                    break;
                }
            }
        }
    }

    async fn check_backend_health(addr: &str) -> bool {
        // Try to connect to backend
        match timeout(Duration::from_secs(5), TcpStream::connect(addr)).await {
            Ok(Ok(_)) => true,
            _ => false,
        }
    }

    async fn metrics_loop(
        metrics: Arc<ProxyMetrics>,
        cancel_token: CancellationToken,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    println!("Metrics:");
                    println!("  Total requests: {}",
                        metrics.requests_total.load(Ordering::Relaxed));
                    println!("  Failed requests: {}",
                        metrics.requests_failed.load(Ordering::Relaxed));
                    println!("  Active connections: {}",
                        metrics.active_connections.load(Ordering::Relaxed));
                }
                _ = cancel_token.cancelled() => {
                    println!("Metrics shutting down");
                    break;
                }
            }
        }
    }

    pub fn shutdown(&self) {
        self.shutdown_token.cancel();
    }
}

// Signal handling for graceful shutdown
pub async fn setup_signal_handlers(server: Arc<ProxyServer>) {
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("Ctrl+C received, shutting down...");
            server.shutdown();
        }
    }
}
```

**Checkpoint Tests**:

```rust
#[tokio::test]
async fn test_graceful_shutdown() {
    let server = ProxyServer::new(
        "127.0.0.1:8003".to_string(),
        vec!["127.0.0.1:9003".to_string()],
    );

    let server_handle = tokio::spawn(async move {
        server.run().await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Trigger shutdown
    server.shutdown();

    // Server should stop accepting connections
    tokio::time::sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_health_check_marks_backend_unhealthy() {
    // Start proxy with backend that goes down
    // Health check should mark it unhealthy
    // Requests should be routed to other backends
}

#[tokio::test]
async fn test_metrics_tracking() {
    // Make requests
    // Check metrics are updated correctly
}
```

**Check Your Understanding**:
- How does `CancellationToken` enable graceful shutdown?
- Why run health checks in background task?
- How do atomic operations work for metrics?
- What's the difference between graceful and forceful shutdown?

---

### Complete Project Summary

**What You Built**:
1. Async TCP proxy with bidirectional forwarding
2. HTTP/1.1 request/response parser
3. Connection pool with health checks
4. Backpressure handling with bounded channels
5. Timeout handling for clients and backends
6. Graceful shutdown with signal handling
7. Multiple backend support with load balancing

**Key Concepts Practiced**:
- Async networking (`TcpListener`, `TcpStream`, `tokio::spawn`)
- Connection pooling for resource reuse
- Buffered I/O (`BufReader`, `BufWriter`)
- Backpressure with bounded channels
- Timeout handling (`tokio::time::timeout`)
- Health checks and monitoring
- Graceful shutdown (`CancellationToken`)

**Production Patterns**:
- Connection reuse (150x latency improvement)
- Load shedding (503 when overloaded)
- Circuit breaking (health checks)
- Observability (metrics)
- Clean shutdown (no dropped requests)

**Real-World Applications**:
- Reverse proxies (NGINX, HAProxy)
- API gateways (Kong, Tyk)
- Service mesh (Istio, Linkerd)
- Load balancers (AWS ALB, Traefik)
- CDN edge servers

**Extension Ideas**:
1. **TLS support**: `tokio-rustls` for HTTPS
2. **HTTP/2**: Use `h2` crate
3. **WebSocket**: Upgrade connections
4. **Caching**: Cache responses in memory/Redis
5. **Rate limiting**: Per-client rate limits
6. **Authentication**: JWT validation
7. **Request transformation**: Modify headers/body
8. **Logging**: Structured logging with tracing
9. **Distributed tracing**: OpenTelemetry
10. **Configuration**: Hot reload without restart

This project teaches production async patterns used in real proxies, load balancers, and API gateways!
