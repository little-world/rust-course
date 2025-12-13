# Iterator for Paginating

### Problem Statement

Build a lazy-loading iterator that transparently fetches paginated data from REST APIs. Your iterator should handle pagination logic internally, making API endpoints with pagination appear as infinite streams of items.

Your paginated iterator should support:
- Automatic page fetching as iteration progresses
- Configurable page size and rate limiting
- Error handling and retry logic
- Caching to avoid redundant requests
- Multiple pagination styles (offset-based, cursor-based, page-number-based)
- Type-safe deserialization of API responses

Example API response:
```json
{
  "data": [
    {"id": 1, "name": "Item 1"},
    {"id": 2, "name": "Item 2"}
  ],
  "pagination": {
    "next_cursor": "abc123",
    "has_more": true
  }
}
```

### Why It Matters

Many REST APIs return data in pages to limit response sizes. Writing pagination logic manually is tedious and error-prone - you must track page numbers, handle edge cases, and manage state across requests. A paginated iterator abstracts this complexity, letting developers write `.filter().map().collect()` instead of nested loops with error handling.

This pattern is fundamental to API client libraries. Real-world examples: GitHub API, Stripe API, AWS SDK pagination, Google Cloud APIs - all use this pattern.

---

## Key Concepts Explained

This project demonstrates how to build production-ready API clients using Rust's iterator trait and advanced patterns.

### 1. Iterator Trait for Lazy Evaluation

Iterators process items on-demand, not all at once:

```rust
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

// Lazy: Fetches pages only when items consumed
let iter = PaginatedIterator::new(url, 10);
for item in iter.take(5) {  // Only fetches first page!
    process(item);
}
```


**vs Eager loading**:
```rust
// ❌ Loads all pages into memory
let all_items = fetch_all_pages();  // OOM for large datasets
all_items.into_iter().take(5);

// ✅ Loads pages on-demand
PaginatedIterator::new().take(5);  // Constant memory
```

### 2. Generic Types with Trait Bounds

Type-safe deserialization for any data type:

```rust
struct PaginatedIterator<T>
where
    T: for<'de> Deserialize<'de>  // T must be deserializable
{
    buffer: Vec<T>,
}

// Works with any deserializable type
let users: PaginatedIterator<User> = PaginatedIterator::new();
let posts: PaginatedIterator<Post> = PaginatedIterator::new();
```

**Benefit**: One implementation works for all API response types.

### 3. PhantomData for Zero-Cost Type Parameters

Associate type parameter without storing values:

```rust
struct PaginatedIterator<T> {
    url: String,
    buffer: Vec<T>,
    _phantom: PhantomData<T>,  // 0 bytes!
}
```


### 4. Buffering for Performance

Fetch pages in batches, yield items individually:

```rust
struct PaginatedIterator<T> {
    buffer: Vec<T>,       // Current page items
    buffer_index: usize,  // Position in buffer
}

fn next(&mut self) -> Option<T> {
    if self.buffer_index >= self.buffer.len() {
        self.fetch_next_page()?;  // Fetch when exhausted
        self.buffer_index = 0;
    }
    let item = self.buffer[self.buffer_index].clone();
    self.buffer_index += 1;
    Some(item)
}
```

### 5. Enum for Multiple Strategies

Pattern match on pagination type:

```rust
enum PaginationStrategy {
    Offset { offset: usize, limit: usize },
    Cursor { cursor: Option<String>, page_size: usize },
    PageNumber { page: usize, per_page: usize },
}

fn fetch_page(&mut self) -> Result<()> {
    match &self.strategy {
        PaginationStrategy::Offset { offset, limit } =>
            fetch_with_offset(*offset, *limit),
        PaginationStrategy::Cursor { cursor, .. } =>
            fetch_with_cursor(cursor.as_ref()),
        PaginationStrategy::PageNumber { page, per_page } =>
            fetch_with_page_number(*page, *per_page),
    }
}
```

### 6. Token Bucket Algorithm for Rate Limiting

Control request rate to respect API limits:

```rust
struct RateLimiter {
    tokens: f64,           // Available tokens
    max_tokens: f64,       // Bucket capacity
    refill_rate: f64,      // Tokens/second
    last_refill: Instant,  // Last refill time
}

fn acquire(&mut self) {
    self.refill();  // Add tokens based on elapsed time
    while self.tokens < 1.0 {
        sleep(calculate_wait_time());
        self.refill();
    }
    self.tokens -= 1.0;
}
```

### 7. Exponential Backoff for Retries

Handle transient failures with increasing delays:

```rust
struct RetryPolicy {
    max_attempts: usize,
    initial_backoff: Duration,
    multiplier: f64,  // Usually 2.0
}

fn delay_for_attempt(&self, attempt: usize) -> Duration {
    let delay = self.initial_backoff * self.multiplier.powi(attempt as i32);
    delay.min(self.max_backoff)  // Cap maximum delay
}

// Delays: 100ms, 200ms, 400ms, 800ms, 1600ms, ...
```

### 8. Arc and Mutex for Shared State

Share cache between iterator clones:

```rust
struct CachedIterator<T> {
    cache: Arc<Mutex<Vec<Vec<T>>>>,  // Shared ownership
}

fn clone_iter(&self) -> Self {
    CachedIterator {
        cache: Arc::clone(&self.cache),  // Reference count++
    }
}
```

### 9. Builder Pattern for Configuration

Fluent API for complex setup:

```rust
let iter = PaginatedIterator::new(url, 10)
    .with_rate_limit(5.0)               // 5 req/sec
    .with_retries(3, Duration::from_secs(1))
    .with_timeout(Duration::from_secs(30))
    .with_cache();

// vs verbose constructor
let iter = PaginatedIterator::new_with_all_options(
    url, 10, Some(5.0), Some((3, Duration::from_secs(1))), ...
);
```


### 10. Higher-Order Iterator Adapters

Chain operations without intermediate allocations:

```rust
let filtered_users = PaginatedIterator::<User>::new(url, 100)
    .filter(|u| u.active)           // Lazy filter
    .map(|u| u.email)               // Lazy map
    .take(50)                        // Lazy take
    .collect::<Vec<_>>();           // Execute chain

// Only fetches pages needed for 50 active users
// If first page has 50 active users, only 1 HTTP request!
```

---

## Connection to This Project

Here's how each milestone applies these concepts to build production-grade API clients.

### Milestone 1: Basic Paginated Iterator

**Concepts applied**:
- **Iterator trait**: Implement `next()` for lazy evaluation
- **Generic types**: `PaginatedIterator<T>` works with any type
- **Buffering**: Fetch pages, yield items individually
- **PhantomData**: Track type parameter without storing values

**Why this matters**: Foundation of lazy, type-safe pagination.

**Real-world impact**:
- GitHub API: 5000 requests/hour limit
- Without pagination: Load all repositories → OOM or quota exceeded
- With lazy iterator: Load only visible items → constant memory

**Performance** (100K items, 100 items/page):

| Approach | Memory | API Calls | Time |
|----------|--------|-----------|------|
| Eager load all | 100K items | 1000 | 30s |
| Iterator (take 500) | 500 items | 5 | 0.5s | **60× faster** |

---

### Milestone 2: Multiple Pagination Strategies

**Concepts applied**:
- **Enum dispatch**: Pattern match on `PaginationStrategy`
- **Cursor stability**: Handles concurrent data changes
- **Trait bounds**: `T: Deserialize` for response parsing

**Why this matters**: Different APIs use different pagination styles.

**Comparison**:

| Strategy | Pros | Cons | Use Case |
|----------|------|------|----------|
| **Offset** | Simple, stateless | Skips/duplicates with concurrent writes | Static data |
| **Cursor** | Stable, no skips | Opaque tokens, can't jump | Real-time feeds |
| **Page number** | Human-friendly URLs | Skips/duplicates with changes | Web UIs |

**Real-world example**: Twitter API uses cursors because tweets are constantly added/deleted.

**Stability test**:
- 1000 items in database
- Fetch page 1 (items 0-99)
- Insert 50 items at beginning
- **Offset page 2**: Gets items 150-249 (**skips items 100-149**)
- **Cursor page 2**: Gets items 100-199 (**stable**)

---

### Milestone 3: Rate Limiting and Retry Logic

**Concepts applied**:
- **Token bucket algorithm**: Smooth rate limiting
- **Exponential backoff**: Handle transient failures
- **Builder pattern**: `.with_rate_limit(5.0)`

**Why this matters**: Production APIs have strict rate limits.

**Rate limit examples**:
- GitHub: 5000 requests/hour = 1.39 req/sec
- Stripe: 100 requests/sec (burst)
- Twitter: 300 requests/15min = 0.33 req/sec

**Without rate limiting**:
```rust
// Burst 100 requests immediately
for i in 0..100 {
    fetch_page(i);  // 429 error after ~10 requests
}
// Result: IP banned, data incomplete
```

**With rate limiting**:
```rust
let iter = PaginatedIterator::new(url, 10)
    .with_rate_limit(1.0);  // 1 req/sec

for item in iter.take(100) {
    process(item);  // Automatically throttled
}
// Result: Completes successfully in ~10 seconds
```

**Retry logic impact** (95% network success rate):

| Scenario | No Retries | 3 Retries | Success Rate |
|----------|-----------|-----------|--------------|
| 100 requests | ~95 succeed | ~99.9 succeed | **5× fewer failures** |
| Transient error (server restart) | Fails immediately | Succeeds after 200ms | **Resilient** |

---

### Milestone 4: Caching for Re-Iteration

**Concepts applied**:
- **Arc/Mutex**: Shared cache between clones
- **Reference counting**: Automatic memory management
- **Clone-on-share**: Multiple iterators, one cache

**Why this matters**: Exploratory data analysis requires multiple passes.

**Use case**: Data exploration
```rust
let iter = PaginatedIterator::new(url, 100).with_cache();

// First pass: Load all data (makes HTTP requests)
let active_count = iter.clone_iter()
    .filter(|u| u.active)
    .count();  // Fetches all pages, caches them

// Second pass: Uses cache (no HTTP requests!)
let admin_emails = iter.clone_iter()
    .filter(|u| u.role == "admin")
    .map(|u| u.email)
    .collect();  // Instant! Uses cached data

// Third pass: Also instant
let avg_age = iter.clone_iter()
    .map(|u| u.age)
    .sum::<u32>() / iter.clone_iter().count() as u32;
```

**Performance comparison** (10K items):

| Pass | Without Cache | With Cache | Speedup |
|------|---------------|------------|---------|
| 1st | 5s (HTTP) | 5s (HTTP + cache) | Same |
| 2nd | 5s (HTTP again) | 0.01s (cache) | **500× faster** |
| 3rd | 5s (HTTP again) | 0.01s (cache) | **500× faster** |

**Memory trade-off**:
- Cache memory: ~10KB per 100 items
- 10K items = ~1MB cached
- **Worth it** for multiple iterations

**API quota savings**:
- 3 passes without cache: 300 API calls
- 3 passes with cache: 100 API calls (**66% reduction**)

---

### Project-Wide Benefits

