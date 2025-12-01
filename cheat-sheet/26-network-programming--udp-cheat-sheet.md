### UDP Cheat Sheet
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