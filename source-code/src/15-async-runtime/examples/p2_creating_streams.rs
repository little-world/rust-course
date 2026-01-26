//! Pattern 2: Stream Processing
//! Creating streams
//!
//! Run with: cargo run --example p2_creating_streams

use std::time::Duration;
use futures::stream;

async fn create_streams() {
    // From iterator - instant conversion of known data
    let _s = stream::iter(vec![1, 2, 3, 4, 5]);

    // From channel - producer task sends values over time
    let (tx, rx) = tokio::sync::mpsc::channel(10);
    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(i).await.unwrap();
        }
    });
    let _s = tokio_stream::wrappers::ReceiverStream::new(rx);

    // Interval stream - time-based events
    let _s = stream::StreamExt::take(
        tokio_stream::wrappers::IntervalStream::new(
            tokio::time::interval(Duration::from_millis(100))
        ),
        5,  // Stop after 5 ticks
    );

    println!("Streams created successfully");
}

#[tokio::main]
async fn main() {
    create_streams().await;
}
