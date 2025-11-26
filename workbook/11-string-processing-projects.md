# Chapter 11: String Processing - Programming Projects

## Project 1: Zero-Copy Log Parser and Analyzer

### Problem Statement

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

### Why It Matters

String processing is often a bottleneck in data pipelines. Naive approaches allocate strings for every operation, causing excessive memory usage and garbage collection pressure. Zero-copy techniques eliminate allocations by working with string slices (`&str`), providing 10-100x speedup for parsing-heavy workloads.

`Cow<str>` enables "modify only if needed" pattern: if no transformation required, return borrowed data; otherwise allocate only when necessary. This is crucial for high-throughput systems (log processors, web servers, parsers).

### Use Cases

- Log aggregation and analysis (Elasticsearch, Splunk-style systems)
- Web server access log processing
- Security log analysis (SIEM systems)
- Application monitoring and debugging
- Log-based metrics extraction
- Compliance and audit log processing

### Solution Outline

#### Step 1: Basic Log Entry Parser with &str Slices
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

#### Step 2: Zero-Copy Filtering with Iterator Chains
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

#### Step 3: Cow<str> for Conditional Normalization
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

#### Step 4: String Builder for Efficient Concatenation
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

#### Step 5: Search Index with String Interning
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

#### Step 6: Parallel Processing with Rayon
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

### Testing Strategies

1. **Unit Tests**: Test each parser with known inputs
2. **Memory Tests**: Monitor allocations with different approaches
3. **Performance Tests**: Benchmark zero-copy vs allocation-heavy approaches
4. **Correctness Tests**: Verify no data loss during processing
5. **Property Tests**: Verify invariants (e.g., interned strings are unique)
6. **Integration Tests**: End-to-end with real log files

---

## Project 2: Text Editor Buffer with Gap Buffer

### Problem Statement

Build a text editor buffer data structure that efficiently handles text insertion and deletion at cursor position. Implement gap buffer algorithm for O(1) insert/delete at cursor with minimal memory overhead.

Your text editor should:
- Support cursor movement (forward, backward, start, end)
- Insert character at cursor position in O(1)
- Delete character at cursor in O(1)
- Support multi-line text
- Undo/redo functionality
- Efficient memory usage (no reallocation on most operations)

### Why It Matters

Text editors need to handle millions of characters with frequent insertions/deletions. Naive approaches (Vec\<char>, String) require O(n) operations to insert in middle. Gap buffer achieves O(1) insertion at cursor by maintaining a gap at cursor position.

This pattern is used by: Emacs, many terminal emulators, and high-performance text editors. Understanding gap buffer teaches memory layout optimization and amortized analysis.

### Use Cases

- Text editors (Emacs, vim-style)
- Terminal emulators (handling backspace, insert mode)
- Command-line input buffers
- Rich text editors
- Code editors with syntax highlighting

### Solution Outline

#### Step 1: Basic Gap Buffer Structure
**Goal**: Implement gap buffer with insert and delete at cursor.

**What to implement**:
- `GapBuffer` with `Vec<u8>` backing storage
- Gap start and gap end indices
- `insert_at_cursor()` places char in gap
- `delete_at_cursor()` expands gap
- Move gap to cursor position when needed

**Why this step**: Core gap buffer algorithm. Understanding gap concept is essential.

**Testing hint**: Test insert/delete at various positions. Verify gap is maintained correctly. Test edge cases (empty buffer, full buffer).

