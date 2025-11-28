# Chapter 14: Threading Patterns - Programming Projects

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

## Milestone 1: Basic MPSC Channel Communication

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

## Milestone 2: Multi-Stage Pipeline

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

## Milestone 3: Bounded Channels and Backpressure

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

## Milestone 4: Graceful Shutdown

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

## Milestone 5: Error Handling and Monitoring

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

## Milestone 6: Benchmark vs Shared State



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

### Testing Strategies

1. **Unit Tests**: Test each stage independently
2. **Integration Tests**: Test complete pipeline end-to-end
3. **Concurrency Tests**: Verify thread safety with ThreadSanitizer
4. **Performance Tests**: Benchmark throughput scaling
5. **Stress Tests**: 1M+ messages, verify no message loss
6. **Shutdown Tests**: Verify graceful termination

---

This project comprehensively demonstrates message-passing concurrency using channels, from basic MPSC patterns through multi-stage pipelines, backpressure, graceful shutdown, error handling, and performance benchmarks comparing channels vs shared state.

---

## Project 2: Parallel Image Processor with Thread Pool

### Problem Statement

Build a parallel image processing application using a thread pool to process multiple images concurrently. The system should resize, filter, and save images using worker threads, with task distribution and result collection.

### Use Cases

- Image/video processing pipelines
- Web server request handling
- Batch data processing
- Parallel compilation systems
- Database query execution
- Scientific simulations

---

### Why It Matters

Thread pools amortize thread creation overhead and limit resource usage. Creating threads per task is expensive (1-2ms per spawn) and unbounded. Thread pool reuses threads and queues excess work.

For 10,000 small tasks:
- Spawn per task: 10-20 seconds (thread creation overhead)
- Thread pool (8 workers): 1-2 seconds (reuse threads)

Your image processor should:
- Load images from directory
- Distribute processing across worker threads
- Apply transformations (resize, blur, brightness adjustment)
- Save processed images to output directory
- Report progress and completion status
- Handle errors gracefully (corrupted images, disk full)


## Milestone 1: Basic Thread Pool Implementation

Implement a simple thread pool with fixed number of worker threads. Workers pull tasks from shared queue and execute them.

### Architecture

**Structs:**
- `ThreadPool` - Manages worker threads
  - **Field** `workers: Vec<JoinHandle<()>>` - Worker thread handles
  - **Field** `sender: Sender<Job>` - Task submission channel
  - **Field** `shutdown: Arc<AtomicBool>` - Shutdown signal

- `Job` - Unit of work
  - Type alias: `Box<dyn FnOnce() + Send + 'static>`

**Key Functions:**
- `new(size: usize) -> ThreadPool` - Create pool with N workers
- `execute<F>(&self, f: F)` where `F: FnOnce() + Send + 'static` - Submit task
- `shutdown(self)` - Stop all workers gracefully

**Role Each Plays:**
- Worker threads: Loop receiving and executing jobs
- Shared channel: Distributes work across workers
- Shutdown flag: Coordinates graceful termination

### Checkpoint Tests

```rust
#[test]
fn test_thread_pool_execution() {
    use std::sync::{Arc, Mutex};

    let pool = ThreadPool::new(4);
    let counter = Arc::new(Mutex::new(0));

    for _ in 0..100 {
        let c = counter.clone();
        pool.execute(move || {
            let mut num = c.lock().unwrap();
            *num += 1;
        });
    }

    pool.shutdown();

    assert_eq!(*counter.lock().unwrap(), 100);
}

#[test]
fn test_parallel_speedup() {
    use std::time::Instant;

    let pool = ThreadPool::new(4);
    let start = Instant::now();

    for _ in 0..8 {
        pool.execute(|| {
            thread::sleep(Duration::from_millis(100));
        });
    }

    pool.shutdown();
    let elapsed = start.elapsed();

    // 8 tasks @ 100ms each on 4 workers ≈ 200ms total
    assert!(elapsed < Duration::from_millis(300));
}
```

### Starter Code

```rust
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<JoinHandle<()>>,
    sender: Sender<Job>,
    shutdown: Arc<AtomicBool>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        // TODO: Create channel for jobs
        // Spawn 'size' worker threads
        // Each worker loops: recv job -> execute -> repeat
        // Return ThreadPool with workers and sender
        unimplemented!()
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // TODO: Box closure and send through channel
        // Hint: self.sender.send(Box::new(f)).unwrap()
        unimplemented!()
    }

    pub fn shutdown(self) {
        // TODO: Set shutdown flag
        // Drop sender to close channel
        // Join all worker threads
        unimplemented!()
    }
}

fn worker_loop(receiver: Arc<Mutex<Receiver<Job>>>, shutdown: Arc<AtomicBool>) {
    // TODO: Loop while !shutdown:
    //   - Lock receiver
    //   - Try to recv job (with timeout to check shutdown)
    //   - If job received, execute it
    //   - Drop lock
    unimplemented!()
}
```

#### Why previous Milestone is not enough: N/A - Foundation Milestone.

**What's the improvement:** Thread pool vs spawn-per-task:
- Spawn-per-task: 1000 tasks × 1ms spawn = 1 second overhead
- Thread pool: 0 overhead (threads pre-spawned)

For high-frequency tasks (web requests, image tiles), thread pool is mandatory.

---

## Milestone 2: Image Processing Tasks

### Introduction

