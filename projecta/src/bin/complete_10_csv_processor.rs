use csv::{Reader, ReaderBuilder, StringRecord};
use rayon::prelude::*;
use rusqlite::{Connection, Transaction};
use std::{
    cmp::Ordering,
    collections::HashSet,
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
    time::Instant,
};

// =============================================================================
// Milestone 1: Basic CSV Parser with Structured Records
// =============================================================================

/// CSV record representing a user
#[derive(Debug, Clone)]
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
        if row.len() < 5 {
            return Err(ParseError::InvalidFormat(format!("{:?}", row)));
        }

        let id = row
            .get(0)
            .ok_or_else(|| ParseError::MissingField("id".into()))?
            .parse()
            .map_err(|value: std::num::ParseIntError| ParseError::InvalidType {
                field: "id".into(),
                value: value.to_string(),
            })?;

        let age = row
            .get(3)
            .ok_or_else(|| ParseError::MissingField("age".into()))?
            .parse()
            .map_err(|value: std::num::ParseIntError| ParseError::InvalidType {
                field: "age".into(),
                value: value.to_string(),
            })?;

        Ok(UserRecord {
            id,
            name: row
                .get(1)
                .ok_or_else(|| ParseError::MissingField("name".into()))?
                .to_string(),
            email: row
                .get(2)
                .ok_or_else(|| ParseError::MissingField("email".into()))?
                .to_string(),
            age,
            country: row
                .get(4)
                .ok_or_else(|| ParseError::MissingField("country".into()))?
                .to_string(),
        })
    }
}

/// Parse entire CSV file
/// Role: Read file and convert all valid rows
pub fn parse_csv(path: &str) -> Result<Vec<UserRecord>, Box<dyn Error>> {
    let mut reader = Reader::from_path(path)?;
    let mut records = Vec::new();

    for result in reader.records() {
        let record = result?;
        if let Ok(user) = UserRecord::from_csv_row(&record) {
            records.push(user);
        }
    }

    Ok(records)
}

// =============================================================================
// Milestone 2: Pre-Allocate Capacity to Eliminate Reallocations
// =============================================================================

/// Count lines in file
/// Role: Estimate capacity needed for Vec
pub fn count_lines(path: &str) -> Result<usize, io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().count())
}

/// Parse CSV with pre-allocated capacity
/// Role: Eliminate reallocations during parsing
pub fn parse_csv_optimized(path: &str) -> Result<Vec<UserRecord>, Box<dyn Error>> {
    let total_lines = count_lines(path)?;
    let mut reader = Reader::from_path(path)?;
    let mut records = Vec::with_capacity(total_lines.saturating_sub(1));

    for result in reader.records() {
        let record = result?;
        if let Ok(user) = UserRecord::from_csv_row(&record) {
            records.push(user);
        }
    }

    Ok(records)
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
        Self {
            vec: Vec::with_capacity(capacity),
            stats: AllocationStats {
                allocations: 1,
                ..Default::default()
            },
        }
    }

    /// Push with reallocation tracking
    /// Role: Monitor when reallocations occur
    pub fn push(&mut self, value: T) {
        if self.vec.len() == self.vec.capacity() {
            self.stats.reallocations += 1;
            self.stats.bytes_copied += self.vec.len() * std::mem::size_of::<T>();
        }
        self.vec.push(value);
    }

    /// Get statistics
    /// Role: Query allocation metrics
    pub fn stats(&self) -> &AllocationStats {
        &self.stats
    }
}

// =============================================================================
// Milestone 3: Streaming Processing with Chunking
// =============================================================================

/// Process CSV in chunks with callback
/// Role: Enable processing files larger than RAM
pub fn process_csv_chunked<F>(
    path: &str,
    chunk_size: usize,
    mut process_chunk: F,
) -> Result<(), Box<dyn Error>>
where
    F: FnMut(&[UserRecord]),
{
    let mut reader = ReaderBuilder::new().has_headers(true).from_path(path)?;
    let mut chunk = Vec::with_capacity(chunk_size);

    for result in reader.records() {
        let record = result?;
        if let Ok(user) = UserRecord::from_csv_row(&record) {
            chunk.push(user);
        }

        if chunk.len() == chunk_size {
            process_chunk(&chunk);
            chunk.clear();
        }
    }

    if !chunk.is_empty() {
        process_chunk(&chunk);
    }

    Ok(())
}

