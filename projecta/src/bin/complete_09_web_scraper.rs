use futures::future::join_all;
use rand::Rng;
use reqwest::Client;
use std::{
    future::Future,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use thiserror::Error;
use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore},
    time::{sleep, timeout},
};

// =============================================================================
// Milestone 1: Basic Async HTTP Client with Error Types
// =============================================================================

#[derive(Error, Debug, Clone)]
pub enum ScraperError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Request timed out after {0}ms")]
    TimeoutError(u64),

    #[error("HTTP {status} error for {url}")]
    HttpError { status: u16, url: String },

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Circuit breaker is open")]
    CircuitBreakerOpen,

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
}

impl From<reqwest::Error> for ScraperError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            return ScraperError::TimeoutError(0);
        }

        if let Some(status) = err.status() {
            return ScraperError::HttpError {
                status: status.as_u16(),
                url: err.url().map(|u| u.to_string()).unwrap_or_default(),
            };
        }

        ScraperError::NetworkError(err.to_string())
    }
}

pub async fn fetch_url(url: &str) -> Result<String, ScraperError> {
    let client = Client::new();
    let response = client.get(url).send().await.map_err(ScraperError::from)?;

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

// =============================================================================
// Milestone 2: Enforcing Timeouts on Network Operations
// =============================================================================

pub async fn fetch_url_with_timeout(url: &str, timeout_ms: u64) -> Result<String, ScraperError> {
    match timeout(Duration::from_millis(timeout_ms), fetch_url(url)).await {
        Ok(result) => result,
        Err(_) => Err(ScraperError::TimeoutError(timeout_ms)),
    }
}

pub fn create_client(timeout_ms: u64) -> Client {
    Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .expect("HTTP client")
}

// =============================================================================
// Milestone 3: Retry with Exponential Backoff + Jitter
// =============================================================================

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: usize,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub jitter: bool,
}

impl RetryConfig {
    pub fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 1_000,
            max_backoff_ms: 30_000,
            jitter: true,
        }
    }

    pub fn backoff_duration(&self, attempt: usize) -> Duration {
        let multiplier = 1u64
            .checked_shl(attempt as u32)
            .unwrap_or(u64::MAX);
        let base = self.initial_backoff_ms.saturating_mul(multiplier);
        let capped = base.min(self.max_backoff_ms);
        let millis = if self.jitter {
            let mut rng = rand::thread_rng();
            (capped as f64 * rng.gen_range(0.9..1.1)) as u64
        } else {
            capped
        };
        Duration::from_millis(millis)
    }
}

impl ScraperError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ScraperError::NetworkError(_)
                | ScraperError::TimeoutError(_)
                | ScraperError::HttpError { status: 500..=599, .. }
        )
    }
}

struct RetryOutcome {
    result: Result<String, ScraperError>,
    attempt_count: usize,
}

async fn fetch_with_retry_internal(
    client: &Client,
    url: &str,
    timeout_ms: u64,
    retry_config: &RetryConfig,
) -> RetryOutcome {
    let mut attempt = 0;
    let max_attempts = retry_config.max_attempts.max(1);

    loop {
        attempt += 1;
        let future = async {
            let response = client.get(url).send().await.map_err(ScraperError::from)?;
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
        };
        let result = match timeout(Duration::from_millis(timeout_ms), future).await {
            Ok(output) => output,
            Err(_) => Err(ScraperError::TimeoutError(timeout_ms)),
        };

        match result {
            Ok(body) => {
                return RetryOutcome {
                    result: Ok(body),
                    attempt_count: attempt,
                }
            }
            Err(err) => {
                if attempt >= max_attempts || !err.is_retryable() {
                    return RetryOutcome {
                        result: Err(err),
                        attempt_count: attempt,
                    };
                }

                let delay = retry_config.backoff_duration(attempt - 1);
                sleep(delay).await;
            }
        }
    }
}

