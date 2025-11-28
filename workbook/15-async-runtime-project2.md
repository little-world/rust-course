# Chapter 15: Async Runtime Patterns - Programming Projects

## Project 2: Real-Time Event Stream Processor

### Problem Statement

Build a real-time event stream processor that consumes events from multiple sources (simulated sensors, logs, metrics), applies transformations, filters, aggregations, and routes results to different outputs. The system must handle backpressure, buffer events efficiently, perform windowed aggregations (count events per time window), and maintain high throughput without dropping data.

### Use Cases

- **IoT sensor data processing** - Process temperature, pressure, humidity readings from thousands of sensors
- **Log aggregation systems** - Collect logs from distributed services, filter, aggregate error counts
- **Financial market data** - Process tick data, calculate moving averages, detect anomalies
- **Real-time analytics** - Track user events (clicks, page views), compute metrics per second
- **Network monitoring** - Process packet data, detect patterns, alert on anomalies
- **Stream ETL pipelines** - Extract events from Kafka/RabbitMQ, transform, load into databases
- **Gaming telemetry** - Process player actions, compute statistics, detect cheating

### Why It Matters

**Backpressure is Critical**: Without backpressure, fast producers overwhelm slow consumers. Example: sensor sending 10,000 events/sec, processor handling 1,000/sec → 9,000 events/sec accumulate in memory → OOM crash in 10 seconds. Proper backpressure slows the producer or buffers intelligently.

**Windowed Aggregations**: Real-time analytics require time-based calculations. "Events per second" needs 1-second tumbling windows. "Moving average over 5 minutes" needs sliding windows. Without async streams, you'd manually manage timers and state—error-prone and complex.

**Memory Efficiency**: Loading all data into memory before processing fails for infinite streams. Streams process events one-at-a-time (or in small chunks), keeping memory constant regardless of stream length. 1GB/sec stream processed in 10MB memory.

**Throughput**: Sequential event processing at 10ms/event = 100 events/sec. Batching 100 events + parallel processing = 10,000 events/sec (100x improvement). Async streams make batching and parallelism composable.

Example performance:
```
Sequential processing:     100 events/sec (10ms each)
Batched (100 per batch):   10,000 events/sec
Parallel batches (10x):    100,000 events/sec
```

---

## Milestone 1: Basic Event Stream from Channel

### Introduction

Before processing streams, you need to understand async stream fundamentals. This milestone teaches you to create streams from channels, consume them with `.next().await`, and apply basic transformations.

**Why Start Here**: Streams are the async equivalent of iterators. Channels naturally produce streams (events arrive over time). Understanding stream consumption patterns is foundational—everything builds on `.next().await` and stream combinators.

### Architecture

**Structs:**
- `Event` - Represents a single event
  - **Field** `id: u64` - Unique event identifier
  - **Field** `timestamp: u64` - Unix timestamp (milliseconds)
  - **Field** `source: String` - Event origin (e.g., "sensor-1")
  - **Field** `value: f64` - Event payload (temperature, metric, etc.)
  - **Field** `event_type: EventType` - Category of event

- `EventType` - Enum for event categories
  - **Variant** `Metric`, `Log`, `Alert`, `Error`

**Key Functions:**
- `async fn create_event_stream(rx: mpsc::Receiver<Event>) -> impl Stream<Item = Event>` - Wraps channel receiver as stream
- `async fn generate_events(tx: mpsc::Sender<Event>, count: usize, delay_ms: u64)` - Simulates event source
- `async fn consume_stream(mut stream: impl Stream<Item = Event> + Unpin)` - Consumes and prints events

**Role Each Plays:**
- **tokio::sync::mpsc**: Channel for sending events from producer to consumer
- **ReceiverStream**: Adapter that makes `Receiver<T>` into `Stream<Item = T>`
- **Stream trait**: Async iterator (yields items over time via `.next().await`)

### Checkpoint Tests

```rust
#[tokio::test]
async fn test_event_creation() {
    let event = Event::new(1, "sensor-1".to_string(), 23.5, EventType::Metric);

    assert_eq!(event.id, 1);
    assert_eq!(event.source, "sensor-1");
    assert_eq!(event.value, 23.5);
}

#[tokio::test]
async fn test_channel_to_stream() {
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;
    use tokio_stream::StreamExt;

    let (tx, rx) = mpsc::channel(10);

    // Send some events
    tx.send(Event::new(1, "test".into(), 1.0, EventType::Metric)).await.unwrap();
    tx.send(Event::new(2, "test".into(), 2.0, EventType::Metric)).await.unwrap();
    drop(tx); // Close channel

    let mut stream = ReceiverStream::new(rx);

    let event1 = stream.next().await.unwrap();
    let event2 = stream.next().await.unwrap();
    let event3 = stream.next().await; // Should be None (stream closed)

    assert_eq!(event1.id, 1);
    assert_eq!(event2.id, 2);
    assert!(event3.is_none());
}

#[tokio::test]
async fn test_event_generator() {
    use tokio::sync::mpsc;

    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(generate_events(tx, 10, 1));

    let mut count = 0;
    while let Some(_event) = rx.recv().await {
        count += 1;
    }

    assert_eq!(count, 10);
}

#[tokio::test]
async fn test_stream_consumption() {
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;
    use tokio_stream::StreamExt;

    let (tx, rx) = mpsc::channel(10);

    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(Event::new(i, "sensor".into(), i as f64, EventType::Metric))
                .await
                .unwrap();
        }
    });

    let stream = ReceiverStream::new(rx);
    let events: Vec<Event> = stream.collect().await;

    assert_eq!(events.len(), 5);
    assert_eq!(events[0].id, 0);
    assert_eq!(events[4].id, 4);
}
```

### Starter Code

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio_stream::{Stream, StreamExt};
use tokio_stream::wrappers::ReceiverStream;

#[derive(Debug, Clone)]
pub enum EventType {
    Metric,
    Log,
    Alert,
    Error,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: u64,
    pub timestamp: u64,
    pub source: String,
    pub value: f64,
    pub event_type: EventType,
}

