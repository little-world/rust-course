//! Pattern 1: TCP Client Examples
//!
//! Demonstrates simple and interactive TCP client patterns.

use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt, BufReader};

/// Connect to a server and exchange messages
async fn tcp_client_example() -> tokio::io::Result<()> {
    println!("--- Simple TCP Client ---");

    // Connect to the server
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Connected to server");

    // Send a message
    let message = "Hello, Server!\n";
    stream.write_all(message.as_bytes()).await?;
    println!("Sent: {}", message.trim());

    // Read the response
    let mut buffer = vec![0; 1024];
    let n = stream.read(&mut buffer).await?;

    println!("Received: {}", String::from_utf8_lossy(&buffer[..n]).trim());

    Ok(())
}

/// Interactive client that can send and receive concurrently
async fn interactive_client() -> tokio::io::Result<()> {
    println!("\n--- Interactive TCP Client ---");
    println!("Type messages to send. Press Ctrl+C to exit.\n");

    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Connected to server");

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Spawn a task to handle incoming messages
    let read_handle = tokio::spawn(async move {
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    println!("\nServer closed connection");
                    break;
                }
                Ok(_) => print!("Server: {}", line),
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    break;
                }
            }
        }
    });

    // Main task handles user input and sending
    let write_handle = tokio::spawn(async move {
        let stdin = BufReader::new(tokio::io::stdin());
        let mut lines = stdin.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if writer.write_all(format!("{}\n", line).as_bytes()).await.is_err() {
                break;
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = read_handle => println!("Read task finished"),
        _ = write_handle => println!("Write task finished"),
    }

    Ok(())
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    println!("=== Pattern 1: TCP Client Examples ===\n");
    println!("Make sure a server is running on port 8080 first!\n");

    // Try simple client first
    match tcp_client_example().await {
        Ok(_) => println!("Simple client test completed\n"),
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            eprintln!("Start a server first with: cargo run --bin p1_tcp_async");
            return Ok(());
        }
    }

    // Then try interactive client
    interactive_client().await?;

    Ok(())
}
