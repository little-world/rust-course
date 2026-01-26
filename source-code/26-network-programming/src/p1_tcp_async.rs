//! Pattern 1: Async TCP Server with Tokio
//!
//! Demonstrates using Tokio for async I/O, handling thousands of
//! concurrent connections efficiently with lightweight tasks.

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Async echo server using Tokio
/// Can handle thousands of concurrent connections efficiently
#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    println!("=== Pattern 1: Async TCP Server with Tokio ===\n");

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Async server listening on port 8080");
    println!("Connect with: nc localhost 8080");
    println!("Can handle 10,000+ concurrent connections!\n");

    loop {
        // Accept is async - other tasks can run while waiting
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);

        // Spawn an async task for this connection
        // Tasks are much cheaper than threads (~1KB vs ~2MB)
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket).await {
                eprintln!("Error handling {}: {}", addr, e);
            }
        });
    }
}

async fn handle_connection(mut socket: TcpStream) -> tokio::io::Result<()> {
    let mut buffer = vec![0; 1024];

    loop {
        // Async read - yields to other tasks while waiting for data
        let n = socket.read(&mut buffer).await?;

        if n == 0 {
            // Connection closed
            println!("Connection closed");
            return Ok(());
        }

        let received = String::from_utf8_lossy(&buffer[..n]);
        println!("Received: {}", received.trim());

        // Echo the data back
        socket.write_all(&buffer[..n]).await?;
    }
}
