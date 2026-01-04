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

This `Counter` struct demonstrates the simplest form of a custom iterator. It iterates from 0 up to a maximum value.

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

// You can now use `Counter` with any iterator methods.
let sum: u32 = Counter { current: 0, max: 5 }.sum();
assert_eq!(sum, 10); // 0 + 1 + 2 + 3 + 4
```

### Example: Implementing `IntoIterator`

To make a custom collection work with `for` loops, you need to implement `IntoIterator`. Here, we implement it for a `RingBuffer` for owned, borrowed, and mutably borrowed iteration.

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
```

### Example: An Infinite Iterator

Iterators can represent infinite sequences because they are lazy. This `Fibonacci` iterator will produce numbers forever until the sequence overflows or is stopped by an adapter like `.take()`.

```rust
struct Fibonacci {
    current: u64,
    next: u64,
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.current;
        // Use `checked_add` to handle potential overflow gracefully.
        let new_next = self.current.checked_add(self.next)?;
        self.current = self.next;
        self.next = new_next;
        Some(result)
    }
}

// We can take the first 10 Fibonacci numbers.
let fibs: Vec<_> = Fibonacci { current: 0, next: 1 }.take(10).collect();
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

This function processes a slice of numbers by filtering positive numbers, squaring them, filtering again, and finally summing the result. No intermediate `Vec` is created.

```rust
fn process_numbers(input: &[i32]) -> i32 {
    input
        .iter()
        .filter(|&&x| x > 0)
        .map(|&x| x * x)
        .filter(|&x| x < 1000)
        .sum() // The iterator is consumed only at the end.
}
```

### Example: Using `windows` for Sliding Window

The `.windows()` method creates an iterator that yields overlapping slices of the original data. This is a zero-allocation way to implement sliding window algorithms.

```rust
fn moving_average(data: &[f64], window_size: usize) -> Vec<f64> {
    data.windows(window_size)
        .map(|window| window.iter().sum::<f64>() / window_size as f64)
        .collect()
}
```

### Example: `fold` for Custom Reductions

Instead of creating a new collection just to count items, `fold` can be used to perform custom aggregations in a single pass with no allocations.

```rust
fn count_long_strings(strings: &[&str]) -> usize {
    strings
        .iter()
        .fold(0, |count, &s| if s.len() > 10 { count + 1 } else { count })
}
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

This iterator chain filters log lines twice and folds the remainder into a per-level counter without building temporary collections.

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
        .filter(|entry| entry.level == "ERROR" || entry.level == "WARN")
        .filter(|entry| entry.timestamp > 1_000_000)
        .map(|entry| &entry.level)
        .fold(HashMap::new(), |mut map, level| {
            *map.entry(level.clone()).or_insert(0) += 1;
            map
        })
}
```

### Example: Chunking with stateful iteration

`from_fn` keeps the stateful `Vec` and drains `chunk_size` elements at a time, yielding batches lazily.

```rust
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
```

### Example: Interleaving iterators

This custom iterator alternates between two inputs while remaining lazy and falling back to whichever still has data.

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
```

### Example: Cartesian product

`flat_map` composes the nested iteration so every pair of elements from the two slices is produced without intermediate vectors.

```rust
fn cartesian_product<T: Clone>(a: &[T], b: &[T]) -> impl Iterator<Item = (T, T)> + '_ {
    a.iter()
        .flat_map(move |x| b.iter().map(move |y| (x.clone(), y.clone())))
}
```

### Example: Group by key

One pass and a `HashMap` are enough to accumulate elements into key buckets defined by any closure.

```rust
use std::collections::HashMap;

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
```

### Example: Scan for cumulative operations

`.scan` threads mutable state to emit a running total as each value is consumed.

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
```

### Example: Take while and skip while for prefix/suffix operations

Two iterator adapters split a slice of lines at the first blank row, yielding header and body views without copying.

```rust
fn extract_header_body(lines: &[String]) -> (Vec<&String>, Vec<&String>) {
    let header: Vec<_> = lines.iter().take_while(|line| !line.is_empty()).collect();
    let body: Vec<_> = lines
        .iter()
        .skip_while(|line| !line.is_empty())
        .skip(1)
        .collect();
    (header, body)
}
```

### Example: Peekable for lookahead

Wrapping `chars()` in `peekable()` enables a tokenizer that can inspect the next character before deciding how to advance.

```rust
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

