//! Pattern 1: Future Composition
//! join! - wait for all futures
//!
//! Run with: cargo run --example p1_join

use std::time::Duration;

async fn fetch_user_name(user_id: u64) -> Result<String, String> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    if user_id == 0 {
        Err("Invalid user ID".to_string())
    } else {
        Ok(format!("User_{}", user_id))
    }
}

async fn concurrent_fetch() {
    // All three start immediately, complete in ~100ms total (not 300ms)
    let (result1, result2, result3) = tokio::join!(
        fetch_user_name(1),
        fetch_user_name(2),
        fetch_user_name(3),
    );

    println!("Results: {:?}, {:?}, {:?}", result1, result2, result3);
}

#[tokio::main]
async fn main() {
    concurrent_fetch().await;
}
