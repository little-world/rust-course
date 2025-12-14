# Async I/O Patterns
This chapter explores async I/O patterns using Tokio—handling thousands of concurrent connections, buffered streams, backpressure, connection pooling, and timeouts. Async I/O allows one thread to manage thousands of operations by yielding when blocked, solving the C10K problem.


## Pattern 1: Tokio File and Network I/O

**Problem**: Synchronous I/O blocks threads waiting for data. One thread per connection doesn't scale—10K connections need 20GB RAM (2MB stack each).

**Solution**: Use Tokio async I/O where operations return futures you `.await`. TcpListener/TcpStream for network accept/read/write.

**Why It Matters**: Single thread handles 10K connections in 10MB memory (vs 20GB threaded). Solves C10K problem—web servers serve 10K+ concurrent users.

**Use Cases**: Web servers (HTTP, HTTPS), API gateways, chat servers, WebSocket servers, game servers, HTTP clients (concurrent requests), file servers (mixing file and network I/O), concurrent file processing, real-time data pipelines.

### Example: Async File Operations

Read/write files without blocking async runtime. Concurrent file operations should parallelize.

```rust
// Note: Add to Cargo.toml:
// [dependencies]
// tokio = { version = "1", features = ["full"] }

use tokio::fs::{File, OpenOptions};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

```

### Example: Read entire file into a String
This example walks through how to read entire file into a string.

```rust
async fn read_file(path: &str) -> io::Result<String> {
    tokio::fs::read_to_string(path).await
    // Convenience method: allocates a String, reads entire file
    // Returns Err if file missing, unreadable, or contains invalid UTF-8
}

```

### Example: Read entire file into Vec<u8>
This example walks through how to read entire file into vec<u8>.

```rust
async fn read_bytes(path: &str) -> io::Result<Vec<u8>> {
    tokio::fs::read(path).await
    // Reads entire file into memory
    // More efficient than read_to_string for binary data
}

```

### Example: Write string to file (overwrites existing content)
This example walks through how to write string to file (overwrites existing content).

```rust
async fn write_file(path: &str, content: &str) -> io::Result<()> {
    tokio::fs::write(path, content).await
    // Convenience method: creates file, writes content, closes file
    // Overwrites existing file! Use append_to_file if you want to append
}

```

### Example: Manual read with buffer control
This example walks through manual read with buffer control.

```rust
async fn read_with_buffer(path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(path).await?;
    let mut buffer = Vec::new();

    // read_to_end() allocates as needed while reading
    file.read_to_end(&mut buffer).await?;
    Ok(buffer)
}

```

### Example: Manual write with explicit handle
This example walks through manual write with explicit handle.

```rust
async fn write_with_handle(path: &str, data: &[u8]) -> io::Result<()> {
    let mut file = File::create(path).await?;

    // write_all() ensures all bytes are written (loops if needed)
    file.write_all(data).await?;

    // flush() ensures buffered data reaches the OS
    // (Tokio buffers writes internally for efficiency)
    file.flush().await?;
    Ok(())
}

```

### Example: Append to existing file (or create if missing)
This example walks through how to append to existing file (or create if missing).

```rust
async fn append_to_file(path: &str, content: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)   // Append mode: writes go to end of file
        .create(true)   // Create file if it doesn't exist
        .open(path)
        .await?;

    file.write_all(content.as_bytes()).await?;
    file.write_all(b"\n").await?;  // Add newline separator
    Ok(())
}

```

### Example: Copy file asynchronously
This example walks through copy file asynchronously.

```rust
async fn copy_file(src: &str, dst: &str) -> io::Result<u64> {
    tokio::fs::copy(src, dst).await
    // Efficiently copies src to dst using OS-level optimizations when possible
}

```

### Example: Example usage
This example walks through example usage.

```rust
#[tokio::main]
async fn main() -> io::Result<()> {
    // Read file asynchronously
    let content = read_file("example.txt").await?;
    println!("File content: {}", content);

    // Write file asynchronously
    write_file("output.txt", "Hello, async!").await?;

    Ok(())
}
```


### Example: Async Line Reading

Reading line-by-line is essential for processing log files, CSV data, and other line-oriented formats. The async version uses `BufReader` to buffer reads efficiently, minimizing system calls.

**Why buffering matters:** Unbuffered reads perform one system call per byte or small chunk, which is catastrophically slow. A `BufReader` reads large chunks (8KB by default) into an internal buffer, then serves bytes from that buffer. This reduces system calls by 100-1000x, making line-by-line reading practical.

```rust
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, BufReader};

```

### Example: Read all lines into memory
This example walks through how to read all lines into memory.

