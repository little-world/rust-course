# Chapter 10: Vec & Slice Manipulation - Programming Projects

## Project 1: High-Performance CSV Batch Processor

### Problem Statement

Build a high-performance CSV processor that reads large CSV files, performs transformations, validates data, and writes results in batches to a database or output file. The processor must handle files larger than available RAM using efficient chunking, minimize allocations through capacity pre-allocation and vector reuse, and achieve maximum throughput through proper batching strategies.

Your processor should:
- Parse CSV files line-by-line without loading entire file
- Transform and validate records (type conversion, constraint checking)
- Batch records for efficient database inserts (e.g., 1000 records per batch)
- Handle errors gracefully (skip invalid rows with logging)
- Support filtering and deduplication
- Optimize memory usage through capacity management

Example workflow:
```
Input CSV: users.csv (100M rows, 5GB)
Operations: Parse → Validate → Transform → Deduplicate → Batch insert (1000/batch)
Output: PostgreSQL database or output.csv
Performance target: Process 100K rows/second
```

### Why It Matters

CSV processing is ubiquitous in data engineering. Naive approaches (loading entire file, allocating for each row, single-row inserts) are 100-1000x slower than optimized versions. Proper capacity management eliminates allocation overhead. Batching reduces database round-trips from 100K to 100. Chunking enables processing files of any size with constant memory.

These optimization patterns apply broadly: log processing, data migration, ETL pipelines, file format conversion, data cleaning.

### Use Cases

- Data migration (CSV → Database)
- ETL pipelines (extract, transform, load)
- Log file processing and aggregation
- Data cleaning and validation
- Report generation from large datasets
- A/B test data analysis

### Solution Outline

#### Step 1: Basic CSV Parser with Struct
**Goal**: Parse CSV file into structured records.

**What to implement**:
- Define `Record` struct for your data (e.g., user records)
- Parse CSV line-by-line using `csv` crate or manual parsing
- Convert string fields to appropriate types
- Handle parsing errors gracefully

**Why this step**: Foundation for data processing. Establishes data structure and basic parsing.

**Testing hint**: Test with valid and invalid CSV. Verify field parsing. Test various delimiters and quote handling.

```rust
use csv::Reader;
use std::fs::File;

#[derive(Debug, Clone)]
pub struct UserRecord {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub age: u32,
    pub country: String,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidFormat(String),
    InvalidType { field: String, value: String },
    MissingField(String),
}

impl UserRecord {
    pub fn from_csv_row(row: &csv::StringRecord) -> Result<Self, ParseError> {
        let id = row.get(0)
            .ok_or_else(|| ParseError::MissingField("id".to_string()))?
            .parse::<u64>()
            .map_err(|_| ParseError::InvalidType {
                field: "id".to_string(),
                value: row.get(0).unwrap().to_string(),
            })?;

        let name = row.get(1)
            .ok_or_else(|| ParseError::MissingField("name".to_string()))?
            .to_string();

        let email = row.get(2)
            .ok_or_else(|| ParseError::MissingField("email".to_string()))?
            .to_string();

        let age = row.get(3)
            .ok_or_else(|| ParseError::MissingField("age".to_string()))?
            .parse::<u32>()
            .map_err(|_| ParseError::InvalidType {
                field: "age".to_string(),
                value: row.get(3).unwrap().to_string(),
            })?;

        let country = row.get(4)
            .ok_or_else(|| ParseError::MissingField("country".to_string()))?
            .to_string();

        Ok(UserRecord { id, name, email, age, country })
    }
}

pub fn parse_csv(path: &str) -> Result<Vec<UserRecord>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.records() {
        let row = result?;
        match UserRecord::from_csv_row(&row) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Skipping invalid row: {:?}", e),
        }
    }

    Ok(records)
}
```

---

#### Step 2: Pre-Allocate Capacity to Eliminate Reallocations
**Goal**: Optimize memory allocations through capacity pre-allocation.

**What to implement**:
- Count lines in file (or estimate) before parsing
- Pre-allocate `Vec::with_capacity(n)` for records
- Measure allocation count before/after optimization
- Track and display allocation statistics

**Why the previous step is not enough**: Step 1 uses `Vec::new()`, which starts with capacity 0. As you push records, the vector reallocates (capacity doubling) multiple times. For 1M records, this causes ~20 reallocations, each copying all existing data.

**What's the improvement**: Pre-allocating eliminates reallocations entirely. Instead of 20 allocations with O(n log n) total copying, we get 1 allocation with zero copying. For 1M records:
- Before: ~20 allocations, ~2M items copied
- After: 1 allocation, 0 items copied
This is 10-50x faster for large datasets.

**Optimization focus**: Speed and memory efficiency through allocation elimination.

**Testing hint**: Use `std::alloc::System` with a custom allocator to track allocations. Compare allocation counts. Benchmark with large CSV files.

```rust
use std::io::{BufRead, BufReader};

pub fn count_lines(path: &str) -> Result<usize, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().count())
}

pub fn parse_csv_optimized(path: &str) -> Result<Vec<UserRecord>, Box<dyn std::error::Error>> {
    // Pre-count lines for capacity
    let line_count = count_lines(path)?;
    let estimated_records = line_count.saturating_sub(1); // Subtract header

    let file = File::open(path)?;
    let mut reader = Reader::from_reader(file);

    // Pre-allocate capacity
    let mut records = Vec::with_capacity(estimated_records);

    for result in reader.records() {
        let row = result?;
        match UserRecord::from_csv_row(&row) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Skipping invalid row: {:?}", e),
        }
    }

    println!("Capacity: {}, Length: {}", records.capacity(), records.len());
    // Verify we allocated exactly once (capacity equals initial allocation)

    Ok(records)
}
```

---

