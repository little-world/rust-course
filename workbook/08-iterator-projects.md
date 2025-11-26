# Chapter 08: Iterator Patterns & Combinators - Programming Projects

## Project 1: Log Analysis Pipeline

### Problem Statement

Build a streaming log analyzer that processes large server log files (potentially larger than available RAM) to extract insights without loading the entire file into memory. The analyzer should parse log entries, filter based on various criteria, compute statistics, and identify patterns - all while maintaining constant memory usage regardless of file size.

Your log analyzer should support:
- Parsing log entries with timestamp, log level, service name, and message
- Filtering by time range, log level, and service
- Computing statistics (error rates, request counts per service)
- Finding top-K most frequent error messages
- Generating summary reports

Example log format:
```
2024-11-26T10:15:30.123Z INFO auth-service User login successful: user_id=12345
2024-11-26T10:15:31.456Z ERROR payment-service Payment processing failed: insufficient_funds
2024-11-26T10:15:32.789Z WARN cache-service Cache miss rate exceeding threshold: 45%
```

### Why It Matters

Log analysis is a fundamental DevOps task. Production systems generate gigabytes of logs daily, making it impossible to load them entirely into memory. Iterator-based streaming processing enables analyzing logs of any size with constant memory usage. This pattern applies to any large-scale data processing: ETL pipelines, data analytics, monitoring systems, and real-time stream processing.

### Use Cases

- DevOps: Analyzing server logs to identify errors, performance bottlenecks, or security incidents
- Security: Detecting suspicious patterns in access logs
- Analytics: Computing metrics from event streams
- Monitoring: Real-time alerting based on log patterns
- Debugging: Finding specific errors in production logs

### Solution Outline

Your solution should follow these stepping stones:

#### Step 1: Basic Log Entry Parser
**Goal**: Create a `LogEntry` struct and parse a single log line.

**What to implement**:
- Define `LogEntry` struct with fields: timestamp, level, service, message
- Implement `LogEntry::parse(line: &str) -> Result<LogEntry, ParseError>`
- Handle invalid log formats gracefully

**Why this step**: Before processing logs, you need a data structure and parser. This establishes the foundation for all subsequent work.

**Testing hint**: Test with valid and invalid log lines. Verify all fields are extracted correctly.

```rust
// Example structure
struct LogEntry {
    timestamp: String,
    level: LogLevel,
    service: String,
    message: String,
}

enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}
```

---

#### Step 2: File Iterator Without Loading Entire File
**Goal**: Create an iterator that reads log file line-by-line.

**What to implement**:
- Use `BufReader` to read file line-by-line
- Create iterator that yields `Result<LogEntry, Error>`
- Ensure no more than one line is in memory at a time

**Why the previous step is not enough**: Step 1 parses a single line, but we need to process entire files without loading them into memory. A naive approach would read all lines into a `Vec`, causing OOM errors with large files.

**What's the improvement**: Using `BufReader::lines()` with iterators enables streaming - only the current line occupies memory. This allows processing files of any size with constant O(1) memory usage (excluding statistics storage).

**Testing hint**: Test with a small file, then create a large test file (e.g., 100MB) and verify your program's memory usage stays constant. Use `time` command or memory profilers.

```rust
// Example iterator structure
struct LogFileIterator {
    reader: BufReader<File>,
}

impl Iterator for LogFileIterator {
    type Item = Result<LogEntry, Error>;
    // ...
}
```

---

#### Step 3: Filtering Pipeline with Zero Allocation
**Goal**: Add filtering by log level and time range without intermediate collections.

**What to implement**:
- Filter by log level (e.g., only ERROR and WARN)
- Filter by time range (logs between two timestamps)
- Chain filters using iterator adapters
- Do NOT call `.collect()` between filters

**Why the previous step is not enough**: Reading files is useful, but we need to filter relevant logs. A naive approach creates intermediate `Vec`s for each filter step, wasting memory.

