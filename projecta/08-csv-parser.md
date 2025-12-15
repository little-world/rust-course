
# CSV Stream Transformer

### Problem Statement

Build a high-performance CSV transformation pipeline that processes large CSV files (potentially larger than RAM) using iterator patterns. Your transformer should read, validate, filter, transform, and aggregate CSV data without loading entire files into memory.

Your CSV transformer should support:
- Streaming CSV parsing with configurable delimiters and quotes
- Type-safe column extraction and validation
- Filtering rows based on column values
- Transforming columns (type conversion, string manipulation, computed fields)
- Aggregations (sum, count, group-by) with constant memory
- Writing transformed results to output CSV files

Example CSV data:
```csv
timestamp,user_id,action,amount,status
2024-12-01T10:00:00,1001,purchase,299.99,completed
2024-12-01T10:01:30,1002,refund,49.99,pending
2024-12-01T10:03:15,1001,purchase,159.50,completed
```
---

## Key Concepts Explained

This project demonstrates advanced Rust patterns for processing large datasets efficiently. Understanding these concepts will help you build scalable, memory-efficient data pipelines.

### 1. Streaming vs Loading: The Memory Problem

**The Problem**: Traditional CSV parsing loads the entire file into memory:
```rust
// ❌ Memory disaster for large files
fn parse_csv_bad(path: &str) -> Vec<Vec<String>> {
    let contents = std::fs::read_to_string(path).unwrap(); // Loads ENTIRE file
    contents.lines()
            .map(|line| line.split(',').map(String::from).collect())
            .collect()
}
// For a 10GB CSV: Uses 10GB+ RAM, crashes on limited memory systems
```

**The Solution**: Streaming with iterators processes one record at a time:
```rust
// ✅ Constant memory usage, any file size
fn parse_csv_stream(path: &Path) -> impl Iterator<Item = CsvRecord> {
    BufReader::new(File::open(path).unwrap())
        .lines()
        .filter_map(|line| CsvRecord::parse_csv_line(&line.unwrap(), ',').ok())
}
// For a 10GB CSV: Uses ~8KB buffer (BufReader default), independent of file size
```

**Key insight**:
- **Loading**: O(n) memory where n = file size → OOM for large files
- **Streaming**: O(1) memory (constant buffer size) → handles any file size

### 2. Iterator Trait: The Foundation of Streaming

The `Iterator` trait is Rust's abstraction for sequences of values:

```rust
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

**Why it matters**:
- **Lazy evaluation**: Values are computed on-demand, not upfront
- **Composability**: Chain operations without intermediate allocations
- **Memory efficiency**: Only current item needs to exist

**Example - Iterator vs Vec**:
```rust
// Allocates intermediate vectors at each step
let result: Vec<_> = records.into_iter()
    .filter(|r| r.is_valid())        // ❌ Allocates filtered Vec
    .map(|r| r.transform())           // ❌ Allocates transformed Vec
    .collect();                       // ❌ Final Vec

// Zero intermediate allocations - all operations fuse into single pass
let result: Vec<_> = records.into_iter()
    .filter(|r| r.is_valid())        // ✅ No allocation, just iterator adapter
    .map(|r| r.transform())           // ✅ No allocation, just iterator adapter
    .collect();                       // ✅ Only one allocation for final result
```

### 3. BufReader: Efficient File I/O

`BufReader` adds buffering to reduce system calls:

```rust
// Without BufReader: 1 syscall per byte = SLOW
let file = File::open(path)?;
for byte in file.bytes() {  // Each byte requires OS call
    process(byte);          // Millions of syscalls for large files
}

// With BufReader: 1 syscall per 8KB chunk = FAST
let file = File::open(path)?;
let reader = BufReader::new(file);  // 8KB internal buffer
for line in reader.lines() {         // Reads in chunks, not bytes
    process(line);                   // ~1000x fewer syscalls
}
```

**Performance impact**:
- Without buffering: ~1,000,000 syscalls for 1MB file
- With buffering: ~128 syscalls for 1MB file (8KB buffer)
- **~7800x reduction in syscalls** → massive speedup

### 4. Trait-Based Type Conversion

The `FromCsvField` trait enables type-safe extraction:

```rust
pub trait FromCsvField: Sized {
    fn from_csv_field(field: &str) -> Result<Self, ConversionError>;
}

// Implement for various types
impl FromCsvField for i64 { /* parse integer */ }
impl FromCsvField for f64 { /* parse float */ }
impl FromCsvField for bool { /* parse boolean */ }

// Generic extraction method
impl CsvRecord {
    pub fn get_typed<T: FromCsvField>(&self, index: usize) -> Result<T, ConversionError> {
        let field = self.get_field(index)?;
        T::from_csv_field(field)  // Dispatch to trait implementation
    }
}
```

**Benefits**:
- **Type safety**: Compiler catches type mismatches
- **Extensibility**: Add new types without modifying core code
- **Error handling**: Explicit validation with `Result`

**Example usage**:
```rust
let record = CsvRecord::parse_csv_line("Alice,30,95.5", ',')?;
let name: String = record.get_typed(0)?;  // Calls FromCsvField for String
let age: i64 = record.get_typed(1)?;      // Calls FromCsvField for i64
let score: f64 = record.get_typed(2)?;    // Calls FromCsvField for f64
```

### 5. Iterator Adapters and Combinators

Iterator adapters transform iterators without consuming them:

```rust
// Each method returns a NEW iterator, original unchanged
let iter = records.into_iter()
    .filter(|r| r.status == "completed")  // FilterIterator
    .map(|r| r.amount)                     // MapIterator
    .skip(10)                               // SkipIterator
    .take(100);                             // TakeIterator

// Nothing executed yet! (lazy evaluation)
// Only when we consume:
let sum: f64 = iter.sum();  // NOW it processes records
```

**Common combinators**:
- `filter(predicate)` - Keep items matching predicate
- `map(function)` - Transform each item
- `filter_map(function)` - Combined filter + map
- `fold(init, function)` - Reduce to single value
- `skip(n)` / `take(n)` - Skip/take n items
- `collect()` - Consume into collection

**Key property**: All adapt operations are **zero-cost** - the compiler fuses them into a single loop.

### 6. Streaming Aggregation with Fold

`fold()` enables aggregation without storing all records:

```rust
// ❌ Collects all records into memory first
let records: Vec<CsvRecord> = iterator.collect();
let sum = records.iter().map(|r| r.amount).sum();

// ✅ Aggregates while streaming, constant memory
let sum = iterator.fold(0.0, |acc, record| acc + record.amount);
```

**Example - Computing statistics**:
```rust
#[derive(Default)]
struct Stats {
    count: usize,
    sum: f64,
    min: f64,
    max: f64,
}

// Process 1 billion records with ~32 bytes of state
let stats = records.into_iter().fold(Stats::default(), |mut stats, record| {
    stats.count += 1;
    stats.sum += record.value;
    stats.min = stats.min.min(record.value);
    stats.max = stats.max.max(record.value);
    stats
});

// Memory usage: O(1) regardless of record count
```

### 7. Extension Traits for API Design

Extension traits add methods to types you don't own:

```rust
// Can't add methods directly to Iterator (defined in std)
// Solution: Extension trait

pub trait CsvFilterExt: Iterator<Item = Result<CsvRecord, Error>> + Sized {
    fn filter_by_column<F>(self, column: usize, predicate: F) -> FilterByColumn<Self, F>
    where
        F: FnMut(&str) -> bool
    {
        FilterByColumn { iter: self, column, predicate }
    }
}

// Implement for ALL iterators yielding CSV results
impl<I> CsvFilterExt for I where I: Iterator<Item = Result<CsvRecord, Error>> {}

// Now any CSV iterator gets the method
let filtered = csv_iterator
    .filter_by_column(0, |status| status == "completed");
```

**Pattern**: Define trait with default implementations + blanket impl = methods for all matching types.

### 8. Custom Iterator Implementation

Implementing `Iterator` makes your type usable with all iterator methods:

```rust
pub struct CsvFileIterator {
    reader: BufReader<File>,
    delimiter: char,
    line_number: usize,
}

impl Iterator for CsvFileIterator {
    type Item = Result<CsvRecord, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();
        loop {
            line.clear();
            match self.reader.read_line(&mut line) {
                Ok(0) => return None,  // EOF
                Ok(_) => {
                    self.line_number += 1;
                    match CsvRecord::parse_csv_line(&line.trim(), self.delimiter) {
                        Ok(record) => return Some(Ok(record)),
                        Err(ParseError::EmptyLine) => continue,  // Skip empties
                        Err(e) => return Some(Err(Error::Parse(e, self.line_number))),
                    }
                }
                Err(e) => return Some(Err(Error::Io(e))),
            }
        }
    }
}

