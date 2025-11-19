# Async Runtime Patterns

[Future Composition](#future-composition)

- Problem: Nested .await creates callback hell; verbose manual spawning; sequential when parallel possible
- Solution: Combinators map(), and_then(); join!/try_join! for concurrent; select! for racing
- Why It Matters: 3x faster with concurrent execution; eliminates callback hell; ? makes errors concise
- Use Cases: HTTP batching, database composition, microservice orchestration, retry logic, fan-out/fan-in

[Stream Processing](#stream-processing)

- Problem: Standard iterators block; collecting wastes memory; manual polling verbose; backpressure complex
- Solution: Stream trait with combinators; .buffered(n) for concurrency; automatic backpressure
- Why It Matters: Process GB files in constant memory; backpressure prevents overwhelming consumers
- Use Cases: WebSocket messages, sensor aggregation, log streaming, database results, real-time analytics

[Async/Await Patterns](#async-await-patterns)

- Problem: Manual polling complex; combinator chains unreadable; cancellation unsafe; Send bounds tricky
- Solution: async fn with .await; spawn_blocking() for blocking; cancellation-safe design
- Why It Matters: Transforms spaghetti to readable code; Send bounds catch issues at compile time
- Use Cases: Web servers, database clients, HTTP clients, file I/O, microservices, chat servers

[Select and Timeout Patterns](#select-and-timeout-patterns)

- Problem: Indefinite waits cause hangs; need first-completion handling; no fallback mechanism
- Solution: select! for racing; timeout() for bounds; interval() for periodic tasks
- Why It Matters: Prevents resource leaks; enables responsive UIs; critical for production systems
- Use Cases: HTTP timeouts, idle connections, health checks, graceful shutdown, rate limiting, circuit breakers

[Runtime Comparison](#runtime-comparison)

- Problem: Wrong runtime impacts performance and ecosystem; mixing causes conflicts; varied overhead
- Solution: Tokio for general use; async-std for simplicity; smol for lightweight; embassy for embedded
- Why It Matters: Determines ecosystem access; memory footprint varies 10x; prevents painful switching
- Use Cases: Tokio for servers, async-std for learning, smol for single-threaded, embassy for embedded

[Tokio Cheat Sheet](#tokio-cheat-sheet)
 - common **tokio** functions

# Overview
This chapter explores asynchronous programming patterns in Rust using async/await and async runtimes. We'll cover future composition, stream processing, concurrency patterns, timeout handling, and runtime comparisons through practical, production-ready examples.



## Pattern 1: Future Composition

**Problem**: Chaining async operations with nested `.await` calls creates deeply nested code (callback hell). Running multiple async operations concurrently with manual spawning is verbose. Error handling across async boundaries requires repetitive `match` statements. Sequential operations when some could run in parallel wastes time. Transforming future results requires awkward combinator chains or intermediate variables.

**Solution**: Use future combinators: `map()`, `and_then()`, `or_else()` for chaining transformations. Use `join!` and `try_join!` to run futures concurrently, waiting for all. Use `select!` to race futures, completing on first. Use `?` operator for error propagation in async functions. Leverage `.await` for sequential dependencies, combinators for parallel execution.

**Why It Matters**: Proper composition determines performance and readability. Sequential `.await` on 3 independent operations takes 300ms; concurrent `join!` takes 100ms—3x faster. Combinators eliminate callback hell, making async code as readable as sync. Error handling with `?` is concise: one character vs 5-line match. Understanding when to use sequential vs concurrent execution is critical for I/O-bound applications.

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

//=============================================
// Pattern 1: Basic future composition with map
//=============================================
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
    // Map over the result
    fetch_user_name(user_id)
        .await
        .map(|name| name.to_uppercase())
}

//=====================================
// Pattern 2: Chaining async operations
//=====================================
async fn fetch_user_posts(user_id: u64) -> Result<Vec<String>, String> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    Ok(vec![
        format!("Post 1 by user {}", user_id),
        format!("Post 2 by user {}", user_id),
    ])
}

async fn get_user_with_posts(user_id: u64) -> Result<(String, Vec<String>), String> {
    let name = fetch_user_name(user_id).await?;
    let posts = fetch_user_posts(user_id).await?;
    Ok((name, posts))
}

//============================================
// Pattern 3: Error conversion and propagation
//============================================
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
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(AppError::NotFound);
    }

    let data = response.json().await?;
    Ok(data)
}

//=================================================================
// Pattern 4: Combining multiple futures with different error types
//=================================================================
use futures::future::TryFutureExt;

async fn complex_operation() -> Result<String, AppError> {
    let data1 = fetch_json_data("https://api.example.com/data1")
        .await?;

    let data2 = fetch_json_data("https://api.example.com/data2")
        .await?;

    // Process both results
    Ok(format!("Combined: {:?} and {:?}", data1, data2))
}

//=====================================
// Real-world: HTTP client with retries
//=====================================
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
                tokio::time::sleep(Duration::from_secs(2u64.pow(attempts as u32))).await;
            }
        }
    }
}

