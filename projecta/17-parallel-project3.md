# Chapter 17: Parallel Algorithms - Map-Reduce Framework for Distributed Log Analysis

## Project: Production-Grade Map-Reduce Framework

### Problem Statement

Build a scalable map-reduce framework for distributed log analysis, implementing the classic data parallelism pattern used by Hadoop and Spark. The system must efficiently process gigabytes of log data, supporting filtering, aggregation, and multi-stage pipelines.

The framework must:
- Parse and process log files (web server, application logs)
- Execute parallel map operations across data chunks
- Shuffle and partition intermediate results by key
- Perform parallel reduce aggregations
- Optimize with combiners to reduce data movement
- Support multi-stage map-reduce pipelines
- Scale to multi-core systems and large datasets

### Use Cases

- **Web Server Analytics**: Count requests by endpoint, analyze HTTP status codes
- **Application Monitoring**: Aggregate errors by type, track performance metrics
- **Security Analysis**: Detect anomalies, count failed login attempts
- **Business Intelligence**: User activity tracking, conversion funnel analysis
- **ETL Pipelines**: Transform, aggregate, and load data for analytics
- **Distributed Systems**: Real-time log aggregation from multiple services

### Why It Matters

**Performance Impact:**
- Sequential processing: O(n) single-threaded - slow for GB+ files
- Parallel map-reduce: O(n/p) where p = cores - 8-16x speedup
- Combiner optimization: Reduces shuffle data by 50-90%

**Real-World Scale:**
- Typical web server: 10-100 GB logs/day
- Large services: 1-10 TB logs/day (Google, AWS)
- Map-reduce enables: Processing terabytes in minutes vs hours

**Why Map-Reduce:**
```
Sequential: Process 100 GB in 30 minutes (single core)
Parallel:   Process 100 GB in 2-4 minutes (16 cores)
Combiner:   Reduce shuffle from 10 GB to 1 GB (10x less network)
```

Example:
```
Log: "GET /api/users 200 123ms"
Map: (endpoint, count) � [("/api/users", 1), ("/api/users", 1), ...]
Shuffle: Group by key � {"/api/users": [1, 1, 1, ...]}
Reduce: Sum counts � {"/api/users": 15234}
```

**Optimization Importance:**
Processing 1 TB of logs:
- No combiner: Shuffle 100 GB � 10 min network transfer
- With combiner: Shuffle 10 GB � 1 min network transfer
- 10x faster pipeline!

---

## Milestone 1: Sequential Log Processor

### Introduction

Implement a basic sequential log processor that parses, filters, and counts log entries. This establishes the foundation for data structures and operations before introducing parallelism.

Sequential processing is simple but inefficient for large datasets. It serves as the baseline to measure parallel speedup.

### Architecture

**Structs:**
- `LogEntry` - Parsed log line
  - **Field** `timestamp: String` - ISO 8601 timestamp
  - **Field** `level: LogLevel` - INFO, WARN, ERROR, DEBUG
  - **Field** `endpoint: String` - HTTP endpoint or service name
  - **Field** `status_code: u16` - HTTP status code
  - **Field** `response_time_ms: u64` - Response time in milliseconds
  - **Field** `user_id: Option<String>` - User identifier
  - **Function** `parse(line: &str) -> Result<Self, ParseError>` - Parse log line

- `LogLevel` - Enum for log levels
  - **Variant** `DEBUG` - Debug messages
  - **Variant** `INFO` - Informational
  - **Variant** `WARN` - Warnings
  - **Variant** `ERROR` - Errors

- `LogProcessor` - Sequential processor
  - **Function** `filter(&self, entries: Vec<LogEntry>, predicate: F) -> Vec<LogEntry>` - Filter entries
  - **Function** `map<K, V>(&self, entries: Vec<LogEntry>, mapper: F) -> Vec<(K, V)>` - Map to key-value pairs
  - **Function** `reduce<K, V>(&self, pairs: Vec<(K, V)>, reducer: F) -> HashMap<K, V>` - Reduce by key
  - **Function** `count_by_endpoint(&self, entries: Vec<LogEntry>) -> HashMap<String, u64>` - Count per endpoint
  - **Function** `average_response_time(&self, entries: Vec<LogEntry>) -> HashMap<String, f64>` - Avg response time

**Role Each Plays:**
- LogEntry: Structured representation of log line
- LogLevel: Type-safe log level handling
- LogProcessor: Sequential operations baseline
- Map: Transform entries to key-value pairs
- Reduce: Aggregate values by key

### Checkpoint Tests

```rust
#[test]
fn test_log_parsing() {
    let line = "2024-01-15T10:30:00Z INFO GET /api/users 200 45ms user=alice";
    let entry = LogEntry::parse(line).unwrap();

    assert_eq!(entry.level, LogLevel::INFO);
    assert_eq!(entry.endpoint, "/api/users");
    assert_eq!(entry.status_code, 200);
    assert_eq!(entry.response_time_ms, 45);
}

#[test]
fn test_sequential_filter() {
    let processor = LogProcessor::new();
    let entries = vec![
        LogEntry { level: LogLevel::ERROR, endpoint: "/api/login".into(), ..Default::default() },
        LogEntry { level: LogLevel::INFO, endpoint: "/api/users".into(), ..Default::default() },
        LogEntry { level: LogLevel::ERROR, endpoint: "/api/checkout".into(), ..Default::default() },
    ];

    let errors = processor.filter(entries, |e| e.level == LogLevel::ERROR);
    assert_eq!(errors.len(), 2);
}

#[test]
fn test_sequential_map() {
    let processor = LogProcessor::new();
    let entries = vec![
        LogEntry { endpoint: "/api/users".into(), ..Default::default() },
        LogEntry { endpoint: "/api/login".into(), ..Default::default() },
        LogEntry { endpoint: "/api/users".into(), ..Default::default() },
    ];

    let pairs = processor.map(entries, |e| (e.endpoint.clone(), 1u64));
    assert_eq!(pairs.len(), 3);
}

#[test]
fn test_sequential_reduce() {
    let processor = LogProcessor::new();
    let pairs = vec![
        ("/api/users".to_string(), 1u64),
        ("/api/login".to_string(), 1u64),
        ("/api/users".to_string(), 1u64),
    ];

    let counts = processor.reduce(pairs, |acc, val| acc + val);
    assert_eq!(counts.get("/api/users"), Some(&2));
    assert_eq!(counts.get("/api/login"), Some(&1));
}

#[test]
fn test_count_by_endpoint() {
    let processor = LogProcessor::new();
    let entries = vec![
        LogEntry { endpoint: "/api/users".into(), ..Default::default() },
        LogEntry { endpoint: "/api/users".into(), ..Default::default() },
        LogEntry { endpoint: "/api/login".into(), ..Default::default() },
    ];

    let counts = processor.count_by_endpoint(entries);
    assert_eq!(counts.get("/api/users"), Some(&2));
    assert_eq!(counts.get("/api/login"), Some(&1));
}

#[test]
fn test_average_response_time() {
    let processor = LogProcessor::new();
    let entries = vec![
        LogEntry { endpoint: "/api/users".into(), response_time_ms: 50, ..Default::default() },
        LogEntry { endpoint: "/api/users".into(), response_time_ms: 100, ..Default::default() },
        LogEntry { endpoint: "/api/login".into(), response_time_ms: 200, ..Default::default() },
    ];

    let avg = processor.average_response_time(entries);
    assert_eq!(avg.get("/api/users"), Some(&75.0));
    assert_eq!(avg.get("/api/login"), Some(&200.0));
}
```

