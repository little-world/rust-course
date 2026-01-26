//! Pattern 1: Future Composition
//! try_join! - wait for all, fail fast on error
//!
//! Run with: cargo run --example p1_try_join

use std::time::Duration;

async fn fetch_user_name(user_id: u64) -> Result<String, String> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    if user_id == 0 {
        Err("Invalid user ID".to_string())
    } else {
        Ok(format!("User_{}", user_id))
    }
}

async fn concurrent_fetch_fail_fast() -> Result<(String, String, String), String> {
    // If user 2 fails, user 3 is cancelled immediately
    tokio::try_join!(
        fetch_user_name(1),
        fetch_user_name(2),
        fetch_user_name(3),
    )
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let (u1, u2, u3) = concurrent_fetch_fail_fast().await?;
    println!("Users: {}, {}, {}", u1, u2, u3);
    Ok(())
}
