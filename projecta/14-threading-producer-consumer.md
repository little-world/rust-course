## Project 1: Producer-Consumer Pipeline with Channels

### Problem Statement

Build a multi-stage data processing pipeline using message-passing channels. The system processes log entries through multiple stages: parsing, filtering, enrichment, and aggregation. Each stage runs in parallel threads, communicating via channels.

### Use Cases

- Log aggregation and analysis systems
- ETL (Extract, Transform, Load) pipelines
- Real-time stream processing
- Video/audio transcoding pipelines
- Distributed task processing
- Microservices communication




### Why It Matters

Channels enable decoupled parallelism: producers and consumers run independently without shared state. This eliminates data races and simplifies reasoning. Bounded channels provide backpressure preventing memory exhaustion. Message passing scales to distributed systems (same pattern as actor models, microservices).

Under load, pipelined architecture achieves throughput limited only by slowest stage. Without pipelining, stages execute sequentially—3 stages @ 100ms each = 300ms latency. With pipelining: 100ms latency, 10x higher throughput.

Example pipeline:
```
Raw Logs → Parser → Filter → Enricher → Aggregator → Results
```


Your pipeline should:
- Parse raw log lines into structured log entries
- Filter entries by severity level
- Enrich entries with metadata (timestamps, tags)
- Aggregate statistics (counts per severity, per source)
- Handle backpressure when consumers are slow
- Gracefully shutdown all stages
---

### Milestone 1: Basic MPSC Channel Communication

### Introduction

Implement a simple producer-consumer pattern using Rust's MPSC (Multi-Producer Single-Consumer) channels. This establishes the foundation for understanding channel semantics and message passing.

### Architecture

**Structs:**
- `LogEntry` - Parsed log message
    - **Field** `timestamp: u64` - When log was created
    - **Field** `level: LogLevel` - Severity (Debug, Info, Warn, Error)
    - **Field** `message: String` - Log content
    - **Field** `source: String` - Where log originated

- `LogLevel` - Severity enum
    - **Variant** `Debug`, `Info`, `Warn`, `Error`

**Key Functions:**
- `producer_thread(tx: Sender<LogEntry>)` - Generate log entries
- `consumer_thread(rx: Receiver<LogEntry>)` - Process log entries
- `parse_log_line(line: &str) -> Option<LogEntry>` - Parse raw string

**Role Each Plays:**
- Channel: Thread-safe queue for message passing
- Sender: Can be cloned for multiple producers
- Receiver: Single consumer drains messages
- MPSC: Many producers, one consumer pattern

### Checkpoint Tests

```rust
#[test]
fn test_basic_send_receive() {
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();

    tx.send(LogEntry::new(LogLevel::Info, "Test message".into())).unwrap();

    let received = rx.recv().unwrap();
    assert_eq!(received.level, LogLevel::Info);
    assert_eq!(received.message, "Test message");
}

#[test]
fn test_multiple_producers() {
    use std::sync::mpsc;
    use std::thread;

    let (tx, rx) = mpsc::channel();

    // Spawn 3 producer threads
    for i in 0..3 {
        let tx_clone = tx.clone();
        thread::spawn(move || {
            tx_clone.send(LogEntry::new(
                LogLevel::Info,
                format!("Message from producer {}", i)
            )).unwrap();
        });
    }

    drop(tx); // Drop original sender

    let mut count = 0;
    while let Ok(_) = rx.recv() {
        count += 1;
    }

    assert_eq!(count, 3);
}

#[test]
fn test_channel_closed() {
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();

    drop(tx); // Close sender

    // Receive should return error when channel closed
    assert!(rx.recv().is_err());
}
```

### Starter Code

