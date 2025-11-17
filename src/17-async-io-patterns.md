# 17. Async I/O Patterns

## Overview

Asynchronous I/O represents a fundamental shift in how programs handle input and output operations. While synchronous I/O blocks a thread until data is ready, async I/O allows a single thread to manage thousands of concurrent operations by yielding control whenever an operation would block. This chapter explores Tokio, Rust's most popular async runtime, and the patterns that make high-performance network services possible.

**The Async I/O Philosophy**

Traditional synchronous I/O follows a simple model: when you read from a file or network socket, your thread waits until data arrives. This works beautifully for single-threaded programs or when you have one thread per connection. But when handling thousands of concurrent connections (like a web server), dedicating one OS thread per connection becomes prohibitively expensive—each thread consumes memory for its stack (typically 2MB), and context switching between thousands of threads destroys CPU cache locality.

Async I/O solves this by allowing a single thread to manage many operations concurrently. When an operation would block (waiting for network data, disk I/O, etc.), the current task yields control back to the runtime, which then runs another task. When the original operation completes (data arrives), the runtime resumes that task. This cooperative multitasking model allows a small number of threads to handle massive concurrency.

**When to Use Async I/O**

Choose async I/O when:
- Building network servers handling many concurrent connections (web servers, API gateways, chat servers)
- Making many concurrent outbound requests (web scrapers, API aggregators, microservices)
- Building real-time systems where latency matters more than raw throughput
- Working with I/O-bound workloads where tasks spend most time waiting

Stick with synchronous I/O when:
- Building simple CLI tools or scripts with minimal concurrency
- Working with CPU-bound workloads (parsing, computation, compression)
- Interfacing with blocking APIs that don't have async alternatives
- Prototyping or learning—sync code is simpler to understand

**The Tokio Runtime**

Tokio is Rust's most mature async runtime, providing:
- A multi-threaded work-stealing scheduler that balances tasks across CPU cores
- Async versions of standard I/O primitives (`File`, `TcpStream`, `UdpSocket`)
- Timers, timeouts, and delays
- Synchronization primitives (`Mutex`, `Semaphore`, `RwLock`)
- Utilities for backpressure, cancellation, and graceful shutdown

The runtime uses a work-stealing scheduler: if one thread runs out of tasks, it "steals" tasks from other threads' queues. This ensures CPU cores stay busy and tasks get distributed evenly. The `#[tokio::main]` macro sets up this runtime and makes your `main` function async.

**Key Design Principles**

1. **Cooperative Multitasking**: Tasks must yield control regularly. A task that never awaits will starve other tasks on the same thread.

2. **Structured Concurrency**: Use `tokio::spawn` to create independent tasks, but prefer combinators like `join!` or `select!` for coordinated concurrency.

3. **Backpressure**: Design systems that can slow down producers when consumers can't keep up. Unbounded queues lead to memory exhaustion.

4. **Cancellation Safety**: Operations should clean up properly when canceled (via timeouts, dropped futures, or explicit cancellation tokens).

Let's explore the patterns that make async I/O powerful and safe.

---

## Tokio File and Network I/O

### Basic Async File Operations