Add image processing functionality: load, resize, apply filters, save. Distribute tasks across thread pool workers.

### Architecture

**Structs:**
- `ImageTask` - Processing job
  - **Field** `input_path: PathBuf` - Source image
  - **Field** `output_path: PathBuf` - Destination
  - **Field** `operations: Vec<Operation>` - Transformations to apply

- `Operation` - Transformation enum
  - **Variant** `Resize(u32, u32)` - New dimensions
  - **Variant** `Blur(f32)` - Blur radius
  - **Variant** `Brighten(i32)` - Brightness delta

**Key Functions:**
- `process_image(task: ImageTask) -> Result<(), ImageError>`
- `load_image(path: &Path) -> Result<ImageBuffer, ImageError>`
- `save_image(image: &ImageBuffer, path: &Path) -> Result<(), ImageError>`

### Checkpoint Tests

```rust
#[test]
fn test_image_resize() {
    let task = ImageTask {
        input_path: PathBuf::from("test.png"),
        output_path: PathBuf::from("out.png"),
        operations: vec![Operation::Resize(100, 100)],
    };

    let result = process_image(task);
    assert!(result.is_ok());
}

#[test]
fn test_parallel_processing() {
    let pool = ThreadPool::new(4);
    let counter = Arc::new(AtomicUsize::new(0));

    for i in 0..10 {
        let c = counter.clone();
        pool.execute(move || {
            // Simulate image processing
            thread::sleep(Duration::from_millis(50));
            c.fetch_add(1, Ordering::SeqCst);
        });
    }

    pool.shutdown();
    assert_eq!(counter.load(Ordering::SeqCst), 10);
}
```

### Starter Code

```rust
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Operation {
    Resize(u32, u32),
    Blur(f32),
    Brighten(i32),
}

pub struct ImageTask {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub operations: Vec<Operation>,
}

pub fn process_image(task: ImageTask) -> Result<(), String> {
    // TODO: Load image from input_path
    // Apply each operation in sequence
    // Save to output_path
    // Hint: Use image crate for actual processing
    //   let mut img = image::open(&task.input_path)?;
    //   for op in task.operations {
    //     img = apply_operation(img, op);
    //   }
    //   img.save(&task.output_path)?;
    unimplemented!()
}

fn apply_operation(img: ImageBuffer, op: Operation) -> ImageBuffer {
    // TODO: Match on operation and apply transformation
    // Resize: image::resize()
    // Blur: image::blur()
    // Brighten: image::brighten()
    unimplemented!()
}
```

**Why previous Milestone is not enough:** Thread pool without real work is just overhead. Need actual tasks to process.

**What's the improvement:** Parallel image processing scales linearly:
- Sequential: 10 images × 500ms = 5 seconds
- Parallel (8 cores): 10 images / 8 = ~625ms

For batch processing (thousands of images), parallelism is essential.

---

## Milestone 3: Progress Tracking and Results

### Introduction

Track processing progress and collect results. Report completion percentage, failed tasks, and aggregate statistics.

### Architecture

**Enhanced Structs:**
- `ProcessingResult` - Task outcome
  - **Field** `task_id: usize`
  - **Field** `status: TaskStatus` - Success/Failed
  - **Field** `duration: Duration` - Processing time
  - **Field** `error: Option<String>` - Error message if failed

- `ProgressTracker` - Monitor progress
  - **Field** `total: usize` - Total tasks
  - **Field** `completed: AtomicUsize` - Finished count
  - **Field** `results: Mutex<Vec<ProcessingResult>>`

**Key Functions:**
- `track_progress(tracker: Arc<ProgressTracker>)` - Progress reporter thread
- `wait_for_completion(tracker: Arc<ProgressTracker>) -> Vec<ProcessingResult>`

### Checkpoint Tests

```rust
#[test]
fn test_progress_tracking() {
    let tracker = Arc::new(ProgressTracker::new(10));

    for i in 0..10 {
        let t = tracker.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            t.record_completion(i, TaskStatus::Success, Duration::from_millis(10), None);
        });
    }

    let results = tracker.wait_for_completion();
    assert_eq!(results.len(), 10);
    assert_eq!(tracker.completed.load(Ordering::SeqCst), 10);
}

#[test]
fn test_error_collection() {
    let tracker = Arc::new(ProgressTracker::new(5));

    for i in 0..5 {
        let t = tracker.clone();
        thread::spawn(move || {
            if i % 2 == 0 {
                t.record_completion(i, TaskStatus::Success, Duration::from_millis(10), None);
            } else {
                t.record_completion(
                    i,
                    TaskStatus::Failed,
                    Duration::from_millis(5),
                    Some("Processing error".to_string())
                );
            }
        });
    }

    let results = tracker.wait_for_completion();
    let failed = results.iter().filter(|r| matches!(r.status, TaskStatus::Failed)).count();
    assert_eq!(failed, 2);
}
```

### Starter Code

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum TaskStatus {
    Success,
    Failed,
}

pub struct ProcessingResult {
    pub task_id: usize,
    pub status: TaskStatus,
    pub duration: Duration,
    pub error: Option<String>,
}

pub struct ProgressTracker {
    total: usize,
    completed: AtomicUsize,
    results: Mutex<Vec<ProcessingResult>>,
}

impl ProgressTracker {
    pub fn new(total: usize) -> Self {
        // TODO: Initialize with total count and empty results
        unimplemented!()
    }

