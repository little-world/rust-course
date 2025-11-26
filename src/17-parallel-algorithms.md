# Parallel Algorithms
This chapter explores parallel algorithm patterns using Rust's ecosystem, focusing on data parallelism with Rayon, work partitioning strategies, parallel reduction patterns, pipeline parallelism, and SIMD vectorization. We'll cover practical, production-ready examples for maximizing CPU utilization.



## Pattern 1: Rayon Patterns

**Problem**: Sequential code wastes modern multi-core CPUs—a 4-core CPU runs sequential code at 25% utilization. Manual threading with std::thread is complex: need to partition data, spawn threads, collect results, handle errors, avoid data races. Locks introduce contention and deadlocks. Thread pools require careful management. Want parallelism without the pain.

**Solution**: Use Rayon's parallel iterators with `.par_iter()`. Rayon provides work-stealing scheduler that automatically balances load across threads. Replace `.iter()` with `.par_iter()` for instant parallelism. No manual thread management, no locks, no data race concerns (enforced by type system). Supports map, filter, fold, reduce—all the iterator methods you know, now parallel.

**Why It Matters**: Trivial code change yields massive speedups. Image processing 1M pixels: sequential 500ms, parallel 80ms (6x faster on 8-core). Data validation 100K records: sequential 2s, parallel 300ms (6.6x faster). Rayon's work stealing handles irregular workloads automatically—no manual load balancing needed. Type system prevents data races at compile time. Production-ready: powers Firefox's parallel CSS engine.

**Use Cases**: Image processing (grayscale, filters, resizing), data validation (emails, phone numbers, formats), log parsing and analysis, batch data transformations, scientific computing, map-reduce operations, any embarrassingly parallel workload.


### Example: Parallel Iterator Basics

Convert sequential operations to parallel execution with minimal code changes.

```rust
//=========================
// Note: Add to Cargo.toml:
//=========================
// rayon = "1.8"

use rayon::prelude::*;
use std::time::Instant;

//==========================
// Pattern 1: Basic par_iter
//==========================
fn parallel_map_example() {
    let numbers: Vec<i32> = (0..1_000_000).collect();

    // Sequential
    let start = Instant::now();
    let sequential: Vec<i32> = numbers.iter().map(|&x| x * x).collect();
    let seq_time = start.elapsed();

    // Parallel
    let start = Instant::now();
    let parallel: Vec<i32> = numbers.par_iter().map(|&x| x * x).collect();
    let par_time = start.elapsed();

    println!("Sequential: {:?}", seq_time);
    println!("Parallel: {:?}", par_time);
    println!("Speedup: {:.2}x", seq_time.as_secs_f64() / par_time.as_secs_f64());

    assert_eq!(sequential, parallel);
}

//=====================================================
// Pattern 2: par_iter vs par_iter_mut vs into_par_iter
//=====================================================
fn iterator_variants() {
    let mut data = vec![1, 2, 3, 4, 5];

    // Immutable parallel iteration
    let sum: i32 = data.par_iter().sum();
    println!("Sum: {}", sum);

    // Mutable parallel iteration
    data.par_iter_mut().for_each(|x| *x *= 2);
    println!("Doubled: {:?}", data);

    // Consuming parallel iteration
    let owned_data = vec![1, 2, 3, 4, 5];
    let squares: Vec<i32> = owned_data.into_par_iter().map(|x| x * x).collect();
    println!("Squares: {:?}", squares);
}

//===================================
// Pattern 3: Parallel filter and map
//===================================
fn parallel_filter_map() {
    let numbers: Vec<i32> = (0..10_000).collect();

    let result: Vec<i32> = numbers
        .par_iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * x)
        .collect();

    println!("Filtered and squared {} numbers", result.len());
}

//=============================
// Pattern 4: Parallel flat_map
//=============================
fn parallel_flat_map() {
    let ranges: Vec<std::ops::Range<i32>> = vec![0..10, 10..20, 20..30];

    let flattened: Vec<i32> = ranges
        .into_par_iter()
        .flat_map(|range| range.into_par_iter())
        .collect();

    println!("Flattened {} items", flattened.len());
}

//=============================
// Real-world: Image processing
//=============================
struct Image {
    pixels: Vec<u8>,
    width: usize,
    height: usize,
}

impl Image {
    fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: vec![128; width * height * 3], // RGB
            width,
            height,
        }
    }

    fn apply_filter_parallel(&mut self, filter: fn(u8) -> u8) {
        self.pixels.par_iter_mut().for_each(|pixel| {
            *pixel = filter(*pixel);
        });
    }

    fn grayscale_parallel(&mut self) {
        self.pixels
            .par_chunks_mut(3)
            .for_each(|rgb| {
                let gray = ((rgb[0] as u32 + rgb[1] as u32 + rgb[2] as u32) / 3) as u8;
                rgb[0] = gray;
                rgb[1] = gray;
                rgb[2] = gray;
            });
    }

    fn brightness_parallel(&mut self, delta: i16) {
        self.pixels.par_iter_mut().for_each(|pixel| {
            *pixel = (*pixel as i16 + delta).clamp(0, 255) as u8;
        });
    }
}

//============================
// Real-world: Data validation
//============================
fn validate_emails_parallel(emails: Vec<String>) -> Vec<(String, bool)> {
    emails
        .into_par_iter()
        .map(|email| {
            let is_valid = email.contains('@') && email.contains('.');
            (email, is_valid)
        })
        .collect()
}

//========================
// Real-world: Log parsing
//========================
#[derive(Debug)]
struct LogEntry {
    timestamp: u64,
    level: String,
    message: String,
}

fn parse_logs_parallel(lines: Vec<String>) -> Vec<LogEntry> {
    lines
        .into_par_iter()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 3 {
                Some(LogEntry {
                    timestamp: parts[0].parse().ok()?,
                    level: parts[1].to_string(),
                    message: parts[2].to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

fn main() {
    println!("=== Parallel Map ===\n");
    parallel_map_example();

    println!("\n=== Iterator Variants ===\n");
    iterator_variants();

    println!("\n=== Filter Map ===\n");
    parallel_filter_map();

    println!("\n=== Image Processing ===\n");
    let mut img = Image::new(1920, 1080);

    let start = Instant::now();
    img.grayscale_parallel();
    println!("Grayscale: {:?}", start.elapsed());

    let start = Instant::now();
    img.brightness_parallel(10);
    println!("Brightness: {:?}", start.elapsed());

    println!("\n=== Email Validation ===\n");
    let emails = vec![
        "user@example.com".to_string(),
        "invalid-email".to_string(),
        "another@test.org".to_string(),
    ];

    let results = validate_emails_parallel(emails);
    for (email, valid) in results {
        println!("{}: {}", email, if valid { "✓" } else { "✗" });
    }
}
```

**Rayon Benefits**:
- **Minimal code changes**: Add `.par_` prefix
- **Work stealing**: Automatic load balancing
- **Type safe**: Same API as sequential iterators
- **No data races**: Enforced by type system

---

### Example: par_bridge for Dynamic Sources

Parallelize iterators that don't implement `ParallelIterator` directly, or process items as they arrive.

