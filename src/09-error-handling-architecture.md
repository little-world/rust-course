# Error Handling Architecture
 Unlike exceptions in languages like Java or Python, Rust uses explicit return types (`Result<T, E>`) to force handling of errors at compile time. This approach eliminates entire classes of bugs: forgotten error checks, unexpected exception propagation, and unclear error boundaries.


The key insight is that error handling in Rust is not just about reporting failures—it's about encoding your program's error domain in the type system, making impossible states unrepresentable, and providing excellent diagnostics when things go wrong.

## Pattern 1: Custom Error Enums for Libraries

**Problem**: Returning simple strings or a generic `Box<dyn Error>` from a library is not ideal. A `String` error loses all type information, making it impossible for a caller to programmatically handle different kinds of failures.

**Solution**: Define a custom `enum` for your library's errors. Each variant of the enum represents a distinct failure mode.

**Why It Matters**: A custom error enum makes your library's API transparent and robust. It allows users to `match` on specific error variants and handle them appropriately—for example, retrying a `Timeout` error but aborting on a `PermissionDenied` error.

**Use Cases**:
-   Public libraries where callers need to distinguish between different failure modes.
-   Systems with complex or domain-specific errors, such as network protocols, database clients, or parsers.
-   Any situation where an error is an expected part of the program's control flow.

### Example: A Simple Custom Error Enum

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
    // `?` auto-converts io::Error and ParseIntError into DataError
    // because of the `#[from]` attributes.
    let content = std::fs::read_to_string(path)?;
    let number = content.trim().parse()?;
    Ok(number)
}
// Usage: match on specific error variants
match parse_number("") {
    Err(ParseError::EmptyInput) => {}, _ => {}
}
let num = load_and_parse_number("config.txt")?;
```

### Example: `#[non_exhaustive]` for Library Stability

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
// Usage: handle known errors with wildcard for future variants
match err { ApiError::Timeout => retry(), _ => fail() }
```

## Pattern 2: `anyhow` for Application-Level Errors

**Problem**: In application code (as opposed to library code), you often don't need to handle each specific error type. Your main goal is to understand *why* an operation failed and report it to the user or a logging service.

**Solution**: Use the `anyhow` crate. `anyhow::Result` is a type alias for `Result<T, anyhow::Error>`, where `anyhow::Error` is a dynamic error type that can hold any error that implements `std::error::Error`.

**Why It Matters**: `anyhow` provides the convenience of a single, easy-to-use error type for your application while preserving the full chain of underlying causes. It strikes a balance between ease of use and detailed diagnostics, which is perfect for the top levels of an application.

**Use Cases**:
-   The main logic of CLI tools, web servers, and other applications.
-   Any situation where you care more about the error *message* and *context* than the specific error *type*.
-   Prototyping and writing examples where detailed error handling is not the main focus.

### Example: Structured Error with Multiple Fields

Complex errors often need multiple pieces of context to be actionable—an HTTP error isn't useful without the URL, method, and status code. Using a struct with named fields instead of an enum variant keeps all relevant information together. The `#[source]` attribute links to the underlying cause, preserving the full error chain for debugging.

```rust
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
// Usage: create an HTTP error with full context
let err = HttpError {
    method: "GET".into(), url: "/api".into(),
    status: 404, body: None, source: None
};
```

### Example: Error Builder for Complex Errors

When errors have many optional fields, a builder pattern prevents constructors with numerous parameters. The builder accumulates context fluently, making error construction readable at call sites. Calling `build()` at the end produces the final error with all accumulated context.

