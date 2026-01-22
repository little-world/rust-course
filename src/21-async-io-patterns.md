# Async I/O Patterns
This chapter explores async I/O patterns using Tokio. Handling thousands of concurrent connections, buffered streams, backpressure, connection pooling, and timeouts. Async I/O allows one thread to manage thousands of operations by yielding when blocked, solving the C10K problem.


## Pattern 1: Tokio File and Network I/O

**Problem**: Synchronous I/O blocks threads waiting for data. One thread per connection doesn't scale—10K connections need 20GB RAM (2MB stack each).

**Solution**: Use Tokio async I/O where operations return futures you `.await`. TcpListener/TcpStream for network accept/read/write.

**Why It Matters**: Single thread handles 10K connections in 10MB memory (vs 20GB threaded). Solves C10K problem—web servers serve 10K+ concurrent users.

**Use Cases**: Web servers (HTTP, HTTPS), API gateways, chat servers, WebSocket servers, game servers, HTTP clients (concurrent requests), file servers (mixing file and network I/O), concurrent file processing, real-time data pipelines.

### Example: Read entire file into a String
`tokio::fs::read_to_string` loads the entire file into memory as a String, handling open/read/UTF-8 conversion in one call.
Use for config files and small data files; for large files or binary data, use streaming to avoid loading everything into memory.

```rust
async fn read_file(path: &str) -> io::Result<String> {
    tokio::fs::read_to_string(path).await
    // Convenience method: allocates a String, reads entire file
    // Returns Err if file missing, unreadable, or contains invalid UTF-8
}
let content = read_file("config.json").await?;

```

### Example: Read entire file into Vec<u8>
`tokio::fs::read` loads a file's raw bytes into a `Vec<u8>` without UTF-8 validation—ideal for binary files (images, serialized data).
The entire file loads into memory at once; for large files, consider memory-mapped I/O or streaming reads.

```rust
async fn read_bytes(path: &str) -> io::Result<Vec<u8>> {
    tokio::fs::read(path).await
    // Reads entire file into memory
    // More efficient than read_to_string for binary data
}
let bytes = read_bytes("image.png").await?;

```

### Example: Write string to file (overwrites existing content)
`tokio::fs::write` creates/truncates a file and writes content atomically—handles creation, writing, and closing in one call.
Warning: this replaces existing content entirely. Use `OpenOptions` with `append(true)` for appending; parent directories must exist.

```rust
async fn write_file(path: &str, content: &str) -> io::Result<()> {
    tokio::fs::write(path, content).await
    // Convenience method: creates file, writes content, closes file
    // Overwrites existing file! Use append_to_file if you want to append
}
write_file("output.txt", "Hello async!").await?;

```

### Example: Manual read with buffer control
Manual file handling gives you control over buffer allocation and read patterns via `File::open`.
`read_to_end` appends to the provided buffer, allowing you to pre-allocate or reuse buffers.

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
`File::create` opens a file for writing; `write_all` ensures every byte is written, retrying partial writes automatically.
`flush` forces buffered data to the OS—without it, data might remain in Tokio's internal buffers.

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
`OpenOptions` provides fine-grained control over file opening; `append(true)` positions writes at end of file.
`create(true)` creates the file if it doesn't exist, making this pattern idempotent for log files.

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
append_to_file("log.txt", "New entry").await?;

```

### Example: Copy file asynchronously
`tokio::fs::copy` duplicates a file using OS-optimized operations (e.g., copy_file_range on Linux), returning bytes copied.
The operation is atomic from the caller's perspective; for very large files or progress callbacks, consider manual chunked copying.

```rust
async fn copy_file(src: &str, dst: &str) -> io::Result<u64> {
    tokio::fs::copy(src, dst).await
    // Efficiently copies src to dst using OS-level optimizations when possible
}
let bytes_copied = copy_file("src.txt", "dst.txt").await?;

```

### Example: Example usage
The `#[tokio::main]` macro transforms `main` into an async entry point by creating a Tokio runtime.
Error propagation with `?` works like sync code; this is the standard pattern for Tokio application entry points.

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


### Example: Read all lines into memory
`BufReader::new` wraps a file with an 8KB buffer; `lines()` returns an async iterator yielding one line at a time.
This loads all lines into a `Vec`, suitable for small-to-medium files where you need random access to lines.

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
let lines = read_lines("data.txt").await?;

```

### Example: Process large file line by line without loading into memory
Streaming line-by-line processing keeps memory usage constant regardless of file size.
Each line is processed and dropped before reading the next; `while let Some(line)` handles end-of-file naturally.

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
process_large_file("access.log").await?; // Memory-efficient

```

### Example: Read first N lines (useful for previews or headers)
Reading a fixed number of lines is efficient for previewing files or parsing headers like CSV column names.
The loop terminates early if the file has fewer than N lines—avoids reading entire file for large files.

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
let header = read_first_n_lines("file.csv", 1).await?;
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


