//! Pattern 6: Error Handling in Async Contexts
//! Example: Async Error Propagation
//!
//! Run with: cargo run --example p6_async_propagation

use anyhow::{Context, Result};
use std::time::Duration;

#[derive(Debug)]
struct User {
    id: u64,
    name: String,
}

#[derive(Debug)]
struct Response {
    body: String,
}

/// Simulate HTTP request.
async fn make_http_request(id: u64) -> Result<Response> {
    tokio::time::sleep(Duration::from_millis(10)).await;

    if id == 0 {
        anyhow::bail!("User ID cannot be 0");
    }

    Ok(Response {
        body: format!(r#"{{"id":{},"name":"User{}"}}"#, id, id),
    })
}

/// Simulate response parsing.
async fn parse_response(resp: Response) -> Result<User> {
    tokio::time::sleep(Duration::from_millis(5)).await;

    // Simple parsing simulation
    if resp.body.contains("error") {
        anyhow::bail!("Response contained error");
    }

    Ok(User {
        id: 1,
        name: "ParsedUser".to_string(),
    })
}

/// Fetch user data with error propagation.
async fn fetch_user_data(id: u64) -> Result<User> {
    let response = make_http_request(id)
        .await
        .context("Failed to make HTTP request")?;

    let user = parse_response(response)
        .await
        .context("Failed to parse response")?;

    Ok(user)
}

/// Demonstrate chained async operations.
async fn load_user_profile(id: u64) -> Result<String> {
    let user = fetch_user_data(id)
        .await
        .context("Failed to fetch user")?;

    // Simulate additional async work
    tokio::time::sleep(Duration::from_millis(5)).await;

    Ok(format!("Profile: {} (ID: {})", user.name, user.id))
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Async Error Propagation ===\n");

    // Successful request
    println!("=== Successful Request ===");
    match fetch_user_data(42).await {
        Ok(user) => println!("  Success: {:?}", user),
        Err(e) => println!("  Error: {}", e),
    }

    // Failed request (ID = 0)
    println!("\n=== Failed Request (ID = 0) ===");
    match fetch_user_data(0).await {
        Ok(user) => println!("  Success: {:?}", user),
        Err(e) => {
            println!("  Error chain:");
            for (i, cause) in e.chain().enumerate() {
                println!("    {}: {}", i, cause);
            }
        }
    }

    // Full profile load
    println!("\n=== Full Profile Load ===");
    for id in [1, 0] {
        print!("  Loading profile for ID {}: ", id);
        match load_user_profile(id).await {
            Ok(profile) => println!("{}", profile),
            Err(e) => println!("Error: {}", e),
        }
    }

    println!("\n=== Async Error Patterns ===");
    println!("1. ? works across .await points");
    println!("2. .context() adds info at each async boundary");
    println!("3. Errors bubble up through async call chain");
    println!("4. Same patterns as sync code!");

    println!("\n=== Example Code ===");
    println!("async fn fetch_user(id: u64) -> Result<User> {{");
    println!("    let resp = make_request(id).await?;  // ? works!");
    println!("    let user = parse(resp).await?;");
    println!("    Ok(user)");
    println!("}}");

    println!("\n=== Key Points ===");
    println!("1. ? operator works seamlessly in async functions");
    println!("2. anyhow::Context works with async Results");
    println!("3. Error chain preserved across await points");
    println!("4. Each .await can independently fail");

    Ok(())
}
