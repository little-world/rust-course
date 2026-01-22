# Async Runtime Patterns
This chapter explores asynchronous programming patterns in Rust using async/await and async runtimes. We'll cover future composition, stream processing, concurrency patterns, timeout handling, and runtime comparisons through practical, production-ready examples.





## Pattern 1: Future Composition

**Problem**: Chaining async operations with nested `.await` calls creates deeply nested code (callback hell). Running multiple async operations concurrently with manual spawning is verbose.

**Solution**: Use future combinators: `map()`, `and_then()`, `or_else()` for chaining transformations. Use `join!` and `try_join!` to run futures concurrently, waiting for all.

**Why It Matters**: Proper composition determines performance and readability. Sequential `.await` on 3 independent operations takes 300ms; concurrent `join!` takes 100ms—3x faster.

**Use Cases**: HTTP request batching (parallel API calls), database query composition (dependent queries), microservice orchestration, retry logic with fallbacks, concurrent file operations, fan-out/fan-in patterns.


### to Cargo.toml
```toml
tokio = { version = "1.35", features = ["full"] }
reqwest = "0.11"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Example: Basic future composition with map

Chain async operations with synchronous transformations using combinators. The `.await` keyword completes the async call first, then `.map()` transforms the `Result` synchronously without requiring another await. This pattern keeps transformation logic clean and composable.

```rust
use std::time::Duration;

async fn fetch_user_name(user_id: u64) -> Result<String, String> {
    // Simulate API call
    tokio::time::sleep(Duration::from_millis(100)).await;

    if user_id == 0 {
        Err("Invalid user ID".to_string())
    } else {
        Ok(format!("User_{}", user_id))
    }
}

async fn get_user_name_uppercase(user_id: u64) -> Result<String, String> {
    // Map over the result: await completes async, map transforms sync
    fetch_user_name(user_id)
        .await
        .map(|name| name.to_uppercase())
}

#[tokio::main]
async fn main() {
    match get_user_name_uppercase(42).await {
        Ok(name) => println!("User: {}", name),  // "USER_42"
        Err(e) => println!("Error: {}", e),
    }
}
```

### Example: Chaining async operations

The `?` operator provides early return on error. If `fetch_user_name` fails, we immediately return that error without attempting to fetch posts. This pattern mirrors sequential function calls in synchronous code while maintaining clean error propagation.

```rust
async fn fetch_user_posts(user_id: u64) -> Result<Vec<String>, String> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    Ok(vec![
        format!("Post 1 by user {}", user_id),
        format!("Post 2 by user {}", user_id),
    ])
}

async fn get_user_with_posts(user_id: u64) -> Result<(String, Vec<String>), String> {
    let name = fetch_user_name(user_id).await?;  // Early return if fails
    let posts = fetch_user_posts(user_id).await?;
    Ok((name, posts))
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let (name, posts) = get_user_with_posts(1).await?;
    println!("{} has {} posts", name, posts.len());
    Ok(())
}
```

### Example: Error conversion and propagation

Create a unified error type implementing the `From` trait to automatically convert library-specific errors into your application error type. The `?` operator leverages these `From` implementations to convert `reqwest::Error` to `AppError::Network` automatically, enabling seamless error propagation.

```rust
#[derive(Debug)]
enum AppError {
    Network(String),
    NotFound,
    InvalidData(String),
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Network(err.to_string())
    }
}

async fn fetch_json_data(url: &str) -> Result<serde_json::Value, AppError> {
    let response = reqwest::get(url).await?;  // Auto-converts reqwest::Error

    if !response.status().is_success() {
        return Err(AppError::NotFound);
    }

    let data = response.json().await?;
    Ok(data)
}

#[tokio::main]
async fn main() {
    match fetch_json_data("https://api.github.com/users/rust-lang").await {
        Ok(data) => println!("Got: {}", data),
        Err(AppError::Network(e)) => println!("Network error: {}", e),
        Err(AppError::NotFound) => println!("Resource not found"),
        Err(AppError::InvalidData(e)) => println!("Bad data: {}", e),
    }
}
```

### Example: HTTP client with retries

Generic retry wrapper implementing exponential backoff for resilient HTTP requests. The closure-based design using `FnMut() -> Fut` allows retrying any async operation with configurable maximum attempts and increasing delays between retries to prevent overwhelming failing services.

```rust
async fn fetch_with_retry<F, Fut, T, E>(
    mut f: F,
    max_retries: usize,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempts = 0;

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;
                if attempts >= max_retries {
                    return Err(e);
                }
                println!("Attempt {} failed: {}. Retrying...", attempts, e);
                let delay = Duration::from_secs(2u64.pow(attempts as u32));
                tokio::time::sleep(delay).await;
            }
        }
    }
}

// Usage: wrap any async operation with retry logic
async fn fetch_data_with_retry(url: String) -> Result<String, reqwest::Error> {
    fetch_with_retry(
        || async { reqwest::get(&url).await?.text().await },
        3,  // max 3 attempts
    ).await
}

#[tokio::main]
async fn main() {
    match fetch_data_with_retry("https://api.example.com/data".to_string()).await {
        Ok(data) => println!("Fetched: {}", data),
        Err(e) => println!("Failed after retries: {}", e),
    }
}
```

**Future Composition Patterns**:
- **map**: Transform success value
- **and_then**: Chain dependent operations
- **or_else**: Handle errors and recover
- **? operator**: Early return on error

---

### Example: join! - wait for all futures

The `join!` macro runs futures concurrently, waiting for all to complete before continuing. Three independent 100ms operations finish in approximately 100ms total rather than 300ms sequentially. Returns a tuple containing all results in declaration order.

```rust
async fn concurrent_fetch() {
    // All three start immediately, complete in ~100ms total (not 300ms)
    let (result1, result2, result3) = tokio::join!(
        fetch_user_name(1),
        fetch_user_name(2),
        fetch_user_name(3),
    );

    println!("Results: {:?}, {:?}, {:?}", result1, result2, result3);
}

#[tokio::main]
async fn main() {
    concurrent_fetch().await;
}
```

### Example: try_join! - wait for all, fail fast on error

The `try_join!` macro works like `join!` but for `Result`-returning futures. If any future fails, it immediately cancels remaining futures and returns that error. On success, it unwraps all `Ok` values into a tuple for convenient destructuring.

```rust
async fn concurrent_fetch_fail_fast() -> Result<(String, String, String), String> {
    // If user 2 fails, user 3 is cancelled immediately
    tokio::try_join!(
        fetch_user_name(1),
        fetch_user_name(2),
        fetch_user_name(3),
    )
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let (u1, u2, u3) = concurrent_fetch_fail_fast().await?;
    println!("Users: {}, {}, {}", u1, u2, u3);
    Ok(())
}
```

### Example: select! - race futures, take first to complete

The `select!` macro races multiple futures concurrently, returning when the first one completes and automatically cancelling the others. This pattern is essential for implementing timeouts, redundant requests for reliability, and responding to cancellation signals.

```rust
use tokio::time::sleep;

async fn race_requests() -> String {
    tokio::select! {
        result = fetch_user_name(1) => {
            format!("Server 1 responded first: {:?}", result)
        }
        result = fetch_user_name(2) => {
            format!("Server 2 responded first: {:?}", result)
        }
        _ = sleep(Duration::from_secs(1)) => {
            "Both servers too slow - timeout".to_string()
        }
    }
}

#[tokio::main]
async fn main() {
    let winner = race_requests().await;
    println!("{}", winner);
}
```

### Example: Dynamic number of futures with FuturesUnordered

When the number of futures is determined at runtime, use `FuturesUnordered` to manage them efficiently. Results stream back in completion order with the fastest finishing first, rather than the original submission order, maximizing throughput.

```rust
use futures::stream::{FuturesUnordered, StreamExt};