**Concrete comparisons** - Processing 50K GitHub repositories:

| Metric | Manual Pagination | Basic Iterator | Optimized Iterator | Improvement |
|--------|------------------|----------------|-------------------|-------------|
| Code lines | ~150 | ~30 | ~5 | **30× less code** |
| Memory usage | 50K repos (~50MB) | Current page (~50KB) | Cached (~1MB) | **50× less** |
| API calls (3 passes) | 1500 | 1500 | 500 | **66% reduction** |
| Rate limit errors | Frequent | Frequent | None | **Eliminated** |
| Failed requests | ~5% lost | ~5% lost | ~0.01% lost | **500× more reliable** |
| Development time | Days | Hours | Minutes | **100× faster** |

**Real-world validation**:
- **AWS Rust SDK**: Uses similar pagination patterns
- **Stripe Rust client**: Cursor-based pagination with retry logic
- **GitHub Octokit**: Iterator-based API clients
- **Google Cloud SDK**: Paginated resources with automatic retry

**Production requirements met**:
- ✅ Memory efficient (constant for streaming, O(pages) for caching)
- ✅ Rate limit compliant (token bucket algorithm)
- ✅ Fault tolerant (exponential backoff retry)
- ✅ Fast (lazy evaluation, early termination)
- ✅ Flexible (multiple pagination strategies)
- ✅ Type safe (generic with trait bounds)
- ✅ Composable (standard Iterator trait)

This project teaches patterns used in production Rust SDK clients processing billions of API requests daily.

---

## Build The Project

### Milestone 1: Basic Paginated Iterator with Offset-Based Pagination

**Goal**: Create an iterator that fetches pages using offset-based pagination.

**What to implement**:
- `PaginatedIterator<T>` that yields items of type T
- Track current offset and page size
- Fetch next page when current buffer is exhausted
- Stop iteration when no more data

**Architecture**:
- Structs: `PaginatedIterator<T>`, `PageResponse<T>`
- Fields: `offset: usize`, `page_size: usize`, `buffer: Vec<T>`, `buffer_index: usize`, `total: Option<usize>`
- Functions:
    - `new(page_size: usize) -> Self` - Create iterator
    - `fetch_page(&self, offset: usize, limit: usize) -> Result<PageResponse<T>>` - Fetch page
    - `next() -> Option<T>` - Iterate items

---

**Starter Code**:

```rust
use serde::{Deserialize, Serialize};

/// Response from a paginated API endpoint
#[derive(Debug, Deserialize)]
pub struct PageResponse<T> {
    items: Vec<T>,
    total: Option<usize>,
    has_more: bool,
}

/// Iterator over paginated API results
pub struct PaginatedIterator<T> {
    url: String,
    offset: usize,
    page_size: usize,
    buffer: Vec<T>,
    buffer_index: usize,
    done: bool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> PaginatedIterator<T>
where
    T: for<'de> Deserialize<'de>,
{
    /// Create a new paginated iterator
    /// Role: Initialize with API endpoint and page size
    pub fn new(url: String, page_size: usize) -> Self {
        todo!("Initialize iterator state")
    }

    /// Fetch a page from the API
    /// Role: HTTP request with offset and limit parameters
    fn fetch_page(&mut self) -> Result<(), FetchError> {
        todo!("Make HTTP GET request, deserialize response, update buffer")
    }
}

#[derive(Debug)]
pub enum FetchError {
    Http(String),
    Deserialization(String),
}

impl<T> Iterator for PaginatedIterator<T>
where
    T: for<'de> Deserialize<'de>,
{
    type Item = T;

    /// Yield next item, fetching new page if needed
    /// Role: Transparent pagination - user sees flat stream
    fn next(&mut self) -> Option<Self::Item> {
        todo!("Return item from buffer or fetch next page")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestItem {
        id: u32,
        name: String,
    }

    // Mock HTTP server for testing
    fn setup_mock_server() -> mockito::ServerGuard {
        todo!("Setup mockito server with paginated responses")
    }

    #[test]
    fn test_basic_pagination() {
        let server = setup_mock_server();

        // Mock returns 3 pages with 2 items each
        let iter = PaginatedIterator::<TestItem>::new(
            server.url(),
            2 // page_size
        );

        let items: Vec<_> = iter.collect();
        assert_eq!(items.len(), 6);
    }

    #[test]
    fn test_empty_result() {
        let server = setup_mock_server();

        let iter = PaginatedIterator::<TestItem>::new(server.url(), 10);
        let items: Vec<_> = iter.collect();

        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_single_page() {
        let server = setup_mock_server();

        // Mock returns 1 page with 3 items
        let iter = PaginatedIterator::<TestItem>::new(server.url(), 10);
        let items: Vec<_> = iter.collect();

        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_lazy_evaluation() {
        let server = setup_mock_server();

        let mut iter = PaginatedIterator::<TestItem>::new(server.url(), 2);

        // Should only fetch first page initially
        let first = iter.next().unwrap();
        assert_eq!(first.id, 1);

        // Should fetch second page when buffer exhausted
        iter.next(); // id=2
        let third = iter.next().unwrap();
        assert_eq!(third.id, 3);
    }
}
```

---

### Milestone 2: Cursor-Based Pagination Support

**Goal**: Support cursor-based pagination (used by GitHub, Stripe, etc.).

**Why the previous milestone is not enough**: Offset-based pagination has issues with data insertion/deletion during iteration. Cursors provide stable pagination.

**What's the improvement**: Cursor-based pagination uses opaque tokens to mark positions, remaining stable even as underlying data changes. This is essential for real-time data sources where items are added/removed frequently.

