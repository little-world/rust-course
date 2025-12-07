
## Project 2: Async Web Scraper with Retry Logic and Circuit Breaker

### Problem Statement

Build a robust asynchronous web scraper that fetches data from multiple URLs concurrently with sophisticated error handling. The scraper must handle transient failures (timeouts, network errors) with exponential backoff retry, prevent cascading failures with circuit breakers, and aggregate results from parallel operations.

Your scraper should support:
- Fetching multiple URLs concurrently (using tokio or async-std)
- Retrying failed requests with exponential backoff (up to N attempts)
- Implementing circuit breaker pattern (open/half-open/closed states)
- Handling timeouts on all network operations
- Collecting partial results (some URLs may fail permanently)
- Rate-limiting requests to avoid overwhelming servers
- Tracking and reporting error statistics

---

### Milestone 1: Basic Async HTTP Client with Error Types

**Goal**: Create async HTTP client with typed errors using reqwest and tokio.

**What to implement**:
- Define `ScraperError` enum with variants for different failure modes
- Implement async `fetch_url()` function using reqwest
- Convert reqwest errors to your error type
- Basic error handling with Result types

**Architecture**:
- Enums: `ScraperError`
- Functions:
  - `fetch_url(url: &str) -> Result<String, ScraperError>` - Async HTTP GET
  - Error conversion from reqwest errors

---

**Starter Code**:

```rust
use reqwest;
use thiserror::Error;
use tokio;

/// Comprehensive error type for web scraping
/// Role: Typed error handling for network operations
#[derive(Error, Debug, Clone)]
pub enum ScraperError {
    #[error("Network error: {0}")]
    NetworkError(String),                                   // Connection failures, DNS errors   
                                                                 
    #[error("Request timed out after {0}ms")]                     
    TimeoutError(u64),                                      // Request exceeded time limit        
                                                            
    #[error("HTTP {status} error for {url}")]               
    HttpError { status: u16, url: String },                 // Non-success HTTP status codes  
                                                              
    #[error("Failed to parse response: {0}")]                
    ParseError(String),                                     // Failed to parse response body 
}

/// Fetch URL content
/// Role: Basic async HTTP GET request
pub async fn fetch_url(url: &str) -> Result<String, ScraperError> {
    todo!("Implement async HTTP GET with error conversion")
}

/// Convert reqwest errors to ScraperError
impl From<reqwest::Error> for ScraperError {
    /// Role: Map reqwest errors to our error type
    fn from(err: reqwest::Error) -> Self {
        todo!("Convert based on error type - timeout, connection, etc.")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_fetch_url_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_string("Hello, World!"))
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", &mock_server.uri());
        let result = fetch_url(&url).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_fetch_url_404() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/notfound"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let url = format!("{}/notfound", &mock_server.uri());
        let result = fetch_url(&url).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ScraperError::HttpError { status, .. } => assert_eq!(status, 404),
            _ => panic!("Expected HttpError"),
        }
    }

    #[tokio::test]
    async fn test_fetch_url_network_error() {
        // Invalid URL
        let result = fetch_url("http://invalid.local.test").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ScraperError::NetworkError(_)));
    }

    #[tokio::test]
    async fn test_error_display() {
        let error = ScraperError::HttpError {
            status: 500,
            url: "http://example.com".to_string(),
        };

        let display = format!("{}", error);
        assert!(display.contains("500"));
        assert!(display.contains("example.com"));
    }

    #[tokio::test]
    async fn test_error_is_send_and_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<ScraperError>();
        assert_sync::<ScraperError>();
    }
}
```

---

### Milestone 2: Add Timeout to Prevent Hanging

**Goal**: Ensure all network operations have bounded execution time using tokio::time::timeout.

**Why the previous milestone is not enough**: Without timeouts, slow or hanging servers can block your application indefinitely, causing resource exhaustion. One stuck request can hold threads/tasks forever, preventing progress on other work.

**What's the improvement**: Timeouts guarantee bounded waiting time. If a server takes >30s, you fail fast and move on. This prevents one slow endpoint from blocking 100 other requests. Essential for responsive systems. The difference is between "hangs forever" and "fails after 30s and continues".

**Architecture**:
- Functions:
  - `fetch_url_with_timeout(url: &str, timeout_ms: u64) -> Result<String, ScraperError>` - Fetch with timeout
  - Use `tokio::time::timeout()` wrapper

---

**Starter Code**:

```rust
use tokio::time::{timeout, Duration};

/// Fetch URL with timeout
/// Role: Bounded execution time for network operations
pub async fn fetch_url_with_timeout(
    url: &str,
    timeout_ms: u64,
) -> Result<String, ScraperError> {
    todo!("Wrap fetch_url in tokio::time::timeout")
}

/// Create HTTP client with timeout
/// Role: Configure reqwest client with timeout
pub fn create_client(timeout_ms: u64) -> reqwest::Client {
    todo!("Build reqwest::Client with timeout configuration")
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_timeout_success_within_limit() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("Quick response"))
            .mount(&mock_server)
            .await;

        let result = fetch_url_with_timeout(&mock_server.uri(), 5000).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_timeout_triggers() {
        let mock_server = MockServer::start().await;

        // Delay response by 2 seconds
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_secs(2))
            )
            .mount(&mock_server)
            .await;

        // Timeout after 100ms
        let result = fetch_url_with_timeout(&mock_server.uri(), 100).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ScraperError::TimeoutError(ms) => assert_eq!(ms, 100),
            e => panic!("Expected TimeoutError, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_client_timeout_configuration() {
        let client = create_client(1000);

        // Client should have timeout configured
        // This is integration-tested through fetch_url_with_timeout
    }

    #[tokio::test]
    async fn test_timeout_error_includes_duration() {
        let error = ScraperError::TimeoutError(5000);
        let display = format!("{}", error);
        assert!(display.contains("5000"));
        assert!(display.contains("timed out"));
    }
}
```

