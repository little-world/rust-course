//! Pattern 1: Builder Pattern Variations
//! Example: Basic Consuming Builder
//!
//! Run with: cargo run --example p1_consuming_builder

use std::time::Duration;

#[derive(Debug)]
pub struct Request {
    url: String,
    method: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    timeout: Option<Duration>,
    retry_count: u32,
    follow_redirects: bool,
}

pub struct RequestBuilder {
    url: String,
    method: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    timeout: Option<Duration>,
    retry_count: u32,
    follow_redirects: bool,
}

impl Request {
    // Provide a convenient entry point to the builder.
    pub fn builder(url: impl Into<String>) -> RequestBuilder {
        RequestBuilder::new(url)
    }
}

impl RequestBuilder {
    // The `new` function sets defaults for all fields.
    pub fn new(url: impl Into<String>) -> Self {
        RequestBuilder {
            url: url.into(),
            method: "GET".to_string(),
            headers: Vec::new(),
            body: None,
            timeout: None,
            retry_count: 0,
            follow_redirects: true,
        }
    }

    // Each method takes `self` and returns `self` for chaining.
    pub fn method(mut self, method: impl Into<String>) -> Self {
        self.method = method.into();
        self
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn retry_count(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }

    pub fn follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    // The `build` method consumes builder, creates final object.
    pub fn build(self) -> Request {
        Request {
            url: self.url,
            method: self.method,
            headers: self.headers,
            body: self.body,
            timeout: self.timeout,
            retry_count: self.retry_count,
            follow_redirects: self.follow_redirects,
        }
    }
}

fn main() {
    println!("=== Basic Consuming Builder ===");
    // Usage: Chain setters for readable configuration; build() creates final object.
    let request = Request::builder("https://api.example.com")
        .method("POST")
        .header("Authorization", "Bearer token")
        .header("Content-Type", "application/json")
        .body(r#"{"data": "value"}"#)
        .timeout(Duration::from_secs(30))
        .retry_count(3)
        .build();

    println!("Built request: {:#?}", request);

    println!("\n=== GET Request with Defaults ===");
    let get_request = Request::builder("https://api.example.com/users")
        .header("Accept", "application/json")
        .build();

    println!("GET request: {:#?}", get_request);

    println!("\n=== Builder Cannot Be Reused ===");
    println!("Each setter consumes `self`, so the builder moves through the chain.");
    println!("After .build(), the builder is consumed and cannot be reused.");

    // This would NOT compile:
    // let builder = Request::builder("https://example.com");
    // let r1 = builder.build();
    // let r2 = builder.build(); // ERROR: use of moved value
}