async fn fetch_data_with_retry(url: String) -> Result<String, reqwest::Error> {
    fetch_with_retry(
        || async {
            reqwest::get(&url)
                .await?
                .text()
                .await
        },
        3,
    )
    .await
}

#[tokio::main]
async fn main() {
    println!("=== Future Composition ===\n");

    match get_user_name_uppercase(42).await {
        Ok(name) => println!("User name: {}", name),
        Err(e) => println!("Error: {}", e),
    }

    match get_user_with_posts(42).await {
        Ok((name, posts)) => {
            println!("User: {}", name);
            println!("Posts: {:?}", posts);
        }
        Err(e) => println!("Error: {}", e),
    }
}
```

**Future Composition Patterns**:
- **map**: Transform success value
- **and_then**: Chain dependent operations
- **or_else**: Handle errors and recover
- **? operator**: Early return on error

---

### Example: Concurrent Future Execution

Execute multiple independent futures concurrently to improve throughput.

```rust
use tokio;
use std::time::Duration;

//========================================
// Pattern 1: join! - wait for all futures
//========================================
async fn concurrent_fetch() {
    let (result1, result2, result3) = tokio::join!(
        fetch_user_name(1),
        fetch_user_name(2),
        fetch_user_name(3),
    );

    println!("Results: {:?}, {:?}, {:?}", result1, result2, result3);
}

//========================================================
// Pattern 2: try_join! - wait for all, fail fast on error
//========================================================
async fn concurrent_fetch_fail_fast() -> Result<(String, String, String), String> {
    tokio::try_join!(
        fetch_user_name(1),
        fetch_user_name(2),
        fetch_user_name(3),
    )
}

//==========================================================
// Pattern 3: select! - race futures, take first to complete
//==========================================================
use tokio::time::sleep;

async fn race_requests() -> String {
    tokio::select! {
        result = fetch_user_name(1) => {
            format!("First: {:?}", result)
        }
        result = fetch_user_name(2) => {
            format!("Second: {:?}", result)
        }
        _ = sleep(Duration::from_secs(1)) => {
            "Timeout".to_string()
        }
    }
}

//===========================================================
// Pattern 4: Dynamic number of futures with FuturesUnordered
//===========================================================
use futures::stream::{FuturesUnordered, StreamExt};

async fn fetch_all_users(user_ids: Vec<u64>) -> Vec<Result<String, String>> {
    let futures: FuturesUnordered<_> = user_ids
        .into_iter()
        .map(|id| fetch_user_name(id))
        .collect();

    futures.collect().await
}

//==============================================
// Real-world: Parallel HTTP requests with limit
//==============================================
use futures::stream::FuturesOrdered;