```rust
pub struct GapBuffer {
    buffer: Vec<u8>,
    gap_start: usize,
    gap_end: usize,
}

impl GapBuffer {
    pub fn new(capacity: usize) -> Self {
        GapBuffer {
            buffer: vec![0; capacity],
            gap_start: 0,
            gap_end: capacity,
        }
    }

    pub fn gap_size(&self) -> usize {
        self.gap_end - self.gap_start
    }

    pub fn len(&self) -> usize {
        self.buffer.len() - self.gap_size()
    }

    pub fn move_gap_to(&mut self, position: usize) {
        if position < self.gap_start {
            // Move gap backward
            let distance = self.gap_start - position;
            self.buffer.copy_within(position..self.gap_start, self.gap_end - distance);
            self.gap_end -= distance;
            self.gap_start = position;
        } else if position > self.gap_start {
            // Move gap forward
            let distance = position - self.gap_start;
            self.buffer.copy_within(self.gap_end..self.gap_end + distance, self.gap_start);
            self.gap_start += distance;
            self.gap_end += distance;
        }
    }

    pub fn insert(&mut self, ch: u8) {
        if self.gap_size() == 0 {
            self.grow();
        }

        self.buffer[self.gap_start] = ch;
        self.gap_start += 1;
    }

    pub fn delete(&mut self) -> Option<u8> {
        if self.gap_start == 0 {
            return None;
        }

        self.gap_start -= 1;
        Some(self.buffer[self.gap_start])
    }

    fn grow(&mut self) {
        let new_capacity = self.buffer.len() * 2;
        let old_gap_size = self.gap_size();

        self.buffer.resize(new_capacity, 0);
        self.gap_end = self.buffer.len();

        // Move content after gap to end
        let content_after_gap = self.buffer.len() - old_gap_size - self.gap_start;
        if content_after_gap > 0 {
            self.buffer.copy_within(
                self.gap_start..self.gap_start + content_after_gap,
                self.gap_end - content_after_gap
            );
        }
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        result.push_str(std::str::from_utf8(&self.buffer[..self.gap_start]).unwrap());
        result.push_str(std::str::from_utf8(&self.buffer[self.gap_end..]).unwrap());
        result
    }
}
```

---

#### Step 2: Cursor Management and Operations
**Goal**: Add cursor abstraction for user-friendly interface.

**What to implement**:
- `Cursor` struct tracking position
- Move cursor (left, right, start, end)
- Insert/delete operations relative to cursor
- Ensure gap follows cursor

**Why the previous step is not enough**: Raw gap buffer works but is low-level. Cursor abstraction provides intuitive interface.

**What's the improvement**: Cursor makes gap buffer usable like real text editor. Moving cursor efficiently moves gap, maintaining O(1) insert/delete.

**Testing hint**: Test cursor movements. Verify gap moves with cursor. Test insert/delete at cursor.

```rust
pub struct TextBuffer {
    gap_buffer: GapBuffer,
    cursor: usize,
}

impl TextBuffer {
    pub fn new() -> Self {
        TextBuffer {
            gap_buffer: GapBuffer::new(128),
            cursor: 0,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.gap_buffer.move_gap_to(self.cursor);

        for byte in ch.to_string().bytes() {
            self.gap_buffer.insert(byte);
        }

        self.cursor += ch.len_utf8();
    }

    pub fn delete_char(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }

        self.gap_buffer.move_gap_to(self.cursor);

        // Handle UTF-8 character boundaries
        if let Some(_) = self.gap_buffer.delete() {
            self.cursor -= 1;
            true
        } else {
            false
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.gap_buffer.len() {
            self.cursor += 1;
        }
    }

    pub fn move_cursor_start(&mut self) {
        self.cursor = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor = self.gap_buffer.len();
    }

    pub fn text(&self) -> String {
        self.gap_buffer.to_string()
    }

    pub fn cursor_position(&self) -> usize {
        self.cursor
    }
}
```

---

#### Step 3: Multi-Line Support with Line Index
**Goal**: Add efficient line-based operations (goto line, insert line).

**What to implement**:
- Track line boundaries (newline positions)
- Map cursor position to (line, column)
- Operations: goto_line, insert_newline, delete_line
- Update line index on edits

**Why the previous step is not enough**: Single-line buffer works for simple cases, but real editors need multi-line support.

**What's the improvement**: Line index enables O(log n) line lookups and line-based operations. Essential for displaying line numbers, goto line commands.

**Testing hint**: Test multi-line text. Verify line boundaries are tracked. Test goto_line accuracy.

