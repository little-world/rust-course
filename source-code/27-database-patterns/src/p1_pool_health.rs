//! Pattern 1: Pool Health Checks and Monitoring
//!
//! Demonstrates monitoring pool health in production applications.
//! Essential for detecting issues before they impact users.

use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::NoTls;

fn create_pool() -> Result<Pool, Box<dyn std::error::Error + Send + Sync>> {
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

/// Monitor pool health and return alerts
fn monitor_pool_health(pool: &Pool) -> Vec<String> {
    let status = pool.status();
    let mut alerts = Vec::new();

    println!("Pool status:");
    println!("  Available connections: {}", status.available);
    println!("  Size: {}", status.size);
    println!("  Max size: {}", status.max_size);

    // Alert if pool is exhausted
    if status.available == 0 && status.size >= status.max_size {
        alerts.push("CRITICAL: Connection pool exhausted!".to_string());
    }

    // Alert if pool is mostly idle (might be oversized)
    if status.available > status.max_size * 3 / 4 {
        alerts.push("INFO: Pool mostly idle, consider reducing size".to_string());
    }

    // Alert if pool is under pressure
    if status.available < status.max_size / 4 && status.size > 0 {
        alerts.push("WARNING: Pool under pressure, connections running low".to_string());
    }

    alerts
}

/// Simulate pool status for demonstration
fn demonstrate_health_scenarios() {
    println!("=== Health Check Scenarios ===\n");

    // Scenario 1: Healthy pool
    println!("Scenario 1: Healthy Pool");
    println!("  Available: 15, Size: 20, Max: 20");
    println!("  Status: OK - Pool has spare capacity\n");

    // Scenario 2: Pool exhausted
    println!("Scenario 2: Pool Exhausted");
    println!("  Available: 0, Size: 20, Max: 20");
    println!("  Status: CRITICAL - New requests will block/timeout");
    println!("  Action: Increase max_size or investigate slow queries\n");

    // Scenario 3: Pool oversized
    println!("Scenario 3: Pool Oversized");
    println!("  Available: 18, Size: 20, Max: 20");
    println!("  Status: INFO - Most connections idle");
    println!("  Action: Consider reducing max_size to free database resources\n");

    // Scenario 4: Growing pressure
    println!("Scenario 4: Under Pressure");
    println!("  Available: 3, Size: 20, Max: 20");
    println!("  Status: WARNING - Pool running low");
    println!("  Action: Monitor closely, may need to scale\n");
}

/// Graceful shutdown pattern
async fn shutdown_pool(pool: Pool) {
    println!("Shutting down pool...");

    // Close the pool - stops accepting new checkouts
    pool.close();

    // Give active connections time to finish
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    println!("Pool shutdown complete");
}

#[tokio::main]
async fn main() {
    println!("=== Pattern 1: Pool Health Checks and Monitoring ===\n");

    // Show health check scenarios
    demonstrate_health_scenarios();

    println!("--- Real Pool Health Check ---\n");

    match create_pool() {
        Ok(pool) => {
            let alerts = monitor_pool_health(&pool);

            if alerts.is_empty() {
                println!("\nNo alerts - pool is healthy");
            } else {
                println!("\nAlerts:");
                for alert in alerts {
                    println!("  {}", alert);
                }
            }

            // Demonstrate graceful shutdown
            println!("\n--- Graceful Shutdown ---");
            shutdown_pool(pool).await;
        }
        Err(e) => {
            println!("Could not create pool: {}", e);
            println!("(Expected without running database)\n");
            println!("In production, monitor these metrics:");
            println!("  - available: Ready connections");
            println!("  - size: Total allocated connections");
            println!("  - max_size: Pool capacity limit");
            println!("\nExport to Prometheus/Grafana for dashboards and alerting.");
        }
    }

    println!("\nHealth monitoring example completed!");
}