```rust
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: String) -> Self {
        // TODO: Create LogEntry with current timestamp
        // Use std::time::SystemTime::now() or counter
        // Set source to "unknown" for now
        unimplemented!()
    }
}

pub fn producer_thread(tx: Sender<LogEntry>, num_messages: usize) {
    // TODO: Generate num_messages log entries
    // Send each through channel
    // Vary log levels (use modulo to cycle through)
    // Hint: for i in 0..num_messages
    //         let level = match i % 4 {...}
    //         tx.send(LogEntry::new(level, format!("Log {}", i))).unwrap()
    unimplemented!()
}

pub fn consumer_thread(rx: Receiver<LogEntry>) -> Vec<LogEntry> {
    // TODO: Receive all messages until channel closes
    // Collect into Vec and return
    // Hint: let mut logs = Vec::new();
    //       while let Ok(entry) = rx.recv() { logs.push(entry); }
    unimplemented!()
}
```

**Why previous Milestone is not enough:** N/A - Foundation Milestone.

**What's the improvement:** Channels provide lock-free message passing:
- Shared state approach: Mutex<Vec<LogEntry>> - all threads contend for lock
- Channel approach: Lock-free queue, producers and consumers independent

For 8 producers + 1 consumer:
- Mutex: Serialized access, ~1-core performance
- Channel: Parallel sending, 8× throughput

---

### Milestone 2: Multi-Stage Pipeline

### Introduction

Build a 3-stage pipeline where each stage runs in a separate thread: Parser → Filter → Aggregator. Stages communicate via channels, enabling parallel processing.

### Architecture

**Pipeline Stages:**
1. **Parser**: Raw strings → LogEntry
2. **Filter**: LogEntry → LogEntry (drop low-priority logs)
3. **Aggregator**: LogEntry → Statistics

**Enhanced Structs:**
- `LogStats` - Aggregated statistics
    - **Field** `total_count: usize`
    - **Field** `count_by_level: HashMap<LogLevel, usize>`
    - **Field** `count_by_source: HashMap<String, usize>`

**Key Functions:**
- `parser_stage(input: Receiver<String>, output: Sender<LogEntry>)`
- `filter_stage(input: Receiver<LogEntry>, output: Sender<LogEntry>, min_level: LogLevel)`
- `aggregator_stage(input: Receiver<LogEntry>) -> LogStats`

**Role Each Plays:**
- Each stage is independent thread
- Input/output channels decouple stages
- Backpressure: if filter is slow, parser blocks on send

### Checkpoint Tests

```rust
#[test]
fn test_pipeline_throughput() {
    // Create 3-stage pipeline
    let (parser_tx, parser_rx) = mpsc::channel();
    let (filter_tx, filter_rx) = mpsc::channel();
    let (aggr_tx, aggr_rx) = mpsc::channel();

    // Start stages
    let parser_handle = thread::spawn(move || {
        parser_stage(parser_rx, filter_tx)
    });

    let filter_handle = thread::spawn(move || {
        filter_stage(filter_rx, aggr_tx, LogLevel::Info)
    });

    let aggregator_handle = thread::spawn(move || {
        aggregator_stage(aggr_rx)
    });

    // Send raw logs
    for i in 0..100 {
        parser_tx.send(format!("INFO Log message {}", i)).unwrap();
    }
    drop(parser_tx);

    // Wait for completion
    parser_handle.join().unwrap();
    filter_handle.join().unwrap();
    let stats = aggregator_handle.join().unwrap();

    assert_eq!(stats.total_count, 100);
}

#[test]
fn test_filter_drops_debug() {
    let (tx_in, rx_in) = mpsc::channel();
    let (tx_out, rx_out) = mpsc::channel();

    thread::spawn(move || {
        filter_stage(rx_in, tx_out, LogLevel::Info)
    });

    // Send Debug and Info messages
    tx_in.send(LogEntry::new(LogLevel::Debug, "Debug msg".into())).unwrap();
    tx_in.send(LogEntry::new(LogLevel::Info, "Info msg".into())).unwrap();
    drop(tx_in);

    let results: Vec<_> = rx_out.iter().collect();
    assert_eq!(results.len(), 1); // Only Info message passed
    assert_eq!(results[0].level, LogLevel::Info);
}
```

