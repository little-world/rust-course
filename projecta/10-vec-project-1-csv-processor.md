# Chapter 10: Vec & Slice Manipulation - Project 1

## Project 1: High-Performance CSV Batch Processor

### Problem Statement

Build a high-performance CSV processor that reads large CSV files, performs transformations, validates data, and writes results in batches to a database or output file. The processor must handle files larger than available RAM using efficient chunking, minimize allocations through capacity pre-allocation and vector reuse, and achieve maximum throughput through proper batching strategies.

Your processor should support:
- Parsing CSV files line-by-line without loading entire file
- Transforming and validating records (type conversion, constraint checking)
- Batching records for efficient database inserts (e.g., 1000 records per batch)
- Handling errors gracefully (skip invalid rows with logging)
- Supporting filtering and deduplication
- Optimizing memory usage through capacity management

Example workflow:
```
Input CSV: users.csv (100M rows, 5GB)
Operations: Parse → Validate → Transform → Deduplicate → Batch insert (1000/batch)
Output: PostgreSQL database or output.csv
Performance target: Process 100K rows/second
```


### Milestone 1: Basic CSV Parser with Structured Records

**Goal**: Parse CSV file into structured records with error handling.

**What to implement**:
- Define `UserRecord` struct for data representation
- Parse CSV line-by-line using csv crate
- Convert string fields to appropriate types
- Handle parsing errors gracefully

**Architecture**:
- Structs: `UserRecord`, `ParseError`
- Fields (UserRecord): `id: u64`, `name: String`, `email: String`, `age: u32`, `country: String`
- Enums: `ParseError` (InvalidFormat, InvalidType, MissingField)
- Functions:
  - `UserRecord::from_csv_row(&csv::StringRecord) -> Result<Self, ParseError>` - Parse single row
  - `parse_csv(path: &str) -> Result<Vec<UserRecord>, Box<dyn Error>>` - Parse entire file

---

**Starter Code**:

```rust
use csv::{Reader, StringRecord};
use std::error::Error;
use std::fs::File;

/// CSV record representing a user
#[derive(Debug, Clone, PartialEq)]
pub struct UserRecord {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub age: u32,
    pub country: String,
}

/// CSV parsing errors
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid CSV format: {0}")]
    InvalidFormat(String),

    #[error("Invalid type for field '{field}': '{value}'")]
    InvalidType { field: String, value: String },

    #[error("Missing required field: {0}")]
    MissingField(String),
}

impl UserRecord {
    /// Parse CSV row into UserRecord
    /// Role: Convert StringRecord to typed struct
    pub fn from_csv_row(row: &StringRecord) -> Result<Self, ParseError> {
        todo!("Extract fields, parse types, handle errors")
    }
}

/// Parse entire CSV file
/// Role: Read file and convert all valid rows
pub fn parse_csv(path: &str) -> Result<Vec<UserRecord>, Box<dyn Error>> {
    todo!("Open file, iterate rows, collect results")
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
    fn test_parse_valid_row() {
        let csv_content = "id,name,email,age,country\n1,Alice,alice@test.com,30,US";
        let file = create_test_csv(csv_content);

        let records = parse_csv(file.path().to_str().unwrap()).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[0].email, "alice@test.com");
        assert_eq!(records[0].age, 30);
        assert_eq!(records[0].country, "US");
    }

    #[test]
    fn test_parse_multiple_rows() {
        let csv_content = "\
id,name,email,age,country
1,Alice,alice@test.com,30,US
2,Bob,bob@test.com,25,UK
3,Charlie,charlie@test.com,35,CA";

        let file = create_test_csv(csv_content);
        let records = parse_csv(file.path().to_str().unwrap()).unwrap();

        assert_eq!(records.len(), 3);
        assert_eq!(records[1].name, "Bob");
        assert_eq!(records[2].age, 35);
    }

    #[test]
    fn test_parse_invalid_age() {
        let row = StringRecord::from(vec!["1", "Alice", "alice@test.com", "invalid", "US"]);
        let result = UserRecord::from_csv_row(&row);

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InvalidType { field, .. } => assert_eq!(field, "age"),
            _ => panic!("Expected InvalidType error"),
        }
    }

    #[test]
    fn test_parse_missing_field() {
        let row = StringRecord::from(vec!["1", "Alice", "alice@test.com"]);
        let result = UserRecord::from_csv_row(&row);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseError::MissingField(_)));
    }

    #[test]
    fn test_parse_skips_invalid_rows() {
        let csv_content = "\
id,name,email,age,country
1,Alice,alice@test.com,30,US
2,Bob,bob@test.com,invalid_age,UK
3,Charlie,charlie@test.com,35,CA";

        let file = create_test_csv(csv_content);
        let records = parse_csv(file.path().to_str().unwrap()).unwrap();

        // Should skip invalid row
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[1].id, 3);
    }

    #[test]
    fn test_parse_empty_file() {
        let csv_content = "id,name,email,age,country\n";
        let file = create_test_csv(csv_content);

        let records = parse_csv(file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 0);
    }
}
```