    pub fn record_completion(
        &self,
        task_id: usize,
        status: TaskStatus,
        duration: Duration,
        error: Option<String>,
    ) {
        // TODO: Increment completed counter
        // Add result to results vec (with lock)
        unimplemented!()
    }

    pub fn progress_percentage(&self) -> f64 {
        // TODO: Calculate completion percentage
        // Hint: (completed / total) * 100.0
        unimplemented!()
    }

    pub fn wait_for_completion(&self) -> Vec<ProcessingResult> {
        // TODO: Spin until completed == total
        // Return cloned results vec
        unimplemented!()
    }
}

pub fn progress_reporter(tracker: Arc<ProgressTracker>) {
    // TODO: Loop printing progress every 100ms
    // Example: "Progress: 45/100 (45%)"
    // Exit when completed == total
    unimplemented!()
}
```

**Why previous Milestone is not enough:** No visibility into processing status. Users want progress bars and error reports.

**What's the improvement:** Progress tracking enables UX and debugging:
- No tracking: Black box, no idea if hung or processing
- With tracking: Real-time progress, failed task identification

For long-running batch jobs, progress reporting is mandatory.

---

## Milestone 4: Dynamic Task Submission

### Introduction

Support submitting tasks dynamically while processing continues. Add tasks from multiple threads without blocking.

### Architecture

**Enhanced Pool:**
- Allow task submission from any thread
- Handle varying load (elastic work queue)
- Report queue depth for monitoring

**Key Functions:**
- `execute_with_timeout(&self, f: Job, timeout: Duration) -> Result<(), TimeoutError>`
- `queue_depth(&self) -> usize` - Number of pending tasks

### Checkpoint Tests

```rust
#[test]
fn test_dynamic_submission() {
    let pool = ThreadPool::new(4);

    // Submit initial batch
    for i in 0..10 {
        pool.execute(move || println!("Task {}", i));
    }

    // Submit more tasks while processing
    thread::sleep(Duration::from_millis(50));
    for i in 10..20 {
        pool.execute(move || println!("Task {}", i));
    }

    pool.shutdown();
}

#[test]
fn test_multi_threaded_submission() {
    let pool = Arc::new(ThreadPool::new(4));
    let mut submitters = vec![];

    for _ in 0..4 {
        let p = pool.clone();
        let handle = thread::spawn(move || {
            for i in 0..100 {
                p.execute(move || {
                    thread::sleep(Duration::from_micros(10));
                });
            }
        });
        submitters.push(handle);
    }

    for h in submitters {
        h.join().unwrap();
    }
}
```

### Starter Code

```rust
impl ThreadPool {
    pub fn execute_with_timeout<F>(
        &self,
        f: F,
        timeout: Duration,
    ) -> Result<(), String>
    where
        F: FnOnce() + Send + 'static,
    {
        // TODO: Try to send job with timeout
        // Use sync_channel with timeout instead of regular channel
        // Return Err if send times out
        unimplemented!()
    }

    pub fn queue_depth(&self) -> usize {
        // TODO: Track pending tasks
        // Could use Arc<AtomicUsize> incremented on send, decremented on execute
        unimplemented!()
    }

    pub fn active_workers(&self) -> usize {
        // TODO: Track number of workers currently executing
        // Use Arc<AtomicUsize> incremented before execute, decremented after
        unimplemented!()
    }
}
```

**Why previous Milestone is not enough:** Static workload doesn't reflect reality. Real systems have dynamic, unpredictable task arrival.

**What's the improvement:** Dynamic submission enables real-world patterns:
- Web server: New requests arrive while processing existing
- Stream processing: Events arrive continuously
- Adaptive systems: Task generation based on results

---

## Milestone 5: Adaptive Pool Sizing

### Introduction

Automatically adjust worker count based on load. Scale up when queue grows, scale down when idle.

### Architecture

**Adaptive Logic:**
- Monitor queue depth and worker utilization
- Spawn workers if queue > threshold × current_workers
- Terminate idle workers after timeout

**Key Functions:**
- `scale_up(&mut self, count: usize)` - Add workers
- `scale_down(&mut self, count: usize)` - Remove workers
- `auto_scale(&self)` - Background thread monitoring and adjusting

### Checkpoint Tests

```rust
#[test]
fn test_scale_up() {
    let mut pool = ThreadPool::new(2);

    // Submit many tasks to trigger scaling
    for _ in 0..100 {
        pool.execute(|| thread::sleep(Duration::from_millis(10)));
    }

    thread::sleep(Duration::from_millis(50));

    // Pool should have scaled up
    assert!(pool.worker_count() > 2);
}

#[test]
fn test_scale_down() {
    let mut pool = ThreadPool::new(8);

    // Submit few tasks
    for _ in 0..4 {
        pool.execute(|| thread::sleep(Duration::from_millis(10)));
    }

    thread::sleep(Duration::from_secs(2)); // Wait for idle timeout

    // Pool should have scaled down
    assert!(pool.worker_count() < 8);
}
```

### Starter Code

```rust
pub struct AdaptiveThreadPool {
    workers: Arc<Mutex<Vec<JoinHandle<()>>>>,
    sender: Sender<Job>,
    min_workers: usize,
    max_workers: usize,
    queue_threshold: usize,
}

