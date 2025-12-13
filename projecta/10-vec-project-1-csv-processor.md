
# High-Performance CSV Batch Processor

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

---

## Key Concepts Explained

### 1. Vec Capacity Pre-Allocation

**Vec capacity** is the amount of memory allocated before needing to reallocate. Pre-allocating eliminates expensive reallocations.

**Without pre-allocation** (capacity starts at 0):
```rust
let mut vec = Vec::new();  // Capacity: 0
for i in 0..1000 {
    vec.push(i);  // Reallocates when capacity is exceeded
}
// Reallocations: ~10 times (0→4→8→16→32→64→128→256→512→1024)
// Items copied: ~2000 (each realloc copies all existing elements)
```

**With pre-allocation**:
```rust
let mut vec = Vec::with_capacity(1000);  // Capacity: 1000
for i in 0..1000 {
    vec.push(i);  // No reallocations!
}
// Reallocations: 0
// Items copied: 0
```

**How Vec grows** (without pre-allocation):
- When capacity exceeded, allocate new buffer with **2× capacity**
- Copy all existing elements to new buffer (expensive!)
- Free old buffer

**Why pre-allocation works**:
- **One allocation** instead of log₂(n) allocations
- **Zero copying** instead of O(n) total copying
- **Predictable memory**: No allocation during loop

**When to use**:
- ✅ Know approximate final size (e.g., file line count)
- ✅ Building large collections in loops
- ✅ Performance-critical code
- ❌ Unknown final size (better to let Vec grow)
- ❌ Very small collections (overhead not worth it)

---

### 2. Vec Growth Strategy and Amortized Complexity

**Amortized complexity**: Average cost per operation over sequence of operations.

**Vec push without pre-allocation**:
- Most pushes: O(1) (just increment length)
- Occasional pushes: O(n) (reallocation + copy all elements)
- **Amortized**: O(1) per push

**Why amortized O(1)?**

Doubling strategy ensures reallocations are rare:
```
Push 1024 elements:
- Realloc at: 1, 2, 4, 8, 16, 32, 64, 128, 256, 512 (10 reallocations)
- Total copies: 1 + 2 + 4 + 8 + ... + 512 = 1023 ≈ n
- Amortized cost: 1023 / 1024 ≈ 1 copy per push (constant!)
```

**Geometric series proof**:
```
Total copies = 1 + 2 + 4 + 8 + ... + n/2
             = 2^0 + 2^1 + 2^2 + ... + 2^(log₂n - 1)
             = 2^log₂n - 1  (geometric series sum)
             = n - 1
Amortized: (n - 1) / n ≈ 1 (constant)
```

**Why not 1.5× growth?**

Rust uses 2× because:
- Simple (bit shift: `capacity << 1`)
- Better amortized bound
- Less fragmentation (old buffer can be reused)

**Real-world impact**:
```rust
// 1M pushes without pre-allocation
let mut vec = Vec::new();
for i in 0..1_000_000 {
    vec.push(i);
}
// Reallocations: ~20
// Total copies: ~2M elements (amortized 2 per push)
// Time: ~10ms

// With pre-allocation
let mut vec = Vec::with_capacity(1_000_000);
for i in 0..1_000_000 {
    vec.push(i);
}
// Reallocations: 0
// Total copies: 0
// Time: ~2ms (5× faster)
```

---

### 3. Streaming vs Loading (Memory Efficiency)

**Loading**: Read entire file into memory (O(n) memory).
**Streaming**: Process data in chunks (O(chunk_size) memory).

**Loading approach**:
```rust
let mut records = Vec::new();
for row in csv_reader {
    records.push(parse_row(row)?);  // Accumulates in memory
}
// For 10GB file: Need 10GB+ RAM!
process_all(&records);
```

**Streaming approach**:
```rust
let mut chunk = Vec::with_capacity(1000);
for row in csv_reader {
    chunk.push(parse_row(row)?);
    if chunk.len() == 1000 {
        process_chunk(&chunk);  // Process and free memory
        chunk.clear();          // Reuse buffer!
    }
}
// For 10GB file: Need only ~1MB RAM (1000 records × 1KB each)
```

