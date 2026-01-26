//! Pattern 1: Line-based Protocol Server
//!
//! Demonstrates a line-based protocol useful for SMTP, FTP, or custom text protocols.
//! Uses BufReader for efficient buffered reading.

use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// A line-based protocol server
/// Useful for protocols like SMTP, FTP, or custom text protocols
#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    println!("=== Pattern 1: Line-based Protocol Server ===\n");

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Line-based server listening on port 8080");
    println!("Commands: HELLO, TIME, QUIT");
    println!("Connect with: nc localhost 8080\n");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Connection from {}", addr);

        tokio::spawn(async move {
            // BufReader provides efficient buffered reading
            let (reader, mut writer) = socket.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            // Send welcome message
            let _ = writer.write_all(b"Welcome! Commands: HELLO, TIME, QUIT\n").await;

            loop {
                line.clear();

                // Read until we get a newline
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        // EOF - connection closed
                        println!("Client {} disconnected", addr);
                        break;
                    }
                    Ok(_) => {
                        println!("[{}] Command: {}", addr, line.trim());

                        // Process the line
                        let response = process_command(&line);

                        // Send response
                        if let Err(e) = writer.write_all(response.as_bytes()).await {
                            eprintln!("Failed to write to {}: {}", addr, e);
                            break;
                        }

                        // Check for quit
                        if line.trim().to_uppercase() == "QUIT" {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading from {}: {}", addr, e);
                        break;
                    }
                }
            }
        });
    }
}

fn process_command(line: &str) -> String {
    let line = line.trim();

    match line.to_uppercase().as_str() {
        "HELLO" => "WORLD\n".to_string(),
        "TIME" => format!("Current time: {:?}\n", std::time::SystemTime::now()),
        "QUIT" => "BYE\n".to_string(),
        _ => format!("ECHO: {}\n", line),
    }
}
