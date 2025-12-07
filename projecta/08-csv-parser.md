
## Project 4: CSV Stream Transformer with Iterator Adapters

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
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

//==============================================================================
// Part 1: CSV Parsing
//==============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct CsvRecord {
    fields: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnterminatedQuote,
    EmptyLine,
}

impl CsvRecord {
    pub fn parse_csv_line(line: &str, delimiter: char) -> Result<Self, ParseError> {
        if line.trim().is_empty() {
            return Err(ParseError::EmptyLine);
        }

        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    if in_quotes {
                        // Check for escaped quote
                        if chars.peek() == Some(&'"') {
                            current_field.push('"');
                            chars.next();
                        } else {
                            in_quotes = false;
                        }
                    } else {
                        in_quotes = true;
                    }
                }
                c if c == delimiter && !in_quotes => {
                    fields.push(current_field.clone());
                    current_field.clear();
                }
                _ => {
                    current_field.push(ch);
                }
            }
        }

        if in_quotes {
            return Err(ParseError::UnterminatedQuote);
        }

        fields.push(current_field);
        Ok(CsvRecord { fields })
    }

    pub fn get_field(&self, index: usize) -> Option<&str> {
        self.fields.get(index).map(|s| s.as_str())
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}

//==============================================================================
// Part 2: Streaming File Iterator
//==============================================================================

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
        Ok(CsvFileIterator {
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
                Ok(0) => return None, // EOF
                Ok(_) => {
                    self.line_number += 1;

                    match CsvRecord::parse_csv_line(&line.trim(), self.delimiter) {
                        Ok(record) => return Some(Ok(record)),
                        Err(ParseError::EmptyLine) => continue, // Skip empty lines
                        Err(e) => return Some(Err(Error::Parse(e, self.line_number))),
                    }
                }
                Err(e) => return Some(Err(Error::Io(e))),
            }
        }
    }
}

//==============================================================================
// Part 3: Type-Safe Column Extraction
//==============================================================================

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
        field.trim().parse().map_err(ConversionError::ParseInt)
    }
}

impl FromCsvField for f64 {
    fn from_csv_field(field: &str) -> Result<Self, ConversionError> {
        field.trim().parse().map_err(ConversionError::ParseFloat)
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

//==============================================================================
// Part 4: Aggregations
//==============================================================================

#[derive(Debug, Clone)]
pub struct CsvAggregator {
    count: usize,
    sum: f64,
    min: f64,
    max: f64,
}

impl CsvAggregator {
    pub fn new() -> Self {
        CsvAggregator {
            count: 0,
            sum: 0.0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }

    pub fn update(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    pub fn mean(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f64)
        }
    }

    pub fn merge(&mut self, other: CsvAggregator) {
        self.count += other.count;
        self.sum += other.sum;
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
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
}

pub struct GroupedAggregator<K> {
    groups: HashMap<K, CsvAggregator>,
}

impl<K: Eq + std::hash::Hash> GroupedAggregator<K> {
    pub fn new() -> Self {
        GroupedAggregator {
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

    pub fn merge(&mut self, other: GroupedAggregator<K>) {
        for (key, agg) in other.groups {
            self.groups
                .entry(key)
                .and_modify(|existing| existing.merge(agg.clone()))
                .or_insert(agg);
        }
    }
}

//==============================================================================
// Example Usage
//==============================================================================

fn main() -> std::io::Result<()> {
    println!("=== CSV Transformer Examples ===\n");

    // Example 1: Basic CSV reading and parsing
    println!("Example 1: Reading CSV");
    {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new()?;
        writeln!(
            file,
            "name,age,score\nAlice,30,95.5\nBob,25,87.3\nCharlie,35,92.1"
        )?;

        for (i, result) in CsvFileIterator::new(file.path(), ',')?.enumerate() {
            match result {
                Ok(record) => {
                    println!(
                        "Row {}: {:?}",
                        i,
                        (0..record.field_count())
                            .map(|idx| record.get_field(idx).unwrap())
                            .collect::<Vec<_>>()
                    );
                }
                Err(e) => eprintln!("Error: {:?}", e),
            }
        }
    }
    println!();

    // Example 2: Type-safe extraction
    println!("Example 2: Type-Safe Extraction");
    {
        let record = CsvRecord::parse_csv_line("Alice,30,95.5", ',').unwrap();
        let name: String = record.get_typed(0).unwrap();
        let age: i64 = record.get_typed(1).unwrap();
        let score: f64 = record.get_typed(2).unwrap();

        println!("Name: {}, Age: {}, Score: {}", name, age, score);
    }
    println!();

    // Example 3: Aggregation
    println!("Example 3: Column Aggregation");
    {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new()?;
        writeln!(file, "value\n10.5\n20.3\n15.7\n30.1\n25.4")?;

        let agg = CsvAggregator::from_column(
            CsvFileIterator::new(file.path(), ',')?
                .skip(1)
                .filter_map(Result::ok),
            0,
        );

        println!("Count: {}", agg.count);
        println!("Sum: {:.2}", agg.sum);
        println!("Mean: {:.2}", agg.mean().unwrap());
        println!("Min: {:.2}", agg.min);
        println!("Max: {:.2}", agg.max);
    }
    println!();

    // Example 4: Filtered aggregation
    println!("Example 4: Filtered Aggregation");
    {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new()?;
        writeln!(
            file,
            "status,amount\ncompleted,100.50\npending,200.25\ncompleted,150.75\nfailed,50.00\ncompleted,75.25"
        )?;

        let completed_sum: f64 = CsvFileIterator::new(file.path(), ',')?
            .skip(1)
            .filter_map(Result::ok)
            .filter(|record| record.get_field(0) == Some("completed"))
            .filter_map(|record| record.get_typed::<f64>(1).ok())
            .sum();

        println!("Total completed amount: ${:.2}", completed_sum);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_parsing() {
        let record = CsvRecord::parse_csv_line(r#"foo,"bar,baz",qux"#, ',').unwrap();
        assert_eq!(record.field_count(), 3);
        assert_eq!(record.get_field(1), Some("bar,baz"));
    }

    #[test]
    fn test_type_conversion() {
        let record = CsvRecord::parse_csv_line("42,3.14,true", ',').unwrap();
        assert_eq!(record.get_typed::<i64>(0).unwrap(), 42);
        assert_eq!(record.get_typed::<f64>(1).unwrap(), 3.14);
        assert_eq!(record.get_typed::<bool>(2).unwrap(), true);
    }

    #[test]
    fn test_aggregation() {
        let records = vec![
            CsvRecord::parse_csv_line("10", ',').unwrap(),
            CsvRecord::parse_csv_line("20", ',').unwrap(),
            CsvRecord::parse_csv_line("30", ',').unwrap(),
        ];

        let agg = CsvAggregator::from_column(records.into_iter(), 0);
        assert_eq!(agg.count, 3);
        assert_eq!(agg.sum, 60.0);
        assert_eq!(agg.mean(), Some(20.0));
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