### Example: TCP Echo Server
`TcpListener::bind` creates a server socket; the loop calls `accept().await` which yields until a client connects.
`tokio::spawn` creates a lightweight task per client—this "spawn per connection" pattern is the foundation of scalable async servers.

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
run_tcp_server("127.0.0.1:8080").await?; // Starts echo server

```

### Example: Handle a single client connection (echo protocol)
The handler reads and echoes data until disconnect; `read().await` returns bytes read or 0 on EOF (client closed).
`write_all` ensures all bytes are sent; this read-until-EOF loop pattern is the core of request-response protocols.

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
`TcpStream::connect` performs async DNS resolution and TCP handshake; the client sends with `write_all` and reads with `read_to_end`.
Note: `read_to_end` blocks until server closes connection; for streaming protocols, use framing to know message boundaries.

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
let resp = tcp_client("127.0.0.1:8080", "Hello").await?;

```

### Example: HTTP-like request handling (simplified)
This demonstrates basic HTTP handler structure—read request, parse it, send response with proper format (status line, headers, body).
In production, use frameworks like `hyper` or `axum` that handle parsing, routing, and protocol compliance.

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
tokio::spawn(async { http_handler(socket).await }); // Per-connection
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



### Example: UDP Echo Server
`UdpSocket::bind` creates a connectionless socket; `recv_from` waits for datagrams, returning data and sender's address.
Unlike TCP, UDP has no connection state—datagrams are independent and may arrive out of order; delivery is not guaranteed.

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
udp_server("0.0.0.0:8888").await?; // Starts UDP echo server

```

### Example: UDP Client
Binding to `0.0.0.0:0` lets the OS assign an ephemeral port; `send_to` transmits datagrams with no delivery guarantee.
`recv_from` waits for a response; add timeouts in production since UDP responses may never arrive.

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
let resp = udp_client("127.0.0.1:8888", "ping").await?;
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
    unix_server("/tmp/my.sock").await?; unix_client("/tmp/my.sock", "msg").await?;
}
```


---

## Pattern 2: Buffered Async Streams

**Problem**: Byte-by-byte async reads/writes trigger many syscalls—catastrophically inefficient. Need to batch I/O operations.

**Solution**: Use AsyncBufReadExt trait with read_line(), lines() for text. BufReader wraps async readers (File, TcpStream) with 8KB default buffer.

**Why It Matters**: Buffering reduces syscalls by 100x (byte-by-byte → chunked). Reading 1MB file: unbuffered = 1M syscalls, buffered = ~128 syscalls.

**Use Cases**: Line-based protocols (HTTP headers, SMTP, Redis protocol), chat protocols (newline-delimited), log streaming, CSV/JSON over network, codec-based protocols (protobuf, MessagePack), WebSocket framing, custom protocol parsers.


### Example: Buffered reading with custom buffer size
`BufReader::with_capacity` tunes buffer size—larger buffers mean fewer syscalls but more memory (default 8KB works for most cases).
Profile your workload to find optimal size; common choices are 8KB, 64KB, or matching your filesystem's block size.

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
buffered_read("large.log").await?; // Efficient line-by-line

```

### Example: Buffered writing with custom buffer size
`BufWriter` accumulates writes in memory, flushing when buffer fills—multiple small writes become one syscall.
The final `flush()` is critical: without it, buffered data may be lost if the writer is dropped.

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
buffered_write("out.txt", &["line1", "line2"]).await?;

```

### Example: Copy with buffering
Combining `BufReader` and `BufWriter` makes copying efficient; `tokio::io::copy` transfers data between any async streams.
More flexible than `tokio::fs::copy`—use for copying between sockets/files or when transforming data during copy.

```rust
async fn buffered_copy(src: &str, dst: &str) -> io::Result<u64> {
    let src_file = File::open(src).await?;
    let dst_file = File::create(dst).await?;

    let mut reader = BufReader::new(src_file);
    let mut writer = BufWriter::new(dst_file);

    // copy() efficiently transfers data, using the buffers to minimize system calls
    tokio::io::copy(&mut reader, &mut writer).await
}
let bytes = buffered_copy("src.txt", "dst.txt").await?;
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
Implementing `AsyncRead` creates adapters that transform data as it flows; `poll_read` delegates then transforms bytes in place.
This pattern enables composable stream processing: chain multiple transformers (uppercase -> encrypt -> compress).

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
// Usage example
async fn use_uppercase_reader() -> io::Result<()> {
    use tokio::fs::File;

    let file = File::open("input.txt").await?;
    let mut reader = UppercaseReader { inner: file };

    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).await?;
    println!("Uppercased: {}", buffer);

    Ok(())
}
use_uppercase_reader().await?; // Transforms file content to uppercase
```
### Example: Split stream into read and write halves
Many protocols benefit from splitting a stream into independent read and write halves. This allows one task to read while another writes, or enables implementing full-duplex protocols where requests and responses flow concurrently.

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
split_stream_example().await?; // Independent read/write tasks
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
framed_lines().await?; // Send/receive newline-delimited messages

