//! Pattern 1: Multi-threaded TCP Server
//!
//! Demonstrates spawning a thread per connection to handle multiple clients
//! concurrently. Each thread handles one client independently.

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

/// Multi-threaded server that spawns a thread per client
/// This scales better but can exhaust system resources with many connections
fn multithreaded_server() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Multi-threaded server listening on port 8080");
    println!("Connect with multiple clients: nc localhost 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Spawn a new thread for each connection
                thread::spawn(move || {
                    if let Err(e) = handle_client_thread(stream) {
                        eprintln!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client_thread(mut stream: TcpStream) -> std::io::Result<()> {
    let addr = stream.peer_addr()?;
    println!("Thread handling client: {}", addr);

    let mut buffer = [0; 1024];

    loop {
        let bytes_read = stream.read(&mut buffer)?;

        if bytes_read == 0 {
            println!("Client {} disconnected", addr);
            break;
        }

        let received = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("[{}] Received: {}", addr, received.trim());

        // Echo back
        stream.write_all(&buffer[..bytes_read])?;
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    println!("=== Pattern 1: Multi-threaded TCP Server ===\n");
    println!("This server spawns a thread per connection.\n");

    multithreaded_server()
}
