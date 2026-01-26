//! Pattern 4: Select and Timeout Patterns
//! Timeout with retry
//!
//! Run with: cargo run --example p4_timeout_retry

use std::time::Duration;
use tokio::time::{sleep, timeout};

async fn timeout_with_retry() {
    for attempt in 1..=3u64 {
        let operation = async {
            sleep(Duration::from_millis(attempt * 400)).await;
            if attempt < 3 {
                Err("Failed")
            } else {
                Ok("Success")
            }
        };

        match timeout(Duration::from_secs(1), operation).await {
            Ok(Ok(result)) => {
                println!("Success: {}", result);
                break;
            }
            Ok(Err(e)) => {
                println!("Attempt {} failed: {}", attempt, e);
            }
            Err(_) => {
                println!("Attempt {} timed out", attempt);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    timeout_with_retry().await;
}