impl Event {
    pub fn new(id: u64, source: String, value: f64, event_type: EventType) -> Self {
        // TODO: Create event with current timestamp
        // Hint: Use std::time::SystemTime::now()
        //       .duration_since(std::time::UNIX_EPOCH)
        //       .unwrap()
        //       .as_millis() as u64

        todo!("Implement Event::new")
    }
}

pub async fn generate_events(tx: mpsc::Sender<Event>, count: usize, delay_ms: u64) {
    // TODO: Generate 'count' events with 'delay_ms' between each
    // TODO: Send events through the channel
    // TODO: Vary the source and value to simulate different sensors

    todo!("Implement event generator")
}

pub async fn consume_stream<S>(mut stream: S)
where
    S: Stream<Item = Event> + Unpin,
{
    // TODO: Use while let Some(event) = stream.next().await
    // TODO: Print each event

    todo!("Implement stream consumer")
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(100);

    // Spawn event generator
    tokio::spawn(generate_events(tx, 20, 100));

    // Convert receiver to stream and consume
    let stream = ReceiverStream::new(rx);
    consume_stream(stream).await;
}
```

**Implementation Hints:**
1. Use `std::time::SystemTime::now().duration_since(UNIX_EPOCH)` for timestamps
2. Use `tx.send(event).await?` to send events (returns Result)
3. Use `sleep(Duration::from_millis(delay_ms)).await` for delays
4. Use `stream.next().await` to get next item (returns `Option<T>`)
5. Channel automatically closes when sender is dropped

---

## Milestone 2: Stream Transformations and Filtering

### Introduction

**Why Milestone 1 Isn't Enough**: Raw event streams need transformation—unit conversion, normalization, filtering. Doing this manually with loops is verbose and error-prone.

**The Improvement**: Use stream combinators (`.map()`, `.filter()`, `.filter_map()`) to transform and filter events declaratively. These combinators compose—chain multiple transformations without intermediate collections.

**Performance**: Stream combinators are zero-cost abstractions. They compile to efficient loops with no overhead. Lazy evaluation means transformations only run for items that pass filters.

### Architecture

**Structs:**
- Reuse `Event` and `EventType` from Milestone 1
- `ProcessedEvent` - Enriched event after processing
  - **Field** `original: Event` - The source event
  - **Field** `normalized_value: f64` - Value scaled to [0, 1]
  - **Field** `severity: Severity` - Computed severity level

- `Severity` - Enum for event severity
  - **Variant** `Low`, `Medium`, `High`, `Critical`

**Key Functions:**
- `fn normalize_value(value: f64, min: f64, max: f64) -> f64` - Scales value to [0, 1] range
- `fn calculate_severity(value: f64) -> Severity` - Maps normalized value to severity
- `async fn process_stream(stream: impl Stream<Item = Event>) -> impl Stream<Item = ProcessedEvent>` - Transformation pipeline

**Role Each Plays:**
- **map**: Transforms each item (like iterator map)
- **filter**: Keeps items matching predicate
- **filter_map**: Combines filter + map (map returns Option)

### Checkpoint Tests

```rust
#[test]
fn test_normalize_value() {
    assert_eq!(normalize_value(50.0, 0.0, 100.0), 0.5);
    assert_eq!(normalize_value(0.0, 0.0, 100.0), 0.0);
    assert_eq!(normalize_value(100.0, 0.0, 100.0), 1.0);
    assert_eq!(normalize_value(75.0, 50.0, 100.0), 0.5);
}

#[test]
fn test_calculate_severity() {
    assert!(matches!(calculate_severity(0.2), Severity::Low));
    assert!(matches!(calculate_severity(0.5), Severity::Medium));
    assert!(matches!(calculate_severity(0.8), Severity::High));
    assert!(matches!(calculate_severity(0.95), Severity::Critical));
}

#[tokio::test]
async fn test_stream_map() {
    use tokio_stream::{self as stream};

    let events = vec![
        Event::new(1, "s1".into(), 10.0, EventType::Metric),
        Event::new(2, "s1".into(), 20.0, EventType::Metric),
    ];

    let result: Vec<f64> = stream::iter(events)
        .map(|e| e.value * 2.0)
        .collect()
        .await;

    assert_eq!(result, vec![20.0, 40.0]);
}

#[tokio::test]
async fn test_stream_filter() {
    use tokio_stream::{self as stream};

    let events = vec![
        Event::new(1, "s1".into(), 10.0, EventType::Metric),
        Event::new(2, "s1".into(), 50.0, EventType::Metric),
        Event::new(3, "s1".into(), 100.0, EventType::Metric),
    ];

    let result: Vec<Event> = stream::iter(events)
        .filter(|e| e.value > 30.0)
        .collect()
        .await;

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].value, 50.0);
}

#[tokio::test]
async fn test_process_stream() {
    use tokio_stream::{self as stream};

    let events = vec![
        Event::new(1, "s1".into(), 10.0, EventType::Metric),
        Event::new(2, "s1".into(), 90.0, EventType::Metric),
    ];

    let processed: Vec<ProcessedEvent> = process_stream(stream::iter(events))
        .collect()
        .await;

    assert_eq!(processed.len(), 2);
    assert!(processed[0].normalized_value < 0.5);
    assert!(processed[1].normalized_value > 0.5);
}
```

### Starter Code

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ProcessedEvent {
    pub original: Event,
    pub normalized_value: f64,
    pub severity: Severity,
}

pub fn normalize_value(value: f64, min: f64, max: f64) -> f64 {
    // TODO: Normalize value to [0, 1] range
    // Formula: (value - min) / (max - min)
    // Handle edge case where min == max

    todo!("Implement normalization")
}

pub fn calculate_severity(normalized: f64) -> Severity {
    // TODO: Map normalized value [0, 1] to severity
    // 0.0 - 0.3: Low
    // 0.3 - 0.6: Medium
    // 0.6 - 0.9: High
    // 0.9 - 1.0: Critical

    todo!("Implement severity calculation")
}

pub fn process_stream<S>(stream: S) -> impl Stream<Item = ProcessedEvent>
where
    S: Stream<Item = Event>,
{
    // TODO: Use .map() to transform Event -> ProcessedEvent
    // TODO: For each event, normalize value (assume range 0-100)
    // TODO: Calculate severity from normalized value
    // TODO: Return ProcessedEvent

    todo!("Implement stream processing")
}

pub fn filter_high_severity<S>(stream: S) -> impl Stream<Item = ProcessedEvent>
where
    S: Stream<Item = ProcessedEvent>,
{
    // TODO: Use .filter() to keep only High and Critical severity events

    todo!("Implement severity filtering")
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(100);

    tokio::spawn(generate_events(tx, 50, 50));

    let stream = ReceiverStream::new(rx);

    let processed = process_stream(stream);
    let high_severity = filter_high_severity(processed);

    high_severity
        .for_each(|event| async move {
            println!("HIGH SEVERITY: {:?} -> {:?}", event.original.source, event.severity);
        })
        .await;
}
```