```rust
pub struct HttpErrorBuilder {
    method: String,
    url: String,
    status: u16,
    body: Option<String>,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl HttpErrorBuilder {
    pub fn new(
        method: impl Into<String>,
        url: impl Into<String>,
        status: u16,
    ) -> Self {
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

    pub fn source(
        mut self,
        err: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        self.source = Some(err);
        self
    }

    pub fn build(self) -> HttpError {
        HttpError {
            method: self.method, url: self.url, status: self.status,
            body: self.body, source: self.source,
        }
    }
}
// Usage: build complex errors fluently
let err = HttpErrorBuilder::new("POST", "/users", 500)
    .body("Internal error".into()).build();
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

### Example: Basic Error Propagation with `?`

The `?` operator unwraps `Ok` values and early-returns `Err` values, replacing verbose `match` statements with a single character. It automatically calls `Into::into()` on the error, enabling automatic type conversion. This makes the happy path linear and easy to follow.

```rust
fn read_username(path: &str) -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string(path)?; // Early return on error
    let username = content.trim().to_string();
    Ok(username)
}
// Usage: propagate I/O errors with the ? operator
let name = read_username("user.txt")?;
```

### Example: Error Type Conversion with `?`

Using `#[from]` attribute with `thiserror` implements `From` for automatic error conversion. When `?` is applied, the error is automatically converted to the function's return type. This lets you mix different error types in one function without manual conversion.

```rust
#[derive(Error, Debug)]
enum AppError {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Parse error")]
    Parse(#[from] ParseError),
}

fn process_file(path: &str) -> Result<i32, AppError> {
    let content = std::fs::read_to_string(path)?;
    let number = parse_number(&content)?;
    Ok(number)
}
// Usage: mix error types with automatic conversion
let n = process_file("number.txt")?;
```

### Example: Manual Error Mapping

When automatic conversion isn't enough, `map_err` transforms errors with custom logic. This is useful for adding context like file paths or enriching errors with domain-specific information. The closure receives the original error and returns your custom error type.

```rust
fn read_and_validate(path: &str) -> Result<String, IoError> {
    std::fs::read_to_string(path).map_err(|e| IoError::ReadFailed {
        path: path.to_string(), source: e
    })?;
    Ok("valid".to_string())
}
```

### Example: Fallible Iterator Processing

Collecting an iterator of `Result`s into `Result<Vec<T>, E>` stops at the first error and returns it. This fail-fast behavior is often what you want—no point processing further if one item fails. The type annotation on `collect()` drives this behavior.

```rust
fn parse_all_numbers(lines: Vec<&str>) -> Result<Vec<i32>, ParseError> {
    lines.into_iter().map(parse_number).collect() // Stops at first error
}
// Usage: collect results, failing fast on first error
let nums = parse_all_numbers(vec!["1", "2", "bad"]);
```

### Example: Collect Successes, Log Failures

Using `filter_map` with `.ok()` converts `Result` to `Option`, keeping only successes. The `map_err` before `.ok()` lets you log or record each failure before discarding it. This lenient approach processes everything possible instead of failing on the first error.

```rust
fn parse_all_lenient(lines: Vec<&str>) -> Vec<i32> {
    lines.into_iter()
        .filter_map(|line| {
            parse_number(line)
                .map_err(|e| eprintln!("Parse '{}': {}", line, e))
                .ok()
        })
        .collect()
}
// Usage: parse all valid values, logging failures
let nums = parse_all_lenient(vec!["1", "bad", "3"]);
```

### Example: Early Return with Multiple Error Types

Using `anyhow::Error` with `.context()` handles mixed error types elegantly in application code. Each `?` propagates a different error type, but `context()` wraps them all in `anyhow::Error`. The context strings create a chain explaining what operation failed at each level.

```rust
fn complex_operation(path: &str) -> Result<String, anyhow::Error> {
    let data = std::fs::read_to_string(path)
        .context("Failed to read input file")?;
    let parsed: Config = serde_json::from_str(&data)
        .context("Failed to parse JSON")?;
    validate_config(&parsed).context("Config validation")?;
    Ok("success".to_string())
}
```

### Example: Recovering from Specific Errors

Match guards (`if` in match arms) let you handle specific error variants differently. Here, `NotFound` triggers a fallback to defaults while other I/O errors propagate. This pattern is common for optional configuration files or cache misses.

```rust
fn read_or_default(path: &str) -> Result<String, std::io::Error> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok("default config".to_string())
        }
        Err(e) => Err(e),
    }
}
// Usage: fall back to default if file not found
let content = read_or_default("missing.txt")?;
```

### Example: Retry Logic with Error Inspection

A retry loop attempts an operation multiple times, logging failures between attempts. The match guard `if attempts < max_attempts` distinguishes retriable failures from final failure. Adding a delay between attempts prevents hammering a struggling service.