```rust
pub struct MultiLineBuffer {
    buffer: TextBuffer,
    line_starts: Vec<usize>,  // Positions of line starts
}

impl MultiLineBuffer {
    pub fn new() -> Self {
        MultiLineBuffer {
            buffer: TextBuffer::new(),
            line_starts: vec![0],
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        let cursor = self.buffer.cursor_position();
        self.buffer.insert_char(ch);

        if ch == '\n' {
            // Find insertion point in line_starts
            let line_index = self.line_starts.partition_point(|&pos| pos <= cursor);
            self.line_starts.insert(line_index, cursor + 1);

            // Update all line starts after insertion
            for pos in &mut self.line_starts[line_index + 1..] {
                *pos += 1;
            }
        } else {
            // Update all line starts after insertion
            let line_index = self.line_starts.partition_point(|&pos| pos <= cursor);
            for pos in &mut self.line_starts[line_index..] {
                *pos += ch.len_utf8();
            }
        }
    }

    pub fn cursor_to_line_col(&self, cursor: usize) -> (usize, usize) {
        let line = self.line_starts.partition_point(|&pos| pos <= cursor);
        let line_start = if line > 0 {
            self.line_starts[line - 1]
        } else {
            0
        };
        let column = cursor - line_start;
        (line, column)
    }

    pub fn line_col_to_cursor(&self, line: usize, column: usize) -> Option<usize> {
        if line >= self.line_starts.len() {
            return None;
        }

        let line_start = self.line_starts[line];
        Some(line_start + column)
    }

    pub fn goto_line(&mut self, line: usize) {
        if let Some(line_start) = self.line_starts.get(line) {
            self.buffer.cursor = *line_start;
        }
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }
}
```

---

#### Step 4: Undo/Redo with Command Pattern
**Goal**: Implement undo/redo functionality.

**What to implement**:
- `EditCommand` enum (Insert, Delete, etc.)
- Command history stack
- Undo: reverse command and add to redo stack
- Redo: replay command
- Batch commands (group multiple edits as single undo)

**Why the previous step is not enough**: Real editors need undo/redo. Users expect to revert mistakes.

**What's the improvement**: Command pattern enables undo/redo with minimal overhead. Each edit stores command (type + data), not entire buffer state.

**Testing hint**: Test undo/redo sequences. Verify state is restored correctly. Test undo limit.

```rust
#[derive(Clone)]
pub enum EditCommand {
    InsertChar { position: usize, ch: char },
    DeleteChar { position: usize, ch: char },
    InsertText { position: usize, text: String },
    DeleteText { position: usize, text: String },
}

impl EditCommand {
    pub fn inverse(&self) -> EditCommand {
        match self {
            EditCommand::InsertChar { position, ch } => {
                EditCommand::DeleteChar { position: *position, ch: *ch }
            }
            EditCommand::DeleteChar { position, ch } => {
                EditCommand::InsertChar { position: *position, ch: *ch }
            }
            EditCommand::InsertText { position, text } => {
                EditCommand::DeleteText { position: *position, text: text.clone() }
            }
            EditCommand::DeleteText { position, text } => {
                EditCommand::InsertText { position: *position, text: text.clone() }
            }
        }
    }
}

pub struct EditorWithUndo {
    buffer: MultiLineBuffer,
    undo_stack: Vec<EditCommand>,
    redo_stack: Vec<EditCommand>,
    max_undo: usize,
}

impl EditorWithUndo {
    pub fn new() -> Self {
        EditorWithUndo {
            buffer: MultiLineBuffer::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo: 1000,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        let position = self.buffer.buffer.cursor_position();
        self.buffer.insert_char(ch);

        let command = EditCommand::InsertChar { position, ch };
        self.add_to_undo(command);
    }

    pub fn delete_char(&mut self) {
        let position = self.buffer.buffer.cursor_position();
        if position == 0 {
            return;
        }

        // Get character being deleted (simplified)
        let ch = ' '; // Would need to extract actual char from buffer

        if self.buffer.buffer.delete_char() {
            let command = EditCommand::DeleteChar { position, ch };
            self.add_to_undo(command);
        }
    }

    fn add_to_undo(&mut self, command: EditCommand) {
        self.undo_stack.push(command);
        self.redo_stack.clear();  // Clear redo stack on new edit

        if self.undo_stack.len() > self.max_undo {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some(command) = self.undo_stack.pop() {
            self.execute_command(&command.inverse());
            self.redo_stack.push(command);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(command) = self.redo_stack.pop() {
            self.execute_command(&command);
            self.undo_stack.push(command);
            true
        } else {
            false
        }
    }

    fn execute_command(&mut self, command: &EditCommand) {
        // Execute command without adding to undo stack
        match command {
            EditCommand::InsertChar { position, ch } => {
                self.buffer.buffer.cursor = *position;
                self.buffer.insert_char(*ch);
            }
            EditCommand::DeleteChar { position, .. } => {
                self.buffer.buffer.cursor = *position;
                self.buffer.buffer.delete_char();
            }
            _ => {}
        }
    }
}
```

