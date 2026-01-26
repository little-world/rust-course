//! Pattern 3: HTTP POST Requests
//!
//! Demonstrates POST requests with JSON body and form data.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct CreateUser {
    username: String,
    email: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct HttpBinPostResponse {
    json: Option<CreateUser>,
    form: Option<HashMap<String, String>>,
    data: String,
}

/// POST request with JSON body
async fn post_json() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- POST with JSON Body ---\n");

    let client = Client::new();

    let new_user = CreateUser {
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    let response = client
        .post("https://httpbin.org/post")
        .json(&new_user)
        .send()
        .await?;

    println!("Status: {}", response.status());

    let body: serde_json::Value = response.json().await?;
    println!("Server received JSON: {}", body["json"]);

    Ok(())
}

/// POST with form data
async fn post_form() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- POST with Form Data ---\n");

    let client = Client::new();

    let mut form_data = HashMap::new();
    form_data.insert("username", "bob");
    form_data.insert("password", "secret123");

    let response = client
        .post("https://httpbin.org/post")
        .form(&form_data)
        .send()
        .await?;

    println!("Status: {}", response.status());

    let body: serde_json::Value = response.json().await?;
    println!("Server received form: {}", body["form"]);

    Ok(())
}

/// POST with raw body
async fn post_raw() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- POST with Raw Body ---\n");

    let client = Client::new();

    let response = client
        .post("https://httpbin.org/post")
        .header("Content-Type", "text/plain")
        .body("This is raw text data")
        .send()
        .await?;

    println!("Status: {}", response.status());

    let body: serde_json::Value = response.json().await?;
    println!("Server received data: {}", body["data"]);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pattern 3: HTTP POST Requests ===\n");

    post_json().await?;
    post_form().await?;
    post_raw().await?;

    println!("\nPOST examples completed!");

    Ok(())
}
