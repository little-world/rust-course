# Project 1: Zero-Copy Log Parser and Analyzer

## Problem Statement

Build a high-performance log parser that processes large log files using zero-copy string operations. The parser should extract fields from log lines without allocating intermediate strings, use `Cow<str>` for conditional modifications, and achieve maximum throughput through efficient string slicing.

Your parser should:
- Parse structured logs (e.g., Apache/nginx format, JSON logs)
- Extract fields (timestamp, level, service, message) as `&str` slices
- Filter logs by criteria without copying strings
- Transform fields only when necessary (using `Cow<str>`)
- Build search index for fast queries
- Support multiple log formats

Example log formats:
```
Apache: 127.0.0.1 - - [10/Oct/2024:13:55:36 -0700] "GET /index.html HTTP/1.1" 200 2326
JSON: {"timestamp":"2024-10-10T13:55:36Z","level":"ERROR","service":"auth","message":"Login failed"}
Syslog: Oct 10 13:55:36 hostname service[1234]: Error message here
```

## Why It Matters

String processing is often a bottleneck in data pipelines. Naive approaches allocate strings for every operation, causing excessive memory usage and garbage collection pressure. Zero-copy techniques eliminate allocations by working with string slices (`&str`), providing 10-100x speedup for parsing-heavy workloads.

`Cow<str>` enables "modify only if needed" pattern: if no transformation required, return borrowed data; otherwise allocate only when necessary. This is crucial for high-throughput systems (log processors, web servers, parsers).

## Use Cases

- Log aggregation and analysis (Elasticsearch, Splunk-style systems)
- Web server access log processing
- Security log analysis (SIEM systems)
- Application monitoring and debugging
- Log-based metrics extraction
- Compliance and audit log processing

---

## Introduction to String Processing Concepts

String processing is fundamental to systems programming, yet naive approaches can cripple performance. Understanding Rust's string model—with its distinction between owned `String` and borrowed `&str`, zero-copy slicing, and smart allocation strategies—is essential for building high-performance parsers and data processors.

### 1. String vs &str: Owned vs Borrowed

Rust distinguishes between owned and borrowed string data with different types:

**`String` - Owned, Heap-Allocated**:
```rust
let s = String::from("hello");  // Heap allocation
let s2 = s.clone();             // Another heap allocation
```
- **Owns** its data on the heap
- **Growable** and **mutable**
- **Allocated** memory that must be freed

**`&str` - Borrowed, Zero-Copy Slice**:
```rust
let s = "hello";           // String literal (&str)
let slice = &s[0..3];      // Slice into existing data (no allocation!)
```
- **Borrows** data from somewhere else (String, literal, file buffer)
- **Immutable** view into string data
- **Zero-cost** - no allocation, just pointer + length

**Key Insight**: Most parsing operations should work with `&str` slices to avoid allocations.

### 2. Zero-Copy String Slicing

String slicing creates views into existing data without copying:

**Naive Approach (Copies)**:
```rust
let line = String::from("timestamp=2024-10-10 level=ERROR message=failed");

// Allocates 3 new strings!
let timestamp = line.split('=').nth(1).unwrap().to_string();
let level = line.split('=').nth(3).unwrap().to_string();
let message = line.split('=').nth(5).unwrap().to_string();
```

**Zero-Copy Approach (Slices)**:
```rust
let line = "timestamp=2024-10-10 level=ERROR message=failed";

// No allocations - just slices into 'line'
let parts: Vec<&str> = line.split(' ').collect();
let timestamp = parts[0].split('=').nth(1).unwrap();  // &str slice
let level = parts[1].split('=').nth(1).unwrap();      // &str slice
let message = parts[2].split('=').nth(1).unwrap();    // &str slice
```

**Performance Impact**:
- Naive: 3 allocations + 3 memory copies
- Zero-copy: 0 allocations + 0 copies
- For 1M log lines: 3M allocations vs 0 (1000x memory reduction!)

### 3. Lifetimes for Borrowed Data

When storing `&str` slices in structs, lifetimes track where the data comes from:

**Without Lifetime Annotation**:
```rust
struct LogEntry {
    level: &str,  // ❌ Won't compile - how long does this reference live?
}
```

**With Lifetime Annotation**:
```rust
struct LogEntry<'a> {
    level: &'a str,  // ✅ Borrows from data that lives at least as long as 'a
}

fn parse(line: &str) -> LogEntry<'_> {
    LogEntry { level: &line[0..5] }  // Slice borrows from 'line'
}
```

**Lifetime Rules**:
- `&'a str` means "borrowed for lifetime 'a"
- Output lifetime tied to input lifetime: `fn parse<'a>(line: &'a str) -> LogEntry<'a>`
- Compiler ensures slices don't outlive the data they point to

**Why This Matters**: Prevents use-after-free bugs at compile time. You can't accidentally return a slice into a dropped String.

### 4. Cow<str> for Conditional Allocation

`Cow` (Clone on Write) enables "modify only if needed" pattern:

**The Problem**:
```rust
// Always allocates, even if already lowercase!
let normalized = service.to_lowercase();
```

**Cow Solution**:
```rust
use std::borrow::Cow;

fn normalize(s: &str) -> Cow<str> {
    if s.chars().all(|c| !c.is_uppercase()) {
        Cow::Borrowed(s)  // No allocation - return original
    } else {
        Cow::Owned(s.to_lowercase())  // Allocate only when needed
    }
}

// 90% already lowercase → 90% less allocations!
let services = vec!["auth", "web", "DB", "api", "Cache"];
let normalized: Vec<Cow<str>> = services.iter().map(|s| normalize(s)).collect();
// Only "DB" and "Cache" allocated new strings
```

