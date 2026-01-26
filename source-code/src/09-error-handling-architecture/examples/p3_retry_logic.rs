//! Pattern 3: Error Propagation Strategies
//! Example: Retry Logic with Error Inspection
//!
//! Run with: cargo run --example p3_retry_logic

use std::time::Duration;

/// Retry an operation with configurable attempts and delay.
fn retry_on_error<F, T, E>(
    mut f: F,
    max_attempts: usize,
    delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Display,
{
    let mut attempts = 0;
    loop {
        attempts += 1;
        match f() {
            Ok(value) => return Ok(value),
            Err(e) if attempts < max_attempts => {
                eprintln!("  Attempt {}/{}: {}, retrying...", attempts, max_attempts, e);
                std::thread::sleep(delay);
            }
            Err(e) => {
                eprintln!("  Attempt {}/{}: {}, giving up", attempts, max_attempts, e);
                return Err(e);
            }
        }
    }
}

/// Retry with exponential backoff.
fn retry_with_backoff<F, T, E>(
    mut f: F,
    max_attempts: usize,
    initial_delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Display,
{
    let mut attempts = 0;
    let mut delay = initial_delay;

    loop {
        attempts += 1;
        match f() {
            Ok(value) => return Ok(value),
            Err(e) if attempts < max_attempts => {
                eprintln!(
                    "  Attempt {}/{}: {}, waiting {:?}...",
                    attempts, max_attempts, e, delay
                );
                std::thread::sleep(delay);
                delay *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
}

/// Simulates a flaky operation that fails randomly.
fn flaky_operation(failure_rate: f64) -> Result<String, &'static str> {
    if rand_simple() < failure_rate {
        Err("transient failure")
    } else {
        Ok("success!".to_string())
    }
}

/// Simple pseudo-random for demo (not cryptographically secure).
fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 100) as f64 / 100.0
}

/// Simulates countdown to success.
static mut COUNTER: i32 = 0;

fn eventually_succeeds() -> Result<String, &'static str> {
    unsafe {
        COUNTER += 1;
        if COUNTER < 3 {
            Err("not ready yet")
        } else {
            COUNTER = 0;
            Ok("ready!".to_string())
        }
    }
}

fn main() {
    println!("=== Retry Logic ===\n");

    // Fixed delay retry
    println!("=== Fixed Delay Retry ===");
    let result = retry_on_error(
        eventually_succeeds,
        5,
        Duration::from_millis(100),
    );
    println!("  Result: {:?}\n", result);

    // Exponential backoff
    println!("=== Exponential Backoff ===");
    unsafe { COUNTER = 0; }
    let result = retry_with_backoff(
        eventually_succeeds,
        5,
        Duration::from_millis(50),
    );
    println!("  Result: {:?}\n", result);

    // Retry with flaky operation
    println!("=== Flaky Operation (50% failure rate) ===");
    for i in 1..=3 {
        println!("  Run {}:", i);
        let result = retry_on_error(
            || flaky_operation(0.5),
            3,
            Duration::from_millis(50),
        );
        println!("    Final: {:?}\n", result);
    }

    println!("=== Retry Strategies ===");
    println!("1. Fixed delay:       sleep(100ms) between attempts");
    println!("2. Linear backoff:    delay += 100ms each attempt");
    println!("3. Exponential:       delay *= 2 each attempt");
    println!("4. Jitter:            delay + random(0..delay) to prevent thundering herd");
    println!("5. Circuit breaker:   stop retrying after threshold");

    println!("\n=== When to Retry ===");
    println!("DO retry:");
    println!("  - Network timeouts");
    println!("  - Rate limiting (429)");
    println!("  - Server errors (500, 502, 503, 504)");
    println!("  - Transient failures");
    println!();
    println!("DON'T retry:");
    println!("  - Client errors (400, 401, 403, 404)");
    println!("  - Validation failures");
    println!("  - Authentication errors");
    println!("  - Resource not found");

    println!("\n=== Key Points ===");
    println!("1. Match guard separates retriable from final errors");
    println!("2. Exponential backoff prevents hammering failing services");
    println!("3. Max attempts prevents infinite loops");
    println!("4. Log each retry for observability");
}
