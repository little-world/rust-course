### TCP Cheat Sheet
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