# Async I/O Patterns

[Tokio File and Network I/O](#pattern-1-tokio-file-and-network-io)

- Problem: Synchronous I/O blocks threads; 1 thread/connection doesn't scale; 10K connections need 20GB RAM; blocking file I/O stalls network tasks
- Solution: Tokio async I/O with .await; TcpListener/TcpStream for network; tokio::fs for files; work-stealing scheduler spreads load
- Why It Matters: Single thread handles 10K connections in 10MB; C10K problem solved; async file I/O won't block network tasks; work-stealing uses all cores
- Use Cases: Web servers, API gateways, chat servers, websockets, HTTP clients, file servers, concurrent file processing

[Buffered Async Streams](#pattern-2-buffered-async-streams)

- Problem: Byte-by-byte async reads/writes inefficient; need batching; no async equivalent of BufReader; line-by-line async parsing verbose
- Solution: AsyncBufReadExt with read_line(), lines(); AsyncWriteExt with write_all(); BufReader/BufWriter wrap streams; tokio_util::codec for framing
- Why It Matters: Buffering reduces syscalls by 100x; lines() yields async iterator; codecs handle framing (length-prefixed, delimited); essential for protocols
- Use Cases: Line-based protocols (HTTP, SMTP), chat protocols, log streaming, CSV/JSON over network, codec-based protocols, WebSocket framing

[Backpressure Handling](#pattern-3-backpressure-handling)

- Problem: Fast producer overwhelms slow consumer; unbounded queues cause OOM; no flow control between tasks; need to slow upstream when downstream lags
- Solution: Bounded channels (mpsc with capacity); Semaphore for concurrency limits; buffer_unordered(n) for stream concurrency; rate limiting with governor
- Why It Matters: Prevents OOM from unbounded growth; fast network reader won't overwhelm slow file writer; graceful degradation under load; critical for production
- Use Cases: Producer-consumer pipelines, streaming aggregation, rate limiting HTTP clients, connection pooling, download managers, WebSocket servers

[Connection Pooling](#pattern-4-connection-pooling)

- Problem: Creating TCP/DB connections expensive (100ms+ handshake); can't scale to 1 connection/request; need connection reuse; must limit concurrent connections
- Solution: bb8/deadpool for connection pools; configure min/max size; timeouts for acquisition; health checks; recycle connections between requests
- Why It Matters: Reduces latency 100x (reuse vs new connection); limits concurrent connections; handles connection failures gracefully; essential for databases/HTTP
- Use Cases: Database connection pools (Postgres, Redis), HTTP client pools, gRPC connection pools, connection-limited APIs, microservices

[Timeout and Cancellation](#pattern-5-timeout-and-cancellation)

- Problem: Async operations can hang forever; need time bounds; must cancel slow operations; resource leaks from stuck tasks; graceful shutdown complex
- Solution: tokio::time::timeout() wraps futures; select! for cancellation; CancellationToken for coordinated shutdown; drop cancels futures; timeout pattern
- Why It Matters: Prevents resource leaks from hung operations; timeout HTTP requests fail fast; select! enables responsive cancellation; safe cleanup on shutdown
- Use Cases: HTTP request timeouts, database query timeouts, graceful shutdown, user cancellation, health checks, circuit breakers, deadline propagation


[Tokio File IO Cheat Sheet](#tokio-file-io-cheat-sheet)
- a long list of useful functions

[Tokio Network IO Cheat Sheet](#tokio-network-io-cheat-sheet)
- a long list of useful functions

### Overview
This chapter explores async I/O patterns using Tokio—handling thousands of concurrent connections, buffered streams, backpressure, connection pooling, and timeouts. Async I/O allows one thread to manage thousands of operations by yielding when blocked, solving the C10K problem.


## Pattern 1: Tokio File and Network I/O

**Problem**: Synchronous I/O blocks threads waiting for data. One thread per connection doesn't scale—10K connections need 20GB RAM (2MB stack each). Context switching between thousands of threads destroys performance. Blocking file operations stall network tasks. CPU-bound tasks mixed with I/O starve either. Need to handle thousands of concurrent connections efficiently.

**Solution**: Use Tokio async I/O where operations return futures you `.await`. TcpListener/TcpStream for network accept/read/write. tokio::fs for file operations (uses thread pool). Work-stealing scheduler distributes tasks across cores. Spawn tasks with tokio::spawn for concurrency. Use tokio::task::spawn_blocking for CPU-bound work. Single runtime handles thousands of connections.

**Why It Matters**: Single thread handles 10K connections in 10MB memory (vs 20GB threaded). Solves C10K problem—web servers serve 10K+ concurrent users. Async file I/O won't block network handlers. Work-stealing keeps all cores busy. Context switch overhead eliminated. Enables web servers, chat servers, API gateways at scale. Without async I/O, 1 thread/connection model fails at high concurrency.

**Use Cases**: Web servers (HTTP, HTTPS), API gateways, chat servers, WebSocket servers, game servers, HTTP clients (concurrent requests), file servers (mixing file and network I/O), concurrent file processing, real-time data pipelines.

### Example: Async File Operations

Read/write files without blocking async runtime. Concurrent file operations should parallelize.

```rust
// Note: Add to Cargo.toml:
// [dependencies]
// tokio = { version = "1", features = ["full"] }

use tokio::fs::{File, OpenOptions};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

//===============================
// Read entire file into a String
//===============================
async fn read_file(path: &str) -> io::Result<String> {
    tokio::fs::read_to_string(path).await
    // Convenience method: allocates a String, reads entire file
    // Returns Err if file missing, unreadable, or contains invalid UTF-8
}

//==============================
// Read entire file into Vec<u8>
//==============================
async fn read_bytes(path: &str) -> io::Result<Vec<u8>> {
    tokio::fs::read(path).await
    // Reads entire file into memory
    // More efficient than read_to_string for binary data
}

//===================================================
// Write string to file (overwrites existing content)
//===================================================
async fn write_file(path: &str, content: &str) -> io::Result<()> {
    tokio::fs::write(path, content).await
    // Convenience method: creates file, writes content, closes file
    // Overwrites existing file! Use append_to_file if you want to append
}

//================================
// Manual read with buffer control
//================================
async fn read_with_buffer(path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(path).await?;
    let mut buffer = Vec::new();

    // read_to_end() allocates as needed while reading
    file.read_to_end(&mut buffer).await?;
    Ok(buffer)
}

//==================================
// Manual write with explicit handle
//==================================
async fn write_with_handle(path: &str, data: &[u8]) -> io::Result<()> {
    let mut file = File::create(path).await?;

    // write_all() ensures all bytes are written (loops if needed)
    file.write_all(data).await?;

    // flush() ensures buffered data reaches the OS
    // (Tokio buffers writes internally for efficiency)
    file.flush().await?;
    Ok(())
}

//===============================================
// Append to existing file (or create if missing)
//===============================================
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

//=========================
// Copy file asynchronously
//=========================
async fn copy_file(src: &str, dst: &str) -> io::Result<u64> {
    tokio::fs::copy(src, dst).await
    // Efficiently copies src to dst using OS-level optimizations when possible
}

//==============
// Example usage
//==============
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

//===========================
// Read all lines into memory
//===========================
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

//============================================================
// Process large file line by line without loading into memory
//============================================================
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

//====================================================
// Read first N lines (useful for previews or headers)
//====================================================
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

//================
// TCP Echo Server
//================
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

//==================================================
// Handle a single client connection (echo protocol)
//==================================================
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

//===========
// TCP Client
//===========
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

//========================================
// HTTP-like request handling (simplified)
//========================================
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

//================
// UDP Echo Server
//================
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

//===========
// UDP Client
//===========
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

**Problem**: Byte-by-byte async reads/writes trigger many syscalls—catastrophically inefficient. Need to batch I/O operations. Parsing line-by-line from network streams requires buffering. No async equivalent of std::io::BufReader. Protocol implementations need framing (where messages start/end). Need to read "lines" from TCP streams efficiently.

**Solution**: Use AsyncBufReadExt trait with read_line(), lines() for text. BufReader wraps async readers (File, TcpStream) with 8KB default buffer. BufWriter batches writes. Use tokio_util::codec for custom framing (length-prefixed, delimited). AsyncWriteExt provides write_all() for full writes. Adjust buffer size with with_capacity() based on workload.

**Why It Matters**: Buffering reduces syscalls by 100x (byte-by-byte → chunked). Reading 1MB file: unbuffered = 1M syscalls, buffered = ~128 syscalls. lines() provides async iterator over lines—essential for protocols. Codecs abstract framing complexity. Without buffering, async I/O slower than sync. Critical for all network protocols (HTTP, WebSocket, custom).

**Use Cases**: Line-based protocols (HTTP headers, SMTP, Redis protocol), chat protocols (newline-delimited), log streaming, CSV/JSON over network, codec-based protocols (protobuf, MessagePack), WebSocket framing, custom protocol parsers.

### Example: Using BufReader and BufWriter
Minimize syscalls for async read/write operations with batching.

```rust
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

//=========================================
// Buffered reading with custom buffer size
//=========================================
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

//=========================================
// Buffered writing with custom buffer size
//=========================================
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

//====================
// Copy with buffering
//====================
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

//=========================================
// Custom async reader that uppercases data
//=========================================
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

//==============
// Usage example
//==============
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

//========================================
// Split stream into read and write halves
//========================================
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

//=====================
// Line-delimited codec
//=====================
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

//==========================================
// Custom codec for length-prefixed messages
//==========================================
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

**Problem**: Fast producer overwhelms slow consumer causing unbounded memory growth. Web scraper fetches faster than parser processes—queue grows until OOM. Network reader floods file writer with data. No flow control between tasks. Unbounded channels cause memory exhaustion. Need to slow upstream when downstream can't keep up. Graceful degradation under load requires limiting concurrency.

**Solution**: Use bounded mpsc channels with capacity limit—send() blocks when full. Semaphore limits concurrent operations (e.g., max 10 concurrent requests). Stream's buffer_unordered(n) controls concurrency. Rate limiting with governor crate or tokio::time::interval(). Feedback loops where consumer signals capacity. Drop excess load rather than queue indefinitely (shed load).

**Why It Matters**: Prevents OOM from unbounded queues—production systems must bound memory. Fast network source won't overwhelm slow disk sink. Enables graceful degradation: under load, slow down rather than crash. Critical for production: without backpressure, spike in traffic causes OOM. Database connection pools need backpressure to prevent overload. Streaming systems fail without flow control.

**Use Cases**: Producer-consumer pipelines (network → processing → disk), streaming aggregation (sensor data, logs), rate-limited HTTP clients (respect API limits), connection pools (bound concurrent connections), download managers (limit concurrent downloads), WebSocket servers (per-client backpressure), data pipelines (ETL systems).

### Example: Manual Backpressure with Bounded Channels

Control flow between producer and consumer to prevent memory exhaustion.

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

//===========================
// Producer with backpressure
//===========================
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

//==========================================================
// Consumer (intentionally slow to demonstrate backpressure)
//==========================================================
async fn consumer(mut rx: mpsc::Receiver<i32>) {
    while let Some(value) = rx.recv().await {
        println!("Processing: {}", value);
        // Simulate slow processing
        sleep(Duration::from_millis(100)).await;
    }
}

//===========================
// Usage with bounded channel
//===========================
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

//=============================================
// Simple rate limiter (token bucket algorithm)
//=============================================
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

//=================================
// TCP server with connection limit
//=================================
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

**Problem**: Creating TCP/database connections expensive—100ms+ for TCP handshake + TLS + auth. Can't scale to 1 new connection per request (too slow). Need connection reuse across requests. Databases/APIs limit concurrent connections (Postgres default: 100). Creating 1000 connections for 1000 requests overwhelms server. Connection lifecycle management (idle timeout, health checks) complex.

**Solution**: Use bb8 or deadpool crates for production-ready pools. Configure min/max pool size (e.g., 10-50 connections). Set timeouts for connection acquisition. Implement health checks to detect stale connections. Recycle connections between requests. Pool manages lifecycle: creates on-demand, reuses idle, removes unhealthy. Set idle timeout to prevent keeping stale connections.

**Why It Matters**: Reduces latency 100x—reuse (1ms) vs new connection (100ms). Prevents connection exhaustion: pool limits concurrent connections. Handles transient failures: auto-reconnect, health checks. Essential for databases: Postgres/MySQL have connection limits. HTTP clients benefit similarly. Without pooling, high-load services fail (connection limit exceeded). Connection setup dominates latency for small queries.

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

//=================================================
// RAII wrapper: returns connection to pool on drop
//=================================================
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

//======
// Usage
//======
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

**Problem**: Async operations can hang forever without time bounds—network request to unresponsive server blocks indefinitely. Need to cancel slow operations to prevent resource leaks. Stuck tasks accumulate connections until exhaustion. Graceful shutdown requires canceling all tasks. User cancellation (close browser tab) must stop server-side work. Circuit breakers need timeout detection. No timeout means single slow client hangs entire server.

**Solution**: Use tokio::time::timeout() to wrap futures with duration limit—returns Err on timeout. select! races multiple futures, enabling cancellation when another completes. CancellationToken for coordinated shutdown across tasks. Dropping a future cancels it automatically (structured concurrency). Timeout pattern: primary with fallback. Per-request timeouts prevent slow requests blocking others.

**Why It Matters**: Prevents resource leaks from hung operations—without timeout, stuck connections never close. HTTP requests must timeout (client disappeared, network partition). Database queries need timeout (prevent long-running queries). Graceful shutdown impossible without cancellation—tasks must stop on SIGTERM. Circuit breakers rely on timeouts to detect failing services. Production systems fail without timeouts—one slow dependency hangs entire service.

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

//=====================================================
// Race three operations—return the first to complete
//=====================================================
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

//=========================================
// Biased select (checks branches in order)
//=========================================
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

//========================================================================
// Collective timeout: all operations must complete within 5 seconds total
//========================================================================
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

//========================================================
// Individual timeouts: each operation has its own timeout
//========================================================
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

//=========================================================
// Generic retry logic with timeout and exponential backoff
//=========================================================
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

//======
// Usage
//======
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


### Tokio File IO Cheat Sheet

```rust
// ===== TOKIO FILE I/O =====
use tokio::fs;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, AsyncBufReadExt, AsyncSeekExt, BufReader, BufWriter};

// File opening (async)
fs::File::open("file.txt").await?                    // Open for reading
fs::File::create("file.txt").await?                  // Create/truncate for writing
fs::OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .append(true)
    .open("file.txt")
    .await?                                          // Custom open options

// Quick read operations (async)
fs::read("file.txt").await?                         // Read entire file to Vec<u8>
fs::read_to_string("file.txt").await?               // Read entire file to String

// Quick write operations (async)
fs::write("file.txt", b"data").await?               // Write bytes to file
fs::write("file.txt", "text").await?                // Write string to file

// Reading from File (async)
let mut file = fs::File::open("file.txt").await?;
let mut buffer = Vec::new();
file.read_to_end(&mut buffer).await?                // Read all bytes
let mut buffer = String::new();
file.read_to_string(&mut buffer).await?             // Read as string
let mut buffer = [0u8; 1024];
let n = file.read(&mut buffer).await?               // Read up to buffer size
file.read_exact(&mut buffer).await?                 // Read exact amount or error

// Writing to File (async)
let mut file = fs::File::create("file.txt").await?;
file.write(b"data").await?                          // Write bytes, returns bytes written
file.write_all(b"data").await?                      // Write all bytes or error
file.flush().await?                                  // Flush to disk

// Buffered reading (async)
let file = fs::File::open("file.txt").await?;
let reader = BufReader::new(file);
let mut reader = BufReader::with_capacity(size, file); // Custom buffer size

let mut lines = reader.lines();                      // Get lines stream
while let Some(line) = lines.next_line().await? {
    println!("{}", line);
}

reader.read_line(&mut string).await?                // Read one line
reader.read_until(b'\n', &mut buffer).await?        // Read until delimiter

// Buffered writing (async)
let file = fs::File::create("file.txt").await?;
let mut writer = BufWriter::new(file);
let mut writer = BufWriter::with_capacity(size, file); // Custom buffer size

writer.write_all(b"data").await?                    // Buffered write
writer.flush().await?                                // Flush buffer to disk

// Seeking (async)
file.seek(io::SeekFrom::Start(0)).await?            // Seek to byte position from start
file.seek(io::SeekFrom::End(-10)).await?            // Seek from end
file.seek(io::SeekFrom::Current(5)).await?          // Seek relative to current
file.rewind().await?                                 // Seek to start
let pos = file.stream_position().await?             // Get current position

// Metadata operations (async)
let metadata = fs::metadata("file.txt").await?      // Get file metadata
let metadata = file.metadata().await?               // From file handle
metadata.len()                                       // File size in bytes
metadata.is_file()                                   // Check if regular file
metadata.is_dir()                                    // Check if directory
metadata.is_symlink()                               // Check if symbolic link
metadata.modified()?                                 // Last modified time

// File operations (async)
fs::copy("src.txt", "dst.txt").await?               // Copy file, returns bytes copied
fs::rename("old.txt", "new.txt").await?             // Rename/move file
fs::remove_file("file.txt").await?                  // Delete file
fs::hard_link("original.txt", "link.txt").await?   // Create hard link
fs::symlink("original.txt", "link.txt").await?     // Create symbolic link
fs::read_link("link.txt").await?                    // Read symlink target
fs::canonicalize("./file.txt").await?               // Get absolute path

// Directory operations (async)
fs::create_dir("dir").await?                        // Create single directory
fs::create_dir_all("path/to/dir").await?           // Create directory and parents
fs::remove_dir("dir").await?                        // Remove empty directory
fs::remove_dir_all("dir").await?                    // Remove directory and contents

let mut entries = fs::read_dir("dir").await?;       // Get directory entries stream
while let Some(entry) = entries.next_entry().await? {
    println!("{:?}", entry.path());                 // Get full path
    println!("{:?}", entry.file_name());            // Get file name
    let metadata = entry.metadata().await?;          // Get metadata
    let file_type = entry.file_type().await?;       // Get file type
}

// File synchronization (async)
file.sync_all().await?                               // Sync data and metadata to disk
file.sync_data().await?                              // Sync only data to disk

// Set file length (async)
file.set_len(100).await?                            // Set file size (truncate/extend)

// Permissions (async)
let perms = metadata.permissions();
fs::set_permissions("file.txt", perms).await?       // Apply permissions

// Common file patterns (async)
// Read file line by line
let file = fs::File::open("file.txt").await?;
let reader = BufReader::new(file);
let mut lines = reader.lines();
while let Some(line) = lines.next_line().await? {
    // process line
}

// Write multiple lines
let file = fs::File::create("file.txt").await?;
let mut writer = BufWriter::new(file);
for item in items {
    writer.write_all(format!("{}\n", item).as_bytes()).await?;
}
writer.flush().await?;

// Copy with progress
let mut src = fs::File::open("src.txt").await?;
let mut dst = fs::File::create("dst.txt").await?;
let mut buffer = [0u8; 8192];
loop {
    let n = src.read(&mut buffer).await?;
    if n == 0 { break; }
    dst.write_all(&buffer[..n]).await?;
}


```
### Tokio Network IO Cheat Sheet
```rust
// ===== TOKIO NETWORK I/O =====
use tokio::net::{TcpListener, TcpStream, UdpSocket, UnixListener, UnixStream};

// TCP Server
let listener = TcpListener::bind("127.0.0.1:8080").await?; // Bind to address
let local_addr = listener.local_addr()?;            // Get bound address
let (socket, addr) = listener.accept().await?;      // Accept connection

// Accept loop
loop {
    let (socket, addr) = listener.accept().await?;
    tokio::spawn(async move {
        handle_client(socket).await;
    });
}

// TCP Client
let stream = TcpStream::connect("127.0.0.1:8080").await?; // Connect to server
stream.peer_addr()?                                  // Get remote address
stream.local_addr()?                                 // Get local address

// TCP Read/Write
stream.readable().await?                             // Wait until readable
stream.writable().await?                             // Wait until writable
let n = stream.read(&mut buffer).await?             // Read data
stream.read_exact(&mut buffer).await?               // Read exact amount
stream.write_all(b"data").await?                    // Write all data
stream.flush().await?                                // Flush write buffer

// Split TCP stream for concurrent read/write
let (mut read_half, mut write_half) = stream.into_split();
read_half.read(&mut buffer).await?;
write_half.write_all(b"data").await?;

// Or with borrowing
let (read_half, write_half) = stream.split();

// TCP socket options
stream.set_nodelay(true)?                            // Disable Nagle's algorithm
stream.nodelay()?                                    // Get nodelay status
stream.set_ttl(64)?                                  // Set TTL
stream.ttl()?                                        // Get TTL
stream.set_linger(Some(Duration::from_secs(5)))?   // Set SO_LINGER

// Shutdown TCP stream
stream.shutdown().await?                             // Shutdown both directions

// UDP Socket
let socket = UdpSocket::bind("127.0.0.1:8080").await?; // Bind UDP socket
socket.connect("127.0.0.1:9090").await?             // Connect to remote (optional)

let n = socket.send_to(b"data", "127.0.0.1:9090").await?; // Send to address
let (n, addr) = socket.recv_from(&mut buffer).await?; // Receive from any

socket.send(b"data").await?                         // Send to connected address
socket.recv(&mut buffer).await?                     // Receive from connected address

socket.local_addr()?                                 // Get local address
socket.peer_addr()?                                  // Get connected peer address

// UDP socket options
socket.set_broadcast(true)?                          // Enable broadcast
socket.broadcast()?                                  // Get broadcast status
socket.set_ttl(64)?                                  // Set TTL
socket.ttl()?                                        // Get TTL

// Unix Domain Sockets (Unix only)
#[cfg(unix)]
{
    // Unix stream server
    let listener = UnixListener::bind("/tmp/socket")?;
    let (socket, addr) = listener.accept().await?;
    
    // Unix stream client
    let stream = UnixStream::connect("/tmp/socket").await?;
    
    // Unix datagram
    use tokio::net::UnixDatagram;
    let socket = UnixDatagram::bind("/tmp/sock1")?;
    socket.connect("/tmp/sock2")?;
}

// Buffered network I/O
let stream = TcpStream::connect("127.0.0.1:8080").await?;
let reader = BufReader::new(stream);
let mut lines = reader.lines();
while let Some(line) = lines.next_line().await? {
    println!("{}", line);
}

let stream = TcpStream::connect("127.0.0.1:8080").await?;
let mut writer = BufWriter::new(stream);
writer.write_all(b"data").await?;
writer.flush().await?;

// HTTP-like line protocol
let stream = TcpStream::connect("127.0.0.1:8080").await?;
let (reader, mut writer) = tokio::io::split(stream);
let mut reader = BufReader::new(reader);

writer.write_all(b"GET / HTTP/1.1\r\n\r\n").await?;
let mut response = String::new();
reader.read_line(&mut response).await?;

// Copy between streams
tokio::io::copy(&mut source, &mut dest).await?     // Copy all data
tokio::io::copy_buf(&mut buf_source, &mut dest).await?; // Copy from buffered

// Common network patterns
// Echo server
let listener = TcpListener::bind("127.0.0.1:8080").await?;
loop {
    let (mut socket, _) = listener.accept().await?;
    tokio::spawn(async move {
        let mut buf = [0; 1024];
        loop {
            let n = socket.read(&mut buf).await.unwrap();
            if n == 0 { break; }
            socket.write_all(&buf[..n]).await.unwrap();
        }
    });
}

// HTTP-like request
let mut stream = TcpStream::connect("example.com:80").await?;
stream.write_all(b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n").await?;
let mut response = Vec::new();
stream.read_to_end(&mut response).await?;

// Connection timeout
use tokio::time::{timeout, Duration};
let stream = timeout(
    Duration::from_secs(5),
    TcpStream::connect("127.0.0.1:8080")
).await??;

// Concurrent connections
let mut handles = vec![];
for i in 0..10 {
    let handle = tokio::spawn(async move {
        let stream = TcpStream::connect("127.0.0.1:8080").await?;
        // work with stream
        Ok::<_, io::Error>(())
    });
    handles.push(handle);
}
for handle in handles {
    handle.await??;
}

// Bidirectional communication
let stream = TcpStream::connect("127.0.0.1:8080").await?;
let (read, write) = stream.into_split();

let read_task = tokio::spawn(async move {
    let mut reader = BufReader::new(read);
    let mut line = String::new();
    while reader.read_line(&mut line).await? > 0 {
        println!("Received: {}", line);
        line.clear();
    }
    Ok::<_, io::Error>(())
});

let write_task = tokio::spawn(async move {
    let mut write = write;
    for i in 0..10 {
        write.write_all(format!("Message {}\n", i).as_bytes()).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Ok::<_, io::Error>(())
});

read_task.await??;
write_task.await??;

// UDP echo server
let socket = UdpSocket::bind("127.0.0.1:8080").await?;
let mut buf = [0; 1024];
loop {
    let (n, addr) = socket.recv_from(&mut buf).await?;
    socket.send_to(&buf[..n], addr).await?;
}

// Graceful shutdown with signal
use tokio::signal;
let listener = TcpListener::bind("127.0.0.1:8080").await?;
let shutdown = signal::ctrl_c();
tokio::pin!(shutdown);

loop {
    tokio::select! {
        result = listener.accept() => {
            let (socket, _) = result?;
            tokio::spawn(handle_connection(socket));
        }
        _ = &mut shutdown => {
            println!("Shutting down");
            break;
        }
    }
}
```