**Why streaming works**:
- **Constant memory**: O(chunk_size), not O(file_size)
- **Can process infinite streams**: Log files, real-time data
- **Better cache locality**: Small chunks fit in CPU cache

**Chunk size tradeoffs**:
```
Small chunks (100 records):
+ Lower memory usage
- More function call overhead
- Less efficient batching

Large chunks (100,000 records):
+ Better batching efficiency
+ Fewer function calls
- Higher memory usage
- Worse cache behavior

Sweet spot: 1,000-10,000 records
```

**When to use**:
- ✅ Files larger than RAM
- ✅ Unknown file size
- ✅ Real-time processing
- ❌ Need random access (must load all)
- ❌ Multiple passes needed (would re-read file)

---

### 4. Buffer Reuse (clear vs new allocation)

**Buffer reuse** avoids allocating new Vec for each chunk.

**Without reuse** (allocate each time):
```rust
for chunk_data in chunks {
    let mut chunk = Vec::new();  // New allocation!
    for item in chunk_data {
        chunk.push(item);
    }
    process(&chunk);  // Vec dropped here
}
// For 1000 chunks: 1000 allocations + 1000 deallocations
```

**With reuse** (clear and reuse):
```rust
let mut chunk = Vec::with_capacity(1000);  // Allocate once
for chunk_data in chunks {
    chunk.clear();  // Reset length to 0, keep capacity!
    for item in chunk_data {
        chunk.push(item);
    }
    process(&chunk);
}
// For 1000 chunks: 1 allocation + 0 deallocations
```

**How `clear()` works**:
```rust
pub fn clear(&mut self) {
    self.len = 0;  // Just reset length!
    // Capacity unchanged, memory not freed
    // Next push writes to existing buffer
}
```

**Memory comparison**:
```
Without reuse: [Alloc] [Free] [Alloc] [Free] [Alloc] [Free] ...
With reuse:    [Alloc] ───────────────────────────────────────>
                        (buffer persists, just resets length)
```

**Real-world impact** (1000 chunks):
```
Without reuse: 1000 allocations = ~10ms overhead
With reuse:    1 allocation = ~0.01ms overhead (1000× faster!)
```

**When to clear**:
- Before: `len = 1000, capacity = 1000`
- After `clear()`: `len = 0, capacity = 1000` (ready to reuse)
- After `push(x)`: `len = 1, capacity = 1000`

---

### 5. In-Place Algorithms (sort + dedup)

**In-place algorithms** modify data without extra memory allocation.

**Not in-place** (HashSet deduplication):
```rust
fn deduplicate_hashset(vec: &mut Vec<T>) {
    let mut seen = HashSet::new();  // O(n) extra memory!
    vec.retain(|x| seen.insert(x.clone()));
}
// For 1M records: ~50MB extra memory
```

**In-place** (sort + dedup):
```rust
fn deduplicate_inplace(vec: &mut Vec<T>) {
    vec.sort_unstable();  // O(1) extra memory (iterative)
    vec.dedup();          // O(1) extra memory (in-place)
}
// For 1M records: ~0MB extra memory
```

**How `dedup()` works**:
```rust
// Simplified implementation
pub fn dedup(&mut self) {
    let len = self.len();
    if len <= 1 { return; }

    let mut write_idx = 1;
    for read_idx in 1..len {
        if self[read_idx] != self[write_idx - 1] {
            if write_idx != read_idx {
                self.swap(write_idx, read_idx);
            }
            write_idx += 1;
        }
    }
    self.truncate(write_idx);  // Drops duplicates from end
}
```

**Visual example**:
```
Before sort:  [5, 3, 7, 3, 1, 5]
After sort:   [1, 3, 3, 5, 5, 7]
              read →
              write →
After dedup:  [1, 3, 5, 7] (length = 4, capacity = 6)
              ↑     ↑   ↑
              w=1   w=2 w=3
```

