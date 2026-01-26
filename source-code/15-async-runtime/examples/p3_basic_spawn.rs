//! Pattern 3: Async/Await Patterns
//! Basic task spawning
//!
//! Run with: cargo run --example p3_basic_spawn

use std::time::Duration;

async fn spawn_basic_tasks() {
    let handle1 = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        println!("Task 1 complete");
        42
    });

    let handle2 = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(200)).await;
        println!("Task 2 complete");
        100
    });

    let (result1, result2) = tokio::join!(handle1, handle2);
    println!("Results: {:?}, {:?}", result1, result2);
}

#[tokio::main]
async fn main() {
    spawn_basic_tasks().await;
}