### Starter Code

```rust
use std::collections::HashMap;
use std::fmt;

// ============================================================================
// LOG LEVEL
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::DEBUG => write!(f, "DEBUG"),
            LogLevel::INFO => write!(f, "INFO"),
            LogLevel::WARN => write!(f, "WARN"),
            LogLevel::ERROR => write!(f, "ERROR"),
        }
    }
}

// ============================================================================
// LOG ENTRY
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub endpoint: String,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub user_id: Option<String>,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::INFO
    }
}

#[derive(Debug)]
pub struct ParseError(String);

impl LogEntry {
    pub fn parse(line: &str) -> Result<Self, ParseError> {
        // TODO: Parse log line
        // Format: "2024-01-15T10:30:00Z INFO GET /api/users 200 45ms user=alice"
        //
        // Split by whitespace and extract fields:
        // 1. Timestamp (ISO 8601)
        // 2. Level (INFO, WARN, ERROR, DEBUG)
        // 3. HTTP method (skip)
        // 4. Endpoint
        // 5. Status code
        // 6. Response time (parse number from "45ms")
        // 7. User (optional, parse from "user=alice")
        //
        // let parts: Vec<&str> = line.split_whitespace().collect();
        // if parts.len() < 6 {
        //     return Err(ParseError("Invalid log format".into()));
        // }
        //
        // let level = match parts[1] {
        //     "DEBUG" => LogLevel::DEBUG,
        //     "INFO" => LogLevel::INFO,
        //     "WARN" => LogLevel::WARN,
        //     "ERROR" => LogLevel::ERROR,
        //     _ => return Err(ParseError("Invalid log level".into())),
        // };
        //
        // let response_time = parts[5]
        //     .trim_end_matches("ms")
        //     .parse()
        //     .map_err(|_| ParseError("Invalid response time".into()))?;
        //
        // let user_id = parts.get(6).and_then(|s| {
        //     s.strip_prefix("user=").map(|u| u.to_string())
        // });
        //
        // Ok(LogEntry {
        //     timestamp: parts[0].to_string(),
        //     level,
        //     endpoint: parts[3].to_string(),
        //     status_code: parts[4].parse().map_err(|_| ParseError("Invalid status code".into()))?,
        //     response_time_ms: response_time,
        //     user_id,
        // })
        todo!()
    }
}

// ============================================================================
// SEQUENTIAL LOG PROCESSOR
// ============================================================================

pub struct LogProcessor;

impl LogProcessor {
    pub fn new() -> Self {
        Self
    }

    pub fn filter<F>(&self, entries: Vec<LogEntry>, predicate: F) -> Vec<LogEntry>
    where
        F: Fn(&LogEntry) -> bool,
    {
        // TODO: Filter entries based on predicate
        // entries.into_iter().filter(predicate).collect()
        todo!()
    }

    pub fn map<K, V, F>(&self, entries: Vec<LogEntry>, mapper: F) -> Vec<(K, V)>
    where
        F: Fn(&LogEntry) -> (K, V),
    {
        // TODO: Map entries to key-value pairs
        // entries.iter().map(mapper).collect()
        todo!()
    }

    pub fn reduce<K, V, F>(&self, pairs: Vec<(K, V)>, reducer: F) -> HashMap<K, V>
    where
        K: std::hash::Hash + Eq,
        F: Fn(V, V) -> V,
    {
        // TODO: Reduce pairs by key
        //
        // Group by key and apply reducer function
        //
        // let mut result = HashMap::new();
        // for (key, value) in pairs {
        //     result.entry(key)
        //         .and_modify(|v| *v = reducer(*v, value))
        //         .or_insert(value);
        // }
        // result
        todo!()
    }

    pub fn count_by_endpoint(&self, entries: Vec<LogEntry>) -> HashMap<String, u64> {
        // TODO: Count entries per endpoint
        //
        // Use map + reduce pattern:
        // 1. Map each entry to (endpoint, 1)
        // 2. Reduce by summing counts
        //
        // let pairs = self.map(entries, |e| (e.endpoint.clone(), 1u64));
        // self.reduce(pairs, |a, b| a + b)
        todo!()
    }

    pub fn average_response_time(&self, entries: Vec<LogEntry>) -> HashMap<String, f64> {
        // TODO: Calculate average response time per endpoint
        //
        // Strategy:
        // 1. Map to (endpoint, (sum, count))
        // 2. Reduce by adding sums and counts
        // 3. Divide sum by count to get average
        //
        // let pairs = self.map(entries, |e| {
        //     (e.endpoint.clone(), (e.response_time_ms as f64, 1u64))
        // });
        //
        // let aggregated = self.reduce(pairs, |(sum_a, count_a), (sum_b, count_b)| {
        //     (sum_a + sum_b, count_a + count_b)
        // });
        //
        // aggregated.into_iter()
        //     .map(|(k, (sum, count))| (k, sum / count as f64))
        //     .collect()
        todo!()
    }
}
```

---

## Milestone 2: Parallel Map Phase

### Introduction

**Why Milestone 1 Is Not Enough:**
Sequential processing doesn't utilize multiple CPU cores. For a 10 GB log file on an 8-core machine, we're using only 12.5% of available compute power.

**What We're Improving:**
Implement parallel map phase using Rayon. Split input into chunks, process each chunk on a separate thread, then merge results.

**Performance:**
```
Sequential: 10 GB in 60 seconds (single core)
Parallel:   10 GB in 8 seconds (8 cores) - 7.5x speedup
```

### Architecture

**Dependencies:**
```toml
[dependencies]
rayon = "1.8"
num_cpus = "1.16"
```

**Modified Structs:**
- `ParallelMapReduce` - Parallel map-reduce framework
  - **Field** `chunk_size: usize` - Number of entries per chunk
  - **Function** `parallel_map<K, V>(&self, entries: Vec<LogEntry>, mapper: F) -> Vec<(K, V)>` - Parallel map
  - **Function** `chunk_data(&self, entries: Vec<LogEntry>) -> Vec<Vec<LogEntry>>` - Split into chunks

**Key Functions:**
- `chunk_data`: Divide dataset into equal-sized chunks
- `parallel_map`: Process chunks in parallel using Rayon
- Merge results from all threads

**Role Each Plays:**
- Chunking: Divide work for parallelism
- Parallel map: Execute mapper on each chunk concurrently
- Thread pool: Rayon manages thread creation and scheduling

### Checkpoint Tests