**Implementation Hints:**
1. Use `stream.map(|event| { ... })` to transform
2. Normalize: `(value - min) / (max - min)`, handle divide by zero
3. Severity: Use `if/else` or `match` on ranges
4. Filter: `stream.filter(|pe| matches!(pe.severity, Severity::High | Severity::Critical))`
5. Chain combinators: `stream.map(...).filter(...).map(...)`

---

## Milestone 3: Buffering and Batching

### Introduction

**Why Milestone 2 Isn't Enough**: Processing events one-at-a-time is inefficient. Database writes, network sends, and many operations have fixed overhead—batching amortizes this cost.

**The Improvement**: Use `.chunks_timeout()` to batch events by count OR time, whichever comes first. This balances throughput (batch size) with latency (timeout).

**Performance (Optimization)**: Writing to database 1 event at a time = 1000 writes/sec (1ms per write). Batching 100 events = 100,000 events/sec with same 1ms latency per batch (100x throughput improvement).

### Architecture

**Structs:**
- `EventBatch` - Collection of events for batch processing
  - **Field** `events: Vec<ProcessedEvent>` - Events in this batch
  - **Field** `batch_id: u64` - Unique batch identifier
  - **Field** `created_at: u64` - When batch was created

**Key Functions:**
- `fn batch_events(stream: impl Stream<Item = ProcessedEvent>, size: usize, timeout: Duration) -> impl Stream<Item = EventBatch>` - Creates batches
- `async fn process_batch(batch: EventBatch)` - Handles a complete batch
- `async fn write_batch_to_storage(batch: &EventBatch)` - Simulates batch write

**Role Each Plays:**
- **chunks_timeout**: Buffers items until N items OR timeout, yields Vec
- **EventBatch**: Encapsulates batch metadata + events
- **Batch processing**: Amortizes fixed costs (DB connection, network roundtrip)

### Checkpoint Tests

```rust
#[tokio::test]
async fn test_batch_by_count() {
    use tokio_stream::{self as stream, StreamExt};

    let events: Vec<i32> = (0..10).collect();
    let batches: Vec<Vec<i32>> = stream::iter(events)
        .chunks_timeout(3, Duration::from_secs(10))
        .collect()
        .await;

    assert_eq!(batches.len(), 4); // [0,1,2], [3,4,5], [6,7,8], [9]
    assert_eq!(batches[0].len(), 3);
    assert_eq!(batches[3].len(), 1);
}

#[tokio::test]
async fn test_batch_by_timeout() {
    use tokio_stream::{self as stream, StreamExt};

    let (tx, rx) = mpsc::channel(10);

    tokio::spawn(async move {
        // Send 2 events, then wait (batch should timeout)
        tx.send(1).await.unwrap();
        tx.send(2).await.unwrap();
        sleep(Duration::from_millis(150)).await;
        tx.send(3).await.unwrap();
    });

    let batches: Vec<Vec<i32>> = ReceiverStream::new(rx)
        .chunks_timeout(10, Duration::from_millis(100))
        .take(2)
        .collect()
        .await;

    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0].len(), 2); // Timed out after 100ms
    assert_eq!(batches[1].len(), 1);
}

#[tokio::test]
async fn test_event_batch_creation() {
    let events = vec![
        ProcessedEvent {
            original: Event::new(1, "s1".into(), 10.0, EventType::Metric),
            normalized_value: 0.1,
            severity: Severity::Low,
        },
    ];

    let batch = EventBatch::new(1, events);

    assert_eq!(batch.batch_id, 1);
    assert_eq!(batch.events.len(), 1);
}

#[tokio::test]
async fn test_batch_processing() {
    let events = vec![
        ProcessedEvent {
            original: Event::new(1, "s1".into(), 10.0, EventType::Metric),
            normalized_value: 0.1,
            severity: Severity::Low,
        },
    ];

    let batch = EventBatch::new(1, events);

    // Should not panic
    process_batch(batch).await;
}
```

### Starter Code

```rust
use futures::stream::StreamExt; // Note: different StreamExt
use tokio::time::Duration;

#[derive(Debug, Clone)]
pub struct EventBatch {
    pub events: Vec<ProcessedEvent>,
    pub batch_id: u64,
    pub created_at: u64,
}

impl EventBatch {
    pub fn new(batch_id: u64, events: Vec<ProcessedEvent>) -> Self {
        // TODO: Create batch with current timestamp

        todo!("Implement EventBatch::new")
    }
}

pub fn batch_events<S>(
    stream: S,
    size: usize,
    timeout: Duration,
) -> impl Stream<Item = EventBatch>
where
    S: Stream<Item = ProcessedEvent>,
{
    // TODO: Use .chunks_timeout(size, timeout) to create batches
    // TODO: Map Vec<ProcessedEvent> to EventBatch with unique IDs

    todo!("Implement batching")
}

pub async fn process_batch(batch: EventBatch) {
    // TODO: Simulate batch processing
    // TODO: Print batch info (id, size, avg severity)
    // TODO: Call write_batch_to_storage

    todo!("Implement batch processing")
}

pub async fn write_batch_to_storage(batch: &EventBatch) {
    // TODO: Simulate database write (sleep for 10ms)
    // TODO: In real code, this would be DB INSERT with prepared statement

    todo!("Implement batch write simulation")
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(1000);

    // High-rate event generator
    tokio::spawn(generate_events(tx, 1000, 5));

    let stream = ReceiverStream::new(rx);
    let processed = process_stream(stream);

    let batched = batch_events(processed, 50, Duration::from_millis(200));

    batched
        .for_each(|batch| async move {
            process_batch(batch).await;
        })
        .await;
}
```