#### Step 3: Streaming Processing with Chunking
**Goal**: Process file in chunks to support files larger than RAM.

**What to implement**:
- Process CSV in chunks of N records (e.g., 10,000)
- Use `Vec::with_capacity()` for chunk buffer
- Reuse chunk buffer with `clear()` (retains capacity)
- Process each chunk (transform, validate, output)
- Never load entire file into memory

**Why the previous step is not enough**: Step 2 loads entire file into memory. This fails for files larger than RAM (10GB+ CSVs are common in production).

**What's the improvement**: Chunking processes data in fixed-size windows. Memory usage is O(chunk_size), not O(file_size). A 10GB file with 1GB RAM? No problem—process 10K records at a time.

**Optimization focus**: Memory efficiency—constant memory usage regardless of file size.

**Implementation note**: Reusing the chunk buffer (clear instead of allocating new Vec) eliminates per-chunk allocations.

**Testing hint**: Test with large file (generate 10M rows). Monitor memory usage with Activity Monitor/htop. Verify memory stays constant.

```rust
pub fn process_csv_chunked<F>(
    path: &str,
    chunk_size: usize,
    mut process_chunk: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(&[UserRecord]),
{
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(file);

    // Allocate chunk buffer once
    let mut chunk = Vec::with_capacity(chunk_size);
    let mut total_processed = 0;

    for result in reader.records() {
        let row = result?;

        match UserRecord::from_csv_row(&row) {
            Ok(record) => {
                chunk.push(record);

                // Process when chunk is full
                if chunk.len() == chunk_size {
                    process_chunk(&chunk);
                    total_processed += chunk.len();

                    // Clear but retain capacity (no reallocation)
                    chunk.clear();
                }
            }
            Err(e) => eprintln!("Skipping invalid row: {:?}", e),
        }
    }

    // Process remaining records
    if !chunk.is_empty() {
        process_chunk(&chunk);
        total_processed += chunk.len();
    }

    println!("Total processed: {}", total_processed);
    Ok(())
}

// Usage:
process_csv_chunked("large_file.csv", 10_000, |chunk| {
    println!("Processing chunk of {} records", chunk.len());
    // Transform, validate, insert to DB, etc.
})?;
```

---

#### Step 4: Batch Database Inserts with Transaction
**Goal**: Insert records to database in batches for maximum throughput.

**What to implement**:
- Batch INSERT statements (INSERT multiple rows in one query)
- Use transactions for atomicity
- Benchmark: single-row vs batched inserts
- Handle partial batch failures gracefully

**Why the previous step is not enough**: Processing chunks is great, but inserting one record at a time to database is extremely slow due to network round-trips and transaction overhead.

**What's the improvement**: Batch inserts dramatically reduce overhead:
- Single-row inserts: 100K rows = 100K queries = 100K round-trips ≈ 100 seconds
- Batched inserts (1000/batch): 100K rows = 100 queries = 100 round-trips ≈ 1 second

This is 100x speedup! Batching amortizes connection, parsing, and transaction overhead.

**Optimization focus**: Speed through batching (reducing I/O overhead).

**Testing hint**: Benchmark single-row vs batched inserts. Measure rows/second. Test rollback on constraint violation.

```rust
use rusqlite::{Connection, Transaction};

pub fn insert_batch(
    tx: &Transaction,
    records: &[UserRecord],
) -> Result<(), rusqlite::Error> {
    if records.is_empty() {
        return Ok(());
    }

    // Build multi-row INSERT
    let mut sql = String::from(
        "INSERT INTO users (id, name, email, age, country) VALUES "
    );

    let placeholders: Vec<String> = records
        .iter()
        .map(|_| "(?, ?, ?, ?, ?)")
        .collect();
    sql.push_str(&placeholders.join(", "));

    let mut stmt = tx.prepare(&sql)?;

    // Flatten parameters
    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
    for record in records {
        params.push(&record.id);
        params.push(&record.name);
        params.push(&record.email);
        params.push(&record.age);
        params.push(&record.country);
    }

    stmt.execute(params.as_slice())?;
    Ok(())
}

pub fn import_csv_to_db(
    path: &str,
    db_path: &str,
    batch_size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            age INTEGER NOT NULL,
            country TEXT NOT NULL
        )",
        [],
    )?;

    let mut batch_count = 0;

    process_csv_chunked(path, batch_size, |chunk| {
        let tx = conn.transaction().unwrap();
        insert_batch(&tx, chunk).unwrap();
        tx.commit().unwrap();
        batch_count += 1;
        if batch_count % 10 == 0 {
            println!("Processed {} batches", batch_count);
        }
    })?;

    Ok(())
}
```

---

#### Step 5: In-Place Deduplication with sort + dedup
**Goal**: Remove duplicate records efficiently using sorting and in-place deduplication.

**What to implement**:
- Implement `Eq`, `Ord` for `UserRecord` (compare by ID or email)
- Sort chunk with `sort_unstable()` (faster than stable sort)
- Deduplicate with `dedup()` (removes consecutive duplicates)
- Compare: naive HashSet approach vs sort+dedup

**Why the previous step is not enough**: Duplicate records waste storage and cause constraint violations. Naive deduplication using `HashSet` requires O(n) extra memory and is slower for large datasets.

**What's the improvement**: Sort + dedup is in-place (O(1) extra memory) and cache-friendly:
- HashSet approach: O(n) memory, random access (cache misses)
- Sort + dedup: O(1) memory, sequential access (cache hits)

For 1M records:
- HashSet: ~50MB overhead, ~100ms
- Sort + dedup: ~0MB overhead, ~50ms (with unstable sort)

**Optimization focus**: Memory efficiency and speed through in-place algorithms.