async fn fetch_all_users(user_ids: Vec<u64>) -> Vec<Result<String, String>> {
    // Works with any number of IDs - determined at runtime
    let futures: FuturesUnordered<_> = user_ids
        .into_iter()
        .map(|id| fetch_user_name(id))
        .collect();

    // Results arrive in completion order, not submission order
    futures.collect().await
}

#[tokio::main]
async fn main() {
    let users = fetch_all_users(vec![1, 2, 3, 4, 5]).await;
    println!("Fetched {} users", users.len());
}

```
### Example: Parallel HTTP requests with limit

Processes URLs in fixed-size batches to limit concurrent connections. Each chunk runs in parallel via `join_all`, but chunks themselves execute sequentially. This approach provides simple concurrency limiting without requiring semaphores or complex coordination logic.

```rust
async fn fetch_urls_concurrently(
    urls: Vec<String>, max_concurrent: usize
) -> Vec<Result<String, reqwest::Error>> {
    let mut results = Vec::new();

    for chunk in urls.chunks(max_concurrent) {
        let futures: Vec<_> = chunk
            .iter()
            .map(|url| async move {
                reqwest::get(url)
                    .await?
                    .text()
                    .await
            })
            .collect();

        let chunk_results = futures::future::join_all(futures).await;
        results.extend(chunk_results);
    }

    results
}

#[tokio::main]
async fn main() {
    let urls: Vec<_> = (0..10)
        .map(|i| format!("https://httpbin.org/get?id={}", i))
        .collect();
    let results = fetch_urls_concurrently(urls, 3).await;
    println!("Fetched {} URLs ({} succeeded)",
        results.len(),
        results.iter().filter(|r| r.is_ok()).count());
}
```

### Example: Timeout wrapper

A generic wrapper function that adds timeout capability to any future. Returns `Ok(result)` if the operation completes within the specified duration, or `Err(Elapsed)` if the timeout expires. The wrapped future is automatically cancelled when timeout occurs.

```rust
async fn with_timeout<F, T>(
    future: F,
    duration: Duration,
) -> Result<T, tokio::time::error::Elapsed>
where
    F: std::future::Future<Output = T>,
{
    tokio::time::timeout(duration, future).await
}

#[tokio::main]
async fn main() {
    match with_timeout(
        async { tokio::time::sleep(Duration::from_millis(50)).await; "done" },
        Duration::from_millis(100),
    ).await {
        Ok(result) => println!("Completed: {}", result),
        Err(_) => println!("Timed out"),
    }
}
```

### Example: Cancellation-safe write

Implements atomic file writes that either complete fully or leave no partial data. The `sync_all()` call ensures all data is flushed to disk before returning success. This prevents data corruption if the operation is cancelled or the system crashes mid-write.

```rust
async fn cancellation_safe_write(data: String) -> Result<(), std::io::Error> {
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    let mut file = File::create("output.txt").await?;
    file.write_all(data.as_bytes()).await?;
    file.sync_all().await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    match cancellation_safe_write("Hello, World!".to_string()).await {
        Ok(_) => println!("File written successfully"),
        Err(e) => println!("Write failed: {}", e),
    }
}
```

**Concurrent Patterns**:
- **join!**: All complete, collect all results
- **try_join!**: All complete or fail fast
- **select!**: First to complete wins
- **FuturesUnordered**: Dynamic collection, unordered completion
- **join_all**: Dynamic collection, ordered results

---

## Pattern 2: Stream Processing

**Problem**: Processing infinite or unbounded sequences (websocket messages, sensor data, log streams) with standard iterators blocks thread. Collecting entire stream into Vec before processing wastes memory for large datasets.

**Solution**: Use `Stream` trait (async iterator) to yield values over time without blocking. Apply stream combinators: `.map()`, `.filter()`, `.fold()`, `.buffered()` for transformations.

**Why It Matters**: Streams enable processing data larger than memory—GB log file analyzed in constant memory. WebSocket connections handle millions of messages without collecting all.

**Use Cases**: WebSocket message processing, sensor data aggregation, log file streaming, database query result streaming, event sourcing, pub-sub systems, real-time analytics, infinite data sources.


### Example: Creating streams

Three primary ways to create async streams: from iterators using `stream::iter()` for known data, from channels enabling producer-consumer patterns with backpressure, and from intervals for time-based events. Each approach suits different use cases.

```rust
use std::time::Duration;
use futures::stream::{self, StreamExt};

async fn create_streams() {
    // From iterator - instant conversion of known data
    let _s = stream::iter(vec![1, 2, 3, 4, 5]);

    // From channel - producer task sends values over time
    let (tx, rx) = tokio::sync::mpsc::channel(10);
    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(i).await.unwrap();
        }
    });
    let _s = tokio_stream::wrappers::ReceiverStream::new(rx);

    // Interval stream - time-based events
    let _s = stream::StreamExt::take(
        tokio_stream::wrappers::IntervalStream::new(
            tokio::time::interval(Duration::from_millis(100))
        ),
        5,  // Stop after 5 ticks
    );

    println!("Streams created successfully");
}

#[tokio::main]
async fn main() {
    create_streams().await;
}
```

### Example: Map and filter

Stream combinators mirror iterator patterns: `filter()` retains elements matching a predicate, while `map()` transforms each element. Streams use lazy evaluation, processing elements only when consumed via `.collect()` or iterated with `.next()`.

```rust
use futures::stream::{self, StreamExt};

async fn transform_stream() {
    let stream = stream::iter(1..=10)
        .filter(|x| futures::future::ready(x % 2 == 0))  // Keep evens: 2, 4, 6, 8, 10
        .map(|x| x * 2);         // Double: 4, 8, 12, 16, 20

    let results: Vec<i32> = stream.collect().await;
    println!("Transformed: {:?}", results);  // [4, 8, 12, 16, 20]
}

#[tokio::main]
async fn main() {
    transform_stream().await;
}
```

### Example: Then (async map)

Use `.then()` when transformation requires async operations. The closure returns a Future that gets awaited for each element. Elements process sequentially by default; combine with `.buffer_unordered()` to enable concurrent processing of multiple elements.

```rust
use std::time::Duration;
use futures::stream::{self, StreamExt};

async fn async_transform_stream() {
    let stream = stream::iter(1..=5)
        .then(|x| async move {
            // Async operation per element
            tokio::time::sleep(Duration::from_millis(10)).await;
            x * x
        });

    let results: Vec<i32> = stream.collect().await;
    println!("Async transformed: {:?}", results);  // [1, 4, 9, 16, 25]
}

#[tokio::main]
async fn main() {
    async_transform_stream().await;
}
```

### Example: Fold and reduce

Aggregates an entire stream into a single accumulated value. The `fold(initial, closure)` method applies the closure to each element along with the current accumulator value, ultimately returning the final computed result after processing all elements.

```rust
use futures::stream::{self, StreamExt};

async fn aggregate_stream() {
    let sum = stream::iter(1..=100)
        .fold(0, |acc, x| futures::future::ready(acc + x))  // Sum: 0+1+2+...+100
        .await;

    println!("Sum: {}", sum);  // 5050
}

#[tokio::main]
async fn main() {
    aggregate_stream().await;
}
```

### Example: Take and skip

Pagination primitives for stream slicing: `skip(n)` discards the first n elements, while `take(n)` limits output to n elements then stops. Combine both for offset-based pagination using the pattern `skip(page * size).take(size)` for paged results.

```rust
use futures::stream::{self, StreamExt};

async fn limit_stream() {
    let results: Vec<i32> = stream::iter(1..=100)
        .skip(10)   // Skip first 10 (1-10)
        .take(5)    // Take next 5 (11-15)
        .collect()
        .await;

    println!("Limited: {:?}", results);  // [11, 12, 13, 14, 15]
}

