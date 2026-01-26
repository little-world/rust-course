//! Pattern 1: Future Composition
//! Chaining async operations
//!
//! Run with: cargo run --example p1_chaining

use std::time::Duration;

async fn fetch_user_name(user_id: u64) -> Result<String, String> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    if user_id == 0 {
        Err("Invalid user ID".to_string())
    } else {
        Ok(format!("User_{}", user_id))
    }
}

async fn fetch_user_posts(user_id: u64) -> Result<Vec<String>, String> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    Ok(vec![
        format!("Post 1 by user {}", user_id),
        format!("Post 2 by user {}", user_id),
    ])
}

async fn get_user_with_posts(user_id: u64) -> Result<(String, Vec<String>), String> {
    let name = fetch_user_name(user_id).await?;  // Early return if fails
    let posts = fetch_user_posts(user_id).await?;
    Ok((name, posts))
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let (name, posts) = get_user_with_posts(1).await?;
    println!("{} has {} posts", name, posts.len());
    Ok(())
}
