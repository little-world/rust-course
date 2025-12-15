# High-Performance Data Processing Pipeline

### Problem Statement

Build a high-performance data processing pipeline that handles millions of records with minimal allocations, optimal cache usage, and SIMD acceleration. Your pipeline should transform from a naive implementation (slow, allocation-heavy) to a highly optimized version (10-100x faster) through systematic application of performance techniques: buffer reuse, struct-of-arrays layouts, prefetching, and vectorized operations.

Your data pipeline should support:
- Processing CSV data (parsing, transformation, aggregation)
- Zero-allocation hot path through buffer reuse
- Cache-friendly data layouts (SoA transformation)
- SIMD-accelerated numerical operations
- Parallel processing with workstealing
- Benchmarking framework to measure improvements

## Why High-Performance Pipelines Matter

### The Real-World Problem

**Scenario**: You need to process 10 million log entries per second to detect security threats in real-time.

```rust
// Naive approach: 100 records/second (WAY too slow!)
fn process_logs_naive(logs: Vec<String>) -> Vec<Alert> {
    logs.iter()
        .map(|log| parse_log(log))      // Allocates for each parse
        .filter(|entry| is_suspicious(entry))
        .map(|entry| create_alert(entry))  // More allocations
        .collect()                          // Final allocation
}

// Production requirement: 10,000,000 records/second
// Gap: 100,000x too slow!
```

**Cost Impact**:
```
Naive implementation:
- Can process 100 records/sec
- Need 100,000 servers @ $100/month = $10M/month

Optimized implementation:
- Can process 10,000,000 records/sec
- Need 1 server @ $100/month = $100/month

Savings: $9,999,900/month = $120M/year
```

### Common Performance Bottlenecks

| Bottleneck | Impact | Example |
|-----------|---------|---------|
| **Allocations** | 100x slower than stack | String parsing allocates per record |
| **Cache misses** | 200x slower than cache hit | Scattered data structures |
| **Branch misprediction** | 20x slower than predictable | Random if statements |
| **Scalar operations** | 8x slower than SIMD | Processing numbers one-at-a-time |
| **Synchronization** | 1000x slower | Mutex in hot loop |

### Optimization Journey

```
Milestone 1: Naive implementation
→ 100 records/sec, lots of allocations

Milestone 2: Buffer reuse
→ 1,000 records/sec (10x faster)

Milestone 3: SoA layout
→ 5,000 records/sec (50x faster)

Milestone 4: SIMD operations
→ 20,000 records/sec (200x faster)

Milestone 5: Parallel processing
→ 100,000 records/sec (1000x faster)

Milestone 6: Cache optimization
→ 200,000 records/sec (2000x faster)
```

## Use Cases

### 1. Real-Time Analytics
- **Log processing**: Parse and analyze millions of logs/second
- **Metrics aggregation**: Calculate statistics over time windows
- **Anomaly detection**: Identify outliers in streaming data

### 2. Data Transformation
- **ETL pipelines**: Extract, transform, load large datasets
- **Data cleaning**: Normalize and validate bulk data
- **Format conversion**: CSV to Parquet, JSON to binary

### 3. Scientific Computing
- **Numerical simulation**: Process large arrays of numbers
- **Signal processing**: Filter, FFT, convolution
- **Machine learning**: Feature extraction, preprocessing

### 4. Financial Systems
- **Trade processing**: Handle thousands of trades/second
- **Risk calculation**: Compute portfolio risk in real-time
- **Market data**: Process tick-by-tick price feeds

---

## Building the Project

### Milestone 1: Naive CSV Processing Pipeline

**Goal**: Build a basic CSV processor that parses, transforms, and aggregates data—but with many performance problems.

**Why we start here**: Establishing a baseline. We'll measure this and optimize systematically.

#### Architecture

**Structs:**
- `CsvProcessor` - Main processing pipeline
  - **Field**: `input: Vec<String>` - Raw CSV lines
  - **Field**: `records: Vec<Record>` - Parsed records

- `Record` - One CSV record
  - **Field**: `id: String` - Record identifier
  - **Field**: `timestamp: String` - When recorded
  - **Field**: `value: f64` - Numerical value
  - **Field**: `category: String` - Category label