**Performance Impact**: For data that's mostly already clean, Cow reduces allocations by 90%+.

### 5. String Methods and Zero-Copy Operations

Many string operations return `&str` slices without allocating:

**Zero-Copy Methods** (return `&str`):
```rust
let s = "  hello world  ";
let trimmed = s.trim();              // &str - no allocation
let slice = &s[2..7];                // &str - slice
let prefix = s.strip_prefix("  ");  // Option<&str>
```

**Allocating Methods** (return `String`):
```rust
let s = "hello";
let owned = s.to_string();       // String - allocates
let uppercase = s.to_uppercase(); // String - allocates
let replaced = s.replace("h", "j"); // String - allocates
```

**Split Returns Iterator** (lazy, zero-copy):
```rust
let line = "a,b,c,d";
let parts = line.split(',');  // Iterator<Item = &str>, no allocation yet!
// Allocates only when collected:
let vec: Vec<&str> = parts.collect();
```

**Strategy**: Use zero-copy methods wherever possible, allocate only when necessary.

### 6. String Builder Pattern for Efficient Concatenation

Concatenating strings with `+` or `format!` is inefficient:

**Naive Concatenation** (Multiple Allocations):
```rust
let mut s = String::new();
for i in 0..1000 {
    s = s + &i.to_string();  // Reallocates on EVERY iteration!
    s = s + ",";
}
// ~2000 allocations!
```

**Builder Pattern** (Single Allocation):
```rust
let mut s = String::with_capacity(5000);  // Pre-allocate
for i in 0..1000 {
    s.push_str(&i.to_string());
    s.push(',');
}
// 1 allocation!
```

**Performance**: For building long strings, pre-allocation eliminates reallocations (100x faster for large strings).

### 7. String Interning for Deduplication

String interning stores each unique string once, using indices to refer to it:

**Without Interning** (Duplicates):
```rust
// 1M log entries, 10 unique service names
let services = vec!["auth"; 1_000_000];  // 1M copies of "auth"
// Memory: ~4MB
```

**With Interning** (Deduplicated):
```rust
struct Interner {
    strings: Vec<String>,
    map: HashMap<String, usize>,
}

// Store "auth" once, use index (4 bytes) for each log
let service_id: usize = interner.intern("auth");  // Returns 0
// Memory: 4 bytes + 1M × 4 bytes = ~4MB (vs 4MB for strings)
// But for 10 services: 40 bytes + 4MB indices vs 40MB strings!
```

**Savings**: For categorical data with many duplicates (log levels, service names, countries), interning reduces memory 100-1000x.

### 8. UTF-8 Encoding and String Validation

Rust strings are **always valid UTF-8**, preventing encoding bugs:

**UTF-8 Basics**:
- **1 byte**: ASCII (0-127)
- **2 bytes**: Latin, Greek, Cyrillic, etc.
- **3 bytes**: Most of Unicode (Chinese, Japanese, etc.)
- **4 bytes**: Emoji, rare characters

**String Indexing** (Not by Byte!):
```rust
let s = "Hello, 世界!";
let slice = &s[7..13];  // ✅ Gets "世界" (6 bytes: 3 bytes each)
// let c = s[7];        // ❌ Won't compile - can't index by byte
```

**Why This Matters**: You can't slice at arbitrary byte positions—must slice at character boundaries. Rust prevents invalid UTF-8:

```rust
let bytes = vec![0xFF, 0xFF];
let s = String::from_utf8(bytes);  // Result::Err - invalid UTF-8
```

**Conversion from Bytes**:
```rust
// Safe (validates)
let s = String::from_utf8(bytes)?;

// Unsafe (assumes valid)
let s = unsafe { String::from_utf8_unchecked(bytes) };
```

### 9. Iterator Chains for Lazy String Processing

Iterator chains enable processing strings without intermediate allocations:

**Eager Evaluation** (Multiple Allocations):
```rust
let lines = vec!["ERROR: failed", "INFO: ok", "ERROR: timeout"];

// Step 1: filter (allocates Vec)
let errors: Vec<_> = lines.iter().filter(|s| s.contains("ERROR")).collect();

// Step 2: extract message (allocates Vec again)
let messages: Vec<_> = errors.iter().map(|s| &s[7..]).collect();

// 2 intermediate Vec allocations
```

**Lazy Evaluation** (Zero Intermediate Allocations):
```rust
let messages: Vec<_> = lines
    .iter()
    .filter(|s| s.contains("ERROR"))  // Iterator adapter (lazy)
    .map(|s| &s[7..])                 // Iterator adapter (lazy)
    .collect();                       // Single allocation at end

// 1 final Vec allocation, 0 intermediate
```

**Key Insight**: Iterator adapters (`filter`, `map`, `filter_map`) are lazy—they don't allocate until `collect()`.

### 10. Parallel String Processing with Rayon

Rayon enables data-parallel string processing with minimal code changes:

**Sequential Processing**:
```rust
let results: Vec<_> = lines
    .iter()
    .filter_map(|line| parse_log(line))
    .collect();
// Uses 1 core
```

**Parallel Processing**:
```rust
use rayon::prelude::*;

let results: Vec<_> = lines
    .par_iter()              // par_iter instead of iter
    .filter_map(|line| parse_log(line))
    .collect();
// Uses all cores automatically!
```

**Performance**: For CPU-bound parsing, near-linear speedup with core count (8 cores ≈ 7x faster).

**Caution**: Thread overhead means parallel only helps for non-trivial work per item (parsing log lines ✅, simple splits ❌).

