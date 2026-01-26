//! Pattern 6: Error Handling in Async Contexts
//! Example: Retry with Exponential Backoff
//!
//! Run with: cargo run --example p6_retry_backoff

use anyhow::{Context, Result};
use std::future::Future;
use std::time::Duration;

/// Retry an async operation with exponential backoff.
async fn retry_with_backoff<F, Fut, T>(
    f: F,
    max_attempts: usize,
    initial_delay: Duration,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempt = 0;
    let mut delay = initial_delay;

    loop {
        attempt += 1;
        match f().await {
            Ok(value) => {
                if attempt > 1 {
                    println!("  Succeeded on attempt {}", attempt);
                }
                return Ok(value);
            }
            Err(e) if attempt >= max_attempts => {
                return Err(e.context(format!("Failed after {} attempts", attempt)));
            }
            Err(e) => {
                println!("  Attempt {}/{}: {}", attempt, max_attempts, e);
                println!("  Waiting {:?} before retry...", delay);
                tokio::time::sleep(delay).await;
                delay = delay * 2; // Exponential backoff
            }
        }
    }
}

/// Retry with jitter to avoid thundering herd.
async fn retry_with_jitter<F, Fut, T>(
    f: F,
    max_attempts: usize,
    base_delay: Duration,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempt = 0;

    loop {
        attempt += 1;
        match f().await {
            Ok(value) => return Ok(value),
            Err(e) if attempt >= max_attempts => {
                return Err(e.context(format!("Failed after {} attempts", attempt)));
            }
            Err(e) => {
                // Exponential backoff with jitter
                let base = base_delay.as_millis() as u64 * 2u64.pow(attempt as u32 - 1);
                let jitter = rand_simple() % (base / 2 + 1);
                let delay = Duration::from_millis(base + jitter);

                println!("  Attempt {}/{}: {}", attempt, max_attempts, e);
                println!("  Waiting {:?} (with jitter)...", delay);
                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Simple pseudo-random for demo.
fn rand_simple() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as u64
        % 1000
}

/// Simulate flaky operation that succeeds after some attempts.
static mut CALL_COUNT: u32 = 0;

async fn flaky_operation() -> Result<String> {
    unsafe {
        CALL_COUNT += 1;
        if CALL_COUNT < 3 {
            anyhow::bail!("Transient failure (attempt {})", CALL_COUNT)
        }
        CALL_COUNT = 0;
        Ok("Success!".to_string())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Retry with Exponential Backoff ===\n");

    // Basic exponential backoff
    println!("=== Exponential Backoff ===");
    unsafe { CALL_COUNT = 0; }
    let result = retry_with_backoff(
        flaky_operation,
        5,
        Duration::from_millis(50),
    )
    .await;
    println!("  Result: {:?}\n", result);

    // With jitter
    println!("=== With Jitter ===");
    unsafe { CALL_COUNT = 0; }
    let result = retry_with_jitter(
        flaky_operation,
        5,
        Duration::from_millis(50),
    )
    .await;
    println!("  Result: {:?}\n", result);

    // Demonstrate max attempts exceeded
    println!("=== Max Attempts Exceeded ===");
    let always_fails = || async { Err::<(), _>(anyhow::anyhow!("Always fails")) };
    let result = retry_with_backoff(
        always_fails,
        3,
        Duration::from_millis(50),
    )
    .await;
    println!("  Result: {:?}", result);

    println!("\n=== Backoff Strategies ===");
    println!("Constant:     delay = 100ms (not recommended)");
    println!("Linear:       delay = 100ms * attempt");
    println!("Exponential:  delay = 100ms * 2^attempt");
    println!("With jitter:  delay = exponential + random(0..delay/2)");
    println!("Decorrelated: delay = random(base..prev_delay * 3)");

    println!("\n=== Why Jitter? ===");
    println!("Without jitter: all clients retry at same time");
    println!("  -> Server gets hit with thundering herd");
    println!("  -> Makes outage worse");
    println!();
    println!("With jitter: clients retry at different times");
    println!("  -> Load spreads out");
    println!("  -> Server can recover");

    println!("\n=== Retry Configuration ===");
    println!("max_attempts:  3-5 for user-facing, more for background");
    println!("initial_delay: 50-200ms for interactive, seconds for batch");
    println!("max_delay:     Cap at 30s-60s to avoid excessive waits");

    println!("\n=== Key Points ===");
    println!("1. Exponential backoff prevents hammering failing services");
    println!("2. Jitter prevents thundering herd on recovery");
    println!("3. Cap maximum delay for reasonable UX");
    println!("4. Include attempt count in final error");

    Ok(())
}
