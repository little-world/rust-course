# Iterator Patterns & Combinators

Iterators are providing a unified interface for processing sequences of data. Unlike loops in many languages, Rust iterators are zero-cost abstractions: they compile down to the same machine code as hand-written loops, yet offer composability, expressiveness, and safety.

 The key insight is that iterators aren't just for collections—they're a design pattern for lazy, composable computation that can model streaming algorithms, state machines, and complex data transformations.


## Pattern 1: Custom Iterators and `IntoIterator`

**Problem**: You have a custom data structure (like a tree, graph, or a special-purpose buffer) and you want to allow users to loop over it using a standard `for` loop. Returning a `Vec` of items is inefficient as it requires allocating memory for all items at once.

**Solution**: Implement the `Iterator` trait for a helper struct that holds the iteration state. Then, implement the `IntoIterator` trait for your main data structure, which creates and returns an instance of your iterator struct.

**Why It Matters**: This pattern provides a clean, idiomatic, and efficient way to expose the contents of your data structures. Because iterators are lazy, no computation or allocation happens until the caller actually starts consuming items.

**Use Cases**:
-   Custom collections like trees, graphs, or ring buffers.
-   Infinite or procedurally generated sequences (e.g., Fibonacci numbers, prime numbers).
-   Stateful generators that compute values on the fly.
-   Adapters for external data sources or APIs.

### Example: A Basic Custom Iterator

This `Counter` struct demonstrates the simplest form of a custom iterator. It holds mutable state (`current`) that advances each time `next()` is called. Returning `None` signals the end of iteration, allowing the iterator to be used with all standard adapters.

```rust
struct Counter {
    current: u32,
    max: u32,
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
}

// Usage: create a counter and sum its values
let sum: u32 = Counter { current: 0, max: 5 }.sum();
assert_eq!(sum, 10); // 0 + 1 + 2 + 3 + 4
```

### Example: Implementing `IntoIterator`

To make a custom collection work with `for` loops, you need to implement `IntoIterator`. The trait has two associated types: `Item` (what you yield) and `IntoIter` (the iterator type). Implementing it for `&T` and `&mut T` as well enables `for item in &collection` syntax.

```rust
struct RingBuffer<T> {
    data: Vec<T>,
}

// For `for item in my_buffer` (consumes the buffer)
impl<T> IntoIterator for RingBuffer<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

// For `for item in &my_buffer` (borrows the buffer)
impl<'a, T> IntoIterator for &'a RingBuffer<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}
// Usage: iterate over a custom collection with for loop
let buffer = RingBuffer { data: vec![1, 2, 3] };
for item in &buffer { println!("{}", item); }
```

### Example: An Infinite Iterator

Iterators can represent infinite sequences because they are lazy—no values are computed until requested. This `Fibonacci` iterator produces numbers indefinitely until overflow or until stopped by an adapter like `.take()`. Using `checked_add` returns `None` on overflow, gracefully terminating the sequence.

```rust
struct Fibonacci {
    current: u64,
    next: u64,
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.current;
        // Use `checked_add` to handle overflow gracefully.
        let new_next = self.current.checked_add(self.next)?;
        self.current = self.next;
        self.next = new_next;
        Some(result)
    }
}

// Usage: take the first 10 Fibonacci numbers
let fib = Fibonacci { current: 0, next: 1 };
let fibs: Vec<_> = fib.take(10).collect();
assert_eq!(fibs, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
```

**Custom iterator guidelines:**
1. **Implement size_hint when possible**: Enables optimizations in collect() and other consumers
2. **Use checked arithmetic**: Prevent overflow in stateful iterators
3. **Provide IntoIterator for all three forms**: Owned, borrowed, and mutably borrowed
4. **Extension traits for chainability**: Make custom adapters feel native
5. **Lazy evaluation**: Only compute values when next() is called

## Pattern 2: Zero-Allocation Iteration

**Problem**: When processing large datasets, chaining operations like `map` and `filter` can be inefficient if each step allocates a new, intermediate collection. This can lead to high memory usage and poor cache performance.

