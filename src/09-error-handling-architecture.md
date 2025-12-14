# Error Handling Architecture

Error handling is one of Rust's most carefully designed features. Unlike exceptions in languages like Java or Python, Rust uses explicit return types (`Result<T, E>`) to force handling of errors at compile time. This approach eliminates entire classes of bugs: forgotten error checks, unexpected exception propagation, and unclear error boundaries.

For programmers, mastering Rust's error handling means understanding not just the mechanics of `Result` and `Option`, but the architectural patterns for designing error types that are ergonomic, composable, and performant. This chapter covers error handling from the ground up: from simple library functions to complex distributed systems with rich error context.

The key insight is that error handling in Rust is not just about reporting failures—it's about encoding your program's error domain in the type system, making impossible states unrepresentable, and providing excellent diagnostics when things go wrong.

## Pattern 1: Custom Error Enums for Libraries

**Problem**: Returning simple strings or a generic `Box<dyn Error>` from a library is not ideal. A `String` error loses all type information, making it impossible for a caller to programmatically handle different kinds of failures.

**Solution**: Define a custom `enum` for your library's errors. Each variant of the enum represents a distinct failure mode.

**Why It Matters**: A custom error enum makes your library's API transparent and robust. It allows users to `match` on specific error variants and handle them appropriately—for example, retrying a `Timeout` error but aborting on a `PermissionDenied` error.

**Use Cases**:
-   Public libraries where callers need to distinguish between different failure modes.
-   Systems with complex or domain-specific errors, such as network protocols, database clients, or parsers.
-   Any situation where an error is an expected part of the program's control flow.

### Example 1: A Simple Custom Error Enum

This shows a basic error enum for a parsing function. Each variant represents a specific reason the parsing could fail.

```rust
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("the input was empty")]
    EmptyInput,
    #[error("the input format was invalid")]
    InvalidFormat,
    #[error("the number was too large")]
    NumberTooLarge,
}

fn parse_number(input: &str) -> Result<i32, ParseError> {
    if input.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    input.parse().map_err(|_| ParseError::InvalidFormat)
}

#[derive(Error, Debug)]
pub enum DataError {
    #[error("failed to read data")]
    Io(#[from] io::Error),
    #[error("failed to parse number")]
    Parse(#[from] ParseIntError),
}

fn load_and_parse_number(path: &str) -> Result<i32, DataError> {
    // The `?` operator will automatically convert `io::Error` and `ParseIntError`
    // into `DataError` because of the `#[from]` attributes.
    let content = std::fs::read_to_string(path)?;
    let number = content.trim().parse()?;
    Ok(number)
}
```

### Example 3: `#[non_exhaustive]` for Library Stability

When publishing a library, you may want to add new error variants in the future without it being a breaking change. The `#[non_exhaustive]` attribute tells users of your library that they must include a wildcard `_` arm in their match statements, ensuring their code won't break if you add a new variant.

```rust
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("the network request failed")]
    NetworkError,
    #[error("the request timed out")]
    Timeout,
    // A new variant could be added here in a future version.
}
```

## Pattern 2: `anyhow` for Application-Level Errors

**Problem**: In application code (as opposed to library code), you often don't need to handle each specific error type. Your main goal is to understand *why* an operation failed and report it to the user or a logging service.

**Solution**: Use the `anyhow` crate. `anyhow::Result` is a type alias for `Result<T, anyhow::Error>`, where `anyhow::Error` is a dynamic error type that can hold any error that implements `std::error::Error`.

**Why It Matters**: `anyhow` provides the convenience of a single, easy-to-use error type for your application while preserving the full chain of underlying causes. It strikes a balance between ease of use and detailed diagnostics, which is perfect for the top levels of an application.

**Use Cases**:
-   The main logic of CLI tools, web servers, and other applications.
-   Any situation where you care more about the error *message* and *context* than the specific error *type*.
-   Prototyping and writing examples where detailed error handling is not the main focus.

### Example 1: Using `anyhow::Result` and `context`

This function shows how `anyhow` can be used to handle errors from different libraries (`std::fs` and `serde_json`) and add context to them.