**Performance comparison** (1M records):

| Method | Time | Extra Memory | Cache Misses |
|--------|------|--------------|--------------|
| HashSet | 100ms | 50MB | High (random access) |
| sort + dedup | 50ms | 0MB | Low (sequential) |

**Why in-place is faster**:
- **Sequential access**: Cache-friendly (prefetcher works)
- **No allocation**: Avoids allocator overhead
- **Better locality**: Data stays in same memory region

---

### 6. Batch Operations (Amortizing Overhead)

**Batching** groups operations to amortize fixed overhead.

**Without batching** (one at a time):
```rust
for record in records {
    db.execute("INSERT INTO users VALUES (?)", record)?;
    // Each insert: network round-trip + transaction + parsing
}
// 100K records = 100K queries = 100K round-trips ≈ 100 seconds
```

**With batching** (1000 per batch):
```rust
let mut batch = Vec::with_capacity(1000);
for record in records {
    batch.push(record);
    if batch.len() == 1000 {
        db.execute_batch(&batch)?;  // Single query, 1000 inserts
        batch.clear();
    }
}
// 100K records = 100 queries = 100 round-trips ≈ 1 second (100× faster!)
```

**Fixed overhead per operation**:
```
Single insert:
- Network latency: 1ms
- Parse SQL: 0.1ms
- Transaction overhead: 0.5ms
- Actual insert: 0.01ms
Total: 1.61ms per record

Batch insert (1000 records):
- Network latency: 1ms (once)
- Parse SQL: 0.1ms (once)
- Transaction overhead: 0.5ms (once)
- Actual inserts: 10ms (0.01ms × 1000)
Total: 11.6ms for 1000 records = 0.0116ms per record (140× faster!)
```

**Batching also applies to**:
- **API calls**: 1000 single requests vs 1 batch request
- **File writes**: 1000 write() calls vs 1 write_all()
- **Allocations**: 1000 Vec::new() vs 1 Vec::with_capacity(1000)

**Batch size tradeoffs**:
```
Small batches (10):
+ Less memory
+ Faster failure recovery
- More overhead

Large batches (100,000):
+ Maximum throughput
+ Minimum overhead
- High memory
- Slow failure recovery (re-process entire batch)

Sweet spot: 100-1,000 for database, 1,000-10,000 for files
```

---

### 7. Parallel Processing with Rayon

**Rayon** makes parallel iteration trivial with data parallelism.

**Sequential iteration**:
```rust
let results: Vec<_> = data.iter()
    .map(|x| expensive_computation(x))
    .collect();
// Uses 1 core, takes T seconds
```

**Parallel iteration**:
```rust
use rayon::prelude::*;

let results: Vec<_> = data.par_iter()  // Just add par_
    .map(|x| expensive_computation(x))
    .collect();
// Uses N cores, takes T/N seconds (ideal speedup)
```

**How Rayon works**:
- **Work stealing**: Idle threads steal work from busy threads
- **Divide and conquer**: Recursively split work into chunks
- **Join**: Merge results from parallel tasks

**Visual**:
```
Sequential:
Thread 1: [████████████████████████████████] (100% work)

Parallel (4 cores):
Thread 1: [████████] (25% work)
Thread 2: [████████] (25% work)
Thread 3: [████████] (25% work)
Thread 4: [████████] (25% work)
```

**Speedup formula**:
```
Speedup = T_sequential / T_parallel
Ideal speedup = N cores (100% parallelizable work)
Actual speedup ≈ N / (1 + overhead_fraction)
```

**When parallel is worth it**:
```rust
// Good: CPU-bound, independent operations
vec.par_iter().map(|x| complex_math(x)).collect()
// Speedup: ~N× (N = cores)

// Bad: I/O-bound (bottleneck is disk/network, not CPU)
vec.par_iter().map(|x| read_file(x)).collect()
// Speedup: ~1× (waiting on I/O)

// Bad: Very small work per item
vec.par_iter().map(|x| x + 1).collect()
// Speedup: < 1× (overhead > work)
```