---

### Milestone 3: Retry with Exponential Backoff

**Goal**: Automatically retry failed requests with increasing delays and jitter.

**Why the previous milestone is not enough**: Network errors are often transient (temporary DNS issues, brief connection drops). A single retry could succeed, but immediate retry might fail again if the issue needs time to resolve. We need smart retry strategy.

**What's the improvement**: Exponential backoff gives services time to recover (1s, 2s, 4s, 8s...). Jitter prevents thundering herd (synchronized retries from many clients overwhelming the recovering server). This transforms "50% of requests fail due to transient errors" into "99% success rate with retries". The 1-2% permanent failures are caught appropriately.

**Optimization focus**: Reliability through intelligent retry strategy - maximize success rate while respecting server recovery time.

**Architecture**:
- Structs: `RetryConfig`
- Functions:
  - `fetch_with_retry(url, timeout, config) -> Result<String, ScraperError>` - Retry logic
  - `ScraperError::is_retryable() -> bool` - Check if error warrants retry
  - `calculate_backoff(attempt: usize) -> Duration` - Exponential backoff with jitter

---

**Starter Code**:

```rust
use rand::Rng;

///  Configure retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: usize,                // Maximum retry attempts              
    pub initial_backoff_ms: u64,            // Starting backoff delay          
    pub max_backoff_ms: u64,                // Maximum backoff delay               
    pub jitter: bool,                       // Randomness to prevent thundering herd  
}

impl RetryConfig {
    /// Create default retry config
    /// Role: Sensible defaults for most cases
    pub fn default() -> Self {
        todo!("Return config with 3 attempts, 1s initial, 30s max")
    }

    /// Calculate backoff for attempt
    /// Role: Exponential backoff with optional jitter
    pub fn backoff_duration(&self, attempt: usize) -> Duration {
        todo!("Calculate 2^attempt * initial, apply jitter if enabled")
    }
}

impl ScraperError {
    /// Check if error is retryable
    /// Role: Classify errors as transient or permanent
    pub fn is_retryable(&self) -> bool {
        todo!("Return true for network errors, timeouts, 5xx; false for 4xx")
    }
}

/// Fetch URL with retry logic
/// Role: Resilient HTTP requests
pub async fn fetch_with_retry(
    url: &str,
    timeout_ms: u64,
    retry_config: &RetryConfig,
) -> Result<String, ScraperError> {
    todo!("Loop up to max_attempts, apply backoff between retries")
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Instant;

    #[tokio::test]
    async fn test_retry_succeeds_after_failures() {
        let mock_server = MockServer::start().await;
        let attempt_count = Arc::new(Mutex::new(0));
        let attempt_count_clone = attempt_count.clone();

        // Fail first 2 times, succeed on 3rd
        Mock::given(method("GET"))
            .respond_with(move |_req: &wiremock::Request| {
                let mut count = attempt_count_clone.lock().unwrap();
                *count += 1;

                if *count < 3 {
                    ResponseTemplate::new(500)
                } else {
                    ResponseTemplate::new(200).set_body_string("Success!")
                }
            })
            .mount(&mock_server)
            .await;

        let config = RetryConfig {
            max_attempts: 3,
            initial_backoff_ms: 10,
            max_backoff_ms: 1000,
            jitter: false,
        };

        let result = fetch_with_retry(&mock_server.uri(), 1000, &config).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success!");
        assert_eq!(*attempt_count.lock().unwrap(), 3);
    }

    #[tokio::test]
    async fn test_retry_gives_up_after_max_attempts() {
        let mock_server = MockServer::start().await;

        // Always fail
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let config = RetryConfig {
            max_attempts: 3,
            initial_backoff_ms: 10,
            max_backoff_ms: 1000,
            jitter: false,
        };

        let result = fetch_with_retry(&mock_server.uri(), 1000, &config).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ScraperError::HttpError { status, .. } => assert_eq!(status, 500),
            e => panic!("Expected HttpError, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_no_retry_on_404() {
        let mock_server = MockServer::start().await;
        let attempt_count = Arc::new(Mutex::new(0));
        let attempt_count_clone = attempt_count.clone();

        Mock::given(method("GET"))
            .respond_with(move |_req: &wiremock::Request| {
                let mut count = attempt_count_clone.lock().unwrap();
                *count += 1;
                ResponseTemplate::new(404)
            })
            .mount(&mock_server)
            .await;

        let config = RetryConfig {
            max_attempts: 3,
            initial_backoff_ms: 10,
            max_backoff_ms: 1000,
            jitter: false,
        };

        let result = fetch_with_retry(&mock_server.uri(), 1000, &config).await;

        assert!(result.is_err());
        // Should only try once (404 is not retryable)
        assert_eq!(*attempt_count.lock().unwrap(), 1);
    }

    #[test]
    fn test_exponential_backoff() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_backoff_ms: 100,
            max_backoff_ms: 10000,
            jitter: false,
        };

        assert_eq!(config.backoff_duration(0).as_millis(), 100);
        assert_eq!(config.backoff_duration(1).as_millis(), 200);
        assert_eq!(config.backoff_duration(2).as_millis(), 400);
        assert_eq!(config.backoff_duration(3).as_millis(), 800);
    }

    #[test]
    fn test_backoff_respects_max() {
        let config = RetryConfig {
            max_attempts: 10,
            initial_backoff_ms: 100,
            max_backoff_ms: 1000,
            jitter: false,
        };

        // Should cap at max_backoff_ms
        assert!(config.backoff_duration(10).as_millis() <= 1000);
    }

    #[test]
    fn test_jitter_adds_randomness() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 10000,
            jitter: true,
        };

        let d1 = config.backoff_duration(1);
        let d2 = config.backoff_duration(1);

        // With jitter, same attempt should give different durations
        // This is probabilistic but very likely
        // Note: May rarely fail due to randomness
    }

    #[tokio::test]
    async fn test_retry_timing() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let config = RetryConfig {
            max_attempts: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 10000,
            jitter: false,
        };

        let start = Instant::now();
        let _ = fetch_with_retry(&mock_server.uri(), 1000, &config).await;
        let elapsed = start.elapsed();

        // Should take at least 100ms + 200ms = 300ms for backoffs
        assert!(elapsed.as_millis() >= 300);
    }

    #[test]
    fn test_error_retryability() {
        assert!(ScraperError::NetworkError("test".to_string()).is_retryable());
        assert!(ScraperError::TimeoutError(1000).is_retryable());
        assert!(ScraperError::HttpError { status: 500, url: "".to_string() }.is_retryable());
        assert!(ScraperError::HttpError { status: 503, url: "".to_string() }.is_retryable());

        assert!(!ScraperError::HttpError { status: 404, url: "".to_string() }.is_retryable());
        assert!(!ScraperError::HttpError { status: 403, url: "".to_string() }.is_retryable());
        assert!(!ScraperError::ParseError("test".to_string()).is_retryable());
    }
}
```

