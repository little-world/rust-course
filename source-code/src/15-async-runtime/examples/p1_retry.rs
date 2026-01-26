//! Pattern 1: Future Composition
//! HTTP client with retries
//!
//! Run with: cargo run --example p1_retry

use std::time::Duration;

async fn fetch_with_retry<F, Fut, T, E>(
    mut f: F,
    max_retries: usize,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempts = 0;

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;
                if attempts >= max_retries {
                    return Err(e);
                }
                println!("Attempt {} failed: {}. Retrying...", attempts, e);
                let delay = Duration::from_secs(2u64.pow(attempts as u32));
                tokio::time::sleep(delay).await;
            }
        }
    }
}

// Usage: wrap any async operation with retry logic
async fn fetch_data_with_retry(url: String) -> Result<String, reqwest::Error> {
    fetch_with_retry(
        || async { reqwest::get(&url).await?.text().await },
        3,  // max 3 attempts
    ).await
}

#[tokio::main]
async fn main() {
    match fetch_data_with_retry("https://api.example.com/data".to_string()).await {
        Ok(data) => println!("Fetched: {}", data),
        Err(e) => println!("Failed after retries: {}", e),
    }
}