pub async fn fetch_with_retry(
    url: &str,
    timeout_ms: u64,
    retry_config: &RetryConfig,
) -> Result<String, ScraperError> {
    let client = create_client(timeout_ms);
    fetch_with_retry_internal(&client, url, timeout_ms, retry_config)
        .await
        .result
}

// =============================================================================
// Milestone 4: Circuit Breaker Pattern
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
}

pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_threshold: usize,
    success_threshold: usize,
    timeout: Duration,
    consecutive_failures: Arc<Mutex<usize>>,
    consecutive_successes: Arc<Mutex<usize>>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, timeout: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_threshold: failure_threshold.max(1),
            success_threshold: 1,
            timeout,
            consecutive_failures: Arc::new(Mutex::new(0)),
            consecutive_successes: Arc::new(Mutex::new(0)),
        }
    }

    pub fn state(&self) -> CircuitState {
        self.state.lock().unwrap().clone()
    }

    fn should_attempt(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        match *state {
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() >= self.timeout {
                    *state = CircuitState::HalfOpen;
                    true
                } else {
                    false
                }
            }
            _ => true,
        }
    }

    pub async fn call<F, T>(&self, f: F) -> Result<T, ScraperError>
    where
        F: Future<Output = Result<T, ScraperError>>,
    {
        if !self.should_attempt() {
            return Err(ScraperError::CircuitBreakerOpen);
        }

        match f.await {
            Ok(value) => {
                self.on_success();
                Ok(value)
            }
            Err(err) => {
                self.on_failure();
                Err(err)
            }
        }
    }

    fn on_success(&self) {
        *self.consecutive_failures.lock().unwrap() = 0;
        let mut successes = self.consecutive_successes.lock().unwrap();
        *successes += 1;

        let mut state = self.state.lock().unwrap();
        if matches!(*state, CircuitState::HalfOpen) && *successes >= self.success_threshold {
            *state = CircuitState::Closed;
            *successes = 0;
        }
    }

    fn on_failure(&self) {
        *self.consecutive_successes.lock().unwrap() = 0;
        let mut failures = self.consecutive_failures.lock().unwrap();
        *failures += 1;

        if *failures >= self.failure_threshold {
            let mut state = self.state.lock().unwrap();
            *state = CircuitState::Open {
                opened_at: Instant::now(),
            };
            *failures = 0;
        }
    }
}

// =============================================================================
// Milestone 5: Concurrent Fetching + Partial Results
// =============================================================================

#[derive(Debug)]
pub struct FetchResult {
    pub url: String,
    pub result: Result<String, ScraperError>,
    pub duration_ms: u64,
    pub attempt_count: usize,
}

impl FetchResult {
    pub fn is_success(&self) -> bool {
        self.result.is_ok()
    }

    pub fn content(&self) -> Option<&str> {
        self.result.as_ref().ok().map(|s| s.as_str())
    }
}

#[derive(Debug)]
pub struct FetchSummary {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub total_duration_ms: u64,
    pub avg_duration_ms: u64,
}

impl FetchSummary {
    pub fn from_results(results: &[FetchResult]) -> Self {
        let total = results.len();
        let success = results.iter().filter(|r| r.is_success()).count();
        let failed = total.saturating_sub(success);
        let total_duration_ms: u64 = results.iter().map(|r| r.duration_ms).sum();
        let avg_duration_ms = if total > 0 {
            total_duration_ms / total as u64
        } else {
            0
        };

        Self {
            total,
            success,
            failed,
            total_duration_ms,
            avg_duration_ms,
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.success as f64 / self.total as f64) * 100.0
        }
    }
}

