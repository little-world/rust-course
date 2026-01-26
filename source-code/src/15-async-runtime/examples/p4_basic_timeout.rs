//! Pattern 4: Select and Timeout Patterns
//! Basic timeout
//!
//! Run with: cargo run --example p4_basic_timeout

use std::time::Duration;
use tokio::time::{sleep, timeout};

async fn basic_timeout() {
    let operation = async {
        sleep(Duration::from_secs(2)).await;
        "Completed"
    };

    match timeout(Duration::from_secs(1), operation).await {
        Ok(result) => println!("Result: {}", result),
        Err(_) => println!("Operation timed out"),
    }
}

#[tokio::main]
async fn main() {
    basic_timeout().await;
}
