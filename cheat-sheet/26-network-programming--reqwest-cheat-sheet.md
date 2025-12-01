### Reqwest Cheat Sheet
```rust
// Cargo.toml dependencies:
/*
[dependencies]
reqwest = { version = "0.11", features = ["json", "blocking", "cookies", "multipart"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
*/

use reqwest::{Client, Response, Error, StatusCode, Method, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// ===== BASIC GET REQUEST =====
// Async GET request
#[tokio::main]
async fn basic_get() -> Result<(), Error> {
    let response = reqwest::get("https://api.example.com/data").await?;
    
    // Get response status
    let status = response.status();
    println!("Status: {}", status);
    
    // Get response text
    let body = response.text().await?;
    println!("Body: {}", body);
    
    Ok(())
}

// Blocking GET request
fn blocking_get() -> Result<(), Error> {
    let response = reqwest::blocking::get("https://api.example.com/data")?;
    let body = response.text()?;
    println!("Body: {}", body);
    Ok(())
}

// ===== CLIENT CREATION =====
// Create reusable client
async fn create_client() -> Result<(), Error> {
    let client = Client::new();                              // Basic client
    
    let response = client
        .get("https://api.example.com/data")
        .send()
        .await?;
    
    Ok(())
}

// Client with configuration
async fn configured_client() -> Result<(), Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))                    // Request timeout
        .connect_timeout(Duration::from_secs(10))            // Connect timeout
        .user_agent("MyApp/1.0")                             // User agent
        .gzip(true)                                          // Enable gzip
        .brotli(true)                                        // Enable brotli
        .deflate(true)                                       // Enable deflate
        .redirect(reqwest::redirect::Policy::limited(10))   // Max redirects
        .pool_max_idle_per_host(10)                         // Connection pool
        .pool_idle_timeout(Duration::from_secs(90))         // Idle timeout
        .tcp_keepalive(Duration::from_secs(60))             // Keep-alive
        .build()?;
    
    Ok(())
}

// ===== HTTP METHODS =====
async fn http_methods(client: &Client) -> Result<(), Error> {
    // GET
    let response = client.get("https://api.example.com/resource").send().await?;
    
    // POST
    let response = client.post("https://api.example.com/resource")
        .body("request body")
        .send()
        .await?;
    
    // PUT
    let response = client.put("https://api.example.com/resource")
        .body("updated data")
        .send()
        .await?;
    
    // PATCH
    let response = client.patch("https://api.example.com/resource")
        .body("partial update")
        .send()
        .await?;
    
    // DELETE
    let response = client.delete("https://api.example.com/resource").send().await?;
    
    // HEAD
    let response = client.head("https://api.example.com/resource").send().await?;
    
    // Custom method
    let response = client.request(Method::OPTIONS, "https://api.example.com")
        .send()
        .await?;
    
    Ok(())
}

// ===== REQUEST HEADERS =====
async fn request_headers(client: &Client) -> Result<(), Error> {
    let response = client.get("https://api.example.com/data")
        .header("Authorization", "Bearer token123")          // Single header
        .header("X-Custom-Header", "value")
        .header(header::ACCEPT, "application/json")         // Typed header
        .header(header::CONTENT_TYPE, "application/json")
        .send()
        .await?;
    
    // Multiple headers with HeaderMap
    let mut headers = header::HeaderMap::new();
    headers.insert(header::AUTHORIZATION, "Bearer token".parse().unwrap());
    headers.insert(header::ACCEPT, "application/json".parse().unwrap());
    
    let response = client.get("https://api.example.com/data")
        .headers(headers)
        .send()
        .await?;
    
    Ok(())
}

// ===== RESPONSE HEADERS =====
async fn response_headers() -> Result<(), Error> {
    let response = reqwest::get("https://api.example.com/data").await?;
    
    // Access headers
    let headers = response.headers();
    
    // Get specific header
    if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        println!("Content-Type: {:?}", content_type);
    }
    
    // Iterate headers
    for (name, value) in headers.iter() {
        println!("{}: {:?}", name, value);
    }
    
    // Get header as string
    if let Some(value) = headers.get("X-Custom").and_then(|v| v.to_str().ok()) {
        println!("Custom header: {}", value);
    }
    
    Ok(())
}

// ===== QUERY PARAMETERS =====
async fn query_parameters(client: &Client) -> Result<(), Error> {
    // Using tuples
    let response = client.get("https://api.example.com/search")
        .query(&[("q", "rust"), ("page", "1")])
        .send()
        .await?;
    
    // Using HashMap
    let mut params = HashMap::new();
    params.insert("q", "rust");
    params.insert("limit", "10");
    
    let response = client.get("https://api.example.com/search")
        .query(&params)
        .send()
        .await?;
    
    // Using struct with serde
    #[derive(Serialize)]
    struct SearchQuery {
        q: String,
        page: u32,
        limit: u32,
    }
    
    let query = SearchQuery {
        q: "rust".to_string(),
        page: 1,
        limit: 10,
    };
    
    let response = client.get("https://api.example.com/search")
        .query(&query)
        .send()
        .await?;
    
    Ok(())
}

// ===== JSON REQUEST/RESPONSE =====
#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

async fn json_requests(client: &Client) -> Result<(), Error> {
    // Send JSON
    let new_user = User {
        id: 0,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    
    let response = client.post("https://api.example.com/users")
        .json(&new_user)                                     // Serialize to JSON
        .send()
        .await?;
    
    // Receive JSON
    let user: User = response.json().await?;                 // Deserialize from JSON
    println!("Created user: {:?}", user);
    
    Ok(())
}

// Get JSON directly
async fn get_json() -> Result<User, Error> {
    let user = reqwest::get("https://api.example.com/users/1")
        .await?
        .json::<User>()
        .await?;
    
    Ok(user)
}

// ===== FORM DATA =====
async fn form_data(client: &Client) -> Result<(), Error> {
    // URL-encoded form
    let params = [("username", "alice"), ("password", "secret")];
    
    let response = client.post("https://api.example.com/login")
        .form(&params)                                       // application/x-www-form-urlencoded
        .send()
        .await?;
    
    // Using HashMap
    let mut form = HashMap::new();
    form.insert("username", "alice");
    form.insert("password", "secret");
    
    let response = client.post("https://api.example.com/login")
        .form(&form)
        .send()
        .await?;
    
    Ok(())
}

// ===== MULTIPART FORM =====
use reqwest::multipart;

async fn multipart_form(client: &Client) -> Result<(), Error> {
    // Create multipart form
    let form = multipart::Form::new()
        .text("name", "Alice")                               // Text field
        .text("email", "alice@example.com")
        .file("avatar", "/path/to/avatar.png")              // File field
        .await?;
    
    let response = client.post("https://api.example.com/upload")
        .multipart(form)
        .send()
        .await?;
    
    Ok(())
}

// Upload file from bytes
async fn upload_bytes(client: &Client) -> Result<(), Error> {
    let file_data = std::fs::read("file.txt")?;
    
    let part = multipart::Part::bytes(file_data)
        .file_name("file.txt")
        .mime_str("text/plain")?;
    
    let form = multipart::Form::new()
        .part("file", part);
    
    let response = client.post("https://api.example.com/upload")
        .multipart(form)
        .send()
        .await?;
    
    Ok(())
}

// ===== RESPONSE HANDLING =====
async fn response_handling() -> Result<(), Error> {
    let response = reqwest::get("https://api.example.com/data").await?;
    
    // Get status code
    let status = response.status();
    println!("Status: {}", status);
    
    // Check status
    if status.is_success() {
        println!("Success!");
    } else if status.is_client_error() {
        println!("Client error: {}", status);
    } else if status.is_server_error() {
        println!("Server error: {}", status);
    }
    
    // Specific status codes
    match status {
        StatusCode::OK => println!("OK"),
        StatusCode::NOT_FOUND => println!("Not found"),
        StatusCode::UNAUTHORIZED => println!("Unauthorized"),
        _ => println!("Other status: {}", status),
    }
    
    // Error for status
    let response = response.error_for_status()?;            // Error if not 2xx
    
    // Get response as different types
    let text = response.text().await?;                       // As text
    // let json: MyType = response.json().await?;            // As JSON
    // let bytes = response.bytes().await?;                  // As bytes
    
    Ok(())
}

// ===== STREAMING RESPONSES =====
use futures_util::StreamExt;

async fn stream_response() -> Result<(), Error> {
    let response = reqwest::get("https://api.example.com/large-file").await?;
    
    let mut stream = response.bytes_stream();
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        println!("Received {} bytes", chunk.len());
        // Process chunk
    }
    
    Ok(())
}

// Download file with progress
async fn download_file() -> Result<(), Error> {
    let response = reqwest::get("https://example.com/file.zip").await?;
    let total_size = response.content_length().unwrap_or(0);
    
    let mut downloaded = 0u64;
    let mut stream = response.bytes_stream();
    let mut file = tokio::fs::File::create("output.zip").await?;
    
    use tokio::io::AsyncWriteExt;
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        
        let progress = (downloaded as f64 / total_size as f64) * 100.0;
        println!("Downloaded: {:.2}%", progress);
    }
    
    Ok(())
}

// ===== AUTHENTICATION =====
async fn basic_auth(client: &Client) -> Result<(), Error> {
    // Basic authentication
    let response = client.get("https://api.example.com/protected")
        .basic_auth("username", Some("password"))
        .send()
        .await?;
    
    Ok(())
}

async fn bearer_token(client: &Client) -> Result<(), Error> {
    // Bearer token
    let response = client.get("https://api.example.com/protected")
        .bearer_auth("your-token-here")
        .send()
        .await?;
    
    Ok(())
}

// ===== COOKIES =====
async fn cookies_example() -> Result<(), Error> {
    // Client with cookie store
    let client = Client::builder()
        .cookie_store(true)                                  // Enable cookie store
        .build()?;
    
    // First request sets cookies
    let response = client.get("https://api.example.com/login")
        .send()
        .await?;
    
    // Second request uses stored cookies
    let response = client.get("https://api.example.com/profile")
        .send()
        .await?;
    
    Ok(())
}

// Custom cookie jar
use reqwest::cookie::Jar;
use std::sync::Arc;

async fn custom_cookie_jar() -> Result<(), Error> {
    let jar = Arc::new(Jar::default());
    
    // Add cookies manually
    let url = "https://api.example.com".parse().unwrap();
    jar.add_cookie_str("session=abc123", &url);
    
    let client = Client::builder()
        .cookie_provider(Arc::clone(&jar))
        .build()?;
    
    let response = client.get("https://api.example.com/data")
        .send()
        .await?;
    
    Ok(())
}

// ===== PROXY =====
async fn proxy_example() -> Result<(), Error> {
    // HTTP proxy
    let proxy = reqwest::Proxy::http("http://proxy.example.com:8080")?;
    
    let client = Client::builder()
        .proxy(proxy)
        .build()?;
    
    // HTTPS proxy
    let proxy = reqwest::Proxy::https("https://proxy.example.com:8080")?;
    
    // All protocols
    let proxy = reqwest::Proxy::all("http://proxy.example.com:8080")?;
    
    // Proxy with authentication
    let proxy = reqwest::Proxy::http("http://proxy.example.com:8080")?
        .basic_auth("username", "password");
    
    let client = Client::builder()
        .proxy(proxy)
        .build()?;
    
    Ok(())
}

// ===== TIMEOUTS =====
async fn timeout_example() -> Result<(), Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))                    // Overall timeout
        .connect_timeout(Duration::from_secs(10))            // Connect timeout
        .build()?;
    
    // Per-request timeout
    let response = client.get("https://api.example.com/slow")
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    
    Ok(())
}

// ===== RETRIES =====
async fn retry_request() -> Result<(), Error> {
    let client = Client::new();
    let mut attempts = 0;
    let max_attempts = 3;
    
    loop {
        attempts += 1;
        
        match client.get("https://api.example.com/data").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    return Ok(());
                }
            }
            Err(e) if attempts < max_attempts => {
                println!("Attempt {} failed: {}", attempts, e);
                tokio::time::sleep(Duration::from_secs(2u64.pow(attempts))).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}

// ===== ERROR HANDLING =====
async fn error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let result = reqwest::get("https://api.example.com/data").await;
    
    match result {
        Ok(response) => {
            // Check status
            if !response.status().is_success() {
                return Err(format!("HTTP error: {}", response.status()).into());
            }
            
            let body = response.text().await?;
            println!("Body: {}", body);
        }
        Err(e) => {
            // Check error type
            if e.is_timeout() {
                println!("Request timed out");
            } else if e.is_connect() {
                println!("Connection error");
            } else if e.is_redirect() {
                println!("Redirect error");
            } else if e.is_status() {
                println!("Status error: {:?}", e.status());
            } else if e.is_request() {
                println!("Request error");
            }
            
            return Err(e.into());
        }
    }
    
    Ok(())
}

// ===== TLS/SSL CONFIGURATION =====
async fn tls_configuration() -> Result<(), Error> {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)                   // Accept invalid certs (DANGER!)
        .danger_accept_invalid_hostnames(true)               // Accept invalid hostnames (DANGER!)
        .min_tls_version(reqwest::tls::Version::TLS_1_2)    // Minimum TLS version
        .build()?;
    
    Ok(())
}

// Custom TLS with certificate
async fn custom_certificate() -> Result<(), Error> {
    let cert = std::fs::read("cert.pem")?;
    let cert = reqwest::Certificate::from_pem(&cert)?;
    
    let client = Client::builder()
        .add_root_certificate(cert)
        .build()?;
    
    Ok(())
}

// ===== COMMON PATTERNS =====

// Pattern 1: API client wrapper
struct ApiClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl ApiClient {
    fn new(base_url: String, api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
        
        ApiClient { client, base_url, api_key }
    }
    
    async fn get_user(&self, id: u32) -> Result<User, Error> {
        let url = format!("{}/users/{}", self.base_url, id);
        
        let user = self.client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(user)
    }
    
    async fn create_user(&self, user: &User) -> Result<User, Error> {
        let url = format!("{}/users", self.base_url);
        
        let created = self.client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(user)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(created)
    }
}

// Pattern 2: Pagination
async fn fetch_all_pages(client: &Client) -> Result<Vec<User>, Error> {
    let mut all_users = Vec::new();
    let mut page = 1;
    
    loop {
        let response: Vec<User> = client
            .get("https://api.example.com/users")
            .query(&[("page", page), ("limit", 100)])
            .send()
            .await?
            .json()
            .await?;
        
        if response.is_empty() {
            break;
        }
        
        all_users.extend(response);
        page += 1;
    }
    
    Ok(all_users)
}

// Pattern 3: Parallel requests
async fn parallel_requests(client: &Client, ids: Vec<u32>) -> Result<Vec<User>, Error> {
    let futures: Vec<_> = ids
        .into_iter()
        .map(|id| {
            let client = client.clone();
            async move {
                let url = format!("https://api.example.com/users/{}", id);
                client.get(&url).send().await?.json::<User>().await
            }
        })
        .collect();
    
    let results = futures::future::join_all(futures).await;
    
    let users: Result<Vec<_>, _> = results.into_iter().collect();
    users
}

// Pattern 4: Rate limiting
use std::time::Instant;

async fn rate_limited_requests(client: &Client, urls: Vec<String>) -> Result<(), Error> {
    let rate_limit = Duration::from_millis(100);             // 10 requests per second
    let mut last_request = Instant::now();
    
    for url in urls {
        let elapsed = last_request.elapsed();
        if elapsed < rate_limit {
            tokio::time::sleep(rate_limit - elapsed).await;
        }
        
        client.get(&url).send().await?;
        last_request = Instant::now();
    }
    
    Ok(())
}

// Pattern 5: Conditional requests (ETag)
async fn conditional_request(client: &Client, etag: Option<&str>) -> Result<(), Error> {
    let mut request = client.get("https://api.example.com/data");
    
    if let Some(etag) = etag {
        request = request.header(header::IF_NONE_MATCH, etag);
    }
    
    let response = request.send().await?;
    
    match response.status() {
        StatusCode::NOT_MODIFIED => {
            println!("Data not modified");
        }
        StatusCode::OK => {
            let new_etag = response.headers()
                .get(header::ETAG)
                .and_then(|v| v.to_str().ok());
            
            let body = response.text().await?;
            println!("New data, ETag: {:?}", new_etag);
        }
        _ => {}
    }
    
    Ok(())
}
```