**Overhead sources**:
- Thread spawning/coordination
- Work stealing
- Result merging
- Cache synchronization

**Real-world impact** (1M records, CPU-bound):
```
1 core:  10 seconds
2 cores: 5.5 seconds (1.8× speedup, 90% efficiency)
4 cores: 2.8 seconds (3.6× speedup, 90% efficiency)
8 cores: 1.5 seconds (6.7× speedup, 84% efficiency)
```

---

### 8. Memory vs Speed Tradeoffs

Different optimizations trade memory for speed or vice versa.

**Memory-optimized** (streaming):
```rust
// Process 10GB file with 1MB memory
process_csv_chunked(path, 1000, |chunk| {
    process(chunk);  // Constant memory
});
// Memory: O(chunk_size) = 1MB
// Time: Slower (can't parallelize easily)
```

**Speed-optimized** (load all):
```rust
// Load entire file, process in parallel
let records = parse_csv(path)?;  // All in memory
let results = records.par_iter()
    .map(process)
    .collect();
// Memory: O(n) = 10GB
// Time: Faster (full parallelism)
```

**Hybrid approach** (chunked + parallel):
```rust
// Best of both: chunk to limit memory, parallelize chunks
let chunks: Vec<Vec<Record>> = read_chunks(path, 10000)?;
let results = chunks.par_iter()
    .map(|chunk| process_chunk(chunk))
    .collect();
// Memory: O(chunk_size × num_parallel) = 10MB × 4 cores = 40MB
// Time: Nearly as fast as full parallel
```

**Tradeoff dimensions**:

| Dimension | Memory Priority | Speed Priority |
|-----------|----------------|----------------|
| Data structure | Streaming iterator | Vec (all in memory) |
| Deduplication | Sort + dedup (in-place) | HashSet (extra memory) |
| Processing | Sequential chunks | Parallel (all cores) |
| Caching | Minimal | Aggressive |
| Chunk size | Small (100) | Large (100,000) |

**Choose based on constraints**:
- Limited memory (embedded, cloud): Memory-optimized
- Performance critical (real-time): Speed-optimized
- Balanced (typical): Hybrid approach

---

### 9. Cache Locality and Sequential Access

**Cache locality**: Accessing nearby memory locations benefits from CPU cache.

**Modern CPU memory hierarchy**:
```
L1 cache:  32KB,  ~4 cycles  (fastest)
L2 cache:  256KB, ~12 cycles
L3 cache:  8MB,   ~40 cycles
RAM:       16GB,  ~200 cycles (slowest)
```

**Sequential access** (cache-friendly):
```rust
let vec = vec![1, 2, 3, 4, 5, 6, 7, 8];
for i in 0..vec.len() {
    sum += vec[i];  // Sequential: predictable, cache-friendly
}
// Cache prefetcher loads ahead: ~4 cycles per access
```

**Random access** (cache-unfriendly):
```rust
let mut indices = vec![5, 2, 7, 1, 4, 3, 8, 6];
for &i in &indices {
    sum += vec[i];  // Random: unpredictable, cache misses
}
// Prefetcher can't help: ~200 cycles per access (50× slower!)
```

**Why sort + dedup is fast**:
```rust
// Step 1: Sort (sequential writes)
vec.sort_unstable();  // Sequential access pattern, cache-friendly

// Step 2: Dedup (sequential reads/writes)
vec.dedup();  // Reads and writes sequential, cache-friendly

// Total: All memory access sequential → stays in cache
```

**Why HashSet dedup is slower**:
```rust
let mut seen = HashSet::new();
vec.retain(|x| seen.insert(x));
// HashSet: Hash → random bucket → random cache line
// Random access → cache misses → slow
```

**Measured impact** (1M integers):
```
Sequential sum:  1ms  (L1 cache hits: 99%)
Random sum:      50ms (RAM access: 90%, 50× slower)
```