```rust
use rayon::prelude::*;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

//===========================================
// Pattern 1: Bridge from sequential iterator
//===========================================
fn par_bridge_basic() {
    let iter = (0..1000).filter(|x| x % 2 == 0);

    // Bridge to parallel
    let sum: i32 = iter.par_bridge().map(|x| x * x).sum();
    println!("Sum: {}", sum);
}

//========================================
// Pattern 2: Bridge from channel receiver
//========================================
fn par_bridge_from_channel() {
    let (tx, rx) = mpsc::channel();

    // Producer thread
    thread::spawn(move || {
        for i in 0..1000 {
            tx.send(i).unwrap();
            thread::sleep(Duration::from_micros(10));
        }
    });

    // Parallel processing of channel items
    let sum: i32 = rx
        .into_iter()
        .par_bridge()
        .map(|x| {
            // Expensive computation
            thread::sleep(Duration::from_micros(100));
            x * x
        })
        .sum();

    println!("Channel sum: {}", sum);
}

//==================================
// Real-world: File system traversal
//==================================
use std::fs;
use std::path::PathBuf;

fn find_large_files_parallel(root: &str, min_size: u64) -> Vec<(PathBuf, u64)> {
    fn visit_dirs(path: PathBuf) -> Box<dyn Iterator<Item = PathBuf>> {
        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(_) => return Box::new(std::iter::empty()),
        };

        let iter = entries.filter_map(|e| e.ok()).flat_map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(path)
            } else {
                Box::new(std::iter::once(path))
            }
        });

        Box::new(iter)
    }

    visit_dirs(PathBuf::from(root))
        .par_bridge()
        .filter_map(|path| {
            let metadata = fs::metadata(&path).ok()?;
            let size = metadata.len();
            if size >= min_size {
                Some((path, size))
            } else {
                None
            }
        })
        .collect()
}

//===================================
// Real-world: Database query results
//===================================
struct DatabaseIterator {
    current: usize,
    total: usize,
}

impl Iterator for DatabaseIterator {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.total {
            let value = self.current as i32;
            self.current += 1;
            // Simulate database fetch delay
            thread::sleep(Duration::from_micros(10));
            Some(value)
        } else {
            None
        }
    }
}

fn process_database_results() {
    let db_iter = DatabaseIterator {
        current: 0,
        total: 1000,
    };

    // Process results in parallel as they arrive
    let sum: i32 = db_iter
        .par_bridge()
        .map(|x| x * 2)
        .sum();

    println!("Database result sum: {}", sum);
}

//======================================
// Real-world: Network stream processing
//======================================
fn process_network_stream() {
    let (tx, rx) = mpsc::channel();

    // Simulate network packets arriving
    thread::spawn(move || {
        for i in 0..100 {
            let packet = format!("packet_{}", i);
            tx.send(packet).unwrap();
            thread::sleep(Duration::from_millis(5));
        }
    });

    // Process packets in parallel
    let processed: Vec<String> = rx
        .into_iter()
        .par_bridge()
        .map(|packet| {
            // Expensive processing (e.g., parsing, validation)
            thread::sleep(Duration::from_millis(10));
            format!("processed_{}", packet)
        })
        .collect();

    println!("Processed {} packets", processed.len());
}

fn main() {
    println!("=== par_bridge Basic ===\n");
    par_bridge_basic();

    println!("\n=== par_bridge from Channel ===\n");
    par_bridge_from_channel();

    println!("\n=== Database Iterator ===\n");
    process_database_results();

    println!("\n=== Network Stream ===\n");
    process_network_stream();
}
```

**par_bridge Use Cases**:
- **Channel receivers**: Process items as they arrive
- **Custom iterators**: Database cursors, file system traversal
- **Lazy evaluation**: Only compute when needed
- **Adaptive parallelism**: Work stealing adapts to varying workload

---

## Pattern 2: Work Partitioning Strategies

**Problem**: Bad work partitioning kills parallel performance. Too-small chunks (100 items) cause overhead—thread spawn/join costs dominate. Too-large chunks (100K items) cause load imbalance—one thread still working while others idle. Static partitioning fails with irregular workloads. Cache misses destroy performance when data access isn't local.

**Solution**: Choose grain size based on work: 1K-10K items for simple operations, larger for cache-heavy work. Use Rayon's adaptive chunking (default) for uniform work, explicit chunking for cache optimization. Employ recursive parallelism (rayon::join) for divide-and-conquer algorithms. Use cache blocking for matrix operations. Let work stealing handle irregular workloads automatically.

**Why It Matters**: Grain size tuning: 2-3x performance difference. Matrix multiply with blocking: 5x faster due to cache hits. Quicksort with proper sequential cutoff: 3x faster than naive parallel. Dynamic load balancing handles 90/10 workload distribution with near-linear speedup, while static partitioning stalls. Real example: video encoding with variable frame complexity—work stealing achieves 95% CPU utilization vs 60% with static.

**Use Cases**: Matrix operations (multiply, transpose, factorization), sorting algorithms (quicksort, mergesort), divide-and-conquer (tree traversal, expression evaluation), irregular workloads (graph algorithms), cache-sensitive operations (blocked algorithms).


### Example: Chunking and Load Balancing

Partition work efficiently across threads to minimize overhead and maximize CPU utilization.