---

### Milestone 2: Pre-Allocate Capacity to Eliminate Reallocations

**Goal**: Optimize memory allocations through capacity pre-allocation.

**Why the previous milestone is not enough**: Milestone 1 uses `Vec::new()`, which starts with capacity 0. As you push records, the vector reallocates (capacity doubling) multiple times. For 1M records, this causes ~20 reallocations, each copying all existing data.

**What's the improvement**: Pre-allocating eliminates reallocations entirely. Instead of 20 allocations with O(n log n) total copying, we get 1 allocation with zero copying. For 1M records:
- Before: ~20 allocations, ~2M items copied
- After: 1 allocation, 0 items copied

This is 10-50x faster for large datasets.

**Optimization focus**: Speed and memory efficiency through allocation elimination.

**Architecture**:
- Functions:
  - `count_lines(path: &str) -> Result<usize, io::Error>` - Count file lines
  - `parse_csv_optimized(path: &str) -> Result<Vec<UserRecord>, Box<dyn Error>>` - Parse with pre-allocation

---

**Starter Code**:

```rust
use std::io::{BufRead, BufReader};

/// Count lines in file
/// Role: Estimate capacity needed for Vec
pub fn count_lines(path: &str) -> Result<usize, std::io::Error> {
    todo!("Open file, count lines using BufReader")
}

/// Parse CSV with pre-allocated capacity
/// Role: Eliminate reallocations during parsing
pub fn parse_csv_optimized(path: &str) -> Result<Vec<UserRecord>, Box<dyn Error>> {
    todo!("Count lines first, allocate Vec::with_capacity, parse")
}

/// Track allocation statistics
/// Role: Measure allocation efficiency
#[derive(Debug, Default)]
pub struct AllocationStats {
    pub allocations: usize,
    pub reallocations: usize,
    pub bytes_copied: usize,
}

/// Wrapper to track Vec allocations
/// Role: Observe allocation behavior
pub struct TrackedVec<T> {
    vec: Vec<T>,
    stats: AllocationStats,
}

impl<T> TrackedVec<T> {
    /// Create with capacity tracking
    /// Role: Initialize with known capacity
    pub fn with_capacity(capacity: usize) -> Self {
        todo!("Create Vec, track initial allocation")
    }

    /// Push with reallocation tracking
    /// Role: Monitor when reallocations occur
    pub fn push(&mut self, value: T) {
        todo!("Check capacity before push, track realloc if needed")
    }

    /// Get statistics
    /// Role: Query allocation metrics
    pub fn stats(&self) -> &AllocationStats {
        &self.stats
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
    fn test_count_lines() {
        let csv_content = "header\nrow1\nrow2\nrow3";
        let file = create_test_csv(csv_content);

        let count = count_lines(file.path().to_str().unwrap()).unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn test_optimized_parsing_allocates_once() {
        let csv_content = "\
id,name,email,age,country
1,Alice,alice@test.com,30,US
2,Bob,bob@test.com,25,UK
3,Charlie,charlie@test.com,35,CA";

        let file = create_test_csv(csv_content);
        let records = parse_csv_optimized(file.path().to_str().unwrap()).unwrap();

        assert_eq!(records.len(), 3);

        // Verify capacity matches initial allocation
        // (capacity should be close to line count - 1 for header)
        assert!(records.capacity() >= records.len());
    }

    #[test]
    fn test_tracked_vec_no_reallocations() {
        let mut vec = TrackedVec::with_capacity(100);

        for i in 0..100 {
            vec.push(i);
        }

        let stats = vec.stats();
        assert_eq!(stats.allocations, 1); // Only initial allocation
        assert_eq!(stats.reallocations, 0); // No reallocations
    }

    #[test]
    fn test_tracked_vec_with_reallocations() {
        let mut vec = TrackedVec::with_capacity(10);

        for i in 0..100 {
            vec.push(i);
        }

        let stats = vec.stats();
        assert_eq!(stats.allocations, 1);
        assert!(stats.reallocations > 0); // Should have reallocated
    }

    #[test]
    fn test_performance_comparison() {
        use std::time::Instant;

        let csv_content: String = {
            let mut content = String::from("id,name,email,age,country\n");
            for i in 0..10000 {
                content.push_str(&format!("{},User{},user{}@test.com,{},US\n", i, i, i, 20 + (i % 50)));
            }
            content
        };

        let file = create_test_csv(&csv_content);

        // Without pre-allocation
        let start = Instant::now();
        let records1 = parse_csv(file.path().to_str().unwrap()).unwrap();
        let time1 = start.elapsed();

        // With pre-allocation
        let start = Instant::now();
        let records2 = parse_csv_optimized(file.path().to_str().unwrap()).unwrap();
        let time2 = start.elapsed();

        assert_eq!(records1.len(), records2.len());

        println!("Without pre-allocation: {:?}", time1);
        println!("With pre-allocation: {:?}", time2);

        // Optimized should be faster (though margin varies)
        // This is more for observation than assertion
    }

    #[test]
    fn test_capacity_efficiency() {
        let csv_content = "\
id,name,email,age,country
1,Alice,alice@test.com,30,US
2,Bob,bob@test.com,25,UK
3,Charlie,charlie@test.com,35,CA";

        let file = create_test_csv(csv_content);
        let records = parse_csv_optimized(file.path().to_str().unwrap()).unwrap();

        // Capacity should not be wastefully large
        assert!(records.capacity() < records.len() * 2);
    }
}
```

