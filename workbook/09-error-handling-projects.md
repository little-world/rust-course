# Chapter 09: Error Handling Architecture - Programming Projects

## Project 1: Configuration Validator with Rich Error Context

### Problem Statement

Build a configuration file validator that parses TOML/JSON configuration files and validates them against a schema with comprehensive error reporting. The validator should collect ALL validation errors (not just the first), provide actionable error messages with suggestions, track error locations (line/column), and help users fix configuration problems quickly.

Your validator should:
- Parse configuration files (TOML or JSON)
- Validate against a schema (required fields, type constraints, value ranges)
- Collect multiple errors in a single validation pass
- Report errors with file location, field path, actual vs expected values
- Suggest fixes for common mistakes (typos, missing required fields)
- Distinguish between parsing errors and validation errors

Example config validation:
```toml
[database]
host = "localhost"
port = "invalid"  # Should be number
max_connections = 1000  # Exceeds maximum of 500

[server]
# Missing required field: address
timeout = -5  # Should be positive
```

### Why It Matters

Configuration errors are among the most frustrating bugs in production systems. Poor error messages lead to trial-and-error debugging, wasting developer time. Good error handling in configuration validation:
- Catches all problems before deployment
- Provides actionable feedback (not just "invalid config")
- Suggests corrections for common mistakes
- Prevents cascading failures from misconfiguration

This pattern applies to any validation system: API request validation, command-line argument parsing, data import validation, compiler error reporting.

### Use Cases

- Application configuration (web servers, databases, microservices)
- CI/CD pipeline validation (validating .yml files)
- Infrastructure as code (validating Terraform, Kubernetes manifests)
- Form validation in web applications
- Data import validation (CSV, JSON data validation)
- Compiler/linter error reporting

### Solution Outline

#### Step 1: Basic Error Type with thiserror
**Goal**: Define a comprehensive error type for configuration validation.

**What to implement**:
- Define `ConfigError` enum with variants: `ParseError`, `MissingField`, `InvalidType`, `InvalidValue`, `OutOfRange`
- Use `thiserror` to derive `Display` and `Error` traits
- Include context in each variant (field name, location, expected/actual values)
- Implement custom display messages that are user-friendly

**Why this step**: Before handling errors well, you need a proper error type. Using strings would lose type information and make programmatic error handling impossible.

**Testing hint**: Test each error variant displays correctly. Verify error messages are clear and actionable.

```rust
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ConfigError {
    #[error("Failed to parse config file at line {line}, column {col}: {message}")]
    ParseError {
        line: usize,
        col: usize,
        message: String,
    },

    #[error("Missing required field: '{field}' in section [{section}]")]
    MissingField {
        section: String,
        field: String,
        suggestion: Option<String>,  // Suggest similar field names
    },

    #[error("Invalid type for field '{field}': expected {expected}, got {actual}")]
    InvalidType {
        field: String,
        expected: String,
        actual: String,
        location: Location,
    },

    #[error("Invalid value for field '{field}': {reason}")]
    InvalidValue {
        field: String,
        value: String,
        reason: String,
        location: Location,
    },

    #[error("Value {value} for field '{field}' is out of range (min: {min}, max: {max})")]
    OutOfRange {
        field: String,
        value: i64,
        min: i64,
        max: i64,
    },
}

#[derive(Debug, Clone)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}
```

---

#### Step 2: Parse Configuration File with Error Context
**Goal**: Parse TOML/JSON and preserve location information for error reporting.

**What to implement**:
- Use `serde_json` or `toml` crate for parsing
- Wrap parsing errors with file location
- Convert parser errors to your `ConfigError::ParseError`
- Implement `From<ParseError>` for automatic conversion with `?` operator

**Why the previous step is not enough**: Having error types is great, but we need to actually parse files and capture where errors occur.

**What's the improvement**: Preserving location information (line/column) transforms "parse failed" into "parse failed at line 15, column 8: expected comma". This reduces debugging time from minutes to seconds.

**Testing hint**: Test with valid and invalid config files. Verify line/column numbers are accurate. Test with various syntax errors.

```rust
use serde_json::Value;
use std::fs;

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::ParseError {
            line: err.line(),
            col: err.column(),
            message: err.to_string(),
        }
    }
}

pub fn parse_config(path: &str) -> Result<Value, ConfigError> {
    let content = fs::read_to_string(path)
        .map_err(|e| ConfigError::ParseError {
            line: 0,
            col: 0,
            message: format!("Failed to read file: {}", e),
        })?;

    let config: Value = serde_json::from_str(&content)?;  // ? converts via From
    Ok(config)
}
```

---

#### Step 3: Collect Multiple Errors (Don't Fail Fast)
**Goal**: Validate entire configuration and collect ALL errors, not just the first.