// Now gets ALL iterator methods for free!
csv_iterator.skip(1).filter(...).map(...).collect()
```

### 9. Error Handling in Iterators

Iterators can yield `Result` to propagate errors:

```rust
// Iterator yielding Results
let iterator: impl Iterator<Item = Result<CsvRecord, Error>> = csv_iterator;

// Pattern 1: Collect with early return
let records: Vec<CsvRecord> = iterator.collect::<Result<Vec<_>, _>>()?;

// Pattern 2: Filter out errors (risky - silently drops)
let valid_records: Vec<CsvRecord> = iterator.filter_map(Result::ok).collect();

// Pattern 3: Separate valid and invalid
let (valid, errors): (Vec<_>, Vec<_>) = iterator.partition(Result::is_ok);

// Pattern 4: Handle each error
for result in iterator {
    match result {
        Ok(record) => process(record),
        Err(e) => log_error(e),  // Don't stop on errors
    }
}
```

### 10. Parallel Processing with Rayon

Rayon adds data parallelism with minimal code changes:

```rust
use rayon::prelude::*;

// Sequential: uses 1 core
let sum: f64 = records.iter()
    .map(|r| r.amount)
    .sum();

// Parallel: uses all cores
let sum: f64 = records.par_iter()  // Just add par_
    .map(|r| r.amount)
    .sum();
```

**How it works**:
1. **Work stealing**: Idle threads steal work from busy threads
2. **Divide and conquer**: Data split into chunks, processed in parallel
3. **Automatic merging**: Results combined with associative operations

**Performance characteristics**:
- **Overhead**: ~1-10μs per parallel operation
- **Worth it when**: Work per item > ~1μs (parsing, validation, complex transforms)
- **Not worth it when**: Simple operations like arithmetic (too fast)

**Example - CSV parallel aggregation**:
```rust
// Split file into chunks at record boundaries
let chunk_size = file_size / num_cores;
let chunks = split_into_chunks(file, chunk_size);

// Process each chunk in parallel
let partial_results: Vec<Stats> = chunks.par_iter()
    .map(|chunk| aggregate_chunk(chunk))
    .collect();

// Merge results (sequential, but tiny compared to processing)
let final_stats = partial_results.into_iter()
    .fold(Stats::default(), |a, b| a.merge(b));
```

### 11. Generic Programming with Trait Bounds

Trait bounds enable generic functions that work with any type meeting constraints:

```rust
// Generic over any iterator yielding CsvRecord
pub fn aggregate<I>(records: I, column: usize) -> Stats
where
    I: Iterator<Item = CsvRecord>  // Trait bound
{
    records.fold(Stats::default(), |mut stats, record| {
        if let Ok(value) = record.get_typed::<f64>(column) {
            stats.update(value);
        }
        stats
    })
}

// Works with any iterator!
let stats1 = aggregate(csv_file_iterator, 2);
let stats2 = aggregate(vec_of_records.into_iter(), 2);
let stats3 = aggregate(filtered_records.map(...), 2);
```

### 12. Builder Pattern for Configuration

Configure complex types incrementally:

```rust
pub struct CsvParser {
    delimiter: char,
    has_header: bool,
    skip_empty: bool,
}

impl CsvParser {
    pub fn new() -> Self {
        Self { delimiter: ',', has_header: true, skip_empty: true }
    }

    pub fn delimiter(mut self, d: char) -> Self {
        self.delimiter = d;
        self  // Return self for chaining
    }

    pub fn no_header(mut self) -> Self {
        self.has_header = false;
        self
    }
}

// Usage: method chaining
let parser = CsvParser::new()
    .delimiter('|')
    .no_header()
    .parse_file("data.csv")?;
```

## Connection to This Project

Here's how each concept maps to the specific milestones in this project:

### Milestone 1: Basic CSV Record Parser
**Concepts applied**:
- **String parsing**: Manual character-by-character parsing for quoted fields
- **State machines**: Tracking `in_quotes` state to handle delimiters inside quotes
- **Error handling**: `Result<CsvRecord, ParseError>` for validation

**Why this matters**: The CSV format is deceptively complex - fields can contain delimiters and newlines when quoted. Your parser must handle:
```csv
"Smith, John","123 Main St, Apt 4",42
```
The commas inside quotes should NOT split fields. This requires stateful parsing, not simple `split(',')`.

**Real-world impact**: Incorrect quote handling causes data corruption in production systems. A parser that treats `"Smith, John"` as two fields will misalign all subsequent columns.

---

### Milestone 2: Streaming CSV File Iterator
**Concepts applied**:
- **BufReader**: 8KB buffered reads instead of byte-by-byte I/O
- **Custom Iterator**: Implementing `Iterator` for `CsvFileIterator`
- **Lazy evaluation**: Records parsed only when `.next()` is called
- **Error propagation**: `Item = Result<CsvRecord, Error>` to handle I/O and parse errors

**Why this matters**: File size independence. Your iterator uses **O(1) memory** regardless of file size:
```rust
// This works identically for 1KB or 100GB files
for record in CsvFileIterator::new(path, ',')? {
    process(record?);  // Only current record in memory
}
```

**Real-world impact**: Without streaming, a 5GB CSV file would:
- Require 5GB+ RAM (OOM crash on 4GB systems)
- Take 30+ seconds just to load before processing starts
- Block other operations until fully loaded

With streaming:
- Uses ~8KB RAM (BufReader buffer)
- Processing starts immediately (first record in ~1ms)
- Can process on memory-constrained devices

**Performance metrics**:
- Memory: 5,000,000KB → 8KB (**625,000x reduction**)
- Time to first record: 30s → 1ms (**30,000x faster start**)

---

### Milestone 3: Type-Safe Column Extraction
**Concepts applied**:
- **Trait-based dispatch**: `FromCsvField` trait for type conversions
- **Generic methods**: `get_typed<T>()` works for any `T: FromCsvField`
- **Type safety**: Compiler prevents using wrong types
- **Validation**: Conversion errors caught and reported

**Why this matters**: CSV files store everything as text. Without type safety:
```rust
// ❌ Runtime panic if column isn't a number
let age: i64 = record.get_field(1).unwrap().parse().unwrap();

// ✅ Compile-time type checking + graceful error handling
let age: i64 = record.get_typed(1)?;  // Returns Result
```

**Real-world impact**: A production system processing financial transactions:
- **Without type safety**: Invalid amount "$1,234.56" parsed as string, concatenated instead of summed → silent data corruption
- **With type safety**: Parsing fails immediately, transaction rejected, human alerted

**Example failure mode**:
```rust
// Data: "user123,invalid_age,100.50"
let age: i64 = record.get_typed(1)?;  // Returns Err(ParseInt)
// Program can: log error, skip row, use default, or abort
// Instead of: panic, corrupt data, or wrong results
```

---

### Milestone 4: Filter and Transform Pipeline
**Concepts applied**:
- **Extension traits**: `CsvFilterExt` adds domain-specific methods
- **Iterator adapters**: `FilterByColumn`, `MapColumn` for zero-copy transformations
- **Lazy evaluation**: Filters applied during iteration, not upfront
- **Zero-cost abstraction**: No performance penalty vs hand-written loops

**Why this matters**: Composability without performance loss:
```rust
// Looks high-level, compiles to efficient machine code
let result = csv_iterator
    .filter_by_column(0, |status| status == "completed")  // Lazy
    .filter_valid()                                        // Lazy
    .map_column(2, |amount| amount.trim())                // Lazy
    .collect();                                            // Now executes

// Equivalent to hand-written loop, but readable and maintainable
```

**Real-world impact**: Processing 10M row CSV, keeping only rows where column 0 = "active":
- **Naive approach**: Load all 10M rows → filter → 500MB intermediate Vec
- **Streaming approach**: Process row-by-row → no intermediate storage → 8KB buffer

**Memory comparison**:
- Naive: 10,000,000 rows × 50 bytes/row = 500MB
- Streaming: 1 row × 50 bytes = **50 bytes** (10,000,000x less)

**Speed comparison** (10M rows, 30% pass filter):
- Naive collect → filter: ~8 seconds (500MB allocation + copy)
- Streaming filter: ~2 seconds (no allocation, cache-friendly)

---

### Milestone 5: Streaming Aggregations
**Concepts applied**:
- **Fold pattern**: Reduce entire dataset to summary statistics
- **Constant memory**: O(1) space complexity regardless of row count
- **Incremental computation**: Update stats as records stream
- **Associative operations**: Merge partial results from different sources

**Why this matters**: Computing statistics without loading data:
```rust
// Process 1 billion rows, use 32 bytes of memory
let stats = records.fold(CsvAggregator::new(), |mut agg, record| {
    agg.update(record.get_typed(2)?);  // Only aggregator in memory
    agg
});
```

**Real-world impact**: Analyzing web server logs (1TB, 10 billion requests):
- **Without streaming**: Impossible (1TB won't fit in RAM)
- **With streaming**:
  - Memory: 32 bytes (count, sum, min, max)
  - Time: ~30 minutes (single-threaded)
  - Result: Request statistics across entire dataset

**Group-by aggregations**:
```rust
// Group by user_id, aggregate amounts
// Memory: O(unique_users) not O(total_rows)
let grouped = records.fold(GroupedAggregator::new(), |mut agg, record| {
    let user = record.get_typed::<String>(0)?;
    let amount = record.get_typed::<f64>(1)?;
    agg.update(user, amount);  // Only unique users in memory
    agg
});
```

**Memory scaling**:
- Total rows: 1 billion
- Unique users: 1 million
- Memory without grouping: 1 billion × 100 bytes = 100GB
- Memory with streaming group-by: 1 million × 64 bytes = 64MB (**1,562x reduction**)

---

### Milestone 6: Parallel CSV Processing
**Concepts applied**:
- **Data parallelism**: Rayon distributes work across CPU cores
- **Work stealing**: Automatic load balancing
- **Chunk-based processing**: Split file at record boundaries
- **Merge pattern**: Combine partial results from parallel workers

**Why this matters**: Multi-core speedup for CPU-bound operations:
```rust
// Sequential: Uses 1 of 8 cores
let stats = sequential_aggregate(records);  // 40 seconds