#[tokio::main]
async fn main() {
    limit_stream().await;
}
```

### Example: Rate Limiting

Semaphore-based rate limiting controls concurrent operations by limiting available permits. Here, the semaphore allows maximum 5 concurrent executions. While `buffer_unordered(10)` permits 10 futures in flight, the semaphore restricts actual parallel execution to 5.

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

async fn rate_limited_requests(urls: Vec<String>) {
    let semaphore = Arc::new(Semaphore::new(5)); // Max 5 concurrent

    let stream = stream::iter(urls)
        .map(|url| {
            let permit = Arc::clone(&semaphore);
            async move {
                let _permit = permit.acquire().await.unwrap();  // Wait for permit
                println!("Fetching: {}", url);
                tokio::time::sleep(Duration::from_millis(100)).await;
                format!("Response from {}", url)
            }  // Permit released when dropped
        })
        .buffer_unordered(10);  // Allow 10 in-flight, but semaphore limits to 5

    let results: Vec<String> = stream.collect().await;
    println!("Fetched {} URLs", results.len());
}

#[tokio::main]
async fn main() {
    let urls: Vec<_> = (0..20).map(|i| format!("https://api.example.com/{}", i)).collect();
    rate_limited_requests(urls).await;
}
```

### Example: Batch processing

Processes items in fixed-size batches for controlled throughput. The `chunks()` method divides input into batches, each processed sequentially with a delay between batches. This approach is useful for rate limiting bulk operations or respecting API quotas.

```rust
async fn batch_process<T: std::fmt::Debug>(items: Vec<T>, batch_size: usize) {
    for (i, batch) in items.chunks(batch_size).enumerate() {
        println!("Processing batch {}: {:?}", i, batch);
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::main]
async fn main() {
    batch_process((0..25).collect::<Vec<_>>(), 10).await;
}
```

### Example: Stream merging

Combines two independent streams into a single merged stream, yielding elements interleaved as they become ready from either source. The order of output depends on which stream produces values first, enabling efficient multiplexing of multiple data sources.

```rust
use futures::stream;
use tokio_stream::StreamExt;

async fn merge_streams() {
    let stream1 = stream::iter(vec![1, 2, 3]);
    let stream2 = stream::iter(vec![4, 5, 6]);
    let merged = StreamExt::merge(stream1, stream2);
    let results: Vec<i32> = merged.collect().await;
    println!("Merged: {:?}", results);
}

#[tokio::main]
async fn main() {
    merge_streams().await;
}
```

**Stream Combinators**:
- **map/filter**: Synchronous transformation
- **then**: Async transformation
- **fold**: Aggregation
- **buffer_unordered**: Concurrent processing
- **merge**: Combine multiple streams

---

### Example: Stream from Async Generators

Manual `Stream` trait implementation requires `poll_next()` returning `Ready(Some(item))` to yield values, `Ready(None)` when exhausted, or `Pending` to signal waiting. While this provides full control, prefer channel-based patterns for most use cases.

```rust
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Stream;
use tokio_stream::StreamExt;

struct CounterStream {
    count: u32,
    max: u32,
}

impl CounterStream {
    fn new(max: u32) -> Self {
        Self { count: 0, max }
    }
}

impl Stream for CounterStream {
    type Item = u32;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.count < self.max {
            let current = self.count;
            self.count += 1;
            Poll::Ready(Some(current))  // Yield next value
        } else {
            Poll::Ready(None)  // Stream exhausted
        }
    }
}

#[tokio::main]
async fn main() {
    let mut stream = CounterStream::new(5);
    while let Some(n) = stream.next().await {
        println!("Count: {}", n);  // 0, 1, 2, 3, 4
    }
}
```

### Example: Async generator pattern using channels

Spawn a producer task that sends values through a bounded channel, then wrap the receiver as a stream. The channel automatically provides backpressure when the consumer processes slower than the producer generates, preventing memory overflow.

```rust
async fn number_generator(max: u32) -> impl Stream<Item = u32> {
    let (tx, rx) = tokio::sync::mpsc::channel(10);  // Buffer 10 items

    tokio::spawn(async move {
        for i in 0..max {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            if tx.send(i).await.is_err() {
                break;  // Consumer dropped, stop producing
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
}

#[tokio::main]
async fn main() {
    use tokio_stream::StreamExt;
    let mut stream = number_generator(5).await;
    while let Some(n) = stream.next().await {
        println!("Generated: {}", n);  // 0, 1, 2, 3, 4 (with delays)
    }
}
```

### Example: File watcher stream
```rust

// Real-world: 
use notify::{Watcher, RecursiveMode, Event};

async fn file_watcher_stream(path: String) -> impl Stream<Item = notify::Result<Event>> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    tokio::task::spawn_blocking(move || {
        let (notify_tx, notify_rx) = std::sync::mpsc::channel();

        let mut watcher = notify::recommended_watcher(notify_tx).unwrap();
        watcher.watch(path.as_ref(), RecursiveMode::Recursive).unwrap();

        for event in notify_rx {
            if tx.blocking_send(event).is_err() {
                break;
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
}
// Real-world: WebSocket message stream (simulation)
use std::time::Duration;

#[derive(Debug)]
enum WsMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping,
    Close,
}

async fn websocket_stream() -> impl Stream<Item = WsMessage> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    tokio::spawn(async move {
        let messages = vec![
            WsMessage::Text("Hello".to_string()),
            WsMessage::Text("World".to_string()),
            WsMessage::Ping,
            WsMessage::Binary(vec![1, 2, 3]),
            WsMessage::Close,
        ];

        for msg in messages {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
}

#[tokio::main]
async fn main() {
    use tokio_stream::StreamExt;
    let mut stream = websocket_stream().await;
    while let Some(msg) = stream.next().await {
        println!("Message: {:?}", msg);
    }
}
```

### Example: Database query result stream

Streams database rows incrementally without loading the entire result set into memory. Each row is fetched and yielded individually through the channel, enabling processing of large query results with constant memory usage regardless of total row count.

```rust
#[derive(Debug)]
struct Row { id: u64, data: String }

async fn database_query_stream(query: String) -> impl Stream<Item = Row> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    tokio::spawn(async move {
        for i in 0..10 {
            tokio::time::sleep(Duration::from_millis(10)).await;
            if tx.send(Row { id: i, data: format!("Data {}", i) }).await.is_err() { break; }
        }
    });
    tokio_stream::wrappers::ReceiverStream::new(rx)
}

#[tokio::main]
async fn main() {
    use tokio_stream::StreamExt;
    let mut stream = database_query_stream("SELECT * FROM users".to_string()).await;
    while let Some(row) = stream.next().await {
        println!("Row: {:?}", row);
    }
}
```

### Example: Interval-based stream

Creates a stream that emits sequential values at fixed time intervals. Each tick of the interval triggers emission of the next value through the channel. Useful for implementing periodic tasks, heartbeats, polling mechanisms, or time-series data generation.

```rust
async fn ticker_stream(interval: Duration, count: usize) -> impl Stream<Item = u64> {
    let (tx, rx) = tokio::sync::mpsc::channel(10);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(interval);
        for i in 0..count {
            interval.tick().await;
            if tx.send(i as u64).await.is_err() { break; }
        }
    });
    tokio_stream::wrappers::ReceiverStream::new(rx)
}

#[tokio::main]
async fn main() {
    use tokio_stream::StreamExt;
    let mut stream = ticker_stream(Duration::from_millis(100), 5).await;
    while let Some(tick) = stream.next().await {
        println!("Tick: {}", tick);
    }
}
```

**Stream Creation Patterns**:
- **Manual implementation**: Full control with `Stream` trait
- **Channel-based**: Producer task sends to channel
- **Interval**: Time-based events
- **External sources**: File system, WebSocket, database

---

## Pattern 3: Async/Await Patterns

**Problem**: Manual future polling with `.poll()` is complex and error-prone. Combinator chains (`.and_then().map()`) become unreadable for complex logic.

