//! Pattern 5: Runtime Comparison
//! Multi-threaded runtime (default)
//!
//! Run with: cargo run --example p5_multi_threaded

use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("Running on multi-threaded runtime");

    let handles: Vec<_> = (0..10)
        .map(|i| {
            tokio::spawn(async move {
                println!("Task {} on thread {:?}", i, std::thread::current().id());
                tokio::time::sleep(Duration::from_millis(10)).await;
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }
}
