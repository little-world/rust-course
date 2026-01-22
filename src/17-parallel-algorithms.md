# Parallel Algorithms
This chapter explores parallel patterns using Rust's ecosystem, focusing on data parallelism with Rayon, work partitioning strategies, parallel reduction patterns, pipeline parallelism, and SIMD vectorization. We'll cover practical, production-ready examples for maximizing CPU utilization.

## Pattern 1: Rayon Patterns

**Problem**: Sequential code wastes modern multi-core CPUs—a 4-core CPU runs sequential code at 25% utilization. Manual threading with std::thread is complex: need to partition data, spawn threads, collect results, handle errors, avoid data races.

**Solution**: Use Rayon's parallel iterators with `.par_iter()`. Rayon provides work-stealing scheduler that automatically balances load across threads.

**Why It Matters**: Trivial code change yields massive speedups. Image processing 1M pixels: sequential 500ms, parallel 80ms (6x faster on 8-core).

**Use Cases**: Image processing (grayscale, filters, resizing), data validation (emails, phone numbers, formats), log parsing and analysis, batch data transformations, scientific computing, map-reduce operations, any embarrassingly parallel workload.


### Example: Parallel Iterator Basics

Replace `.iter()` with `.par_iter()` to distribute work across CPU cores using Rayon's work-stealing thread pool. Rayon balances load dynamically—threads that finish early steal work from slower threads. Avoid for small datasets (<1K items), I/O-bound work, or sequential dependencies.

```rust

fn parallel_map_example() {
    let numbers: Vec<i64> = (0..1_000_000).collect();

    // Sequential
    let start = Instant::now();
    let sequential: Vec<i64> = numbers.iter().map(|&x| x * x).collect();
    let seq_time = start.elapsed();

    // Parallel
    let start = Instant::now();
    let parallel: Vec<i64> = numbers.par_iter().map(|&x| x * x).collect();
    let par_time = start.elapsed();

    println!("Sequential: {:?}", seq_time);
    println!("Parallel: {:?}", par_time);
    println!("Speedup: {:.2}x", seq_time.as_secs_f64() / par_time.as_secs_f64());

    assert_eq!(sequential, parallel);
}

```

### Example: par_iter vs par_iter_mut vs into_par_iter

Three variants: `par_iter()` borrows immutably, `par_iter_mut()` borrows mutably for in-place modification, `into_par_iter()` consumes. Rayon ensures each thread accesses disjoint elements—no data races. Use `into_par_iter()` for expensive-to-clone types to avoid clone overhead.

```rust

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

 iterator_variants(); 
// Output: Sum: 15, Doubled: [2, 4, 6, 8, 10], Squares: [1, 4, 9, 16, 25]
```

### Example: Parallel filter and map

Chains `filter()` and `map()` with lazy evaluation—no intermediate collections, one fused pass. Filter early: `filter().map()` processes fewer elements than `map().filter()` when map is expensive. Rayon's work-stealing handles irregular workloads from filtering automatically.

```rust

fn parallel_filter_map() {
    let numbers: Vec<i32> = (0..10_000).collect();

    let result: Vec<i32> = numbers
        .par_iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * x)
        .collect();

    println!("Filtered and squared {} numbers", result.len());
}

```

### Example: Parallel flat_map

Flattens nested structures in parallel—each input produces multiple outputs combined into one collection. Essential for one-to-many transformations like directory traversal or tokenization. Nested `flat_map(|r| r.into_par_iter())` creates two-level parallelism. 

```rust

fn parallel_flat_map() {
    let ranges: Vec<std::ops::Range<i32>> = vec![0..10, 10..20, 20..30];

    let flattened: Vec<i32> = ranges
        .into_par_iter()
        .flat_map(|range| range.into_par_iter())
        .collect();

    println!("Flattened {} items", flattened.len());
}
```


### Image Processing

Parallel image ops on pixel data using `par_iter_mut()` and `par_chunks_mut()`. Grayscale processes RGB triplets; brightness adjusts each pixel. Perfect for data-parallel workloads where each pixel is independent.

