//! Pattern 3: Basic HTTP Client
//!
//! Demonstrates basic HTTP GET requests using reqwest with JSON deserialization.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct HttpBinResponse {
    origin: String,
    url: String,
}

/// Simple GET request
async fn simple_get_request() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Simple GET Request ---\n");

    // GET request returns a Response
    let response = reqwest::get("https://httpbin.org/get").await?;

    println!("Status: {}", response.status());
    println!("Content-Type: {:?}", response.headers().get("content-type"));

    // Read the response body as text
    let body = response.text().await?;
    println!("Body length: {} bytes", body.len());
    println!("Body preview: {}...", &body[..200.min(body.len())]);

    Ok(())
}

/// GET request with JSON deserialization
async fn get_json() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- GET with JSON Deserialization ---\n");

    let response: HttpBinResponse = reqwest::get("https://httpbin.org/get")
        .await?
        .json()
        .await?;

    println!("Deserialized response:");
    println!("  Origin: {}", response.origin);
    println!("  URL: {}", response.url);

    Ok(())
}

/// GET with query parameters
async fn get_with_params() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- GET with Query Parameters ---\n");

    let client = reqwest::Client::new();

    let response = client
        .get("https://httpbin.org/get")
        .query(&[("name", "Alice"), ("age", "30")])
        .send()
        .await?;

    println!("Status: {}", response.status());

    let body: serde_json::Value = response.json().await?;
    println!("Query args received by server: {}", body["args"]);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pattern 3: Basic HTTP Client ===\n");

    simple_get_request().await?;
    get_json().await?;
    get_with_params().await?;

    println!("\nHTTP client examples completed!");

    Ok(())
}
