//! Pattern 2: UDP Broadcast
//!
//! Demonstrates broadcasting UDP messages to all hosts on the local network.
//! Used for service discovery protocols like mDNS, SSDP (UPnP), and DHCP.

use tokio::net::UdpSocket;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Broadcast a message to all hosts on the local network
async fn udp_broadcast() -> tokio::io::Result<()> {
    println!("--- UDP Broadcaster ---\n");

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    println!("Broadcaster bound to {}", socket.local_addr()?);

    // Enable broadcast
    socket.set_broadcast(true)?;
    println!("Broadcast enabled");

    // Broadcast address (255.255.255.255 reaches all hosts on local network)
    let broadcast_addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)),
        8081
    );

    let message = b"Service Discovery Request";
    socket.send_to(message, broadcast_addr).await?;

    println!("Broadcast sent to {}: {:?}", broadcast_addr, String::from_utf8_lossy(message));
    Ok(())
}

/// Listen for broadcast messages
async fn udp_broadcast_listener() -> tokio::io::Result<()> {
    println!("--- UDP Broadcast Listener ---\n");

    let socket = UdpSocket::bind("0.0.0.0:8081").await?;
    socket.set_broadcast(true)?;
    println!("Listening for broadcasts on port 8081...");

    let mut buffer = vec![0u8; 1024];

    // Listen for a few seconds
    let timeout = tokio::time::timeout(
        tokio::time::Duration::from_secs(10),
        async {
            loop {
                let (len, addr) = socket.recv_from(&mut buffer).await?;
                println!("Broadcast from {}: {}",
                    addr,
                    String::from_utf8_lossy(&buffer[..len])
                );
            }
            #[allow(unreachable_code)]
            Ok::<(), tokio::io::Error>(())
        }
    );

    match timeout.await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => eprintln!("Error: {}", e),
        Err(_) => println!("\nListener timeout (10 seconds)"),
    }

    Ok(())
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    println!("=== Pattern 2: UDP Broadcast ===\n");

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "listen" {
        // Run as listener
        udp_broadcast_listener().await?;
    } else {
        // Run as broadcaster
        println!("Running as broadcaster.");
        println!("To listen: cargo run --bin p2_udp_broadcast -- listen\n");

        udp_broadcast().await?;

        println!("\nBroadcast complete!");
        println!("Note: Run listener in another terminal to receive broadcasts.");
    }

    Ok(())
}