```rust
use anyhow::{Context, Result};

struct Config;

//===============================================
// Pattern: Structured error with multiple fields
//===============================================
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

//==========================================
// Pattern: Error builder for complex errors
//==========================================
pub struct HttpErrorBuilder {
    method: String,
    url: String,
    status: u16,
    body: Option<String>,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl HttpErrorBuilder {
    pub fn new(method: impl Into<String>, url: impl Into<String>, status: u16) -> Self {
        HttpErrorBuilder {
            method: method.into(),
            url: url.into(),
            status,
            body: None,
            source: None,
        }
    }

    pub fn body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    pub fn source(mut self, err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        self.source = Some(err);
        self
    }

    pub fn build(self) -> HttpError {
        HttpError {
            method: self.method,
            url: self.url,
            status: self.status,
            body: self.body,
            source: self.source,
        }
    }
}
```

**Error type design principles:**
1. **Specific variants**: Each error variant represents a distinct failure mode
2. **Include context**: Path, line number, user input that caused error
3. **Chain sources**: Preserve underlying errors via `source()`
4. **Display for users, Debug for developers**: Display should be readable, Debug should be complete
5. **Non-exhaustive for libraries**: Allow adding variants without breaking changes
6. **Derive when possible**: Use thiserror to reduce boilerplate

**Library vs Application errors:**
- **Libraries**: Specific error types, no context loss, caller decides handling
- **Applications**: Opaque errors (anyhow), focus on diagnostics, fail fast

## Pattern 2: Error Propagation Strategies

**Problem**: Explicit error handling with `match` and `if let` at every fallible call creates deeply nested code and obscures business logic. Transforming errors manually (wrapping `io::Error` in your `AppError`) is repetitive.

**Solution**: Use the `?` operator for concise error propagation—it early-returns `Err` and unwraps `Ok`. Implement `From` trait to enable automatic error conversion with `?`.

**Why It Matters**: The `?` operator reduces error handling from 5+ lines per call to a single character, making error paths as readable as success paths. Automatic error conversion via `From` eliminates boilerplate while preserving type safety.

**Use Cases**: Application code with mixed error types (I/O, parsing, validation), batch processing that needs to collect all errors, network code requiring retries, operations that can fall back to alternatives, data pipelines with lenient error handling.

### Examples