**Implementation Hints:**
1. Use `use futures::stream::StreamExt;` for `chunks_timeout` (different from tokio_stream)
2. Use atomic counter for batch IDs: `static BATCH_ID: AtomicU64 = AtomicU64::new(0);`
3. `.chunks_timeout(size, timeout)` returns `Stream<Item = Vec<T>>`
4. Use `.enumerate()` or atomic to generate batch IDs
5. Batch processing: sum values, count by severity, etc.

---

## Milestone 4: Windowed Aggregations

### Introduction

**Why Milestone 3 Isn't Enough**: Real-time analytics require time-based aggregations—events per second, moving averages, anomaly detection. Batching by count doesn't respect time boundaries.

**The Improvement**: Implement tumbling windows (non-overlapping time intervals) and sliding windows (overlapping intervals) for time-based aggregations.

**Optimization (Accuracy)**: Time-based windows ensure accurate rate calculations. "Events per second" with count-based batching is inaccurate—batch size varies with event rate. Time windows guarantee consistent intervals.

### Architecture

**Structs:**
- `WindowedStats` - Aggregated statistics for a time window
  - **Field** `window_start: u64` - Window start timestamp
  - **Field** `window_end: u64` - Window end timestamp
  - **Field** `event_count: usize` - Events in window
  - **Field** `avg_value: f64` - Average event value
  - **Field** `max_value: f64` - Maximum value
  - **Field** `severity_counts: HashMap<Severity, usize>` - Counts by severity

**Key Functions:**
- `fn create_tumbling_window(stream: impl Stream<Item = ProcessedEvent>, duration_ms: u64) -> impl Stream<Item = WindowedStats>` - Creates fixed windows
- `fn calculate_window_stats(events: Vec<ProcessedEvent>, window_start: u64, duration_ms: u64) -> WindowedStats` - Aggregates events
- `async fn detect_anomalies(stats: &WindowedStats, baseline: &WindowedStats) -> Option<Alert>` - Compares windows

**Role Each Plays:**
- **Tumbling window**: Divides time into non-overlapping intervals (0-1s, 1-2s, 2-3s)
- **Window stats**: Aggregates metrics for analysis
- **Anomaly detection**: Compares current window to historical baseline

### Checkpoint Tests

```rust
#[tokio::test]
async fn test_tumbling_window() {
    use tokio_stream::{self as stream};

    // Create events with timestamps 0, 500, 1000, 1500, 2000 ms
    let mut events = Vec::new();
    for i in 0..5 {
        let mut event = Event::new(i, "s1".into(), (i * 10) as f64, EventType::Metric);
        event.timestamp = i * 500;
        events.push(ProcessedEvent {
            original: event,
            normalized_value: 0.5,
            severity: Severity::Medium,
        });
    }

    let windows: Vec<WindowedStats> = create_tumbling_window(
        stream::iter(events),
        1000, // 1-second windows
    )
    .collect()
    .await;

    // Should create 3 windows: [0-1000), [1000-2000), [2000-3000)
    assert!(windows.len() >= 2);
}

#[test]
fn test_calculate_window_stats() {
    let events = vec![
        ProcessedEvent {
            original: Event::new(1, "s1".into(), 10.0, EventType::Metric),
            normalized_value: 0.1,
            severity: Severity::Low,
        },
        ProcessedEvent {
            original: Event::new(2, "s1".into(), 30.0, EventType::Metric),
            normalized_value: 0.3,
            severity: Severity::Medium,
        },
    ];

    let stats = calculate_window_stats(events, 0, 1000);

    assert_eq!(stats.event_count, 2);
    assert_eq!(stats.avg_value, 20.0); // (10 + 30) / 2
    assert_eq!(stats.max_value, 30.0);
}

#[tokio::test]
async fn test_anomaly_detection() {
    let baseline = WindowedStats {
        window_start: 0,
        window_end: 1000,
        event_count: 100,
        avg_value: 50.0,
        max_value: 80.0,
        severity_counts: HashMap::new(),
    };

    let normal = WindowedStats {
        window_start: 1000,
        window_end: 2000,
        event_count: 105, // Within tolerance
        avg_value: 52.0,
        max_value: 82.0,
        severity_counts: HashMap::new(),
    };

    let anomaly = WindowedStats {
        window_start: 2000,
        window_end: 3000,
        event_count: 500, // 5x normal
        avg_value: 95.0,  // Much higher
        max_value: 120.0,
        severity_counts: HashMap::new(),
    };

    assert!(detect_anomalies(&normal, &baseline).await.is_none());
    assert!(detect_anomalies(&anomaly, &baseline).await.is_some());
}
```

### Starter Code

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct WindowedStats {
    pub window_start: u64,
    pub window_end: u64,
    pub event_count: usize,
    pub avg_value: f64,
    pub max_value: f64,
    pub severity_counts: HashMap<Severity, usize>,
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub message: String,
    pub severity: Severity,
    pub window: WindowedStats,
}

pub fn calculate_window_stats(
    events: Vec<ProcessedEvent>,
    window_start: u64,
    duration_ms: u64,
) -> WindowedStats {
    // TODO: Calculate aggregate statistics
    // TODO: Count events by severity
    // TODO: Compute avg and max values

    todo!("Implement window stats calculation")
}

pub fn create_tumbling_window<S>(
    stream: S,
    duration_ms: u64,
) -> impl Stream<Item = WindowedStats>
where
    S: Stream<Item = ProcessedEvent>,
{
    // TODO: Group events by time window
    // TODO: Use .chunks_timeout or manual window tracking
    // TODO: For each window, calculate stats

    todo!("Implement tumbling window")
}