```rust
use rayon::prelude::*;
use std::time::Instant;

//=======================
// Pattern 1: Chunk sizes
//=======================
fn chunk_size_comparison() {
    let data: Vec<i32> = (0..1_000_000).collect();

    // Default chunking (Rayon decides)
    let start = Instant::now();
    let sum1: i32 = data.par_iter().sum();
    let default_time = start.elapsed();

    // Custom chunk size (too small - more overhead)
    let start = Instant::now();
    let sum2: i32 = data.par_chunks(100).map(|chunk| chunk.iter().sum::<i32>()).sum();
    let small_chunk_time = start.elapsed();

    // Custom chunk size (balanced)
    let start = Instant::now();
    let sum3: i32 = data.par_chunks(10_000).map(|chunk| chunk.iter().sum::<i32>()).sum();
    let balanced_chunk_time = start.elapsed();

    println!("Default: {:?}", default_time);
    println!("Small chunks (100): {:?}", small_chunk_time);
    println!("Balanced chunks (10k): {:?}", balanced_chunk_time);

    assert_eq!(sum1, sum2);
    assert_eq!(sum2, sum3);
}

//=====================================
// Pattern 2: Work splitting strategies
//=====================================
fn work_splitting_strategies() {
    let data: Vec<i32> = (0..100_000).collect();

    // Strategy 1: Equal splits (good for uniform work)
    let chunk_size = data.len() / rayon::current_num_threads();
    let result1: Vec<i32> = data
        .par_chunks(chunk_size)
        .flat_map(|chunk| chunk.iter().map(|&x| x * x))
        .collect();

    // Strategy 2: Adaptive (good for non-uniform work)
    let result2: Vec<i32> = data.par_iter().map(|&x| x * x).collect();

    assert_eq!(result1.len(), result2.len());
}

//================================================
// Real-world: Matrix multiplication with blocking
//================================================
struct Matrix {
    data: Vec<f64>,
    rows: usize,
    cols: usize,
}

impl Matrix {
    fn new(rows: usize, cols: usize) -> Self {
        Self {
            data: vec![0.0; rows * cols],
            rows,
            cols,
        }
    }

    fn get(&self, row: usize, col: usize) -> f64 {
        self.data[row * self.cols + col]
    }

    fn set(&mut self, row: usize, col: usize, value: f64) {
        self.data[row * self.cols + col] = value;
    }

    // Parallel matrix multiplication with blocking for cache efficiency
    fn multiply_blocked(&self, other: &Matrix, block_size: usize) -> Matrix {
        assert_eq!(self.cols, other.rows);

        let mut result = Matrix::new(self.rows, other.cols);

        // Partition work by output blocks
        let row_blocks: Vec<usize> = (0..self.rows).step_by(block_size).collect();
        let col_blocks: Vec<usize> = (0..other.cols).step_by(block_size).collect();

        row_blocks.par_iter().for_each(|&row_start| {
            for &col_start in &col_blocks {
                // Process block
                let row_end = (row_start + block_size).min(self.rows);
                let col_end = (col_start + block_size).min(other.cols);

                for i in row_start..row_end {
                    for j in col_start..col_end {
                        let mut sum = 0.0;
                        for k in 0..self.cols {
                            sum += self.get(i, k) * other.get(k, j);
                        }
                        unsafe {
                            let ptr = result.data.as_ptr() as *mut f64;
                            *ptr.add(i * result.cols + j) = sum;
                        }
                    }
                }
            }
        });

        result
    }
}

//========================================================
// Real-world: Parallel merge sort with optimal grain size
//========================================================
fn parallel_merge_sort<T: Ord + Send>(arr: &mut [T], grain_size: usize) {
    if arr.len() <= grain_size {
        arr.sort();
        return;
    }

    let mid = arr.len() / 2;
    let (left, right) = arr.split_at_mut(mid);

    rayon::join(
        || parallel_merge_sort(left, grain_size),
        || parallel_merge_sort(right, grain_size),
    );

    // Merge (in-place merge omitted for brevity)
    let mut temp = Vec::with_capacity(arr.len());
    let mut i = 0;
    let mut j = mid;

    while i < mid && j < arr.len() {
        if arr[i] <= arr[j] {
            temp.push(std::mem::replace(&mut arr[i], unsafe { std::ptr::read(&arr[0]) }));
            i += 1;
        } else {
            temp.push(std::mem::replace(&mut arr[j], unsafe { std::ptr::read(&arr[0]) }));
            j += 1;
        }
    }

    while i < mid {
        temp.push(std::mem::replace(&mut arr[i], unsafe { std::ptr::read(&arr[0]) }));
        i += 1;
    }

    while j < arr.len() {
        temp.push(std::mem::replace(&mut arr[j], unsafe { std::ptr::read(&arr[0]) }));
        j += 1;
    }

    for (i, item) in temp.into_iter().enumerate() {
        arr[i] = item;
    }
}

//==================================
// Pattern 3: Dynamic load balancing
//==================================
fn dynamic_load_balancing() {
    // Simulate irregular workload
    let work_items: Vec<usize> = (0..1000).map(|i| i % 100).collect();

    let start = Instant::now();

    // Rayon automatically balances work through work stealing
    let results: Vec<usize> = work_items
        .par_iter()
        .map(|&work| {
            // Simulate variable work
            let mut sum = 0;
            for _ in 0..work {
                sum += 1;
            }
            sum
        })
        .collect();

    println!("Dynamic balancing: {:?}", start.elapsed());
    println!("Total work: {}", results.iter().sum::<usize>());
}

//=============================
// Pattern 4: Grain size tuning
//=============================
fn grain_size_tuning() {
    let data: Vec<i32> = (0..1_000_000).collect();

    for grain_size in [100, 1_000, 10_000, 100_000] {
        let start = Instant::now();

        let sum: i32 = data
            .par_chunks(grain_size)
            .map(|chunk| chunk.iter().sum::<i32>())
            .sum();

        println!("Grain size {}: {:?}", grain_size, start.elapsed());
    }
}

fn main() {
    println!("=== Chunk Size Comparison ===\n");
    chunk_size_comparison();

    println!("\n=== Dynamic Load Balancing ===\n");
    dynamic_load_balancing();

    println!("\n=== Grain Size Tuning ===\n");
    grain_size_tuning();

    println!("\n=== Parallel Merge Sort ===\n");
    let mut data: Vec<i32> = (0..100_000).rev().collect();
    let start = Instant::now();
    parallel_merge_sort(&mut data, 1000);
    println!("Sort time: {:?}", start.elapsed());
    println!("Sorted: {}", data.windows(2).all(|w| w[0] <= w[1]));
}
```

**Work Partitioning Guidelines**:
- **Grain size**: Larger grains reduce overhead, but may cause load imbalance
- **Chunk size**: Balance overhead vs parallelism (typically 1000-10000 items)
- **Cache blocking**: Improve cache locality with block-based partitioning
- **Work stealing**: Rayon automatically balances irregular workloads

---

### Example: Recursive Parallelism

 Parallelize divide-and-conquer algorithms efficiently.

