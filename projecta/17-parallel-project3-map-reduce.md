# Map-Reduce Framework for Distributed Log Analysis

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
- No combiner: Shuffle 100 GB → 10 min network transfer
- With combiner: Shuffle 10 GB → 1 min network transfer
- 10x faster pipeline!

---

## Key Concepts Explained

### 1. Map-Reduce Programming Model

**What Is It?**
Map-Reduce is a programming model for processing large datasets in parallel by dividing work into two phases: mapping (transformation) and reducing (aggregation).

**Three Core Phases:**
```
1. MAP:     Transform input → key-value pairs
2. SHUFFLE: Group pairs by key → partitions
3. REDUCE:  Aggregate values per key → final result
```

**Visual Example:**
```
Input data: ["GET /api/users", "GET /api/login", "GET /api/users"]

MAP phase (transform to key-value pairs):
    ↓
[("/api/users", 1), ("/api/login", 1), ("/api/users", 1)]

SHUFFLE phase (group by key):
    ↓
{
  "/api/users": [1, 1],
  "/api/login": [1]
}

REDUCE phase (aggregate values):
    ↓
{
  "/api/users": 2,
  "/api/login": 1
}
```

**Map Function:**
```rust
// Transform: LogEntry → (Key, Value)
fn map(entry: &LogEntry) -> (String, u64) {
    (entry.endpoint.clone(), 1)
    // Emits one key-value pair per log entry
}

// Can emit multiple pairs per input:
fn map_words(line: &str) -> Vec<(String, u64)> {
    line.split_whitespace()
        .map(|word| (word.to_string(), 1))
        .collect()
}
```

**Reduce Function:**
```rust
// Aggregate: Vec<Value> → Value
fn reduce(key: String, values: Vec<u64>) -> u64 {
    values.into_iter().sum()
    // Combines all values for a key
}

// Can perform any aggregation:
fn reduce_average(key: String, values: Vec<(f64, u64)>) -> f64 {
    let (sum, count) = values.into_iter()
        .fold((0.0, 0u64), |(s, c), (val_s, val_c)| (s + val_s, c + val_c));
    sum / count as f64
}
```

**Why Map-Reduce?**

1. **Simplicity**: Programmer only writes map and reduce functions
2. **Scalability**: Framework handles parallelism and distribution
3. **Fault Tolerance**: Failed tasks can be restarted
4. **Flexibility**: Works for many problems (counting, averaging, joining, filtering)

**Functional Programming Roots:**
```rust
// Map-reduce is built on functional primitives:
let result = data
    .map(|x| transform(x))           // MAP
    .group_by(|pair| pair.0)         // SHUFFLE (implicit)
    .map(|(key, values)| reduce(key, values))  // REDUCE
    .collect();
```

**Real-World Systems:**
- **Hadoop MapReduce**: Distributed batch processing (disk-based)
- **Apache Spark**: In-memory distributed computing (100x faster than Hadoop)
- **Google MapReduce**: Original paper (2004), processed 20+ PB/day
- **Our Framework**: Single-machine multi-core version

**Comparison:**
```
Sequential processing:
for entry in logs {
    let key = entry.endpoint;
    counts[key] += 1;
}
Time: O(n) single-threaded

Map-Reduce:
Map:    Process chunks in parallel   → O(n/p)
Shuffle: Hash partition (parallel)    → O(n/p)
Reduce: Aggregate partitions in parallel → O(k/p) where k = unique keys

Total: O(n/p + k/p) ≈ O(n/p) when k << n
Speedup: ~p (number of cores)
```

---

### 2. Data Parallelism vs Task Parallelism

**What Is It?**
Data parallelism divides data into chunks and applies the same operation to each chunk concurrently. Task parallelism executes different operations concurrently.

**Data Parallelism (Map-Reduce):**
```rust
// Same operation (count) on different data chunks
let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
let chunks = [
    [1, 2],  // Chunk 0
    [3, 4],  // Chunk 1
    [5, 6],  // Chunk 2
    [7, 8],  // Chunk 3
];

// Process in parallel:
Thread 0: sum([1, 2]) = 3
Thread 1: sum([3, 4]) = 7
Thread 2: sum([5, 6]) = 11
Thread 3: sum([7, 8]) = 15

// Combine results: 3 + 7 + 11 + 15 = 36
```

**Task Parallelism (Different Operations):**
```rust
// Different operations on same or different data
Thread 0: parse_logs(file1)        // Task: parsing
Thread 1: compress_images(dir)     // Task: compression
Thread 2: send_emails(users)       // Task: I/O
Thread 3: calculate_stats(data)    // Task: computation

// Each thread does completely different work
```

**Visual Comparison:**
```
DATA PARALLELISM:
Data: [█][█][█][█][█][█][█][█]
       ↓  ↓  ↓  ↓  ↓  ↓  ↓  ↓
Threads: T0 T1 T2 T3 T0 T1 T2 T3
Operation: [MAP] [MAP] [MAP] [MAP]  (same operation)

TASK PARALLELISM:
Tasks: [Parse] [Compress] [Send] [Calculate]
        ↓       ↓          ↓      ↓
Threads: T0     T1         T2     T3
         (different operations)
```

**Map-Reduce is Data Parallel:**
```rust
// Process log chunks in parallel
let chunks = chunk_data(logs, chunk_size);

// SAME OPERATION on each chunk:
let results: Vec<_> = chunks.par_iter()
    .map(|chunk| {
        // Every thread executes this same code
        chunk.iter()
            .map(|entry| (entry.endpoint.clone(), 1))
            .collect::<Vec<_>>()
    })
    .collect();

// Key: Same transformation, different data
```

**Advantages of Data Parallelism:**

1. **Load Balancing**: Easy to distribute work evenly
   ```
   1M logs / 8 cores = 125K logs per core
   ```

2. **Scalability**: Works with any number of cores
   ```
   1 core:  1000ms
   8 cores: 125ms  (8x speedup)
   16 cores: 62ms  (16x speedup)
   ```

3. **Simplicity**: No complex synchronization
   ```rust
   // Each chunk is independent
   chunks.par_iter().map(process_chunk)
   // No locks, no coordination during map phase
   ```

4. **Predictable Performance**: Depends on data size, not task complexity
   ```
   2x data = 2x time (scales linearly)
   ```

**When Data Parallelism Works Best:**
- Homogeneous data (all items similar)
- Same processing time per item
- Independent operations (no dependencies between items)
- Large datasets (>10K items to amortize overhead)

**When Data Parallelism Struggles:**
- Skewed data (some items take 100x longer)
- Small datasets (overhead dominates)
- Dependencies between items (need sequential processing)

**Example: Log Processing:**
```rust
// Data parallel: Perfect fit!
let logs = vec![entry1, entry2, entry3, ..., entry_1M];

// Why it works:
// 1. Each log entry is independent
// 2. Processing time similar (~1μs per entry)
// 3. Same operation (parse, map to key-value)
// 4. Large dataset (1M entries)

// Result: Near-linear speedup
Sequential: 1000ms
8 cores:    125ms (8x speedup)
```

**Rayon's Data Parallelism:**
```rust
use rayon::prelude::*;

// Parallel map
let results: Vec<_> = data.par_iter()
    .map(|x| expensive_computation(x))
    .collect();

// Parallel filter
let filtered: Vec<_> = data.par_iter()
    .filter(|x| predicate(x))
    .collect();

// Parallel fold (reduce)
let sum: u64 = data.par_iter()
    .map(|x| x.value)
    .sum();

// Rayon automatically:
// - Divides data into chunks
// - Distributes across thread pool
// - Work stealing for load balance
```