```rust
#[test]
fn test_chunking() {
    let mr = ParallelMapReduce::new(1000);
    let entries = (0..10000).map(|i| LogEntry {
        endpoint: format!("/api/{}", i % 10),
        ..Default::default()
    }).collect();

    let chunks = mr.chunk_data(entries);
    assert_eq!(chunks.len(), 10); // 10000 / 1000 = 10 chunks
    assert_eq!(chunks[0].len(), 1000);
}

#[test]
fn test_parallel_map() {
    let mr = ParallelMapReduce::new(100);
    let entries = (0..1000).map(|i| LogEntry {
        endpoint: format!("/api/{}", i % 10),
        ..Default::default()
    }).collect();

    let pairs = mr.parallel_map(entries, |e| (e.endpoint.clone(), 1u64));
    assert_eq!(pairs.len(), 1000);
}

#[test]
fn test_parallel_correctness() {
    let mr = ParallelMapReduce::new(100);
    let entries = vec![
        LogEntry { endpoint: "/api/users".into(), ..Default::default() },
        LogEntry { endpoint: "/api/login".into(), ..Default::default() },
        LogEntry { endpoint: "/api/users".into(), ..Default::default() },
    ];

    let pairs = mr.parallel_map(entries, |e| (e.endpoint.clone(), 1u64));

    // Count should match sequential version
    let mut counts = HashMap::new();
    for (k, v) in pairs {
        *counts.entry(k).or_insert(0) += v;
    }

    assert_eq!(counts.get("/api/users"), Some(&2));
    assert_eq!(counts.get("/api/login"), Some(&1));
}

#[test]
fn test_parallel_speedup() {
    use std::time::Instant;

    let entries: Vec<LogEntry> = (0..100000).map(|i| LogEntry {
        endpoint: format!("/api/{}", i % 100),
        response_time_ms: i as u64,
        ..Default::default()
    }).collect();

    // Sequential
    let processor = LogProcessor::new();
    let start = Instant::now();
    let seq_result = processor.map(entries.clone(), |e| (e.endpoint.clone(), 1u64));
    let seq_time = start.elapsed();

    // Parallel
    let mr = ParallelMapReduce::new(1000);
    let start = Instant::now();
    let par_result = mr.parallel_map(entries, |e| (e.endpoint.clone(), 1u64));
    let par_time = start.elapsed();

    println!("Sequential: {:?}", seq_time);
    println!("Parallel: {:?}", par_time);
    println!("Speedup: {:.2}x", seq_time.as_secs_f64() / par_time.as_secs_f64());

    assert_eq!(seq_result.len(), par_result.len());
}
```

### Starter Code

```rust
use rayon::prelude::*;

pub struct ParallelMapReduce {
    chunk_size: usize,
}

impl ParallelMapReduce {
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }

    pub fn chunk_data(&self, entries: Vec<LogEntry>) -> Vec<Vec<LogEntry>> {
        // TODO: Split entries into chunks
        //
        // entries.chunks(self.chunk_size)
        //     .map(|chunk| chunk.to_vec())
        //     .collect()
        todo!()
    }

    pub fn parallel_map<K, V, F>(&self, entries: Vec<LogEntry>, mapper: F) -> Vec<(K, V)>
    where
        K: Send,
        V: Send,
        F: Fn(&LogEntry) -> (K, V) + Sync + Send,
    {
        // TODO: Parallel map using Rayon
        //
        // 1. Chunk data
        // 2. Process each chunk in parallel
        // 3. Flatten results
        //
        // let chunks = self.chunk_data(entries);
        //
        // chunks.par_iter()
        //     .flat_map(|chunk| {
        //         chunk.iter().map(&mapper).collect::<Vec<_>>()
        //     })
        //     .collect()
        todo!()
    }
}
```

---

## Milestone 3: Shuffle/Partition Phase

### Introduction

**Why Milestone 2 Is Not Enough:**
Parallel map produces unordered key-value pairs scattered across threads. We need to group by key before reducing. This "shuffle" phase is critical for correctness.

**What We're Improving:**
Implement hash-based partitioning to group pairs by key. Use deterministic hashing to ensure same keys go to same partition.

**Partitioning Strategy:**
```
Input:  [("a", 1), ("b", 2), ("a", 3), ("c", 4), ("b", 5)]
Hash:   hash("a") % 3 = 0, hash("b") % 3 = 1, hash("c") % 3 = 2
Partition 0: [("a", 1), ("a", 3)]
Partition 1: [("b", 2), ("b", 5)]
Partition 2: [("c", 4)]
```

### Architecture

**Structs:**
- `Partitioner` - Hash-based partitioning
  - **Field** `num_partitions: usize` - Number of partitions
  - **Function** `partition<K, V>(&self, pairs: Vec<(K, V)>) -> Vec<Vec<(K, V)>>` - Hash partition
  - **Function** `hash_key(&self, key: &K) -> usize` - Compute partition index

- `ParallelMapReduce` (extended)
  - **Function** `shuffle<K, V>(&self, pairs: Vec<(K, V)>) -> Vec<HashMap<K, Vec<V>>>` - Group by key per partition

**Key Functions:**
- Hash partitioning: `partition_id = hash(key) % num_partitions`
- Group by key: Collect all values for each key in partition
- Deterministic: Same key always goes to same partition

**Role Each Plays:**
- Partitioner: Distribute keys across partitions
- Hash function: Ensure even distribution
- Grouping: Prepare for reduce phase

### Checkpoint Tests

```rust
#[test]
fn test_partitioning() {
    let partitioner = Partitioner::new(4);
    let pairs = vec![
        ("key1".to_string(), 1),
        ("key2".to_string(), 2),
        ("key1".to_string(), 3),
        ("key3".to_string(), 4),
    ];

    let partitions = partitioner.partition(pairs);
    assert_eq!(partitions.len(), 4);

    // All "key1" should be in same partition
    let key1_partition = partitions.iter()
        .find(|p| p.iter().any(|(k, _)| k == "key1"))
        .unwrap();

    let key1_count = key1_partition.iter()
        .filter(|(k, _)| k == "key1")
        .count();
    assert_eq!(key1_count, 2);
}

#[test]
fn test_shuffle_grouping() {
    let mr = ParallelMapReduce::new(100);
    let pairs = vec![
        ("key1".to_string(), 1),
        ("key2".to_string(), 2),
        ("key1".to_string(), 3),
        ("key2".to_string(), 4),
    ];

    let partitions = mr.shuffle(pairs);

    // Each partition should have grouped values by key
    for partition in &partitions {
        for (key, values) in partition {
            // All values for a key should be in one partition
            match key.as_str() {
                "key1" => assert_eq!(values.len(), 2),
                "key2" => assert_eq!(values.len(), 2),
                _ => {}
            }
        }
    }
}

#[test]
fn test_hash_determinism() {
    let partitioner = Partitioner::new(8);

    // Same key should always hash to same partition
    let key = "test_key";
    let hash1 = partitioner.hash_key(&key.to_string());
    let hash2 = partitioner.hash_key(&key.to_string());

    assert_eq!(hash1, hash2);
}

#[test]
fn test_even_distribution() {
    let partitioner = Partitioner::new(8);

    // Generate many keys and check distribution
    let pairs: Vec<(String, u64)> = (0..10000)
        .map(|i| (format!("key{}", i), i))
        .collect();

    let partitions = partitioner.partition(pairs);

    // Each partition should have roughly equal number of items
    let avg = 10000 / 8;
    for partition in &partitions {
        let count = partition.len();
        assert!(count > avg / 2 && count < avg * 2, "Partition size: {}", count);
    }
}
```