**What's the improvement**: Iterator adapter composition (`.filter().filter()`) creates a single-pass pipeline. All filters are evaluated on-the-fly as elements flow through, with zero intermediate allocations. This is 10-100x faster than collecting after each step.

**Testing hint**: Profile memory usage with multiple filters. Compare with a version that collects after each filter to see the difference. Verify filtered results are correct.

```rust
// Example usage
log_iterator
    .filter(|entry| matches!(entry.level, LogLevel::Error | LogLevel::Warn))
    .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
```

---

#### Step 4: Statistics Aggregation with Fold
**Goal**: Compute statistics without storing all log entries.

**What to implement**:
- Count total logs, errors, warnings, info messages
- Count logs per service using `HashMap<String, usize>`
- Use `.fold()` to accumulate statistics in a single pass
- Calculate error rate percentage

**Why the previous step is not enough**: Filtering finds relevant logs, but we need insights. Storing all entries to count them later wastes memory.

**What's the improvement**: Using `.fold()` maintains only aggregate statistics (counts, sums) rather than storing every log entry. Memory usage is O(k) where k is the number of unique services, not O(n) where n is total log lines. For a file with 1M logs and 10 services, this is ~10KB vs ~100MB.

**Testing hint**: Verify statistics are correct by testing with known inputs. Test with logs from multiple services. Ensure percentages are calculated correctly.

```rust
struct LogStats {
    total: usize,
    errors: usize,
    warnings: usize,
    info: usize,
    per_service: HashMap<String, ServiceStats>,
}

// Use fold to accumulate
let stats = log_iterator.fold(LogStats::new(), |mut stats, entry| {
    stats.update(entry);
    stats
});
```

---

#### Step 5: Top-K Error Messages with Min-Heap
**Goal**: Find the K most frequent error messages efficiently.

**What to implement**:
- Count error message frequencies using `HashMap<String, usize>`
- Implement streaming Top-K algorithm with `BinaryHeap`
- Maintain heap of size K (not full dataset)
- Return top K most frequent errors

**Why the previous step is not enough**: We have aggregate counts but need to identify the most common errors. Sorting all error messages requires O(n log n) time and O(n) space.

**What's the improvement**: A min-heap of size K maintains only the top K elements, using O(K) space instead of O(n). For finding top 10 errors from 100K unique error messages, this is ~10 entries in memory vs 100K. Time complexity improves to O(n log K) instead of O(n log n).

**Optimization focus**: Space optimization - minimal memory footprint while maintaining accuracy.

**Testing hint**: Test with known frequency distributions. Verify heap maintains correct top-K. Test edge cases (K larger than unique messages).

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

fn top_k_errors(entries: impl Iterator<Item = LogEntry>, k: usize) -> Vec<(String, usize)> {
    // Count frequencies
    let mut frequencies = HashMap::new();
    for entry in entries.filter(|e| e.level == LogLevel::Error) {
        *frequencies.entry(entry.message.clone()).or_insert(0) += 1;
    }

    // Find top K using min-heap
    // Implementation here
}
```

---

#### Step 6: Parallel Processing with Rayon
**Goal**: Speed up analysis of large files using multiple CPU cores.

**What to implement**:
- Split file into chunks that can be processed independently
- Use Rayon's `.par_iter()` for parallel processing
- Combine results from parallel workers
- Compare performance: sequential vs parallel

**Why the previous step is not enough**: Steps 1-5 process logs sequentially, using only one CPU core. On an 8-core machine, we're using only 12.5% of available compute power.

**What's the improvement**: Parallel processing with Rayon distributes work across all CPU cores, providing near-linear speedup. For CPU-bound parsing and filtering, an 8-core system can achieve 7-8x speedup. This transforms "analyze 1GB in 10 minutes" into "analyze 1GB in ~75 seconds".

**Optimization focus**: Speed optimization through parallelism - using all available CPU cores.

**Implementation note**: File I/O may become the bottleneck, so chunk-based parallel processing works best. Read chunks into memory, then process chunks in parallel.

**Testing hint**: Use a large log file (100MB+). Time sequential vs parallel versions. Use `htop` or Activity Monitor to verify all cores are utilized. Calculate speedup ratio.

```rust
use rayon::prelude::*;

