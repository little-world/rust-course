//! Pattern 6: Error Handling in Async Contexts
//! Example: Racing Operations with tokio::select!
//!
//! Run with: cargo run --example p6_select_race

use anyhow::Result;
use std::time::Duration;

#[derive(Debug, Clone)]
struct User {
    id: u64,
    name: String,
    source: String,
}

/// Simulate primary data source (slower but authoritative).
async fn fetch_from_primary(id: u64) -> Result<User> {
    tokio::time::sleep(Duration::from_millis(200)).await;
    Ok(User {
        id,
        name: format!("User{}", id),
        source: "primary".to_string(),
    })
}

/// Simulate secondary data source (faster but may be stale).
async fn fetch_from_secondary(id: u64) -> Result<User> {
    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(User {
        id,
        name: format!("CachedUser{}", id),
        source: "secondary".to_string(),
    })
}

/// Fetch from whichever source responds first.
async fn fetch_with_fallback(id: u64) -> Result<User> {
    tokio::select! {
        result = fetch_from_primary(id) => {
            println!("  Primary responded first");
            result
        }
        result = fetch_from_secondary(id) => {
            println!("  Secondary responded first");
            result
        }
    }
}

/// Fetch with timeout, falling back to secondary.
async fn fetch_with_primary_timeout(id: u64, timeout_ms: u64) -> Result<User> {
    let timeout = Duration::from_millis(timeout_ms);

    tokio::select! {
        result = async {
            tokio::time::timeout(timeout, fetch_from_primary(id)).await
        } => {
            match result {
                Ok(user) => {
                    println!("  Primary succeeded within timeout");
                    user
                }
                Err(_) => {
                    println!("  Primary timed out, trying secondary...");
                    fetch_from_secondary(id).await
                }
            }
        }
    }
}

/// Process with graceful shutdown.
async fn process_with_shutdown(mut shutdown: tokio::sync::broadcast::Receiver<()>) {
    let mut count = 0;

    loop {
        tokio::select! {
            _ = shutdown.recv() => {
                println!("  Received shutdown signal after {} operations", count);
                return;
            }
            _ = async {
                tokio::time::sleep(Duration::from_millis(50)).await;
                count += 1;
                println!("  Completed operation {}", count);
            } => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Racing Operations with select! ===\n");

    // Race two sources
    println!("=== Race Primary vs Secondary ===");
    for _ in 0..3 {
        match fetch_with_fallback(42).await {
            Ok(user) => println!("    Got: {:?}\n", user),
            Err(e) => println!("    Error: {}\n", e),
        }
    }

    // Primary with timeout
    println!("=== Primary with Timeout ===");
    println!("  Timeout 100ms (primary takes 200ms):");
    match fetch_with_primary_timeout(1, 100).await {
        Ok(user) => println!("    Got: {:?}", user),
        Err(e) => println!("    Error: {}", e),
    }

    println!("\n  Timeout 300ms (primary takes 200ms):");
    match fetch_with_primary_timeout(1, 300).await {
        Ok(user) => println!("    Got: {:?}", user),
        Err(e) => println!("    Error: {}", e),
    }

    // Shutdown example
    println!("\n=== Graceful Shutdown ===");
    let (tx, rx) = tokio::sync::broadcast::channel(1);

    let handle = tokio::spawn(process_with_shutdown(rx));

    // Let it run for a bit
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Signal shutdown
    println!("  Sending shutdown signal...");
    tx.send(()).ok();

    handle.await?;

    println!("\n=== select! Patterns ===");
    println!("Race to completion:");
    println!("  select! {{");
    println!("      r = source_a() => handle_a(r),");
    println!("      r = source_b() => handle_b(r),");
    println!("  }}");
    println!();
    println!("Timeout with fallback:");
    println!("  select! {{");
    println!("      r = timeout(dur, primary) => r.or(fallback),");
    println!("  }}");
    println!();
    println!("Cancellable loop:");
    println!("  loop {{ select! {{");
    println!("      _ = shutdown.recv() => return,");
    println!("      _ = do_work() => {{}},");
    println!("  }}}}");

    println!("\n=== Key Points ===");
    println!("1. select! runs first future to complete");
    println!("2. Other branches are cancelled");
    println!("3. Use for racing, timeouts, graceful shutdown");
    println!("4. Ensure cancelled operations are safe to abort");

    Ok(())
}
