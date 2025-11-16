# 17. Async I/O Patterns

## Tokio File and Network I/O

### Basic Async File Operations

```rust
// Note: Add to Cargo.toml:
// [dependencies]
// tokio = { version = "1", features = ["full"] }

use tokio::fs::{File, OpenOptions};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

// Read entire file
async fn read_file(path: &str) -> io::Result<String> {
    tokio::fs::read_to_string(path).await
}

// Read file to bytes
async fn read_bytes(path: &str) -> io::Result<Vec<u8>> {
    tokio::fs::read(path).await
}

// Write to file
async fn write_file(path: &str, content: &str) -> io::Result<()> {
    tokio::fs::write(path, content).await
}

// Manual read with buffer
async fn read_with_buffer(path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;
    Ok(buffer)
}

// Manual write
async fn write_with_handle(path: &str, data: &[u8]) -> io::Result<()> {
    let mut file = File::create(path).await?;
    file.write_all(data).await?;
    file.flush().await?;
    Ok(())
}

// Append to file
async fn append_to_file(path: &str, content: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .await?;

    file.write_all(content.as_bytes()).await?;
    file.write_all(b"\n").await?;
    Ok(())
}

// Copy file asynchronously
async fn copy_file(src: &str, dst: &str) -> io::Result<u64> {
    tokio::fs::copy(src, dst).await
}

// Example usage
#[tokio::main]
async fn main() -> io::Result<()> {
    let content = read_file("example.txt").await?;
    println!("File content: {}", content);

    write_file("output.txt", "Hello, async!").await?;

    Ok(())
}
```

### Async Line Reading

```rust
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, BufReader};

// Read lines from file
async fn read_lines(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    let mut line_stream = reader.lines();
    while let Some(line) = line_stream.next_line().await? {
        lines.push(line);
    }

    Ok(lines)
}

// Process large file line by line
async fn process_large_file(path: &str) -> io::Result<()> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut count = 0;
    while let Some(line) = lines.next_line().await? {
        if !line.starts_with('#') {
            // Process non-comment lines
            count += 1;
        }
    }

    println!("Processed {} lines", count);
    Ok(())
}

// Read lines with limit
async fn read_first_n_lines(path: &str, n: usize) -> io::Result<Vec<String>> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut result = Vec::new();

    for _ in 0..n {
        if let Some(line) = lines.next_line().await? {
            result.push(line);
        } else {
            break;
        }
    }

    Ok(result)
}
```

### TCP Network I/O

```rust
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

// TCP Server
async fn run_tcp_server(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Server listening on {}", addr);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);

        // Spawn a task for each connection
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
}

async fn handle_client(mut socket: TcpStream) -> io::Result<()> {
    let mut buffer = [0; 1024];

    loop {
        let n = socket.read(&mut buffer).await?;

        if n == 0 {
            // Connection closed
            return Ok(());
        }

        // Echo back
        socket.write_all(&buffer[..n]).await?;
    }
}

// TCP Client
async fn tcp_client(addr: &str, message: &str) -> io::Result<String> {
    let mut stream = TcpStream::connect(addr).await?;

    // Send message
    stream.write_all(message.as_bytes()).await?;

    // Read response
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;

    Ok(String::from_utf8_lossy(&buffer).to_string())
}

// HTTP-like request handling
async fn http_handler(mut socket: TcpStream) -> io::Result<()> {
    let mut buffer = [0; 4096];
    let n = socket.read(&mut buffer).await?;

    let request = String::from_utf8_lossy(&buffer[..n]);
    println!("Request: {}", request);

    let response = "HTTP/1.1 200 OK\r\n\
                   Content-Type: text/plain\r\n\
                   Content-Length: 13\r\n\
                   \r\n\
                   Hello, World!";

    socket.write_all(response.as_bytes()).await?;
    Ok(())
}
```

### UDP Network I/O

```rust
use tokio::net::UdpSocket;
use tokio::io;

// UDP Echo Server
async fn udp_server(addr: &str) -> io::Result<()> {
    let socket = UdpSocket::bind(addr).await?;
    println!("UDP server listening on {}", addr);

    let mut buffer = [0; 1024];

    loop {
        let (len, addr) = socket.recv_from(&mut buffer).await?;
        println!("Received {} bytes from {}", len, addr);

        // Echo back
        socket.send_to(&buffer[..len], addr).await?;
    }
}

// UDP Client
async fn udp_client(server_addr: &str, message: &str) -> io::Result<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    socket.send_to(message.as_bytes(), server_addr).await?;

    let mut buffer = [0; 1024];
    let (len, _) = socket.recv_from(&mut buffer).await?;

    Ok(String::from_utf8_lossy(&buffer[..len]).to_string())
}
```