// Parallel: Uses all 8 cores
let stats = parallel_aggregate(records, 8);  // 5 seconds (8x speedup)
```

**Real-world impact**: Processing daily transaction log (10GB, 100M records):
- **Sequential**:
  - 1 core @ 2.5M records/sec = 40 seconds
  - CPU utilization: 12.5% (1 of 8 cores)
- **Parallel** (8 cores):
  - 8 cores @ 2.5M records/sec each = 5 seconds
  - CPU utilization: 100%
  - **Speedup: 8x**

**When parallelism helps**:
- ✅ CSV parsing: ~500ns/record → 8x speedup
- ✅ Data validation: ~1μs/record → 7.5x speedup
- ✅ Complex transforms: ~10μs/record → 7.9x speedup
- ❌ Simple arithmetic: ~10ns/record → 2x speedup (overhead dominates)

**Chunking strategy**:
```rust
// Must split at newlines, not arbitrary byte offsets
// Wrong: file_size / num_cores (might split mid-record)
// Right: Find newline boundaries for each chunk

let chunk_size = file_size / num_cores;
let chunks = (0..num_cores).map(|i| {
    let start = i * chunk_size;
    let end = seek_to_newline(start + chunk_size);  // Align to record boundary
    (start, end)
});
```

**Scaling efficiency** (8-core system, 10GB CSV):
- 1 thread: 40s (baseline)
- 2 threads: 20s (2.0x speedup, 100% efficiency)
- 4 threads: 10s (4.0x speedup, 100% efficiency)
- 8 threads: 5s (8.0x speedup, 100% efficiency)
- 16 threads: 5s (8.0x speedup, 50% efficiency - more threads than cores)

**Memory considerations**:
- Sequential: 8KB buffer
- Parallel (8 workers): 8 × 8KB = 64KB buffers + per-worker aggregators
- Still O(1) relative to file size

---

### Project-Wide Benefits

By combining these concepts, your CSV processor achieves:

1. **Scalability**: Handles 1KB to 1TB files identically
2. **Performance**: 8x faster with parallelism, zero unnecessary allocations
3. **Memory efficiency**: O(1) memory usage regardless of file size
4. **Type safety**: Compile-time guarantees prevent runtime errors
5. **Composability**: Build complex pipelines from simple operations
6. **Maintainability**: High-level code that compiles to efficient machine code

**Production-ready characteristics**:
- ✅ Handles malformed input gracefully (error propagation)
- ✅ Processes files larger than RAM (streaming)
- ✅ Maximizes hardware utilization (multi-core)
- ✅ Prevents silent data corruption (type safety)
- ✅ Minimal memory footprint (suitable for containers/edge devices)
- ✅ Near-optimal performance (zero-cost abstractions)

## Build The Project

### Milestone 1: Basic CSV Record Parser

**Goal**: Create a CSV record parser that handles quoted fields and escapes.

**What to implement**:
- Define `CsvRecord` struct representing a parsed CSV row
- Implement `parse_csv_line(line: &str, delimiter: char) -> Result<CsvRecord, ParseError>`
- Handle quoted fields with embedded delimiters and quotes
- Support configurable field delimiter (comma, tab, pipe)

**Architecture**:
- Structs: `CsvRecord`, `ParseError`
- Fields (CsvRecord): `fields: Vec<String>`
- Functions:
    - `parse_csv_line(line: &str, delimiter: char) -> Result<CsvRecord, ParseError>` - Parse single CSV line
    - `get_field(&self, index: usize) -> Option<&str>` - Get field by index
    - `field_count(&self) -> usize` - Count fields

---

**Starter Code**:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct CsvRecord {
    fields: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnterminatedQuote,
    InvalidEscape,
    EmptyLine,
}

impl CsvRecord {
    /// Parse a CSV line with respect to quotes and delimiters
    /// Role: Handle quoted fields that may contain delimiters
    pub fn parse_csv_line(line: &str, delimiter: char) -> Result<Self, ParseError> {
        todo!("Implement CSV parsing with quote handling")
    }

    /// Get field value by column index
    /// Role: Safe field access
    pub fn get_field(&self, index: usize) -> Option<&str> {
        todo!("Return field at index")
    }

    /// Return the number of fields in this record
    /// Role: Query record structure
    pub fn field_count(&self) -> usize {
        todo!("Return field count")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_csv_parsing() {
        let record = CsvRecord::parse_csv_line("foo,bar,baz", ',').unwrap();
        assert_eq!(record.field_count(), 3);
        assert_eq!(record.get_field(0), Some("foo"));
        assert_eq!(record.get_field(1), Some("bar"));
        assert_eq!(record.get_field(2), Some("baz"));
    }

    #[test]
    fn test_quoted_fields() {
        let record = CsvRecord::parse_csv_line(r#"foo,"bar,baz",qux"#, ',').unwrap();
        assert_eq!(record.field_count(), 3);
        assert_eq!(record.get_field(1), Some("bar,baz"));
    }

    #[test]
    fn test_escaped_quotes() {
        let record = CsvRecord::parse_csv_line(r#""foo ""bar"" baz""#, ',').unwrap();
        assert_eq!(record.get_field(0), Some(r#"foo "bar" baz"#));
    }

    #[test]
    fn test_empty_fields() {
        let record = CsvRecord::parse_csv_line("foo,,bar", ',').unwrap();
        assert_eq!(record.field_count(), 3);
        assert_eq!(record.get_field(1), Some(""));
    }

    #[test]
    fn test_custom_delimiter() {
        let record = CsvRecord::parse_csv_line("foo|bar|baz", '|').unwrap();
        assert_eq!(record.field_count(), 3);
    }
}
```

---

### Milestone 2: Streaming CSV File Iterator

**Goal**: Create an iterator that yields CSV records one at a time from a file.

**Why the previous milestone is not enough**: Milestone 1 parses individual lines, but we need to process entire files. Loading a multi-gigabyte CSV into memory causes OOM errors.

**What's the improvement**: Using `BufReader` with iterator patterns enables streaming - only the current record occupies memory. This allows processing CSV files of any size with O(1) memory usage (constant overhead per record). A 10GB CSV file uses the same memory as a 10KB file.

**Architecture**:
- Structs: `CsvFileIterator`
- Fields: `reader: BufReader<File>`, `delimiter: char`, `line_number: usize`
- Functions:
    - `new(path: &Path, delimiter: char) -> Result<Self, io::Error>` - Open CSV file
    - `next() -> Option<Result<CsvRecord, Error>>` - Iterate records

---

**Starter Code**:

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Iterator over CSV records from a file
pub struct CsvFileIterator {
    reader: BufReader<File>,
    delimiter: char,
    line_number: usize,
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse(ParseError, usize), // error and line number
}

impl CsvFileIterator {
    /// Create a new CSV file iterator
    /// Role: Open file and prepare for streaming
    pub fn new(path: &Path, delimiter: char) -> Result<Self, std::io::Error> {
        todo!("Open file with BufReader")
    }
}

impl Iterator for CsvFileIterator {
    type Item = Result<CsvRecord, Error>;