### Starter Code

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct Partitioner {
    num_partitions: usize,
}

impl Partitioner {
    pub fn new(num_partitions: usize) -> Self {
        Self { num_partitions }
    }

    pub fn hash_key<K: Hash>(&self, key: &K) -> usize {
        // TODO: Compute hash and modulo for partition index
        //
        // let mut hasher = DefaultHasher::new();
        // key.hash(&mut hasher);
        // (hasher.finish() as usize) % self.num_partitions
        todo!()
    }

    pub fn partition<K, V>(&self, pairs: Vec<(K, V)>) -> Vec<Vec<(K, V)>>
    where
        K: Hash + Clone,
    {
        // TODO: Partition pairs by key hash
        //
        // 1. Create empty partitions
        // 2. For each pair, compute partition index
        // 3. Add pair to appropriate partition
        //
        // let mut partitions: Vec<Vec<(K, V)>> = (0..self.num_partitions)
        //     .map(|_| Vec::new())
        //     .collect();
        //
        // for (key, value) in pairs {
        //     let idx = self.hash_key(&key);
        //     partitions[idx].push((key, value));
        // }
        //
        // partitions
        todo!()
    }
}

impl ParallelMapReduce {
    pub fn shuffle<K, V>(&self, pairs: Vec<(K, V)>) -> Vec<HashMap<K, Vec<V>>>
    where
        K: Hash + Eq + Clone,
    {
        // TODO: Partition and group by key
        //
        // 1. Partition pairs by hash
        // 2. For each partition, group values by key
        //
        // let partitioner = Partitioner::new(num_cpus::get());
        // let partitions = partitioner.partition(pairs);
        //
        // partitions.into_iter()
        //     .map(|partition| {
        //         let mut grouped = HashMap::new();
        //         for (key, value) in partition {
        //             grouped.entry(key).or_insert_with(Vec::new).push(value);
        //         }
        //         grouped
        //     })
        //     .collect()
        todo!()
    }
}
```

---

## Milestone 4: Parallel Reduce Phase

### Introduction

**Why Milestone 3 Is Not Enough:**
After shuffling, we have partitions of grouped data, but reduction is still sequential. We need parallel reduction to fully utilize cores.

**What We're Improving:**
Execute reduce operations in parallel across partitions. Each partition is independent, so they can be reduced concurrently.

**Parallel Strategy:**
```
Partition 0: {"a": [1, 3]} → reduce → {"a": 4}  (Thread 0)
Partition 1: {"b": [2, 5]} → reduce → {"b": 7}  (Thread 1)
Partition 2: {"c": [4]}    → reduce → {"c": 4}  (Thread 2)
Merge: {"a": 4, "b": 7, "c": 4}
```

### Architecture

**Modified Structs:**
- `ParallelMapReduce` (extended)
  - **Function** `parallel_reduce<K, V>(&self, partitions: Vec<HashMap<K, Vec<V>>>, reducer: F) -> HashMap<K, V>` - Parallel reduce
  - **Function** `map_reduce<K, V>(&self, entries: Vec<LogEntry>, mapper: M, reducer: R) -> HashMap<K, V>` - Full pipeline

**Key Functions:**
- `parallel_reduce`: Reduce each partition in parallel using Rayon
- `merge_results`: Combine partition results into final HashMap
- `map_reduce`: Complete map-shuffle-reduce pipeline

**Role Each Plays:**
- Parallel reduce: Process partitions concurrently
- Rayon: Thread pool management
- Merge: Combine independent partition results

### Checkpoint Tests

```rust
#[test]
fn test_parallel_reduce() {
    let mr = ParallelMapReduce::new(100);

    let mut partitions = Vec::new();
    let mut p1 = HashMap::new();
    p1.insert("key1".to_string(), vec![1, 2, 3]);
    partitions.push(p1);

    let mut p2 = HashMap::new();
    p2.insert("key2".to_string(), vec![4, 5]);
    partitions.push(p2);

    let result = mr.parallel_reduce(partitions, |values| {
        values.into_iter().sum::<u64>()
    });

    assert_eq!(result.get("key1"), Some(&6));
    assert_eq!(result.get("key2"), Some(&9));
}

#[test]
fn test_full_map_reduce() {
    let mr = ParallelMapReduce::new(100);
    let entries = vec![
        LogEntry { endpoint: "/api/users".into(), ..Default::default() },
        LogEntry { endpoint: "/api/login".into(), ..Default::default() },
        LogEntry { endpoint: "/api/users".into(), ..Default::default() },
    ];

    let result = mr.map_reduce(
        entries,
        |e| (e.endpoint.clone(), 1u64),
        |values| values.into_iter().sum(),
    );

    assert_eq!(result.get("/api/users"), Some(&2));
    assert_eq!(result.get("/api/login"), Some(&1));
}

#[test]
fn test_average_with_map_reduce() {
    let mr = ParallelMapReduce::new(100);
    let entries = vec![
        LogEntry { endpoint: "/api/users".into(), response_time_ms: 50, ..Default::default() },
        LogEntry { endpoint: "/api/users".into(), response_time_ms: 100, ..Default::default() },
        LogEntry { endpoint: "/api/login".into(), response_time_ms: 200, ..Default::default() },
    ];

    let result = mr.map_reduce(
        entries,
        |e| (e.endpoint.clone(), (e.response_time_ms as f64, 1u64)),
        |values| {
            let (sum, count) = values.into_iter()
                .fold((0.0, 0u64), |(s, c), (val_s, val_c)| (s + val_s, c + val_c));
            sum / count as f64
        },
    );

    assert_eq!(result.get("/api/users"), Some(&75.0));
}

#[test]
fn benchmark_parallel_reduce() {
    use std::time::Instant;

    let mr = ParallelMapReduce::new(1000);
    let entries: Vec<LogEntry> = (0..1000000).map(|i| LogEntry {
        endpoint: format!("/api/{}", i % 1000),
        ..Default::default()
    }).collect();

    let start = Instant::now();
    let result = mr.map_reduce(
        entries,
        |e| (e.endpoint.clone(), 1u64),
        |values| values.into_iter().sum(),
    );
    let elapsed = start.elapsed();

    println!("Parallel map-reduce: {:?}", elapsed);
    println!("Unique keys: {}", result.len());
    assert_eq!(result.len(), 1000);
}
```

### Starter Code

```rust
impl ParallelMapReduce {
    pub fn parallel_reduce<K, V, F>(&self, partitions: Vec<HashMap<K, Vec<V>>>, reducer: F) -> HashMap<K, V>
    where
        K: Hash + Eq + Send,
        V: Send,
        F: Fn(Vec<V>) -> V + Sync + Send,
    {
        // TODO: Reduce partitions in parallel
        //
        // 1. Process each partition in parallel
        // 2. Apply reducer to values for each key
        // 3. Merge all partition results
        //
        // partitions.into_par_iter()
        //     .flat_map(|partition| {
        //         partition.into_iter()
        //             .map(|(key, values)| (key, reducer(values)))
        //             .collect::<Vec<_>>()
        //     })
        //     .collect()
        todo!()
    }