### Connection to This Project

This log parser project applies every string concept in a realistic, performance-critical scenario:

**Zero-Copy Slicing (Step 1)**: `LogEntry<'a>` stores `&str` slices instead of `String`, eliminating allocations during parsing. For 1M log lines, this saves 1M+ allocations.

**Lifetimes (Step 1)**: The `'a` lifetime ties `LogEntry` fields to the input line—compiler ensures slices don't outlive the data they point to, preventing use-after-free.

**Iterator Chains (Step 2)**: Filtering logs uses chained iterators (`filter_map` → `filter` → `filter`) that process lazily. No intermediate Vec allocations—single pass from file to final results.

**Cow<str> (Step 3)**: Normalization returns `Cow::Borrowed` when no changes needed (90% of logs), `Cow::Owned` only when modifying (10%). This reduces allocations by 9x.

**String Builder (Step 4)**: Formatting output with `String::with_capacity()` + `push_str()` eliminates reallocations. For 1M formatted lines, this is 10x faster than `format!` macro.

**String Interning (Step 5)**: Deduplicating service names and log levels reduces memory from ~20MB to ~200 bytes for categorical fields. Essential for in-memory indexing of millions of logs.

**UTF-8 Handling (All Steps)**: Rust's UTF-8 validation ensures log messages with international characters (emojis, Chinese, etc.) are handled correctly without corruption.

**Lazy Evaluation (Step 2)**: Reading files line-by-line with iterator chains means constant memory usage regardless of file size. Can process GB files with KB of memory.

**Parallel Processing (Step 6)**: Rayon's `par_chunks` parallelizes parsing across CPU cores. On an 8-core machine, parsing 1M logs drops from 10s to ~1.5s.

By the end of this project, you'll have built a **production-grade log processor** achieving 100-1000x better performance than naive string handling—the same techniques used in Elasticsearch, Splunk, and other high-throughput data systems.

---

## Solution Outline

### Step 1: Basic Log Entry Parser with &str Slices
**Goal**: Parse log line into fields without allocating strings.

**What to implement**:
- Define `LogEntry<'a>` struct with lifetime-annotated `&str` fields
- Parse line using `split()`, `find()`, `trim()` (all return `&str`)
- Handle different delimiters (space, comma, JSON)
- Validate fields but don't copy data

**Why this step**: Foundation for zero-copy parsing. Establishes lifetime relationships between input and parsed fields.

**Testing hint**: Test with various log formats. Verify no allocations using memory profiler. Test lifetime correctness.

```rust
#[derive(Debug, PartialEq)]
pub struct LogEntry<'a> {
    pub timestamp: &'a str,
    pub level: &'a str,
    pub service: &'a str,
    pub message: &'a str,
}

impl<'a> LogEntry<'a> {
    pub fn parse_apache(line: &'a str) -> Option<Self> {
        // Parse: IP - - [timestamp] "method path protocol" status size
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 10 {
            return None;
        }

        // Extract timestamp from [timestamp]
        let timestamp_start = line.find('[')?;
        let timestamp_end = line.find(']')?;
        let timestamp = &line[timestamp_start + 1..timestamp_end];

        // Extract method/path from "GET /path HTTP/1.1"
        let quote_start = line.find('"')?;
        let quote_end = line[quote_start + 1..].find('"')? + quote_start + 1;
        let request = &line[quote_start + 1..quote_end];

        let request_parts: Vec<&str> = request.split_whitespace().collect();
        let path = request_parts.get(1)?;

        Some(LogEntry {
            timestamp,
            level: "INFO",  // Apache logs don't have explicit level
            service: "web",
            message: path,
        })
    }

    pub fn parse_json(line: &'a str) -> Option<Self> {
        // Simple JSON parsing without allocations
        // In production, use serde_json with zero-copy deserialization

        let timestamp = extract_json_field(line, "timestamp")?;
        let level = extract_json_field(line, "level")?;
        let service = extract_json_field(line, "service")?;
        let message = extract_json_field(line, "message")?;

        Some(LogEntry {
            timestamp,
            level,
            service,
            message,
        })
    }
}

fn extract_json_field<'a>(json: &'a str, field: &str) -> Option<&'a str> {
    let pattern = format!("\"{}\":\"", field);
    let start = json.find(&pattern)? + pattern.len();
    let end = json[start..].find('"')? + start;
    Some(&json[start..end])
}
```

---

### Step 2: Zero-Copy Filtering with Iterator Chains
**Goal**: Filter log entries without collecting intermediate results.

**What to implement**:
- Create iterator over log lines
- Chain filters (by level, by service, by time range)
- Use `filter()` and `filter_map()` for zero-copy filtering
- Collect only final results

**Why the previous step is not enough**: Parsing is useful, but we need to filter logs. Collecting after each filter wastes memory.

**What's the improvement**: Iterator chains with filters process data lazily without intermediate allocations. For 1M logs with 3 filters:
- Naive (collect after each): 3 temporary `Vec`s, ~3M entries allocated
- Iterator chain: 0 temporary allocations, single pass

**Optimization focus**: Memory efficiency through lazy evaluation.

**Testing hint**: Verify filters work correctly. Test with large files. Monitor memory usage (should be constant).

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct LogIterator<R> {
    reader: BufReader<R>,
    line_buffer: String,
}

impl<R: std::io::Read> LogIterator<R> {
    pub fn new(reader: R) -> Self {
        LogIterator {
            reader: BufReader::new(reader),
            line_buffer: String::new(),
        }
    }
}