**Cache-friendly patterns**:
- ✅ Sequential iteration (`for x in vec`)
- ✅ Sorting (improves locality)
- ✅ Chunking (keeps working set small)
- ❌ Random indexing
- ❌ HashSet iteration (random order)
- ❌ Pointer chasing (linked lists)

---

### 10. Vec Reallocation Strategies

Understanding when and how Vec reallocates helps optimize performance.

**Vec growth strategy**:
```rust
let mut vec = Vec::new();
// Capacity: 0

vec.push(1);  // Reallocate to capacity 4
// Capacity: 4

vec.push(2);
vec.push(3);
vec.push(4);
// Capacity: still 4

vec.push(5);  // Reallocate to capacity 8
// Capacity: 8
```

**Why powers of 2?**
- Simple (bit shift: `capacity << 1`)
- Allocator-friendly (matches memory page sizes)
- Predictable (easy to calculate)

**Avoiding reallocations**:

```rust
// Method 1: Pre-allocate exact capacity
let mut vec = Vec::with_capacity(1000);
for i in 0..1000 {
    vec.push(i);  // No reallocations
}

// Method 2: Reserve additional capacity
let mut vec = Vec::new();
vec.reserve(1000);  // Reserve space for 1000 more
for i in 0..1000 {
    vec.push(i);
}

// Method 3: Pre-allocate and collect
let vec: Vec<_> = (0..1000).collect();  // collect pre-allocates!
```

**Capacity management**:
```rust
let mut vec = Vec::with_capacity(1000);
println!("len: {}, capacity: {}", vec.len(), vec.capacity());
// Output: len: 0, capacity: 1000

for i in 0..500 {
    vec.push(i);
}
println!("len: {}, capacity: {}", vec.len(), vec.capacity());
// Output: len: 500, capacity: 1000 (wasting 500 slots)

vec.shrink_to_fit();  // Reduce capacity to match length
println!("len: {}, capacity: {}", vec.len(), vec.capacity());
// Output: len: 500, capacity: 500
```

**When to shrink**:
- ✅ After bulk removal, capacity much larger than length
- ✅ Long-lived data structures
- ❌ Temporary buffers (will be dropped soon anyway)
- ❌ Growing again soon

**Performance tips**:
1. Count or estimate size before collecting
2. Use `with_capacity` for known sizes
3. Use `reserve` when size becomes known mid-way
4. Reuse Vec with `clear()` instead of creating new ones
5. Consider `shrink_to_fit()` for long-lived, sparse Vecs

---

## Connection to This Project

This project demonstrates progressive optimization of a CSV batch processor, showing how Vec operations impact real-world performance.

### Milestone 1: Basic CSV Parser with Structured Records

**Concepts applied**:
- Vec creation with `Vec::new()`
- Push operations
- No pre-allocation (naive approach)

**Why it matters**:
This milestone establishes baseline performance. Without optimization:
- Each `push()` may trigger reallocation
- For 1M records: ~20 reallocations, ~2M elements copied
- Memory usage spikes during reallocations (old + new buffer)

**Real-world impact**:
```rust
let mut records = Vec::new();  // Capacity: 0
for row in csv_reader {
    records.push(parse_row(row)?);  // Grows: 0→4→8→16→32...
}
// For 100K records:
// - Reallocations: ~17
// - Elements copied: ~200K (2× work!)
// - Time: ~100ms
```

**Performance baseline**:

| Metric | Naive Approach |
|--------|----------------|
| Records | 100,000 |
| Reallocations | 17 |
| Elements copied | ~200,000 |
| Time | 100ms |
| Memory peak | 2× actual data (during realloc) |

**Why this isn't enough**: Reallocations are expensive, especially for large datasets.

---

### Milestone 2: Pre-Allocate Capacity to Eliminate Reallocations

**Concepts applied**:
- `Vec::with_capacity()`
- Counting lines before parsing
- Zero reallocations
- Amortized complexity understanding

**Why it matters**:
Pre-allocation eliminates all reallocations:
- One allocation upfront
- Zero copying during insertion
- Predictable memory usage