**Performance Model:**
```
Data parallelism speedup formula:

Speedup = Sequential_time / Parallel_time
        ≈ p (number of cores)

With overhead:
Speedup = 1 / (1/p + overhead_fraction)

Example: 8 cores, 10% overhead
Speedup = 1 / (1/8 + 0.1) = 1 / 0.225 ≈ 4.4x

Less than perfect 8x due to:
- Thread creation overhead
- Data copying/chunking
- Cache contention
- Synchronization (shuffle phase)
```

---

### 3. Hash-Based Partitioning for Data Distribution

**What Is It?**
Hash partitioning distributes data across partitions using a hash function, ensuring keys with the same hash always go to the same partition.

**Why Partitioning?**
After parallel map, we have scattered key-value pairs:
```
Thread 0 output: [("a", 1), ("b", 2), ("a", 3)]
Thread 1 output: [("c", 4), ("a", 5), ("b", 6)]
Thread 2 output: [("a", 7), ("b", 8), ("c", 9)]

Problem: Key "a" appears in all threads!
Need to group all "a" values together before reduce.
```

**Hash Partitioning Solution:**
```rust
fn partition_id(key: &str, num_partitions: usize) -> usize {
    let hash = compute_hash(key);
    hash % num_partitions
}

// Example with 3 partitions:
partition_id("a", 3) = hash("a") % 3 = 157 % 3 = 1  (always 1)
partition_id("b", 3) = hash("b") % 3 = 289 % 3 = 1  (always 1)
partition_id("c", 3) = hash("c") % 3 = 412 % 3 = 1  (always 1)

// Deterministic: Same key always goes to same partition!
```

**Visual Example:**
```
Input pairs (scattered):
[("apple", 1), ("banana", 2), ("apple", 3), ("cherry", 4), ("banana", 5)]

Hash partitioning (3 partitions):
hash("apple")  % 3 = 2 → Partition 2
hash("banana") % 3 = 0 → Partition 0
hash("cherry") % 3 = 1 → Partition 1

Result:
Partition 0: [("banana", 2), ("banana", 5)]
Partition 1: [("cherry", 4)]
Partition 2: [("apple", 1), ("apple", 3)]

Now each partition can be reduced independently!
```

**Implementation:**
```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn partition<K, V>(pairs: Vec<(K, V)>, num_partitions: usize) -> Vec<Vec<(K, V)>>
where
    K: Hash + Clone,
{
    // Create empty partitions
    let mut partitions: Vec<Vec<(K, V)>> = (0..num_partitions)
        .map(|_| Vec::new())
        .collect();

    // Distribute pairs
    for (key, value) in pairs {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let idx = (hasher.finish() as usize) % num_partitions;

        partitions[idx].push((key, value));
    }

    partitions
}
```

**Hash Function Properties:**

1. **Deterministic**: Same input always produces same hash
   ```rust
   hash("hello") == hash("hello")  // Always true
   ```

2. **Uniform Distribution**: Spreads keys evenly
   ```
   1M keys → 8 partitions
   Each partition gets ~125K keys (±10%)
   ```

3. **Fast**: O(1) computation
   ```
   hash() takes ~10-20ns (faster than memory access)
   ```

4. **Avalanche Effect**: Small input change → large hash change
   ```
   hash("hello") = 0x1A2B3C4D
   hash("hella") = 0x9F8E7D6C  (completely different!)
   ```

**Rust's DefaultHasher:**
```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

let mut hasher = DefaultHasher::new();
"key".hash(&mut hasher);
let hash_value = hasher.finish();  // u64

// DefaultHasher uses SipHash-1-3 (cryptographically secure)
// Fast: ~20ns per key
// Good distribution: low collision rate
```

**Load Balance Test:**
```rust
#[test]
fn test_even_distribution() {
    let num_partitions = 8;
    let pairs: Vec<(String, u64)> = (0..10000)
        .map(|i| (format!("key{}", i), i))
        .collect();

    let partitions = partition(pairs, num_partitions);

    // Check distribution
    let avg = 10000 / num_partitions;  // 1250
    for (i, partition) in partitions.iter().enumerate() {
        let count = partition.len();
        println!("Partition {}: {} items", i, count);

        // Should be within 20% of average
        assert!(count > avg * 8 / 10);  // > 1000
        assert!(count < avg * 12 / 10); // < 1500
    }
}

// Typical output:
// Partition 0: 1247 items
// Partition 1: 1253 items
// Partition 2: 1241 items
// ...
// Good balance!
```

**Why Not Other Partitioning Strategies?**

**Range Partitioning:**
```rust
// Divide by key range
partition_id = if key < "m" { 0 } else { 1 }

// Problems:
// - Skewed distribution (more keys start with 'a' than 'z')
// - Requires knowing key distribution in advance
```

**Round-Robin Partitioning:**
```rust
// Assign to partitions sequentially
partition_id = counter++ % num_partitions

// Problems:
// - Same key goes to different partitions!
// - Can't group by key for reduce phase
// - Breaks map-reduce correctness
```

**Random Partitioning:**
```rust
partition_id = random() % num_partitions

// Problems:
// - Same key goes to different partitions (non-deterministic)
// - Can't reproduce results
// - Breaks reduce correctness
```

**Hash Partitioning Wins:**
- Deterministic (same key → same partition)
- Even distribution (good load balance)
- Fast (O(1) computation)
- No prior knowledge needed
- Standard in all map-reduce systems

**Shuffle Phase Performance:**
```
1M key-value pairs, 8 partitions

Hash computation: 1M * 20ns = 20ms
Partition insertion: 1M * 10ns = 10ms
Memory allocation: ~8MB (8 vectors)

Total shuffle overhead: ~30ms

Compare to:
- Map phase: 200ms (compute-heavy)
- Reduce phase: 100ms (aggregation)

Shuffle is only 10% of total time!
```

**Parallelizing Shuffle:**
```rust
// Can partition in parallel by locking partitions
use std::sync::Mutex;

let partitions: Vec<Mutex<Vec<(K, V)>>> = (0..num_partitions)
    .map(|_| Mutex::new(Vec::new()))
    .collect();

pairs.par_iter().for_each(|(key, value)| {
    let idx = hash_key(key) % num_partitions;
    partitions[idx].lock().unwrap().push((key.clone(), value.clone()));
});

// But: Lock contention can reduce parallelism
// Better: Partition sequentially (fast enough), or use lock-free structures
```

**Optimal Number of Partitions:**
```
Too few:  Reduce phase not parallel enough
Too many: More overhead, worse cache locality

Rule of thumb: num_partitions = num_cores * 2

8 cores → 16 partitions
- Each reducer gets 2 partitions
- Good load balance with work stealing
```

---

### 4. Chunking and Data Splitting Strategies

**What Is It?**
Chunking divides large datasets into smaller pieces that can be processed independently in parallel.

**Why Chunk?**
```rust
// Process 1M log entries
let logs: Vec<LogEntry> = ...; // 1M items

// Without chunking: All threads fight for same data structure
logs.par_iter().for_each(|entry| process(entry));
// Rayon creates thousands of micro-tasks (overhead!)

// With chunking: Each thread gets substantial work
let chunks = chunk_data(logs, 10_000);  // 100 chunks
chunks.par_iter().for_each(|chunk| {
    // Each thread processes 10K items at once
    for entry in chunk {
        process(entry);
    }
});
// Only 100 tasks, less overhead, better cache locality
```