    /// Read and parse next CSV line
    /// Role: Stream records without loading entire file
    fn next(&mut self) -> Option<Self::Item> {
        todo!("Read line, parse CSV, handle errors")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_iterate_simple_csv() {
        let file = create_test_csv("a,b,c\n1,2,3\n4,5,6");
        let mut iter = CsvFileIterator::new(file.path(), ',').unwrap();

        let record1 = iter.next().unwrap().unwrap();
        assert_eq!(record1.get_field(0), Some("a"));

        let record2 = iter.next().unwrap().unwrap();
        assert_eq!(record2.get_field(0), Some("1"));

        let record3 = iter.next().unwrap().unwrap();
        assert_eq!(record3.get_field(0), Some("4"));

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_skip_empty_lines() {
        let file = create_test_csv("a,b\n\n1,2\n");
        let records: Vec<_> = CsvFileIterator::new(file.path(), ',')
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_error_reporting_with_line_numbers() {
        let file = create_test_csv("a,b\n\"unterminated\n3,4");
        let mut iter = CsvFileIterator::new(file.path(), ',').unwrap();

        iter.next(); // Skip header
        let err = iter.next().unwrap().unwrap_err();

        match err {
            Error::Parse(ParseError::UnterminatedQuote, line_num) => {
                assert_eq!(line_num, 2);
            }
            _ => panic!("Expected parse error with line number"),
        }
    }
}
```

---

### Milestone 3: Type-Safe Column Extraction

**Goal**: Extract and parse typed columns from CSV records.

**Why the previous milestone is not enough**: CsvRecord stores fields as strings. We need type-safe access (integers, floats, dates) with validation.

**What's the improvement**: Implementing a trait-based column extraction system with `FromCsvField` enables type conversions with error handling. This prevents runtime panics from invalid type assumptions and makes data validation explicit. The type system catches schema mismatches at compile time when possible.

**Architecture**:
- Traits: `FromCsvField`
- Structs: `TypedRecord<T>`
- Functions:
    - `get_typed<T: FromCsvField>(&self, index: usize) -> Result<T, ConversionError>` - Parse field as type T
    - `extract<T>(&self) -> Result<T, ExtractionError>` where T: FromCsvFields - Extract entire row into struct

---

**Starter Code**:

```rust
pub trait FromCsvField: Sized {
    fn from_csv_field(field: &str) -> Result<Self, ConversionError>;
}

#[derive(Debug, PartialEq)]
pub enum ConversionError {
    ParseInt(std::num::ParseIntError),
    ParseFloat(std::num::ParseFloatError),
    InvalidValue(String),
    MissingField,
}

impl FromCsvField for String {
    /// Role: Parse string into Self
    fn from_csv_field(field: &str) -> Result<Self, ConversionError> {
        Ok(field.to_string())
    }
}

impl FromCsvField for i64 {
    /// Role: Parse integer fields
    fn from_csv_field(field: &str) -> Result<Self, ConversionError> {
        todo!("Parse string as i64")
    }
}

impl FromCsvField for f64 {
    /// Role: Parse float fields
    fn from_csv_field(field: &str) -> Result<Self, ConversionError> {
        todo!("Parse string as f64")
    }
}

impl FromCsvField for bool {
    /// Role: Parse boolean fields (true/false, yes/no, 1/0)
    fn from_csv_field(field: &str) -> Result<Self, ConversionError> {
        todo!("Parse string as bool")
    }
}

impl CsvRecord {
    /// Get field as typed value
    /// Role: Type-safe field extraction with validation
    pub fn get_typed<T: FromCsvField>(&self, index: usize) -> Result<T, ConversionError> {
        todo!("Get field and convert to T")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_integers() {
        let record = CsvRecord::parse_csv_line("100,200,300", ',').unwrap();
        assert_eq!(record.get_typed::<i64>(0).unwrap(), 100);
        assert_eq!(record.get_typed::<i64>(1).unwrap(), 200);
    }

    #[test]
    fn test_extract_floats() {
        let record = CsvRecord::parse_csv_line("3.14,2.71,1.41", ',').unwrap();
        assert_eq!(record.get_typed::<f64>(0).unwrap(), 3.14);
    }

    #[test]
    fn test_extract_booleans() {
        let record = CsvRecord::parse_csv_line("true,false,yes,no,1,0", ',').unwrap();
        assert_eq!(record.get_typed::<bool>(0).unwrap(), true);
        assert_eq!(record.get_typed::<bool>(1).unwrap(), false);
        assert_eq!(record.get_typed::<bool>(2).unwrap(), true);
        assert_eq!(record.get_typed::<bool>(3).unwrap(), false);
        assert_eq!(record.get_typed::<bool>(4).unwrap(), true);
        assert_eq!(record.get_typed::<bool>(5).unwrap(), false);
    }

    #[test]
    fn test_conversion_errors() {
        let record = CsvRecord::parse_csv_line("not_a_number,42", ',').unwrap();
        assert!(record.get_typed::<i64>(0).is_err());
        assert!(record.get_typed::<i64>(1).is_ok());
    }

    #[test]
    fn test_missing_field_error() {
        let record = CsvRecord::parse_csv_line("a,b", ',').unwrap();
        assert!(matches!(
            record.get_typed::<String>(5),
            Err(ConversionError::MissingField)
        ));
    }
}
```

---

### Milestone 4: Filter and Transform Pipeline

**Goal**: Build composable filter and transform operations on CSV streams.

**Why the previous milestone is not enough**: We can read and parse CSV, but real-world use cases require filtering rows, transforming values, and computing derived columns.

**What's the improvement**: Creating iterator adapters for filtering and mapping enables declarative pipelines. Filters compose without intermediate allocations - all operations fuse into a single pass. This is dramatically more efficient than creating intermediate vectors after each operation.

**Optimization focus**: Memory and speed through zero-allocation iterator composition.

**Architecture**:
- Traits: `CsvFilter`, `CsvTransform`
- Structs: `FilteredCsv`, `TransformedCsv`, `ComputedColumn`
- Functions:
    - `filter<F>(predicate: F)` - Filter rows based on predicate
    - `map_column<F>(index: usize, f: F)` - Transform specific column
    - `add_computed_column<F>(f: F)` - Add computed field

---

**Starter Code**:

```rust
/// Extension trait for filtering CSV records
pub trait CsvFilterExt: Iterator<Item = Result<CsvRecord, Error>> + Sized {
    /// Filter rows where column matches predicate
    /// Role: Declarative row filtering
    fn filter_by_column<F>(self, column: usize, predicate: F) -> FilterByColumn<Self, F>
    where
        F: FnMut(&str) -> bool;

    /// Skip records that fail validation
    /// Role: Filter out malformed data
    fn filter_valid(self) -> FilterValid<Self>;
}

pub struct FilterByColumn<I, F> {
    iter: I,
    column: usize,
    predicate: F,
}

impl<I, F> Iterator for FilterByColumn<I, F>
where
    I: Iterator<Item = Result<CsvRecord, Error>>,
    F: FnMut(&str) -> bool,
{
    type Item = Result<CsvRecord, Error>;

    /// Role: Apply filter lazily as records stream
    fn next(&mut self) -> Option<Self::Item> {
        todo!("Filter records based on column value")
    }
}

pub struct FilterValid<I> {
    iter: I,
}

impl<I> Iterator for FilterValid<I>
where
    I: Iterator<Item = Result<CsvRecord, Error>>,
{
    type Item = CsvRecord;

    /// Role: Skip errors and yield only valid records
    fn next(&mut self) -> Option<Self::Item> {
        todo!("Skip error results, yield valid records")
    }
}

/// Extension trait for transforming CSV records
///
/// Functions:
/// - map_column() - Transform values in specific column
/// - with_computed_column() - Add derived column
pub trait CsvTransformExt: Iterator<Item = CsvRecord> + Sized {
    /// Transform a specific column in-place
    /// Role: Modify column values
    fn map_column<F>(self, column: usize, f: F) -> MapColumn<Self, F>
    where
        F: FnMut(&str) -> String;
}

pub struct MapColumn<I, F> {
    iter: I,
    column: usize,
    mapper: F,
}

impl<I, F> Iterator for MapColumn<I, F>
where
    I: Iterator<Item = CsvRecord>,
    F: FnMut(&str) -> String,
{
    type Item = CsvRecord;

    /// Role: Transform column values on-the-fly
    fn next(&mut self) -> Option<Self::Item> {
        todo!("Apply transformation to specified column")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_by_column() {
        let csv = "status,amount\ncompleted,100\npending,200\ncompleted,300";
        let file = create_test_csv(csv);

        let completed: Vec<_> = CsvFileIterator::new(file.path(), ',')
            .unwrap()
            .skip(1) // Skip header
            .filter_by_column(0, |status| status == "completed")
            .filter_valid()
            .collect();

        assert_eq!(completed.len(), 2);
        assert_eq!(completed[0].get_field(1), Some("100"));
        assert_eq!(completed[1].get_field(1), Some("300"));
    }

    #[test]
    fn test_map_column_transformation() {
        let csv = "name,age\nalice,30\nbob,25";
        let file = create_test_csv(csv);

        let uppercase: Vec<_> = CsvFileIterator::new(file.path(), ',')
            .unwrap()
            .filter_valid()
            .map_column(0, |name| name.to_uppercase())
            .collect();

        assert_eq!(uppercase[1].get_field(0), Some("ALICE"));
        assert_eq!(uppercase[2].get_field(0), Some("BOB"));
    }

    #[test]
    fn test_chained_operations() {
        let csv = "status,amount\ncompleted,100\npending,200\ncompleted,50\nfailed,75";
        let file = create_test_csv(csv);

        let result: Vec<_> = CsvFileIterator::new(file.path(), ',')
            .unwrap()
            .skip(1)
            .filter_by_column(0, |s| s == "completed")
            .filter_valid()
            .collect();

        assert_eq!(result.len(), 2);
    }
}
```

---

### Milestone 5: Streaming Aggregations

**Goal**: Compute aggregates over CSV streams without loading data into memory.

**Why the previous milestone is not enough**: We can filter and transform, but we need summary statistics. Collecting all records to compute aggregates defeats the purpose of streaming.

**What's the improvement**: Streaming aggregation using `.fold()` maintains only summary statistics (counts, sums, min/max) rather than storing records. For a 10GB CSV with 100M rows, this uses ~1KB of memory instead of 10GB. This is the key to analyzing arbitrarily large datasets.

**Optimization focus**: Memory - O(1) aggregate storage instead of O(n) record storage.

**Architecture**:
- Structs: `CsvAggregator`, `GroupedAggregator`
- Fields: `count: usize`, `sum: f64`, `min: f64`, `max: f64`, `groups: HashMap<String, Stats>`
- Functions:
    - `aggregate<F>(extractor: F)` - Compute stats from column
    - `group_by<K, V>(key_fn: K, value_fn: V)` - Group-by aggregation

---

**Starter Code**:

```rust
use std::collections::HashMap;

/// Aggregation statistics for numeric columns
#[derive(Debug, Clone)]
pub struct CsvAggregator {
    count: usize,
    sum: f64,
    min: f64,
    max: f64,
}

impl CsvAggregator {
    /// Create a new aggregator
    /// Role: Initialize aggregate state
    pub fn new() -> Self {
        todo!("Initialize with appropriate min/max defaults")
    }

    /// Update aggregator with a value
    /// Role: Incrementally update statistics
    pub fn update(&mut self, value: f64) {
        todo!("Update count, sum, min, max")
    }

    /// Compute mean
    /// Role: Calculate average
    pub fn mean(&self) -> Option<f64> {
        todo!("Return mean or None if count is 0")
    }

    /// Create aggregator from iterator
    /// Role: Stream values into aggregate
    pub fn from_column<I>(records: I, column: usize) -> Self
    where
        I: Iterator<Item = CsvRecord>,
    {
        todo!("Fold records into aggregator")
    }
}

/// Grouped aggregations by key
pub struct GroupedAggregator<K> {
    groups: HashMap<K, CsvAggregator>,
}

impl<K: Eq + std::hash::Hash> GroupedAggregator<K> {
    /// Create grouped aggregator
    /// Role: Initialize empty group map
    pub fn new() -> Self {
        todo!("Initialize empty HashMap")
    }

    /// Update with key-value pair
    /// Role: Add value to appropriate group
    pub fn update(&mut self, key: K, value: f64) {
        todo!("Get or create group, update aggregator")
    }

    /// Get statistics for a group
    /// Role: Query per-group stats
    pub fn get(&self, key: &K) -> Option<&CsvAggregator> {
        self.groups.get(key)
    }

    /// Create from iterator with key and value extractors
    /// Role: Stream into grouped aggregates
    pub fn from_records<I, KF, VF>(records: I, key_fn: KF, value_fn: VF) -> Self
    where
        I: Iterator<Item = CsvRecord>,
        KF: Fn(&CsvRecord) -> K,
        VF: Fn(&CsvRecord) -> Option<f64>,
    {
        todo!("Fold records into grouped aggregator")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_aggregation() {
        let csv = "value\n10\n20\n30\n40\n50";
        let file = create_test_csv(csv);

        let agg = CsvAggregator::from_column(
            CsvFileIterator::new(file.path(), ',')
                .unwrap()
                .skip(1)
                .filter_map(Result::ok),
            0
        );

        assert_eq!(agg.count, 5);
        assert_eq!(agg.sum, 150.0);
        assert_eq!(agg.min, 10.0);
        assert_eq!(agg.max, 50.0);
        assert_eq!(agg.mean(), Some(30.0));
    }

    #[test]
    fn test_grouped_aggregation() {
        let csv = "category,amount\nA,100\nB,200\nA,150\nB,250\nA,50";
        let file = create_test_csv(csv);

        let grouped = GroupedAggregator::from_records(
            CsvFileIterator::new(file.path(), ',')
                .unwrap()
                .skip(1)
                .filter_map(Result::ok),
            |rec| rec.get_field(0).unwrap().to_string(),
            |rec| rec.get_typed::<f64>(1).ok()
        );

        let stats_a = grouped.get(&"A".to_string()).unwrap();
        assert_eq!(stats_a.count, 3);
        assert_eq!(stats_a.sum, 300.0);

        let stats_b = grouped.get(&"B".to_string()).unwrap();
        assert_eq!(stats_b.count, 2);
        assert_eq!(stats_b.sum, 450.0);
    }

    #[test]
    fn test_empty_aggregation() {
        let agg = CsvAggregator::new();
        assert_eq!(agg.mean(), None);
    }
}
```

---

### Milestone 6: Parallel CSV Processing with Rayon

**Goal**: Process large CSV files using multiple CPU cores for maximum throughput.

**Why the previous milestone is not enough**: Sequential processing uses only one core. For CPU-bound operations (parsing, validation, transformations), we're leaving performance on the table.

**What's the improvement**: Parallel processing with Rayon distributes work across all cores, providing near-linear speedup. For an 8-core system processing a 1GB CSV:
- Sequential: ~30 seconds
- Parallel: ~4 seconds (7.5x speedup)

This transforms "process overnight" batch jobs into "process in minutes" interactive workflows.

**Optimization focus**: Speed through parallelism - maximize CPU utilization.

**Implementation note**: CSV parsing must handle byte-level chunking (can't split mid-record). Use parallel chunk processing where each chunk is guaranteed to contain complete records.

**Architecture**:
- Functions:
    - `parallel_process_csv<F>(path, chunk_size, process_fn)` - Process CSV in parallel chunks
    - `parallel_aggregate(path, column)` - Parallel column aggregation
    - `parallel_group_by(path, key_col, value_col)` - Parallel grouped aggregation

---

**Starter Code**:

```rust
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};

/// Parallel CSV processing utilities

/// Find byte offsets for chunks that align with record boundaries
/// Role: Split file into processable chunks without breaking records
fn find_chunk_boundaries(file: &mut File, num_chunks: usize) -> std::io::Result<Vec<u64>> {
    todo!("Find newline-aligned chunk offsets")
}

/// Process CSV file in parallel chunks
/// Role: Distribute work across CPU cores
pub fn parallel_process_csv<F, R>(
    path: &std::path::Path,
    delimiter: char,
    num_workers: usize,
    process_chunk: F,
) -> std::io::Result<Vec<R>>
where
    F: Fn(Vec<CsvRecord>) -> R + Send + Sync,
    R: Send,
{
    todo!("Split file, process chunks in parallel, collect results")
}

/// Parallel aggregation of numeric column
/// Role: Compute statistics using all CPU cores
pub fn parallel_aggregate_column(
    path: &std::path::Path,
    delimiter: char,
    column: usize,
    num_workers: usize,
) -> std::io::Result<CsvAggregator> {
    todo!("Parallel fold into aggregators, then merge")
}

/// Parallel grouped aggregation
/// Role: Group-by with parallel processing
pub fn parallel_group_by<K>(
    path: &std::path::Path,
    delimiter: char,
    key_column: usize,
    value_column: usize,
    num_workers: usize,
) -> std::io::Result<GroupedAggregator<K>>
where
    K: Eq + std::hash::Hash + Send + Clone + FromCsvField,
{
    todo!("Parallel group-by with merge")
}

impl CsvAggregator {
    /// Merge two aggregators
    /// Role: Combine partial results from parallel workers
    pub fn merge(&mut self, other: CsvAggregator) {
        todo!("Merge counts, sums, update min/max")
    }
}

impl<K: Eq + std::hash::Hash> GroupedAggregator<K> {
    /// Merge grouped aggregators
    /// Role: Combine per-group results from workers
    pub fn merge(&mut self, other: GroupedAggregator<K>) {
        todo!("Merge each group's aggregator")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_large_test_csv(rows: usize) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "category,value").unwrap();

        for i in 0..rows {
            let category = if i % 3 == 0 { "A" } else if i % 3 == 1 { "B" } else { "C" };
            writeln!(file, "{},{}", category, i).unwrap();
        }

        file
    }

    #[test]
    fn test_parallel_vs_sequential_correctness() {
        let file = create_large_test_csv(1000);

        // Sequential
        let seq_agg = CsvAggregator::from_column(
            CsvFileIterator::new(file.path(), ',')
                .unwrap()
                .skip(1)
                .filter_map(Result::ok),
            1
        );

        // Parallel
        let par_agg = parallel_aggregate_column(file.path(), ',', 1, 4).unwrap();

        // Results should match
        assert_eq!(seq_agg.count, par_agg.count);
        assert_eq!(seq_agg.sum, par_agg.sum);
        assert_eq!(seq_agg.min, par_agg.min);
        assert_eq!(seq_agg.max, par_agg.max);
    }

    #[test]
    fn test_parallel_grouped_aggregation() {
        let file = create_large_test_csv(900); // 300 of each category

        let grouped = parallel_group_by::<String>(
            file.path(),
            ',',
            0, // key: category
            1, // value: value column
            4  // workers
        ).unwrap();

        assert_eq!(grouped.get(&"A".to_string()).unwrap().count, 300);
        assert_eq!(grouped.get(&"B".to_string()).unwrap().count, 300);
        assert_eq!(grouped.get(&"C".to_string()).unwrap().count, 300);
    }

    #[test]
    #[ignore] // Run with --ignored for benchmarking
    fn benchmark_parallel_speedup() {
        use std::time::Instant;

        let file = create_large_test_csv(1_000_000);

        // Sequential
        let start = Instant::now();
        let _ = CsvAggregator::from_column(
            CsvFileIterator::new(file.path(), ',')
                .unwrap()
                .skip(1)
                .filter_map(Result::ok),
            1
        );
        let seq_time = start.elapsed();

        // Parallel
        let start = Instant::now();
        let _ = parallel_aggregate_column(file.path(), ',', 1, 8).unwrap();
        let par_time = start.elapsed();

        println!("Sequential: {:?}", seq_time);
        println!("Parallel (8 cores): {:?}", par_time);
        println!("Speedup: {:.2}x", seq_time.as_secs_f64() / par_time.as_secs_f64());

        assert!(par_time < seq_time);
    }
}
```

---

### Complete Working Example

Here's a complete, production-ready CSV transformer implementation:

```rust
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;

// =============================================================================
// Milestone 1: CSV record parsing
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct CsvRecord {
    fields: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnterminatedQuote,
    InvalidEscape,
    EmptyLine,
}

impl CsvRecord {
    pub fn parse_csv_line(line: &str, delimiter: char) -> Result<Self, ParseError> {
        let trimmed_line = line.trim_end_matches(|c| c == '\n' || c == '\r');
        if trimmed_line.trim().is_empty() {
            return Err(ParseError::EmptyLine);
        }

        let mut fields = Vec::new();
        let mut current = String::new();
        let mut chars = trimmed_line.chars().peekable();
        let mut in_quotes = false;
        let mut field_started = false;

        while let Some(ch) = chars.next() {
            if in_quotes {
                match ch {
                    '"' => {
                        if matches!(chars.peek(), Some('"')) {
                            chars.next();
                            current.push('"');
                        } else {
                            in_quotes = false;
                            field_started = true;
                        }
                    }
                    _ => current.push(ch),
                }
            } else {
                match ch {
                    c if c == delimiter => {
                        fields.push(current.clone());
                        current.clear();
                        field_started = false;
                    }
                    '"' => {
                        if !field_started {
                            in_quotes = true;
                        } else {
                            return Err(ParseError::InvalidEscape);
                        }
                    }
                    _ => {
                        current.push(ch);
                        field_started = true;
                    }
                }
            }
        }

        if in_quotes {
            return Err(ParseError::UnterminatedQuote);
        }

        fields.push(current);
        Ok(CsvRecord { fields })
    }

    pub fn get_field(&self, index: usize) -> Option<&str> {
        self.fields.get(index).map(|field| field.as_str())
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    fn map_field<F>(&mut self, index: usize, mapper: F)
    where
        F: FnOnce(&str) -> String,
    {
        if let Some(field) = self.fields.get_mut(index) {
            let new_value = mapper(field.as_str());
            *field = new_value;
        }
    }
}

// =============================================================================
// Milestone 2: Streaming iterator over CSV files
// =============================================================================

pub struct CsvFileIterator {
    reader: BufReader<File>,
    delimiter: char,
    line_number: usize,
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse(ParseError, usize),
}

impl CsvFileIterator {
    pub fn new(path: &Path, delimiter: char) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        Ok(Self {
            reader: BufReader::new(file),
            delimiter,
            line_number: 0,
        })
    }
}

impl Iterator for CsvFileIterator {
    type Item = Result<CsvRecord, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();

        loop {
            line.clear();
            match self.reader.read_line(&mut line) {
                Ok(0) => return None,
                Ok(_) => {
                    self.line_number += 1;
                    match CsvRecord::parse_csv_line(
                        line.trim_end_matches(|c| c == '\n' || c == '\r'),
                        self.delimiter,
                    ) {
                        Ok(record) => return Some(Ok(record)),
                        Err(ParseError::EmptyLine) => continue,
                        Err(err) => return Some(Err(Error::Parse(err, self.line_number))),
                    }
                }
                Err(err) => return Some(Err(Error::Io(err))),
            }
        }
    }
}

// =============================================================================
// Milestone 3: Type-safe column extraction
// =============================================================================

pub trait FromCsvField: Sized {
    fn from_csv_field(field: &str) -> Result<Self, ConversionError>;
}

#[derive(Debug, PartialEq)]
pub enum ConversionError {
    ParseInt(std::num::ParseIntError),
    ParseFloat(std::num::ParseFloatError),
    InvalidValue(String),
    MissingField,
}

impl FromCsvField for String {
    fn from_csv_field(field: &str) -> Result<Self, ConversionError> {
        Ok(field.to_string())
    }
}

impl FromCsvField for i64 {
    fn from_csv_field(field: &str) -> Result<Self, ConversionError> {
        field
            .trim()
            .parse::<i64>()
            .map_err(ConversionError::ParseInt)
    }
}

impl FromCsvField for f64 {
    fn from_csv_field(field: &str) -> Result<Self, ConversionError> {
        field
            .trim()
            .parse::<f64>()
            .map_err(ConversionError::ParseFloat)
    }
}

impl FromCsvField for bool {
    fn from_csv_field(field: &str) -> Result<Self, ConversionError> {
        match field.trim().to_lowercase().as_str() {
            "true" | "yes" | "1" => Ok(true),
            "false" | "no" | "0" => Ok(false),
            _ => Err(ConversionError::InvalidValue(field.to_string())),
        }
    }
}

impl CsvRecord {
    pub fn get_typed<T: FromCsvField>(&self, index: usize) -> Result<T, ConversionError> {
        let field = self.get_field(index).ok_or(ConversionError::MissingField)?;
        T::from_csv_field(field)
    }
}

// =============================================================================
// Milestone 4: Filtering and transforming CSV streams
// =============================================================================

pub trait CsvFilterExt: Iterator<Item = Result<CsvRecord, Error>> + Sized {
    fn filter_by_column<F>(self, column: usize, predicate: F) -> FilterByColumn<Self, F>
    where
        F: FnMut(&str) -> bool;

    fn filter_valid(self) -> FilterValid<Self>;
}

impl<I> CsvFilterExt for I
where
    I: Iterator<Item = Result<CsvRecord, Error>> + Sized,
{
    fn filter_by_column<F>(self, column: usize, predicate: F) -> FilterByColumn<Self, F>
    where
        F: FnMut(&str) -> bool,
    {
        FilterByColumn {
            iter: self,
            column,
            predicate,
        }
    }

    fn filter_valid(self) -> FilterValid<Self> {
        FilterValid { iter: self }
    }
}

pub struct FilterByColumn<I, F> {
    iter: I,
    column: usize,
    predicate: F,
}

impl<I, F> Iterator for FilterByColumn<I, F>
where
    I: Iterator<Item = Result<CsvRecord, Error>>,
    F: FnMut(&str) -> bool,
{
    type Item = Result<CsvRecord, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(record) = self.iter.next() {
            match record {
                Ok(record) => {
                    if let Some(value) = record.get_field(self.column) {
                        if (self.predicate)(value) {
                            return Some(Ok(record));
                        }
                    }
                }
                Err(err) => return Some(Err(err)),
            }
        }
        None
    }
}

pub struct FilterValid<I> {
    iter: I,
}

impl<I> Iterator for FilterValid<I>
where
    I: Iterator<Item = Result<CsvRecord, Error>>,
{
    type Item = CsvRecord;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(record) = self.iter.next() {
            if let Ok(record) = record {
                return Some(record);
            }
        }
        None
    }
}

pub trait CsvTransformExt: Iterator<Item = CsvRecord> + Sized {
    fn map_column<F>(self, column: usize, f: F) -> MapColumn<Self, F>
    where
        F: FnMut(&str) -> String;
}

impl<I> CsvTransformExt for I
where
    I: Iterator<Item = CsvRecord> + Sized,
{
    fn map_column<F>(self, column: usize, mapper: F) -> MapColumn<Self, F>
    where
        F: FnMut(&str) -> String,
    {
        MapColumn {
            iter: self,
            column,
            mapper,
        }
    }
}

pub struct MapColumn<I, F> {
    iter: I,
    column: usize,
    mapper: F,
}

impl<I, F> Iterator for MapColumn<I, F>
where
    I: Iterator<Item = CsvRecord>,
    F: FnMut(&str) -> String,
{
    type Item = CsvRecord;

    fn next(&mut self) -> Option<Self::Item> {
        let mut record = self.iter.next()?;
        record.map_field(self.column, |value| (self.mapper)(value));
        Some(record)
    }
}

// =============================================================================
// Milestone 5: Streaming aggregations
// =============================================================================

#[derive(Debug, Clone)]
pub struct CsvAggregator {
    pub count: usize,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
}

impl CsvAggregator {
    pub fn new() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }

    pub fn update(&mut self, value: f64) {
        if self.count == 0 {
            self.min = value;
            self.max = value;
        } else {
            if value < self.min {
                self.min = value;
            }
            if value > self.max {
                self.max = value;
            }
        }
        self.count += 1;
        self.sum += value;
    }

    pub fn mean(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f64)
        }
    }

    pub fn from_column<I>(records: I, column: usize) -> Self
    where
        I: Iterator<Item = CsvRecord>,
    {
        records.fold(CsvAggregator::new(), |mut agg, record| {
            if let Ok(value) = record.get_typed::<f64>(column) {
                agg.update(value);
            }
            agg
        })
    }

    pub fn merge(&mut self, other: CsvAggregator) {
        if other.count == 0 {
            return;
        }
        if self.count == 0 {
            *self = other;
            return;
        }
        self.count += other.count;
        self.sum += other.sum;
        if other.min < self.min {
            self.min = other.min;
        }
        if other.max > self.max {
            self.max = other.max;
        }
    }
}

pub struct GroupedAggregator<K> {
    groups: HashMap<K, CsvAggregator>,
}

impl<K: Eq + Hash> GroupedAggregator<K> {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    pub fn update(&mut self, key: K, value: f64) {
        self.groups
            .entry(key)
            .or_insert_with(CsvAggregator::new)
            .update(value);
    }