**What to implement**:
- Create `ValidationResult<T>` type that accumulates errors
- Validate all fields, collecting errors into a `Vec<ConfigError>`
- Return all errors at once for complete feedback
- Distinguish between fatal errors (can't continue) and validation errors

**Why the previous step is not enough**: Failing on the first error creates a frustrating cycle: fix one error, run again, find next error, repeat. Users want to see all problems at once.

**What's the improvement**: Collecting errors enables a "fix all at once" workflow. Instead of 10 validation cycles, users get all errors in one run. This is 10x faster feedback for complex configurations.

**Testing hint**: Create config with multiple errors. Verify all are reported. Test that validation continues after encountering errors.

```rust
pub struct ValidationErrors {
    errors: Vec<ConfigError>,
}

impl ValidationErrors {
    fn new() -> Self {
        ValidationErrors { errors: Vec::new() }
    }

    fn add(&mut self, error: ConfigError) {
        self.errors.push(error);
    }

    fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    fn into_result<T>(self, value: T) -> Result<T, Vec<ConfigError>> {
        if self.has_errors() {
            Err(self.errors)
        } else {
            Ok(value)
        }
    }
}

pub fn validate_config(config: &Value) -> Result<ValidatedConfig, Vec<ConfigError>> {
    let mut errors = ValidationErrors::new();

    // Validate all fields, accumulating errors
    if let Err(e) = validate_database_section(config) {
        errors.add(e);
    }

    if let Err(e) = validate_server_section(config) {
        errors.add(e);
    }

    // Continue validating other sections...

    errors.into_result(ValidatedConfig::from(config))
}
```

---

#### Step 4: Add Suggestions for Common Mistakes
**Goal**: Enhance error messages with actionable suggestions using string similarity.

**What to implement**:
- Implement Levenshtein distance or use `strsim` crate
- Suggest similar field names for typos ("did you mean 'timeout'?")
- Suggest common fixes for value errors ("use positive number")
- Add context about where to find documentation

**Why the previous step is not enough**: Reporting errors is good, but users still need to figure out how to fix them. Suggestions transform "something is wrong" into "here's how to fix it".

**What's the improvement**: Suggestions reduce cognitive load. Instead of users searching documentation or source code, the validator tells them exactly what to do. This is especially valuable for infrequent users or large configuration schemas.

**Testing hint**: Test typo suggestions (e.g., "timout" suggests "timeout"). Verify suggestions only appear when similarity is high enough. Test with no similar fields.

```rust
use strsim::levenshtein;

fn find_similar_field(typo: &str, valid_fields: &[&str]) -> Option<String> {
    let mut best_match = None;
    let mut best_distance = usize::MAX;

    for &field in valid_fields {
        let distance = levenshtein(typo, field);
        if distance < best_distance && distance <= 2 {  // Max 2 edits
            best_distance = distance;
            best_match = Some(field.to_string());
        }
    }

    best_match
}

fn validate_field_exists(
    config: &Value,
    section: &str,
    field: &str,
    valid_fields: &[&str],
) -> Result<(), ConfigError> {
    if config[section].get(field).is_none() {
        let suggestion = find_similar_field(field, valid_fields);
        return Err(ConfigError::MissingField {
            section: section.to_string(),
            field: field.to_string(),
            suggestion,
        });
    }
    Ok(())
}
```

---

#### Step 5: Type-Safe Schema Validation with Builder Pattern
**Goal**: Create a fluent API for defining validation schemas.

**What to implement**:
- `SchemaBuilder` for defining required/optional fields
- Field validators (type, range, regex, custom predicates)
- Chainable validation rules
- Schema can be defined once and reused

**Why the previous step is not enough**: Hardcoding validation logic is brittle and verbose. Each new field requires code changes. A schema-driven approach is more maintainable.

**What's the improvement**: Schema-based validation separates "what to validate" from "how to validate". Adding a new field is configuration, not code. Schemas can be serialized, versioned, and shared. This is essential for systems where non-programmers define validation rules.

**Testing hint**: Define schema for test config. Verify all rules are enforced. Test required vs optional fields. Test range validation.

```rust
pub struct Schema {
    fields: Vec<FieldSchema>,
}

pub struct FieldSchema {
    path: String,  // e.g., "database.port"
    required: bool,
    validator: Box<dyn Fn(&Value) -> Result<(), String>>,
}

pub struct SchemaBuilder {
    fields: Vec<FieldSchema>,
}

impl SchemaBuilder {
    pub fn new() -> Self {
        SchemaBuilder { fields: Vec::new() }
    }

    pub fn required_field(mut self, path: &str, validator: impl Fn(&Value) -> Result<(), String> + 'static) -> Self {
        self.fields.push(FieldSchema {
            path: path.to_string(),
            required: true,
            validator: Box::new(validator),
        });
        self
    }

    pub fn optional_field(mut self, path: &str, validator: impl Fn(&Value) -> Result<(), String> + 'static) -> Self {
        self.fields.push(FieldSchema {
            path: path.to_string(),
            required: false,
            validator: Box::new(validator),
        });
        self
    }

    pub fn build(self) -> Schema {
        Schema { fields: self.fields }
    }
}

// Usage:
let schema = SchemaBuilder::new()
    .required_field("database.host", |v| {
        v.as_str().ok_or_else(|| "must be string".to_string())?;
        Ok(())
    })
    .required_field("database.port", |v| {
        let port = v.as_i64().ok_or_else(|| "must be integer".to_string())?;
        if port < 1 || port > 65535 {
            return Err("must be between 1 and 65535".to_string());
        }
        Ok(())
    })
    .build();
```

---

#### Step 6: Formatted Error Output with Color and Context
**Goal**: Pretty-print errors with colors, context snippets, and helpful formatting.

**What to implement**:
- Use `colored` or `termcolor` crate for colored output
- Show snippet of config file around error location
- Highlight problematic line with arrow/caret
- Group errors by section
- Add summary at the end (e.g., "Found 5 errors in 2 sections")

**Why the previous step is not enough**: A list of error structs is machine-readable but not user-friendly. Developers need visual, scannable output.

**What's the improvement**: Visual formatting with colors and context makes errors instantly understandable. Compare:
- Plain: `Error at line 15: invalid port value`
- Formatted: Shows line 15 highlighted in red with arrow pointing to error, shows expected vs actual in different colors, adds suggestion in cyan

This is the difference between "usable" and "delightful" developer experience.

**Testing hint**: Manually test output appearance. Verify colors work in terminal. Test with NO_COLOR environment variable. Test grouped output.

```rust
use colored::*;

pub fn format_errors(errors: &[ConfigError], file_content: &str) -> String {
    let lines: Vec<&str> = file_content.lines().collect();
    let mut output = String::new();

    output.push_str(&format!("\n{}\n", "Configuration Validation Errors:".red().bold()));
    output.push_str(&format!("{}\n\n", "=".repeat(50)));

    for (i, error) in errors.iter().enumerate() {
        output.push_str(&format!("{}. ", i + 1));

        match error {
            ConfigError::ParseError { line, col, message } => {
                output.push_str(&format!("{}\n", message.red()));

                // Show context
                if *line > 0 && *line <= lines.len() {
                    let line_content = lines[*line - 1];
                    output.push_str(&format!("   {} | {}\n", line.to_string().blue(), line_content));
                    output.push_str(&format!("   {} | {}{}\n",
                        " ".repeat(line.to_string().len()),
                        " ".repeat(*col),
                        "^".red().bold()
                    ));
                }
            }
            ConfigError::MissingField { section, field, suggestion } => {
                output.push_str(&format!(
                    "{}\n   Section: {}\n   Field: {}\n",
                    "Missing required field".red(),
                    section.yellow(),
                    field.yellow().bold()
                ));

                if let Some(similar) = suggestion {
                    output.push_str(&format!(
                        "   {}: Did you mean '{}'?\n",
                        "Suggestion".cyan(),
                        similar.green()
                    ));
                }
            }
            ConfigError::InvalidValue { field, value, reason, .. } => {
                output.push_str(&format!(
                    "{}\n   Field: {}\n   Value: {}\n   Reason: {}\n",
                    "Invalid value".red(),
                    field.yellow(),
                    value.yellow().bold(),
                    reason
                ));
            }
            _ => output.push_str(&format!("{}\n", error.to_string().red())),
        }

        output.push_str("\n");
    }

    output.push_str(&format!(
        "\n{} {} error{} found\n",
        "Summary:".bold(),
        errors.len(),
        if errors.len() == 1 { "" } else { "s" }
    ));

    output
}
```

---

### Testing Strategies

1. **Unit Tests**: Test each validator independently
2. **Integration Tests**: Test complete validation pipeline with various configs
3. **Golden Tests**: Store expected error output and compare
4. **Fuzzing**: Use `cargo-fuzz` to find edge cases in parsing
5. **Error Coverage**: Ensure every error variant is tested
6. **User Testing**: Have someone unfamiliar with the code try to fix validation errors

---

## Project 2: Async Web Scraper with Retry Logic and Circuit Breaker

### Problem Statement

Build a robust asynchronous web scraper that fetches data from multiple URLs concurrently with sophisticated error handling. The scraper must handle transient failures (timeouts, network errors) with exponential backoff retry, prevent cascading failures with circuit breakers, and aggregate results from parallel operations.

Your scraper should:
- Fetch multiple URLs concurrently (using `tokio` or `async-std`)
- Retry failed requests with exponential backoff (up to N attempts)
- Implement circuit breaker pattern (open/half-open/closed states)
- Handle timeouts on all network operations
- Collect partial results (some URLs may fail permanently)
- Rate-limit requests to avoid overwhelming servers
- Track and report error statistics

### Why It Matters

Network operations are inherently unreliable. Without proper error handling:
- Transient failures cause total failure (should retry)
- Slow/failing services cause cascading failures across the system
- Timeout misconfigurations lead to resource exhaustion
- Hard failures on partial success lose valuable data

This pattern is fundamental to distributed systems: microservices, API clients, data pipelines, monitoring systems. Circuit breakers prevent thundering herd problems and allow failing services to recover.

### Use Cases

- Web scraping (data collection, monitoring, testing)
- Microservice communication with fallbacks
- API clients with retry logic
- Distributed batch processing
- Real-time data aggregation from multiple sources
- Health checking and monitoring systems

### Solution Outline

#### Step 1: Basic Async HTTP Client with Error Types
**Goal**: Create async HTTP client with typed errors.

**What to implement**:
- Use `reqwest` for HTTP requests
- Define `ScraperError` enum: `NetworkError`, `TimeoutError`, `ParseError`, `HttpError(status_code)`
- Implement async `fetch_url()` function
- Convert `reqwest` errors to your error type

**Why this step**: Foundation for async error handling. Establishes error taxonomy for network operations.

**Testing hint**: Use `httpbin.org` or local test server. Test various status codes. Test network errors by using invalid URLs.

```rust
use reqwest;
use thiserror::Error;
use tokio;

#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Request timed out after {0}ms")]
    TimeoutError(u64),

    #[error("HTTP error: status {status}")]
    HttpError { status: u16, url: String },

    #[error("Failed to parse response: {0}")]
    ParseError(String),
}

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

    let body = response.text()
        .await
        .map_err(|e| ScraperError::ParseError(e.to_string()))?;

    Ok(body)
}
```

---

#### Step 2: Add Timeout to Prevent Hanging
**Goal**: Ensure all network operations have bounded execution time.

**What to implement**:
- Add timeout to HTTP requests using `tokio::time::timeout`
- Configure reasonable timeout (e.g., 30 seconds)
- Convert timeout errors to `ScraperError::TimeoutError`
- Make timeout configurable

**Why the previous step is not enough**: Without timeouts, slow or hanging servers can block your application indefinitely, causing resource exhaustion.

**What's the improvement**: Timeouts guarantee bounded waiting time. If a server takes >30s, you fail fast and move on. This prevents one slow endpoint from blocking 100 other requests. Essential for responsive systems.

**Testing hint**: Test with deliberately slow endpoints. Use `tokio::time::sleep` in test server. Verify timeout triggers correctly.

```rust
use tokio::time::{timeout, Duration};

pub async fn fetch_url_with_timeout(
    url: &str,
    timeout_ms: u64,
) -> Result<String, ScraperError> {
    let fetch_future = reqwest::get(url);

    let response = timeout(Duration::from_millis(timeout_ms), fetch_future)
        .await
        .map_err(|_| ScraperError::TimeoutError(timeout_ms))?
        .map_err(|e| ScraperError::NetworkError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(ScraperError::HttpError {
            status: response.status().as_u16(),
            url: url.to_string(),
        });
    }

    let body = timeout(
        Duration::from_millis(timeout_ms),
        response.text()
    )
        .await
        .map_err(|_| ScraperError::TimeoutError(timeout_ms))?
        .map_err(|e| ScraperError::ParseError(e.to_string()))?;

    Ok(body)
}
```

---

#### Step 3: Retry with Exponential Backoff
**Goal**: Automatically retry failed requests with increasing delays.

**What to implement**:
- Retry logic with configurable max attempts
- Exponential backoff: 1s, 2s, 4s, 8s, ...
- Only retry on transient errors (network, timeout), not permanent errors (404, 403)
- Add jitter to prevent thundering herd

**Why the previous step is not enough**: Network errors are often transient. A single retry could succeed, but immediate retry might fail again. We need smart retry strategy.

**What's the improvement**: Exponential backoff gives services time to recover. Jitter prevents synchronized retries from many clients (thundering herd). This turns "50% of requests fail due to transient errors" into "99% success rate with retries".

**Optimization focus**: Reliability through intelligent retry strategy.

**Testing hint**: Test with service that fails N times then succeeds. Verify backoff timing. Test that 404s don't retry.

```rust
use rand::Rng;

pub struct RetryConfig {
    pub max_attempts: usize,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

impl ScraperError {
    fn is_retryable(&self) -> bool {
        matches!(self,
            ScraperError::NetworkError(_) |
            ScraperError::TimeoutError(_) |
            ScraperError::HttpError { status: 500..=599, .. }
        )
    }
}

pub async fn fetch_with_retry(
    url: &str,
    timeout_ms: u64,
    retry_config: &RetryConfig,
) -> Result<String, ScraperError> {
    let mut attempt = 0;
    let mut backoff_ms = retry_config.initial_backoff_ms;

    loop {
        attempt += 1;

        match fetch_url_with_timeout(url, timeout_ms).await {
            Ok(response) => return Ok(response),
            Err(e) if attempt >= retry_config.max_attempts => {
                return Err(e);
            }
            Err(e) if !e.is_retryable() => {
                return Err(e);
            }
            Err(_) => {
                // Add jitter: ±25% randomness
                let jitter = rand::thread_rng().gen_range(0.75..=1.25);
                let sleep_ms = (backoff_ms as f64 * jitter) as u64;

                tokio::time::sleep(Duration::from_millis(sleep_ms)).await;

                // Exponential backoff
                backoff_ms = (backoff_ms * 2).min(retry_config.max_backoff_ms);
            }
        }
    }
}
```

---

#### Step 4: Circuit Breaker Pattern
**Goal**: Prevent repeated calls to failing services.

**What to implement**:
- Circuit breaker with three states: Closed, Open, HalfOpen
- Track failure rate (e.g., if >50% fail, open circuit)
- When open, fail immediately without calling service
- After timeout, transition to half-open (try one request)
- If half-open succeeds, close circuit; if fails, reopen

**Why the previous step is not enough**: Retries help with transient failures, but if a service is completely down, retrying wastes time and resources. Circuit breaker fails fast.

**What's the improvement**: Circuit breakers prevent cascading failures. If service A is down, service B (which depends on A) stops hammering it with requests, allowing A to recover. This prevents resource exhaustion and reduces latency (fail in 1ms instead of waiting 30s for timeout).

**Optimization focus**: Speed (fail fast) and reliability (allow recovery).

**Testing hint**: Test state transitions. Verify circuit opens after threshold failures. Test half-open → closed transition. Test that open circuit fails immediately.

```rust
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};

#[derive(Debug, Clone)]
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
    failures: Arc<Mutex<usize>>,
    successes: Arc<Mutex<usize>>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, timeout: Duration) -> Self {
        CircuitBreaker {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_threshold,
            success_threshold: 2,  // Need 2 successes to close from half-open
            timeout,
            failures: Arc::new(Mutex::new(0)),
            successes: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn call<F, T, E>(&self, f: F) -> Result<T, ScraperError>
    where
        F: Future<Output = Result<T, E>>,
        E: Into<ScraperError>,
    {
        // Check if circuit is open
        {
            let mut state = self.state.lock().unwrap();
            match *state {
                CircuitState::Open { opened_at } => {
                    if opened_at.elapsed() > self.timeout {
                        // Transition to half-open
                        *state = CircuitState::HalfOpen;
                        *self.successes.lock().unwrap() = 0;
                    } else {
                        return Err(ScraperError::NetworkError(
                            "Circuit breaker is open".to_string()
                        ));
                    }
                }
                _ => {}
            }
        }

        // Execute the call
        match f.await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(e) => {
                self.on_failure();
                Err(e.into())
            }
        }
    }

    fn on_success(&self) {
        let mut successes = self.successes.lock().unwrap();
        *successes += 1;
        *self.failures.lock().unwrap() = 0;

        let mut state = self.state.lock().unwrap();
        if matches!(*state, CircuitState::HalfOpen) && *successes >= self.success_threshold {
            *state = CircuitState::Closed;
        }
    }

    fn on_failure(&self) {
        let mut failures = self.failures.lock().unwrap();
        *failures += 1;
        *self.successes.lock().unwrap() = 0;

        if *failures >= self.failure_threshold {
            let mut state = self.state.lock().unwrap();
            *state = CircuitState::Open {
                opened_at: Instant::now(),
            };
        }
    }
}
```

---

#### Step 5: Concurrent Fetching with Partial Results
**Goal**: Fetch multiple URLs concurrently and collect partial results.

**What to implement**:
- Use `tokio::spawn` or `futures::join_all` for concurrent fetches
- Collect results even if some URLs fail
- Return `Vec<Result<String, ScraperError>>` for per-URL results
- Add summary statistics (success count, failure count, timing)

**Why the previous step is not enough**: Sequential fetching is slow. Fetching 100 URLs at 1 second each takes 100 seconds. Concurrent fetching takes ~1 second.

**What's the improvement**: Parallelism provides massive speedup for I/O-bound operations. But naive `try_join_all` fails if ANY request fails. Collecting partial results means "get as much data as possible" rather than "all or nothing".

**Optimization focus**: Speed through parallelism and resilience through partial results.

**Testing hint**: Test with mix of working and failing URLs. Verify concurrent execution (use timing). Test that one failure doesn't block others.

```rust
use futures::future::join_all;

pub struct FetchResult {
    pub url: String,
    pub result: Result<String, ScraperError>,
    pub duration_ms: u64,
}

pub async fn fetch_all(
    urls: &[String],
    timeout_ms: u64,
    retry_config: &RetryConfig,
    circuit_breaker: &Arc<CircuitBreaker>,
) -> Vec<FetchResult> {
    let futures = urls.iter().map(|url| {
        let url = url.clone();
        let cb = circuit_breaker.clone();
        async move {
            let start = Instant::now();
            let result = cb.call(
                fetch_with_retry(&url, timeout_ms, retry_config)
            ).await;
            FetchResult {
                url,
                result,
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
    });

    join_all(futures).await
}

pub struct FetchSummary {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub total_duration_ms: u64,
}

impl FetchSummary {
    pub fn from_results(results: &[FetchResult]) -> Self {
        let total = results.len();
        let success = results.iter().filter(|r| r.result.is_ok()).count();
        let failed = total - success;
        let total_duration_ms = results.iter().map(|r| r.duration_ms).max().unwrap_or(0);

        FetchSummary {
            total,
            success,
            failed,
            total_duration_ms,
        }
    }
}
```

---

#### Step 6: Rate Limiting and Resource Management
**Goal**: Add rate limiting to avoid overwhelming servers.

**What to implement**:
- Token bucket or leaky bucket rate limiter
- Limit concurrent requests (semaphore)
- Track resource usage (memory, connections)
- Graceful shutdown (finish in-flight requests)

**Why the previous step is not enough**: Unlimited concurrent requests can overwhelm target servers (causing 429 errors) or exhaust client resources (memory, file descriptors).

**What's the improvement**: Rate limiting is respectful (doesn't DDoS targets) and prevents resource exhaustion. Semaphore limits concurrent requests: instead of launching 10,000 tasks simultaneously (running out of memory), limit to 100 concurrent tasks.