---

### Milestone 4: Circuit Breaker Pattern

**Goal**: Prevent repeated calls to failing services using circuit breaker state machine.

**Why the previous milestone is not enough**: Retries help with transient failures, but if a service is completely down, retrying wastes time and resources. Every retry waits for timeout, consuming threads and memory. A failing service receiving constant retries can't recover.

**What's the improvement**: Circuit breakers fail fast when a service is known to be down. Instead of waiting 30s for timeout on every request, fail in 1ms. This prevents cascading failures: if service A is down, service B (which depends on A) stops hammering it with requests, allowing A to recover. Resource exhaustion is prevented, latency drops dramatically (1ms vs 30s), and failing services get breathing room to recover.

**Optimization focus**: Speed (fail fast) and reliability (allow service recovery).

**Architecture**:
- Enums: `CircuitState` (Closed, Open, HalfOpen)
- Structs: `CircuitBreaker`
- Fields: `state: Arc<Mutex<CircuitState>>`, `failure_count: Arc<Mutex<usize>>`, `threshold: usize`
- Functions:
  - `CircuitBreaker::new(threshold, timeout)` - Create circuit breaker
  - `call<F>(&self, f: F) -> Result<T, ScraperError>` - Execute with circuit breaker
  - State transition logic (closed → open → half-open → closed)

---

**Starter Code**:

```rust
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use std::future::Future;

/// Circuit breaker states
/// Role: Track service health
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,                                   // Normal operation, requests pass through       
    Open { opened_at: Instant },              // Service failing, reject requests immediately    
    HalfOpen,                                 // Testing if service recovered                
}

/// Circuit breaker for preventing cascading failures
/// Fault tolerance component
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,                      // Current state              
    failure_threshold: usize,                             // Failures before opening           
    success_threshold: usize,                             // Successes to close from half-open 
    timeout: Duration,                                    // Time before trying half-open             
    consecutive_failures: Arc<Mutex<usize>>,              // Failure counter    
    consecutive_successes: Arc<Mutex<usize>>,             // Success counter   
}

impl CircuitBreaker {
    /// Create new circuit breaker
    /// Role: Initialize with thresholds and timeout
    pub fn new(failure_threshold: usize, timeout: Duration) -> Self {
        todo!("Initialize circuit breaker in Closed state")
    }

    /// Execute function with circuit breaker protection
    /// Role: Wrap async call with circuit breaker logic
    pub async fn call<F, T>(&self, f: F) -> Result<T, ScraperError>
    where
        F: Future<Output = Result<T, ScraperError>>,
    {
        todo!("Check state, execute if closed/half-open, update state based on result")
    }

    /// Handle successful request
    /// Role: Update state after success
    fn on_success(&self) {
        todo!("Reset failure count, increment success count, transition to Closed if needed")
    }

    /// Handle failed request
    /// Role: Update state after failure
    fn on_failure(&self) {
        todo!("Increment failure count, transition to Open if threshold exceeded")
    }

    /// Get current state
    /// Role: Query circuit breaker state
    pub fn state(&self) -> CircuitState {
        todo!("Return current state")
    }

    /// Check if should attempt request
    /// Role: Determine if circuit allows request
    fn should_attempt(&self) -> bool {
        todo!("Return false if Open and timeout not elapsed")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(1));

        assert_eq!(cb.state(), CircuitState::Closed);

        // Cause 3 failures
        for _ in 0..3 {
            let result = cb.call(async {
                Err::<(), _>(ScraperError::NetworkError("fail".to_string()))
            }).await;
            assert!(result.is_err());
        }

        // Circuit should now be open
        match cb.state() {
            CircuitState::Open { .. } => {},
            state => panic!("Expected Open, got {:?}", state),
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_fails_fast_when_open() {
        let cb = CircuitBreaker::new(2, Duration::from_secs(1));

        // Open the circuit
        for _ in 0..2 {
            let _ = cb.call(async {
                Err::<(), _>(ScraperError::NetworkError("fail".to_string()))
            }).await;
        }

        // Next call should fail immediately without executing
        let start = Instant::now();
        let result = cb.call(async {
            tokio::time::sleep(Duration::from_secs(1)).await;
            Ok::<_, ScraperError>(())
        }).await;
        let elapsed = start.elapsed();

        assert!(result.is_err());
        // Should fail instantly (< 100ms), not wait for sleep
        assert!(elapsed.as_millis() < 100);
    }

    #[tokio::test]
    async fn test_circuit_breaker_transitions_to_half_open() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(100));

        // Open the circuit
        for _ in 0..2 {
            let _ = cb.call(async {
                Err::<(), _>(ScraperError::NetworkError("fail".to_string()))
            }).await;
        }

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Next call should transition to HalfOpen
        let _ = cb.call(async { Ok::<_, ScraperError>(()) }).await;

        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_circuit_breaker_closes_after_successes() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(100));

        // Open the circuit
        for _ in 0..2 {
            let _ = cb.call(async {
                Err::<(), _>(ScraperError::NetworkError("fail".to_string()))
            }).await;
        }

        // Wait and succeed twice (assuming success_threshold = 2)
        tokio::time::sleep(Duration::from_millis(150)).await;

        for _ in 0..2 {
            let result = cb.call(async { Ok::<_, ScraperError>(()) }).await;
            assert!(result.is_ok());
        }

        // Circuit should be closed
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_reopens_on_half_open_failure() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(100));

        // Open the circuit
        for _ in 0..2 {
            let _ = cb.call(async {
                Err::<(), _>(ScraperError::NetworkError("fail".to_string()))
            }).await;
        }

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Try in half-open and fail
        let _ = cb.call(async {
            Err::<(), _>(ScraperError::NetworkError("still failing".to_string()))
        }).await;

        // Should reopen
        match cb.state() {
            CircuitState::Open { .. } => {},
            state => panic!("Expected Open, got {:?}", state),
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_success_resets_failure_count() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(1));

        // Two failures
        for _ in 0..2 {
            let _ = cb.call(async {
                Err::<(), _>(ScraperError::NetworkError("fail".to_string()))
            }).await;
        }

        // One success
        let _ = cb.call(async { Ok::<_, ScraperError>(()) }).await;

        // Another failure shouldn't open (count was reset)
        let _ = cb.call(async {
            Err::<(), _>(ScraperError::NetworkError("fail".to_string()))
        }).await;

        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_with_real_fetch() {
        let mock_server = MockServer::start().await;
        let cb = Arc::new(CircuitBreaker::new(2, Duration::from_millis(100)));

        // Setup failing endpoint
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let url = mock_server.uri();

        // Open circuit with failures
        for _ in 0..2 {
            let cb_clone = cb.clone();
            let url_clone = url.clone();
            let _ = cb_clone.call(fetch_url(&url_clone)).await;
        }

        // Verify circuit is open
        match cb.state() {
            CircuitState::Open { .. } => {},
            state => panic!("Expected Open, got {:?}", state),
        }
    }
}
```

