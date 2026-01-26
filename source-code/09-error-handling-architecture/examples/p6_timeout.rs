//! Pattern 6: Error Handling in Async Contexts
//! Example: Timeout Wrapping
//!
//! Run with: cargo run --example p6_timeout

use anyhow::{Context, Result};
use std::time::Duration;

/// Simulates a slow operation.
async fn slow_operation(delay_ms: u64) -> Result<String> {
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    Ok(format!("Completed after {}ms", delay_ms))
}

/// Wrap any async operation with a timeout.
async fn with_timeout<T>(
    duration: Duration,
    operation: impl std::future::Future<Output = Result<T>>,
    context: &str,
) -> Result<T> {
    tokio::time::timeout(duration, operation)
        .await
        .map_err(|_| anyhow::anyhow!("Operation timed out after {:?}", duration))?
        .context(context.to_string())
}

/// Fetch data with configurable timeout.
async fn fetch_with_timeout(id: u64, timeout_ms: u64) -> Result<String> {
    let timeout = Duration::from_millis(timeout_ms);

    tokio::time::timeout(timeout, slow_operation(100))
        .await
        .map_err(|_| anyhow::anyhow!("Timeout fetching data for ID {}", id))?
}

/// Multiple operations with individual timeouts.
async fn complex_operation() -> Result<String> {
    // Each step has its own timeout
    let step1 = with_timeout(
        Duration::from_millis(200),
        slow_operation(50),
        "Step 1: fetch config",
    )
    .await?;

    let step2 = with_timeout(
        Duration::from_millis(200),
        slow_operation(100),
        "Step 2: load data",
    )
    .await?;

    let step3 = with_timeout(
        Duration::from_millis(200),
        slow_operation(50),
        "Step 3: process",
    )
    .await?;

    Ok(format!("{} -> {} -> {}", step1, step2, step3))
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Timeout Wrapping ===\n");

    // Operation that completes in time
    println!("=== Fast Operation (100ms work, 200ms timeout) ===");
    match fetch_with_timeout(1, 200).await {
        Ok(result) => println!("  Success: {}", result),
        Err(e) => println!("  Error: {}", e),
    }

    // Operation that times out
    println!("\n=== Slow Operation (100ms work, 50ms timeout) ===");
    match fetch_with_timeout(2, 50).await {
        Ok(result) => println!("  Success: {}", result),
        Err(e) => println!("  Error: {}", e),
    }

    // Complex operation with multiple timeouts
    println!("\n=== Complex Operation (multiple steps) ===");
    match complex_operation().await {
        Ok(result) => println!("  Success: {}", result),
        Err(e) => println!("  Error: {}", e),
    }

    // Demonstrate timeout with context
    println!("\n=== Timeout with Context ===");
    let result = with_timeout(
        Duration::from_millis(50),
        slow_operation(200),
        "fetching user profile",
    )
    .await;

    match result {
        Ok(s) => println!("  Success: {}", s),
        Err(e) => {
            println!("  Error chain:");
            for cause in e.chain() {
                println!("    - {}", cause);
            }
        }
    }

    println!("\n=== Timeout Patterns ===");
    println!("Basic timeout:");
    println!("  tokio::time::timeout(duration, future).await");
    println!("    -> Result<T, Elapsed>");
    println!();
    println!("With context:");
    println!("  timeout(dur, op).await");
    println!("    .map_err(|_| anyhow!(\"timeout\"))?");

    println!("\n=== Recommended Timeouts ===");
    println!("Database queries:     1-5 seconds");
    println!("HTTP requests:        5-30 seconds");
    println!("File operations:      1-10 seconds");
    println!("Health checks:        1-3 seconds");
    println!("Background jobs:      varies (minutes to hours)");

    println!("\n=== Key Points ===");
    println!("1. Always timeout external calls (DB, HTTP, files)");
    println!("2. Include timeout duration in error message");
    println!("3. Different operations need different timeouts");
    println!("4. Timeout prevents unbounded waits that hang services");

    Ok(())
}
