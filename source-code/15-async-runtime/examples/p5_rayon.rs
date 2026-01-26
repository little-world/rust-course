//! Pattern 5: Runtime Comparison
//! CPU-bound work with rayon
//!
//! Run with: cargo run --example p5_rayon

use rayon::prelude::*;

async fn cpu_bound_with_rayon() {
    let numbers: Vec<u64> = (0..1_000_000).collect();

    let sum = tokio::task::spawn_blocking(move || {
        numbers.par_iter().sum::<u64>()
    }).await.unwrap();

    println!("Sum: {}", sum);
}

#[tokio::main]
async fn main() {
    cpu_bound_with_rayon().await;
}
