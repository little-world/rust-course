//! Pattern 1: deadpool Async Connection Pooling
//!
//! Demonstrates async-first connection pooling with deadpool.
//! Unlike r2d2, deadpool's get() yields to other tasks while waiting.

use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod};
use std::error::Error;
use tokio_postgres::NoTls;

/// Create an async connection pool
fn create_async_pool() -> Result<Pool, Box<dyn Error + Send + Sync>> {
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.user = Some("user".to_string());
    cfg.password = Some("password".to_string());
    cfg.dbname = Some("mydb".to_string());

    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    cfg.pool = Some(deadpool::managed::PoolConfig {
        max_size: 20,
        timeouts: deadpool::managed::Timeouts {
            wait: Some(std::time::Duration::from_secs(5)),
            create: Some(std::time::Duration::from_secs(5)),
            recycle: Some(std::time::Duration::from_secs(5)),
        },
        queue_mode: deadpool::managed::QueueMode::Fifo,
    });

    Ok(cfg.create_pool(None, NoTls)?)
}

/// Fetch user asynchronously
async fn fetch_user_async(pool: &Pool, user_id: i32) -> Result<String, Box<dyn Error + Send + Sync>> {
    // Get connection asynchronously
    // This awaits instead of blocking
    let client = pool.get().await?;

    // Execute query
    let row = client
        .query_one("SELECT username FROM users WHERE id = $1", &[&user_id])
        .await?;

    let username: String = row.get(0);

    // Connection returns to pool on drop
    Ok(username)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("=== Pattern 1: deadpool Async Connection Pooling ===\n");

    println!("--- Why deadpool for async? ---\n");
    println!("r2d2's blocking get() wastes a thread while waiting.");
    println!("deadpool's get().await yields to other tasks.\n");

    println!("--- Configuration ---\n");
    println!("RecyclingMethod::Fast - Skip connection validation (faster)");
    println!("RecyclingMethod::Verified - Validate recycled connections (safer)\n");

    println!("Timeouts:");
    println!("  wait: How long to wait for available connection");
    println!("  create: How long to establish new connection");
    println!("  recycle: How long to validate recycled connection\n");

    // Try to create pool
    match create_async_pool() {
        Ok(pool) => {
            println!("Pool created successfully!");
            let status = pool.status();
            println!("  Max size: {}", status.max_size);
            println!("  Size: {}", status.size);
            println!("  Available: {}", status.available);

            // Demonstrate concurrent queries
            println!("\n--- Simulating Concurrent Queries ---\n");

            let tasks: Vec<_> = (1..=5)
                .map(|id| {
                    let pool = pool.clone();
                    tokio::spawn(async move {
                        match fetch_user_async(&pool, id).await {
                            Ok(username) => println!("Task {}: fetched {}", id, username),
                            Err(e) => println!("Task {}: error - {}", id, e),
                        }
                    })
                })
                .collect();

            // Wait for all queries
            for task in tasks {
                let _ = task.await;
            }

            println!("\nAll concurrent queries completed!");
        }
        Err(e) => {
            println!("Failed to create pool: {}", e);
            println!("\nThis is expected if no database is running.");
            println!("The pattern demonstrates async-native pooling:");
            println!("  - pool.get().await yields instead of blocking");
            println!("  - Clone pool cheaply (Arc) for spawned tasks");
            println!("  - Serve thousands of requests with 20-50 connections");
        }
    }

    println!("\ndeadpool async example completed!");

    Ok(())
}