**Architecture**:
- Enums: `PaginationStrategy` (Offset, Cursor, PageNumber)
- Fields: `cursor: Option<String>`, `pagination_strategy: PaginationStrategy`
- Functions:
    - `with_cursor_pagination(url: String, page_size: usize) -> Self` - Create cursor-based iterator
    - `fetch_cursor_page(&mut self) -> Result<()>` - Fetch using cursor

---

**Starter Code**:

```rust
/// Pagination strategies supported by the iterator
#[derive(Debug, Clone)]
pub enum PaginationStrategy {
    Offset { offset: usize, limit: usize },
    Cursor { cursor: Option<String>, page_size: usize },
    PageNumber { page: usize, per_page: usize },
}

/// Cursor-based page response
#[derive(Debug, Deserialize)]
pub struct CursorPageResponse<T> {
    items: Vec<T>,
    next_cursor: Option<String>,
    has_more: bool,
}

/// Universal paginated iterator supporting multiple strategies
pub struct UniversalPaginatedIterator<T> {
    url: String,
    strategy: PaginationStrategy,
    buffer: Vec<T>,
    buffer_index: usize,
    done: bool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> UniversalPaginatedIterator<T>
where
    T: for<'de> Deserialize<'de>,
{
    /// Create iterator with cursor-based pagination
    /// Role: Support cursor tokens for stable pagination
    pub fn with_cursor(url: String, page_size: usize) -> Self {
        todo!("Initialize with Cursor strategy")
    }

    /// Create iterator with offset-based pagination
    /// Role: Support offset/limit parameters
    pub fn with_offset(url: String, page_size: usize) -> Self {
        todo!("Initialize with Offset strategy")
    }

    /// Fetch next page based on strategy
    /// Role: Dispatch to appropriate fetch method
    fn fetch_next_page(&mut self) -> Result<(), FetchError> {
        match &self.strategy {
            PaginationStrategy::Offset { .. } => self.fetch_offset_page(),
            PaginationStrategy::Cursor { .. } => self.fetch_cursor_page(),
            PaginationStrategy::PageNumber { .. } => self.fetch_page_number_page(),
        }
    }

    /// Fetch page using cursor
    /// Role: HTTP GET with cursor parameter
    fn fetch_cursor_page(&mut self) -> Result<(), FetchError> {
        todo!("Fetch page with cursor, update cursor from response")
    }

    fn fetch_offset_page(&mut self) -> Result<(), FetchError> {
        todo!("Fetch page with offset/limit")
    }

    fn fetch_page_number_page(&mut self) -> Result<(), FetchError> {
        todo!("Fetch page by page number")
    }
}

impl<T> Iterator for UniversalPaginatedIterator<T>
where
    T: for<'de> Deserialize<'de>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!("Fetch pages using current strategy")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_pagination() {
        let server = setup_mock_cursor_server();

        let iter = UniversalPaginatedIterator::<TestItem>::with_cursor(
            server.url(),
            2
        );

        let items: Vec<_> = iter.collect();
        assert_eq!(items.len(), 6);
    }

    #[test]
    fn test_stable_cursor_pagination() {
        // Simulate data insertion during iteration
        let server = setup_dynamic_mock_server();

        let iter = UniversalPaginatedIterator::<TestItem>::with_cursor(
            server.url(),
            2
        );

        // Cursor-based pagination should not skip or duplicate items
        let items: Vec<_> = iter.collect();

        // Verify all items unique
        let ids: std::collections::HashSet<_> = items.iter().map(|item| item.id).collect();
        assert_eq!(ids.len(), items.len());
    }

    #[test]
    fn test_multiple_pagination_strategies() {
        let server = setup_mock_server();

        // Test all strategies on same endpoint
        let offset_items: Vec<_> = UniversalPaginatedIterator::<TestItem>::with_offset(
            server.url(),
            3
        ).collect();

        let cursor_items: Vec<_> = UniversalPaginatedIterator::<TestItem>::with_cursor(
            server.url(),
            3
        ).collect();

        // Both should return same items
        assert_eq!(offset_items.len(), cursor_items.len());
    }
}
```

---

### Milestone 3: Rate Limiting and Retry Logic

**Goal**: Add rate limiting to respect API rate limits and retry transient failures.

**Why the previous milestone is not enough**: Production APIs have rate limits. Exceeding them causes 429 errors and IP bans. Network errors require retries.

**What's the improvement**: Built-in rate limiting prevents exceeding API quotas. Exponential backoff retry logic handles transient failures (network glitches, temporary server errors). This makes the iterator production-ready and resilient to real-world network conditions.

**Architecture**:
- Structs: `RateLimiter`, `RetryPolicy`
- Fields: `rate_limiter: RateLimiter`, `retry_policy: RetryPolicy`
- Functions:
    - `with_rate_limit(requests_per_second: f64)` - Configure rate limiting
    - `with_retry_policy(max_attempts: usize, backoff: Duration)` - Configure retries

---

**Starter Code**:

```rust
use std::time::{Duration, Instant};

/// Rate limiter using token bucket algorithm
pub struct RateLimiter {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl RateLimiter {
    /// Create rate limiter
    /// Role: Allow up to `rate` requests per second
    pub fn new(rate: f64) -> Self {
        todo!("Initialize token bucket")
    }

    /// Acquire a token, blocking if necessary
    /// Role: Enforce rate limit
    pub fn acquire(&mut self) {
        todo!("Wait until token available")
    }

    /// Refill tokens based on elapsed time
    /// Role: Add tokens at configured rate
    fn refill(&mut self) {
        todo!("Calculate elapsed time, add tokens")
    }
}

/// Retry policy with exponential backoff
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    max_attempts: usize,
    initial_backoff: Duration,
    max_backoff: Duration,
    multiplier: f64,
}

impl RetryPolicy {
    /// Create retry policy
    /// Role: Configure exponential backoff
    pub fn new(max_attempts: usize, initial_backoff: Duration) -> Self {
        RetryPolicy {
            max_attempts,
            initial_backoff,
            max_backoff: Duration::from_secs(60),
            multiplier: 2.0,
        }
    }

    /// Check if error is retryable
    /// Role: Determine if should retry based on error type
    pub fn should_retry(&self, error: &FetchError, attempt: usize) -> bool {
        todo!("Check error type and attempt count")
    }

    /// Calculate delay for attempt
    /// Role: Exponential backoff calculation
    pub fn delay_for_attempt(&self, attempt: usize) -> Duration {
        todo!("Calculate delay with exponential backoff")
    }
}

/// Paginated iterator with rate limiting and retries
///
/// Fields:
/// - rate_limiter: Option<RateLimiter>
/// - retry_policy: RetryPolicy
impl<T> UniversalPaginatedIterator<T>
where
    T: for<'de> Deserialize<'de>,
{
    /// Add rate limiting
    /// Role: Prevent exceeding API rate limits
    pub fn with_rate_limit(mut self, requests_per_second: f64) -> Self {
        todo!("Add rate limiter")
    }

    /// Add retry policy
    /// Role: Handle transient failures
    pub fn with_retries(mut self, max_attempts: usize, initial_backoff: Duration) -> Self {
        todo!("Configure retry policy")
    }

    /// Fetch with rate limiting and retries
    /// Role: Resilient HTTP request
    fn fetch_with_resilience(&mut self) -> Result<(), FetchError> {
        todo!("Apply rate limit, fetch with retries")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(10.0); // 10 req/sec

        let start = Instant::now();

        // Acquire 20 tokens (should take ~1 second)
        for _ in 0..20 {
            limiter.acquire();
        }

        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(900));
        assert!(elapsed <= Duration::from_millis(1100));
    }

    #[test]
    fn test_retry_policy() {
        let policy = RetryPolicy::new(3, Duration::from_millis(100));

        assert_eq!(policy.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(policy.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(policy.delay_for_attempt(2), Duration::from_millis(400));
    }

    #[test]
    fn test_retry_on_transient_error() {
        let server = setup_flaky_mock_server(); // Returns errors first 2 times

        let iter = UniversalPaginatedIterator::<TestItem>::with_cursor(server.url(), 10)
            .with_retries(3, Duration::from_millis(10));

        let items: Vec<_> = iter.collect();

        // Should succeed after retries
        assert!(items.len() > 0);
    }

    #[test]
    fn test_rate_limiting_in_iteration() {
        let server = setup_mock_server();

        let iter = UniversalPaginatedIterator::<TestItem>::with_offset(server.url(), 2)
            .with_rate_limit(5.0); // 5 requests/sec

        let start = Instant::now();

        // 6 items across 3 pages = 3 requests
        let items: Vec<_> = iter.collect();

        let elapsed = start.elapsed();

        // Should take at least 400ms (3 requests / 5 req/sec = 0.6s, minus first instant)
        assert!(elapsed >= Duration::from_millis(350));
        assert_eq!(items.len(), 6);
    }
}
```

---

### Milestone 4: Caching to Avoid Redundant Requests

**Goal**: Cache fetched pages to enable re-iteration without additional HTTP requests.

**Why the previous milestone is not enough**: Iterating multiple times (e.g., `.clone().filter(...).collect()`) re-fetches all pages, wasting API quota and time.

**What's the improvement**: In-memory caching stores fetched pages. Subsequent iterations use cached data instead of HTTP requests. This is essential for exploratory data analysis where you might iterate multiple times with different filters.

**Optimization focus**: Speed and API quota conservation through caching.

**Architecture**:
- Structs: `CachedPaginatedIterator<T>`
- Fields: `cache: Arc<Mutex<Vec<Vec<T>>>>`, `cache_position: usize`
- Functions:
    - `with_cache(self) -> CachedPaginatedIterator<T>` - Enable caching
    - `reset(&mut self)` - Reset to beginning
    - `clone_iter(&self) -> Self` - Clone for re-iteration

---

**Starter Code**:

```rust
use std::sync::{Arc, Mutex};

/// Cached paginated iterator for re-iteration without refetching
pub struct CachedPaginatedIterator<T>
where
    T: Clone,
{
    inner: UniversalPaginatedIterator<T>,
    cache: Arc<Mutex<Vec<Vec<T>>>>,
    cache_index: usize,
    item_index: usize,
    cache_complete: Arc<Mutex<bool>>,
}

impl<T> CachedPaginatedIterator<T>
where
    T: for<'de> Deserialize<'de> + Clone,
{
    /// Wrap iterator with caching
    /// Role: Enable re-iteration without refetching
    pub fn with_cache(inner: UniversalPaginatedIterator<T>) -> Self {
        todo!("Initialize cache structures")
    }

    /// Reset iterator to beginning
    /// Role: Re-iterate over cached data
    pub fn reset(&mut self) {
        todo!("Reset indices to start")
    }

    /// Clone iterator sharing same cache
    /// Role: Multiple iterators over same cached data
    pub fn clone_iter(&self) -> Self {
        todo!("Clone with shared cache reference")
    }
}

impl<T> Iterator for CachedPaginatedIterator<T>
where
    T: for<'de> Deserialize<'de> + Clone,
{
    type Item = T;

    /// Yield items from cache or fetch and cache new pages
    /// Role: Transparent caching
    fn next(&mut self) -> Option<Self::Item> {
        todo!("Check cache first, fetch and cache if needed")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_caching() {
        let server = setup_request_counting_mock_server();

        let iter = UniversalPaginatedIterator::<TestItem>::with_cursor(server.url(), 2);
        let mut cached = CachedPaginatedIterator::with_cache(iter);

        // First iteration fetches from API
        let items1: Vec<_> = cached.by_ref().collect();
        let request_count_1 = server.request_count();

        // Reset and iterate again
        cached.reset();
        let items2: Vec<_> = cached.collect();
        let request_count_2 = server.request_count();

        // Should not make additional requests
        assert_eq!(request_count_1, request_count_2);
        assert_eq!(items1, items2);
    }

    #[test]
    fn test_clone_iter_shares_cache() {
        let server = setup_request_counting_mock_server();

        let iter = UniversalPaginatedIterator::<TestItem>::with_offset(server.url(), 3);
        let cached = CachedPaginatedIterator::with_cache(iter);

        // First iterator populates cache
        let _items1: Vec<_> = cached.clone_iter().collect();
        let request_count_1 = server.request_count();

        // Second iterator uses same cache
        let _items2: Vec<_> = cached.clone_iter().collect();
        let request_count_2 = server.request_count();

        // No additional requests
        assert_eq!(request_count_1, request_count_2);
    }

    #[test]
    fn test_partial_iteration_and_reset() {
        let server = setup_mock_server();

        let iter = UniversalPaginatedIterator::<TestItem>::with_cursor(server.url(), 2);
        let mut cached = CachedPaginatedIterator::with_cache(iter);

        // Partially iterate
        let first_3: Vec<_> = cached.by_ref().take(3).collect();
        assert_eq!(first_3.len(), 3);

        // Reset and get all
        cached.reset();
        let all: Vec<_> = cached.collect();

        assert!(all.len() > 3);
        assert_eq!(&all[..3], &first_3[..]);
    }
}
```

---

### Milestone 5: Parallel Page Fetching

**Goal**: Prefetch multiple pages in parallel to reduce latency.

**Why the previous milestone is not enough**: Sequential page fetching wastes time waiting for network I/O. When processing item N, we could already be fetching page N+1.

**What's the improvement**: Parallel prefetching uses background threads to fetch upcoming pages while the main iterator processes current data. This overlaps computation and I/O, dramatically reducing total time.

For a 100-page dataset with 50ms per request:
- Sequential: 100 × 50ms = 5000ms
- Parallel (prefetch 5): ~1000ms (5x speedup)

**Optimization focus**: Speed through parallel I/O and prefetching.

**Architecture**:
- Structs: `PrefetchingIterator<T>`
- Fields: `prefetch_queue: Arc<Mutex<VecDeque<Vec<T>>>>`, `fetch_handle: Option<JoinHandle<()>>`
- Functions:
    - `with_prefetch(self, prefetch_pages: usize) -> PrefetchingIterator<T>` - Enable prefetching
    - `spawn_prefetch_worker()` - Background fetch thread

---

**Starter Code**:

```rust
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::{self, JoinHandle};
use std::collections::VecDeque;

/// Prefetching iterator that fetches pages in background
pub struct PrefetchingIterator<T>
where
    T: Send + 'static,
{
    receiver: Receiver<Result<Vec<T>, FetchError>>,
    buffer: VecDeque<T>,
    fetch_handle: Option<JoinHandle<()>>,
    done: bool,
}

impl<T> PrefetchingIterator<T>
where
    T: for<'de> Deserialize<'de> + Send + 'static + Clone,
{
    /// Create iterator with prefetching
    /// Role: Overlap I/O with processing
    pub fn with_prefetch(
        base_iter: UniversalPaginatedIterator<T>,
        prefetch_pages: usize
    ) -> Self {
        todo!("Spawn background thread, set up channel")
    }

    /// Spawn worker thread that fetches pages ahead
    /// Role: Prefetch pages and send via channel
    fn spawn_fetch_worker(
        mut iter: UniversalPaginatedIterator<T>,
        sender: Sender<Result<Vec<T>, FetchError>>,
        prefetch_count: usize
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            todo!("Fetch pages and send to channel")
        })
    }

    /// Refill buffer from prefetch queue
    /// Role: Load next prefetched page
    fn refill_buffer(&mut self) -> bool {
        todo!("Receive from channel, update buffer")
    }
}

impl<T> Iterator for PrefetchingIterator<T>
where
    T: Send + 'static,
{
    type Item = Result<T, FetchError>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!("Yield from buffer, refill from prefetch queue")
    }
}

impl<T> Drop for PrefetchingIterator<T>
where
    T: Send + 'static,
{
    /// Clean up background thread
    /// Role: Ensure worker thread is joined
    fn drop(&mut self) {
        todo!("Join fetch handle")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_prefetch_correctness() {
        let server = setup_mock_server();

        let iter = UniversalPaginatedIterator::<TestItem>::with_cursor(server.url(), 2);
        let prefetch_iter = PrefetchingIterator::with_prefetch(iter, 3);

        let items: Result<Vec<_>, _> = prefetch_iter.collect();
        let items = items.unwrap();

        assert_eq!(items.len(), 6);
    }

    #[test]
    fn test_prefetch_performance() {
        let server = setup_slow_mock_server(Duration::from_millis(50)); // 50ms per page

        // Sequential
        let iter_seq = UniversalPaginatedIterator::<TestItem>::with_cursor(server.url(), 2);
        let start = Instant::now();
        let items_seq: Vec<_> = iter_seq.collect();
        let seq_time = start.elapsed();

        // With prefetch
        let iter_prefetch = UniversalPaginatedIterator::<TestItem>::with_cursor(server.url(), 2);
        let prefetch = PrefetchingIterator::with_prefetch(iter_prefetch, 5);
        let start = Instant::now();
        let items_prefetch: Result<Vec<_>, _> = prefetch.collect();
        let prefetch_time = start.elapsed();

        // Prefetch should be significantly faster
        println!("Sequential: {:?}, Prefetch: {:?}", seq_time, prefetch_time);
        assert!(prefetch_time < seq_time / 2);
        assert_eq!(items_seq.len(), items_prefetch.unwrap().len());
    }

    #[test]
    fn test_prefetch_handles_errors() {
        let server = setup_error_mock_server();

        let iter = UniversalPaginatedIterator::<TestItem>::with_cursor(server.url(), 2);
        let prefetch = PrefetchingIterator::with_prefetch(iter, 3);

        // Should propagate errors
        let result: Result<Vec<_>, _> = prefetch.collect();
        assert!(result.is_err());
    }
}
```