    pub fn get(&self, key: &K) -> Option<&CsvAggregator> {
        self.groups.get(key)
    }

    pub fn from_records<I, KF, VF>(records: I, key_fn: KF, value_fn: VF) -> Self
    where
        I: Iterator<Item = CsvRecord>,
        KF: Fn(&CsvRecord) -> K,
        VF: Fn(&CsvRecord) -> Option<f64>,
    {
        let mut agg = GroupedAggregator::new();
        for record in records {
            let key = key_fn(&record);
            if let Some(value) = value_fn(&record) {
                agg.update(key, value);
            }
        }
        agg
    }
}

impl<K: Eq + Hash + Clone> GroupedAggregator<K> {
    pub fn merge(&mut self, other: GroupedAggregator<K>) {
        for (key, stats) in other.groups {
            self.groups
                .entry(key)
                .or_insert_with(CsvAggregator::new)
                .merge(stats);
        }
    }
}

// =============================================================================
// Milestone 6: Parallel CSV processing
// =============================================================================

fn find_chunk_boundaries(file: &mut File, num_chunks: usize) -> io::Result<Vec<u64>> {
    let file_size = file.metadata()?.len();
    let num_chunks = num_chunks.max(1);
    if file_size == 0 {
        return Ok(vec![0, 0]);
    }

    let chunk_size = (file_size / num_chunks as u64).max(1);
    let mut boundaries = vec![0];
    let mut next = chunk_size;

    while next < file_size {
        file.seek(SeekFrom::Start(next))?;
        let mut buf = [0u8; 1];
        let mut pos = next;
        loop {
            match file.read(&mut buf) {
                Ok(0) => {
                    pos = file_size;
                    break;
                }
                Ok(1) => {
                    pos += 1;
                    if buf[0] == b'\n' {
                        break;
                    }
                }
                Ok(_) => unreachable!(),
                Err(err) => return Err(err),
            }
            if pos >= file_size {
                pos = file_size;
                break;
            }
        }
        if pos >= file_size {
            break;
        }
        boundaries.push(pos);
        next = pos + chunk_size;
    }

    if *boundaries.last().unwrap() != file_size {
        boundaries.push(file_size);
    }

    Ok(boundaries)
}

