# Chapter 5: Iterator Patterns & Combinators

## Overview

Iterators are one of Rust's most powerful abstractions, providing a unified interface for processing sequences of data. Unlike loops in many languages, Rust iterators are zero-cost abstractions: they compile down to the same machine code as hand-written loops, yet offer composability, expressiveness, and safety.

This chapter explores advanced iterator patterns that experienced programmers can leverage to write efficient, elegant code. The key insight is that iterators aren't just for collections—they're a design pattern for lazy, composable computation that can model streaming algorithms, state machines, and complex data transformations.

The patterns we'll explore include:
- Custom iterators and implementing IntoIterator
- Zero-allocation iteration strategies
- Iterator adapter composition for complex transformations
- Streaming algorithms for large datasets
- Parallel iteration with rayon

## Iterator Foundation

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

Implementing custom iterators allows you to provide lazy, composable iteration over your data structures without allocating intermediate collections.

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

Iterators enable processing data without intermediate allocations by leveraging lazy evaluation and iterator adapters.

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

Complex data transformations can be expressed as compositions of simple iterator adapters, creating readable and efficient pipelines.

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

Streaming algorithms process data incrementally without loading entire datasets into memory, enabling efficient handling of large or infinite data sources.

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

Rayon provides data parallelism through parallel iterators that automatically distribute work across CPU cores while maintaining iterator semantics.

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
            //================================
            // Expensive computation per chunk
            //================================
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

## Summary

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