    pub fn map_reduce<K, V, M, R>(
        &self,
        entries: Vec<LogEntry>,
        mapper: M,
        reducer: R,
    ) -> HashMap<K, V>
    where
        K: Hash + Eq + Clone + Send,
        V: Send,
        M: Fn(&LogEntry) -> (K, V) + Sync + Send,
        R: Fn(Vec<V>) -> V + Sync + Send,
    {
        // TODO: Complete map-reduce pipeline
        //
        // 1. Parallel map
        // 2. Shuffle
        // 3. Parallel reduce
        //
        // let pairs = self.parallel_map(entries, mapper);
        // let partitions = self.shuffle(pairs);
        // self.parallel_reduce(partitions, reducer)
        todo!()
    }
}
```

---

## Milestone 5: Combiner Optimization

### Introduction

**Why Milestone 4 Is Not Enough:**
The shuffle phase moves large amounts of data between map and reduce. For operations like sum/count, we can aggregate locally before shuffling.

**What We're Improving:**
Implement combiners that pre-aggregate data within each map task before shuffling. This dramatically reduces data movement.

**Combiner Benefit:**
```
Without combiner:
Map output: 1M pairs → Shuffle 1M pairs → Reduce

With combiner:
Map output: 1M pairs → Local reduce to 10K pairs → Shuffle 10K pairs → Reduce
Shuffle reduced by 99%!
```

Example:
```
Map chunk: [("a", 1), ("a", 1), ("b", 1), ("a", 1)]
Without combiner: Send 4 pairs
With combiner: Send [("a", 3), ("b", 1)] - 2 pairs instead of 4
```

### Architecture

**Modified Structs:**
- `ParallelMapReduce` (extended)
  - **Field** `use_combiner: bool` - Enable/disable combiner
  - **Function** `map_with_combiner<K, V>(&self, entries: Vec<LogEntry>, mapper: M, combiner: C) -> Vec<(K, V)>` - Map with local aggregation
  - **Function** `local_reduce<K, V>(&self, pairs: Vec<(K, V)>, combiner: F) -> Vec<(K, V)>` - Combine within chunk

**Key Functions:**
- `local_reduce`: Aggregate pairs within each map task
- Combiner function: Same signature as reducer (can reuse)
- Optimization: Reduces shuffle data by 50-99% for aggregations

**Role Each Plays:**
- Combiner: Pre-aggregation before shuffle
- Local reduce: Group and aggregate within chunk
- Network optimization: Less data transferred

### Checkpoint Tests

```rust
#[test]
fn test_local_reduce() {
    let mr = ParallelMapReduce::new(100);
    let pairs = vec![
        ("key1".to_string(), 1u64),
        ("key2".to_string(), 2u64),
        ("key1".to_string(), 3u64),
        ("key1".to_string(), 4u64),
    ];

    let combined = mr.local_reduce(pairs, |values| values.into_iter().sum());

    // Should have 2 pairs instead of 4
    assert_eq!(combined.len(), 2);

    let mut map: HashMap<String, u64> = combined.into_iter().collect();
    assert_eq!(map.get("key1"), Some(&8)); // 1 + 3 + 4
    assert_eq!(map.get("key2"), Some(&2));
}

#[test]
fn test_combiner_reduces_shuffle() {
    let mr_no_combiner = ParallelMapReduce::new(100).with_combiner(false);
    let mr_combiner = ParallelMapReduce::new(100).with_combiner(true);

    let entries: Vec<LogEntry> = (0..10000).map(|i| LogEntry {
        endpoint: format!("/api/{}", i % 10), // Only 10 unique keys
        ..Default::default()
    }).collect();

    // Without combiner: 10000 pairs
    let pairs_no_combiner = mr_no_combiner.parallel_map(entries.clone(), |e| {
        (e.endpoint.clone(), 1u64)
    });

    // With combiner: ~10 pairs per chunk
    let pairs_combiner = mr_combiner.map_with_combiner(
        entries,
        |e| (e.endpoint.clone(), 1u64),
        |values| values.into_iter().sum(),
    );

    println!("Without combiner: {} pairs", pairs_no_combiner.len());
    println!("With combiner: {} pairs", pairs_combiner.len());

    assert!(pairs_combiner.len() < pairs_no_combiner.len() / 10);
}

#[test]
fn test_combiner_correctness() {
    let mr = ParallelMapReduce::new(100).with_combiner(true);
    let entries: Vec<LogEntry> = (0..1000).map(|i| LogEntry {
        endpoint: format!("/api/{}", i % 10),
        ..Default::default()
    }).collect();

    let result = mr.map_reduce(
        entries,
        |e| (e.endpoint.clone(), 1u64),
        |values| values.into_iter().sum(),
    );

    // Should count correctly despite combiner
    assert_eq!(result.len(), 10);
    for (_, count) in result {
        assert_eq!(count, 100); // Each endpoint appears 100 times
    }
}

#[test]
fn benchmark_combiner_speedup() {
    use std::time::Instant;

    let entries: Vec<LogEntry> = (0..1000000).map(|i| LogEntry {
        endpoint: format!("/api/{}", i % 100),
        ..Default::default()
    }).collect();

    // Without combiner
    let mr_no_combiner = ParallelMapReduce::new(1000).with_combiner(false);
    let start = Instant::now();
    let result1 = mr_no_combiner.map_reduce(
        entries.clone(),
        |e| (e.endpoint.clone(), 1u64),
        |values| values.into_iter().sum(),
    );
    let time_no_combiner = start.elapsed();

    // With combiner
    let mr_combiner = ParallelMapReduce::new(1000).with_combiner(true);
    let start = Instant::now();
    let result2 = mr_combiner.map_reduce(
        entries,
        |e| (e.endpoint.clone(), 1u64),
        |values| values.into_iter().sum(),
    );
    let time_combiner = start.elapsed();

    println!("Without combiner: {:?}", time_no_combiner);
    println!("With combiner: {:?}", time_combiner);
    println!("Speedup: {:.2}x", time_no_combiner.as_secs_f64() / time_combiner.as_secs_f64());

    assert_eq!(result1, result2); // Results should be identical
}
```

### Starter Code

```rust
impl ParallelMapReduce {
    pub fn with_combiner(mut self, use_combiner: bool) -> Self {
        self.use_combiner = use_combiner;
        self
    }

