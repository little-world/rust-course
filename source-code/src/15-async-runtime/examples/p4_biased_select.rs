//! Pattern 4: Select and Timeout Patterns
//! Biased select (priority)
//!
//! Run with: cargo run --example p4_biased_select

use std::time::Duration;
use tokio::sync::mpsc;

async fn biased_select() {
    let (tx_high, mut rx_high) = mpsc::channel::<String>(10);
    let (tx_low, mut rx_low) = mpsc::channel::<String>(10);

    tokio::spawn(async move {
        tx_high.send("High priority".to_string()).await.unwrap();
        tx_low.send("Low priority".to_string()).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Biased: always checks branches in order
    tokio::select! {
        biased;

        Some(msg) = rx_high.recv() => {
            println!("High: {}", msg);
        }
        Some(msg) = rx_low.recv() => {
            println!("Low: {}", msg);
        }
    }
}

#[tokio::main]
async fn main() {
    biased_select().await;
}
