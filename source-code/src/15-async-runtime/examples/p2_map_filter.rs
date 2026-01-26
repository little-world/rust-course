//! Pattern 2: Stream Processing
//! Map and filter
//!
//! Run with: cargo run --example p2_map_filter

use futures::stream::{self, StreamExt};

async fn transform_stream() {
    let stream = stream::iter(1..=10)
        .filter(|x| futures::future::ready(x % 2 == 0))  // Keep evens: 2, 4, 6, 8, 10
        .map(|x| x * 2);         // Double: 4, 8, 12, 16, 20

    let results: Vec<i32> = stream.collect().await;
    println!("Transformed: {:?}", results);  // [4, 8, 12, 16, 20]
}

#[tokio::main]
async fn main() {
    transform_stream().await;
}