---

#### Step 5: Performance Comparison with Alternatives
**Goal**: Benchmark gap buffer vs Vec\<char>, String, Rope.

**What to implement**:
- Implement same operations with Vec\<char>
- Implement with String
- Benchmark: random insertions, sequential insertions, deletions
- Compare memory usage

**Why the previous step is not enough**: Understanding why gap buffer is chosen requires comparing alternatives.

**What's the improvement**: Benchmarks reveal trade-offs:
- Vec: O(n) insert in middle, simple
- Gap buffer: O(1) insert at cursor, O(n) gap movement
- Rope: O(log n) insert anywhere, complex

Gap buffer wins for sequential editing (typical text editing pattern).

**Testing hint**: Test with realistic editing patterns (typing, backspace, cursor movement). Measure operations/second.

```rust
use std::time::Instant;

// Vec<char> implementation
pub struct VecBuffer {
    chars: Vec<char>,
    cursor: usize,
}

impl VecBuffer {
    pub fn new() -> Self {
        VecBuffer {
            chars: Vec::new(),
            cursor: 0,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.chars.insert(self.cursor, ch);
        self.cursor += 1;
    }

    pub fn delete_char(&mut self) -> bool {
        if self.cursor > 0 {
            self.chars.remove(self.cursor - 1);
            self.cursor -= 1;
            true
        } else {
            false
        }
    }
}

pub fn benchmark_editors() {
    let operations = 10_000;

    // Gap buffer
    let start = Instant::now();
    let mut gap_buffer = TextBuffer::new();
    for i in 0..operations {
        gap_buffer.insert_char('a');
        if i % 2 == 0 {
            gap_buffer.move_cursor_left();
        }
    }
    println!("Gap buffer: {:?}", start.elapsed());

    // Vec buffer
    let start = Instant::now();
    let mut vec_buffer = VecBuffer::new();
    for i in 0..operations {
        vec_buffer.insert_char('a');
        if i % 2 == 0 && vec_buffer.cursor > 0 {
            vec_buffer.cursor -= 1;
        }
    }
    println!("Vec buffer: {:?}", start.elapsed());

    // String buffer
    let start = Instant::now();
    let mut string_buffer = String::new();
    for _ in 0..operations {
        string_buffer.insert(string_buffer.len() / 2, 'a');
    }
    println!("String buffer: {:?}", start.elapsed());
}
```

---

#### Step 6: Optimize Memory Layout for Cache
**Goal**: Optimize gap buffer for cache efficiency.

**What to implement**:
- Measure cache misses with different gap sizes
- Experiment with gap size strategy (fixed vs dynamic)
- Add prefetching hints (advanced)
- Profile cache performance

**Why the previous step is not enough**: Algorithmic complexity is O(1), but constant factors matter. Cache efficiency can provide 2-10x speedup.

**What's the improvement**: Smaller gaps fit in cache, larger gaps reduce gap movement frequency. Optimal gap size balances these trade-offs.

**Optimization focus**: Speed through cache optimization.

**Testing hint**: Use perf tools (Linux) or Instruments (macOS) to measure cache misses. Test different gap sizes.

```rust
// Advanced: configurable gap growth strategy
pub struct OptimizedGapBuffer {
    buffer: Vec<u8>,
    gap_start: usize,
    gap_end: usize,
    growth_strategy: GrowthStrategy,
}

pub enum GrowthStrategy {
    Fixed(usize),      // Fixed gap size
    Proportional(f32), // Gap size as proportion of buffer
    Adaptive,          // Adjust based on edit pattern
}

impl OptimizedGapBuffer {
    pub fn new_with_strategy(capacity: usize, strategy: GrowthStrategy) -> Self {
        let initial_gap = match strategy {
            GrowthStrategy::Fixed(size) => size,
            GrowthStrategy::Proportional(ratio) => (capacity as f32 * ratio) as usize,
            GrowthStrategy::Adaptive => capacity / 4,
        };

        OptimizedGapBuffer {
            buffer: vec![0; capacity],
            gap_start: 0,
            gap_end: initial_gap,
            growth_strategy: strategy,
        }
    }

    // Optimized gap movement with prefetch
    pub fn move_gap_optimized(&mut self, position: usize) {
        // Implementation with prefetch hints
        // Would use platform-specific intrinsics in production
    }
}
```