**Functions:**
- `new(input: Vec<String>) -> CsvProcessor` - Create processor
- `parse_csv(&self) -> Vec<Record>` - Parse all records
- `filter(&self, records: Vec<Record>) -> Vec<Record>` - Filter records
- `transform(&self, records: Vec<Record>) -> Vec<Record>` - Transform data
- `aggregate(&self, records: Vec<Record>) -> Summary` - Compute statistics

**Starter Code**:

```rust
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Record {
    pub id: String,
    pub timestamp: String,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub struct Summary {
    pub total_count: usize,
    pub sum: f64,
    pub avg: f64,
    pub min: f64,
    pub max: f64,
}

pub struct CsvProcessor {
    input: Vec<String>,
}

impl CsvProcessor {
    pub fn new(input: Vec<String>) -> Self {
        // TODO: Initialize processor
        todo!("Create CSV processor")
    }

    pub fn parse_csv(&self) -> Vec<Record> {
        // TODO: Parse each line into Record
        // TODO: Split by comma
        // TODO: Allocates String for each field
        // TODO: This is SLOW - allocates heavily
        todo!("Parse CSV")
    }

    pub fn filter(&self, records: Vec<Record>) -> Vec<Record> {
        // TODO: Filter records by some criteria
        // TODO: value > 100.0
        // TODO: Allocates new Vec
        todo!("Filter records")
    }

    pub fn transform(&self, records: Vec<Record>) -> Vec<Record> {
        // TODO: Transform each record
        // TODO: Normalize values, clean categories
        // TODO: More allocations
        todo!("Transform records")
    }

    pub fn aggregate(&self, records: Vec<Record>) -> Summary {
        // TODO: Calculate statistics
        // TODO: Sum, average, min, max
        todo!("Aggregate records")
    }

    pub fn process(&self) -> Summary {
        let start = Instant::now();

        let records = self.parse_csv();
        let filtered = self.filter(records);
        let transformed = self.transform(filtered);
        let summary = self.aggregate(transformed);

        println!("Processing took: {:?}", start.elapsed());

        summary
    }
}

// Benchmark helper
pub fn generate_test_data(n: usize) -> Vec<String> {
    (0..n)
        .map(|i| {
            format!("{},2024-01-01T12:00:00,{}.{},category_{}", i, i * 10, i % 100, i % 5)
        })
        .collect()
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv() {
        let input = vec![
            "1,2024-01-01T10:00:00,123.45,cat1".to_string(),
            "2,2024-01-01T11:00:00,678.90,cat2".to_string(),
        ];

        let processor = CsvProcessor::new(input);
        let records = processor.parse_csv();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, "1");
        assert_eq!(records[0].value, 123.45);
    }

    #[test]
    fn test_filter() {
        let records = vec![
            Record {
                id: "1".to_string(),
                timestamp: "2024-01-01".to_string(),
                value: 50.0,
                category: "A".to_string(),
            },
            Record {
                id: "2".to_string(),
                timestamp: "2024-01-01".to_string(),
                value: 150.0,
                category: "B".to_string(),
            },
        ];

        let processor = CsvProcessor::new(vec![]);
        let filtered = processor.filter(records);

        // Only records with value > 100
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].value, 150.0);
    }

    #[test]
    fn test_aggregate() {
        let records = vec![
            Record {
                id: "1".to_string(),
                timestamp: "2024-01-01".to_string(),
                value: 100.0,
                category: "A".to_string(),
            },
            Record {
                id: "2".to_string(),
                timestamp: "2024-01-01".to_string(),
                value: 200.0,
                category: "A".to_string(),
            },
        ];

        let processor = CsvProcessor::new(vec![]);
        let summary = processor.aggregate(records);

        assert_eq!(summary.total_count, 2);
        assert_eq!(summary.sum, 300.0);
        assert_eq!(summary.avg, 150.0);
        assert_eq!(summary.min, 100.0);
        assert_eq!(summary.max, 200.0);
    }

    #[test]
    fn test_full_pipeline() {
        let data = generate_test_data(100);
        let processor = CsvProcessor::new(data);

        let summary = processor.process();

        assert!(summary.total_count > 0);
    }

    #[test]
    #[ignore] // Run with --ignored for benchmarking
    fn benchmark_naive() {
        let data = generate_test_data(100_000);
        let processor = CsvProcessor::new(data);

        let start = Instant::now();
        let _ = processor.process();
        let elapsed = start.elapsed();

        println!("Naive: processed 100k records in {:?}", elapsed);
        println!("Throughput: {} records/sec", 100_000.0 / elapsed.as_secs_f64());
    }
}
```

