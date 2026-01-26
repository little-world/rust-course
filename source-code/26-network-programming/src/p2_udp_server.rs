//! Pattern 2: UDP Echo Server
//!
//! Demonstrates a connectionless UDP server that receives datagrams
//! and echoes them back to the sender.

use tokio::net::UdpSocket;
use std::io;

/// UDP echo server
/// Receives datagrams and echoes them back to the sender
#[tokio::main]
async fn main() -> io::Result<()> {
    println!("=== Pattern 2: UDP Echo Server ===\n");

    // Bind to a port
    let socket = UdpSocket::bind("127.0.0.1:8080").await?;
    println!("UDP server listening on port 8080");
    println!("Test with: cargo run --bin p2_udp_client");
    println!("Or: echo 'hello' | nc -u localhost 8080\n");

    let mut buffer = vec![0u8; 1024];

    loop {
        // Receive a datagram
        // recv_from returns the number of bytes and the sender's address
        let (len, addr) = socket.recv_from(&mut buffer).await?;

        let message = String::from_utf8_lossy(&buffer[..len]);
        println!("Received {} bytes from {}: {}", len, addr, message.trim());

        // Echo it back
        socket.send_to(&buffer[..len], addr).await?;
        println!("Echoed back to {}", addr);
    }
}