---

### Milestone 3: Streaming Processing with Chunking

**Goal**: Process file in chunks to support files larger than RAM.

**Why the previous milestone is not enough**: Milestone 2 loads entire file into memory. This fails for files larger than RAM (10GB+ CSVs are common in production).

**What's the improvement**: Chunking processes data in fixed-size windows. Memory usage is O(chunk_size), not O(file_size). A 10GB file with 1GB RAM? No problem—process 10K records at a time. Reusing the chunk buffer (clear instead of allocating new Vec) eliminates per-chunk allocations.

**Optimization focus**: Memory efficiency—constant memory usage regardless of file size.

**Architecture**:
- Functions:
  - `process_csv_chunked<F>(path, chunk_size, process_chunk) -> Result<(), Error>` - Streaming processor
  - Callback: `F: FnMut(&[UserRecord])` - Process each chunk

---

**Starter Code**:

```rust
/// Process CSV in chunks with callback
/// Role: Enable processing files larger than RAM
pub fn process_csv_chunked<F>(
    path: &str,                        //  Input CSV file                   
    chunk_size: usize,                 //  Records per chunk          
    mut process_chunk: F,              //  Callback for each chunk 
) -> Result<(), Box<dyn Error>>
where
    F: FnMut(&[UserRecord]),
{
    todo!("Read CSV, accumulate into chunks, call callback when full")
}

/// Statistics for chunked processing
#[derive(Debug, Default)]
pub struct ChunkStats {
    pub total_chunks: usize,            // Number of chunks processed        
    pub total_records: usize,           // Total records processed          
    pub peak_memory_bytes: usize,       // Maximum chunk size in memory 
}

/// Process CSV with statistics tracking
/// Role: Monitor chunking efficiency
pub fn process_csv_chunked_with_stats<F>(
    path: &str,
    chunk_size: usize,
    mut process_chunk: F,
) -> Result<ChunkStats, Box<dyn Error>>
where
    F: FnMut(&[UserRecord]),
{
    todo!("Process chunks, track statistics")
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_chunked_processing() {
        let csv_content = "\
id,name,email,age,country
1,Alice,alice@test.com,30,US
2,Bob,bob@test.com,25,UK
3,Charlie,charlie@test.com,35,CA
4,Diana,diana@test.com,28,FR
5,Eve,eve@test.com,32,DE";

        let file = create_test_csv(csv_content);

        let chunks_processed = Arc::new(Mutex::new(0));
        let total_records = Arc::new(Mutex::new(0));

        let chunks_clone = chunks_processed.clone();
        let records_clone = total_records.clone();

        process_csv_chunked(file.path().to_str().unwrap(), 2, |chunk| {
            *chunks_clone.lock().unwrap() += 1;
            *records_clone.lock().unwrap() += chunk.len();
        })
        .unwrap();

        assert_eq!(*chunks_processed.lock().unwrap(), 3); // 2 + 2 + 1 = 3 chunks
        assert_eq!(*total_records.lock().unwrap(), 5);
    }

    #[test]
    fn test_chunk_buffer_reuse() {
        let csv_content: String = {
            let mut content = String::from("id,name,email,age,country\n");
            for i in 0..1000 {
                content.push_str(&format!("{},User{},user{}@test.com,{},US\n", i, i, i, 20 + (i % 50)));
            }
            content
        };

        let file = create_test_csv(&csv_content);

        let chunk_sizes = Arc::new(Mutex::new(Vec::new()));
        let sizes_clone = chunk_sizes.clone();

        process_csv_chunked(file.path().to_str().unwrap(), 100, |chunk| {
            sizes_clone.lock().unwrap().push(chunk.len());
        })
        .unwrap();

        let sizes = chunk_sizes.lock().unwrap();

        // All but last chunk should be exactly chunk_size
        for &size in sizes.iter().take(sizes.len() - 1) {
            assert_eq!(size, 100);
        }

        // Last chunk may be smaller
        assert!(*sizes.last().unwrap() <= 100);
    }

    #[test]
    fn test_memory_usage_constant() {
        // This test verifies memory doesn't grow with file size
        let csv_content: String = {
            let mut content = String::from("id,name,email,age,country\n");
            for i in 0..10000 {
                content.push_str(&format!("{},User{},user{}@test.com,{},US\n", i, i, i, 20 + (i % 50)));
            }
            content
        };

        let file = create_test_csv(&csv_content);

        let max_chunk_size = Arc::new(Mutex::new(0));
        let max_clone = max_chunk_size.clone();

        process_csv_chunked(file.path().to_str().unwrap(), 1000, |chunk| {
            let size = chunk.len();
            let mut max = max_clone.lock().unwrap();
            if size > *max {
                *max = size;
            }
        })
        .unwrap();

        // Max chunk size should not exceed chunk_size parameter
        assert!(*max_chunk_size.lock().unwrap() <= 1000);
    }

    #[test]
    fn test_process_empty_file() {
        let csv_content = "id,name,email,age,country\n";
        let file = create_test_csv(csv_content);

        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        process_csv_chunked(file.path().to_str().unwrap(), 100, |_chunk| {
            *called_clone.lock().unwrap() = true;
        })
        .unwrap();

        // Callback should not be called for empty file
        assert!(!*called.lock().unwrap());
    }

    #[test]
    fn test_chunked_with_stats() {
        let csv_content: String = {
            let mut content = String::from("id,name,email,age,country\n");
            for i in 0..500 {
                content.push_str(&format!("{},User{},user{}@test.com,{},US\n", i, i, i, 20 + (i % 50)));
            }
            content
        };

        let file = create_test_csv(&csv_content);

        let stats = process_csv_chunked_with_stats(
            file.path().to_str().unwrap(),
            100,
            |_chunk| {
                // Process chunk
            },
        )
        .unwrap();

        assert_eq!(stats.total_chunks, 5); // 500 / 100 = 5
        assert_eq!(stats.total_records, 500);
    }
}
```

