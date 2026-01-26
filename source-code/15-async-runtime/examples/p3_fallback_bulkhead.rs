//! Pattern 3: Async/Await Patterns
//! Fallback pattern and Bulkhead pattern
//!
//! Run with: cargo run --example p3_fallback_bulkhead

use std::sync::Arc;

async fn fetch_with_fallback<F, Fut, T>(
    primary: F,
    fallback_value: T,
) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error>>>,
{
    match primary().await {
        Ok(value) => value,
        Err(e) => {
            println!("Primary failed: {}. Using fallback.", e);
            fallback_value
        }
    }
}

struct Bulkhead {
    semaphore: Arc<tokio::sync::Semaphore>,
}

impl Bulkhead {
    fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(tokio::sync::Semaphore::new(max_concurrent)),
        }
    }

    async fn execute<F, Fut, T>(&self, operation: F) -> Result<T, String>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        match self.semaphore.try_acquire() {
            Ok(permit) => {
                let result = operation().await;
                drop(permit);
                Ok(result)
            }
            Err(_) => Err("Bulkhead full - request rejected".to_string()),
        }
    }
}

#[tokio::main]
async fn main() {
    // Fallback pattern example
    let result = fetch_with_fallback(
        || async { Ok::<_, Box<dyn std::error::Error>>("Primary data".to_string()) },
        "Fallback data".to_string(),
    ).await;
    println!("Result: {}", result);

    // Bulkhead pattern example
    let bulkhead = Bulkhead::new(2);
    let result = bulkhead.execute(|| async { 42 }).await;
    println!("Bulkhead result: {:?}", result);
}
