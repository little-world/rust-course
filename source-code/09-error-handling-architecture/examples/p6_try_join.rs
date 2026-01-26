//! Pattern 6: Error Handling in Async Contexts
//! Example: Concurrent Operations with try_join_all
//!
//! Run with: cargo run --example p6_try_join

use anyhow::{Context, Result};
use futures::future::try_join_all;
use std::time::Duration;

#[derive(Debug)]
struct User {
    id: u64,
    name: String,
}

/// Simulate fetching a user.
async fn fetch_user(id: u64) -> Result<User> {
    tokio::time::sleep(Duration::from_millis(50)).await;

    if id == 0 {
        anyhow::bail!("User ID 0 is invalid");
    }
    if id > 100 {
        anyhow::bail!("User ID {} not found", id);
    }

    Ok(User {
        id,
        name: format!("User{}", id),
    })
}

/// Fetch multiple users concurrently, fail if any fails.
async fn fetch_all_users(ids: Vec<u64>) -> Result<Vec<User>> {
    let futures = ids.into_iter().map(fetch_user);
    try_join_all(futures)
        .await
        .context("Failed to fetch all users")
}

/// Fetch users with partial success (collect all results).
async fn fetch_users_lenient(ids: Vec<u64>) -> (Vec<User>, Vec<String>) {
    let futures = ids.into_iter().map(|id| async move {
        fetch_user(id).await.map_err(|e| format!("ID {}: {}", id, e))
    });

    let results = futures::future::join_all(futures).await;

    let mut users = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok(user) => users.push(user),
            Err(e) => errors.push(e),
        }
    }

    (users, errors)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Concurrent Operations with try_join_all ===\n");

    // All successful
    println!("=== All Successful ===");
    let ids = vec![1, 2, 3, 4, 5];
    match fetch_all_users(ids).await {
        Ok(users) => {
            println!("  Fetched {} users:", users.len());
            for user in &users {
                println!("    - {:?}", user);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }

    // One failure (fail fast)
    println!("\n=== One Failure (fail fast) ===");
    let ids = vec![1, 2, 0, 4, 5]; // ID 0 will fail
    match fetch_all_users(ids).await {
        Ok(users) => println!("  Fetched {} users", users.len()),
        Err(e) => println!("  Error (stopped at first failure): {}", e),
    }

    // Lenient mode (collect all)
    println!("\n=== Lenient Mode (collect all) ===");
    let ids = vec![1, 0, 3, 200, 5]; // 0 and 200 will fail
    let (users, errors) = fetch_users_lenient(ids).await;
    println!("  Successes: {} users", users.len());
    for user in &users {
        println!("    - {:?}", user);
    }
    println!("  Failures: {}", errors.len());
    for err in &errors {
        println!("    - {}", err);
    }

    println!("\n=== try_join_all vs join_all ===");
    println!("try_join_all:");
    println!("  - Returns Result<Vec<T>, E>");
    println!("  - Fails fast on first error");
    println!("  - Cancels remaining futures on error");
    println!();
    println!("join_all:");
    println!("  - Returns Vec<Result<T, E>>");
    println!("  - Waits for all futures");
    println!("  - You handle each result individually");

    println!("\n=== Choosing Between Them ===");
    println!("Use try_join_all when:");
    println!("  - All results are required");
    println!("  - One failure means entire operation fails");
    println!("  - Example: fetching data for a single request");
    println!();
    println!("Use join_all when:");
    println!("  - Partial results are acceptable");
    println!("  - Want to report all errors");
    println!("  - Example: batch processing, data migration");

    println!("\n=== Key Points ===");
    println!("1. try_join_all for fail-fast concurrent operations");
    println!("2. join_all + partition for lenient concurrent operations");
    println!("3. Futures run concurrently, not sequentially");
    println!("4. Cancelled futures may have side effects (be careful!)");

    Ok(())
}