**Solution**: Use `async fn` and `.await` for sequential async code that reads like sync. Mark functions `async` to return `impl Future`.

**Why It Matters**: Async/await transforms async programming from callback spaghetti to readable imperative code. HTTP request handler with 5 operations: combinator chain is 20 lines of `.and_then()`, async/await is 10 lines reading like sync.

**Use Cases**: Web servers (async request handlers), database clients (async queries), HTTP clients (async requests), file I/O (async read/write), microservices (async RPC), chat servers, real-time systems.

### Example: Task Spawning and Structured Concurrency

Demonstrates spawning concurrent tasks, managing their complete lifecycle, and coordinating their completion. Structured concurrency ensures all spawned work completes before the parent scope exits, preventing orphaned tasks and enabling proper resource cleanup.

```rust
use tokio;
use std::time::Duration;

```

### Example: Basic task spawning

The `tokio::spawn()` function creates concurrent tasks and returns a `JoinHandle` for awaiting results. Tasks execute in parallel across the thread pool. Spawned tasks require `'static` lifetime, so use `Arc` for shared data or move ownership.

```rust
async fn spawn_basic_tasks() {
    let handle1 = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        println!("Task 1 complete");
        42
    });

    let handle2 = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(200)).await;
        println!("Task 2 complete");
        100
    });

    let (result1, result2) = tokio::join!(handle1, handle2);
    println!("Results: {:?}, {:?}", result1, result2);
}

#[tokio::main]
async fn main() {
    spawn_basic_tasks().await;
}
```

### Example: Structured concurrency with JoinSet

`JoinSet` manages dynamic collections of spawned tasks with automatic cleanup when dropped. The `join_next()` method returns results in completion order, processing the fastest tasks first. Any incomplete tasks are automatically cancelled when the JoinSet is dropped.

```rust

async fn structured_concurrency() {
    use tokio::task::JoinSet;

    let mut set = JoinSet::new();

    for i in 0..5 {
        set.spawn(async move {
            tokio::time::sleep(Duration::from_millis(i * 50)).await;
            println!("Task {} done", i);
            i
        });
    }

    // Wait for all tasks
    while let Some(result) = set.join_next().await {
        match result {
            Ok(value) => println!("Got: {}", value),
            Err(e) => println!("Task failed: {}", e),
        }
    }
}

#[tokio::main]
async fn main() {
    structured_concurrency().await;
}
```

### Example: Scoped tasks with JoinSet (guaranteed completion)

Guarantees all spawned work completes before the function returns. The `while let` loop drains all tasks from the JoinSet, collecting every result before proceeding. This pattern ensures no work is abandoned or left running unexpectedly.

```rust
use tokio::task::JoinSet;

async fn scoped_tasks_with_joinset() {
    let data = vec![1, 2, 3, 4, 5];
    let mut set = JoinSet::new();

    for item in data {
        set.spawn(async move {
            // Process item
            item * 2
        });
    }

    // Wait for all tasks - guaranteed to complete before we continue
    let mut results = Vec::new();
    while let Some(result) = set.join_next().await {
        results.push(result.unwrap());
    }

    println!("Results: {:?}", results);
}

#[tokio::main]
async fn main() {
    scoped_tasks_with_joinset().await;
}
```

### Example: Task cancellation

`CancellationToken` enables cooperative task cancellation across async boundaries. Tasks monitor cancellation using `select!` with `token.cancelled()`. The `child_token()` method creates hierarchical cancellation, allowing parent cancellation to propagate to all child tasks automatically.

```rust

use tokio_util::sync::CancellationToken;

async fn cancellable_task() {
    let token = CancellationToken::new();
    let child_token = token.child_token();

    let task = tokio::spawn(async move {
        tokio::select! {
            _ = child_token.cancelled() => {
                println!("Task cancelled");
            }
            _ = tokio::time::sleep(Duration::from_secs(10)) => {
                println!("Task completed normally");
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;
    token.cancel();

    task.await.unwrap();
}

#[tokio::main]
async fn main() {
    cancellable_task().await;
}
```
### Example: Worker pool pattern

Fixed-size worker pool processing tasks from a shared queue via `Arc<Mutex<Receiver>>`. Workers loop until the sender is dropped, signaling shutdown. This pattern bounds concurrency to a fixed number of workers while providing natural backpressure through the channel.

```rust
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

type Task = Box<dyn FnOnce() + Send + 'static>;

struct WorkerPool {
    sender: mpsc::Sender<Task>,
}

impl WorkerPool {
    fn new(num_workers: usize) -> Self {
        let (tx, rx) = mpsc::channel::<Task>(100);
        let rx = Arc::new(tokio::sync::Mutex::new(rx));

        for i in 0..num_workers {
            let rx = Arc::clone(&rx);
            tokio::spawn(async move {
                loop {
                    let task = {
                        let mut guard = rx.lock().await;
                        guard.recv().await
                    };
                    match task {
                        Some(task) => {
                            println!("Worker {} executing task", i);
                            task();
                        }
                        None => break, // Channel closed
                    }
                }
            });
        }

        Self { sender: tx }
    }

    async fn submit<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(task)).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    let pool = WorkerPool::new(4);
    for i in 0..10 {
        pool.submit(move || println!("Task {} executed", i)).await;
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```
### Example: Supervisor pattern (restart on failure)

Automatically restarts failed tasks up to N times with configurable delays between attempts. Inspired by Erlang supervisor trees, this pattern provides fault tolerance for critical background services that need automatic recovery from transient failures.

```rust
async fn supervised_task<F, Fut>(
    mut task_fn: F,
    max_restarts: usize,
) where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    for attempt in 0..=max_restarts {
        let handle = tokio::spawn(task_fn());

        match handle.await {
            Ok(_) => {
                println!("Task completed successfully");
                break;
            }
            Err(e) => {
                if attempt < max_restarts {
                    println!("Task failed (attempt {}): {}. Restarting...", attempt + 1, e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                } else {
                    println!("Task failed after {} attempts", max_restarts + 1);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    supervised_task(|| async { println!("Running task"); }, 3).await;
}
```
### Example: Background task with graceful shutdown

Uses a `watch` channel to broadcast shutdown signals to multiple receivers. The service loops with `select!` monitoring both work and shutdown channels. When shutdown arrives, the current iteration completes gracefully before the service stops cleanly.

```rust
async fn background_service(shutdown: tokio::sync::watch::Receiver<bool>) {
    let mut shutdown = shutdown;
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                println!("Background service tick");
            }
            _ = shutdown.changed() => {
                println!("Shutdown signal received");
                break;
            }
        }
    }

    println!("Background service stopped");
}

async fn run_with_graceful_shutdown() {
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    let service = tokio::spawn(background_service(shutdown_rx));

    tokio::time::sleep(Duration::from_secs(3)).await;

    println!("Sending shutdown signal");
    shutdown_tx.send(true).unwrap();

    service.await.unwrap();
}

#[tokio::main]
async fn main() {
    run_with_graceful_shutdown().await;
}
```


**Task Management Patterns**:
- **spawn**: Create independent task
- **JoinSet**: Manage dynamic set of tasks
- **Cancellation**: Cooperative cancellation with tokens
- **Supervisor**: Auto-restart on failure
- **Graceful shutdown**: Clean task termination

---

### Example: Result propagation with ?

The `?` operator propagates errors up the async call stack, returning early when encountering an `Err` value. This creates clean, linear error handling code without deeply nested match expressions, making async error flows read like synchronous code.

```rust

async fn fetch_user_data(user_id: u64) -> Result<String, String> {
    if user_id == 0 {
        return Err("Invalid ID".to_string());
    }
    Ok(format!("User {}", user_id))
}

async fn get_user_profile(user_id: u64) -> Result<String, String> {
    let data = fetch_user_data(user_id).await?;
    let profile = format!("Profile: {}", data);
    Ok(profile)
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let profile = get_user_profile(42).await?;
    println!("{}", profile);
    Ok(())
}
```

