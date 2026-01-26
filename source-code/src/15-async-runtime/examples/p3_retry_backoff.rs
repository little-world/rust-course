//! Pattern 3: Async/Await Patterns
//! Retry with exponential backoff
//!
//! Run with: cargo run --example p3_retry_backoff

use std::time::Duration;

async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    max_retries: u32,
    initial_delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut delay = initial_delay;

    for attempt in 0..max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries - 1 => {
                println!("Attempt {} failed: {}. Retrying in {:?}...", attempt + 1, e, delay);
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }

    unreachable!()
}

#[tokio::main]
async fn main() {
    let result = retry_with_backoff(
        || async { Ok::<_, &str>("Success!") },
        3,
        Duration::from_millis(100),
    ).await;
    println!("Result: {:?}", result);
}
