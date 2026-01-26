//! Pattern 2: Stream Processing
//! Then (async map)
//!
//! Run with: cargo run --example p2_then

use std::time::Duration;
use futures::stream::{self, StreamExt};

async fn async_transform_stream() {
    let stream = stream::iter(1..=5)
        .then(|x| async move {
            // Async operation per element
            tokio::time::sleep(Duration::from_millis(10)).await;
            x * x
        });

    let results: Vec<i32> = stream.collect().await;
    println!("Async transformed: {:?}", results);  // [1, 4, 9, 16, 25]
}

#[tokio::main]
async fn main() {
    async_transform_stream().await;
}
