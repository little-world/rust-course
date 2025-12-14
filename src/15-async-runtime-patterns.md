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
This example walks through the basics of future composition with map, highlighting each step so you can reuse the pattern.

```rust

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

```

### Example: Chaining async operations
This example shows chaining async operations to illustrate where the pattern fits best.

```rust

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

```

### Example: Error conversion and propagation
This example shows how to error conversion and propagation in practice, emphasizing why it works.

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
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(AppError::NotFound);
    }

    let data = response.json().await?;
    Ok(data)
}

```

### Example: Combining multiple futures with different error types
This example shows combining multiple futures with different error types to illustrate where the pattern fits best.

```rust

use futures::future::TryFutureExt;

async fn complex_operation() -> Result<String, AppError> {
    let data1 = fetch_json_data("https://api.example.com/data1")
        .await?;

    let data2 = fetch_json_data("https://api.example.com/data2")
        .await?;

    // Process both results
    Ok(format!("Combined: {:?} and {:?}", data1, data2))
}
// Real-world: HTTP client with retries
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

```

### Example: join! - wait for all futures
This example shows how to use join! to wait for all futures without over-synchronizing.

```rust

async fn concurrent_fetch() {
    let (result1, result2, result3) = tokio::join!(
        fetch_user_name(1),
        fetch_user_name(2),
        fetch_user_name(3),
    );

    println!("Results: {:?}, {:?}, {:?}", result1, result2, result3);
}

```

### Example: try_join! - wait for all, fail fast on error
This example shows how to use try_join! to wait for all, fail fast on error without over-synchronizing.

```rust

async fn concurrent_fetch_fail_fast() -> Result<(String, String, String), String> {
    tokio::try_join!(
        fetch_user_name(1),
        fetch_user_name(2),
        fetch_user_name(3),
    )
}

```

### Example: select! - race futures, take first to complete
This example shows how to use select! to race futures, take first to complete without over-synchronizing.

```rust

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

```

### Example: Dynamic number of futures with FuturesUnordered
This example shows how to dynamic number of futures with FuturesUnordered in practice, emphasizing why it works.

```rust

use futures::stream::{FuturesUnordered, StreamExt};

async fn fetch_all_users(user_ids: Vec<u64>) -> Vec<Result<String, String>> {
    let futures: FuturesUnordered<_> = user_ids
        .into_iter()
        .map(|id| fetch_user_name(id))
        .collect();

    futures.collect().await
}
// Real-world: Parallel HTTP requests with limit
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
// Real-world: Timeout wrapper
async fn with_timeout<F, T>(
    future: F,
    duration: Duration,
) -> Result<T, tokio::time::error::Elapsed>
where
    F: std::future::Future<Output = T>,
{
    tokio::time::timeout(duration, future).await
}
// Real-world: Cancellation-safe operations
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

**Problem**: Processing infinite or unbounded sequences (websocket messages, sensor data, log streams) with standard iterators blocks thread. Collecting entire stream into Vec before processing wastes memory for large datasets.

**Solution**: Use `Stream` trait (async iterator) to yield values over time without blocking. Apply stream combinators: `.map()`, `.filter()`, `.fold()`, `.buffered()` for transformations.

**Why It Matters**: Streams enable processing data larger than memory—GB log file analyzed in constant memory. WebSocket connections handle millions of messages without collecting all.

**Use Cases**: WebSocket message processing, sensor data aggregation, log file streaming, database query result streaming, event sourcing, pub-sub systems, real-time analytics, infinite data sources.

### Example: Stream Combinators

 Process async sequences of data with transformations, filtering, and aggregation.

```rust
use tokio;
use tokio_stream::{self as stream, StreamExt};
use std::time::Duration;

```

### Example: Creating streams
This example shows creating streams to illustrate where the pattern fits best.

```rust

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

```

### Example: Map and filter
This example shows how to map and filter in practice, emphasizing why it works.

```rust

async fn transform_stream() {
    let stream = stream::iter(1..=10)
        .filter(|x| x % 2 == 0)
        .map(|x| x * 2);

    let results: Vec<i32> = stream.collect().await;
    println!("Transformed: {:?}", results);
}

```

### Example: Then (async map)
This example shows how to then (async map) in practice, emphasizing why it works.

```rust

async fn async_transform_stream() {
    let stream = stream::iter(1..=5)
        .then(|x| async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            x * x
        });

    let results: Vec<i32> = stream.collect().await;
    println!("Async transformed: {:?}", results);
}

```

### Example: Fold and reduce
This example shows how to fold and reduce in practice, emphasizing why it works.

```rust

async fn aggregate_stream() {
    let sum = stream::iter(1..=100)
        .fold(0, |acc, x| acc + x)
        .await;

    println!("Sum: {}", sum);
}

```

### Example: Take and skip
This example shows how to take and skip in practice, emphasizing why it works.

```rust

async fn limit_stream() {
    let results: Vec<i32> = stream::iter(1..=100)
        .skip(10)
        .take(5)
        .collect()
        .await;

    println!("Limited: {:?}", results);
}
// Real-world: Rate limiting
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
// Real-world: Batch processing
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
// Real-world: Stream merging
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

```

### Example: Manual stream implementation
This example shows how to manual stream implementation while calling out the practical trade-offs.

```rust

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

```

### Example: Async generator pattern using channels
This example shows how to async generator pattern using channels in practice, emphasizing why it works.

```rust

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
// Real-world: File watcher stream
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
// Real-world: Database query result stream
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

```

