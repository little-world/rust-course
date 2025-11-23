# Vec & Slice Manipulation

[Pattern 1: Capacity Management and Amortization](#pattern-1-capacity-management-and-amortization)

- Problem: Incremental growth causes repeated reallocations; 100K elements
  = ~17 reallocations
- Solution: Pre-allocate with with_capacity(), reserve() before bulk ops,
  reuse with clear()
- Why It Matters: 10-100x performance improvement; 1M elements: naive ~20
  reallocations, pre-allocated = 1
- Use Cases: Batch processing, query results, temporary buffers in loops,
  large datasets

[Pattern 2: Slice Algorithms](#pattern-2-slice-algorithms)

- Problem: Linear search O(N) when binary O(log N) possible; full sort for
  median wastes time
- Solution: binary_search, partition_point for O(log N);
  select_nth_unstable for O(N) median
- Why It Matters: 1000x difference for 1M elements; median 10-100x faster
  than full sort
- Use Cases: Database query optimization, statistics, deduplication,
  priority queues

[Pattern 3: Chunking and Windowing](#pattern-3-chunking-and-windowing)

- Problem: Element-by-element processing slow; overlapping subsequences
  waste memory
- Solution: .chunks(n) for batches, .windows(n) for overlapping,
  .chunks_exact(n) for uniform
- Why It Matters: 10-50x faster with cache locality; zero allocation for
  moving statistics
- Use Cases: Batch processing, signal processing, image tiles, parallel
  computation

[Pattern 4: Zero-Copy Slicing](#pattern-4-zero-copy-slicing)

- Problem: Parsing allocates per field; 1M CSV × 10 fields = 10M
  allocations
- Solution: Return borrowed slices, use split(), design structs with
  multiple slice views
- Why It Matters: 10-100x faster; 100MB CSV: GB of temps vs constant
  memory
- Use Cases: CSV/JSON parsing, protocol parsers, text processing, binary
  formats

[Pattern 5: SIMD Operations](#pattern-5-simd-operations)

- Problem: Scalar code uses 1 element/instruction when CPUs can do 4-16
- Solution: Use std::simd or packed_simd2; process SIMD-width chunks with
  remainder
- Why It Matters: 4-16x speedups; 1M floats: 1M ops vs 125K ops with
  8-wide
- Use Cases: Image/video processing, audio, numerical computing,
  compression

[Pattern 6: Advanced Slice Patterns](#pattern-6-advanced-slice-patterns)

- Problem: Removing during iteration complex; mutable access to two parts
  violates borrow checker
- Solution: drain() for removal, retain_mut() for in-place filter,
  split_at_mut() for dual access
- Why It Matters: In-place compaction avoids O(N²); split_at_mut enables
  simultaneous work
- Use Cases: Vector compaction, gap buffers, sorting, parallel processing,
  efficient removal

[Vector Cheat Sheet](#vector-and-slices-cheat-sheet)
 - a lot of **vector** functions

## Overview

Vectors and slices are the workhorses of Rust data processing. `Vec<T>` provides dynamic, heap-allocated arrays with amortized O(1) append operations, while slices (`&[T]`, `&mut [T]`) provide views into contiguous sequences without ownership. Understanding how to efficiently manipulate these types is essential for writing high-performance Rust code.

This chapter explores advanced patterns for working with vectors and slices that experienced programmers can leverage to build efficient systems. The key insight is that careful capacity management, zero-copy operations, and algorithmic choices can dramatically impact performance—often by orders of magnitude.

The patterns we'll explore include:
- Capacity management to minimize allocations and optimize amortization
- Slice algorithms for searching, sorting, and partitioning
- Chunking and windowing patterns for batch processing
- Zero-copy slicing techniques to avoid unnecessary allocations
- SIMD operations for data-parallel computation



## Pattern 1: Capacity Management and Amortization

**Problem**: Growing vectors incrementally triggers repeated reallocations—each doubling copies all existing elements. Building a 100K-element vector without pre-allocation causes ~17 reallocations and copies 200K elements total. `Vec::new()` followed by repeated `push()` in loops is a common performance bottleneck. Over-allocating wastes memory for long-lived data structures.

**Solution**: Use `Vec::with_capacity(n)` when size is known upfront. Call `reserve(n)` before bulk operations to pre-allocate space. Reuse vectors with `.clear()` (retains capacity) instead of allocating new ones. Use `shrink_to_fit()` for long-lived vectors where excess capacity wastes memory. Monitor allocation patterns by tracking capacity changes.

**Why It Matters**: Pre-allocation can improve performance by 10-100x for vector construction. A data pipeline building 1M-element results: naive approach does ~20 reallocations copying ~2M elements. Pre-allocated approach: one allocation, zero copies. Memory reuse with `.clear()` eliminates allocation entirely in loops. For real-time systems, avoiding mid-operation reallocations prevents latency spikes.

**Use Cases**: Batch processing (pre-allocate for batch size), collecting query results (reserve based on estimated count), temporary buffers in loops (reuse with clear), building large datasets (with_capacity), long-lived lookup tables (shrink_to_fit after construction).

### Examples

```rust
//=========================================
// Pattern: Pre-allocate when size is known
//=========================================
fn process_batch(items: &[Item]) -> Vec<ProcessedItem> {
    let mut results = Vec::with_capacity(items.len());
    for item in items {
        results.push(process(item));
    }
    results
}

//============================================
// Pattern: Reserve for iterative construction
//============================================
fn build_result_set(queries: &[Query]) -> Vec<Result> {
    let mut results = Vec::new();
    
    // Estimate total size to avoid multiple reallocations
    let estimated_total: usize = queries.iter()
        .map(|q| q.estimated_results())
        .sum();
    
    results.reserve(estimated_total);
    
    for query in queries {
        for result in execute_query(query) {
            results.push(result);
        }
    }
    
    results
}

//===========================================
// Pattern: Reuse vectors to avoid allocation
//===========================================
fn batch_processor(batches: &[Batch]) -> Vec<Vec<Output>> {
    let mut buffer = Vec::with_capacity(1000);
    let mut all_results = Vec::with_capacity(batches.len());
    
    for batch in batches {
        buffer.clear(); // Retains capacity
        
        for item in &batch.items {
            buffer.push(process_item(item));
        }
        
        // Clone only the used portion
        all_results.push(buffer.clone());
    }
    
    all_results
}

//===================================
// Pattern: Amortized growth tracking
//===================================
struct GrowableBuffer<T> {
    data: Vec<T>,
    allocations: usize,
}

impl<T> GrowableBuffer<T> {
    fn new() -> Self {
        GrowableBuffer {
            data: Vec::new(),
            allocations: 0,
        }
    }
    
    fn push(&mut self, value: T) {
        let old_cap = self.data.capacity();
        self.data.push(value);
        let new_cap = self.data.capacity();
        
        if new_cap > old_cap {
            self.allocations += 1;
        }
    }
    
    fn stats(&self) -> (usize, usize, usize) {
        (self.data.len(), self.data.capacity(), self.allocations)
    }
}

//==================================================
// Pattern: Avoid over-allocation with shrink_to_fit
//==================================================
fn build_lookup_table(entries: &[Entry]) -> Vec<IndexEntry> {
    let mut table = Vec::with_capacity(entries.len() * 2); // Over-estimate
    
    for entry in entries {
        if entry.should_index() {
            table.push(IndexEntry::from(entry));
        }
    }
    // Reclaim unused space for long-lived data
    table.shrink_to_fit();
    table
}

//=======================================
// Pattern: Capacity hints for collecting
//=======================================
fn collect_filtered(items: impl Iterator<Item = i32>) -> Vec<i32> {
    // When collecting from iterators, size_hint can optimize allocation
    let (lower, upper) = items.size_hint();
    
    let mut result = if let Some(upper) = upper {
        Vec::with_capacity(upper)
    } else {
        Vec::with_capacity(lower)
    };
    
    result.extend(items);
    result
}

//=====================================
// Pattern: Batch insertion with extend
//=====================================
fn merge_results(target: &mut Vec<String>, sources: &[Vec<String>]) {
    let total: usize = sources.iter().map(|v| v.len()).sum();
    target.reserve(total);
    
    for source in sources {
        target.extend_from_slice(source);
    }
}

//=============================================
// Use case: Building large vectors efficiently
//=============================================
fn generate_dataset(n: usize) -> Vec<DataPoint> {
    let mut data = Vec::with_capacity(n);
    
    for i in 0..n {
        data.push(DataPoint {
            id: i,
            value: compute_value(i),
            metadata: generate_metadata(i),
        });
    }
    
    assert_eq!(data.len(), data.capacity()); // No reallocation occurred
    data
}
```

**Capacity management principles:**
1. **Pre-allocate when size is known**: Use `with_capacity` to avoid reallocations
2. **Reserve before bulk operations**: Prevent multiple reallocations during growth
3. **Reuse vectors with clear()**: Retains capacity for the next use
4. **Shrink long-lived vectors**: Use `shrink_to_fit` to reclaim memory
5. **Track growth for performance analysis**: Monitor reallocation frequency in hot paths

## Pattern 2: Slice Algorithms

**Problem**: Linear search through large sorted arrays is O(N) when O(log N) binary search is possible. Sorting entire datasets to find median or top-K wastes O(N log N) time. In-place partitioning requires manual index juggling and is error-prone. Removing elements from vectors requires shifting elements repeatedly. Unstable vs stable sort choice impacts performance by 2-3x.

**Solution**: Use `binary_search` and `partition_point` for O(log N) searches on sorted data. Apply `select_nth_unstable` for O(N) median/top-K finding without full sort. Use `sort_unstable` for primitives (faster than stable sort). Leverage `rotate_left/right` for efficient cyclic shifts. Use `dedup` on sorted vectors for O(N) deduplication. Apply `retain` for in-place filtered removal.

**Why It Matters**: Algorithm choice dramatically affects performance. Finding an element: linear search O(N) vs binary search O(log N) is 1000x difference for 1M elements. Finding median: full sort O(N log N) vs `select_nth_unstable` O(N) saves 10-100x. Stable sort uses extra memory and runs 2x slower than unstable sort when ordering of equal elements doesn't matter. These built-in slice methods are highly optimized and tested—don't reimplement them.

**Use Cases**: Database query optimization (binary search on sorted indices), statistics computation (median, percentiles with select_nth), data deduplication (sort + dedup), priority queues (partition by priority), cyclic buffers (rotate operations), filtering with memory constraints (retain vs filter+collect).

### Examples

```rust
//===============================================
// Pattern: Binary search (requires sorted slice)
//===============================================
fn find_user_by_id(users: &[User], id: u64) -> Option<&User> {
    // users must be sorted by id
    users.binary_search_by_key(&id, |u| u.id)
        .ok()
        .map(|idx| &users[idx])
}

//===========================================
// Pattern: Partition point for range queries
//===========================================
fn find_range(sorted: &[i32], min: i32, max: i32) -> &[i32] {
    let start = sorted.partition_point(|&x| x < min);
    let end = sorted.partition_point(|&x| x <= max);
    &sorted[start..end]
}

//================================
// Pattern: Partition by predicate
//================================
fn separate_valid_invalid(items: &mut [Item]) -> (&mut [Item], &mut [Item]) {
    let pivot = items.iter().partition_point(|item| item.is_valid());
    items.split_at_mut(pivot)
}

//=================================================
// Pattern: In-place sorting with custom comparator
//=================================================
fn sort_by_priority(tasks: &mut [Task]) {
    tasks.sort_by(|a, b| {
        // Sort by priority descending, then by timestamp ascending
        b.priority.cmp(&a.priority)
            .then_with(|| a.timestamp.cmp(&b.timestamp))
    });
}

//=================================================================================
// Pattern: Unstable sort for performance (doesn't preserve order of equal elements)
//=================================================================================
fn sort_large_dataset(data: &mut [f64]) {
    // sort_unstable is faster than sort for primitive types
    data.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
}

//==================================================
// Pattern: Partial sorting with select_nth_unstable
//==================================================
fn find_median(values: &mut [f64]) -> f64 {
    let mid = values.len() / 2;
    let (_, median, _) = values.select_nth_unstable(mid);
    *median
}

fn top_k_elements(values: &mut [i32], k: usize) -> &[i32] {
    let (_, _, right) = values.select_nth_unstable(values.len() - k);
    right
}

//======================================
// Pattern: Rotate for cyclic operations
//======================================
fn rotate_buffer(buffer: &mut [u8], offset: usize) {
    buffer.rotate_left(offset % buffer.len());
}

//=========================================
// Pattern: Deduplication (requires sorted)
//=========================================
fn unique_sorted(items: &mut Vec<i32>) {
    items.sort_unstable();
    items.dedup();
}

//=========================================
// Pattern: Remove items matching predicate
//=========================================
fn remove_invalid(items: &mut Vec<Item>) {
    items.retain(|item| item.is_valid());
}

//==========================
// Pattern: Reverse in-place
//==========================
fn reverse_segments(data: &mut [u8], segment_size: usize) {
    for chunk in data.chunks_mut(segment_size) {
        chunk.reverse();
    }
}

//============================
// Pattern: Fill and fill_with
//============================
fn initialize_buffer(buffer: &mut [u8], pattern: u8) {
    buffer.fill(pattern);
}

fn initialize_with_indices(buffer: &mut [usize]) {
    let mut counter = 0;
    buffer.fill_with(|| {
        let val = counter;
        counter += 1;
        val
    });
}

//==============================
// Pattern: Swap and swap ranges
//==============================
fn swap_halves(data: &mut [u8]) {
    let mid = data.len() / 2;
    let (left, right) = data.split_at_mut(mid);
    let min_len = left.len().min(right.len());
    left[..min_len].swap_with_slice(&mut right[..min_len]);
}

//=======================================================
// Pattern: Contains and starts_with for pattern matching
//=======================================================
fn has_magic_header(data: &[u8]) -> bool {
    const MAGIC: &[u8] = b"PNG\x89";
    data.starts_with(MAGIC)
}

//=======================================
// Pattern: Find subsequence with windows
//=======================================
fn find_pattern(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len())
        .position(|window| window == needle)
}
```

**Slice algorithm guidelines:**
1. **Use binary_search for sorted data**: O(log n) vs O(n) for linear search
2. **Prefer sort_unstable for primitives**: Faster than stable sort when order doesn't matter
3. **Use select_nth_unstable for top-k**: O(n) vs O(n log n) for full sort
4. **Partition instead of filter+collect**: Avoids allocation
5. **Use rotate for cyclic shifts**: More efficient than manual copying

## Pattern 3: Chunking and Windowing

**Problem**: Processing large datasets element-by-element is slow due to function call overhead and poor cache locality. Batch operations require manual index calculation and are error-prone. Computing moving averages or detecting patterns requires overlapping subsequences—collecting them into vectors wastes memory. Remainder handling after chunking leads to off-by-one errors.

**Solution**: Use `.chunks(n)` for non-overlapping fixed-size batches. Use `.windows(n)` for overlapping subsequences (moving averages, pattern detection). Apply `.chunks_exact(n)` when you need uniform chunks with explicit remainder handling. Use `.chunks_mut(n)` for in-place batch transformations. Leverage `.rchunks(n)` for reverse-order processing. Combine with `.step_by(n)` for strided access.

**Why It Matters**: Chunking improves cache locality—processing 1000-element chunks instead of individual elements can be 10-50x faster. Window operations enable signal processing algorithms (FFT, convolution) without collecting intermediate vectors—zero allocation for moving statistics. Chunks enable trivial parallelization: split data into N chunks, process on N threads. These abstractions prevent index errors that plague manual chunking code.

**Use Cases**: Batch processing (database inserts, API requests), signal processing (moving averages, FFT windows), image processing (tile-based operations), parallel computation (divide work across threads), network packet assembly (fixed-size frames), time-series analysis (rolling statistics).

### Examples

```rust
//===========================
// Pattern: Fixed-size chunks
//===========================
fn process_in_batches(data: &[u8], batch_size: usize) -> Vec<ProcessedBatch> {
    data.chunks(batch_size)
        .map(|chunk| process_batch(chunk))
        .collect()
}

//====================================================
// Pattern: Mutable chunks for in-place transformation
//====================================================
fn normalize_batches(data: &mut [f64], batch_size: usize) {
    for chunk in data.chunks_mut(batch_size) {
        let sum: f64 = chunk.iter().sum();
        let mean = sum / chunk.len() as f64;
        
        for value in chunk {
            *value -= mean;
        }
    }
}

//==============================================
// Pattern: Exact chunks with remainder handling
//==============================================
fn process_with_remainder(data: &[u8], chunk_size: usize) {
    let (chunks, remainder) = data.as_chunks::<8>();
    
    for chunk in chunks {
        // Process full 8-byte chunks
        process_aligned_chunk(chunk);
    }
    
    if !remainder.is_empty() {
        // Handle remaining bytes
        process_partial_chunk(remainder);
    }
}

//=========================
// Pattern: Sliding windows
//=========================
fn moving_average(values: &[f64], window_size: usize) -> Vec<f64> {
    values.windows(window_size)
        .map(|window| {
            let sum: f64 = window.iter().sum();
            sum / window.len() as f64
        })
        .collect()
}

//==========================================
// Pattern: Pairwise operations with windows
//==========================================
fn compute_deltas(values: &[i32]) -> Vec<i32> {
    values.windows(2)
        .map(|pair| pair[1] - pair[0])
        .collect()
}

//======================================
// Pattern: Overlapping pattern matching
//======================================
fn find_consecutive_sequences(data: &[i32], target: i32) -> Vec<usize> {
    data.windows(3)
        .enumerate()
        .filter_map(|(i, window)| {
            if window.iter().all(|&x| x == target) {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

//=========================================
// Pattern: Chunk-based parallel processing
//=========================================
fn parallel_sum(data: &[i64]) -> i64 {
    use std::thread;
    
    let chunk_size = data.len() / num_cpus::get();
    let handles: Vec<_> = data.chunks(chunk_size)
        .map(|chunk| {
            let chunk = chunk.to_vec();
            thread::spawn(move || chunk.iter().sum::<i64>())
        })
        .collect();
    
    handles.into_iter()
        .map(|h| h.join().unwrap())
        .sum()
}

//=======================================
// Pattern: rchunks for reverse iteration
//=======================================
fn process_backwards(data: &[u8], chunk_size: usize) {
    for chunk in data.rchunks(chunk_size) {
        process_chunk(chunk);
    }
}

//================================
// Pattern: chunks_exact vs chunks
//================================
fn encode_blocks(data: &[u8], block_size: usize) -> Vec<EncodedBlock> {
    let mut encoded = Vec::new();
    
    // Process full blocks
    for block in data.chunks_exact(block_size) {
        encoded.push(encode_full_block(block));
    }
    
    // Handle remainder
    let remainder = &data[data.len() - (data.len() % block_size)..];
    if !remainder.is_empty() {
        encoded.push(encode_partial_block(remainder));
    }
    
    encoded
}

//=====================================
// Pattern: Strided access with step_by
//=====================================
fn sample_every_nth(data: &[f64], n: usize) -> Vec<f64> {
    data.iter()
        .step_by(n)
        .copied()
        .collect()
}

//================================
// Pattern: Split into equal parts
//================================
fn split_into_n_parts(data: &[u8], n: usize) -> Vec<&[u8]> {
    let chunk_size = (data.len() + n - 1) / n; // Ceiling division
    data.chunks(chunk_size).collect()
}

//=====================================================
// Use case: Signal processing with overlapping windows
//=====================================================
fn compute_spectrogram(signal: &[f32], window_size: usize, hop_size: usize) -> Vec<Vec<f32>> {
    let mut result = Vec::new();
    
    for i in (0..signal.len()).step_by(hop_size) {
        if i + window_size <= signal.len() {
            let window = &signal[i..i + window_size];
            let spectrum = fft(window);
            result.push(spectrum);
        }
    }
    
    result
}
```

**Chunking and windowing principles:**
1. **Use chunks for batch processing**: Non-overlapping segments
2. **Use windows for sliding operations**: Moving averages, pattern detection
3. **Handle remainders explicitly**: chunks_exact makes remainder handling clear
4. **Prefer chunks over manual indexing**: More idiomatic and less error-prone
5. **Consider rchunks for reverse processing**: Cleaner than reversing then chunking

## Pattern 4: Zero-Copy Slicing

**Problem**: Parsing structured data by extracting fields into owned `String`/`Vec` causes allocation for every field—parsing 1M CSV records with 10 fields allocates 10M strings. Returning data from functions forces cloning entire vectors. Protocol parsing allocates separate buffers for headers, payloads, and metadata. These allocations dominate parsing performance.

**Solution**: Return borrowed slices (`&[T]`, `&str`) that reference the original data. Use `split()`, `split_at()`, and range indexing to create views. Design data structures that hold multiple slices into one allocation. Use lifetime parameters to tie slices to source data. Parse in-place by returning references rather than owned copies.

**Why It Matters**: Zero-copy parsing can be 10-100x faster than allocating approach. Parsing a 100MB CSV file: allocating approach needs gigabytes of temporary memory and causes GC pressure. Zero-copy approach uses constant memory and eliminates allocation overhead entirely. Network protocol parsing benefits dramatically—handling 1M requests/second becomes feasible. The borrow checker ensures slices can't outlive data, making this safe.

**Use Cases**: CSV/JSON parsing (return field slices), network protocol parsers (split packets into views), text processing (split without allocation), binary format parsing (frame headers/payloads), configuration file parsing, streaming data processors.

### Examples

```rust
//==========================================
// Pattern: Return slices instead of cloning
//==========================================
fn find_field<'a>(record: &'a [u8], field_index: usize) -> &'a [u8] {
    let mut start = 0;
    let mut current_field = 0;
    
    for (i, &byte) in record.iter().enumerate() {
        if byte == b',' {
            if current_field == field_index {
                return &record[start..i];
            }
            current_field += 1;
            start = i + 1;
        }
    }
    
    if current_field == field_index {
        &record[start..]
    } else {
        &[]
    }
}

//==================================
// Pattern: Split without allocation
//==================================
fn parse_csv_line(line: &str) -> Vec<&str> {
    line.split(',')
        .map(|s| s.trim())
        .collect()
}

//=============================================
// Pattern: Multiple slices from one allocation
//=============================================
struct Frame<'a> {
    header: &'a [u8],
    payload: &'a [u8],
    checksum: &'a [u8],
}

impl<'a> Frame<'a> {
    fn parse(data: &'a [u8]) -> Result<Self, ParseError> {
        if data.len() < 10 {
            return Err(ParseError::TooShort);
        }
        
        Ok(Frame {
            header: &data[0..4],
            payload: &data[4..data.len() - 4],
            checksum: &data[data.len() - 4..],
        })
    }
}

//=================================
// Pattern: Borrowing with split_at
//=================================
fn process_header_and_body(data: &[u8]) -> Result<(Header, Vec<Item>), Error> {
    let (header_bytes, body) = data.split_at(HEADER_SIZE);
    let header = parse_header(header_bytes)?;
    let items = parse_body(body)?;
    Ok((header, items))
}

//========================================
// Pattern: Cow for conditional allocation
//========================================
use std::borrow::Cow;

fn decode_field(field: &[u8]) -> Cow<str> {
    match std::str::from_utf8(field) {
        Ok(s) => Cow::Borrowed(s),
        Err(_) => {
            // Only allocate when we need to fix encoding
            Cow::Owned(String::from_utf8_lossy(field).into_owned())
        }
    }
}

//=========================================================
// Pattern: split_first and split_last for protocol parsing
//=========================================================
fn parse_packet(data: &[u8]) -> Result<Packet, ParseError> {
    let (&version, rest) = data.split_first()
        .ok_or(ParseError::Empty)?;
    
    let (payload, &checksum) = rest.split_last()
        .ok_or(ParseError::NoChecksum)?;
    
    Ok(Packet { version, payload, checksum })
}

//======================================
// Pattern: Iterating without collecting
//======================================
fn sum_valid_numbers(data: &str) -> i32 {
    data.split(',')
        .filter_map(|s| s.trim().parse::<i32>().ok())
        .sum()
}

//==================================
// Pattern: Slicing during iteration
//==================================
fn process_blocks(data: &[u8], block_size: usize) -> Vec<BlockResult> {
    (0..data.len())
        .step_by(block_size)
        .map(|i| {
            let end = (i + block_size).min(data.len());
            process_block(&data[i..end])
        })
        .collect()
}

//============================================
// Pattern: Split and group without allocation
//============================================
fn group_by_delimiter(data: &[u8], delimiter: u8) -> Vec<&[u8]> {
    let mut groups = Vec::new();
    let mut start = 0;
    
    for (i, &byte) in data.iter().enumerate() {
        if byte == delimiter {
            groups.push(&data[start..i]);
            start = i + 1;
        }
    }
    
    if start < data.len() {
        groups.push(&data[start..]);
    }
    
    groups
}

//=======================================================
// Pattern: Mutable slice operations without reallocation
//=======================================================
fn swap_bytes_in_place(data: &mut [u8]) {
    for pair in data.chunks_exact_mut(2) {
        pair.swap(0, 1);
    }
}

//=====================================
// Use case: Zero-copy protocol parsing
//=====================================
struct HttpRequest<'a> {
    method: &'a str,
    path: &'a str,
    headers: Vec<(&'a str, &'a str)>,
    body: &'a [u8],
}

impl<'a> HttpRequest<'a> {
    fn parse(data: &'a [u8]) -> Result<Self, ParseError> {
        let data_str = std::str::from_utf8(data)
            .map_err(|_| ParseError::InvalidUtf8)?;
        
        let (head, body) = data_str.split_once("\r\n\r\n")
            .ok_or(ParseError::NoBodySeparator)?;
        
        let mut lines = head.lines();
        let request_line = lines.next().ok_or(ParseError::Empty)?;
        
        let mut parts = request_line.split_whitespace();
        let method = parts.next().ok_or(ParseError::NoMethod)?;
        let path = parts.next().ok_or(ParseError::NoPath)?;
        
        let headers: Vec<_> = lines
            .filter_map(|line| line.split_once(": "))
            .collect();
        
        Ok(HttpRequest {
            method,
            path,
            headers,
            body: body.as_bytes(),
        })
    }
}
```

**Zero-copy principles:**
1. **Return slices when possible**: Avoid cloning unless necessary
2. **Use split methods**: split, split_at, split_once for parsing
3. **Leverage Cow for conditional cloning**: Only allocate when modification is needed
4. **Parse in-place**: Use iterators and slices instead of collecting
5. **Design APIs for borrowing**: Accept &[T] instead of Vec<T> when ownership isn't needed

## Pattern 5: SIMD Operations

**Problem**: Processing large arrays element-by-element leaves CPU vector units idle—modern CPUs can process 4-16 elements per instruction but scalar code uses only one. Image processing, audio encoding, numerical computing, and data compression are bottlenecked by sequential processing. Writing SIMD intrinsics directly is platform-specific and unsafe.

**Solution**: Use portable SIMD through `std::simd` (nightly) or crates like `packed_simd2`. Process data in SIMD-width chunks (4/8/16 elements). Use `as_chunks()` or `chunks_exact()` to separate aligned chunks from remainder. Apply SIMD operations (add, multiply, compare) across vector lanes simultaneously. Handle remainder with scalar code.

**Why It Matters**: SIMD provides 4-16x speedups for data-parallel operations. Processing 1M floats: scalar takes 1M operations, SIMD with 8-wide vectors takes 125K operations. Image processing (applying filters, color conversion) becomes 10x faster. Checksum computation, compression, and cryptography all benefit. Rust's portable SIMD compiles to optimal instructions for target CPU without unsafe code, unlike C intrinsics.

**Use Cases**: Image/video processing (filters, transformations), audio processing (effects, encoding), numerical computing (matrix operations, scientific simulations), compression algorithms, checksums and hashing, database query execution, machine learning inference.

### Examples

```rust
//=======================================
// Pattern: Manual SIMD with chunks_exact
//=======================================
fn sum_bytes(data: &[u8]) -> u64 {
    let (chunks, remainder) = data.as_chunks::<8>();
    
    let mut sum = 0u64;
    
    // Process 8 bytes at a time
    for chunk in chunks {
        for &byte in chunk {
            sum += byte as u64;
        }
    }
    
    // Handle remainder
    for &byte in remainder {
        sum += byte as u64;
    }
    
    sum
}

//=====================================
// Pattern: Aligned operations for SIMD
//=====================================
#[repr(align(32))]
struct AlignedBuffer([f32; 8]);

fn process_aligned(data: &[AlignedBuffer]) -> Vec<f32> {
    data.iter()
        .flat_map(|buf| buf.0.iter())
        .map(|&x| x * 2.0)
        .collect()
}

//===========================================
// Pattern: Using std::simd (nightly feature)
//===========================================
#[cfg(feature = "portable_simd")]
fn add_vectors_simd(a: &[f32], b: &[f32], result: &mut [f32]) {
    use std::simd::*;
    
    let lanes = 4;
    let (a_chunks, a_remainder) = a.as_chunks::<4>();
    let (b_chunks, b_remainder) = b.as_chunks::<4>();
    let (result_chunks, result_remainder) = result.as_chunks_mut::<4>();
    
    // SIMD processing
    for ((a_chunk, b_chunk), result_chunk) in 
        a_chunks.iter().zip(b_chunks).zip(result_chunks) 
    {
        let a_simd = f32x4::from_array(*a_chunk);
        let b_simd = f32x4::from_array(*b_chunk);
        let sum = a_simd + b_simd;
        *result_chunk = sum.to_array();
    }
    
    // Handle remainder
    for ((a, b), result) in 
        a_remainder.iter().zip(b_remainder).zip(result_remainder) 
    {
        *result = a + b;
    }
}

//===========================
// Pattern: Vectorized search
//===========================
fn find_byte_simd(haystack: &[u8], needle: u8) -> Option<usize> {
    let (chunks, remainder) = haystack.as_chunks::<16>();
    
    for (i, chunk) in chunks.iter().enumerate() {
        for (j, &byte) in chunk.iter().enumerate() {
            if byte == needle {
                return Some(i * 16 + j);
            }
        }
    }
    
    let offset = chunks.len() * 16;
    remainder.iter()
        .position(|&b| b == needle)
        .map(|pos| offset + pos)
}

//========================
// Pattern: SIMD reduction
//========================
fn sum_f32_vectorized(data: &[f32]) -> f32 {
    let (chunks, remainder) = data.as_chunks::<8>();
    
    let mut sums = [0.0f32; 8];
    
    for chunk in chunks {
        for (i, &value) in chunk.iter().enumerate() {
            sums[i] += value;
        }
    }
    
    let chunk_sum: f32 = sums.iter().sum();
    let remainder_sum: f32 = remainder.iter().sum();
    
    chunk_sum + remainder_sum
}

//==================================
// Pattern: Parallel SIMD with rayon
//==================================
#[cfg(feature = "rayon")]
fn parallel_simd_transform(data: &mut [f32]) {
    use rayon::prelude::*;
    
    data.par_chunks_mut(1024)
        .for_each(|chunk| {
            for value in chunk {
                *value = value.sqrt();
            }
        });
}

//===========================================
// Pattern: Auto-vectorization with iterators
//===========================================
fn scale_values(data: &mut [f32], scale: f32) {
    // Compiler can auto-vectorize this loop
    for value in data {
        *value *= scale;
    }
}

//======================================
// Pattern: Explicit SIMD with as_chunks
//======================================
fn dot_product_chunks(a: &[f32], b: &[f32]) -> f32 {
    let (a_chunks, a_rem) = a.as_chunks::<4>();
    let (b_chunks, b_rem) = b.as_chunks::<4>();
    
    let mut sum = 0.0;
    
    // Process 4 elements at a time
    for (a_chunk, b_chunk) in a_chunks.iter().zip(b_chunks) {
        for i in 0..4 {
            sum += a_chunk[i] * b_chunk[i];
        }
    }
    
    // Handle remainder
    for (a_val, b_val) in a_rem.iter().zip(b_rem) {
        sum += a_val * b_val;
    }
    
    sum
}

//=====================================
// Use case: Image processing with SIMD
//=====================================
fn grayscale_simd(rgb_data: &[u8], output: &mut [u8]) {
    assert_eq!(rgb_data.len() % 3, 0);
    assert_eq!(output.len(), rgb_data.len() / 3);
    
    for (i, pixel) in rgb_data.chunks_exact(3).enumerate() {
        let r = pixel[0] as u32;
        let g = pixel[1] as u32;
        let b = pixel[2] as u32;
        
        // Weighted average for luminance
        let gray = ((r * 77 + g * 150 + b * 29) >> 8) as u8;
        output[i] = gray;
    }
}

//================================
// Pattern: Memory layout for SIMD
//================================
#[repr(C)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

fn normalize_vectors(vectors: &mut [Vec3]) {
    for v in vectors {
        let len = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
        if len > 0.0 {
            v.x /= len;
            v.y /= len;
            v.z /= len;
        }
    }
}
```

**SIMD principles:**
1. **Use as_chunks for aligned access**: Processes fixed-size chunks efficiently
2. **Handle remainders explicitly**: Don't forget the last few elements
3. **Align data structures**: Use #[repr(align(N))] for better SIMD performance
4. **Let the compiler auto-vectorize**: Simple loops often vectorize automatically
5. **Profile before optimizing**: SIMD isn't always faster for small datasets

## Pattern 6: Advanced Slice Patterns

**Problem**: Removing elements during iteration requires complex index tracking. Transforming vectors in-place while filtering requires multiple passes. Safe mutable access to two separate parts of a slice violates borrow checker rules. Gap buffer operations and split-at-mut patterns need careful manual implementation. Type conversions between slice representations cause unnecessary copies.

**Solution**: Use `drain()` for removing ranges while iterating. Apply `retain_mut()` for in-place filtering with mutation. Use `split_at_mut()` to safely get two mutable sub-slices. Leverage `swap()` and `swap_with_slice()` for in-place rearrangement. Apply `from_raw_parts()` and transmute (unsafe) for zero-cost slice type conversions when layout-compatible. Use `split_array_ref` for compile-time length checking.

**Why It Matters**: These patterns enable complex transformations without temporary allocations. In-place compaction with swap avoids O(N²) repeated deletions. `split_at_mut` lets you work on slice parts simultaneously—impossible with single mutable borrow. Drain enables efficient element removal while processing remaining elements. Understanding these advanced patterns is the difference between elegant solutions and fighting the borrow checker with workarounds.

**Use Cases**: In-place vector compaction, gap buffer implementations, sorting implementations with pivot splitting, parallel processing with split_at_mut, memory-efficient filtering, zero-copy type conversions (e.g., &[u8] to &[u32]), efficient element removal patterns.

### Examples

```rust
//============================================
// Pattern: In-place transformation with drain
//============================================
fn compact_vector(vec: &mut Vec<Item>) {
    let mut write_index = 0;
    
    for read_index in 0..vec.len() {
        if vec[read_index].should_keep() {
            if read_index != write_index {
                vec.swap(read_index, write_index);
            }
            write_index += 1;
        }
    }
    
    vec.truncate(write_index);
}

//===========================
// Pattern: drain with filter
//===========================
fn extract_matching(vec: &mut Vec<Item>, predicate: impl Fn(&Item) -> bool) -> Vec<Item> {
    let mut extracted = Vec::new();
    let mut i = 0;
    
    while i < vec.len() {
        if predicate(&vec[i]) {
            extracted.push(vec.remove(i));
        } else {
            i += 1;
        }
    }
    
    extracted
}

//=====================================
// Pattern: splice for replacing ranges
//=====================================
fn replace_range(vec: &mut Vec<i32>, start: usize, end: usize, replacement: &[i32]) {
    vec.splice(start..end, replacement.iter().copied());
}

//============================================
// Pattern: Efficient insertion with split_off
//============================================
fn insert_slice_at(vec: &mut Vec<u8>, index: usize, data: &[u8]) {
    let tail = vec.split_off(index);
    vec.extend_from_slice(data);
    vec.extend_from_slice(&tail);
}

//========================================
// Pattern: Deque operations with VecDeque
//========================================
use std::collections::VecDeque;

fn sliding_window_max(values: &[i32], window_size: usize) -> Vec<i32> {
    let mut result = Vec::new();
    let mut deque = VecDeque::new();
    
    for (i, &value) in values.iter().enumerate() {
        // Remove elements outside window
        while deque.front().map_or(false, |&idx| idx <= i.saturating_sub(window_size)) {
            deque.pop_front();
        }
        
        // Remove smaller elements from back
        while deque.back().map_or(false, |&idx| values[idx] < value) {
            deque.pop_back();
        }
        
        deque.push_back(i);
        
        if i >= window_size - 1 {
            result.push(values[*deque.front().unwrap()]);
        }
    }
    
    result
}

//=======================================
// Pattern: Circular buffer with wrapping
//=======================================
struct CircularBuffer<T> {
    data: Vec<T>,
    head: usize,
    tail: usize,
    size: usize,
}

impl<T: Default + Clone> CircularBuffer<T> {
    fn new(capacity: usize) -> Self {
        CircularBuffer {
            data: vec![T::default(); capacity],
            head: 0,
            tail: 0,
            size: 0,
        }
    }
    
    fn push(&mut self, item: T) {
        self.data[self.tail] = item;
        self.tail = (self.tail + 1) % self.data.len();
        
        if self.size < self.data.len() {
            self.size += 1;
        } else {
            self.head = (self.head + 1) % self.data.len();
        }
    }
    
    fn as_slices(&self) -> (&[T], &[T]) {
        if self.head <= self.tail {
            (&self.data[self.head..self.tail], &[])
        } else {
            (&self.data[self.head..], &self.data[..self.tail])
        }
    }
}

//==================================
// Pattern: Copy-free slice swapping
//==================================
fn interleave_slices(a: &mut [u8], b: &mut [u8]) {
    assert_eq!(a.len(), b.len());
    
    for (a_val, b_val) in a.iter_mut().zip(b.iter_mut()) {
        std::mem::swap(a_val, b_val);
    }
}

//==============================================
// Pattern: Batch commit with extend_from_within
//==============================================
fn duplicate_segment(vec: &mut Vec<u8>, start: usize, end: usize) {
    vec.extend_from_within(start..end);
}

//============================
// Pattern: Partition in-place
//============================
fn partition_by_sign(values: &mut [i32]) -> usize {
    let mut left = 0;
    let mut right = values.len();
    
    while left < right {
        if values[left] >= 0 {
            left += 1;
        } else {
            right -= 1;
            values.swap(left, right);
        }
    }
    
    left
}

//===================================================
// Pattern: Three-way partition (Dutch National Flag)
//===================================================
fn partition_three_way(values: &mut [i32], pivot: i32) -> (usize, usize) {
    let mut low = 0;
    let mut mid = 0;
    let mut high = values.len();
    
    while mid < high {
        if values[mid] < pivot {
            values.swap(low, mid);
            low += 1;
            mid += 1;
        } else if values[mid] > pivot {
            high -= 1;
            values.swap(mid, high);
        } else {
            mid += 1;
        }
    }
    
    (low, high)
}
```

**Advanced patterns guidelines:**
1. **Use drain for selective removal**: More efficient than repeated remove()
2. **Prefer retain over drain+filter**: Built-in and optimized
3. **Use VecDeque for double-ended operations**: Better than Vec for queue operations
4. **Implement circular buffers for fixed-size windows**: Avoid shifting elements
5. **Partition in-place when possible**: Avoids allocation

## Summary

Vectors and slices are fundamental to Rust data processing. By mastering capacity management, algorithmic operations, chunking patterns, zero-copy techniques, and SIMD optimizations, you can write high-performance code that competes with C/C++ while maintaining Rust's safety guarantees.

**Key takeaways:**
1. Pre-allocate with `with_capacity` to eliminate reallocations
2. Use slice algorithms (binary_search, partition_point, sort_unstable) for common operations
3. Leverage chunks and windows for batch processing and signal processing
4. Return slices instead of cloning to achieve zero-copy parsing
5. Use as_chunks and SIMD-friendly layouts for data-parallel operations
6. Profile your code—premature optimization often leads to complexity without gains

**Performance dos and don'ts:**

✓ **Do:**
- Pre-allocate when size is known or estimable
- Use `sort_unstable` for primitives
- Process data in chunks for better cache locality
- Return slices instead of cloning
- Let the compiler auto-vectorize simple loops

✗ **Don't:**
- Use `push` in loops without pre-allocating
- Call `shrink_to_fit` on frequently modified vectors
- Clone large slices when borrowing suffices
- Implement manual SIMD without profiling first
- Forget to handle remainders in chunked operations

Vector and slice manipulation is about understanding trade-offs: allocation cost vs memory usage, stable vs unstable sorting, copying vs borrowing. Master these patterns to build efficient, idiomatic Rust systems that maximize performance while preserving safety.


## Vector and Slices Cheat Sheet
```rust
// ===== CREATING VECTORS =====
// Empty vector
let v: Vec<i32> = Vec::new();                       // Empty Vec
let v = Vec::<i32>::new();                          // With turbofish
let v: Vec<i32> = vec![];                           // Empty with macro

// With initial values
let v = vec![1, 2, 3, 4, 5];                        // Vec macro
let v = vec![0; 5];                                 // [0, 0, 0, 0, 0]

// With capacity
let mut v = Vec::with_capacity(10);                 // Pre-allocate space

// From array
let v = Vec::from([1, 2, 3]);                       // From array
let v: Vec<i32> = [1, 2, 3].to_vec();              // Array to Vec

// From iterator
let v: Vec<_> = (0..10).collect();                  // Collect from range
let v: Vec<_> = "hello".chars().collect();         // Collect chars

// From slice
let slice = &[1, 2, 3];
let v = slice.to_vec();                             // Clone slice to Vec

// ===== VECTOR CAPACITY =====
let mut v = Vec::with_capacity(10);
v.len()                                             // Number of elements
v.capacity()                                        // Allocated capacity
v.is_empty()                                        // Check if empty

v.reserve(20)                                       // Reserve additional space
v.reserve_exact(20)                                 // Reserve exact space
v.shrink_to_fit()                                   // Reduce capacity to len
v.shrink_to(5)                                      // Shrink to at least n capacity

// ===== ADDING ELEMENTS =====
let mut v = vec![1, 2, 3];

v.push(4)                                           // Add to end
v.append(&mut vec![5, 6])                          // Append another Vec (moves)
v.extend([7, 8, 9])                                // Extend from iterable
v.extend_from_slice(&[10, 11])                     // Extend from slice

v.insert(0, 0)                                      // Insert at index
v.insert(v.len(), 99)                              // Insert at end

// ===== REMOVING ELEMENTS =====
let mut v = vec![1, 2, 3, 4, 5];

v.pop()                                             // Remove last: Option<T>
v.remove(2)                                         // Remove at index: T (shifts elements)
v.swap_remove(2)                                    // Remove at index: T (swaps with last, O(1))

v.clear()                                           // Remove all elements
v.truncate(3)                                       // Keep only first n elements

v.retain(|x| *x > 2)                               // Keep elements matching predicate
v.retain_mut(|x| { *x *= 2; *x > 4 })            // Retain with mutation

v.dedup()                                           // Remove consecutive duplicates
v.dedup_by(|a, b| a == b)                          // Dedup with custom comparison
v.dedup_by_key(|x| *x)                             // Dedup by key function

// Drain - remove and return iterator
let drained: Vec<_> = v.drain(1..3).collect();     // Remove range
let all: Vec<_> = v.drain(..).collect();           // Remove all (empties vec)
v.drain_filter(|x| *x % 2 == 0);                   // Remove matching (nightly)

// ===== ACCESSING ELEMENTS =====
let v = vec![1, 2, 3, 4, 5];

v[0]                                                // Index access (panics if out of bounds)
v.get(0)                                            // Safe access: Option<&T>
v.get(10)                                           // Returns None if out of bounds
v.get_mut(0)                                        // Mutable access: Option<&mut T>

v.first()                                           // First element: Option<&T>
v.last()                                            // Last element: Option<&T>
v.first_mut()                                       // Mutable first: Option<&mut T>
v.last_mut()                                        // Mutable last: Option<&mut T>

// ===== SLICING =====
let v = vec![1, 2, 3, 4, 5];

&v[..]                                              // Full slice
&v[1..4]                                            // Range: [2, 3, 4]
&v[..3]                                             // First 3: [1, 2, 3]
&v[2..]                                             // From index 2: [3, 4, 5]
&v[1..=3]                                           // Inclusive range: [2, 3, 4]

v.get(1..4)                                         // Safe slice: Option<&[T]>
v.get_mut(1..4)                                     // Mutable slice: Option<&mut [T]>

// ===== SLICE OPERATIONS =====
let slice = &[1, 2, 3, 4, 5];

slice.len()                                         // Length
slice.is_empty()                                    // Check if empty
slice.first()                                       // First element: Option<&T>
slice.last()                                        // Last element: Option<&T>

slice.split_first()                                 // (first, rest): Option<(&T, &[T])>
slice.split_last()                                  // (last, rest): Option<(&T, &[T])>

// Splitting
slice.split_at(2)                                   // Split at index: (&[T], &[T])
slice.split(|x| *x == 3)                           // Split by predicate: Iterator
slice.splitn(2, |x| *x == 3)                       // Split n times
slice.split_inclusive(|x| *x == 3)                 // Include delimiter in chunks
slice.rsplit(|x| *x == 3)                          // Split from right

// Chunks
slice.chunks(2)                                     // [[1,2], [3,4], [5]]
slice.chunks_exact(2)                              // [[1,2], [3,4]] (no remainder)
slice.rchunks(2)                                    // Chunks from right
slice.windows(3)                                    // [[1,2,3], [2,3,4], [3,4,5]]

// Mutable versions
let mut v = vec![1, 2, 3, 4, 5];
let slice = &mut v[..];
slice.split_at_mut(2)                              // Mutable split
slice.chunks_mut(2)                                // Mutable chunks
slice.split_first_mut()                            // Mutable split first

// ===== SEARCHING =====
let v = vec![1, 2, 3, 4, 5];

v.contains(&3)                                      // Check if contains value
v.binary_search(&3)                                // Binary search: Result<usize, usize>
v.binary_search_by(|x| x.cmp(&3))                 // Binary search with comparator
v.binary_search_by_key(&3, |x| *x)                // Binary search by key

v.starts_with(&[1, 2])                             // Check prefix
v.ends_with(&[4, 5])                               // Check suffix

// ===== SORTING =====
let mut v = vec![3, 1, 4, 1, 5, 9];

v.sort()                                            // Sort in place (stable)
v.sort_unstable()                                   // Unstable sort (faster)
v.sort_by(|a, b| a.cmp(b))                         // Sort with comparator
v.sort_by(|a, b| b.cmp(a))                         // Reverse sort
v.sort_by_key(|x| *x)                              // Sort by key function
v.sort_by_cached_key(|x| expensive(*x))           // Cache key computations

v.reverse()                                         // Reverse in place

// Check if sorted
v.is_sorted()                                       // Check if sorted
v.is_sorted_by(|a, b| a <= b)                     // Check with comparator
v.is_sorted_by_key(|x| *x)                        // Check by key

// ===== REORDERING =====
let mut v = vec![1, 2, 3, 4, 5];

v.swap(0, 4)                                        // Swap elements at indices
v.rotate_left(2)                                    // Rotate left: [3,4,5,1,2]
v.rotate_right(2)                                   // Rotate right
v.reverse()                                         // Reverse in place

// ===== FILLING =====
let mut v = vec![0; 5];

v.fill(42)                                          // Fill with value: [42,42,42,42,42]
v.fill_with(|| rand::random())                     // Fill with function

// ===== COPYING AND CLONING =====
let v = vec![1, 2, 3];

v.clone()                                           // Deep clone
v.to_vec()                                          // Clone to new Vec (for slices)

// Copy from slice
let mut dest = vec![0; 5];
let src = [1, 2, 3];
dest[..3].copy_from_slice(&src)                    // Copy exact slice
dest.clone_from_slice(&src)                        // Clone from slice

// ===== COMPARING =====
let v1 = vec![1, 2, 3];
let v2 = vec![1, 2, 3];

v1 == v2                                            // Equality
v1 != v2                                            // Inequality
v1 < v2                                             // Lexicographic comparison
v1.cmp(&v2)                                         // Ordering

// ===== ITERATION =====
let v = vec![1, 2, 3, 4, 5];

for x in &v {                                       // Borrow elements
    println!("{}", x);
}

for x in &mut v {                                   // Mutable borrow
    *x *= 2;
}

for x in v {                                        // Consume vector
    println!("{}", x);
}

// Iterator methods
v.iter()                                            // Iterator over &T
v.iter_mut()                                        // Iterator over &mut T
v.into_iter()                                       // Iterator over T (consume)

// ===== TRANSFORMING =====
let v = vec![1, 2, 3, 4, 5];

// Map to new vector
let doubled: Vec<_> = v.iter().map(|x| x * 2).collect();

// Filter
let evens: Vec<_> = v.iter().filter(|&&x| x % 2 == 0).collect();

// Partition
let (even, odd): (Vec<_>, Vec<_>) = v.iter()
    .partition(|&&x| x % 2 == 0);

// ===== JOINING AND SPLITTING =====
let v = vec![vec![1, 2], vec![3, 4], vec![5]];

// Flatten
let flat: Vec<_> = v.into_iter().flatten().collect(); // [1,2,3,4,5]
let flat: Vec<_> = v.concat();                     // Concatenate all

// Join with separator (for slices)
let v = vec![vec![1, 2], vec![3, 4]];
let joined = v.join(&0);                           // [1,2,0,3,4]

// ===== CONVERTING =====
let v = vec![1, 2, 3];

// To array (requires const generic)
let arr: [i32; 3] = v.try_into().unwrap();         // Vec to array
let arr: [i32; 3] = v[..3].try_into().unwrap();   // Slice to array

// To boxed slice
let boxed: Box<[i32]> = v.into_boxed_slice();      // Vec to Box<[T]>
let v: Vec<i32> = boxed.into_vec();                // Box<[T]> to Vec

// To string (for u8)
let bytes = vec![72, 101, 108, 108, 111];
let s = String::from_utf8(bytes).unwrap();         // Vec<u8> to String
let bytes = s.into_bytes();                        // String to Vec<u8>

// ===== SPECIAL OPERATIONS =====
let mut v = vec![1, 2, 3, 4, 5];

// Splice - replace range
v.splice(1..3, vec![10, 20]);                      // Replace [2,3] with [10,20]

// Leak - get static slice
let leak: &'static [i32] = v.leak();               // Leak memory, get static ref

// Resize
v.resize(10, 0)                                     // Resize to length 10, fill with 0
v.resize_with(10, Default::default)                // Resize with function

// Split off
let mut v = vec![1, 2, 3, 4, 5];
let v2 = v.split_off(3);                           // Split at index: v=[1,2,3], v2=[4,5]

// ===== RAW POINTERS =====
let v = vec![1, 2, 3];

v.as_ptr()                                          // Get raw pointer: *const T
v.as_mut_ptr()                                      // Get mutable raw pointer: *mut T

// Construct from raw parts (unsafe)
let mut v = vec![1, 2, 3];
let ptr = v.as_mut_ptr();
let len = v.len();
let cap = v.capacity();
std::mem::forget(v);
let v = unsafe { Vec::from_raw_parts(ptr, len, cap) };

// ===== COMMON PATTERNS =====
// Pattern 1: Remove duplicates (requires sorting)
let mut v = vec![1, 2, 2, 3, 3, 3, 4];
v.sort();
v.dedup();                                          // [1, 2, 3, 4]

// Pattern 2: Remove duplicates (preserve order)
use std::collections::HashSet;
let v = vec![1, 2, 2, 3, 1];
let unique: Vec<_> = v.into_iter()
    .collect::<HashSet<_>>()
    .into_iter()
    .collect();

// Pattern 3: Find and remove
let mut v = vec![1, 2, 3, 4, 5];
if let Some(pos) = v.iter().position(|&x| x == 3) {
    v.remove(pos);
}

// Pattern 4: Swap remove multiple
let mut v = vec![1, 2, 3, 4, 5];
let indices = vec![1, 3];
for &i in indices.iter().rev() {                   // Remove from back to preserve indices
    v.swap_remove(i);
}

// Pattern 5: Group consecutive
let v = vec![1, 1, 2, 2, 2, 3];
let groups: Vec<Vec<_>> = v.chunk_by(|a, b| a == b)
    .map(|chunk| chunk.to_vec())
    .collect();

// Pattern 6: Sliding window processing
let v = vec![1, 2, 3, 4, 5];
for window in v.windows(3) {
    let sum: i32 = window.iter().sum();
    println!("{:?} -> {}", window, sum);
}

// Pattern 7: Matrix (Vec<Vec<T>>)
let matrix: Vec<Vec<i32>> = vec![
    vec![1, 2, 3],
    vec![4, 5, 6],
];
let element = matrix[0][1];                        // 2

// Pattern 8: Flatten matrix
let flat: Vec<_> = matrix.into_iter().flatten().collect();

// Pattern 9: Transpose matrix
fn transpose<T: Clone>(matrix: Vec<Vec<T>>) -> Vec<Vec<T>> {
    if matrix.is_empty() { return vec![]; }
    let rows = matrix.len();
    let cols = matrix[0].len();
    (0..cols)
        .map(|col| (0..rows).map(|row| matrix[row][col].clone()).collect())
        .collect()
}

// Pattern 10: Ring buffer using rotate
let mut buffer = vec![1, 2, 3, 4, 5];
buffer.rotate_left(1);                             // [2,3,4,5,1]
buffer[4] = 6;                                      // [2,3,4,5,6]

// Pattern 11: Conditional push
let mut v = vec![];
let value = 42;
if condition {
    v.push(value);
}

// Pattern 12: Extend conditionally
let mut v = vec![1, 2, 3];
if condition {
    v.extend([4, 5, 6]);
}

// Pattern 13: Remove while iterating (drain_filter)
let mut v = vec![1, 2, 3, 4, 5];
let removed: Vec<_> = v.extract_if(|x| *x % 2 == 0).collect(); // Nightly

// Pattern 14: Safe index with get
let v = vec![1, 2, 3];
match v.get(index) {
    Some(&value) => println!("Found: {}", value),
    None => println!("Index out of bounds"),
}

// Pattern 15: Batch operations
let mut v = vec![1, 2, 3, 4, 5, 6];
for chunk in v.chunks_mut(2) {
    chunk[0] *= 10;
}
```