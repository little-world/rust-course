//! Pattern 2: Stream Processing
//! Stream merging
//!
//! Run with: cargo run --example p2_merge

use futures::stream;
use tokio_stream::StreamExt;

async fn merge_streams() {
    let stream1 = stream::iter(vec![1, 2, 3]);
    let stream2 = stream::iter(vec![4, 5, 6]);
    let merged = StreamExt::merge(stream1, stream2);
    let results: Vec<i32> = merged.collect().await;
    println!("Merged: {:?}", results);
}

#[tokio::main]
async fn main() {
    merge_streams().await;
}