**Optimization focus**: Resource efficiency and reliability.

**Testing hint**: Test with rate limit lower than request count. Verify requests are throttled. Test semaphore limits concurrency. Monitor memory usage.

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
    rate_per_second: usize,
    last_reset: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    pub fn new(max_concurrent: usize, rate_per_second: usize) -> Self {
        RateLimiter {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            rate_per_second,
            last_reset: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub async fn acquire(&self) -> tokio::sync::SemaphorePermit<'_> {
        // Simple rate limiting: sleep if needed
        let mut last_reset = self.last_reset.lock().unwrap();
        let elapsed = last_reset.elapsed();
        if elapsed < Duration::from_secs(1) {
            let sleep_time = Duration::from_secs(1) - elapsed;
            drop(last_reset);  // Release lock before sleeping
            tokio::time::sleep(sleep_time).await;
            *self.last_reset.lock().unwrap() = Instant::now();
        }

        self.semaphore.acquire().await.unwrap()
    }
}

pub async fn fetch_all_with_rate_limit(
    urls: &[String],
    timeout_ms: u64,
    retry_config: &RetryConfig,
    circuit_breaker: &Arc<CircuitBreaker>,
    rate_limiter: &RateLimiter,
) -> Vec<FetchResult> {
    let futures = urls.iter().map(|url| {
        let url = url.clone();
        let cb = circuit_breaker.clone();
        async move {
            let _permit = rate_limiter.acquire().await;  // Acquire before fetching

            let start = Instant::now();
            let result = cb.call(
                fetch_with_retry(&url, timeout_ms, retry_config)
            ).await;
            FetchResult {
                url,
                result,
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
    });

    join_all(futures).await
}
```

---

### Testing Strategies

1. **Unit Tests**: Test each component (retry, circuit breaker) independently
2. **Integration Tests**: Use real HTTP test server (e.g., `wiremock`)
3. **Chaos Testing**: Randomly fail requests to test resilience
4. **Performance Tests**: Measure concurrent vs sequential performance
5. **Load Tests**: Test with 1000+ URLs
6. **Timeout Tests**: Verify all operations have bounded time

---

## Project 3: Database Query Builder with Error Context

### Problem Statement

Build a type-safe SQL query builder that constructs queries programmatically and provides rich error context when queries fail. The builder should prevent SQL injection, validate queries at construction time, and report detailed error information including the generated SQL, bound parameters, execution time, and suggestions for fixing common mistakes.

Your query builder should:
- Construct SELECT, INSERT, UPDATE, DELETE queries type-safely
- Prevent SQL injection through parameterized queries
- Validate table/column names exist (optional schema validation)
- Provide detailed error context when queries fail
- Track query execution time for performance monitoring
- Suggest indexes for slow queries
- Support transactions with proper error handling

### Why It Matters

Raw SQL strings are error-prone: typos, SQL injection vulnerabilities, unclear error messages. Query builders provide type safety and better error messages. When a query fails in production, having the exact SQL, parameters, and execution context dramatically reduces debugging time from hours to minutes.

This pattern applies to any database interaction: ORMs (like Diesel, SQLx), REST API clients, GraphQL query builders.

### Use Cases

- Web applications with database backends
- Data analysis scripts querying databases
- ETL pipelines transforming data
- Admin tools and dashboards
- API servers with complex queries
- Testing database schemas

### Solution Outline

#### Step 1: Basic Query Builder with Type-Safe Construction
**Goal**: Build SELECT queries programmatically without string concatenation.

**What to implement**:
- `QueryBuilder` struct with methods: `select()`, `from()`, `where_()`, `build()`
- Store query components (tables, columns, conditions)
- Generate SQL string from components
- Prevent empty queries

**Why this step**: Foundation for type-safe query construction. Establishes builder pattern.

**Testing hint**: Test various SELECT queries. Verify generated SQL is correct. Test edge cases (no WHERE clause, multiple columns).

```rust
pub struct QueryBuilder {
    columns: Vec<String>,
    table: Option<String>,
    conditions: Vec<String>,
    parameters: Vec<Value>,
}

impl QueryBuilder {
    pub fn new() -> Self {
        QueryBuilder {
            columns: Vec::new(),
            table: None,
            conditions: Vec::new(),
            parameters: Vec::new(),
        }
    }

    pub fn select(mut self, columns: &[&str]) -> Self {
        self.columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn from(mut self, table: &str) -> Self {
        self.table = Some(table.to_string());
        self
    }

    pub fn where_(mut self, condition: &str, param: Value) -> Self {
        self.conditions.push(condition.to_string());
        self.parameters.push(param);
        self
    }

    pub fn build(self) -> Result<Query, QueryError> {
        let table = self.table.ok_or(QueryError::MissingTable)?;

        if self.columns.is_empty() {
            return Err(QueryError::MissingColumns);
        }

        let mut sql = format!("SELECT {} FROM {}", self.columns.join(", "), table);

        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.conditions.join(" AND "));
        }

        Ok(Query {
            sql,
            parameters: self.parameters,
        })
    }
}

// Usage:
let query = QueryBuilder::new()
    .select(&["id", "name", "email"])
    .from("users")
    .where_("age > ?", Value::Integer(18))
    .build()?;
```

---

#### Step 2: Error Types with Query Context
**Goal**: Define comprehensive error types that include query context.

**What to implement**:
- `QueryError` enum with variants for different failure modes
- Include SQL query text in errors
- Include bound parameters
- Include execution timing
- Use `thiserror` for ergonomic error handling

**Why the previous step is not enough**: Basic query construction works, but when execution fails, you need context to debug.

**What's the improvement**: Error context transforms "query failed" into "query `SELECT * FROM users WHERE age > ?` with params [18] failed after 125ms: table 'users' not found". This provides everything needed for debugging without looking at logs or source code.

**Testing hint**: Create errors and verify context is preserved. Test Display implementation is readable.

```rust
#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Missing table in query")]
    MissingTable,

    #[error("Missing columns in SELECT")]
    MissingColumns,

    #[error("Table '{table}' not found")]
    TableNotFound { table: String },

    #[error("Column '{column}' not found in table '{table}'")]
    ColumnNotFound { table: String, column: String },

    #[error("Query execution failed: {message}\nSQL: {sql}\nParameters: {params:?}\nDuration: {duration_ms}ms")]
    ExecutionError {
        sql: String,
        params: Vec<String>,
        message: String,
        duration_ms: u64,
    },

    #[error("Query timeout after {timeout_ms}ms\nSQL: {sql}")]
    TimeoutError {
        sql: String,
        timeout_ms: u64,
    },

    #[error("SQL injection attempt detected in: {input}")]
    InjectionDetected { input: String },
}