---

### Milestone 4: Batch Database Inserts with Transactions

**Goal**: Insert records to database in batches for maximum throughput.

**Why the previous milestone is not enough**: Processing chunks is great, but inserting one record at a time to database is extremely slow due to network round-trips and transaction overhead.

**What's the improvement**: Batch inserts dramatically reduce overhead:
- Single-row inserts: 100K rows = 100K queries = 100K round-trips ≈ 100 seconds
- Batched inserts (1000/batch): 100K rows = 100 queries = 100 round-trips ≈ 1 second

This is 100x speedup! Batching amortizes connection, parsing, and transaction overhead.

**Optimization focus**: Speed through batching (reducing I/O overhead).

**Architecture**:
- Functions:
  - `insert_batch(tx: &Transaction, records: &[UserRecord]) -> Result<(), rusqlite::Error>` - Batch insert
  - `import_csv_to_db(path, db_path, batch_size) -> Result<(), Error>` - Complete import

---

**Starter Code**:

```rust
use rusqlite::{Connection, Transaction, params};

/// Insert batch of records in single query
/// Multi-row INSERT
/// Role: Minimize database round-trips
pub fn insert_batch(
    tx: &Transaction,
    records: &[UserRecord],
) -> Result<(), rusqlite::Error> {
    todo!("Build multi-row INSERT statement, execute with all parameters")
}

/// Create database schema
/// Role: Initialize tables
pub fn create_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    todo!("CREATE TABLE users with appropriate columns")
}

/// Import CSV to database with batching
/// Role: Production-ready CSV import
pub fn import_csv_to_db(
    path: &str,                        // CSV file path               
    db_path: &str,                     // SQLite database path     
    batch_size: usize,                 // Records per batch     
) -> Result<(), Box<dyn Error>> {
    todo!("Create schema, process CSV in chunks, batch insert with transactions")
}

/// Database import statistics
#[derive(Debug, Default)]
pub struct ImportStats {
    pub records_imported: usize,           // Successful inserts  
    pub records_failed: usize,             // Failed inserts        
    pub batches_processed: usize,          // Number of batches  
    pub duration_ms: u64,                  // Total time                 
}

/// Import with detailed statistics
/// Role: Monitor import performance
pub fn import_csv_to_db_with_stats(
    path: &str,
    db_path: &str,
    batch_size: usize,
) -> Result<ImportStats, Box<dyn Error>> {
    todo!("Track timing, counts, report statistics")
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_create_schema() {
        let db = NamedTempFile::new().unwrap();
        let conn = Connection::open(db.path()).unwrap();

        create_schema(&conn).unwrap();

        // Verify table exists
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='users'")
            .unwrap();
        let exists: bool = stmt.exists([]).unwrap();
        assert!(exists);
    }

    #[test]
    fn test_insert_single_batch() {
        let db = NamedTempFile::new().unwrap();
        let conn = Connection::open(db.path()).unwrap();
        create_schema(&conn).unwrap();

        let records = vec![
            UserRecord {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@test.com".to_string(),
                age: 30,
                country: "US".to_string(),
            },
            UserRecord {
                id: 2,
                name: "Bob".to_string(),
                email: "bob@test.com".to_string(),
                age: 25,
                country: "UK".to_string(),
            },
        ];

        let tx = conn.transaction().unwrap();
        insert_batch(&tx, &records).unwrap();
        tx.commit().unwrap();

        // Verify records inserted
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
            .unwrap();

        assert_eq!(count, 2);
    }

    #[test]
    fn test_import_csv_to_db() {
        let csv_content = "\
id,name,email,age,country
1,Alice,alice@test.com,30,US
2,Bob,bob@test.com,25,UK
3,Charlie,charlie@test.com,35,CA";

        let csv_file = create_test_csv(csv_content);
        let db = NamedTempFile::new().unwrap();

        import_csv_to_db(
            csv_file.path().to_str().unwrap(),
            db.path().to_str().unwrap(),
            10,
        )
        .unwrap();

        let conn = Connection::open(db.path()).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
            .unwrap();

        assert_eq!(count, 3);
    }

    #[test]
    fn test_batch_transaction_atomicity() {
        let db = NamedTempFile::new().unwrap();
        let conn = Connection::open(db.path()).unwrap();
        create_schema(&conn).unwrap();

        // Add unique constraint on email
        conn.execute(
            "CREATE UNIQUE INDEX idx_email ON users(email)",
            [],
        )
        .unwrap();

        let records = vec![
            UserRecord {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@test.com".to_string(),
                age: 30,
                country: "US".to_string(),
            },
            UserRecord {
                id: 2,
                name: "Bob".to_string(),
                email: "alice@test.com".to_string(), // Duplicate email
                age: 25,
                country: "UK".to_string(),
            },
        ];

        let tx = conn.transaction().unwrap();
        let result = insert_batch(&tx, &records);

        // Should fail due to duplicate
        assert!(result.is_err());

        // Don't commit transaction
        drop(tx);

        // No records should be inserted
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
            .unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn test_performance_batch_vs_single() {
        use std::time::Instant;

        let csv_content: String = {
            let mut content = String::from("id,name,email,age,country\n");
            for i in 0..1000 {
                content.push_str(&format!("{},User{},user{}@test.com,{},US\n", i, i, i, 20 + (i % 50)));
            }
            content
        };

        let csv_file = create_test_csv(&csv_content);

        // Batch insert
        let db_batch = NamedTempFile::new().unwrap();
        let start = Instant::now();
        import_csv_to_db(
            csv_file.path().to_str().unwrap(),
            db_batch.path().to_str().unwrap(),
            100, // Batch size
        )
        .unwrap();
        let batch_time = start.elapsed();

        // Single row insert
        let db_single = NamedTempFile::new().unwrap();
        let start = Instant::now();
        import_csv_to_db(
            csv_file.path().to_str().unwrap(),
            db_single.path().to_str().unwrap(),
            1, // Single row
        )
        .unwrap();
        let single_time = start.elapsed();

        println!("Batch insert: {:?}", batch_time);
        println!("Single row insert: {:?}", single_time);

        // Batch should be significantly faster
        assert!(batch_time < single_time);
    }

    #[test]
    fn test_import_with_stats() {
        let csv_content: String = {
            let mut content = String::from("id,name,email,age,country\n");
            for i in 0..500 {
                content.push_str(&format!("{},User{},user{}@test.com,{},US\n", i, i, i, 20 + (i % 50)));
            }
            content
        };

        let csv_file = create_test_csv(&csv_content);
        let db = NamedTempFile::new().unwrap();

        let stats = import_csv_to_db_with_stats(
            csv_file.path().to_str().unwrap(),
            db.path().to_str().unwrap(),
            100,
        )
        .unwrap();

        assert_eq!(stats.records_imported, 500);
        assert_eq!(stats.batches_processed, 5); // 500 / 100
        assert!(stats.duration_ms > 0);
    }
}
```

