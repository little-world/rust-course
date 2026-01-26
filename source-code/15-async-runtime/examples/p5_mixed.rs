//! Pattern 5: Runtime Comparison
//! Mixed workload (I/O and CPU)
//!
//! Run with: cargo run --example p5_mixed

use std::time::Duration;

async fn mixed_workload() {
    let io_task = tokio::spawn(async {
        for i in 0..5 {
            println!("I/O task {}", i);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    let cpu_task = tokio::task::spawn_blocking(|| {
        for i in 0..5 {
            println!("CPU task {}", i);
            std::thread::sleep(Duration::from_millis(100));

            // Simulate CPU-intensive work
            let _ = (0..1_000_000).sum::<u64>();
        }
    });

    let _ = tokio::join!(io_task, cpu_task);
}

#[tokio::main]
async fn main() {
    mixed_workload().await;
}