pub struct QueryContext {
    pub sql: String,
    pub parameters: Vec<Value>,
    pub execution_time_ms: Option<u64>,
}

impl QueryContext {
    pub fn into_error(self, message: String) -> QueryError {
        QueryError::ExecutionError {
            sql: self.sql,
            params: self.parameters.iter().map(|v| format!("{:?}", v)).collect(),
            message,
            duration_ms: self.execution_time_ms.unwrap_or(0),
        }
    }
}
```

---

#### Step 3: Schema Validation at Build Time
**Goal**: Validate table and column names against schema before execution.

**What to implement**:
- `Schema` struct storing table and column definitions
- Validate table exists in `from()`
- Validate columns exist in `select()`
- Return detailed errors for typos with suggestions

**Why the previous step is not enough**: Runtime SQL errors require executing the query (which might be slow or have side effects). Build-time validation catches errors earlier.

**What's the improvement**: Failing at build time instead of execution time is faster and safer. Testing queries doesn't require database connection. Suggestions help fix typos: "Column 'emial' not found. Did you mean 'email'?"

**Testing hint**: Create schema with known tables/columns. Test validation catches invalid names. Test suggestions for typos.

```rust
use std::collections::{HashMap, HashSet};

pub struct Schema {
    tables: HashMap<String, TableSchema>,
}