**Solution**: Chain iterator adapters together without calling `.collect()` until the very end. Iterators in Rust are "lazy," meaning they don't do any work until a "consuming" method like `collect()`, `sum()`, or `count()` is called.

**Why It Matters**: This pattern is fundamental to writing high-performance data processing code in Rust. It allows you to write high-level, declarative code that is just as fast as a hand-written, low-level loop.

**Use Cases**:
-   Data processing pipelines (e.g., in ETL jobs or data analysis).
-   Filtering, transforming, and aggregating data from large files or databases.
-   High-performance code in hot paths, such as in network servers or game engines.
-   Parsing and stream processing.

### Example: Chaining Adapters without Intermediate Collections

This function processes a slice of numbers by filtering, mapping, filtering again, and summing—all in a single pass. No intermediate `Vec` is created because each adapter wraps the previous one without allocating. The compiler often optimizes such chains into a single loop.

```rust
fn process_numbers(input: &[i32]) -> i32 {
    input
        .iter()
        .filter(|&&x| x > 0)
        .map(|&x| x * x)
        .filter(|&x| x < 1000)
        .sum() // The iterator is consumed only at the end.
}
// Usage: filter, map, filter, and sum in a single pass
let result = process_numbers(&[-1, 2, 3, 50]); // 2*2 + 3*3 = 13
```

### Example: Using `windows` for Sliding Window

The `.windows()` method creates an iterator that yields overlapping slices of the original data. Each slice is a view into the original array, so no copying occurs during iteration. This is ideal for implementing moving averages, convolutions, or any algorithm that examines consecutive elements.

```rust
fn moving_average(data: &[f64], window_size: usize) -> Vec<f64> {
    data.windows(window_size)
        .map(|w| w.iter().sum::<f64>() / window_size as f64)
        .collect()
}
// Usage: compute moving average over sliding windows
let avgs = moving_average(&[1.0, 2.0, 3.0, 4.0], 2);
```

### Example: `fold` for Custom Reductions

Instead of creating a new collection just to count items, `fold` performs custom aggregations in a single pass. It takes an initial accumulator and a closure that combines each element with the running total. This is more flexible than specialized methods like `sum()` or `count()`.

```rust
fn count_long_strings(strings: &[&str]) -> usize {
    strings
        .iter()
        .fold(0, |n, &s| if s.len() > 10 { n + 1 } else { n })
}
// Usage: count strings longer than 10 characters
let count = count_long_strings(&["short", "a long string here"]);
// count = 1 (only "a long string here" has len > 10)
```

**Zero-allocation principles:**
1. **Chain adapters instead of collecting**: Each adapter adds minimal overhead
2. **Use fold/try_fold for custom reduction**: Avoid filter().count() patterns
3. **Leverage from_fn for stateful generation**: No need for custom iterator types
4. **Prefer borrowed iteration**: Use .iter() over .into_iter() when possible
5. **Iterator::flatten and flat_map**: Flatten nested structures without intermediate Vec

## Pattern 3: Advanced Iterator Composition

**Problem**: Standard iterator adapters cover many use cases, but sometimes you need more specialized logic, such as stateful transformations, lookahead, or custom grouping, without resorting to manual loops.

**Solution**: Use more advanced adapters like `.scan()`, `.peekable()`, and `.flat_map()` to build complex, declarative data processing pipelines. `.scan()` is perfect for stateful transformations like cumulative sums.

**Why It Matters**: These advanced tools allow you to express complex logic while staying within the iterator paradigm. This keeps your code declarative, composable, and often more performant than a manual implementation, as the compiler can still optimize the entire iterator chain.

**Use Cases**:
-   Log analysis and data transformation pipelines.
-   Parsers that need lookahead.
-   Grouping and aggregating data by a key.
-   Generating cumulative statistics or running totals.
-   Interleaving or merging multiple data streams.


### Example: Complex filtering and transformation pipeline

This iterator chain filters log entries by level and timestamp, then groups results by level. Multiple filters chain together without creating intermediate vectors. The final `fold` accumulates counts into a `HashMap` in a single pass.

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct LogEntry {
    timestamp: u64,
    level: String,
    message: String,
}

