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