---

### Testing Strategies

1. **Unit Tests**: Test each operation independently
2. **Property Tests**: Verify buffer contents match expected string
3. **Performance Tests**: Benchmark against alternatives
4. **Memory Tests**: Measure memory usage and allocations
5. **Stress Tests**: Test with large documents (1M+ characters)
6. **Cache Profiling**: Use perf/Instruments to measure cache efficiency

---

## Project 3: Fast String Search with Boyer-Moore Algorithm

### Problem Statement

Implement the Boyer-Moore string search algorithm for finding patterns in text efficiently. This algorithm is used in grep, text editors, and search systems because it can skip large portions of text, making it faster than naive search for most cases.

Your implementation should:
- Build bad character and good suffix tables
- Search for pattern in text with O(n/m) average case (faster than O(n))
- Support case-insensitive search
- Find all occurrences efficiently
- Benchmark against naive search

### Why It Matters

String search is fundamental to text processing. Naive search is O(n*m), checking every position. Boyer-Moore is O(n/m) average case, skipping text based on mismatches. For searching "pattern" in 1MB text:
- Naive: ~1M comparisons
- Boyer-Moore: ~150K comparisons (7x faster)

This algorithm is used in grep, text editors (find functionality), DNA sequence matching, plagiarism detection.

### Use Cases

- Text editors (find/replace)
- Log analysis (grep-style search)
- DNA sequence matching (bioinformatics)
- Intrusion detection (packet inspection)
- Plagiarism detection
- Search engines (document scanning)

### Solution Outline

#### Step 1: Naive String Search (Baseline)
**Goal**: Implement naive search for comparison.

**What to implement**:
- Search pattern in text character by character
- Return all match positions
- Measure operations count

**Why this step**: Establish baseline for comparison. Understanding naive approach makes Boyer-Moore improvements clear.

**Testing hint**: Test with various patterns and texts. Verify all matches found. Count comparisons.

```rust
pub fn naive_search(text: &str, pattern: &str) -> Vec<usize> {
    let text_bytes = text.as_bytes();
    let pattern_bytes = pattern.as_bytes();
    let mut matches = Vec::new();

    if pattern.is_empty() || pattern.len() > text.len() {
        return matches;
    }

    for i in 0..=(text.len() - pattern.len()) {
        let mut match_found = true;

        for j in 0..pattern.len() {
            if text_bytes[i + j] != pattern_bytes[j] {
                match_found = false;
                break;
            }
        }

        if match_found {
            matches.push(i);
        }
    }

    matches
}
```

---

#### Step 2: Build Bad Character Table
**Goal**: Implement bad character heuristic for skipping.

**What to implement**:
- Build table mapping each character to last occurrence in pattern
- On mismatch, skip based on bad character
- Handle characters not in pattern

**Why the previous step is not enough**: Naive search checks every position. Bad character heuristic skips positions based on mismatched character.

**What's the improvement**: When mismatch occurs, skip to align pattern with last occurrence of mismatched character. This can skip multiple positions:
- Naive: Always advances by 1
- Bad character: Can skip by pattern length

**Testing hint**: Test table construction. Verify skipping logic. Test with patterns having repeated characters.