fn analyze_logs(logs: &[LogEntry]) -> HashMap<String, usize> {
    logs.iter()
        .filter(|e| e.level == "ERROR" || e.level == "WARN")
        .filter(|entry| entry.timestamp > 1_000_000)
        .map(|entry| &entry.level)
        .fold(HashMap::new(), |mut map, level| {
            *map.entry(level.clone()).or_insert(0) += 1;
            map
        })
}
// Usage: filter logs by level and count occurrences
let counts = analyze_logs(&logs); // {"ERROR": 5, "WARN": 3}
```

### Example: Chunking with stateful iteration

`std::iter::from_fn` creates an iterator from a closure, perfect for stateful generation. Here the closure captures a `Vec` and drains `chunk_size` elements each call. This yields batches lazily without requiring a custom iterator struct.

```rust
fn process_in_chunks<T>(
    items: Vec<T>,
    chunk_size: usize,
) -> impl Iterator<Item = Vec<T>> {
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
// Usage: process items in batches of specified size
for batch in process_in_chunks(vec![1,2,3,4,5], 2) {
    println!("{:?}", batch);
}
```

### Example: Interleaving iterators

This custom iterator alternates between two input iterators, yielding one element from each in turn. When one iterator is exhausted, it falls back to the other until both are empty. The `use_a` flag tracks which source to pull from next.

```rust
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

fn interleave<I, J>(
    a: I,
    b: J,
) -> Interleave<I::IntoIter, J::IntoIter>
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
// Usage: alternate elements from two iterators
let merged: Vec<_> = interleave([1, 3], [2, 4]).collect();
// merged = [1, 2, 3, 4]
```

### Example: Cartesian product

`flat_map` composes nested iteration, producing every pair of elements from two slices. The outer closure captures `b` and the inner `map` pairs each `x` with every `y`. No intermediate vectors are created—pairs are generated lazily on demand.

```rust
fn cartesian_product<T: Clone>(
    a: &[T],
    b: &[T],
) -> impl Iterator<Item = (T, T)> + '_ {
    a.iter().flat_map(move |x| {
        b.iter().map(move |y| (x.clone(), y.clone()))
    })
}
// Usage: generate all pairs from two slices
let pairs: Vec<_> = cartesian_product(&[1, 2], &[3, 4]).collect();
// pairs = [(1, 3), (1, 4), (2, 3), (2, 4)]
```

### Example: Group by key

A single pass with `fold` and a `HashMap` groups elements by an arbitrary key function. The `or_default()` method ensures each key starts with an empty vector. This is a common pattern for bucketing data without multiple iterations.

```rust
use std::collections::HashMap;