```rust
//========================================
// Pattern: Basic error propagation with ?
//========================================
fn read_username(path: &str) -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string(path)?;  // Returns early on error
    let username = content.trim().to_string();
    Ok(username)
}

//======================================
// Pattern: Error type conversion with ?
//======================================
#[derive(Error, Debug)]
enum AppError {
    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Parse error")]
    Parse(#[from] ParseError),
}

fn process_file(path: &str) -> Result<i32, AppError> {
    let content = std::fs::read_to_string(path)?;  // Converts io::Error to AppError
    let number = parse_number(&content)?;           // Converts ParseError to AppError
    Ok(number)
}

//==============================
// Pattern: Manual error mapping
//==============================
fn read_and_validate(path: &str) -> Result<String, IoError> {
    std::fs::read_to_string(path)
        .map_err(|e| IoError::ReadFailed {
            path: path.to_string(),
            source: e,
        })?;

    Ok("valid".to_string())
}

//======================================
// Pattern: Fallible iterator processing
//======================================
fn parse_all_numbers(lines: Vec<&str>) -> Result<Vec<i32>, ParseError> {
    lines
        .into_iter()
        .map(parse_number)
        .collect()  // Collects Result<Vec<T>, E> - stops at first error
}

//=========================================
// Pattern: Collect successes, log failures
//=========================================
fn parse_all_lenient(lines: Vec<&str>) -> Vec<i32> {
    lines
        .into_iter()
        .filter_map(|line| {
            parse_number(line)
                .map_err(|e| eprintln!("Failed to parse '{}': {}", line, e))
                .ok()
        })
        .collect()
}

//================================================
// Pattern: Early return with multiple error types
//================================================
fn complex_operation(path: &str) -> Result<String, anyhow::Error> {
    let data = std::fs::read_to_string(path)
        .context("Failed to read input file")?;

    let parsed: Config = serde_json::from_str(&data)
        .context("Failed to parse JSON")?;

    validate_config(&parsed)
        .context("Config validation failed")?;

    Ok("success".to_string())
}

fn validate_config(_config: &Config) -> Result<(), ValidationError> {
    Ok(())
}

#[derive(Error, Debug)]
#[error("validation error")]
struct ValidationError;

//=========================================
// Pattern: Recovering from specific errors
//=========================================
fn read_or_default(path: &str) -> Result<String, std::io::Error> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok("default config".to_string())
        }
        Err(e) => Err(e),
    }
}

//===========================================
// Pattern: Retry logic with error inspection
//===========================================
fn retry_on_timeout<F, T, E>(mut f: F, max_attempts: usize) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Display,
{
    let mut attempts = 0;
    loop {
        attempts += 1;
        match f() {
            Ok(value) => return Ok(value),
            Err(e) if attempts < max_attempts => {
                eprintln!("Attempt {} failed: {}, retrying...", attempts, e);
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => return Err(e),
        }
    }
}

//====================================
// Pattern: Error context accumulation
//====================================
fn process_with_context(path: &str) -> Result<i32> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path))?;

    let number = parse_number(&content)
        .with_context(|| format!("Failed to parse content from {}", path))?;

    if number < 0 {
        anyhow::bail!("Number must be positive, got: {}", number);
    }

    Ok(number)
}

//=======================================================
// Pattern: Partition results into successes and failures
//=======================================================
fn partition_results<T, E>(results: Vec<Result<T, E>>) -> (Vec<T>, Vec<E>) {
    results.into_iter().partition_map(|r| match r {
        Ok(v) => itertools::Either::Left(v),
        Err(e) => itertools::Either::Right(e),
    })
}

use itertools::Itertools;
```

**Propagation strategies:**
- **Immediate propagation (`?`)**: Most common, fail fast
- **Map errors**: Add context before propagating
- **Collect errors**: Accumulate multiple failures
- **Recover**: Handle specific errors, propagate others
- **Retry**: Attempt operation multiple times
- **Log and continue**: Record failure but don't propagate

**When to use each:**
- Libraries: Specific error types, minimal context
- Applications: Rich context (anyhow), helpful diagnostics
- Batch processing: Collect all errors
- Network operations: Retry with backoff

## Pattern 3: Custom Error Types with Context

**Problem**: Generic errors like "parse error" or "database query failed" provide no actionable information. Was it line 47 or line 1832?

**Solution**: Enrich errors with context at the point of failure using `anyhow::Context` for applications or structured fields in custom error types for libraries. Include: what operation failed, what input caused it, where in the input (line/column for parsers, row for databases), timing information, suggestions for fixing.

**Why It Matters**: "Parse error at line 847, column 23: expected '}', got EOF. Suggestion: check for unclosed braces" points directly to the bug.

**Use Cases**: Parsers and compilers (provide line/column and code snippet), configuration validation (suggest valid values), database operations (include query and parameters), file I/O (include paths and operations attempted), network requests (include URL, method, status).

### Examples

