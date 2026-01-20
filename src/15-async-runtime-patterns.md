# Async Runtime Patterns
This chapter explores asynchronous programming patterns in Rust using async/await and async runtimes. We'll cover future composition, stream processing, concurrency patterns, timeout handling, and runtime comparisons through practical, production-ready examples.



## Pattern 1: Future Composition

**Problem**: Chaining async operations with nested `.await` calls creates deeply nested code (callback hell). Running multiple async operations concurrently with manual spawning is verbose.

**Solution**: Use future combinators: `map()`, `and_then()`, `or_else()` for chaining transformations. Use `join!` and `try_join!` to run futures concurrently, waiting for all.

**Why It Matters**: Proper composition determines performance and readability. Sequential `.await` on 3 independent operations takes 300ms; concurrent `join!` takes 100ms—3x faster.

**Use Cases**: HTTP request batching (parallel API calls), database query composition (dependent queries), microservice orchestration, retry logic with fallbacks, concurrent file operations, fan-out/fan-in patterns.

### Example: Future Combinators and Error Handling

Compose multiple async operations, handle errors gracefully, and transform results without nested callbacks.

```rust
// Note: Add to Cargo.toml:
// tokio = { version = "1.35", features = ["full"] }
// reqwest = "0.11"
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"

use tokio;
use std::time::Duration;

```

### Example: Basic future composition with map

Demonstrates the fundamental pattern of chaining async operations with synchronous transformations. The `fetch_user_name` function simulates an API call that returns a `Result`, while `get_user_name_uppercase` shows how to use `.map()` on the Result to transform the success value. The `.await` completes the async operation, then `.map()` transforms the result synchronously without another await. This keeps the code linear and readable rather than nested callbacks.

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

Shows how to chain multiple async operations where each depends on the previous result using the `?` operator. First fetches the user name, then fetches their posts—the second call can't happen until the first completes because we need the user data. The `?` operator provides early return on error, so if `fetch_user_name` fails, we immediately return that error without attempting to fetch posts. This is the async equivalent of sequential function calls in synchronous code.

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

Demonstrates how to create a unified error type that can represent multiple failure modes (network errors, not found, invalid data) and automatically convert from library errors using `From` trait. The `reqwest::Error` automatically converts to `AppError::Network` via the `?` operator. This pattern is critical for production code where you need to handle errors from multiple sources (HTTP client, JSON parsing, business logic) in a consistent way.

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

### Example: Combining multiple futures with different error types

Shows how to sequence multiple async calls that share the same error type. Each call uses `?` for early return on failure. Note these calls are sequential—data2 waits for data1 to complete. For parallel execution, see `join!` examples below.

```rust
async fn complex_operation() -> Result<String, AppError> {
    let data1 = fetch_json_data("https://api.example.com/data1").await?;
    let data2 = fetch_json_data("https://api.example.com/data2").await?;
    Ok(format!("Combined: {:?} and {:?}", data1, data2))
}
```

### Example: HTTP client with retries

Implements a generic retry wrapper with exponential backoff (2^attempts seconds between failures). The closure-based design (`FnMut() -> Fut`) allows retrying any async operation. This pattern is essential for resilient distributed systems where transient failures are common.

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

Runs multiple independent futures concurrently and waits for ALL to complete. Unlike sequential `.await` chains, `join!` starts all futures simultaneously—if each takes 100ms, three sequential awaits take 300ms, but `join!` takes only ~100ms. Returns a tuple of all results regardless of completion order. Use when you need all results before proceeding (e.g., fetching user profile + settings + notifications in parallel).

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

Like `join!` but for fallible futures (returning `Result`). If ANY future fails, `try_join!` cancels the remaining futures and returns immediately with that error—"fail fast" semantics. On success, unwraps all `Ok` values into a tuple. Use when all operations must succeed (e.g., loading required config files where any missing file is fatal).

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

Races multiple futures and returns when the FIRST one completes, cancelling all others. Unlike `join!` which waits for all, `select!` is for "whichever finishes first wins" scenarios. The unfinished futures are dropped (cancelled). Use for timeouts (operation vs timer), redundant requests (first server to respond), or cancellation (work vs cancel signal). Note: cancelled futures stop at their next `.await` point.

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