```rust
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
//Usage
let mut img = Image::new(1920, 1080);
img.grayscale_parallel();
img.brightness_parallel(10);

```
### Log Parsing

Parses log lines in parallel using `filter_map()` to skip invalid entries. Each line is parsed independently—perfect parallelism. Combine with `filter()` to select specific log levels.

```rust
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
```

**Rayon Benefits**:
- **Minimal code changes**: Add `.par_` prefix
- **Work stealing**: Automatic load balancing
- **Type safe**: Same API as sequential iterators
- **No data races**: Enforced by type system

---


### Example: Bridge from sequential iterator

Bridges a standard iterator into Rayon's parallel world—downstream ops run in parallel. If 90% of time is in the source, `par_bridge()` provides only 10% speedup. Order may not be preserved—use `enumerate()` and sort if needed.

```rust
fn par_bridge_basic() {
    let iter = (0..1000).filter(|x| x % 2 == 0);

    // Bridge to parallel
    let sum: i32 = iter.par_bridge().map(|x| x * x).sum();
    println!("Sum: {}", sum);
}
```

### Example: Bridge from channel receiver

Parallelizes items from a channel—producer sends, workers process in parallel as items arrive. Decouples production from processing rate, reducing latency for streaming. Use `sync_channel(bound)` for backpressure.

```rust

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
```

### Example: File system traversal

Recursively traverses directories using a sequential iterator, then processes files in parallel via `par_bridge()`. I/O-bound traversal runs single-threaded while CPU-bound filtering runs parallel.

```rust
use std::fs;
use std::path::PathBuf;

fn find_large_files_parallel(root: &str, min_size: u64) -> Vec<(PathBuf, u64)> {
    fn visit_dirs(path: PathBuf) -> Box<dyn Iterator<Item = PathBuf> + Send> {
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
```

### Example: Database query results

Custom iterator simulates database fetches, `par_bridge()` processes results in parallel as they arrive. Useful when query returns rows one-at-a-time but processing is expensive.

```rust
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
```
### Example: Network stream processing

Processes network packets in parallel as they arrive via channel. Producer sends packets at network rate, consumers process them in parallel—decouples arrival from processing.

```rust
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
```

**par_bridge Use Cases**:
- **Channel receivers**: Process items as they arrive
- **Custom iterators**: Database cursors, file system traversal
- **Lazy evaluation**: Only compute when needed
- **Adaptive parallelism**: Work stealing adapts to varying workload

---

## Pattern 2: Work Partitioning Strategies

**Problem**: Bad work partitioning kills parallel performance. Too-small chunks (100 items) cause overhead—thread spawn/join costs dominate.

**Solution**: Choose grain size based on work: 1K-10K items for simple operations, larger for cache-heavy work. Use Rayon's adaptive chunking (default) for uniform work, explicit chunking for cache optimization.

**Why It Matters**: Grain size tuning: 2-3x performance difference. Matrix multiply with blocking: 5x faster due to cache hits.

**Use Cases**: Matrix operations (multiply, transpose, factorization), sorting algorithms (quicksort, mergesort), divide-and-conquer (tree traversal, expression evaluation), irregular workloads (graph algorithms), cache-sensitive operations (blocked algorithms).


### Example: Chunk sizes

Compares chunk sizes: default (Rayon adaptive), small (100), balanced (10K). Too-small chunks cause overhead; too-large causes imbalance. Start with defaults, tune if profiling shows issues.

```rust

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

```

### Example: Work splitting strategies

Compares equal splits (N/threads items each) vs adaptive (Rayon work-stealing). Static is cache-friendly; adaptive handles irregular workloads. Use static for uniform work, adaptive for variable work.

