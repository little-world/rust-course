//! Pattern 4: Select and Timeout Patterns
//! Health check with timeout
//!
//! Run with: cargo run --example p4_health_check

use std::time::Duration;
use tokio::time::timeout;

async fn health_check(url: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let check = async {
        let response = reqwest::get(url).await?;
        Ok::<bool, Box<dyn std::error::Error + Send + Sync>>(response.status().is_success())
    };

    timeout(Duration::from_secs(5), check)
        .await
        .map_err(|_| -> Box<dyn std::error::Error + Send + Sync> { "Health check timed out".into() })?
}

#[tokio::main]
async fn main() {
    match health_check("https://example.com").await {
        Ok(healthy) => println!("Service healthy: {}", healthy),
        Err(e) => println!("Health check failed: {}", e),
    }
}
