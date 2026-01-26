//! Pattern 2: UDP Client
//!
//! Demonstrates sending UDP datagrams to a server and receiving responses.

use tokio::net::UdpSocket;

/// Send a UDP message and wait for a response
#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    println!("=== Pattern 2: UDP Client ===\n");
    println!("Make sure UDP server is running: cargo run --bin p2_udp_server\n");

    // Bind to any available port
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let local_addr = socket.local_addr()?;
    println!("Client bound to {}", local_addr);

    // Connect sets the default destination
    // This doesn't establish a connection (UDP is connectionless)
    // but allows using send/recv instead of send_to/recv_from
    socket.connect("127.0.0.1:8080").await?;
    println!("Connected to 127.0.0.1:8080\n");

    // Send some messages
    let messages = [
        "Hello, UDP Server!",
        "This is a test message",
        "UDP is connectionless",
        "Goodbye!",
    ];

    for message in messages {
        println!("Sending: {}", message);
        socket.send(message.as_bytes()).await?;

        // Wait for a response
        let mut buffer = vec![0u8; 1024];
        let len = socket.recv(&mut buffer).await?;

        println!("Received: {}\n", String::from_utf8_lossy(&buffer[..len]));

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    println!("UDP client demo completed!");

    Ok(())
}