**Problem**: Loading entire files or datasets into memory causes OOM errors with large data (multi-GB log files, database dumps). Computing statistics requires multiple passes over data, multiplying I/O cost.

**Solution**: Process data one element at a time using iterators, maintaining only essential state (aggregates, sliding windows, top-K heaps). Use `BufReader::lines()` for line-by-line file processing.

**Why It Matters**: Streaming algorithms enable processing datasets larger than RAM—a 100GB log file processes with constant 1MB memory. Single-pass algorithms are dramatically faster: computing average + variance in one pass vs two passes halves I/O time.

**Use Cases**: Log file analysis (grep-like filtering, statistics), database ETL (processing query results), real-time analytics (streaming averages, alerting), sensor data processing, network packet analysis, infinite sequences (event streams, live data feeds), CSV/JSON parsing of large files.


### Example: Line-by-line file processing

`BufReader::lines` streams through a file lazily, letting you filter and count matches without loading the whole file.

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

fn count_lines_matching(path: &str, pattern: &str) -> std::io::Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(reader
        .lines()
        .filter_map(Result::ok)
        .filter(|line| line.contains(pattern))
        .count())
}
```

### Example: Streaming average calculation

A lightweight accumulator keeps just the sum and count so you can compute an average in one pass over any iterator.

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

fn compute_streaming_average(numbers: impl Iterator<Item = f64>) -> Option<f64> {
    let mut avg = StreamingAverage::new();
    for num in numbers {
        avg.add(num);
    }
    avg.average()
}
```

### Example: Top-K elements without sorting

A `BinaryHeap` tracks just the best `k` candidates, avoiding a full sort regardless of input size.

```rust
use std::cmp::Reverse;
use std::collections::BinaryHeap;

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
```

### Example: Sliding window statistics

A reusable `SlidingWindow` struct with a `VecDeque` tracks the latest `window_size` values and emits sums as it slides forward.

```rust
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
```

### Example: Streaming deduplication

A `HashSet` remembers which values have been seen so duplicates are filtered out lazily as the iterator advances.

```rust
use std::collections::HashSet;

fn deduplicate_stream<T>(iter: impl Iterator<Item = T>) -> impl Iterator<Item = T>
where
    T: Eq + std::hash::Hash + Clone,
{
    let mut seen = HashSet::new();
    iter.filter(move |item| seen.insert(item.clone()))
}
```

### Example: Rate limiting iterator

`RateLimited` wraps any iterator and sleeps as needed so successive items are yielded no faster than the configured interval.

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
```

### Example: Buffered batch processing

Accumulate elements in a buffer and flush them to a callback once `batch_size` is reached, reducing per-item overhead.

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
```

### Example: Streaming merge of sorted iterators

Two ordered inputs are merged into a new iterator by manually peeking at the next value from each source.

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
```

### Example: CSV parsing with streaming

Each CSV line is read, split, and trimmed on the fly so no intermediate representation is needed.

```rust
use std::io::BufRead;

fn parse_csv_stream(reader: impl BufRead) -> impl Iterator<Item = Vec<String>> {
    reader.lines().filter_map(Result::ok).map(|line| {
        line.split(',')
            .map(|s| s.trim().to_string())
            .collect()
    })
}
```

### Example: Lazy transformation chain

This pipeline reads a file lazily, normalizes each line, and folds it into a frequency map without ever materializing the whole dataset.

```rust
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
        .fold(HashMap::new(), |mut map, word| {
            *map.entry(word).or_insert(0) += 1;
            map
        })
        .into_iter()
        .collect();

    Ok(result)
}
```

## Pattern 4: Parallel Iteration with Rayon

**Problem**: Processing a large collection sequentially can be slow, leaving multiple CPU cores idle. Manually writing multi-threaded code to parallelize such tasks is complex, error-prone, and requires careful synchronization.

**Solution**: Use the **Rayon** library. By simply changing `.iter()` to `.par_iter()`, Rayon can automatically parallelize your iterator chain across all available CPU cores.

**Why It Matters**: Rayon offers a massive performance boost for data-parallel tasks with minimal code changes. For a CPU-bound task, you can often achieve a near-linear speedup with the number of cores.

**Use Cases**:
-   Data-intensive computations like image processing or scientific simulations.
-   Batch processing of large numbers of files or database records.
-   Sorting and searching very large datasets.
-   Any "embarrassingly parallel" task that can be broken down into independent chunks.

### Example: Basic Parallel Iteration

By changing `.iter()` to `.par_iter()`, this function becomes a parallel operation. Rayon handles all the complexity of threading and data distribution.

```rust
use rayon::prelude::*;

