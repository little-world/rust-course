//! Pattern 2: Structured Errors with Multiple Fields
//! Example: HTTP Error with Rich Context
//!
//! Run with: cargo run --example p2_http_error

use thiserror::Error;

/// Structured HTTP error with all relevant context.
/// Complex errors benefit from struct form over enum variants.
#[derive(Error, Debug)]
#[error("HTTP request failed: {method} {url} (status: {status})")]
pub struct HttpError {
    pub method: String,
    pub url: String,
    pub status: u16,
    pub body: Option<String>,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl HttpError {
    /// Create a new HTTP error with required fields.
    pub fn new(method: impl Into<String>, url: impl Into<String>, status: u16) -> Self {
        HttpError {
            method: method.into(),
            url: url.into(),
            status,
            body: None,
            source: None,
        }
    }

    /// Add response body to the error.
    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Add underlying source error.
    pub fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Check if error is retryable based on status code.
    pub fn is_retryable(&self) -> bool {
        matches!(self.status, 429 | 500 | 502 | 503 | 504)
    }

    /// Format detailed error for logging.
    pub fn detailed_message(&self) -> String {
        let mut msg = format!(
            "HTTP Error:\n  Method: {}\n  URL: {}\n  Status: {}",
            self.method, self.url, self.status
        );
        if let Some(body) = &self.body {
            msg.push_str(&format!("\n  Body: {}", body));
        }
        if let Some(source) = &self.source {
            msg.push_str(&format!("\n  Cause: {}", source));
        }
        msg
    }
}

/// Simulate an HTTP request that might fail.
fn make_request(url: &str, should_fail: bool) -> Result<String, HttpError> {
    if should_fail {
        Err(HttpError::new("GET", url, 404)
            .with_body(r#"{"error": "Resource not found"}"#))
    } else {
        Ok(r#"{"data": "success"}"#.to_string())
    }
}

fn main() {
    println!("=== Structured HTTP Errors ===\n");

    // Create errors with different levels of context
    let err1 = HttpError::new("GET", "/api/users/123", 404);
    println!("Simple error:\n  {}\n", err1);

    let err2 = HttpError::new("POST", "/api/orders", 500)
        .with_body(r#"{"error": "Internal server error"}"#);
    println!("Error with body:\n  {}\n", err2);

    let io_err = std::io::Error::new(std::io::ErrorKind::TimedOut, "connection timed out");
    let err3 = HttpError::new("GET", "/api/health", 503)
        .with_source(io_err);
    println!("Error with source:\n  {}\n", err3);

    // Demonstrate detailed logging
    println!("=== Detailed Error Format ===");
    println!("{}\n", err2.detailed_message());

    // Check retryable status
    println!("=== Retryable Errors ===");
    let errors = vec![
        HttpError::new("GET", "/api", 404),
        HttpError::new("GET", "/api", 429),
        HttpError::new("GET", "/api", 500),
        HttpError::new("GET", "/api", 503),
    ];

    for err in errors {
        println!("  Status {}: retryable = {}", err.status, err.is_retryable());
    }

    // Simulate request handling
    println!("\n=== Request Handling ===");
    match make_request("/api/users", false) {
        Ok(response) => println!("Success: {}", response),
        Err(e) => println!("Failed: {}", e),
    }

    match make_request("/api/missing", true) {
        Ok(response) => println!("Success: {}", response),
        Err(e) => {
            println!("Failed: {}", e);
            if e.is_retryable() {
                println!("  -> Will retry...");
            }
        }
    }

    println!("\n=== Key Points ===");
    println!("1. Struct errors group related context together");
    println!("2. #[error(\"...\")] interpolates fields into message");
    println!("3. #[source] preserves the error chain for debugging");
    println!("4. Builder methods add optional context fluently");
}
