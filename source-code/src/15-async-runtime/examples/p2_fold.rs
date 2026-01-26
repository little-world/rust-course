//! Pattern 2: Stream Processing
//! Fold and reduce
//!
//! Run with: cargo run --example p2_fold

use futures::stream::{self, StreamExt};

async fn aggregate_stream() {
    let sum = stream::iter(1..=100)
        .fold(0, |acc, x| futures::future::ready(acc + x))  // Sum: 0+1+2+...+100
        .await;

    println!("Sum: {}", sum);  // 5050
}

#[tokio::main]
async fn main() {
    aggregate_stream().await;
}
