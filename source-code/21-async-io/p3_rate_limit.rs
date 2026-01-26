// Pattern 3: Rate Limiting and Semaphores
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration, Instant};

// Simple rate limiter (token bucket algorithm)
struct RateLimiter {
    max_per_second: u32,
    last_reset: Instant,
    count: u32,
}

impl RateLimiter {
    fn new(max_per_second: u32) -> Self {
        RateLimiter {
            max_per_second,
            last_reset: Instant::now(),
            count: 0,
        }
    }

    async fn acquire(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_reset);

        // Reset counter every second
        if elapsed >= Duration::from_secs(1) {
            self.last_reset = now;
            self.count = 0;
        }

        // If we've hit the rate limit, wait until next second
        if self.count >= self.max_per_second {
            let wait_time = Duration::from_secs(1) - elapsed;
            println!("  Rate limit reached, waiting {:?}", wait_time);
            sleep(wait_time).await;
            self.last_reset = Instant::now();
            self.count = 0;
        }

        self.count += 1;
    }
}

// Rate limited requests demo
async fn rate_limited_requests() {
    println!("=== Rate Limiter Demo ===");
    println!("Limiting to 5 requests per second\n");

    let mut limiter = RateLimiter::new(5); // 5 requests per second
    let start = Instant::now();

    for i in 0..15 {
        // acquire() blocks if we've exceeded the rate limit
        limiter.acquire().await;
        println!("Request {} at {:?}", i, start.elapsed());
    }

    println!("\nTotal time for 15 requests: {:?}", start.elapsed());
    println!("Expected: ~3 seconds (5 req/sec * 3 batches)");
}

// Semaphore for Concurrency Control
async fn concurrent_with_limit() {
    println!("\n=== Semaphore Demo ===");
    println!("Running 10 tasks with max 3 concurrent\n");

    // Semaphore with 3 permits = max 3 concurrent tasks
    let semaphore = Arc::new(Semaphore::new(3));
    let start = Instant::now();

    let mut handles = vec![];

    for i in 0..10 {
        // acquire_owned() waits if no permits are available
        let permit = semaphore.clone().acquire_owned().await.unwrap();

        let handle = tokio::spawn(async move {
            println!("  Task {} started at {:?}", i, start.elapsed());
            sleep(Duration::from_millis(500)).await;
            println!("  Task {} completed at {:?}", i, start.elapsed());
            // Permit is dropped here, releasing it back to the semaphore
            drop(permit);
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    println!("\nTotal time: {:?}", start.elapsed());
    println!("Expected: ~2 seconds (10 tasks / 3 concurrent * 0.5s each)");
}

// Demonstrate try_acquire for non-blocking semaphore
async fn try_acquire_demo() {
    println!("\n=== Try Acquire Demo ===");
    println!("Demonstrating non-blocking semaphore acquisition\n");

    let semaphore = Arc::new(Semaphore::new(2));

    // Acquire both permits
    let _permit1 = semaphore.clone().acquire_owned().await.unwrap();
    let _permit2 = semaphore.clone().acquire_owned().await.unwrap();

    println!("Acquired 2 permits, none left");
    println!("Available permits: {}", semaphore.available_permits());

    // Try to acquire (should fail immediately)
    match semaphore.clone().try_acquire_owned() {
        Ok(_) => println!("Unexpected: got permit"),
        Err(_) => println!("try_acquire failed (as expected - no permits available)"),
    }

    // Release one permit
    drop(_permit1);
    println!("\nReleased one permit");
    println!("Available permits: {}", semaphore.available_permits());

    // Now try_acquire should succeed
    match semaphore.clone().try_acquire_owned() {
        Ok(p) => {
            println!("try_acquire succeeded!");
            drop(p);
        }
        Err(_) => println!("Unexpected: couldn't get permit"),
    }
}

// Combined rate limiting and concurrency control
async fn combined_demo() {
    println!("\n=== Combined Rate Limit + Concurrency Control ===");
    println!("Max 5 req/sec AND max 2 concurrent\n");

    let semaphore = Arc::new(Semaphore::new(2));
    let rate_limiter = Arc::new(tokio::sync::Mutex::new(RateLimiter::new(5)));
    let start = Instant::now();

    let mut handles = vec![];

    for i in 0..10 {
        let sem = semaphore.clone();
        let limiter = rate_limiter.clone();

        let handle = tokio::spawn(async move {
            // First, respect rate limit
            {
                let mut limiter = limiter.lock().await;
                limiter.acquire().await;
            }

            // Then, respect concurrency limit
            let _permit = sem.acquire_owned().await.unwrap();

            println!("  Request {} executing at {:?}", i, start.elapsed());
            sleep(Duration::from_millis(300)).await;
            println!("  Request {} done at {:?}", i, start.elapsed());
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    println!("\nTotal time: {:?}", start.elapsed());
}

#[tokio::main]
async fn main() {
    println!("=== Rate Limiting and Semaphores Demo ===\n");

    rate_limited_requests().await;
    concurrent_with_limit().await;
    try_acquire_demo().await;
    combined_demo().await;

    println!("\nRate limiting demo completed");
}
