// Pattern 1: UDP Network I/O
use std::io;
use tokio::net::UdpSocket;
use tokio::time::{timeout, Duration};

// UDP Echo Server
async fn udp_server(addr: &str) -> io::Result<()> {
    let socket = UdpSocket::bind(addr).await?;
    println!("UDP server listening on {}", addr);

    let mut buffer = [0; 1024];

    loop {
        // recv_from() awaits a datagram from any sender
        // Returns (bytes_received, sender_address)
        let (len, addr) = socket.recv_from(&mut buffer).await?;
        println!("Received {} bytes from {}: {:?}",
                 len, addr, String::from_utf8_lossy(&buffer[..len]));

        // Echo the datagram back to the sender
        // UDP doesn't guarantee delivery, so send_to() might succeed
        // even if the datagram never arrives
        socket.send_to(&buffer[..len], addr).await?;
        println!("Echoed back to {}", addr);
    }
}

// UDP Client
async fn udp_client(server_addr: &str, message: &str) -> io::Result<String> {
    // Bind to a random local port (0.0.0.0:0 means "any port")
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    // Send datagram to server
    socket.send_to(message.as_bytes(), server_addr).await?;

    // Wait for response with timeout (UDP responses may never arrive)
    let mut buffer = [0; 1024];

    let result = timeout(
        Duration::from_secs(3),
        socket.recv_from(&mut buffer)
    ).await;

    match result {
        Ok(Ok((len, _))) => Ok(String::from_utf8_lossy(&buffer[..len]).to_string()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(io::Error::new(io::ErrorKind::TimedOut, "UDP response timed out")),
    }
}

// UDP broadcast example
async fn udp_broadcast(port: u16, message: &str) -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    // Enable broadcast
    socket.set_broadcast(true)?;

    // Send to broadcast address
    let broadcast_addr = format!("255.255.255.255:{}", port);
    socket.send_to(message.as_bytes(), &broadcast_addr).await?;
    println!("Broadcast sent to {}", broadcast_addr);

    Ok(())
}

// Run server in a task and test with client
async fn demo_udp() -> io::Result<()> {
    let server_addr = "127.0.0.1:8888";

    // Start server in background task
    let server_handle = tokio::spawn(async move {
        if let Err(e) = udp_server(server_addr).await {
            eprintln!("Server error: {}", e);
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test with client
    println!("\n=== UDP Client Test ===");

    let messages = vec!["Hello UDP!", "Ping", "Test message 123"];

    for msg in messages {
        println!("Sending: {}", msg);
        match udp_client(server_addr, msg).await {
            Ok(response) => println!("Response: {}\n", response),
            Err(e) => println!("Error: {}\n", e),
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    // Abort the server (it runs forever otherwise)
    server_handle.abort();

    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("=== UDP Demo ===\n");

    // Check for command line argument to run server only
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "server" {
        println!("Running UDP server only mode");
        println!("Use 'nc -u localhost 8888' to test\n");
        return udp_server("127.0.0.1:8888").await;
    }

    // Run integrated demo
    demo_udp().await?;

    println!("UDP demo completed");
    Ok(())
}
