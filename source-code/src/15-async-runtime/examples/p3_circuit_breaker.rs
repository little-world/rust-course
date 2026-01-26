//! Pattern 3: Async/Await Patterns
//! Circuit breaker
//!
//! Run with: cargo run --example p3_circuit_breaker

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Clone, Copy, Debug)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_threshold: usize,
    failures: Arc<Mutex<usize>>,
    #[allow(dead_code)]
    success_threshold: usize,
    timeout: Duration,
}

impl CircuitBreaker {
    fn new(failure_threshold: usize, success_threshold: usize, timeout: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_threshold,
            failures: Arc::new(Mutex::new(0)),
            success_threshold,
            timeout,
        }
    }

    async fn call<F, Fut, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: From<String>,
    {
        let state = *self.state.lock().await;

        match state {
            CircuitState::Open => {
                return Err(E::from("Circuit breaker is open".to_string()));
            }
            CircuitState::HalfOpen => {
                // Try to recover
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }

        match operation().await {
            Ok(result) => {
                // Reset failures
                *self.failures.lock().await = 0;
                if matches!(state, CircuitState::HalfOpen) {
                    *self.state.lock().await = CircuitState::Closed;
                }
                Ok(result)
            }
            Err(e) => {
                let mut failures = self.failures.lock().await;
                *failures += 1;

                if *failures >= self.failure_threshold {
                    println!("Circuit breaker opened due to {} failures", failures);
                    *self.state.lock().await = CircuitState::Open;

                    // Schedule transition to half-open
                    let state = Arc::clone(&self.state);
                    let timeout = self.timeout;
                    tokio::spawn(async move {
                        tokio::time::sleep(timeout).await;
                        *state.lock().await = CircuitState::HalfOpen;
                        println!("Circuit breaker transitioned to half-open");
                    });
                }

                Err(e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let breaker = CircuitBreaker::new(3, 2, Duration::from_secs(2));
    let result: Result<String, String> = breaker.call(|| async {
        Ok("Success".to_string())
    }).await;
    println!("Result: {:?}", result);
}