File I/O in Tokio mirrors the synchronous API from `std::fs`, but every operation returns a future that you must `.await`. Under the hood, Tokio uses a thread pool for file operations (because most operating systems don't provide truly async file I/O APIs). This means file operations still block—just on background threads instead of your async tasks.

**When to use async file I/O:**
- When you need to perform file operations alongside network I/O without blocking your async runtime
- When reading/writing many files concurrently (Tokio's thread pool parallelizes the work)
- When building services that mix file and network operations

**When synchronous file I/O is better:**
- For simple scripts or CLI tools where blocking is acceptable
- When doing sequential file processing with no other concurrent work
- When every operation is a file operation (no benefit from async)

```rust
//=========================
// Note: Add to Cargo.toml:
//=========================
// [dependencies]
//===============================================
// tokio = { version = "1", features = ["full"] }
//===============================================

use tokio::fs::{File, OpenOptions};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

//===============================
// Read entire file into a String
//===============================
// Use this for: Config files, small text files, HTML templates
//==================================================================
// Avoid for: Large files (>100MB), binary data where you need &[u8]
//==================================================================
async fn read_file(path: &str) -> io::Result<String> {
    tokio::fs::read_to_string(path).await
    // Convenience method: allocates a String, reads entire file
    // Returns Err if file missing, unreadable, or contains invalid UTF-8
}

//==============================
// Read entire file into Vec<u8>
//==============================
// Use this for: Binary files, images, data files, when you need raw bytes
async fn read_bytes(path: &str) -> io::Result<Vec<u8>> {
    tokio::fs::read(path).await
    // Reads entire file into memory
    // More efficient than read_to_string for binary data
}

//===================================================
// Write string to file (overwrites existing content)
//===================================================
// Use this for: Writing config files, logs, generated HTML
async fn write_file(path: &str, content: &str) -> io::Result<()> {
    tokio::fs::write(path, content).await
    // Convenience method: creates file, writes content, closes file
    // Overwrites existing file! Use append_to_file if you want to append
}

//================================
// Manual read with buffer control
//================================
// Use this when: You need precise control over memory allocation
//================================================
// or want to reuse a buffer across multiple reads
//================================================
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
// Use this when: You need to write multiple chunks or ensure flushing
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
// Use this for: Log files, audit trails, incremental data collection
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
// Use this when: Copying large files alongside other async work
//===================================
// Returns the number of bytes copied
//===================================
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

### Async Line Reading

Reading line-by-line is essential for processing log files, CSV data, and other line-oriented formats. The async version uses `BufReader` to buffer reads efficiently, minimizing system calls.

**Why buffering matters:** Unbuffered reads perform one system call per byte or small chunk, which is catastrophically slow. A `BufReader` reads large chunks (8KB by default) into an internal buffer, then serves bytes from that buffer. This reduces system calls by 100-1000x, making line-by-line reading practical.

```rust
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, BufReader};

//===========================
// Read all lines into memory
//===========================
// Use this for: Small to medium files where you need all lines at once
//===========================================================================
// Avoid for: Large files (>1GB) where you should process lines one at a time
//===========================================================================
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
// Use this for: Log analysis, large CSV files, data processing pipelines
//============================================================
// This is memory-efficient: only one line in memory at a time
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
// Use this for: Reading CSV headers, previewing log files, sampling data
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

### TCP Network I/O

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
// This is the canonical async server pattern: accept connections in a loop,
//=======================================================================
// spawn a task for each connection. The runtime handles task scheduling.
//=======================================================================
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
// Connect to a server, send a message, read the response
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
// This demonstrates a more realistic server that reads a request,
//===================================
// processes it, and sends a response
//===================================
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

### UDP Network I/O

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
// UDP is connectionless, so there's no accept() loop.
//==============================================================
// Instead, we receive datagrams from anyone and echo them back.
//==============================================================
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
// Send a datagram and wait for a response
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

### Unix Domain Sockets

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

## Buffered Async Streams

### Using BufReader and BufWriter

Buffering is critical for I/O performance. Without buffering, every `read()` or `write()` call becomes a system call, which is expensive (context switch to kernel mode, potential scheduling delay, cache pollution). Buffering amortizes these costs by performing fewer, larger I/O operations.

**The Buffering Trade-off**

- **BufReader**: Reads large chunks from the underlying source, serves bytes from an internal buffer. Dramatically reduces system calls for small reads.
- **BufWriter**: Accumulates writes in a buffer, flushes to the underlying sink when the buffer is full or `.flush()` is called. Critical for performance when making many small writes.

**When to adjust buffer sizes:**
- Default buffer size is 8KB, which is good for most workloads
- Increase buffer size (16KB-64KB) for high-bandwidth sequential I/O
- Decrease buffer size (1KB-4KB) for low-latency interactive protocols

```rust
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

//=========================================
// Buffered reading with custom buffer size
//=========================================
// Use this for: Log files, line-oriented data, any sequential reading
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
// Use this for: Writing many small chunks (logs, CSV rows, JSON lines)
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
// tokio::io::copy automatically uses buffering for efficiency
async fn buffered_copy(src: &str, dst: &str) -> io::Result<u64> {
    let src_file = File::open(src).await?;
    let dst_file = File::create(dst).await?;

    let mut reader = BufReader::new(src_file);
    let mut writer = BufWriter::new(dst_file);

    // copy() efficiently transfers data, using the buffers to minimize system calls
    tokio::io::copy(&mut reader, &mut writer).await
}
```

### Stream Processing with AsyncRead/AsyncWrite

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
// This demonstrates the AsyncRead trait: poll_read() is called
//==============================================================
// by the runtime to read data. We delegate to the inner reader,
//==============================================================
// then transform the data.
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

### Stream Splitting and Framing

Many protocols benefit from splitting a stream into independent read and write halves. This allows one task to read while another writes, or enables implementing full-duplex protocols where requests and responses flow concurrently.

```rust
use tokio::net::TcpStream;
use tokio::io::{self, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

//========================================
// Split stream into read and write halves
//========================================
// This pattern is essential for full-duplex protocols where you
//====================================
// need to read and write concurrently
//====================================
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

### Using tokio_util::codec for Framing

Framing solves a fundamental problem in network protocols: how do you know where one message ends and the next begins? Raw TCP gives you a byte stream with no message boundaries. Codecs provide message framing—they handle splitting the stream into discrete messages and vice versa.

**Common framing strategies:**
- **Line-delimited**: Messages separated by newlines (simple, human-readable)
- **Length-prefixed**: Each message starts with its length (efficient, binary-safe)
- **Fixed-size**: All messages are the same size (simple but inflexible)
- **Delimiter-based**: Messages separated by a special byte sequence (like HTTP headers separated by `\r\n\r\n`)

```rust
//===================
// Add to Cargo.toml:
//===================
// tokio-util = { version = "0.7", features = ["codec"] }

use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};
use futures::{SinkExt, StreamExt};