When you have a dynamic (runtime-determined) number of futures, use `FuturesUnordered`. Unlike `join!` which requires compile-time fixed count, `FuturesUnordered` accepts any iterator of futures. Results stream out in completion order (not submission order)—the fastest responses arrive first. Use for batch API calls, parallel downloads, or any fan-out pattern where count varies.

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
```

### Example: Timeout wrapper

Generic function to add timeout to any future. Returns `Ok(result)` if completed in time, `Err(Elapsed)` on timeout.

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

Atomic file write that either completes fully or not at all. Uses sync_all to ensure data is flushed to disk.

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

Shows the three main ways to create async streams: from iterators (for known data), from channels (for producer-consumer patterns), and from intervals (for time-based events). `stream::iter()` converts any iterator into a stream. Channels let a spawned task push values into a stream. Interval streams fire periodically for polling, heartbeats, or rate limiting.

```rust
async fn create_streams() {
    // From iterator - instant conversion of known data
    let s = stream::iter(vec![1, 2, 3, 4, 5]);

    // From channel - producer task sends values over time
    let (tx, rx) = tokio::sync::mpsc::channel(10);
    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(i).await.unwrap();
        }
    });
    let s = tokio_stream::wrappers::ReceiverStream::new(rx);

    // Interval stream - time-based events
    let s = stream::StreamExt::take(
        tokio_stream::wrappers::IntervalStream::new(
            tokio::time::interval(Duration::from_millis(100))
        ),
        5,  // Stop after 5 ticks
    );
}