pub async fn fetch_all(
    urls: Vec<String>,
    timeout_ms: u64,
    retry_config: &RetryConfig,
    circuit_breaker: &Arc<CircuitBreaker>,
) -> Vec<FetchResult> {
    let client = create_client(timeout_ms);

    let futures = urls.into_iter().map(|url| {
        let client = client.clone();
        let retry = retry_config.clone();
        let breaker = circuit_breaker.clone();

        async move {
            let start = Instant::now();
            let attempts = Arc::new(Mutex::new(0_usize));
            let attempts_inner = attempts.clone();

            let result = breaker
                .call(async {
                    let outcome = fetch_with_retry_internal(&client, &url, timeout_ms, &retry).await;
                    *attempts_inner.lock().unwrap() = outcome.attempt_count;
                    outcome.result
                })
                .await;

            let attempt_count = {
                let guard = attempts.lock().unwrap();
                *guard
            };

            FetchResult {
                url,
                result,
                duration_ms: start.elapsed().as_millis() as u64,
                attempt_count,
            }
        }
    });

    join_all(futures).await
}

pub async fn fetch_all_with_limit(
    urls: Vec<String>,
    timeout_ms: u64,
    retry_config: &RetryConfig,
    circuit_breaker: &Arc<CircuitBreaker>,
    max_concurrent: usize,
) -> Vec<FetchResult> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent.max(1)));
    let client = create_client(timeout_ms);

    let futures = urls.into_iter().map(|url| {
        let semaphore = semaphore.clone();
        let client = client.clone();
        let retry = retry_config.clone();
        let breaker = circuit_breaker.clone();

        async move {
            let _permit = semaphore.acquire_owned().await.expect("permit");
            let start = Instant::now();
            let attempts = Arc::new(Mutex::new(0_usize));
            let attempts_inner = attempts.clone();

            let result = breaker
                .call(async {
                    let outcome = fetch_with_retry_internal(&client, &url, timeout_ms, &retry).await;
                    *attempts_inner.lock().unwrap() = outcome.attempt_count;
                    outcome.result
                })
                .await;

            let attempt_count = {
                let guard = attempts.lock().unwrap();
                *guard
            };

            FetchResult {
                url,
                result,
                duration_ms: start.elapsed().as_millis() as u64,
                attempt_count,
            }
        }
    });

    join_all(futures).await
}

// =============================================================================
// Milestone 6: Rate Limiting and Resource Management
// =============================================================================

#[derive(Clone)]
pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
    rate_per_second: f64,
    last_request: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    pub fn new(max_concurrent: usize, rate_per_second: f64) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent.max(1))),
            rate_per_second: rate_per_second.max(0.1),
            last_request: Arc::new(Mutex::new(Instant::now() - Duration::from_secs(1))),
        }
    }

    pub async fn acquire(&self) -> OwnedSemaphorePermit {
        let permit = self.semaphore.clone().acquire_owned().await.expect("permit");
        let interval = Duration::from_secs_f64(1.0 / self.rate_per_second);
        let mut last = self.last_request.lock().unwrap();
        let now = Instant::now();
        let next_allowed = *last + interval;
        if now < next_allowed {
            sleep(next_allowed - now).await;
        }
        *last = Instant::now();
        permit
    }
}

pub struct ResourceManager {
    active_requests: Arc<Mutex<usize>>,
    total_bytes: Arc<Mutex<u64>>,
    max_memory_bytes: u64,
}

impl ResourceManager {
    pub fn new(max_memory_bytes: u64) -> Self {
        Self {
            active_requests: Arc::new(Mutex::new(0)),
            total_bytes: Arc::new(Mutex::new(0)),
            max_memory_bytes,
        }
    }

    pub fn can_proceed(&self) -> bool {
        *self.total_bytes.lock().unwrap() <= self.max_memory_bytes
    }

    pub fn start_request(&self) {
        *self.active_requests.lock().unwrap() += 1;
    }

    pub fn end_request(&self, bytes: u64) {
        let mut active = self.active_requests.lock().unwrap();
        if *active > 0 {
            *active -= 1;
        }
        *self.total_bytes.lock().unwrap() += bytes;
    }
}

