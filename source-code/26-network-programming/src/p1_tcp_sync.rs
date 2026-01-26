//! Pattern 1: Synchronous TCP Echo Server
//!
//! Demonstrates a basic blocking TCP server that handles one client at a time.
//! This shows the fundamental TCP concepts before introducing async patterns.

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

/// A basic echo server that handles one client at a time
/// This is synchronous and will block on each operation
fn simple_echo_server() -> std::io::Result<()> {
    // Bind to localhost on port 8080
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server listening on port 8080");
    println!("Connect with: nc localhost 8080");
    println!("Or run: cargo run --bin p1_tcp_client");

    // Accept connections in a loop
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection from: {}", stream.peer_addr()?);
                handle_client(stream)?;
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024];

    loop {
        // Read data from the client
        let bytes_read = stream.read(&mut buffer)?;

        // If bytes_read is 0, the client has disconnected
        if bytes_read == 0 {
            println!("Client disconnected");
            break;
        }

        let received = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Received: {}", received.trim());

        // Echo the data back to the client
        stream.write_all(&buffer[..bytes_read])?;
        stream.flush()?;
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    println!("=== Pattern 1: Synchronous TCP Echo Server ===\n");
    println!("This server handles one client at a time (blocking).\n");

    simple_echo_server()
}
