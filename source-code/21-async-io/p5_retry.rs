// Pattern 5: Retry with Timeout and Exponential Backoff
use std::io;
use tokio::time::{sleep, timeout, Duration, Instant};

// Generic retry logic with timeout and exponential backoff
async fn retry_with_timeout<F, Fut, T>(
    mut operation: F,
    max_retries: usize,
    timeout_duration: Duration,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = io::Result<T>>,
{
    for attempt in 0..max_retries {
        println!("  Attempt {} of {}", attempt + 1, max_retries);

        match timeout(timeout_duration, operation()).await {
            Ok(Ok(result)) => {
                println!("  Success on attempt {}", attempt + 1);
                return Ok(result);
            }
            Ok(Err(e)) => {
                println!("  Attempt {} failed: {}", attempt + 1, e);
            }
            Err(_) => {
                println!("  Attempt {} timed out", attempt + 1);
            }
        }

        if attempt < max_retries - 1 {
            // Exponential backoff: wait longer after each failure
            let backoff = Duration::from_millis(100 * 2_u64.pow(attempt as u32));
            println!("  Waiting {:?} before retry...", backoff);
            sleep(backoff).await;
        }
    }

    Err("All retry attempts failed".into())
}

// Simulated unreliable operation
async fn unreliable_operation(fail_probability: f64) -> io::Result<String> {
    // Simulate some work
    sleep(Duration::from_millis(50)).await;

    if rand::random::<f64>() < fail_probability {
        Err(io::Error::new(io::ErrorKind::Other, "Random failure"))
    } else {
        Ok("Operation succeeded!".to_string())
    }
}

// Simulated slow operation (for timeout testing)
async fn slow_operation(delay_ms: u64) -> io::Result<String> {
    sleep(Duration::from_millis(delay_ms)).await;
    Ok("Slow operation completed".to_string())
}

// Demonstrate retry with high failure rate
async fn retry_demo_failures() {
    println!("=== Retry Demo (High Failure Rate) ===\n");
    println!("Operation has 70% chance of failure");

    let result = retry_with_timeout(
        || unreliable_operation(0.7),  // 70% failure rate
        5,  // Max 5 retries
        Duration::from_secs(2),  // 2-second timeout per attempt
    ).await;

    match result {
        Ok(s) => println!("\nFinal result: {}", s),
        Err(e) => println!("\nFailed after all retries: {}", e),
    }
}

// Demonstrate retry with timeouts
async fn retry_demo_timeouts() {
    println!("\n=== Retry Demo (Timeouts) ===\n");
    println!("Operation takes random 50-150ms, timeout is 100ms");

    let result = retry_with_timeout(
        || async {
            // Random delay between 50-150ms
            let delay = 50 + (rand::random::<u64>() % 100);
            slow_operation(delay).await
        },
        5,
        Duration::from_millis(100),  // 100ms timeout
    ).await;

    match result {
        Ok(s) => println!("\nFinal result: {}", s),
        Err(e) => println!("\nFailed after all retries: {}", e),
    }
}

// Retry with jitter to prevent thundering herd
async fn retry_with_jitter<F, Fut, T>(
    mut operation: F,
    max_retries: usize,
    base_delay: Duration,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = io::Result<T>>,
{
    for attempt in 0..max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                println!("  Attempt {} failed: {}", attempt + 1, e);

                if attempt < max_retries - 1 {
                    // Exponential backoff with jitter
                    let exp_delay = base_delay * 2_u32.pow(attempt as u32);
                    let jitter = Duration::from_millis(rand::random::<u64>() % 100);
                    let total_delay = exp_delay + jitter;

                    println!("  Waiting {:?} (with jitter)...", total_delay);
                    sleep(total_delay).await;
                }
            }
        }
    }

    Err("All retry attempts failed".into())
}

// Demonstrate retry with jitter
async fn retry_demo_jitter() {
    println!("\n=== Retry Demo (With Jitter) ===\n");
    println!("Exponential backoff with random jitter prevents thundering herd\n");

    let result = retry_with_jitter(
        || unreliable_operation(0.6),  // 60% failure rate
        4,
        Duration::from_millis(100),  // Base delay
    ).await;

    match result {
        Ok(s) => println!("\nFinal result: {}", s),
        Err(e) => println!("\nFailed after all retries: {}", e),
    }
}

// Circuit breaker pattern
struct CircuitBreaker {
    failure_count: usize,
    threshold: usize,
    last_failure: Option<Instant>,
    reset_timeout: Duration,
}

impl CircuitBreaker {
    fn new(threshold: usize, reset_timeout: Duration) -> Self {
        CircuitBreaker {
            failure_count: 0,
            threshold,
            last_failure: None,
            reset_timeout,
        }
    }

    fn is_open(&self) -> bool {
        if self.failure_count >= self.threshold {
            if let Some(last) = self.last_failure {
                // Check if reset timeout has passed
                if last.elapsed() < self.reset_timeout {
                    return true;  // Circuit is open
                }
            }
        }
        false
    }

    fn record_success(&mut self) {
        self.failure_count = 0;
        self.last_failure = None;
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Instant::now());
    }
}

// Demonstrate circuit breaker
async fn circuit_breaker_demo() {
    println!("\n=== Circuit Breaker Demo ===\n");
    println!("Circuit opens after 3 failures, resets after 1 second\n");

    let mut breaker = CircuitBreaker::new(3, Duration::from_secs(1));

    for i in 0..10 {
        if breaker.is_open() {
            println!("Request {}: REJECTED (circuit open)", i);
            sleep(Duration::from_millis(300)).await;
            continue;
        }

        println!("Request {}: Attempting...", i);

        // Simulate operation that fails first 5 times
        let result = if i < 5 {
            Err(io::Error::new(io::ErrorKind::Other, "Simulated failure"))
        } else {
            Ok("Success")
        };

        match result {
            Ok(s) => {
                println!("Request {}: {}", i, s);
                breaker.record_success();
            }
            Err(e) => {
                println!("Request {}: Failed - {}", i, e);
                breaker.record_failure();
            }
        }

        sleep(Duration::from_millis(200)).await;
    }

    println!("\nWaiting for circuit to reset...");
    sleep(Duration::from_secs(1)).await;

    println!("\nCircuit should be closed now:");
    if !breaker.is_open() {
        println!("Circuit is CLOSED - ready to accept requests");
    }
}

#[tokio::main]
async fn main() {
    println!("=== Retry Patterns Demo ===\n");

    retry_demo_failures().await;
    retry_demo_timeouts().await;
    retry_demo_jitter().await;
    circuit_breaker_demo().await;

    println!("\nRetry patterns demo completed");
}