// Process chunks in parallel
let chunks: Vec<String> = read_file_in_chunks(path, chunk_size);
let results: Vec<LogStats> = chunks
    .par_iter()
    .map(|chunk| analyze_chunk(chunk))
    .collect();

// Combine results
let final_stats = results.into_iter()
    .fold(LogStats::new(), |mut acc, stats| {
        acc.merge(stats);
        acc
    });
```

---

### Testing Strategies

1. **Unit Tests**: Test each component (parser, filters, statistics) with small, controlled inputs
2. **Property Tests**: Use property-based testing to verify invariants (e.g., filtered count ≤ total count)
3. **Performance Tests**:
   - Generate large test files (100MB-1GB)
   - Measure memory usage with `/usr/bin/time -l` (macOS) or `/usr/bin/time -v` (Linux)
   - Verify constant memory usage regardless of file size
   - Compare sequential vs parallel performance
4. **Integration Tests**: Test complete pipeline with realistic log files
5. **Benchmark**: Use `cargo bench` with Criterion.rs to measure performance improvements

---

## Project 2: Custom Data Processing Pipeline with Iterator Adapters

### Problem Statement

Build a flexible, composable data processing framework using custom iterator adapters. Your framework should provide reusable combinators that can be chained together to express complex data transformations declaratively.

Implement these custom iterator adapters:
1. **Batch**: Group elements into fixed-size batches
2. **SlidingWindow**: Provide overlapping windows of elements
3. **Interleave**: Alternate elements from two iterators
4. **Deduplicate**: Remove consecutive duplicate elements
5. **TakeUntilCondition**: Take elements until a condition is met (including the matching element)
6. **MapWithIndex**: Map with access to element index
7. **InspectWith**: Debug adapter that applies a function for side effects

Then compose these adapters to solve real-world problems like:
- Processing sensor data streams with sliding window averages
- Batch processing API requests
- Deduplicating sorted data streams
- Complex ETL transformations

### Why It Matters

Iterator adapters are the building blocks of data processing pipelines. Creating custom adapters lets you encode domain-specific operations as reusable, composable components. This pattern is fundamental to functional programming and enables writing declarative, self-documenting code. Libraries like Itertools, async-stream, and tokio-stream use these patterns extensively.

### Use Cases

- Data Engineering: Building ETL pipelines with custom transformations
- Stream Processing: Real-time data processing with windowing and batching
- API Clients: Batching requests for efficiency
- Scientific Computing: Processing sensor data, time-series analysis
- Game Development: Entity processing, animation pipelines

### Solution Outline

#### Step 1: Batch Iterator Adapter
**Goal**: Create an iterator that groups elements into fixed-size batches.

**What to implement**:
- `Batch<I>` struct that wraps an iterator
- Implement `Iterator` trait yielding `Vec<T>` batches
- Handle last batch (may be smaller than batch_size)
- Create extension trait for chainability

**Why this step**: Batching is fundamental for many operations (batch API requests, parallel processing, database inserts). This establishes the pattern for creating custom adapters.

**Testing hint**: Test with exact multiples of batch size and with remainders. Test empty iterators.

```rust
struct Batch<I: Iterator> {
    iter: I,
    batch_size: usize,
}

impl<I: Iterator> Iterator for Batch<I> {
    type Item = Vec<I::Item>;
    // ...
}

trait BatchExt: Iterator {
    fn batch(self, size: usize) -> Batch<Self>
    where
        Self: Sized;
}