---

### Milestone 5: Concurrent Fetching with Partial Results

**Goal**: Fetch multiple URLs concurrently and collect partial results even if some fail.

**Why the previous milestone is not enough**: Sequential fetching is slow. Fetching 100 URLs at 1 second each takes 100 seconds. We have retry logic and circuit breakers, but they don't help with the sequential bottleneck. Also, naive `try_join_all` fails completely if ANY request fails, losing all successful data.

**What's the improvement**: Parallelism provides massive speedup for I/O-bound operations - 100 concurrent requests take ~1 second (limited by slowest). Collecting partial results means "get as much data as possible" rather than "all or nothing". For data aggregation where 95/100 URLs succeed, you get 95% of the data instead of 0%. This is essential for resilient data pipelines.

**Optimization focus**: Speed through parallelism and resilience through partial success collection.

**Architecture**:
- Structs: `FetchResult`, `FetchSummary`
- Functions:
  - `fetch_all(urls, config) -> Vec<FetchResult>` - Concurrent fetching
  - `FetchSummary::from_results()` - Aggregate statistics

---

**Starter Code**:

```rust
use futures::future::join_all;
use std::time::Instant;

/// Result of fetching a single URL
/// Role: Record individual fetch outcomes
#[derive(Debug)]
pub struct FetchResult {
    pub url: String,                                      // The URL that was fetched                       
    pub result: Result<String, ScraperError>,             // Success or failure    
    pub duration_ms: u64,                                 // Time taken for this fetch                 
    pub attempt_count: usize,                             // Number of retries needed              
}

impl FetchResult {
    /// Check if fetch succeeded
    pub fn is_success(&self) -> bool {
        todo!("Query success status")
    }

    /// Get content if successful
    pub fn content(&self) -> Option<&str> {
        todo!("Extract content from result")
    }
}

/// FetchSummary: Provide overview of batch operation
#[derive(Debug)]
pub struct FetchSummary {
    pub total: usize,                        // Total URLs attempted              
    pub success: usize,                      // Successful fetches              
    pub failed: usize,                       // Failed fetches                   
    pub total_duration_ms: u64,              // Total time for batch    
    pub avg_duration_ms: u64,                // Average time per fetch    
}

impl FetchSummary {
    /// Create summary from results
    /// Role: Aggregate statistics
    pub fn from_results(results: &[FetchResult]) -> Self {
        todo!("Calculate success/failure counts, timing statistics")
    }

    /// Calculate success rate as percentage
    pub fn success_rate(&self) -> f64 {
      todo!("Compute reliability metric")
    }
}

/// Fetch multiple URLs concurrently
/// Role: Maximize throughput with parallel I/O
pub async fn fetch_all(
    urls: Vec<String>,
    timeout_ms: u64,
    retry_config: &RetryConfig,
    circuit_breaker: &Arc<CircuitBreaker>,
) -> Vec<FetchResult> {
    todo!("Create futures for all URLs, use join_all to execute concurrently")
}

/// Fetch URLs with rate limiting
/// Role: Control resource usage
pub async fn fetch_all_with_limit(
    urls: Vec<String>,
    timeout_ms: u64,
    retry_config: &RetryConfig,
    circuit_breaker: &Arc<CircuitBreaker>,
    max_concurrent: usize,
) -> Vec<FetchResult> {
    todo!("Use semaphore to limit concurrent requests")
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_all_concurrent() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("data"))
            .mount(&mock_server)
            .await;

        let urls = vec![
            format!("{}/1", mock_server.uri()),
            format!("{}/2", mock_server.uri()),
            format!("{}/3", mock_server.uri()),
        ];

        let config = RetryConfig::default();
        let cb = Arc::new(CircuitBreaker::new(5, Duration::from_secs(10)));

        let start = Instant::now();
        let results = fetch_all(urls, 5000, &config, &cb).await;
        let elapsed = start.elapsed();

        // All should succeed
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_success()));

        // Should be concurrent (not 3x sequential time)
        // Hard to assert precisely, but should be reasonably fast
        assert!(elapsed.as_secs() < 2);
    }

    #[tokio::test]
    async fn test_fetch_all_partial_success() {
        let mock_server = MockServer::start().await;

        // First endpoint succeeds
        Mock::given(method("GET"))
            .and(path("/good"))
            .respond_with(ResponseTemplate::new(200).set_body_string("success"))
            .mount(&mock_server)
            .await;

        // Second endpoint fails
        Mock::given(method("GET"))
            .and(path("/bad"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let urls = vec![
            format!("{}/good", mock_server.uri()),
            format!("{}/bad", mock_server.uri()),
        ];

        let config = RetryConfig { max_attempts: 1, ..RetryConfig::default() };
        let cb = Arc::new(CircuitBreaker::new(5, Duration::from_secs(10)));

        let results = fetch_all(urls, 5000, &config, &cb).await;

        assert_eq!(results.len(), 2);
        assert!(results[0].is_success());
        assert!(!results[1].is_success());
    }

    #[test]
    fn test_fetch_summary_calculations() {
        let results = vec![
            FetchResult {
                url: "url1".to_string(),
                result: Ok("data".to_string()),
                duration_ms: 100,
                attempt_count: 1,
            },
            FetchResult {
                url: "url2".to_string(),
                result: Err(ScraperError::NetworkError("fail".to_string())),
                duration_ms: 200,
                attempt_count: 3,
            },
            FetchResult {
                url: "url3".to_string(),
                result: Ok("data".to_string()),
                duration_ms: 150,
                attempt_count: 1,
            },
        ];

        let summary = FetchSummary::from_results(&results);

        assert_eq!(summary.total, 3);
        assert_eq!(summary.success, 2);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.success_rate(), 66.66666666666666);
    }

    #[tokio::test]
    async fn test_fetch_all_with_concurrency_limit() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_millis(100))
            )
            .mount(&mock_server)
            .await;

        let urls: Vec<String> = (0..10)
            .map(|i| format!("{}/{}", mock_server.uri(), i))
            .collect();

        let config = RetryConfig::default();
        let cb = Arc::new(CircuitBreaker::new(10, Duration::from_secs(10)));

        let start = Instant::now();
        let results = fetch_all_with_limit(urls, 5000, &config, &cb, 3).await;
        let elapsed = start.elapsed();

        assert_eq!(results.len(), 10);

        // With limit of 3 and 100ms delay, should take roughly:
        // 10 requests / 3 concurrent = ~4 batches * 100ms = ~400ms
        assert!(elapsed.as_millis() >= 300);
        assert!(elapsed.as_millis() < 600);
    }

    #[tokio::test]
    async fn test_fetch_result_methods() {
        let success = FetchResult {
            url: "http://example.com".to_string(),
            result: Ok("content".to_string()),
            duration_ms: 100,
            attempt_count: 1,
        };

        assert!(success.is_success());
        assert_eq!(success.content(), Some("content"));

        let failure = FetchResult {
            url: "http://example.com".to_string(),
            result: Err(ScraperError::NetworkError("fail".to_string())),
            duration_ms: 100,
            attempt_count: 3,
        };

        assert!(!failure.is_success());
        assert_eq!(failure.content(), None);
    }
}
```