pub struct TableSchema {
    columns: HashSet<String>,
}

impl Schema {
    pub fn new() -> Self {
        Schema {
            tables: HashMap::new(),
        }
    }

    pub fn add_table(&mut self, name: &str, columns: &[&str]) {
        self.tables.insert(
            name.to_string(),
            TableSchema {
                columns: columns.iter().map(|s| s.to_string()).collect(),
            },
        );
    }

    pub fn validate_table(&self, table: &str) -> Result<(), QueryError> {
        if !self.tables.contains_key(table) {
            return Err(QueryError::TableNotFound {
                table: table.to_string(),
            });
        }
        Ok(())
    }

    pub fn validate_column(&self, table: &str, column: &str) -> Result<(), QueryError> {
        let table_schema = self.tables.get(table)
            .ok_or_else(|| QueryError::TableNotFound { table: table.to_string() })?;

        if !table_schema.columns.contains(column) {
            return Err(QueryError::ColumnNotFound {
                table: table.to_string(),
                column: column.to_string(),
            });
        }

        Ok(())
    }
}

// Modified QueryBuilder with schema validation
impl QueryBuilder {
    pub fn new_with_schema(schema: &Schema) -> Self {
        QueryBuilder {
            columns: Vec::new(),
            table: None,
            conditions: Vec::new(),
            parameters: Vec::new(),
            schema: Some(schema),
        }
    }