    pub fn local_reduce<K, V, F>(&self, pairs: Vec<(K, V)>, combiner: F) -> Vec<(K, V)>
    where
        K: Hash + Eq,
        F: Fn(Vec<V>) -> V,
    {
        // TODO: Aggregate pairs locally within chunk
        //
        // 1. Group by key
        // 2. Apply combiner function
        // 3. Return aggregated pairs
        //
        // let mut grouped: HashMap<K, Vec<V>> = HashMap::new();
        // for (key, value) in pairs {
        //     grouped.entry(key).or_insert_with(Vec::new).push(value);
        // }
        //
        // grouped.into_iter()
        //     .map(|(key, values)| (key, combiner(values)))
        //     .collect()
        todo!()
    }

    pub fn map_with_combiner<K, V, M, C>(
        &self,
        entries: Vec<LogEntry>,
        mapper: M,
        combiner: C,
    ) -> Vec<(K, V)>
    where
        K: Hash + Eq + Send + Clone,
        V: Send,
        M: Fn(&LogEntry) -> (K, V) + Sync + Send,
        C: Fn(Vec<V>) -> V + Sync + Send,
    {
        // TODO: Map with combiner
        //
        // 1. Chunk data
        // 2. For each chunk in parallel:
        //    a. Map entries to pairs
        //    b. Apply local combiner to reduce pairs
        // 3. Flatten results
        //
        // let chunks = self.chunk_data(entries);
        //
        // chunks.into_par_iter()
        //     .flat_map(|chunk| {
        //         let pairs: Vec<(K, V)> = chunk.iter().map(&mapper).collect();
        //         self.local_reduce(pairs, &combiner)
        //     })
        //     .collect()
        todo!()
    }
}
```

---

## Milestone 6: Multi-Stage Pipelines

### Introduction

**Why Milestone 5 Is Not Enough:**
Real-world analytics often require multiple transformations. For example: filter errors → count by endpoint → find top 10. This requires chaining map-reduce stages.

**What We're Improving:**
Support multi-stage pipelines where output of one map-reduce feeds into another. Enable complex analytics workflows.

**Pipeline Example:**
```
Stage 1: Filter ERROR logs → Count by endpoint
Stage 2: Take counts → Find top 10 → Format output
Stage 3: Group by hour → Count per hour
```

### Architecture

**Structs:**
- `PipelineBuilder` - Simplified builder for common operations
  - **Function** `count_by_endpoint(&self, entries: Vec<LogEntry>) -> HashMap<String, u64>`
  - **Function** `average_response_time(&self, entries: Vec<LogEntry>) -> HashMap<String, f64>`
  - **Function** `error_rate_by_endpoint(&self, entries: Vec<LogEntry>) -> HashMap<String, f64>`
  - **Function** `top_k_endpoints(&self, entries: Vec<LogEntry>, k: usize) -> Vec<(String, u64)>`

**Key Functions:**
- Builder pattern: Fluent API for common operations
- Filter + map-reduce: Combine filtering with aggregation
- Top-K: Find most frequent items

**Role Each Plays:**
- Pipeline: Orchestrate multi-stage processing
- Builder: Simplify common analytics patterns
- Composition: Build complex analytics from simple operations

### Checkpoint Tests

```rust
#[test]
fn test_simple_pipeline() {
    let mr = ParallelMapReduce::new(100);
    let builder = PipelineBuilder::new(&mr);

    let entries = vec![
        LogEntry { endpoint: "/api/users".into(), ..Default::default() },
        LogEntry { endpoint: "/api/users".into(), ..Default::default() },
        LogEntry { endpoint: "/api/login".into(), ..Default::default() },
    ];

    let counts = builder.count_by_endpoint(entries);

    assert_eq!(counts.get("/api/users"), Some(&2));
    assert_eq!(counts.get("/api/login"), Some(&1));
}

#[test]
fn test_filter_then_count() {
    let mr = ParallelMapReduce::new(100);
    let builder = PipelineBuilder::new(&mr);

    let entries = vec![
        LogEntry { level: LogLevel::ERROR, endpoint: "/api/users".into(), ..Default::default() },
        LogEntry { level: LogLevel::INFO, endpoint: "/api/users".into(), ..Default::default() },
        LogEntry { level: LogLevel::ERROR, endpoint: "/api/users".into(), ..Default::default() },
    ];

    // Filter errors, then count
    let errors: Vec<LogEntry> = entries.into_par_iter()
        .filter(|e| e.level == LogLevel::ERROR)
        .collect();

    let counts = builder.count_by_endpoint(errors);

    // Only 2 errors for /api/users
    assert_eq!(counts.get("/api/users"), Some(&2));
}

#[test]
fn test_error_rate_pipeline() {
    let mr = ParallelMapReduce::new(100);
    let builder = PipelineBuilder::new(&mr);

    let entries = vec![
        LogEntry { level: LogLevel::ERROR, endpoint: "/api/users".into(), ..Default::default() },
        LogEntry { level: LogLevel::INFO, endpoint: "/api/users".into(), ..Default::default() },
        LogEntry { level: LogLevel::ERROR, endpoint: "/api/users".into(), ..Default::default() },
        LogEntry { level: LogLevel::INFO, endpoint: "/api/users".into(), ..Default::default() },
    ];

    let error_rates = builder.error_rate_by_endpoint(entries);

    let rate = error_rates.get("/api/users").unwrap();
    assert_eq!(*rate, 50.0); // 2 errors out of 4 = 50%
}

#[test]
fn test_top_k_endpoints() {
    let mr = ParallelMapReduce::new(100);
    let builder = PipelineBuilder::new(&mr);

    let entries: Vec<LogEntry> = (0..100).map(|i| LogEntry {
        endpoint: format!("/api/{}", i % 5),
        ..Default::default()
    }).collect();

    let top3 = builder.top_k_endpoints(entries, 3);

    assert_eq!(top3.len(), 3);
    // Each should have 20 requests
    for (_, count) in top3 {
        assert_eq!(count, 20);
    }
}
```

### Starter Code

```rust
// ============================================================================
// PIPELINE BUILDER - Simplified API for common operations
// ============================================================================

