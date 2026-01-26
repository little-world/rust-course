//! Pattern 4: Select and Timeout Patterns
//! Deadline tracking
//!
//! Run with: cargo run --example p4_deadline

use std::time::Duration;
use tokio::time::{sleep, timeout, Instant};

async fn with_deadline<F, T>(
    future: F,
    deadline: Instant,
) -> Result<T, &'static str>
where
    F: std::future::Future<Output = T>,
{
    let duration = deadline.saturating_duration_since(Instant::now());

    match timeout(duration, future).await {
        Ok(result) => Ok(result),
        Err(_) => Err("Deadline exceeded"),
    }
}

async fn deadline_example() {
    let deadline = Instant::now() + Duration::from_secs(1);

    let result = with_deadline(
        async {
            sleep(Duration::from_millis(500)).await;
            42
        },
        deadline,
    ).await;

    println!("Result: {:?}", result);
}

#[tokio::main]
async fn main() {
    deadline_example().await;
}
