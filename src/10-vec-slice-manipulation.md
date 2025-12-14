# Vec & Slice Manipulation

Vectors and slices are the workhorses of Rust data processing. `Vec<T>` provides dynamic, heap-allocated arrays with amortized O(1) append operations, while slices (`&[T]`, `&mut [T]`) provide views into contiguous sequences without ownership. Understanding how to efficiently manipulate these types is essential for writing high-performance Rust code.

This chapter explores advanced patterns for working with vectors and slices that experienced programmers can leverage to build efficient systems. The key insight is that careful capacity management, zero-copy operations, and algorithmic choices can dramatically impact performance—often by orders of magnitude.

The patterns we'll explore include:
- Capacity management to minimize allocations and optimize amortization
- Slice algorithms for searching, sorting, and partitioning
- Chunking and windowing patterns for batch processing
- Zero-copy slicing techniques to avoid unnecessary allocations
- SIMD operations for data-parallel computation



## Pattern 1: Capacity Management and Amortization

**Problem**: Growing vectors incrementally triggers repeated reallocations—each doubling copies all existing elements. Building a 100K-element vector without pre-allocation causes ~17 reallocations and copies 200K elements total.

**Solution**: Use `Vec::with_capacity(n)` when size is known upfront. Call `reserve(n)` before bulk operations to pre-allocate space.

**Why It Matters**: Pre-allocation can improve performance by 10-100x for vector construction. A data pipeline building 1M-element results: naive approach does ~20 reallocations copying ~2M elements.

**Use Cases**: Batch processing (pre-allocate for batch size), collecting query results (reserve based on estimated count), temporary buffers in loops (reuse with clear), building large datasets (with_capacity), long-lived lookup tables (shrink_to_fit after construction).

### Example 1: Pre-allocate When Size is Known

When you know how many elements you'll add to a vector, pre-allocating with `with_capacity` eliminates all reallocations during construction. This is the single most impactful optimization for vector building.

```rust
fn process_batch(items: &[Item]) -> Vec<ProcessedItem> {
    let mut results = Vec::with_capacity(items.len());
    for item in items {
        results.push(process(item));
    }
    results
}
```

### Example 2: Reserve Before Iterative Construction

When building vectors from multiple sources, estimate the total size upfront and reserve space once. This avoids multiple reallocations as the vector grows.

```rust
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
```

### Example 3: Reuse Vectors to Avoid Allocation

In loops where you build temporary vectors repeatedly, reuse a single buffer by calling `clear()` between iterations. This retains the allocated capacity and eliminates allocation overhead entirely.

```rust
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
```

### Example 4: Track Amortized Growth

Monitoring allocation patterns helps identify performance problems. This wrapper tracks how many reallocations occur during vector growth, revealing whether pre-allocation is needed.

```rust
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
```

### Example 5: Shrink Long-Lived Data Structures

When a vector is over-allocated and will remain in memory for a long time, use `shrink_to_fit` to reclaim the excess capacity. This is particularly important for lookup tables and cached data.

```rust
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
```

### Example 6: Use Iterator Size Hints

When collecting from iterators, leverage the `size_hint` to pre-allocate optimal capacity. This is especially useful when the iterator provides accurate bounds.

```rust
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
```

### Example 7: Batch Insertion with Extend

When merging multiple vectors into one, calculate the total size upfront and reserve all needed space at once. This prevents reallocations during the merge operation.

```rust
fn merge_results(target: &mut Vec<String>, sources: &[Vec<String>]) {
    let total: usize = sources.iter().map(|v| v.len()).sum();
    target.reserve(total);

    for source in sources {
        target.extend_from_slice(source);
    }
}
```

### Example 8: Building Large Datasets Efficiently

For datasets where you know the exact size, pre-allocation ensures zero reallocations during construction. The assertion at the end verifies that no reallocation occurred.