### Starter Code

```rust
use std::collections::HashMap;

#[derive(Default)]
pub struct LogStats {
    pub total_count: usize,
    pub count_by_level: HashMap<LogLevel, usize>,
    pub count_by_source: HashMap<String, usize>,
}

pub fn parser_stage(input: Receiver<String>, output: Sender<LogEntry>) {
    // TODO: Receive raw log strings
    // Parse each into LogEntry (or skip if invalid)
    // Send LogEntry to output channel
    // Hint: while let Ok(line) = input.recv() {
    //         if let Some(entry) = parse_log_line(&line) {
    //           output.send(entry).unwrap();
    //         }
    //       }
    unimplemented!()
}

pub fn filter_stage(
    input: Receiver<LogEntry>,
    output: Sender<LogEntry>,
    min_level: LogLevel,
) {
    // TODO: Receive log entries
    // Filter out entries below min_level
    // Send filtered entries to output
    // Hint: LogLevel ordering: Debug < Info < Warn < Error
    unimplemented!()
}

pub fn aggregator_stage(input: Receiver<LogEntry>) -> LogStats {
    // TODO: Receive all log entries
    // Count total, by level, by source
    // Return LogStats
    // Hint: let mut stats = LogStats::default();
    //       while let Ok(entry) = input.recv() {
    //         stats.total_count += 1;
    //         *stats.count_by_level.entry(entry.level).or_insert(0) += 1;
    //         ...
    //       }
    unimplemented!()
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    // TODO: Parse "LEVEL message" format
    // Example: "INFO Starting server" → LogEntry with Info level
    // Return None if parse fails
    unimplemented!()
}
```

**Why previous Milestone is not enough:** Single producer-consumer doesn't utilize multiple stages running in parallel. Pipeline enables concurrent processing of different messages at each stage.

**What's the improvement:** Pipelined parallelism increases throughput:
- Sequential: Parse 100ms + Filter 50ms + Aggregate 50ms = 200ms per message
- Pipelined: All stages run concurrently, throughput limited by slowest (100ms)

For 1000 messages:
- Sequential: 200 seconds
- Pipelined: ~100 seconds (2× faster)

---

### Milestone 3: Bounded Channels and Backpressure

### Introduction

Use bounded channels to limit queue sizes and implement backpressure. This prevents fast producers from overwhelming slow consumers and exhausting memory.

### Architecture

**Bounded Channel:**
- `sync_channel(capacity)` - Channel with fixed buffer size
- Sender blocks when buffer full (backpressure)
- Prevents unbounded memory growth

**Enhanced Metrics:**
- Track messages dropped (when buffer full)
- Measure queue depths
- Monitor blocking time

**Key Functions:**
- `bounded_producer(tx: SyncSender<LogEntry>, rate_limit: Duration)`
- `slow_consumer(rx: Receiver<LogEntry>, process_time: Duration)`

### Checkpoint Tests

```rust
#[test]
fn test_backpressure_blocks() {
    use std::sync::mpsc::sync_channel;
    use std::time::Instant;

    let (tx, rx) = sync_channel(2); // Small buffer

    // Send 3 items quickly - 3rd should block
    tx.send(1).unwrap();
    tx.send(2).unwrap();

    let start = Instant::now();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        rx.recv().unwrap(); // Drain one item
    });

    tx.send(3).unwrap(); // Should block until recv() called
    let elapsed = start.elapsed();

    assert!(elapsed >= Duration::from_millis(90)); // Blocked ~100ms
}

#[test]
fn test_try_send_nonblocking() {
    use std::sync::mpsc::sync_channel;

    let (tx, _rx) = sync_channel(1);

    tx.send(1).unwrap(); // Fills buffer
    assert!(tx.try_send(2).is_err()); // Should fail immediately
}
```