### Example: Retry with exponential backoff

Retries failed operations with exponentially increasing delays between attempts. Starting from an initial delay, each subsequent failure doubles the wait time until reaching maximum retries. This pattern prevents overwhelming failing services while allowing recovery time.

```rust

async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    max_retries: u32,
    initial_delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut delay = initial_delay;

    for attempt in 0..max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries - 1 => {
                println!("Attempt {} failed: {}. Retrying in {:?}...", attempt + 1, e, delay);
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }

    unreachable!()
}

#[tokio::main]
async fn main() {
    let result = retry_with_backoff(
        || async { Ok::<_, &str>("Success!") },
        3,
        Duration::from_millis(100),
    ).await;
    println!("Result: {:?}", result);
}
```

### Example: Circuit breaker

Implements three states: Closed for normal operation, Open to fail fast and reject requests, and HalfOpen to test recovery. The circuit opens after reaching a failure threshold, protecting downstream services. Essential for resilient microservice architectures.

```rust

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Copy, Debug)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_threshold: usize,
    failures: Arc<Mutex<usize>>,
    success_threshold: usize,
    timeout: Duration,
}

impl CircuitBreaker {
    fn new(failure_threshold: usize, success_threshold: usize, timeout: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_threshold,
            failures: Arc::new(Mutex::new(0)),
            success_threshold,
            timeout,
        }
    }

    async fn call<F, Fut, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: From<String>,
    {
        let state = *self.state.lock().await;

        match state {
            CircuitState::Open => {
                return Err(E::from("Circuit breaker is open".to_string()));
            }
            CircuitState::HalfOpen => {
                // Try to recover
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }

        match operation().await {
            Ok(result) => {
                // Reset failures
                *self.failures.lock().await = 0;
                if matches!(state, CircuitState::HalfOpen) {
                    *self.state.lock().await = CircuitState::Closed;
                }
                Ok(result)
            }
            Err(e) => {
                let mut failures = self.failures.lock().await;
                *failures += 1;

                if *failures >= self.failure_threshold {
                    println!("Circuit breaker opened due to {} failures", failures);
                    *self.state.lock().await = CircuitState::Open;

                    // Schedule transition to half-open
                    let state = Arc::clone(&self.state);
                    let timeout = self.timeout;
                    tokio::spawn(async move {
                        tokio::time::sleep(timeout).await;
                        *state.lock().await = CircuitState::HalfOpen;
                        println!("Circuit breaker transitioned to half-open");
                    });
                }

                Err(e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let breaker = CircuitBreaker::new(3, 2, Duration::from_secs(2));
    let result: Result<String, String> = breaker.call(|| async {
        Ok("Success".to_string())
    }).await;
    println!("Result: {:?}", result);
}
```

### Example: Fallback pattern

Returns a default value when the primary operation fails, enabling graceful degradation of service quality rather than complete failure. Ideal for non-critical data paths like cached content, default settings, or supplementary information that enhances but isn't essential.

```rust
async fn fetch_with_fallback<F, Fut, T>(
    primary: F,
    fallback_value: T,
) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error>>>,
{
    match primary().await {
        Ok(value) => value,
        Err(e) => {
            println!("Primary failed: {}. Using fallback.", e);
            fallback_value
        }
    }
}
```

### Example: Bulkhead pattern (resource isolation)

Limits concurrent access to a resource using a semaphore for isolation. The `try_acquire()` method fails fast when no permits are available, rejecting excess requests immediately. This prevents one component from consuming all shared resources.

```rust
struct Bulkhead {
    semaphore: Arc<tokio::sync::Semaphore>,
}

impl Bulkhead {
    fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(tokio::sync::Semaphore::new(max_concurrent)),
        }
    }

    async fn execute<F, Fut, T>(&self, operation: F) -> Result<T, String>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        match self.semaphore.try_acquire() {
            Ok(permit) => {
                let result = operation().await;
                drop(permit);
                Ok(result)
            }
            Err(_) => Err("Bulkhead full - request rejected".to_string()),
        }
    }
}

#[tokio::main]
async fn main() {
    // Fallback pattern example
    let result = fetch_with_fallback(
        || async { Ok::<_, Box<dyn std::error::Error>>("Primary data".to_string()) },
        "Fallback data".to_string(),
    ).await;
    println!("Result: {}", result);

    // Bulkhead pattern example
    let bulkhead = Bulkhead::new(2);
    let result = bulkhead.execute(|| async { 42 }).await;
    println!("Bulkhead result: {:?}", result);
}
```

**Error Handling Patterns**:
- **Retry**: Exponential backoff
- **Circuit breaker**: Prevent cascading failures
- **Fallback**: Default value on error
- **Bulkhead**: Limit concurrent requests



## Pattern 4: Select and Timeout Patterns

**Problem**: Waiting indefinitely for async operations causes hangs—network request that never responds blocks forever. Need to handle whichever of multiple operations completes first (user input vs network response).

**Solution**: Use `tokio::select!` to race multiple futures, completing when first finishes. Use `tokio::time::timeout()` to bound operation duration.

**Why It Matters**: Timeouts prevent resource leaks from hung operations—HTTP server without timeouts accumulates connections from slow clients until memory exhausted. Select enables responsive UIs: user input cancels background computation immediately.

**Use Cases**: HTTP clients (request timeouts), connection management (idle timeouts), health checks (periodic pings), graceful shutdown (timeout on cleanup), rate limiting (interval-based), user cancellation (input vs background work), circuit breakers.

### Example: Select Patterns

The `select!` macro races multiple futures concurrently, executing whichever branch completes first while cancelling the others. Branches are checked in random order for fairness by default. Use this pattern for multiplexing events from different sources.

```rust
use std::time::Duration;
use tokio::sync::mpsc;

async fn select_two_channels() {
    let (tx1, mut rx1) = mpsc::channel::<i32>(10);
    let (tx2, mut rx2) = mpsc::channel::<String>(10);

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        tx1.send(42).await.unwrap();
    });

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        tx2.send("Hello".to_string()).await.unwrap();
    });

    tokio::select! {
        Some(num) = rx1.recv() => {
            println!("Got number: {}", num);
        }
        Some(msg) = rx2.recv() => {
            println!("Got message: {}", msg);
        }
    }
}

#[tokio::main]
async fn main() {
    select_two_channels().await;
}
```

### Example: Select in a loop

Implements an event loop using `select!` over multiple sources until all are exhausted. Guard conditions like `if !done` disable branches after their channels close. The `else` branch fires when all guarded branches become disabled, signaling loop termination.

```rust
use std::time::Duration;
use tokio::sync::mpsc;

async fn select_loop() {
    let (tx1, mut rx1) = mpsc::channel::<i32>(10);
    let (tx2, mut rx2) = mpsc::channel::<String>(10);

    // Spawn producers
    tokio::spawn(async move {
        for i in 0..5 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            tx1.send(i).await.unwrap();
        }
    });

    tokio::spawn(async move {
        for i in 0..3 {
            tokio::time::sleep(Duration::from_millis(150)).await;
            tx2.send(format!("msg_{}", i)).await.unwrap();
        }
    });

    let mut done1 = false;
    let mut done2 = false;

    loop {
        tokio::select! {
            result = rx1.recv(), if !done1 => {
                match result {
                    Some(num) => println!("Number: {}", num),
                    None => done1 = true,
                }
            }
            result = rx2.recv(), if !done2 => {
                match result {
                    Some(msg) => println!("Message: {}", msg),
                    None => done2 = true,
                }
            }
            else => {
                println!("Both channels closed");
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    select_loop().await;
}
```

### Example: Biased select (priority)

Adding `biased;` forces `select!` to check branches in declaration order rather than randomly. The first ready branch always wins, enabling priority-based handling. Use this for priority queues where certain event sources should take precedence over others.