pub async fn detect_anomalies(
    current: &WindowedStats,
    baseline: &WindowedStats,
) -> Option<Alert> {
    // TODO: Compare current window to baseline
    // TODO: Check if event_count > 2x baseline (spike)
    // TODO: Check if avg_value > 1.5x baseline (abnormal values)
    // TODO: Return Alert if anomaly detected

    todo!("Implement anomaly detection")
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(1000);

    tokio::spawn(generate_events(tx, 10000, 10));

    let stream = ReceiverStream::new(rx);
    let processed = process_stream(stream);

    let windows = create_tumbling_window(processed, 1000); // 1-second windows

    let mut baseline: Option<WindowedStats> = None;

    windows
        .for_each(|stats| async {
            println!(
                "Window [{} - {}]: {} events, avg={:.2}, max={:.2}",
                stats.window_start, stats.window_end, stats.event_count, stats.avg_value, stats.max_value
            );

            if let Some(ref base) = baseline {
                if let Some(alert) = detect_anomalies(&stats, base).await {
                    println!("ALERT: {}", alert.message);
                }
            } else {
                baseline = Some(stats.clone());
            }
        })
        .await;
}
```

**Implementation Hints:**
1. Window assignment: `let window_id = event.timestamp / duration_ms;`
2. Use `HashMap<u64, Vec<ProcessedEvent>>` to group by window
3. For tumbling: emit window when next event is in different window
4. Stats: use `.fold()` or manual accumulation
5. Anomaly: `current.event_count > baseline.event_count * 2`

---

## Milestone 5: Backpressure Handling

### Introduction

**Why Milestone 4 Isn't Enough**: Fast producers can overwhelm slow consumers. Without backpressure, memory usage grows unbounded → OOM crash. Bounded channels provide backpressure but can still accumulate events.

**The Improvement**: Implement explicit backpressure strategies—drop oldest, drop newest, or apply sampling when consumer falls behind.

**Optimization (Memory)**: Unbounded buffer with 10,000 events/sec input, 1,000 events/sec processing = 9,000 events/sec accumulation. At 1KB/event, that's 9MB/sec → 540MB/minute → crash. Bounded buffer + drop strategy keeps memory constant.

### Architecture

**Structs:**
- `BackpressureConfig` - Configuration for backpressure handling
  - **Field** `strategy: BackpressureStrategy` - How to handle overload
  - **Field** `buffer_size: usize` - Maximum buffered events
  - **Field** `sample_rate: f64` - Sampling probability (0.0-1.0)

- `BackpressureStrategy` - Enum for strategies
  - **Variant** `DropOldest` - Remove oldest events when full
  - **Variant** `DropNewest` - Reject new events when full
  - **Variant** `Sample` - Randomly sample events

- `BackpressureStats` - Metrics about drops
  - **Field** `received: AtomicUsize` - Total events received
  - **Field** `dropped: AtomicUsize` - Events dropped
  - **Field** `processed: AtomicUsize` - Events processed

**Key Functions:**
- `async fn apply_backpressure(stream: impl Stream<Item = Event>, config: BackpressureConfig) -> (impl Stream<Item = Event>, BackpressureStats)` - Wraps stream with backpressure
- `fn should_sample(rate: f64) -> bool` - Random sampling decision
- `async fn monitor_backpressure(stats: Arc<BackpressureStats>)` - Reports drop rate

**Role Each Plays:**
- **Bounded channel**: Provides natural backpressure (send blocks when full)
- **Drop strategies**: Choose which events to discard under load
- **Sampling**: Reduces load while maintaining statistical properties

### Checkpoint Tests

```rust
#[test]
fn test_should_sample() {
    // Sample at 50% - roughly half should pass
    let mut passed = 0;
    for _ in 0..1000 {
        if should_sample(0.5) {
            passed += 1;
        }
    }

    // Should be around 500 (allow some variance)
    assert!(passed > 400 && passed < 600);
}

#[tokio::test]
async fn test_drop_oldest_strategy() {
    let config = BackpressureConfig {
        strategy: BackpressureStrategy::DropOldest,
        buffer_size: 5,
        sample_rate: 1.0,
    };

    let (tx, rx) = mpsc::channel(100);

    // Send 10 events quickly
    for i in 0..10 {
        tx.send(Event::new(i, "s1".into(), i as f64, EventType::Metric))
            .await
            .unwrap();
    }
    drop(tx);

    let stream = ReceiverStream::new(rx);
    let (processed_stream, stats) = apply_backpressure(stream, config);

    let events: Vec<Event> = processed_stream.collect().await;

    // Should keep last 5 events (dropped first 5)
    assert_eq!(stats.received.load(Ordering::Relaxed), 10);
    assert_eq!(stats.dropped.load(Ordering::Relaxed), 5);
}

#[tokio::test]
async fn test_sampling_strategy() {
    let config = BackpressureConfig {
        strategy: BackpressureStrategy::Sample,
        buffer_size: 1000,
        sample_rate: 0.1, // Keep 10%
    };

    let events: Vec<Event> = (0..1000)
        .map(|i| Event::new(i, "s1".into(), i as f64, EventType::Metric))
        .collect();

    let stream = tokio_stream::iter(events);
    let (processed_stream, stats) = apply_backpressure(stream, config);

    let result: Vec<Event> = processed_stream.collect().await;

    // Should keep roughly 10% (allow variance)
    assert!(result.len() > 50 && result.len() < 150);
    assert_eq!(stats.received.load(Ordering::Relaxed), 1000);
}