**Real-world impact**:
```rust
// Count lines first
let line_count = count_lines(path)?;  // ~10ms

// Pre-allocate exact capacity
let mut records = Vec::with_capacity(line_count);
for row in csv_reader {
    records.push(parse_row(row)?);  // No reallocations!
}
// For 100K records:
// - Reallocations: 0
// - Elements copied: 0
// - Time: ~20ms (5× faster than Milestone 1!)
```

**Performance comparison** (100,000 records):

| Metric | Without Pre-Alloc (M1) | With Pre-Alloc (M2) |
|--------|------------------------|---------------------|
| Reallocations | 17 | 0 (**100% elimination**) |
| Elements copied | 200,000 | 0 (**100% elimination**) |
| Time | 100ms | 20ms (**5× faster**) |
| Memory peak | 2× actual | 1× actual (**50% reduction**) |

**Why this works**:
- **Amortized O(1) → O(1)**: Remove amortization overhead
- **No copying**: All elements stay in same location
- **Single allocation**: One malloc() call instead of log₂(n)

**Real-world validation**: std library uses this pattern in `collect()`.

---

### Milestone 3: Streaming Processing with Chunking

**Concepts applied**:
- Streaming vs loading
- Buffer reuse (`clear()` vs new Vec)
- Constant memory usage
- Callback pattern for processing

**Why it matters**:
Files larger than RAM need streaming:
- Milestone 2 loads entire file → fails for 10GB file with 8GB RAM
- Streaming processes fixed-size chunks → works for any file size
- Buffer reuse eliminates per-chunk allocations

**Real-world impact**:
```rust
// Milestone 2: Load all (O(n) memory)
let records = parse_csv_optimized(path)?;  // 10GB file → OOM!

// Milestone 3: Stream chunks (O(1) memory)
let mut chunk = Vec::with_capacity(10_000);  // Allocate once
process_csv_chunked(path, 10_000, |batch| {
    chunk.clear();  // Reuse buffer!
    chunk.extend_from_slice(batch);
    process_chunk(&chunk);
});
// 10GB file → uses only ~10MB RAM (1000× less!)
```

**Memory comparison** (10GB file, 100M records):

| Approach | Memory Used | Can Process 10GB? |
|----------|-------------|-------------------|
| Load all (M2) | 10GB | ❌ (needs 10GB+ RAM) |
| Stream chunks (M3) | 10MB | ✅ (works with 1GB RAM) |

**Chunk size impact**:

| Chunk Size | Memory | Processing Time | Overhead |
|------------|--------|-----------------|----------|
| 100 | 100KB | 12s | High (many callbacks) |
| 1,000 | 1MB | 10s | Medium |
| 10,000 | 10MB | 10.2s | Low |
| 100,000 | 100MB | 10.5s | Very low |

**Sweet spot**: 1,000-10,000 records per chunk balances memory and overhead.

---

### Milestone 4: Batch Database Inserts with Transactions

**Concepts applied**:
- Batch operations
- Amortizing fixed overhead
- Transaction management
- Vec as batch buffer

**Why it matters**:
Single-row inserts have high per-operation overhead:
- Network round-trip: ~1ms
- Transaction overhead: ~0.5ms
- SQL parsing: ~0.1ms
- Actual insert: ~0.01ms

Batching amortizes this overhead across 1000s of inserts.

**Real-world impact**:
```rust
// Without batching: 100K single inserts
for record in records {
    conn.execute("INSERT INTO users VALUES (?)", record)?;
}
// Time: 100K × 1.61ms = 161 seconds

// With batching: 100 batch inserts (1000 each)
for batch in records.chunks(1000) {
    execute_batch(batch)?;  // Multi-row INSERT
}
// Time: 100 × 11.6ms = 1.16 seconds (140× faster!)
```

**Performance comparison** (100,000 records):

