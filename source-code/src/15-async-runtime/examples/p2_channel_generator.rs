//! Pattern 2: Stream Processing
//! Async generator pattern using channels
//!
//! Run with: cargo run --example p2_channel_generator

use std::time::Duration;
use futures::Stream;
use tokio_stream::StreamExt;

async fn number_generator(max: u32) -> impl Stream<Item = u32> {
    let (tx, rx) = tokio::sync::mpsc::channel(10);  // Buffer 10 items

    tokio::spawn(async move {
        for i in 0..max {
            tokio::time::sleep(Duration::from_millis(10)).await;
            if tx.send(i).await.is_err() {
                break;  // Consumer dropped, stop producing
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
}

#[tokio::main]
async fn main() {
    let mut stream = number_generator(5).await;
    while let Some(n) = stream.next().await {
        println!("Generated: {}", n);  // 0, 1, 2, 3, 4 (with delays)
    }
}