**Chunk Size Trade-offs:**

**Too Small (1-10 items per chunk):**
```
Pros:
- Perfect load balance
- Can handle skewed workloads

Cons:
- High task creation overhead
- Poor cache locality (random access)
- Thread synchronization overhead dominates

Example: 1M items, chunk_size=10
= 100,000 chunks
= 100,000 task creations
= Overhead: 100,000 * 1μs = 100ms (too much!)
```

**Too Large (100K-1M items per chunk):**
```
Pros:
- Low overhead (few tasks)
- Good cache locality

Cons:
- Poor load balance
- Can't utilize all cores
- One slow chunk delays entire pipeline

Example: 1M items, chunk_size=500K, 8 cores
= 2 chunks
= Only 2 cores busy, 6 cores idle
= Wasted parallelism
```

**Just Right (1K-10K items per chunk):**
```
Sweet spot: num_chunks = num_cores * 4 to 8

1M items, 8 cores, chunk_size=10K
= 100 chunks
= Each core gets ~12 chunks
= Good load balance with work stealing
= Low overhead: 100 * 1μs = 0.1ms (negligible)
```

**Adaptive Chunking:**
```rust
pub fn optimal_chunk_size(total_items: usize, num_cores: usize) -> usize {
    let target_chunks = num_cores * 4;
    let chunk_size = total_items / target_chunks;

    // Clamp to reasonable range
    chunk_size.max(1000).min(100_000)
}

// Example:
// 100K items, 8 cores → chunk_size = 100K / 32 = 3125
// 1M items, 8 cores  → chunk_size = 1M / 32 = 31250
// 10M items, 8 cores → chunk_size = 100K (clamped)
```

**Implementation:**
```rust
pub fn chunk_data<T>(data: Vec<T>, chunk_size: usize) -> Vec<Vec<T>> {
    data.chunks(chunk_size)
        .map(|chunk| chunk.to_vec())
        .collect()
}

// Or without copying:
pub fn chunk_data_ref<T>(data: &[T], chunk_size: usize) -> Vec<&[T]> {
    data.chunks(chunk_size).collect()
}
```

**Cache Effects:**
```
Chunk Size:     Cache Behavior:
100 bytes      L1 cache (32KB) - hot data, 1ns access
10KB           L2 cache (256KB) - warm data, 3ns access
100KB          L3 cache (8MB) - cool data, 15ns access
1MB+           RAM (GB) - cold data, 80ns access

Optimal: Keep chunk in L2/L3 cache
= 10KB to 100KB per chunk
```

**Work Stealing with Chunks:**
```
Initial distribution (8 cores, 16 chunks):
Core 0: [chunk0, chunk8]
Core 1: [chunk1, chunk9]
Core 2: [chunk2, chunk10]
Core 3: [chunk3, chunk11]
Core 4: [chunk4, chunk12]
Core 5: [chunk5, chunk13]
Core 6: [chunk6, chunk14]
Core 7: [chunk7, chunk15]

If Core 0 finishes early:
Core 0: → steals chunk9 from Core 1

Work stealing is more effective with smaller chunks:
- 16 chunks: Can steal 1/16 of work
- 1000 chunks: Can steal 1/1000 of work (fine-grained)
```

**Rayon's Automatic Chunking:**
```rust
use rayon::prelude::*;

// Rayon chooses chunk size automatically
data.par_iter().for_each(|item| process(item));

// Explicit chunking for control
data.par_chunks(10_000).for_each(|chunk| {
    for item in chunk {
        process(item);
    }
});

// Minimum chunk size (don't split below this)
data.par_iter()
    .with_min_len(1000)
    .for_each(|item| process(item));
```

**Benchmarking Chunk Sizes:**
```rust
#[test]
fn benchmark_chunk_sizes() {
    let data: Vec<u64> = (0..1_000_000).collect();

    for chunk_size in [100, 1_000, 10_000, 100_000] {
        let start = Instant::now();

        let chunks = data.chunks(chunk_size);
        let sum: u64 = chunks.par_bridge()
            .map(|chunk| chunk.iter().sum::<u64>())
            .sum();

        let elapsed = start.elapsed();
        println!("Chunk size {}: {:?}", chunk_size, elapsed);
    }
}

// Typical results (8 cores):
// Chunk size 100:     45ms  (too much overhead)
// Chunk size 1000:    12ms  (good)
// Chunk size 10000:   10ms  (optimal)
// Chunk size 100000:  15ms  (poor load balance)
```

**Domain-Specific Chunking:**

**Text Processing:**
```rust
// Chunk by lines (keep records intact)
fn chunk_by_lines(text: &str, lines_per_chunk: usize) -> Vec<Vec<&str>> {
    text.lines()
        .collect::<Vec<_>>()
        .chunks(lines_per_chunk)
        .map(|c| c.to_vec())
        .collect()
}
```

**Image Processing:**
```rust
// Chunk by rows (spatial locality)
fn chunk_image(image: &Image, rows_per_chunk: usize) -> Vec<ImageChunk> {
    image.rows()
        .chunks(rows_per_chunk)
        .map(|rows| ImageChunk::new(rows))
        .collect()
}
```

**Time Series:**
```rust
// Chunk by time windows
fn chunk_by_time(events: &[Event], window_size: Duration) -> Vec<Vec<Event>> {
    // Group events within same time window
    // Ensures temporal locality
    todo!()
}
```

---

### 5. Combiner Optimization and Local Aggregation

**What Is It?**
A combiner performs local aggregation within each map task before the shuffle phase, dramatically reducing the amount of data transferred between map and reduce.

**Without Combiner:**
```
Map output: 1M pairs → Shuffle 1M pairs → Reduce

Example:
Chunk 1 map output:
  [("endpoint1", 1), ("endpoint1", 1), ("endpoint1", 1), ..., ("endpoint1", 1)]
  (100 pairs for same endpoint)

Shuffle: Send all 100 pairs → Partition

Total shuffle: 1M pairs * 16 bytes = 16 MB
```

**With Combiner:**
```
Map output: 1M pairs → Combiner (local reduce) → 10K pairs → Shuffle → Reduce

Example:
Chunk 1 map output:
  [("endpoint1", 1), ("endpoint1", 1), ...]  (100 pairs)

Combiner: Local aggregation
  [("endpoint1", 100)]  (1 pair!)

Shuffle: Send 1 pair instead of 100 → 99% reduction!

Total shuffle: 10K pairs * 16 bytes = 160 KB (100x less!)
```

**Visual Comparison:**
```
WITHOUT COMBINER:
Map Chunk 1: [a:1, a:1, b:1, a:1] → Shuffle → [a:1, a:1, a:1, b:1]
Map Chunk 2: [b:1, c:1, b:1, a:1] → Shuffle → [a:1, b:1, b:1, c:1]
             ↓ ↓ ↓ ↓ ↓ ↓ ↓ ↓ (8 pairs shuffled)

WITH COMBINER:
Map Chunk 1: [a:1, a:1, b:1, a:1] → Combiner → [a:3, b:1] → Shuffle
Map Chunk 2: [b:1, c:1, b:1, a:1] → Combiner → [a:1, b:2, c:1] → Shuffle
             ↓ ↓ (5 pairs shuffled instead of 8 - 37% reduction)
```

**When Combiner Works:**

The combiner must be **associative** and **commutative**:

```rust
// GOOD: Sum (associative + commutative)
fn reduce_sum(values: Vec<u64>) -> u64 {
    values.into_iter().sum()
}

// (a + b) + c = a + (b + c)  ← Associative
// a + b = b + a                ← Commutative

// Can apply combiner:
[1, 2, 3, 4, 5, 6]
→ Combine: [(1+2+3), (4+5+6)] = [6, 15]
→ Reduce: 6 + 15 = 21 ✓

// GOOD: Count
fn reduce_count(values: Vec<u64>) -> u64 {
    values.len() as u64
}

// GOOD: Max/Min
fn reduce_max(values: Vec<u64>) -> u64 {
    values.into_iter().max().unwrap()
}

// GOOD: Average (with tuple)
fn reduce_avg(values: Vec<(f64, u64)>) -> (f64, u64) {
    values.into_iter()
        .fold((0.0, 0u64), |(sum, count), (s, c)| (sum + s, count + c))
}
// Combiner returns (sum, count), final reduce divides sum/count
```

**When Combiner DOESN'T Work:**

```rust
// BAD: Median (not associative)
fn reduce_median(values: Vec<u64>) -> u64 {
    let mut sorted = values.clone();
    sorted.sort();
    sorted[sorted.len() / 2]
}

// BAD: Mode (most frequent value - not associative)
fn reduce_mode(values: Vec<u64>) -> u64 {
    // Most frequent value
    let counts = count_frequencies(values);
    counts.into_iter().max_by_key(|(_, count)| *count).unwrap().0
}
```

**Implementation:**
```rust
pub fn local_reduce<K, V, F>(pairs: Vec<(K, V)>, combiner: F) -> Vec<(K, V)>
where
    K: Hash + Eq,
    F: Fn(Vec<V>) -> V,
{
    // Group by key
    let mut grouped: HashMap<K, Vec<V>> = HashMap::new();
    for (key, value) in pairs {
        grouped.entry(key).or_insert_with(Vec::new).push(value);
    }

    // Apply combiner to each key's values
    grouped.into_iter()
        .map(|(key, values)| (key, combiner(values)))
        .collect()
}

// Usage in map phase:
pub fn map_with_combiner<K, V, M, C>(
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
    let chunks = chunk_data(entries);

    chunks.into_par_iter()
        .flat_map(|chunk| {
            // Map phase
            let pairs: Vec<(K, V)> = chunk.iter().map(&mapper).collect();

            // Combiner: Local reduce within chunk
            local_reduce(pairs, &combiner)
        })
        .collect()
}
```

**Combiner Benefit Measurement:**
```rust
#[test]
fn test_combiner_benefit() {
    let entries: Vec<LogEntry> = (0..100_000).map(|i| LogEntry {
        endpoint: format!("/api/{}", i % 100),  // 100 unique endpoints
        ..Default::default()
    }).collect();

    // Without combiner
    let mr_no_combiner = ParallelMapReduce::new(1000).with_combiner(false);
    let pairs_no_combiner = mr_no_combiner.parallel_map(entries.clone(), |e| {
        (e.endpoint.clone(), 1u64)
    });
    println!("Without combiner: {} pairs", pairs_no_combiner.len());
    // Output: 100,000 pairs

    // With combiner
    let mr_combiner = ParallelMapReduce::new(1000).with_combiner(true);
    let pairs_combiner = mr_combiner.map_with_combiner(
        entries,
        |e| (e.endpoint.clone(), 1u64),
        |values| values.into_iter().sum(),
    );
    println!("With combiner: {} pairs", pairs_combiner.len());
    // Output: ~10,000 pairs (one per chunk per unique key)
    // 100 chunks * 100 keys = 10,000 (worst case)
    // Typically: ~1,000-2,000 pairs (90-98% reduction!)

    let reduction = 100.0 * (1.0 - pairs_combiner.len() as f64 / pairs_no_combiner.len() as f64);
    println!("Shuffle reduction: {:.1}%", reduction);
}
```

**Performance Impact:**
```
Real-world example: 1M log entries, 1000 unique endpoints

Without combiner:
- Map output: 1M pairs
- Shuffle: 1M pairs * 20 bytes = 20 MB
- Memory allocations: 1M
- Reduce input: 1M pairs to aggregate

With combiner (chunk_size=10K):
- Map output: 1M pairs
- Combiner: Aggregate locally → 100K pairs (100 chunks * 1000 keys)
- Shuffle: 100K pairs * 20 bytes = 2 MB (10x less!)
- Memory allocations: 100K (10x fewer)
- Reduce input: 100K pairs (10x less work)

Performance:
- Without: 500ms total (200ms map, 200ms shuffle, 100ms reduce)
- With:    300ms total (200ms map, 50ms shuffle, 50ms reduce)
- Speedup: 1.67x from combiner alone!
```

**Combiner in Hadoop/Spark:**
- Hadoop: Optional combiner class (same as reducer)
- Spark: Automatic combining in `reduceByKey`, `aggregateByKey`
- Can provide 2-10x speedup for large shuffles
- Essential for processing TB+ datasets (reduces network by 90%+)

---

### 6. Rayon's Parallel Iterators and Thread Pools

**What Is It?**
Rayon provides data-parallel programming through parallel iterators that automatically distribute work across a thread pool.

**Sequential vs Parallel Iterators:**
```rust
// Sequential iterator
let sum: u64 = data.iter()
    .map(|x| expensive(x))
    .filter(|x| x > &100)
    .sum();

// Parallel iterator (just add .par_iter())
use rayon::prelude::*;

let sum: u64 = data.par_iter()
    .map(|x| expensive(x))      // Runs in parallel
    .filter(|x| x > &100)        // Runs in parallel
    .sum();                      // Parallel reduction

// Same API, automatic parallelism!
```

**Rayon's Thread Pool:**
```rust
// Global thread pool (created automatically)
// Number of threads = logical CPU cores

use rayon::ThreadPoolBuilder;

// Custom thread pool
let pool = ThreadPoolBuilder::new()
    .num_threads(8)
    .build()
    .unwrap();

pool.install(|| {
    // Code runs in custom pool
    data.par_iter().for_each(|x| process(x));
});

// Default pool uses num_cpus::get() threads
// Typically: 8 threads on 8-core, 16 on 16-core, etc.
```

**Work Stealing:**
```
Rayon uses work-stealing scheduler:

Thread 0 deque: [task1, task2, task3, task4]
Thread 1 deque: [task5, task6]              ← Done early
Thread 2 deque: [task7, task8, task9]

Thread 1 is idle → steals from Thread 0:
Thread 0 deque: [task1, task2, task3]       ← Stolen task4
Thread 1 deque: [task4]                     ← Now working
Thread 2 deque: [task7, task8, task9]

Advantages:
- Automatic load balancing
- No manual work distribution
- Efficient: steal from tail (oldest work, likely bigger chunk)
```

**Parallel Operations:**

**map:**
```rust
// Transform each element
let results: Vec<_> = data.par_iter()
    .map(|x| x * 2)
    .collect();
```

**filter:**
```rust
// Keep elements matching predicate
let filtered: Vec<_> = data.par_iter()
    .filter(|x| x % 2 == 0)
    .collect();
```

**flat_map:**
```rust
// Map and flatten (crucial for map-reduce)
let pairs: Vec<_> = chunks.par_iter()
    .flat_map(|chunk| {
        chunk.iter().map(|x| (x.key, x.value)).collect::<Vec<_>>()
    })
    .collect();
```