```rust
//======================================
// Pattern: Error with location tracking
//======================================
#[derive(Error, Debug)]
#[error("Parse error at line {line}, column {column}: {message}")]
pub struct ParseErrorWithLocation {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub snippet: Option<String>,
}

impl ParseErrorWithLocation {
    pub fn new(line: usize, column: usize, message: String) -> Self {
        ParseErrorWithLocation {
            line,
            column,
            message,
            snippet: None,
        }
    }

    pub fn with_snippet(mut self, snippet: String) -> Self {
        self.snippet = Some(snippet);
        self
    }
}

//==================================================
// Pattern: Error with stack trace (using backtrace)
//==================================================
#[derive(Debug)]
pub struct DetailedError {
    message: String,
    context: Vec<String>,
    backtrace: std::backtrace::Backtrace,
}

impl DetailedError {
    pub fn new(message: impl Into<String>) -> Self {
        DetailedError {
            message: message.into(),
            context: Vec::new(),
            backtrace: std::backtrace::Backtrace::capture(),
        }
    }

    pub fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context.push(ctx.into());
        self
    }
}

impl std::fmt::Display for DetailedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        for ctx in &self.context {
            write!(f, "\n  in {}", ctx)?;
        }
        write!(f, "\n\nBacktrace:\n{}", self.backtrace)?;
        Ok(())
    }
}

impl std::error::Error for DetailedError {}

//========================================
// Pattern: Error with structured metadata
//========================================
#[derive(Error, Debug)]
#[error("Database query failed")]
pub struct QueryError {
    pub query: String,
    pub duration_ms: u64,
    pub row_count: Option<usize>,
    pub parameters: Vec<String>,
    #[source]
    pub source: Box<dyn std::error::Error + Send + Sync>,
}

impl QueryError {
    pub fn display_detailed(&self) -> String {
        format!(
            "Query failed after {}ms\nQuery: {}\nParameters: {:?}\nError: {}",
            self.duration_ms, self.query, self.parameters, self.source
        )
    }
}

//=========================
// Pattern: Context wrapper
//=========================
pub struct ErrorContext<E> {
    error: E,
    context: Vec<String>,
}

impl<E: std::error::Error> ErrorContext<E> {
    pub fn new(error: E) -> Self {
        ErrorContext {
            error,
            context: Vec::new(),
        }
    }

    pub fn add_context(mut self, ctx: impl Into<String>) -> Self {
        self.context.push(ctx.into());
        self
    }
}

impl<E: std::error::Error> std::fmt::Display for ErrorContext<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.error)?;
        for ctx in &self.context {
            writeln!(f, "  Context: {}", ctx)?;
        }
        Ok(())
    }
}

impl<E: std::error::Error + 'static> std::error::Error for ErrorContext<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

//================================
// Pattern: Error with suggestions
//================================
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required field: {field}\n  Suggestion: {suggestion}")]
    MissingField { field: String, suggestion: String },

    #[error("Invalid value for {field}: {value}\n  Expected: {expected}")]
    InvalidValue {
        field: String,
        value: String,
        expected: String,
    },
}

impl ConfigError {
    pub fn missing_field(field: impl Into<String>) -> Self {
        let field = field.into();
        let suggestion = match field.as_str() {
            "database_url" => "Add DATABASE_URL to your .env file".to_string(),
            "api_key" => "Set API_KEY environment variable".to_string(),
            _ => format!("Add {} to configuration", field),
        };
        ConfigError::MissingField { field, suggestion }
    }
}

//===========================
// Pattern: Error aggregation
//===========================
#[derive(Error, Debug)]
#[error("Multiple errors occurred")]
pub struct MultiError {
    errors: Vec<Box<dyn std::error::Error + Send + Sync>>,
}

impl MultiError {
    pub fn new() -> Self {
        MultiError { errors: Vec::new() }
    }

    pub fn add(&mut self, error: Box<dyn std::error::Error + Send + Sync>) {
        self.errors.push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn into_result<T>(self, value: T) -> Result<T, Self> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            Err(self)
        }
    }
}

impl std::fmt::Display for MultiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Multiple errors occurred ({}):", self.errors.len())?;
        for (i, err) in self.errors.iter().enumerate() {
            writeln!(f, "  {}. {}", i + 1, err)?;
        }
        Ok(())
    }
}

// Usage
fn validate_all(items: Vec<Item>) -> Result<(), MultiError> {
    let mut errors = MultiError::new();

    for item in items {
        if let Err(e) = validate_item(&item) {
            errors.add(Box::new(e));
        }
    }

    errors.into_result(())
}

struct Item;

#[derive(Error, Debug)]
#[error("validation failed")]
struct ItemValidationError;

fn validate_item(_item: &Item) -> Result<(), ItemValidationError> {
    Ok(())
}
```

**Context best practices:**
1. **Include inputs**: What data caused the error?
2. **Include operation**: What were you trying to do?
3. **Include location**: File, line, function
4. **Include timing**: When did it happen? How long did it take?
5. **Include suggestions**: How can the user fix it?
6. **Chain sources**: Preserve the full error chain