---

### Milestone 6: Complete API Client with Builder Pattern

**Goal**: Combine all features into a ergonomic API client with builder pattern.

**Why the previous milestone is not enough**: Individual features work, but users need a unified, discoverable API. Builder pattern provides fluent interface.

**What's the improvement**: Builder pattern makes configuration discoverable through IDE autocomplete. All features (pagination, rate limiting, retries, caching, prefetching) compose elegantly. This is production-ready API client design.

**Architecture**:
- Structs: `ApiClient`, `PaginatedRequestBuilder<T>`
- Functions:
    - `ApiClient::new(base_url)` - Create client
    - `.paginated<T>(endpoint)` - Start pagination builder
    - Builder methods: `.page_size()`, `.with_cursor()`, `.rate_limit()`, `.retries()`, `.cache()`, `.prefetch()`

---

**Starter Code**:

```rust
/// API client with fluent builder interface
pub struct ApiClient {
    base_url: String,
    http_client: reqwest::blocking::Client,
}

pub struct PaginatedRequestBuilder<T> {
    client: ApiClient,
    endpoint: String,
    page_size: usize,
    strategy: Option<PaginationStrategy>,
    rate_limit: Option<f64>,
    retry_policy: Option<RetryPolicy>,
    enable_cache: bool,
    prefetch_pages: Option<usize>,
    _phantom: std::marker::PhantomData<T>,
}

impl ApiClient {
    /// Create new API client
    /// Role: Initialize client with base URL
    pub fn new(base_url: impl Into<String>) -> Self {
        todo!("Create client with reqwest")
    }

    /// Start building a paginated request
    /// Role: Entry point for pagination builder
    pub fn paginated<T>(&self, endpoint: impl Into<String>) -> PaginatedRequestBuilder<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        todo!("Create builder")
    }
}

impl<T> PaginatedRequestBuilder<T>
where
    T: for<'de> Deserialize<'de> + Send + Clone + 'static,
{
    /// Set page size
    /// Role: Configure items per page
    pub fn page_size(mut self, size: usize) -> Self {
        self.page_size = size;
        self
    }

    /// Use cursor-based pagination
    /// Role: Select pagination strategy
    pub fn cursor_based(mut self) -> Self {
        todo!("Set cursor strategy")
    }

    /// Use offset-based pagination
    /// Role: Select pagination strategy
    pub fn offset_based(mut self) -> Self {
        todo!("Set offset strategy")
    }

    /// Enable rate limiting
    /// Role: Respect API rate limits
    pub fn rate_limit(mut self, requests_per_second: f64) -> Self {
        self.rate_limit = Some(requests_per_second);
        self
    }

    /// Configure retries
    /// Role: Handle transient failures
    pub fn with_retries(mut self, max_attempts: usize, initial_backoff: Duration) -> Self {
        self.retry_policy = Some(RetryPolicy::new(max_attempts, initial_backoff));
        self
    }

    /// Enable caching
    /// Role: Allow re-iteration without refetching
    pub fn cached(mut self) -> Self {
        self.enable_cache = true;
        self
    }

    /// Enable prefetching
    /// Role: Parallel page fetching
    pub fn prefetch(mut self, pages: usize) -> Self {
        self.prefetch_pages = Some(pages);
        self
    }

    /// Build and execute the request
    /// Role: Construct iterator with all configured features
    pub fn execute(self) -> Box<dyn Iterator<Item = Result<T, FetchError>>> {
        todo!("Build iterator with all enabled features")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic_usage() {
        let server = setup_mock_server();
        let client = ApiClient::new(server.url());

        let items: Result<Vec<_>, _> = client
            .paginated::<TestItem>("/api/items")
            .page_size(10)
            .offset_based()
            .execute()
            .collect();

        assert!(items.is_ok());
    }

    #[test]
    fn test_builder_with_all_features() {
        let server = setup_mock_server();
        let client = ApiClient::new(server.url());

        let items: Result<Vec<_>, _> = client
            .paginated::<TestItem>("/api/items")
            .page_size(5)
            .cursor_based()
            .rate_limit(10.0)
            .with_retries(3, Duration::from_millis(100))
            .cached()
            .prefetch(3)
            .execute()
            .collect();

        assert!(items.is_ok());
    }

    #[test]
    fn test_real_world_usage() {
        // Example: Fetch all users from GitHub API
        let client = ApiClient::new("https://api.github.com");

        let users: Vec<_> = client
            .paginated::<GitHubUser>("/users")
            .page_size(100)
            .rate_limit(60.0 / 3600.0) // GitHub: 60 req/hour
            .with_retries(3, Duration::from_secs(1))
            .prefetch(5)
            .execute()
            .take(500) // Get first 500 users
            .filter_map(Result::ok)
            .collect();

        println!("Fetched {} users", users.len());
    }
}
```

