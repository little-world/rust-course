//! Pattern 2: Stream Processing
//! Rate Limiting
//!
//! Run with: cargo run --example p2_rate_limiting

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use futures::stream::{self, StreamExt};

async fn rate_limited_requests(urls: Vec<String>) {
    let semaphore = Arc::new(Semaphore::new(5)); // Max 5 concurrent

    let stream = stream::iter(urls)
        .map(|url| {
            let permit = Arc::clone(&semaphore);
            async move {
                let _permit = permit.acquire().await.unwrap();  // Wait for permit
                println!("Fetching: {}", url);
                tokio::time::sleep(Duration::from_millis(100)).await;
                format!("Response from {}", url)
            }  // Permit released when dropped
        })
        .buffer_unordered(10);  // Allow 10 in-flight, but semaphore limits to 5

    let results: Vec<String> = stream.collect().await;
    println!("Fetched {} URLs", results.len());
}

#[tokio::main]
async fn main() {
    let urls: Vec<_> = (0..20).map(|i| format!("https://api.example.com/{}", i)).collect();
    rate_limited_requests(urls).await;
}