/// Statistics for chunked processing
#[derive(Debug, Default)]
pub struct ChunkStats {
    pub total_chunks: usize,
    pub total_records: usize,
    pub peak_memory_bytes: usize,
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
    let mut stats = ChunkStats::default();
    process_csv_chunked(path, chunk_size, |chunk| {
        stats.total_chunks += 1;
        stats.total_records += chunk.len();
        stats.peak_memory_bytes = stats
            .peak_memory_bytes
            .max(chunk.len() * std::mem::size_of::<UserRecord>());
        process_chunk(chunk);
    })?;
    Ok(stats)
}

// =============================================================================
// Milestone 4: Batch Database Inserts with Transactions
// =============================================================================

/// Insert batch of records in single query
/// Multi-row INSERT
/// Role: Minimize database round-trips
pub fn insert_batch(tx: &Transaction, records: &[UserRecord]) -> Result<(), rusqlite::Error> {
    if records.is_empty() {
        return Ok(());
    }

    let mut sql = String::from("INSERT INTO users (id, name, email, age, country) VALUES ");
    let mut params_vec: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(records.len() * 5);

    for (idx, record) in records.iter().enumerate() {
        if idx > 0 {
            sql.push_str(", ");
        }
        sql.push_str("(?, ?, ?, ?, ?)");
        params_vec.push(&record.id);
        params_vec.push(&record.name);
        params_vec.push(&record.email);
        params_vec.push(&record.age);
        params_vec.push(&record.country);
    }

    let mut stmt = tx.prepare(&sql)?;
    stmt.execute(params_vec.as_slice())?;
    Ok(())
}

/// Create database schema
/// Role: Initialize tables
pub fn create_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
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
    Ok(())
}

/// Import CSV to database with batching
/// Role: Production-ready CSV import
pub fn import_csv_to_db(
    path: &str,
    db_path: &str,
    batch_size: usize,
) -> Result<(), Box<dyn Error>> {
    let mut conn = Connection::open(db_path)?;
    create_schema(&conn)?;

    process_csv_chunked(path, batch_size, |chunk| {
        let tx = conn.transaction().unwrap();
        insert_batch(&tx, chunk).unwrap();
        tx.commit().unwrap();
    })?;

    Ok(())
}

/// Database import statistics
#[derive(Debug, Default)]
pub struct ImportStats {
    pub records_imported: usize,
    pub records_failed: usize,
    pub batches_processed: usize,
    pub duration_ms: u64,
}

/// Import with detailed statistics
/// Role: Monitor import performance
pub fn import_csv_to_db_with_stats(
    path: &str,
    db_path: &str,
    batch_size: usize,
) -> Result<ImportStats, Box<dyn Error>> {
    let mut conn = Connection::open(db_path)?;
    create_schema(&conn)?;

    let start = Instant::now();
    let mut stats = ImportStats::default();

    process_csv_chunked(path, batch_size, |chunk| {
        let tx = conn.transaction().unwrap();
        match insert_batch(&tx, chunk) {
            Ok(_) => {
                tx.commit().unwrap();
                stats.records_imported += chunk.len();
                stats.batches_processed += 1;
            }
            Err(_) => stats.records_failed += chunk.len(),
        }
    })?;

    stats.duration_ms = start.elapsed().as_millis() as u64;
    Ok(stats)
}

// =============================================================================
// Milestone 5: In-Place Deduplication with sort + dedup
// =============================================================================

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

/// In-place deduplication using sort
/// Role: Memory-efficient deduplication
pub fn deduplicate_chunk(chunk: &mut Vec<UserRecord>) {
    chunk.sort_unstable_by(|a, b| a.id.cmp(&b.id));
    chunk.dedup_by(|a, b| a.id == b.id);
}

/// HashSet-based deduplication
/// Role: Comparison baseline
pub fn deduplicate_hashset(chunk: &mut Vec<UserRecord>) {
    let mut seen = HashSet::new();
    chunk.retain(|record| seen.insert(record.id));
}

/// Benchmark deduplication strategies
/// Role: Measure optimization impact
pub fn benchmark_dedup(records: &mut Vec<UserRecord>) {
    let mut chunk_sort = records.clone();
    let start = Instant::now();
    deduplicate_chunk(&mut chunk_sort);
    let sort_time = start.elapsed();

    let mut chunk_hash = records.clone();
    let start = Instant::now();
    deduplicate_hashset(&mut chunk_hash);
    let hash_time = start.elapsed();

    println!("Sort+dedup: {:?}", sort_time);
    println!("HashSet dedup: {:?}", hash_time);
}