```rust
use std::time::Duration;
use tokio::sync::mpsc;

async fn biased_select() {
    let (tx_high, mut rx_high) = mpsc::channel::<String>(10);
    let (tx_low, mut rx_low) = mpsc::channel::<String>(10);

    tokio::spawn(async move {
        tx_high.send("High priority".to_string()).await.unwrap();
        tx_low.send("Low priority".to_string()).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Biased: always checks branches in order
    tokio::select! {
        biased;

        Some(msg) = rx_high.recv() => {
            println!("High: {}", msg);
        }
        Some(msg) = rx_low.recv() => {
            println!("Low: {}", msg);
        }
    }
}

#[tokio::main]
async fn main() {
    biased_select().await;
}
```
### Example: Request with cancellation

Races an ongoing request against a cancellation signal using `select!`. If the cancellation arrives first, the request future is dropped and cancelled. For spawned tasks requiring true cooperative cancellation, use `CancellationToken` to signal tasks to stop gracefully.

```rust
use std::time::Duration;
use tokio::sync::mpsc;

async fn request_with_cancel() {
    let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);

    let request = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).await;
        "Request complete"
    });

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        cancel_tx.send(()).await.unwrap();
    });

    tokio::select! {
        result = request => {
            println!("Request finished: {:?}", result);
        }
        _ = cancel_rx.recv() => {
            println!("Request cancelled");
        }
    }
}

#[tokio::main]
async fn main() {
    request_with_cancel().await;
}
```
### Example: Server with shutdown signal

Server event loop using `select!` to multiplex between incoming requests and a shutdown signal channel. When shutdown is received, the loop breaks and exits. Currently in-flight requests complete processing while new incoming requests are rejected.

```rust
use std::time::Duration;
use tokio::sync::mpsc;

async fn server_with_shutdown() {
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    let (request_tx, mut request_rx) = mpsc::channel::<String>(10);

    // Simulate incoming requests
    let request_tx_clone = request_tx.clone();
    tokio::spawn(async move {
        for i in 0..10 {
            tokio::time::sleep(Duration::from_millis(200)).await;
            if request_tx_clone.send(format!("Request {}", i)).await.is_err() {
                break;
            }
        }
    });

    // Simulate shutdown after 1 second
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        shutdown_tx.send(()).await.unwrap();
    });

    // Server loop
    loop {
        tokio::select! {
            Some(req) = request_rx.recv() => {
                println!("Processing: {}", req);
            }
            _ = shutdown_rx.recv() => {
                println!("Shutdown signal received");
                break;
            }
        }
    }

    println!("Server stopped");
}
```

### Example: Select with default (non-blocking)

The `else` branch in `select!` fires when all other branches cannot make progress. This enables non-blocking polling: check if data is immediately available and return it, otherwise continue execution immediately without waiting for any future.

```rust
use std::time::Duration;
use tokio::sync::mpsc;

async fn select_with_default() {
    let (tx, mut rx) = mpsc::channel::<i32>(10);

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        tx.send(42).await.unwrap();
    });

    // Try to receive immediately
    tokio::select! {
        Some(value) = rx.recv() => {
            println!("Got value: {}", value);
        }
        else => {
            println!("No value available immediately");
        }
    }

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Try again after delay
    tokio::select! {
        Some(value) = rx.recv() => {
            println!("Got value: {}", value);
        }
        else => {
            println!("No value available");
        }
    }
}

#[tokio::main]
async fn main() {
    select_with_default().await;
}
```
---

### Example: Basic timeout

The `timeout(duration, future)` function wraps any future with a time limit. Returns `Ok(result)` if the operation completes within the duration, or `Err(Elapsed)` when time expires. The wrapped future is automatically cancelled when timeout occurs.

```rust
use std::time::Duration;
use tokio::time::{sleep, timeout};

async fn basic_timeout() {
    let operation = async {
        sleep(Duration::from_secs(2)).await;
        "Completed"
    };

    match timeout(Duration::from_secs(1), operation).await {
        Ok(result) => println!("Result: {}", result),
        Err(_) => println!("Operation timed out"),
    }
}

#[tokio::main]
async fn main() {
    basic_timeout().await;
}
```

### Example: Timeout with retry

Combines timeout and retry patterns where each attempt receives a fresh timeout. Handles three outcomes: success, failure from operation error, and timeout from no response. Treating timeout as a retriable error handles both slow and failing services.

```rust
use std::time::Duration;
use tokio::time::{sleep, timeout};

async fn timeout_with_retry() {
    for attempt in 1..=3u64 {
        let operation = async {
            sleep(Duration::from_millis(attempt * 400)).await;
            if attempt < 3 {
                Err("Failed")
            } else {
                Ok("Success")
            }
        };

        match timeout(Duration::from_secs(1), operation).await {
            Ok(Ok(result)) => {
                println!("Success: {}", result);
                break;
            }
            Ok(Err(e)) => {
                println!("Attempt {} failed: {}", attempt, e);
            }
            Err(_) => {
                println!("Attempt {} timed out", attempt);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    timeout_with_retry().await;
}
```

### Example: Deadline tracking

Deadlines specify absolute completion times rather than relative durations. Convert deadline to remaining duration with `deadline.saturating_duration_since(Instant::now())`. Deadlines work better for multi-step operations where all steps must complete within a shared time budget.

```rust
use std::time::Duration;
use tokio::time::{sleep, timeout, Instant};

async fn with_deadline<F, T>(
    future: F,
    deadline: Instant,
) -> Result<T, &'static str>
where
    F: std::future::Future<Output = T>,
{
    let duration = deadline.saturating_duration_since(Instant::now());

    match timeout(duration, future).await {
        Ok(result) => Ok(result),
        Err(_) => Err("Deadline exceeded"),
    }
}

async fn deadline_example() {
    let deadline = Instant::now() + Duration::from_secs(1);

    let result = with_deadline(
        async {
            sleep(Duration::from_millis(500)).await;
            42
        },
        deadline,
    ).await;

    println!("Result: {:?}", result);
}

#[tokio::main]
async fn main() {
    deadline_example().await;
}
```

### Example: Timeout for multiple operations

Applies a single timeout encompassing an entire batch of operations. All operations must complete within the total allocated time. If any individual operation takes too long, the entire batch fails with a timeout error.

```rust
use std::time::Duration;
use tokio::time::{sleep, timeout};

async fn timeout_all() {
    let operations = vec![
        tokio::spawn(async {
            sleep(Duration::from_millis(100)).await;
            1
        }),
        tokio::spawn(async {
            sleep(Duration::from_millis(200)).await;
            2
        }),
        tokio::spawn(async {
            sleep(Duration::from_millis(300)).await;
            3
        }),
    ];

    let all_done = async {
        let mut results = Vec::new();
        for handle in operations {
            results.push(handle.await.unwrap());
        }
        results
    };

    match timeout(Duration::from_millis(250), all_done).await {
        Ok(results) => println!("All done: {:?}", results),
        Err(_) => println!("Not all operations completed in time"),
    }
}

#[tokio::main]
async fn main() {
    timeout_all().await;
}
```

### Example: Rate limiter with timeout

Implements token bucket rate limiting using a semaphore with periodic refill. The `acquire_with_timeout` method fails fast if no tokens become available within the time limit, preventing indefinite waits. Essential for enforcing API rate limits.