### Unix Domain Sockets

```rust
#[cfg(unix)]
mod unix_sockets {
    use tokio::net::{UnixListener, UnixStream};
    use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

    pub async fn unix_server(path: &str) -> io::Result<()> {
        // Remove old socket file if exists
        let _ = std::fs::remove_file(path);

        let listener = UnixListener::bind(path)?;
        println!("Unix socket server listening on {}", path);

        loop {
            let (mut socket, _) = listener.accept().await?;

            tokio::spawn(async move {
                let mut buffer = [0; 1024];

                while let Ok(n) = socket.read(&mut buffer).await {
                    if n == 0 {
                        break;
                    }
                    let _ = socket.write_all(&buffer[..n]).await;
                }
            });
        }
    }

    pub async fn unix_client(path: &str, message: &str) -> io::Result<String> {
        let mut stream = UnixStream::connect(path).await?;

        stream.write_all(message.as_bytes()).await?;

        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).await?;

        Ok(String::from_utf8_lossy(&buffer).to_string())
    }
}
```

## Buffered Async Streams

### Using BufReader and BufWriter

```rust
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

// Buffered reading
async fn buffered_read(path: &str) -> io::Result<()> {
    let file = File::open(path).await?;
    let reader = BufReader::with_capacity(8192, file);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        println!("{}", line);
    }

    Ok(())
}

// Buffered writing
async fn buffered_write(path: &str, lines: &[&str]) -> io::Result<()> {
    let file = File::create(path).await?;
    let mut writer = BufWriter::with_capacity(8192, file);

    for line in lines {
        writer.write_all(line.as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }

    writer.flush().await?;
    Ok(())
}

// Copy with buffering
async fn buffered_copy(src: &str, dst: &str) -> io::Result<u64> {
    let src_file = File::open(src).await?;
    let dst_file = File::create(dst).await?;

    let mut reader = BufReader::new(src_file);
    let mut writer = BufWriter::new(dst_file);

    tokio::io::copy(&mut reader, &mut writer).await
}
```

### Stream Processing with AsyncRead/AsyncWrite

```rust
use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use std::pin::Pin;
use std::task::{Context, Poll};

// Custom async reader that uppercases data
struct UppercaseReader<R> {
    inner: R,
}

impl<R: AsyncRead + Unpin> AsyncRead for UppercaseReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let before_len = buf.filled().len();

        match Pin::new(&mut self.inner).poll_read(cx, buf) {
            Poll::Ready(Ok(())) => {
                // Uppercase the newly read bytes
                let filled = buf.filled_mut();
                for byte in &mut filled[before_len..] {
                    if byte.is_ascii_lowercase() {
                        *byte = byte.to_ascii_uppercase();
                    }
                }
                Poll::Ready(Ok(()))
            }
            other => other,
        }
    }
}

// Usage
async fn use_uppercase_reader() -> io::Result<()> {
    use tokio::fs::File;

    let file = File::open("input.txt").await?;
    let mut reader = UppercaseReader { inner: file };

    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).await?;
    println!("Uppercased: {}", buffer);

    Ok(())
}
```

### Stream Splitting and Framing

```rust
use tokio::net::TcpStream;
use tokio::io::{self, AsyncRead, AsyncWrite, BufReader};

// Split stream into read and write halves
async fn split_stream_example() -> io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let (read_half, write_half) = stream.into_split();

    // Spawn reader task
    let reader_task = tokio::spawn(async move {
        let mut reader = BufReader::new(read_half);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            println!("Received: {}", line);
        }

        Ok::<_, io::Error>(())
    });

    // Spawn writer task
    let writer_task = tokio::spawn(async move {
        let mut writer = write_half;

        for i in 0..10 {
            writer.write_all(format!("Message {}\n", i).as_bytes()).await?;
        }

        Ok::<_, io::Error>(())
    });

    reader_task.await??;
    writer_task.await??;

    Ok(())
}
```

### Using tokio_util::codec for Framing