---

### Milestone 5: In-Place Deduplication with sort + dedup

**Goal**: Remove duplicate records efficiently using sorting and in-place deduplication.

**Why the previous milestone is not enough**: Duplicate records waste storage and cause constraint violations. Naive deduplication using `HashSet` requires O(n) extra memory and is slower for large datasets.

**What's the improvement**: Sort + dedup is in-place (O(1) extra memory) and cache-friendly:
- HashSet approach: O(n) memory, random access (cache misses)
- Sort + dedup: O(1) memory, sequential access (cache hits)

For 1M records:
- HashSet: ~50MB overhead, ~100ms
- Sort + dedup: ~0MB overhead, ~50ms (with unstable sort)

**Optimization focus**: Memory efficiency and speed through in-place algorithms.

**Architecture**:
- Traits: Implement `Eq`, `Ord` for `UserRecord`
- Functions:
  - `deduplicate_chunk(chunk: &mut Vec<UserRecord>)` - In-place dedup
  - `deduplicate_hashset(chunk: &mut Vec<UserRecord>)` - HashSet comparison
  - `benchmark_dedup(records: &mut Vec<UserRecord>)` - Performance comparison

---

**Starter Code**:

```rust
use std::cmp::Ordering;
use std::collections::HashSet;

/// Implement equality based on ID
/// Role: Define uniqueness criterion
impl PartialEq for UserRecord {
    fn eq(&self, other: &Self) -> bool {
        todo!("Compare by ID or email")
    }
}

impl Eq for UserRecord {}

/// Implement ordering based on ID
/// Role: Enable sorting
impl PartialOrd for UserRecord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UserRecord {
    fn cmp(&self, other: &Self) -> Ordering {
        todo!("Compare by ID")
    }
}

/// In-place deduplication using sort
/// Sort and remove consecutive duplicates
/// Role: Memory-efficient deduplication
pub fn deduplicate_chunk(chunk: &mut Vec<UserRecord>) {
    todo!("Sort unstable, then dedup")
}

/// HashSet-based deduplication
/// Use HashSet for uniqueness
/// Role: Comparison baseline
pub fn deduplicate_hashset(chunk: &mut Vec<UserRecord>) {
    todo!("Use HashSet::insert to filter, retain unique")
}

/// Benchmark deduplication strategies
/// Compare performance
/// Role: Measure optimization impact
pub fn benchmark_dedup(records: &mut Vec<UserRecord>) {
    todo!("Clone records, time both approaches, report results")
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedup_removes_duplicates() {
        let mut records = vec![
            UserRecord { id: 1, name: "Alice".to_string(), email: "alice@test.com".to_string(), age: 30, country: "US".to_string() },
            UserRecord { id: 2, name: "Bob".to_string(), email: "bob@test.com".to_string(), age: 25, country: "UK".to_string() },
            UserRecord { id: 1, name: "Alice Duplicate".to_string(), email: "alice2@test.com".to_string(), age: 31, country: "CA".to_string() },
            UserRecord { id: 3, name: "Charlie".to_string(), email: "charlie@test.com".to_string(), age: 35, country: "FR".to_string() },
        ];

        deduplicate_chunk(&mut records);

        assert_eq!(records.len(), 3); // IDs: 1, 2, 3
    }

    #[test]
    fn test_dedup_maintains_order_of_unique() {
        let mut records = vec![
            UserRecord { id: 3, name: "Charlie".to_string(), email: "c@test.com".to_string(), age: 35, country: "FR".to_string() },
            UserRecord { id: 1, name: "Alice".to_string(), email: "a@test.com".to_string(), age: 30, country: "US".to_string() },
            UserRecord { id: 2, name: "Bob".to_string(), email: "b@test.com".to_string(), age: 25, country: "UK".to_string() },
        ];

        deduplicate_chunk(&mut records);

        // After sort and dedup, should be ordered by ID
        assert_eq!(records[0].id, 1);
        assert_eq!(records[1].id, 2);
        assert_eq!(records[2].id, 3);
    }

    #[test]
    fn test_dedup_empty_vec() {
        let mut records: Vec<UserRecord> = vec![];
        deduplicate_chunk(&mut records);
        assert_eq!(records.len(), 0);
    }

    #[test]
    fn test_dedup_no_duplicates() {
        let mut records = vec![
            UserRecord { id: 1, name: "Alice".to_string(), email: "a@test.com".to_string(), age: 30, country: "US".to_string() },
            UserRecord { id: 2, name: "Bob".to_string(), email: "b@test.com".to_string(), age: 25, country: "UK".to_string() },
        ];

        let original_len = records.len();
        deduplicate_chunk(&mut records);

        assert_eq!(records.len(), original_len);
    }

    #[test]
    fn test_dedup_all_duplicates() {
        let mut records = vec![
            UserRecord { id: 1, name: "Alice".to_string(), email: "a@test.com".to_string(), age: 30, country: "US".to_string() },
            UserRecord { id: 1, name: "Alice2".to_string(), email: "a2@test.com".to_string(), age: 31, country: "CA".to_string() },
            UserRecord { id: 1, name: "Alice3".to_string(), email: "a3@test.com".to_string(), age: 32, country: "UK".to_string() },
        ];

        deduplicate_chunk(&mut records);

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, 1);
    }

    #[test]
    fn test_hashset_dedup_correctness() {
        let mut records = vec![
            UserRecord { id: 1, name: "Alice".to_string(), email: "a@test.com".to_string(), age: 30, country: "US".to_string() },
            UserRecord { id: 2, name: "Bob".to_string(), email: "b@test.com".to_string(), age: 25, country: "UK".to_string() },
            UserRecord { id: 1, name: "Alice Duplicate".to_string(), email: "a2@test.com".to_string(), age: 31, country: "CA".to_string() },
        ];

        deduplicate_hashset(&mut records);

        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_dedup_methods_equivalent() {
        let original = vec![
            UserRecord { id: 5, name: "E".to_string(), email: "e@test.com".to_string(), age: 30, country: "US".to_string() },
            UserRecord { id: 2, name: "B".to_string(), email: "b@test.com".to_string(), age: 25, country: "UK".to_string() },
            UserRecord { id: 5, name: "E2".to_string(), email: "e2@test.com".to_string(), age: 31, country: "CA".to_string() },
            UserRecord { id: 3, name: "C".to_string(), email: "c@test.com".to_string(), age: 28, country: "FR".to_string() },
        ];

        let mut records1 = original.clone();
        let mut records2 = original.clone();

        deduplicate_chunk(&mut records1);
        deduplicate_hashset(&mut records2);

        // Both should have same count
        assert_eq!(records1.len(), records2.len());
    }

    #[test]
    fn test_dedup_performance() {
        use std::time::Instant;

        let mut records: Vec<UserRecord> = Vec::new();

        // Create 10K records with 50% duplicates
        for i in 0..5000 {
            records.push(UserRecord {
                id: i,
                name: format!("User{}", i),
                email: format!("user{}@test.com", i),
                age: 20 + (i as u32 % 50),
                country: "US".to_string(),
            });
            // Add duplicate
            records.push(UserRecord {
                id: i,
                name: format!("UserDup{}", i),
                email: format!("dup{}@test.com", i),
                age: 21 + (i as u32 % 50),
                country: "UK".to_string(),
            });
        }

        let mut test1 = records.clone();
        let start = Instant::now();
        deduplicate_chunk(&mut test1);
        let sort_time = start.elapsed();

        let mut test2 = records.clone();
        let start = Instant::now();
        deduplicate_hashset(&mut test2);
        let hash_time = start.elapsed();

        println!("Sort+dedup: {:?}", sort_time);
        println!("HashSet: {:?}", hash_time);

        // Both should produce same unique count
        assert_eq!(test1.len(), test2.len());
    }
}
```