**Testing hint**: Create CSV with duplicates. Verify dedup works. Benchmark HashSet vs sort+dedup. Test with millions of records.

```rust
use std::cmp::Ordering;

impl PartialEq for UserRecord {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for UserRecord {}

impl PartialOrd for UserRecord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UserRecord {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

pub fn deduplicate_chunk(chunk: &mut Vec<UserRecord>) {
    // Sort by ID (or other key)
    chunk.sort_unstable();

    // Remove consecutive duplicates in-place
    chunk.dedup();
}

// Compare with HashSet approach:
use std::collections::HashSet;

pub fn deduplicate_hashset(chunk: &mut Vec<UserRecord>) {
    let mut seen = HashSet::new();
    chunk.retain(|record| seen.insert(record.id));
}

// Benchmark both approaches
use std::time::Instant;

pub fn benchmark_dedup(records: &mut Vec<UserRecord>) {
    let mut test1 = records.clone();
    let start = Instant::now();
    deduplicate_chunk(&mut test1);
    println!("sort+dedup: {:?}", start.elapsed());

    let mut test2 = records.clone();
    let start = Instant::now();
    deduplicate_hashset(&mut test2);
    println!("HashSet: {:?}", start.elapsed());
}
```

---

#### Step 6: Parallel Processing with Rayon
**Goal**: Process multiple chunks in parallel for maximum CPU utilization.

**What to implement**:
- Split file into chunks
- Process chunks in parallel using Rayon
- Collect results from parallel workers
- Benchmark sequential vs parallel processing

**Why the previous step is not enough**: Steps 1-5 are sequential, using only one CPU core. On an 8-core machine, we waste 87.5% of computing power.

**What's the improvement**: Parallel processing provides linear speedup with core count:
- Sequential (1 core): 100 seconds
- Parallel (8 cores): ~13 seconds (8x speedup)

For CPU-bound operations (parsing, validation, transformation), parallelism is nearly free performance.

**Optimization focus**: Speed through parallelism—utilizing all CPU cores.

**Implementation note**: I/O can be a bottleneck. Best approach: read file sequentially into chunks, then process chunks in parallel.

**Testing hint**: Benchmark with large file. Use `htop` to verify all cores are utilized. Measure speedup ratio.

```rust
use rayon::prelude::*;

pub fn process_csv_parallel(
    path: &str,
    chunk_size: usize,
) -> Result<Vec<UserRecord>, Box<dyn std::error::Error>> {
    // Read file into chunks
    let mut chunks: Vec<Vec<UserRecord>> = Vec::new();
    let mut current_chunk = Vec::with_capacity(chunk_size);

    let file = File::open(path)?;
    let mut reader = Reader::from_reader(file);

    for result in reader.records() {
        let row = result?;
        if let Ok(record) = UserRecord::from_csv_row(&row) {
            current_chunk.push(record);

            if current_chunk.len() == chunk_size {
                chunks.push(current_chunk);
                current_chunk = Vec::with_capacity(chunk_size);
            }
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    // Process chunks in parallel
    let processed: Vec<Vec<UserRecord>> = chunks
        .into_par_iter()
        .map(|mut chunk| {
            // Transform records
            for record in &mut chunk {
                // Apply transformations
                record.email = record.email.to_lowercase();
                record.country = record.country.to_uppercase();
            }

            // Deduplicate
            deduplicate_chunk(&mut chunk);

            chunk
        })
        .collect();

    // Flatten results
    let mut results = Vec::new();
    for chunk in processed {
        results.extend(chunk);
    }

    Ok(results)
}

// Benchmark parallel vs sequential
pub fn benchmark_parallel(path: &str, chunk_size: usize) {
    let start = Instant::now();
    let records = parse_csv_optimized(path).unwrap();
    println!("Sequential: {:?} ({} records)", start.elapsed(), records.len());

    let start = Instant::now();
    let records = process_csv_parallel(path, chunk_size).unwrap();
    println!("Parallel: {:?} ({} records)", start.elapsed(), records.len());
}
```

---

### Testing Strategies

1. **Unit Tests**: Test parsing, validation, deduplication independently
2. **Integration Tests**: End-to-end with test CSV files
3. **Performance Tests**: Benchmark each optimization
4. **Memory Tests**: Monitor memory usage with large files
5. **Correctness Tests**: Verify no data loss during processing
6. **Stress Tests**: Process 100M+ row files

---

## Project 2: Time-Series Data Analyzer with Sliding Windows

### Problem Statement

Build a time-series data analyzer that computes statistics over sliding windows of data. The analyzer processes sensor readings, financial data, or metrics streams and computes aggregates (moving averages, min/max, standard deviation) using efficient windowing algorithms with zero-copy slicing.

Your analyzer should:
- Support multiple window sizes (e.g., 10-second, 1-minute, 5-minute windows)
- Compute statistics: moving average, min, max, median, percentiles
- Handle streaming data (process data as it arrives)
- Use efficient algorithms: O(n) sliding window, not O(n*w)
- Provide zero-copy views into data windows
- Detect anomalies (values outside expected range)

### Why It Matters

Time-series analysis is fundamental to monitoring, finance, IoT, and analytics. Naive implementations compute aggregates by re-scanning the entire window on each update (O(n*w) time). Efficient sliding window algorithms maintain state incrementally (O(n) time), providing 100-1000x speedup for large windows.

Zero-copy slicing enables analyzing data without allocating intermediate buffers, crucial for high-throughput systems.

### Use Cases

- System monitoring (CPU, memory, network metrics)
- Financial trading (moving averages, Bollinger bands)
- IoT sensor data (temperature, pressure trends)
- Network traffic analysis (bandwidth, latency)
- Application performance monitoring (request rates, response times)
- Scientific data analysis (signal processing)

### Solution Outline