**Context anti-patterns:**
- Redundant information already in source error
- Sensitive data (passwords, tokens) in error messages
- Too much context (stack of 20+ context strings)
- Context that's obvious from source code

## Pattern 4: Recoverable vs Unrecoverable Errors

**Problem**: Using `Result` for everything forces callers to handle programmer errors (bugs) that should never occur, cluttering code with defensive checks. Using `panic!` for recoverable errors (file not found, network timeout) makes software brittle—crashes instead of graceful degradation.

**Solution**: Use `Result` for expected failures that callers should handle (file not found, parse errors, network failures). Use `panic!` for programmer errors and invariant violations (out-of-bounds indexing, assertion failures, contract violations).

**Why It Matters**: Using `panic!` for programmer errors catches bugs immediately in development—if an array index is out of bounds, the program crashes with a clear error rather than returning a `Result` that might be ignored. Using `Result` for external failures enables graceful degradation—a web server can return 503 for database timeout instead of crashing.

**Use Cases**: Libraries use `Result` (let caller decide), applications can `panic!` at startup for missing config, long-running services use `Result` with fallbacks, embedded systems may `panic!` on OOM, test code uses `unwrap()` liberally, FFI boundaries must catch panics with `catch_unwind`.

### Examples
 
```rust
//=====================================
// Pattern: Panic for programmer errors
//=====================================
fn get_user(users: &[User], index: usize) -> &User {
    &users[index]  // Panics on out-of-bounds - caller bug
}

//======================================
// Pattern: Result for expected failures
//======================================
fn find_user(users: &[User], id: u64) -> Option<&User> {
    users.iter().find(|u| u.id == id)  // None is expected
}

struct User {
    id: u64,
    name: String,
}

//=========================================
// Pattern: Expect with informative message
//=========================================
fn initialize() {
    let config = load_config()
        .expect("Failed to load config: ensure config.toml exists in working directory");
}

fn load_config() -> Result<String, std::io::Error> {
    std::fs::read_to_string("config.toml")
}

//=========================
// Pattern: Unwrap in tests
//=========================
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse() {
        let result = parse_number("42").unwrap();  // OK in tests
        assert_eq!(result, 42);
    }

    use super::*;
}

//==========================
// Pattern: Debug assertions
//==========================
fn compute_checksum(data: &[u8]) -> u32 {
    debug_assert!(!data.is_empty(), "Data must not be empty");
    data.iter().map(|&b| b as u32).sum()
}

//==============================
// Pattern: Fail fast at startup
//==============================
fn main() -> Result<()> {
    let config = load_config()?;  // Fail if config missing
    let db = connect_database(&config)?;  // Fail if DB unreachable

    // Application starts only if initialization succeeds
    run_server(db)?;
    Ok(())
}

fn connect_database(_config: &str) -> Result<Database> {
    Ok(Database)
}

fn run_server(_db: Database) -> Result<()> {
    Ok(())
}

struct Database;

//==============================
// Pattern: Graceful degradation
//==============================
fn get_user_with_fallback(id: u64) -> User {
    match fetch_user_from_cache(id) {
        Ok(user) => user,
        Err(_) => {
            eprintln!("Cache miss for user {}, fetching from DB", id);
            fetch_user_from_db(id).unwrap_or_else(|_| User {
                id,
                name: "Unknown".to_string(),
            })
        }
    }
}

fn fetch_user_from_cache(_id: u64) -> Result<User, CacheError> {
    Err(CacheError)
}

fn fetch_user_from_db(_id: u64) -> Result<User, DbError> {
    Err(DbError)
}

#[derive(Error, Debug)]
#[error("cache error")]
struct CacheError;

#[derive(Error, Debug)]
#[error("db error")]
struct DbError;

//================================
// Pattern: Poisoning vs panicking
//================================
use std::sync::{Mutex, PoisonError};

fn update_counter(counter: &Mutex<i32>) -> Result<(), String> {
    match counter.lock() {
        Ok(mut c) => {
            *c += 1;
            Ok(())
        }
        Err(poisoned) => {
            // Mutex poisoned due to panic in another thread
            eprintln!("Mutex poisoned, recovering...");
            let mut c = poisoned.into_inner();
            *c += 1;
            Ok(())
        }
    }
}

//==================================
// Pattern: Abort on critical errors
//==================================
fn write_checkpoint(data: &[u8]) -> Result<()> {
    std::fs::write("checkpoint.dat", data).map_err(|e| {
        eprintln!("CRITICAL: Failed to write checkpoint: {}", e);
        eprintln!("Data integrity cannot be guaranteed. Aborting.");
        std::process::abort();
    })
}

//=========================================
// Pattern: Catch unwind for FFI boundaries
//=========================================
use std::panic::{catch_unwind, AssertUnwindSafe};

#[no_mangle]
pub extern "C" fn safe_compute(input: i32) -> i32 {
    match catch_unwind(AssertUnwindSafe(|| risky_computation(input))) {
        Ok(result) => result,
        Err(_) => {
            eprintln!("Computation panicked, returning default");
            0
        }
    }
}

fn risky_computation(input: i32) -> i32 {
    if input < 0 {
        panic!("Negative input!");
    }
    input * 2
}
```

