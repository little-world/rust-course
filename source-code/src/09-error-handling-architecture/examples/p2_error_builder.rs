//! Pattern 2: Error Builder Pattern
//! Example: Fluent Error Construction
//!
//! Run with: cargo run --example p2_error_builder

use thiserror::Error;

/// HTTP error with many optional fields.
#[derive(Error, Debug)]
#[error("HTTP {method} {url} failed (status: {status})")]
pub struct HttpError {
    pub method: String,
    pub url: String,
    pub status: u16,
    pub body: Option<String>,
    pub headers: Option<Vec<(String, String)>>,
    pub duration_ms: Option<u64>,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

/// Builder for constructing HttpError with many optional fields.
/// Prevents constructors with numerous parameters.
pub struct HttpErrorBuilder {
    method: String,
    url: String,
    status: u16,
    body: Option<String>,
    headers: Option<Vec<(String, String)>>,
    duration_ms: Option<u64>,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl HttpErrorBuilder {
    /// Create a new builder with required fields.
    pub fn new(method: impl Into<String>, url: impl Into<String>, status: u16) -> Self {
        HttpErrorBuilder {
            method: method.into(),
            url: url.into(),
            status,
            body: None,
            headers: None,
            duration_ms: None,
            source: None,
        }
    }

    /// Add response body.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Add response headers.
    pub fn headers(mut self, headers: Vec<(String, String)>) -> Self {
        self.headers = Some(headers);
        self
    }

    /// Add request duration.
    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = Some(ms);
        self
    }

    /// Add underlying source error.
    pub fn source(mut self, err: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(err));
        self
    }

    /// Build the final HttpError.
    pub fn build(self) -> HttpError {
        HttpError {
            method: self.method,
            url: self.url,
            status: self.status,
            body: self.body,
            headers: self.headers,
            duration_ms: self.duration_ms,
            source: self.source,
        }
    }
}

impl HttpError {
    /// Format a detailed error report for logging.
    pub fn report(&self) -> String {
        let mut lines = vec![format!("HTTP Error Report:")];
        lines.push(format!("  Request: {} {}", self.method, self.url));
        lines.push(format!("  Status: {}", self.status));

        if let Some(ms) = self.duration_ms {
            lines.push(format!("  Duration: {}ms", ms));
        }

        if let Some(headers) = &self.headers {
            lines.push("  Headers:".to_string());
            for (k, v) in headers {
                lines.push(format!("    {}: {}", k, v));
            }
        }

        if let Some(body) = &self.body {
            lines.push(format!("  Body: {}", body));
        }

        if let Some(source) = &self.source {
            lines.push(format!("  Cause: {}", source));
        }

        lines.join("\n")
    }
}

fn main() {
    println!("=== Error Builder Pattern ===\n");

    // Simple error with just required fields
    let err1 = HttpErrorBuilder::new("GET", "/api/users", 404).build();
    println!("Simple error:\n  {}\n", err1);

    // Error with some optional fields
    let err2 = HttpErrorBuilder::new("POST", "/api/orders", 500)
        .body(r#"{"error": "Database connection failed"}"#)
        .duration_ms(1523)
        .build();
    println!("Error with body and duration:\n  {}\n", err2);

    // Fully detailed error
    let io_error = std::io::Error::new(std::io::ErrorKind::TimedOut, "read timeout");
    let err3 = HttpErrorBuilder::new("GET", "/api/reports/large", 504)
        .body("Gateway Timeout")
        .headers(vec![
            ("X-Request-Id".into(), "abc-123".into()),
            ("X-Retry-After".into(), "30".into()),
        ])
        .duration_ms(30000)
        .source(io_error)
        .build();

    println!("=== Full Error Report ===");
    println!("{}\n", err3.report());

    // Demonstrate readability at call sites
    println!("=== Builder vs Direct Construction ===");
    println!("Builder pattern makes construction readable:");
    println!();
    println!("  HttpErrorBuilder::new(\"POST\", \"/users\", 500)");
    println!("      .body(\"Internal error\")");
    println!("      .duration_ms(1500)");
    println!("      .build()");
    println!();
    println!("vs direct construction with many fields:");
    println!();
    println!("  HttpError {{");
    println!("      method: \"POST\".into(),");
    println!("      url: \"/users\".into(),");
    println!("      status: 500,");
    println!("      body: Some(\"Internal error\".into()),");
    println!("      headers: None,");
    println!("      duration_ms: Some(1500),");
    println!("      source: None,");
    println!("  }}");

    println!("\n=== Key Points ===");
    println!("1. Builder pattern handles errors with many optional fields");
    println!("2. Required fields go in new(), optional in builder methods");
    println!("3. Call site clearly shows what context is being added");
    println!("4. Easy to extend with new fields without breaking API");
}