**fold/reduce:**
```rust
// Parallel aggregation
let sum: u64 = data.par_iter()
    .fold(|| 0u64, |acc, x| acc + x)  // Per-thread accumulator
    .sum();  // Combine thread results

// Or simpler:
let sum: u64 = data.par_iter().sum();
```

**for_each:**
```rust
// Side effects (no return value)
data.par_iter().for_each(|x| {
    println!("Processing {}", x);
});
```

**Parallel Chunking:**
```rust
// Process in chunks
data.par_chunks(1000).for_each(|chunk| {
    // Each thread gets 1000-item chunk
    process_batch(chunk);
});

// Mutable chunks
data.par_chunks_mut(1000).for_each(|chunk| {
    for item in chunk {
        *item *= 2;  // Modify in place
    }
});
```

**Rayon vs Manual Threading:**

**Manual (painful):**
```rust
use std::thread;

let num_threads = 8;
let chunk_size = data.len() / num_threads;
let mut handles = vec![];

for i in 0..num_threads {
    let start = i * chunk_size;
    let end = if i == num_threads - 1 { data.len() } else { (i + 1) * chunk_size };
    let data_slice = &data[start..end];

    let handle = thread::spawn(move || {
        let mut local_sum = 0;
        for item in data_slice {
            local_sum += expensive(item);
        }
        local_sum
    });

    handles.push(handle);
}

let sum: u64 = handles.into_iter()
    .map(|h| h.join().unwrap())
    .sum();

// 20+ lines, manual work distribution, no work stealing
```

**Rayon (easy):**
```rust
use rayon::prelude::*;

let sum: u64 = data.par_iter()
    .map(|item| expensive(item))
    .sum();

// 3 lines, automatic parallelism, work stealing included
```

**Performance Characteristics:**
```
Overhead:
- Thread pool creation: ~1ms (one-time)
- Task spawn: ~50ns per task
- Work stealing: ~100ns per steal

Speedup (8 cores):
- Tiny tasks (<1μs): 2-4x (overhead dominates)
- Small tasks (10μs): 5-7x (good)
- Large tasks (>100μs): 7-8x (excellent, near-linear)
```

**Rayon Best Practices:**

1. **Use `par_iter()` not `iter().par_bridge()`:**
   ```rust
   // Good: Native parallel iterator
   data.par_iter().map(f).collect()

   // Bad: Bridge from sequential (more overhead)
   data.iter().par_bridge().map(f).collect()
   ```

2. **Minimize data copying:**
   ```rust
   // Good: Reference iteration
   data.par_iter().for_each(|x| process(x));

   // Bad: Cloning data
   data.clone().into_par_iter().for_each(|x| process(x));
   ```

3. **Use appropriate granularity:**
   ```rust
   // Good: Reasonable chunk size
   data.par_chunks(1000).for_each(process_chunk);

   // Bad: Tiny chunks (too much overhead)
   data.par_chunks(10).for_each(process_chunk);
   ```

4. **Avoid excessive synchronization:**
   ```rust
   // Bad: Locking in hot loop
   let counter = Mutex::new(0);
   data.par_iter().for_each(|_| {
       *counter.lock().unwrap() += 1;  // Serializes!
   });

   // Good: Per-thread accumulation
   let count = data.par_iter().count();  // Parallel, no locks
   ```

---

### 7. Send and Sync Traits for Thread Safety

**What Is It?**
`Send` and `Sync` are marker traits that ensure types can be safely used in concurrent contexts.

**Send:** Type can be transferred across thread boundaries
```rust
// T: Send means:
// Can move ownership from one thread to another

fn spawn_thread<T: Send>(data: T) {
    std::thread::spawn(move || {
        process(data);  // data moved into thread
    });
}
```

**Sync:** Type can be safely referenced from multiple threads
```rust
// T: Sync means:
// &T can be shared across threads
// (T is safe to access through shared reference concurrently)

fn share_across_threads<T: Sync>(data: &T) {
    std::thread::scope(|s| {
        s.spawn(|| read(data));  // Thread 1 reads
        s.spawn(|| read(data));  // Thread 2 reads
    });
}
```

**Relationship:**
```
T: Send + Sync
↓
Can move across threads AND can share references across threads

Examples:
- i32, u64, f64: Send + Sync (Copy types, immutable)
- String, Vec<T>: Send + Sync (if T: Send + Sync)
- Arc<T>: Send + Sync (if T: Send + Sync)
- Mutex<T>: Send + Sync (interior mutability with locking)
- AtomicUsize: Send + Sync (lock-free)
```

**Not Send:**
```rust
// Rc<T>: NOT Send
// (Reference counting not atomic, not thread-safe)

let rc = Rc::new(42);
// Can't send to thread:
// std::thread::spawn(move || println!("{}", rc));  // ERROR!

// Use Arc instead (atomic reference counting)
let arc = Arc::new(42);
std::thread::spawn(move || println!("{}", arc));  // OK!
```

**Not Sync:**
```rust
// Cell<T>: NOT Sync
// (Interior mutability without atomics, not thread-safe)

let cell = Cell::new(42);
// Can't share across threads:
// std::thread::scope(|s| {
//     s.spawn(|| cell.set(100));  // ERROR!
// });

// Use Mutex or AtomicUsize instead
let mutex = Mutex::new(42);
std::thread::scope(|s| {
    s.spawn(|| *mutex.lock().unwrap() = 100);  // OK!
});
```

**Map-Reduce Requirements:**
```rust
pub fn parallel_map<K, V, F>(&self, entries: Vec<LogEntry>, mapper: F) -> Vec<(K, V)>
where
    K: Send,              // Keys must be sendable (moved across threads)
    V: Send,              // Values must be sendable
    F: Fn(&LogEntry) -> (K, V) + Sync + Send,
//     ^^^                         ^^^^   ^^^^
//     Can be called                |      |
//                                  |      Can move closure to thread
//                                  Can share closure across threads
{
    chunks.par_iter()
        .flat_map(|chunk| chunk.iter().map(&mapper).collect::<Vec<_>>())
        .collect()
}
```

**Why these bounds?**

**K: Send, V: Send:**
```rust
// Each thread creates (K, V) pairs
// Pairs must be moved back to main thread for collection
// Therefore K and V must be Send

let pairs: Vec<(K, V)> = chunks.par_iter()
    .flat_map(|chunk| {
        // Thread creates pairs
        chunk.iter().map(|entry| {
            let key: K = ...;    // Created in thread
            let value: V = ...;  // Created in thread
            (key, value)         // Moved to main thread (requires Send)
        }).collect()
    })
    .collect();
```

**F: Sync:**
```rust
// Mapper closure is shared across all threads
// Each thread calls &mapper (shared reference)
// Therefore F must be Sync

chunks.par_iter()  // Multiple threads
    .flat_map(|chunk| {
        chunk.iter().map(&mapper)  // Each thread uses &mapper (requires Sync)
    })
```

**F: Send:**
```rust
// Rayon may need to move closure between threads for work stealing
// Therefore F must be Send
```

**Common Mistakes:**

**Mistake 1: Using Rc in parallel code**
```rust
use std::rc::Rc;

let data = Rc::new(vec![1, 2, 3]);

// ERROR: Rc is not Send
data.par_iter().for_each(|x| process(x));
//   ^^^^^^^^ Rc is not Send

// Fix: Use Arc
use std::sync::Arc;
let data = Arc::new(vec![1, 2, 3]);
data.par_iter().for_each(|x| process(x));  // OK!
```