// Usage: (1..10).batch(3) yields [1,2,3], [4,5,6], [7,8,9]
```

---

#### Step 2: Sliding Window Iterator
**Goal**: Create overlapping windows of elements for moving averages or pattern detection.

**What to implement**:
- `SlidingWindow<I>` that maintains a buffer of size N
- Yields windows as they slide across the stream
- Use `VecDeque` for efficient push/pop
- Handle initial window fill-up

**Why the previous step is not enough**: Batch creates non-overlapping chunks, but many algorithms need overlapping windows (moving averages, pattern matching, signal processing).

**What's the improvement**: Sliding windows enable algorithms that need context from previous elements. Instead of processing elements in isolation, we can look at neighborhoods. This is essential for time-series analysis, smoothing filters, and pattern recognition.

**Testing hint**: Test window sliding behavior. Verify first window appears only when buffer is full. Test window contents at each step.

```rust
use std::collections::VecDeque;

struct SlidingWindow<I: Iterator> {
    iter: I,
    window: VecDeque<I::Item>,
    window_size: usize,
}

// Usage: data.sliding_window(3) for 3-element moving window
```

---

#### Step 3: Interleave and Deduplicate Adapters
**Goal**: Add more utility adapters for common patterns.

**What to implement**:
- `Interleave<I, J>`: Alternate between two iterators (a, b, a, b, ...)
- `Deduplicate<I>`: Skip consecutive duplicates (using `PartialEq`)
- Handle exhaustion of one iterator in Interleave
- Make both chainable with extension traits

**Why the previous step is not enough**: We have grouping and windowing but need more utility operators for real-world pipelines.

**What's the improvement**: These adapters handle common patterns (merging streams, deduplicating sorted data) that otherwise require manual state management. They compose with other adapters, building a rich vocabulary for data transformation.

**Testing hint**:
- Interleave: Test with equal and unequal length iterators
- Deduplicate: Test with consecutive and non-consecutive duplicates

```rust
struct Interleave<I, J> {
    a: I,
    b: J,
    use_a: bool,
}

struct Deduplicate<I: Iterator> {
    iter: I,
    last: Option<I::Item>,
}
```

---

#### Step 4: Advanced Adapters - TakeUntil and MapWithIndex
**Goal**: Implement adapters with more complex control flow.

**What to implement**:
- `TakeUntilCondition<I, F>`: Take elements until predicate matches, including the matching element
- `MapWithIndex<I, F>`: Map with both element and index
- Handle state correctly (index tracking, predicate evaluation)

**Why the previous step is not enough**: We need adapters that maintain internal state beyond just the wrapped iterator.

**What's the improvement**: These adapters demonstrate stateful iteration - maintaining counters or flags across `next()` calls. This pattern enables parsing, state machines, and conditional processing. MapWithIndex avoids the boilerplate of manually zipping with enumerate.

**Testing hint**:
- TakeUntilCondition: Verify it includes the matching element
- MapWithIndex: Verify indices are correct and start from 0

```rust
struct TakeUntilCondition<I, F> {
    iter: I,
    predicate: F,
    done: bool,
}

