//! Pattern 1: Pool Configuration Tuning
//!
//! Demonstrates how to tune connection pool parameters for different workloads.
//! Each parameter controls specific behavior that affects performance.

use r2d2::Pool;
use r2d2_postgres::{postgres::NoTls, PostgresConnectionManager};
use std::time::Duration;

type PostgresPool = Pool<PostgresConnectionManager<NoTls>>;

fn configure_pool_detailed(manager: PostgresConnectionManager<NoTls>) -> Result<PostgresPool, r2d2::Error> {
    Pool::builder()
        // Maximum number of connections to create
        // Higher = more concurrent requests, but more database load
        .max_size(20)
        // Minimum idle connections to maintain
        // Higher = faster response for bursts, but more idle resources
        .min_idle(Some(5))
        // How long to wait for a connection before timing out
        // Too low = errors during load spikes
        // Too high = slow responses when pool exhausted
        .connection_timeout(Duration::from_secs(10))
        // Test connections before use to ensure they're alive
        // Adds overhead but prevents using dead connections
        .test_on_check_out(true)
        // How long a connection can be idle before being closed
        // Prevents accumulating stale connections
        .idle_timeout(Some(Duration::from_secs(300)))
        // Maximum lifetime of a connection before forced recreation
        // Ensures connections don't grow stale over time
        .max_lifetime(Some(Duration::from_secs(1800)))
        .build(manager)
}

fn main() {
    println!("=== Pattern 1: Pool Configuration Tuning ===\n");

    println!("--- Configuration Parameters ---\n");

    println!("max_size: Maximum connections to create");
    println!("  - Higher = more concurrent requests");
    println!("  - Too high = overwhelms database (PostgreSQL default: 100 max_connections)\n");

    println!("min_idle: Minimum idle connections to maintain");
    println!("  - Higher = faster response for traffic bursts");
    println!("  - Trade-off: memory for latency\n");

    println!("connection_timeout: How long to wait for a connection");
    println!("  - Too low = errors during load spikes");
    println!("  - Too high = slow responses when exhausted\n");

    println!("test_on_check_out: Validate connections before use");
    println!("  - Adds ~1ms overhead");
    println!("  - Prevents 'connection reset' errors from stale connections\n");

    println!("idle_timeout: Close connections idle too long");
    println!("  - Database may have killed them anyway");
    println!("  - Frees resources during low traffic\n");

    println!("max_lifetime: Force periodic connection recycling");
    println!("  - Prevents memory leaks or state accumulation");
    println!("  - Typically 30 minutes to 1 hour\n");

    println!("--- Sizing Guidelines ---\n");

    println!("Small app (< 100 concurrent users): max_size = 5-10");
    println!("Medium app (100-1000 users):        max_size = 10-20");
    println!("Large app (1000+ users):            max_size = 20-50");
    println!("\nNote: Going higher than 50 often indicates other bottlenecks.\n");

    // Demonstrate creating a tuned pool
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:pass@localhost/mydb".to_string());

    println!("--- Creating Tuned Pool ---\n");

    match database_url.parse() {
        Ok(config) => {
            let manager = PostgresConnectionManager::new(config, NoTls);
            match configure_pool_detailed(manager) {
                Ok(pool) => {
                    let state = pool.state();
                    println!("Pool created successfully!");
                    println!("  Connections: {}", state.connections);
                    println!("  Idle: {}", state.idle_connections);
                }
                Err(e) => {
                    println!("Pool creation failed: {}", e);
                    println!("(Expected without running database)");
                }
            }
        }
        Err(e) => {
            println!("Invalid database URL: {}", e);
        }
    }

    println!("\nPool tuning example completed!");
}