pub struct PipelineBuilder<'a> {
    mr: &'a ParallelMapReduce,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(mr: &'a ParallelMapReduce) -> Self {
        Self { mr }
    }

    pub fn count_by_endpoint(&self, entries: Vec<LogEntry>) -> HashMap<String, u64> {
        self.mr.map_reduce(
            entries,
            |e| (e.endpoint.clone(), 1u64),
            |v| v.into_iter().sum(),
        )
    }

    pub fn average_response_time(&self, entries: Vec<LogEntry>) -> HashMap<String, f64> {
        let result = self.mr.map_reduce(
            entries,
            |e| (e.endpoint.clone(), (e.response_time_ms as f64, 1u64)),
            |v| {
                let (sum, count) = v.into_iter()
                    .fold((0.0, 0u64), |(s, c), (val_s, val_c)| (s + val_s, c + val_c));
                (sum, count)
            },
        );

        result.into_iter()
            .map(|(k, (sum, count))| (k, sum / count as f64))
            .collect()
    }

    pub fn error_rate_by_endpoint(&self, entries: Vec<LogEntry>) -> HashMap<String, f64> {
        // TODO: Calculate error rate (errors / total requests) per endpoint
        //
        // Use map-reduce to count total and errors per endpoint
        // Then calculate percentage
        //
        // let result = self.mr.map_reduce(
        //     entries,
        //     |e| {
        //         let is_error = if e.level == LogLevel::ERROR { 1u64 } else { 0u64 };
        //         (e.endpoint.clone(), (1u64, is_error))
        //     },
        //     |v| {
        //         v.into_iter().fold((0, 0), |(total, errors), (t, e)| {
        //             (total + t, errors + e)
        //         })
        //     },
        // );
        //
        // result.into_iter()
        //     .map(|(k, (total, errors))| (k, errors as f64 / total as f64 * 100.0))
        //     .collect()
        todo!()
    }

    pub fn top_k_endpoints(&self, entries: Vec<LogEntry>, k: usize) -> Vec<(String, u64)> {
        // TODO: Find top K endpoints by request count
        //
        // 1. Count by endpoint
        // 2. Sort by count descending
        // 3. Take top K
        //
        // let counts = self.count_by_endpoint(entries);
        //
        // let mut sorted: Vec<_> = counts.into_iter().collect();
        // sorted.sort_by(|a, b| b.1.cmp(&a.1));
        // sorted.truncate(k);
        // sorted
        todo!()
    }
}
```

---

## Complete Working Example

```rust
use rayon::prelude::*;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ============================================================================
// LOG STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub endpoint: String,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub user_id: Option<String>,
}

impl Default for LogEntry {
    fn default() -> Self {
        Self {
            timestamp: String::new(),
            level: LogLevel::INFO,
            endpoint: String::new(),
            status_code: 200,
            response_time_ms: 0,
            user_id: None,
        }
    }
}

impl LogEntry {
    pub fn parse(line: &str) -> Result<Self, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            return Err("Invalid log format".into());
        }

        let level = match parts[1] {
            "DEBUG" => LogLevel::DEBUG,
            "INFO" => LogLevel::INFO,
            "WARN" => LogLevel::WARN,
            "ERROR" => LogLevel::ERROR,
            _ => return Err("Invalid log level".into()),
        };

        let response_time = parts[5]
            .trim_end_matches("ms")
            .parse()
            .map_err(|_| "Invalid response time")?;

        let user_id = parts.get(6).and_then(|s| {
            s.strip_prefix("user=").map(|u| u.to_string())
        });

        Ok(LogEntry {
            timestamp: parts[0].to_string(),
            level,
            endpoint: parts[3].to_string(),
            status_code: parts[4].parse().map_err(|_| "Invalid status code")?,
            response_time_ms: response_time,
            user_id,
        })
    }
}

// ============================================================================
// PARALLEL MAP-REDUCE FRAMEWORK
// ============================================================================

pub struct ParallelMapReduce {
    chunk_size: usize,
    use_combiner: bool,
}