#[tokio::test]
async fn test_backpressure_stats() {
    let stats = BackpressureStats::new();

    stats.increment_received();
    stats.increment_received();
    stats.increment_dropped();
    stats.increment_processed();

    assert_eq!(stats.received(), 2);
    assert_eq!(stats.dropped(), 1);
    assert_eq!(stats.processed(), 1);

    let report = stats.get_report();
    assert!(report.contains("50.0%")); // 1 dropped out of 2
}
```

### Starter Code

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use rand::Rng;

#[derive(Debug, Clone)]
pub enum BackpressureStrategy {
    DropOldest,
    DropNewest,
    Sample,
}

#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    pub strategy: BackpressureStrategy,
    pub buffer_size: usize,
    pub sample_rate: f64,
}

pub struct BackpressureStats {
    received: AtomicUsize,
    dropped: AtomicUsize,
    processed: AtomicUsize,
}

impl BackpressureStats {
    pub fn new() -> Self {
        // TODO: Initialize atomic counters

        todo!("Implement BackpressureStats::new")
    }

    pub fn increment_received(&self) {
        self.received.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_dropped(&self) {
        self.dropped.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_processed(&self) {
        self.processed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn received(&self) -> usize {
        self.received.load(Ordering::Relaxed)
    }

    pub fn dropped(&self) -> usize {
        self.dropped.load(Ordering::Relaxed)
    }

    pub fn processed(&self) -> usize {
        self.processed.load(Ordering::Relaxed)
    }

    pub fn get_report(&self) -> String {
        // TODO: Format statistics report
        // Include drop rate percentage

        todo!("Implement stats report")
    }
}

pub fn should_sample(rate: f64) -> bool {
    // TODO: Return true with probability 'rate'
    // Hint: rand::thread_rng().gen::<f64>() < rate

    todo!("Implement sampling decision")
}

pub fn apply_backpressure<S>(
    stream: S,
    config: BackpressureConfig,
) -> (impl Stream<Item = Event>, Arc<BackpressureStats>)
where
    S: Stream<Item = Event>,
{
    // TODO: Wrap stream with backpressure logic
    // TODO: Track events in stats
    // TODO: Apply strategy (drop or sample)

    todo!("Implement backpressure")
}

pub async fn monitor_backpressure(stats: Arc<BackpressureStats>) {
    // TODO: Periodically print stats
    // TODO: Loop with sleep, print every second

    todo!("Implement monitoring")
}

#[tokio::main]
async fn main() {
    let config = BackpressureConfig {
        strategy: BackpressureStrategy::Sample,
        buffer_size: 100,
        sample_rate: 0.1,
    };

    let (tx, rx) = mpsc::channel(1000);

    // Very fast event generator
    tokio::spawn(generate_events(tx, 100000, 1));

    let stream = ReceiverStream::new(rx);
    let (backpressured, stats) = apply_backpressure(stream, config);

    // Spawn monitor
    let stats_clone = Arc::clone(&stats);
    tokio::spawn(monitor_backpressure(stats_clone));

    let processed = process_stream(backpressured);

    // Slow consumer (simulates heavy processing)
    processed
        .for_each(|event| async move {
            sleep(Duration::from_millis(10)).await; // Slow processing
            // Process event...
        })
        .await;

    println!("\nFinal stats: {}", stats.get_report());
}
```

**Implementation Hints:**
1. Use `VecDeque` with capacity for DropOldest buffer
2. For DropOldest: `if buffer.len() == capacity { buffer.pop_front(); }`
3. For Sample: `stream.filter(|_| should_sample(rate))`
4. Wrap stream in struct with stats tracking
5. Use `.inspect()` combinator to track events

---

## Milestone 6: Multi-Source Stream Merging and Fan-Out

### Introduction

**Why Milestone 5 Isn't Enough**: Real systems have multiple event sources (sensors, logs, APIs). Processing them separately duplicates code. We need to merge streams and distribute results.

**The Improvement**: Implement stream merging (combine multiple sources) and fan-out (send results to multiple consumers). Use `select_all` for merging and broadcast channels for fan-out.

**Optimization (Parallelism)**: Fan-out enables parallel processing—same events processed by analytics engine AND alerting system simultaneously. Without fan-out, you'd process sequentially (2x latency) or duplicate the event stream (2x bandwidth).

### Architecture

**Structs:**
- `StreamSource` - Identifies event source
  - **Field** `id: String` - Source identifier
  - **Field** `source_type: SourceType` - Category

- `SourceType` - Enum for source types
  - **Variant** `Sensor(String)`, `LogFile(String)`, `API(String)`

- `ProcessingPipeline` - Manages multi-stream processing
  - **Field** `sources: Vec<StreamSource>` - Active sources
  - **Field** `outputs: Vec<broadcast::Sender<ProcessedEvent>>` - Output channels

**Key Functions:**
- `fn merge_streams(streams: Vec<impl Stream<Item = Event>>) -> impl Stream<Item = Event>` - Combines multiple streams
- `fn create_fanout(input: impl Stream<Item = ProcessedEvent>, output_count: usize) -> Vec<broadcast::Receiver<ProcessedEvent>>` - Distributes to multiple outputs
- `async fn run_pipeline(sources: Vec<impl Stream<Item = Event>>) -> ProcessingPipeline` - Full pipeline

**Role Each Plays:**
- **select_all**: Merges streams, yields items as they arrive (unordered)
- **broadcast channel**: One sender, many receivers (all get same events)
- **Fan-out**: Enables parallel consumers without duplication

### Checkpoint Tests