```rust
fn work_splitting_strategies() {
    let data: Vec<i32> = (0..100_000).collect();

    // Strategy 1: Equal splits (good for uniform work)
    let chunk_size = data.len() / rayon::current_num_threads();
    let result1: Vec<i32> = data
        .par_chunks(chunk_size.max(1))
        .flat_map(|chunk| chunk.par_iter().map(|&x| x * x))
        .collect();

    // Strategy 2: Adaptive (good for non-uniform work)
    let result2: Vec<i32> = data.par_iter().map(|&x| x * x).collect();

    assert_eq!(result1.len(), result2.len());
}
```

### Example: Matrix multiplication with blocking

Processes matrices in blocks that fit L1/L2 cache for better memory access patterns. Partitions output by blocks; each thread processes assigned blocks. Achieves 5x+ speedup from cache efficiency.

```rust
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
```
### Example: Parallel merge sort with optimal grain size

Parallelizes merge sort with `rayon::join()` for divide step. Switches to sequential `arr.sort()` below grain_size threshold to avoid overhead. Grain size of 1000-10000 typically optimal.

```rust
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
// Usage
let mut data: Vec<i32> = (0..100_000).rev().collect();
parallel_merge_sort(&mut data, 1000);
println!("Sorted: {}", data.windows(2).all(|w| w[0] <= w[1]));
```

### Example: Dynamic load balancing

Demonstrates work-stealing on irregular workloads where each item has different cost. Rayon automatically rebalances—threads that finish early steal from others.

```rust

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

```

### Example: Grain size tuning

Tests grain sizes (100, 1K, 10K, 100K) to find optimal balance. Larger grains reduce overhead; smaller grains improve load balance. Optimal depends on CPU cache and work complexity.

```rust

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
```

**Work Partitioning Guidelines**:
- **Grain size**: Larger grains reduce overhead, but may cause load imbalance
- **Chunk size**: Balance overhead vs parallelism (typically 1000-10000 items)
- **Cache blocking**: Improve cache locality with block-based partitioning
- **Work stealing**: Rayon automatically balances irregular workloads

---


### Example: Parallel quicksort

Uses `rayon::join()` to sort partitions in parallel—divide step spawns two tasks. Switches to sequential sort below threshold to avoid overhead. Achieves near-linear speedup on multi-core CPUs.

```rust

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
// Usage
let mut data: Vec<i32> = (0..100_000).rev().collect();
parallel_quicksort(&mut data);
println!("Sorted: {}", data.windows(2).all(|w| w[0] <= w[1]));

```

### Example: Parallel tree traversal

Traverses tree branches in parallel using `rayon::join()` for recursive descent. Returns vector of values in pre-order. Well-suited for balanced trees; unbalanced trees may cause work imbalance.

```rust

#[derive(Debug)]
struct TreeNode<T> {
    value: T,
    left: Option<Box<TreeNode<T>>>,
    right: Option<Box<TreeNode<T>>>,
}

impl<T: Send + Sync> TreeNode<T> {
    fn parallel_map<F, U>(&self, f: &F) -> TreeNode<U>
    where
        F: Fn(&T) -> U + Sync,
        U: Send + Sync,
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
        T: std::ops::Add<Output = T> + Default + Copy + Send + Sync,
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
```

### Example: Parallel Fibonacci (demonstrative, not efficient)

Demonstrates recursive parallelism with sequential cutoff at n<20 to avoid overhead. Teaching example only—real Fibonacci uses iterative or matrix methods.

```rust

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
// Usage
let result = parallel_fib(35);

```

### Example: Parallel directory size calculation

Recursively calculates directory sizes using `rayon::join()` for parallel subdirectory traversal. Sums file sizes at each level. I/O bound but parallelism helps on SSDs and networked storage.

```rust
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

// Usage: 
let size = parallel_dir_size("/home/user/projects"); // bytes
``` 
### Example Parallel expression evaluation

Evaluates expression trees by computing sub-expressions in parallel via `rayon::join()`. Binary ops split left/right subtrees across threads. Speedup depends on tree structure and operation cost.