impl<R: std::io::Read> Iterator for LogIterator<R> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.line_buffer.clear();
        match self.reader.read_line(&mut self.line_buffer) {
            Ok(0) => None,  // EOF
            Ok(_) => Some(self.line_buffer.clone()),
            Err(_) => None,
        }
    }
}

pub fn filter_logs<'a>(
    lines: impl Iterator<Item = &'a str>,
    min_level: &str,
    service_filter: Option<&str>,
) -> impl Iterator<Item = LogEntry<'a>> {
    lines
        .filter_map(|line| LogEntry::parse_json(line))
        .filter(move |entry| {
            // Filter by level
            let level_priority = |l: &str| match l {
                "DEBUG" => 0,
                "INFO" => 1,
                "WARN" => 2,
                "ERROR" => 3,
                _ => 0,
            };

            level_priority(entry.level) >= level_priority(min_level)
        })
        .filter(move |entry| {
            // Filter by service if specified
            service_filter.map_or(true, |s| entry.service == s)
        })
}

// Usage: completely lazy, no allocations until collect
pub fn analyze_logs(path: &str) -> std::io::Result<Vec<LogEntry>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let errors: Vec<LogEntry> = reader
        .lines()
        .filter_map(Result::ok)
        .filter_map(|line| {
            // Parse each line (lifetime tied to string in iteration)
            // For zero-copy, we need to handle differently...
            // This requires streaming with owned strings or lifetimes
            None // Placeholder
        })
        .collect();

    Ok(errors)
}
```

---

### Step 3: Cow<str> for Conditional Normalization
**Goal**: Normalize log fields only when necessary using `Cow<str>`.

**What to implement**:
- Create `NormalizedLogEntry` with `Cow<str>` fields
- Normalize functions: lowercase, trim whitespace, remove special chars
- Return `Cow::Borrowed` if no changes needed
- Return `Cow::Owned` only when modified

**Why the previous step is not enough**: Some logs need normalization (case-insensitive search, trimming whitespace), but allocating for every entry is wasteful when most don't need changes.

**What's the improvement**: `Cow<str>` enables "lazy cloning"—allocate only when modification is necessary. For 1M logs where 10% need normalization:
- Eager allocation: 1M strings allocated
- Cow: 100K strings allocated (10x less memory)

**Optimization focus**: Memory efficiency through conditional allocation.

**Testing hint**: Test that unmodified strings return Borrowed. Test that modified strings return Owned. Verify memory usage difference.

```rust
use std::borrow::Cow;

pub struct NormalizedLogEntry<'a> {
    pub timestamp: Cow<'a, str>,
    pub level: Cow<'a, str>,
    pub service: Cow<'a, str>,
    pub message: Cow<'a, str>,
}

pub fn normalize_lowercase(s: &str) -> Cow<str> {
    if s.chars().all(|c| !c.is_uppercase()) {
        // Already lowercase, no allocation needed
        Cow::Borrowed(s)
    } else {
        // Needs conversion, allocate
        Cow::Owned(s.to_lowercase())
    }
}

pub fn normalize_trim(s: &str) -> Cow<str> {
    let trimmed = s.trim();
    if trimmed.len() == s.len() {
        // No whitespace removed, no allocation
        Cow::Borrowed(s)
    } else {
        // Whitespace removed, allocate
        Cow::Owned(trimmed.to_string())
    }
}

pub fn normalize_service_name(s: &str) -> Cow<str> {
    // Normalize: lowercase + remove special chars
    let needs_normalization = s.chars().any(|c| c.is_uppercase() || !c.is_alphanumeric());

    if !needs_normalization {
        Cow::Borrowed(s)
    } else {
        let normalized: String = s
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();
        Cow::Owned(normalized)
    }
}

impl<'a> NormalizedLogEntry<'a> {
    pub fn from_entry(entry: LogEntry<'a>) -> Self {
        NormalizedLogEntry {
            timestamp: Cow::Borrowed(entry.timestamp),
            level: normalize_lowercase(entry.level),
            service: normalize_service_name(entry.service),
            message: normalize_trim(entry.message),
        }
    }
}

// Benchmark: eager allocation vs Cow
use std::time::Instant;

pub fn benchmark_normalization(entries: &[LogEntry]) {
    // Eager allocation
    let start = Instant::now();
    let _eager: Vec<String> = entries
        .iter()
        .map(|e| e.service.to_lowercase())
        .collect();
    println!("Eager allocation: {:?}", start.elapsed());

    // Cow-based
    let start = Instant::now();
    let _cow: Vec<Cow<str>> = entries
        .iter()
        .map(|e| normalize_lowercase(e.service))
        .collect();
    println!("Cow allocation: {:?}", start.elapsed());
}
```

---

### Step 4: String Builder for Efficient Concatenation
**Goal**: Build formatted output efficiently using string builder pattern.

**What to implement**:
- `LogFormatter` that builds formatted strings
- Pre-allocate capacity based on average log size
- Chain formatting operations
- Support different output formats (JSON, CSV, plain text)

**Why the previous step is not enough**: Building output strings with `+` operator or `format!` causes multiple allocations and copies.

**What's the improvement**: Pre-allocated `String::with_capacity()` + `push_str()` eliminates reallocations:
- Naive concatenation: 10 strings = 10 allocations + copies
- Builder with capacity: 1 allocation, 0 copies

For building 1M formatted log lines:
- Naive: ~10M allocations
- Builder: ~1M allocations (10x improvement)

**Optimization focus**: Speed and memory through pre-allocation.

**Testing hint**: Benchmark concatenation methods. Verify capacity is sufficient (no reallocations). Test different output formats.

```rust
pub struct LogFormatter {
    buffer: String,
}