#### Step 1: Basic Sliding Window with VecDeque
**Goal**: Implement fixed-size sliding window that maintains recent N elements.

**What to implement**:
- Use `VecDeque` for efficient push/pop from both ends
- `push()` adds element, `pop_front()` if window full
- Compute basic statistics (average, min, max)

**Why this step**: Foundation for windowing. VecDeque provides O(1) push/pop.

**Testing hint**: Test window fills correctly. Test oldest elements are removed. Verify window size constraint.

```rust
use std::collections::VecDeque;

pub struct SlidingWindow<T> {
    window: VecDeque<T>,
    capacity: usize,
}

impl<T: Clone> SlidingWindow<T> {
    pub fn new(capacity: usize) -> Self {
        SlidingWindow {
            window: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, value: T) -> Option<T> {
        let removed = if self.window.len() == self.capacity {
            self.window.pop_front()
        } else {
            None
        };

        self.window.push_back(value);
        removed
    }

    pub fn as_slice(&self) -> &[T] {
        // VecDeque provides contiguous slices
        let (slice1, slice2) = self.window.as_slices();
        if slice2.is_empty() {
            slice1
        } else {
            // Handle wrap-around case (less common after initial fill)
            // For simplicity, return slice1 or use make_contiguous()
            slice1
        }
    }

    pub fn len(&self) -> usize {
        self.window.len()
    }

    pub fn is_full(&self) -> bool {
        self.window.len() == self.capacity
    }
}

// Statistics for f64 windows
impl SlidingWindow<f64> {
    pub fn average(&self) -> Option<f64> {
        if self.window.is_empty() {
            None
        } else {
            Some(self.window.iter().sum::<f64>() / self.window.len() as f64)
        }
    }

    pub fn min(&self) -> Option<f64> {
        self.window.iter().copied().min_by(|a, b| a.partial_cmp(b).unwrap())
    }

    pub fn max(&self) -> Option<f64> {
        self.window.iter().copied().max_by(|a, b| a.partial_cmp(b).unwrap())
    }
}
```

---

#### Step 2: Incremental Statistics (Avoid Re-Scanning)
**Goal**: Maintain running sum to compute average in O(1) instead of O(n).

**What to implement**:
- Track `running_sum` that updates incrementally
- When adding value: `running_sum += value`
- When removing value: `running_sum -= old_value`
- Average = running_sum / window_size

**Why the previous step is not enough**: Step 1 computes average by summing entire window on every call (O(n)). For a stream of 1M values with window size 1000, this is 1 billion operations.

**What's the improvement**: Incremental updates reduce average computation from O(n) to O(1). Instead of summing 1000 values per update, we add one and subtract one. For 1M updates:
- Before: 1M × 1000 = 1 billion operations
- After: 1M × 2 = 2 million operations (500x faster)

**Optimization focus**: Speed through algorithmic improvement (O(n) → O(1)).

**Testing hint**: Verify incremental average equals naive average. Test with large windows. Benchmark performance difference.

```rust
pub struct IncrementalWindow {
    window: VecDeque<f64>,
    capacity: usize,
    running_sum: f64,
    running_sum_sq: f64,  // For variance
}

impl IncrementalWindow {
    pub fn new(capacity: usize) -> Self {
        IncrementalWindow {
            window: VecDeque::with_capacity(capacity),
            capacity,
            running_sum: 0.0,
            running_sum_sq: 0.0,
        }
    }

    pub fn push(&mut self, value: f64) {
        // Remove oldest if full
        if self.window.len() == self.capacity {
            let old = self.window.pop_front().unwrap();
            self.running_sum -= old;
            self.running_sum_sq -= old * old;
        }

        // Add new value
        self.window.push_back(value);
        self.running_sum += value;
        self.running_sum_sq += value * value;
    }

    pub fn average(&self) -> Option<f64> {
        if self.window.is_empty() {
            None
        } else {
            Some(self.running_sum / self.window.len() as f64)
        }
    }

    pub fn variance(&self) -> Option<f64> {
        if self.window.len() < 2 {
            None
        } else {
            let n = self.window.len() as f64;
            let mean = self.running_sum / n;
            let variance = (self.running_sum_sq / n) - (mean * mean);
            Some(variance.max(0.0))  // Handle floating point errors
        }
    }

    pub fn std_dev(&self) -> Option<f64> {
        self.variance().map(|v| v.sqrt())
    }
}
```

---

#### Step 3: Min/Max with Deque (Monotonic Queue)
**Goal**: Maintain min/max in O(1) amortized time using monotonic deque.

**What to implement**:
- Maintain auxiliary deque that stores potential min/max candidates
- Keep deque monotonically increasing (for min) or decreasing (for max)
- Remove elements that fall outside window
- Min/max is always at front of deque

**Why the previous step is not enough**: Finding min/max requires scanning window (O(n)). For 1M updates with window 1000, this is 1 billion operations.

**What's the improvement**: Monotonic deque maintains min/max in O(1) amortized time. Algorithm keeps only elements that could be min/max in future:
- For min: if new element is smaller than back of deque, pop back (it can never be min)
- Front of deque is always current min

Complexity: Each element pushed once, popped at most once → O(1) amortized.

**Optimization focus**: Speed through clever data structure (O(n) → O(1) amortized).

**Testing hint**: Verify min/max are correct. Test with increasing/decreasing sequences. Benchmark vs naive approach.