```rust
use rayon::prelude::*;

//==============================
// Pattern 1: Parallel quicksort
//==============================
fn parallel_quicksort<T: Ord + Send>(arr: &mut [T]) {
    if arr.len() <= 1 {
        return;
    }

    let pivot_idx = partition(arr);

    let (left, right) = arr.split_at_mut(pivot_idx);

    // Parallelize recursion
    rayon::join(
        || parallel_quicksort(left),
        || parallel_quicksort(&mut right[1..]),
    );
}

fn partition<T: Ord>(arr: &mut [T]) -> usize {
    let len = arr.len();
    let pivot_idx = len / 2;
    arr.swap(pivot_idx, len - 1);

    let mut i = 0;
    for j in 0..len - 1 {
        if arr[j] <= arr[len - 1] {
            arr.swap(i, j);
            i += 1;
        }
    }

    arr.swap(i, len - 1);
    i
}

//===================================
// Pattern 2: Parallel tree traversal
//===================================
#[derive(Debug)]
struct TreeNode<T> {
    value: T,
    left: Option<Box<TreeNode<T>>>,
    right: Option<Box<TreeNode<T>>>,
}

impl<T: Send> TreeNode<T> {
    fn parallel_map<F, U>(&self, f: &F) -> TreeNode<U>
    where
        F: Fn(&T) -> U + Sync,
        U: Send,
    {
        let value = f(&self.value);

        let (left, right) = rayon::join(
            || self.left.as_ref().map(|node| Box::new(node.parallel_map(f))),
            || self.right.as_ref().map(|node| Box::new(node.parallel_map(f))),
        );

        TreeNode { value, left, right }
    }

    fn parallel_sum(&self) -> T
    where
        T: std::ops::Add<Output = T> + Default + Copy + Send,
    {
        let mut sum = self.value;

        let (left_sum, right_sum) = rayon::join(
            || self.left.as_ref().map_or(T::default(), |node| node.parallel_sum()),
            || self.right.as_ref().map_or(T::default(), |node| node.parallel_sum()),
        );

        sum = sum + left_sum + right_sum;
        sum
    }
}

//=============================================================
// Pattern 3: Parallel Fibonacci (demonstrative, not efficient)
//=============================================================
fn parallel_fib(n: u32) -> u64 {
    if n <= 1 {
        return n as u64;
    }

    if n < 20 {
        // Sequential threshold to avoid overhead
        return fib_sequential(n);
    }

    let (a, b) = rayon::join(
        || parallel_fib(n - 1),
        || parallel_fib(n - 2),
    );

    a + b
}

fn fib_sequential(n: u32) -> u64 {
    if n <= 1 {
        return n as u64;
    }
    let mut a = 0;
    let mut b = 1;
    for _ in 2..=n {
        let c = a + b;
        a = b;
        b = c;
    }
    b
}

//================================================
// Real-world: Parallel directory size calculation
//================================================
use std::fs;
use std::path::Path;

fn parallel_dir_size<P: AsRef<Path>>(path: P) -> u64 {
    let path = path.as_ref();

    if path.is_file() {
        return fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    }

    if !path.is_dir() {
        return 0;
    }

    let entries: Vec<_> = fs::read_dir(path)
        .ok()
        .map(|entries| entries.filter_map(|e| e.ok()).collect())
        .unwrap_or_default();

    entries
        .par_iter()
        .map(|entry| parallel_dir_size(entry.path()))
        .sum()
}

//===========================================
// Real-world: Parallel expression evaluation
//===========================================
#[derive(Debug, Clone)]
enum Expr {
    Num(i32),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
}

impl Expr {
    fn parallel_eval(&self) -> i32 {
        match self {
            Expr::Num(n) => *n,
            Expr::Add(left, right) => {
                let (l, r) = rayon::join(
                    || left.parallel_eval(),
                    || right.parallel_eval(),
                );
                l + r
            }
            Expr::Mul(left, right) => {
                let (l, r) = rayon::join(
                    || left.parallel_eval(),
                    || right.parallel_eval(),
                );
                l * r
            }
            Expr::Sub(left, right) => {
                let (l, r) = rayon::join(
                    || left.parallel_eval(),
                    || right.parallel_eval(),
                );
                l - r
            }
        }
    }
}

fn main() {
    println!("=== Parallel Quicksort ===\n");
    let mut data: Vec<i32> = (0..100_000).rev().collect();
    let start = std::time::Instant::now();
    parallel_quicksort(&mut data);
    println!("Sort time: {:?}", start.elapsed());
    println!("Sorted: {}", data.windows(2).all(|w| w[0] <= w[1]));

    println!("\n=== Parallel Fibonacci ===\n");
    let start = std::time::Instant::now();
    let result = parallel_fib(35);
    println!("fib(35) = {} in {:?}", result, start.elapsed());

    println!("\n=== Expression Evaluation ===\n");
    let expr = Expr::Add(
        Box::new(Expr::Mul(
            Box::new(Expr::Num(5)),
            Box::new(Expr::Num(10)),
        )),
        Box::new(Expr::Sub(
            Box::new(Expr::Num(20)),
            Box::new(Expr::Num(8)),
        )),
    );

    let result = expr.parallel_eval();
    println!("Expression result: {}", result);
}
```

**Recursive Parallelism Tips**:
- **Sequential cutoff**: Switch to sequential below threshold
- **rayon::join**: Parallel fork-join primitive
- **Balance**: Ensure subtasks have similar work
- **Overhead**: Avoid creating too many small tasks

---

## Pattern 3: Parallel Reduce and Fold

**Problem**: Aggregating results from parallel operations is non-trivial. Simple sum/min/max work, but custom aggregations (histograms, statistics, merging maps) require careful design. Must use associative operations for correctness. Non-associative ops give wrong results. Need to combine per-thread accumulators efficiently. Performance suffers with poor reduce strategy.

**Solution**: Use `reduce` for simple aggregations (sum, min, max, product). Use `fold + reduce` pattern for custom accumulators: fold builds per-thread state, reduce combines them. Ensure operations are associative: (a op b) op c = a op (b op c). Commutative helps performance but isn't required. For histograms: fold builds per-thread HashMap, reduce merges them.

**Why It Matters**: Statistics in one parallel pass instead of multiple sequential passes. Histogram generation: parallel fold+reduce 10x faster than sequential. Word frequency counting: 8x speedup on 8 cores. Variance calculation: single parallel pass vs two sequential passes. Real example: analyzing 1GB log file—parallel histogram 500ms vs 5s sequential.

**Use Cases**: Statistics computation (mean, variance, stddev), histograms and frequency counting, word counting in text processing, aggregating results from parallel operations, merging sorted chunks, custom accumulators (sets, maps).

### Example: Parallel Reduction Patterns

Efficiently combine results from parallel operations.