```rust
async fn read_lines(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    let mut line_stream = reader.lines();
    // lines() returns a stream that yields one line at a time
    // Lines are split on \n or \r\n, with the newline removed

    while let Some(line) = line_stream.next_line().await? {
        lines.push(line);
    }

    Ok(lines)
}

```

### Example: Process large file line by line without loading into memory
This example walks through process large file line by line without loading into memory.

```rust
async fn process_large_file(path: &str) -> io::Result<()> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut count = 0;
    while let Some(line) = lines.next_line().await? {
        if !line.starts_with('#') {
            // Process non-comment lines
            // Each line is processed and dropped before reading the next
            count += 1;
        }
    }

    println!("Processed {} lines", count);
    Ok(())
}

```

### Example: Read first N lines (useful for previews or headers)
This example walks through how to read first n lines (useful for previews or headers).

```rust
async fn read_first_n_lines(path: &str, n: usize) -> io::Result<Vec<String>> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut result = Vec::new();

    for _ in 0..n {
        if let Some(line) = lines.next_line().await? {
            result.push(line);
        } else {
            break;  // File has fewer than n lines
        }
    }

    Ok(result)
}
```


### Example: TCP Network I/O

TCP (Transmission Control Protocol) provides reliable, ordered, connection-oriented communication. Tokio's TCP implementation is truly async—it uses the OS's non-blocking I/O facilities (epoll on Linux, kqueue on BSD/macOS, IOCP on Windows) to wait for data without blocking threads.

**The TCP Server Pattern**

A typical async TCP server follows this pattern:
1. Bind a `TcpListener` to an address
2. Loop forever, accepting connections with `.accept().await`
3. For each connection, spawn a new task with `tokio::spawn`
4. Each task handles one client independently
5. When a client disconnects, its task completes and is cleaned up

This pattern allows one thread to handle thousands of concurrent connections. The runtime multiplexes all connections on a small thread pool.

```rust
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

```

### Example: TCP Echo Server
This example walks through tcp echo server.

```rust
async fn run_tcp_server(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Server listening on {}", addr);

    loop {
        // accept() awaits the next incoming connection
        // Returns (socket, peer_address)
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);

        // Spawn a task for each connection
        // Each task runs independently, allowing concurrent clients
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
    // Note: This loop never exits. In production, you'd add graceful shutdown.
}

```

### Example: Handle a single client connection (echo protocol)
This example walks through how to handle a single client connection (echo protocol).

```rust
async fn handle_client(mut socket: TcpStream) -> io::Result<()> {
    let mut buffer = [0; 1024];

    loop {
        // read() awaits data from the client
        // Returns number of bytes read, or 0 on EOF (client disconnected)
        let n = socket.read(&mut buffer).await?;

        if n == 0 {
            // Client closed the connection gracefully
            return Ok(());
        }

        // Echo the data back to the client
        // write_all() ensures all bytes are sent (loops if the write is partial)
        socket.write_all(&buffer[..n]).await?;
    }
}

```

### Example: TCP Client
This example walks through tcp client.

```rust
async fn tcp_client(addr: &str, message: &str) -> io::Result<String> {
    // connect() performs DNS resolution (if needed) and TCP handshake
    let mut stream = TcpStream::connect(addr).await?;

    // Send message
    stream.write_all(message.as_bytes()).await?;

    // Read response
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;
    // Note: read_to_end() reads until EOF, which means the server
    // must close the connection after responding

    Ok(String::from_utf8_lossy(&buffer).to_string())
}

```

### Example: HTTP-like request handling (simplified)
This example walks through http-like request handling (simplified).

```rust
async fn http_handler(mut socket: TcpStream) -> io::Result<()> {
    let mut buffer = [0; 4096];

    // Read the HTTP request
    let n = socket.read(&mut buffer).await?;

    let request = String::from_utf8_lossy(&buffer[..n]);
    println!("Request: {}", request);

    // Send HTTP response
    // In production, you'd parse the request and route to handlers
    let response = "HTTP/1.1 200 OK\r\n\
                   Content-Type: text/plain\r\n\
                   Content-Length: 13\r\n\
                   \r\n\
                   Hello, World!";

    socket.write_all(response.as_bytes()).await?;
    Ok(())
}
```


### Example: UDP Network I/O

UDP (User Datagram Protocol) is connectionless and unreliable—packets may arrive out of order, be duplicated, or be lost entirely. But UDP has advantages: lower latency (no connection setup), simpler protocol, and support for broadcast/multicast.