```rust
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
                eprintln!("Attempt {attempts}: {e}, retrying");
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => return Err(e),
        }
    }
}
```

### Example: Error Context Accumulation

`with_context` takes a closure that builds context strings lazily—only allocating if an error occurs. The `bail!` macro creates an error and returns immediately, useful for validation failures. This builds a rich error chain that explains exactly what went wrong.

```rust
fn process_with_context(path: &str) -> Result<i32> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path))?;
    let number = parse_number(&content)
        .with_context(|| format!("Failed to parse {}", path))?;
    if number < 0 {
        anyhow::bail!("Number must be positive, got: {}", number);
    }
    Ok(number)
}
```

### Example: Partition Results into Successes and Failures

`partition_map` from `itertools` separates a collection of `Result`s into two vectors in one pass. `Either::Left` collects successes, `Either::Right` collects errors. This is useful when you want to process all items and report all failures together.

```rust
fn partition_results<T, E>(results: Vec<Result<T, E>>) -> (Vec<T>, Vec<E>) {
    results.into_iter().partition_map(|r| match r {
        Ok(v) => itertools::Either::Left(v),
        Err(e) => itertools::Either::Right(e),
    })
}
// Usage: separate successes and failures
let (ok, err) = partition_results(vec![Ok(1), Err("bad"), Ok(2)]);
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

### Example: Error with Location Tracking

For parsers and compilers, line and column numbers transform vague errors into precise debugging information. The `#[error]` format string interpolates struct fields directly, creating readable messages. A builder-style `with_snippet` method optionally attaches the offending source code.

```rust
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
        ParseErrorWithLocation { line, column, message, snippet: None }
    }

    pub fn with_snippet(mut self, snippet: String) -> Self {
        self.snippet = Some(snippet);
        self
    }
}
// Usage: create parse error with precise location
let err = ParseErrorWithLocation::new(10, 5, "unexpected token".into());
// Displays: "Parse error at line 10, column 5: unexpected token"
```

### Example: Error with Stack Trace

Capturing a backtrace at error creation time shows exactly where the error originated. The `Backtrace::capture()` call records the call stack, which is invaluable for debugging complex error paths. Context can be accumulated as the error propagates up the call stack.

```rust
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
```

### Example: Error with Structured Metadata

Database errors benefit from structured context: the query, parameters, timing, and row count. A `display_detailed` method formats this for logging while the standard `Display` stays concise for user-facing output. The `#[source]` attribute preserves the underlying database driver error.

```rust
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
            "Query failed after {}ms\nQuery: {}\nError: {}",
            self.duration_ms, self.query, self.source)
    }
}
```

### Example: Context Wrapper

A generic wrapper adds context to any error type without modifying it. The `add_context` builder method accumulates explanations as the error bubbles up. Implementing `source()` preserves the error chain for debugging tools.

```rust
pub struct ErrorContext<E> {
    error: E,
    context: Vec<String>,
}

impl<E: std::error::Error> ErrorContext<E> {
    pub fn new(error: E) -> Self {
        ErrorContext { error, context: Vec::new() }
    }

    pub fn add_context(mut self, ctx: impl Into<String>) -> Self {
        self.context.push(ctx.into());
        self
    }
}

impl<E: std::error::Error + 'static> std::error::Error for ErrorContext<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}
```

### Example: Error with Suggestions

Actionable errors tell users how to fix problems, not just what went wrong. The `missing_field` constructor automatically generates context-aware suggestions based on the field name. Including expected values in validation errors eliminates guesswork.

```rust
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing field: {field}\n  Hint: {suggestion}")]
    MissingField { field: String, suggestion: String },

    #[error("Invalid {field}: {value}\n  Expected: {expected}")]
    InvalidValue { field: String, value: String, expected: String },
}

impl ConfigError {
    pub fn missing_field(field: impl Into<String>) -> Self {
        let field = field.into();
        let suggestion = match field.as_str() {
            "database_url" => "Add DATABASE_URL to .env".into(),
            "api_key" => "Set API_KEY env variable".to_string(),
            _ => format!("Add {} to configuration", field),
        };
        ConfigError::MissingField { field, suggestion }
    }
}
// Usage: create actionable error with fix suggestion
let err = ConfigError::missing_field("database_url");
// Displays: "Missing field: database_url\n  Hint: Add DATABASE_URL to .env"
```

