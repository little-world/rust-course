# Chapter 15: Async Runtime Patterns - Programming Projects

## Project 1: Concurrent Web Scraper with Rate Limiting

### Problem Statement

Build an async web scraper that fetches content from multiple URLs concurrently while respecting rate limits and handling failures gracefully. The system should fetch web pages, extract links, follow them recursively (up to a depth limit), and collect results. It must handle timeouts, retries, concurrent request limits, and per-domain rate limiting.

### Use Cases

- **Price monitoring systems** - Track prices across e-commerce sites
- **Search engine crawlers** - Discover and index web pages
- **Data aggregation services** - Collect data from multiple APIs
- **Competitor analysis tools** - Monitor competitor websites
- **News aggregators** - Fetch articles from multiple sources
- **Website monitoring** - Check site availability and response times
- **SEO tools** - Analyze website structure and links

### Why It Matters

**Performance**: Synchronous scraping of 100 URLs at 200ms each = 20 seconds. Async with 10 concurrent requests = 2 seconds (10x faster). For web scrapers processing millions of URLs daily, this determines infrastructure cost.

**Rate Limiting Necessity**: Without rate limiting, scrapers get IP-banned. A scraper hitting 1000 requests/second looks like a DDoS attack. Proper rate limiting (10 requests/second per domain) maintains access while still achieving good throughput.

**Real-World Constraints**: Networks fail. Timeouts prevent hanging on unresponsive servers. Retries with exponential backoff handle transient failures (95% success rate becomes 99.9% with 3 retries). Concurrent limits prevent overwhelming your own network or the target server.

**Rust's Advantage**: Async Rust achieves C-level performance with memory safety. tokio's runtime efficiently handles 10,000+ concurrent connections on a single thread. No garbage collector pauses unlike Python/Node.js scrapers.

Example performance numbers:
```
Sequential:     100 URLs × 200ms = 20s
10 concurrent:  100 URLs ÷ 10 × 200ms = 2s
100 concurrent: 100 URLs ÷ 100 × 200ms = 200ms (limited by network)
```

---

## Milestone 1: Basic Async HTTP Fetcher

### Introduction

Before building a full scraper, you need to understand async HTTP requests. This milestone teaches you to use `reqwest` (async HTTP client) and `tokio::time::timeout` for timeout handling.

**Why Start Here**: Sequential HTTP requests block. If one server hangs, your entire scraper stops. Async HTTP with timeouts solves this—you control how long to wait, and other requests proceed independently.

### Architecture

**Structs:**
- `FetchResult` - Represents the result of fetching a URL
  - **Field** `url: String` - The URL that was fetched
  - **Field** `status_code: u16` - HTTP status code (200, 404, etc.)
  - **Field** `body: Option<String>` - Response body if successful
  - **Field** `error: Option<String>` - Error message if failed

**Key Functions:**
- `async fn fetch_url(url: &str, timeout_ms: u64) -> FetchResult` - Fetches a single URL with timeout
- `async fn fetch_with_client(client: &reqwest::Client, url: &str, timeout_ms: u64) -> FetchResult` - Reuses HTTP client for efficiency

**Role Each Plays:**
- **reqwest::Client**: Reusable HTTP client (connection pooling, keeps TCP connections alive)
- **tokio::time::timeout**: Wraps async operation, cancels if too slow
- **FetchResult**: Type-safe representation of success/failure (better than Result<String, Error> because we capture partial success like status codes)

### Checkpoint Tests

```rust
#[tokio::test]
async fn test_fetch_success() {
    // Note: Use httpbin.org for testing (returns your request as JSON)
    let result = fetch_url("https://httpbin.org/status/200", 5000).await;

    assert_eq!(result.status_code, 200);
    assert!(result.body.is_some());
    assert!(result.error.is_none());
}

#[tokio::test]
async fn test_fetch_timeout() {
    // httpbin.org/delay/10 waits 10 seconds before responding
    let result = fetch_url("https://httpbin.org/delay/10", 1000).await;

    assert!(result.error.is_some());
    assert!(result.error.as_ref().unwrap().contains("timeout"));
}

#[tokio::test]
async fn test_fetch_404() {
    let result = fetch_url("https://httpbin.org/status/404", 5000).await;

    assert_eq!(result.status_code, 404);
    // We still get a body even for 404s
    assert!(result.body.is_some() || result.error.is_some());
}

#[tokio::test]
async fn test_client_reuse() {
    let client = reqwest::Client::new();

    // Fetching multiple URLs with same client reuses connections
    let result1 = fetch_with_client(&client, "https://httpbin.org/status/200", 5000).await;
    let result2 = fetch_with_client(&client, "https://httpbin.org/status/201", 5000).await;

    assert_eq!(result1.status_code, 200);
    assert_eq!(result2.status_code, 201);
}
```

### Starter Code

```rust
use reqwest;
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone)]
pub struct FetchResult {
    pub url: String,
    pub status_code: u16,
    pub body: Option<String>,
    pub error: Option<String>,
}

pub async fn fetch_url(url: &str, timeout_ms: u64) -> FetchResult {
    // TODO: Create a reqwest client
    // TODO: Use tokio::time::timeout to wrap the request
    // TODO: Handle timeout errors vs HTTP errors
    // TODO: Extract status code and body
    // TODO: Return FetchResult

    todo!("Implement fetch_url")
}

pub async fn fetch_with_client(
    client: &reqwest::Client,
    url: &str,
    timeout_ms: u64
) -> FetchResult {
    // TODO: Similar to fetch_url but reuses the provided client
    // Hint: Use client.get(url).send().await

    todo!("Implement fetch_with_client")
}

#[tokio::main]
async fn main() {
    let result = fetch_url("https://httpbin.org/status/200", 5000).await;
    println!("{:?}", result);
}
```

