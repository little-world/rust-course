# Vec & Slice Manipulation

Vectors and slices are the workhorses of Rust data processing. `Vec<T>` provides dynamic, heap-allocated arrays with amortized O(1) append operations, while slices (`&[T]`, `&mut [T]`) provide views into contiguous sequences without ownership. Understanding how to efficiently manipulate these types is essential for writing high-performance Rust code.

 The key insight is that careful capacity management, zero-copy operations, and algorithmic choices can dramatically impact performance—often by orders of magnitude.

## Pattern 1: Capacity Management and Amortization

**Problem**: Growing vectors incrementally triggers repeated reallocations—each doubling copies all existing elements. Building a 100K-element vector without pre-allocation causes ~17 reallocations and copies 200K elements total.

**Solution**: Use `Vec::with_capacity(n)` when size is known upfront. Call `reserve(n)` before bulk operations to pre-allocate space.

**Why It Matters**: Pre-allocation can improve performance by 10-100x for vector construction. A data pipeline building 1M-element results: naive approach does ~20 reallocations copying ~2M elements.

**Use Cases**: Batch processing (pre-allocate for batch size), collecting query results (reserve based on estimated count), temporary buffers in loops (reuse with clear), building large datasets (with_capacity), long-lived lookup tables (shrink_to_fit after construction).

### Example: Pre-allocate When Size is Known

When you know how many elements you'll add to a vector, pre-allocating with `with_capacity` eliminates all reallocations during construction. This is the single most impactful optimization for vector building. The resulting vector has exactly the capacity needed, so no memory is wasted and no copying occurs during the loop.

```rust
fn process_batch(items: &[Item]) -> Vec<ProcessedItem> {
    let mut results = Vec::with_capacity(items.len());
    for item in items {
        results.push(process(item));
    }
    results
}
```

### Example: Reserve Before Iterative Construction

When building vectors from multiple sources, estimate the total size upfront and reserve space once. This avoids multiple reallocations as the vector grows. Even rough estimates provide significant benefits—over-estimating slightly is better than triggering repeated reallocations.

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

### Example: Reuse Vectors to Avoid Allocation

In loops where you build temporary vectors repeatedly, reuse a single buffer by calling `clear()` between iterations. This retains the allocated capacity and eliminates allocation overhead entirely. After the first iteration, subsequent iterations have zero allocation cost regardless of batch size.

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

### Example: Track Amortized Growth

Monitoring allocation patterns helps identify performance problems. This wrapper tracks how many reallocations occur during vector growth, revealing whether pre-allocation is needed. Use this diagnostic tool in development to find hot paths that would benefit from `with_capacity`.

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

### Example: Shrink Long-Lived Data Structures

When a vector is over-allocated and will remain in memory for a long time, use `shrink_to_fit` to reclaim the excess capacity. This is particularly important for lookup tables and cached data. Avoid calling this on vectors that will grow again soon, as it would trigger another reallocation.