impl LogFormatter {
    pub fn with_capacity(capacity: usize) -> Self {
        LogFormatter {
            buffer: String::with_capacity(capacity),
        }
    }

    pub fn format_json(&mut self, entry: &LogEntry) -> &str {
        self.buffer.clear();

        self.buffer.push_str("{\"timestamp\":\"");
        self.buffer.push_str(entry.timestamp);
        self.buffer.push_str("\",\"level\":\"");
        self.buffer.push_str(entry.level);
        self.buffer.push_str("\",\"service\":\"");
        self.buffer.push_str(entry.service);
        self.buffer.push_str("\",\"message\":\"");
        self.buffer.push_str(entry.message);
        self.buffer.push_str("\"}");

        &self.buffer
    }

    pub fn format_csv(&mut self, entry: &LogEntry) -> &str {
        self.buffer.clear();

        self.buffer.push_str(entry.timestamp);
        self.buffer.push(',');
        self.buffer.push_str(entry.level);
        self.buffer.push(',');
        self.buffer.push_str(entry.service);
        self.buffer.push(',');
        self.buffer.push('"');
        self.buffer.push_str(entry.message);
        self.buffer.push('"');

        &self.buffer
    }
}

// Benchmark: format! vs builder
pub fn benchmark_formatting(entries: &[LogEntry]) {
    // Using format! macro
    let start = Instant::now();
    let _formatted: Vec<String> = entries
        .iter()
        .map(|e| format!("{},{},{},{}", e.timestamp, e.level, e.service, e.message))
        .collect();
    println!("format! macro: {:?}", start.elapsed());

    // Using builder
    let start = Instant::now();
    let mut formatter = LogFormatter::with_capacity(256);
    let _formatted: Vec<String> = entries
        .iter()
        .map(|e| formatter.format_csv(e).to_string())
        .collect();
    println!("Builder: {:?}", start.elapsed());
}
```

---

### Step 5: Search Index with String Interning
**Goal**: Build searchable index with deduplicated strings.

**What to implement**:
- String interner (deduplicate common strings)
- Index log entries by service, level
- Support fast lookup: "find all logs from service X"
- Measure memory savings from interning

**Why the previous step is not enough**: Storing millions of log entries with duplicate service names and levels wastes memory.

**What's the improvement**: String interning stores each unique string once. For 1M logs with 10 unique services:
- Without interning: 1M service name copies ≈ 20MB
- With interning: 10 service names ≈ 200 bytes (100,000x less!)

This is crucial for in-memory log analysis with millions of entries.

**Optimization focus**: Memory efficiency through deduplication.

**Testing hint**: Measure memory usage before/after interning. Verify string equality works. Test with realistic log data.

```rust
use std::collections::HashMap;

pub struct StringInterner {
    strings: Vec<String>,
    indices: HashMap<String, usize>,
}

impl StringInterner {
    pub fn new() -> Self {
        StringInterner {
            strings: Vec::new(),
            indices: HashMap::new(),
        }
    }

    pub fn intern(&mut self, s: &str) -> usize {
        if let Some(&index) = self.indices.get(s) {
            return index;
        }

        let index = self.strings.len();
        self.strings.push(s.to_string());
        self.indices.insert(s.to_string(), index);
        index
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        self.strings.get(index).map(|s| s.as_str())
    }
}

pub struct InternedLogEntry {
    pub timestamp: String,  // Timestamps are usually unique
    pub level: usize,       // Interned
    pub service: usize,     // Interned
    pub message: String,
}

pub struct LogIndex {
    interner: StringInterner,
    entries: Vec<InternedLogEntry>,
    by_service: HashMap<usize, Vec<usize>>,  // service_id -> entry indices
    by_level: HashMap<usize, Vec<usize>>,    // level_id -> entry indices
}

impl LogIndex {
    pub fn new() -> Self {
        LogIndex {
            interner: StringInterner::new(),
            entries: Vec::new(),
            by_service: HashMap::new(),
            by_level: HashMap::new(),
        }
    }

    pub fn add(&mut self, entry: LogEntry) {
        let level_id = self.interner.intern(entry.level);
        let service_id = self.interner.intern(entry.service);

        let entry_index = self.entries.len();
        self.entries.push(InternedLogEntry {
            timestamp: entry.timestamp.to_string(),
            level: level_id,
            service: service_id,
            message: entry.message.to_string(),
        });

        self.by_service
            .entry(service_id)
            .or_insert_with(Vec::new)
            .push(entry_index);

        self.by_level
            .entry(level_id)
            .or_insert_with(Vec::new)
            .push(entry_index);
    }

    pub fn find_by_service(&self, service: &str) -> Vec<&InternedLogEntry> {
        if let Some(&service_id) = self.interner.indices.get(service) {
            if let Some(indices) = self.by_service.get(&service_id) {
                return indices.iter().map(|&i| &self.entries[i]).collect();
            }
        }
        Vec::new()
    }

    pub fn memory_stats(&self) -> MemoryStats {
        let interner_memory = self.interner.strings
            .iter()
            .map(|s| s.len())
            .sum::<usize>();

        let entries_memory = self.entries.len() * std::mem::size_of::<InternedLogEntry>();

        MemoryStats {
            interner_memory,
            entries_memory,
            total_entries: self.entries.len(),
            unique_strings: self.interner.strings.len(),
        }
    }
}