```rust
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

**Problem**: Linear search through large sorted arrays is O(N) when O(log N) binary search is possible. Sorting entire datasets to find median or top-K wastes O(N log N) time.

**Solution**: Use `binary_search` and `partition_point` for O(log N) searches on sorted data. Apply `select_nth_unstable` for O(N) median/top-K finding without full sort.

**Why It Matters**: Algorithm choice dramatically affects performance. Finding an element: linear search O(N) vs binary search O(log N) is 1000x difference for 1M elements.

**Use Cases**: Database query optimization (binary search on sorted indices), statistics computation (median, percentiles with select_nth), data deduplication (sort + dedup), priority queues (partition by priority), cyclic buffers (rotate operations), filtering with memory constraints (retain vs filter+collect).

### Example 1: Binary Search on Sorted Data

Binary search provides O(log N) lookup on sorted slices, dramatically faster than linear search for large datasets. The slice must be sorted by the search key for correct results.

```rust
fn find_user_by_id(users: &[User], id: u64) -> Option<&User> {
    // users must be sorted by id
    users.binary_search_by_key(&id, |u| u.id)
        .ok()
        .map(|idx| &users[idx])
}

```

### Example 2: Partition Point for Range Queries

`partition_point` finds the index where a predicate transitions from true to false, enabling efficient range queries on sorted data. This is particularly useful for database-style queries.

```rust
fn find_range(sorted: &[i32], min: i32, max: i32) -> &[i32] {
    let start = sorted.partition_point(|&x| x < min);
    let end = sorted.partition_point(|&x| x <= max);
    &sorted[start..end]
}
```

### Example 3: Partition by Predicate

Partitioning separates elements based on a condition without allocating a new vector. This returns mutable references to both the matching and non-matching segments.

```rust
fn separate_valid_invalid(items: &mut [Item]) -> (&mut [Item], &mut [Item]) {
    let pivot = items.iter().partition_point(|item| item.is_valid());
    items.split_at_mut(pivot)
}
```

### Example 4: Custom Sorting with Comparators

Complex sorting criteria can be expressed with `sort_by`, chaining multiple comparisons. This example sorts by priority (descending) with timestamp as a tiebreaker (ascending).

```rust
fn sort_by_priority(tasks: &mut [Task]) {
    tasks.sort_by(|a, b| {
        // Sort by priority descending, then by timestamp ascending
        b.priority.cmp(&a.priority)
            .then_with(|| a.timestamp.cmp(&b.timestamp))
    });
}
```

### Example 5: Unstable Sort for Performance

When the relative order of equal elements doesn't matter, `sort_unstable` runs significantly faster than stable sort. This is ideal for primitive types and performance-critical code.

```rust
fn sort_large_dataset(data: &mut [f64]) {
    // sort_unstable is faster than sort for primitive types
    data.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
}
```

### Example 6: Finding Median and Top-K Elements

`select_nth_unstable` performs partial sorting in O(N) time, making it perfect for finding medians, percentiles, and top-K elements without sorting the entire array.

```rust
fn find_median(values: &mut [f64]) -> f64 {
    let mid = values.len() / 2;
    let (_, median, _) = values.select_nth_unstable(mid);
    *median
}

