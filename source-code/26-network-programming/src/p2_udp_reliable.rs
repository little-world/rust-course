//! Pattern 2: Reliable UDP Pattern
//!
//! Demonstrates adding reliability to UDP with timeouts and retries.
//! This pattern is the basis for protocols like DNS and QUIC.

use tokio::net::UdpSocket;
use tokio::time::{timeout, Duration};

/// Send a UDP request with retries
async fn reliable_udp_request(
    socket: &UdpSocket,
    message: &[u8],
    server_addr: &str,
    max_retries: usize,
) -> tokio::io::Result<Vec<u8>> {
    let mut buffer = vec![0u8; 1024];

    for attempt in 0..max_retries {
        // Send the request
        println!("  Attempt {}: Sending to {}", attempt + 1, server_addr);
        socket.send_to(message, server_addr).await?;

        // Wait for response with timeout
        match timeout(Duration::from_secs(2), socket.recv_from(&mut buffer)).await {
            Ok(Ok((len, addr))) => {
                // Success! Return the response
                println!("  Success! Received {} bytes from {}", len, addr);
                return Ok(buffer[..len].to_vec());
            }
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                // Timeout - retry
                println!("  Attempt {} timed out after 2s, retrying...", attempt + 1);
                continue;
            }
        }
    }

    Err(tokio::io::Error::new(
        tokio::io::ErrorKind::TimedOut,
        "All retry attempts failed"
    ))
}

/// Simple unreliable server that sometimes doesn't respond
async fn unreliable_server() -> tokio::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:8082").await?;
    println!("Unreliable server listening on port 8082");
    println!("(Responds to ~50% of messages to simulate packet loss)\n");

    let mut buffer = vec![0u8; 1024];
    let mut counter = 0;

    loop {
        let (len, addr) = socket.recv_from(&mut buffer).await?;
        counter += 1;

        let message = String::from_utf8_lossy(&buffer[..len]);
        println!("[Server] Received from {}: {}", addr, message.trim());

        // Simulate 50% packet loss
        if counter % 2 == 0 {
            println!("[Server] Simulating packet loss - not responding");
            continue;
        }

        println!("[Server] Responding...");
        socket.send_to(b"ACK", addr).await?;
    }
}

async fn run_client() -> tokio::io::Result<()> {
    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    println!("\n--- Reliable UDP Client ---");
    println!("Will retry up to 3 times with 2s timeout per attempt\n");

    let messages = ["Message 1", "Message 2", "Message 3", "Message 4"];

    for msg in messages {
        println!("Sending: {}", msg);

        match reliable_udp_request(&socket, msg.as_bytes(), "127.0.0.1:8082", 3).await {
            Ok(response) => {
                println!("  Response: {}\n", String::from_utf8_lossy(&response));
            }
            Err(e) => {
                println!("  Failed after all retries: {}\n", e);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    println!("=== Pattern 2: Reliable UDP with Retries ===\n");

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "server" {
        // Run as server only
        unreliable_server().await?;
    } else if args.len() > 1 && args[1] == "client" {
        // Run as client only
        run_client().await?;
    } else {
        // Run both server and client together
        println!("Running both server and client for demo.");
        println!("Run separately with: --server or --client\n");

        let server_handle = tokio::spawn(async {
            unreliable_server().await
        });

        let client_handle = tokio::spawn(async {
            run_client().await
        });

        // Wait for client to finish, then stop
        let _ = client_handle.await;
        println!("\nDemo complete! Press Ctrl+C to stop server.");

        // Keep server running so user can test manually
        let _ = server_handle.await;
    }

    Ok(())
}