pub async fn fetch_all_managed(
    urls: Vec<String>,
    timeout_ms: u64,
    retry_config: &RetryConfig,
    circuit_breaker: &Arc<CircuitBreaker>,
    rate_limiter: &RateLimiter,
    resource_manager: &Arc<ResourceManager>,
) -> Vec<FetchResult> {
    let client = create_client(timeout_ms);

    let futures = urls.into_iter().map(|url| {
        let client = client.clone();
        let retry = retry_config.clone();
        let breaker = circuit_breaker.clone();
        let limiter = rate_limiter.clone();
        let manager = resource_manager.clone();

        async move {
            let permit = limiter.acquire().await;
            if !manager.can_proceed() {
                drop(permit);
                return FetchResult {
                    url,
                    result: Err(ScraperError::ResourceLimitExceeded(
                        "memory limit reached".to_string(),
                    )),
                    duration_ms: 0,
                    attempt_count: 0,
                };
            }

            manager.start_request();
            let start = Instant::now();
            let attempts = Arc::new(Mutex::new(0_usize));
            let attempts_inner = attempts.clone();

            let result = breaker
                .call(async {
                    let outcome = fetch_with_retry_internal(&client, &url, timeout_ms, &retry).await;
                    *attempts_inner.lock().unwrap() = outcome.attempt_count;
                    outcome.result
                })
                .await;

            let attempt_count = {
                let guard = attempts.lock().unwrap();
                *guard
            };

            let bytes = result.as_ref().ok().map(|body| body.len() as u64).unwrap_or(0);
            manager.end_request(bytes);
            drop(permit);

            FetchResult {
                url,
                result,
                duration_ms: start.elapsed().as_millis() as u64,
                attempt_count,
            }
        }
    });

    join_all(futures).await
}