impl AdaptiveThreadPool {
    pub fn new(min: usize, max: usize) -> Self {
        // TODO: Initialize with min workers
        // Spawn monitoring thread for auto-scaling
        unimplemented!()
    }

    pub fn scale_up(&mut self, count: usize) {
        // TODO: Spawn 'count' new workers
        // Don't exceed max_workers
        unimplemented!()
    }

    pub fn scale_down(&mut self, count: usize) {
        // TODO: Signal 'count' workers to exit
        // Don't go below min_workers
        // Use special "exit" message in channel
        unimplemented!()
    }

    pub fn worker_count(&self) -> usize {
        self.workers.lock().unwrap().len()
    }
}

fn auto_scale_monitor(pool: Arc<AdaptiveThreadPool>) {
    // TODO: Loop checking queue depth
    // If queue_depth > threshold * workers: scale_up()
    // If workers idle for > 30s: scale_down()
    unimplemented!()
}
```

**Why previous Milestone is not enough:** Fixed pool size is inefficient. Overprovisioned when idle (waste resources), underprovisioned during peaks (high latency).

**What's the improvement:** Adaptive sizing optimizes resource usage:
- Fixed 100 workers: Wastes 95% resources during low load
- Adaptive 5-100 workers: Scales to load, saves resources

For cloud deployments, adaptive sizing reduces costs by 50-90%.

---

## Milestone 6: Benchmark vs Sequential

### Introduction

Benchmark thread pool against sequential processing. Measure speedup with varying worker counts and task sizes.

### Architecture

**Benchmarks:**
- Fixed workload (1000 tasks)
- Vary task duration: 1ms, 10ms, 100ms
- Vary worker count: 1, 2, 4, 8, 16
- Measure total time and tasks/sec

### Starter Code

```rust
pub struct Benchmark;

impl Benchmark {
    pub fn benchmark_sequential(num_tasks: usize, task_duration: Duration) -> Duration {
        let start = Instant::now();

        for _ in 0..num_tasks {
            thread::sleep(task_duration);
        }

        start.elapsed()
    }

    pub fn benchmark_thread_pool(
        num_tasks: usize,
        num_workers: usize,
        task_duration: Duration,
    ) -> Duration {
        let pool = ThreadPool::new(num_workers);
        let start = Instant::now();

        for _ in 0..num_tasks {
            pool.execute(move || {
                thread::sleep(task_duration);
            });
        }

        pool.shutdown();
        start.elapsed()
    }

    pub fn run_comparison() {
        println!("=== Thread Pool vs Sequential Performance ===\n");

        let num_tasks = 100;
        let task_duration = Duration::from_millis(10);
        let thread_counts = [1, 2, 4, 8];

        let seq_time = Self::benchmark_sequential(num_tasks, task_duration);
        println!("Sequential: {:?}\n", seq_time);

        for &num_threads in &thread_counts {
            let pool_time = Self::benchmark_thread_pool(num_tasks, num_threads, task_duration);
            let speedup = seq_time.as_secs_f64() / pool_time.as_secs_f64();

            println!("Thread Pool ({} workers):", num_threads);
            println!("  Time: {:?}", pool_time);
            println!("  Speedup: {:.2}x\n", speedup);
        }
    }
}
```

**Why previous Milestone is not enough:** Performance claims need validation.

**What's the improvement:** Empirical speedup data:
- 1 worker: 1× (same as sequential)
- 4 workers: 3.8-4× speedup
- 8 workers: 7-8× speedup

Validates parallel efficiency and guides worker count selection.

---

### Complete Working Example

```rust
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<JoinHandle<()>>,
    sender: Sender<Job>,
    shutdown: Arc<AtomicBool>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let (sender, receiver) = channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let shutdown = Arc::new(AtomicBool::new(false));

        let mut workers = Vec::with_capacity(size);

        for _ in 0..size {
            let receiver = receiver.clone();
            let shutdown = shutdown.clone();

            let handle = thread::spawn(move || {
                worker_loop(receiver, shutdown);
            });

            workers.push(handle);
        }

        ThreadPool {
            workers,
            sender,
            shutdown,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(f)).unwrap();
    }

    pub fn shutdown(self) {
        self.shutdown.store(true, Ordering::SeqCst);
        drop(self.sender);

        for worker in self.workers {
            worker.join().unwrap();
        }
    }
}

fn worker_loop(receiver: Arc<Mutex<Receiver<Job>>>, shutdown: Arc<AtomicBool>) {
    loop {
        let job = {
            let receiver = receiver.lock().unwrap();
            receiver.recv()
        };

        match job {
            Ok(job) => job(),
            Err(_) => {
                if shutdown.load(Ordering::SeqCst) {
                    break;
                }
            }
        }
    }
}

