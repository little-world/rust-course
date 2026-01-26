//! Pattern 6: Error Handling in Async Contexts
//! Example: Stream Error Recovery
//!
//! Run with: cargo run --example p6_stream_recovery

use anyhow::Result;
use futures::stream::BoxStream;
use futures::StreamExt;
use std::time::Duration;

/// Create a stream that occasionally fails.
/// Returns a boxed stream to satisfy `Unpin` requirements.
fn create_flaky_stream() -> BoxStream<'static, Result<i32>> {
    futures::stream::iter(0..10)
        .then(|i| async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            if i == 3 || i == 7 {
                Err(anyhow::anyhow!("Transient error at item {}", i))
            } else {
                Ok(i * 10)
            }
        })
        .boxed()
}

/// Process stream, logging errors and continuing.
async fn process_stream_lenient(mut stream: BoxStream<'_, Result<i32>>) -> (Vec<i32>, usize) {
    let mut successes = Vec::new();
    let mut error_count = 0;

    while let Some(result) = stream.next().await {
        match result {
            Ok(value) => {
                println!("  Processed: {}", value);
                successes.push(value);
            }
            Err(e) => {
                eprintln!("  Error (skipping): {}", e);
                error_count += 1;
            }
        }
    }

    (successes, error_count)
}

/// Process stream, stopping on first error.
async fn process_stream_strict(mut stream: BoxStream<'_, Result<i32>>) -> Result<Vec<i32>> {
    let mut results = Vec::new();

    while let Some(result) = stream.next().await {
        results.push(result?); // Propagate error
    }

    Ok(results)
}

/// Process stream with error threshold.
async fn process_stream_with_threshold(
    mut stream: BoxStream<'_, Result<i32>>,
    max_errors: usize,
) -> Result<Vec<i32>> {
    let mut results = Vec::new();
    let mut error_count = 0;

    while let Some(result) = stream.next().await {
        match result {
            Ok(value) => results.push(value),
            Err(e) => {
                error_count += 1;
                eprintln!("  Error {}/{}: {}", error_count, max_errors, e);
                if error_count >= max_errors {
                    anyhow::bail!("Error threshold exceeded ({} errors)", error_count);
                }
            }
        }
    }

    Ok(results)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Stream Error Recovery ===\n");

    // Lenient processing (log and continue)
    println!("=== Lenient Mode ===");
    let stream = create_flaky_stream();
    let (values, errors) = process_stream_lenient(stream).await;
    println!("  Completed: {} values, {} errors\n", values.len(), errors);

    // Strict processing (fail on first error)
    println!("=== Strict Mode ===");
    let stream = create_flaky_stream();
    match process_stream_strict(stream).await {
        Ok(values) => println!("  Completed: {} values", values.len()),
        Err(e) => println!("  Stopped at first error: {}", e),
    }

    // Threshold processing
    println!("\n=== Threshold Mode (max 1 error) ===");
    let stream = create_flaky_stream();
    match process_stream_with_threshold(stream, 1).await {
        Ok(values) => println!("  Completed: {} values", values.len()),
        Err(e) => println!("  Aborted: {}", e),
    }

    println!("\n=== Threshold Mode (max 3 errors) ===");
    let stream = create_flaky_stream();
    match process_stream_with_threshold(stream, 3).await {
        Ok(values) => println!("  Completed: {} values", values.len()),
        Err(e) => println!("  Aborted: {}", e),
    }

    println!("\n=== Stream Processing Strategies ===");
    println!("Lenient (log and continue):");
    println!("  - Use for data pipelines where partial results OK");
    println!("  - Collect stats on error rate");
    println!();
    println!("Strict (fail fast):");
    println!("  - Use when all items must succeed");
    println!("  - Transactions, ordered operations");
    println!();
    println!("Threshold:");
    println!("  - Tolerate some errors, abort on excessive failure");
    println!("  - Good for flaky sources with retry upstream");

    println!("\n=== Key Points ===");
    println!("1. Streams process items one at a time");
    println!("2. Can handle each error independently");
    println!("3. Choose strategy based on requirements");
    println!("4. Track error metrics for observability");

    Ok(())
}
