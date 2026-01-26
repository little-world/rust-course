//! Pattern 2: Stream Processing
//! Take and skip
//!
//! Run with: cargo run --example p2_take_skip

use futures::stream::{self, StreamExt};

async fn limit_stream() {
    let results: Vec<i32> = stream::iter(1..=100)
        .skip(10)   // Skip first 10 (1-10)
        .take(5)    // Take next 5 (11-15)
        .collect()
        .await;

    println!("Limited: {:?}", results);  // [11, 12, 13, 14, 15]
}

#[tokio::main]
async fn main() {
    limit_stream().await;
}