fn main() {
    println!("=== Thread Pool Demo ===\n");

    let pool = ThreadPool::new(4);
    let counter = Arc::new(AtomicUsize::new(0));

    println!("Submitting 100 tasks to thread pool with 4 workers...");

    let start = Instant::now();

    for i in 0..100 {
        let c = counter.clone();
        pool.execute(move || {
            // Simulate work
            thread::sleep(Duration::from_millis(10));
            c.fetch_add(1, Ordering::SeqCst);

            if i % 20 == 0 {
                println!("Task {} completed", i);
            }
        });
    }

    pool.shutdown();
    let elapsed = start.elapsed();

    println!("\nAll tasks completed!");
    println!("Total tasks: {}", counter.load(Ordering::SeqCst));
    println!("Time elapsed: {:?}", elapsed);
    println!("Throughput: {:.0} tasks/sec", 100.0 / elapsed.as_secs_f64());
}
```

### Testing Strategies

1. **Concurrency Tests**: Verify thread safety with ThreadSanitizer
2. **Load Tests**: 10K+ tasks, verify no deadlocks
3. **Shutdown Tests**: Clean termination under load
4. **Performance Tests**: Measure speedup vs sequential
5. **Stress Tests**: Rapid task submission from many threads

---

This project comprehensively demonstrates thread pool patterns, from basic implementation through dynamic submission, adaptive sizing, and performance benchmarks.

---

## Project 3: Shared Counter Service with Arc/Mutex

### Problem Statement

Build a multi-threaded counter service using shared state synchronization with Arc and Mutex. The service provides thread-safe increment, decrement, and query operations with high concurrency.

Your counter service should:
- Support concurrent increment/decrement from multiple threads
- Provide atomic read operations
- Track operation statistics (total operations, contention events)
- Optimize for read-heavy workloads using RwLock
- Implement deadlock-free complex operations
- Compare performance: Mutex vs RwLock vs Atomics

### Why It Matters

Shared state is unavoidable in many systems: caches, connection pools, metrics. Mutexes ensure safety but create contention. Understanding when to use Mutex vs RwLock vs Atomics is critical for performance.

For 1M operations with 8 threads:
- Naive Mutex: 500ms (serialized)
- RwLock (90% reads): 100ms (parallel reads)
- AtomicU64: 50ms (lock-free)

Critical for: metrics systems, caches, connection pools, resource managers.

### Use Cases

- Metrics and monitoring systems
- Rate limiters
- Connection pool managers
- Cache implementations
- Resource quota tracking
- Distributed counters

---

## Milestone 1: Basic Arc/Mutex Counter

### Introduction

Implement a thread-safe counter using Arc<Mutex<T>>. Multiple threads can safely increment/decrement through the mutex.

### Architecture

**Structs:**
- `Counter` - Thread-safe counter
  - **Field** `value: Arc<Mutex<i64>>` - Protected counter value

**Key Functions:**
- `new() -> Counter` - Create counter at 0
- `increment(&self)` - Add 1
- `decrement(&self)` - Subtract 1
- `get(&self) -> i64` - Read current value

**Role Each Plays:**
- Arc: Shared ownership across threads
- Mutex: Ensures exclusive access for modifications
- Lock guard: Automatic unlock when dropped

### Checkpoint Tests

```rust
#[test]
fn test_concurrent_increments() {
    let counter = Arc::new(Counter::new());
    let mut handles = vec![];

    for _ in 0..10 {
        let c = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                c.increment();
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(counter.get(), 10_000);
}

#[test]
fn test_mixed_operations() {
    let counter = Arc::new(Counter::new());

    let c1 = counter.clone();
    let h1 = thread::spawn(move || {
        for _ in 0..100 {
            c1.increment();
        }
    });

    let c2 = counter.clone();
    let h2 = thread::spawn(move || {
        for _ in 0..50 {
            c2.decrement();
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();

    assert_eq!(counter.get(), 50);
}
```

### Starter Code

```rust
use std::sync::{Arc, Mutex};

pub struct Counter {
    value: Arc<Mutex<i64>>,
}

impl Counter {
    pub fn new() -> Self {
        // TODO: Initialize with Arc<Mutex<i64>> at 0
        unimplemented!()
    }

    pub fn increment(&self) {
        // TODO: Lock mutex, increment value, unlock automatically
        // Hint: let mut val = self.value.lock().unwrap();
        //       *val += 1;
        unimplemented!()
    }

    pub fn decrement(&self) {
        // TODO: Lock mutex, decrement value
        unimplemented!()
    }

    pub fn get(&self) -> i64 {
        // TODO: Lock mutex, read value, return
        unimplemented!()
    }

    pub fn add(&self, amount: i64) {
        // TODO: Lock once and add amount
        unimplemented!()
    }
}

impl Clone for Counter {
    fn clone(&self) -> Self {
        Counter {
            value: self.value.clone(), // Clone Arc, not Mutex
        }
    }
}
```

**Why previous Milestone is not enough:** N/A - Foundation Milestone.

**What's the improvement:** Arc/Mutex provides safe shared state:
- Unsafe: `static mut COUNTER` - data races, undefined behavior
- Safe: `Arc<Mutex<T>>` - compiler-enforced mutual exclusion

For concurrent counters, Arc/Mutex is the safe default.

---

## Milestone 2: Contention Monitoring

### Introduction

Add metrics to track mutex contention: lock acquisition time, waiting threads, lock hold duration.

### Architecture

**Enhanced Counter:**
- Track lock wait times
- Count contention events (when lock is already held)
- Measure critical section duration

**Structs:**
- `ContentionStats` - Metrics
  - **Field** `total_locks: AtomicU64`
  - **Field** `contention_events: AtomicU64`
  - **Field** `total_wait_time_us: AtomicU64`

### Checkpoint Tests

```rust
#[test]
fn test_contention_tracking() {
    let counter = Arc::new(MonitoredCounter::new());
    let mut handles = vec![];

    // High contention workload
    for _ in 0..8 {
        let c = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                c.increment();
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    let stats = counter.stats();
    println!("Contention events: {}", stats.contention_events);
    println!("Avg wait time: {}μs", stats.avg_wait_time_us());
}
```

### Starter Code

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub struct ContentionStats {
    pub total_locks: AtomicU64,
    pub contention_events: AtomicU64,
    pub total_wait_time_us: AtomicU64,
}

impl ContentionStats {
    pub fn avg_wait_time_us(&self) -> u64 {
        let total = self.total_locks.load(Ordering::Relaxed);
        if total == 0 {
            0
        } else {
            self.total_wait_time_us.load(Ordering::Relaxed) / total
        }
    }
}

pub struct MonitoredCounter {
    value: Arc<Mutex<i64>>,
    stats: Arc<ContentionStats>,
}

impl MonitoredCounter {
    pub fn increment(&self) {
        let start = Instant::now();

        // Try to lock - if contended, record it
        let mut val = self.value.lock().unwrap();

        let wait_time = start.elapsed();
        self.stats.total_locks.fetch_add(1, Ordering::Relaxed);
        self.stats.total_wait_time_us.fetch_add(
            wait_time.as_micros() as u64,
            Ordering::Relaxed
        );

        if wait_time > Duration::from_micros(1) {
            self.stats.contention_events.fetch_add(1, Ordering::Relaxed);
        }

        *val += 1;
    }

    pub fn stats(&self) -> &ContentionStats {
        &self.stats
    }
}
```

**Why previous Milestone is not enough:** Can't optimize without measuring. Contention metrics reveal bottlenecks.

**What's the improvement:** Monitoring enables optimization:
- High contention → Use RwLock or sharding
- Long hold times → Reduce critical section
- Identify hotspots → Targeted optimization

---

## Milestone 3: RwLock for Read-Heavy Workloads

### Introduction

Optimize for read-heavy access patterns using RwLock. Multiple readers can access concurrently, writers get exclusive access.

### Architecture

**RwLock Semantics:**
- Multiple readers simultaneously (shared access)
- Single writer exclusively (exclusive access)
- Readers block writers, writers block everyone

**Comparison:**
- Mutex: All operations serialized
- RwLock: Reads parallel, writes exclusive

### Checkpoint Tests

```rust
#[test]
fn test_concurrent_reads() {
    let counter = Arc::new(RwCounter::new());

    // Set value
    counter.set(42);

    // Spawn many readers
    let mut handles = vec![];
    for _ in 0..10 {
        let c = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                assert_eq!(c.get(), 42);
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn test_read_write_mix() {
    let counter = Arc::new(RwCounter::new());

    // 9 readers, 1 writer
    let mut handles = vec![];

    for _ in 0..9 {
        let c = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                c.get();
            }
        });
        handles.push(handle);
    }

    let c = counter.clone();
    let writer = thread::spawn(move || {
        for _ in 0..100 {
            c.increment();
        }
    });

    handles.push(writer);

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(counter.get(), 100);
}
```

### Starter Code

```rust
use std::sync::RwLock;

pub struct RwCounter {
    value: Arc<RwLock<i64>>,
}

impl RwCounter {
    pub fn new() -> Self {
        // TODO: Initialize with Arc<RwLock<i64>>
        unimplemented!()
    }

    pub fn increment(&self) {
        // TODO: Acquire write lock, increment
        // Hint: let mut val = self.value.write().unwrap();
        //       *val += 1;
        unimplemented!()
    }

    pub fn get(&self) -> i64 {
        // TODO: Acquire read lock, return value
        // Hint: let val = self.value.read().unwrap();
        //       *val
        unimplemented!()
    }

    pub fn set(&self, new_value: i64) {
        // TODO: Acquire write lock, set value
        unimplemented!()
    }
}
```

**Why previous Milestone is not enough:** Mutex serializes all access, even reads. For read-heavy workloads (90%+ reads), this wastes concurrency.

**What's the improvement:** RwLock enables parallel reads:
- Mutex (90% reads): 1× throughput (all serialized)
- RwLock (90% reads): 8× throughput (reads parallel)

For caches and metrics, RwLock is often 5-10× faster.

---

## Milestone 4: Lock-Free with Atomics

### Introduction

Eliminate locks entirely using atomic operations. AtomicU64 provides lock-free increment/decrement with fetch_add.

### Architecture

**Atomic Operations:**
- `fetch_add`: Atomically add and return old value
- `fetch_sub`: Atomically subtract
- `load`: Read current value
- `store`: Write new value

**Memory Ordering:**
- `Relaxed`: No synchronization (fastest)
- `Acquire/Release`: Synchronizes with other operations
- `SeqCst`: Strongest guarantees (slowest)

### Checkpoint Tests

```rust
#[test]
fn test_atomic_counter() {
    let counter = Arc::new(AtomicCounter::new());
    let mut handles = vec![];

    for _ in 0..10 {
        let c = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..10000 {
                c.increment();
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(counter.get(), 100_000);
}

#[test]
fn test_atomic_performance() {
    let counter = Arc::new(AtomicCounter::new());
    let start = Instant::now();

    let mut handles = vec![];
    for _ in 0..8 {
        let c = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100_000 {
                c.increment();
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    let elapsed = start.elapsed();
    println!("Atomic: {}μs per op", elapsed.as_micros() / 800_000);
}
```

### Starter Code

```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct AtomicCounter {
    value: Arc<AtomicU64>,
}

impl AtomicCounter {
    pub fn new() -> Self {
        // TODO: Initialize with Arc<AtomicU64>
        unimplemented!()
    }

    pub fn increment(&self) {
        // TODO: Use fetch_add with Relaxed ordering
        // Hint: self.value.fetch_add(1, Ordering::Relaxed);
        unimplemented!()
    }

    pub fn decrement(&self) {
        // TODO: Use fetch_sub
        unimplemented!()
    }

    pub fn get(&self) -> u64 {
        // TODO: Use load
        unimplemented!()
    }

    pub fn add(&self, amount: u64) {
        self.value.fetch_add(amount, Ordering::Relaxed);
    }
}
```

**Why previous Milestone is not enough:** Even RwLock has overhead (syscalls, context switches). Atomics are lock-free and fastest.

**What's the improvement:** Atomics provide maximum throughput:
- Mutex: 2-5μs per operation
- RwLock: 1-3μs per operation (reads)
- Atomic: 0.01-0.1μs per operation (100× faster!)

For high-frequency counters (metrics, rate limiters), atomics are mandatory.

---

## Milestone 5: Deadlock Prevention

### Introduction

Implement complex operations safely without deadlocks. Use lock ordering, try_lock, and timeout patterns.

### Architecture

**Deadlock Scenarios:**
1. **Lock ordering**: Thread A locks M1→M2, Thread B locks M2→M1
2. **Nested locks**: Function calls itself, tries to reacquire same lock
3. **Circular wait**: A waits for B, B waits for C, C waits for A

**Prevention Strategies:**
- **Lock ordering**: Always acquire locks in consistent order
- **Try-lock**: Don't block, retry later if lock unavailable
- **Timeout**: Give up after deadline

### Checkpoint Tests

```rust
#[test]
fn test_transfer_no_deadlock() {
    let counter1 = Arc::new(Counter::new());
    let counter2 = Arc::new(Counter::new());

    counter1.add(100);
    counter2.add(50);

    let c1 = counter1.clone();
    let c2 = counter2.clone();
    let h1 = thread::spawn(move || {
        for _ in 0..100 {
            transfer(&c1, &c2, 1);
        }
    });

    let c1 = counter1.clone();
    let c2 = counter2.clone();
    let h2 = thread::spawn(move || {
        for _ in 0..100 {
            transfer(&c2, &c1, 1);
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();

    // Total should be conserved
    assert_eq!(counter1.get() + counter2.get(), 150);
}
```

### Starter Code

```rust
pub fn transfer(from: &Counter, to: &Counter, amount: i64) -> Result<(), String> {
    // TODO: Implement deadlock-free transfer
    // Strategy 1: Lock ordering - always lock lower address first
    // Strategy 2: Try-lock with retry
    // Strategy 3: Use global lock for multi-resource operations

    // Lock ordering approach:
    let (first, second) = if (from as *const Counter) < (to as *const Counter) {
        (from, to)
    } else {
        (to, from)
    };

    // TODO: Lock first, then second
    // Subtract from 'from', add to 'to'
    // Check balance before transfer

    unimplemented!()
}

pub fn try_transfer_with_timeout(
    from: &Counter,
    to: &Counter,
    amount: i64,
    timeout: Duration,
) -> Result<(), String> {
    // TODO: Use try_lock with timeout
    // Retry until timeout expires
    // Hint: Use Instant::now() and loop with try_lock()
    unimplemented!()
}
```

**Why previous Milestone is not enough:** Simple operations don't reveal deadlock risks. Complex operations (transfers, swaps) need careful design.

**What's the improvement:** Deadlock prevention ensures progress:
- No prevention: System hangs, requires restart
- With prevention: Operations always complete (or fail gracefully)

For production systems, deadlock freedom is mandatory.

---

## Milestone 6: Performance Comparison

### Introduction

Benchmark all approaches: Mutex vs RwLock vs Atomic. Measure throughput under different read/write ratios.

### Architecture

**Benchmarks:**
- Vary read/write ratio: 50/50, 70/30, 90/10, 99/1
- Vary thread count: 1, 2, 4, 8, 16
- Fixed workload: 1M operations

### Starter Code

```rust
pub struct Benchmark;

impl Benchmark {
    pub fn benchmark_mutex(num_ops: usize, num_threads: usize, read_ratio: f64) -> Duration {
        let counter = Arc::new(Counter::new());
        let start = Instant::now();

        let mut handles = vec![];
        for _ in 0..num_threads {
            let c = counter.clone();
            let ops_per_thread = num_ops / num_threads;
            let handle = thread::spawn(move || {
                for _ in 0..ops_per_thread {
                    if rand::random::<f64>() < read_ratio {
                        c.get();
                    } else {
                        c.increment();
                    }
                }
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().unwrap();
        }

        start.elapsed()
    }

    pub fn benchmark_rwlock(num_ops: usize, num_threads: usize, read_ratio: f64) -> Duration {
        // TODO: Similar to mutex but use RwCounter
        unimplemented!()
    }

    pub fn benchmark_atomic(num_ops: usize, num_threads: usize) -> Duration {
        // TODO: Use AtomicCounter (no read/write distinction)
        unimplemented!()
    }

    pub fn run_comparison() {
        println!("=== Synchronization Performance Comparison ===\n");

        let num_ops = 1_000_000;
        let num_threads = 8;

        for read_ratio in [0.5, 0.7, 0.9, 0.99] {
            println!("Read ratio: {:.0}%", read_ratio * 100.0);

            let mutex_time = Self::benchmark_mutex(num_ops, num_threads, read_ratio);
            let rwlock_time = Self::benchmark_rwlock(num_ops, num_threads, read_ratio);
            let atomic_time = Self::benchmark_atomic(num_ops, num_threads);

            println!("  Mutex:   {:?}", mutex_time);
            println!("  RwLock:  {:?} ({:.2}x)", rwlock_time, mutex_time.as_secs_f64() / rwlock_time.as_secs_f64());
            println!("  Atomic:  {:?} ({:.2}x)\n", atomic_time, mutex_time.as_secs_f64() / atomic_time.as_secs_f64());
        }
    }
}
```

**Why previous Milestone is not enough:** Need empirical data to choose synchronization primitive.

**What's the improvement:** Measured performance guides design:
- 50% reads: Mutex ≈ RwLock (frequent writes block readers)
- 90% reads: RwLock 5× faster than Mutex
- 99% reads: RwLock 10× faster, Atomic 100× faster

For high-contention read-heavy workloads, atomics provide orders of magnitude improvement.

---

### Complete Working Example

```rust
use std::sync::{Arc, Mutex, RwLock, atomic::{AtomicU64, Ordering}};
use std::thread;
use std::time::{Duration, Instant};

// Mutex-based counter
pub struct MutexCounter {
    value: Arc<Mutex<i64>>,
}

impl MutexCounter {
    pub fn new() -> Self {
        MutexCounter {
            value: Arc::new(Mutex::new(0)),
        }
    }

    pub fn increment(&self) {
        let mut val = self.value.lock().unwrap();
        *val += 1;
    }

    pub fn get(&self) -> i64 {
        *self.value.lock().unwrap()
    }
}

impl Clone for MutexCounter {
    fn clone(&self) -> Self {
        MutexCounter {
            value: self.value.clone(),
        }
    }
}

// RwLock-based counter
pub struct RwCounter {
    value: Arc<RwLock<i64>>,
}

impl RwCounter {
    pub fn new() -> Self {
        RwCounter {
            value: Arc::new(RwLock::new(0)),
        }
    }

    pub fn increment(&self) {
        let mut val = self.value.write().unwrap();
        *val += 1;
    }

    pub fn get(&self) -> i64 {
        *self.value.read().unwrap()
    }
}

impl Clone for RwCounter {
    fn clone(&self) -> Self {
        RwCounter {
            value: self.value.clone(),
        }
    }
}

// Atomic counter
pub struct AtomicCounter {
    value: Arc<AtomicU64>,
}

impl AtomicCounter {
    pub fn new() -> Self {
        AtomicCounter {
            value: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn increment(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

impl Clone for AtomicCounter {
    fn clone(&self) -> Self {
        AtomicCounter {
            value: self.value.clone(),
        }
    }
}

fn main() {
    println!("=== Shared Counter Service Demo ===\n");

    // Mutex counter
    println!("1. Mutex Counter:");
    let mutex_counter = Arc::new(MutexCounter::new());
    let mut handles = vec![];

    for i in 0..4 {
        let c = mutex_counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                c.increment();
            }
            println!("  Thread {} completed", i);
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    println!("Final count: {}\n", mutex_counter.get());

    // RwLock counter
    println!("2. RwLock Counter (read-heavy):");
    let rw_counter = Arc::new(RwCounter::new());
    let mut handles = vec![];

    // 1 writer
    let c = rw_counter.clone();
    let writer = thread::spawn(move || {
        for _ in 0..1000 {
            c.increment();
        }
    });
    handles.push(writer);

    // 10 readers
    for i in 0..10 {
        let c = rw_counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                let _ = c.get();
            }
            println!("  Reader {} completed", i);
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    println!("Final count: {}\n", rw_counter.get());

    // Atomic counter
    println!("3. Atomic Counter:");
    let atomic_counter = Arc::new(AtomicCounter::new());
    let mut handles = vec![];

    let start = Instant::now();

    for i in 0..8 {
        let c = atomic_counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100_000 {
                c.increment();
            }
            println!("  Thread {} completed", i);
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    let elapsed = start.elapsed();

    println!("Final count: {}", atomic_counter.get());
    println!("Time: {:?}", elapsed);
    println!("Throughput: {:.0} ops/sec", 800_000.0 / elapsed.as_secs_f64());
}
```

### Testing Strategies

1. **Correctness Tests**: Verify final counter values
2. **Concurrency Tests**: High thread count, verify no races
3. **Deadlock Tests**: Complex operations (transfer, swap)
4. **Performance Tests**: Compare Mutex/RwLock/Atomic
5. **Stress Tests**: 1M+ operations, sustained load

---

This project comprehensively demonstrates shared state synchronization patterns, from basic Mutex through RwLock optimization, lock-free atomics, deadlock prevention, and performance benchmarks comparing all approaches.

---

**All three Chapter 14 projects demonstrate:**
1. Message passing with channels (Project 1)
2. Thread pools for parallel work (Project 2)
3. Shared state synchronization (Project 3)

Each includes 6 progressive Milestones, checkpoint tests, starter code, complete working examples, and performance benchmarks.