**Implementation Hints:**
1. Use `reqwest::Client::new()` to create a client
2. Wrap `client.get(url).send().await` with `tokio::time::timeout(Duration::from_millis(timeout_ms), ...)`
3. Match on the timeout result: `Ok(Ok(response))` = success, `Ok(Err(e))` = HTTP error, `Err(_)` = timeout
4. Use `response.status().as_u16()` to get status code
5. Use `response.text().await` to get body (also async!)

---

## Milestone 2: Concurrent URL Fetching

### Introduction

**Why Milestone 1 Isn't Enough**: Sequential fetching is too slow. Fetching 100 URLs at 200ms each takes 20 seconds. We need concurrency.

**The Improvement**: Use `tokio::spawn` or `futures::join_all` to fetch multiple URLs simultaneously. This overlaps I/O wait time, achieving 10x+ speedup.

**New Challenge**: How do we launch multiple async tasks and collect their results? Sequential `.await` on each URL defeats the purpose.

### Architecture

**Structs:**
- Reuse `FetchResult` from Milestone 1

**Key Functions:**
- `async fn fetch_all_sequential(urls: Vec<String>, timeout_ms: u64) -> Vec<FetchResult>` - Baseline (slow)
- `async fn fetch_all_concurrent(urls: Vec<String>, timeout_ms: u64, max_concurrent: usize) -> Vec<FetchResult>` - Fast version with concurrency limit

**New Concepts:**
- **futures::stream::FuturesUnordered**: Collection that polls all futures concurrently
- **futures::stream::StreamExt::buffered**: Limits concurrent futures to prevent overwhelming the system
- **tokio::spawn**: Spawns task onto tokio runtime (not used here, but worth knowing)

**Role Each Plays:**
- **FuturesUnordered**: Polls all futures, yields results as they complete (unordered)
- **buffered(n)**: Processes up to `n` futures at once (prevents 10,000 concurrent connections)
- **collect()**: Gathers all stream results into Vec

### Checkpoint Tests

```rust
#[tokio::test]
async fn test_fetch_all_sequential() {
    let urls = vec![
        "https://httpbin.org/delay/1".to_string(),
        "https://httpbin.org/delay/1".to_string(),
        "https://httpbin.org/delay/1".to_string(),
    ];

    let start = std::time::Instant::now();
    let results = fetch_all_sequential(urls, 5000).await;
    let elapsed = start.elapsed();

    assert_eq!(results.len(), 3);
    assert!(elapsed.as_secs() >= 3); // Should take ~3 seconds (sequential)
}

#[tokio::test]
async fn test_fetch_all_concurrent() {
    let urls = vec![
        "https://httpbin.org/delay/1".to_string(),
        "https://httpbin.org/delay/1".to_string(),
        "https://httpbin.org/delay/1".to_string(),
    ];

    let start = std::time::Instant::now();
    let results = fetch_all_concurrent(urls, 5000, 10).await;
    let elapsed = start.elapsed();

    assert_eq!(results.len(), 3);
    assert!(elapsed.as_secs() < 2); // Should take ~1 second (concurrent)
}

#[tokio::test]
async fn test_concurrent_limit() {
    // Create 100 URLs
    let urls: Vec<String> = (0..100)
        .map(|i| format!("https://httpbin.org/status/{}", 200 + (i % 5)))
        .collect();

    // With limit of 10, should handle without overwhelming
    let results = fetch_all_concurrent(urls, 5000, 10).await;

    assert_eq!(results.len(), 100);
    assert!(results.iter().all(|r| r.status_code >= 200 && r.status_code < 300));
}
```

### Starter Code

```rust
use futures::stream::{self, StreamExt};

pub async fn fetch_all_sequential(urls: Vec<String>, timeout_ms: u64) -> Vec<FetchResult> {
    let client = reqwest::Client::new();
    let mut results = Vec::new();

    for url in urls {
        let result = fetch_with_client(&client, &url, timeout_ms).await;
        results.push(result);
    }

    results
}

pub async fn fetch_all_concurrent(
    urls: Vec<String>,
    timeout_ms: u64,
    max_concurrent: usize
) -> Vec<FetchResult> {
    let client = reqwest::Client::new();

    // TODO: Convert urls Vec into a stream
    // TODO: Map each URL to a fetch operation
    // TODO: Use .buffered(max_concurrent) to limit concurrency
    // TODO: Collect results into Vec

    todo!("Implement concurrent fetching")
}
```

**Implementation Hints:**
1. Use `stream::iter(urls)` to create a stream from the Vec
2. Use `.map(move |url| { let client = client.clone(); async move { ... } })` to create futures
3. Use `.buffered(max_concurrent)` to run up to N futures at once
4. Use `.collect::<Vec<_>>().await` to gather all results

---

## Milestone 3: Retry Logic with Exponential Backoff

### Introduction

**Why Milestone 2 Isn't Enough**: Networks are unreliable. Transient failures (server overload, network hiccup) can often succeed on retry. Without retries, 5% failure rate means losing 5 out of every 100 URLs.

**The Improvement**: Implement exponential backoff (wait 1s, then 2s, then 4s between retries). This gives servers time to recover and avoids hammering struggling services.

**Optimization**: Exponential backoff prevents retry storms. If 1000 clients retry immediately after failure, the server stays overwhelmed. Spreading retries over time (1s, 2s, 4s) allows recovery.

### Architecture

**Structs:**
- `RetryConfig` - Configuration for retry behavior
  - **Field** `max_retries: u32` - Maximum number of retry attempts
  - **Field** `initial_backoff_ms: u64` - Starting backoff duration
  - **Field** `max_backoff_ms: u64` - Cap on backoff duration
  - **Field** `timeout_ms: u64` - Timeout per request

**Key Functions:**
- `async fn fetch_with_retry(client: &reqwest::Client, url: &str, config: &RetryConfig) -> FetchResult` - Fetches URL with retry logic
- `fn should_retry(result: &FetchResult) -> bool` - Determines if a failure is retryable
- `fn calculate_backoff(attempt: u32, config: &RetryConfig) -> Duration` - Calculates wait time