```rust
use std::collections::VecDeque;

pub struct MinMaxWindow {
    window: VecDeque<(usize, f64)>,  // (index, value)
    min_deque: VecDeque<(usize, f64)>,
    max_deque: VecDeque<(usize, f64)>,
    capacity: usize,
    index: usize,
}

impl MinMaxWindow {
    pub fn new(capacity: usize) -> Self {
        MinMaxWindow {
            window: VecDeque::with_capacity(capacity),
            min_deque: VecDeque::new(),
            max_deque: VecDeque::new(),
            capacity,
            index: 0,
        }
    }

    pub fn push(&mut self, value: f64) {
        // Remove oldest if full
        if self.window.len() == self.capacity {
            let (old_idx, _) = self.window.pop_front().unwrap();

            // Remove from min_deque if it's the old value
            if let Some(&(idx, _)) = self.min_deque.front() {
                if idx == old_idx {
                    self.min_deque.pop_front();
                }
            }

            // Remove from max_deque if it's the old value
            if let Some(&(idx, _)) = self.max_deque.front() {
                if idx == old_idx {
                    self.max_deque.pop_front();
                }
            }
        }

        // Add new value
        let current = (self.index, value);
        self.window.push_back(current);

        // Maintain min_deque (monotonically increasing)
        while let Some(&(_, back_val)) = self.min_deque.back() {
            if back_val >= value {
                self.min_deque.pop_back();
            } else {
                break;
            }
        }
        self.min_deque.push_back(current);

        // Maintain max_deque (monotonically decreasing)
        while let Some(&(_, back_val)) = self.max_deque.back() {
            if back_val <= value {
                self.max_deque.pop_back();
            } else {
                break;
            }
        }
        self.max_deque.push_back(current);

        self.index += 1;
    }

    pub fn min(&self) -> Option<f64> {
        self.min_deque.front().map(|(_, val)| *val)
    }

    pub fn max(&self) -> Option<f64> {
        self.max_deque.front().map(|(_, val)| *val)
    }
}
```

---

#### Step 4: Median and Percentiles with select_nth_unstable
**Goal**: Compute median efficiently using quickselect algorithm.

**What to implement**:
- Copy window to temporary buffer
- Use `slice::select_nth_unstable()` for O(n) median
- Compute arbitrary percentiles (p50, p95, p99)
- Compare with sorting approach

**Why the previous step is not enough**: We have mean, min, max but not median or percentiles. Naive approach sorts entire window (O(n log n)).

**What's the improvement**: Quickselect finds k-th element in O(n) average time, faster than sorting:
- Sorting: O(n log n) ≈ 10,000 ops for n=1000
- Quickselect: O(n) ≈ 1,000 ops (10x faster)

