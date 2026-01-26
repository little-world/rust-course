//! Pattern 5: Runtime Comparison
//! Blocking operations
//!
//! Run with: cargo run --example p5_blocking

use std::time::Duration;

async fn handle_blocking_operations() {
    // Bad: blocks the async runtime
    // std::thread::sleep(Duration::from_secs(1));

    // Good: run blocking code on dedicated thread pool
    let result = tokio::task::spawn_blocking(|| {
        std::thread::sleep(Duration::from_secs(1));
        "Blocking operation complete"
    }).await.unwrap();

    println!("{}", result);
}

#[tokio::main]
async fn main() {
    handle_blocking_operations().await;
}