```rust
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

**Problem**: Aggregating results from parallel operations is non-trivial. Simple sum/min/max work, but custom aggregations (histograms, statistics, merging maps) require careful design.

**Solution**: Use `reduce` for simple aggregations (sum, min, max, product). Use `fold + reduce` pattern for custom accumulators: fold builds per-thread state, reduce combines them.

**Why It Matters**: Statistics in one parallel pass instead of multiple sequential passes. Histogram generation: parallel fold+reduce 10x faster than sequential.

**Use Cases**: Statistics computation (mean, variance, stddev), histograms and frequency counting, word counting in text processing, aggregating results from parallel operations, merging sorted chunks, custom accumulators (sets, maps).


### Example: Simple reduce (sum, min, max)
Built-in reductions `sum()`, `min()`, `max()`, `product()` work directly on parallel iterators. Each thread computes local result, then results merge. Watch for overflow with `product()`.

```rust

fn simple_reductions() {
    let numbers: Vec<i64> = (1..=1_000_000).collect();

    // Sum
    let sum: i64 = numbers.par_iter().sum();
    println!("Sum: {}", sum);

    // Min/Max
    let min = numbers.par_iter().min().unwrap();
    let max = numbers.par_iter().max().unwrap();
    println!("Min: {}, Max: {}", min, max);

    // Product (be careful of overflow!)
    let small_numbers: Vec<i64> = (1..=10).collect();
    let product: i64 = small_numbers.par_iter().product();
    println!("Product: {}", product);
}

// Usage: 
simple_reductions(); // Output: Sum: 500000500000, Min: 1, Max: 1000000, Product: 3628800
```

### Example: Reduce with custom operation
Uses `reduce(|| identity, |a, b| combine)` for custom aggregations. Operation must be associative. Examples: string concatenation, finding closest element to target.

```rust

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

```

### Example: fold vs reduce
`fold()` creates per-thread accumulators, then requires `reduce()` to merge. Use fold when accumulator type differs from element type—essential for histograms, word counts, multi-field stats.

```rust
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

```

### Example: fold_with for custom accumulators
Collects multiple statistics (count, sum, min, max) in one parallel pass. Each thread maintains its own accumulator; `reduce()` merges at end. Much faster than multiple passes.

```rust

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
```
### Example: Parallel histogram
```rust
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
fn main() {
    let data: Vec<i32> = (0..10000).map(|i| i % 100).collect();
    let histogram = parallel_histogram(data);
    println!("Histogram buckets: {}", histogram.len());
    println!("Bucket 50: {}", histogram.get(&50).unwrap_or(&0));
}
```
### Example: Word frequency count

Splits text in parallel, builds per-thread HashMaps, merges with reduce. `par_split_whitespace()` handles tokenization. Pattern applies to any frequency counting task.

```rust
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
fn main() {
    let text = "the quick brown fox jumps over the lazy dog the fox".to_string();
    let freq = word_frequency_parallel(text);
    for (word, count) in freq.iter().take(5) {
        println!("{}: {}", word, count);
    }
}
```

### Example: Parallel variance calculation

Two-pass algorithm: first pass computes mean in parallel, second computes variance. More numerically stable than one-pass. Both passes use `par_iter()`.

```rust
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
fn main() {
    let numbers: Vec<f64> = (1..=100).map(|x| x as f64).collect();
    let (mean, variance) = parallel_variance(&numbers);
    println!("Mean: {:.2}, Variance: {:.2}, StdDev: {:.2}", mean, variance variance.sqrt());
}
```
### Example Merge sorted chunks

Merges sorted chunks in parallel using tree reduction—pairs merge concurrently, then merge results. Each iteration halves chunk count. O(log n) merge rounds with parallel work in each.

```rust
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
```

**Reduction Patterns**:
- **reduce**: Simple aggregation (sum, min, max)
- **fold + reduce**: Custom accumulator, then combine
- **Associative operations**: Required for correctness
- **Commutative**: Not required but helps performance

---

## Pattern 4: Pipeline Parallelism

**Problem**: Multi-stage data processing often bottlenecks on slowest stage. Sequential pipeline wastes CPU—decode thread idle while enhance runs.

**Solution**: Use channel-based pipelines with separate threads per stage. Rayon's par_iter at each stage for intra-stage parallelism.

**Why It Matters**: Image processing pipeline: sequential 300ms, staged parallel 100ms (3x faster). ETL pipeline processing 1M records: sequential 10min, parallel pipeline 2min (5x speedup).

**Use Cases**: ETL (Extract-Transform-Load) data pipelines, image/video processing (decode→enhance→compress), log analysis (parse→enrich→filter→aggregate), data transformation chains, streaming data processing, multi-stage batch jobs.


### Example: Simple pipeline with channels
Three-stage pipeline: generate→transform→filter→consume with channels between stages. `par_bridge()` parallelizes each stage. `sync_channel(bound)` provides backpressure.

```rust
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
```
### Example: Image processing pipeline

Chains decode→enhance→compress stages using `into_par_iter().map().map().map()`. All images process through stages in parallel. Stages overlap—while one batch decodes, another enhances.

```rust
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

    // Usage: let processed = ImagePipeline::process_batch(raw_images);

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
fn main() {
    let images: Vec<Vec<u8>> = (0..100).map(|_| vec![128; 1000]).collect();
    let start = std::time::Instant::now();
    let processed = ImagePipeline::process_batch(images);
}
```
### Example: Log processing pipeline

Chains parse→filter→aggregate stages using parallel iterators. Each stage processes all items in parallel. Uses `filter_map()` to combine parse+filter, reducing intermediate allocations.

```rust
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

    // Usage: let errors = LogPipeline::process(raw_logs);

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
fn main() {
    let logs: Vec<RawLog> = (0..1000)
        .map(|i| RawLog(format!("{}|{}|message_{}", i, if i % 10 == 0 { "ERROR" } else { "INFO" }, i)))
        .collect();
    let errors = LogPipeline::process(logs);
    println!("Found {} errors", errors.len());
}