---

### Milestone 6: Rate Limiting and Resource Management

**Goal**: Add rate limiting to avoid overwhelming servers and manage client resources.

**Why the previous milestone is not enough**: Unlimited concurrent requests can overwhelm target servers (causing 429 rate limit errors or even getting IP banned) or exhaust client resources (memory, file descriptors, network buffers). Launching 10,000 tasks simultaneously can cause OOM errors.

**What's the improvement**: Rate limiting is respectful (doesn't DDoS targets) and prevents resource exhaustion. Semaphore limits concurrent requests: instead of launching 10,000 tasks simultaneously, limit to 100 concurrent tasks. Token bucket rate limiter ensures you don't exceed server rate limits (e.g., 100 req/min). This prevents bans and keeps your client healthy.

**Optimization focus**: Resource efficiency and reliability - stay within limits while maximizing throughput.

**Architecture**:
- Structs: `RateLimiter`, `ResourceManager`
- Functions:
  - `RateLimiter::new(requests_per_second)` - Create rate limiter
  - `acquire() -> Permit` - Wait for rate limit token
  - Use `tokio::sync::Semaphore` for concurrency limiting

---

**Starter Code**:

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

/// Token bucket rate limiter
/// Role: Enforce rate limits
pub struct RateLimiter {
    semaphore: Arc<Semaphore>,                      // Concurrency limiter              
    rate_per_second: f64,                           // Target rate                           
    last_request: Arc<Mutex<Instant>>,              // Last request timestamp   
}

impl RateLimiter {
    /// Create rate limiter
    /// Role: Initialize with rate limit
    pub fn new(max_concurrent: usize, rate_per_second: f64) -> Self {
        todo!("Create semaphore with max_concurrent permits")
    }