### Starter Code

```rust
use std::sync::mpsc::{sync_channel, SyncSender, Receiver, TrySendError};
use std::time::{Duration, Instant};

pub struct BackpressureMetrics {
    pub messages_sent: usize,
    pub messages_dropped: usize,
    pub total_block_time: Duration,
}

pub fn bounded_producer(
    tx: SyncSender<LogEntry>,
    num_messages: usize,
    rate_limit: Duration,
) -> BackpressureMetrics {
    // TODO: Send messages with rate limiting
    // Track blocking time when send() blocks
    // Count dropped messages if using try_send
    // Hint: let start = Instant::now();
    //       match tx.try_send(entry) {
    //         Ok(_) => metrics.messages_sent += 1,
    //         Err(TrySendError::Full(_)) => metrics.messages_dropped += 1,
    //         Err(TrySendError::Disconnected(_)) => break,
    //       }
    unimplemented!()
}

pub fn slow_consumer(
    rx: Receiver<LogEntry>,
    process_time: Duration,
) -> usize {
    // TODO: Receive messages, process slowly
    // Sleep for process_time per message
    // Return count of messages processed
    unimplemented!()
}
```

**Why previous Milestone is not enough:** Unbounded channels can grow infinitely if producer is faster than consumer. With unlimited logs, memory exhaustion crashes program.

**What's the improvement:** Bounded channels provide automatic backpressure:
- Unbounded: 1000 msgs/sec producer, 100 msgs/sec consumer → 900 msgs/sec queue growth → OOM
- Bounded (capacity 100): Producer slows to match consumer rate → stable memory

For production systems, bounded channels prevent cascading failures.

---

### Milestone 4: Graceful Shutdown

### Introduction

Implement clean shutdown of pipeline: signal all stages to stop, drain remaining messages, collect final statistics. Handle shutdown during message processing.

### Architecture

**Shutdown Mechanisms:**
1. **Channel closure**: Drop senders to signal "no more data"
2. **Shutdown signal**: Separate channel with shutdown message
3. **Timeout**: Force shutdown after deadline

**Enhanced Shutdown:**
- `ShutdownSignal` - Broadcast shutdown to all stages
- Drain-and-exit: Process remaining messages before stopping
- Timeout: Force-kill after grace period

**Key Functions:**
- `shutdown_coordinator(signal_tx: Sender<()>)`
- `graceful_worker(data_rx: Receiver<T>, shutdown_rx: Receiver<()>)`

### Checkpoint Tests

```rust
#[test]
fn test_shutdown_signal() {
    let (shutdown_tx, shutdown_rx) = mpsc::channel();
    let (data_tx, data_rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let mut count = 0;
        loop {
            select! {
                recv(data_rx) -> msg => {
                    if msg.is_ok() {
                        count += 1;
                    }
                }
                recv(shutdown_rx) -> _ => {
                    println!("Shutdown signal received");
                    break;
                }
            }
        }
        count
    });

    data_tx.send(1).unwrap();
    data_tx.send(2).unwrap();

    shutdown_tx.send(()).unwrap();

    let count = handle.join().unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_drain_on_shutdown() {
    let (tx, rx) = mpsc::channel();

    // Send messages
    for i in 0..10 {
        tx.send(i).unwrap();
    }
    drop(tx); // Signal no more data

    // Consumer should drain all
    let results: Vec<_> = rx.iter().collect();
    assert_eq!(results.len(), 10);
}
```

### Starter Code

