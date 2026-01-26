//! Pattern 1: r2d2 Connection Pool
//!
//! Demonstrates the classic synchronous connection pool using r2d2.
//! r2d2 works with any resource that implements its traits, not just databases.
//!
//! Note: Requires a running PostgreSQL database to execute.

use r2d2::{Pool, PooledConnection};
use r2d2_postgres::{postgres::NoTls, PostgresConnectionManager};
use std::error::Error;

type PostgresPool = Pool<PostgresConnectionManager<NoTls>>;
type PostgresPooledConnection = PooledConnection<PostgresConnectionManager<NoTls>>;

/// Initialize a connection pool
fn create_pool(database_url: &str) -> Result<PostgresPool, Box<dyn Error>> {
    let manager = PostgresConnectionManager::new(database_url.parse()?, NoTls);

    let pool = Pool::builder()
        .max_size(15) // Maximum 15 connections
        .min_idle(Some(5)) // Keep at least 5 idle connections
        .connection_timeout(std::time::Duration::from_secs(5))
        .build(manager)?;

    Ok(pool)
}

/// Example: Using the pool
fn fetch_user(pool: &PostgresPool, user_id: i32) -> Result<String, Box<dyn Error>> {
    // Get a connection from the pool
    // This blocks if all connections are in use, until one becomes available
    let mut conn = pool.get()?;

    // Use the connection
    let row = conn.query_one("SELECT username FROM users WHERE id = $1", &[&user_id])?;

    let username: String = row.get(0);

    // Connection automatically returns to pool when `conn` is dropped
    Ok(username)
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Pattern 1: r2d2 Connection Pool ===\n");

    // In a real application, get this from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:pass@localhost/mydb".to_string());

    println!("Creating connection pool...");
    println!("Database URL: {}", database_url);

    // Demonstrate pool creation (will fail without actual database)
    match create_pool(&database_url) {
        Ok(pool) => {
            println!("Pool created successfully!");
            println!("Pool state: {:?}", pool.state());

            // The pool can be cloned cheaply (Arc internally)
            // and shared across threads
            let pool_clone = pool.clone();

            let handle = std::thread::spawn(move || {
                println!("Thread: attempting to fetch user...");
                match fetch_user(&pool_clone, 42) {
                    Ok(username) => println!("Thread: fetched user: {}", username),
                    Err(e) => println!("Thread: error fetching user: {}", e),
                }
            });

            // Both threads can use the pool concurrently
            match fetch_user(&pool, 1) {
                Ok(username) => println!("Main: fetched user: {}", username),
                Err(e) => println!("Main: error fetching user: {}", e),
            }

            let _ = handle.join();
        }
        Err(e) => {
            println!("Failed to create pool: {}", e);
            println!("\nThis is expected if no database is running.");
            println!("To test, start PostgreSQL and set DATABASE_URL environment variable.");
        }
    }

    println!("\n--- Key Concepts ---");
    println!("1. Pool::builder() configures pool behavior");
    println!("2. pool.get() returns a PooledConnection smart pointer");
    println!("3. Connection auto-returns to pool when dropped");
    println!("4. Pool is Clone (Arc internally) for thread sharing");

    Ok(())
}