    pub fn from(mut self, table: &str) -> Result<Self, QueryError> {
        if let Some(schema) = &self.schema {
            schema.validate_table(table)?;
        }
        self.table = Some(table.to_string());
        Ok(self)
    }

    pub fn select(mut self, columns: &[&str]) -> Result<Self, QueryError> {
        if let Some(schema) = &self.schema {
            if let Some(table) = &self.table {
                for column in columns {
                    schema.validate_column(table, column)?;
                }
            }
        }
        self.columns = columns.iter().map(|s| s.to_string()).collect();
        Ok(self)
    }
}
```

---

#### Step 4: Execute Queries with Detailed Error Context
**Goal**: Execute queries against real database with error enrichment.

**What to implement**:
- Integrate with database driver (e.g., `rusqlite`, `sqlx`)
- Time query execution
- Catch and enrich database errors with query context
- Add retry logic for transient database errors (connection issues)

**Why the previous step is not enough**: Validation is great, but queries still need to execute against real databases where other errors can occur (constraints, locks, etc.).

**What's the improvement**: Execution timing helps identify slow queries. Error enrichment provides full context. Retry logic handles transient failures (connection drops).

**Testing hint**: Use in-memory SQLite for tests. Test various database errors. Verify timing is accurate. Test retry on connection errors.

```rust
use rusqlite::{Connection, params};
use std::time::Instant;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn execute(&self, query: &Query) -> Result<Vec<Row>, QueryError> {
        let start = Instant::now();

        let mut stmt = self.conn.prepare(&query.sql)
            .map_err(|e| {
                QueryError::ExecutionError {
                    sql: query.sql.clone(),
                    params: query.parameters.iter().map(|v| format!("{:?}", v)).collect(),
                    message: e.to_string(),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            })?;

        let rows = stmt.query_map(params_from_slice(&query.parameters), |row| {
            // Convert row to your Row type
            Ok(Row::from_rusqlite(row))
        })
        .map_err(|e| {
            QueryError::ExecutionError {
                sql: query.sql.clone(),
                params: query.parameters.iter().map(|v| format!("{:?}", v)).collect(),
                message: e.to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
            }
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            QueryError::ExecutionError {
                sql: query.sql.clone(),
                params: query.parameters.iter().map(|v| format!("{:?}", v)).collect(),
                message: e.to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
            }
        })?;

        Ok(rows)
    }
}
```

---

#### Step 5: Transaction Support with Proper Error Handling
**Goal**: Add transaction support with rollback on error.

**What to implement**:
- `Transaction` type that wraps database connection
- `commit()` and `rollback()` methods
- Automatic rollback on drop if not committed
- Execute multiple queries in transaction
- Handle deadlocks and constraint violations

**Why the previous step is not enough**: Single queries are atomic, but complex operations need multiple queries to be atomic together (all succeed or all fail).

**What's the improvement**: Transactions provide ACID guarantees. If any query in a transaction fails, all changes are rolled back, preventing partial updates. This prevents data corruption and inconsistencies.

**Testing hint**: Test commit and rollback. Test automatic rollback on drop. Test constraint violations within transactions.

```rust
pub struct Transaction<'conn> {
    conn: &'conn Connection,
    committed: bool,
}