### Example: Error Aggregation

When validating multiple items, collecting all errors is more useful than stopping at the first. `MultiError` accumulates failures and converts to `Result` only when checked. The `Display` implementation numbers each error for easy reference.

```rust
#[derive(Error, Debug)]
#[error("Multiple errors occurred")]
pub struct MultiError {
    errors: Vec<Box<dyn std::error::Error + Send + Sync>>,
}

impl MultiError {
    pub fn new() -> Self { MultiError { errors: Vec::new() } }

    pub fn add(&mut self, error: Box<dyn std::error::Error + Send + Sync>) {
        self.errors.push(error);
    }

    pub fn into_result<T>(self, value: T) -> Result<T, Self> {
        if self.errors.is_empty() { Ok(value) } else { Err(self) }
    }
}

fn validate_all(items: Vec<Item>) -> Result<(), MultiError> {
    let mut errors = MultiError::new();
    for item in items {
        if let Err(e) = validate_item(&item) {
            errors.add(Box::new(e));
        }
    }
    errors.into_result(())
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

### Example: Panic for Programmer Errors

Indexing directly into a slice panics on out-of-bounds access—this is intentional because such access indicates a caller bug. The panic message includes the index and length, making the bug obvious during development. If the caller could provide an invalid index, use `get()` instead.

```rust
fn get_user(users: &[User], index: usize) -> &User {
    &users[index]  // Panics on out-of-bounds - caller bug
}
// Usage: access user by index (panics if invalid)
let user = get_user(&users, 0); // Panics if users is empty
```

### Example: Result for Expected Failures

When absence is an expected outcome, return `Option` or `Result` instead of panicking. A user not found by ID is a normal case, not a bug. This lets callers decide how to handle the missing case.

```rust
fn find_user(users: &[User], id: u64) -> Option<&User> {
    users.iter().find(|u| u.id == id)  // None is expected
}
// Usage: search for user, handling absence gracefully
if let Some(user) = find_user(&users, 42) { /* ... */ }
```

### Example: Expect with Informative Message

`.expect()` panics with a custom message, making failures easier to diagnose than bare `.unwrap()`. Use it for errors that indicate setup problems or invariant violations. The message should explain what was expected, not just that something failed.

```rust
fn initialize() {
    let config = load_config()
        .expect("Failed to load config.toml");
}
```

### Example: Unwrap in Tests

Test code can use `.unwrap()` liberally because test failures already produce clear output. The panic shows exactly which line failed and what the error was. This keeps test code concise and focused on assertions.

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse() {
        let result = parse_number("42").unwrap();  // OK in tests
        assert_eq!(result, 42);
    }
}
```

### Example: Debug Assertions

`debug_assert!` checks invariants in debug builds but compiles to nothing in release builds. Use it for expensive checks or preconditions that should never fail if the code is correct. This catches bugs during development without impacting production performance.

```rust
fn compute_checksum(data: &[u8]) -> u32 {
    debug_assert!(!data.is_empty(), "Data must not be empty");
    data.iter().map(|&b| b as u32).sum()
}
```

### Example: Fail Fast at Startup

Applications should verify all dependencies (config files, databases, network) before starting to serve requests. Using `?` in `main()` propagates errors up and exits with a non-zero status. This prevents partially-initialized services from accepting traffic.

```rust
fn main() -> Result<()> {
    let config = load_config()?;         // Fail if config missing
    let db = connect_database(&config)?; // Fail if DB unreachable
    run_server(db)?;                     // Only start if init succeeds
    Ok(())
}
```

### Example: Graceful Degradation

Long-running services should degrade gracefully rather than crash on transient failures. This pattern tries cache first, falls back to database, and returns a default if both fail. Each fallback is logged so operators can investigate.