#[derive(Debug)]
pub struct MemoryStats {
    pub interner_memory: usize,
    pub entries_memory: usize,
    pub total_entries: usize,
    pub unique_strings: usize,
}
```

---

### Step 6: Parallel Processing with Rayon
**Goal**: Process log files in parallel for maximum throughput.

**What to implement**:
- Read file in chunks
- Parse chunks in parallel
- Merge results into unified index
- Benchmark sequential vs parallel

**Why the previous step is not enough**: Sequential processing uses only one core, wasting CPU resources.

**What's the improvement**: Parallel parsing utilizes all CPU cores:
- Sequential: 100K logs/sec (1 core)
- Parallel: 700K logs/sec (8 cores, ~7x speedup)

For production log processors handling GB/day, this is crucial.

**Optimization focus**: Speed through parallelism.

**Testing hint**: Benchmark with large files. Verify all cores utilized. Ensure no data loss.

```rust
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

pub fn process_logs_parallel(path: &str) -> std::io::Result<LogIndex> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read lines into chunks
    let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();
    let chunk_size = 10_000;

    // Process chunks in parallel
    let results: Vec<Vec<LogEntry>> = lines
        .par_chunks(chunk_size)
        .map(|chunk| {
            chunk
                .iter()
                .filter_map(|line| LogEntry::parse_json(line))
                .collect()
        })
        .collect();

    // Merge into single index
    let mut index = LogIndex::new();
    for chunk_results in results {
        for entry in chunk_results {
            index.add(entry);
        }
    }

    Ok(index)
}

// Benchmark
pub fn benchmark_parallel_parsing(path: &str) {
    let start = Instant::now();
    let _index = process_logs_parallel(path).unwrap();
    let parallel_time = start.elapsed();
    println!("Parallel: {:?}", parallel_time);

    // Compare with sequential (from previous steps)
    // Sequential version would process without par_chunks
}
```

---

### Complete Working Example

```rust
use rayon::prelude::*;
use std::{
    borrow::Cow,
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader, Read},
};

// =============================================================================
// Milestone 1: Zero-Copy Log Entry Parser
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogEntry<'a> {
    pub timestamp: &'a str,
    pub level: &'a str,
    pub service: &'a str,
    pub message: &'a str,
}

impl<'a> LogEntry<'a> {
    pub fn parse_line(line: &'a str) -> Option<Self> {
        let trimmed = line.trim_start();
        if trimmed.starts_with('{') {
            Self::parse_json(trimmed)
        } else if trimmed.contains("[") && trimmed.contains("]") {
            Self::parse_apache(trimmed)
        } else {
            Self::parse_syslog(trimmed)
        }
    }

    pub fn parse_apache(line: &'a str) -> Option<Self> {
        let timestamp_start = line.find('[')?;
        let timestamp_end = line[timestamp_start + 1..].find(']')? + timestamp_start + 1;
        let timestamp = &line[timestamp_start + 1..timestamp_end];

        let quote_start = line.find('"')?;
        let quote_end = line[quote_start + 1..].find('"')? + quote_start + 1;
        let request = &line[quote_start + 1..quote_end];
        let mut request_parts = request.split_whitespace();
        let path = request_parts.nth(1).unwrap_or("/");

        Some(LogEntry {
            timestamp,
            level: "INFO",
            service: "apache",
            message: path,
        })
    }

    pub fn parse_json(line: &'a str) -> Option<Self> {
        let timestamp = extract_json_field(line, "timestamp")?;
        let level = extract_json_field(line, "level")?;
        let service = extract_json_field(line, "service")?;
        let message = extract_json_field(line, "message")?;

        Some(LogEntry {
            timestamp,
            level,
            service,
            message,
        })
    }

    pub fn parse_syslog(line: &'a str) -> Option<Self> {
        let mut parts = line.splitn(5, ' ');
        let month = parts.next()?;
        let day = parts.next()?;
        let time = parts.next()?;
        let _host = parts.next()?;
        let rest = parts.next()?;
        let timestamp = &line[..month.len() + 1 + day.len() + 1 + time.len()];

        let (service, message) = rest.split_once(':')?;
        Some(LogEntry {
            timestamp,
            level: "INFO",
            service: service.trim(),
            message: message.trim(),
        })
    }
}

fn extract_json_field<'a>(json: &'a str, field: &str) -> Option<&'a str> {
    let needle = format!("\"{}\":\"", field);
    let start = json.find(&needle)? + needle.len();
    let end = json[start..].find('"')? + start;
    Some(&json[start..end])
}

// =============================================================================
// Milestone 2: Iterator-Based Zero-Copy Filtering
// =============================================================================

pub struct LogIterator<R> {
    reader: BufReader<R>,
    line_buffer: String,
}

impl<R: Read> LogIterator<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            line_buffer: String::new(),
        }
    }
}

impl<R: Read> Iterator for LogIterator<R> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.line_buffer.clear();
        match self.reader.read_line(&mut self.line_buffer) {
            Ok(0) => None,
            Ok(_) => {
                if self.line_buffer.ends_with('\n') {
                    self.line_buffer.pop();
                    if self.line_buffer.ends_with('\r') {
                        self.line_buffer.pop();
                    }
                }
                Some(self.line_buffer.clone())
            }
            Err(_) => None,
        }
    }
}

pub fn filter_logs<'a, I>(
    lines: I,
    min_level: &str,
    service_filter: Option<&'a str>,
) -> impl Iterator<Item = LogEntry<'a>>
where
    I: IntoIterator<Item = &'a str>,
{
    let min_priority = level_priority(min_level);
    lines
        .into_iter()
        .filter_map(|line| LogEntry::parse_line(line))
        .filter(move |entry| level_priority(entry.level) >= min_priority)
        .filter(move |entry| service_filter.map_or(true, |service| entry.service == service))
}