For streaming percentiles, this is significant. Note: median requires copying window (can't be incremental like mean).

**Optimization focus**: Speed through better algorithm (O(n log n) → O(n)).

**Testing hint**: Verify median correctness. Test with odd/even window sizes. Benchmark quickselect vs sort.

```rust
impl IncrementalWindow {
    pub fn median(&self) -> Option<f64> {
        if self.window.is_empty() {
            return None;
        }

        // Copy to temporary buffer (required for select_nth_unstable)
        let mut temp: Vec<f64> = self.window.iter().copied().collect();
        let mid = temp.len() / 2;

        if temp.len() % 2 == 0 {
            // Even length: average of two middle elements
            temp.select_nth_unstable_by(mid - 1, |a, b| {
                a.partial_cmp(b).unwrap()
            });
            let lower = temp[mid - 1];

            temp.select_nth_unstable_by(mid, |a, b| {
                a.partial_cmp(b).unwrap()
            });
            let upper = temp[mid];

            Some((lower + upper) / 2.0)
        } else {
            // Odd length: middle element
            temp.select_nth_unstable_by(mid, |a, b| {
                a.partial_cmp(b).unwrap()
            });
            Some(temp[mid])
        }
    }

    pub fn percentile(&self, p: f64) -> Option<f64> {
        if self.window.is_empty() || p < 0.0 || p > 100.0 {
            return None;
        }

        let mut temp: Vec<f64> = self.window.iter().copied().collect();
        let index = ((p / 100.0) * (temp.len() - 1) as f64).round() as usize;

        temp.select_nth_unstable_by(index, |a, b| {
            a.partial_cmp(b).unwrap()
        });

        Some(temp[index])
    }
}
```

---

#### Step 5: Multiple Windows Simultaneously
**Goal**: Track multiple window sizes (1min, 5min, 1hour) with single pass.

**What to implement**:
- Create `MultiWindowAnalyzer` with multiple windows
- Update all windows on each data point
- Compute stats for each window
- Use `Vec` of windows, iterate once per update

**Why the previous step is not enough**: Often we need statistics at multiple time scales (short-term and long-term trends). Processing data separately for each window multiplies computational cost.

**What's the improvement**: Single-pass multi-window processing shares data ingestion cost. For 3 windows:
- Separate processing: 3 passes over data
- Combined processing: 1 pass over data (3x faster)

**Optimization focus**: Speed through single-pass processing.

**Testing hint**: Verify each window maintains correct state. Test with different window sizes. Benchmark single vs multiple passes.

```rust
pub struct MultiWindowAnalyzer {
    windows: Vec<IncrementalWindow>,
    window_sizes: Vec<usize>,
}

impl MultiWindowAnalyzer {
    pub fn new(window_sizes: Vec<usize>) -> Self {
        let windows = window_sizes
            .iter()
            .map(|&size| IncrementalWindow::new(size))
            .collect();

        MultiWindowAnalyzer {
            windows,
            window_sizes,
        }
    }

    pub fn push(&mut self, value: f64) {
        for window in &mut self.windows {
            window.push(value);
        }
    }

    pub fn get_stats(&self, window_index: usize) -> Option<WindowStats> {
        self.windows.get(window_index).map(|w| WindowStats {
            average: w.average(),
            std_dev: w.std_dev(),
            median: w.median(),
            window_size: self.window_sizes[window_index],
        })
    }

    pub fn all_stats(&self) -> Vec<WindowStats> {
        self.windows
            .iter()
            .zip(&self.window_sizes)
            .map(|(w, &size)| WindowStats {
                average: w.average(),
                std_dev: w.std_dev(),
                median: w.median(),
                window_size: size,
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct WindowStats {
    pub average: Option<f64>,
    pub std_dev: Option<f64>,
    pub median: Option<f64>,
    pub window_size: usize,
}
```

---

#### Step 6: Anomaly Detection with Z-Score
**Goal**: Detect anomalies using statistical thresholds.

**What to implement**:
- Compute z-score: `(value - mean) / std_dev`
- Flag values with |z-score| > threshold (e.g., 3.0)
- Track anomaly frequency
- Provide anomaly report with context

**Why the previous step is not enough**: Statistics alone don't identify problems. Anomaly detection enables proactive monitoring and alerting.

**What's the improvement**: Automated anomaly detection catches issues in real-time. Instead of humans watching dashboards, systems alert on unusual patterns. Z-score method is simple yet effective for many use cases.

**Testing hint**: Test with normal data (few anomalies). Test with obvious outliers. Verify z-score calculation. Test different thresholds.

```rust
pub struct AnomalyDetector {
    analyzer: MultiWindowAnalyzer,
    threshold: f64,  // Z-score threshold
    anomalies: Vec<Anomaly>,
}

#[derive(Debug)]
pub struct Anomaly {
    pub value: f64,
    pub z_score: f64,
    pub timestamp: usize,
    pub window_stats: WindowStats,
}

impl AnomalyDetector {
    pub fn new(window_sizes: Vec<usize>, threshold: f64) -> Self {
        AnomalyDetector {
            analyzer: MultiWindowAnalyzer::new(window_sizes),
            threshold,
            anomalies: Vec::new(),
        }
    }

    pub fn push(&mut self, value: f64, timestamp: usize) -> Option<Anomaly> {
        self.analyzer.push(value);

        // Check primary window (first one)
        if let Some(stats) = self.analyzer.get_stats(0) {
            if let (Some(mean), Some(std_dev)) = (stats.average, stats.std_dev) {
                if std_dev > 0.0 {
                    let z_score = (value - mean) / std_dev;

                    if z_score.abs() > self.threshold {
                        let anomaly = Anomaly {
                            value,
                            z_score,
                            timestamp,
                            window_stats: stats,
                        };
                        self.anomalies.push(anomaly.clone());
                        return Some(anomaly);
                    }
                }
            }
        }

        None
    }

    pub fn anomaly_rate(&self, total_points: usize) -> f64 {
        if total_points == 0 {
            0.0
        } else {
            self.anomalies.len() as f64 / total_points as f64
        }
    }
}

// Usage example:
fn monitor_sensor_data(readings: Vec<f64>) {
    let mut detector = AnomalyDetector::new(
        vec![100, 500, 1000],  // 100, 500, 1000 sample windows
        3.0  // 3 standard deviations
    );

    for (i, &reading) in readings.iter().enumerate() {
        if let Some(anomaly) = detector.push(reading, i) {
            println!(
                "ANOMALY at {}: value={:.2}, z-score={:.2}, mean={:.2}, std_dev={:.2}",
                anomaly.timestamp,
                anomaly.value,
                anomaly.z_score,
                anomaly.window_stats.average.unwrap(),
                anomaly.window_stats.std_dev.unwrap()
            );
        }
    }

    println!(
        "Anomaly rate: {:.2}%",
        detector.anomaly_rate(readings.len()) * 100.0
    );
}
```

---

### Testing Strategies

1. **Unit Tests**: Test each window algorithm independently
2. **Property Tests**: Verify incremental stats equal batch stats
3. **Performance Tests**: Benchmark O(1) vs O(n) algorithms
4. **Correctness Tests**: Compare with reference implementations
5. **Stress Tests**: Process millions of data points
6. **Anomaly Tests**: Test with synthetic data (normal + outliers)

---

## Project 3: Binary Search and Sorted Data Structures

### Problem Statement

Build efficient search and query systems leveraging binary search on sorted data. Implement various binary search variants (exact match, lower bound, upper bound, range queries) and create data structures that maintain sorted invariants for O(log n) operations.

Your project should include:
- Generic binary search implementation
- Database-like range queries
- Auto-complete / prefix matching with binary search
- Merge sorted sequences efficiently
- Maintain sorted invariants for incremental updates

### Why It Matters

Binary search is one of the most fundamental algorithms: O(log n) vs O(n) is the difference between 20 operations and 1,000,000 operations for n=1M. Many systems rely on sorted data: databases (B-trees), file systems, network routing tables, autocomplete systems.

Understanding binary search variants enables building efficient query systems without databases.

### Use Cases

- Database query optimization (range queries, filters)
- Auto-complete and search suggestion systems
- Log analysis (finding events in time range)
- Network routing (longest prefix match)
- Game development (spatial queries, collision detection)
- File synchronization (diff algorithms)

### Solution Outline

#### Step 1: Implement Binary Search Variants
**Goal**: Implement exact match, lower_bound, upper_bound binary searches.

**What to implement**:
- `binary_search_exact()`: Find exact match
- `binary_search_lower_bound()`: Find first element >= target
- `binary_search_upper_bound()`: Find first element > target
- Use Rust's slice methods as reference: `binary_search()`, `partition_point()`

**Why this step**: Foundation for all sorted data operations. Understanding binary search variants is essential.

**Testing hint**: Test with various arrays and targets. Test edge cases (empty, all same, target not in array).

```rust
pub fn binary_search_exact<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
    let mut left = 0;
    let mut right = arr.len();

    while left < right {
        let mid = left + (right - left) / 2;

        match arr[mid].cmp(target) {
            std::cmp::Ordering::Equal => return Some(mid),
            std::cmp::Ordering::Less => left = mid + 1,
            std::cmp::Ordering::Greater => right = mid,
        }
    }

    None
}

pub fn binary_search_lower_bound<T: Ord>(arr: &[T], target: &T) -> usize {
    let mut left = 0;
    let mut right = arr.len();

    while left < right {
        let mid = left + (right - left) / 2;

        if &arr[mid] < target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    left
}

pub fn binary_search_upper_bound<T: Ord>(arr: &[T], target: &T) -> usize {
    let mut left = 0;
    let mut right = arr.len();

    while left < right {
        let mid = left + (right - left) / 2;

        if &arr[mid] <= target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    left
}
```

---

#### Step 2: Range Queries with Binary Search
**Goal**: Implement efficient range queries: find all elements in [start, end].

**What to implement**:
- Use lower_bound(start) and upper_bound(end)
- Return slice view (zero-copy)
- Support half-open ranges [start, end), closed [start, end]
- Count elements in range without materializing slice

**Why the previous step is not enough**: Single element lookup is useful, but range queries are essential for time-series, databases, and filtering.

**What's the improvement**: Range queries using two binary searches are O(log n + k) where k is result size. Naive linear scan is O(n). For finding 100 elements in 1M element array:
- Linear scan: ~1,000,000 comparisons
- Binary search range: ~40 comparisons + 100 results

**Optimization focus**: Speed through binary search (O(n) → O(log n)).

**Testing hint**: Test various ranges. Test empty ranges. Verify zero-copy (no allocation).

```rust
pub fn range_query<T: Ord>(arr: &[T], start: &T, end: &T) -> &[T] {
    let left = binary_search_lower_bound(arr, start);
    let right = binary_search_upper_bound(arr, end);

    &arr[left..right]
}

pub fn count_in_range<T: Ord>(arr: &[T], start: &T, end: &T) -> usize {
    let left = binary_search_lower_bound(arr, start);
    let right = binary_search_upper_bound(arr, end);
    right - left
}

// Example: Time-series log queries
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct LogEntry {
    pub timestamp: u64,
    pub message: String,
}

pub fn query_logs_by_time(logs: &[LogEntry], start_time: u64, end_time: u64) -> &[LogEntry] {
    let start_entry = LogEntry {
        timestamp: start_time,
        message: String::new(),
    };
    let end_entry = LogEntry {
        timestamp: end_time,
        message: String::new(),
    };

    range_query(logs, &start_entry, &end_entry)
}
```

---

#### Step 3: Auto-Complete with Prefix Matching
**Goal**: Implement auto-complete using binary search on sorted strings.

**What to implement**:
- Find all strings with given prefix
- Use binary search to find first match
- Linear scan until prefix no longer matches
- Optimize: use upper_bound with modified string (prefix + '\u{10ffff}')

**Why the previous step is not enough**: Exact and range queries work for known values, but prefix matching is needed for search and auto-complete.

**What's the improvement**: Binary search + prefix scan is O(log n + k). Building a trie would be O(n) space and complex. For moderate-sized dictionaries (10K-1M words), sorted array + binary search is simpler and faster.

**Testing hint**: Test with dictionary of words. Verify all matches found. Test empty prefix (all words), non-existent prefix (no matches).

```rust
pub fn prefix_search<'a>(words: &'a [String], prefix: &str) -> &'a [String] {
    if prefix.is_empty() {
        return words;
    }

    // Find first word >= prefix
    let start = words.partition_point(|w| w.as_str() < prefix);

    // Find first word that doesn't start with prefix
    let end = words[start..]
        .partition_point(|w| w.starts_with(prefix)) + start;

    &words[start..end]
}

// Auto-complete example
pub struct AutoComplete {
    words: Vec<String>,
}

impl AutoComplete {
    pub fn new(mut words: Vec<String>) -> Self {
        words.sort_unstable();
        words.dedup();
        AutoComplete { words }
    }

    pub fn suggest(&self, prefix: &str) -> Vec<&str> {
        prefix_search(&self.words, prefix)
            .iter()
            .take(10)  // Limit to top 10 suggestions
            .map(|s| s.as_str())
            .collect()
    }
}

// Usage:
let autocomplete = AutoComplete::new(vec![
    "apple".to_string(),
    "application".to_string(),
    "apply".to_string(),
    "banana".to_string(),
    "band".to_string(),
]);

let suggestions = autocomplete.suggest("app");
// Returns: ["apple", "application", "apply"]
```

---

#### Step 4: Merge Sorted Sequences (K-Way Merge)
**Goal**: Efficiently merge multiple sorted sequences.

**What to implement**:
- 2-way merge using two pointers
- K-way merge using min-heap (priority queue)
- Compare performance: repeated 2-way vs K-way

**Why the previous step is not enough**: Individual sorted sequences are useful, but often we need to combine multiple sources (log files, database shards, sorted chunks).

**What's the improvement**: K-way merge with heap is O(n log k) where n is total elements, k is number of sequences. Repeated 2-way merge is O(nk). For k=100:
- Repeated 2-way: 100× slower
- K-way with heap: Optimal

**Optimization focus**: Speed through better algorithm.

**Testing hint**: Test with 2+ sequences. Verify output is sorted. Test with duplicate values. Benchmark 2-way vs K-way.

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

// 2-way merge
pub fn merge_two<T: Ord + Clone>(left: &[T], right: &[T]) -> Vec<T> {
    let mut result = Vec::with_capacity(left.len() + right.len());
    let mut i = 0;
    let mut j = 0;

    while i < left.len() && j < right.len() {
        if left[i] <= right[j] {
            result.push(left[i].clone());
            i += 1;
        } else {
            result.push(right[j].clone());
            j += 1;
        }
    }

    result.extend_from_slice(&left[i..]);
    result.extend_from_slice(&right[j..]);
    result
}

// K-way merge with heap
pub fn merge_k<T: Ord + Clone>(sequences: &[&[T]]) -> Vec<T> {
    let total_size: usize = sequences.iter().map(|s| s.len()).sum();
    let mut result = Vec::with_capacity(total_size);

    // Min-heap of (value, sequence_index, element_index)
    let mut heap = BinaryHeap::new();

    // Initialize heap with first element from each sequence
    for (seq_idx, seq) in sequences.iter().enumerate() {
        if let Some(first) = seq.first() {
            heap.push(Reverse((first.clone(), seq_idx, 0)));
        }
    }

    while let Some(Reverse((value, seq_idx, elem_idx))) = heap.pop() {
        result.push(value);

        // Add next element from same sequence
        let next_idx = elem_idx + 1;
        if next_idx < sequences[seq_idx].len() {
            heap.push(Reverse((
                sequences[seq_idx][next_idx].clone(),
                seq_idx,
                next_idx,
            )));
        }
    }

    result
}
```

---

#### Step 5: Sorted Set with Incremental Updates
**Goal**: Maintain sorted collection with efficient insert/remove/search.

**What to implement**:
- `SortedVec<T>` maintaining sorted invariant
- `insert()`: binary search + insert at correct position
- `remove()`: binary search + remove
- `contains()`: binary search
- Track operations for amortization

**Why the previous step is not enough**: Static sorted arrays are fast for queries but can't handle updates. Need dynamic sorted collection.

**What's the improvement**: Binary search for insertion point gives O(log n) search + O(n) shift. Still faster than hash table for small sets (<1000 elements) due to cache locality. For larger sets, consider BTreeSet.

**Optimization focus**: When to use sorted Vec vs BTreeSet vs HashSet.

**Testing hint**: Test insert maintains sorted order. Test remove. Benchmark vs BTreeSet and HashSet for different sizes.

```rust
pub struct SortedVec<T> {
    data: Vec<T>,
}

impl<T: Ord> SortedVec<T> {
    pub fn new() -> Self {
        SortedVec { data: Vec::new() }
    }

    pub fn insert(&mut self, value: T) {
        let pos = self.data.partition_point(|x| x < &value);
        self.data.insert(pos, value);
    }

    pub fn remove(&mut self, value: &T) -> bool {
        if let Ok(pos) = self.data.binary_search(value) {
            self.data.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        self.data.binary_search(value).is_ok()
    }

    pub fn range(&self, start: &T, end: &T) -> &[T] {
        range_query(&self.data, start, end)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

// Benchmark: SortedVec vs BTreeSet vs HashSet
use std::collections::{BTreeSet, HashSet};
use std::time::Instant;

pub fn benchmark_collections(n: usize) {
    let values: Vec<i32> = (0..n as i32).collect();

    // SortedVec
    let start = Instant::now();
    let mut sorted_vec = SortedVec::new();
    for &v in &values {
        sorted_vec.insert(v);
    }
    println!("SortedVec insert: {:?}", start.elapsed());

    // BTreeSet
    let start = Instant::now();
    let mut btree = BTreeSet::new();
    for &v in &values {
        btree.insert(v);
    }
    println!("BTreeSet insert: {:?}", start.elapsed());

    // HashSet
    let start = Instant::now();
    let mut hash = HashSet::new();
    for &v in &values {
        hash.insert(v);
    }
    println!("HashSet insert: {:?}", start.elapsed());
}
```

---

#### Step 6: Optimized Search with SIMD (Bonus)
**Goal**: Use SIMD for faster linear scans in small sorted chunks.

**What to implement**:
- Hybrid approach: binary search to narrow range, SIMD scan final chunk
- Use `std::simd` (nightly) or manual SIMD
- Compare: pure binary search vs hybrid approach

**Why the previous step is not enough**: Binary search has many branches (log n), causing branch mispredictions. For small arrays (<100 elements), linear SIMD scan can be faster.

**What's the improvement**: SIMD processes 4-16 elements simultaneously. For final 32-element chunk:
- Binary search: 5 branches (log₂32)
- SIMD scan: 2-8 operations (32÷4 to 32÷16)
No branches, better performance on modern CPUs.

**Optimization focus**: Speed through SIMD and reducing branches.

**Note**: This is advanced optimization, more valuable for learning than practical use (compiler often auto-vectorizes).

**Testing hint**: Benchmark on different array sizes. Verify correctness. Test branch prediction impact.

```rust
// Simplified example (actual SIMD requires nightly or platform-specific intrinsics)
// This demonstrates the concept

pub fn hybrid_search<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
    const THRESHOLD: usize = 32;

    if arr.len() <= THRESHOLD {
        // Use linear scan for small arrays
        arr.iter().position(|x| x == target)
    } else {
        // Binary search to narrow down
        match arr.binary_search(target) {
            Ok(pos) => Some(pos),
            Err(_) => None,
        }
    }
}

// For actual SIMD, would use:
// #[cfg(target_arch = "x86_64")]
// use std::arch::x86_64::*;
// unsafe { _mm_cmpeq_epi32(...) }
```

---

### Testing Strategies

1. **Correctness Tests**: Verify search results against linear scan
2. **Edge Case Tests**: Empty arrays, single element, duplicates
3. **Performance Tests**: Benchmark binary search vs linear scan
4. **Property Tests**: Verify sorted invariants maintained
5. **Stress Tests**: Test with millions of elements

---

These three projects comprehensively cover Vec & Slice manipulation patterns, teaching capacity management, efficient algorithms (binary search, quickselect, sliding windows), and optimization techniques that achieve orders-of-magnitude performance improvements.