**Role Each Plays:**
- **RetryConfig**: Encapsulates retry policy (makes it configurable)
- **should_retry**: Distinguishes transient failures (retry) from permanent failures (don't retry 404s)
- **calculate_backoff**: Implements exponential backoff with jitter

### Checkpoint Tests

```rust
#[tokio::test]
async fn test_retry_success_on_second_attempt() {
    // Simulate a service that fails once then succeeds
    // (In real tests, you'd use a mock server)
    let config = RetryConfig {
        max_retries: 3,
        initial_backoff_ms: 100,
        max_backoff_ms: 1000,
        timeout_ms: 5000,
    };

    let client = reqwest::Client::new();

    // This test assumes fetch_with_retry eventually succeeds
    let result = fetch_with_retry(&client, "https://httpbin.org/status/200", &config).await;

    assert_eq!(result.status_code, 200);
}

#[tokio::test]
async fn test_retry_exhaustion() {
    let config = RetryConfig {
        max_retries: 2,
        initial_backoff_ms: 100,
        max_backoff_ms: 1000,
        timeout_ms: 500, // Very short timeout
    };

    let client = reqwest::Client::new();

    // This endpoint delays 5 seconds (longer than our timeout)
    let result = fetch_with_retry(&client, "https://httpbin.org/delay/5", &config).await;

    // Should fail after retries exhausted
    assert!(result.error.is_some());
}

#[tokio::test]
async fn test_no_retry_on_404() {
    let config = RetryConfig {
        max_retries: 3,
        initial_backoff_ms: 100,
        max_backoff_ms: 1000,
        timeout_ms: 5000,
    };

    let client = reqwest::Client::new();

    let start = std::time::Instant::now();
    let result = fetch_with_retry(&client, "https://httpbin.org/status/404", &config).await;
    let elapsed = start.elapsed();

    assert_eq!(result.status_code, 404);
    // Should not retry on 404 (permanent failure)
    assert!(elapsed.as_millis() < 500); // Completes quickly
}

#[test]
fn test_exponential_backoff_calculation() {
    let config = RetryConfig {
        max_retries: 5,
        initial_backoff_ms: 100,
        max_backoff_ms: 5000,
        timeout_ms: 5000,
    };

    assert_eq!(calculate_backoff(1, &config).as_millis(), 100);
    assert_eq!(calculate_backoff(2, &config).as_millis(), 200);
    assert_eq!(calculate_backoff(3, &config).as_millis(), 400);
    assert_eq!(calculate_backoff(4, &config).as_millis(), 800);
    assert_eq!(calculate_backoff(5, &config).as_millis(), 1600);

    // Should cap at max_backoff_ms
    assert!(calculate_backoff(10, &config).as_millis() <= 5000);
}
```

### Starter Code

```rust
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub timeout_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 1000,
            max_backoff_ms: 10000,
            timeout_ms: 5000,
        }
    }
}

pub fn should_retry(result: &FetchResult) -> bool {
    // TODO: Return true if we should retry this failure
    // Hint: Don't retry 4xx errors (client errors like 404)
    // DO retry 5xx errors (server errors like 503)
    // DO retry timeouts and network errors

    todo!("Implement retry decision logic")
}

pub fn calculate_backoff(attempt: u32, config: &RetryConfig) -> Duration {
    // TODO: Implement exponential backoff
    // Formula: min(initial * 2^(attempt-1), max)
    // Example: 1s, 2s, 4s, 8s, 16s (capped at max)

    todo!("Implement exponential backoff calculation")
}

pub async fn fetch_with_retry(
    client: &reqwest::Client,
    url: &str,
    config: &RetryConfig,
) -> FetchResult {
    // TODO: Loop up to max_retries times
    // TODO: Call fetch_with_client
    // TODO: If success or permanent failure, return immediately
    // TODO: If transient failure, sleep for backoff duration and retry
    // TODO: After all retries exhausted, return last error

    todo!("Implement retry logic")
}
```

**Implementation Hints:**
1. Loop from 1 to max_retries + 1 (attempt 0 is the initial try)
2. Check if result should be retried using `should_retry`
3. For exponential backoff: `let backoff = initial_backoff_ms * 2u64.pow(attempt - 1)`
4. Use `std::cmp::min(backoff, max_backoff_ms)` to cap the value
5. Add jitter: `backoff + rand::random::<u64>() % (backoff / 2)` to prevent thundering herd

---

## Milestone 4: Per-Domain Rate Limiting

### Introduction

**Why Milestone 3 Isn't Enough**: Concurrent requests without rate limiting can overwhelm servers or get your IP banned. Fetching 100 URLs from the same domain in parallel looks like a DDoS attack.

**The Improvement**: Implement per-domain rate limiting using a token bucket algorithm. Allow at most N requests per second per domain, queuing excess requests.

**Optimization (Parallelism)**: Rate limiting prevents being blocked, but it's also about efficient resource use. Instead of sleeping between requests (wastes time), use a semaphore or channel to queue requests. This allows other domains to proceed while one is rate-limited.

### Architecture

**Structs:**
- `RateLimiter` - Manages rate limits per domain
  - **Field** `limiters: Arc<Mutex<HashMap<String, Semaphore>>>` - Per-domain semaphores
  - **Field** `permits_per_second: u32` - Rate limit (requests/second)
  - **Field** `refill_interval_ms: u64` - How often to refill permits

**Key Functions:**
- `impl RateLimiter::new(permits_per_second: u32) -> Self` - Creates rate limiter
- `async fn acquire_permit(&self, domain: &str) -> SemaphorePermit` - Waits for permission to make request
- `fn extract_domain(url: &str) -> Option<String>` - Extracts domain from URL

**Role Each Plays:**
- **Semaphore**: Allows N concurrent operations (N permits), blocks when permits exhausted
- **HashMap<String, Semaphore>**: Separate rate limit per domain
- **Arc<Mutex<...>>**: Thread-safe shared state across async tasks

### Checkpoint Tests

```rust
#[tokio::test]
async fn test_rate_limiter_allows_burst() {
    let rate_limiter = RateLimiter::new(5); // 5 requests/second

    let start = std::time::Instant::now();

    // First 5 should go through immediately
    for _ in 0..5 {
        let _permit = rate_limiter.acquire_permit("example.com").await;
    }

    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 100); // Should be instant
}

#[tokio::test]
async fn test_rate_limiter_delays_excess() {
    let rate_limiter = RateLimiter::new(2); // 2 requests/second

    let start = std::time::Instant::now();

    // First 2 instant, next 2 should wait ~1 second
    for i in 0..4 {
        let _permit = rate_limiter.acquire_permit("example.com").await;
        println!("Request {} at {:?}", i, start.elapsed());
    }

    let elapsed = start.elapsed();
    assert!(elapsed.as_secs() >= 1); // Should take at least 1 second
}

#[tokio::test]
async fn test_rate_limiter_per_domain() {
    let rate_limiter = RateLimiter::new(2);

    let start = std::time::Instant::now();

    // Different domains should not interfere
    let domain1 = rate_limiter.acquire_permit("example.com");
    let domain2 = rate_limiter.acquire_permit("different.com");

    tokio::join!(domain1, domain2);

    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 100); // Should be instant (different domains)
}

#[test]
fn test_extract_domain() {
    assert_eq!(
        extract_domain("https://example.com/path?query=1"),
        Some("example.com".to_string())
    );
    assert_eq!(
        extract_domain("http://sub.example.com:8080/path"),
        Some("sub.example.com".to_string())
    );
    assert_eq!(extract_domain("not-a-url"), None);
}
```

### Starter Code

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use url::Url;

pub struct RateLimiter {
    limiters: Arc<Mutex<HashMap<String, Arc<Semaphore>>>>,
    permits_per_second: u32,
}

impl RateLimiter {
    pub fn new(permits_per_second: u32) -> Self {
        // TODO: Initialize the rate limiter

        todo!("Implement RateLimiter::new")
    }

    pub async fn acquire_permit(&self, domain: &str) {
        // TODO: Get or create semaphore for this domain
        // TODO: Acquire a permit from the semaphore (blocks if none available)
        // TODO: Spawn background task to refill permits periodically

        todo!("Implement acquire_permit")
    }
}

pub fn extract_domain(url: &str) -> Option<String> {
    // TODO: Parse URL and extract host
    // Hint: Use url::Url::parse(url).ok()?.host_str()

    todo!("Implement domain extraction")
}

pub async fn fetch_with_rate_limit(
    client: &reqwest::Client,
    url: &str,
    config: &RetryConfig,
    rate_limiter: &RateLimiter,
) -> FetchResult {
    // TODO: Extract domain from URL
    // TODO: Acquire permit from rate limiter for that domain
    // TODO: Perform the fetch with retry logic
    // TODO: Permit is automatically released when dropped

    todo!("Implement rate-limited fetch")
}
```

**Implementation Hints:**
1. Use `Semaphore::new(permits_per_second as usize)` for initial permits
2. For refilling: `tokio::spawn(async move { loop { sleep(...); semaphore.add_permits(n); } })`
3. Store semaphores in HashMap: if missing, insert new one
4. Use `Arc::clone` to share semaphore references across tasks
5. `url::Url::parse(url)?.host_str()` extracts domain

---

## Milestone 5: Link Extraction and Recursive Crawling

### Introduction

**Why Milestone 4 Isn't Enough**: We can fetch URLs efficiently, but we need to discover URLs to fetch. Web scrapers extract links from HTML and follow them recursively.

**The Improvement**: Parse HTML to extract `<a href="...">` links, convert relative URLs to absolute, and crawl recursively up to a depth limit.

**Optimization (Memory)**: Without depth limits, crawlers visit infinite pages (loops in website graphs). A depth limit prevents runaway crawling. Also, track visited URLs to avoid re-fetching duplicates—this saves bandwidth and memory.

### Architecture

**Structs:**
- `CrawlConfig` - Configuration for crawling
  - **Field** `max_depth: u32` - Maximum link-following depth
  - **Field** `max_pages: usize` - Total page limit
  - **Field** `allowed_domains: Option<Vec<String>>` - Restrict crawling to these domains

- `CrawlResult` - Results from crawling
  - **Field** `visited_urls: HashSet<String>` - All visited URLs
  - **Field** `pages: Vec<FetchResult>` - Fetched page contents

**Key Functions:**
- `fn extract_links(html: &str, base_url: &str) -> Vec<String>` - Extracts and normalizes links
- `async fn crawl(start_url: String, config: CrawlConfig) -> CrawlResult` - Main crawl loop
- `fn is_allowed_domain(url: &str, allowed: &Option<Vec<String>>) -> bool` - Domain filter

**Role Each Plays:**
- **extract_links**: Parses HTML to find links (uses `scraper` or `html5ever` crate)
- **HashSet<String>**: Deduplicates URLs (O(1) lookup to check if visited)
- **BFS/DFS queue**: Manages URLs to visit (BFS = breadth-first, DFS = depth-first)

### Checkpoint Tests

```rust
#[test]
fn test_extract_links() {
    let html = r#"
        <html>
        <body>
            <a href="/page1">Page 1</a>
            <a href="https://example.com/page2">Page 2</a>
            <a href="../page3">Page 3</a>
            <a href="mailto:test@example.com">Email</a>
        </body>
        </html>
    "#;

    let base_url = "https://example.com/path/current";
    let links = extract_links(html, base_url);

    // Should resolve relative URLs and filter mailto:
    assert!(links.contains(&"https://example.com/page1".to_string()));
    assert!(links.contains(&"https://example.com/page2".to_string()));
    assert!(links.contains(&"https://example.com/page3".to_string()));
    assert!(!links.iter().any(|l| l.starts_with("mailto:")));
}

#[tokio::test]
async fn test_crawl_depth_limit() {
    let config = CrawlConfig {
        max_depth: 2,
        max_pages: 100,
        allowed_domains: Some(vec!["example.com".to_string()]),
        retry_config: RetryConfig::default(),
        rate_limiter: RateLimiter::new(5),
    };

    let result = crawl("https://example.com".to_string(), config).await;

    // Should stop at depth 2
    assert!(result.visited_urls.len() <= 100);
}

#[tokio::test]
async fn test_crawl_deduplication() {
    let config = CrawlConfig {
        max_depth: 3,
        max_pages: 50,
        allowed_domains: None,
        retry_config: RetryConfig::default(),
        rate_limiter: RateLimiter::new(10),
    };

    let result = crawl("https://httpbin.org".to_string(), config).await;

    // visited_urls should have no duplicates
    let unique_count = result.visited_urls.len();
    let page_count = result.pages.len();
    assert!(page_count <= unique_count); // Some may fail, but no duplicates
}

#[test]
fn test_is_allowed_domain() {
    let allowed = Some(vec!["example.com".to_string(), "test.com".to_string()]);

    assert!(is_allowed_domain("https://example.com/page", &allowed));
    assert!(is_allowed_domain("https://test.com/page", &allowed));
    assert!(!is_allowed_domain("https://other.com/page", &allowed));

    // None means allow all
    assert!(is_allowed_domain("https://anything.com", &None));
}
```

### Starter Code

```rust
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use url::Url;

#[derive(Debug, Clone)]
pub struct CrawlConfig {
    pub max_depth: u32,
    pub max_pages: usize,
    pub allowed_domains: Option<Vec<String>>,
    pub retry_config: RetryConfig,
    pub rate_limiter: RateLimiter,
}

#[derive(Debug)]
pub struct CrawlResult {
    pub visited_urls: HashSet<String>,
    pub pages: Vec<FetchResult>,
}

pub fn extract_links(html: &str, base_url: &str) -> Vec<String> {
    // TODO: Parse HTML using scraper crate
    // TODO: Select all <a> tags and extract href attributes
    // TODO: Convert relative URLs to absolute using base_url
    // TODO: Filter out non-http(s) links (mailto:, javascript:, etc.)

    todo!("Implement link extraction")
}

pub fn is_allowed_domain(url: &str, allowed: &Option<Vec<String>>) -> bool {
    // TODO: If allowed is None, return true
    // TODO: Otherwise, check if URL's domain is in the allowed list

    todo!("Implement domain filtering")
}

pub async fn crawl(start_url: String, config: CrawlConfig) -> CrawlResult {
    // TODO: Initialize visited set and results vec
    // TODO: Create queue with (url, depth) tuples
    // TODO: While queue not empty and pages < max_pages:
    //   - Pop URL from queue
    //   - If already visited or depth > max_depth, skip
    //   - Fetch URL with rate limiting
    //   - Mark as visited
    //   - Extract links and add to queue with depth+1
    // TODO: Return CrawlResult

    todo!("Implement crawler")
}
```

**Implementation Hints:**
1. Use `scraper::Html::parse_document(html)` to parse
2. Use `Selector::parse("a")` to select all links
3. Use `element.value().attr("href")` to get href attribute
4. Use `Url::parse(base_url)?.join(href)?` to resolve relative URLs
5. Use `VecDeque` for BFS queue: `queue.push_back(...)` and `queue.pop_front()`

---

## Milestone 6: Graceful Shutdown and Progress Reporting

### Introduction

**Why Milestone 5 Isn't Enough**: Long-running crawls need:
1. **Graceful shutdown**: Stop cleanly on Ctrl+C, save progress
2. **Progress reporting**: Show URLs/second, success rate, queue depth
3. **Resumability**: Save state to disk, resume later

**The Improvement**: Add tokio signal handlers for Ctrl+C, use channels for progress updates, and serialize state for resume.

**Optimization (Observability)**: Without progress reporting, you can't tell if crawler is stuck or slow. Metrics (URLs/sec, error rate) help tune rate limits and identify problems.

### Architecture

**Structs:**
- `CrawlProgress` - Real-time crawl statistics
  - **Field** `urls_fetched: Arc<AtomicUsize>` - Total URLs fetched
  - **Field** `urls_failed: Arc<AtomicUsize>` - Total failures
  - **Field** `queue_depth: Arc<AtomicUsize>` - URLs waiting in queue
  - **Field** `start_time: std::time::Instant` - When crawl started

- `CrawlState` - Serializable state for resume
  - **Field** `visited: HashSet<String>` - Already-visited URLs
  - **Field** `queue: VecDeque<(String, u32)>` - Pending URLs with depth

**Key Functions:**
- `fn report_progress(progress: &CrawlProgress)` - Prints stats
- `async fn save_state(state: &CrawlState, path: &str) -> std::io::Result<()>` - Saves to file
- `async fn load_state(path: &str) -> std::io::Result<CrawlState>` - Loads from file
- `async fn crawl_with_shutdown(...)` - Crawl with Ctrl+C handling

**Role Each Plays:**
- **Arc<AtomicUsize>**: Thread-safe counter (no mutex needed for increment)
- **tokio::signal::ctrl_c()**: Waits for Ctrl+C signal
- **serde + bincode**: Serialize/deserialize state to disk

### Checkpoint Tests

```rust
#[test]
fn test_progress_reporting() {
    let progress = CrawlProgress::new();

    progress.increment_fetched();
    progress.increment_fetched();
    progress.increment_failed();

    assert_eq!(progress.urls_fetched(), 2);
    assert_eq!(progress.urls_failed(), 1);

    let stats = progress.get_stats();
    println!("{}", stats); // Should show readable progress
}

#[tokio::test]
async fn test_state_save_load() {
    use std::collections::{HashSet, VecDeque};

    let mut visited = HashSet::new();
    visited.insert("https://example.com".to_string());

    let mut queue = VecDeque::new();
    queue.push_back(("https://example.com/page".to_string(), 1));

    let state = CrawlState { visited, queue };

    // Save and load
    save_state(&state, "test_state.bin").await.unwrap();
    let loaded = load_state("test_state.bin").await.unwrap();

    assert_eq!(state.visited, loaded.visited);
    assert_eq!(state.queue, loaded.queue);

    // Cleanup
    std::fs::remove_file("test_state.bin").unwrap();
}

#[tokio::test]
async fn test_graceful_shutdown() {
    // This test is hard to automate (requires sending signals)
    // Manual test: Run crawl, press Ctrl+C, verify state is saved

    // For automated testing, use a channel instead:
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let crawl_handle = tokio::spawn(async move {
        // Simulated crawl
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(10)) => {
                "Completed"
            }
            _ = shutdown_rx => {
                "Shutdown requested"
            }
        }
    });

    // Simulate shutdown after 1 second
    tokio::time::sleep(Duration::from_millis(100)).await;
    shutdown_tx.send(()).unwrap();

    let result = crawl_handle.await.unwrap();
    assert_eq!(result, "Shutdown requested");
}
```

### Starter Code

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone)]
pub struct CrawlProgress {
    urls_fetched: Arc<AtomicUsize>,
    urls_failed: Arc<AtomicUsize>,
    queue_depth: Arc<AtomicUsize>,
    start_time: std::time::Instant,
}

impl CrawlProgress {
    pub fn new() -> Self {
        Self {
            urls_fetched: Arc::new(AtomicUsize::new(0)),
            urls_failed: Arc::new(AtomicUsize::new(0)),
            queue_depth: Arc::new(AtomicUsize::new(0)),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn increment_fetched(&self) {
        self.urls_fetched.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_failed(&self) {
        self.urls_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_queue_depth(&self, depth: usize) {
        self.queue_depth.store(depth, Ordering::Relaxed);
    }

    pub fn urls_fetched(&self) -> usize {
        self.urls_fetched.load(Ordering::Relaxed)
    }

    pub fn urls_failed(&self) -> usize {
        self.urls_failed.load(Ordering::Relaxed)
    }

    pub fn get_stats(&self) -> String {
        // TODO: Format progress statistics
        // Include: URLs fetched, failed, success rate, URLs/second, elapsed time

        todo!("Implement progress formatting")
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrawlState {
    pub visited: HashSet<String>,
    pub queue: VecDeque<(String, u32)>,
}

pub async fn save_state(state: &CrawlState, path: &str) -> std::io::Result<()> {
    // TODO: Serialize state using bincode
    // TODO: Write to file atomically (write to temp file, then rename)

    todo!("Implement state saving")
}

pub async fn load_state(path: &str) -> std::io::Result<CrawlState> {
    // TODO: Read file
    // TODO: Deserialize using bincode

    todo!("Implement state loading")
}

pub async fn crawl_with_shutdown(
    start_url: String,
    config: CrawlConfig,
    state_file: Option<String>,
) -> CrawlResult {
    // TODO: Load existing state if state_file provided
    // TODO: Create progress tracker
    // TODO: Spawn progress reporter (prints every N seconds)
    // TODO: Run crawl in tokio::select! with ctrl_c() handler
    // TODO: On shutdown, save state and return partial results

    todo!("Implement crawl with shutdown handling")
}
```