fn level_priority(level: &str) -> u8 {
    match level {
        "DEBUG" => 1,
        "INFO" => 2,
        "WARN" | "WARNING" => 3,
        "ERROR" => 4,
        _ => 0,
    }
}

// =============================================================================
// Milestone 3: Cow-Based Normalization
// =============================================================================

pub struct NormalizedLogEntry<'a> {
    pub timestamp: Cow<'a, str>,
    pub level: Cow<'a, str>,
    pub service: Cow<'a, str>,
    pub message: Cow<'a, str>,
}

pub fn normalize_lowercase(input: &str) -> Cow<'_, str> {
    if input.chars().all(|c| !c.is_uppercase()) {
        Cow::Borrowed(input)
    } else {
        Cow::Owned(input.to_lowercase())
    }
}

pub fn normalize_trim(input: &str) -> Cow<'_, str> {
    let trimmed = input.trim();
    if trimmed.len() == input.len() {
        Cow::Borrowed(input)
    } else {
        Cow::Owned(trimmed.to_string())
    }
}

pub fn normalize_service_name(input: &str) -> Cow<'_, str> {
    let needs_change = input
        .chars()
        .any(|c| c.is_uppercase() || !(c.is_ascii_alphanumeric() || c == '-'));
    if !needs_change {
        Cow::Borrowed(input)
    } else {
        let mut normalized = String::with_capacity(input.len());
        for ch in input.chars() {
            if ch.is_ascii_alphanumeric() || ch == '-' {
                normalized.push(ch.to_ascii_lowercase());
            }
        }
        Cow::Owned(normalized)
    }
}

impl<'a> NormalizedLogEntry<'a> {
    pub fn from_entry(entry: LogEntry<'a>) -> Self {
        Self {
            timestamp: Cow::Borrowed(entry.timestamp),
            level: normalize_lowercase(entry.level),
            service: normalize_service_name(entry.service),
            message: normalize_trim(entry.message),
        }
    }
}

// =============================================================================
// Milestone 4: String Builder Formatter
// =============================================================================

pub struct LogFormatter {
    buffer: String,
}

impl LogFormatter {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: String::with_capacity(capacity),
        }
    }

    pub fn format_json<'a>(&'a mut self, entry: &LogEntry) -> &'a str {
        self.buffer.clear();
        self.buffer.push_str("{\"timestamp\":\"");
        self.buffer.push_str(entry.timestamp);
        self.buffer.push_str("\",\"level\":\"");
        self.buffer.push_str(entry.level);
        self.buffer.push_str("\",\"service\":\"");
        self.buffer.push_str(entry.service);
        self.buffer.push_str("\",\"message\":\"");
        self.buffer.push_str(entry.message);
        self.buffer.push_str("\"}");
        &self.buffer
    }

    pub fn format_plain<'a>(&'a mut self, entry: &LogEntry) -> &'a str {
        self.buffer.clear();
        self.buffer.push_str(entry.timestamp);
        self.buffer.push_str(" | ");
        self.buffer.push_str(entry.level);
        self.buffer.push_str(" | ");
        self.buffer.push_str(entry.service);
        self.buffer.push_str(" | ");
        self.buffer.push_str(entry.message);
        &self.buffer
    }
}

// =============================================================================
// Milestone 5: Search Index with String Interning
// =============================================================================

pub struct StringInterner {
    strings: Vec<String>,
    indices: HashMap<String, usize>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            indices: HashMap::new(),
        }
    }

    pub fn intern(&mut self, value: &str) -> usize {
        if let Some(&idx) = self.indices.get(value) {
            return idx;
        }
        let idx = self.strings.len();
        self.strings.push(value.to_string());
        self.indices.insert(value.to_string(), idx);
        idx
    }

    pub fn get(&self, idx: usize) -> Option<&str> {
        self.strings.get(idx).map(|s| s.as_str())
    }
}

pub struct InternedLogEntry {
    pub timestamp: String,
    pub level: usize,
    pub service: usize,
    pub message: String,
}

pub struct LogIndex {
    interner: StringInterner,
    entries: Vec<InternedLogEntry>,
    by_service: HashMap<usize, Vec<usize>>,
    by_level: HashMap<usize, Vec<usize>>,
}

impl LogIndex {
    pub fn new() -> Self {
        Self {
            interner: StringInterner::new(),
            entries: Vec::new(),
            by_service: HashMap::new(),
            by_level: HashMap::new(),
        }
    }

    pub fn add(&mut self, entry: LogEntry<'_>) {
        let level_id = self.interner.intern(entry.level);
        let service_id = self.interner.intern(entry.service);
        let entry_id = self.entries.len();
        self.entries.push(InternedLogEntry {
            timestamp: entry.timestamp.to_string(),
            level: level_id,
            service: service_id,
            message: entry.message.to_string(),
        });
        self.by_service
            .entry(service_id)
            .or_default()
            .push(entry_id);
        self.by_level.entry(level_id).or_default().push(entry_id);
    }

    pub fn find_by_service(&self, service: &str) -> Vec<&InternedLogEntry> {
        self.interner
            .indices
            .get(service)
            .and_then(|id| self.by_service.get(id))
            .map(|indices| indices.iter().map(|&i| &self.entries[i]).collect())
            .unwrap_or_default()
    }

    pub fn find_by_level(&self, level: &str) -> Vec<&InternedLogEntry> {
        self.interner
            .indices
            .get(level)
            .and_then(|id| self.by_level.get(id))
            .map(|indices| indices.iter().map(|&i| &self.entries[i]).collect())
            .unwrap_or_default()
    }