fn top_k_elements(values: &mut [i32], k: usize) -> &[i32] {
    let (_, _, right) = values.select_nth_unstable(values.len() - k);
    right
}
```

### Example 7: Efficient Cyclic Rotation

`rotate_left` and `rotate_right` perform cyclic shifts efficiently without temporary buffers. This is essential for ring buffers and circular data structures.

```rust
fn rotate_buffer(buffer: &mut [u8], offset: usize) {
    buffer.rotate_left(offset % buffer.len());
}
```

### Example 8: Deduplication on Sorted Data

For removing duplicates, sort first then call `dedup`. This is O(N log N) for the sort plus O(N) for dedup, much faster than checking each element against all others.

```rust
fn unique_sorted(items: &mut Vec<i32>) {
    items.sort_unstable();
    items.dedup();
}
```

### Example 9: In-Place Filtering with Retain

`retain` removes elements that don't match a predicate without allocating a new vector. This is more efficient than `filter().collect()` when you want to modify in place.

```rust
fn remove_invalid(items: &mut Vec<Item>) {
    items.retain(|item| item.is_valid());
}
```

### Example 10: Reverse Operations

Reversing slices in-place is O(N/2) swaps. Combined with chunking, you can reverse segments of data efficiently.

```rust
fn reverse_segments(data: &mut [u8], segment_size: usize) {
    for chunk in data.chunks_mut(segment_size) {
        chunk.reverse();
    }
}
```

### Example 11: Filling Slices

`fill` sets all elements to a value, while `fill_with` uses a closure to generate values. This is useful for initialization and resetting buffers.

```rust
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
```

### Example 12: Swapping Slice Ranges

`swap_with_slice` exchanges the contents of two mutable slices in place without allocation. This is useful for data rearrangement and buffer management.

```rust
fn swap_halves(data: &mut [u8]) {
    let mid = data.len() / 2;
    let (left, right) = data.split_at_mut(mid);
    let min_len = left.len().min(right.len());
    left[..min_len].swap_with_slice(&mut right[..min_len]);
}
```

### Example 13: Pattern Matching with Starts/Ends

Checking for prefixes and suffixes is a common pattern in protocol parsing and file format detection.

```rust
fn has_magic_header(data: &[u8]) -> bool {
    const MAGIC: &[u8] = b"PNG\x89";
    data.starts_with(MAGIC)
}
```

### Example 14: Finding Subsequences

Searching for a pattern within a slice can be done efficiently using windows combined with position finding.

```rust
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

**Problem**: Processing large datasets element-by-element is slow due to function call overhead and poor cache locality. Batch operations require manual index calculation and are error-prone.

**Solution**: Use `.chunks(n)` for non-overlapping fixed-size batches. Use `.windows(n)` for overlapping subsequences (moving averages, pattern detection).

**Why It Matters**: Chunking improves cache locality—processing 1000-element chunks instead of individual elements can be 10-50x faster. Window operations enable signal processing algorithms (FFT, convolution) without collecting intermediate vectors—zero allocation for moving statistics.

**Use Cases**: Batch processing (database inserts, API requests), signal processing (moving averages, FFT windows), image processing (tile-based operations), parallel computation (divide work across threads), network packet assembly (fixed-size frames), time-series analysis (rolling statistics).

### Example 1: Fixed-Size Chunking

`chunks` divides a slice into non-overlapping segments of a specified size. The last chunk may be smaller if the slice length isn't evenly divisible.

```rust
fn process_in_batches(data: &[u8], batch_size: usize) -> Vec<ProcessedBatch> {
    data.chunks(batch_size)
        .map(|chunk| process_batch(chunk))
        .collect()
}

```

### Example 2: Mutable Chunks for In-Place Transformation

`chunks_mut` provides mutable access to each chunk, enabling in-place transformations without copying data. This is ideal for batch normalization and similar operations.

```rust
fn normalize_batches(data: &mut [f64], batch_size: usize) {
    for chunk in data.chunks_mut(batch_size) {
        let sum: f64 = chunk.iter().sum();
        let mean = sum / chunk.len() as f64;

        for value in chunk {
            *value -= mean;
        }
    }
}
```

### Example 3: Exact Chunks with Remainder Handling

`as_chunks` splits a slice into fixed-size arrays with compile-time size checking, returning both the aligned chunks and the remainder. This is essential for SIMD-optimized code.

```rust
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
```

### Example 4: Sliding Windows for Moving Averages

`windows` creates overlapping views of the slice, perfect for computing rolling statistics without allocating intermediate buffers.

```rust
fn moving_average(values: &[f64], window_size: usize) -> Vec<f64> {
    values.windows(window_size)
        .map(|window| {
            let sum: f64 = window.iter().sum();
            sum / window.len() as f64
        })
        .collect()
}
```