fn group_by<K, V, F>(items: Vec<V>, key_fn: F) -> HashMap<K, Vec<V>>
where
    K: Eq + std::hash::Hash,
    F: Fn(&V) -> K,
{
    items.into_iter().fold(HashMap::new(), |mut map, item| {
        map.entry(key_fn(&item)).or_default().push(item);
        map
    })
}
// Usage: group numbers by even/odd
let grouped = group_by(vec![1, 2, 3, 4], |x| x % 2);
// grouped = {0: [2, 4], 1: [1, 3]}
```

### Example: Scan for cumulative operations

`.scan()` threads mutable state through an iterator, emitting transformed values along the way. Unlike `fold`, which produces a single final result, `scan` yields intermediate results. This is perfect for running totals, prefix sums, or any stateful transformation.

```rust
fn cumulative_sum(numbers: &[i32]) -> Vec<i32> {
    numbers
        .iter()
        .scan(0, |state, &x| {
            *state += x;
            Some(*state)
        })
        .collect()
}
// Usage: compute running totals (prefix sums)
let sums = cumulative_sum(&[1, 2, 3, 4]); // [1, 3, 6, 10]
```

### Example: Take while and skip while for prefix/suffix operations

`take_while` yields elements until the predicate returns false, then stops. `skip_while` discards elements until the predicate returns false, then yields the rest. Combining them splits data at a boundary without copying—useful for parsing headers from bodies.

```rust
fn extract_header_body(
    lines: &[String],
) -> (Vec<&String>, Vec<&String>) {
    let header: Vec<_> = lines.iter()
        .take_while(|line| !line.is_empty())
        .collect();
    let body: Vec<_> = lines
        .iter()
        .skip_while(|line| !line.is_empty())
        .skip(1)
        .collect();
    (header, body)
}
// Usage: split text at empty line into header and body
let lines = vec!["Header".into(), "".into(), "Body".into()];
let (h, b) = extract_header_body(&lines);
```

### Example: Peekable for lookahead

Wrapping an iterator in `.peekable()` lets you inspect the next element without consuming it. This is essential for parsers and tokenizers that need lookahead to decide how to proceed. The `peek()` method returns `Option<&Item>` while `next()` advances normally.

```rust
#[derive(Debug)]
enum Token {
    Number(i32),
    Op(char),
}

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
// Usage: tokenize mathematical expression
let tokens = parse_tokens("12 + 34 * 5");
// tokens = [Number(12), Op('+'), Number(34), Op('*'), Number(5)]
```

**Adapter composition principles:**
1. **Build pipelines incrementally**: Each step should be independently testable
2. **Use scan for stateful transformations**: More expressive than manual state tracking
3. **Peekable for lookahead**: Essential for parsers and state machines
4. **Prefer iterator methods over manual loops**: More composable and optimizable
5. **fold for custom aggregations**: More flexible than sum/collect

## Pattern 4: Streaming Algorithms

**Problem**: Loading entire files or datasets into memory causes OOM errors with large data (multi-GB log files, database dumps). Computing statistics requires multiple passes over data, multiplying I/O cost.

**Solution**: Process data one element at a time using iterators, maintaining only essential state (aggregates, sliding windows, top-K heaps). Use `BufReader::lines()` for line-by-line file processing.

**Why It Matters**: Streaming algorithms enable processing datasets larger than RAM—a 100GB log file processes with constant 1MB memory. Single-pass algorithms are dramatically faster: computing average + variance in one pass vs two passes halves I/O time.

**Use Cases**: Log file analysis (grep-like filtering, statistics), database ETL (processing query results), real-time analytics (streaming averages, alerting), sensor data processing, network packet analysis, infinite sequences (event streams, live data feeds), CSV/JSON parsing of large files.


### Example: Line-by-line file processing

`BufReader::lines` streams through a file lazily, letting you filter and count matches without loading the whole file. Each line is read on demand, so memory usage stays constant regardless of file size. The `filter_map(Result::ok)` idiom silently skips lines that fail to decode, keeping the pipeline clean.

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

fn count_lines_matching(
    path: &str,
    pattern: &str,
) -> std::io::Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(reader
        .lines()
        .filter_map(Result::ok)
        .filter(|line| line.contains(pattern))
        .count())
}
// Usage: count lines containing "ERROR" in a log file
let errors = count_lines_matching("/var/log/app.log", "ERROR")?;
```

### Example: Streaming average calculation

A lightweight accumulator keeps just the sum and count so you can compute an average in one pass over any iterator. This approach requires O(1) memory regardless of input size, making it suitable for unbounded streams. The `Option` return handles the edge case of an empty sequence cleanly.

```rust
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

fn compute_streaming_average(
    numbers: impl Iterator<Item = f64>,
) -> Option<f64> {
    let mut avg = StreamingAverage::new();
    for num in numbers {
        avg.add(num);
    }
    avg.average()
}
// Usage: compute average in a single pass
let avg = compute_streaming_average([1.0, 2.0, 3.0].into_iter());
```

### Example: Top-K elements without sorting

A `BinaryHeap` tracks just the best `k` candidates, avoiding a full sort regardless of input size. Using `Reverse` wraps the heap as a min-heap so the smallest of the top-k is always on top for efficient comparison. This O(n log k) algorithm is dramatically faster than O(n log n) sorting when k is small relative to n.