```rust
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::timeout;

struct RateLimiter {
    semaphore: Arc<Semaphore>,
    #[allow(dead_code)]
    refill_amount: usize,
    #[allow(dead_code)]
    refill_interval: Duration,
}

impl RateLimiter {
    fn new(capacity: usize, refill_amount: usize, refill_interval: Duration) -> Self {
        let semaphore = Arc::new(Semaphore::new(capacity));

        // Refill task
        let sem = Arc::clone(&semaphore);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(refill_interval);
            loop {
                interval.tick().await;
                sem.add_permits(refill_amount);
            }
        });

        Self {
            semaphore,
            refill_amount,
            refill_interval,
        }
    }

    async fn acquire_with_timeout(&self, timeout_duration: Duration) -> Result<(), &'static str> {
        match timeout(timeout_duration, self.semaphore.acquire()).await {
            Ok(Ok(permit)) => {
                permit.forget(); // Consume permit
                Ok(())
            }
            Ok(Err(_)) => Err("Semaphore closed"),
            Err(_) => Err("Timeout acquiring rate limit"),
        }
    }
}

#[tokio::main]
async fn main() {
    let limiter = RateLimiter::new(5, 1, Duration::from_secs(1));
    match limiter.acquire_with_timeout(Duration::from_millis(100)).await {
        Ok(()) => println!("Acquired rate limit token"),
        Err(e) => println!("Failed: {}", e),
    }
}
```

### Example: Health check with timeout

Performs HTTP health checks with timeout to detect unresponsive services quickly. Returns `Ok(true)` for healthy services responding with success status, `Ok(false)` for unhealthy responses, and `Err` for timeout or network failures.

```rust
use std::time::Duration;
use tokio::time::timeout;

async fn health_check(url: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let check = async {
        let response = reqwest::get(url).await?;
        Ok::<bool, Box<dyn std::error::Error + Send + Sync>>(response.status().is_success())
    };

    timeout(Duration::from_secs(5), check)
        .await
        .map_err(|_| -> Box<dyn std::error::Error + Send + Sync> { "Health check timed out".into() })?
}

#[tokio::main]
async fn main() {
    match health_check("https://example.com").await {
        Ok(healthy) => println!("Service healthy: {}", healthy),
        Err(e) => println!("Health check failed: {}", e),
    }
}
```

### Example: Graceful timeout (finish current work)

Waits for all workers to complete with an upper time bound. Workers receive a grace period to finish their current work, then the system proceeds regardless of completion status. This balances clean shutdown requirements against service availability.

```rust
use std::time::Duration;
use tokio::time::timeout;

async fn graceful_shutdown_with_timeout(
    workers: Vec<tokio::task::JoinHandle<()>>,
    grace_period: Duration,
) {
    let shutdown = async {
        for worker in workers {
            worker.await.ok();
        }
    };

    match timeout(grace_period, shutdown).await {
        Ok(_) => println!("All workers stopped gracefully"),
        Err(_) => println!("Timeout - forcing shutdown"),
    }
}

#[tokio::main]
async fn main() {
    let workers = vec![
        tokio::spawn(async { tokio::time::sleep(Duration::from_millis(100)).await }),
        tokio::spawn(async { tokio::time::sleep(Duration::from_millis(200)).await }),
    ];
    graceful_shutdown_with_timeout(workers, Duration::from_secs(1)).await;
}
```
---

## Pattern 5: Runtime Comparison

**Problem**: Choosing wrong async runtime impacts performance, features, and maintainability. Tokio dominates ecosystem but isn't always best choice.

**Solution**: Use Tokio for general-purpose applications: mature, full-featured, excellent ecosystem. Use async-std for simpler API, closer to std library patterns.

**Why It Matters**: Runtime choice determines ecosystem access—Tokio has 10x more compatible libraries than alternatives. Performance varies: work-stealing vs single-threaded, epoll vs io_uring.

**Use Cases**: Tokio for web servers, databases, general applications. async-std for learning, simpler projects. smol for single-threaded, minimal overhead. embassy for embedded systems, bare-metal. Runtime-agnostic libraries for maximum compatibility.


### Example: Multi-threaded runtime (default)

The `#[tokio::main]` attribute creates a multi-threaded runtime with worker threads equal to CPU core count by default. Work-stealing scheduling automatically distributes tasks across threads, ensuring efficient load balancing and maximum CPU utilization.

```rust
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("Running on multi-threaded runtime");

    let handles: Vec<_> = (0..10)
        .map(|i| {
            tokio::spawn(async move {
                println!("Task {} on thread {:?}", i, std::thread::current().id());
                tokio::time::sleep(Duration::from_millis(10)).await;
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }
}
```

### Example: Single-threaded runtime

Using `flavor = "current_thread"` runs all tasks on the main thread only. This removes the `Send` requirement for spawned futures. Ideal for CLI tools, WebAssembly targets, or when using `!Send` types like `Rc` and `RefCell`.

```rust

#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("Running on single-threaded runtime");

    let thread_id = std::thread::current().id();

    for i in 0..5 {
        tokio::spawn(async move {
            println!("Task {} on thread {:?}", i, std::thread::current().id());
        }).await.unwrap();
    }

    println!("All tasks ran on thread {:?}", thread_id);
}
```

### Example: Custom runtime configuration

Build the runtime manually using `Builder` for fine-grained control over configuration. Customize worker thread count, thread names, and stack sizes. Use `block_on()` to execute async code on the custom runtime from synchronous contexts.

```rust
use std::time::Duration;

fn custom_runtime_example() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("my-worker")
        .thread_stack_size(3 * 1024 * 1024)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        println!("Running on custom runtime");

        for i in 0..4 {
            tokio::spawn(async move {
                println!("Task {} started", i);
                tokio::time::sleep(Duration::from_millis(100)).await;
            });
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    });
}

fn main() {
    custom_runtime_example();
}
```

### Example: Blocking operations

Never block the async runtime directly with synchronous operations. Use `spawn_blocking()` to run blocking code on a dedicated thread pool separate from async workers. Essential for synchronous I/O, CPU-intensive work, and blocking FFI calls.

```rust
use std::time::Duration;

async fn handle_blocking_operations() {
    // Bad: blocks the async runtime
    // std::thread::sleep(Duration::from_secs(1));

    // Good: run blocking code on dedicated thread pool
    let result = tokio::task::spawn_blocking(|| {
        std::thread::sleep(Duration::from_secs(1));
        "Blocking operation complete"
    }).await.unwrap();

    println!("{}", result);
}

#[tokio::main]
async fn main() {
    handle_blocking_operations().await;
}
```

### Example: Local task set (for !Send futures)

`LocalSet` enables running `!Send` futures that use types like `Rc` and `RefCell`. Tasks spawned via `spawn_local()` are guaranteed to stay on the current thread. Use this for WebAssembly or when interfacing with non-threadsafe libraries.

```rust
use tokio::task::LocalSet;

async fn local_task_set_example() {
    use std::rc::Rc;

    let local = LocalSet::new();

    let nonsend_data = Rc::new(42);

    local.run_until(async move {
        let data = Rc::clone(&nonsend_data);

        tokio::task::spawn_local(async move {
            println!("Local task with Rc: {}", data);
        }).await.unwrap();
    }).await;
}

#[tokio::main]
async fn main() {
    local_task_set_example().await;
}
```

### Example: CPU-bound work with rayon

Combine Tokio for async I/O operations with Rayon for parallel CPU-intensive computation. Wrap Rayon parallel operations inside `spawn_blocking()` to execute them on the blocking thread pool, preventing them from blocking the async runtime.

```rust
use rayon::prelude::*;

async fn cpu_bound_with_rayon() {
    let numbers: Vec<u64> = (0..1_000_000).collect();

    let sum = tokio::task::spawn_blocking(move || {
        numbers.par_iter().sum::<u64>()
    }).await.unwrap();

    println!("Sum: {}", sum);
}

#[tokio::main]
async fn main() {
    cpu_bound_with_rayon().await;
}
```

### Example: Mixed workload (I/O and CPU)

Run I/O-bound and CPU-bound tasks concurrently using appropriate primitives. I/O tasks use async sleep on the runtime, while CPU tasks use `spawn_blocking` for the blocking pool. The `join!` macro waits for both without blocking.