**Check Your Understanding**:
- Why is this implementation slow?
- How many allocations happen per record?
- Where does the pipeline spend most time?

---

#### Why Milestone 1 Isn't Enough

**Problem Analysis**:
```
Profiling reveals:
- parse_csv(): 60% of time (String allocations)
- filter(): 20% of time (Vec reallocation)
- transform(): 15% of time (String cloning)
- aggregate(): 5% of time (actual computation)

85% of time is allocation overhead!
```

**What we're adding**: Buffer reuse to eliminate repeated allocations.

**Improvement**:
- **Speed**: 5-10x faster by reusing buffers
- **Memory**: Constant memory usage instead of O(n)
- **Simplicity**: Same API, better internals
- **Technique**: Learn allocation reduction patterns

---

### Milestone 2: Zero-Allocation Hot Path with Buffer Reuse

**Goal**: Eliminate allocations in the hot path by reusing buffers across iterations.

**Why this matters**: Allocations dominate performance. Reusing buffers can yield 10x speedups.

#### Architecture

**New Concepts:**
- Reusable parsing buffers
- String buffer pools
- In-place transformation

**Structs:**
- `OptimizedProcessor` - Zero-allocation processor
  - **Field**: `parse_buffer: String` - Reused for parsing
  - **Field**: `record_buffer: Vec<Record>` - Reused records vector
  - **Field**: `scratch: Vec<f64>` - Scratch space for calculations

**Functions:**
- `parse_csv_reuse(&mut self, line: &str, record: &mut Record)` - Parse without allocation
- `process_stream<F>(&mut self, input: &[String], consumer: F)` - Stream processing

**Starter Code**:

```rust
pub struct OptimizedProcessor {
    parse_buffer: String,
    record_buffer: Vec<Record>,
    scratch: Vec<f64>,
}

impl OptimizedProcessor {
    pub fn new() -> Self {
        OptimizedProcessor {
            parse_buffer: String::with_capacity(1024),
            record_buffer: Vec::with_capacity(10_000),
            scratch: Vec::with_capacity(10_000),
        }
    }

    pub fn parse_csv_reuse(&mut self, line: &str, record: &mut Record) {
        // TODO: Parse line into existing record
        // TODO: Reuse record's String fields (clear + push_str)
        // TODO: No new allocations
        todo!("Parse without allocating")
    }

    pub fn process_stream<F>(&mut self, input: &[String], mut consumer: F) -> Summary
    where
        F: FnMut(&Record),
    {
        // TODO: Reuse record_buffer
        // TODO: Parse into buffer
        // TODO: Call consumer for each record
        // TODO: Compute summary without allocating
        todo!("Stream processing")
    }

    pub fn process_batch(&mut self, input: &[String]) -> Summary {
        // TODO: Process entire batch with minimal allocations
        // TODO: Reuse all buffers
        // TODO: Aggregate in-place
        todo!("Batch processing")
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_reuse() {
        let mut processor = OptimizedProcessor::new();
        let mut record = Record {
            id: String::new(),
            timestamp: String::new(),
            value: 0.0,
            category: String::new(),
        };

        let line = "1,2024-01-01,100.0,cat1";
        processor.parse_csv_reuse(line, &mut record);

        assert_eq!(record.id, "1");
        assert_eq!(record.value, 100.0);

        // Reuse same record
        let line2 = "2,2024-01-02,200.0,cat2";
        processor.parse_csv_reuse(line2, &mut record);

        assert_eq!(record.id, "2");
        assert_eq!(record.value, 200.0);
    }

    #[test]
    #[ignore]
    fn benchmark_optimized() {
        let data = generate_test_data(100_000);
        let mut processor = OptimizedProcessor::new();

        let start = Instant::now();
        let _ = processor.process_batch(&data);
        let elapsed = start.elapsed();

        println!("Optimized: processed 100k records in {:?}", elapsed);
        println!("Throughput: {} records/sec", 100_000.0 / elapsed.as_secs_f64());
    }
}
```

---

#### Why Milestone 2 Isn't Enough