impl ParallelMapReduce {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunk_size,
            use_combiner: true,
        }
    }

    pub fn with_combiner(mut self, use_combiner: bool) -> Self {
        self.use_combiner = use_combiner;
        self
    }

    fn chunk_data(&self, entries: Vec<LogEntry>) -> Vec<Vec<LogEntry>> {
        entries.chunks(self.chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    pub fn parallel_map<K, V, F>(&self, entries: Vec<LogEntry>, mapper: F) -> Vec<(K, V)>
    where
        K: Send,
        V: Send,
        F: Fn(&LogEntry) -> (K, V) + Sync + Send,
    {
        let chunks = self.chunk_data(entries);

        chunks.par_iter()
            .flat_map(|chunk| {
                chunk.iter().map(&mapper).collect::<Vec<_>>()
            })
            .collect()
    }

    fn local_reduce<K, V, F>(&self, pairs: Vec<(K, V)>, combiner: F) -> Vec<(K, V)>
    where
        K: Hash + Eq,
        F: Fn(Vec<V>) -> V,
    {
        let mut grouped: HashMap<K, Vec<V>> = HashMap::new();
        for (key, value) in pairs {
            grouped.entry(key).or_insert_with(Vec::new).push(value);
        }

        grouped.into_iter()
            .map(|(key, values)| (key, combiner(values)))
            .collect()
    }

    pub fn map_with_combiner<K, V, M, C>(
        &self,
        entries: Vec<LogEntry>,
        mapper: M,
        combiner: C,
    ) -> Vec<(K, V)>
    where
        K: Hash + Eq + Send + Clone,
        V: Send,
        M: Fn(&LogEntry) -> (K, V) + Sync + Send,
        C: Fn(Vec<V>) -> V + Sync + Send,
    {
        let chunks = self.chunk_data(entries);

        chunks.into_par_iter()
            .flat_map(|chunk| {
                let pairs: Vec<(K, V)> = chunk.iter().map(&mapper).collect();
                self.local_reduce(pairs, &combiner)
            })
            .collect()
    }

    pub fn shuffle<K, V>(&self, pairs: Vec<(K, V)>) -> Vec<HashMap<K, Vec<V>>>
    where
        K: Hash + Eq + Clone,
    {
        let num_partitions = num_cpus::get();
        let mut partitions: Vec<HashMap<K, Vec<V>>> = (0..num_partitions)
            .map(|_| HashMap::new())
            .collect();

        for (key, value) in pairs {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            let idx = (hasher.finish() as usize) % num_partitions;

            partitions[idx]
                .entry(key)
                .or_insert_with(Vec::new)
                .push(value);
        }

        partitions
    }

    pub fn parallel_reduce<K, V, F>(
        &self,
        partitions: Vec<HashMap<K, Vec<V>>>,
        reducer: F,
    ) -> HashMap<K, V>
    where
        K: Hash + Eq + Send,
        V: Send,
        F: Fn(Vec<V>) -> V + Sync + Send,
    {
        partitions.into_par_iter()
            .flat_map(|partition| {
                partition.into_iter()
                    .map(|(key, values)| (key, reducer(values)))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn map_reduce<K, V, M, R>(
        &self,
        entries: Vec<LogEntry>,
        mapper: M,
        reducer: R,
    ) -> HashMap<K, V>
    where
        K: Hash + Eq + Clone + Send,
        V: Send,
        M: Fn(&LogEntry) -> (K, V) + Sync + Send,
        R: Fn(Vec<V>) -> V + Sync + Send,
    {
        let pairs = if self.use_combiner {
            self.map_with_combiner(entries, mapper, &reducer)
        } else {
            self.parallel_map(entries, mapper)
        };

        let partitions = self.shuffle(pairs);
        self.parallel_reduce(partitions, reducer)
    }
}

// ============================================================================
// HIGH-LEVEL ANALYTICS API
// ============================================================================

pub struct LogAnalytics {
    mr: ParallelMapReduce,
}

impl LogAnalytics {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            mr: ParallelMapReduce::new(chunk_size),
        }
    }

    pub fn count_by_endpoint(&self, entries: Vec<LogEntry>) -> HashMap<String, u64> {
        self.mr.map_reduce(
            entries,
            |e| (e.endpoint.clone(), 1u64),
            |v| v.into_iter().sum(),
        )
    }

    pub fn average_response_time(&self, entries: Vec<LogEntry>) -> HashMap<String, f64> {
        let result = self.mr.map_reduce(
            entries,
            |e| (e.endpoint.clone(), (e.response_time_ms as f64, 1u64)),
            |v| {
                v.into_iter().fold((0.0, 0u64), |(s, c), (val_s, val_c)| {
                    (s + val_s, c + val_c)
                })
            },
        );

        result.into_iter()
            .map(|(k, (sum, count))| (k, sum / count as f64))
            .collect()
    }

    pub fn error_rate_by_endpoint(&self, entries: Vec<LogEntry>) -> HashMap<String, f64> {
        let result = self.mr.map_reduce(
            entries,
            |e| {
                let is_error = if e.level == LogLevel::ERROR { 1u64 } else { 0u64 };
                (e.endpoint.clone(), (1u64, is_error))
            },
            |v| {
                v.into_iter().fold((0, 0), |(total, errors), (t, e)| {
                    (total + t, errors + e)
                })
            },
        );

        result.into_iter()
            .map(|(k, (total, errors))| (k, errors as f64 / total as f64 * 100.0))
            .collect()
    }

    pub fn top_k_endpoints(&self, entries: Vec<LogEntry>, k: usize) -> Vec<(String, u64)> {
        let counts = self.count_by_endpoint(entries);

        let mut sorted: Vec<_> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(k);
        sorted
    }

    pub fn filter_errors(&self, entries: Vec<LogEntry>) -> Vec<LogEntry> {
        entries.into_par_iter()
            .filter(|e| e.level == LogLevel::ERROR)
            .collect()
    }
}

// ============================================================================
// EXAMPLE USAGE
// ============================================================================

fn main() {
    println!("=== Map-Reduce Log Analysis Demo ===\n");

    // Generate sample logs
    let mut logs = Vec::new();
    for i in 0..10000 {
        let level = match i % 10 {
            0 => LogLevel::ERROR,
            1..=2 => LogLevel::WARN,
            _ => LogLevel::INFO,
        };

        let endpoint = format!("/api/{}", ["users", "login", "checkout", "cart"][i % 4]);

        logs.push(LogEntry {
            timestamp: format!("2024-01-15T10:{:02}:00Z", i % 60),
            level,
            endpoint,
            status_code: if level == LogLevel::ERROR { 500 } else { 200 },
            response_time_ms: (i % 300) as u64,
            user_id: Some(format!("user{}", i % 100)),
        });
    }

    let analytics = LogAnalytics::new(1000);

    // Count by endpoint
    println!("--- Request Counts ---");
    let counts = analytics.count_by_endpoint(logs.clone());
    for (endpoint, count) in &counts {
        println!("{}: {}", endpoint, count);
    }

    // Average response time
    println!("\n--- Average Response Time ---");
    let avg_times = analytics.average_response_time(logs.clone());
    for (endpoint, avg) in &avg_times {
        println!("{}: {:.2}ms", endpoint, avg);
    }

    // Error rates
    println!("\n--- Error Rates ---");
    let error_rates = analytics.error_rate_by_endpoint(logs.clone());
    for (endpoint, rate) in &error_rates {
        println!("{}: {:.2}%", endpoint, rate);
    }

    // Top endpoints
    println!("\n--- Top 3 Endpoints ---");
    let top = analytics.top_k_endpoints(logs.clone(), 3);
    for (i, (endpoint, count)) in top.iter().enumerate() {
        println!("{}. {}: {} requests", i + 1, endpoint, count);
    }

    // Benchmark
    use std::time::Instant;
    println!("\n--- Performance Benchmark ---");

    let start = Instant::now();
    let _ = analytics.count_by_endpoint(logs.clone());
    let elapsed = start.elapsed();

    println!("Processed {} logs in {:?}", logs.len(), elapsed);
    println!("Throughput: {:.2}M logs/sec", logs.len() as f64 / elapsed.as_secs_f64() / 1_000_000.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_reduce_count() {
        let mr = ParallelMapReduce::new(100);
        let entries = vec![
            LogEntry {
                endpoint: "/api/users".into(),
                ..Default::default()
            },
            LogEntry {
                endpoint: "/api/users".into(),
                ..Default::default()
            },
        ];

        let result = mr.map_reduce(
            entries,
            |e| (e.endpoint.clone(), 1u64),
            |v| v.into_iter().sum(),
        );

        assert_eq!(result.get("/api/users"), Some(&2));
    }

    #[test]
    fn test_analytics() {
        let analytics = LogAnalytics::new(100);
        let entries = vec![
            LogEntry {
                level: LogLevel::INFO,
                endpoint: "/api/users".into(),
                response_time_ms: 50,
                ..Default::default()
            },
            LogEntry {
                level: LogLevel::ERROR,
                endpoint: "/api/users".into(),
                response_time_ms: 100,
                ..Default::default()
            },
        ];

        let counts = analytics.count_by_endpoint(entries.clone());
        assert_eq!(counts.get("/api/users"), Some(&2));

        let error_rate = analytics.error_rate_by_endpoint(entries);
        assert_eq!(error_rate.get("/api/users"), Some(&50.0));
    }
}
```

---

## Summary

This comprehensive map-reduce framework project teaches production-grade parallel data processing!

### What You Built:

1. **Milestone 1**: Sequential log processor (baseline)
2. **Milestone 2**: Parallel map phase (8-16x speedup)
3. **Milestone 3**: Shuffle/partition phase (hash-based grouping)
4. **Milestone 4**: Parallel reduce phase (independent partition reduction)
5. **Milestone 5**: Combiner optimization (50-90% less shuffle data)
6. **Milestone 6**: Multi-stage pipelines (complex analytics workflows)

### Key Concepts Learned:

- **Map-Reduce Pattern**: Industry-standard data parallelism model
- **Data Partitioning**: Hash-based distribution for parallel processing
- **Shuffle Phase**: Grouping intermediate results by key
- **Combiner Optimization**: Local aggregation to reduce network traffic
- **Pipeline Composition**: Chaining map-reduce operations
- **Parallel Aggregation**: Concurrent reduction across partitions

### Performance Optimization:

- Sequential: O(n) single-threaded
- Parallel map: O(n/p) where p = cores
- Combiner: Reduces shuffle by 50-99%
- Real-world: Process 100 GB logs in 2-4 minutes vs 30 minutes

### Real-World Applications:

- **Log Analysis**: Count requests, analyze errors, track performance
- **ETL Pipelines**: Transform and aggregate data at scale
- **Business Analytics**: User behavior, conversion tracking
- **Security**: Anomaly detection, threat analysis
- **Monitoring**: Aggregate metrics from distributed systems

This framework mirrors production systems like Hadoop MapReduce and Apache Spark![project-3-arena-allocator-with-lifetime-variance.md](../workbook-projects/project-3-arena-allocator-with-lifetime-variance.md)