```rust
// Add to Cargo.toml:
// tokio-util = { version = "0.7", features = ["codec"] }

use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};
use futures::{SinkExt, StreamExt};

// Line-delimited codec
async fn framed_lines() -> io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let mut framed = Framed::new(stream, LinesCodec::new());

    // Send lines
    framed.send("Hello, World!".to_string()).await?;
    framed.send("Another line".to_string()).await?;

    // Receive lines
    while let Some(result) = framed.next().await {
        match result {
            Ok(line) => println!("Received: {}", line),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}

// Custom codec for length-prefixed messages
use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

struct LengthPrefixedCodec;

impl Decoder for LengthPrefixedCodec {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None); // Need more data
        }

        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_be_bytes(length_bytes) as usize;

        if src.len() < 4 + length {
            return Ok(None); // Need more data
        }

        src.advance(4);
        let data = src.split_to(length).to_vec();
        Ok(Some(data))
    }
}

impl Encoder<Vec<u8>> for LengthPrefixedCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Vec<u8>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let length = item.len() as u32;
        dst.put_u32(length);
        dst.put_slice(&item);
        Ok(())
    }
}

async fn length_prefixed_example() -> io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let mut framed = Framed::new(stream, LengthPrefixedCodec);

    // Send message
    framed.send(b"Hello, World!".to_vec()).await?;

    // Receive message
    if let Some(result) = framed.next().await {
        let data = result?;
        println!("Received {} bytes", data.len());
    }

    Ok(())
}
```

## Backpressure Handling

### Manual Backpressure with Channels

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

// Producer with backpressure
async fn producer_with_backpressure(tx: mpsc::Sender<i32>) {
    for i in 0..100 {
        // send() provides backpressure - blocks when channel is full
        if let Err(_) = tx.send(i).await {
            println!("Receiver dropped");
            break;
        }
        println!("Sent: {}", i);
    }
}

// Consumer
async fn consumer(mut rx: mpsc::Receiver<i32>) {
    while let Some(value) = rx.recv().await {
        println!("Processing: {}", value);
        // Simulate slow processing
        sleep(Duration::from_millis(100)).await;
    }
}

// Usage with bounded channel
async fn backpressure_example() {
    let (tx, rx) = mpsc::channel(10); // Bounded channel with capacity 10

    let producer = tokio::spawn(producer_with_backpressure(tx));
    let consumer = tokio::spawn(consumer(rx));

    let _ = tokio::join!(producer, consumer);
}
```

### Stream Backpressure with buffering

```rust
use futures::stream::{self, StreamExt};
use tokio::time::{sleep, Duration};

async fn stream_backpressure() {
    let stream = stream::iter(0..100)
        .map(|i| async move {
            println!("Generating: {}", i);
            i
        })
        .buffered(5) // Process at most 5 futures concurrently
        .for_each(|value| async move {
            println!("Processing: {}", value);
            sleep(Duration::from_millis(100)).await;
        })
        .await;
}
```

### Rate Limiting

```rust
use tokio::time::{sleep, Duration, Instant};

// Simple rate limiter
struct RateLimiter {
    max_per_second: u32,
    last_reset: Instant,
    count: u32,
}

impl RateLimiter {
    fn new(max_per_second: u32) -> Self {
        RateLimiter {
            max_per_second,
            last_reset: Instant::now(),
            count: 0,
        }
    }

    async fn acquire(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_reset);

        if elapsed >= Duration::from_secs(1) {
            // Reset counter
            self.last_reset = now;
            self.count = 0;
        }

        if self.count >= self.max_per_second {
            // Wait until next second
            let wait_time = Duration::from_secs(1) - elapsed;
            sleep(wait_time).await;
            self.last_reset = Instant::now();
            self.count = 0;
        }

        self.count += 1;
    }
}

async fn rate_limited_requests() {
    let mut limiter = RateLimiter::new(10); // 10 requests per second

    for i in 0..50 {
        limiter.acquire().await;
        println!("Request {}", i);
        // Make request...
    }
}
```

### Semaphore for Concurrency Control

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

async fn concurrent_with_limit() {
    let semaphore = Arc::new(Semaphore::new(5)); // Max 5 concurrent tasks

    let mut handles = vec![];

    for i in 0..20 {
        let permit = semaphore.clone().acquire_owned().await.unwrap();

        let handle = tokio::spawn(async move {
            println!("Task {} started", i);
            sleep(Duration::from_secs(1)).await;
            println!("Task {} completed", i);
            drop(permit); // Release permit
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
```