```rust
use rayon::prelude::*;
use std::collections::HashMap;

//=========================================
// Pattern 1: Simple reduce (sum, min, max)
//=========================================
fn simple_reductions() {
    let numbers: Vec<i32> = (1..=1_000_000).collect();

    // Sum
    let sum: i32 = numbers.par_iter().sum();
    println!("Sum: {}", sum);

    // Min/Max
    let min = numbers.par_iter().min().unwrap();
    let max = numbers.par_iter().max().unwrap();
    println!("Min: {}, Max: {}", min, max);

    // Product (be careful of overflow!)
    let small_numbers: Vec<i32> = (1..=10).collect();
    let product: i32 = small_numbers.par_iter().product();
    println!("Product: {}", product);
}

//========================================
// Pattern 2: Reduce with custom operation
//========================================
fn custom_reduce() {
    let numbers: Vec<i32> = (1..=100).collect();

    // Custom reduction: concatenate all numbers
    let concatenated = numbers
        .par_iter()
        .map(|n| n.to_string())
        .reduce(|| String::new(), |a, b| format!("{},{}", a, b));

    println!("Concatenated (first 50 chars): {}", &concatenated[..50.min(concatenated.len())]);

    // Find element closest to target
    let target = 42;
    let closest = numbers
        .par_iter()
        .reduce(
            || &numbers[0],
            |a, b| {
                if (a - target).abs() < (b - target).abs() {
                    a
                } else {
                    b
                }
            },
        );

    println!("Closest to {}: {}", target, closest);
}

//==========================
// Pattern 3: fold vs reduce
//==========================
fn fold_vs_reduce() {
    let numbers: Vec<i32> = (1..=1000).collect();

    // fold: provide identity and combine function
    let sum_fold = numbers.par_iter().fold(
        || 0, // Identity function
        |acc, &x| acc + x, // Fold function
    ).sum::<i32>(); // Reduce the folded results

    // reduce: simpler but less flexible
    let sum_reduce = numbers.par_iter().sum::<i32>();

    assert_eq!(sum_fold, sum_reduce);
    println!("Sum: {}", sum_fold);
}

//=============================================
// Pattern 4: fold_with for custom accumulators
//=============================================
fn fold_with_accumulator() {
    let numbers: Vec<i32> = (1..=100).collect();

    // Collect statistics in one pass
    #[derive(Default)]
    struct Stats {
        count: usize,
        sum: i64,
        min: i32,
        max: i32,
    }

    let stats = numbers
        .par_iter()
        .fold(
            || Stats {
                count: 0,
                sum: 0,
                min: i32::MAX,
                max: i32::MIN,
            },
            |mut acc, &x| {
                acc.count += 1;
                acc.sum += x as i64;
                acc.min = acc.min.min(x);
                acc.max = acc.max.max(x);
                acc
            },
        )
        .reduce(
            || Stats::default(),
            |a, b| Stats {
                count: a.count + b.count,
                sum: a.sum + b.sum,
                min: a.min.min(b.min),
                max: a.max.max(b.max),
            },
        );

    println!("Count: {}", stats.count);
    println!("Average: {:.2}", stats.sum as f64 / stats.count as f64);
    println!("Min: {}, Max: {}", stats.min, stats.max);
}

//===============================
// Real-world: Parallel histogram
//===============================
fn parallel_histogram(data: Vec<i32>) -> HashMap<i32, usize> {
    data.par_iter()
        .fold(
            || HashMap::new(),
            |mut map, &value| {
                *map.entry(value).or_insert(0) += 1;
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

//=================================
// Real-world: Word frequency count
//=================================
fn word_frequency_parallel(text: String) -> HashMap<String, usize> {
    text.par_split_whitespace()
        .fold(
            || HashMap::new(),
            |mut map, word| {
                *map.entry(word.to_lowercase()).or_insert(0) += 1;
                map
            },
        )
        .reduce(
            || HashMap::new(),
            |mut a, b| {
                for (word, count) in b {
                    *a.entry(word).or_insert(0) += count;
                }
                a
            },
        )
}

//==========================================
// Real-world: Parallel variance calculation
//==========================================
fn parallel_variance(numbers: &[f64]) -> (f64, f64) {
    // Two-pass algorithm (more numerically stable)
    let mean = numbers.par_iter().sum::<f64>() / numbers.len() as f64;

    let variance = numbers
        .par_iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>()
        / numbers.len() as f64;

    (mean, variance)
}

//================================
// Real-world: Merge sorted chunks
//================================
fn parallel_merge_reduce(mut chunks: Vec<Vec<i32>>) -> Vec<i32> {
    while chunks.len() > 1 {
        chunks = chunks
            .par_chunks(2)
            .map(|pair| {
                if pair.len() == 2 {
                    merge(&pair[0], &pair[1])
                } else {
                    pair[0].clone()
                }
            })
            .collect();
    }

    chunks.into_iter().next().unwrap_or_default()
}

fn merge(a: &[i32], b: &[i32]) -> Vec<i32> {
    let mut result = Vec::with_capacity(a.len() + b.len());
    let mut i = 0;
    let mut j = 0;

    while i < a.len() && j < b.len() {
        if a[i] <= b[j] {
            result.push(a[i]);
            i += 1;
        } else {
            result.push(b[j]);
            j += 1;
        }
    }

    result.extend_from_slice(&a[i..]);
    result.extend_from_slice(&b[j..]);
    result
}

fn main() {
    println!("=== Simple Reductions ===\n");
    simple_reductions();

    println!("\n=== Custom Reduce ===\n");
    custom_reduce();

    println!("\n=== Fold with Accumulator ===\n");
    fold_with_accumulator();

    println!("\n=== Parallel Histogram ===\n");
    let data: Vec<i32> = (0..10000).map(|i| i % 100).collect();
    let histogram = parallel_histogram(data);
    println!("Histogram buckets: {}", histogram.len());
    println!("Bucket 50: {}", histogram.get(&50).unwrap_or(&0));

    println!("\n=== Word Frequency ===\n");
    let text = "the quick brown fox jumps over the lazy dog the fox".to_string();
    let freq = word_frequency_parallel(text);
    for (word, count) in freq.iter().take(5) {
        println!("{}: {}", word, count);
    }

    println!("\n=== Variance ===\n");
    let numbers: Vec<f64> = (1..=100).map(|x| x as f64).collect();
    let (mean, variance) = parallel_variance(&numbers);
    println!("Mean: {:.2}, Variance: {:.2}, StdDev: {:.2}", mean, variance, variance.sqrt());
}
```

**Reduction Patterns**:
- **reduce**: Simple aggregation (sum, min, max)
- **fold + reduce**: Custom accumulator, then combine
- **Associative operations**: Required for correctness
- **Commutative**: Not required but helps performance

---

## Pattern 4: Pipeline Parallelism

**Problem**: Multi-stage data processing often bottlenecks on slowest stage. Sequential pipeline wastes CPU—decode thread idle while enhance runs. Poor stage balance causes bubbles. Backpressure issues with unbounded buffers causing OOM. Different stages have different computational costs (decode: 10ms, enhance: 50ms, compress: 20ms)—need different parallelism levels.

**Solution**: Use channel-based pipelines with separate threads per stage. Rayon's par_iter at each stage for intra-stage parallelism. Bounded channels (mpsc::sync_channel) for backpressure. Balance parallelism per stage based on cost. For iterator-based pipelines, chain parallel operations. Combine data parallelism (Rayon) with task parallelism (stages).

**Why It Matters**: Image processing pipeline: sequential 300ms, staged parallel 100ms (3x faster). ETL pipeline processing 1M records: sequential 10min, parallel pipeline 2min (5x speedup). Log analysis: parse→enrich→filter runs stages concurrently, 4x throughput. Real example: video transcoding pipeline with decode→filter→encode stages, each utilizing multiple cores.

**Use Cases**: ETL (Extract-Transform-Load) data pipelines, image/video processing (decode→enhance→compress), log analysis (parse→enrich→filter→aggregate), data transformation chains, streaming data processing, multi-stage batch jobs.


### Example: Multi-Stage Pipelines

Process data through multiple transformation stages with different computational costs.:

```rust
use rayon::prelude::*;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

//=========================================
// Pattern 1: Simple pipeline with channels
//=========================================
fn simple_pipeline() {
    let (stage1_tx, stage1_rx) = mpsc::sync_channel(100);
    let (stage2_tx, stage2_rx) = mpsc::sync_channel(100);
    let (stage3_tx, stage3_rx) = mpsc::sync_channel(100);

    // Stage 1: Data generation
    let producer = thread::spawn(move || {
        for i in 0..1000 {
            stage1_tx.send(i).unwrap();
        }
    });

    // Stage 2: Transform (parallel)
    let stage2 = thread::spawn(move || {
        stage1_rx
            .into_iter()
            .par_bridge()
            .for_each_with(stage2_tx, |tx, item| {
                let transformed = item * 2;
                tx.send(transformed).unwrap();
            });
    });

    // Stage 3: Filter (parallel)
    let stage3 = thread::spawn(move || {
        stage2_rx
            .into_iter()
            .par_bridge()
            .filter(|&x| x % 4 == 0)
            .for_each_with(stage3_tx, |tx, item| {
                tx.send(item).unwrap();
            });
    });

    // Consumer
    let consumer = thread::spawn(move || {
        let sum: i32 = stage3_rx.into_iter().sum();
        sum
    });

    producer.join().unwrap();
    stage2.join().unwrap();
    stage3.join().unwrap();
    let result = consumer.join().unwrap();

    println!("Pipeline result: {}", result);
}

//======================================
// Real-world: Image processing pipeline
//======================================
struct ImagePipeline;

impl ImagePipeline {
    fn process_batch(images: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        images
            .into_par_iter()
            .map(|img| Self::stage1_decode(img))
            .map(|img| Self::stage2_enhance(img))
            .map(|img| Self::stage3_compress(img))
            .collect()
    }

    fn stage1_decode(data: Vec<u8>) -> Vec<u8> {
        // Simulate decoding
        thread::sleep(Duration::from_micros(100));
        data
    }

    fn stage2_enhance(mut data: Vec<u8>) -> Vec<u8> {
        // Simulate enhancement
        for pixel in &mut data {
            *pixel = pixel.saturating_add(10);
        }
        data
    }

    fn stage3_compress(data: Vec<u8>) -> Vec<u8> {
        // Simulate compression
        thread::sleep(Duration::from_micros(50));
        data
    }
}

//====================================
// Real-world: Log processing pipeline
//====================================
#[derive(Debug, Clone)]
struct RawLog(String);

#[derive(Debug, Clone)]
struct ParsedLog {
    timestamp: u64,
    level: String,
    message: String,
}

#[derive(Debug, Clone)]
struct EnrichedLog {
    log: ParsedLog,
    metadata: String,
}

struct LogPipeline;

impl LogPipeline {
    fn process(logs: Vec<RawLog>) -> Vec<EnrichedLog> {
        logs.into_par_iter()
            .filter_map(|raw| Self::parse(raw))
            .map(|parsed| Self::enrich(parsed))
            .filter(|enriched| enriched.log.level == "ERROR")
            .collect()
    }

    fn parse(raw: RawLog) -> Option<ParsedLog> {
        let parts: Vec<&str> = raw.0.split('|').collect();
        if parts.len() >= 3 {
            Some(ParsedLog {
                timestamp: parts[0].parse().ok()?,
                level: parts[1].to_string(),
                message: parts[2].to_string(),
            })
        } else {
            None
        }
    }

    fn enrich(log: ParsedLog) -> EnrichedLog {
        EnrichedLog {
            log: log.clone(),
            metadata: format!("enriched_{}", log.timestamp),
        }
    }
}

//======================================================
// Pattern 2: Parallel stages with different parallelism
//======================================================
fn multi_stage_parallel() {
    let data: Vec<i32> = (0..10000).collect();

    // Stage 1: Light processing (high parallelism)
    let stage1: Vec<i32> = data
        .par_iter()
        .map(|&x| x + 1)
        .collect();

    // Stage 2: Heavy processing (moderate parallelism)
    let stage2: Vec<i32> = stage1
        .par_chunks(100) // Larger chunks for heavy work
        .flat_map(|chunk| {
            chunk.iter().map(|&x| {
                // Simulate heavy computation
                let mut result = x;
                for _ in 0..100 {
                    result = (result * 2) % 1000;
                }
                result
            })
        })
        .collect();

    // Stage 3: Aggregation
    let sum: i32 = stage2.par_iter().sum();

    println!("Multi-stage result: {}", sum);
}

//====================================================
// Real-world: ETL pipeline (Extract, Transform, Load)
//====================================================
struct EtlPipeline;

impl EtlPipeline {
    fn run(input_files: Vec<String>) -> Vec<(String, usize)> {
        input_files
            .into_par_iter()
            // Extract: Read files in parallel
            .filter_map(|file| Self::extract(&file))
            // Transform: Process data in parallel
            .map(|data| Self::transform(data))
            // Load: Aggregate results
            .collect()
    }

    fn extract(file: &str) -> Option<Vec<String>> {
        // Simulate file reading
        Some(vec![format!("data_from_{}", file)])
    }

    fn transform(data: Vec<String>) -> (String, usize) {
        // Simulate transformation
        let processed = data
            .par_iter()
            .map(|s| s.to_uppercase())
            .collect::<Vec<_>>();

        ("transformed".to_string(), processed.len())
    }
}

fn main() {
    println!("=== Simple Pipeline ===\n");
    simple_pipeline();

    println!("\n=== Image Processing Pipeline ===\n");
    let images: Vec<Vec<u8>> = (0..100).map(|_| vec![128; 1000]).collect();
    let start = std::time::Instant::now();
    let processed = ImagePipeline::process_batch(images);
    println!("Processed {} images in {:?}", processed.len(), start.elapsed());

    println!("\n=== Log Processing Pipeline ===\n");
    let logs: Vec<RawLog> = (0..1000)
        .map(|i| RawLog(format!("{}|{}|message_{}", i, if i % 10 == 0 { "ERROR" } else { "INFO" }, i)))
        .collect();

    let errors = LogPipeline::process(logs);
    println!("Found {} errors", errors.len());

    println!("\n=== Multi-Stage Parallel ===\n");
    multi_stage_parallel();

    println!("\n=== ETL Pipeline ===\n");
    let files: Vec<String> = (0..100).map(|i| format!("file_{}.csv", i)).collect();
    let results = EtlPipeline::run(files);
    println!("Processed {} files", results.len());
}
```

**Pipeline Patterns**:
- **Channel-based**: Explicit stages with bounded buffers
- **Iterator-based**: Chain transformations with par_iter
- **Staged parallelism**: Different parallelism per stage
- **Backpressure**: Bounded channels prevent memory issues

---

## Pattern 5: SIMD Parallelism

**Problem**: CPU vector units (AVX2: 8 floats, AVX-512: 16 floats) sit idle with scalar code. Data-level parallelism untapped—process 1 element when hardware can do 8. Memory bandwidth wasted without vectorization. Auto-vectorization fails with complex code (branches, scattered access). Combining threading and SIMD requires careful data layout.

**Solution**: Write SIMD-friendly code: contiguous arrays, simple operations, no branches in hot loops. Use Struct-of-Arrays (SoA) instead of Array-of-Structs (AoS) for better vectorization. Combine Rayon threading with SIMD-friendly inner loops. Let compiler auto-vectorize when possible. Use explicit chunking aligned to SIMD width. Profile with `cargo rustc -- --emit asm` to verify vectorization.

**Why It Matters**: Matrix multiply: 10x speedup with SIMD+threading vs scalar sequential. Dot product: 4-8x faster with vectorization. Image convolution: SIMD on inner loops doubles performance. Real example: signal processing 1M samples—scalar 100ms, SIMD 15ms (6.6x), SIMD+8 threads 2ms (50x total). Memory bandwidth becomes bottleneck, not compute.

**Use Cases**: Matrix operations (multiply, transpose, dot product), image processing (convolution, filters), signal processing (FFT, filters), scientific computing (numerical methods), vector arithmetic, statistical computations.


### Example: Portable SIMD with std::simd

Vectorize operations across array elements for maximum throughput.