### Example 5: Pairwise Operations

Windows of size 2 enable efficient computation of differences, ratios, or other pairwise operations between adjacent elements.

```rust
fn compute_deltas(values: &[i32]) -> Vec<i32> {
    values.windows(2)
        .map(|pair| pair[1] - pair[0])
        .collect()
}
```

### Example 6: Pattern Matching in Overlapping Windows

Windows combined with filtering enable detection of consecutive sequences or patterns spanning multiple elements.

```rust
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
```

### Example 7: Parallel Processing with Chunks

Chunking naturally partitions work for parallel processing. Each thread processes one chunk independently.

```rust
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
```

### Example 8: Reverse Chunking

`rchunks` processes chunks from the end of the slice backward, useful for parsing formats where metadata appears at the end.

```rust
fn process_backwards(data: &[u8], chunk_size: usize) {
    for chunk in data.rchunks(chunk_size) {
        process_chunk(chunk);
    }
}
```

### Example 9: Exact Chunks vs Regular Chunks

`chunks_exact` guarantees all chunks (except the explicit remainder) have the exact size, simplifying algorithms that require uniform blocks.

```rust
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
```

### Example 10: Strided Access with Step By

Combining iteration with `step_by` enables sampling every Nth element, useful for downsampling data.

```rust
fn sample_every_nth(data: &[f64], n: usize) -> Vec<f64> {
    data.iter()
        .step_by(n)
        .copied()
        .collect()
}
```

### Example 11: Splitting into Equal Parts

Dividing data into N roughly equal parts is common for load balancing across workers.

```rust
fn split_into_n_parts(data: &[u8], n: usize) -> Vec<&[u8]> {
    let chunk_size = (data.len() + n - 1) / n; // Ceiling division
    data.chunks(chunk_size).collect()
}
```

### Example 12: Signal Processing with Overlapping Windows

Advanced windowing combines `step_by` for hop size with manual slicing for overlapping FFT windows in spectrograms.

```rust
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

**Problem**: Parsing structured data by extracting fields into owned `String`/`Vec` causes allocation for every field—parsing 1M CSV records with 10 fields allocates 10M strings. Returning data from functions forces cloning entire vectors.

**Solution**: Return borrowed slices (`&[T]`, `&str`) that reference the original data. Use `split()`, `split_at()`, and range indexing to create views.

**Why It Matters**: Zero-copy parsing can be 10-100x faster than allocating approach. Parsing a 100MB CSV file: allocating approach needs gigabytes of temporary memory and causes GC pressure.

**Use Cases**: CSV/JSON parsing (return field slices), network protocol parsers (split packets into views), text processing (split without allocation), binary format parsing (frame headers/payloads), configuration file parsing, streaming data processors.

### Example 1: Return Slices Instead of Cloning

By returning borrowed slices instead of owned strings, CSV parsing avoids allocating memory for every field, dramatically improving performance.

```rust
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

