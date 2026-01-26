// Pattern 1: TCP Echo Server
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

// Handle a single client connection (echo protocol)
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

        println!("Received {} bytes: {:?}", n, String::from_utf8_lossy(&buffer[..n]));

        // Echo the data back to the client
        // write_all() ensures all bytes are sent (loops if the write is partial)
        socket.write_all(&buffer[..n]).await?;
    }
}

// TCP Echo Server
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
            println!("Client {} disconnected", addr);
        });
    }
    // Note: This loop never exits. In production, you'd add graceful shutdown.
}

// HTTP-like request handling (simplified)
async fn http_handler(mut socket: TcpStream) -> io::Result<()> {
    let mut buffer = [0; 4096];

    // Read the HTTP request
    let n = socket.read(&mut buffer).await?;

    let request = String::from_utf8_lossy(&buffer[..n]);
    println!("Request:\n{}", request);

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

// Simple HTTP server
async fn run_http_server(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("HTTP server listening on {}", addr);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("HTTP request from {}", addr);

        tokio::spawn(async move {
            if let Err(e) = http_handler(socket).await {
                eprintln!("HTTP error: {}", e);
            }
        });
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("=== TCP Server Demo ===");
    println!("Starting echo server on 127.0.0.1:8080");
    println!("Use 'nc localhost 8080' or the TCP client to test");
    println!("Press Ctrl+C to stop\n");

    // Run the echo server (this blocks forever)
    run_tcp_server("127.0.0.1:8080").await
}