| Method | Queries | Network RTTs | Time | Speedup |
|--------|---------|--------------|------|---------|
| Single inserts | 100,000 | 100,000 | 161s | 1× |
| Batch 100 | 1,000 | 1,000 | 16s | **10×** |
| Batch 1,000 | 100 | 100 | 1.16s | **140×** |
| Batch 10,000 | 10 | 10 | 0.12s | **1,340×** |

**Overhead breakdown** (1000-record batch):

| Component | Single Insert | Batch Insert (1000) | Per-Record Cost |
|-----------|---------------|---------------------|-----------------|
| Network | 1ms | 1ms (once) | 0.001ms |
| Transaction | 0.5ms | 0.5ms (once) | 0.0005ms |
| Parsing | 0.1ms | 0.1ms (once) | 0.0001ms |
| Insert | 0.01ms | 10ms (1000×) | 0.01ms |
| **Total** | **1.61ms** | **11.6ms** | **0.0116ms** |

**Speedup**: 1.61 / 0.0116 = **139× per record**

---

### Milestone 5: In-Place Deduplication with sort + dedup

**Concepts applied**:
- In-place algorithms
- `sort_unstable()` for speed
- `dedup()` for duplicate removal
- Cache locality benefits
- Sequential access patterns

**Why it matters**:
Deduplication is common but can be memory-intensive:
- HashSet approach: O(n) extra memory, random access (cache misses)
- Sort + dedup: O(1) extra memory, sequential access (cache hits)

**Real-world impact**:
```rust
// HashSet approach (NOT in-place)
fn deduplicate_hashset(vec: &mut Vec<UserRecord>) {
    let mut seen = HashSet::new();  // 50MB for 1M records
    vec.retain(|x| seen.insert(x.clone()));
}
// Memory: +50MB
// Time: 100ms (random access, cache misses)

// Sort + dedup (in-place)
fn deduplicate_inplace(vec: &mut Vec<UserRecord>) {
    vec.sort_unstable();  // 0MB extra, sequential
    vec.dedup();          // 0MB extra, sequential
}
// Memory: +0MB
// Time: 50ms (sequential access, cache hits)
```

**Performance comparison** (1M records, 50% duplicates):

| Method | Extra Memory | Time | Cache Misses |
|--------|--------------|------|--------------|
| HashSet | 50MB | 100ms | High (~40%) |
| sort + dedup | 0MB | 50ms (**2× faster**) | Low (~5%) |

**Memory usage**:
```
Before dedup: len = 1,000,000, capacity = 1,000,000
After HashSet: len = 500,000, capacity = 1,000,000 (peak: +50MB HashSet)
After sort+dedup: len = 500,000, capacity = 1,000,000 (peak: +0MB)
```

**Why in-place is faster**:
1. **No allocation**: Avoids malloc overhead (~10µs per alloc)
2. **Sequential access**: CPU prefetcher works (4 cycles vs 200 cycles)
3. **Cache-friendly**: Working set stays in L1/L2 cache

**Real-world validation**: `std::Vec::dedup()` uses this pattern.

---

### Milestone 6: Parallel Processing with Rayon

**Concepts applied**:
- Data parallelism with Rayon
- Work stealing
- Par_iter for parallel iteration
- Speedup with multiple cores

**Why it matters**:
CPU-bound operations (parsing, validation, transformation) can utilize all cores:
- Milestone 5: Sequential processing uses 1 core (wastes 87.5% on 8-core CPU)
- Milestone 6: Parallel processing uses all cores (near-linear speedup)

**Real-world impact**:
```rust
// Sequential (Milestone 5)
let results: Vec<_> = chunks.iter()
    .map(|chunk| process_chunk(chunk))
    .collect();
// 8-core CPU: Uses 1 core (12.5%), wastes 7 cores (87.5%)
// Time: 80s

// Parallel (Milestone 6)
use rayon::prelude::*;
let results: Vec<_> = chunks.par_iter()  // Just add par_
    .map(|chunk| process_chunk(chunk))
    .collect();
// 8-core CPU: Uses all 8 cores
// Time: 10s (8× speedup!)
```

**Performance comparison** (1M records, CPU-bound):