```

### Example 2: Split Without Allocation

The `split` iterator creates string slices on the fly without allocating a vector until `collect` is called. Each slice references the original string.

```rust
fn parse_csv_line(line: &str) -> Vec<&str> {
    line.split(',')
        .map(|s| s.trim())
        .collect()
}
```

### Example 3: Multiple Slices from One Allocation

A struct can hold multiple slices all pointing into a single backing buffer, enabling zero-copy frame parsing for network protocols.

```rust
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
```

### Example 4: Split At for Header/Body Separation

`split_at` divides a slice at a specific index, creating two non-overlapping views perfect for fixed-size header parsing.

```rust
fn process_header_and_body(data: &[u8]) -> Result<(Header, Vec<Item>), Error> {
    let (header_bytes, body) = data.split_at(HEADER_SIZE);
    let header = parse_header(header_bytes)?;
    let items = parse_body(body)?;
    Ok((header, items))
}
```

### Example 5: Copy-on-Write with Cow

`Cow` (Clone on Write) enables APIs that only allocate when modification is needed, borrowing otherwise. Perfect for conditional string encoding fixes.

```rust
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
```

### Example 6: Split First and Last for Protocol Parsing

`split_first` and `split_last` extract single elements while returning a slice of the remainder, ideal for version bytes and checksums.

```rust
fn parse_packet(data: &[u8]) -> Result<Packet, ParseError> {
    let (&version, rest) = data.split_first()
        .ok_or(ParseError::Empty)?;

    let (payload, &checksum) = rest.split_last()
        .ok_or(ParseError::NoChecksum)?;

    Ok(Packet { version, payload, checksum })
}
```

### Example 7: Iterating Without Collecting

When the final result doesn't need individual slices, process them in the iterator pipeline without allocating a vector.

```rust
fn sum_valid_numbers(data: &str) -> i32 {
    data.split(',')
        .filter_map(|s| s.trim().parse::<i32>().ok())
        .sum()
}
```

### Example 8: Slicing During Iteration

Manual range-based slicing combined with iteration enables custom chunk processing without the constraints of fixed-size chunks.

```rust
fn process_blocks(data: &[u8], block_size: usize) -> Vec<BlockResult> {
    (0..data.len())
        .step_by(block_size)
        .map(|i| {
            let end = (i + block_size).min(data.len());
            process_block(&data[i..end])
        })
        .collect()
}
```

### Example 9: Grouping by Delimiter

Manual delimiter-based splitting provides more control than the built-in `split` method, useful for binary data or custom delimiters.

```rust
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
```

### Example 10: In-Place Mutable Operations

Mutable slices enable in-place transformations like byte swapping without any allocation.

```rust
fn swap_bytes_in_place(data: &mut [u8]) {
    for pair in data.chunks_exact_mut(2) {
        pair.swap(0, 1);
    }
}
```

### Example 11: Complete Zero-Copy HTTP Parser

This comprehensive example shows how an entire HTTP request parser can work with zero allocations, storing only slices into the original buffer.

```rust
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

**Problem**: Processing large arrays element-by-element leaves CPU vector units idle—modern CPUs can process 4-16 elements per instruction but scalar code uses only one. Image processing, audio encoding, numerical computing, and data compression are bottlenecked by sequential processing.

**Solution**: Use portable SIMD through `std::simd` (nightly) or crates like `packed_simd2`. Process data in SIMD-width chunks (4/8/16 elements).

**Why It Matters**: SIMD provides 4-16x speedups for data-parallel operations. Processing 1M floats: scalar takes 1M operations, SIMD with 8-wide vectors takes 125K operations.

**Use Cases**: Image/video processing (filters, transformations), audio processing (effects, encoding), numerical computing (matrix operations, scientific simulations), compression algorithms, checksums and hashing, database query execution, machine learning inference.

### Example 1: Manual SIMD-Friendly Chunking

Processing data in fixed-size chunks enables the compiler to auto-vectorize, and provides a clear structure for manual SIMD optimization.

```rust
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

```

### Example 2: Aligned Data Structures

Proper memory alignment is crucial for SIMD performance. Using `#[repr(align(N))]` ensures data is aligned for vector instructions.

```rust
#[repr(align(32))]
struct AlignedBuffer([f32; 8]);

fn process_aligned(data: &[AlignedBuffer]) -> Vec<f32> {
    data.iter()
        .flat_map(|buf| buf.0.iter())
        .map(|&x| x * 2.0)
        .collect()
}
```

### Example 3: Portable SIMD with std::simd

Rust's portable SIMD (nightly) provides safe, cross-platform vector operations that compile to optimal CPU instructions.

```rust
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
```

### Example 4: SIMD Search Operations

Searching through large buffers can benefit from SIMD parallelism by checking multiple elements simultaneously.

```rust
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
```

### Example 5: SIMD Reduction Operations

Reduction operations like sum benefit from SIMD by accumulating multiple lanes in parallel before the final reduction.

```rust
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
```