```rust
use std::cmp::Reverse;
use std::collections::BinaryHeap;

fn top_k<T: Ord>(
    iter: impl Iterator<Item = T>,
    k: usize,
) -> Vec<T> {
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
// Usage: find top 3 elements without sorting
let top3 = top_k([5, 1, 9, 3, 7].into_iter(), 3); // [5, 7, 9]
```

### Example: Sliding window statistics

A reusable `SlidingWindow` struct with a `VecDeque` tracks the latest `window_size` values and emits sums as it slides forward. When the window is full, `push` removes the oldest element and returns it so the caller can update running aggregates incrementally. This amortized O(1) per-element approach is far more efficient than recomputing statistics from scratch each time.

```rust
use std::collections::VecDeque;

struct SlidingWindow<T> {
    window: VecDeque<T>,
    capacity: usize,
}

impl<T> SlidingWindow<T> {
    fn new(capacity: usize) -> Self {
        SlidingWindow {
            window: VecDeque::with_capacity(capacity),
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

fn sliding_window_sum(
    numbers: impl Iterator<Item = i32>,
    window_size: usize,
) -> impl Iterator<Item = i32> {
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
// Usage: compute rolling sums with window size 3
let sums: Vec<_> =
    sliding_window_sum([1, 2, 3, 4, 5].into_iter(), 3).collect();
// sums = [6, 9, 12]  (1+2+3, 2+3+4, 3+4+5)
```

### Example: Streaming deduplication

A `HashSet` remembers which values have been seen so duplicates are filtered out lazily as the iterator advances. The `insert` method returns `true` only if the item was new, making it a perfect predicate for `filter`. Memory grows only with the number of unique elements, not total elements processed.

```rust
use std::collections::HashSet;

fn deduplicate_stream<T>(
    iter: impl Iterator<Item = T>,
) -> impl Iterator<Item = T>
where
    T: Eq + std::hash::Hash + Clone,
{
    let mut seen = HashSet::new();
    iter.filter(move |item| seen.insert(item.clone()))
}
// Usage: remove duplicates while streaming
let unique: Vec<_> = deduplicate_stream(
    [1, 2, 1, 3, 2].into_iter()).collect();
```

### Example: Rate limiting iterator

`RateLimited` wraps any iterator and sleeps as needed so successive items are yielded no faster than the configured interval. By tracking `last_yield` with an `Instant`, the adapter calculates exactly how long to wait before returning the next item. This is useful for throttling API calls, pacing network requests, or implementing backpressure.

```rust
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
// Usage: throttle API calls to one per second
let api_calls = vec!["request1", "request2", "request3"];
let calls = api_calls.into_iter();
for call in RateLimited::new(calls, Duration::from_secs(1)) {
    println!("Making: {}", call);
}
```

### Example: Buffered batch processing

Accumulate elements in a buffer and flush them to a callback once `batch_size` is reached, reducing per-item overhead. The `std::mem::replace` trick swaps in an empty buffer while yielding the full one, avoiding unnecessary allocations. A final flush handles any remaining items that didn't fill a complete batch.

```rust
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
            process_batch(std::mem::replace(
                &mut batch,
                Vec::with_capacity(batch_size),
            ));
        }
    }

    if !batch.is_empty() {
        process_batch(batch);
    }
}
// Usage: process database records in batches of 100
let items = (0..250).collect::<Vec<_>>().into_iter();
process_in_batches(items, 100, |batch| {
    println!("Processing batch of {} items", batch.len());
});
// Output: "Batch of 100 items" (x2), then "Batch of 50 items"
```

### Example: Streaming merge of sorted iterators

Two ordered inputs are merged into a new iterator by manually peeking at the next value from each source. The `from_fn` closure compares buffered values from `a_next` and `b_next`, always yielding the smaller one. This produces a single sorted stream in O(n + m) time without requiring additional memory for sorting.

```rust
fn merge_sorted<T: Ord>(
    mut a: impl Iterator<Item = T>,
    mut b: impl Iterator<Item = T>,
) -> impl Iterator<Item = T> {
    let mut a_next = a.next();
    let mut b_next = b.next();

    std::iter::from_fn(move || match (&a_next, &b_next) {
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
    })
}
// Usage: merge two sorted streams into one
let merged: Vec<_> = merge_sorted(
    [1, 3, 5].into_iter(),
    [2, 4, 6].into_iter()
).collect();
// merged = [1, 2, 3, 4, 5, 6]
```