**When to use UDP:**
- Low-latency gaming or real-time video where occasional packet loss is acceptable
- DNS queries (simple request-response where retries are easy)
- Service discovery and heartbeat protocols
- Metrics and logging where some data loss is tolerable

**When to use TCP instead:**
- When you need guaranteed delivery and ordering
- When you're transferring bulk data (files, database replication)
- When you need flow control and congestion management

```rust
use tokio::net::UdpSocket;
use tokio::io;

```

### Example: UDP Echo Server
This example walks through udp echo server.

```rust
async fn udp_server(addr: &str) -> io::Result<()> {
    let socket = UdpSocket::bind(addr).await?;
    println!("UDP server listening on {}", addr);

    let mut buffer = [0; 1024];

    loop {
        // recv_from() awaits a datagram from any sender
        // Returns (bytes_received, sender_address)
        let (len, addr) = socket.recv_from(&mut buffer).await?;
        println!("Received {} bytes from {}", len, addr);

        // Echo the datagram back to the sender
        // UDP doesn't guarantee delivery, so send_to() might succeed
        // even if the datagram never arrives
        socket.send_to(&buffer[..len], addr).await?;
    }
}

```

### Example: UDP Client
This example walks through udp client.

```rust
async fn udp_client(server_addr: &str, message: &str) -> io::Result<String> {
    // Bind to a random local port (0.0.0.0:0 means "any port")
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    // Send datagram to server
    socket.send_to(message.as_bytes(), server_addr).await?;

    // Wait for response
    let mut buffer = [0; 1024];
    let (len, _) = socket.recv_from(&mut buffer).await?;

    Ok(String::from_utf8_lossy(&buffer[..len]).to_string())
}
```


### Example: Unix Domain Sockets

Unix domain sockets provide inter-process communication (IPC) on the same machine. They're faster than TCP (no network stack overhead) and support passing file descriptors between processes.

**When to use Unix sockets:**
- Communication between processes on the same machine (Docker daemon, database connections)
- When you need higher performance than TCP for local IPC
- When you want filesystem-based access control (socket file permissions)

```rust
#[cfg(unix)]
mod unix_sockets {
    use tokio::net::{UnixListener, UnixStream};
    use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

    pub async fn unix_server(path: &str) -> io::Result<()> {
        // Remove old socket file if exists (bind fails if file exists)
        let _ = std::fs::remove_file(path);

        let listener = UnixListener::bind(path)?;
        println!("Unix socket server listening on {}", path);

        loop {
            let (mut socket, _) = listener.accept().await?;

            // Spawn task for each connection (same pattern as TCP)
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


---

## Pattern 2: Buffered Async Streams

**Problem**: Byte-by-byte async reads/writes trigger many syscalls—catastrophically inefficient. Need to batch I/O operations.

**Solution**: Use AsyncBufReadExt trait with read_line(), lines() for text. BufReader wraps async readers (File, TcpStream) with 8KB default buffer.

**Why It Matters**: Buffering reduces syscalls by 100x (byte-by-byte → chunked). Reading 1MB file: unbuffered = 1M syscalls, buffered = ~128 syscalls.

**Use Cases**: Line-based protocols (HTTP headers, SMTP, Redis protocol), chat protocols (newline-delimited), log streaming, CSV/JSON over network, codec-based protocols (protobuf, MessagePack), WebSocket framing, custom protocol parsers.

### Example: Using BufReader and BufWriter
Minimize syscalls for async read/write operations with batching.

```rust
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

```

### Example: Buffered reading with custom buffer size
This example walks through buffered reading with custom buffer size.

```rust
async fn buffered_read(path: &str) -> io::Result<()> {
    let file = File::open(path).await?;

    // Create BufReader with 8KB buffer (adjust based on your workload)
    let reader = BufReader::with_capacity(8192, file);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        println!("{}", line);
        // Processing each line is fast because BufReader minimizes system calls
    }

    Ok(())
}