```rust
fn get_user_with_fallback(id: u64) -> User {
    match fetch_user_from_cache(id) {
        Ok(user) => user,
        Err(_) => {
            eprintln!("Cache miss for user {id}, fetching from DB");
            fetch_user_from_db(id).unwrap_or_else(|_| User {
                id, name: "Unknown".to_string(),
            })
        }
    }
}
```

### Example: Mutex Poisoning Recovery

When a thread panics while holding a mutex, the mutex becomes "poisoned" to prevent access to potentially corrupted state. You can recover by calling `into_inner()` on the poison error if you're confident the data is still valid. This decision should be made carefully based on what the panicking thread was doing.

```rust
fn update_counter(counter: &Mutex<i32>) -> Result<(), String> {
    match counter.lock() {
        Ok(mut c) => { *c += 1; Ok(()) }
        Err(poisoned) => {
            eprintln!("Mutex poisoned, recovering...");
            let mut c = poisoned.into_inner();
            *c += 1;
            Ok(())
        }
    }
}
```

### Example: Abort on Critical Errors

For data-critical applications, `std::process::abort()` is safer than `panic!` because it can't be caught with `catch_unwind`. Use this when continuing could cause data corruption or when the error indicates a fundamental system problem. The process terminates immediately without running destructors.

```rust
fn write_checkpoint(data: &[u8]) -> Result<()> {
    std::fs::write("checkpoint.dat", data).map_err(|e| {
        eprintln!("CRITICAL: Failed to write checkpoint: {}", e);
        std::process::abort();
    })
}
```

### Example: Catch Unwind for FFI Boundaries

Panics must not cross FFI boundaries—unwinding into C code is undefined behavior. `catch_unwind` captures panics and converts them to normal return values. The `AssertUnwindSafe` wrapper marks closures as safe to unwind, since the compiler can't verify this automatically.