### Flow Control in Network Servers

```rust
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use std::sync::Arc;

async fn server_with_connection_limit(addr: &str, max_connections: usize) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let semaphore = Arc::new(Semaphore::new(max_connections));

    println!("Server listening on {} (max {} connections)", addr, max_connections);

    loop {
        let (socket, addr) = listener.accept().await?;

        // Try to acquire permit
        let permit = semaphore.clone().acquire_owned().await.unwrap();

        println!("New connection from {} ({} slots available)",
                 addr,
                 semaphore.available_permits());

        tokio::spawn(async move {
            // Handle connection
            let _ = handle_client(socket).await;
            drop(permit); // Release slot when done
        });
    }
}
```

## Connection Pooling

### Simple Connection Pool

```rust
use tokio::sync::Mutex;
use std::sync::Arc;

struct Connection {
    id: usize,
}

impl Connection {
    async fn new(id: usize) -> io::Result<Self> {
        // Simulate connection setup
        sleep(Duration::from_millis(100)).await;
        Ok(Connection { id })
    }

    async fn execute(&self, query: &str) -> io::Result<String> {
        println!("Connection {} executing: {}", self.id, query);
        sleep(Duration::from_millis(50)).await;
        Ok(format!("Result from connection {}", self.id))
    }
}

struct SimplePool {
    connections: Arc<Mutex<Vec<Connection>>>,
    max_size: usize,
}

impl SimplePool {
    async fn new(size: usize) -> io::Result<Self> {
        let mut connections = Vec::new();

        for i in 0..size {
            connections.push(Connection::new(i).await?);
        }

        Ok(SimplePool {
            connections: Arc::new(Mutex::new(connections)),
            max_size: size,
        })
    }

    async fn acquire(&self) -> io::Result<PooledConnection> {
        loop {
            let mut pool = self.connections.lock().await;

            if let Some(conn) = pool.pop() {
                return Ok(PooledConnection {
                    conn: Some(conn),
                    pool: self.connections.clone(),
                });
            }

            drop(pool);
            sleep(Duration::from_millis(10)).await;
        }
    }
}

struct PooledConnection {
    conn: Option<Connection>,
    pool: Arc<Mutex<Vec<Connection>>>,
}

impl PooledConnection {
    async fn execute(&self, query: &str) -> io::Result<String> {
        self.conn.as_ref().unwrap().execute(query).await
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            let pool = self.pool.clone();
            tokio::spawn(async move {
                pool.lock().await.push(conn);
            });
        }
    }
}

// Usage
async fn use_pool() -> io::Result<()> {
    let pool = SimplePool::new(5).await?;

    let mut handles = vec![];

    for i in 0..20 {
        let pool = pool.clone();
        let handle = tokio::spawn(async move {
            let conn = pool.acquire().await.unwrap();
            let result = conn.execute(&format!("Query {}", i)).await.unwrap();
            println!("{}", result);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}
```

### Advanced Pool with deadpool

```rust
// Add to Cargo.toml:
// deadpool = "0.10"

use deadpool::managed::{Manager, Pool, RecycleResult};
use async_trait::async_trait;

struct MyConnection {
    id: usize,
}

struct MyManager {
    next_id: Arc<Mutex<usize>>,
}

#[async_trait]
impl Manager for MyManager {
    type Type = MyConnection;
    type Error = io::Error;

    async fn create(&self) -> Result<MyConnection, io::Error> {
        let mut id = self.next_id.lock().await;
        let conn = MyConnection { id: *id };
        *id += 1;
        println!("Created connection {}", conn.id);
        Ok(conn)
    }

    async fn recycle(&self, conn: &mut MyConnection) -> RecycleResult<io::Error> {
        println!("Recycling connection {}", conn.id);
        Ok(())
    }
}

async fn use_deadpool() -> io::Result<()> {
    let manager = MyManager {
        next_id: Arc::new(Mutex::new(0)),
    };

    let pool = Pool::builder(manager)
        .max_size(5)
        .build()
        .unwrap();

    let mut handles = vec![];

    for i in 0..20 {
        let pool = pool.clone();
        let handle = tokio::spawn(async move {
            let conn = pool.get().await.unwrap();
            println!("Using connection {} for task {}", conn.id, i);
            sleep(Duration::from_millis(100)).await;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}
```