```

### Example: Buffered writing with custom buffer size
This example walks through buffered writing with custom buffer size.

```rust
async fn buffered_write(path: &str, lines: &[&str]) -> io::Result<()> {
    let file = File::create(path).await?;

    // BufWriter accumulates writes, flushes when buffer is full
    let mut writer = BufWriter::with_capacity(8192, file);

    for line in lines {
        // These writes don't immediately hit disk—they go to the buffer
        writer.write_all(line.as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }

    // flush() ensures all buffered data is written to disk
    // Without this, some data might remain in the buffer!
    writer.flush().await?;
    Ok(())
}

```

### Example: Copy with buffering
This example walks through copy with buffering.

```rust
async fn buffered_copy(src: &str, dst: &str) -> io::Result<u64> {
    let src_file = File::open(src).await?;
    let dst_file = File::create(dst).await?;

    let mut reader = BufReader::new(src_file);
    let mut writer = BufWriter::new(dst_file);

    // copy() efficiently transfers data, using the buffers to minimize system calls
    tokio::io::copy(&mut reader, &mut writer).await
}
```


### Example: Stream Processing with AsyncRead/AsyncWrite

Implementing custom `AsyncRead` or `AsyncWrite` allows you to transform data as it's read or written. This is useful for encryption, compression, encoding, or protocol framing.

**When to implement AsyncRead/AsyncWrite:**
- Building a custom protocol or encoding layer
- Adding transparent encryption/compression
- Implementing adapters between different I/O types
- Creating testable mock I/O objects

```rust
use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use std::pin::Pin;
use std::task::{Context, Poll};

```

### Example: Custom async reader that uppercases data
This example walks through custom async reader that uppercases data.

```rust
struct UppercaseReader<R> {
    inner: R,
}

impl<R: AsyncRead + Unpin> AsyncRead for UppercaseReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // Track how many bytes were in the buffer before reading
        let before_len = buf.filled().len();

        // Delegate to the inner reader
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

```

### Example: Usage example
This example walks through usage example.

```rust
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


### Example: Stream Splitting and Framing

Many protocols benefit from splitting a stream into independent read and write halves. This allows one task to read while another writes, or enables implementing full-duplex protocols where requests and responses flow concurrently.

```rust
use tokio::net::TcpStream;
use tokio::io::{self, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

```

### Example: Split stream into read and write halves
This example walks through split stream into read and write halves.

```rust
async fn split_stream_example() -> io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;

    // into_split() divides the stream into independent halves
    // The read half can be used in one task, write half in another
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

    // Wait for both tasks to complete
    reader_task.await??;
    writer_task.await??;

    Ok(())
}
```


### Example: Using tokio_util::codec for Framing

Framing solves a fundamental problem in network protocols: how do you know where one message ends and the next begins? Raw TCP gives you a byte stream with no message boundaries. Codecs provide message framing—they handle splitting the stream into discrete messages and vice versa.

**Common framing strategies:**
- **Line-delimited**: Messages separated by newlines (simple, human-readable)
- **Length-prefixed**: Each message starts with its length (efficient, binary-safe)
- **Fixed-size**: All messages are the same size (simple but inflexible)
- **Delimiter-based**: Messages separated by a special byte sequence (like HTTP headers separated by `\r\n\r\n`)

```rust
// Add to Cargo.toml:
// tokio-util = { version = "0.7", features = ["codec"] }

use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};
use futures::{SinkExt, StreamExt};

```

### Example: Line-delimited codec
This example walks through line-delimited codec.

```rust
async fn framed_lines() -> io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;

    // Framed wraps a stream and codec, providing a Stream of messages
    // and a Sink for sending messages
    let mut framed = Framed::new(stream, LinesCodec::new());

    // Send lines (Sink interface)
    framed.send("Hello, World!".to_string()).await?;
    framed.send("Another line".to_string()).await?;

    // Receive lines (Stream interface)
    while let Some(result) = framed.next().await {
        match result {
            Ok(line) => println!("Received: {}", line),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}

```

### Example: Custom codec for length-prefixed messages
This example walks through custom codec for length-prefixed messages.

```rust
use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

struct LengthPrefixedCodec;

impl Decoder for LengthPrefixedCodec {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Need at least 4 bytes for the length prefix
        if src.len() < 4 {
            return Ok(None); // Need more data
        }

        // Read the length prefix (big-endian u32)
        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_be_bytes(length_bytes) as usize;

        // Check if we have the complete message
        if src.len() < 4 + length {
            return Ok(None); // Need more data
        }

        // We have a complete message—extract it
        src.advance(4);  // Skip the length prefix
        let data = src.split_to(length).to_vec();
        Ok(Some(data))
    }
}

impl Encoder<Vec<u8>> for LengthPrefixedCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Vec<u8>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Write length prefix
        let length = item.len() as u32;
        dst.put_u32(length);

        // Write message data
        dst.put_slice(&item);
        Ok(())
    }
}

async fn length_prefixed_example() -> io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let mut framed = Framed::new(stream, LengthPrefixedCodec);

    // Send message (automatically prefixed with length)
    framed.send(b"Hello, World!".to_vec()).await?;

    // Receive message (automatically unpacked from length-prefixed format)
    if let Some(result) = framed.next().await {
        let data = result?;
        println!("Received {} bytes", data.len());
    }

    Ok(())
}
```


---

## Pattern 3: Backpressure Handling

**Problem**: Fast producer overwhelms slow consumer causing unbounded memory growth. Web scraper fetches faster than parser processes—queue grows until OOM.

**Solution**: Use bounded mpsc channels with capacity limit—send() blocks when full. Semaphore limits concurrent operations (e.g., max 10 concurrent requests).

**Why It Matters**: Prevents OOM from unbounded queues—production systems must bound memory. Fast network source won't overwhelm slow disk sink.

**Use Cases**: Producer-consumer pipelines (network → processing → disk), streaming aggregation (sensor data, logs), rate-limited HTTP clients (respect API limits), connection pools (bound concurrent connections), download managers (limit concurrent downloads), WebSocket servers (per-client backpressure), data pipelines (ETL systems).

### Example: Manual Backpressure with Bounded Channels

Control flow between producer and consumer to prevent memory exhaustion.

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

```

### Example: Producer with backpressure
This example walks through producer with backpressure.

```rust
async fn producer_with_backpressure(tx: mpsc::Sender<i32>) {
    for i in 0..100 {
        // send() provides backpressure—blocks when channel is full
        // This ensures we don't overwhelm the consumer
        if let Err(_) = tx.send(i).await {
            println!("Receiver dropped");
            break;
        }
        println!("Sent: {}", i);
    }
}

```

### Example: Consumer (intentionally slow to demonstrate backpressure)
This example walks through consumer (intentionally slow to demonstrate backpressure).

```rust
async fn consumer(mut rx: mpsc::Receiver<i32>) {
    while let Some(value) = rx.recv().await {
        println!("Processing: {}", value);
        // Simulate slow processing
        sleep(Duration::from_millis(100)).await;
    }
}

```

### Example: Usage with bounded channel
This example walks through usage with bounded channel.

```rust
async fn backpressure_example() {
    // Bounded channel with capacity 10
    // Producer can get ahead by 10 items, then must wait
    let (tx, rx) = mpsc::channel(10);

    let producer = tokio::spawn(producer_with_backpressure(tx));
    let consumer = tokio::spawn(consumer(rx));

    let _ = tokio::join!(producer, consumer);
}
```


### Example: Stream Backpressure with Buffering

The `buffered()` combinator limits the number of futures executing concurrently, providing backpressure for stream processing.

```rust
use futures::stream::{self, StreamExt};
use tokio::time::{sleep, Duration};

async fn stream_backpressure() {
    let stream = stream::iter(0..100)
        .map(|i| async move {
            println!("Generating: {}", i);
            i
        })
        // buffered(5) means at most 5 futures run concurrently
        // This provides backpressure: we won't start future #6 until one completes
        .buffered(5)
        .for_each(|value| async move {
            println!("Processing: {}", value);
            sleep(Duration::from_millis(100)).await;
        })
        .await;
}
```


### Example: Rate Limiting

Rate limiting controls the rate of operations to avoid overwhelming downstream systems or respecting API rate limits.

**When to use rate limiting:**
- Calling third-party APIs with rate limits (e.g., "100 requests per minute")
- Protecting downstream services from being overwhelmed
- Implementing fair scheduling among multiple clients
- Smoothing bursty traffic

```rust
use tokio::time::{sleep, Duration, Instant};

```

### Example: Simple rate limiter (token bucket algorithm)
This example walks through simple rate limiter (token bucket algorithm).

```rust
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

        // Reset counter every second
        if elapsed >= Duration::from_secs(1) {
            self.last_reset = now;
            self.count = 0;
        }

        // If we've hit the rate limit, wait until next second
        if self.count >= self.max_per_second {
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
        // acquire() blocks if we've exceeded the rate limit
        limiter.acquire().await;
        println!("Request {}", i);
        // Make request...
    }
}
```


### Example: Semaphore for Concurrency Control

A semaphore limits the number of concurrent operations. This is essential for controlling resource usage (database connections, file handles, HTTP requests).

**Semaphore vs. Channel:**
- Use a **semaphore** when you just need to limit concurrency (e.g., "at most 5 concurrent HTTP requests")
- Use a **channel** when you need to pass data between tasks with bounded buffering

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

async fn concurrent_with_limit() {
    // Semaphore with 5 permits = max 5 concurrent tasks
    let semaphore = Arc::new(Semaphore::new(5));

    let mut handles = vec![];

    for i in 0..20 {
        // acquire_owned() waits if no permits are available
        let permit = semaphore.clone().acquire_owned().await.unwrap();

        let handle = tokio::spawn(async move {
            println!("Task {} started", i);
            sleep(Duration::from_secs(1)).await;
            println!("Task {} completed", i);
            // Permit is dropped here, releasing it back to the semaphore
            drop(permit);
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
}
```


