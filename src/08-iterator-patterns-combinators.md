# Iterator Patterns & Combinators

[Pattern 1: Custom Iterators and IntoIterator](#pattern-1-custom-iterators-and-intoiterator)

- Problem: Returning Vec forces allocation; exposing internals breaks
  encapsulation; manual indexing error-prone
- Solution: Implement Iterator trait; IntoIterator for all three forms;
  return impl Iterator
- Why It Matters: Eliminates unnecessary allocations; lazy evaluation;
  free composition
- Use Cases: Custom collections, infinite sequences, computed ranges,
  generators

[Pattern 2: Zero-Allocation Iteration](#pattern-2-zero-allocation-iteration)

- Problem: Intermediate collections waste memory; allocations dominate
  runtime in hot paths
- Solution: Chain adapters without collect(); use fold/try_fold; leverage
  windows/chunks
- Why It Matters: 10-100x faster; zero intermediates; compiler optimizes
  to hand-written loops
- Use Cases: Data pipelines, large datasets, hot paths, parsing, real-time
  streams

[Pattern 3: Iterator Adapter Composition](#pattern-3-iterator-adapter-composition)

- Problem: Nested loops hard to read; intermediate collections waste
  memory; custom operations require boilerplate
- Solution: Compose with map/filter/flat_map/scan/peekable; build custom
  adapters
- Why It Matters: Declarative pipelines document intent; compiler
  optimizes chains to single loop
- Use Cases: Log analysis, ETL, parsers, grouping, batching, complex
  filters

[Pattern 4: Streaming Algorithms](#pattern-4-streaming-algorithms)

- Problem: Loading entire datasets causes OOM; multiple passes multiply
  I/O cost
- Solution: Process one element at a time; maintain minimal state; use
  BufReader::lines()
- Why It Matters: Process data larger than RAM; single-pass algorithms
  halve I/O time
- Use Cases: Log analysis, database ETL, real-time analytics, sensor data,
  infinite streams

[Pattern 5: Parallel Iteration with Rayon](#pattern-5-parallel-iteration-with-rayon)

- Problem: Sequential iteration wastes cores; manual threading complex and
  error-prone
- Solution: Replace .iter() with .par_iter(); let Rayon handle
  work-stealing
- Why It Matters: Near-linear speedup; single character change for 8x
  performance
- Use Cases: Data-intensive computations, batch processing, scientific
  computing, map-reduce

[Iterator Cheat Sheet](#iterator-cheat-sheet)
- common **iterator** pattern

### Overview

Iterators are one of Rust's most powerful abstractions, providing a unified interface for processing sequences of data. Unlike loops in many languages, Rust iterators are zero-cost abstractions: they compile down to the same machine code as hand-written loops, yet offer composability, expressiveness, and safety.

This chapter explores advanced iterator patterns that experienced programmers can leverage to write efficient, elegant code. The key insight is that iterators aren't just for collections—they're a design pattern for lazy, composable computation that can model streaming algorithms, state machines, and complex data transformations.

The patterns we'll explore include:
- Custom iterators and implementing IntoIterator
- Zero-allocation iteration strategies
- Iterator adapter composition for complex transformations
- Streaming algorithms for large datasets
- Parallel iteration with rayon

### Iterator Cheat Sheet

```rust
// Core iterator traits
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    // 70+ provided methods built on next()
}

trait IntoIterator {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;
    fn into_iter(self) -> Self::IntoIter;
}

// Common iterator methods
iter.map(|x| x * 2)              // Transform each element
iter.filter(|x| *x > 0)          // Keep only matching elements
iter.fold(0, |acc, x| acc + x)   // Reduce to single value
iter.collect::<Vec<_>>()         // Consume into collection
iter.take(5)                     // Limit to first n elements
iter.skip(3)                     // Skip first n elements
iter.chain(other)                // Concatenate iterators
iter.zip(other)                  // Pair elements from two iterators
iter.enumerate()                 // Add indices
iter.flat_map(|x| vec![x, x])    // Map and flatten

// Iterator consumers (methods that consume the iterator)
iter.count()                     // Count elements
iter.sum()                       // Sum numeric elements
iter.any(|x| x > 5)             // Check if any match
iter.all(|x| x > 0)             // Check if all match
iter.find(|x| *x == target)     // Find first match
```

## Pattern 1: Custom Iterators and IntoIterator

**Problem**: Returning `Vec` from collection methods forces immediate allocation and copying—even if the caller only needs the first few elements. Exposing internal structure breaks encapsulation. External iteration with manual indexing is error-prone and doesn't work with custom data structures (graphs, trees, generators). Standard for-loops require allocating collections first.

**Solution**: Implement the `Iterator` trait for custom types to enable lazy, composable iteration. Implement `IntoIterator` for owned, borrowed (`&T`), and mutable (`&mut T`) forms to enable for-loop syntax. Use iterator adapters (wrapping other iterators) to extend functionality. Return `impl Iterator` from functions to hide implementation details while enabling zero-allocation iteration.

**Why It Matters**: Custom iterators eliminate unnecessary allocations—a function returning "first 10 primes" as `Vec<u64>` allocates even if caller only checks the first. Iterators are lazy: `Fibonacci::new().take(10)` computes 10 values, not infinite. They compose: `.filter().map().take()` chains without intermediate vectors. This is transformative for API design: libraries can expose iteration without committing to storage format, and code using them gets full iterator method access (map, filter, fold, etc.) for free.

**Use Cases**: Custom collections (trees, graphs, circular buffers), infinite sequences (Fibonacci, primes, random numbers), computed ranges (2D coordinates, date ranges), stateful generators, adapters for external APIs, zero-copy views into data structures.

### Examples

```rust
//===============================
// Pattern: Basic custom iterator
//===============================
struct Counter {
    current: u32,
    max: u32,
}

impl Counter {
    fn new(max: u32) -> Self {
        Counter { current: 0, max }
    }
}

impl Iterator for Counter {
    type Item = u32;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.max {
            let result = self.current;
            self.current += 1;
            Some(result)
        } else {
            None
        }
    }
    
    // Optional: provide size_hint for optimization
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.max - self.current) as usize;
        (remaining, Some(remaining))
    }
}

//=============================================
// Pattern: IntoIterator for custom collections
//=============================================
struct RingBuffer<T> {
    data: Vec<T>,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    fn new(capacity: usize) -> Self {
        RingBuffer {
            data: Vec::new(),
            capacity,
        }
    }
}

// Implement IntoIterator for owned consumption
impl<T> IntoIterator for RingBuffer<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

// Implement IntoIterator for borrowing
impl<'a, T> IntoIterator for &'a RingBuffer<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

// Implement IntoIterator for mutable borrowing
impl<'a, T> IntoIterator for &'a mut RingBuffer<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

//======================================
// Pattern: Iterator with internal state
//======================================
struct Fibonacci {
    current: u64,
    next: u64,
}

impl Fibonacci {
    fn new() -> Self {
        Fibonacci { current: 0, next: 1 }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;
    
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.current;
        let new_next = self.current.checked_add(self.next)?;
        self.current = self.next;
        self.next = new_next;
        Some(result)
    }
}

//================================================
// Pattern: Stateful iterator with filtering logic
//================================================
struct PrimeNumbers {
    current: u64,
    primes: Vec<u64>,
}

impl PrimeNumbers {
    fn new() -> Self {
        PrimeNumbers {
            current: 2,
            primes: Vec::new(),
        }
    }
    
    fn is_prime(&self, n: u64) -> bool {
        for &p in &self.primes {
            if p * p > n {
                break;
            }
            if n % p == 0 {
                return false;
            }
        }
        true
    }
}

impl Iterator for PrimeNumbers {
    type Item = u64;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.is_prime(self.current) {
                let result = self.current;
                self.primes.push(result);
                self.current += if self.current == 2 { 1 } else { 2 };
                return Some(result);
            }
            self.current += if self.current == 2 { 1 } else { 2 };
        }
    }
}

//===================================================
// Pattern: Iterator adapter (wraps another iterator)
//===================================================
struct Batched<I>
where
    I: Iterator,
{
    iter: I,
    batch_size: usize,
}

impl<I> Batched<I>
where
    I: Iterator,
{
    fn new(iter: I, batch_size: usize) -> Self {
        Batched { iter, batch_size }
    }
}

impl<I> Iterator for Batched<I>
where
    I: Iterator,
{
    type Item = Vec<I::Item>;
    
    fn next(&mut self) -> Option<Self::Item> {
        let mut batch = Vec::with_capacity(self.batch_size);
        
        for _ in 0..self.batch_size {
            match self.iter.next() {
                Some(item) => batch.push(item),
                None => break,
            }
        }
        
        if batch.is_empty() {
            None
        } else {
            Some(batch)
        }
    }
}

// Extension trait for batching
trait BatchedExt: Iterator {
    fn batched(self, batch_size: usize) -> Batched<Self>
    where
        Self: Sized,
    {
        Batched::new(self, batch_size)
    }
}

impl<I: Iterator> BatchedExt for I {}
```

**Custom iterator guidelines:**
1. **Implement size_hint when possible**: Enables optimizations in collect() and other consumers
2. **Use checked arithmetic**: Prevent overflow in stateful iterators
3. **Provide IntoIterator for all three forms**: Owned, borrowed, and mutably borrowed
4. **Extension traits for chainability**: Make custom adapters feel native
5. **Lazy evaluation**: Only compute values when next() is called

## Pattern 2: Zero-Allocation Iteration

**Problem**: Processing collections with intermediate steps typically requires allocating temporary vectors: `filter` → allocate Vec → `map` → allocate another Vec → `collect`. For large datasets or hot paths, these allocations dominate runtime. Calling `.collect()` after every transformation step wastes memory. Manual loops avoid allocation but lose composability and are verbose.

**Solution**: Chain iterator adapters without calling `.collect()` until the final result. Use `.iter()` for borrowing instead of `.into_iter()` which consumes. Leverage `fold` and `try_fold` for custom reductions without temporary storage. Use `from_fn` for stateful generators. Employ `.windows()` and `.chunks()` for sliding operations without copying data. Return `impl Iterator` to avoid boxing or collecting.

**Why It Matters**: Zero-allocation iteration can be 10-100x faster than collecting intermediate results. Processing 1M elements with 3 transformations: naive approach allocates 3M+ elements across temporary vectors. Iterator chains allocate zero intermediates—just iterate once, applying transformations on-the-fly. For data pipelines, this means gigabytes saved and cache-friendly sequential access. The compiler often optimizes iterator chains to the same machine code as hand-written loops, giving you high-level abstraction at zero cost.

**Use Cases**: Data processing pipelines (ETL, analytics), filtering and transforming large datasets, hot path operations in servers, parsing without intermediate buffers, mathematical computations on sequences, real-time stream processing.

### Examples

```rust
//===================================================
// Pattern: Chaining without intermediate collections
//===================================================
fn process_numbers(input: &[i32]) -> i32 {
    input
        .iter()
        .filter(|&&x| x > 0)
        .map(|&x| x * x)
        .filter(|&x| x < 1000)
        .sum()
}

//========================================================
// Pattern: Iterator windows for sliding window algorithms
//========================================================
fn moving_average(data: &[f64], window_size: usize) -> Vec<f64> {
    data.windows(window_size)
        .map(|w| w.iter().sum::<f64>() / window_size as f64)
        .collect()
}

//======================================
// Pattern: Avoiding collect() with fold
//======================================
fn count_matches<T>(iter: impl Iterator<Item = T>, predicate: impl Fn(&T) -> bool) -> usize {
    // Instead of: iter.filter(predicate).count()
    iter.fold(0, |count, item| {
        if predicate(&item) {
            count + 1
        } else {
            count
        }
    })
}

//=======================================================
// Pattern: Iterator over computed values (no allocation)
//=======================================================
struct Range2D {
    x: i32,
    y: i32,
    max_x: i32,
    max_y: i32,
}

impl Range2D {
    fn new(max_x: i32, max_y: i32) -> Self {
        Range2D { x: 0, y: 0, max_x, max_y }
    }
}

impl Iterator for Range2D {
    type Item = (i32, i32);
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.y >= self.max_y {
            return None;
        }
        
        let result = (self.x, self.y);
        
        self.x += 1;
        if self.x >= self.max_x {
            self.x = 0;
            self.y += 1;
        }
        
        Some(result)
    }
}

//================================================
// Pattern: Generator-like iteration with closures
//================================================
fn generate<T>(mut state: impl FnMut() -> Option<T>) -> impl Iterator<Item = T> {
    std::iter::from_fn(move || state())
}

//=========================================
// Usage: infinite iterator without storage
//=========================================
fn random_numbers() -> impl Iterator<Item = u32> {
    let mut seed = 12345u32;
    generate(move || {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        Some(seed)
    })
}

//============================================
// Pattern: Chain iterators without collecting
//============================================
fn process_multiple_sources(a: &[i32], b: &[i32], c: &[i32]) -> i32 {
    a.iter()
        .chain(b.iter())
        .chain(c.iter())
        .filter(|&&x| x % 2 == 0)
        .sum()
}

//==========================================================
// Pattern: Flat-map for nested iteration without allocation
//==========================================================
fn expand_ranges(ranges: &[(i32, i32)]) -> impl Iterator<Item = i32> + '_ {
    ranges.iter().flat_map(|&(start, end)| start..end)
}

//====================================================
// Pattern: Try-fold for early exit without allocation
//====================================================
fn find_sum_exceeding(numbers: &[i32], threshold: i32) -> Option<i32> {
    numbers
        .iter()
        .try_fold(0, |sum, &x| {
            let new_sum = sum + x;
            if new_sum > threshold {
                Err(new_sum)
            } else {
                Ok(new_sum)
            }
        })
        .err()
}
```

**Zero-allocation principles:**
1. **Chain adapters instead of collecting**: Each adapter adds minimal overhead
2. **Use fold/try_fold for custom reduction**: Avoid filter().count() patterns
3. **Leverage from_fn for stateful generation**: No need for custom iterator types
4. **Prefer borrowed iteration**: Use .iter() over .into_iter() when possible
5. **Iterator::flatten and flat_map**: Flatten nested structures without intermediate Vec

## Pattern 3: Iterator Adapter Composition

**Problem**: Complex data transformations expressed as nested loops are hard to read and error-prone. Breaking transformations into separate functions with intermediate collections wastes memory. State machines for parsing or grouping require manual bookkeeping. Expressing operations like "group by key", "interleave two sequences", or "cartesian product" requires custom code that can't leverage standard library optimizations.

**Solution**: Compose transformations using iterator adapters: `.map()`, `.filter()`, `.flat_map()`, `.scan()`, `.take_while()`, `.skip_while()`, `.zip()`, `.chain()`, `.peekable()`, etc. Build custom adapters for domain-specific operations. Use `.scan()` for stateful transformations and `.peekable()` for lookahead. Combine adapters to express complex operations declaratively.

**Why It Matters**: Iterator composition turns imperative procedural code into declarative pipelines that document intent. A log analysis pipeline: `logs.filter(errors).filter(recent).group_by(level).count()` reads like a specification. Custom adapters (interleave, batch, group_by) compose with standard ones, building a vocabulary for your domain. The compiler optimizes these chains: a 5-adapter pipeline often compiles to a single loop. This enables writing code that is simultaneously readable, composable, and fast.

**Use Cases**: Log analysis pipelines, data transformation ETL, parsers with lookahead (peekable), grouping and aggregation, mathematical sequences, interleaving or merging streams, batching for bulk processing, complex filtering logic.

### Examples

```rust
use std::collections::HashMap;

//=======================================================
// Pattern: Complex filtering and transformation pipeline
//=======================================================
#[derive(Debug, Clone)]
struct LogEntry {
    timestamp: u64,
    level: String,
    message: String,
}

fn analyze_logs(logs: &[LogEntry]) -> HashMap<String, usize> {
    logs.iter()
        .filter(|entry| entry.level == "ERROR" || entry.level == "WARN")
        .filter(|entry| entry.timestamp > 1000000)
        .map(|entry| &entry.level)
        .fold(HashMap::new(), |mut map, level| {
            *map.entry(level.clone()).or_insert(0) += 1;
            map
        })
}

//==========================================
// Pattern: Chunking with stateful iteration
//==========================================
fn process_in_chunks<T>(items: Vec<T>, chunk_size: usize) -> impl Iterator<Item = Vec<T>> {
    let mut items = items;
    std::iter::from_fn(move || {
        if items.is_empty() {
            None
        } else {
            let drain_end = chunk_size.min(items.len());
            Some(items.drain(..drain_end).collect())
        }
    })
}

//================================
// Pattern: Interleaving iterators
//================================
struct Interleave<I, J> {
    a: I,
    b: J,
    use_a: bool,
}

impl<I, J> Iterator for Interleave<I, J>
where
    I: Iterator,
    J: Iterator<Item = I::Item>,
{
    type Item = I::Item;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.use_a {
            self.use_a = false;
            self.a.next().or_else(|| self.b.next())
        } else {
            self.use_a = true;
            self.b.next().or_else(|| self.a.next())
        }
    }
}

fn interleave<I, J>(a: I, b: J) -> Interleave<I::IntoIter, J::IntoIter>
where
    I: IntoIterator,
    J: IntoIterator<Item = I::Item>,
{
    Interleave {
        a: a.into_iter(),
        b: b.into_iter(),
        use_a: true,
    }
}

//===========================
// Pattern: Cartesian product
//===========================
fn cartesian_product<T: Clone>(a: &[T], b: &[T]) -> impl Iterator<Item = (T, T)> + '_ {
    a.iter().flat_map(move |x| {
        b.iter().map(move |y| (x.clone(), y.clone()))
    })
}

//========================================
// Pattern: Group by key (without sorting)
//========================================
fn group_by<K, V, F>(items: Vec<V>, key_fn: F) -> HashMap<K, Vec<V>>
where
    K: Eq + std::hash::Hash,
    F: Fn(&V) -> K,
{
    items.into_iter().fold(HashMap::new(), |mut map, item| {
        map.entry(key_fn(&item)).or_insert_with(Vec::new).push(item);
        map
    })
}

//========================================
// Pattern: Scan for cumulative operations
//========================================
fn cumulative_sum(numbers: &[i32]) -> Vec<i32> {
    numbers
        .iter()
        .scan(0, |state, &x| {
            *state += x;
            Some(*state)
        })
        .collect()
}

//================================================================
// Pattern: Take while and skip while for prefix/suffix operations
//================================================================
fn extract_header_body(lines: &[String]) -> (Vec<&String>, Vec<&String>) {
    let header: Vec<_> = lines.iter().take_while(|line| !line.is_empty()).collect();
    let body: Vec<_> = lines.iter().skip_while(|line| !line.is_empty()).skip(1).collect();
    (header, body)
}

//================================
// Pattern: Peekable for lookahead
//================================
fn parse_tokens(input: &str) -> Vec<Token> {
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();
    
    while let Some(&ch) = chars.peek() {
        match ch {
            '0'..='9' => {
                let num: String = chars
                    .by_ref()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                tokens.push(Token::Number(num.parse().unwrap()));
            }
            '+' | '-' | '*' | '/' => {
                tokens.push(Token::Op(ch));
                chars.next();
            }
            ' ' => {
                chars.next();
            }
            _ => {
                chars.next();
            }
        }
    }
    
    tokens
}

#[derive(Debug)]
enum Token {
    Number(i32),
    Op(char),
}
```

**Adapter composition principles:**
1. **Build pipelines incrementally**: Each step should be independently testable
2. **Use scan for stateful transformations**: More expressive than manual state tracking
3. **Peekable for lookahead**: Essential for parsers and state machines
4. **Prefer iterator methods over manual loops**: More composable and optimizable
5. **fold for custom aggregations**: More flexible than sum/collect

## Pattern 4: Streaming Algorithms

**Problem**: Loading entire files or datasets into memory causes OOM errors with large data (multi-GB log files, database dumps). Computing statistics requires multiple passes over data, multiplying I/O cost. Finding top-K elements naively requires sorting entire dataset. Algorithms that need to "see all data" seem to require loading everything. Batch processing forces awkward chunking logic.

**Solution**: Process data one element at a time using iterators, maintaining only essential state (aggregates, sliding windows, top-K heaps). Use `BufReader::lines()` for line-by-line file processing. Implement single-pass streaming algorithms (moving average, cumulative sum, top-K with min-heap). Merge sorted streams without loading both. Use `fold` for incremental aggregation. Batch process with fixed-size windows while streaming.

**Why It Matters**: Streaming algorithms enable processing datasets larger than RAM—a 100GB log file processes with constant 1MB memory. Single-pass algorithms are dramatically faster: computing average + variance in one pass vs two passes halves I/O time. Top-K with a heap of size K uses O(K) memory, not O(N). Real-time systems process infinite streams (network packets, sensor data, user events) that can't be collected first. This is the difference between "works on test data" and "works on production scale".

**Use Cases**: Log file analysis (grep-like filtering, statistics), database ETL (processing query results), real-time analytics (streaming averages, alerting), sensor data processing, network packet analysis, infinite sequences (event streams, live data feeds), CSV/JSON parsing of large files.

### Examples

```rust
use std::io::{BufRead, BufReader, Read};
use std::fs::File;

//======================================
// Pattern: Line-by-line file processing
//======================================
fn count_lines_matching(path: &str, pattern: &str) -> std::io::Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    Ok(reader
        .lines()
        .filter_map(Result::ok)
        .filter(|line| line.contains(pattern))
        .count())
}

//=======================================
// Pattern: Streaming average calculation
//=======================================
struct StreamingAverage {
    sum: f64,
    count: usize,
}

impl StreamingAverage {
    fn new() -> Self {
        StreamingAverage { sum: 0.0, count: 0 }
    }
    
    fn add(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;
    }
    
    fn average(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f64)
        }
    }
}

fn compute_streaming_average(numbers: impl Iterator<Item = f64>) -> Option<f64> {
    let mut avg = StreamingAverage::new();
    for num in numbers {
        avg.add(num);
    }
    avg.average()
}

//=======================================================
// Pattern: Top-K elements without sorting entire dataset
//=======================================================
use std::collections::BinaryHeap;
use std::cmp::Reverse;

fn top_k<T: Ord>(iter: impl Iterator<Item = T>, k: usize) -> Vec<T> {
    let mut heap = BinaryHeap::new();
    
    for item in iter {
        if heap.len() < k {
            heap.push(Reverse(item));
        } else if let Some(&Reverse(ref min)) = heap.peek() {
            if &item > min {
                heap.pop();
                heap.push(Reverse(item));
            }
        }
    }
    
    heap.into_iter().map(|Reverse(x)| x).collect()
}

//===================================
// Pattern: Sliding window statistics
//===================================
struct SlidingWindow<T> {
    window: std::collections::VecDeque<T>,
    capacity: usize,
}

impl<T> SlidingWindow<T> {
    fn new(capacity: usize) -> Self {
        SlidingWindow {
            window: std::collections::VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    
    fn push(&mut self, value: T) -> Option<T> {
        if self.window.len() == self.capacity {
            let removed = self.window.pop_front();
            self.window.push_back(value);
            removed
        } else {
            self.window.push_back(value);
            None
        }
    }
    
    fn iter(&self) -> impl Iterator<Item = &T> {
        self.window.iter()
    }
}

fn sliding_window_sum(numbers: impl Iterator<Item = i32>, window_size: usize) -> impl Iterator<Item = i32> {
    let mut window = SlidingWindow::new(window_size);
    let mut sum = 0;
    let mut initialized = false;
    
    numbers.filter_map(move |num| {
        if let Some(old) = window.push(num) {
            sum = sum - old + num;
            Some(sum)
        } else {
            sum += num;
            if window.window.len() == window_size {
                initialized = true;
            }
            if initialized {
                Some(sum)
            } else {
                None
            }
        }
    })
}

//=================================
// Pattern: Streaming deduplication
//=================================
use std::collections::HashSet;

fn deduplicate_stream<T>(iter: impl Iterator<Item = T>) -> impl Iterator<Item = T>
where
    T: Eq + std::hash::Hash + Clone,
{
    let mut seen = HashSet::new();
    iter.filter(move |item| seen.insert(item.clone()))
}

//================================
// Pattern: Rate limiting iterator
//================================
use std::time::{Duration, Instant};

struct RateLimited<I> {
    iter: I,
    interval: Duration,
    last_yield: Option<Instant>,
}

impl<I: Iterator> RateLimited<I> {
    fn new(iter: I, interval: Duration) -> Self {
        RateLimited {
            iter,
            interval,
            last_yield: None,
        }
    }
}

impl<I: Iterator> Iterator for RateLimited<I> {
    type Item = I::Item;
    
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(last) = self.last_yield {
            let elapsed = last.elapsed();
            if elapsed < self.interval {
                std::thread::sleep(self.interval - elapsed);
            }
        }
        
        let item = self.iter.next()?;
        self.last_yield = Some(Instant::now());
        Some(item)
    }
}

//===================================
// Pattern: Buffered batch processing
//===================================
fn process_in_batches<T, F>(
    iter: impl Iterator<Item = T>,
    batch_size: usize,
    mut process_batch: F,
) where
    F: FnMut(Vec<T>),
{
    let mut batch = Vec::with_capacity(batch_size);
    
    for item in iter {
        batch.push(item);
        if batch.len() == batch_size {
            process_batch(std::mem::replace(&mut batch, Vec::with_capacity(batch_size)));
        }
    }
    
    if !batch.is_empty() {
        process_batch(batch);
    }
}

//=============================================
// Pattern: Streaming merge of sorted iterators
//=============================================
fn merge_sorted<T: Ord>(
    mut a: impl Iterator<Item = T>,
    mut b: impl Iterator<Item = T>,
) -> impl Iterator<Item = T> {
    let mut a_next = a.next();
    let mut b_next = b.next();
    
    std::iter::from_fn(move || {
        match (&a_next, &b_next) {
            (Some(a_val), Some(b_val)) => {
                if a_val <= b_val {
                    let result = a_next.take();
                    a_next = a.next();
                    result
                } else {
                    let result = b_next.take();
                    b_next = b.next();
                    result
                }
            }
            (Some(_), None) => {
                let result = a_next.take();
                a_next = a.next();
                result
            }
            (None, Some(_)) => {
                let result = b_next.take();
                b_next = b.next();
                result
            }
            (None, None) => None,
        }
    })
}

//====================================
// Pattern: CSV parsing with streaming
//====================================
fn parse_csv_stream(reader: impl BufRead) -> impl Iterator<Item = Vec<String>> {
    reader.lines().filter_map(Result::ok).map(|line| {
        line.split(',')
            .map(|s| s.trim().to_string())
            .collect()
    })
}

//===================================================
// Pattern: Lazy transformation chain for large files
//===================================================
fn process_large_file(path: &str) -> std::io::Result<Vec<(String, usize)>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    let result = reader
        .lines()
        .filter_map(Result::ok)
        .filter(|line| !line.is_empty())
        .map(|line| line.to_lowercase())
        .flat_map(|line| {
            line.split_whitespace()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .fold(std::collections::HashMap::new(), |mut map, word| {
            *map.entry(word).or_insert(0) += 1;
            map
        })
        .into_iter()
        .collect();
    
    Ok(result)
}
```

**Streaming algorithm principles:**
1. **Process one element at a time**: Never load entire dataset into memory
2. **Use BufReader for files**: Efficiently reads line-by-line or in chunks
3. **Maintain minimal state**: Only store aggregates, not raw data
4. **Combine with fold/scan**: Build running computations
5. **Leverage lazy evaluation**: Transformations happen on-demand

## Pattern 5: Parallel Iteration with Rayon

**Problem**: Sequential iteration leaves CPU cores idle—processing 1M elements on an 8-core machine uses 12.5% capacity. Manual threading with channels and thread pools is complex and error-prone. Partitioning work across threads requires careful load balancing. Race conditions and deadlocks plague hand-written parallel code. Data parallelism seems to require giving up iterator abstractions.

**Solution**: Replace `.iter()` with `.par_iter()` from Rayon to enable automatic parallelization. Use `.par_chunks()` for better cache locality. Apply `fold` + `reduce` for parallel aggregation. Use `par_sort` for automatic parallel sorting. Let Rayon's work-stealing scheduler balance load. Use `par_bridge()` to parallelize sequential iterators. Leverage `scope` for fine-grained control when needed.

**Why It Matters**: Parallel iteration provides near-linear speedup with CPU cores—8 cores can process 7-8x faster with a single character change (`.par_iter()`). Rayon automatically handles work distribution, load balancing, and thread management. The work-stealing scheduler prevents idle threads while maintaining cache efficiency. This enables writing high-level declarative code that runs at maximum hardware speed. For data processing (image processing, log analysis, simulations), parallelization is often free performance: change one word, get 8x speedup.

**Use Cases**: Data-intensive computations (matrix operations, image/video processing), batch processing (file processing, database imports), scientific computing (simulations, Monte Carlo), sorting and searching large datasets, map-reduce patterns, parallel validation, embarrassingly parallel workloads.

### Examples

```rust
use rayon::prelude::*;

//==================================
// Pattern: Basic parallel iteration
//==================================
fn parallel_sum(numbers: &[i32]) -> i32 {
    numbers.par_iter().sum()
}

fn parallel_map(numbers: &[i32]) -> Vec<i32> {
    numbers.par_iter().map(|&x| x * x).collect()
}

//===============================================
// Pattern: Parallel filtering and transformation
//===============================================
#[derive(Debug, Clone)]
struct Record {
    id: u64,
    value: f64,
    category: String,
}

fn parallel_process_records(records: &[Record], threshold: f64) -> Vec<Record> {
    records
        .par_iter()
        .filter(|r| r.value > threshold)
        .filter(|r| r.category == "active")
        .cloned()
        .collect()
}

//===============================================
// Pattern: Parallel reduce with custom operation
//===============================================
fn parallel_max(numbers: &[i32]) -> Option<i32> {
    numbers.par_iter().copied().max()
}

fn parallel_custom_reduce(numbers: &[i32]) -> i32 {
    numbers
        .par_iter()
        .fold(|| 0, |acc, &x| acc + x * x)
        .sum()
}

//=======================
// Pattern: Parallel sort
//=======================
fn parallel_sort(mut data: Vec<i32>) -> Vec<i32> {
    data.par_sort_unstable();
    data
}

//=====================================
// Pattern: Parallel chunked processing
//=====================================
fn parallel_chunk_processing(data: &[u8], chunk_size: usize) -> Vec<u32> {
    data.par_chunks(chunk_size)
        .map(|chunk| {
            // Expensive computation per chunk
            chunk.iter().map(|&b| b as u32).sum()
        })
        .collect()
}

//==================================
// Pattern: Parallel file processing
//==================================
fn parallel_process_files(paths: &[String]) -> Vec<usize> {
    paths
        .par_iter()
        .map(|path| {
            std::fs::read_to_string(path)
                .map(|content| content.lines().count())
                .unwrap_or(0)
        })
        .collect()
}

//====================================
// Pattern: Parallel find (early exit)
//====================================
fn parallel_find_first(numbers: &[i32], target: i32) -> Option<usize> {
    numbers
        .par_iter()
        .position_any(|&x| x == target)
}

//=====================================
// Pattern: Parallel fold with combiner
//=====================================
fn parallel_word_count(lines: &[String]) -> usize {
    lines
        .par_iter()
        .map(|line| line.split_whitespace().count())
        .sum()
}

//============================
// Pattern: Parallel partition
//============================
fn parallel_partition(numbers: Vec<i32>) -> (Vec<i32>, Vec<i32>) {
    numbers
        .into_par_iter()
        .partition(|&x| x % 2 == 0)
}

//=================================================
// Pattern: Parallel nested iteration with flat_map
//=================================================
fn parallel_matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let b_cols: Vec<Vec<f64>> = (0..b[0].len())
        .map(|col| b.iter().map(|row| row[col]).collect())
        .collect();
    
    a.par_iter()
        .map(|row| {
            b_cols
                .iter()
                .map(|col| row.iter().zip(col.iter()).map(|(a, b)| a * b).sum())
                .collect()
        })
        .collect()
}

//===============================================================
// Pattern: Parallel bridge for converting sequential to parallel
//===============================================================
use std::sync::mpsc::channel;

fn parallel_bridge_example() {
    let (sender, receiver) = channel();
    
    std::thread::spawn(move || {
        for i in 0..1000 {
            sender.send(i).unwrap();
        }
    });
    
    let sum: i32 = receiver.into_iter().par_bridge().map(|x| x * x).sum();
    println!("Sum: {}", sum);
}

//============================================
// Pattern: Controlling parallelism with scope
//============================================
fn parallel_with_scope(data: &mut [i32]) {
    rayon::scope(|s| {
        let mid = data.len() / 2;
        let (left, right) = data.split_at_mut(mid);
        
        s.spawn(|_| {
            for x in left.iter_mut() {
                *x *= 2;
            }
        });
        
        s.spawn(|_| {
            for x in right.iter_mut() {
                *x *= 3;
            }
        });
    });
}

//=====================================
// Pattern: Parallel map-reduce pattern
//=====================================
fn parallel_map_reduce(data: &[String]) -> std::collections::HashMap<String, usize> {
    use std::collections::HashMap;
    
    data.par_iter()
        .fold(
            || HashMap::new(),
            |mut map, line| {
                for word in line.split_whitespace() {
                    *map.entry(word.to_string()).or_insert(0) += 1;
                }
                map
            },
        )
        .reduce(
            || HashMap::new(),
            |mut a, b| {
                for (key, count) in b {
                    *a.entry(key).or_insert(0) += count;
                }
                a
            },
        )
}

//================================================
// Pattern: Parallel pipeline with multiple stages
//================================================
fn parallel_pipeline(data: &[i32]) -> Vec<i32> {
    data.par_iter()
        .map(|&x| x * 2)        // Stage 1: multiply
        .filter(|&x| x > 100)   // Stage 2: filter
        .map(|x| x / 3)         // Stage 3: divide
        .collect()
}

//==================================
// Pattern: Custom parallel iterator
//==================================
struct ParallelRange {
    start: usize,
    end: usize,
}

impl ParallelRange {
    fn new(start: usize, end: usize) -> Self {
        ParallelRange { start, end }
    }
}

//=======================================
// Pattern: Joining parallel computations
//=======================================
fn parallel_join_example(data: &[i32]) -> (i32, i32) {
    let (sum, product) = rayon::join(
        || data.par_iter().sum(),
        || data.par_iter().product(),
    );
    (sum, product)
}
```

**Parallel iteration principles:**
1. **Use par_iter for parallel iteration**: Drop-in replacement for .iter()
2. **Automatic work stealing**: Rayon balances load across threads
3. **fold + reduce for aggregation**: Parallel-friendly accumulation
4. **Chunk size matters**: Use par_chunks for better cache locality
5. **Measure before parallelizing**: Overhead can exceed benefits for small datasets

### Summary

Iterator patterns in Rust enable writing code that is both elegant and efficient. By mastering custom iterators, zero-allocation techniques, adapter composition, streaming algorithms, and parallel iteration, you can:

- **Build composable abstractions**: Iterators chain naturally without sacrificing performance
- **Process data lazily**: Compute only what's needed, when it's needed
- **Handle large datasets**: Stream data without loading everything into memory
- **Leverage parallelism**: Scale computations across CPU cores transparently
- **Maintain zero-cost abstractions**: Iterator code compiles to optimal machine code

**Key takeaways:**
1. Implement custom iterators for domain-specific iteration patterns
2. Chain adapters instead of collecting intermediate results
3. Use fold/scan/try_fold for stateful transformations
4. Stream data with BufReader and lazy iterators for large files
5. Apply rayon's par_iter for automatic parallelization
6. Prefer iterator methods over manual loops for composability

Iterators aren't just a convenience—they're a fundamental design pattern in Rust that enables writing high-level code that runs as fast as hand-optimized loops. Master these patterns to unlock the full power of Rust's iterator ecosystem.


### Pattern Matching Cheat Sheet
```rust
// ===== CREATING ITERATORS =====
// From collections
let vec = vec![1, 2, 3, 4, 5];
vec.iter()                                          // Iterator over &T
vec.iter_mut()                                      // Iterator over &mut T
vec.into_iter()                                     // Iterator over T (consumes)

let arr = [1, 2, 3, 4, 5];
arr.iter()                                          // Iterator over &T
arr.into_iter()                                     // Iterator over &T (arrays)

let slice = &[1, 2, 3, 4, 5][..];
slice.iter()                                        // Iterator over &T

// From ranges
(0..10)                                             // Range: 0 to 9
(0..=10)                                            // Inclusive range: 0 to 10
(0..)                                               // Infinite range from 0
(..10)                                              // Range from start to 9

// Iterate strings
let s = "hello";
s.chars()                                           // Iterator over chars
s.bytes()                                           // Iterator over bytes
s.lines()                                           // Iterator over lines
s.split_whitespace()                                // Iterator over words

// HashMap iteration
use std::collections::HashMap;
let map = HashMap::from([("a", 1), ("b", 2)]);
map.iter()                                          // Iterator over (&K, &V)
map.keys()                                          // Iterator over &K
map.values()                                        // Iterator over &V
map.into_iter()                                     // Iterator over (K, V)

// ===== CONSUMING ITERATORS =====
let vec = vec![1, 2, 3, 4, 5];

// Collect into collection
vec.iter().collect::<Vec<_>>()                     // Collect to Vec
vec.iter().collect::<HashSet<_>>()                 // Collect to HashSet
vec.iter().cloned().collect::<Vec<i32>>()         // Clone elements

// Count elements
vec.iter().count()                                  // Number of elements
vec.iter().filter(|x| **x > 2).count()            // Count matching

// Find elements
vec.iter().find(|x| **x > 3)                       // First matching: Option<&T>
vec.iter().position(|x| *x > 3)                    // Index of first match: Option<usize>
vec.iter().rposition(|x| *x > 3)                   // Last matching index

// Any/all predicates
vec.iter().any(|x| *x > 3)                         // true if any match
vec.iter().all(|x| *x > 0)                         // true if all match

// Nth element
vec.iter().nth(2)                                   // Get element at index: Option<&T>
vec.iter().last()                                   // Get last element

// Max/min
vec.iter().max()                                    // Maximum: Option<&T>
vec.iter().min()                                    // Minimum: Option<&T>
vec.iter().max_by(|a, b| a.cmp(b))                // Max with comparator
vec.iter().max_by_key(|x| x.abs())                // Max by key function

// Sum/product
vec.iter().sum::<i32>()                            // Sum all elements
vec.iter().product::<i32>()                        // Product of all elements

// Fold/reduce
vec.iter().fold(0, |acc, x| acc + x)               // Fold with initial value
vec.iter().reduce(|acc, x| acc + x)                // Reduce (no initial): Option<T>

// For each
vec.iter().for_each(|x| println!("{}", x))        // Apply function to each

// Partition
let (even, odd): (Vec<_>, Vec<_>) = vec.iter()
    .partition(|x| *x % 2 == 0);                   // Split by predicate

// ===== ADAPTER METHODS (LAZY) =====
// Map - transform elements
vec.iter().map(|x| x * 2)                          // Transform each element
vec.iter().map(|x| x.to_string())                  // Change type

// Filter - keep matching elements
vec.iter().filter(|x| **x > 2)                     // Keep elements matching predicate
vec.iter().filter(|x| **x % 2 == 0)               // Keep even numbers

// Filter map - filter and map combined
vec.iter().filter_map(|x| {
    if *x > 2 { Some(x * 2) } else { None }
})

// Take - limit number of elements
vec.iter().take(3)                                  // First 3 elements
vec.iter().take_while(|x| **x < 4)                 // Take while condition true

// Skip - skip elements
vec.iter().skip(2)                                  // Skip first 2 elements
vec.iter().skip_while(|x| **x < 3)                 // Skip while condition true

// Chain - concatenate iterators
vec.iter().chain([6, 7, 8].iter())                 // Concatenate two iterators

// Zip - combine two iterators
let names = vec!["Alice", "Bob"];
let ages = vec![25, 30];
names.iter().zip(ages.iter())                      // Iterator over pairs

// Enumerate - add index
vec.iter().enumerate()                              // Iterator over (index, value)

// Cycle - repeat infinitely
vec.iter().cycle()                                  // Infinite repetition
vec.iter().cycle().take(10)                        // Repeat with limit

// Rev - reverse iterator
vec.iter().rev()                                    // Reverse order

// Cloned/copied - convert &T to T
vec.iter().cloned()                                 // Clone each element
vec.iter().copied()                                 // Copy each element (Copy trait)

// Step by - take every nth element
(0..10).step_by(2)                                  // 0, 2, 4, 6, 8

// Scan - stateful map
(1..5).scan(0, |acc, x| {
    *acc += x;
    Some(*acc)
})                                                  // 1, 3, 6, 10 (running sum)

// Flat map - map and flatten
vec![vec![1, 2], vec![3, 4]]
    .iter()
    .flat_map(|v| v.iter())                        // Iterator over all elements

// Flatten - flatten nested structures
vec![vec![1, 2], vec![3, 4]]
    .into_iter()
    .flatten()                                      // 1, 2, 3, 4

// Inspect - peek at elements (for debugging)
vec.iter()
    .inspect(|x| println!("About to filter: {}", x))
    .filter(|x| **x > 2)
    .inspect(|x| println!("After filter: {}", x))
    .collect::<Vec<_>>()

// Peekable - look ahead without consuming
let mut iter = vec.iter().peekable();
if let Some(&&first) = iter.peek() {               // Look at first without consuming
    println!("First: {}", first);
}
iter.next();                                        // Now consume first

// Fuse - stop after first None
let vec = vec![Some(1), Some(2), None, Some(4)];
vec.into_iter().fuse().collect::<Vec<_>>()        // Stops at None

// ===== CHAINING COMBINATORS =====
// Complex pipeline
let result: Vec<_> = vec
    .iter()
    .filter(|x| **x > 1)                           // Keep > 1
    .map(|x| x * 2)                                // Double
    .filter(|x| *x < 10)                           // Keep < 10
    .collect();

// Multiple transformations
let result: i32 = (1..10)
    .filter(|x| x % 2 == 0)                        // Even numbers
    .map(|x| x * x)                                // Square
    .take(3)                                        // First 3
    .sum();                                         // Sum

// ===== IMPLEMENTING ITERATOR =====
struct Counter {
    count: u32,
}

impl Counter {
    fn new() -> Counter {
        Counter { count: 0 }
    }
}

impl Iterator for Counter {
    type Item = u32;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.count < 5 {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}

let counter = Counter::new();
for num in counter {
    println!("{}", num);
}

// ===== ITERATOR TRAITS =====
// Iterator - basic iteration
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

// DoubleEndedIterator - iterate from both ends
let vec = vec![1, 2, 3, 4, 5];
let mut iter = vec.iter();
iter.next()                                         // Some(&1) from front
iter.next_back()                                    // Some(&5) from back

// ExactSizeIterator - known size
vec.iter().len()                                    // Exact remaining length

// FusedIterator - always None after first None

// ===== INFINITE ITERATORS =====
// Repeat value
std::iter::repeat(5).take(3)                       // [5, 5, 5]

// Repeat with function
std::iter::repeat_with(|| rand::random::<i32>())
    .take(5)                                        // 5 random numbers

// Successors - generate sequence
std::iter::successors(Some(1), |n| Some(n * 2))
    .take(5)                                        // [1, 2, 4, 8, 16]

// Once - single element iterator
std::iter::once(5)                                  // Single element

// Empty - empty iterator
std::iter::empty::<i32>()                          // No elements

// From function
std::iter::from_fn(|| Some(42)).take(3)            // Custom generator

// ===== WINDOWS AND CHUNKS =====
let slice = &[1, 2, 3, 4, 5];

// Windows - overlapping chunks
slice.windows(3)                                    // [[1,2,3], [2,3,4], [3,4,5]]

// Chunks - non-overlapping chunks
slice.chunks(2)                                     // [[1,2], [3,4], [5]]
slice.chunks_exact(2)                              // [[1,2], [3,4]] (drops remainder)

// Mutable versions
let mut vec = vec![1, 2, 3, 4, 5];
vec.chunks_mut(2)                                   // Mutable chunks
    .for_each(|chunk| chunk[0] *= 2);

// ===== ITERATOR METHODS WITH CLOSURES =====
// Find with closure
vec.iter().find(|&&x| x > 3)

// Find map - find and transform
vec.iter().find_map(|&x| {
    if x > 3 { Some(x * 2) } else { None }
})

// Map while - map until condition fails
(1..10).map_while(|x| {
    if x < 5 { Some(x * 2) } else { None }
})

// Try fold - fold that can short-circuit
vec.iter().try_fold(0, |acc, &x| {
    if x > 0 { Some(acc + x) } else { None }
})

// Try for each - for each that can short-circuit
vec.iter().try_for_each(|&x| {
    if x > 0 { Some(()) } else { None }
})

// ===== COMMON PATTERNS =====
// Pattern 1: Collect with type inference
let doubled: Vec<i32> = vec.iter().map(|x| x * 2).collect();

// Pattern 2: Filter and collect
let evens: Vec<_> = vec.iter()
    .filter(|&&x| x % 2 == 0)
    .collect();

// Pattern 3: Sum after transformation
let sum: i32 = vec.iter()
    .map(|x| x * 2)
    .sum();

// Pattern 4: Find first matching
if let Some(&value) = vec.iter().find(|&&x| x > 3) {
    println!("Found: {}", value);
}

// Pattern 5: Group by (requires itertools crate)
use itertools::Itertools;
let grouped: Vec<_> = vec.iter()
    .group_by(|&&x| x % 2)
    .into_iter()
    .map(|(key, group)| (key, group.collect::<Vec<_>>()))
    .collect();

// Pattern 6: Cartesian product
let a = vec![1, 2];
let b = vec!['a', 'b'];
for x in &a {
    for y in &b {
        println!("{}{}", x, y);
    }
}

// Or with itertools:
use itertools::Itertools;
a.iter().cartesian_product(b.iter())

// Pattern 7: Chunking with collect
let chunks: Vec<Vec<_>> = vec.chunks(2)
    .map(|chunk| chunk.to_vec())
    .collect();

// Pattern 8: Conditional iteration
let result = if condition {
    vec.iter().map(|x| x * 2).collect()
} else {
    vec.iter().map(|x| x * 3).collect()
};

// Pattern 9: Iterator over Option/Result
let vec = vec![Some(1), None, Some(3)];
let values: Vec<_> = vec.into_iter()
    .flatten()                                      // Skip None values
    .collect();

let results = vec![Ok(1), Err("error"), Ok(3)];
let values: Result<Vec<_>, _> = results.into_iter().collect(); // Collect Result

// Pattern 10: Custom step iteration
(0..10)
    .step_by(2)
    .collect::<Vec<_>>()                           // [0, 2, 4, 6, 8]

// Pattern 11: Zip with index
vec.iter()
    .enumerate()
    .map(|(i, x)| format!("{}: {}", i, x))
    .collect::<Vec<_>>()

// Pattern 12: Parallel iteration with Rayon
use rayon::prelude::*;
vec.par_iter()
    .map(|x| x * 2)
    .collect::<Vec<_>>()

// Pattern 13: Sorted iteration (requires itertools)
use itertools::Itertools;
vec.iter().sorted().collect::<Vec<_>>()

// Pattern 14: Unique elements (requires itertools)
vec.iter().unique().collect::<Vec<_>>()

// Pattern 15: Intersperse elements (requires itertools)
vec.iter().intersperse(&0).collect::<Vec<_>>()    // Insert 0 between elements

// Pattern 16: Take n largest/smallest
use itertools::Itertools;
vec.iter().k_largest(3).collect::<Vec<_>>()

// Pattern 17: Fold with early exit using try_fold
let result = vec.iter().try_fold(0, |acc, &x| {
    if x < 0 {
        None                                        // Stop early
    } else {
        Some(acc + x)
    }
});

// Pattern 18: Create HashMap from iterator
let map: HashMap<_, _> = vec.iter()
    .enumerate()
    .map(|(i, &x)| (i, x))
    .collect();

// Pattern 19: Batching (requires itertools)
use itertools::Itertools;
vec.iter().chunks(3)
    .into_iter()
    .map(|chunk| chunk.collect::<Vec<_>>())
    .collect::<Vec<_>>()

// Pattern 20: Cycle through values
let cycle_iter = vec.iter().cycle().take(10);
```