```rust
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct ShutdownCoordinator {
    shutdown_flag: Arc<AtomicBool>,
    shutdown_senders: Vec<Sender<()>>,
}

impl ShutdownCoordinator {
    pub fn new() -> Self {
        // TODO: Initialize with shutdown flag and empty senders vec
        unimplemented!()
    }

    pub fn register_worker(&mut self) -> (Arc<AtomicBool>, Receiver<()>) {
        // TODO: Create shutdown channel for worker
        // Return flag and receiver
        // Store sender for later broadcast
        unimplemented!()
    }

    pub fn shutdown(&self) {
        // TODO: Set shutdown flag
        // Send signal on all shutdown channels
        unimplemented!()
    }
}

pub fn graceful_worker<T>(
    data_rx: Receiver<T>,
    shutdown_rx: Receiver<()>,
    process_fn: impl Fn(T),
) -> usize {
    // TODO: Loop receiving from data_rx
    // Check shutdown_rx periodically
    // When shutdown received, drain remaining messages
    // Return count of processed messages
    // Hint: Use crossbeam::select! or recv_timeout
    unimplemented!()
}
```

**Why previous Milestone is not enough:** Abrupt termination loses in-flight messages. Servers need clean shutdown to finish current requests. Ungraceful shutdown can corrupt state.

**What's the improvement:** Graceful shutdown preserves data:
- Abrupt: Kill threads mid-processing → lose buffered messages
- Graceful: Signal stop → drain queues → exit cleanly

For critical systems (databases, payment processing), graceful shutdown is mandatory.

---

### Milestone 5: Error Handling and Monitoring

### Introduction

Add comprehensive error handling and monitoring: track message processing errors, pipeline health, throughput metrics, and queue depths.

### Architecture

**Error Handling:**
- Poison messages: Messages that cause processing errors
- Dead letter queue: Failed messages sent to separate channel
- Retry logic: Exponential backoff for transient failures

**Metrics:**
- Messages processed/sec
- Error rate
- Queue depths per stage
- Processing latency

**Key Functions:**
- `process_with_retry<T>(msg: T, max_retries: usize) -> Result<T, Error>`
- `dead_letter_queue(failures: Receiver<(LogEntry, Error)>)`
- `metrics_collector(stats_rx: Receiver<PipelineStats>)`

### Checkpoint Tests

```rust
#[test]
fn test_retry_logic() {
    let mut attempts = 0;
    let result = process_with_retry(
        || {
            attempts += 1;
            if attempts < 3 {
                Err("Transient error")
            } else {
                Ok(42)
            }
        },
        5 // max retries
    );

    assert_eq!(attempts, 3);
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_dead_letter_queue() {
    let (dlq_tx, dlq_rx) = mpsc::channel();

    // Send poison message
    dlq_tx.send((
        LogEntry::new(LogLevel::Error, "Bad data".into()),
        "Parse error".to_string()
    )).unwrap();

    drop(dlq_tx);

    let failures: Vec<_> = dlq_rx.iter().collect();
    assert_eq!(failures.len(), 1);
}
```

### Starter Code

```rust
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct PipelineStats {
    pub stage_name: String,
    pub messages_processed: usize,
    pub messages_failed: usize,
    pub avg_latency_us: u64,
    pub queue_depth: usize,
}

pub fn process_with_retry<T, E>(
    mut operation: impl FnMut() -> Result<T, E>,
    max_retries: usize,
) -> Result<T, E> {
    // TODO: Try operation up to max_retries times
    // Use exponential backoff: sleep 10ms, 20ms, 40ms...
    // Hint: for attempt in 0..max_retries {
    //         match operation() {
    //           Ok(val) => return Ok(val),
    //           Err(e) if attempt == max_retries - 1 => return Err(e),
    //           Err(_) => thread::sleep(Duration::from_millis(10 * 2^attempt)),
    //         }
    //       }
    unimplemented!()
}

pub fn monitored_stage<T>(
    input: Receiver<T>,
    output: Sender<T>,
    stats_tx: Sender<PipelineStats>,
    process_fn: impl Fn(T) -> Result<T, String>,
) {
    // TODO: Process messages, track metrics
    // Send stats periodically (e.g., every 100 messages)
    // Measure latency per message
    unimplemented!()
}

pub fn metrics_collector(stats_rx: Receiver<PipelineStats>) {
    // TODO: Receive stats from all stages
    // Print dashboard or write to monitoring system
    // Calculate aggregate metrics
    unimplemented!()
}
```

