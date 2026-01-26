// Pattern 1: TCP Client
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

// TCP Client - send message and receive response
async fn tcp_client(addr: &str, message: &str) -> io::Result<String> {
    // connect() performs DNS resolution (if needed) and TCP handshake
    let mut stream = TcpStream::connect(addr).await?;

    // Send message
    stream.write_all(message.as_bytes()).await?;

    // For echo server, we need to signal we're done sending
    // by shutting down the write half
    stream.shutdown().await?;

    // Read response
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;
    // Note: read_to_end() reads until EOF, which means the server
    // must close the connection after responding

    Ok(String::from_utf8_lossy(&buffer).to_string())
}

// TCP Client with timeout
async fn tcp_client_with_timeout(addr: &str, message: &str, timeout_secs: u64) -> io::Result<String> {
    let result = timeout(
        Duration::from_secs(timeout_secs),
        async {
            let mut stream = TcpStream::connect(addr).await?;
            stream.write_all(message.as_bytes()).await?;

            // Read response with a fixed buffer (don't wait for EOF)
            let mut buffer = [0; 1024];
            let n = stream.read(&mut buffer).await?;

            Ok::<_, io::Error>(String::from_utf8_lossy(&buffer[..n]).to_string())
        }
    ).await;

    match result {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(io::Error::new(io::ErrorKind::TimedOut, "Connection timed out")),
    }
}

// Interactive client that sends multiple messages
async fn interactive_client(addr: &str) -> io::Result<()> {
    let mut stream = TcpStream::connect(addr).await?;
    println!("Connected to {}", addr);

    let messages = vec!["Hello!", "How are you?", "Goodbye!"];

    for msg in messages {
        println!("Sending: {}", msg);
        stream.write_all(msg.as_bytes()).await?;

        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).await?;

        if n == 0 {
            println!("Server closed connection");
            break;
        }

        println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));

        // Small delay between messages
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("=== TCP Client Demo ===\n");

    let server_addr = "127.0.0.1:8080";

    // Try to connect with timeout
    println!("Attempting to connect to {}...", server_addr);

    match tcp_client_with_timeout(server_addr, "Hello, Server!", 3).await {
        Ok(response) => {
            println!("Server response: {}", response);
        }
        Err(e) => {
            println!("Connection failed: {}", e);
            println!("\nMake sure to start the TCP server first:");
            println!("  cargo run --bin p1_tcp_server");
            return Ok(());
        }
    }

    // Interactive client demo
    println!("\n=== Interactive Client ===");
    match interactive_client(server_addr).await {
        Ok(()) => println!("Interactive session completed"),
        Err(e) => println!("Interactive session error: {}", e),
    }

    println!("\nTCP client demo completed");
    Ok(())
}