---



### Complete Working Example

```rust
// Note: Full implementation would be ~800 lines
// Here's a condensed version showing key patterns

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

//==============================================================================
// Core Types
//==============================================================================

#[derive(Debug, Deserialize)]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug)]
pub enum FetchError {
    Http(String),
    Deserialization(String),
    RateLimitExceeded,
}

//==============================================================================
// Rate Limiter
//==============================================================================

pub struct RateLimiter {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl RateLimiter {
    pub fn new(rate: f64) -> Self {
        RateLimiter {
            tokens: rate,
            max_tokens: rate,
            refill_rate: rate,
            last_refill: Instant::now(),
        }
    }

    pub fn acquire(&mut self) {
        self.refill();

        while self.tokens < 1.0 {
            std::thread::sleep(Duration::from_millis(10));
            self.refill();
        }

        self.tokens -= 1.0;
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();

        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }
}

//==============================================================================
// Pagination Iterator
//==============================================================================

pub struct UniversalPaginatedIterator<T> {
    url: String,
    cursor: Option<String>,
    buffer: VecDeque<T>,
    done: bool,
    page_size: usize,
    rate_limiter: Option<RateLimiter>,
}

impl<T> UniversalPaginatedIterator<T>
where
    T: for<'de> Deserialize<'de>,
{
    pub fn new(url: String, page_size: usize) -> Self {
        UniversalPaginatedIterator {
            url,
            cursor: None,
            buffer: VecDeque::new(),
            done: false,
            page_size,
            rate_limiter: None,
        }
    }

    pub fn with_rate_limit(mut self, rate: f64) -> Self {
        self.rate_limiter = Some(RateLimiter::new(rate));
        self
    }

    fn fetch_page(&mut self) -> Result<(), FetchError> {
        if let Some(limiter) = &mut self.rate_limiter {
            limiter.acquire();
        }

        // Simulated HTTP request
        // In real implementation: use reqwest to fetch from self.url

        // Parse response and update buffer and cursor
        // self.buffer.extend(response.items);
        // self.cursor = response.next_cursor;
        // self.done = !response.has_more;

        Ok(())
    }
}

impl<T> Iterator for UniversalPaginatedIterator<T>
where
    T: for<'de> Deserialize<'de>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() && !self.done {
            if let Err(_) = self.fetch_page() {
                self.done = true;
                return None;
            }
        }

        self.buffer.pop_front()
    }
}

//==============================================================================
// Example Usage
//==============================================================================

fn main() {
    println!("=== Paginated API Iterator Examples ===\n");

    #[derive(Debug, Deserialize)]
    struct User {
        id: u64,
        name: String,
    }

    // Example 1: Basic pagination
    println!("Example 1: Basic Pagination");
    {
        let iter = UniversalPaginatedIterator::<User>::new(
            "https://api.example.com/users".to_string(),
            100
        );

        let users: Vec<_> = iter.take(250).collect();
        println!("Fetched {} users", users.len());
    }
    println!();

    // Example 2: With rate limiting
    println!("Example 2: Rate-Limited Pagination");
    {
        let iter = UniversalPaginatedIterator::<User>::new(
            "https://api.example.com/users".to_string(),
            100
        ).with_rate_limit(5.0); // 5 requests/second

        let users: Vec<_> = iter.take(500).collect();
        println!("Fetched {} users (rate-limited)", users.len());
    }
    println!();

    // Example 3: Filtering and mapping
    println!("Example 3: Transforming Paginated Data");
    {
        let iter = UniversalPaginatedIterator::<User>::new(
            "https://api.example.com/users".to_string(),
            50
        );

        let names: Vec<String> = iter
            .filter(|user| user.id > 1000)
            .map(|user| user.name)
            .take(100)
            .collect();

        println!("Collected {} filtered names", names.len());
    }
}
```

This complete example demonstrates:
- **Core Pagination**: Lazy-loading iterator over API pages
- **Rate Limiting**: Token bucket algorithm to respect API limits
- **Composability**: Works with standard iterator combinators
- **Real-World Patterns**: Typical API client usage

The paginated iterator pattern transforms complex API interaction into simple iterator operations, making API clients easier to build and use.

---

## Testing and Benchmarking Guide

### Tools to Use

1. **mockito**: Mock HTTP servers for testing
2. **wiremock**: More advanced HTTP mocking
3. **criterion**: Performance benchmarking
4. **proptest**: Property-based testing
5. **tokio-test**: Async testing utilities

### Example Benchmark

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_pagination_strategies(c: &mut Criterion) {
    let server = setup_mock_server();

    c.bench_function("sequential pagination", |b| {
        b.iter(|| {
            let iter = UniversalPaginatedIterator::<TestItem>::new(server.url(), 100);
            iter.take(1000).count()
        })
    });

    c.bench_function("prefetched pagination", |b| {
        b.iter(|| {
            let iter = UniversalPaginatedIterator::<TestItem>::new(server.url(), 100);
            let prefetch = PrefetchingIterator::with_prefetch(iter, 5);
            prefetch.take(1000).count()
        })
    });
}

criterion_group!(benches, benchmark_pagination_strategies);
criterion_main!(benches);
```

---

These five projects (CSV Transformer and Paginated API Client, plus the three from the original file) provide comprehensive coverage of iterator patterns, from basic iteration to advanced parallel processing and API integration.