struct MapWithIndex<I, F> {
    iter: I,
    f: F,
    index: usize,
}
```

---

#### Step 5: Compose Adapters for Complex Processing
**Goal**: Combine your custom adapters to solve a real problem.

**What to implement**:
Build a sensor data processor that:
- Reads temperature readings from a stream
- Removes consecutive duplicate readings (Deduplicate)
- Computes 5-reading moving average (SlidingWindow)
- Batches results for efficient storage (Batch)
- Detects anomalies (readings beyond threshold)

**Why the previous step is not enough**: Individual adapters are useful, but real value comes from composition. We need to see how they work together.

**What's the improvement**: Declarative pipelines that read like specifications. Compare this iterator chain to equivalent imperative code with manual loops, state management, and temporary vectors. The iterator version is shorter, clearer, and just as fast (zero-cost abstraction).

**Testing hint**: Create test data with known patterns. Verify each stage of pipeline produces expected results. Test with edge cases (empty stream, single element, all duplicates).

```rust
fn process_sensor_data(readings: impl Iterator<Item = f64>) -> impl Iterator<Item = Vec<f64>> {
    readings
        .deduplicate()
        .sliding_window(5)
        .map(|window| window.iter().sum::<f64>() / window.len() as f64)
        .batch(10)
}
```

---

#### Step 6: Optimize with Early Termination and Lazy Evaluation
**Goal**: Ensure adapters are truly lazy and support early termination.

**What to implement**:
- Verify no computation happens until iteration starts
- Ensure `.take()` short-circuits your adapters
- Add `size_hint()` implementations for optimizations
- Test memory usage with infinite iterators

**Why the previous step is not enough**: Working adapters aren't enough - they must be truly lazy and efficient.

**What's the improvement**:
- **Laziness**: No work until `next()` is called. Creating `(1..).batch(1000)` should be instant, not compute infinite batches.
- **Short-circuiting**: `(1..).batch(100).take(5)` should process exactly 500 elements, not infinite.
- **size_hint()**: Enables optimizations in `collect()` and other consumers.

**Optimization focus**: Memory and speed through lazy evaluation and short-circuiting.

**Testing hint**: Test with infinite iterators + `.take()`. Use `std::hint::black_box` to prevent compiler optimizations in benchmarks. Profile memory usage.

```rust
// Verify laziness
let batch_iter = (1..).batch(1000); // Should be instant
let first_batch = batch_iter.next(); // Only now does computation happen

// Verify short-circuiting
let count = (1..)
    .map(|x| { println!("Processing {}", x); x })
    .batch(100)
    .take(2)
    .count();
// Should print exactly 200 times, not infinite
```

---

### Testing Strategies

1. **Unit Tests**: Test each adapter in isolation
2. **Composition Tests**: Test combinations of adapters
3. **Property Tests**: Use proptest to verify laws:
   - `iter.batch(n).flatten()` should equal `iter.collect()`
   - `iter.deduplicate()` has no consecutive duplicates
4. **Performance Tests**: Compare to manual implementations
5. **Infinite Iterator Tests**: Verify laziness and short-circuiting
6. **Benchmark**: Measure overhead compared to hand-written loops

---

## Project 3: Parallel Data Aggregation Framework

### Problem Statement

Build a high-performance data aggregation system that processes large datasets using parallel iterators. Your system should compute multiple aggregate statistics simultaneously while maximizing CPU utilization.

Implement aggregations for:
1. **Numerical Statistics**: Mean, variance, min, max, median, percentiles
2. **Grouping**: Group by key with per-group aggregates
3. **Top-K**: Find K largest/smallest elements efficiently
4. **Histogram**: Build frequency distributions
5. **Custom Aggregates**: User-defined aggregation functions

Your framework should:
- Process data in parallel using Rayon
- Compute multiple aggregates in a single pass
- Handle datasets larger than RAM using streaming
- Provide 5-10x speedup on multi-core systems

### Why It Matters

Data aggregation is central to analytics, monitoring, and data science. Computing aggregates in parallel is crucial for performance - analyzing 1GB of data should take seconds, not minutes. This project teaches parallel algorithm design, map-reduce patterns, and how to leverage Rust's zero-cost abstractions for high-performance computing.

### Use Cases

- Business Analytics: Computing KPIs from transaction logs
- System Monitoring: Aggregating metrics from distributed systems
- Data Science: Computing statistics for exploratory data analysis
- A/B Testing: Comparing treatment and control groups
- Financial Analysis: Computing portfolio statistics

### Solution Outline

#### Step 1: Sequential Single Aggregate
**Goal**: Compute basic statistics (mean, min, max, count) sequentially.

**What to implement**:
- `Stats` struct holding aggregate values
- Methods to update stats incrementally
- Process iterator element-by-element using `.fold()`

**Why this step**: Establish baseline sequential implementation and data structures before adding complexity.

**Testing hint**: Test with known datasets. Verify mean calculation. Test with empty iterators.

```rust
#[derive(Debug, Clone)]
struct Stats {
    count: usize,
    sum: f64,
    min: f64,
    max: f64,
}