// =============================================================================
// Milestone 6: Parallel Processing with Rayon
// =============================================================================

/// Process CSV chunks in parallel
/// Role: Maximize CPU utilization
pub fn process_csv_parallel(
    path: &str,
    chunk_size: usize,
) -> Result<Vec<UserRecord>, Box<dyn Error>> {
    let mut chunks = Vec::new();
    process_csv_chunked(path, chunk_size, |chunk| {
        chunks.push(chunk.to_vec());
    })?;

    Ok(chunks.into_par_iter().flatten().collect())
}

/// Transform record
/// Role: Example CPU-bound operation
pub fn transform_record(record: &mut UserRecord) {
    record.email = record.email.to_lowercase();
    record.country = record.country.to_uppercase();
}

/// Parallel CSV processor with transformations
/// Role: Full parallel pipeline
pub fn process_and_transform_parallel(
    path: &str,
    chunk_size: usize,
) -> Result<Vec<UserRecord>, Box<dyn Error>> {
    let mut chunks = Vec::new();
    process_csv_chunked(path, chunk_size, |chunk| {
        chunks.push(chunk.to_vec());
    })?;

    let mut records: Vec<UserRecord> = chunks
        .into_par_iter()
        .flat_map(|mut chunk| {
            chunk.par_iter_mut().for_each(transform_record);
            deduplicate_chunk(&mut chunk);
            chunk
        })
        .collect();

    deduplicate_chunk(&mut records);
    Ok(records)
}

