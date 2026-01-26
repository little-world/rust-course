//! Pattern 4: Select and Timeout Patterns
//! Graceful timeout (finish current work)
//!
//! Run with: cargo run --example p4_graceful_timeout

use std::time::Duration;
use tokio::time::timeout;

async fn graceful_shutdown_with_timeout(
    workers: Vec<tokio::task::JoinHandle<()>>,
    grace_period: Duration,
) {
    let shutdown = async {
        for worker in workers {
            worker.await.ok();
        }
    };

    match timeout(grace_period, shutdown).await {
        Ok(_) => println!("All workers stopped gracefully"),
        Err(_) => println!("Timeout - forcing shutdown"),
    }
}

#[tokio::main]
async fn main() {
    let workers = vec![
        tokio::spawn(async { tokio::time::sleep(Duration::from_millis(100)).await }),
        tokio::spawn(async { tokio::time::sleep(Duration::from_millis(200)).await }),
    ];
    graceful_shutdown_with_timeout(workers, Duration::from_secs(1)).await;
}