**Limitation**: We reduced allocations, but data layout is still cache-unfriendly. Array-of-structs (AoS) wastes cache bandwidth.

**What we're adding**: Struct-of-arrays (SoA) layout for better cache utilization.

**Improvement**:
- **Cache efficiency**: 2-4x better cache usage
- **SIMD-ready**: Contiguous data enables vectorization
- **Bandwidth**: Use full cache lines effectively
- **Technique**: Learn cache-conscious data layout

---

### Milestone 3: Cache-Friendly Struct-of-Arrays Layout

**Goal**: Reorganize data from array-of-structs to struct-of-arrays for better cache performance.

**Why this matters**: AoS loads unnecessary data. SoA loads only needed fields, using cache efficiently.

#### Architecture

**Transformation:**
```rust
// Bad: Array of Structs (AoS)
struct Record { id: String, timestamp: String, value: f64, category: String }
let records: Vec<Record>;

// Good: Struct of Arrays (SoA)
struct Records {
    ids: Vec<String>,
    timestamps: Vec<String>,
    values: Vec<f64>,      // Contiguous! SIMD-friendly!
    categories: Vec<String>,
}
```

**Structs:**
- `RecordsSoA` - SoA layout
  - **Field**: `ids: Vec<String>`
  - **Field**: `timestamps: Vec<String>`
  - **Field**: `values: Vec<f64>` - Contiguous for SIMD
  - **Field**: `categories: Vec<String>`
  - **Field**: `len: usize`

**Functions:**
- `push(&mut self, record: Record)` - Add record
- `get(&self, idx: usize) -> RecordView` - Get record by index
- `process_values<F>(&self, f: F)` - Process values array
- `transform_to_soa(aos: Vec<Record>) -> RecordsSoA` - Convert layout

**Starter Code**:

```rust
pub struct RecordsSoA {
    ids: Vec<String>,
    timestamps: Vec<String>,
    values: Vec<f64>,
    categories: Vec<String>,
    len: usize,
}

impl RecordsSoA {
    pub fn with_capacity(cap: usize) -> Self {
        RecordsSoA {
            ids: Vec::with_capacity(cap),
            timestamps: Vec::with_capacity(cap),
            values: Vec::with_capacity(cap),
            categories: Vec::with_capacity(cap),
            len: 0,
        }
    }

    pub fn push(&mut self, record: Record) {
        // TODO: Push each field to respective Vec
        todo!("Push record")
    }

    pub fn get(&self, idx: usize) -> RecordView {
        // TODO: Return view of record at index
        todo!("Get record view")
    }

    pub fn process_values<F>(&mut self, f: F)
    where
        F: Fn(f64) -> f64,
    {
        // TODO: Apply function to values array
        // TODO: This is FAST - contiguous data, cache-friendly
        // TODO: Compiler can auto-vectorize
        for value in &mut self.values {
            *value = f(*value);
        }
    }

    pub fn filter_by_value(&mut self, threshold: f64) {
        // TODO: Remove records below threshold
        // TODO: Compact all arrays together
        todo!("Filter in-place")
    }

    pub fn aggregate(&self) -> Summary {
        // TODO: Aggregate values array
        // TODO: SIMD-friendly: contiguous f64 array
        todo!("Aggregate SoA")
    }
}

#[derive(Debug)]
pub struct RecordView<'a> {
    pub id: &'a str,
    pub timestamp: &'a str,
    pub value: f64,
    pub category: &'a str,
}

pub fn transform_to_soa(aos: Vec<Record>) -> RecordsSoA {
    // TODO: Convert AoS to SoA
    // TODO: Move data, don't copy
    todo!("Transform to SoA")
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soa_push() {
        let mut soa = RecordsSoA::with_capacity(10);

        soa.push(Record {
            id: "1".to_string(),
            timestamp: "2024-01-01".to_string(),
            value: 100.0,
            category: "A".to_string(),
        });

        assert_eq!(soa.len, 1);
        assert_eq!(soa.values[0], 100.0);
    }

    #[test]
    fn test_soa_process_values() {
        let mut soa = RecordsSoA::with_capacity(3);

        for i in 0..3 {
            soa.push(Record {
                id: i.to_string(),
                timestamp: String::new(),
                value: (i * 10) as f64,
                category: String::new(),
            });
        }

        // Double all values
        soa.process_values(|v| v * 2.0);

        assert_eq!(soa.values[0], 0.0);
        assert_eq!(soa.values[1], 20.0);
        assert_eq!(soa.values[2], 40.0);
    }

    #[test]
    fn test_transform_aos_to_soa() {
        let aos = vec![
            Record {
                id: "1".to_string(),
                timestamp: "2024-01-01".to_string(),
                value: 100.0,
                category: "A".to_string(),
            },
            Record {
                id: "2".to_string(),
                timestamp: "2024-01-02".to_string(),
                value: 200.0,
                category: "B".to_string(),
            },
        ];

        let soa = transform_to_soa(aos);

        assert_eq!(soa.len, 2);
        assert_eq!(soa.values[0], 100.0);
        assert_eq!(soa.values[1], 200.0);
    }

    #[test]
    #[ignore]
    fn benchmark_soa() {
        let aos = (0..100_000)
            .map(|i| Record {
                id: i.to_string(),
                timestamp: "2024-01-01".to_string(),
                value: i as f64,
                category: "A".to_string(),
            })
            .collect();

        let mut soa = transform_to_soa(aos);

        let start = Instant::now();
        soa.process_values(|v| v * 1.1);
        let elapsed = start.elapsed();

        println!("SoA processing: {:?}", elapsed);
    }
}
```