// =============================================================================
// Tests for All Milestones
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::Duration;
    use wiremock::{matchers::{method, path}, Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_fetch_url_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_string("Hello"))
            .mount(&server)
            .await;

        let url = format!("{}/test", server.uri());
        let result = fetch_url(&url).await.unwrap();
        assert_eq!(result, "Hello");
    }

    #[tokio::test]
    async fn test_fetch_url_404() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let err = fetch_url(&server.uri()).await.unwrap_err();
        match err {
            ScraperError::HttpError { status, .. } => assert_eq!(status, 404),
            _ => panic!("expected HTTP error"),
        }
    }

    #[tokio::test]
    async fn test_fetch_url_with_timeout_triggers() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
            .mount(&server)
            .await;

        let err = fetch_url_with_timeout(&server.uri(), 50).await.unwrap_err();
        match err {
            ScraperError::TimeoutError(ms) => assert_eq!(ms, 50),
            _ => panic!("expected timeout"),
        }
    }

    #[test]
    fn test_retry_config_backoff() {
        let cfg = RetryConfig {
            max_attempts: 5,
            initial_backoff_ms: 100,
            max_backoff_ms: 500,
            jitter: false,
        };
        assert_eq!(cfg.backoff_duration(0).as_millis(), 100);
        assert_eq!(cfg.backoff_duration(1).as_millis(), 200);
        assert_eq!(cfg.backoff_duration(2).as_millis(), 400);
        assert_eq!(cfg.backoff_duration(3).as_millis(), 500);
    }

    #[tokio::test]
    async fn test_fetch_with_retry_succeeds_after_failures() {
        let server = MockServer::start().await;
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_clone = attempts.clone();

        Mock::given(method("GET"))
            .respond_with(move |_req: &wiremock::Request| {
                let count = attempts_clone.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    ResponseTemplate::new(500)
                } else {
                    ResponseTemplate::new(200).set_body_string("ok")
                }
            })
            .mount(&server)
            .await;

        let cfg = RetryConfig {
            max_attempts: 3,
            initial_backoff_ms: 10,
            max_backoff_ms: 100,
            jitter: false,
        };

        let result = fetch_with_retry(&server.uri(), 1000, &cfg).await.unwrap();
        assert_eq!(result, "ok");
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_circuit_breaker_flow() {
        let breaker = Arc::new(CircuitBreaker::new(2, Duration::from_millis(100)));

        for _ in 0..2 {
            let _ = breaker
                .call(async { Err::<(), _>(ScraperError::NetworkError("fail".into())) })
                .await;
        }

        match breaker.state() {
            CircuitState::Open { .. } => {}
            _ => panic!("expected open"),
        }

        tokio::time::sleep(Duration::from_millis(120)).await;
        let res = breaker
            .call(async { Ok::<_, ScraperError>("ok") })
            .await
            .unwrap();
        assert_eq!(res, "ok");
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_fetch_all_partial_results() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/good"))
            .respond_with(ResponseTemplate::new(200).set_body_string("good"))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/bad"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let urls = vec![
            format!("{}/good", server.uri()),
            format!("{}/bad", server.uri()),
        ];
        let cfg = RetryConfig {
            max_attempts: 1,
            ..RetryConfig::default()
        };
        let breaker = Arc::new(CircuitBreaker::new(5, Duration::from_secs(1)));

        let results = fetch_all(urls, 1_000, &cfg, &breaker).await;
        assert_eq!(results.len(), 2);
        assert!(results[0].is_success());
        assert!(!results[1].is_success());
    }

    #[tokio::test]
    async fn test_fetch_all_with_limit_respects_limit() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200).set_delay(Duration::from_millis(100)),
            )
            .mount(&server)
            .await;

        let urls: Vec<_> = (0..6).map(|i| format!("{}/{}", server.uri(), i)).collect();
        let cfg = RetryConfig::default();
        let breaker = Arc::new(CircuitBreaker::new(10, Duration::from_secs(1)));

        let start = Instant::now();
        let _results = fetch_all_with_limit(urls, 1_000, &cfg, &breaker, 2).await;
        assert!(start.elapsed().as_millis() >= 300);
    }

    #[test]
    fn test_fetch_summary() {
        let results = vec![
            FetchResult {
                url: "a".into(),
                result: Ok("1".into()),
                duration_ms: 100,
                attempt_count: 1,
            },
            FetchResult {
                url: "b".into(),
                result: Err(ScraperError::NetworkError("fail".into())),
                duration_ms: 200,
                attempt_count: 2,
            },
        ];

        let summary = FetchSummary::from_results(&results);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.success, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.avg_duration_ms, 150);
        assert!((summary.success_rate() - 50.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_rate_limiter_enforces_rate() {
        let limiter = RateLimiter::new(2, 5.0);
        let start = Instant::now();

        for _ in 0..5 {
            let _permit = limiter.acquire().await;
        }

        assert!(start.elapsed().as_millis() >= 800);
    }

    #[tokio::test]
    async fn test_resource_manager_tracks_usage() {
        let manager = Arc::new(ResourceManager::new(1_000));
        assert!(manager.can_proceed());
        manager.start_request();
        manager.end_request(500);
        assert!(manager.can_proceed());
        manager.start_request();
        manager.end_request(600);
        assert!(!manager.can_proceed());
    }

    #[tokio::test]
    async fn test_fetch_all_managed() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("data"))
            .mount(&server)
            .await;

        let urls: Vec<_> = (0..3).map(|i| format!("{}/{}", server.uri(), i)).collect();
        let cfg = RetryConfig::default();
        let breaker = Arc::new(CircuitBreaker::new(5, Duration::from_secs(1)));
        let limiter = RateLimiter::new(2, 10.0);
        let manager = Arc::new(ResourceManager::new(10_000));

        let results = fetch_all_managed(urls, 1_000, &cfg, &breaker, &limiter, &manager).await;
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_success()));
    }
}

fn main() {}