impl Stats {
    fn new() -> Self { /* ... */ }

    fn update(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    fn mean(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f64)
        }
    }
}

fn compute_stats(data: &[f64]) -> Stats {
    data.iter().fold(Stats::new(), |mut stats, &value| {
        stats.update(value);
        stats
    })
}
```

---

#### Step 2: Variance and Standard Deviation (Welford's Algorithm)
**Goal**: Add variance computation using numerically stable streaming algorithm.

**What to implement**:
- Implement Welford's online algorithm for variance
- Update `Stats` struct to track mean and M2 (sum of squared differences)
- Add `variance()` and `std_dev()` methods

**Why the previous step is not enough**: Computing variance naively (collect all values, compute mean, then compute squared differences) requires two passes and stores all data. This fails for large datasets.

**What's the improvement**: Welford's algorithm computes variance in a single pass with O(1) memory and is numerically stable (avoids catastrophic cancellation). This is essential for streaming statistics.

**Optimization focus**: Memory (O(1) instead of O(n)) and numerical stability.

**Testing hint**: Test with known variance values. Test with large numbers (verify numerical stability). Compare with two-pass naive implementation.

```rust
struct Stats {
    count: usize,
    mean: f64,
    m2: f64,  // Sum of squared differences from mean
    min: f64,
    max: f64,
}

impl Stats {
    fn update(&mut self, value: f64) {
        self.count += 1;
        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
        // Update min/max
    }

    fn variance(&self) -> Option<f64> {
        if self.count < 2 {
            None
        } else {
            Some(self.m2 / (self.count - 1) as f64)
        }
    }
}
```

---

#### Step 3: Parallel Statistics with Rayon
**Goal**: Compute statistics in parallel across multiple CPU cores.

**What to implement**:
- Use Rayon's `.par_iter()` for parallel iteration
- Implement `merge()` method to combine partial stats
- Use `.fold()` and `.reduce()` pattern for parallel aggregation

**Why the previous step is not enough**: Sequential processing uses only one CPU core. On an 8-core machine, we're wasting 7 cores.

**What's the improvement**: Parallel processing provides near-linear speedup with core count. Processing 100M numbers:
- Sequential (1 core): ~2 seconds
- Parallel (8 cores): ~0.25 seconds (8x speedup)

**Optimization focus**: Speed through parallelism.

**Implementation challenge**: Merging partial statistics requires careful math. Merging means and variances from different partitions is non-trivial.

**Testing hint**: Compare parallel vs sequential results (should be identical). Benchmark with large datasets. Use `htop` to verify all cores are utilized.

```rust
impl Stats {
    fn merge(&mut self, other: Stats) {
        if other.count == 0 {
            return;
        }

        let total_count = self.count + other.count;

        // Merge mean and variance using parallel algorithm
        let delta = other.mean - self.mean;
        let merged_mean = self.mean + delta * (other.count as f64 / total_count as f64);

        self.m2 = self.m2 + other.m2 +
            delta * delta * (self.count * other.count) as f64 / total_count as f64;

        self.count = total_count;
        self.mean = merged_mean;
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }
}

use rayon::prelude::*;

fn parallel_stats(data: &[f64]) -> Stats {
    data.par_iter()
        .fold(Stats::new, |mut stats, &value| {
            stats.update(value);
            stats
        })
        .reduce(Stats::new, |mut a, b| {
            a.merge(b);
            a
        })
}
```

---

#### Step 4: Group By with Parallel Aggregation
**Goal**: Group data by key and compute per-group statistics in parallel.

**What to implement**:
- `GroupedStats<K>` using `HashMap<K, Stats>`
- Parallel group-by using Rayon's fold + reduce pattern
- Merge HashMaps from different partitions

**Why the previous step is not enough**: We can compute global statistics, but often we need per-category statistics (sales by region, errors by service, etc.).

**What's the improvement**: Parallel group-by enables analyzing millions of records grouped by thousands of categories in seconds. Sequential processing would take minutes.

**Testing hint**: Test with multiple groups. Verify per-group stats are correct. Test group merging logic.

```rust
use std::collections::HashMap;
use std::hash::Hash;

