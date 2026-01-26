//! Pattern 2: Stream Processing
//! Batch processing
//!
//! Run with: cargo run --example p2_batch

use std::time::Duration;

async fn batch_process<T: std::fmt::Debug>(items: Vec<T>, batch_size: usize) {
    for (i, batch) in items.chunks(batch_size).enumerate() {
        println!("Processing batch {}: {:?}", i, batch);
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::main]
async fn main() {
    batch_process((0..25).collect::<Vec<_>>(), 10).await;
}