**Mistake 2: Capturing non-Send in closure**
```rust
let rc = Rc::new(42);

// ERROR: Closure captures Rc, not Send
data.par_iter().for_each(|x| {
    println!("{}", *rc);  // Captures rc
});

// Fix: Don't capture, or use Arc
let arc = Arc::new(42);
data.par_iter().for_each(|x| {
    println!("{}", *arc);  // OK!
});
```

**Mistake 3: Mutating shared state without synchronization**
```rust
let mut counter = 0;

// ERROR: Cannot mutate counter from multiple threads
data.par_iter().for_each(|_| {
    counter += 1;  // Data race!
});

// Fix: Use AtomicUsize or Mutex
use std::sync::atomic::{AtomicUsize, Ordering};

let counter = AtomicUsize::new(0);
data.par_iter().for_each(|_| {
    counter.fetch_add(1, Ordering::Relaxed);  // OK!
});
```

**Auto-Derive Rules:**
```rust
// Send is auto-derived if all fields are Send
struct MyStruct {
    field1: String,   // String: Send
    field2: Vec<u32>, // Vec<u32>: Send
}
// MyStruct: Send (automatically)

// Sync is auto-derived if all fields are Sync
struct MyStruct2 {
    field1: i32,      // i32: Sync
    field2: String,   // String: Sync
}
// MyStruct2: Sync (automatically)

// One non-Send field breaks Send
struct NotSend {
    field: Rc<u32>,  // Rc: not Send
}
// NotSend: not Send

// Explicitly opt-out (unsafe!)
unsafe impl<T> Send for MyWrapper<T> {}
unsafe impl<T> Sync for MyWrapper<T> {}
// Only do this if you've ensured thread safety manually!
```

---

### 8. HashMap Operations and Reduce Patterns

**What Is It?**
HashMap-based aggregation is the core of the reduce phase, grouping values by key and applying reduction functions.

**Basic Reduce Pattern:**
```rust
use std::collections::HashMap;

fn reduce<K, V, F>(pairs: Vec<(K, V)>, reducer: F) -> HashMap<K, V>
where
    K: Hash + Eq,
    F: Fn(V, V) -> V,
{
    let mut result = HashMap::new();

    for (key, value) in pairs {
        result.entry(key)
            .and_modify(|existing| *existing = reducer(*existing, value))
            .or_insert(value);
    }

    result
}

// Usage:
let pairs = vec![("a", 1), ("b", 2), ("a", 3), ("b", 4)];
let sums = reduce(pairs, |a, b| a + b);
// Result: {"a": 4, "b": 6}
```

**Entry API:**
```rust
// Method 1: entry + and_modify + or_insert
map.entry(key)
    .and_modify(|v| *v += 1)
    .or_insert(1);

// Method 2: entry + or_insert_with + modify
*map.entry(key).or_insert(0) += 1;

// Method 3: entry match
match map.entry(key) {
    Entry::Occupied(mut e) => *e.get_mut() += 1,
    Entry::Vacant(e) => { e.insert(1); }
}
```

**Common Reduce Patterns:**

**1. Count (most common):**
```rust
fn count_by_key<K: Hash + Eq>(pairs: Vec<(K, u64)>) -> HashMap<K, u64> {
    let mut counts = HashMap::new();
    for (key, value) in pairs {
        *counts.entry(key).or_insert(0) += value;
    }
    counts
}

// Or with reducer:
reduce(pairs, |a, b| a + b)
```

**2. Sum:**
```rust
fn sum_by_key<K: Hash + Eq>(pairs: Vec<(K, f64)>) -> HashMap<K, f64> {
    let mut sums = HashMap::new();
    for (key, value) in pairs {
        *sums.entry(key).or_insert(0.0) += value;
    }
    sums
}
```

**3. Average (requires tuple):**
```rust
fn average_by_key<K: Hash + Eq>(pairs: Vec<(K, f64)>) -> HashMap<K, f64> {
    // First: Accumulate (sum, count)
    let mut acc: HashMap<K, (f64, u64)> = HashMap::new();
    for (key, value) in pairs {
        let entry = acc.entry(key).or_insert((0.0, 0));
        entry.0 += value;
        entry.1 += 1;
    }

    // Then: Divide sum by count
    acc.into_iter()
        .map(|(k, (sum, count))| (k, sum / count as f64))
        .collect()
}
```

**4. Min/Max:**
```rust
fn max_by_key<K: Hash + Eq, V: Ord>(pairs: Vec<(K, V)>) -> HashMap<K, V> {
    let mut maxes = HashMap::new();
    for (key, value) in pairs {
        maxes.entry(key)
            .and_modify(|v| { if value > *v { *v = value.clone(); }})
            .or_insert(value);
    }
    maxes
}
```

**5. Collect into Vec:**
```rust
fn collect_by_key<K: Hash + Eq, V>(pairs: Vec<(K, V)>) -> HashMap<K, Vec<V>> {
    let mut grouped = HashMap::new();
    for (key, value) in pairs {
        grouped.entry(key).or_insert_with(Vec::new).push(value);
    }
    grouped
}
```

**6. Set Union:**
```rust
use std::collections::HashSet;

fn union_by_key<K: Hash + Eq, V: Hash + Eq>(
    pairs: Vec<(K, HashSet<V>)>
) -> HashMap<K, HashSet<V>> {
    let mut result = HashMap::new();
    for (key, set) in pairs {
        result.entry(key)
            .and_modify(|existing| existing.extend(set.clone()))
            .or_insert(set);
    }
    result
}
```

**Performance Considerations:**

**HashMap Capacity:**
```rust
// Pre-allocate if size known
let mut map = HashMap::with_capacity(10000);

// Vs default (starts at 0, grows dynamically)
let mut map = HashMap::new();

// with_capacity avoids reallocation:
// - Default: ~10 reallocations for 10K items
// - Pre-allocated: 0 reallocations
// Speedup: 2-3x for insert-heavy workloads
```

**Hash Function:**
```rust
// Rust uses SipHash-1-3 (cryptographic)
// - Secure against hash collision attacks
// - Slower than non-crypto hashes

// For performance-critical code (trusted input):
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use rustc_hash::FxHasher;

let mut fast_map: HashMap<String, u64, BuildHasherDefault<FxHasher>> =
    HashMap::default();

// FxHasher: 2-3x faster than SipHash
// But: Vulnerable to collision attacks (DoS)
```

**Parallel Reduce with DashMap:**
```rust
// DashMap: Concurrent HashMap (lock-free)
use dashmap::DashMap;
use rayon::prelude::*;

let map = DashMap::new();

pairs.par_iter().for_each(|(key, value)| {
    map.entry(key.clone())
        .and_modify(|v| *v += value)
        .or_insert(*value);
});

let result: HashMap<_, _> = map.into_iter().collect();

// DashMap allows concurrent inserts without locking entire map
// Speedup: 3-5x over Mutex<HashMap> for high contention
```

---

### 9. Pipeline Composition and Multi-Stage Processing

**What Is It?**
Chaining multiple map-reduce operations to build complex analytics workflows.

**Single-Stage:**
```rust
// Count requests by endpoint
let counts = map_reduce(
    logs,
    |e| (e.endpoint.clone(), 1u64),
    |values| values.into_iter().sum()
);
```