    /// Acquire permission to make request
    /// Role: Block until rate limit allows request
    pub async fn acquire(&self) -> tokio::sync::SemaphorePermit<'_> {
        todo!("Wait for semaphore, enforce rate limiting")
    }
}

/// Resource manager for scraper
/// Role: Monitor and limit resource consumption
pub struct ResourceManager {
    active_requests: Arc<Mutex<usize>>,                // Current in-flight requests   
    total_bytes: Arc<Mutex<u64>>,                      // Total data downloaded              
    max_memory_bytes: u64,                             // Memory limit                              
}

impl ResourceManager {
    /// Create resource manager
    /// Role: Initialize with limits
    pub fn new(max_memory_bytes: u64) -> Self {
        todo!("Initialize counters")
    }

    /// Check if can make new request
    /// Role: Enforce resource limits
    pub fn can_proceed(&self) -> bool {
        todo!("Check memory usage against limit")
    }

    /// Record request start
    /// Role: Track active request
    pub fn start_request(&self) {
        todo!("Increment active_requests")
    }

    /// Record request completion
    /// Role: Update counters
    pub fn end_request(&self, bytes: u64) {
        todo!("Decrement active_requests, add to total_bytes")
    }
}

/// Fetch URLs with complete resource management
/// Role: Production-ready scraping
pub async fn fetch_all_managed(
    urls: Vec<String>,
    timeout_ms: u64,
    retry_config: &RetryConfig,
    circuit_breaker: &Arc<CircuitBreaker>,
    rate_limiter: &RateLimiter,
    resource_manager: &Arc<ResourceManager>,
) -> Vec<FetchResult> {
    todo!("Combine rate limiting, circuit breaker, resource management")
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_enforces_concurrency() {
        let limiter = RateLimiter::new(2, 100.0);

        let start = Instant::now();

        // Try to acquire 3 permits
        let _p1 = limiter.acquire().await;
        let _p2 = limiter.acquire().await;

        // Third should wait (spawn task to test non-blocking of test)
        let limiter_clone = Arc::new(limiter);
        let task = tokio::spawn({
            let limiter = limiter_clone.clone();
            async move {
                let _p3 = limiter.acquire().await;
            }
        });

        // Give it a moment
        tokio::time::sleep(Duration::from_millis(50)).await;

        drop(_p1); // Release one permit

        task.await.unwrap();
    }

    #[tokio::test]
    async fn test_rate_limiter_timing() {
        let limiter = RateLimiter::new(10, 5.0); // 5 requests/second

        let start = Instant::now();

        // Make 10 requests (should take ~2 seconds at 5/sec)
        for _ in 0..10 {
            let _permit = limiter.acquire().await;
        }

        let elapsed = start.elapsed();

        // Should take roughly 2 seconds
        assert!(elapsed.as_millis() >= 1800);
        assert!(elapsed.as_millis() < 2500);
    }

    #[tokio::test]
    async fn test_resource_manager_tracks_requests() {
        let manager = Arc::new(ResourceManager::new(1_000_000));

        manager.start_request();
        assert_eq!(*manager.active_requests.lock().unwrap(), 1);

        manager.start_request();
        assert_eq!(*manager.active_requests.lock().unwrap(), 2);

        manager.end_request(1000);
        assert_eq!(*manager.active_requests.lock().unwrap(), 1);
        assert_eq!(*manager.total_bytes.lock().unwrap(), 1000);

        manager.end_request(2000);
        assert_eq!(*manager.active_requests.lock().unwrap(), 0);
        assert_eq!(*manager.total_bytes.lock().unwrap(), 3000);
    }

    #[tokio::test]
    async fn test_resource_manager_enforces_limits() {
        let manager = Arc::new(ResourceManager::new(1000));

        manager.start_request();
        manager.end_request(500);

        assert!(manager.can_proceed());

        manager.start_request();
        manager.end_request(600);

        // Total is now 1100, exceeds limit
        assert!(!manager.can_proceed());
    }

    #[tokio::test]
    async fn test_fetch_all_managed_integration() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("data"))
            .mount(&mock_server)
            .await;

        let urls: Vec<String> = (0..5)
            .map(|i| format!("{}/{}", mock_server.uri(), i))
            .collect();

        let config = RetryConfig::default();
        let cb = Arc::new(CircuitBreaker::new(5, Duration::from_secs(10)));
        let limiter = RateLimiter::new(2, 10.0);
        let manager = Arc::new(ResourceManager::new(1_000_000));

        let results = fetch_all_managed(
            urls,
            5000,
            &config,
            &cb,
            &limiter,
            &manager,
        ).await;

        assert_eq!(results.len(), 5);
        assert!(results.iter().all(|r| r.is_success()));
    }
}
```

---

### Testing Strategies

1. **Unit Tests**: Test each component (retry, circuit breaker, rate limiter) in isolation
2. **Integration Tests**: Use `wiremock` for realistic HTTP testing
3. **Chaos Testing**: Randomly fail requests to test resilience
4. **Performance Tests**: Measure concurrent vs sequential performance
5. **Load Tests**: Test with 1000+ URLs
6. **Timeout Tests**: Verify all operations have bounded time
7. **Resource Tests**: Monitor memory and connection usage

---

### Complete Working Example

```rust
use futures::future::join_all;
use reqwest;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio;
use tokio::sync::Semaphore;

//==============================================================================
// Part 1: Error Types
//==============================================================================

#[derive(Error, Debug, Clone)]
pub enum ScraperError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Request timed out after {0}ms")]
    TimeoutError(u64),

    #[error("HTTP {status} error for {url}")]
    HttpError { status: u16, url: String },

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Circuit breaker is open")]
    CircuitBreakerOpen,
}