### Example: CSV parsing with streaming

Each CSV line is read, split, and trimmed on the fly so no intermediate representation is needed. The `filter_map(Result::ok)` handles I/O errors gracefully by skipping problematic lines. This lazy approach keeps memory usage proportional to line length, not file size.

```rust
use std::io::BufRead;

fn parse_csv_stream(
    reader: impl BufRead,
) -> impl Iterator<Item = Vec<String>> {
    reader.lines().filter_map(Result::ok).map(|line| {
        line.split(',')
            .map(|s| s.trim().to_string())
            .collect()
    })
}
// Usage: parse CSV data lazily from a reader
let reader = std::io::Cursor::new("a,b,c\n1,2,3");
for row in parse_csv_stream(reader) { println!("{:?}", row); }
```

### Example: Lazy transformation chain

This pipeline reads a file lazily, normalizes each line, and folds it into a frequency map without ever materializing the whole dataset. The `flat_map` step splits lines into words, allowing the pipeline to process tokens one at a time. All transformations chain together so only one line is in memory at any moment.

```rust
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn process_large_file(
    path: &str,
) -> std::io::Result<Vec<(String, usize)>> {
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
        .fold(HashMap::new(), |mut map, word| {
            *map.entry(word).or_insert(0) += 1;
            map
        })
        .into_iter()
        .collect();

    Ok(result)
}
```

## Pattern 5: Parallel Iteration with Rayon

**Problem**: Processing a large collection sequentially can be slow, leaving multiple CPU cores idle. Manually writing multi-threaded code to parallelize such tasks is complex, error-prone, and requires careful synchronization.

**Solution**: Use the **Rayon** library. By simply changing `.iter()` to `.par_iter()`, Rayon can automatically parallelize your iterator chain across all available CPU cores.

**Why It Matters**: Rayon offers a massive performance boost for data-parallel tasks with minimal code changes. For a CPU-bound task, you can often achieve a near-linear speedup with the number of cores.

**Use Cases**:
-   Data-intensive computations like image processing or scientific simulations.
-   Batch processing of large numbers of files or database records.
-   Sorting and searching very large datasets.
-   Any "embarrassingly parallel" task that can be broken down into independent chunks.

### Example: Basic Parallel Iteration

By changing `.iter()` to `.par_iter()`, this function becomes a parallel operation. Rayon handles all the complexity of threading, work distribution, and result aggregation automatically. The performance scales nearly linearly with CPU cores for data-parallel operations like this sum-of-squares.

```rust
use rayon::prelude::*;

// This function will run in parallel across multiple cores.
fn parallel_sum_of_squares(numbers: &[i64]) -> i64 {
    numbers
        .par_iter() // The only change needed for parallelization!
        .map(|&x| x * x)
        .sum()
}
// Usage: compute sum of squares in parallel
let result = parallel_sum_of_squares(&[1, 2, 3, 4]);
```

### Example: Parallel sort

`par_sort_unstable` drops into Rayon's divide-and-conquer sorter so large vectors are sorted across all cores. The "unstable" variant doesn't preserve order of equal elements but is significantly faster. For millions of elements, parallel sorting can be 3-4x faster than sequential sorting.

```rust
use rayon::prelude::*;

fn parallel_sort(mut data: Vec<i32>) -> Vec<i32> {
    data.par_sort_unstable();
    data
}
// Usage: sort a vector using multiple cores
let sorted = parallel_sort(vec![3, 1, 4, 1, 5]); // [1, 1, 3, 4, 5]
```

### Example: Parallel chunked processing

`par_chunks` divides the slice evenly and lets Rayon process each chunk independently before collecting the results. Each chunk is processed by a separate thread, improving cache locality compared to interleaved access. This pattern is ideal when the per-element work is small and batching reduces overhead.