async fn fetch_urls_concurrently(urls: Vec<String>, max_concurrent: usize) -> Vec<Result<String, reqwest::Error>> {
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

//============================
// Real-world: Timeout wrapper
//============================
async fn with_timeout<F, T>(
    future: F,
    duration: Duration,
) -> Result<T, tokio::time::error::Elapsed>
where
    F: std::future::Future<Output = T>,
{
    tokio::time::timeout(duration, future).await
}

//=========================================
// Real-world: Cancellation-safe operations
//=========================================
async fn cancellation_safe_write(data: String) -> Result<(), std::io::Error> {
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    let mut file = File::create("output.txt").await?;

    // Write atomically - either all or nothing
    file.write_all(data.as_bytes()).await?;
    file.sync_all().await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    println!("=== Concurrent Execution ===\n");

    concurrent_fetch().await;

    println!("\n=== Fail Fast ===\n");
    match concurrent_fetch_fail_fast().await {
        Ok(results) => println!("All succeeded: {:?}", results),
        Err(e) => println!("One failed: {}", e),
    }

    println!("\n=== Race ===\n");
    let winner = race_requests().await;
    println!("Winner: {}", winner);

    println!("\n=== Dynamic Futures ===\n");
    let results = fetch_all_users(vec![1, 2, 3, 4, 5]).await;
    println!("Fetched {} users", results.len());

    println!("\n=== Timeout ===\n");
    match with_timeout(
        fetch_user_name(1),
        Duration::from_millis(50),
    ).await {
        Ok(name) => println!("Got name: {:?}", name),
        Err(_) => println!("Timed out"),
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

**Problem**: Processing infinite or unbounded sequences (websocket messages, sensor data, log streams) with standard iterators blocks thread. Collecting entire stream into Vec before processing wastes memory for large datasets. Manual polling with `while let` and channels is verbose. Backpressure (slowing producer when consumer lags) needs careful implementation. Chaining transformations on async sequences requires custom boilerplate.

**Solution**: Use `Stream` trait (async iterator) to yield values over time without blocking. Apply stream combinators: `.map()`, `.filter()`, `.fold()`, `.buffered()` for transformations. Use `.next().await` to pull values lazily. Leverage `.buffered(n)` for controlled concurrency. Use `.throttle()` and `.chunks()` for batching and rate limiting. Backpressure is automatic—consumer drives producer.

**Why It Matters**: Streams enable processing data larger than memory—GB log file analyzed in constant memory. WebSocket connections handle millions of messages without collecting all. Backpressure prevents overwhelming slow consumers: HTTP proxy can't let fast upstream flood slow downstream. Stream combinators make event processing pipelines readable and composable. Real-time data processing (sensor feeds, market data, logs) requires streams.

**Use Cases**: WebSocket message processing, sensor data aggregation, log file streaming, database query result streaming, event sourcing, pub-sub systems, real-time analytics, infinite data sources.

### Example: Stream Combinators

 Process async sequences of data with transformations, filtering, and aggregation.

```rust
use tokio;
use tokio_stream::{self as stream, StreamExt};
use std::time::Duration;

//============================
// Pattern 1: Creating streams
//============================
async fn create_streams() {
    // From iterator
    let s = stream::iter(vec![1, 2, 3, 4, 5]);

    // From channel
    let (tx, rx) = tokio::sync::mpsc::channel(10);
    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(i).await.unwrap();
        }
    });
    let s = tokio_stream::wrappers::ReceiverStream::new(rx);

    // Interval stream
    let s = stream::StreamExt::take(
        tokio_stream::wrappers::IntervalStream::new(
            tokio::time::interval(Duration::from_millis(100))
        ),
        5,
    );
}

//==========================
// Pattern 2: Map and filter
//==========================
async fn transform_stream() {
    let stream = stream::iter(1..=10)
        .filter(|x| x % 2 == 0)
        .map(|x| x * 2);

    let results: Vec<i32> = stream.collect().await;
    println!("Transformed: {:?}", results);
}

//============================
// Pattern 3: Then (async map)
//============================
async fn async_transform_stream() {
    let stream = stream::iter(1..=5)
        .then(|x| async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            x * x
        });

    let results: Vec<i32> = stream.collect().await;
    println!("Async transformed: {:?}", results);
}

//===========================
// Pattern 4: Fold and reduce
//===========================
async fn aggregate_stream() {
    let sum = stream::iter(1..=100)
        .fold(0, |acc, x| acc + x)
        .await;

    println!("Sum: {}", sum);
}

//=========================
// Pattern 5: Take and skip
//=========================
async fn limit_stream() {
    let results: Vec<i32> = stream::iter(1..=100)
        .skip(10)
        .take(5)
        .collect()
        .await;

    println!("Limited: {:?}", results);
}

//==========================
// Real-world: Rate limiting
//==========================
use std::sync::Arc;
use tokio::sync::Semaphore;

async fn rate_limited_requests(urls: Vec<String>) {
    let semaphore = Arc::new(Semaphore::new(5)); // Max 5 concurrent

    let stream = stream::iter(urls)
        .map(|url| {
            let permit = Arc::clone(&semaphore);
            async move {
                let _permit = permit.acquire().await.unwrap();
                println!("Fetching: {}", url);
                // Simulate request
                tokio::time::sleep(Duration::from_millis(100)).await;
                format!("Response from {}", url)
            }
        })
        .buffer_unordered(10); // Process up to 10 at once

    let results: Vec<String> = stream.collect().await;
    println!("Fetched {} URLs", results.len());
}

//=============================
// Real-world: Batch processing
//=============================
async fn batch_process<T>(items: Vec<T>, batch_size: usize)
where
    T: Send + 'static,
{
    use futures::stream;

    let batches = items.chunks(batch_size);

    for (i, batch) in batches.enumerate() {
        println!("Processing batch {}: {} items", i, batch.len());
        // Process batch
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

//===========================
// Real-world: Stream merging
//===========================
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
    println!("=== Transform Stream ===\n");
    transform_stream().await;

    println!("\n=== Async Transform ===\n");
    async_transform_stream().await;

    println!("\n=== Aggregate ===\n");
    aggregate_stream().await;

    println!("\n=== Limit ===\n");
    limit_stream().await;

    println!("\n=== Rate Limiting ===\n");
    let urls: Vec<_> = (0..20).map(|i| format!("https://example.com/{}", i)).collect();
    rate_limited_requests(urls).await;

    println!("\n=== Merge ===\n");
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

Create custom async streams from various sources like WebSockets, file watching, or event sources.

```rust
use tokio;
use tokio_stream::{Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};

//========================================
// Pattern 1: Manual stream implementation
//========================================
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
            Poll::Ready(Some(current))
        } else {
            Poll::Ready(None)
        }
    }
}