struct GroupedStats<K: Hash + Eq> {
    groups: HashMap<K, Stats>,
}

impl<K: Hash + Eq> GroupedStats<K> {
    fn update(&mut self, key: K, value: f64) {
        self.groups.entry(key).or_insert_with(Stats::new).update(value);
    }

    fn merge(&mut self, other: GroupedStats<K>) {
        for (key, stats) in other.groups {
            self.groups.entry(key)
                .and_modify(|s| s.merge(stats.clone()))
                .or_insert(stats);
        }
    }
}

fn parallel_group_stats<K>(data: &[(K, f64)]) -> GroupedStats<K>
where
    K: Hash + Eq + Clone + Send + Sync,
{
    data.par_iter()
        .fold(GroupedStats::new, |mut groups, (key, value)| {
            groups.update(key.clone(), *value);
            groups
        })
        .reduce(GroupedStats::new, |mut a, b| {
            a.merge(b);
            a
        })
}
```

---

#### Step 5: Streaming Percentiles and Histogram
**Goal**: Compute percentiles and histograms without storing all data.

**What to implement**:
- **T-Digest** or **Quantile Sketch** for approximate percentiles
- Histogram with fixed bins
- Parallel histogram construction

**Why the previous step is not enough**: Computing exact percentiles requires sorting all data (O(n log n) time, O(n) space). For 1 billion numbers, this is gigabytes of memory.

**What's the improvement**: Approximate percentile algorithms (T-Digest) maintain a compact sketch (~100KB) that gives percentiles within 0.1% error. Histograms provide distribution insights with O(bins) memory instead of O(n).

**Optimization focus**: Memory - O(k) instead of O(n) where k << n.

**Implementation note**: For simplicity, implement histogram with fixed bins. For a challenge, implement T-Digest.

**Testing hint**: Test histogram with known distributions. Verify bin counts sum to total count. Test edge cases (values outside histogram range).

```rust
struct Histogram {
    bins: Vec<usize>,
    min: f64,
    max: f64,
    bin_width: f64,
}

impl Histogram {
    fn new(min: f64, max: f64, num_bins: usize) -> Self {
        Histogram {
            bins: vec![0; num_bins],
            min,
            max,
            bin_width: (max - min) / num_bins as f64,
        }
    }

    fn add(&mut self, value: f64) {
        if value < self.min || value >= self.max {
            return;
        }
        let bin = ((value - self.min) / self.bin_width) as usize;
        self.bins[bin] += 1;
    }

    fn merge(&mut self, other: Histogram) {
        for (i, &count) in other.bins.iter().enumerate() {
            self.bins[i] += count;
        }
    }
}

fn parallel_histogram(data: &[f64], min: f64, max: f64, bins: usize) -> Histogram {
    data.par_iter()
        .fold(|| Histogram::new(min, max, bins), |mut hist, &value| {
            hist.add(value);
            hist
        })
        .reduce(|| Histogram::new(min, max, bins), |mut a, b| {
            a.merge(b);
            a
        })
}
```

---

#### Step 6: Complete Aggregation Framework
**Goal**: Combine all aggregations into a unified framework that computes everything in one pass.

**What to implement**:
- `Aggregator` struct that holds all aggregate states
- Single parallel pass computing: stats, grouped stats, histogram, top-K
- API for querying results

**Why the previous step is not enough**: Computing aggregates separately requires multiple passes over data, multiplying I/O and compute cost.

**What's the improvement**: Single-pass aggregation computes all statistics simultaneously. For 10 aggregates, this is 10x faster than 10 separate passes. This is the map-reduce pattern: map each element to updates for all aggregates, then reduce by merging.

**Optimization focus**: Speed through single-pass processing.

**Testing hint**: Verify all aggregates produce correct results. Benchmark single-pass vs multiple-pass. Test with large datasets (100M+ records).

```rust
struct Aggregator {
    stats: Stats,
    grouped: GroupedStats<String>,
    histogram: Histogram,
    top_k: TopK<f64>,
}

