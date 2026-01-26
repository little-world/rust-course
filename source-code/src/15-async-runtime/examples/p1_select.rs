//! Pattern 1: Future Composition
//! select! - race futures, take first to complete
//!
//! Run with: cargo run --example p1_select

use std::time::Duration;
use tokio::time::sleep;

async fn fetch_user_name(user_id: u64) -> Result<String, String> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    if user_id == 0 {
        Err("Invalid user ID".to_string())
    } else {
        Ok(format!("User_{}", user_id))
    }
}

async fn race_requests() -> String {
    tokio::select! {
        result = fetch_user_name(1) => {
            format!("Server 1 responded first: {:?}", result)
        }
        result = fetch_user_name(2) => {
            format!("Server 2 responded first: {:?}", result)
        }
        _ = sleep(Duration::from_secs(1)) => {
            "Both servers too slow - timeout".to_string()
        }
    }
}

#[tokio::main]
async fn main() {
    let winner = race_requests().await;
    println!("{}", winner);
}
