//! Pattern 1: Connection Pooling
//!
//! Demonstrates a simple connection pool for reusing TCP connections.
//! In production, use libraries like deadpool or bb8.

use tokio::net::TcpStream;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};

/// Simple connection pool for reusing TCP connections
/// In production, use libraries like deadpool or bb8
struct ConnectionPool {
    available: Arc<Mutex<VecDeque<TcpStream>>>,
    address: String,
    #[allow(dead_code)]
    max_size: usize,
}

impl ConnectionPool {
    fn new(address: String, max_size: usize) -> Self {
        ConnectionPool {
            available: Arc::new(Mutex::new(VecDeque::new())),
            address,
            max_size,
        }
    }

    async fn acquire(&self) -> tokio::io::Result<PooledConnection> {
        let mut pool = self.available.lock().await;

        // Try to reuse an existing connection
        if let Some(stream) = pool.pop_front() {
            println!("  Reusing existing connection from pool");
            return Ok(PooledConnection {
                stream: Some(stream),
                pool: self.available.clone(),
            });
        }

        // Otherwise create a new connection
        drop(pool); // Release lock before async operation
        println!("  Creating new connection to {}", self.address);
        let stream = TcpStream::connect(&self.address).await?;

        Ok(PooledConnection {
            stream: Some(stream),
            pool: self.available.clone(),
        })
    }
}

/// RAII wrapper that returns connection to pool on drop
struct PooledConnection {
    stream: Option<TcpStream>,
    pool: Arc<Mutex<VecDeque<TcpStream>>>,
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(stream) = self.stream.take() {
            let pool = self.pool.clone();
            // Return to pool asynchronously
            tokio::spawn(async move {
                println!("  Returning connection to pool");
                pool.lock().await.push_back(stream);
            });
        }
    }
}

impl Deref for PooledConnection {
    type Target = TcpStream;

    fn deref(&self) -> &Self::Target {
        self.stream.as_ref().unwrap()
    }
}

impl DerefMut for PooledConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.stream.as_mut().unwrap()
    }
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    println!("=== Pattern 1: Connection Pooling ===\n");
    println!("This demonstrates reusing connections from a pool.");
    println!("Start a server first: cargo run --bin p1_tcp_async\n");

    let pool = ConnectionPool::new("127.0.0.1:8080".into(), 10);

    println!("--- Acquiring connections ---\n");

    // Simulate multiple requests using the pool
    for i in 1..=5 {
        println!("Request {}:", i);

        match pool.acquire().await {
            Ok(conn) => {
                println!("  Got connection: {:?}", conn.peer_addr());
                // Connection returned to pool when `conn` is dropped
            }
            Err(e) => {
                eprintln!("  Failed to acquire connection: {}", e);
                return Ok(());
            }
        }

        // Small delay to see pool behavior
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    println!("\nConnection pooling demonstration complete!");
    println!("Notice how connections are reused from the pool.");

    Ok(())
}