// This function will run in parallel across multiple cores.
fn parallel_sum_of_squares(numbers: &[i64]) -> i64 {
    numbers
        .par_iter() // The only change needed for parallelization!
        .map(|&x| x * x)
        .sum()
}
```

### Example: Parallel sort

`par_sort_unstable` drops into Rayon’s divide-and-conquer sorter so large vectors are sorted across all cores.

```rust
use rayon::prelude::*;

fn parallel_sort(mut data: Vec<i32>) -> Vec<i32> {
    data.par_sort_unstable();
    data
}
```

### Example: Parallel chunked processing

`par_chunks` divides the slice evenly and lets Rayon process each chunk independently before collecting the results.

```rust
use rayon::prelude::*;

fn parallel_chunk_processing(data: &[u8], chunk_size: usize) -> Vec<u32> {
    data.par_chunks(chunk_size)
        .map(|chunk| chunk.iter().map(|&b| b as u32).sum())
        .collect()
}
```

### Example: Parallel file processing

Each path is read and counted concurrently so filesystem-bound workloads keep all cores busy.

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
```

### Example: Parallel find (early exit)

`position_any` searches in parallel and returns as soon as any worker hits the desired value.

```rust
use rayon::prelude::*;

fn parallel_find_first(numbers: &[i32], target: i32) -> Option<usize> {
    numbers.par_iter().position_any(|&x| x == target)
}
```

### Example: Parallel fold with combiner

Map each line to a count in parallel, then use Rayon’s `sum` combiner to aggregate in a thread-safe way.

```rust
use rayon::prelude::*;

fn parallel_word_count(lines: &[String]) -> usize {
    lines
        .par_iter()
        .map(|line| line.split_whitespace().count())
        .sum()
}
```

### Example: Parallel partition

`partition` splits even and odd numbers in one pass using `into_par_iter` for ownership and parallelism.

```rust
use rayon::prelude::*;

fn parallel_partition(numbers: Vec<i32>) -> (Vec<i32>, Vec<i32>) {
    numbers.into_par_iter().partition(|&x| x % 2 == 0)
}
```

### Example: Parallel nested iteration with `flat_map`

Precompute the columns of `b` and use `par_iter` to multiply each row independently for cache-friendly matrix multiplication.

```rust
use rayon::prelude::*;

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
```

### Example: Parallel bridge for converting sequential to parallel

`par_bridge` consumes a standard iterator (here, an MPSC receiver) and fans it into Rayon workers.

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

    let sum: i32 = receiver.into_iter().par_bridge().map(|x| x * x).sum();
    println!("Sum: {}", sum);
}
```

### Example: Controlling parallelism with scope

`rayon::scope` lets you spawn child tasks over disjoint slices while borrowing data safely.

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
```

### Example: Parallel map-reduce pattern

`fold` builds per-thread hash maps that `reduce` later merges, avoiding contention while counting words.

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

Chaining `map`, `filter`, and another `map` on a `par_iter` builds a streaming parallel pipeline.

```rust
use rayon::prelude::*;

fn parallel_pipeline(data: &[i32]) -> Vec<i32> {
    data.par_iter()
        .map(|&x| x * 2)      // Stage 1: multiply
        .filter(|&x| x > 100) // Stage 2: filter
        .map(|x| x / 3)       // Stage 3: divide
        .collect()
}
```

### Example: Custom parallel iterator

A bespoke range type can later implement Rayon traits to expose fine-grained control over splitting behavior.

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

`rayon::join` runs two closures concurrently and returns both results, ideal for independent aggregations.

```rust
use rayon::prelude::*;

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

### Key takeaways:
1. Implement custom iterators for domain-specific iteration patterns
2. Chain adapters instead of collecting intermediate results
3. Use fold/scan/try_fold for stateful transformations
4. Stream data with BufReader and lazy iterators for large files
5. Apply rayon's par_iter for automatic parallelization
6. Prefer iterator methods over manual loops for composability
