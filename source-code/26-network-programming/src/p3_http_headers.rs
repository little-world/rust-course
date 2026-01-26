//! Pattern 3: HTTP Headers and Authentication
//!
//! Demonstrates setting custom headers and authentication patterns.

use reqwest::{Client, header};

/// Request with custom headers
async fn request_with_headers() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Request with Custom Headers ---\n");

    let client = Client::new();

    let response = client
        .get("https://httpbin.org/headers")
        .header(header::AUTHORIZATION, "Bearer my_api_token_123")
        .header(header::USER_AGENT, "MyRustApp/1.0")
        .header("X-Custom-Header", "custom-value")
        .send()
        .await?;

    println!("Status: {}", response.status());

    let body: serde_json::Value = response.json().await?;
    println!("Server saw headers:");
    for (key, value) in body["headers"].as_object().unwrap() {
        println!("  {}: {}", key, value);
    }

    Ok(())
}

/// Client with default headers for all requests
async fn client_with_defaults() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Client with Default Headers ---\n");

    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_static("Bearer default_token")
    );
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("application/json")
    );

    // Create a client with default headers
    let client = Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    println!("Created client with default headers and 30s timeout");

    // All requests with this client will include the default headers
    let response = client
        .get("https://httpbin.org/headers")
        .send()
        .await?;

    let body: serde_json::Value = response.json().await?;
    println!("Default headers sent:");
    println!("  Authorization: {}", body["headers"]["Authorization"]);
    println!("  Accept: {}", body["headers"]["Accept"]);

    Ok(())
}

/// Basic authentication
async fn basic_auth() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Basic Authentication ---\n");

    let client = Client::new();

    // httpbin.org/basic-auth/{user}/{passwd} returns 200 if auth matches
    let response = client
        .get("https://httpbin.org/basic-auth/myuser/mypassword")
        .basic_auth("myuser", Some("mypassword"))
        .send()
        .await?;

    println!("Status: {} (expected 200 for successful auth)", response.status());

    if response.status().is_success() {
        let body: serde_json::Value = response.json().await?;
        println!("Auth response: {}", body);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pattern 3: HTTP Headers and Authentication ===\n");

    request_with_headers().await?;
    client_with_defaults().await?;
    basic_auth().await?;

    println!("\nHeaders and auth examples completed!");

    Ok(())
}