---

### Milestone 6: Parallel Processing with Rayon

**Goal**: Process multiple chunks in parallel for maximum CPU utilization.

**Why the previous milestone is not enough**: Milestones 1-5 are sequential, using only one CPU core. On an 8-core machine, we waste 87.5% of computing power.

**What's the improvement**: Parallel processing provides linear speedup with core count:
- Sequential (1 core): 100 seconds
- Parallel (8 cores): ~13 seconds (8x speedup)

For CPU-bound operations (parsing, validation, transformation), parallelism is nearly free performance. Best approach: read file sequentially into chunks, then process chunks in parallel.

**Optimization focus**: Speed through parallelism—utilizing all CPU cores.

**Architecture**:
- Functions:
  - `process_csv_parallel(path, chunk_size) -> Result<Vec<UserRecord>, Error>` - Parallel processing
  - `benchmark_parallel(path, chunk_size)` - Performance comparison

---

**Starter Code**:

```rust
use rayon::prelude::*;

/// Process CSV chunks in parallel
/// Multi-threaded CSV processing
/// Role: Maximize CPU utilization
pub fn process_csv_parallel(
    path: &str,
    chunk_size: usize,
) -> Result<Vec<UserRecord>, Box<dyn Error>> {
    todo!("Read into chunks, process with par_iter, flatten results")
}

/// Transform record
/// Role: Example CPU-bound operation
pub fn transform_record(record: &mut UserRecord) {
    todo!("Normalize email, uppercase country, etc.")
}

/// Parallel CSV processor with transformations
/// Process + transform
/// Role: Full parallel pipeline
pub fn process_and_transform_parallel(
    path: &str,
    chunk_size: usize,
) -> Result<Vec<UserRecord>, Box<dyn Error>> {
    todo!("Process chunks in parallel, apply transformations, deduplicate per chunk")
}

/// Benchmark sequential vs parallel
/// Performance comparison
/// Role: Measure parallelism benefit
pub fn benchmark_parallel(path: &str, chunk_size: usize) {
    todo!("Time sequential and parallel processing, report speedup")
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_processing_correctness() {
        let csv_content: String = {
            let mut content = String::from("id,name,email,age,country\n");
            for i in 0..1000 {
                content.push_str(&format!("{},User{},user{}@test.com,{},US\n", i, i, i, 20 + (i % 50)));
            }
            content
        };

        let file = create_test_csv(&csv_content);

        let records_seq = parse_csv_optimized(file.path().to_str().unwrap()).unwrap();
        let records_par = process_csv_parallel(file.path().to_str().unwrap(), 100).unwrap();

        assert_eq!(records_seq.len(), records_par.len());
    }

    #[test]
    fn test_parallel_performance() {
        use std::time::Instant;

        let csv_content: String = {
            let mut content = String::from("id,name,email,age,country\n");
            for i in 0..10000 {
                content.push_str(&format!("{},User{},user{}@test.com,{},US\n", i, i, i, 20 + (i % 50)));
            }
            content
        };

        let file = create_test_csv(&csv_content);

        // Sequential
        let start = Instant::now();
        let records_seq = parse_csv_optimized(file.path().to_str().unwrap()).unwrap();
        let seq_time = start.elapsed();

        // Parallel
        let start = Instant::now();
        let records_par = process_csv_parallel(file.path().to_str().unwrap(), 1000).unwrap();
        let par_time = start.elapsed();

        println!("Sequential: {:?}", seq_time);
        println!("Parallel: {:?}", par_time);

        assert_eq!(records_seq.len(), records_par.len());

        // Parallel should be faster for large datasets
        // (May not always be true for small datasets due to overhead)
    }

    #[test]
    fn test_parallel_with_transformations() {
        let csv_content = "\
id,name,email,age,country
1,Alice,ALICE@TEST.COM,30,us
2,Bob,BOB@TEST.COM,25,uk
3,Charlie,CHARLIE@TEST.COM,35,ca";

        let file = create_test_csv(csv_content);

        let records = process_and_transform_parallel(file.path().to_str().unwrap(), 2).unwrap();

        // Verify transformations applied
        for record in &records {
            // Email should be lowercase
            assert_eq!(record.email, record.email.to_lowercase());
            // Country should be uppercase
            assert_eq!(record.country, record.country.to_uppercase());
        }
    }

    #[test]
    fn test_parallel_deduplication() {
        let csv_content: String = {
            let mut content = String::from("id,name,email,age,country\n");
            for i in 0..100 {
                // Add each record twice
                content.push_str(&format!("{},User{},user{}@test.com,{},US\n", i, i, i, 20 + (i % 50)));
                content.push_str(&format!("{},UserDup{},userdup{}@test.com,{},UK\n", i, i, i, 21 + (i % 50)));
            }
            content
        };

        let file = create_test_csv(&csv_content);

        let records = process_and_transform_parallel(file.path().to_str().unwrap(), 50).unwrap();

        // Should have deduplicated (100 unique IDs)
        assert_eq!(records.len(), 100);
    }

    #[test]
    fn test_parallel_chunk_independence() {
        // Verify chunks process independently
        let csv_content: String = {
            let mut content = String::from("id,name,email,age,country\n");
            for i in 0..100 {
                content.push_str(&format!("{},User{},user{}@test.com,{},US\n", i, i, i, 20 + (i % 50)));
            }
            content
        };

        let file = create_test_csv(&csv_content);

        let records = process_csv_parallel(file.path().to_str().unwrap(), 10).unwrap();

        // All records should be present
        assert_eq!(records.len(), 100);

        // Verify all IDs present
        let mut ids: Vec<u64> = records.iter().map(|r| r.id).collect();
        ids.sort_unstable();

        for (i, &id) in ids.iter().enumerate() {
            assert_eq!(id, i as u64);
        }
    }
}
```