**Decision tree: Result vs Panic**

Use `Result` when:
- Failure is expected (file not found, network timeout)
- Caller should handle the error
- Error can be recovered
- Library code (let caller decide)

Use `panic!` when:
- Programmer error (contract violation)
- Invariant violated (impossible state)
- Continuing would corrupt data
- Prototype/example code

Use `Option` when:
- Absence is a valid state (empty collection)
- No error context needed
- Simpler than Result<T, ()>

## Pattern 5: Error Handling in Async Contexts

**Problem**: Async operations introduce failure modes absent in synchronous code: timeouts (operation took too long), cancellation (task dropped before completion), concurrent failures (10 out of 100 requests failed), and cascading failures (one service down brings down dependent services). Naive async error handling leads to unbounded waits, resource leaks from cancelled operations, and unclear error reporting when multiple concurrent operations fail.

**Solution**: Wrap all I/O operations in timeouts using `tokio::time::timeout`. Use `try_join!` or `try_join_all` to propagate first error from concurrent operations.

**Why It Matters**: Without timeouts, a single slow dependency can hang your entire service—one database query taking 30 seconds blocks all concurrent requests. Without proper cancellation handling, dropped tasks can leave files partially written or transactions uncommitted.

**Use Cases**: Web servers handling concurrent requests, microservices with service-to-service calls, batch processing with concurrent workers, real-time systems with latency requirements, streaming data pipelines, distributed systems requiring fault tolerance.

### Examples