    pub fn memory_stats(&self) -> MemoryStats {
        let interner_memory = self.strings_memory();
        let entries_memory = self.entries.len() * std::mem::size_of::<InternedLogEntry>();
        MemoryStats {
            interner_memory,
            entries_memory,
            total_entries: self.entries.len(),
            unique_strings: self.interner.strings.len(),
        }
    }

    fn strings_memory(&self) -> usize {
        self.interner.strings.iter().map(|s| s.len()).sum()
    }
}

#[derive(Debug)]
pub struct MemoryStats {
    pub interner_memory: usize,
    pub entries_memory: usize,
    pub total_entries: usize,
    pub unique_strings: usize,
}

// =============================================================================
// Milestone 6: Parallel Log Processing
// =============================================================================

pub fn process_logs_parallel(path: &str, chunk_size: usize) -> io::Result<LogIndex> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    for line in reader.lines() {
        lines.push(line?);
    }

    let parsed: Vec<Vec<LogEntry>> = lines
        .par_chunks(chunk_size.max(1))
        .map(|chunk| {
            chunk
                .iter()
                .filter_map(|line| LogEntry::parse_line(line))
                .collect()
        })
        .collect();

    let mut index = LogIndex::new();
    for group in parsed {
        for entry in group {
            index.add(entry);
        }
    }

    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn parse_json_log_line() {
        let line = "{\"timestamp\":\"2024-10-10\",\"level\":\"ERROR\",\"service\":\"auth\",\"message\":\"failed\"}";
        let entry = LogEntry::parse_json(line).unwrap();
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.service, "auth");
    }

    #[test]
    fn parse_apache_log_line() {
        let line = "127.0.0.1 - - [10/Oct/2024:13:55:36 -0700] \"GET /index.html HTTP/1.1\" 200 2326";
        let entry = LogEntry::parse_apache(line).unwrap();
        assert_eq!(entry.timestamp, "10/Oct/2024:13:55:36 -0700");
        assert_eq!(entry.message, "/index.html");
    }

    #[test]
    fn parse_syslog_log_line() {
        let line = "Oct 10 13:55:36 host service[1234]: Something happened";
        let entry = LogEntry::parse_syslog(line).unwrap();
        assert_eq!(entry.service, "service[1234]");
        assert_eq!(entry.message, "Something happened");
    }

    #[test]
    fn iterator_reads_lines() {
        let data = b"line1\nline2\n";
        let iter = LogIterator::new(&data[..]);
        let collected: Vec<String> = iter.collect();
        assert_eq!(collected, vec!["line1".to_string(), "line2".to_string()]);
    }

    #[test]
    fn filter_by_level_and_service() {
        let logs = vec![
            String::from("{\"timestamp\":\"t1\",\"level\":\"INFO\",\"service\":\"api\",\"message\":\"ok\"}"),
            String::from("{\"timestamp\":\"t2\",\"level\":\"ERROR\",\"service\":\"api\",\"message\":\"fail\"}"),
            String::from("{\"timestamp\":\"t3\",\"level\":\"ERROR\",\"service\":\"db\",\"message\":\"oops\"}"),
        ];
        let borrowed: Vec<&str> = logs.iter().map(|s| s.as_str()).collect();
        let filtered: Vec<_> = filter_logs(borrowed, "ERROR", Some("api")).collect();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].message, "fail");
    }

    #[test]
    fn normalization_borrowed_vs_owned() {
        match normalize_lowercase("auth") {
            Cow::Borrowed(_) => {}
            Cow::Owned(_) => panic!("should be borrowed"),
        }
        match normalize_lowercase("AUTH") {
            Cow::Owned(val) => assert_eq!(val, "auth"),
            Cow::Borrowed(_) => panic!("should be owned"),
        }
    }

    #[test]
    fn formatter_produces_json() {
        let entry = LogEntry {
            timestamp: "t",
            level: "INFO",
            service: "svc",
            message: "msg",
        };
        let mut formatter = LogFormatter::with_capacity(128);
        assert_eq!(formatter.format_json(&entry), "{\"timestamp\":\"t\",\"level\":\"INFO\",\"service\":\"svc\",\"message\":\"msg\"}");
    }

    #[test]
    fn interner_deduplicates_strings() {
        let mut interner = StringInterner::new();
        let a = interner.intern("alpha");
        let b = interner.intern("alpha");
        assert_eq!(a, b);
        assert_eq!(interner.get(a), Some("alpha"));
    }

    #[test]
    fn log_index_queries() {
        let mut index = LogIndex::new();
        let entry = LogEntry {
            timestamp: "t",
            level: "ERROR",
            service: "api",
            message: "fail",
        };
        index.add(entry);
        assert_eq!(index.find_by_service("api").len(), 1);
        assert_eq!(index.find_by_level("ERROR").len(), 1);
    }

    #[test]
    fn parallel_processing_builds_index() {
        let content = "{\"timestamp\":\"t1\",\"level\":\"INFO\",\"service\":\"api\",\"message\":\"ok\"}\n{\"timestamp\":\"t2\",\"level\":\"ERROR\",\"service\":\"api\",\"message\":\"fail\"}\n";
        let mut file = NamedTempFile::new().unwrap();
        use std::io::Write;
        file.write_all(content.as_bytes()).unwrap();
        let index = process_logs_parallel(file.path().to_str().unwrap(), 1).unwrap();
        assert_eq!(index.find_by_service("api").len(), 2);
    }
}

```