**Why previous Milestone is not enough:** Production systems need observability. Failures happen—must detect, handle, and monitor. Without metrics, can't diagnose slowdowns.

**What's the improvement:** Error handling prevents cascade failures:
- No error handling: One bad message crashes entire pipeline
- With retry + DLQ: Transient errors recovered, poison messages isolated

Metrics enable optimization:
- Identify bottleneck stages (high queue depth)
- Detect degradation (increasing latency)
- Capacity planning (throughput trends)

---

### Milestone 6: Benchmark vs Shared State



Benchmark channel-based pipeline against shared-state alternative using Mutex<Vec>. Measure throughput and scalability with varying thread counts.

### Architecture

**Implementations to Compare:**
1. **Channel-based**: Current pipeline implementation
2. **Shared-state**: `Arc<Mutex<Vec<LogEntry>>>` with worker threads

**Benchmarks:**
- Fixed workload (100,000 log entries)
- Vary thread count: 1, 2, 4, 8, 16
- Measure total time and entries/sec

### Starter Code

```rust
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct Benchmark;

impl Benchmark {
    pub fn benchmark_channel_pipeline(num_logs: usize, num_workers: usize) -> Duration {
        let start = Instant::now();

        // TODO: Create pipeline with num_workers threads
        // Send num_logs through pipeline
        // Wait for completion
        // Return elapsed time

        unimplemented!()
    }

    pub fn benchmark_shared_state(num_logs: usize, num_workers: usize) -> Duration {
        let logs = Arc::new(Mutex::new(Vec::new()));
        let start = Instant::now();

        // TODO: Spawn num_workers threads
        // Each thread:
        //   - Generate logs
        //   - Lock mutex
        //   - Push to shared vector
        //   - Unlock mutex
        // Wait for all threads
        // Return elapsed time

        unimplemented!()
    }

    pub fn run_comparison() {
        println!("=== Channel vs Shared State Performance ===\n");

        let num_logs = 100_000;
        let thread_counts = [1, 2, 4, 8, 16];

        for &num_threads in &thread_counts {
            println!("Threads: {}", num_threads);

            let channel_time = Self::benchmark_channel_pipeline(num_logs, num_threads);
            let mutex_time = Self::benchmark_shared_state(num_logs, num_threads);

            let channel_throughput = num_logs as f64 / channel_time.as_secs_f64();
            let mutex_throughput = num_logs as f64 / mutex_time.as_secs_f64();

            println!("  Channel: {:?} ({:.0} logs/sec)", channel_time, channel_throughput);
            println!("  Mutex:   {:?} ({:.0} logs/sec)", mutex_time, mutex_throughput);
            println!("  Speedup: {:.2}x\n", channel_throughput / mutex_throughput);
        }
    }
}
```

**Why previous Milestone is not enough:** Performance claims need validation. Benchmarks reveal scalability bottlenecks and guide architecture decisions.

**What's the improvement:** Empirical performance data:
- 1 thread: Channel ≈ Mutex (no contention)
- 4 threads: Channel 3-4× faster
- 8 threads: Channel 5-8× faster
- 16 threads: Channel 8-15× faster

Under high contention, lock-free channels dramatically outperform mutex-based shared state.

---

### Complete Working Example