```rust
use std::time::Duration;

async fn mixed_workload() {
    let io_task = tokio::spawn(async {
        for i in 0..5 {
            println!("I/O task {}", i);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    let cpu_task = tokio::task::spawn_blocking(|| {
        for i in 0..5 {
            println!("CPU task {}", i);
            std::thread::sleep(Duration::from_millis(100));

            // Simulate CPU-intensive work
            let _ = (0..1_000_000).sum::<u64>();
        }
    });

    let _ = tokio::join!(io_task, cpu_task);
}

#[tokio::main]
async fn main() {
    mixed_workload().await;
}
```
---

### Example: Runtime Comparison and Interop

Tokio and async-std provide similar APIs for common async operations. Tokio has the larger ecosystem and more third-party library support, while async-std mirrors standard library patterns. Feature flags enable compiling the same code for both runtimes.

```rust
// Tokio version
#[cfg(feature = "tokio-runtime")]
mod tokio_example {
    use tokio;
    use std::time::Duration;

    #[tokio::main]
    pub async fn run() {
        println!("=== Tokio Runtime ===");

        let handles: Vec<_> = (0..5)
            .map(|i| {
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    i * 2
                })
            })
            .collect();

        for handle in handles {
            let result = handle.await.unwrap();
            println!("Result: {}", result);
        }
    }
}
```

### Example: async-std version

Implements the same logic using async-std APIs instead of Tokio. The JoinHandle returns the value directly without needing `.unwrap()`. The API mirrors standard library naming conventions with `task::spawn` and `task::sleep`.

```rust
#[cfg(feature = "async-std-runtime")]
mod async_std_example {
    use async_std;
    use std::time::Duration;

    #[async_std::main]
    pub async fn run() {
        println!("=== async-std Runtime ===");

        let handles: Vec<_> = (0..5)
            .map(|i| {
                async_std::task::spawn(async move {
                    async_std::task::sleep(Duration::from_millis(100)).await;
                    i * 2
                })
            })
            .collect();

        for handle in handles {
            let result = handle.await;
            println!("Result: {}", result);
        }
    }
}
```

### Example: Runtime-agnostic code using futures

Write portable library code that works with any async runtime using the `futures` crate. Use the generic `Future` trait and combinators instead of runtime-specific APIs like Tokio's spawn. Essential for creating reusable async libraries.

```rust
mod runtime_agnostic {
    use futures::future::{join_all, FutureExt};
    use std::future::Future;
    use std::pin::Pin;

    pub async fn process_items<F, Fut>(
        items: Vec<i32>,
        process: F,
    ) -> Vec<i32>
    where
        F: Fn(i32) -> Fut,
        Fut: Future<Output = i32>,
    {
        let futures: Vec<_> = items.into_iter().map(process).collect();
        join_all(futures).await
    }
}

#[tokio::main]
async fn main() {
    let results = runtime_agnostic::process_items(
        vec![1, 2, 3, 4, 5],
        |x| async move { x * 2 },
    ).await;
    println!("Results: {:?}", results);
}
```

### Feature comparison
 Tokio vs async-std:
 
 Tokio:
 - Work-stealing scheduler (better for CPU-intensive tasks)
 - More configuration options
 - Larger ecosystem (widely used)
 - spawn_blocking for blocking operations
 - Good for web servers, databases
 
 async-std:
 - Simpler API (mirrors std library)
 - Easier to learn
 - Good for general-purpose async
 - Less configuration needed
 - Good for CLI tools, simpler services


             
### Example: Performance comparison

Benchmarks spawn and completion overhead by creating 1000 concurrent tasks with minimal work. Results vary significantly based on workload characteristics and hardware. Always benchmark with your specific use case rather than relying on generic measurements.

```rust
#[cfg(feature = "tokio-runtime")]
async fn tokio_performance_test() {
    use tokio::time::{Instant, Duration};

    let start = Instant::now();

    let handles: Vec<_> = (0..1000)
        .map(|_| {
            tokio::spawn(async {
                tokio::time::sleep(Duration::from_micros(1)).await;
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }

    println!("Tokio: 1000 tasks in {:?}", start.elapsed());
}

#[cfg(feature = "async-std-runtime")]
async fn async_std_performance_test() {
    use async_std::task;
    use std::time::Instant;
    use std::time::Duration;

    let start = Instant::now();

    let handles: Vec<_> = (0..1000)
        .map(|_| {
            task::spawn(async {
                task::sleep(Duration::from_micros(1)).await;
            })
        })
        .collect();

    for handle in handles {
        handle.await;
    }

    println!("async-std: 1000 tasks in {:?}", start.elapsed());
}

#[cfg(feature = "tokio-runtime")]
#[tokio::main]
async fn main() {
    tokio_performance_test().await;
}

#[cfg(feature = "async-std-runtime")]
#[async_std::main]
async fn main() {
    async_std_performance_test().await;
}
```

### Example: using futures crate for compatibility

The `futures::executor::block_on` function runs futures without requiring a full async runtime. Useful for tests, simple scripts, and synchronous contexts. Combinators from the futures crate work with any runtime, enabling portable async code.

```rust
use futures::executor::block_on;
use futures::future::join;

async fn runtime_independent_function() -> i32 {
    42
}

fn interop_example() {
    // Can run with any executor
    let result = block_on(async {
        let (a, b) = join(
            runtime_independent_function(),
            runtime_independent_function(),
        ).await;
        a + b
    });

    println!("Interop result: {}", result);
}

fn main() {
    interop_example();
}
```

 ## Choosing a Runtime:
 
 Use Tokio when:
 - Building high-performance web servers
 - Need fine-grained control over runtime
 - Working with Tokio ecosystem (tonic, axum, etc.)
 - CPU-bound tasks mixed with I/O
 
 Use async-std when:
 - Building CLI tools or simpler services
 - Want std-like API familiarity
 - Primarily I/O-bound workload
 - Simpler application with less configuration
 
 Use runtime-agnostic futures when:
 - Writing libraries
 - Need portability
 - Want to avoid runtime lock-in




**Runtime Comparison**:

| Feature | Tokio | async-std |
|---------|-------|-----------|
| Scheduler | Work-stealing | Work-stealing |
| API Style | Tokio-specific | std-like |
| Ecosystem | Large | Moderate |
| Configuration | Extensive | Minimal |
| Learning Curve | Moderate | Gentle |
| Best For | Web servers, databases | CLI tools, simpler apps |

---

### Summary

This chapter covered async runtime patterns in Rust:

1. **Future Composition**: Combinators, concurrent execution (join/select), error handling
2. **Stream Processing**: Combinators, async generators, rate limiting, batching
3. **Async/Await**: Task spawning, structured concurrency, cancellation, error recovery
4. **Select/Timeout**: Racing futures, deadlines, graceful shutdown, rate limiting
5. **Runtime Comparison**: Tokio vs async-std, features, performance, interoperability

**Key Takeaways**:
- **async/await** provides ergonomic async programming
- **Streams** process sequences of async values
- **select!** enables event-driven programming
- **Timeout** prevents indefinite blocking
- **Tokio** for high-performance servers, **async-std** for simpler apps
- Use **spawn_blocking** for CPU-bound work
- **Structured concurrency** with JoinSet ensures cleanup

**Performance Guidelines**:
- Prefer async for I/O-bound tasks
- Use spawn_blocking for CPU-bound work
- Limit concurrent tasks to avoid overwhelming resources
- Use streams for backpressure
- Benchmark runtime choice for your workload

**Common Patterns**:
- **Circuit breaker**: Prevent cascading failures
- **Retry with backoff**: Handle transient errors
- **Rate limiting**: Control resource usage
- **Graceful shutdown**: Clean termination
- **Request-response**: Structured communication

**Safety**:
- Send/Sync enforce thread safety
- Cancellation is cooperative
- No data races (enforced by type system)
- Borrow checker prevents use-after-free