```rust
// Note: Requires nightly Rust
// Add to Cargo.toml:
// [dependencies]
// packed_simd = "0.3"
// For stable Rust, we'll use a portable SIMD approach

//=====================================
// Pattern 1: Manual SIMD-friendly code
//=====================================
fn simd_friendly_sum(data: &[f32]) -> f32 {
    // Process 4 elements at a time (compiler can auto-vectorize)
    let chunks = data.chunks_exact(4);
    let remainder = chunks.remainder();

    let mut sums = [0.0f32; 4];

    for chunk in chunks {
        sums[0] += chunk[0];
        sums[1] += chunk[1];
        sums[2] += chunk[2];
        sums[3] += chunk[3];
    }

    let chunk_sum: f32 = sums.iter().sum();
    let remainder_sum: f32 = remainder.iter().sum();

    chunk_sum + remainder_sum
}

//============================================
// Pattern 2: Array operations (SIMD-friendly)
//============================================
fn vector_add(a: &[f32], b: &[f32], result: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), result.len());

    // Compiler can auto-vectorize this
    for i in 0..a.len() {
        result[i] = a[i] + b[i];
    }
}

fn vector_add_parallel(a: &[f32], b: &[f32]) -> Vec<f32> {
    use rayon::prelude::*;

    // Combine SIMD and thread parallelism
    a.par_iter()
        .zip(b.par_iter())
        .map(|(&x, &y)| x + y)
        .collect()
}

//=======================================
// Pattern 3: Dot product (SIMD-friendly)
//=======================================
fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());

    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| x * y)
        .sum()
}

fn dot_product_parallel(a: &[f32], b: &[f32]) -> f32 {
    use rayon::prelude::*;

    assert_eq!(a.len(), b.len());

    a.par_iter()
        .zip(b.par_iter())
        .map(|(&x, &y)| x * y)
        .sum()
}

//==================================================
// Real-world: Matrix multiplication with SIMD hints
//==================================================
fn matrix_multiply_simd(a: &[f32], b: &[f32], result: &mut [f32], n: usize) {
    // Matrix dimensions: n x n
    assert_eq!(a.len(), n * n);
    assert_eq!(b.len(), n * n);
    assert_eq!(result.len(), n * n);

    for i in 0..n {
        for j in 0..n {
            let mut sum = 0.0;

            // Inner loop is SIMD-friendly
            for k in 0..n {
                sum += a[i * n + k] * b[k * n + j];
            }

            result[i * n + j] = sum;
        }
    }
}

//======================================
// Parallel + SIMD matrix multiplication
//======================================
fn matrix_multiply_parallel_simd(a: &[f32], b: &[f32], n: usize) -> Vec<f32> {
    use rayon::prelude::*;

    let mut result = vec![0.0; n * n];

    (0..n).into_par_iter().for_each(|i| {
        for j in 0..n {
            let mut sum = 0.0;

            // This loop can be auto-vectorized
            for k in 0..n {
                sum += a[i * n + k] * b[k * n + j];
            }

            result[i * n + j] = sum;
        }
    });

    result
}

//=============================================================
// Pattern 4: Blocked matrix operations (cache + SIMD friendly)
//=============================================================
fn blocked_matrix_multiply(a: &[f32], b: &[f32], result: &mut [f32], n: usize, block_size: usize) {
    use rayon::prelude::*;

    for i_block in (0..n).step_by(block_size) {
        for j_block in (0..n).step_by(block_size) {
            for k_block in (0..n).step_by(block_size) {
                // Process block
                let i_end = (i_block + block_size).min(n);
                let j_end = (j_block + block_size).min(n);
                let k_end = (k_block + block_size).min(n);

                for i in i_block..i_end {
                    for j in j_block..j_end {
                        let mut sum = result[i * n + j];

                        // Inner loop is SIMD-friendly
                        for k in k_block..k_end {
                            sum += a[i * n + k] * b[k * n + j];
                        }

                        result[i * n + j] = sum;
                    }
                }
            }
        }
    }
}

//==============================
// Real-world: Image convolution
//==============================
fn convolve_2d(image: &[f32], kernel: &[f32], width: usize, height: usize, kernel_size: usize) -> Vec<f32> {
    use rayon::prelude::*;

    let offset = kernel_size / 2;
    let mut result = vec![0.0; width * height];

    (offset..height - offset).into_par_iter().for_each(|y| {
        for x in offset..width - offset {
            let mut sum = 0.0;

            // Convolution kernel (SIMD-friendly inner loops)
            for ky in 0..kernel_size {
                for kx in 0..kernel_size {
                    let img_y = y + ky - offset;
                    let img_x = x + kx - offset;
                    let img_idx = img_y * width + img_x;
                    let kernel_idx = ky * kernel_size + kx;

                    sum += image[img_idx] * kernel[kernel_idx];
                }
            }

            result[y * width + x] = sum;
        }
    });

    result
}

//===============================
// Pattern 5: Reduction with SIMD
//===============================
fn parallel_sum_simd(data: &[f32]) -> f32 {
    use rayon::prelude::*;

    // Split into chunks for parallel processing
    data.par_chunks(1024)
        .map(|chunk| {
            // Each chunk can be SIMD-vectorized
            chunk.iter().sum::<f32>()
        })
        .sum()
}

fn main() {
    println!("=== SIMD-Friendly Sum ===\n");

    let data: Vec<f32> = (0..1_000_000).map(|x| x as f32).collect();

    let start = std::time::Instant::now();
    let sum = simd_friendly_sum(&data);
    println!("Sum: {} in {:?}", sum, start.elapsed());

    println!("\n=== Vector Operations ===\n");

    let a: Vec<f32> = (0..1000).map(|x| x as f32).collect();
    let b: Vec<f32> = (0..1000).map(|x| (x * 2) as f32).collect();

    let start = std::time::Instant::now();
    let result = vector_add_parallel(&a, &b);
    println!("Vector add: {} elements in {:?}", result.len(), start.elapsed());

    println!("\n=== Dot Product ===\n");

    let start = std::time::Instant::now();
    let dot = dot_product_parallel(&a, &b);
    println!("Dot product: {} in {:?}", dot, start.elapsed());

    println!("\n=== Matrix Multiplication ===\n");

    let n = 512;
    let a: Vec<f32> = (0..n * n).map(|x| x as f32).collect();
    let b: Vec<f32> = (0..n * n).map(|x| (x * 2) as f32).collect();

    let start = std::time::Instant::now();
    let result = matrix_multiply_parallel_simd(&a, &b, n);
    println!("Matrix multiply ({}x{}): {:?}", n, n, start.elapsed());
    println!("Result checksum: {}", result.iter().sum::<f32>());

    println!("\n=== Parallel Sum with SIMD ===\n");

    let start = std::time::Instant::now();
    let sum = parallel_sum_simd(&data);
    println!("Parallel sum: {} in {:?}", sum, start.elapsed());
}
```

**SIMD Optimization Tips**:
- **Alignment**: Align data to 16/32-byte boundaries
- **Contiguous memory**: Use arrays/slices, not scattered data
- **Inner loops**: Make innermost loops SIMD-friendly
- **Combine with threading**: Rayon + SIMD for maximum performance
- **Profile**: Use compiler output to verify vectorization

---

### Example: Auto-Vectorization and Hints

Help the compiler generate SIMD code effectively.