//==================================================
// Pattern 2: Async generator pattern using channels
//==================================================
async fn number_generator(max: u32) -> impl Stream<Item = u32> {
    let (tx, rx) = tokio::sync::mpsc::channel(10);

    tokio::spawn(async move {
        for i in 0..max {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            if tx.send(i).await.is_err() {
                break;
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
}

//================================
// Real-world: File watcher stream
//================================
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

//==================================================
// Real-world: WebSocket message stream (simulation)
//==================================================
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

//=========================================
// Real-world: Database query result stream
//=========================================
#[derive(Debug)]
struct Row {
    id: u64,
    data: String,
}

async fn database_query_stream(query: String) -> impl Stream<Item = Row> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    tokio::spawn(async move {
        // Simulate database query returning rows
        for i in 0..10 {
            tokio::time::sleep(Duration::from_millis(10)).await;
            let row = Row {
                id: i,
                data: format!("Data {}", i),
            };
            if tx.send(row).await.is_err() {
                break;
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
}

//=================================
// Pattern 3: Interval-based stream
//=================================
async fn ticker_stream(interval: Duration, count: usize) -> impl Stream<Item = u64> {
    let (tx, rx) = tokio::sync::mpsc::channel(10);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(interval);
        for i in 0..count {
            interval.tick().await;
            if tx.send(i as u64).await.is_err() {
                break;
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
}

#[tokio::main]
async fn main() {
    println!("=== Counter Stream ===\n");
    let mut stream = CounterStream::new(5);
    while let Some(n) = stream.next().await {
        println!("Count: {}", n);
    }

    println!("\n=== Number Generator ===\n");
    let mut stream = number_generator(5).await;
    while let Some(n) = stream.next().await {
        println!("Generated: {}", n);
    }

    println!("\n=== WebSocket Stream ===\n");
    let mut stream = websocket_stream().await;
    while let Some(msg) = stream.next().await {
        println!("Message: {:?}", msg);
    }

    println!("\n=== Database Stream ===\n");
    let mut stream = database_query_stream("SELECT * FROM users".to_string()).await;
    while let Some(row) = stream.next().await {
        println!("Row: {:?}", row);
    }

    println!("\n=== Ticker Stream ===\n");
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

**Problem**: Manual future polling with `.poll()` is complex and error-prone. Combinator chains (`.and_then().map()`) become unreadable for complex logic. Async closures aren't stabilized, limiting functional patterns. Cancellation safety—dropped futures may leave resources in inconsistent state. Mixing sync and async code requires careful runtime management. Send bounds cause lifetime issues with spawned tasks.

**Solution**: Use `async fn` and `.await` for sequential async code that reads like sync. Mark functions `async` to return `impl Future`. Use `.await` to suspend until future completes. Wrap blocking code in `spawn_blocking()` to avoid blocking async runtime. Make data `Send + 'static` for spawning. Use `async move {}` blocks to capture variables. Design APIs with cancellation safety in mind (idempotent operations, cleanup in `Drop`).

**Why It Matters**: Async/await transforms async programming from callback spaghetti to readable imperative code. HTTP request handler with 5 operations: combinator chain is 20 lines of `.and_then()`, async/await is 10 lines reading like sync. `.await` hides complexity of future state machines. Send bounds catch thread-safety issues at compile time. Cancellation safety prevents bugs where cancelled request leaves database connection in bad state. This is foundation for all modern async Rust.

**Use Cases**: Web servers (async request handlers), database clients (async queries), HTTP clients (async requests), file I/O (async read/write), microservices (async RPC), chat servers, real-time systems.

### Example: Task Spawning and Structured Concurrency

Spawn concurrent tasks, manage their lifecycle, and coordinate their completion.

```rust
use tokio;
use std::time::Duration;

//===============================
// Pattern 1: Basic task spawning
//===============================
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

//===============================================
// Pattern 2: Structured concurrency with JoinSet
//===============================================
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

//==================================================================
// Pattern 3: Scoped tasks (guaranteed completion before scope ends)
//==================================================================
async fn scoped_tasks() {
    let mut data = vec![1, 2, 3, 4, 5];

    tokio::task::scope(|scope| {
        for item in &mut data {
            scope.spawn(async move {
                *item *= 2;
            });
        }
    });

    println!("Modified data: {:?}", data);
}

//=============================
// Pattern 4: Task cancellation
//=============================
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

//================================
// Real-world: Worker pool pattern
//================================
struct WorkerPool {
    tasks: tokio::sync::mpsc::Sender<Box<dyn FnOnce() + Send + 'static>>,
}

impl WorkerPool {
    fn new(num_workers: usize) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Box<dyn FnOnce() + Send + 'static>>(100);

        for i in 0..num_workers {
            let mut rx = rx.clone();
            tokio::spawn(async move {
                while let Some(task) = rx.recv().await {
                    println!("Worker {} executing task", i);
                    task();
                }
            });
        }

        Self { tasks: tx }
    }

    async fn submit<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.tasks.send(Box::new(task)).await.unwrap();
    }
}

//====================================================
// Real-world: Supervisor pattern (restart on failure)
//====================================================
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

//===================================================
// Real-world: Background task with graceful shutdown
//===================================================
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
    println!("=== Basic Task Spawning ===\n");
    spawn_basic_tasks().await;

    println!("\n=== Structured Concurrency ===\n");
    structured_concurrency().await;

    println!("\n=== Cancellable Task ===\n");
    cancellable_task().await;

    println!("\n=== Graceful Shutdown ===\n");
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

### Example: Error Handling and Recovery

Handle errors in async code, implement retry logic, and provide fallback mechanisms.

```rust
use tokio;
use std::time::Duration;

//=====================================
// Pattern 1: Result propagation with ?
//=====================================
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

//==========================================
// Pattern 2: Retry with exponential backoff
//==========================================
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

//===========================
// Pattern 3: Circuit breaker
//===========================
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

//============================
// Pattern 4: Fallback pattern
//============================
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

//==================================================
// Real-world: Bulkhead pattern (resource isolation)
//==================================================
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
    println!("=== Retry with Backoff ===\n");

    let mut attempts = 0;
    let result = retry_with_backoff(
        || async {
            attempts += 1;
            if attempts < 3 {
                Err("Temporary failure")
            } else {
                Ok("Success!")
            }
        },
        5,
        Duration::from_millis(100),
    ).await;

    println!("Final result: {:?}\n", result);

    println!("=== Circuit Breaker ===\n");

    let breaker = CircuitBreaker::new(3, 2, Duration::from_secs(2));

    for i in 0..10 {
        let result: Result<String, String> = breaker.call(|| async {
            if i < 5 {
                Err("Service unavailable".to_string())
            } else {
                Ok("Success".to_string())
            }
        }).await;

        println!("Call {}: {:?}", i, result);
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
```

**Error Handling Patterns**:
- **Retry**: Exponential backoff
- **Circuit breaker**: Prevent cascading failures
- **Fallback**: Default value on error
- **Bulkhead**: Limit concurrent requests



## Pattern 4: Select and Timeout Patterns

**Problem**: Waiting indefinitely for async operations causes hangs—network request that never responds blocks forever. Need to handle whichever of multiple operations completes first (user input vs network response). Graceful degradation requires fallbacks when primary fails. Periodic operations (heartbeats, polling) require scheduling. Multiple concurrent events need priority handling—urgent messages before routine.

**Solution**: Use `tokio::select!` to race multiple futures, completing when first finishes. Use `tokio::time::timeout()` to bound operation duration. Combine for timeout-with-fallback: primary operation with timeout, fallback on timeout. Use `tokio::time::interval()` for periodic tasks. Pattern-match on select branches to handle different completion scenarios. Use biased select for priority-based handling.

**Why It Matters**: Timeouts prevent resource leaks from hung operations—HTTP server without timeouts accumulates connections from slow clients until memory exhausted. Select enables responsive UIs: user input cancels background computation immediately. Circuit breakers rely on timeouts to detect failing services. Graceful degradation: primary API with 100ms timeout, fallback cache on timeout. Understanding timeout/select patterns is critical for production-ready async systems.

**Use Cases**: HTTP clients (request timeouts), connection management (idle timeouts), health checks (periodic pings), graceful shutdown (timeout on cleanup), rate limiting (interval-based), user cancellation (input vs background work), circuit breakers.

### Example: Select Patterns

Wait on multiple async operations and react to whichever completes first.


```rust
use tokio;
use tokio::sync::mpsc;
use std::time::Duration;

//==========================================
// Pattern 1: Basic select with two branches
//==========================================
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

//============================
// Pattern 2: Select in a loop
//============================
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

//====================================
// Pattern 3: Biased select (priority)
//====================================
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

//======================================
// Real-world: Request with cancellation
//======================================
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

//========================================
// Real-world: Server with shutdown signal
//========================================
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

//==============================================
// Pattern 4: Select with default (non-blocking)
//==============================================
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
    println!("=== Select Two Channels ===\n");
    select_two_channels().await;

    println!("\n=== Select Loop ===\n");
    select_loop().await;

    println!("\n=== Biased Select ===\n");
    biased_select().await;

    println!("\n=== Request with Cancel ===\n");
    request_with_cancel().await;

    println!("\n=== Server with Shutdown ===\n");
    server_with_shutdown().await;

    println!("\n=== Select with Default ===\n");
    select_with_default().await;
}
```

**Select Patterns**:
- **Basic select**: Race multiple futures
- **Loop select**: Continuous event handling
- **Biased select**: Priority ordering
- **With cancellation**: Abort long operations
- **Default**: Non-blocking poll

---

### Example: Timeout and Deadline Patterns

Enforce time limits on async operations to prevent indefinite blocking.

```rust
use tokio;
use tokio::time::{timeout, sleep, Duration, Instant};

//=========================
// Pattern 1: Basic timeout
//=========================
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

//==============================
// Pattern 2: Timeout with retry
//==============================
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

//=============================
// Pattern 3: Deadline tracking
//=============================
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

//===========================================
// Pattern 4: Timeout for multiple operations
//===========================================
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

//======================================
// Real-world: Rate limiter with timeout
//======================================
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

//======================================
// Real-world: Health check with timeout
//======================================
async fn health_check(url: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let check = async {
        let response = reqwest::get(url).await?;
        Ok(response.status().is_success())
    };

    timeout(Duration::from_secs(5), check)
        .await
        .map_err(|_| "Health check timed out".into())?
}

//===================================================
// Real-world: Graceful timeout (finish current work)
//===================================================
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
    println!("=== Basic Timeout ===\n");
    basic_timeout().await;

    println!("\n=== Timeout with Retry ===\n");
    timeout_with_retry().await;

    println!("\n=== Deadline ===\n");
    deadline_example().await;

    println!("\n=== Timeout All ===\n");
    timeout_all().await;

    println!("\n=== Rate Limiter ===\n");
    let limiter = RateLimiter::new(5, 2, Duration::from_millis(500));

    for i in 0..10 {
        match limiter.acquire_with_timeout(Duration::from_secs(1)).await {
            Ok(_) => println!("Request {} allowed", i),
            Err(e) => println!("Request {} rejected: {}", i, e),
        }
        sleep(Duration::from_millis(100)).await;
    }
}
```

**Timeout Patterns**:
- **Basic timeout**: Single operation limit
- **Deadline**: Absolute time limit
- **Timeout all**: Batch operation limit
- **Graceful timeout**: Allow cleanup before forcing stop
- **Rate limiter**: Control request rate with timeout

---

## Pattern 5: Runtime Comparison

**Problem**: Choosing wrong async runtime impacts performance, features, and maintainability. Tokio dominates ecosystem but isn't always best choice. Single-threaded vs multi-threaded runtimes have different characteristics. No-std embedded systems can't use Tokio. Mixing runtimes in dependencies causes conflicts. Runtime overhead varies: some add microseconds, others nearly zero. Feature sets differ: timer resolution, I/O drivers, task spawning, metrics.

**Solution**: Use Tokio for general-purpose applications: mature, full-featured, excellent ecosystem. Use async-std for simpler API, closer to std library patterns. Use smol for lightweight footprint, single-threaded executor. Use embassy for embedded no-std environments. Evaluate based on: ecosystem (library compatibility), performance (latency, throughput), features (timers, I/O, sync primitives), and platform support (WASM, embedded).

**Why It Matters**: Runtime choice determines ecosystem access—Tokio has 10x more compatible libraries than alternatives. Performance varies: work-stealing vs single-threaded, epoll vs io_uring. Memory footprint: Tokio ~500KB, smol ~50KB—critical for embedded. Embassy enables async on microcontrollers without std. Understanding trade-offs prevents mid-project runtime switching (painful). Wrong choice: using Tokio in WASM adds megabytes, smol adds kilobytes.

**Use Cases**: Tokio for web servers, databases, general applications. async-std for learning, simpler projects. smol for single-threaded, minimal overhead. embassy for embedded systems, bare-metal. Runtime-agnostic libraries for maximum compatibility.

### Example: Tokio Runtime Features

Understand Tokio's features and how to configure them for different workloads.

```rust
use tokio;
use std::time::Duration;

//============================================
// Pattern 1: Multi-threaded runtime (default)
//============================================
#[tokio::main]
async fn multi_threaded_example() {
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

//===================================
// Pattern 2: Single-threaded runtime
//===================================
#[tokio::main(flavor = "current_thread")]
async fn single_threaded_example() {
    println!("Running on single-threaded runtime");

    let thread_id = std::thread::current().id();

    for i in 0..5 {
        tokio::spawn(async move {
            println!("Task {} on thread {:?}", i, std::thread::current().id());
        }).await.unwrap();
    }

    println!("All tasks ran on thread {:?}", thread_id);
}

//========================================
// Pattern 3: Custom runtime configuration
//========================================
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

//===============================
// Pattern 4: Blocking operations
//===============================
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

//==============================================
// Pattern 5: Local task set (for !Send futures)
//==============================================
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

//======================================
// Real-world: CPU-bound work with rayon
//======================================
use rayon::prelude::*;

async fn cpu_bound_with_rayon() {
    let numbers: Vec<u64> = (0..1_000_000).collect();

    let sum = tokio::task::spawn_blocking(move || {
        numbers.par_iter().sum::<u64>()
    }).await.unwrap();

    println!("Sum: {}", sum);
}

//=========================================
// Real-world: Mixed workload (I/O and CPU)
//=========================================
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

fn main() {
    println!("=== Multi-threaded Runtime ===\n");
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(multi_threaded_example());

    println!("\n=== Custom Runtime ===\n");
    custom_runtime_example();

    println!("\n=== Blocking Operations ===\n");
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(handle_blocking_operations());

    println!("\n=== Local Task Set ===\n");
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(local_task_set_example());

    println!("\n=== Mixed Workload ===\n");
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(mixed_workload());
}
```

**Tokio Features**:
- **Multi-threaded**: Work-stealing scheduler
- **Current-thread**: Single-threaded for simpler workloads
- **spawn_blocking**: Offload blocking operations
- **LocalSet**: Support !Send futures
- **Configurable**: Thread count, stack size, naming

---

### Example: Runtime Comparison and Interop

 Compare Tokio and async-std, understand trade-offs, and enable interoperability.

```rust
//==============
// Tokio version
//==============
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

//==================
// async-std version
//==================
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

//====================================
// Runtime-agnostic code using futures
//====================================
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

//===================
// Feature comparison
//===================
/**
 * Tokio vs async-std:
 *
 * Tokio:
 * - Work-stealing scheduler (better for CPU-intensive tasks)
 * - More configuration options
 * - Larger ecosystem (widely used)
 * - spawn_blocking for blocking operations
 * - Good for web servers, databases
 *
 * async-std:
 * - Simpler API (mirrors std library)
 * - Easier to learn
 * - Good for general-purpose async
 * - Less configuration needed
 * - Good for CLI tools, simpler services
 */

//===============================
// Performance comparison example
//===============================
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

//===============================================
// Interop: using futures crate for compatibility
//===============================================
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

/**
 * Choosing a Runtime:
 *
 * Use Tokio when:
 * - Building high-performance web servers
 * - Need fine-grained control over runtime
 * - Working with Tokio ecosystem (tonic, axum, etc.)
 * - CPU-bound tasks mixed with I/O
 *
 * Use async-std when:
 * - Building CLI tools or simpler services
 * - Want std-like API familiarity
 * - Primarily I/O-bound workload
 * - Simpler application with less configuration
 *
 * Use runtime-agnostic futures when:
 * - Writing libraries
 * - Need portability
 * - Want to avoid runtime lock-in
 */

fn main() {
    println!("=== Runtime Interop ===\n");
    interop_example();

    #[cfg(feature = "tokio-runtime")]
    tokio_example::run();

    #[cfg(feature = "async-std-runtime")]
    async_std_example::run();
}
```

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

## Summary

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

## Tokio Cheat Sheet
```rust
// Runtime creation
#[tokio::main]
async fn main() { }                                  // Default runtime (multi-thread)

#[tokio::main(flavor = "current_thread")]
async fn main() { }                                  // Single-threaded runtime

#[tokio::main(worker_threads = 4)]
async fn main() { }                                  // Custom thread count

Runtime::new().unwrap()                              // Manual runtime creation
Builder::new_multi_thread().build().unwrap()         // Builder pattern

// Basic async/await
async fn my_function() -> Result<T, E> { }          // Define async function
my_function().await                                  // Call and await
my_function().await?                                 // Await and propagate error

// Task spawning
tokio::spawn(async { /* code */ })                  // Spawn task, returns JoinHandle
tokio::spawn(async move { /* code */ })             // Move ownership into task
handle.await                                         // Wait for task completion
handle.await.unwrap()                                // Wait and unwrap result
handle.abort()                                       // Cancel task

// Blocking operations
tokio::task::spawn_blocking(|| { /* sync code */ }) // Run blocking code in thread pool
tokio::task::block_in_place(|| { /* sync code */ }) // Block current worker thread

// Task management
tokio::task::yield_now().await                       // Yield to scheduler
tokio::task::spawn_local(async { /* code */ })      // Spawn on LocalSet (not Send)

// Sleeping & timing
tokio::time::sleep(Duration::from_secs(1)).await    // Async sleep
tokio::time::sleep_until(instant).await              // Sleep until specific time
tokio::time::interval(duration)                      // Create ticker
let mut interval = tokio::time::interval(Duration::from_secs(1));
interval.tick().await                                // Wait for next tick

// Timeout
tokio::time::timeout(duration, future).await         // Timeout future, returns Result
tokio::time::timeout_at(instant, future).await       // Timeout at instant

// Channels (async)
let (tx, rx) = mpsc::channel(capacity)              // Bounded MPSC channel
let (tx, rx) = mpsc::unbounded_channel()            // Unbounded MPSC channel
tx.send(value).await                                 // Send, async, returns Result
rx.recv().await                                      // Receive, returns Option<T>
tx.try_send(value)                                   // Non-blocking send
rx.try_recv()                                        // Non-blocking receive

let (tx, rx) = oneshot::channel()                   // One-shot channel
tx.send(value)                                       // Send once (not async)
rx.await                                             // Receive once

let (tx, rx) = broadcast::channel(capacity)         // Broadcast channel
tx.send(value)                                       // Send to all receivers
rx.recv().await                                      // Receive broadcast message

let (tx, rx) = watch::channel(initial)              // Watch channel (shared state)
tx.send(value)                                       // Update watched value
rx.borrow()                                          // Read current value
rx.changed().await                                   // Wait for change

// Synchronization primitives
let mutex = Mutex::new(data)                         // Async Mutex
let guard = mutex.lock().await                       // Lock asynchronously
let guard = mutex.try_lock()                         // Try lock (non-async)

let rw = RwLock::new(data)                          // Async RwLock
let read = rw.read().await                           // Read lock
let write = rw.write().await                         // Write lock

let semaphore = Semaphore::new(n)                    // Semaphore with n permits
let permit = semaphore.acquire().await.unwrap()      // Acquire permit
let permit = semaphore.try_acquire()                 // Try acquire (non-async)
semaphore.add_permits(n)                             // Add permits

let barrier = Barrier::new(n)                        // Async barrier
barrier.wait().await                                 // Wait for n tasks

let notify = Notify::new()                           // Async notification
notify.notified().await                              // Wait for notification
notify.notify_one()                                  // Notify one waiter
notify.notify_waiters()                              // Notify all waiters

// Select (wait on multiple futures)
tokio::select! {
    result1 = future1 => { /* handle */ },
    result2 = future2 => { /* handle */ },
    else => { /* no future ready */ }
}

// Join (wait for all)
tokio::join!(future1, future2, future3)             // Wait for all, return tuple
let (r1, r2) = tokio::join!(f1, f2);

// Try join (wait for all, propagate errors)
tokio::try_join!(future1, future2)                  // Return Result of tuple

// Pin & unpin
futures::pin_mut!(future)                            // Pin to stack
Box::pin(future)                                     // Pin to heap

// Stream operations (requires tokio-stream crate)
use tokio_stream::StreamExt;
stream.next().await                                  // Get next item
stream.collect::<Vec<_>>().await                    // Collect all items
stream.for_each(|item| async { /* work */ }).await // Process each item
stream.map(|x| x * 2)                               // Transform items
stream.filter(|x| future_returns_bool)              // Filter items
stream.take(n)                                       // Take first n items
stream.skip(n)                                       // Skip first n items

// File I/O (async)
use tokio::fs;
fs::read("file.txt").await                           // Read entire file
fs::write("file.txt", data).await                    // Write entire file
fs::read_to_string("file.txt").await                // Read as string
let file = fs::File::open("file.txt").await         // Open file
let mut file = fs::File::create("file.txt").await   // Create file

use tokio::io::{AsyncReadExt, AsyncWriteExt};
file.read_to_end(&mut buf).await                    // Read to buffer
file.write_all(data).await                           // Write all bytes
file.read_exact(&mut buf).await                     // Read exact amount

// Network I/O
use tokio::net::{TcpListener, TcpStream};
let listener = TcpListener::bind("127.0.0.1:8080").await // Bind TCP listener
let (socket, addr) = listener.accept().await        // Accept connection
let stream = TcpStream::connect("127.0.0.1:8080").await // Connect TCP

stream.readable().await                              // Wait until readable
stream.writable().await                              // Wait until writable
stream.read(&mut buf).await                          // Read from socket
stream.write_all(data).await                         // Write to socket

// UDP
use tokio::net::UdpSocket;
let socket = UdpSocket::bind("127.0.0.1:8080").await // Bind UDP
socket.send_to(data, addr).await                     // Send datagram
socket.recv_from(&mut buf).await                     // Receive datagram

// Process spawning
use tokio::process::Command;
let output = Command::new("ls").output().await       // Run and get output
let child = Command::new("ls").spawn()               // Spawn child process
child.wait().await                                   // Wait for child

// Signal handling
use tokio::signal;
signal::ctrl_c().await                               // Wait for Ctrl+C

// Graceful shutdown pattern
let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
tokio::select! {
    _ = server_task => {},
    _ = shutdown_rx.recv() => {
        // cleanup
    }
}

// Retry pattern with backoff
use tokio::time::{sleep, Duration};
let mut retries = 0;
loop {
    match operation().await {
        Ok(result) => break result,
        Err(e) if retries < 3 => {
            retries += 1;
            sleep(Duration::from_secs(2u64.pow(retries))).await;
        }
        Err(e) => return Err(e),
    }
}

// Common patterns
let shared = Arc::new(Mutex::new(data));             // Shared async state
let clone = Arc::clone(&shared);
tokio::spawn(async move {
    let mut guard = clone.lock().await;
    *guard += 1;
});

// Fan-out pattern (spawn multiple tasks)
let handles: Vec<_> = (0..10)
    .map(|i| tokio::spawn(async move { work(i).await }))
    .collect();
for handle in handles {
    handle.await.unwrap();
}

// Concurrent stream processing
use futures::stream::{self, StreamExt};
stream::iter(items)
    .for_each_concurrent(10, |item| async move {
        process(item).await;
    })
    .await;
```