```rust
use tokio;
use std::time::Duration;

//=================================
// Pattern: Async error propagation
//=================================
async fn fetch_user_data(id: u64) -> Result<User> {
    let response = make_http_request(id).await?;
    let user = parse_response(response).await?;
    Ok(user)
}

async fn make_http_request(_id: u64) -> Result<String> {
    Ok("response".to_string())
}

async fn parse_response(_response: String) -> Result<User> {
    Ok(User { id: 1, name: "Alice".to_string() })
}

//==============================
// Pattern: Timeout with context
//==============================
async fn fetch_with_timeout(id: u64) -> Result<User> {
    tokio::time::timeout(Duration::from_secs(5), fetch_user_data(id))
        .await
        .map_err(|_| anyhow::anyhow!("Timeout fetching user {}", id))?
}

//=============================================
// Pattern: Concurrent operations with try_join
//=============================================
async fn fetch_multiple_users(ids: Vec<u64>) -> Result<Vec<User>> {
    let futures = ids.into_iter().map(fetch_user_data);

    futures::future::try_join_all(futures)
        .await
        .context("Failed to fetch all users")
}

//==================================
// Pattern: Race multiple operations
//==================================
async fn fetch_with_fallback(id: u64) -> Result<User> {
    tokio::select! {
        result = fetch_from_primary(id) => result,
        result = fetch_from_secondary(id) => result,
    }
}

async fn fetch_from_primary(_id: u64) -> Result<User> {
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(User { id: 1, name: "Primary".to_string() })
}

async fn fetch_from_secondary(_id: u64) -> Result<User> {
    tokio::time::sleep(Duration::from_secs(2)).await;
    Ok(User { id: 1, name: "Secondary".to_string() })
}

//=============================================
// Pattern: Error recovery in stream processing
//=============================================
use futures::StreamExt;

async fn process_stream(mut stream: impl futures::Stream<Item = Result<i32>> + Unpin) {
    while let Some(result) = stream.next().await {
        match result {
            Ok(value) => println!("Processed: {}", value),
            Err(e) => {
                eprintln!("Error in stream: {}", e);
                // Continue processing despite errors
            }
        }
    }
}

//==================================================
// Pattern: Aggregating errors from concurrent tasks
//==================================================
async fn parallel_validation(items: Vec<Item>) -> Result<(), MultiError> {
    let handles: Vec<_> = items
        .into_iter()
        .map(|item| tokio::spawn(async move { validate_item(&item) }))
        .collect();

    let mut errors = MultiError::new();

    for handle in handles {
        if let Ok(Err(e)) = handle.await {
            errors.add(Box::new(e));
        }
    }

    errors.into_result(())
}

//====================================
// Pattern: Graceful shutdown on error
//====================================
async fn run_worker(mut shutdown: tokio::sync::broadcast::Receiver<()>) -> Result<()> {
    loop {
        tokio::select! {
            _ = shutdown.recv() => {
                println!("Shutting down gracefully");
                return Ok(());
            }
            result = do_work() => {
                if let Err(e) = result {
                    eprintln!("Work failed: {}", e);
                    // Decide whether to continue or stop
                    if is_fatal(&e) {
                        return Err(e);
                    }
                }
            }
        }
    }
}

async fn do_work() -> Result<()> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    Ok(())
}

fn is_fatal(_error: &anyhow::Error) -> bool {
    false
}

//========================================
// Pattern: Retry with exponential backoff
//========================================
async fn retry_with_backoff<F, T, Fut>(f: F, max_attempts: usize) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0;
    let mut delay = Duration::from_millis(100);

    loop {
        attempt += 1;
        match f().await {
            Ok(value) => return Ok(value),
            Err(e) if attempt >= max_attempts => {
                return Err(e.context(format!("Failed after {} attempts", attempt)));
            }
            Err(e) => {
                eprintln!("Attempt {} failed: {}, retrying in {:?}", attempt, e, delay);
                tokio::time::sleep(delay).await;
                delay *= 2;  // Exponential backoff
            }
        }
    }
}

//======================================
// Pattern: Cancellation-safe operations
//======================================
async fn cancellation_safe_write(data: &[u8]) -> Result<()> {
    let temp_path = "temp_file.tmp";
    let final_path = "final_file.dat";

    // Write to temp file (cancellation here is OK)
    tokio::fs::write(temp_path, data).await
        .context("Failed to write temp file")?;

    // Atomic rename (fast, unlikely to be cancelled)
    tokio::fs::rename(temp_path, final_path).await
        .context("Failed to rename file")?;

    Ok(())
}

//=================================
// Pattern: Circuit breaker pattern
//=================================
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct CircuitBreaker {
    failure_count: Arc<AtomicUsize>,
    threshold: usize,
}

impl CircuitBreaker {
    fn new(threshold: usize) -> Self {
        CircuitBreaker {
            failure_count: Arc::new(AtomicUsize::new(0)),
            threshold,
        }
    }

    async fn call<F, T, Fut>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let failures = self.failure_count.load(Ordering::Relaxed);

        if failures >= self.threshold {
            anyhow::bail!("Circuit breaker open: too many failures");
        }

        match f().await {
            Ok(value) => {
                self.failure_count.store(0, Ordering::Relaxed);
                Ok(value)
            }
            Err(e) => {
                self.failure_count.fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
        }
    }
}
```