### Example: Flow Control in Network Servers

Limiting concurrent connections prevents resource exhaustion and ensures fair service under load.

```rust
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use std::sync::Arc;

```

### Example: TCP server with connection limit
This example walks through tcp server with connection limit.

```rust
async fn server_with_connection_limit(addr: &str, max_connections: usize) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let semaphore = Arc::new(Semaphore::new(max_connections));

    println!("Server listening on {} (max {} connections)", addr, max_connections);

    loop {
        let (socket, addr) = listener.accept().await?;

        // Try to acquire permit (blocks if at connection limit)
        let permit = semaphore.clone().acquire_owned().await.unwrap();

        println!("New connection from {} ({} slots available)",
                 addr,
                 semaphore.available_permits());

        tokio::spawn(async move {
            // Handle connection
            let _ = handle_client(socket).await;
            // Permit dropped here, freeing a connection slot
            drop(permit);
        });
    }
}
```


---

## Pattern 4: Connection Pooling

**Problem**: Creating TCP/database connections expensive—100ms+ for TCP handshake + TLS + auth. Can't scale to 1 new connection per request (too slow).

**Solution**: Use bb8 or deadpool crates for production-ready pools. Configure min/max pool size (e.g., 10-50 connections).

**Why It Matters**: Reduces latency 100x—reuse (1ms) vs new connection (100ms). Prevents connection exhaustion: pool limits concurrent connections.