### Example 6: Combining Parallelism with SIMD

Rayon's parallel iterators combined with SIMD operations enable multi-core data parallelism for maximum throughput.

```rust
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
```

### Example 7: Auto-Vectorization

Simple loops are often auto-vectorized by the compiler. Writing clear, simple code can be as fast as manual SIMD.

```rust
fn scale_values(data: &mut [f32], scale: f32) {
    // Compiler can auto-vectorize this loop
    for value in data {
        *value *= scale;
    }
}
```

### Example 8: Dot Product with Chunking

Dot products are fundamental linear algebra operations that benefit significantly from SIMD processing.

```rust
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
```

### Example 9: Image Processing Pipeline

Converting RGB to grayscale is embarrassingly parallel and benefits from SIMD processing of pixel data.

```rust
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
```

### Example 10: SIMD-Friendly Data Layouts

Structure-of-arrays layout is more SIMD-friendly than array-of-structures for vector operations.

```rust
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

**Problem**: Removing elements during iteration requires complex index tracking. Transforming vectors in-place while filtering requires multiple passes.

**Solution**: Use `drain()` for removing ranges while iterating. Apply `retain_mut()` for in-place filtering with mutation.

**Why It Matters**: These patterns enable complex transformations without temporary allocations. In-place compaction with swap avoids O(N²) repeated deletions.

**Use Cases**: In-place vector compaction, gap buffer implementations, sorting implementations with pivot splitting, parallel processing with split_at_mut, memory-efficient filtering, zero-copy type conversions (e.g., &[u8] to &[u32]), efficient element removal patterns.

### Example 1: In-Place Vector Compaction

Compacting a vector by removing unwanted elements without allocation uses a two-pointer technique with swap operations.

```rust
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

```

### Example 2: Extracting Elements with Drain

Draining elements that match a predicate into a separate vector while preserving the original's capacity.

```rust
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
```

### Example 3: Splice for Range Replacement

`splice` removes a range and replaces it with new elements in a single operation, more efficient than separate remove and insert.

```rust
fn replace_range(vec: &mut Vec<i32>, start: usize, end: usize, replacement: &[i32]) {
    vec.splice(start..end, replacement.iter().copied());
}
```

### Example 4: Efficient Mid-Vector Insertion

`split_off` divides a vector at an index, enabling efficient insertion without shifting all elements twice.

```rust
fn insert_slice_at(vec: &mut Vec<u8>, index: usize, data: &[u8]) {
    let tail = vec.split_off(index);
    vec.extend_from_slice(data);
    vec.extend_from_slice(&tail);
}
```

### Example 5: Sliding Window Maximum with Deque

VecDeque enables efficient double-ended operations crucial for algorithms like sliding window maximum.

```rust
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
```

### Example 6: Circular Buffer Implementation

Circular buffers use modular arithmetic to wrap indices, providing efficient fixed-size queues without shifting elements.

```rust
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
```

### Example 7: Copy-Free Slice Swapping

`std::mem::swap` enables efficient element-wise swapping between two mutable slices without temporary storage.

```rust
fn interleave_slices(a: &mut [u8], b: &mut [u8]) {
    assert_eq!(a.len(), b.len());

    for (a_val, b_val) in a.iter_mut().zip(b.iter_mut()) {
        std::mem::swap(a_val, b_val);
    }
}
```

### Example 8: Self-Referential Duplication

`extend_from_within` copies a range from within the vector and appends it, useful for repeating patterns.

```rust
fn duplicate_segment(vec: &mut Vec<u8>, start: usize, end: usize) {
    vec.extend_from_within(start..end);
}
```

### Example 9: Binary Partitioning

Lomuto partition scheme efficiently separates elements based on a predicate, foundational for quicksort and selection algorithms.

```rust
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
```

### Example 10: Three-Way Partitioning

Dutch National Flag algorithm partitions into three regions (less than, equal to, greater than) in a single pass.

```rust
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

### Summary

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