### Example: Interval-based stream
This example shows interval-based stream to illustrate where the pattern fits best.

```rust

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
This example walks through the basics of task spawning, highlighting each step so you can reuse the pattern.

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

```

### Example: Structured concurrency with JoinSet
This example shows structured concurrency with JoinSet to illustrate where the pattern fits best.

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

```

### Example: Scoped tasks (guaranteed completion before scope ends)
This example shows scoped tasks (guaranteed completion before scope ends) to illustrate where the pattern fits best.

```rust

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

```

### Example: Task cancellation
This example shows how to task cancellation in practice, emphasizing why it works.

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
// Real-world: Worker pool pattern
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
// Real-world: Supervisor pattern (restart on failure)
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
// Real-world: Background task with graceful shutdown
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

```

### Example: Result propagation with ?
This example shows how to result propagation with ? in practice, emphasizing why it works.

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

```

### Example: Retry with exponential backoff
This example shows how to retry with exponential backoff in practice, emphasizing why it works.

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

```

### Example: Circuit breaker
This example shows how to circuit breaker in practice, emphasizing why it works.

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

```

### Example: Fallback pattern
This example shows how to fallback pattern in practice, emphasizing why it works.

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
// Real-world: Bulkhead pattern (resource isolation)
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

**Problem**: Waiting indefinitely for async operations causes hangs—network request that never responds blocks forever. Need to handle whichever of multiple operations completes first (user input vs network response).

**Solution**: Use `tokio::select!` to race multiple futures, completing when first finishes. Use `tokio::time::timeout()` to bound operation duration.

**Why It Matters**: Timeouts prevent resource leaks from hung operations—HTTP server without timeouts accumulates connections from slow clients until memory exhausted. Select enables responsive UIs: user input cancels background computation immediately.

**Use Cases**: HTTP clients (request timeouts), connection management (idle timeouts), health checks (periodic pings), graceful shutdown (timeout on cleanup), rate limiting (interval-based), user cancellation (input vs background work), circuit breakers.

### Example: Select Patterns

Wait on multiple async operations and react to whichever completes first.


```rust
use tokio;
use tokio::sync::mpsc;
use std::time::Duration;

```

### Example: Basic select with two branches
This example walks through the basics of select with two branches, highlighting each step so you can reuse the pattern.

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

```

### Example: Select in a loop
This example shows how to select in a loop in practice, emphasizing why it works.

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

```

### Example: Biased select (priority)
This example shows biased select (priority) to illustrate where the pattern fits best.

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
// Real-world: Request with cancellation
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
// Real-world: Server with shutdown signal
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
This example shows how to select with default (non-blocking) in practice, emphasizing why it works.

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

```

### Example: Basic timeout
This example walks through the basics of timeout, highlighting each step so you can reuse the pattern.

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

```

### Example: Timeout with retry
This example shows how to timeout with retry in practice, emphasizing why it works.

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

```

### Example: Deadline tracking
This example shows how to deadline tracking in practice, emphasizing why it works.

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

```

### Example: Timeout for multiple operations
This example shows how to timeout for multiple operations in practice, emphasizing why it works.

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
// Real-world: Rate limiter with timeout
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
// Real-world: Health check with timeout
async fn health_check(url: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let check = async {
        let response = reqwest::get(url).await?;
        Ok(response.status().is_success())
    };

    timeout(Duration::from_secs(5), check)
        .await
        .map_err(|_| "Health check timed out".into())?
}
// Real-world: Graceful timeout (finish current work)
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

**Problem**: Choosing wrong async runtime impacts performance, features, and maintainability. Tokio dominates ecosystem but isn't always best choice.

**Solution**: Use Tokio for general-purpose applications: mature, full-featured, excellent ecosystem. Use async-std for simpler API, closer to std library patterns.

**Why It Matters**: Runtime choice determines ecosystem access—Tokio has 10x more compatible libraries than alternatives. Performance varies: work-stealing vs single-threaded, epoll vs io_uring.

**Use Cases**: Tokio for web servers, databases, general applications. async-std for learning, simpler projects. smol for single-threaded, minimal overhead. embassy for embedded systems, bare-metal. Runtime-agnostic libraries for maximum compatibility.

### Example: Tokio Runtime Features

Understand Tokio's features and how to configure them for different workloads.

```rust
use tokio;
use std::time::Duration;

```

### Example: Multi-threaded runtime (default)
This example shows multi-threaded runtime (default) to illustrate where the pattern fits best.

```rust

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

```

### Example: Single-threaded runtime
This example shows single-threaded runtime to illustrate where the pattern fits best.

```rust

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

```

### Example: Custom runtime configuration
This example shows how to custom runtime configuration while calling out the practical trade-offs.

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

```

### Example: Blocking operations
This example shows blocking operations to illustrate where the pattern fits best.

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

```

### Example: Local task set (for !Send futures)
This example shows how to local task set (for !Send futures) in practice, emphasizing why it works.

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
// Real-world: CPU-bound work with rayon
use rayon::prelude::*;

async fn cpu_bound_with_rayon() {
    let numbers: Vec<u64> = (0..1_000_000).collect();

    let sum = tokio::task::spawn_blocking(move || {
        numbers.par_iter().sum::<u64>()
    }).await.unwrap();

    println!("Sum: {}", sum);
}
// Real-world: Mixed workload (I/O and CPU)
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
// async-std version
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
// Runtime-agnostic code using futures
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
// Feature comparison
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
// Performance comparison example
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
// Interop: using futures crate for compatibility
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