impl<'conn> Transaction<'conn> {
    pub fn new(conn: &'conn Connection) -> Result<Self, QueryError> {
        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| QueryError::ExecutionError {
                sql: "BEGIN TRANSACTION".to_string(),
                params: vec![],
                message: e.to_string(),
                duration_ms: 0,
            })?;

        Ok(Transaction {
            conn,
            committed: false,
        })
    }

    pub fn execute(&self, query: &Query) -> Result<Vec<Row>, QueryError> {
        // Similar to Database::execute but uses transaction connection
        // ...
    }

    pub fn commit(mut self) -> Result<(), QueryError> {
        self.conn.execute("COMMIT", [])
            .map_err(|e| QueryError::ExecutionError {
                sql: "COMMIT".to_string(),
                params: vec![],
                message: e.to_string(),
                duration_ms: 0,
            })?;
        self.committed = true;
        Ok(())
    }

    pub fn rollback(mut self) -> Result<(), QueryError> {
        self.conn.execute("ROLLBACK", [])
            .map_err(|e| QueryError::ExecutionError {
                sql: "ROLLBACK".to_string(),
                params: vec![],
                message: e.to_string(),
                duration_ms: 0,
            })?;
        self.committed = true;
        Ok(())
    }
}

impl<'conn> Drop for Transaction<'conn> {
    fn drop(&mut self) {
        if !self.committed {
            // Rollback on drop if not explicitly committed
            let _ = self.conn.execute("ROLLBACK", []);
        }
    }
}