```rust
use rayon::prelude::*;

fn parallel_chunk_processing(
    data: &[u8],
    chunk_size: usize,
) -> Vec<u32> {
    data.par_chunks(chunk_size)
        .map(|chunk| chunk.iter().map(|&b| b as u32).sum())
        .collect()
}
// Usage: process data in chunks across multiple threads
let sums = parallel_chunk_processing(&[1, 2, 3, 4], 2); // [3, 7]
```

### Example: Parallel file processing

Each path is read and counted concurrently so filesystem-bound workloads keep all cores busy. The `par_iter` on paths distributes file reads across threads, overlapping I/O wait times with computation. Results are collected in order, preserving the mapping between input paths and output counts.

```rust
use rayon::prelude::*;

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
// Usage: count lines in multiple files concurrently
let counts = parallel_process_files(
    &["file1.txt".into(), "file2.txt".into()]);
```

### Example: Parallel find (early exit)

`position_any` searches in parallel and returns as soon as any worker hits the desired value. Unlike sequential search which must check elements in order, parallel search can find matches anywhere in the slice first. Note that if multiple matches exist, this returns an arbitrary match, not necessarily the first one.

```rust
use rayon::prelude::*;

fn parallel_find_first(
    numbers: &[i32],
    target: i32,
) -> Option<usize> {
    numbers.par_iter().position_any(|&x| x == target)
}
// Usage: find element position using parallel search
let idx = parallel_find_first(&[3, 1, 4, 1, 5], 4); // Some(2)
```

### Example: Parallel fold with combiner

Map each line to a count in parallel, then use Rayon's `sum` combiner to aggregate in a thread-safe way. Each thread processes a portion of lines independently, computing local counts without synchronization. The final `sum()` efficiently combines partial results using Rayon's parallel reduction.

```rust
use rayon::prelude::*;

fn parallel_word_count(lines: &[String]) -> usize {
    lines
        .par_iter()
        .map(|line| line.split_whitespace().count())
        .sum()
}
// Usage: count words across lines in parallel
let lines = &["hello world".into(), "foo".into()];
let count = parallel_word_count(lines); // 3
```

### Example: Parallel partition

`partition` splits even and odd numbers in one pass using `into_par_iter` for ownership and parallelism. Elements are moved into the appropriate output vector based on the predicate, with no copying needed. Both output vectors are built concurrently, making this faster than sequential partition for large datasets.

```rust
use rayon::prelude::*;

fn parallel_partition(numbers: Vec<i32>) -> (Vec<i32>, Vec<i32>) {
    numbers.into_par_iter().partition(|&x| x % 2 == 0)
}
// Usage: split numbers into even and odd in parallel
let (evens, odds) = parallel_partition(vec![1, 2, 3, 4]);
```

### Example: Parallel nested iteration with `flat_map`

Precompute the columns of `b` and use `par_iter` to multiply each row independently for cache-friendly matrix multiplication. Transposing `b` into `b_cols` ensures that column access is contiguous in memory, dramatically improving cache performance. Each output row is computed by a separate thread with no shared mutable state.

```rust
use rayon::prelude::*;

fn parallel_matrix_multiply(
    a: &[Vec<f64>],
    b: &[Vec<f64>],
) -> Vec<Vec<f64>> {
    let b_cols: Vec<Vec<f64>> = (0..b[0].len())
        .map(|col| b.iter().map(|row| row[col]).collect())
        .collect();

    a.par_iter()
        .map(|row| {
            b_cols
                .iter()
                .map(|col| {
                    row.iter().zip(col).map(|(a, b)| a * b).sum()
                })
                .collect()
        })
        .collect()
}
// Usage: multiply two matrices in parallel
let a = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
let b = vec![vec![5.0, 6.0], vec![7.0, 8.0]];
let c = parallel_matrix_multiply(&a, &b);
// c = [[19.0, 22.0], [43.0, 50.0]]
```

### Example: Parallel bridge for converting sequential to parallel

`par_bridge` consumes a standard iterator (here, an MPSC receiver) and fans it into Rayon workers. This is useful when you have an inherently sequential source (channels, network streams) but want parallel processing. The bridge buffers items and distributes them across worker threads dynamically.