**Multi-Stage Pipeline:**
```rust
// Stage 1: Filter errors
let errors = logs.into_par_iter()
    .filter(|e| e.level == LogLevel::ERROR)
    .collect();

// Stage 2: Count by endpoint
let error_counts = map_reduce(
    errors,
    |e| (e.endpoint.clone(), 1u64),
    |values| values.into_iter().sum()
);

// Stage 3: Find top 10
let mut sorted: Vec<_> = error_counts.into_iter().collect();
sorted.sort_by(|a, b| b.1.cmp(&a.1));
let top10 = sorted.into_iter().take(10).collect::<Vec<_>>();
```

**Builder Pattern:**
```rust
pub struct PipelineBuilder<'a> {
    mr: &'a ParallelMapReduce,
}

impl<'a> PipelineBuilder<'a> {
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
            |v| v.into_iter().fold((0.0, 0u64), |(s, c), (val_s, val_c)| {
                (s + val_s, c + val_c)
            }),
        );

        result.into_iter()
            .map(|(k, (sum, count))| (k, sum / count as f64))
            .collect()
    }

    pub fn error_rate(&self, entries: Vec<LogEntry>) -> HashMap<String, f64> {
        let result = self.mr.map_reduce(
            entries,
            |e| {
                let is_error = if e.level == LogLevel::ERROR { 1u64 } else { 0u64 };
                (e.endpoint.clone(), (1u64, is_error))
            },
            |v| v.into_iter().fold((0, 0), |(total, errors), (t, e)| {
                (total + t, errors + e)
            }),
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
}

// Usage:
let builder = PipelineBuilder::new(&mr);
let counts = builder.count_by_endpoint(logs.clone());
let avg_times = builder.average_response_time(logs.clone());
let top10 = builder.top_k_endpoints(logs, 10);
```

**Complex Pipeline Example:**
```rust
// Analyze error patterns by hour and endpoint
fn analyze_errors(logs: Vec<LogEntry>) -> HashMap<(String, String), ErrorStats> {
    // Stage 1: Filter errors
    let errors: Vec<_> = logs.into_par_iter()
        .filter(|e| e.level == LogLevel::ERROR)
        .collect();

    // Stage 2: Extract hour and endpoint
    let keyed: Vec<_> = errors.into_par_iter()
        .map(|e| {
            let hour = e.timestamp[..13].to_string();  // "2024-01-15T10"
            let key = (hour, e.endpoint.clone());
            let value = ErrorStats {
                count: 1,
                status_codes: vec![e.status_code],
            };
            (key, value)
        })
        .collect();

    // Stage 3: Aggregate by (hour, endpoint)
    let mr = ParallelMapReduce::new(1000);
    mr.map_reduce(
        keyed,
        |pair| pair.clone(),
        |values| {
            ErrorStats {
                count: values.iter().map(|v| v.count).sum(),
                status_codes: values.into_iter()
                    .flat_map(|v| v.status_codes)
                    .collect(),
            }
        },
    )
}
```

---

### 10. Performance Analysis and Scalability

**What Is It?**
Understanding speedup, efficiency, and scalability characteristics of parallel map-reduce.

**Amdahl's Law (Data Parallel):**
```
Sequential portion is typically small in map-reduce:
- Map: 90% (parallelizable)
- Shuffle: 5% (sequential - partitioning overhead)
- Reduce: 90% (parallelizable)
- Merge: 5% (sequential - final collection)

Effective parallel fraction: 90%
Sequential fraction: 10%

Speedup with 8 cores:
Speedup = 1 / (0.10 + 0.90/8) = 1 / 0.2125 ≈ 4.7x

Maximum speedup (infinite cores):
Speedup = 1 / 0.10 = 10x
```

**Strong Scaling (Fixed Problem Size):**
```
Dataset: 1M log entries

Cores:  Time:   Speedup:  Efficiency:
1       1000ms  1.00x     100%
2       550ms   1.82x     91%
4       300ms   3.33x     83%
8       180ms   5.56x     69%
16      120ms   8.33x     52%

Observations:
- Speedup sublinear (not 2x, 4x, 8x)
- Efficiency decreases with more cores
- Overhead becomes significant (shuffle, synchronization)
```

**Weak Scaling (Problem Size Scales with Cores):**
```
Cores:  Dataset:    Time:    Efficiency:
1       125K        125ms    100%
2       250K        135ms    93%
4       500K        145ms    86%
8       1M          160ms    78%
16      2M          190ms    66%

Better efficiency than strong scaling
But still degrades due to:
- Shuffle overhead grows with partitions
- Cache contention increases
```

**Combiner Impact:**
```
Without combiner:
- Shuffle: 200ms (transferring 10M pairs)
- Total: 600ms

With combiner (90% reduction):
- Shuffle: 20ms (transferring 1M pairs)
- Total: 420ms

Speedup from combiner: 1.43x
```

**Optimal Chunk Size Analysis:**
```
Dataset: 1M entries, 8 cores

Chunk size:  Chunks:  Overhead:  Load Balance:  Total Time:
100          10,000   50ms       Perfect        250ms
1,000        1,000    5ms        Perfect        155ms
10,000       100      0.5ms      Good           150ms  ← Optimal
100,000      10       0.05ms     Poor           200ms

Too small: Overhead dominates
Too large: Poor load balance
Optimal: 10-20x more chunks than cores
```

**Scalability Limits:**

1. **Memory Bandwidth:**
   ```
   8 cores, each processing 125K entries/s
   = 1M entries/s
   = 1M * 100 bytes = 100 MB/s

   System memory bandwidth: 40 GB/s
   Utilization: 100 MB / 40 GB = 0.25%

   Map-reduce is memory-bound, not CPU-bound!
   Adding more cores won't help beyond ~32 cores
   ```

2. **Shuffle Bottleneck:**
   ```
   Shuffle scales as O(n * p) where p = partitions

   For 1M items:
   4 partitions:  O(4M) operations
   8 partitions:  O(8M) operations
   16 partitions: O(16M) operations

   Linear growth in shuffle cost with parallelism
   ```

3. **Overhead:**
   ```
   Fixed overhead per parallel invocation:
   - Thread synchronization: ~1μs
   - Task creation: ~50ns
   - Memory allocation: ~100ns

   For 1000 chunks:
   Overhead = 1000 * 1μs = 1ms (negligible)

   For 100,000 chunks:
   Overhead = 100,000 * 1μs = 100ms (significant!)
   ```

**Real-World Performance:**
```
Log processing benchmark:
Dataset: 100 GB logs (1B entries)
Machine: 16-core, 64 GB RAM

Sequential: 30 minutes (single-threaded)
Parallel (8 cores): 4 minutes (7.5x speedup)
Parallel (16 cores): 2.5 minutes (12x speedup)
With combiner (16 cores): 1.5 minutes (20x speedup)

Throughput: 1B entries / 90s = 11M entries/sec
```

---

## Connection to This Project

This section maps the concepts explained above to specific milestones in the map-reduce project.

### Milestone 1: Sequential Log Processor

**Concepts Used:**
- **Basic Map-Reduce Pattern**: Implement sequential map (transform) and reduce (aggregate) operations on log entries
- **HashMap Operations**: Use entry API for counting and aggregation (`entry().or_insert()`)
- **Functional Composition**: Chain filter → map → reduce operations

**Key Insights:**
- Sequential baseline establishes correctness before adding parallelism
- O(n) time complexity - single-threaded processing
- Simple HashMap-based reduce pattern: group by key, aggregate values
- Foundation for understanding parallel speedup