```rust
//=================================================
// Pattern 1: Iterator patterns that auto-vectorize
//=================================================
fn auto_vectorize_examples() {
    let data: Vec<f32> = (0..10000).map(|x| x as f32).collect();

    // Good: Simple map (auto-vectorizes)
    let doubled: Vec<f32> = data.iter().map(|&x| x * 2.0).collect();

    // Good: Zip and map (auto-vectorizes)
    let summed: Vec<f32> = data
        .iter()
        .zip(data.iter())
        .map(|(&a, &b)| a + b)
        .collect();

    // Good: Chunks with fold (auto-vectorizes)
    let chunk_sums: Vec<f32> = data
        .chunks(4)
        .map(|chunk| chunk.iter().sum())
        .collect();
}

//======================================================
// Pattern 2: Explicit chunking for better vectorization
//======================================================
fn chunked_operations(data: &[f32]) -> Vec<f32> {
    let mut result = Vec::with_capacity(data.len());

    // Process in SIMD-width chunks
    const SIMD_WIDTH: usize = 8; // Typical for AVX

    for chunk in data.chunks(SIMD_WIDTH) {
        for &value in chunk {
            result.push(value * 2.0 + 1.0);
        }
    }

    result
}

//============================================================
// Pattern 3: Struct of Arrays (SoA) vs Array of Structs (AoS)
//============================================================
// Bad for SIMD: Array of Structs
#[derive(Copy, Clone)]
struct PointAoS {
    x: f32,
    y: f32,
    z: f32,
}

fn process_aos(points: &[PointAoS]) -> Vec<f32> {
    // Poor vectorization: scattered access
    points.iter().map(|p| p.x + p.y + p.z).collect()
}

//================================
// Good for SIMD: Struct of Arrays
//================================
struct PointsSoA {
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
}

impl PointsSoA {
    fn process(&self) -> Vec<f32> {
        // Good vectorization: contiguous access
        self.x
            .iter()
            .zip(self.y.iter())
            .zip(self.z.iter())
            .map(|((&x, &y), &z)| x + y + z)
            .collect()
    }
}

//========================================
// Real-world: Parallel + SIMD Monte Carlo
//========================================
fn monte_carlo_pi_parallel_simd(samples: usize) -> f64 {
    use rayon::prelude::*;
    use rand::Rng;

    let inside: usize = (0..samples)
        .into_par_iter()
        .chunks(1024) // Process in batches
        .map(|chunk| {
            let mut rng = rand::thread_rng();
            let mut count = 0;

            // Inner loop can be vectorized
            for _ in chunk {
                let x: f32 = rng.gen();
                let y: f32 = rng.gen();

                if x * x + y * y <= 1.0 {
                    count += 1;
                }
            }

            count
        })
        .sum();

    4.0 * inside as f64 / samples as f64
}

//===========================================
// Pattern 4: Benchmarking SIMD effectiveness
//===========================================
fn benchmark_vectorization() {
    let data: Vec<f32> = (0..10_000_000).map(|x| x as f32).collect();

    // Version 1: Simple loop
    let start = std::time::Instant::now();
    let mut result1 = Vec::with_capacity(data.len());
    for &x in &data {
        result1.push(x * 2.0 + 1.0);
    }
    let time1 = start.elapsed();

    // Version 2: Iterator (likely vectorized)
    let start = std::time::Instant::now();
    let result2: Vec<f32> = data.iter().map(|&x| x * 2.0 + 1.0).collect();
    let time2 = start.elapsed();

    // Version 3: Parallel + potential vectorization
    use rayon::prelude::*;
    let start = std::time::Instant::now();
    let result3: Vec<f32> = data.par_iter().map(|&x| x * 2.0 + 1.0).collect();
    let time3 = start.elapsed();

    println!("Simple loop: {:?}", time1);
    println!("Iterator: {:?}", time2);
    println!("Parallel: {:?}", time3);
    println!("Speedup (iter vs loop): {:.2}x", time1.as_secs_f64() / time2.as_secs_f64());
    println!("Speedup (parallel vs iter): {:.2}x", time2.as_secs_f64() / time3.as_secs_f64());
}

fn main() {
    println!("=== Auto-Vectorization ===\n");
    auto_vectorize_examples();

    println!("\n=== SoA vs AoS ===\n");

    let points_soa = PointsSoA {
        x: (0..10000).map(|i| i as f32).collect(),
        y: (0..10000).map(|i| (i * 2) as f32).collect(),
        z: (0..10000).map(|i| (i * 3) as f32).collect(),
    };

    let start = std::time::Instant::now();
    let sums = points_soa.process();
    println!("SoA processing: {:?}", start.elapsed());
    println!("Checksum: {}", sums.iter().sum::<f32>());

    println!("\n=== Monte Carlo Pi ===\n");

    let start = std::time::Instant::now();
    let pi = monte_carlo_pi_parallel_simd(10_000_000);
    println!("Pi estimate: {} in {:?}", pi, start.elapsed());

    println!("\n=== Vectorization Benchmark ===\n");
    benchmark_vectorization();
}
```

**Vectorization Guidelines**:
1. **Contiguous data**: Use slices/arrays
2. **Simple operations**: +, -, *, / vectorize well
3. **No branching**: Avoid if/else in hot loops
4. **Struct of Arrays**: Better than Array of Structs
5. **Verify**: Use `cargo rustc -- --emit asm` to check

---

### Summary

This chapter covered parallel algorithm patterns in Rust:

1. **Rayon Patterns**: par_iter, par_bridge for easy parallelization
2. **Work Partitioning**: Chunking, load balancing, recursive parallelism
3. **Parallel Reduce/Fold**: Aggregation patterns, custom accumulators
4. **Pipeline Parallelism**: Multi-stage processing with channels
5. **SIMD Parallelism**: Auto-vectorization, SoA layout, parallel + SIMD

**Key Takeaways**:
- **Rayon** makes data parallelism trivial (add `.par_`)
- **Work stealing** automatically balances irregular workloads
- **Grain size** matters: too small = overhead, too large = imbalance
- **fold + reduce** for custom aggregations
- **Pipeline** stages can have different parallelism levels
- **SIMD + threading** for maximum performance
- **SoA layout** enables better vectorization

**Performance Guidelines**:
- Use Rayon for CPU-bound data parallelism
- Combine thread parallelism and SIMD for best results
- Profile to find bottlenecks (CPU, memory, cache)
- Tune chunk size based on workload
- Use blocked algorithms for cache efficiency

**Common Patterns**:
- **Map**: Transform each element independently
- **Filter**: Select elements based on predicate
- **Reduce**: Aggregate to single value
- **Fold**: Accumulate with custom state
- **Pipeline**: Chain transformations

**When to Parallelize**:
- **Large datasets**: >10,000 items typically
- **CPU-bound**: Compute-intensive operations
- **Independent work**: No dependencies between items
- **Speedup > overhead**: Measure, don't assume

**Pitfalls to Avoid**:
- Over-parallelization (too many small tasks)
- False sharing (cache line contention)
- Sequential bottlenecks (Amdahl's law)
- Ignoring memory bandwidth limits
- Not profiling actual performance
