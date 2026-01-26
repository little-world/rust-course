//! Pattern 6: Error Handling in Async Contexts
//! Example: Circuit Breaker Pattern
//!
//! Run with: cargo run --example p6_circuit_breaker

use anyhow::Result;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Closed,   // Normal operation
    Open,     // Failing fast
    HalfOpen, // Testing if service recovered
}

/// Circuit breaker for protecting against cascading failures.
pub struct CircuitBreaker {
    failure_count: AtomicUsize,
    success_count: AtomicUsize,
    threshold: usize,
    recovery_timeout: Duration,
    state: std::sync::Mutex<State>,
    last_failure: std::sync::Mutex<Option<std::time::Instant>>,
}

impl CircuitBreaker {
    pub fn new(threshold: usize, recovery_timeout: Duration) -> Self {
        CircuitBreaker {
            failure_count: AtomicUsize::new(0),
            success_count: AtomicUsize::new(0),
            threshold,
            recovery_timeout,
            state: std::sync::Mutex::new(State::Closed),
            last_failure: std::sync::Mutex::new(None),
        }
    }

    pub fn state(&self) -> State {
        *self.state.lock().unwrap()
    }

    fn should_allow_request(&self) -> bool {
        let mut state = self.state.lock().unwrap();

        match *state {
            State::Closed => true,
            State::Open => {
                // Check if recovery timeout has passed
                let last_failure = self.last_failure.lock().unwrap();
                if let Some(time) = *last_failure {
                    if time.elapsed() >= self.recovery_timeout {
                        *state = State::HalfOpen;
                        println!("  [CIRCUIT] Transitioning to HalfOpen");
                        return true;
                    }
                }
                false
            }
            State::HalfOpen => true, // Allow one request through
        }
    }

    fn record_success(&self) {
        let mut state = self.state.lock().unwrap();
        self.success_count.fetch_add(1, Ordering::Relaxed);

        if *state == State::HalfOpen {
            // Success in half-open means service recovered
            *state = State::Closed;
            self.failure_count.store(0, Ordering::Relaxed);
            println!("  [CIRCUIT] Service recovered, closing circuit");
        }
    }

    fn record_failure(&self) {
        let mut state = self.state.lock().unwrap();
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;

        *self.last_failure.lock().unwrap() = Some(std::time::Instant::now());

        if *state == State::HalfOpen || failures >= self.threshold {
            *state = State::Open;
            println!(
                "  [CIRCUIT] Opening circuit after {} failures",
                failures
            );
        }
    }

    /// Execute a function through the circuit breaker.
    pub async fn call<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        if !self.should_allow_request() {
            anyhow::bail!("Circuit breaker is open - service unavailable");
        }

        match f().await {
            Ok(value) => {
                self.record_success();
                Ok(value)
            }
            Err(e) => {
                self.record_failure();
                Err(e)
            }
        }
    }
}

/// Simulate external service call.
static mut CALL_COUNT: u32 = 0;
static mut SHOULD_FAIL: bool = true;

async fn external_service() -> Result<String> {
    tokio::time::sleep(Duration::from_millis(10)).await;

    unsafe {
        CALL_COUNT += 1;
        if SHOULD_FAIL && CALL_COUNT <= 5 {
            anyhow::bail!("Service unavailable (call {})", CALL_COUNT)
        }
        Ok(format!("Response from call {}", CALL_COUNT))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Circuit Breaker Pattern ===\n");

    let breaker = Arc::new(CircuitBreaker::new(3, Duration::from_millis(500)));

    // Make requests that will fail
    println!("=== Failing Requests (threshold: 3) ===");
    for i in 1..=5 {
        print!("  Request {}: ", i);
        match breaker.call(external_service).await {
            Ok(r) => println!("Success: {}", r),
            Err(e) => println!("Error: {}", e),
        }
        println!("    Circuit state: {:?}", breaker.state());
    }

    // Wait for recovery timeout
    println!("\n=== Waiting for recovery timeout (500ms) ===");
    tokio::time::sleep(Duration::from_millis(600)).await;

    // Service "recovers"
    unsafe { SHOULD_FAIL = false; }

    println!("\n=== After Recovery ===");
    for i in 1..=3 {
        print!("  Request {}: ", i);
        match breaker.call(external_service).await {
            Ok(r) => println!("Success: {}", r),
            Err(e) => println!("Error: {}", e),
        }
        println!("    Circuit state: {:?}", breaker.state());
    }

    println!("\n=== Circuit Breaker States ===");
    println!("CLOSED:");
    println!("  - Normal operation");
    println!("  - Requests go through");
    println!("  - Failures increment counter");
    println!();
    println!("OPEN:");
    println!("  - Fail fast (no requests to service)");
    println!("  - Wait for recovery timeout");
    println!("  - Prevents cascading failures");
    println!();
    println!("HALF-OPEN:");
    println!("  - Allow one test request");
    println!("  - Success -> CLOSED (service recovered)");
    println!("  - Failure -> OPEN (still failing)");

    println!("\n=== Benefits ===");
    println!("1. Prevents hammering failing service");
    println!("2. Allows dependent services to recover");
    println!("3. Fails fast (better UX than timeout)");
    println!("4. Automatic recovery detection");

    println!("\n=== Key Points ===");
    println!("1. Threshold: failures before opening");
    println!("2. Recovery timeout: time before half-open");
    println!("3. Track success/failure rates for monitoring");
    println!("4. Different breakers for different services");

    Ok(())
}