**Why This Matters:**
This milestone teaches the map-reduce mental model without concurrency complexity. Students learn to decompose problems into map (transform), shuffle (group), and reduce (aggregate) phases sequentially.

---

### Milestone 2: Parallel Map Phase

**Concepts Used:**
- **Data Parallelism**: Apply same operation (map) to different data chunks simultaneously
- **Chunking Strategies**: Split 1M logs into 100 chunks for parallel processing
- **Rayon's Parallel Iterators**: Use `par_iter()` and `flat_map()` for automatic parallelism
- **Send Trait**: Key-value pairs must be `Send` to transfer between threads

**Key Insights:**
- Map phase is embarrassingly parallel (no dependencies between chunks)
- Expected speedup: 6-8x on 8 cores (near-linear for map phase alone)
- Chunk size = dataset_size / (num_cores * 4) for optimal balance
- Rayon handles work stealing automatically

**Performance:**
```
Sequential map: 10 GB in 60s
Parallel map (8 cores): 10 GB in 8s (7.5x speedup)
Overhead: ~5% (chunking, thread coordination)
```

**Why This Matters:**
Students learn that parallelizing the computation-heavy map phase provides most of the speedup. This milestone alone can achieve 6-8x performance improvement.

---

### Milestone 3: Shuffle/Partition Phase

**Concepts Used:**
- **Hash-Based Partitioning**: Distribute pairs across partitions using `hash(key) % num_partitions`
- **Deterministic Hashing**: Same key always maps to same partition (correctness requirement)
- **HashMap for Grouping**: Collect values per key within each partition (`HashMap<K, Vec<V>>`)

**Key Insights:**
- Shuffle is necessary for correctness (group all values for same key)
- Deterministic hashing ensures reproducible results
- Hash function provides even distribution (~125K items per partition for 1M items, 8 partitions)
- Shuffle typically 5-10% of total time (fast compared to map/reduce)

**Algorithm:**
```
1. Hash each key
2. Assign to partition: partition_id = hash % num_partitions
3. Group values by key within each partition
4. Result: Vec<HashMap<K, Vec<V>>> ready for parallel reduce
```

**Why This Matters:**
Students learn that data distribution strategy is crucial for parallel correctness. Hash partitioning is the standard in all production map-reduce systems (Hadoop, Spark, Flink).

---

### Milestone 4: Parallel Reduce Phase

**Concepts Used:**
- **Independent Partition Reduction**: Each partition can be reduced concurrently (no dependencies)
- **Sync + Send Traits**: Reducer function must be `Sync` (shared) and `Send` (movable)
- **Par_iter on Partitions**: Use `partitions.into_par_iter()` for parallel reduction
- **Reduce Patterns**: Sum, count, average, min/max aggregations

**Key Insights:**
- Reduce phase is parallelizable because partitions are independent
- Expected speedup: 6-8x on 8 cores (if keys evenly distributed)
- Load imbalance occurs if some partitions have many more keys
- Final merge is sequential but negligible (just collecting HashMap results)

**Complete Pipeline:**
```
1. parallel_map(): Vec<LogEntry> → Vec<(K, V)>    [parallel, 200ms]
2. shuffle():      Vec<(K, V)> → Vec<HashMap<K, Vec<V>>>  [sequential, 20ms]
3. parallel_reduce(): Vec<HashMap<K, Vec<V>>> → HashMap<K, V>  [parallel, 80ms]

Total: ~300ms (vs 1000ms sequential)
Speedup: 3.3x end-to-end
```

**Why This Matters:**
Students see the complete parallel map-reduce pipeline. The reduce phase completes the parallelization, enabling concurrent aggregation across partitions.

---

### Milestone 5: Combiner Optimization

**Concepts Used:**
- **Local Aggregation**: Pre-reduce within each map chunk before shuffle
- **Combiner Function**: Must be associative and commutative (same function as reducer)
- **Shuffle Reduction**: Combiner typically reduces shuffle data by 50-99%
- **Memory Optimization**: Fewer allocations, less memory pressure

**Key Insights:**
- Combiner dramatically reduces shuffle overhead (100K pairs vs 10M pairs)
- Works for sum, count, max/min, average (with tuples)
- Doesn't work for median, mode (non-associative)
- Trade-off: Small CPU cost for local reduce vs huge shuffle savings

**Performance Impact:**
```
1M logs, 1000 unique keys, 8 cores

Without combiner:
- Map output: 1M pairs
- Shuffle: 1M pairs * 20 bytes = 20 MB
- Reduce input: 1M pairs
- Time: 300ms

With combiner:
- Map output: 1M pairs
- Combiner: 1M → 100K pairs (per-chunk aggregation)
- Shuffle: 100K pairs * 20 bytes = 2 MB (10x less!)
- Reduce input: 100K pairs
- Time: 200ms (1.5x speedup from combiner alone)
```

**Why This Matters:**
Students learn that network/memory transfers often dominate computation. Combiner optimization mirrors Hadoop/Spark combiner, essential for processing TB+ datasets where shuffle is the bottleneck.

---

### Milestone 6: Multi-Stage Pipelines

**Concepts Used:**
- **Pipeline Composition**: Chain multiple map-reduce stages (filter → count → top-K)
- **Builder Pattern**: Provide ergonomic API for common operations
- **Intermediate Results**: Output of one stage becomes input to next
- **Complex Analytics**: error_rate, average_response_time, top_k_endpoints

**Key Insights:**
- Real-world analytics require multiple transformations
- Each stage can be independently parallelized
- Builder pattern simplifies common patterns (count, average, top-K)
- Pipeline overhead minimal if intermediate datasets small

**Example Pipeline:**
```
Stage 1: Filter ERROR logs
  1M logs → 100K errors (10% error rate)
  Time: 50ms (parallel filter)

Stage 2: Count by endpoint
  100K errors → map-reduce → HashMap<String, u64>
  Time: 100ms (parallel map-reduce)

Stage 3: Find top 10
  1000 unique endpoints → sort → top 10
  Time: 5ms (sequential sort, small dataset)

Total: 155ms (vs 1000ms sequential)
Speedup: 6.5x
```

**Why This Matters:**
Students learn to build complex analytics workflows by composing simple operations. This mirrors production data pipelines in Spark, Hadoop, and modern data platforms.

---

## Summary Table

| Milestone | Key Concepts | Expected Speedup | Main Focus |
|-----------|--------------|------------------|------------|
| M1: Sequential | Map-reduce pattern, HashMap reduce | 1x (baseline) | Correctness & mental model |
| M2: Parallel Map | Data parallelism, Chunking, Rayon, Send | 6-8x | Parallelizing computation |
| M3: Shuffle | Hash partitioning, Deterministic grouping | N/A (correctness) | Data distribution |
| M4: Parallel Reduce | Independent partitions, Sync+Send traits | 3-4x end-to-end | Complete parallelization |
| M5: Combiner | Local aggregation, Shuffle reduction | 1.5-2x | Memory/network optimization |
| M6: Pipelines | Composition, Builder pattern, Multi-stage | N/A (usability) | Real-world workflows |

**Overall Learning:**
Map-reduce is the foundational pattern for data-parallel processing at scale. This project demonstrates:
- **8-16x speedup** from parallelization (M2 + M4)
- **2-10x additional speedup** from combiner optimization (M5)
- Total: **16-160x speedup** possible (compute + network optimization)

The framework scales from single machine (this project) to distributed systems (Hadoop/Spark) using the same conceptual model. Understanding single-machine map-reduce is essential for working with modern big data systems.

---

# Build The Project

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