```rust
fn build_lookup_table(entries: &[Entry]) -> Vec<IndexEntry> {
    // Over-estimate capacity
    let mut table = Vec::with_capacity(entries.len() * 2);

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

### Example: Use Iterator Size Hints

When collecting from iterators, leverage the `size_hint` to pre-allocate optimal capacity. This is especially useful when the iterator provides accurate bounds. The standard library's `collect()` already uses size hints, but manual handling gives you more control for custom scenarios.

```rust
fn collect_filtered(items: impl Iterator<Item = i32>) -> Vec<i32> {
    // size_hint can optimize allocation
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

### Example: Batch Insertion with Extend

When merging multiple vectors into one, calculate the total size upfront and reserve all needed space at once. This prevents reallocations during the merge operation. Using `extend_from_slice` is also more efficient than repeated `push` calls due to better optimization opportunities.

```rust
fn merge_results(
    target: &mut Vec<String>,
    sources: &[Vec<String>],
) {
    let total: usize = sources.iter().map(|v| v.len()).sum();
    target.reserve(total);

    for source in sources {
        target.extend_from_slice(source);
    }
}
```

### Example: Building Large Datasets Efficiently

For datasets where you know the exact size, pre-allocation ensures zero reallocations during construction. The assertion at the end verifies that no reallocation occurred. This pattern is essential for generating test data or initializing large lookup tables.

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

    assert_eq!(data.len(), data.capacity()); // No realloc
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

### Example: Binary Search on Sorted Data

Binary search provides O(log N) lookup on sorted slices, dramatically faster than linear search for large datasets. The slice must be sorted by the search key for correct results. Using `binary_search_by_key` extracts the comparison key from complex structs, avoiding manual comparator logic.

```rust
fn find_user_by_id(users: &[User], id: u64) -> Option<&User> {
    // users must be sorted by id
    users.binary_search_by_key(&id, |u| u.id)
        .ok()
        .map(|idx| &users[idx])
}

```

### Example: Partition Point for Range Queries

`partition_point` finds the index where a predicate transitions from true to false, enabling efficient range queries on sorted data. This is particularly useful for database-style queries. Combining two partition points yields both bounds of a range in O(log N) time.

```rust
fn find_range(sorted: &[i32], min: i32, max: i32) -> &[i32] {
    let start = sorted.partition_point(|&x| x < min);
    let end = sorted.partition_point(|&x| x <= max);
    &sorted[start..end]
}
```

### Example: Partition by Predicate

Partitioning separates elements based on a condition without allocating a new vector. This returns mutable references to both the matching and non-matching segments. Note that partitioning requires the slice to already be partitioned—use `partition_point` to find the split index.

```rust
fn separate_valid_invalid(
    items: &mut [Item],
) -> (&mut [Item], &mut [Item]) {
    let pivot = items.iter()
        .partition_point(|item| item.is_valid());
    items.split_at_mut(pivot)
}
```

### Example: Custom Sorting with Comparators

Complex sorting criteria can be expressed with `sort_by`, chaining multiple comparisons. This example sorts by priority (descending) with timestamp as a tiebreaker (ascending). The `then_with` combinator chains comparisons, only evaluating the next level when the previous returns `Equal`.

```rust
fn sort_by_priority(tasks: &mut [Task]) {
    tasks.sort_by(|a, b| {
        // Sort by priority descending, then by timestamp ascending
        b.priority.cmp(&a.priority)
            .then_with(|| a.timestamp.cmp(&b.timestamp))
    });
}
```

### Example: Unstable Sort for Performance

When the relative order of equal elements doesn't matter, `sort_unstable` runs significantly faster than stable sort. This is ideal for primitive types and performance-critical code. The unstable variant uses pattern-defeating quicksort, which is faster and uses less memory than the stable merge sort.

```rust
fn sort_large_dataset(data: &mut [f64]) {
    // sort_unstable is faster than sort for primitive types
    data.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
}
```

### Example: Finding Median and Top-K Elements

`select_nth_unstable` performs partial sorting in O(N) time, making it perfect for finding medians, percentiles, and top-K elements without sorting the entire array. It rearranges elements so all smaller values are before the Nth element and all larger values are after. This is based on the quickselect algorithm, which avoids the full O(N log N) cost of sorting.

```rust
fn find_median(values: &mut [f64]) -> f64 {
    let mid = values.len() / 2;
    let (_, median, _) = values.select_nth_unstable(mid);
    *median
}

fn top_k_elements(values: &mut [i32], k: usize) -> &[i32] {
    let idx = values.len() - k;
    let (_, _, right) = values.select_nth_unstable(idx);
    right
}
```

### Example: Efficient Cyclic Rotation

`rotate_left` and `rotate_right` perform cyclic shifts efficiently without temporary buffers. This is essential for ring buffers and circular data structures. The implementation uses a clever three-reverse algorithm that achieves O(N) time complexity with O(1) extra space.

```rust
fn rotate_buffer(buffer: &mut [u8], offset: usize) {
    buffer.rotate_left(offset % buffer.len());
}
```

### Example: Deduplication on Sorted Data

For removing duplicates, sort first then call `dedup`. This is O(N log N) for the sort plus O(N) for dedup, much faster than checking each element against all others. The `dedup` method only removes consecutive duplicates, which is why sorting must come first.

```rust
fn unique_sorted(items: &mut Vec<i32>) {
    items.sort_unstable();
    items.dedup();
}
```

### Example: In-Place Filtering with Retain

`retain` removes elements that don't match a predicate without allocating a new vector. This is more efficient than `filter().collect()` when you want to modify in place. Elements are shifted down to fill gaps, maintaining relative order of retained elements.

```rust
fn remove_invalid(items: &mut Vec<Item>) {
    items.retain(|item| item.is_valid());
}
```

### Example: Reverse Operations

Reversing slices in-place is O(N/2) swaps. Combined with chunking, you can reverse segments of data efficiently. This pattern is useful for endianness conversion or processing data that arrives in reverse order.

```rust
fn reverse_segments(data: &mut [u8], segment_size: usize) {
    for chunk in data.chunks_mut(segment_size) {
        chunk.reverse();
    }
}
```

### Example: Filling Slices

`fill` sets all elements to a value, while `fill_with` uses a closure to generate values. This is useful for initialization and resetting buffers. The closure variant is called once per element, enabling patterns like incrementing counters or generating random values.

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

### Example: Swapping Slice Ranges

`swap_with_slice` exchanges the contents of two mutable slices in place without allocation. This is useful for data rearrangement and buffer management. The slices must be the same length, which is why `min_len` is computed for the unequal-half case.

```rust
fn swap_halves(data: &mut [u8]) {
    let mid = data.len() / 2;
    let (left, right) = data.split_at_mut(mid);
    let min_len = left.len().min(right.len());
    left[..min_len].swap_with_slice(&mut right[..min_len]);
}
```

### Example: Pattern Matching with Starts/Ends

Checking for prefixes and suffixes is a common pattern in protocol parsing and file format detection. These methods are optimized to compare only the necessary bytes, not the entire slice. Magic number detection often combines this with constant patterns defined at compile time.

```rust
fn has_magic_header(data: &[u8]) -> bool {
    const MAGIC: &[u8] = b"PNG\x89";
    data.starts_with(MAGIC)
}
```

### Example: Finding Subsequences

Searching for a pattern within a slice can be done efficiently using windows combined with position finding. The `windows` iterator creates overlapping views of the slice, each the size of the needle. This is O(N * M) in the worst case, where N is haystack length and M is needle length.

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

### Example: Fixed-Size Chunking

`chunks` divides a slice into non-overlapping segments of a specified size. The last chunk may be smaller if the slice length isn't evenly divisible. This is the foundation for batch processing patterns where work is divided into manageable pieces.

```rust
fn process_in_batches(
    data: &[u8],
    batch_size: usize,
) -> Vec<ProcessedBatch> {
    data.chunks(batch_size)
        .map(|chunk| process_batch(chunk))
        .collect()
}

```

### Example: Mutable Chunks for In-Place Transformation

`chunks_mut` provides mutable access to each chunk, enabling in-place transformations without copying data. This is ideal for batch normalization and similar operations. Each chunk is processed independently, which naturally enables parallel processing patterns.

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

### Example: Exact Chunks with Remainder Handling

`as_chunks` splits a slice into fixed-size arrays with compile-time size checking, returning both the aligned chunks and the remainder. This is essential for SIMD-optimized code. The const generic parameter ensures chunk size is known at compile time, enabling better optimization.

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

### Example: Sliding Windows for Moving Averages

`windows` creates overlapping views of the slice, perfect for computing rolling statistics without allocating intermediate buffers. Unlike chunks, each window overlaps with the previous one, sharing all but one element. The output has `len - window_size + 1` elements for an input of length `len`.

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

### Example: Pairwise Operations

Windows of size 2 enable efficient computation of differences, ratios, or other pairwise operations between adjacent elements. This is the standard pattern for computing derivatives, growth rates, or detecting changes. The result has one fewer element than the input since each output requires two inputs.

```rust
fn compute_deltas(values: &[i32]) -> Vec<i32> {
    values.windows(2)
        .map(|pair| pair[1] - pair[0])
        .collect()
}
```

### Example: Pattern Matching in Overlapping Windows

Windows combined with filtering enable detection of consecutive sequences or patterns spanning multiple elements. The `enumerate` method tracks the starting index of each matching window. This pattern is essential for anomaly detection in time series data.

```rust
fn find_consecutive_sequences(
    data: &[i32],
    target: i32,
) -> Vec<usize> {
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

### Example: Parallel Processing with Chunks

Chunking naturally partitions work for parallel processing. Each thread processes one chunk independently. The chunk size is calculated to distribute work evenly across available CPU cores, maximizing parallelism.

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

### Example: Reverse Chunking

`rchunks` processes chunks from the end of the slice backward, useful for parsing formats where metadata appears at the end. The first chunk produced may be smaller than the requested size if the length isn't evenly divisible. This mirrors `chunks` but in reverse order.

```rust
fn process_backwards(data: &[u8], chunk_size: usize) {
    for chunk in data.rchunks(chunk_size) {
        process_chunk(chunk);
    }
}
```

### Example: Exact Chunks vs Regular Chunks

`chunks_exact` guarantees all chunks (except the explicit remainder) have the exact size, simplifying algorithms that require uniform blocks. Unlike `chunks`, the remainder isn't included in the iteration—you must access it separately via `remainder()`. This is cleaner when full-size blocks and partial blocks need different handling.

```rust
fn encode_blocks(
    data: &[u8],
    block_size: usize,
) -> Vec<EncodedBlock> {
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

### Example: Strided Access with Step By

Combining iteration with `step_by` enables sampling every Nth element, useful for downsampling data. This is memory-efficient since it creates no intermediate collections—elements are skipped lazily. The result contains approximately `len / n` elements.

```rust
fn sample_every_nth(data: &[f64], n: usize) -> Vec<f64> {
    data.iter()
        .step_by(n)
        .copied()
        .collect()
}
```

### Example: Splitting into Equal Parts

Dividing data into N roughly equal parts is common for load balancing across workers. The ceiling division formula ensures no data is lost when the length isn't evenly divisible. Some parts may have one more element than others, but the difference is at most one.

```rust
fn split_into_n_parts(data: &[u8], n: usize) -> Vec<&[u8]> {
    let chunk_size = (data.len() + n - 1) / n; // Ceiling division
    data.chunks(chunk_size).collect()
}
```

### Example: Signal Processing with Overlapping Windows

Advanced windowing combines `step_by` for hop size with manual slicing for overlapping FFT windows in spectrograms. The hop size determines overlap—a hop of `window_size / 2` gives 50% overlap between consecutive windows. This is the standard approach for Short-Time Fourier Transform (STFT) analysis.

```rust
fn compute_spectrogram(
    signal: &[f32],
    window_size: usize,
    hop_size: usize,
) -> Vec<Vec<f32>> {
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

### Example: Return Slices Instead of Cloning

By returning borrowed slices instead of owned strings, CSV parsing avoids allocating memory for every field, dramatically improving performance. The lifetime `'a` ties the returned slice to the input, ensuring the reference remains valid. This pattern can reduce memory allocation by 10-100x in parsing-heavy workloads.

```rust
fn find_field<'a>(
    record: &'a [u8],
    field_index: usize,
) -> &'a [u8] {
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

### Example: Split Without Allocation

The `split` iterator creates string slices on the fly without allocating a vector until `collect` is called. Each slice references the original string. The `trim()` call also returns a slice, not a new allocation, making the entire pipeline allocation-free until `collect`.

```rust
fn parse_csv_line(line: &str) -> Vec<&str> {
    line.split(',')
        .map(|s| s.trim())
        .collect()
}
```

### Example: Multiple Slices from One Allocation

A struct can hold multiple slices all pointing into a single backing buffer, enabling zero-copy frame parsing for network protocols. The lifetime parameter `'a` ensures all slices remain valid as long as the struct exists. This is the foundation of high-performance packet parsers used in network stacks.

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

### Example: Split At for Header/Body Separation

`split_at` divides a slice at a specific index, creating two non-overlapping views perfect for fixed-size header parsing. The operation is O(1) since it just creates two fat pointers—no data is copied. This is the idiomatic way to separate fixed-size headers from variable-length payloads.

```rust
fn process_header_and_body(
    data: &[u8],
) -> Result<(Header, Vec<Item>), Error> {
    let (header_bytes, body) = data.split_at(HEADER_SIZE);
    let header = parse_header(header_bytes)?;
    let items = parse_body(body)?;
    Ok((header, items))
}
```

### Example: Copy-on-Write with Cow

`Cow` (Clone on Write) enables APIs that only allocate when modification is needed, borrowing otherwise. Perfect for conditional string encoding fixes. When the data is already valid UTF-8, `Cow::Borrowed` holds a reference without allocation; only malformed data triggers the `Cow::Owned` path.

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

### Example: Split First and Last for Protocol Parsing

`split_first` and `split_last` extract single elements while returning a slice of the remainder, ideal for version bytes and checksums. Both methods return `Option` to handle empty slices gracefully. Chaining these operations parses fixed-format protocols without manual index arithmetic.

```rust
fn parse_packet(data: &[u8]) -> Result<Packet, ParseError> {
    let (&version, rest) = data.split_first()
        .ok_or(ParseError::Empty)?;

    let (payload, &checksum) = rest.split_last()
        .ok_or(ParseError::NoChecksum)?;

    Ok(Packet { version, payload, checksum })
}
```

### Example: Iterating Without Collecting

When the final result doesn't need individual slices, process them in the iterator pipeline without allocating a vector. The `filter_map` handles parsing failures gracefully by filtering out invalid entries. The final `sum()` consumes the iterator directly without intermediate storage.

```rust
fn sum_valid_numbers(data: &str) -> i32 {
    data.split(',')
        .filter_map(|s| s.trim().parse::<i32>().ok())
        .sum()
}
```

### Example: Slicing During Iteration

Manual range-based slicing combined with iteration enables custom chunk processing without the constraints of fixed-size chunks. The `step_by` iterator generates starting indices, while `.min()` handles the last partial block. This pattern offers more control than `chunks` for variable-size blocks.

```rust
fn process_blocks(
    data: &[u8],
    block_size: usize,
) -> Vec<BlockResult> {
    (0..data.len())
        .step_by(block_size)
        .map(|i| {
            let end = (i + block_size).min(data.len());
            process_block(&data[i..end])
        })
        .collect()
}
```

### Example: Grouping by Delimiter

Manual delimiter-based splitting provides more control than the built-in `split` method, useful for binary data or custom delimiters. This implementation handles the final segment that doesn't end with a delimiter. Unlike `split`, this works with `&[u8]` directly, avoiding UTF-8 validation overhead.

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

### Example: In-Place Mutable Operations

Mutable slices enable in-place transformations like byte swapping without any allocation. The `chunks_exact_mut` ensures each chunk has exactly 2 elements for the swap. This pattern is common in endianness conversion and data format transformations.

```rust
fn swap_bytes_in_place(data: &mut [u8]) {
    for pair in data.chunks_exact_mut(2) {
        pair.swap(0, 1);
    }
}
```

### Example: Complete Zero-Copy HTTP Parser

This comprehensive example shows how an entire HTTP request parser can work with zero allocations, storing only slices into the original buffer. The struct fields are all references tied to the input buffer's lifetime. Only the headers `Vec` allocates, but its contents are still slices into the original data.

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

### Example: Manual SIMD-Friendly Chunking

Processing data in fixed-size chunks enables the compiler to auto-vectorize, and provides a clear structure for manual SIMD optimization. The inner loop over a fixed-size array is especially optimization-friendly since the bounds are compile-time constants. Always handle the remainder separately to avoid bounds-check overhead in the hot path.

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

### Example: Aligned Data Structures

Proper memory alignment is crucial for SIMD performance. Using `#[repr(align(N))]` ensures data is aligned for vector instructions. Misaligned loads can be 2-10x slower on some architectures, making alignment essential for peak throughput.

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

### Example: Portable SIMD with std::simd

Rust's portable SIMD (nightly) provides safe, cross-platform vector operations that compile to optimal CPU instructions. The `f32x4` type represents a vector of 4 floats that can be added in a single instruction. This code compiles to SSE/AVX on x86, NEON on ARM, or falls back to scalar on unsupported platforms.

```rust
#[cfg(feature = "portable_simd")]
fn add_vectors_simd(a: &[f32], b: &[f32], result: &mut [f32]) {
    use std::simd::*;

    let lanes = 4;
    let (a_chunks, a_remainder) = a.as_chunks::<4>();
    let (b_chunks, b_remainder) = b.as_chunks::<4>();
    let (result_chunks, result_rem) = result.as_chunks_mut::<4>();

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

### Example: SIMD Search Operations

Searching through large buffers can benefit from SIMD parallelism by checking multiple elements simultaneously. This example demonstrates the chunking pattern that enables SIMD optimization. Real SIMD search uses vector comparison instructions that check 16-32 bytes per instruction.

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

### Example: SIMD Reduction Operations

Reduction operations like sum benefit from SIMD by accumulating multiple lanes in parallel before the final reduction. Using multiple accumulators (one per SIMD lane) exploits instruction-level parallelism. The final horizontal sum across lanes adds negligible overhead compared to the speedup.

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

### Example: Combining Parallelism with SIMD

Rayon's parallel iterators combined with SIMD operations enable multi-core data parallelism for maximum throughput. Each thread processes a chunk while SIMD accelerates the per-element computation within each chunk. This two-level parallelism can achieve near-theoretical peak performance on modern hardware.

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

### Example: Auto-Vectorization

Simple loops are often auto-vectorized by the compiler. Writing clear, simple code can be as fast as manual SIMD. Compiling with `-C opt-level=3` and `-C target-cpu=native` enables the compiler to use the best available vector instructions.

```rust
fn scale_values(data: &mut [f32], scale: f32) {
    // Compiler can auto-vectorize this loop
    for value in data {
        *value *= scale;
    }
}
```

### Example: Dot Product with Chunking

Dot products are fundamental linear algebra operations that benefit significantly from SIMD processing. Processing 4 multiplications per iteration matches common SIMD register widths. The accumulator pattern minimizes memory traffic by keeping intermediate results in registers.

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

### Example: Image Processing Pipeline

Converting RGB to grayscale is embarrassingly parallel and benefits from SIMD processing of pixel data. The luminance formula weights green most heavily to match human perception. Using integer math with a right-shift avoids expensive floating-point division.

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

### Example: SIMD-Friendly Data Layouts

Structure-of-arrays layout is more SIMD-friendly than array-of-structures for vector operations. When processing many vectors, separating x/y/z into contiguous arrays allows SIMD to load 4-8 components at once. The `#[repr(C)]` ensures predictable memory layout for interop and optimization.

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

### Example: In-Place Vector Compaction

Compacting a vector by removing unwanted elements without allocation uses a two-pointer technique with swap operations. The `write_index` tracks where to place the next kept element, while `read_index` scans through all elements. This is O(N) time regardless of how many elements are removed.

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

### Example: Extracting Elements with Drain

Draining elements that match a predicate into a separate vector while preserving the original's capacity. The `remove` operation shifts subsequent elements, so we don't increment `i` when removing. This is O(N²) worst case; for better performance, use the swap-to-end technique.

```rust
fn extract_matching(
    vec: &mut Vec<Item>,
    predicate: impl Fn(&Item) -> bool,
) -> Vec<Item> {
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

### Example: Splice for Range Replacement

`splice` removes a range and replaces it with new elements in a single operation, more efficient than separate remove and insert. The replacement can be a different size than the removed range—the vector automatically grows or shrinks. The returned iterator yields the removed elements if you need them.

```rust
fn replace_range(
    vec: &mut Vec<i32>,
    start: usize,
    end: usize,
    replacement: &[i32],
) {
    vec.splice(start..end, replacement.iter().copied());
}
```

### Example: Efficient Mid-Vector Insertion

`split_off` divides a vector at an index, enabling efficient insertion without shifting all elements twice. The tail is moved to a new vector, the data is appended, then the tail is appended back. This is more efficient than repeated `insert` calls when adding many elements.

```rust
fn insert_slice_at(vec: &mut Vec<u8>, index: usize, data: &[u8]) {
    let tail = vec.split_off(index);
    vec.extend_from_slice(data);
    vec.extend_from_slice(&tail);
}
```

### Example: Sliding Window Maximum with Deque

VecDeque enables efficient double-ended operations crucial for algorithms like sliding window maximum. The deque stores indices in decreasing order of their values, so the front is always the maximum. This achieves O(N) total time complexity for finding maximum in all sliding windows.

```rust
use std::collections::VecDeque;

fn sliding_window_max(values: &[i32], win_size: usize) -> Vec<i32> {
    let mut result = Vec::new();
    let mut deque = VecDeque::new();

    for (i, &value) in values.iter().enumerate() {
        // Remove elements outside window
        let min_idx = i.saturating_sub(win_size);
        while deque.front().map_or(false, |&idx| idx <= min_idx) {
            deque.pop_front();
        }

        // Remove smaller elements from back
        while deque.back().map_or(false, |&i| values[i] < value) {
            deque.pop_back();
        }

        deque.push_back(i);

        if i >= win_size - 1 {
            result.push(values[*deque.front().unwrap()]);
        }
    }

    result
}
```

### Example: Circular Buffer Implementation

Circular buffers use modular arithmetic to wrap indices, providing efficient fixed-size queues without shifting elements. When full, new elements overwrite the oldest, automatically advancing the head. The `as_slices` method returns two slices representing the logical contiguous view.

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

### Example: Copy-Free Slice Swapping

`std::mem::swap` enables efficient element-wise swapping between two mutable slices without temporary storage. The zip iterator pairs corresponding elements from both slices for simultaneous access. This is useful for interleaving data or implementing in-place transposition.

```rust
fn interleave_slices(a: &mut [u8], b: &mut [u8]) {
    assert_eq!(a.len(), b.len());

    for (a_val, b_val) in a.iter_mut().zip(b.iter_mut()) {
        std::mem::swap(a_val, b_val);
    }
}
```

### Example: Self-Referential Duplication

`extend_from_within` copies a range from within the vector and appends it, useful for repeating patterns. Unlike manual copying, this handles the growing vector safely even when source and destination overlap. This is efficient for run-length encoding decompression or pattern replication.

```rust
fn duplicate_segment(vec: &mut Vec<u8>, start: usize, end: usize) {
    vec.extend_from_within(start..end);
}
```

### Example: Binary Partitioning

Lomuto partition scheme efficiently separates elements based on a predicate, foundational for quicksort and selection algorithms. Non-negative elements accumulate at the front while negative elements are swapped to the back. The returned index is the first position of the "right" partition.

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

### Example: Three-Way Partitioning

Dutch National Flag algorithm partitions into three regions (less than, equal to, greater than) in a single pass. This is optimal for arrays with many duplicate values, as used in three-way quicksort. The returned tuple marks the boundaries between the three regions.

```rust
fn partition_three_way(
    values: &mut [i32],
    pivot: i32,
) -> (usize, usize) {
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