```rust
use std::sync::mpsc::{channel, sync_channel, Sender, Receiver, SyncSender};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

// Types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LogLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_val = match self {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warn => 2,
            LogLevel::Error => 3,
        };
        let other_val = match other {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warn => 2,
            LogLevel::Error => 3,
        };
        self_val.cmp(&other_val)
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        LogEntry {
            timestamp,
            level,
            message,
            source: "unknown".to_string(),
        }
    }
}

#[derive(Default)]
pub struct LogStats {
    pub total_count: usize,
    pub count_by_level: HashMap<LogLevel, usize>,
    pub count_by_source: HashMap<String, usize>,
}

// Pipeline stages
pub fn parser_stage(input: Receiver<String>, output: Sender<LogEntry>) {
    while let Ok(line) = input.recv() {
        if let Some(entry) = parse_log_line(&line) {
            if output.send(entry).is_err() {
                break; // Output closed
            }
        }
    }
}

pub fn filter_stage(
    input: Receiver<LogEntry>,
    output: Sender<LogEntry>,
    min_level: LogLevel,
) {
    while let Ok(entry) = input.recv() {
        if entry.level >= min_level {
            if output.send(entry).is_err() {
                break;
            }
        }
    }
}

pub fn aggregator_stage(input: Receiver<LogEntry>) -> LogStats {
    let mut stats = LogStats::default();

    while let Ok(entry) = input.recv() {
        stats.total_count += 1;
        *stats.count_by_level.entry(entry.level.clone()).or_insert(0) += 1;
        *stats.count_by_source.entry(entry.source.clone()).or_insert(0) += 1;
    }

    stats
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    if parts.len() != 2 {
        return None;
    }

    let level = match parts[0] {
        "DEBUG" => LogLevel::Debug,
        "INFO" => LogLevel::Info,
        "WARN" => LogLevel::Warn,
        "ERROR" => LogLevel::Error,
        _ => return None,
    };

    Some(LogEntry::new(level, parts[1].to_string()))
}

// Example usage
fn main() {
    println!("=== Log Processing Pipeline Demo ===\n");

    // Create 3-stage pipeline
    let (raw_tx, raw_rx) = channel();
    let (parsed_tx, parsed_rx) = channel();
    let (filtered_tx, filtered_rx) = channel();

    // Start stages
    let parser_handle = thread::spawn(move || {
        parser_stage(raw_rx, parsed_tx);
    });

    let filter_handle = thread::spawn(move || {
        filter_stage(parsed_rx, filtered_tx, LogLevel::Info);
    });

    let aggregator_handle = thread::spawn(move || {
        aggregator_stage(filtered_rx)
    });

    // Send raw log lines
    let log_lines = vec![
        "INFO Server started on port 8080",
        "DEBUG Loaded configuration",
        "WARN High memory usage detected",
        "ERROR Failed to connect to database",
        "INFO Request processed successfully",
        "DEBUG Cache hit for user 123",
        "ERROR Timeout waiting for response",
        "INFO Shutdown initiated",
    ];

    for line in log_lines {
        raw_tx.send(line.to_string()).unwrap();
    }
    drop(raw_tx); // Signal no more data

    // Wait for pipeline to complete
    parser_handle.join().unwrap();
    filter_handle.join().unwrap();
    let stats = aggregator_handle.join().unwrap();

    println!("Pipeline Statistics:");
    println!("  Total messages: {}", stats.total_count);
    println!("  By level:");
    for (level, count) in &stats.count_by_level {
        println!("    {:?}: {}", level, count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_pipeline() {
        let (raw_tx, raw_rx) = channel();
        let (parsed_tx, parsed_rx) = channel();
        let (filtered_tx, filtered_rx) = channel();

        thread::spawn(move || parser_stage(raw_rx, parsed_tx));
        thread::spawn(move || filter_stage(parsed_rx, filtered_tx, LogLevel::Info));
        let aggregator = thread::spawn(move || aggregator_stage(filtered_rx));

        raw_tx.send("INFO Test 1".to_string()).unwrap();
        raw_tx.send("DEBUG Test 2".to_string()).unwrap(); // Filtered out
        raw_tx.send("ERROR Test 3".to_string()).unwrap();
        drop(raw_tx);

        let stats = aggregator.join().unwrap();
        assert_eq!(stats.total_count, 2); // Only INFO and ERROR
    }
}
```