```

### Example: Custom codec for length-prefixed messages
Length-prefixed framing prepends each message with its byte length—efficient and binary-safe.
The `Decoder` reads the 4-byte length prefix, waits for the full message; `BytesMut` provides zero-copy buffer management.

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
length_prefixed_example().await?; // Binary protocol framing
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
The producer generates data fast, but `send().await` blocks when the channel is full—the backpressure mechanism.
If the receiver is dropped, `send()` returns `Err`, allowing graceful shutdown detection.

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
The consumer processes items slowly (100ms each); `recv().await` returns `Some(value)` or `None` when sender closes.
The channel acts as a buffer, smoothing out differences between producer and consumer speeds.

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
`mpsc::channel(10)` creates a channel with capacity 10—producer can be at most 10 items ahead of consumer.
Both tasks run concurrently via `tokio::spawn`; `tokio::join!` waits for both to complete.

```rust
async fn backpressure_example() {
    // Bounded channel with capacity 10
    // Producer can get ahead by 10 items, then must wait
    let (tx, rx) = mpsc::channel(10);

    let producer = tokio::spawn(producer_with_backpressure(tx));
    let consumer = tokio::spawn(consumer(rx));

    let _ = tokio::join!(producer, consumer);
}
backpressure_example().await; // Producer waits when channel full
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
stream_backpressure().await; // Max 5 concurrent futures
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
The token bucket algorithm limits operations per time window; `acquire()` tracks operations and waits if limit reached.
When the window expires (1 second), the counter resets, allowing a new burst of operations.

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
rate_limited_requests().await; // Max 10 req/sec
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
concurrent_with_limit().await; // Max 5 concurrent tasks
```


### Example: Flow Control in Network Servers

Limiting concurrent connections prevents resource exhaustion and ensures fair service under load.

```rust
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use std::sync::Arc;

```

### Example: TCP server with connection limit
Combining a TCP server with a semaphore limits concurrent connections; `acquire_owned()` gets an owned permit for the task.
When a connection closes, the permit is dropped, freeing a slot for the next waiting connection.

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
server_with_connection_limit("0.0.0.0:8080", 100).await?;
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
`PooledConnection` wraps a connection with RAII semantics—automatically returned to pool when dropped.
The Drop implementation uses `tokio::spawn` to return the connection asynchronously since Drop cannot be async.

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
Twenty tasks compete for five pooled connections; `pool.acquire().await` blocks until a connection is available.
When the guard (`conn`) goes out of scope, the connection automatically returns to the pool for reuse.

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
use_pool().await?; // 20 tasks share 5 pooled connections
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
use_deadpool().await?; // Production-ready pool with health checks
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
http_connection_pool().await?; // Reuses TCP connections per host
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
with_timeout().await?; // Times out after 5s
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
let data = timeout_with_fallback().await; // Uses fallback if primary slow
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
cancellation_example().await; // Cancel worker after 5 seconds
```


### Example: Select for Racing Operations

`tokio::select!` runs multiple futures concurrently and returns as soon as one completes. This is useful for implementing timeouts, racing multiple data sources, or handling multiple events.

```rust
use tokio::time::{sleep, Duration};

```

### Example: Race three operations—return the first to complete
`tokio::select!` races multiple futures, returning as soon as any one completes; losing futures are dropped (cancelled).
Useful for racing redundant requests, implementing speculative execution, or timeout patterns.

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
let winner = race_operations().await; // Returns "B completed"

```

### Example: Biased select (checks branches in order)
The `biased;` directive makes `select!` check branches in declaration order, not randomly.
Use biased when you have a preferred branch (e.g., shutdown signals should take priority over work).

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
graceful_shutdown_server().await?; // Ctrl+C triggers graceful shutdown

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
Wrapping `join_all` with `timeout` applies a single deadline to all operations combined.
If any operation is slow, the entire group fails; the timeout cancels all in-flight operations on expiry.

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
timeout_multiple().await?; // All 3 tasks must complete within 5s

async fn async_task(id: usize) -> io::Result<String> {
    sleep(Duration::from_secs(1)).await;
    Ok(format!("Task {} completed", id))
}

```

### Example: Individual timeouts: each operation has its own timeout
Wrapping each operation individually with `timeout` gives per-operation deadlines—fast ops succeed even if slow ones timeout.
The result is a vector of `Result`s, letting you handle each success/timeout independently.

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
individual_timeouts().await; // Each task has own timeout
```


### Example: Retry with Timeout

Combining retries with timeouts creates resilient operations that handle transient failures but don't hang forever.

```rust
use tokio::time::{sleep, timeout, Duration};

```

### Example: Generic retry logic with timeout and exponential backoff
This generic retry function combines timeouts with exponential backoff; failures trigger retries with increasing delay.
The generic parameters accept any async operation, making this reusable across HTTP calls, database queries, etc.

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
The retry function handles an unreliable operation that randomly fails 70% of the time.
The `|| async { ... }` closure creates a fresh future for each attempt—necessary because futures cannot be restarted.

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
use_retry().await?; // Retries up to 5 times with exponential backoff
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
deadline_based_processing().await?; // All tasks share 10s deadline
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