```rust
use rayon::prelude::*;
use std::sync::mpsc::channel;

fn parallel_bridge_example() {
    let (sender, receiver) = channel();

    std::thread::spawn(move || {
        for i in 0..1000 {
            sender.send(i).unwrap();
        }
    });

    let sum: i32 = receiver.into_iter()
        .par_bridge()
        .map(|x| x * x)
        .sum();
    println!("Sum: {}", sum);
}
```

### Example: Controlling parallelism with scope

`rayon::scope` lets you spawn child tasks over disjoint slices while borrowing data safely. The `split_at_mut` ensures each spawn has exclusive access to its portion, satisfying Rust's borrowing rules. All spawned tasks complete before `scope` returns, guaranteeing the data is fully processed.

```rust
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
// Usage: process left and right halves concurrently
let mut data = vec![1, 2, 3, 4];
parallel_with_scope(&mut data);
// data = [2, 4, 9, 12]  (left *2, right *3)
```

### Example: Parallel map-reduce pattern

`fold` builds per-thread hash maps that `reduce` later merges, avoiding contention while counting words. Each thread maintains its own local `HashMap`, eliminating the need for locks or atomic operations during accumulation. The `reduce` phase merges these partial maps in a tree structure for efficient parallel combination.

```rust
use rayon::prelude::*;
use std::collections::HashMap;

fn parallel_map_reduce(data: &[String]) -> HashMap<String, usize> {
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
```

### Example: Parallel pipeline with multiple stages

Chaining `map`, `filter`, and another `map` on a `par_iter` builds a streaming parallel pipeline. Unlike sequential iteration, all stages can execute concurrently on different chunks of data. Rayon fuses these operations so each element passes through all stages on the same thread, minimizing synchronization.

```rust
use rayon::prelude::*;

fn parallel_pipeline(data: &[i32]) -> Vec<i32> {
    data.par_iter()
        .map(|&x| x * 2)      // Stage 1: multiply
        .filter(|&x| x > 100) // Stage 2: filter
        .map(|x| x / 3)       // Stage 3: divide
        .collect()
}
// Usage: chain map/filter/map operations in parallel
let result = parallel_pipeline(&[10, 60, 80]);
```

### Example: Custom parallel iterator

A bespoke range type can later implement Rayon traits to expose fine-grained control over splitting behavior. By implementing `ParallelIterator` and `IndexedParallelIterator`, you define how your type splits work across threads. This is useful for custom data structures where standard splitting heuristics don't apply.

```rust
struct ParallelRange {
    start: usize,
    end: usize,
}

impl ParallelRange {
    fn new(start: usize, end: usize) -> Self {
        ParallelRange { start, end }
    }
}
```

### Example: Joining parallel computations

`rayon::join` runs two closures concurrently and returns both results, ideal for independent aggregations. Both computations execute in parallel, with the calling thread participating in one of them. This is more efficient than spawning separate tasks when you have exactly two independent operations to perform.

```rust
use rayon::prelude::*;

fn parallel_join_example(data: &[i32]) -> (i32, i32) {
    let (sum, product) = rayon::join(
        || data.par_iter().sum(),
        || data.par_iter().product(),
    );
    (sum, product)
}
// Usage: compute sum and product concurrently
let (sum, product) = parallel_join_example(&[1, 2, 3, 4]);
// sum = 10, product = 24
```

**Parallel iteration principles:**
1. **Use par_iter for parallel iteration**: Drop-in replacement for .iter()
2. **Automatic work stealing**: Rayon balances load across threads
3. **fold + reduce for aggregation**: Parallel-friendly accumulation
4. **Chunk size matters**: Use par_chunks for better cache locality
5. **Measure before parallelizing**: Overhead can exceed benefits for small datasets

### Key takeaways:
1. Implement custom iterators for domain-specific iteration patterns
2. Chain adapters instead of collecting intermediate results
3. Use fold/scan/try_fold for stateful transformations
4. Stream data with BufReader and lazy iterators for large files
5. Apply rayon's par_iter for automatic parallelization
6. Prefer iterator methods over manual loops for composability
