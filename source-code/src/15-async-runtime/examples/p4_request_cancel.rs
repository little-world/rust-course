//! Pattern 4: Select and Timeout Patterns
//! Request with cancellation
//!
//! Run with: cargo run --example p4_request_cancel

use std::time::Duration;
use tokio::sync::mpsc;

async fn request_with_cancel() {
    let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);

    let request = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).await;
        "Request complete"
    });

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        cancel_tx.send(()).await.unwrap();
    });

    tokio::select! {
        result = request => {
            println!("Request finished: {:?}", result);
        }
        _ = cancel_rx.recv() => {
            println!("Request cancelled");
        }
    }
}

#[tokio::main]
async fn main() {
    request_with_cancel().await;
}