//=====================
// Line-delimited codec
//=====================
// This is the simplest framing: messages are separated by newlines
//======================================================================
// Use this for: Text-based protocols, CLI tools, human-readable formats
//======================================================================
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
// This is the most efficient binary framing: each message is prefixed
//======================================
// with a 4-byte length (big-endian u32)
//======================================
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

## Backpressure Handling

Backpressure is the mechanism by which a slow consumer signals to a fast producer to slow down. Without backpressure, the producer will overwhelm the consumer, causing unbounded memory growth, dropped messages, or system crashes.

**The Backpressure Problem**

Imagine a web scraper that fetches pages faster than it can parse them. Without backpressure:
1. Fetched pages accumulate in an unbounded queue
2. Memory usage grows without limit
3. Eventually, the system runs out of memory and crashes

With backpressure:
1. When the queue is full, the producer waits before fetching more pages
2. Memory usage stays bounded
3. The system remains stable

### Manual Backpressure with Channels

Bounded channels provide automatic backpressure: when the channel is full, `send()` blocks until space is available. This naturally slows down producers to match consumer speed.

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

//===========================
// Producer with backpressure
//===========================
// send() blocks when the channel is full, providing natural backpressure
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
// The channel capacity determines how much buffering we allow
async fn backpressure_example() {
    // Bounded channel with capacity 10
    // Producer can get ahead by 10 items, then must wait
    let (tx, rx) = mpsc::channel(10);

    let producer = tokio::spawn(producer_with_backpressure(tx));
    let consumer = tokio::spawn(consumer(rx));

    let _ = tokio::join!(producer, consumer);
}
```

### Stream Backpressure with Buffering

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

### Rate Limiting

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
// Allows up to max_per_second operations per second
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

### Semaphore for Concurrency Control

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

### Flow Control in Network Servers

Limiting concurrent connections prevents resource exhaustion and ensures fair service under load.

```rust
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use std::sync::Arc;

//=================================
// TCP server with connection limit
//=================================
// This pattern prevents the server from accepting more connections
//===========================================================
// than it can handle, protecting against resource exhaustion
//===========================================================
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

## Connection Pooling

Connection pooling reuses expensive resources (database connections, HTTP clients) across multiple operations, amortizing the setup cost and limiting the total number of connections.

**Why connection pooling matters:**
- Establishing a connection is expensive (TCP handshake, TLS negotiation, authentication)
- Databases and APIs limit the number of concurrent connections
- Creating a new connection for each request is wastefully slow

A connection pool maintains a set of ready-to-use connections. When you need a connection, you acquire one from the pool; when done, you return it to the pool for reuse.

### Simple Connection Pool

This is a basic pool implementation to illustrate the concepts. In production, use a mature library like `deadpool` or `bb8`.

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

### Advanced Pool with deadpool

`deadpool` is a production-ready connection pool library with features like connection recycling, timeouts, and health checks.

```rust
//===================
// Add to Cargo.toml:
//===================
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

### HTTP Client Connection Pool

The `reqwest` HTTP client has built-in connection pooling, which is essential for making many HTTP requests efficiently.

```rust
//===================
// Add to Cargo.toml:
//===================
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

---

## Timeout and Cancellation

Timeouts and cancellation are essential for building reliable systems. Without timeouts, a single slow operation can hang your entire service. Without cancellation, you can't stop long-running work when it's no longer needed.

### Basic Timeouts

Tokio's `timeout()` function wraps any future and returns an error if it doesn't complete within the specified duration.

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

### Timeout with Fallback

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

### Cancellation with CancellationToken

`CancellationToken` provides a mechanism for coordinated cancellation across multiple tasks. When you cancel the token, all tasks listening to it are notified.

**When to use CancellationToken:**
- Implementing graceful shutdown (cancel all background tasks)
- Stopping a group of related tasks when one fails
- Implementing request cancellation in a server

```rust
//===================
// Add to Cargo.toml:
//===================
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

### Select for Racing Operations

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
// Use this when you want to prioritize certain operations
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

### Graceful Shutdown

Graceful shutdown ensures that all in-flight requests complete before the server stops, preventing data loss or corrupted state.

```rust
use tokio::signal;
use tokio::sync::broadcast;

//==================================
// TCP server with graceful shutdown
//==================================
// When Ctrl+C is pressed, the server stops accepting new connections
//=============================================
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

### Timeout for Multiple Operations

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

### Retry with Timeout

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

### Deadline-based Cancellation

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

---

This comprehensive guide covers all major async I/O patterns in Rust with Tokio. The key to mastering async Rust is understanding when to use async (I/O-bound workloads with high concurrency) versus sync (simple scripts, CPU-bound workloads), and how to handle the fundamental challenges of backpressure, cancellation, and resource pooling. These patterns form the foundation for building scalable, reliable network services.