---

### Testing Strategies

1. **Unit Tests**: Test parsing, validation, deduplication independently
2. **Integration Tests**: End-to-end with test CSV files
3. **Performance Tests**: Benchmark each optimization milestone
4. **Memory Tests**: Monitor memory usage with large files using profilers
5. **Correctness Tests**: Verify no data loss during processing
6. **Stress Tests**: Process 10M+ row files
7. **Comparison Tests**: Compare optimized vs naive implementations

---

### Complete Working Example

```rust
// See individual milestones above for complete implementations
// This demonstrates the full pipeline:

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== CSV Batch Processor ===\n");

    let input_path = "large_dataset.csv";
    let db_path = "output.db";

    // Complete pipeline:
    // 1. Process CSV in chunks (memory efficient)
    // 2. Transform and validate records
    // 3. Deduplicate within chunks
    // 4. Batch insert to database

    process_csv_chunked(input_path, 10000, |chunk| {
        let mut chunk = chunk.to_vec();

        // Transform
        for record in &mut chunk {
            transform_record(record);
        }

        // Deduplicate
        deduplicate_chunk(&mut chunk);

        // Would normally insert to database here
        println!("Processed chunk of {} unique records", chunk.len());
    })?;

    Ok(())
}
```

This project demonstrates all key Vec optimization patterns:
- **Capacity pre-allocation** (10-50x speedup)
- **Chunked processing** (constant memory for any file size)
- **In-place algorithms** (zero extra memory for dedup)
- **Batch operations** (100x speedup for database inserts)
- **Parallel processing** (8x speedup on 8 cores)