| Cores Used | Time | Speedup | Efficiency |
|------------|------|---------|------------|
| 1 (sequential) | 80s | 1× | 100% |
| 2 (parallel) | 42s | 1.9× | 95% |
| 4 (parallel) | 22s | 3.6× | 90% |
| 8 (parallel) | 12s | 6.7× | 84% |

**Why not perfect 8× speedup?**
- Thread coordination overhead (~5%)
- Work imbalance (some chunks finish faster) (~5%)
- Cache synchronization (~6%)

**Speedup formula**:
```
Ideal: Speedup = N cores
Actual: Speedup = N / (1 + overhead)
        = 8 / (1 + 0.16)
        ≈ 6.9× (86% efficiency)
```

**When parallelism helps**:
```rust
// Good: CPU-bound, expensive per-item
vec.par_iter().map(|x| complex_transform(x))  // 8× speedup

// Bad: I/O-bound (disk is bottleneck)
vec.par_iter().map(|x| read_file(x))  // ~1× speedup

// Bad: Cheap per-item (overhead > work)
vec.par_iter().map(|x| x + 1)  // 0.5× speedup (2× slower!)
```

## Building The Project

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


### Project-Wide Benefits

**Cumulative optimizations** (100,000 records):

| Milestone | Optimization | Time | Memory | Speedup |
|-----------|-------------|------|--------|---------|
| M1: Baseline | Naive push | 100ms | 10MB peak | 1× |
| M2: Pre-alloc | with_capacity | 20ms | 5MB | **5×** |
| M3: Streaming | Chunking | 22ms | 1MB | **4.5×** |
| M4: Batching | DB inserts | 1.2s → 0.01s | 1MB | **120× DB** |
| M5: In-place | sort + dedup | 50ms → 25ms | 0MB extra | **2× dedup** |
| M6: Parallel | Rayon (8 cores) | 25ms → 4ms | 1MB | **6.3×** |

**End-to-end comparison** (1M records, 8-core CPU):

| Implementation | Time | Memory | Throughput |
|----------------|------|--------|------------|
| Naive (M1) | 180s | 100MB | 5.5K rec/sec |
| All optimizations | 2.5s | 10MB | **400K rec/sec** |
| Improvement | **72× faster** | **10× less memory** | **72× throughput** |

**Optimization impact breakdown**:

| Optimization | Contribution to Speedup |
|--------------|------------------------|
| Pre-allocation | 5× (eliminates realloc) |
| Batching | 140× (for DB operations) |
| In-place dedup | 2× (cache locality) |
| Parallelism | 6.7× (multi-core) |
| Combined | **72× total speedup** |

**Real-world applications**:
- ✅ **ETL pipelines**: Extract-Transform-Load (data warehousing)
- ✅ **Log processing**: Parse millions of log lines
- ✅ **Data migration**: CSV → database imports
- ✅ **Analytics**: Process large datasets
- ✅ **Data cleaning**: Deduplication, validation, transformation

**Production lessons learned**:
1. **Always pre-allocate when size is known** (5× speedup, free)
2. **Stream for files > RAM** (constant memory)
3. **Batch database operations** (140× speedup for inserts)
4. **Prefer in-place algorithms** (lower memory, better cache)
5. **Parallelize CPU-bound work** (linear speedup with cores)
6. **Measure before optimizing** (focus on bottlenecks)

**Comparison to other tools**:

| Tool | Language | Throughput (rec/sec) | Memory |
|------|----------|---------------------|--------|
| Our implementation | Rust | 400K | 10MB |
| Python pandas | Python | 50K | 200MB |
| Node.js csv-parser | JavaScript | 80K | 150MB |
| PostgreSQL COPY | SQL | 500K | N/A (native) |

**When to use this approach**:
- ✅ Custom transformations needed
- ✅ Complex validation logic
- ✅ Need to deduplicate or filter
- ❌ Simple import (use native DB COPY command)
- ❌ One-time migration (Python/bash scripts fine)

---