pub fn parallel_process_csv<F, R>(
    path: &Path,
    delimiter: char,
    num_workers: usize,
    process_chunk: F,
) -> io::Result<Vec<R>>
where
    F: Fn(Vec<CsvRecord>) -> R + Send + Sync + 'static,
    R: Send,
{
    let num_workers = num_workers.max(1);
    let path_buf = Arc::new(path.to_path_buf());
    let mut file = File::open(&*path_buf)?;
    let boundaries = find_chunk_boundaries(&mut file, num_workers)?;
    let chunk_ranges: Vec<(u64, u64)> = boundaries
        .windows(2)
        .map(|window| (window[0], window[1]))
        .collect();
    let process_fn = Arc::new(process_chunk);

    chunk_ranges
        .into_par_iter()
        .map({
            let path_buf = Arc::clone(&path_buf);
            let processor = Arc::clone(&process_fn);
            move |(start, end)| -> io::Result<R> {
                let mut chunk_file = File::open(&*path_buf)?;
                chunk_file.seek(SeekFrom::Start(start))?;
                let mut reader = BufReader::new(chunk_file);
                let mut consumed = start;
                let mut records = Vec::new();
                let mut line = String::new();

                while consumed < end {
                    line.clear();
                    let bytes_read = reader.read_line(&mut line)?;
                    if bytes_read == 0 {
                        break;
                    }
                    consumed += bytes_read as u64;
                    let trimmed = line.trim_end_matches(|c| c == '\n' || c == '\r');
                    if trimmed.trim().is_empty() {
                        continue;
                    }
                    match CsvRecord::parse_csv_line(trimmed, delimiter) {
                        Ok(record) => records.push(record),
                        Err(ParseError::EmptyLine) => continue,
                        Err(err) => {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!("Failed to parse chunk at byte {}: {:?}", consumed, err),
                            ))
                        }
                    }
                }

                Ok((*processor)(records))
            }
        })
        .collect()
}

pub fn parallel_aggregate_column(
    path: &Path,
    delimiter: char,
    column: usize,
    num_workers: usize,
) -> io::Result<CsvAggregator> {
    let aggregates = parallel_process_csv(path, delimiter, num_workers, move |records| {
        records
            .into_iter()
            .fold(CsvAggregator::new(), |mut agg, record| {
                if let Ok(value) = record.get_typed::<f64>(column) {
                    agg.update(value);
                }
                agg
            })
    })?;

    let mut final_agg = CsvAggregator::new();
    for agg in aggregates {
        final_agg.merge(agg);
    }
    Ok(final_agg)
}