```

### Example: Map and filter

Synchronous transformations on streams work like iterator combinators. `filter()` keeps elements matching a predicate, `map()` transforms each element. These are lazy—nothing happens until you consume the stream (with `.collect()`, `.next()`, etc.). Chain multiple operations for efficient pipelines without intermediate allocations.

```rust
async fn transform_stream() {
    let stream = stream::iter(1..=10)
        .filter(|x| x % 2 == 0)  // Keep evens: 2, 4, 6, 8, 10
        .map(|x| x * 2);         // Double: 4, 8, 12, 16, 20

    let results: Vec<i32> = stream.collect().await;
    println!("Transformed: {:?}", results);  // [4, 8, 12, 16, 20]
}
```

### Example: Then (async map)

When your transformation itself is async (e.g., fetching data for each element), use `.then()` instead of `.map()`. The closure returns a Future that will be awaited. Elements are processed sequentially by default—each must complete before the next starts. For concurrent processing, combine with `.buffer_unordered()`.

```rust
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
```

### Example: Fold and reduce

Aggregates stream elements into a single value using an accumulator. `fold(initial, closure)` starts with initial value and applies the closure to each element. Unlike `collect()` which builds a collection, `fold()` computes a single result (sum, max, concatenation, etc.). The closure receives accumulator and current element, returns new accumulator.

```rust
async fn aggregate_stream() {
    let sum = stream::iter(1..=100)
        .fold(0, |acc, x| acc + x)  // Sum: 0+1+2+...+100
        .await;

    println!("Sum: {}", sum);  // 5050
}
```

### Example: Take and skip

Pagination primitives for streams. `skip(n)` discards the first n elements, `take(n)` stops after n elements. Combine for offset-based pagination: `skip(page * size).take(size)`. Efficient for infinite streams—`take(5)` on an infinite stream yields exactly 5 elements then stops.

```rust
async fn limit_stream() {
    let results: Vec<i32> = stream::iter(1..=100)
        .skip(10)   // Skip first 10 (1-10)
        .take(5)    // Take next 5 (11-15)
        .collect()
        .await;

    println!("Limited: {:?}", results);  // [11, 12, 13, 14, 15]
}
```

### Example: Rate Limiting

Controls concurrency using a semaphore. The semaphore limits how many permits can be acquired simultaneously—here, max 5 concurrent requests. `buffer_unordered(10)` allows up to 10 futures to run, but only 5 can actually execute (semaphore-limited). Essential for respecting API rate limits, preventing server overload, or managing database connection pools.

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

Processes items in fixed-size batches. Useful for rate limiting bulk operations.

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

Combines two streams into one, yielding elements interleaved as they become ready.

```rust
async fn merge_streams() {
    use tokio_stream::StreamExt;
    let stream1 = stream::iter(vec![1, 2, 3]);
    let stream2 = stream::iter(vec![4, 5, 6]);
    let merged = stream::StreamExt::merge(stream1, stream2);
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

Manual `Stream` implementation for full control over yielding behavior. Implement `poll_next()` which returns `Poll::Ready(Some(item))` for next value, `Poll::Ready(None)` when exhausted, or `Poll::Pending` to yield to runtime. This low-level approach is rarely needed—prefer channel-based streams for most cases. Use manual impl for zero-allocation streams or complex state machines.

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

The practical way to create async generators: spawn a producer task that sends values through a channel, wrap the receiver as a stream. The producer runs independently, sending values as they become available. The channel provides backpressure—if consumer is slow, producer blocks on `send()`. Use this pattern for WebSocket messages, database cursors, or any producer-consumer scenario.

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

Streams database rows without loading entire result set into memory.

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

Creates a stream that emits values at fixed time intervals.

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

Spawn concurrent tasks, manage their lifecycle, and coordinate their completion.

```rust
use tokio;
use std::time::Duration;

```

### Example: Basic task spawning

`tokio::spawn()` creates independent tasks that run concurrently on the runtime's thread pool. Each spawn returns a `JoinHandle` that can be awaited to get the task's result. Unlike sequential `.await` chains, spawned tasks execute in parallel—two 100ms tasks complete in ~100ms total, not 200ms. Use `join!` to wait for multiple handles simultaneously. Spawned tasks are `'static`, meaning they can't borrow from the surrounding scope (use `Arc` or move ownership instead).

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

`JoinSet` manages a dynamic collection of spawned tasks, providing structured concurrency guarantees. Unlike loose `spawn()` calls, JoinSet tracks all tasks and ensures cleanup when dropped. `join_next()` returns results as tasks complete (not in spawn order), enabling efficient processing of variable-duration work. When JoinSet is dropped, all incomplete tasks are cancelled. This prevents task leaks and ensures proper resource cleanup—essential for long-running servers.

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

This pattern guarantees all spawned work completes before the function returns, similar to `std::thread::scope`. The `while let Some(result) = set.join_next().await` loop drains all tasks, blocking until every one finishes. Unlike fire-and-forget spawns, this ensures you have all results before proceeding. Use this for batch processing where you need to aggregate results, or when spawned tasks must complete before cleanup code runs.

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

`CancellationToken` enables cooperative cancellation across task hierarchies. The parent token can cancel all child tokens simultaneously. Tasks check for cancellation using `select!` between their work and `token.cancelled()`. Cancellation is cooperative—tasks stop at their next `.await` point, not immediately. This allows cleanup code to run. Use `child_token()` to create hierarchical cancellation: cancelling a parent cancels all children, but cancelling a child doesn't affect siblings or parent.

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

A fixed pool of worker tasks that process jobs from a shared queue. Workers share the receiver via `Arc<Mutex<Receiver>>` since mpsc receivers aren't clonable. Each worker loops, acquiring the lock to receive a task, then releasing before execution. When the sender is dropped, `recv()` returns `None` and workers exit cleanly. This pattern bounds concurrency (fixed workers), provides backpressure (channel buffer fills), and enables graceful shutdown (drop sender to stop workers).

```rust
use std::sync::Arc;
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

Automatically restarts failed tasks up to N times, inspired by Erlang's supervisor trees. The supervisor spawns the task, awaits its completion, and on panic (JoinError), waits before restarting. The delay between restarts prevents tight restart loops from consuming resources. Use for critical background services (database connections, health monitors) that should recover from transient failures. Consider exponential backoff for production systems.

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

Uses a `watch` channel to broadcast shutdown signals to background tasks. The service loops with `select!`, handling either interval ticks (normal work) or shutdown signals. `watch::changed()` completes when the sender broadcasts a new value. Unlike `oneshot`, `watch` can notify multiple receivers and be checked multiple times. The service completes its current iteration before stopping—no work is interrupted mid-operation. Essential for servers that need clean shutdown without losing in-flight requests.

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

The `?` operator works seamlessly in async functions, propagating errors up the call stack. When `fetch_user_data` returns `Err`, the `?` immediately returns that error from `get_user_profile` without executing remaining code. This creates clean, linear error handling without nested `match` statements. The async function's return type determines what errors can propagate—here both functions use the same `String` error type, so `?` works directly.

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

Handles transient failures by retrying with increasing delays (100ms, 200ms, 400ms...). The exponential growth prevents overwhelming failing services while giving them time to recover. The generic signature accepts any async operation returning `Result`. Each failure doubles the delay until max retries exceeded. Use for network requests, database connections, and any operation that might fail temporarily but succeed on retry.

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

Prevents cascading failures by "tripping" after repeated errors. Three states: Closed (normal operation), Open (all requests fail fast), HalfOpen (testing if service recovered). After `failure_threshold` failures, circuit opens and rejects requests immediately without calling the failing service. After `timeout`, circuit enters half-open to test recovery. Success in half-open closes the circuit; failure reopens it. Essential for microservices to prevent one failing dependency from taking down the entire system.

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

Returns a default value when the primary operation fails, ensuring the system degrades gracefully rather than failing completely. The fallback value is pre-computed and ready to return immediately. Use for non-critical data: show cached content when API is down, use default settings when config service fails, return empty results instead of errors. Keeps the user experience intact even during partial outages.

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

Limits concurrent access to a resource using a semaphore, preventing one component from consuming all system resources. Named after ship bulkheads that contain flooding to one compartment. `try_acquire()` returns immediately with an error if no permits available, rather than waiting. This "fail fast" behavior prevents request queuing during overload. Use to isolate database pools, external API clients, or any shared resource that could bottleneck under load.

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

`select!` races multiple futures and executes the branch of whichever completes first. The other futures are dropped (cancelled). Here, two channels receive messages at different times—select returns as soon as either has data. Unlike `join!` which waits for all, `select!` is "first one wins." By default, branches are checked in random order for fairness. Use for multiplexing events from different sources into a single handler.

```rust

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

Event loop pattern: continuously `select!` over multiple sources until all are exhausted. The `if !done1` guards disable branches after their channel closes, preventing busy-polling on closed channels. The `else` branch fires when all guarded branches are disabled—signaling loop termination. This pattern is the foundation of async servers: loop over connections, requests, and shutdown signals, handling whichever arrives next.

```rust

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
            Some(num) = rx1.recv(), if !done1 => {
                println!("Number: {}", num);
            }
            Some(msg) = rx2.recv(), if !done2 => {
                println!("Message: {}", msg);
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

`biased;` directive makes `select!` check branches in declaration order instead of randomly. The first ready branch always wins. Use for priority queues: high-priority messages are always processed before low-priority, even if both are ready. Without `biased`, Tokio randomizes to prevent starvation. Use biased select when you explicitly want priority ordering and accept that lower branches may starve under load.

```rust

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

Races a long-running request against a cancellation signal. If cancel arrives first, the request task is dropped (cancelled at its next `.await`). This pattern enables user-initiated cancellation: "Cancel" button sends to channel, select picks it up, background work stops. Note that spawned tasks aren't automatically cancelled when select completes—the JoinHandle is dropped but the task continues. For true cancellation, use `CancellationToken` inside the spawned task.

```rust
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

Classic server event loop: process requests until shutdown signal arrives. The loop `select!`s between incoming requests (normal work) and shutdown channel. On shutdown, break the loop and exit cleanly. Requests already being processed complete; new requests stop being accepted. This is graceful shutdown's foundation. Production servers add a timeout: if shutdown takes too long, force-exit to avoid hanging forever on stuck connections.

```rust
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

The `else` branch in `select!` fires when all other branches can't make progress (channel empty/closed). Unlike normal `select!` which waits, this enables non-blocking checks: "give me data if available, otherwise continue immediately." Use for polling without blocking, combining with timeouts, or implementing try-recv patterns.

```rust
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

`tokio::time::timeout(duration, future)` wraps any future with a time limit. Returns `Ok(result)` if future completes in time, `Err(Elapsed)` if timeout fires first. The wrapped future is cancelled on timeout. Always use timeouts for external operations (network, file I/O) to prevent indefinite hangs.

```rust
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

Combines timeout and retry patterns. Each attempt gets a fresh timeout. Handle three outcomes: success, failure (operation error), and timeout (no response). Treat timeout as retriable error. Pattern handles both slow services and failing services with single code path.

```rust
async fn timeout_with_retry() {
    for attempt in 1..=3 {
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

Deadlines are absolute times ("finish by 2pm"), timeouts are relative durations ("finish in 5s"). Convert deadline to remaining duration with `deadline.saturating_duration_since(Instant::now())`. Deadlines are better for multi-step operations: each step shares the same deadline, automatically shrinking available time.

```rust
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

Applies single timeout to a batch of operations. All operations must complete within the total timeout, not each one individually. If third operation takes too long, entire batch fails. Use when you need "all or nothing within time budget" semantics.

```rust
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

Token bucket rate limiter with automatic refill. Semaphore tracks available tokens, background task refills periodically. `acquire_with_timeout` fails fast if no tokens available within time limit—prevents request queuing forever when rate exceeded. Essential for API clients respecting server rate limits.

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

struct RateLimiter {
    semaphore: Arc<Semaphore>,
    refill_amount: usize,
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

Wraps an HTTP request in a timeout to detect unresponsive services. Health checks must complete quickly—a 30-second timeout defeats the purpose. The `?` after `timeout()` converts timeout error to the function's error type. Returns `Ok(true)` for healthy (2xx status), `Ok(false)` for unhealthy (4xx/5xx), and `Err` for timeout or network failure. Use in load balancers, service meshes, and monitoring systems.

```rust
async fn health_check(url: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let check = async {
        let response = reqwest::get(url).await?;
        Ok(response.status().is_success())
    };

    timeout(Duration::from_secs(5), check)
        .await
        .map_err(|_| "Health check timed out".into())?
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

Waits for workers to complete, but gives up after a grace period. During shutdown, you want to finish in-flight work, but can't wait forever for hung tasks. The timeout provides an upper bound: workers get `grace_period` to finish, then we proceed regardless. In production, log which workers didn't complete and consider force-killing their resources. This balances clean shutdown against availability (new instance can't start until old one exits).

```rust
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

`#[tokio::main]` creates a multi-threaded runtime by default, with worker threads equal to CPU cores. Spawned tasks are distributed across threads via work-stealing: idle threads steal tasks from busy threads' queues. Notice different thread IDs in output—tasks migrate between threads. This maximizes CPU utilization for I/O-bound workloads. The runtime handles all thread management; you just spawn tasks.

```rust

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

`flavor = "current_thread"` runs all tasks on the main thread. No thread synchronization overhead, no `Send` requirement on spawned futures. All tasks share the same thread ID. Simpler and faster for I/O-bound workloads with few concurrent tasks. Ideal for CLI tools, embedded systems, WASM, or when you need `!Send` types like `Rc`. Drawback: one blocking operation stalls everything.

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

Build runtime manually for fine-grained control: worker thread count, thread names (for debugging), stack size, enabled features. Use `block_on()` to run async code on manually-built runtime. Useful for embedding Tokio in larger applications or tuning for specific workloads.

```rust
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

NEVER call blocking code directly in async context—it stalls the entire runtime. Use `spawn_blocking()` to run blocking code on dedicated thread pool, keeping async threads free. Essential for: file I/O (sync std::fs), CPU computation, blocking FFI, legacy libraries. Returns JoinHandle to await result.

```rust
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

`LocalSet` enables spawning `!Send` futures (containing Rc, RefCell, etc.) that can't cross thread boundaries. Tasks spawned with `spawn_local()` stay on current thread. Use for single-threaded async with non-Send data, WASM targets, or wrapping non-threadsafe libraries.

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

Combine Tokio (async I/O) with Rayon (parallel CPU). Wrap Rayon's parallel operations in `spawn_blocking()` to avoid blocking async runtime. Rayon uses work-stealing for CPU parallelism; Tokio handles I/O concurrency. Pattern: fetch data with Tokio, process with Rayon, store results with Tokio.

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

Run I/O-bound and CPU-bound tasks concurrently. I/O task uses async sleep (yields to runtime), CPU task uses spawn_blocking with sync sleep (runs on blocking pool). `join!` waits for both. Neither blocks the other—true concurrent execution of heterogeneous workloads.

```rust
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

    tokio::join!(io_task, cpu_task);
}

#[tokio::main]
async fn main() {
    mixed_workload().await;
}
```
---

### Example: Runtime Comparison and Interop

Side-by-side comparison of Tokio and async-std syntax. APIs are similar: spawn tasks, sleep, collect results. Main differences: Tokio has more features and ecosystem; async-std mirrors std library more closely. Feature flags allow compiling same code for both runtimes.

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

Same logic as Tokio but using async-std APIs. Note: JoinHandle doesn't require `.unwrap()` (async-std tasks don't return Result). API names mirror std: `task::spawn`, `task::sleep`. Swap runtimes by changing feature flag and imports.

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

Write library code that works with any runtime using the `futures` crate. Use generic `Future` trait instead of runtime-specific spawn. Callers provide the async transformation; this code only orchestrates. Essential pattern for reusable async libraries.

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

Benchmark task spawn/completion overhead. Spawn 1000 tasks with minimal work, measure total time. Results vary by workload: Tokio often faster for high-contention scenarios due to work-stealing; async-std may win for simpler workloads. Always benchmark your specific use case.

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

`futures::executor::block_on` runs futures without any runtime—useful for tests, simple scripts, or one-off async operations. Not for production servers (no I/O reactor), but great for compatibility and testing. Combinators from `futures` crate work with any runtime.

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
