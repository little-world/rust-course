//! Pattern 3: Async/Await Patterns
//! Structured concurrency with JoinSet
//!
//! Run with: cargo run --example p3_joinset

use std::time::Duration;
use tokio::task::JoinSet;

async fn structured_concurrency() {
    let mut set = JoinSet::new();

    for i in 0..5 {
        set.spawn(async move {
            tokio::time::sleep(Duration::from_millis(i * 50)).await;
            println!("Task {} done", i);
            i
        });
    }

    // Wait for all tasks
    while let Some(result) = set.join_next().await {
        match result {
            Ok(value) => println!("Got: {}", value),
            Err(e) => println!("Task failed: {}", e),
        }
    }
}

#[tokio::main]
async fn main() {
    structured_concurrency().await;
}
