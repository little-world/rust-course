//! Pattern 4: Select and Timeout Patterns
//! Rate limiter with timeout
//!
//! Run with: cargo run --example p4_rate_limiter

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::timeout;

struct RateLimiter {
    semaphore: Arc<Semaphore>,
    #[allow(dead_code)]
    refill_amount: usize,
    #[allow(dead_code)]
    refill_interval: Duration,
}

impl RateLimiter {
    fn new(capacity: usize, refill_amount: usize, refill_interval: Duration) -> Self {
        let semaphore = Arc::new(Semaphore::new(capacity));

        // Refill task
        let sem = Arc::clone(&semaphore);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(refill_interval);
            loop {
                interval.tick().await;
                sem.add_permits(refill_amount);
            }
        });

        Self {
            semaphore,
            refill_amount,
            refill_interval,
        }
    }

    async fn acquire_with_timeout(&self, timeout_duration: Duration) -> Result<(), &'static str> {
        match timeout(timeout_duration, self.semaphore.acquire()).await {
            Ok(Ok(permit)) => {
                permit.forget(); // Consume permit
                Ok(())
            }
            Ok(Err(_)) => Err("Semaphore closed"),
            Err(_) => Err("Timeout acquiring rate limit"),
        }
    }
}

#[tokio::main]
async fn main() {
    let limiter = RateLimiter::new(5, 1, Duration::from_secs(1));
    match limiter.acquire_with_timeout(Duration::from_millis(100)).await {
        Ok(()) => println!("Acquired rate limit token"),
        Err(e) => println!("Failed: {}", e),
    }
}