```rust
use std::collections::HashMap;

pub struct BoyerMoore {
    pattern: Vec<u8>,
    bad_char_table: HashMap<u8, usize>,
}

impl BoyerMoore {
    pub fn new(pattern: &str) -> Self {
        let pattern_bytes = pattern.as_bytes().to_vec();
        let bad_char_table = Self::build_bad_char_table(&pattern_bytes);

        BoyerMoore {
            pattern: pattern_bytes,
            bad_char_table,
        }
    }

    fn build_bad_char_table(pattern: &[u8]) -> HashMap<u8, usize> {
        let mut table = HashMap::new();

        // For each character, store its rightmost position
        for (i, &ch) in pattern.iter().enumerate() {
            table.insert(ch, i);
        }

        table
    }

    pub fn search(&self, text: &str) -> Vec<usize> {
        let text_bytes = text.as_bytes();
        let mut matches = Vec::new();
        let m = self.pattern.len();
        let n = text_bytes.len();

        if m > n {
            return matches;
        }

        let mut i = 0;
        while i <= n - m {
            let mut j = m as isize - 1;

            // Match pattern from right to left
            while j >= 0 && self.pattern[j as usize] == text_bytes[i + j as usize] {
                j -= 1;
            }

            if j < 0 {
                // Pattern found
                matches.push(i);
                i += m;
            } else {
                // Mismatch - use bad character heuristic
                let bad_char = text_bytes[i + j as usize];
                let shift = if let Some(&last_occurrence) = self.bad_char_table.get(&bad_char) {
                    let shift = j as usize - last_occurrence;
                    shift.max(1)
                } else {
                    j as usize + 1
                };

                i += shift;
            }
        }

        matches
    }
}
```

---

#### Step 3: Add Good Suffix Heuristic
**Goal**: Implement good suffix table for additional skipping.

**What to implement**:
- Build good suffix table
- On mismatch, use both bad character and good suffix
- Take maximum skip from both heuristics

**Why the previous step is not enough**: Bad character heuristic alone doesn't handle all cases optimally. Good suffix adds another skipping strategy.

**What's the improvement**: Good suffix handles cases where bad character gives small skip. Using both heuristics gives maximum skip, making algorithm faster.

**Testing hint**: Test with patterns where good suffix provides larger skip. Verify both heuristics are used.

```rust
impl BoyerMoore {
    pub fn new_with_good_suffix(pattern: &str) -> Self {
        let pattern_bytes = pattern.as_bytes().to_vec();
        let bad_char_table = Self::build_bad_char_table(&pattern_bytes);
        let good_suffix_table = Self::build_good_suffix_table(&pattern_bytes);

        BoyerMoore {
            pattern: pattern_bytes,
            bad_char_table,
            // good_suffix_table, // Add this field
        }
    }

    fn build_good_suffix_table(pattern: &[u8]) -> Vec<usize> {
        let m = pattern.len();
        let mut table = vec![0; m];
        let mut suffix = vec![0; m];

        // Build suffix array
        suffix[m - 1] = m;
        let mut g = m - 1;
        let mut f = 0;

        for i in (0..m - 1).rev() {
            if i > g && suffix[i + m - 1 - f] < i - g {
                suffix[i] = suffix[i + m - 1 - f];
            } else {
                if i < g {
                    g = i;
                }
                f = i;
                while g > 0 && pattern[g - 1] == pattern[g + m - 1 - f] {
                    g -= 1;
                }
                suffix[i] = f - g + 1;
            }
        }

        // Build good suffix table from suffix array
        for i in 0..m {
            table[i] = m;
        }

        let mut j = 0;
        for i in (0..m - 1).rev() {
            if suffix[i] == i + 1 {
                while j < m - 1 - i {
                    if table[j] == m {
                        table[j] = m - 1 - i;
                    }
                    j += 1;
                }
            }
        }

        for i in 0..m - 1 {
            table[m - 1 - suffix[i]] = m - 1 - i;
        }

        table
    }
}
```

---

#### Step 4: Case-Insensitive Search
**Goal**: Support case-insensitive search efficiently.

**What to implement**:
- Normalize pattern and text to lowercase
- Use same Boyer-Moore algorithm
- Alternative: modify tables to handle case

**Why the previous step is not enough**: Case-sensitive search doesn't match "Hello" with "hello". Users often want case-insensitive.

**What's the improvement**: Case-insensitive search broadens matches. Normalizing to lowercase is simplest approach.

**Testing hint**: Test matches across different cases. Verify performance is similar to case-sensitive.