/// Benchmark sequential vs parallel
/// Role: Measure parallelism benefit
pub fn benchmark_parallel(path: &str, chunk_size: usize) {
    let start = Instant::now();
    let _ = parse_csv_optimized(path).unwrap();
    let seq_time = start.elapsed();

    let start = Instant::now();
    let _ = process_csv_parallel(path, chunk_size).unwrap();
    let par_time = start.elapsed();

    println!("Sequential: {:?}", seq_time);
    println!("Parallel: {:?}", par_time);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::{Arc, Mutex};
    use tempfile::NamedTempFile;

    fn create_test_csv(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    // ----- Milestone 1 tests -----
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

        assert!(matches!(
            result.unwrap_err(),
            ParseError::InvalidType { field, .. } if field == "age"
        ));
    }

    #[test]
    fn test_parse_missing_field() {
        let row = StringRecord::from(vec!["1", "Alice", "alice@test.com"]);
        let result = UserRecord::from_csv_row(&row);
        assert!(matches!(result.unwrap_err(), ParseError::InvalidFormat(_)));
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

    // ----- Milestone 2 tests -----
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
        assert!(records.capacity() >= records.len());
    }

    #[test]
    fn test_tracked_vec_no_reallocations() {
        let mut vec = TrackedVec::with_capacity(10);
        for i in 0..10 {
            vec.push(i);
        }
        let stats = vec.stats();
        assert_eq!(stats.allocations, 1);
        assert_eq!(stats.reallocations, 0);
    }

    #[test]
    fn test_tracked_vec_with_reallocations() {
        let mut vec = TrackedVec::with_capacity(4);
        for i in 0..8 {
            vec.push(i);
        }
        let stats = vec.stats();
        assert!(stats.reallocations > 0);
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
        assert!(records.capacity() < records.len() * 2);
    }

    // ----- Milestone 3 tests -----
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

        let chunks = Arc::new(Mutex::new(0));
        let total = Arc::new(Mutex::new(0));

        let chunks_clone = chunks.clone();
        let total_clone = total.clone();

        process_csv_chunked(file.path().to_str().unwrap(), 2, |chunk| {
            *chunks_clone.lock().unwrap() += 1;
            *total_clone.lock().unwrap() += chunk.len();
        })
        .unwrap();

        assert_eq!(*chunks.lock().unwrap(), 3);
        assert_eq!(*total.lock().unwrap(), 5);
    }

    #[test]
    fn test_chunk_buffer_reuse() {
        let mut content = String::from("id,name,email,age,country\n");
        for i in 0..100 {
            content.push_str(&format!("{},User{},user{}@test.com,{},US\n", i, i, i, 20));
        }

        let file = create_test_csv(&content);
        let sizes = Arc::new(Mutex::new(Vec::new()));
        let sizes_clone = sizes.clone();

        process_csv_chunked(file.path().to_str().unwrap(), 10, |chunk| {
            sizes_clone.lock().unwrap().push(chunk.len());
        })
        .unwrap();

        let locked = sizes.lock().unwrap();
        for &size in locked.iter().take(locked.len() - 1) {
            assert_eq!(size, 10);
        }
        assert!(*locked.last().unwrap() <= 10);
    }

    #[test]
    fn test_process_empty_file_chunked() {
        let file = create_test_csv("id,name,email,age,country\n");
        let called = Arc::new(Mutex::new(false));
        let clone = called.clone();
        process_csv_chunked(file.path().to_str().unwrap(), 10, |_| {
            *clone.lock().unwrap() = true;
        })
        .unwrap();
        assert!(!*called.lock().unwrap());
    }

    #[test]
    fn test_chunked_with_stats() {
        let mut content = String::from("id,name,email,age,country\n");
        for i in 0..50 {
            content.push_str(&format!("{},User{},user{}@test.com,{},US\n", i, i, i, 20));
        }

        let file = create_test_csv(&content);
        let stats =
            process_csv_chunked_with_stats(file.path().to_str().unwrap(), 10, |_| {}).unwrap();

        assert_eq!(stats.total_chunks, 5);
        assert_eq!(stats.total_records, 50);
        assert!(stats.peak_memory_bytes > 0);
    }

    // ----- Milestone 4 tests -----
    #[test]
    fn test_create_schema() {
        let db = NamedTempFile::new().unwrap();
        let conn = Connection::open(db.path()).unwrap();
        create_schema(&conn).unwrap();

        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='users'")
            .unwrap();
        assert!(stmt.exists([]).unwrap());
    }

    #[test]
    fn test_insert_single_batch() {
        let db = NamedTempFile::new().unwrap();
        let mut conn = Connection::open(db.path()).unwrap();
        create_schema(&conn).unwrap();

        let records = vec![
            UserRecord {
                id: 1,
                name: "Alice".into(),
                email: "alice@test.com".into(),
                age: 30,
                country: "US".into(),
            },
            UserRecord {
                id: 2,
                name: "Bob".into(),
                email: "bob@test.com".into(),
                age: 25,
                country: "UK".into(),
            },
        ];

        let tx = conn.transaction().unwrap();
        insert_batch(&tx, &records).unwrap();
        tx.commit().unwrap();

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
2,Bob,bob@test.com,25,UK";

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
        assert_eq!(count, 2);
    }

    #[test]
    fn test_batch_transaction_atomicity() {
        let db = NamedTempFile::new().unwrap();
        let mut conn = Connection::open(db.path()).unwrap();
        create_schema(&conn).unwrap();
        conn.execute("CREATE UNIQUE INDEX idx_email ON users(email)", [])
            .unwrap();

        let records = vec![
            UserRecord {
                id: 1,
                name: "Alice".into(),
                email: "alice@test.com".into(),
                age: 30,
                country: "US".into(),
            },
            UserRecord {
                id: 2,
                name: "Bob".into(),
                email: "alice@test.com".into(),
                age: 25,
                country: "UK".into(),
            },
        ];

        let tx = conn.transaction().unwrap();
        assert!(insert_batch(&tx, &records).is_err());
        drop(tx);

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_import_with_stats() {
        let mut content = String::from("id,name,email,age,country\n");
        for i in 0..50 {
            content.push_str(&format!("{},User{},user{}@test.com,{},US\n", i, i, i, 20));
        }
        let csv_file = create_test_csv(&content);
        let db = NamedTempFile::new().unwrap();

        let stats = import_csv_to_db_with_stats(
            csv_file.path().to_str().unwrap(),
            db.path().to_str().unwrap(),
            10,
        )
        .unwrap();

        assert_eq!(stats.records_imported, 50);
        assert_eq!(stats.batches_processed, 5);
        assert!(stats.duration_ms > 0);
    }

    // ----- Milestone 5 tests -----
    #[test]
    fn test_dedup_removes_duplicates() {
        let mut records = vec![
            UserRecord {
                id: 1,
                name: "A".into(),
                email: "a@test.com".into(),
                age: 30,
                country: "US".into(),
            },
            UserRecord {
                id: 2,
                name: "B".into(),
                email: "b@test.com".into(),
                age: 25,
                country: "UK".into(),
            },
            UserRecord {
                id: 1,
                name: "A2".into(),
                email: "a2@test.com".into(),
                age: 31,
                country: "CA".into(),
            },
        ];
        deduplicate_chunk(&mut records);
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_dedup_ordering() {
        let mut records = vec![
            UserRecord {
                id: 3,
                name: "C".into(),
                email: "c@test.com".into(),
                age: 30,
                country: "US".into(),
            },
            UserRecord {
                id: 1,
                name: "A".into(),
                email: "a@test.com".into(),
                age: 25,
                country: "UK".into(),
            },
            UserRecord {
                id: 2,
                name: "B".into(),
                email: "b@test.com".into(),
                age: 25,
                country: "UK".into(),
            },
        ];
        deduplicate_chunk(&mut records);
        assert_eq!(
            records.iter().map(|r| r.id).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn test_hashset_dedup_correctness() {
        let mut records = vec![
            UserRecord {
                id: 1,
                name: "A".into(),
                email: "a@test.com".into(),
                age: 30,
                country: "US".into(),
            },
            UserRecord {
                id: 1,
                name: "A2".into(),
                email: "a2@test.com".into(),
                age: 31,
                country: "CA".into(),
            },
            UserRecord {
                id: 2,
                name: "B".into(),
                email: "b@test.com".into(),
                age: 25,
                country: "UK".into(),
            },
        ];
        deduplicate_hashset(&mut records);
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_dedup_methods_equivalent() {
        let original = vec![
            UserRecord {
                id: 3,
                name: "C".into(),
                email: "c@test.com".into(),
                age: 30,
                country: "US".into(),
            },
            UserRecord {
                id: 1,
                name: "A".into(),
                email: "a@test.com".into(),
                age: 25,
                country: "UK".into(),
            },
            UserRecord {
                id: 3,
                name: "C2".into(),
                email: "c2@test.com".into(),
                age: 32,
                country: "CA".into(),
            },
        ];

        let mut sort = original.clone();
        let mut hash = original.clone();
        deduplicate_chunk(&mut sort);
        deduplicate_hashset(&mut hash);
        assert_eq!(sort.len(), hash.len());
    }

    // ----- Milestone 6 tests -----
    #[test]
    fn test_parallel_processing_correctness() {
        let mut content = String::from("id,name,email,age,country\n");
        for i in 0..100 {
            content.push_str(&format!("{},User{},user{}@test.com,{},US\n", i, i, i, 20));
        }
        let file = create_test_csv(&content);

        let seq = parse_csv_optimized(file.path().to_str().unwrap()).unwrap();
        let par = process_csv_parallel(file.path().to_str().unwrap(), 10).unwrap();
        assert_eq!(seq.len(), par.len());
    }

    #[test]
    fn test_parallel_with_transformations() {
        let csv_content = "\
id,name,email,age,country
1,Alice,ALICE@TEST.COM,30,us
2,Bob,BOB@TEST.COM,25,uk";
        let file = create_test_csv(csv_content);

        let records = process_and_transform_parallel(file.path().to_str().unwrap(), 1).unwrap();
        for record in records {
            assert_eq!(record.email, record.email.to_lowercase());
            assert_eq!(record.country, record.country.to_uppercase());
        }
    }

    #[test]
    fn test_parallel_deduplication() {
        let mut content = String::from("id,name,email,age,country\n");
        for i in 0..50 {
            content.push_str(&format!("{},User{},user{}@test.com,{},US\n", i, i, i, 20));
            content.push_str(&format!("{},UserDup{},dup{}@test.com,{},US\n", i, i, i, 21));
        }
        let file = create_test_csv(&content);

        let records = process_and_transform_parallel(file.path().to_str().unwrap(), 10).unwrap();
        assert_eq!(records.len(), 50);
    }

    #[test]
    fn test_parallel_chunk_independence() {
        let mut content = String::from("id,name,email,age,country\n");
        for i in 0..20 {
            content.push_str(&format!("{},User{},user{}@test.com,{},US\n", i, i, i, 20));
        }
        let file = create_test_csv(&content);

        let records = process_csv_parallel(file.path().to_str().unwrap(), 5).unwrap();
        assert_eq!(records.len(), 20);
    }
}

fn main() {}