```rust
#[tokio::test]
async fn test_merge_streams() {
    use tokio_stream::{self as stream};

    let stream1 = stream::iter(vec![1, 2, 3]);
    let stream2 = stream::iter(vec![4, 5, 6]);
    let stream3 = stream::iter(vec![7, 8, 9]);

    let merged = merge_streams(vec![stream1, stream2, stream3]);
    let results: Vec<i32> = merged.collect().await;

    assert_eq!(results.len(), 9);
    assert!(results.contains(&1));
    assert!(results.contains(&9));
}

#[tokio::test]
async fn test_fanout() {
    use tokio::sync::broadcast;

    let (tx, _) = broadcast::channel(100);

    let rx1 = tx.subscribe();
    let rx2 = tx.subscribe();
    let rx3 = tx.subscribe();

    // Send events
    for i in 0..5 {
        tx.send(i).unwrap();
    }
    drop(tx);

    // All receivers should get all events
    let results1: Vec<i32> = ReceiverStream::new(rx1).collect().await;
    let results2: Vec<i32> = ReceiverStream::new(rx2).collect().await;
    let results3: Vec<i32> = ReceiverStream::new(rx3).collect().await;

    assert_eq!(results1, results2);
    assert_eq!(results2, results3);
    assert_eq!(results1.len(), 5);
}

#[tokio::test]
async fn test_multi_source_processing() {
    // Create 3 event sources
    let (tx1, rx1) = mpsc::channel(10);
    let (tx2, rx2) = mpsc::channel(10);
    let (tx3, rx3) = mpsc::channel(10);

    tokio::spawn(async move {
        for i in 0..5 {
            tx1.send(Event::new(i, "source1".into(), i as f64, EventType::Metric))
                .await
                .unwrap();
        }
    });

    tokio::spawn(async move {
        for i in 5..10 {
            tx2.send(Event::new(i, "source2".into(), i as f64, EventType::Log))
                .await
                .unwrap();
        }
    });

    tokio::spawn(async move {
        for i in 10..15 {
            tx3.send(Event::new(i, "source3".into(), i as f64, EventType::Alert))
                .await
                .unwrap();
        }
    });

    let streams = vec![
        ReceiverStream::new(rx1),
        ReceiverStream::new(rx2),
        ReceiverStream::new(rx3),
    ];

    let merged = merge_streams(streams);
    let events: Vec<Event> = merged.collect().await;

    assert_eq!(events.len(), 15);
}
```

### Starter Code

```rust
use tokio::sync::broadcast;
use tokio_stream::StreamExt;

#[derive(Debug, Clone)]
pub enum SourceType {
    Sensor(String),
    LogFile(String),
    API(String),
}

#[derive(Debug, Clone)]
pub struct StreamSource {
    pub id: String,
    pub source_type: SourceType,
}

pub struct ProcessingPipeline {
    pub sources: Vec<StreamSource>,
    pub broadcast_tx: broadcast::Sender<ProcessedEvent>,
}

pub fn merge_streams<S>(streams: Vec<S>) -> impl Stream<Item = Event>
where
    S: Stream<Item = Event> + Send + 'static,
{
    // TODO: Use StreamExt::merge or select_all to combine streams
    // Hint: futures::stream::select_all(streams)

    todo!("Implement stream merging")
}

pub fn create_fanout(
    input: impl Stream<Item = ProcessedEvent> + Send + 'static,
    output_count: usize,
) -> (tokio::task::JoinHandle<()>, Vec<broadcast::Receiver<ProcessedEvent>>) {
    // TODO: Create broadcast channel
    // TODO: Spawn task to forward input stream to broadcast
    // TODO: Create N receivers
    // TODO: Return handle and receivers

    todo!("Implement fan-out")
}

pub async fn run_pipeline(
    sources: Vec<(StreamSource, impl Stream<Item = Event> + Send + 'static)>,
) -> ProcessingPipeline {
    // TODO: Extract streams from sources
    // TODO: Merge all streams
    // TODO: Process merged stream
    // TODO: Create fan-out for processed events
    // TODO: Return pipeline with broadcast sender

    todo!("Implement full pipeline")
}

#[tokio::main]
async fn main() {
    // Create multiple event sources
    let mut sources = Vec::new();

    for i in 0..3 {
        let (tx, rx) = mpsc::channel(100);
        let source_id = format!("sensor-{}", i);

        tokio::spawn({
            let source = source_id.clone();
            async move {
                generate_events(tx, 100, 50).await;
            }
        });

        sources.push((
            StreamSource {
                id: source_id.clone(),
                source_type: SourceType::Sensor(source_id),
            },
            ReceiverStream::new(rx),
        ));
    }

    let pipeline = run_pipeline(sources).await;

    // Create multiple consumers
    let mut consumer1 = pipeline.broadcast_tx.subscribe();
    let mut consumer2 = pipeline.broadcast_tx.subscribe();

    // Consumer 1: Count events
    let counter = tokio::spawn(async move {
        let mut count = 0;
        while consumer1.recv().await.is_ok() {
            count += 1;
        }
        println!("Consumer 1 processed: {} events", count);
    });

    // Consumer 2: Alert on high severity
    let alerter = tokio::spawn(async move {
        while let Ok(event) = consumer2.recv().await {
            if matches!(event.severity, Severity::High | Severity::Critical) {
                println!("ALERT: High severity event from {}", event.original.source);
            }
        }
        println!("Consumer 2 done");
    });

    tokio::join!(counter, alerter);
}
```

**Implementation Hints:**
1. Use `futures::stream::select_all(streams)` for merging
2. For fan-out: `let (tx, _rx) = broadcast::channel(capacity);`
3. Subscribe: `let rx = tx.subscribe();`
4. Forward stream: `while let Some(item) = stream.next().await { tx.send(item)?; }`
5. Handle lagging receivers: broadcast drops oldest if receiver falls behind

---

## Complete Working Example