**Async error handling principles:**
1. **Timeouts everywhere**: Network calls should have timeouts
2. **Graceful degradation**: Don't fail entire operation for one sub-task
3. **Cancellation safety**: Ensure operations can be cancelled safely
4. **Error aggregation**: Collect errors from concurrent operations
5. **Circuit breakers**: Fail fast when downstream is unavailable
6. **Retry with backoff**: Transient failures should retry with exponential backoff

## Pattern 6: Error Handling Anti-Patterns

```rust
// ❌ Swallowing errors
fn bad_error_handling(path: &str) {
    let _ = std::fs::read_to_string(path);  // Error ignored!
}

// ✓ Log or propagate
fn good_error_handling(path: &str) -> Result<String> {
    std::fs::read_to_string(path)
        .context("Failed to read file")
}

// ❌ Using unwrap in library code
pub fn parse_config(path: &str) -> Config {
    let content = std::fs::read_to_string(path).unwrap();  // Will panic!
    serde_json::from_str(&content).unwrap()
}

// ✓ Return Result, let caller decide
pub fn parse_config_safe(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

// ❌ Generic error messages
fn load_data() -> Result<Vec<u8>> {
    std::fs::read("data.bin")
        .map_err(|e| anyhow::anyhow!("Error: {}", e))  // Unhelpful
}

// ✓ Specific, actionable messages
fn load_data_better() -> Result<Vec<u8>> {
    std::fs::read("data.bin")
        .context("Failed to read data.bin: ensure file exists and is readable")
}

// ❌ Catching and re-panicking
fn bad_panic_handling() {
    if let Err(e) = risky_operation() {
        panic!("Operation failed: {}", e);  // Why not just return Result?
    }
}

fn risky_operation() -> Result<()> {
    Ok(())
}

// ✓ Propagate errors normally
fn good_panic_handling() -> Result<()> {
    risky_operation()?;
    Ok(())
}

// ❌ Over-broad error types
fn overly_generic() -> Result<i32, Box<dyn std::error::Error>> {
    Ok(42)  // Caller doesn't know what errors to expect
}

// ✓ Specific error types
#[derive(Error, Debug)]
enum ParseError2 {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Parse error")]
    Parse(#[from] std::num::ParseIntError),
}

fn specific_errors() -> Result<i32, ParseError2> {
    Ok(42)  // Clear what can fail
}
```

### Error Handling Comparison

| Approach | Use Case | Pros | Cons |
|----------|----------|------|------|
| `Result<T, E>` | Libraries | Type-safe, composable | Verbose, requires error type |
| `anyhow::Result` | Applications | Ergonomic, rich context | Dynamic type, less precise |
| `Option<T>` | Simple absence | Lightweight | No error info |
| `panic!` | Programmer errors | Impossible to ignore | Cannot recover |
| `thiserror` | Custom errors | Reduces boilerplate | Compile-time cost |

### Quick Reference

```rust
// Propagation
let x = may_fail()?;                   // Early return on error
let x = may_fail().context("msg")?;   // Add context

// Conversion
let x: Result<T, E2> = result.map_err(|e| convert(e));

// Recovery
let x = may_fail().unwrap_or(default);
let x = may_fail().unwrap_or_else(|| compute_default());

// Inspection
if let Err(e) = result {
    if e.kind() == ErrorKind::NotFound { /* ... */ }
}

// Chaining
result
    .and_then(|x| process(x))
    .or_else(|e| recover(e))

// Async
let result = timeout(duration, async_op).await?;
let results = try_join_all(futures).await?;
```

### Key Takeaways

1. **Libraries: specific errors, applications: opaque errors**
2. **Add context at error sites, not at error definitions**
3. **Use `?` for clean propagation**
4. **panic! for programmer errors, Result for runtime errors**
5. **Preserve error chains with source()**
6. **Include actionable information in error messages**
7. **Async: timeouts, retries, graceful degradation**
8. **Profile error handling overhead (usually negligible)**
9. **Use thiserror for custom errors, anyhow for applications**
10. **Document error conditions in function signatures**

Rust's error handling transforms errors from exceptional control flow to explicit, type-safe values. This approach makes error handling more verbose but dramatically more reliable, forcing you to consider every failure mode at compile time.