// Usage:
let tx = Transaction::new(&conn)?;
tx.execute(&insert_query)?;
tx.execute(&update_query)?;
tx.commit()?;  // Both queries succeed or both fail
```

---

#### Step 6: Query Performance Monitoring and Suggestions
**Goal**: Add performance monitoring and optimization suggestions.

**What to implement**:
- Track slow queries (execution time > threshold)
- Suggest indexes for slow queries
- Detect full table scans
- Log query statistics
- Provide query explanation (EXPLAIN output)

**Why the previous step is not enough**: Queries work, but performance matters. Slow queries can cripple applications. Developers need guidance on optimization.

**What's the improvement**: Automatic slow query detection catches performance problems early. Index suggestions provide actionable optimization steps. EXPLAIN output helps understand query execution plans.

**Optimization focus**: Speed through index suggestions and query optimization.

**Testing hint**: Create queries with and without indexes. Verify slow query detection. Test EXPLAIN output parsing.

```rust
pub struct QueryStats {
    pub sql: String,
    pub execution_time_ms: u64,
    pub rows_returned: usize,
    pub is_slow: bool,
    pub suggestions: Vec<String>,
}

impl Database {
    pub fn execute_with_stats(
        &self,
        query: &Query,
        slow_threshold_ms: u64,
    ) -> Result<(Vec<Row>, QueryStats), QueryError> {
        let start = Instant::now();
        let rows = self.execute(query)?;
        let duration_ms = start.elapsed().as_millis() as u64;

        let is_slow = duration_ms > slow_threshold_ms;
        let mut suggestions = Vec::new();

        if is_slow {
            // Analyze query for optimization opportunities
            if query.sql.contains("WHERE") && !self.has_index_for_query(&query.sql) {
                suggestions.push(
                    "Consider adding an index on the WHERE clause columns".to_string()
                );
            }

            if self.is_full_table_scan(&query.sql)? {
                suggestions.push(
                    "Query is doing a full table scan. Add indexes to improve performance".to_string()
                );
            }
        }

        let stats = QueryStats {
            sql: query.sql.clone(),
            execution_time_ms: duration_ms,
            rows_returned: rows.len(),
            is_slow,
            suggestions,
        };

        Ok((rows, stats))
    }

    fn is_full_table_scan(&self, sql: &str) -> Result<bool, QueryError> {
        let explain_sql = format!("EXPLAIN QUERY PLAN {}", sql);
        let mut stmt = self.conn.prepare(&explain_sql)
            .map_err(|e| QueryError::ExecutionError {
                sql: explain_sql.clone(),
                params: vec![],
                message: e.to_string(),
                duration_ms: 0,
            })?;

        let plan: String = stmt.query_row([], |row| row.get(3))
            .map_err(|e| QueryError::ExecutionError {
                sql: explain_sql,
                params: vec![],
                message: e.to_string(),
                duration_ms: 0,
            })?;

        Ok(plan.contains("SCAN"))
    }
}
```

---

### Testing Strategies

1. **Unit Tests**: Test query building and validation
2. **Integration Tests**: Test against real SQLite database
3. **Property Tests**: Generate random valid queries and verify they execute
4. **Performance Tests**: Benchmark query execution time
5. **Error Tests**: Test all error paths with invalid queries
6. **Transaction Tests**: Test ACID properties

---

## General Testing Guide

### Recommended Testing Tools

1. **Unit Testing**: `cargo test`
2. **HTTP Mocking**: `wiremock` or `mockito` crates
3. **Database Testing**: In-memory SQLite or `testcontainers`
4. **Property Testing**: `proptest` or `quickcheck`
5. **Async Testing**: `tokio::test` macro
6. **Coverage**: `cargo-tarpaulin`

### Example Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_config_validation() {
        let config = r#"{"database": {"port": 5432}}"#;
        let result = validate_config(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_collect_multiple_errors() {
        let config = r#"{"database": {"port": "invalid", "timeout": -5}}"#;
        let result = validate_config(config);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
    }

    #[tokio::test]
    async fn test_retry_on_timeout() {
        let server = wiremock::MockServer::start().await;
        // Setup mock to fail twice then succeed
        let result = fetch_with_retry(&server.uri(), 1000, &retry_config).await;
        assert!(result.is_ok());
    }
}
```

---

These three projects comprehensively cover error handling patterns in Rust, from configuration validation to async networking to database operations, teaching students how to build robust, maintainable systems with excellent error reporting.