### HTTP Client Connection Pool

```rust
// Add to Cargo.toml:
// reqwest = "0.11"

use reqwest::Client;
use std::time::Duration;

async fn http_connection_pool() -> Result<(), Box<dyn std::error::Error>> {
    // reqwest Client has built-in connection pooling
    let client = Client::builder()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(90))
        .timeout(Duration::from_secs(30))
        .build()?;

    let mut handles = vec![];

    for i in 0..50 {
        let client = client.clone();
        let handle = tokio::spawn(async move {
            let response = client
                .get(&format!("https://api.example.com/item/{}", i))
                .send()
                .await;

            match response {
                Ok(resp) => println!("Request {}: Status {}", i, resp.status()),
                Err(e) => eprintln!("Request {} failed: {}", i, e),
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    Ok(())
}
```

## Timeout and Cancellation

### Basic Timeouts

```rust
use tokio::time::{timeout, Duration};

async fn with_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let result = timeout(
        Duration::from_secs(5),
        async_operation(),
    ).await;

    match result {
        Ok(value) => println!("Operation completed: {:?}", value),
        Err(_) => println!("Operation timed out"),
    }

    Ok(())
}

async fn async_operation() -> io::Result<String> {
    sleep(Duration::from_secs(10)).await;
    Ok("Done".to_string())
}
```

### Timeout with Fallback

```rust
use tokio::time::{timeout, Duration};

async fn timeout_with_fallback() -> String {
    let result = timeout(
        Duration::from_secs(2),
        fetch_data_from_primary(),
    ).await;

    match result {
        Ok(Ok(data)) => data,
        _ => {
            println!("Primary failed, trying fallback");
            fetch_data_from_fallback().await.unwrap_or_default()
        }
    }
}

async fn fetch_data_from_primary() -> io::Result<String> {
    sleep(Duration::from_secs(5)).await;
    Ok("Primary data".to_string())
}

async fn fetch_data_from_fallback() -> io::Result<String> {
    sleep(Duration::from_millis(500)).await;
    Ok("Fallback data".to_string())
}
```

### Cancellation with CancellationToken

```rust
// Add to Cargo.toml:
// tokio-util = "0.7"

use tokio_util::sync::CancellationToken;

async fn cancellable_operation(token: CancellationToken) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                println!("Operation cancelled");
                break;
            }
            _ = interval.tick() => {
                println!("Working...");
            }
        }
    }
}

async fn cancellation_example() {
    let token = CancellationToken::new();
    let worker_token = token.clone();

    let worker = tokio::spawn(async move {
        cancellable_operation(worker_token).await;
    });

    // Let it run for 5 seconds
    sleep(Duration::from_secs(5)).await;

    // Cancel the operation
    token.cancel();

    worker.await.unwrap();
}
```

### Select for Racing Operations

```rust
use tokio::time::{sleep, Duration};

async fn race_operations() -> String {
    tokio::select! {
        result = operation_a() => result,
        result = operation_b() => result,
        result = operation_c() => result,
    }
}

async fn operation_a() -> String {
    sleep(Duration::from_secs(3)).await;
    "A completed".to_string()
}

async fn operation_b() -> String {
    sleep(Duration::from_secs(1)).await;
    "B completed".to_string()
}

async fn operation_c() -> String {
    sleep(Duration::from_secs(2)).await;
    "C completed".to_string()
}

// Biased select (checks branches in order)
async fn biased_select() {
    let mut count = 0;

    loop {
        tokio::select! {
            biased;

            _ = sleep(Duration::from_millis(100)) => {
                count += 1;
                if count >= 10 {
                    break;
                }
            }
            _ = async { println!("Other branch") } => {}
        }
    }
}
```

### Graceful Shutdown