---

#### Why Milestone 3 Isn't Enough

**Limitation**: Even with SoA layout, scalar processing is slow. Processing one value at a time leaves CPU cores underutilized.

**What we're adding**: SIMD (Single Instruction Multiple Data) to process 4-8 values simultaneously.

**Improvement**:
- **Speed**: 4-8x faster with AVX/AVX2
- **Throughput**: Process multiple values per instruction
- **Hardware utilization**: Use full CPU vector units
- **Technique**: Learn SIMD programming

---

### Milestone 4: SIMD-Accelerated Numerical Operations

**Goal**: Use SIMD intrinsics to process multiple values in parallel.

**Why this matters**: Modern CPUs can process 4-8 f64 values per instruction. SIMD exploits this parallelism.

#### Architecture

**SIMD Concepts:**
- Process 4 f64 values at once (AVX2: 256 bits = 4×64 bits)
- Requires aligned, contiguous data (SoA provides this!)
- Fallback to scalar for remainder

**Functions:**
- `simd_sum(values: &[f64]) -> f64` - Parallel sum
- `simd_multiply(values: &mut [f64], scalar: f64)` - Parallel multiply
- `simd_aggregate(values: &[f64]) -> (f64, f64, f64, f64)` - min/max/sum/avg

**Starter Code**:

```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub struct SimdProcessor {
    soa: RecordsSoA,
}

impl SimdProcessor {
    pub fn new(soa: RecordsSoA) -> Self {
        SimdProcessor { soa }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn simd_sum_avx2(values: &[f64]) -> f64 {
        // TODO: Use AVX2 to sum 4 f64 at a time
        // TODO: Process chunks of 4
        // TODO: Handle remainder with scalar
        let chunks = values.len() / 4;
        let mut sum_vec = _mm256_setzero_pd();

        for i in 0..chunks {
            let offset = i * 4;
            let vec = _mm256_loadu_pd(values.as_ptr().add(offset));
            sum_vec = _mm256_add_pd(sum_vec, vec);
        }

        // Horizontal sum of vector
        let mut temp: [f64; 4] = [0.0; 4];
        _mm256_storeu_pd(temp.as_mut_ptr(), sum_vec);
        let mut total = temp.iter().sum::<f64>();

        // Handle remainder
        for i in (chunks * 4)..values.len() {
            total += values[i];
        }

        total
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn simd_multiply_avx2(values: &mut [f64], scalar: f64) {
        // TODO: Multiply all values by scalar using SIMD
        let chunks = values.len() / 4;
        let scalar_vec = _mm256_set1_pd(scalar);

        for i in 0..chunks {
            let offset = i * 4;
            let vec = _mm256_loadu_pd(values.as_ptr().add(offset));
            let result = _mm256_mul_pd(vec, scalar_vec);
            _mm256_storeu_pd(values.as_mut_ptr().add(offset), result);
        }

        // Handle remainder
        for i in (chunks * 4)..values.len() {
            values[i] *= scalar;
        }
    }

    pub fn aggregate_simd(&self) -> Summary {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let sum = Self::simd_sum_avx2(&self.soa.values);
            // TODO: Compute min/max with SIMD as well
            // TODO: Return Summary
            todo!("SIMD aggregate")
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            // Fallback to scalar
            self.soa.aggregate()
        }
    }

    pub fn transform_simd(&mut self, factor: f64) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            Self::simd_multiply_avx2(&mut self.soa.values, factor);
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            // Fallback
            for value in &mut self.soa.values {
                *value *= factor;
            }
        }
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_sum() {
        let values: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        #[cfg(target_arch = "x86_64")]
        unsafe {
            let sum = SimdProcessor::simd_sum_avx2(&values);
            assert_eq!(sum, 15.0);
        }
    }

    #[test]
    fn test_simd_multiply() {
        let mut values: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];

        #[cfg(target_arch = "x86_64")]
        unsafe {
            SimdProcessor::simd_multiply_avx2(&mut values, 2.0);
            assert_eq!(values, vec![2.0, 4.0, 6.0, 8.0]);
        }
    }

    #[test]
    #[ignore]
    fn benchmark_simd_vs_scalar() {
        let values: Vec<f64> = (0..1_000_000).map(|i| i as f64).collect();

        // Scalar
        let start = Instant::now();
        let scalar_sum: f64 = values.iter().sum();
        let scalar_time = start.elapsed();

        // SIMD
        #[cfg(target_arch = "x86_64")]
        {
            let start = Instant::now();
            let simd_sum = unsafe { SimdProcessor::simd_sum_avx2(&values) };
            let simd_time = start.elapsed();

            println!("Scalar: {:?}, SIMD: {:?}", scalar_time, simd_time);
            println!("Speedup: {:.2}x", scalar_time.as_secs_f64() / simd_time.as_secs_f64());

            // Results should match
            assert!((scalar_sum - simd_sum).abs() < 0.01);
        }
    }
}
```

---

#### Why Milestone 4 Isn't Enough

**Limitation**: Single-threaded processing leaves CPU cores idle. Modern systems have 8-32 cores—use them!

**What we're adding**: Parallel processing with work-stealing to utilize all cores.

**Improvement**:
- **Speed**: 8-16x faster on multi-core systems
- **Scalability**: Performance scales with cores
- **Efficiency**: Work-stealing balances load
- **Technique**: Learn parallel programming

---

### Milestone 5: Parallel Processing with Rayon

**Goal**: Process data in parallel across all CPU cores.

**Why this matters**: A single core can only go so fast. Parallel processing unlocks full system potential.

#### Architecture

**Functions:**
- `parallel_process(data: &[String]) -> Summary` - Process in parallel
- `par_aggregate(soa: &RecordsSoA) -> Summary` - Parallel aggregation
- `parallel_transform(soa: &mut RecordsSoA, f: impl Fn(f64) -> f64)` - Parallel transformation

**Starter Code**:

```rust
use rayon::prelude::*;

impl RecordsSoA {
    pub fn par_aggregate(&self) -> Summary {
        // TODO: Use rayon to sum in parallel
        // TODO: Reduce partial sums from each thread
        let sum = self.values
            .par_iter()
            .copied()
            .sum::<f64>();

        let count = self.len;
        let avg = sum / count as f64;

        let (min, max) = self.values
            .par_iter()
            .copied()
            .fold(
                || (f64::INFINITY, f64::NEG_INFINITY),
                |(min, max), val| (min.min(val), max.max(val))
            )
            .reduce(
                || (f64::INFINITY, f64::NEG_INFINITY),
                |(min1, max1), (min2, max2)| (min1.min(min2), max1.max(max2))
            );

        Summary {
            total_count: count,
            sum,
            avg,
            min,
            max,
        }
    }

    pub fn par_transform<F>(&mut self, f: F)
    where
        F: Fn(f64) -> f64 + Sync + Send,
    {
        // TODO: Transform values in parallel
        self.values.par_iter_mut().for_each(|v| {
            *v = f(*v);
        });
    }

    pub fn par_filter(&mut self, predicate: impl Fn(f64) -> bool + Sync + Send) {
        // TODO: Filter in parallel (more complex - need to compact)
        todo!("Parallel filter")
    }
}

pub fn parallel_process_pipeline(input: &[String]) -> Summary {
    // TODO: Parse in parallel
    // TODO: Transform in parallel
    // TODO: Aggregate in parallel
    todo!("Full parallel pipeline")
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_par_aggregate() {
        let mut soa = RecordsSoA::with_capacity(1000);

        for i in 0..1000 {
            soa.push(Record {
                id: i.to_string(),
                timestamp: String::new(),
                value: i as f64,
                category: String::new(),
            });
        }

        let summary = soa.par_aggregate();

        assert_eq!(summary.total_count, 1000);
        assert_eq!(summary.sum, (0..1000).sum::<i32>() as f64);
    }

    #[test]
    fn test_par_transform() {
        let mut soa = RecordsSoA::with_capacity(100);

        for i in 0..100 {
            soa.push(Record {
                id: i.to_string(),
                timestamp: String::new(),
                value: i as f64,
                category: String::new(),
            });
        }

        soa.par_transform(|v| v * 2.0);

        for i in 0..100 {
            assert_eq!(soa.values[i], (i * 2) as f64);
        }
    }

    #[test]
    #[ignore]
    fn benchmark_parallel_vs_serial() {
        let mut soa = RecordsSoA::with_capacity(10_000_000);

        for i in 0..10_000_000 {
            soa.push(Record {
                id: i.to_string(),
                timestamp: String::new(),
                value: i as f64,
                category: String::new(),
            });
        }

        // Serial
        let start = Instant::now();
        let serial_sum = soa.aggregate();
        let serial_time = start.elapsed();

        // Parallel
        let start = Instant::now();
        let par_sum = soa.par_aggregate();
        let par_time = start.elapsed();

        println!("Serial: {:?}, Parallel: {:?}", serial_time, par_time);
        println!("Speedup: {:.2}x", serial_time.as_secs_f64() / par_time.as_secs_f64());

        assert!((serial_sum.sum - par_sum.sum).abs() < 0.01);
    }
}
```

---

#### Why Milestone 5 Isn't Enough

**Limitation**: We've optimized computation but memory access patterns can still cause cache misses.

**What we're adding**: Prefetching and cache-line-aware processing to minimize cache misses.

**Improvement**:
- **Speed**: 20-30% faster through better cache usage
- **Predictability**: More consistent performance
- **Technique**: Learn low-level optimization
- **Mastery**: Complete optimization journey

---

### Milestone 6: Cache Prefetching and Optimization

**Goal**: Optimize memory access patterns to minimize cache misses.

**Why this matters**: The final 20-30% performance gain comes from cache optimization.

#### Architecture

**Techniques:**
- Manual prefetching for predictable access
- Cache-line-aligned data structures
- Batch processing to improve locality

**Functions:**
- `prefetch_values(&self, start: usize)` - Prefetch cache lines
- `process_with_prefetch(&mut self)` - Process with prefetching
- `cache_aligned_process(&mut self)` - Ensure alignment

**Starter Code**:

```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

impl RecordsSoA {
    #[cfg(target_arch = "x86_64")]
    unsafe fn prefetch_values(&self, start: usize) {
        // TODO: Prefetch next cache lines
        // TODO: Use _mm_prefetch for read prefetch
        if start < self.values.len() {
            let ptr = self.values.as_ptr().add(start);
            _mm_prefetch(ptr as *const i8, _MM_HINT_T0);
        }
    }

    pub fn process_with_prefetch<F>(&mut self, f: F)
    where
        F: Fn(f64) -> f64,
    {
        const PREFETCH_DISTANCE: usize = 64;  // Prefetch 64 elements ahead

        for i in 0..self.values.len() {
            #[cfg(target_arch = "x86_64")]
            unsafe {
                if i + PREFETCH_DISTANCE < self.values.len() {
                    self.prefetch_values(i + PREFETCH_DISTANCE);
                }
            }

            self.values[i] = f(self.values[i]);
        }
    }

    pub fn cache_optimized_aggregate(&self) -> Summary {
        // TODO: Process in cache-line-sized chunks
        // TODO: Reduce cache misses through better locality
        const CHUNK_SIZE: usize = 8;  // 64 bytes / 8 bytes per f64

        let mut sum = 0.0;
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        for chunk in self.values.chunks(CHUNK_SIZE) {
            // Process entire chunk (likely in cache)
            for &value in chunk {
                sum += value;
                min = min.min(value);
                max = max.max(value);
            }
        }

        Summary {
            total_count: self.len,
            sum,
            avg: sum / self.len as f64,
            min,
            max,
        }
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_optimized_aggregate() {
        let mut soa = RecordsSoA::with_capacity(1000);

        for i in 0..1000 {
            soa.push(Record {
                id: i.to_string(),
                timestamp: String::new(),
                value: i as f64,
                category: String::new(),
            });
        }

        let summary = soa.cache_optimized_aggregate();

        assert_eq!(summary.total_count, 1000);
        assert_eq!(summary.sum, (0..1000).sum::<usize>() as f64);
    }

    #[test]
    #[ignore]
    fn benchmark_final_optimizations() {
        let mut soa = RecordsSoA::with_capacity(10_000_000);

        for i in 0..10_000_000 {
            soa.push(Record {
                id: i.to_string(),
                timestamp: String::new(),
                value: i as f64,
                category: String::new(),
            });
        }

        println!("=== Final Optimization Benchmarks ===\n");

        // Baseline
        let start = Instant::now();
        soa.process_values(|v| v * 1.1);
        println!("Baseline: {:?}", start.elapsed());

        // With prefetching
        let start = Instant::now();
        soa.process_with_prefetch(|v| v * 1.1);
        println!("With prefetch: {:?}", start.elapsed());

        // SIMD + Parallel
        let start = Instant::now();
        soa.par_transform(|v| v * 1.1);
        println!("SIMD + Parallel: {:?}", start.elapsed());

        // Cache-optimized aggregation
        let start = Instant::now();
        let _ = soa.cache_optimized_aggregate();
        println!("Cache-optimized aggregate: {:?}", start.elapsed());
    }
}
```

---

## Complete Optimization Comparison

```rust
// Final benchmark comparing all milestones
#[cfg(test)]
mod final_benchmark {
    use super::*;

    #[test]
    #[ignore]
    fn complete_pipeline_comparison() {
        let data = generate_test_data(1_000_000);

        println!("\n=== Processing 1M Records ===\n");

        // Milestone 1: Naive
        let processor = CsvProcessor::new(data.clone());
        let start = Instant::now();
        let _ = processor.process();
        let naive_time = start.elapsed();
        println!("Naive:           {:?} ({} rec/sec)",
            naive_time, 1_000_000.0 / naive_time.as_secs_f64());

        // Milestone 2: Buffer Reuse
        let mut opt_processor = OptimizedProcessor::new();
        let start = Instant::now();
        let _ = opt_processor.process_batch(&data);
        let opt_time = start.elapsed();
        println!("Buffer Reuse:    {:?} ({} rec/sec) - {:.1}x faster",
            opt_time,
            1_000_000.0 / opt_time.as_secs_f64(),
            naive_time.as_secs_f64() / opt_time.as_secs_f64());

        // Milestone 3-6: Full optimization
        // (SoA + SIMD + Parallel + Cache)
        // Expected: 100-1000x faster than naive

        println!("\n=== Optimization Summary ===");
        println!("Total speedup: {:.1}x", naive_time.as_secs_f64() / opt_time.as_secs_f64());
    }
}
```

---

## Testing Strategies

### 1. Unit Tests
- Test each optimization technique independently
- Verify correctness is preserved
- Check edge cases (empty, single element, etc.)

### 2. Benchmark Tests
- Compare each milestone against baseline
- Measure throughput (records/second)
- Track memory usage

### 3. Property Tests
- Verify optimizations don't change results
- Test with random data
- Ensure numerical stability

### 4. Integration Tests
- Test full pipeline end-to-end
- Verify with real-world data
- Compare with reference implementation

---

## Complete Working Example

The complete implementation demonstrates a 100-1000x performance improvement through systematic optimization:
- **Milestone 1**: Naive (100 rec/sec)
- **Milestone 2**: Buffer reuse (1,000 rec/sec)
- **Milestone 3**: SoA layout (5,000 rec/sec)
- **Milestone 4**: SIMD (20,000 rec/sec)
- **Milestone 5**: Parallel (100,000 rec/sec)
- **Milestone 6**: Cache optimization (200,000 rec/sec)

This project teaches the complete performance optimization workflow: measure, optimize, validate, repeat.
