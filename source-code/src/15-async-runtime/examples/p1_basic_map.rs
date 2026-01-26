//! Pattern 1: Future Composition
//! Basic future composition with map
//!
//! Run with: cargo run --example p1_basic_map

use std::time::Duration;

async fn fetch_user_name(user_id: u64) -> Result<String, String> {
    // Simulate API call
    tokio::time::sleep(Duration::from_millis(100)).await;

    if user_id == 0 {
        Err("Invalid user ID".to_string())
    } else {
        Ok(format!("User_{}", user_id))
    }
}

async fn get_user_name_uppercase(user_id: u64) -> Result<String, String> {
    // Map over the result: await completes async, map transforms sync
    fetch_user_name(user_id)
        .await
        .map(|name| name.to_uppercase())
}

#[tokio::main]
async fn main() {
    match get_user_name_uppercase(42).await {
        Ok(name) => println!("User: {}", name),  // "USER_42"
        Err(e) => println!("Error: {}", e),
    }
}