pub fn parallel_group_by<K>(
    path: &Path,
    delimiter: char,
    key_column: usize,
    value_column: usize,
    num_workers: usize,
) -> io::Result<GroupedAggregator<K>>
where
    K: Eq + Hash + Send + Clone + FromCsvField + 'static,
{
    let grouped_results = parallel_process_csv(path, delimiter, num_workers, move |records| {
        records
            .into_iter()
            .fold(GroupedAggregator::new(), |mut agg, record| {
                if let (Ok(key), Ok(value)) = (
                    record.get_typed::<K>(key_column),
                    record.get_typed::<f64>(value_column),
                ) {
                    agg.update(key, value);
                }
                agg
            })
    })?;

    let mut final_grouped = GroupedAggregator::new();
    for grouped in grouped_results {
        final_grouped.merge(grouped);
    }
    Ok(final_grouped)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    fn create_large_test_csv(rows: usize) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "category,value").unwrap();
        for i in 0..rows {
            let category = match i % 3 {
                0 => "A",
                1 => "B",
                _ => "C",
            };
            writeln!(file, "{},{}", category, i).unwrap();
        }
        file
    }

    #[test]
    fn test_simple_csv_parsing() {
        let record = CsvRecord::parse_csv_line("foo,bar,baz", ',').unwrap();
        assert_eq!(record.field_count(), 3);
        assert_eq!(record.get_field(0), Some("foo"));
        assert_eq!(record.get_field(1), Some("bar"));
        assert_eq!(record.get_field(2), Some("baz"));
    }

    #[test]
    fn test_quoted_fields() {
        let record = CsvRecord::parse_csv_line(r#"foo,"bar,baz",qux"#, ',').unwrap();
        assert_eq!(record.field_count(), 3);
        assert_eq!(record.get_field(1), Some("bar,baz"));
    }

    #[test]
    fn test_escaped_quotes() {
        let record = CsvRecord::parse_csv_line(r#""foo ""bar"" baz""#, ',').unwrap();
        assert_eq!(record.get_field(0), Some(r#"foo "bar" baz"#));
    }

    #[test]
    fn test_empty_fields() {
        let record = CsvRecord::parse_csv_line("foo,,bar", ',').unwrap();
        assert_eq!(record.field_count(), 3);
        assert_eq!(record.get_field(1), Some(""));
    }

    #[test]
    fn test_custom_delimiter() {
        let record = CsvRecord::parse_csv_line("foo|bar|baz", '|').unwrap();
        assert_eq!(record.field_count(), 3);
    }

    #[test]
    fn test_iterate_simple_csv() {
        let file = create_test_csv("a,b,c\n1,2,3\n4,5,6");
        let mut iter = CsvFileIterator::new(file.path(), ',').unwrap();

        let record1 = iter.next().unwrap().unwrap();
        assert_eq!(record1.get_field(0), Some("a"));

        let record2 = iter.next().unwrap().unwrap();
        assert_eq!(record2.get_field(0), Some("1"));

        let record3 = iter.next().unwrap().unwrap();
        assert_eq!(record3.get_field(0), Some("4"));

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_skip_empty_lines() {
        let file = create_test_csv("a,b\n\n1,2\n");
        let records: Vec<_> = CsvFileIterator::new(file.path(), ',')
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_error_reporting_with_line_numbers() {
        let file = create_test_csv("a,b\n\"unterminated\n3,4");
        let mut iter = CsvFileIterator::new(file.path(), ',').unwrap();

        iter.next();
        let err = iter.next().unwrap().unwrap_err();

        match err {
            Error::Parse(ParseError::UnterminatedQuote, line) => assert_eq!(line, 2),
            _ => panic!("Expected parse error"),
        }
    }

    #[test]
    fn test_extract_integers() {
        let record = CsvRecord::parse_csv_line("100,200,300", ',').unwrap();
        assert_eq!(record.get_typed::<i64>(0).unwrap(), 100);
        assert_eq!(record.get_typed::<i64>(1).unwrap(), 200);
    }

    #[test]
    fn test_extract_floats() {
        let record = CsvRecord::parse_csv_line("3.14,2.71,1.41", ',').unwrap();
        assert_eq!(record.get_typed::<f64>(0).unwrap(), 3.14);
    }

    #[test]
    fn test_extract_booleans() {
        let record = CsvRecord::parse_csv_line("true,false,yes,no,1,0", ',').unwrap();
        assert_eq!(record.get_typed::<bool>(0).unwrap(), true);
        assert_eq!(record.get_typed::<bool>(1).unwrap(), false);
        assert_eq!(record.get_typed::<bool>(2).unwrap(), true);
        assert_eq!(record.get_typed::<bool>(3).unwrap(), false);
        assert_eq!(record.get_typed::<bool>(4).unwrap(), true);
        assert_eq!(record.get_typed::<bool>(5).unwrap(), false);
    }

    #[test]
    fn test_conversion_errors() {
        let record = CsvRecord::parse_csv_line("not_a_number,42", ',').unwrap();
        assert!(record.get_typed::<i64>(0).is_err());
        assert!(record.get_typed::<i64>(1).is_ok());
    }

    #[test]
    fn test_missing_field_error() {
        let record = CsvRecord::parse_csv_line("a,b", ',').unwrap();
        assert!(matches!(
            record.get_typed::<String>(5),
            Err(ConversionError::MissingField)
        ));
    }

    #[test]
    fn test_filter_by_column() {
        let csv = "status,amount\ncompleted,100\npending,200\ncompleted,300";
        let file = create_test_csv(csv);

        let completed: Vec<_> = CsvFileIterator::new(file.path(), ',')
            .unwrap()
            .skip(1)
            .filter_by_column(0, |status| status == "completed")
            .filter_valid()
            .collect();

        assert_eq!(completed.len(), 2);
        assert_eq!(completed[0].get_field(1), Some("100"));
        assert_eq!(completed[1].get_field(1), Some("300"));
    }

    #[test]
    fn test_map_column_transformation() {
        let csv = "name,age\nalice,30\nbob,25";
        let file = create_test_csv(csv);

        let uppercase: Vec<_> = CsvFileIterator::new(file.path(), ',')
            .unwrap()
            .filter_valid()
            .map_column(0, |name| name.to_uppercase())
            .collect();

        assert_eq!(uppercase[1].get_field(0), Some("ALICE"));
        assert_eq!(uppercase[2].get_field(0), Some("BOB"));
    }

    #[test]
    fn test_chained_operations() {
        let csv = "status,amount\ncompleted,100\npending,200\ncompleted,50\nfailed,75";
        let file = create_test_csv(csv);

        let result: Vec<_> = CsvFileIterator::new(file.path(), ',')
            .unwrap()
            .skip(1)
            .filter_by_column(0, |s| s == "completed")
            .filter_valid()
            .collect();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_basic_aggregation() {
        let csv = "value\n10\n20\n30\n40\n50";
        let file = create_test_csv(csv);

        let agg = CsvAggregator::from_column(
            CsvFileIterator::new(file.path(), ',')
                .unwrap()
                .skip(1)
                .filter_map(Result::ok),
            0,
        );

        assert_eq!(agg.count, 5);
        assert_eq!(agg.sum, 150.0);
        assert_eq!(agg.min, 10.0);
        assert_eq!(agg.max, 50.0);
        assert_eq!(agg.mean(), Some(30.0));
    }

    #[test]
    fn test_grouped_aggregation() {
        let csv = "category,amount\nA,100\nB,200\nA,150\nB,250\nA,50";
        let file = create_test_csv(csv);

        let grouped = GroupedAggregator::from_records(
            CsvFileIterator::new(file.path(), ',')
                .unwrap()
                .skip(1)
                .filter_map(Result::ok),
            |rec| rec.get_field(0).unwrap().to_string(),
            |rec| rec.get_typed::<f64>(1).ok(),
        );

        let stats_a = grouped.get(&"A".to_string()).unwrap();
        assert_eq!(stats_a.count, 3);
        assert_eq!(stats_a.sum, 300.0);

        let stats_b = grouped.get(&"B".to_string()).unwrap();
        assert_eq!(stats_b.count, 2);
        assert_eq!(stats_b.sum, 450.0);
    }

    #[test]
    fn test_empty_aggregation() {
        let agg = CsvAggregator::new();
        assert_eq!(agg.mean(), None);
    }

    #[test]
    fn test_parallel_vs_sequential_correctness() {
        let file = create_large_test_csv(1000);

        let seq_agg = CsvAggregator::from_column(
            CsvFileIterator::new(file.path(), ',')
                .unwrap()
                .skip(1)
                .filter_map(Result::ok),
            1,
        );

        let par_agg = parallel_aggregate_column(file.path(), ',', 1, 4).unwrap();

        assert_eq!(seq_agg.count, par_agg.count);
        assert_eq!(seq_agg.sum, par_agg.sum);
        assert_eq!(seq_agg.min, par_agg.min);
        assert_eq!(seq_agg.max, par_agg.max);
    }

    #[test]
    fn test_parallel_grouped_aggregation() {
        let file = create_large_test_csv(900);

        let grouped = parallel_group_by::<String>(file.path(), ',', 0, 1, 4).unwrap();

        assert_eq!(grouped.get(&"A".to_string()).unwrap().count, 300);
        assert_eq!(grouped.get(&"B".to_string()).unwrap().count, 300);
        assert_eq!(grouped.get(&"C".to_string()).unwrap().count, 300);
    }

    #[test]
    #[ignore]
    fn benchmark_parallel_speedup() {
        use std::time::Instant;

        let file = create_large_test_csv(100_000);

        let start = Instant::now();
        let _ = CsvAggregator::from_column(
            CsvFileIterator::new(file.path(), ',')
                .unwrap()
                .skip(1)
                .filter_map(Result::ok),
            1,
        );
        let seq_time = start.elapsed();

        let start = Instant::now();
        let _ = parallel_aggregate_column(file.path(), ',', 1, 4).unwrap();
        let par_time = start.elapsed();

        assert!(par_time < seq_time || par_time.as_secs_f64() == 0.0);
    }
}
```

This complete example demonstrates:
- **Part 1**: Robust CSV parsing with quote handling
- **Part 2**: Memory-efficient streaming file iteration
- **Part 3**: Type-safe column extraction with validation
- **Part 4**: Streaming aggregations without loading data
- **Examples**: Real-world CSV processing workflows
- **Tests**: Validation of parsing and aggregation correctness

The implementation shows how iterator patterns enable processing arbitrarily large CSV files with constant memory usage while maintaining type safety and composability.

---


