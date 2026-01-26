//! Pattern 3: HTTP Error Handling and Retries
//!
//! Demonstrates retry logic with exponential backoff for resilient HTTP clients.

use reqwest::{Client, StatusCode};
use tokio::time::{sleep, Duration};

/// Retry a request on failure with exponential backoff
async fn request_with_retry(
    client: &Client,
    url: &str,
    max_retries: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut attempts = 0;

    loop {
        attempts += 1;
        println!("  Attempt {}/{}", attempts, max_retries);

        match client.get(url).send().await {
            Ok(response) => {
                match response.status() {
                    StatusCode::OK => {
                        println!("  Success!");
                        return Ok(response.text().await?);
                    }
                    StatusCode::TOO_MANY_REQUESTS => {
                        // Rate limited - wait and retry
                        if attempts >= max_retries {
                            return Err("Max retries exceeded (rate limited)".into());
                        }
                        println!("  Rate limited (429), waiting 5s...");
                        sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                    status if status.is_server_error() => {
                        // Server error - retry with backoff
                        if attempts >= max_retries {
                            return Err(format!("Server error after {} retries: {}", max_retries, status).into());
                        }
                        let backoff_secs = 2u64.pow(attempts);
                        println!("  Server error ({}), retrying in {}s...", status, backoff_secs);
                        sleep(Duration::from_secs(backoff_secs)).await;
                        continue;
                    }
                    status => {
                        // Client error - don't retry (4xx except 429)
                        return Err(format!("HTTP error (won't retry): {}", status).into());
                    }
                }
            }
            Err(e) => {
                if attempts >= max_retries {
                    return Err(format!("Request failed after {} attempts: {}", max_retries, e).into());
                }
                let backoff_secs = 2u64.pow(attempts);
                println!("  Network error: {}, retrying in {}s...", e, backoff_secs);
                sleep(Duration::from_secs(backoff_secs)).await;
                continue;
            }
        }
    }
}

/// Test different status codes
async fn test_status_codes() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let test_cases = [
        ("https://httpbin.org/status/200", "Normal success"),
        ("https://httpbin.org/status/404", "Not found (client error)"),
        ("https://httpbin.org/status/500", "Server error (will retry)"),
    ];

    for (url, description) in test_cases {
        println!("\n--- {} ---", description);
        println!("URL: {}", url);

        match request_with_retry(&client, url, 3).await {
            Ok(body) => {
                println!("Response: {} bytes", body.len());
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    Ok(())
}

/// Check response before consuming body
async fn check_response() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Check Response Before Consuming ---\n");

    let response = reqwest::get("https://httpbin.org/json").await?;

    // Check status before reading body
    if !response.status().is_success() {
        return Err(format!("Request failed: {}", response.status()).into());
    }

    // Check content type
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    println!("Content-Type: {}", content_type);

    if !content_type.contains("application/json") {
        return Err("Expected JSON response".into());
    }

    // Now consume the body
    let body: serde_json::Value = response.json().await?;
    println!("JSON response: {}", serde_json::to_string_pretty(&body)?);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pattern 3: HTTP Error Handling and Retries ===\n");

    test_status_codes().await?;
    check_response().await?;

    println!("\nRetry examples completed!");

    Ok(())
}