**Use Cases**: Database connection pools (Postgres, MySQL, Redis), HTTP client connection pools (reqwest with pool), gRPC connection pools, connection-limited APIs (respect limits), microservices (service-to-service), connection-expensive protocols (TLS, SSH).

### Example: Connection Pool Pattern

 Efficiently manage and reuse database or network connections.

```rust
use tokio::sync::Mutex;
use std::sync::Arc;

struct Connection {
    id: usize,
}

impl Connection {
    async fn new(id: usize) -> io::Result<Self> {
        // Simulate connection setup (DNS, TCP handshake, authentication)
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

        // Pre-create all connections
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
                // Got a connection from the pool
                return Ok(PooledConnection {
                    conn: Some(conn),
                    pool: self.connections.clone(),
                });
            }

            // No connections available—wait and retry
            drop(pool);
            sleep(Duration::from_millis(10)).await;
        }
    }
}

```

### Example: RAII wrapper: returns connection to pool on drop
This example walks through raii wrapper: returns connection to pool on drop.

```rust
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
            // Return connection to pool
            let pool = self.pool.clone();
            tokio::spawn(async move {
                pool.lock().await.push(conn);
            });
        }
    }
}

```

### Example: Usage
This example walks through usage.

```rust
async fn use_pool() -> io::Result<()> {
    let pool = SimplePool::new(5).await?;

    let mut handles = vec![];

    // Spawn 20 tasks, but only 5 connections exist
    // Tasks will wait for connections to become available
    for i in 0..20 {
        let pool = pool.clone();
        let handle = tokio::spawn(async move {
            let conn = pool.acquire().await.unwrap();
            let result = conn.execute(&format!("Query {}", i)).await.unwrap();
            println!("{}", result);
            // Connection returned to pool when `conn` is dropped
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}
```


### Example: Advanced Pool with deadpool

`deadpool` is a production-ready connection pool library with features like connection recycling, timeouts, and health checks.

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

    // Called when the pool needs to create a new connection
    async fn create(&self) -> Result<MyConnection, io::Error> {
        let mut id = self.next_id.lock().await;
        let conn = MyConnection { id: *id };
        *id += 1;
        println!("Created connection {}", conn.id);
        Ok(conn)
    }

    // Called when a connection is returned to the pool
    // Allows health checks or cleanup before reuse
    async fn recycle(&self, conn: &mut MyConnection) -> RecycleResult<io::Error> {
        println!("Recycling connection {}", conn.id);
        // Could perform a health check here (e.g., ping the database)
        Ok(())
    }
}