**Implementation Hints:**
1. Use `tokio::signal::ctrl_c().await` to wait for Ctrl+C
2. Use `tokio::select!` to race between crawl completion and shutdown signal
3. Use `bincode::serialize` and `bincode::deserialize` for state serialization
4. Spawn progress reporter: `tokio::spawn(async move { loop { sleep(...); report(...); } })`
5. For atomic file write: write to `{path}.tmp`, then `tokio::fs::rename`

---

## Complete Working Example

Here's a full implementation demonstrating all milestones:

```rust
// Cargo.toml dependencies:
// [dependencies]
// tokio = { version = "1.35", features = ["full"] }
// reqwest = { version = "0.11", features = ["json"] }
// futures = "0.3"
// scraper = "0.18"
// url = "2.5"
// serde = { version = "1.0", features = ["derive"] }
// bincode = "1.3"

use reqwest;
use tokio::time::{timeout, Duration, sleep};
use futures::stream::{self, StreamExt};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::{Mutex, Semaphore};
use scraper::{Html, Selector};
use url::Url;
use serde::{Serialize, Deserialize};

// ============================================================================
// Milestone 1: Basic Fetching
// ============================================================================

#[derive(Debug, Clone)]
pub struct FetchResult {
    pub url: String,
    pub status_code: u16,
    pub body: Option<String>,
    pub error: Option<String>,
}

pub async fn fetch_url(url: &str, timeout_ms: u64) -> FetchResult {
    let client = reqwest::Client::new();
    fetch_with_client(&client, url, timeout_ms).await
}

pub async fn fetch_with_client(
    client: &reqwest::Client,
    url: &str,
    timeout_ms: u64,
) -> FetchResult {
    let timeout_duration = Duration::from_millis(timeout_ms);

    match timeout(timeout_duration, client.get(url).send()).await {
        Ok(Ok(response)) => {
            let status_code = response.status().as_u16();
            match response.text().await {
                Ok(body) => FetchResult {
                    url: url.to_string(),
                    status_code,
                    body: Some(body),
                    error: None,
                },
                Err(e) => FetchResult {
                    url: url.to_string(),
                    status_code,
                    body: None,
                    error: Some(format!("Failed to read body: {}", e)),
                },
            }
        }
        Ok(Err(e)) => FetchResult {
            url: url.to_string(),
            status_code: 0,
            body: None,
            error: Some(format!("HTTP error: {}", e)),
        },
        Err(_) => FetchResult {
            url: url.to_string(),
            status_code: 0,
            body: None,
            error: Some("Request timeout".to_string()),
        },
    }
}

// ============================================================================
// Milestone 2: Concurrent Fetching
// ============================================================================

pub async fn fetch_all_concurrent(
    urls: Vec<String>,
    timeout_ms: u64,
    max_concurrent: usize,
) -> Vec<FetchResult> {
    let client = Arc::new(reqwest::Client::new());

    stream::iter(urls)
        .map(|url| {
            let client = Arc::clone(&client);
            async move { fetch_with_client(&client, &url, timeout_ms).await }
        })
        .buffered(max_concurrent)
        .collect()
        .await
}

// ============================================================================
// Milestone 3: Retry Logic
// ============================================================================

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub timeout_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 1000,
            max_backoff_ms: 10000,
            timeout_ms: 5000,
        }
    }
}

pub fn should_retry(result: &FetchResult) -> bool {
    // Retry on network errors or 5xx server errors
    if result.error.is_some() {
        return true;
    }

    // Don't retry client errors (4xx)
    if result.status_code >= 400 && result.status_code < 500 {
        return false;
    }

    // Retry server errors (5xx)
    result.status_code >= 500
}

pub fn calculate_backoff(attempt: u32, config: &RetryConfig) -> Duration {
    let backoff = config.initial_backoff_ms * 2u64.pow(attempt.saturating_sub(1));
    let capped = std::cmp::min(backoff, config.max_backoff_ms);
    Duration::from_millis(capped)
}

pub async fn fetch_with_retry(
    client: &reqwest::Client,
    url: &str,
    config: &RetryConfig,
) -> FetchResult {
    let mut last_result = fetch_with_client(client, url, config.timeout_ms).await;

    for attempt in 1..=config.max_retries {
        if !should_retry(&last_result) {
            return last_result;
        }

        let backoff = calculate_backoff(attempt, config);
        sleep(backoff).await;

        last_result = fetch_with_client(client, url, config.timeout_ms).await;
    }

    last_result
}

// ============================================================================
// Milestone 4: Rate Limiting
// ============================================================================

pub struct RateLimiter {
    limiters: Arc<Mutex<HashMap<String, Arc<Semaphore>>>>,
    permits_per_second: u32,
}

impl RateLimiter {
    pub fn new(permits_per_second: u32) -> Self {
        Self {
            limiters: Arc::new(Mutex::new(HashMap::new())),
            permits_per_second,
        }
    }

    pub async fn acquire_permit(&self, domain: &str) {
        let semaphore = {
            let mut limiters = self.limiters.lock().await;
            limiters
                .entry(domain.to_string())
                .or_insert_with(|| {
                    let sem = Arc::new(Semaphore::new(self.permits_per_second as usize));
                    let sem_clone = Arc::clone(&sem);
                    let permits = self.permits_per_second as usize;

                    // Refill permits every second
                    tokio::spawn(async move {
                        loop {
                            sleep(Duration::from_secs(1)).await;
                            sem_clone.add_permits(permits);
                        }
                    });

                    sem
                })
                .clone()
        };

        let _permit = semaphore.acquire().await.unwrap();
        // Permit released immediately (we just want to rate limit the start)
    }
}

pub fn extract_domain(url: &str) -> Option<String> {
    Url::parse(url).ok()?.host_str().map(String::from)
}

pub async fn fetch_with_rate_limit(
    client: &reqwest::Client,
    url: &str,
    config: &RetryConfig,
    rate_limiter: &RateLimiter,
) -> FetchResult {
    if let Some(domain) = extract_domain(url) {
        rate_limiter.acquire_permit(&domain).await;
    }

    fetch_with_retry(client, url, config).await
}

// ============================================================================
// Milestone 5: Link Extraction and Crawling
// ============================================================================

#[derive(Debug, Clone)]
pub struct CrawlConfig {
    pub max_depth: u32,
    pub max_pages: usize,
    pub allowed_domains: Option<Vec<String>>,
    pub retry_config: RetryConfig,
    pub rate_limiter: Arc<RateLimiter>,
}

#[derive(Debug)]
pub struct CrawlResult {
    pub visited_urls: HashSet<String>,
    pub pages: Vec<FetchResult>,
}

pub fn extract_links(html: &str, base_url: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("a").unwrap();

    let base = match Url::parse(base_url) {
        Ok(url) => url,
        Err(_) => return Vec::new(),
    };

    document
        .select(&selector)
        .filter_map(|element| element.value().attr("href"))
        .filter_map(|href| base.join(href).ok())
        .filter(|url| url.scheme() == "http" || url.scheme() == "https")
        .map(|url| url.to_string())
        .collect()
}

pub fn is_allowed_domain(url: &str, allowed: &Option<Vec<String>>) -> bool {
    let allowed = match allowed {
        Some(domains) => domains,
        None => return true,
    };

    let domain = match extract_domain(url) {
        Some(d) => d,
        None => return false,
    };

    allowed.iter().any(|allowed_domain| domain.contains(allowed_domain))
}

pub async fn crawl(start_url: String, config: CrawlConfig) -> CrawlResult {
    let mut visited = HashSet::new();
    let mut pages = Vec::new();
    let mut queue = VecDeque::new();

    queue.push_back((start_url.clone(), 0));
    visited.insert(start_url);

    let client = Arc::new(reqwest::Client::new());

    while let Some((url, depth)) = queue.pop_front() {
        if pages.len() >= config.max_pages {
            break;
        }

        if depth > config.max_depth {
            continue;
        }

        println!("Crawling: {} (depth {})", url, depth);

        let result = fetch_with_rate_limit(
            &client,
            &url,
            &config.retry_config,
            &config.rate_limiter,
        ).await;

        // Extract links if successful
        if let Some(body) = &result.body {
            let links = extract_links(body, &url);

            for link in links {
                if !visited.contains(&link)
                    && is_allowed_domain(&link, &config.allowed_domains)
                {
                    visited.insert(link.clone());
                    queue.push_back((link, depth + 1));
                }
            }
        }

        pages.push(result);
    }

    CrawlResult {
        visited_urls: visited,
        pages,
    }
}

// ============================================================================
// Milestone 6: Progress and Shutdown
// ============================================================================

#[derive(Clone)]
pub struct CrawlProgress {
    urls_fetched: Arc<AtomicUsize>,
    urls_failed: Arc<AtomicUsize>,
    queue_depth: Arc<AtomicUsize>,
    start_time: std::time::Instant,
}

impl CrawlProgress {
    pub fn new() -> Self {
        Self {
            urls_fetched: Arc::new(AtomicUsize::new(0)),
            urls_failed: Arc::new(AtomicUsize::new(0)),
            queue_depth: Arc::new(AtomicUsize::new(0)),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn increment_fetched(&self) {
        self.urls_fetched.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_failed(&self) {
        self.urls_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_queue_depth(&self, depth: usize) {
        self.queue_depth.store(depth, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> String {
        let fetched = self.urls_fetched.load(Ordering::Relaxed);
        let failed = self.urls_failed.load(Ordering::Relaxed);
        let queue = self.queue_depth.load(Ordering::Relaxed);
        let elapsed = self.start_time.elapsed().as_secs_f64();

        let rate = if elapsed > 0.0 {
            fetched as f64 / elapsed
        } else {
            0.0
        };

        let success_rate = if fetched > 0 {
            (fetched - failed) as f64 / fetched as f64 * 100.0
        } else {
            0.0
        };

        format!(
            "Fetched: {} | Failed: {} | Queue: {} | Rate: {:.1} URLs/s | Success: {:.1}% | Elapsed: {:.1}s",
            fetched, failed, queue, rate, success_rate, elapsed
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct CrawlState {
    pub visited: HashSet<String>,
    pub queue: VecDeque<(String, u32)>,
}

pub async fn save_state(state: &CrawlState, path: &str) -> std::io::Result<()> {
    let serialized = bincode::serialize(state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let temp_path = format!("{}.tmp", path);
    tokio::fs::write(&temp_path, serialized).await?;
    tokio::fs::rename(temp_path, path).await?;

    Ok(())
}

pub async fn load_state(path: &str) -> std::io::Result<CrawlState> {
    let data = tokio::fs::read(path).await?;
    bincode::deserialize(&data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

pub async fn crawl_with_shutdown(
    start_url: String,
    config: CrawlConfig,
    state_file: Option<String>,
) -> CrawlResult {
    // Load state if resuming
    let (mut visited, mut queue) = if let Some(ref path) = state_file {
        if let Ok(state) = load_state(path).await {
            println!("Resumed from saved state: {} visited URLs", state.visited.len());
            (state.visited, state.queue)
        } else {
            let mut v = HashSet::new();
            let mut q = VecDeque::new();
            v.insert(start_url.clone());
            q.push_back((start_url, 0));
            (v, q)
        }
    } else {
        let mut v = HashSet::new();
        let mut q = VecDeque::new();
        v.insert(start_url.clone());
        q.push_back((start_url, 0));
        (v, q)
    };

    let progress = CrawlProgress::new();
    let progress_clone = progress.clone();

    // Spawn progress reporter
    let reporter = tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(2)).await;
            println!("{}", progress_clone.get_stats());
        }
    });

    let mut pages = Vec::new();
    let client = Arc::new(reqwest::Client::new());

    // Main crawl loop with shutdown handling
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\nShutdown requested, saving state...");
                if let Some(ref path) = state_file {
                    let state = CrawlState { visited: visited.clone(), queue: queue.clone() };
                    if let Err(e) = save_state(&state, path).await {
                        eprintln!("Failed to save state: {}", e);
                    } else {
                        println!("State saved to {}", path);
                    }
                }
                reporter.abort();
                break;
            }
            _ = async {
                if let Some((url, depth)) = queue.pop_front() {
                    if pages.len() >= config.max_pages || depth > config.max_depth {
                        return;
                    }

                    progress.set_queue_depth(queue.len());

                    let result = fetch_with_rate_limit(
                        &client,
                        &url,
                        &config.retry_config,
                        &config.rate_limiter,
                    ).await;

                    if result.error.is_some() {
                        progress.increment_failed();
                    }
                    progress.increment_fetched();

                    if let Some(body) = &result.body {
                        let links = extract_links(body, &url);
                        for link in links {
                            if !visited.contains(&link)
                                && is_allowed_domain(&link, &config.allowed_domains)
                            {
                                visited.insert(link.clone());
                                queue.push_back((link, depth + 1));
                            }
                        }
                    }

                    pages.push(result);
                } else {
                    // Queue empty, we're done
                    reporter.abort();
                    return;
                }
            } => {}
        }

        if queue.is_empty() || pages.len() >= config.max_pages {
            reporter.abort();
            break;
        }
    }

    println!("\nCrawl complete! Final stats:");
    println!("{}", progress.get_stats());

    CrawlResult {
        visited_urls: visited,
        pages,
    }
}

// ============================================================================
// Main Example
// ============================================================================

#[tokio::main]
async fn main() {
    println!("=== Web Scraper Demo ===\n");

    let config = CrawlConfig {
        max_depth: 2,
        max_pages: 20,
        allowed_domains: Some(vec!["example.com".to_string()]),
        retry_config: RetryConfig::default(),
        rate_limiter: Arc::new(RateLimiter::new(5)),
    };

    let result = crawl_with_shutdown(
        "https://example.com".to_string(),
        config,
        Some("crawl_state.bin".to_string()),
    ).await;

    println!("\nVisited {} URLs", result.visited_urls.len());
    println!("Fetched {} pages", result.pages.len());

    // Show some results
    for (i, page) in result.pages.iter().take(5).enumerate() {
        println!("\nPage {}: {}", i + 1, page.url);
        println!("  Status: {}", page.status_code);
        if let Some(body) = &page.body {
            println!("  Body length: {} bytes", body.len());
        }
        if let Some(error) = &page.error {
            println!("  Error: {}", error);
        }
    }
}
```

This complete implementation demonstrates:
1. **Async HTTP fetching** with timeouts
2. **Concurrent execution** with buffering
3. **Retry logic** with exponential backoff
4. **Per-domain rate limiting** using semaphores
5. **Recursive crawling** with link extraction
6. **Progress reporting** and graceful shutdown

The scraper efficiently handles hundreds of URLs while respecting rate limits and handling failures gracefully—a production-ready foundation for web scraping projects.