```rust
use tokio::signal;
use tokio::sync::broadcast;

async fn graceful_shutdown_server() -> io::Result<()> {
    let (shutdown_tx, _) = broadcast::channel(1);
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    println!("Server running. Press Ctrl+C to shutdown.");

    // Spawn shutdown signal handler
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        println!("\nShutdown signal received");
        let _ = shutdown_tx_clone.send(());
    });

    loop {
        let mut shutdown_rx = shutdown_tx.subscribe();

        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((socket, addr)) => {
                        println!("New connection from {}", addr);
                        let shutdown_rx = shutdown_tx.subscribe();

                        tokio::spawn(async move {
                            let _ = handle_connection_with_shutdown(socket, shutdown_rx).await;
                        });
                    }
                    Err(e) => eprintln!("Accept error: {}", e),
                }
            }
            _ = shutdown_rx.recv() => {
                println!("Server shutting down");
                break;
            }
        }
    }

    println!("Server stopped");
    Ok(())
}

async fn handle_connection_with_shutdown(
    mut socket: TcpStream,
    mut shutdown: broadcast::Receiver<()>,
) -> io::Result<()> {
    let mut buffer = [0; 1024];

    loop {
        tokio::select! {
            result = socket.read(&mut buffer) => {
                let n = result?;
                if n == 0 {
                    return Ok(());
                }
                socket.write_all(&buffer[..n]).await?;
            }
            _ = shutdown.recv() => {
                println!("Connection closing due to shutdown");
                return Ok(());
            }
        }
    }
}
```

### Timeout for Multiple Operations

```rust
use tokio::time::{timeout, Duration};
use futures::future::join_all;

async fn timeout_multiple() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let operations = vec![
        async_task(1),
        async_task(2),
        async_task(3),
    ];

    // Timeout for all operations collectively
    let result = timeout(
        Duration::from_secs(5),
        join_all(operations),
    ).await?;

    Ok(result.into_iter().map(|r| r.unwrap()).collect())
}

async fn async_task(id: usize) -> io::Result<String> {
    sleep(Duration::from_secs(1)).await;
    Ok(format!("Task {} completed", id))
}

// Individual timeouts for each operation
async fn individual_timeouts() -> Vec<Result<String, tokio::time::error::Elapsed>> {
    let operations = vec![
        timeout(Duration::from_secs(1), async_task(1)),
        timeout(Duration::from_secs(2), async_task(2)),
        timeout(Duration::from_secs(3), async_task(3)),
    ];

    join_all(operations).await
        .into_iter()
        .map(|r| r.map(|inner| inner.unwrap()))
        .collect()
}
```

### Retry with Timeout

```rust
use tokio::time::{sleep, timeout, Duration};

async fn retry_with_timeout<F, Fut, T>(
    mut operation: F,
    max_retries: usize,
    timeout_duration: Duration,
) -> Result<T, Box<dyn std::error::Error>>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = io::Result<T>>,
{
    for attempt in 0..max_retries {
        match timeout(timeout_duration, operation()).await {
            Ok(Ok(result)) => return Ok(result),
            Ok(Err(e)) => {
                eprintln!("Attempt {} failed: {}", attempt + 1, e);
            }
            Err(_) => {
                eprintln!("Attempt {} timed out", attempt + 1);
            }
        }

        if attempt < max_retries - 1 {
            // Exponential backoff
            let backoff = Duration::from_millis(100 * 2_u64.pow(attempt as u32));
            sleep(backoff).await;
        }
    }

    Err("All retry attempts failed".into())
}

// Usage
async fn use_retry() -> Result<(), Box<dyn std::error::Error>> {
    let result = retry_with_timeout(
        || async {
            // Simulated unreliable operation
            if rand::random::<f32>() < 0.7 {
                Err(io::Error::new(io::ErrorKind::Other, "Random failure"))
            } else {
                Ok("Success!".to_string())
            }
        },
        5,
        Duration::from_secs(2),
    ).await?;

    println!("Result: {}", result);
    Ok(())
}
```

### Deadline-based Cancellation

```rust
use tokio::time::{sleep, timeout_at, Duration, Instant};

async fn deadline_based_processing() -> io::Result<()> {
    let deadline = Instant::now() + Duration::from_secs(10);

    let tasks = vec![
        process_item(1, deadline),
        process_item(2, deadline),
        process_item(3, deadline),
    ];

    let results = futures::future::join_all(tasks).await;

    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(value) => println!("Task {} succeeded: {}", i, value),
            Err(e) => println!("Task {} failed: {}", i, e),
        }
    }

    Ok(())
}

async fn process_item(id: usize, deadline: Instant) -> io::Result<String> {
    timeout_at(deadline, async move {
        sleep(Duration::from_secs(id as u64 * 2)).await;
        Ok(format!("Item {} processed", id))
    })
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Deadline exceeded"))?
}
```

This comprehensive guide covers all major async I/O patterns in Rust with Tokio, including file operations, network I/O, buffering, backpressure handling, connection pooling, and timeout/cancellation strategies.
