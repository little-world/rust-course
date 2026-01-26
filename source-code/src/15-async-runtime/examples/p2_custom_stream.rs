//! Pattern 2: Stream Processing
//! Stream from Async Generators (Manual implementation)
//!
//! Run with: cargo run --example p2_custom_stream

use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Stream;
use tokio_stream::StreamExt;

struct CounterStream {
    count: u32,
    max: u32,
}

impl CounterStream {
    fn new(max: u32) -> Self {
        Self { count: 0, max }
    }
}

impl Stream for CounterStream {
    type Item = u32;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.count < self.max {
            let current = self.count;
            self.count += 1;
            Poll::Ready(Some(current))  // Yield next value
        } else {
            Poll::Ready(None)  // Stream exhausted
        }
    }
}

#[tokio::main]
async fn main() {
    let mut stream = CounterStream::new(5);
    while let Some(n) = stream.next().await {
        println!("Count: {}", n);  // 0, 1, 2, 3, 4
    }
}
