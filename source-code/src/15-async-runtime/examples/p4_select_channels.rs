//! Pattern 4: Select and Timeout Patterns
//! Select two channels
//!
//! Run with: cargo run --example p4_select_channels

use std::time::Duration;
use tokio::sync::mpsc;

async fn select_two_channels() {
    let (tx1, mut rx1) = mpsc::channel::<i32>(10);
    let (tx2, mut rx2) = mpsc::channel::<String>(10);

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        tx1.send(42).await.unwrap();
    });

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        tx2.send("Hello".to_string()).await.unwrap();
    });

    tokio::select! {
        Some(num) = rx1.recv() => {
            println!("Got number: {}", num);
        }
        Some(msg) = rx2.recv() => {
            println!("Got message: {}", msg);
        }
    }
}

#[tokio::main]
async fn main() {
    select_two_channels().await;
}