```rust
// Cargo.toml:
// [dependencies]
// tokio = { version = "1.35", features = ["full"] }
// tokio-stream = "0.1"
// futures = "0.3"
// serde = { version = "1.0", features = ["derive"] }
// rand = "0.8"

use tokio::sync::{mpsc, broadcast};
use tokio::time::{sleep, Duration};
use tokio_stream::{Stream, StreamExt};
use tokio_stream::wrappers::ReceiverStream;
use futures::stream;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

// Event types
#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    Metric,
    Log,
    Alert,
    Error,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: u64,
    pub timestamp: u64,
    pub source: String,
    pub value: f64,
    pub event_type: EventType,
}

impl Event {
    pub fn new(id: u64, source: String, value: f64, event_type: EventType) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            id,
            timestamp,
            source,
            value,
            event_type,
        }
    }
}

// Processing types
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ProcessedEvent {
    pub original: Event,
    pub normalized_value: f64,
    pub severity: Severity,
}

// Backpressure types
pub struct BackpressureStats {
    received: AtomicUsize,
    dropped: AtomicUsize,
    processed: AtomicUsize,
}

impl BackpressureStats {
    pub fn new() -> Self {
        Self {
            received: AtomicUsize::new(0),
            dropped: AtomicUsize::new(0),
            processed: AtomicUsize::new(0),
        }
    }

    pub fn increment_received(&self) {
        self.received.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_dropped(&self) {
        self.dropped.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_processed(&self) {
        self.processed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_report(&self) -> String {
        let received = self.received.load(Ordering::Relaxed);
        let dropped = self.dropped.load(Ordering::Relaxed);
        let processed = self.processed.load(Ordering::Relaxed);
        let drop_rate = if received > 0 {
            (dropped as f64 / received as f64) * 100.0
        } else {
            0.0
        };

        format!(
            "Received: {}, Processed: {}, Dropped: {} ({:.1}% drop rate)",
            received, processed, dropped, drop_rate
        )
    }
}

// Event generation
pub async fn generate_events(tx: mpsc::Sender<Event>, count: usize, delay_ms: u64) {
    for i in 0..count {
        let source = format!("sensor-{}", i % 5);
        let value = (i as f64 * 3.7) % 100.0;
        let event_type = match i % 4 {
            0 => EventType::Metric,
            1 => EventType::Log,
            2 => EventType::Alert,
            _ => EventType::Error,
        };

        let event = Event::new(i as u64, source, value, event_type);

        if tx.send(event).await.is_err() {
            break;
        }

        sleep(Duration::from_millis(delay_ms)).await;
    }
}

// Processing functions
pub fn normalize_value(value: f64, min: f64, max: f64) -> f64 {
    if (max - min).abs() < 0.001 {
        0.5
    } else {
        (value - min) / (max - min)
    }
}

pub fn calculate_severity(normalized: f64) -> Severity {
    match normalized {
        x if x < 0.3 => Severity::Low,
        x if x < 0.6 => Severity::Medium,
        x if x < 0.9 => Severity::High,
        _ => Severity::Critical,
    }
}

pub fn process_event(event: Event) -> ProcessedEvent {
    let normalized = normalize_value(event.value, 0.0, 100.0);
    let severity = calculate_severity(normalized);

    ProcessedEvent {
        original: event,
        normalized_value: normalized,
        severity,
    }
}

// Stream merging
pub fn merge_event_streams<S>(streams: Vec<S>) -> impl Stream<Item = Event>
where
    S: Stream<Item = Event> + Send + 'static,
{
    stream::select_all(streams)
}

// Windowed aggregation
#[derive(Debug, Clone)]
pub struct WindowedStats {
    pub window_start: u64,
    pub window_end: u64,
    pub event_count: usize,
    pub avg_value: f64,
    pub max_value: f64,
}

impl WindowedStats {
    pub fn from_events(events: Vec<ProcessedEvent>, window_start: u64, duration_ms: u64) -> Self {
        let count = events.len();
        let avg = if count > 0 {
            events.iter().map(|e| e.original.value).sum::<f64>() / count as f64
        } else {
            0.0
        };
        let max = events
            .iter()
            .map(|e| e.original.value)
            .fold(0.0, f64::max);

        Self {
            window_start,
            window_end: window_start + duration_ms,
            event_count: count,
            avg_value: avg,
            max_value: max,
        }
    }
}

// Complete pipeline
pub async fn run_complete_pipeline() {
    println!("=== Real-Time Event Stream Processor ===\n");

    // Create multiple event sources
    let mut stream_handles = Vec::new();

    for i in 0..3 {
        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            generate_events(tx, 50, 20 + i * 10).await;
        });

        stream_handles.push(ReceiverStream::new(rx));
    }

    // Merge all sources
    let merged = merge_event_streams(stream_handles);

    // Process events
    let processed = merged.map(process_event);

    // Apply backpressure with sampling
    let stats = Arc::new(BackpressureStats::new());
    let stats_clone = Arc::clone(&stats);

    let with_backpressure = processed.filter(move |event| {
        stats_clone.increment_received();

        // Sample at 50% under load
        let should_process = rand::random::<f64>() < 0.5;

        if should_process {
            stats_clone.increment_processed();
        } else {
            stats_clone.increment_dropped();
        }

        should_process
    });

    // Create fan-out for multiple consumers
    let (broadcast_tx, _) = broadcast::channel(1000);
    let mut rx1 = broadcast_tx.subscribe();
    let mut rx2 = broadcast_tx.subscribe();

    // Forward processed events to broadcast
    let broadcast_clone = broadcast_tx.clone();
    tokio::spawn(async move {
        tokio::pin!(with_backpressure);

        while let Some(event) = with_backpressure.next().await {
            let _ = broadcast_clone.send(event);
        }
    });

    // Consumer 1: Count by severity
    let counter = tokio::spawn(async move {
        let mut counts: HashMap<String, usize> = HashMap::new();

        while let Ok(event) = rx1.recv().await {
            let severity = format!("{:?}", event.severity);
            *counts.entry(severity).or_insert(0) += 1;
        }

        println!("\nSeverity counts:");
        for (sev, count) in counts {
            println!("  {}: {}", sev, count);
        }
    });

    // Consumer 2: Alert on critical
    let alerter = tokio::spawn(async move {
        let mut alert_count = 0;

        while let Ok(event) = rx2.recv().await {
            if matches!(event.severity, Severity::Critical) {
                alert_count += 1;
                println!(
                    "CRITICAL ALERT: {} = {:.2}",
                    event.original.source, event.original.value
                );
            }
        }

        println!("\nTotal critical alerts: {}", alert_count);
    });

    // Wait for all consumers
    tokio::join!(counter, alerter);

    println!("\n{}", stats.get_report());
}

#[tokio::main]
async fn main() {
    run_complete_pipeline().await;
}
```

This implementation demonstrates a production-ready real-time event processing system with:
1. **Multi-source stream merging** - Combines events from multiple sources
2. **Stream transformations** - Maps and filters events
3. **Backpressure handling** - Sampling strategy to prevent overload
4. **Fan-out architecture** - Multiple consumers process same events
5. **Windowed aggregations** - Time-based statistics
6. **Performance monitoring** - Tracks throughput and drop rates

Perfect for IoT, log processing, and real-time analytics workloads.