```rust
impl BoyerMoore {
    pub fn new_case_insensitive(pattern: &str) -> Self {
        let normalized = pattern.to_lowercase();
        Self::new(&normalized)
    }

    pub fn search_case_insensitive(&self, text: &str) -> Vec<usize> {
        let normalized_text = text.to_lowercase();
        self.search(&normalized_text)
    }
}
```

---

#### Step 5: Find All Occurrences with Streaming
**Goal**: Find matches in large files using streaming.

**What to implement**:
- Process file in chunks
- Handle pattern spanning chunk boundaries
- Use iterator for memory efficiency

**Why the previous step is not enough**: Loading entire file into memory fails for large files.

**What's the improvement**: Streaming enables processing files of any size with constant memory. Pattern boundary handling ensures no matches are missed.

**Testing hint**: Test with large files. Test patterns spanning chunks. Verify all matches found.

```rust
use std::io::{BufReader, Read};

pub fn search_file_streaming(
    path: &str,
    pattern: &str,
    chunk_size: usize,
) -> std::io::Result<Vec<usize>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let searcher = BoyerMoore::new(pattern);
    let mut matches = Vec::new();
    let mut buffer = vec![0u8; chunk_size + pattern.len()];
    let mut overlap = 0;
    let mut total_bytes_read = 0;

    loop {
        let bytes_read = reader.read(&mut buffer[overlap..])?;
        if bytes_read == 0 {
            break;
        }

        let search_len = overlap + bytes_read;
        let text = std::str::from_utf8(&buffer[..search_len]).unwrap_or("");

        // Search in current chunk
        for match_pos in searcher.search(text) {
            matches.push(total_bytes_read + match_pos - overlap);
        }

        total_bytes_read += bytes_read;

        // Keep overlap for pattern spanning chunks
        if search_len >= pattern.len() {
            overlap = pattern.len() - 1;
            buffer.copy_within(search_len - overlap..search_len, 0);
        } else {
            overlap = search_len;
        }
    }

    Ok(matches)
}
```

---

#### Step 6: Benchmark and Optimization
**Goal**: Compare performance against naive search and optimize.

**What to implement**:
- Benchmark with various pattern and text sizes
- Measure: operations count, time, cache misses
- Optimize: table lookups, memory layout
- Identify best and worst cases

**Why the previous step is not enough**: Implementation is complete, but understanding performance characteristics is essential.

**What's the improvement**: Benchmarks reveal:
- Best case: O(n/m) when pattern doesn't occur
- Worst case: O(n*m) with many false matches
- Average: Much faster than naive for most real-world text

**Optimization focus**: Understanding when Boyer-Moore excels vs when to use alternatives (e.g., KMP for small alphabets).

**Testing hint**: Benchmark with realistic text (code, prose, DNA). Test with short and long patterns. Compare with Rust's str::find().

```rust
pub fn benchmark_search_algorithms() {
    let text = include_str!("large_text.txt"); // 1MB text
    let pattern = "target";

    // Naive search
    let start = Instant::now();
    let _matches = naive_search(text, pattern);
    let naive_time = start.elapsed();
    println!("Naive search: {:?}", naive_time);

    // Boyer-Moore
    let searcher = BoyerMoore::new(pattern);
    let start = Instant::now();
    let _matches = searcher.search(text);
    let bm_time = start.elapsed();
    println!("Boyer-Moore: {:?}", bm_time);

    // Rust's built-in
    let start = Instant::now();
    let _matches: Vec<usize> = text.match_indices(pattern).map(|(i, _)| i).collect();
    let builtin_time = start.elapsed();
    println!("Built-in find: {:?}", builtin_time);

    println!("Speedup: {:.2}x", naive_time.as_secs_f64() / bm_time.as_secs_f64());
}
```

---

### Testing Strategies

1. **Correctness Tests**: Compare results with naive search
2. **Edge Cases**: Empty pattern, pattern longer than text, no matches
3. **Performance Tests**: Benchmark with various inputs
4. **Property Tests**: Verify all matches are found
5. **Streaming Tests**: Test chunk boundary handling
6. **Real-World Tests**: Test with code, prose, structured data

---

These three projects comprehensively cover string processing patterns in Rust, teaching zero-copy techniques, efficient data structures (gap buffer), and advanced algorithms (Boyer-Moore), providing orders-of-magnitude performance improvements over naive approaches.