```rust
#[no_mangle]
pub extern "C" fn safe_compute(input: i32) -> i32 {
    let result = catch_unwind(AssertUnwindSafe(|| risky_computation(input)));
    match result {
        Ok(value) => value,
        Err(_) => { eprintln!("Computation panicked"); 0 }
    }
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

### Example: Async Error Propagation

The `?` operator works in async functions just like sync ones, enabling clean error propagation across `.await` points. Each await can fail independently, and errors bubble up through the call chain. The function signature's `Result` type documents all possible failure modes.

```rust
async fn fetch_user_data(id: u64) -> Result<User> {
    let response = make_http_request(id).await?;
    let user = parse_response(response).await?;
    Ok(user)
}
// Usage: propagate errors across async boundaries
let user = fetch_user_data(42).await?;
```

### Example: Timeout with Context

Wrapping async operations in `tokio::time::timeout` prevents unbounded waits that can hang your service. The timeout returns `Err(Elapsed)` which you convert to a domain error with context. Always set timeouts on network calls, database queries, and external service calls.

```rust
async fn fetch_with_timeout(id: u64) -> Result<User> {
    let timeout = Duration::from_secs(5);
    tokio::time::timeout(timeout, fetch_user_data(id))
        .await
        .map_err(|_| anyhow::anyhow!("Timeout fetching user {id}"))?
}
// Usage: fetch with 5-second timeout
let user = fetch_with_timeout(42).await?;
```

### Example: Concurrent Operations with try_join

`try_join_all` runs multiple futures concurrently and returns the first error, cancelling remaining futures. This fail-fast behavior is appropriate when all results are needed. For partial success scenarios, use `join_all` and handle each `Result` individually.

```rust
async fn fetch_multiple_users(ids: Vec<u64>) -> Result<Vec<User>> {
    let futures = ids.into_iter().map(fetch_user_data);
    futures::future::try_join_all(futures)
        .await
        .context("Failed to fetch all users")
}
```

### Example: Race Multiple Operations

`tokio::select!` races multiple futures, returning when the first one completes. This is useful for redundant requests where you want the fastest response. The losing branch is cancelled, so ensure your operations are cancellation-safe.

```rust
async fn fetch_with_fallback(id: u64) -> Result<User> {
    tokio::select! {
        result = fetch_from_primary(id) => result,
        result = fetch_from_secondary(id) => result,
    }
}
```

### Example: Error Recovery in Stream Processing

Processing streams item-by-item lets you handle errors without aborting the entire stream. Log or record each failure, then continue with the next item. This lenient approach is appropriate for data pipelines where partial results are useful.

```rust
async fn process_stream(mut stream: impl Stream<Item = Result<i32>> + Unpin) {
    while let Some(result) = stream.next().await {
        match result {
            Ok(value) => println!("Processed: {}", value),
            Err(e) => eprintln!("Error in stream: {}", e), // Continue
        }
    }
}
```

### Example: Aggregating Errors from Concurrent Tasks

When validating many items in parallel, collect all errors rather than failing on the first. Spawn tasks with `tokio::spawn`, await all handles, and accumulate failures into a `MultiError`. This provides a complete picture of what went wrong.

```rust
async fn parallel_validation(items: Vec<Item>) -> Result<(), MultiError> {
    let handles: Vec<_> = items.into_iter()
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
```

### Example: Graceful Shutdown on Error

Long-running workers should listen for shutdown signals while processing work. `tokio::select!` lets you race the shutdown channel against work tasks. Non-fatal errors are logged and processing continues; fatal errors trigger a clean exit.

```rust
async fn run_worker(mut shutdown: broadcast::Receiver<()>) -> Result<()> {
    loop {
        tokio::select! {
            _ = shutdown.recv() => {
                println!("Shutting down gracefully");
                return Ok(());
            }
            result = do_work() => {
                if let Err(e) = result {
                    if is_fatal(&e) { return Err(e); }
                    eprintln!("Work failed: {}", e);
                }
            }
        }
    }
}
```

### Example: Retry with Exponential Backoff

Transient failures (network blips, temporary overload) often succeed on retry. Exponential backoff increases delay between attempts to avoid hammering a struggling service. Cap the maximum attempts and include attempt count in the final error.

```rust
async fn retry_with_backoff<F, Fut, T>(f: F, max_attempts: usize) -> Result<T>
where F: Fn() -> Fut, Fut: Future<Output = Result<T>>
{
    let mut attempt = 0;
    let mut delay = Duration::from_millis(100);
    loop {
        attempt += 1;
        match f().await {
            Ok(value) => return Ok(value),
            Err(e) if attempt >= max_attempts => {
                return Err(e.context(format!("Failed after {attempt} tries")));
            }
            Err(e) => {
                eprintln!("Attempt {attempt}: {e}");
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
        }
    }
}
```

### Example: Cancellation-Safe Operations

If a task is cancelled mid-operation, partially-written files can corrupt data. Write to a temporary file first, then atomically rename. The rename is fast enough that cancellation during it is unlikely, and if it fails, the original file is untouched.

```rust
async fn cancellation_safe_write(data: &[u8]) -> Result<()> {
    let temp_path = "temp_file.tmp";
    let final_path = "final_file.dat";
    tokio::fs::write(temp_path, data).await.context("Write temp")?;
    tokio::fs::rename(temp_path, final_path).await.context("Rename")?;
    Ok(())
}
```

### Example: Circuit Breaker Pattern

A circuit breaker prevents cascading failures by failing fast when a downstream service is unhealthy. After a threshold of failures, requests immediately return an error without attempting the call. Success resets the counter, gradually restoring normal operation.

```rust
struct CircuitBreaker {
    failure_count: Arc<AtomicUsize>,
    threshold: usize,
}

impl CircuitBreaker {
    async fn call<F, Fut, T>(&self, f: F) -> Result<T>
    where F: FnOnce() -> Fut, Fut: Future<Output = Result<T>>
    {
        if self.failure_count.load(Ordering::Relaxed) >= self.threshold {
            anyhow::bail!("Circuit breaker open");
        }
        match f().await {
            Ok(v) => {
                self.failure_count.store(0, Ordering::Relaxed);
                Ok(v)
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
    let content = std::fs::read_to_string(path).unwrap(); // Panic!
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
        .context("Failed to read data.bin: check file exists")
}

// ❌ Catching and re-panicking
fn bad_panic_handling() {
    if let Err(e) = risky_operation() {
        panic!("Operation failed: {}", e); // Return Result!
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