impl Aggregator {
    fn update(&mut self, record: &Record) {
        self.stats.update(record.value);
        self.grouped.update(record.category.clone(), record.value);
        self.histogram.add(record.value);
        self.top_k.add(record.value);
    }

    fn merge(&mut self, other: Aggregator) {
        self.stats.merge(other.stats);
        self.grouped.merge(other.grouped);
        self.histogram.merge(other.histogram);
        self.top_k.merge(other.top_k);
    }
}

fn aggregate_all(data: &[Record]) -> Aggregator {
    data.par_iter()
        .fold(Aggregator::new, |mut agg, record| {
            agg.update(record);
            agg
        })
        .reduce(Aggregator::new, |mut a, b| {
            a.merge(b);
            a
        })
}

// Query results
let result = aggregate_all(&records);
println!("Mean: {}", result.stats.mean());
println!("Variance: {}", result.stats.variance());
println!("Top 10 values: {:?}", result.top_k.top(10));
println!("Histogram: {:?}", result.histogram.bins);
```

---

### Testing Strategies

1. **Correctness Tests**:
   - Test with datasets with known statistics
   - Compare parallel vs sequential results
   - Test merge operations for correctness
2. **Performance Benchmarks**:
   - Sequential vs parallel speedup
   - Single-pass vs multi-pass comparison
   - Memory profiling
3. **Stress Tests**: Test with 100M+ records
4. **Numerical Stability Tests**: Test variance computation with large numbers
5. **Property Tests**: Use proptest to verify aggregate properties
6. **Parallelism Tests**: Verify all cores are utilized

---

## General Testing and Benchmarking Guide

### Tools to Use

1. **Cargo Test**: `cargo test` for unit and integration tests
2. **Criterion**: `cargo bench` for performance benchmarking
3. **Flamegraph**: Profile CPU usage and identify hotspots
4. **Valgrind/Heaptrack**: Memory profiling (Linux)
5. **Instruments**: Memory and CPU profiling (macOS)
6. **htop/Activity Monitor**: Monitor CPU core utilization

### Performance Testing Examples

```rust
// Benchmark with Criterion
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_sequential_vs_parallel(c: &mut Criterion) {
    let data: Vec<f64> = (0..10_000_000).map(|x| x as f64).collect();

    c.bench_function("sequential stats", |b| {
        b.iter(|| compute_stats(black_box(&data)))
    });

    c.bench_function("parallel stats", |b| {
        b.iter(|| parallel_stats(black_box(&data)))
    });
}

criterion_group!(benches, benchmark_sequential_vs_parallel);
criterion_main!(benches);
```

### Memory Usage Testing

```bash
# macOS
/usr/bin/time -l ./target/release/log_analyzer large_file.log

# Linux
/usr/bin/time -v ./target/release/log_analyzer large_file.log
```

### Test Data Generation

```rust
// Generate large test log file
use std::fs::File;
use std::io::{BufWriter, Write};

fn generate_test_logs(path: &str, num_lines: usize) {
    let mut writer = BufWriter::new(File::create(path).unwrap());
    let services = ["auth", "payment", "cache", "api"];
    let levels = ["INFO", "WARN", "ERROR", "DEBUG"];

    for i in 0..num_lines {
        writeln!(
            writer,
            "2024-11-26T{:02}:{:02}:{:02}.123Z {} {} Message {}",
            i / 3600 % 24,
            i / 60 % 60,
            i % 60,
            levels[i % 4],
            services[i % 4],
            i
        ).unwrap();
    }
}
```

---

These three projects progressively build expertise in iterator patterns, from streaming I/O to parallel aggregation, covering the full spectrum of techniques from Chapter 08.