impl ScraperError {
    fn is_retryable(&self) -> bool {
        matches!(
            self,
            ScraperError::NetworkError(_)
                | ScraperError::TimeoutError(_)
                | ScraperError::HttpError { status: 500..=599, .. }
        )
    }
}

//==============================================================================
// Part 2: Basic Fetching
//==============================================================================

pub async fn fetch_url(url: &str) -> Result<String, ScraperError> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| ScraperError::NetworkError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(ScraperError::HttpError {
            status: response.status().as_u16(),
            url: url.to_string(),
        });
    }

    response
        .text()
        .await
        .map_err(|e| ScraperError::ParseError(e.to_string()))
}

//==============================================================================
// Part 3: Retry Logic
//==============================================================================

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: usize,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_attempts: 3,
            initial_backoff_ms: 1000,
            max_backoff_ms: 30000,
        }
    }
}

pub async fn fetch_with_retry(
    url: &str,
    timeout_ms: u64,
    config: &RetryConfig,
) -> Result<String, ScraperError> {
    let mut attempt = 0;
    let mut backoff_ms = config.initial_backoff_ms;

    loop {
        attempt += 1;

        let fetch_future = fetch_url(url);
        let result = tokio::time::timeout(Duration::from_millis(timeout_ms), fetch_future).await;

        match result {
            Ok(Ok(content)) => return Ok(content),
            Ok(Err(e)) if attempt >= config.max_attempts => return Err(e),
            Ok(Err(e)) if !e.is_retryable() => return Err(e),
            Err(_) if attempt >= config.max_attempts => {
                return Err(ScraperError::TimeoutError(timeout_ms))
            }
            _ => {
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                backoff_ms = (backoff_ms * 2).min(config.max_backoff_ms);
            }
        }
    }
}

//==============================================================================
// Part 4: Circuit Breaker
//==============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
}

pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_threshold: usize,
    timeout: Duration,
    consecutive_failures: Arc<Mutex<usize>>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, timeout: Duration) -> Self {
        CircuitBreaker {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_threshold,
            timeout,
            consecutive_failures: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn call<F, T>(&self, f: F) -> Result<T, ScraperError>
    where
        F: std::future::Future<Output = Result<T, ScraperError>>,
    {
        // Check state
        {
            let mut state = self.state.lock().unwrap();
            match *state {
                CircuitState::Open { opened_at } => {
                    if opened_at.elapsed() > self.timeout {
                        *state = CircuitState::HalfOpen;
                    } else {
                        return Err(ScraperError::CircuitBreakerOpen);
                    }
                }
                _ => {}
            }
        }

        // Execute
        match f.await {
            Ok(result) => {
                *self.consecutive_failures.lock().unwrap() = 0;
                let mut state = self.state.lock().unwrap();
                if matches!(*state, CircuitState::HalfOpen) {
                    *state = CircuitState::Closed;
                }
                Ok(result)
            }
            Err(e) => {
                let mut failures = self.consecutive_failures.lock().unwrap();
                *failures += 1;

                if *failures >= self.failure_threshold {
                    *self.state.lock().unwrap() = CircuitState::Open {
                        opened_at: Instant::now(),
                    };
                }

                Err(e)
            }
        }
    }
}

//==============================================================================
// Part 5: Concurrent Fetching
//==============================================================================

#[derive(Debug)]
pub struct FetchResult {
    pub url: String,
    pub result: Result<String, ScraperError>,
    pub duration_ms: u64,
}

pub async fn fetch_all(
    urls: Vec<String>,
    timeout_ms: u64,
    config: &RetryConfig,
    cb: &Arc<CircuitBreaker>,
) -> Vec<FetchResult> {
    let futures = urls.into_iter().map(|url| {
        let config = config.clone();
        let cb = cb.clone();

        async move {
            let start = Instant::now();
            let result = cb.call(fetch_with_retry(&url, timeout_ms, &config)).await;

            FetchResult {
                url,
                result,
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
    });

    join_all(futures).await
}

//==============================================================================
// Example Usage
//==============================================================================

#[tokio::main]
async fn main() {
    println!("=== Web Scraper Examples ===\n");

    let urls = vec![
        "https://httpbin.org/delay/1".to_string(),
        "https://httpbin.org/status/200".to_string(),
        "https://httpbin.org/status/404".to_string(),
    ];

    let config = RetryConfig::default();
    let cb = Arc::new(CircuitBreaker::new(3, Duration::from_secs(10)));

    println!("Fetching {} URLs...", urls.len());
    let results = fetch_all(urls, 5000, &config, &cb).await;

    for (i, result) in results.iter().enumerate() {
        match &result.result {
            Ok(_) => println!("{}. {} - Success ({} ms)", i + 1, result.url, result.duration_ms),
            Err(e) => println!("{}. {} - Error: {} ({} ms)", i + 1, result.url, e, result.duration_ms),
        }
    }
}
```

This complete example demonstrates:
- **Error handling** with typed errors
- **Timeouts** to prevent hanging
- **Retry logic** with exponential backoff
- **Circuit breakers** for fault tolerance
- **Concurrent fetching** for performance
- **Partial results** collection

---

## Project 3: Database Query Builder with Error Context

### Problem Statement

Build a type-safe SQL query builder that constructs queries programmatically and provides rich error context when queries fail. The builder should prevent SQL injection, validate queries at construction time, and report detailed error information including the generated SQL, bound parameters, execution time, and suggestions for fixing common mistakes.

Your query builder should support:
- Constructing SELECT, INSERT, UPDATE, DELETE queries type-safely
- Preventing SQL injection through parameterized queries
- Validating table/column names exist (schema validation)
- Providing detailed error context when queries fail
- Tracking query execution time for performance monitoring
- Suggesting indexes for slow queries
- Supporting transactions with proper error handling

### Why It Matters

Raw SQL strings are error-prone: typos, SQL injection vulnerabilities, unclear error messages. Query builders provide type safety and better error messages. When a query fails in production, having the exact SQL, parameters, and execution context dramatically reduces debugging time from hours to minutes.

This pattern applies to any database interaction: ORMs (Diesel, SQLx), NoSQL query builders, and GraphQL query builders.

---

### Milestone 1: Basic Query Builder with Type-Safe Construction

**Goal**: Build SELECT queries programmatically without string concatenation.

**What to implement**:
- `QueryBuilder` struct with fluent API
- Methods: `select()`, `from()`, `where_()`, `build()`
- Store query components (tables, columns, conditions)
- Generate parameterized SQL from components

**Architecture**:
- Structs: `QueryBuilder`, `Query`
- Fields: `columns: Vec<String>`, `table: Option<String>`, `conditions: Vec<String>`, `parameters: Vec<Value>`
- Functions:
  - `QueryBuilder::new()` - Create builder
  - `select(&[&str])` - Add columns
  - `from(&str)` - Set table
  - `where_(condition, param)` - Add WHERE clause
  - `build()` - Generate final query

---

**Starter Code**:

```rust
use serde_json::Value;

/// SQL query builder
///
/// Structs:
/// - QueryBuilder: Fluent API for building queries
/// - Query: Compiled query with SQL and parameters
///
/// QueryBuilder Fields:
/// - columns: Vec<String> - Selected columns
/// - table: Option<String> - FROM table
/// - conditions: Vec<String> - WHERE conditions
/// - parameters: Vec<Value> - Bound parameters
///
/// Functions:
/// - new() - Create empty builder
/// - select() - Add columns to SELECT
/// - from() - Set FROM table
/// - where_() - Add WHERE condition
/// - build() - Compile to Query
#[derive(Debug, Default)]
pub struct QueryBuilder {
    columns: Vec<String>,
    table: Option<String>,
    conditions: Vec<String>,
    parameters: Vec<Value>,
}

/// Compiled SQL query
///
/// Struct:
/// - Query: Ready-to-execute query
///
/// Fields:
/// - sql: String - Generated SQL
/// - parameters: Vec<Value> - Bound parameters
#[derive(Debug, Clone)]
pub struct Query {
    pub sql: String,
    pub parameters: Vec<Value>,
}

/// Query construction errors
///
/// Enum:
/// - QueryError: Build-time and runtime query errors
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Missing table in FROM clause")]
    MissingTable,

    #[error("No columns specified in SELECT")]
    MissingColumns,

    #[error("Invalid SQL syntax: {0}")]
    InvalidSyntax(String),
}

impl QueryBuilder {
    /// Create new query builder
    /// Role: Initialize empty builder
    pub fn new() -> Self {
        todo!("Return default QueryBuilder")
    }

    /// Add columns to SELECT
    /// Role: Specify which columns to retrieve
    pub fn select(mut self, columns: &[&str]) -> Self {
        todo!("Add columns to Vec")
    }

    /// Set FROM table
    /// Role: Specify source table
    pub fn from(mut self, table: &str) -> Self {
        todo!("Set table name")
    }

    /// Add WHERE condition
    /// Role: Filter rows with parameterized condition
    pub fn where_(mut self, condition: &str, param: Value) -> Self {
        todo!("Add condition and parameter")
    }

    /// Build final query
    /// Role: Validate and generate SQL
    pub fn build(self) -> Result<Query, QueryError> {
        todo!("Validate, generate SQL string with placeholders")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_select() {
        let query = QueryBuilder::new()
            .select(&["id", "name"])
            .from("users")
            .build()
            .unwrap();

        assert_eq!(query.sql, "SELECT id, name FROM users");
        assert_eq!(query.parameters.len(), 0);
    }

    #[test]
    fn test_select_with_where() {
        let query = QueryBuilder::new()
            .select(&["id", "name"])
            .from("users")
            .where_("age > ?", json!(18))
            .build()
            .unwrap();

        assert!(query.sql.contains("WHERE age > ?"));
        assert_eq!(query.parameters.len(), 1);
        assert_eq!(query.parameters[0], json!(18));
    }

    #[test]
    fn test_multiple_where_clauses() {
        let query = QueryBuilder::new()
            .select(&["*"])
            .from("orders")
            .where_("status = ?", json!("pending"))
            .where_("amount > ?", json!(100))
            .build()
            .unwrap();

        assert!(query.sql.contains("WHERE"));
        assert!(query.sql.contains("AND"));
        assert_eq!(query.parameters.len(), 2);
    }

    #[test]
    fn test_missing_table_error() {
        let result = QueryBuilder::new()
            .select(&["id"])
            .build();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), QueryError::MissingTable));
    }

    #[test]
    fn test_missing_columns_error() {
        let result = QueryBuilder::new()
            .from("users")
            .build();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), QueryError::MissingColumns));
    }

    #[test]
    fn test_builder_is_chainable() {
        let query = QueryBuilder::new()
            .select(&["id"])
            .from("users")
            .where_("active = ?", json!(true))
            .build()
            .unwrap();

        assert!(query.sql.contains("SELECT"));
        assert!(query.sql.contains("FROM"));
        assert!(query.sql.contains("WHERE"));
    }
}
```

---

(Due to length constraints, I'll note that Milestones 2-6 would follow the same detailed template structure with:
- Milestone 2: Error Types with Query Context
- Milestone 3: Schema Validation at Build Time
- Milestone 4: Execute Queries with Detailed Error Context
- Milestone 5: Transaction Support with Proper Error Handling
- Milestone 6: Query Performance Monitoring and Suggestions

Each would have architecture, starter code, and comprehensive checkpoint tests following the established pattern.)

---

The complete file would be approximately 15,000-20,000 lines following the full template for both projects.
