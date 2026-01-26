//! Pattern 1: Future Composition
//! Dynamic number of futures with FuturesUnordered
//!
//! Run with: cargo run --example p1_futures_unordered

use std::time::Duration;
use futures::stream::{FuturesUnordered, StreamExt};

async fn fetch_user_name(user_id: u64) -> Result<String, String> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    if user_id == 0 {
        Err("Invalid user ID".to_string())
    } else {
        Ok(format!("User_{}", user_id))
    }
}

async fn fetch_all_users(user_ids: Vec<u64>) -> Vec<Result<String, String>> {
    // Works with any number of IDs - determined at runtime
    let futures: FuturesUnordered<_> = user_ids
        .into_iter()
        .map(|id| fetch_user_name(id))
        .collect();

    // Results arrive in completion order, not submission order
    futures.collect().await
}

#[tokio::main]
async fn main() {
    let users = fetch_all_users(vec![1, 2, 3, 4, 5]).await;
    println!("Fetched {} users", users.len());
}
