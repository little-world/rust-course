//! Pattern 1: Future Composition
//! Error conversion and propagation
//!
//! Run with: cargo run --example p1_error_conversion

#[derive(Debug)]
enum AppError {
    Network(String),
    NotFound,
    InvalidData(String),
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Network(err.to_string())
    }
}

async fn fetch_json_data(url: &str) -> Result<serde_json::Value, AppError> {
    let response = reqwest::get(url).await?;  // Auto-converts reqwest::Error

    if !response.status().is_success() {
        return Err(AppError::NotFound);
    }

    let data = response.json().await?;
    Ok(data)
}

#[tokio::main]
async fn main() {
    match fetch_json_data("https://api.github.com/users/rust-lang").await {
        Ok(data) => println!("Got: {}", data),
        Err(AppError::Network(e)) => println!("Network error: {}", e),
        Err(AppError::NotFound) => println!("Resource not found"),
        Err(AppError::InvalidData(e)) => println!("Bad data: {}", e),
    }
}