```

### Example: Parallel stages with different parallelism
Three stages with different characteristics: light (high parallelism), heavy (larger chunks for cache), aggregation (parallel reduction). Tune chunk size per stage based on work.

```rust

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
            chunk.par_iter().map(|&x| {
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
```

### Example: ETL pipeline (Extract, Transform, Load)

Classic data pipeline: read files in parallel (extract), transform content (transform), aggregate results (load). Each phase uses `par_iter()`. Order-independent stages maximize parallelism.

```rust
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

**Problem**: CPU vector units (AVX2: 8 floats, AVX-512: 16 floats) sit idle with scalar code. Data-level parallelism untapped—process 1 element when hardware can do 8.

**Solution**: Write SIMD-friendly code: contiguous arrays, simple operations, no branches in hot loops. Use Struct-of-Arrays (SoA) instead of Array-of-Structs (AoS) for better vectorization.

**Why It Matters**: Matrix multiply: 10x speedup with SIMD+threading vs scalar sequential. Dot product: 4-8x faster with vectorization.

**Use Cases**: Matrix operations (multiply, transpose, dot product), image processing (convolution, filters), signal processing (FFT, filters), scientific computing (numerical methods), vector arithmetic, statistical computations.



### Example: Manual SIMD-friendly code
Processes 4 elements at a time in separate accumulators, enabling auto-vectorization. Handle remainder separately. Pattern works for sum, dot product, and other reductions.

```rust

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

let data: Vec<f32> = (0..1_000_000).map(|x| x as f32).collect();
let sum = simd_friendly_sum(&data);
```

### Example: Array operations (SIMD-friendly)
Element-wise `a[i] + b[i]` auto-vectorizes when arrays are contiguous. Combine with `par_iter()` for thread parallelism. Use `zip()` for multi-array operations.

```rust

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

let a: Vec<f32> = (0..1000).map(|x| x as f32).collect();
let b: Vec<f32> = (0..1000).map(|x| (x * 2) as f32).collect();
let result = vector_add_parallel(&a, &b);
```

### Example: Dot product (SIMD-friendly)
Dot product (`Σ a[i]*b[i]`) is perfectly suited for SIMD—independent multiply-accumulate ops. Sequential version auto-vectorizes; parallel version adds thread-level parallelism.

```rust

fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());

    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| x * y)
        .sum()
}

// Usage: let result = dot_product(&[1.0, 2.0], &[3.0, 4.0]); // 11.0

fn dot_product_parallel(a: &[f32], b: &[f32]) -> f32 {
    use rayon::prelude::*;

    assert_eq!(a.len(), b.len());

    a.par_iter()
        .zip(b.par_iter())
        .map(|(&x, &y)| x * y)
        .sum()
}
 
let result = dot_product_parallel(&big_vec_a, &big_vec_b);
```

### Matrix multiplication with SIMD hints

Inner k-loop performs contiguous multiply-accumulate operations that compilers can auto-vectorize. Outer loops parallelize over rows with Rayon. Combines threading (row distribution) with SIMD (inner loop) for maximum throughput.

```rust
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
// Parallel + SIMD matrix multiplication
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
fn main() {
    let n = 512;
    let a: Vec<f32> = (0..n * n).map(|x| x as f32).collect();
    let b: Vec<f32> = (0..n * n).map(|x| (x * 2) as f32).collect();

    let result = matrix_multiply_parallel_simd(&a, &b, n);
    println!("Result checksum: {}", result.iter().sum::<f32>());
}

```

### Example: Blocked matrix operations (cache + SIMD friendly)
Processes matrices in blocks that fit L1/L2 cache. Inner loops stay hot in cache. Achieves 5-10x speedup from cache efficiency alone before SIMD/threading.

```rust

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
```
### Example: Image convolution

Applies kernel filter to image—each output pixel computed from neighborhood sum. Outer loop over rows parallelizes with `par_iter()`. Inner kernel loop is SIMD-friendly multiply-accumulate.

```rust
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
```

### Example: Reduction with SIMD
Combines threading (`par_chunks`) with SIMD-friendly chunk processing. Each chunk's `iter().sum()` auto-vectorizes. Achieves both 8x thread speedup and 4-8x SIMD speedup.

```rust
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

```

**SIMD Optimization Tips**:
- **Alignment**: Align data to 16/32-byte boundaries
- **Contiguous memory**: Use arrays/slices, not scattered data
- **Inner loops**: Make innermost loops SIMD-friendly
- **Combine with threading**: Rayon + SIMD for maximum performance
- **Profile**: Use compiler output to verify vectorization

---

### Example: Iterator patterns that auto-vectorize
Simple `map(|x| x * 2.0)` and `zip().map()` auto-vectorize well. Avoid closures with captured state or complex control flow. Test in release builds only.

```rust
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
```

### Example: Explicit chunking for better vectorization
Process in chunks matching SIMD width (8 for AVX). Inner loop over chunk elements becomes single SIMD instruction. Match chunk size to CPU's vector width.

```rust
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
```

### Example: Struct of Arrays (SoA) vs Array of Structs (AoS)
AoS (`Vec<Point>`) scatters x,y,z across memory—bad for SIMD. SoA stores each field contiguously, enabling vectorization. SoA can be 4-8x faster for SIMD workloads.

```rust

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
// Good for SIMD: Struct of Arrays
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
fn main() {
    let points_soa = PointsSoA {
        x: (0..10000).map(|i| i as f32).collect(),
        y: (0..10000).map(|i| (i * 2) as f32).collect(),
        z: (0..10000).map(|i| (i * 3) as f32).collect(),
    };
    let sums = points_soa.process();
}

```
### Example Parallel + SIMD Monte Carlo

Estimates π using random points—threads process chunks while inner loops can auto-vectorize. Combines thread parallelism (chunk distribution) with SIMD-friendly point-in-circle calculation.

```rust
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
let pi = monte_carlo_pi_parallel_simd(10_000_000);
```

### Example: Benchmarking SIMD effectiveness
Compares loop, iterator (likely vectorized), and parallel versions. Speedup loop→iterator reveals SIMD benefit; iterator→parallel reveals threading benefit.

```rust

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