async fn use_deadpool() -> io::Result<()> {
    let manager = MyManager {
        next_id: Arc::new(Mutex::new(0)),
    };

    let pool = Pool::builder(manager)
        .max_size(5)  // Max 5 connections
        .build()
        .unwrap();

    let mut handles = vec![];

    for i in 0..20 {
        let pool = pool.clone();
        let handle = tokio::spawn(async move {
            // get() acquires a connection (waits if pool is exhausted)
            let conn = pool.get().await.unwrap();
            println!("Using connection {} for task {}", conn.id, i);
            sleep(Duration::from_millis(100)).await;
            // Connection returned to pool when dropped
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}
```


### Example: HTTP Client Connection Pool

The `reqwest` HTTP client has built-in connection pooling, which is essential for making many HTTP requests efficiently.

```rust
// Add to Cargo.toml:
// reqwest = "0.11"

use reqwest::Client;
use std::time::Duration;

async fn http_connection_pool() -> Result<(), Box<dyn std::error::Error>> {
    // reqwest Client has built-in connection pooling
    // Multiple requests to the same host reuse TCP connections
    let client = Client::builder()
        .pool_max_idle_per_host(10)  // Keep up to 10 idle connections per host
        .pool_idle_timeout(Duration::from_secs(90))  // Close idle connections after 90s
        .timeout(Duration::from_secs(30))  // Request timeout
        .build()?;

    let mut handles = vec![];

    for i in 0..50 {
        let client = client.clone();
        let handle = tokio::spawn(async move {
            // Requests to the same host reuse connections from the pool
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



## Pattern 5: Timeout and Cancellation

**Problem**: Async operations can hang forever without time bounds—network request to unresponsive server blocks indefinitely. Need to cancel slow operations to prevent resource leaks.

**Solution**: Use tokio::time::timeout() to wrap futures with duration limit—returns Err on timeout. select!

**Why It Matters**: Prevents resource leaks from hung operations—without timeout, stuck connections never close. HTTP requests must timeout (client disappeared, network partition).

**Use Cases**: HTTP request timeouts (prevent hung requests), database query timeouts (prevent slow queries), graceful shutdown (SIGTERM handling), user cancellation (browser closed, request canceled), health checks (must timeout), circuit breakers (timeout = failure signal), deadline propagation (gRPC deadlines), connection idle timeouts.

### Example: Basic Timeout Pattern

Prevent operations from running indefinitely by setting time limits.

```rust
use tokio::time::{timeout, Duration};

async fn with_timeout() -> Result<(), Box<dyn std::error::Error>> {
    // timeout() returns Err if the operation exceeds 5 seconds
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


### Example: Timeout with Fallback

A common pattern is to try a primary operation with a timeout, falling back to an alternative if it fails.

```rust
use tokio::time::{timeout, Duration};

async fn timeout_with_fallback() -> String {
    let result = timeout(
        Duration::from_secs(2),
        fetch_data_from_primary(),
    ).await;

    match result {
        Ok(Ok(data)) => data,  // Primary succeeded
        _ => {
            // Primary timed out or failed—try fallback
            println!("Primary failed, trying fallback");
            fetch_data_from_fallback().await.unwrap_or_default()
        }
    }
}

async fn fetch_data_from_primary() -> io::Result<String> {
    sleep(Duration::from_secs(5)).await;  // Too slow
    Ok("Primary data".to_string())
}

async fn fetch_data_from_fallback() -> io::Result<String> {
    sleep(Duration::from_millis(500)).await;  // Fast fallback
    Ok("Fallback data".to_string())
}
```


### Example: Cancellation with CancellationToken

`CancellationToken` provides a mechanism for coordinated cancellation across multiple tasks. When you cancel the token, all tasks listening to it are notified.

**When to use CancellationToken:**
- Implementing graceful shutdown (cancel all background tasks)
- Stopping a group of related tasks when one fails
- Implementing request cancellation in a server

```rust
// Add to Cargo.toml:
// tokio-util = "0.7"

use tokio_util::sync::CancellationToken;

async fn cancellable_operation(token: CancellationToken) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        tokio::select! {
            // Check if cancellation was requested
            _ = token.cancelled() => {
                println!("Operation cancelled");
                break;
            }
            // Do work
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


### Example: Select for Racing Operations

`tokio::select!` runs multiple futures concurrently and returns as soon as one completes. This is useful for implementing timeouts, racing multiple data sources, or handling multiple events.

```rust
use tokio::time::{sleep, Duration};

```

### Example: Race three operations—return the first to complete
This example walks through race three operations—return the first to complete.

```rust
async fn race_operations() -> String {
    tokio::select! {
        result = operation_a() => result,
        result = operation_b() => result,
        result = operation_c() => result,
    }
    // The other futures are dropped (canceled) when one completes
}

async fn operation_a() -> String {
    sleep(Duration::from_secs(3)).await;
    "A completed".to_string()
}

async fn operation_b() -> String {
    sleep(Duration::from_secs(1)).await;
    "B completed".to_string()  // This completes first
}

async fn operation_c() -> String {
    sleep(Duration::from_secs(2)).await;
    "C completed".to_string()
}

```

### Example: Biased select (checks branches in order)
This example walks through biased select (checks branches in order).

```rust
async fn biased_select() {
    let mut count = 0;

    loop {
        tokio::select! {
            biased;  // Check branches in order (not randomly)

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


### Example: Graceful Shutdown

Graceful shutdown ensures that all in-flight requests complete before the server stops, preventing data loss or corrupted state.

```rust
use tokio::signal;
use tokio::sync::broadcast;

//==================================
// TCP server with graceful shutdown
// When Ctrl+C is pressed, the server stops accepting new connections
// but waits for existing connections to finish
//=============================================
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


### Example: Timeout for Multiple Operations

When running multiple operations concurrently, you can apply a timeout to all of them collectively or to each individually.

```rust
use tokio::time::{timeout, Duration};
use futures::future::join_all;

```

### Example: Collective timeout: all operations must complete within 5 seconds total
This example walks through collective timeout: all operations must complete within 5 seconds total.

```rust
async fn timeout_multiple() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let operations = vec![
        async_task(1),
        async_task(2),
        async_task(3),
    ];

    // Timeout applies to the entire join_all
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

```

### Example: Individual timeouts: each operation has its own timeout
This example walks through individual timeouts: each operation has its own timeout.

```rust
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


### Example: Retry with Timeout

Combining retries with timeouts creates resilient operations that handle transient failures but don't hang forever.

```rust
use tokio::time::{sleep, timeout, Duration};

```

### Example: Generic retry logic with timeout and exponential backoff
This example walks through generic retry logic with timeout and exponential backoff.

```rust
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
            Ok(Ok(result)) => return Ok(result),  // Success
            Ok(Err(e)) => {
                eprintln!("Attempt {} failed: {}", attempt + 1, e);
            }
            Err(_) => {
                eprintln!("Attempt {} timed out", attempt + 1);
            }
        }

        if attempt < max_retries - 1 {
            // Exponential backoff: wait longer after each failure
            let backoff = Duration::from_millis(100 * 2_u64.pow(attempt as u32));
            sleep(backoff).await;
        }
    }

    Err("All retry attempts failed".into())
}

```

### Example: Usage
This example walks through usage.

```rust
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
        5,  // Max 5 retries
        Duration::from_secs(2),  // 2-second timeout per attempt
    ).await?;

    println!("Result: {}", result);
    Ok(())
}
```


### Example: Deadline-based Cancellation

Instead of relative timeouts, you can use absolute deadlines. This is useful when multiple operations share a common deadline.

```rust
use tokio::time::{sleep, timeout_at, Duration, Instant};

async fn deadline_based_processing() -> io::Result<()> {
    // All tasks must complete by this deadline
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
    // timeout_at() uses an absolute deadline instead of a duration
    timeout_at(deadline, async move {
        sleep(Duration::from_secs(id as u64 * 2)).await;
        Ok(format!("Item {} processed", id))
    })
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Deadline exceeded"))?
}
```


### Summary

This chapter covered async I/O patterns using Tokio:

1. **Tokio File and Network I/O**: Async primitives (TcpListener, tokio::fs), work-stealing scheduler, solves C10K problem
2. **Buffered Async Streams**: AsyncBufReadExt for line-by-line, BufReader/BufWriter, codecs for framing
3. **Backpressure Handling**: Bounded channels, Semaphore, buffer_unordered(n), prevents OOM
4. **Connection Pooling**: bb8/deadpool, reduces latency 100x, manages connection lifecycle
5. **Timeout and Cancellation**: timeout(), select!, CancellationToken, prevents resource leaks

**Key Takeaways**:
- Async I/O enables single thread to handle 10K+ connections
- Always use buffering—100x syscall reduction
- Bounded channels provide automatic backpressure
- Connection pooling essential for databases/HTTP
- Timeouts prevent hung operations—critical for production
- Drop cancels futures automatically (structured concurrency)

**Performance Guidelines**:
- Async for I/O-bound, sync for CPU-bound
- Use spawn_blocking for blocking operations
- Buffer size: 8KB default, adjust for workload
- Connection pool: 10-50 connections typical
- Always set timeouts on network operations

**Production Patterns**:
- Graceful shutdown with CancellationToken
- Per-request timeouts prevent slow requests blocking others
- Circuit breakers rely on timeout detection
- Backpressure prevents OOM under load
- Health checks detect stale pooled connections

**Common Pitfalls**:
- Unbounded channels cause OOM
- No timeout = hung connections accumulate
- Blocking in async tasks starves others
- Missing backpressure = memory exhaustion
- Connection pool without health checks keeps stale connections
