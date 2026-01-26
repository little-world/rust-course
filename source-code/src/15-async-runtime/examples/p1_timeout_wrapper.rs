//! Pattern 1: Future Composition
//! Timeout wrapper
//!
//! Run with: cargo run --example p1_timeout_wrapper

use std::time::Duration;

async fn with_timeout<F, T>(
    future: F,
    duration: Duration,
) -> Result<T, tokio::time::error::Elapsed>
where
    F: std::future::Future<Output = T>,
{
    tokio::time::timeout(duration, future).await
}

#[tokio::main]
async fn main() {
    match with_timeout(
        async { tokio::time::sleep(Duration::from_millis(50)).await; "done" },
        Duration::from_millis(100),
    ).await {
        Ok(result) => println!("Completed: {}", result),
        Err(_) => println!("Timed out"),
    }
}
