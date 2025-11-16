# Threading Patterns

This chapter explores concurrent programming patterns in Rust using threads. We'll cover thread lifecycle management, parallel work distribution, message passing, shared state synchronization, and coordination primitives through practical, production-ready examples.

## Table of Contents

1. [Thread Spawn and Join Patterns](#thread-spawn-and-join-patterns)
2. [Thread Pools and Work Stealing](#thread-pools-and-work-stealing)
3. [Message Passing with Channels](#message-passing-with-channels)
4. [Shared State with Arc/Mutex](#shared-state-with-arcmutex)
5. [Barrier and Condvar Patterns](#barrier-and-condvar-patterns)

---

## Thread Spawn and Join Patterns

Thread spawning is the foundation of parallel execution in Rust. Understanding spawn, join, and scoped threads is essential for safe concurrent programming.

### Recipe 1: Basic Thread Spawning and Data Transfer

**Problem**: Execute multiple independent tasks in parallel and collect their results safely.

**Solution**:

```rust
use std::thread;
use std::time::Duration;

//====================================
// Pattern 1: Thread with move closure
//====================================
fn spawn_with_owned_data() {
    let data = vec![1, 2, 3, 4, 5];

    //======================
    // Move data into thread
    //======================
    let handle = thread::spawn(move || {
        let sum: i32 = data.iter().sum();
        println!("Sum calculated by thread: {}", sum);
        sum
    });

    //===============================
    // Wait for thread and get result
    //===============================
    let result = handle.join().unwrap();
    println!("Result from thread: {}", result);
}

//======================================================
// Pattern 2: Multiple threads returning different types
//======================================================
fn parallel_computations() {
    let numbers = vec![1, 2, 3, 4, 5];

    //================================
    // Clone data for multiple threads
    //================================
    let numbers_clone1 = numbers.clone();
    let numbers_clone2 = numbers.clone();

    let sum_handle = thread::spawn(move || {
        numbers_clone1.iter().sum::<i32>()
    });

    let product_handle = thread::spawn(move || {
        numbers_clone2.iter().product::<i32>()
    });

    let sum = sum_handle.join().unwrap();
    let product = product_handle.join().unwrap();

    println!("Sum: {}, Product: {}", sum, product);
}

//======================================
// Pattern 3: Thread with error handling
//======================================
fn thread_with_error_handling() {
    let handle = thread::spawn(|| {
        //=====================================
        // Simulate computation that might fail
        //=====================================
        if rand::random::<bool>() {
            Ok(42)
        } else {
            Err("Computation failed")
        }
    });

    match handle.join() {
        Ok(Ok(value)) => println!("Success: {}", value),
        Ok(Err(e)) => println!("Thread returned error: {}", e),
        Err(_) => println!("Thread panicked!"),
    }
}

//=======================================
// Pattern 4: Named threads for debugging
//=======================================
fn named_threads() {
    let handles: Vec<_> = (0..3)
        .map(|i| {
            thread::Builder::new()
                .name(format!("worker-{}", i))
                .spawn(move || {
                    println!("Thread {} starting", i);
                    thread::sleep(Duration::from_millis(100));
                    println!("Thread {} done", i);
                    i * 2
                })
                .unwrap()
        })
        .collect();

    let results: Vec<i32> = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect();

    println!("Results: {:?}", results);
}

//=============================================
// Real-world example: Parallel file processing
//=============================================
use std::fs;
use std::path::PathBuf;

struct FileProcessor;

impl FileProcessor {
    fn process_files_parallel(paths: Vec<PathBuf>) -> Vec<ProcessResult> {
        let handles: Vec<_> = paths
            .into_iter()
            .map(|path| {
                thread::spawn(move || {
                    Self::process_file(&path)
                })
            })
            .collect();

        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect()
    }

    fn process_file(path: &PathBuf) -> ProcessResult {
        match fs::read_to_string(path) {
            Ok(content) => ProcessResult {
                path: path.clone(),
                line_count: content.lines().count(),
                word_count: content.split_whitespace().count(),
                byte_count: content.len(),
                error: None,
            },
            Err(e) => ProcessResult {
                path: path.clone(),
                line_count: 0,
                word_count: 0,
                byte_count: 0,
                error: Some(e.to_string()),
            },
        }
    }
}

#[derive(Debug)]
struct ProcessResult {
    path: PathBuf,
    line_count: usize,
    word_count: usize,
    byte_count: usize,
    error: Option<String>,
}

fn main() {
    println!("=== Basic Thread Spawning ===\n");
    spawn_with_owned_data();

    println!("\n=== Parallel Computations ===\n");
    parallel_computations();

    println!("\n=== Named Threads ===\n");
    named_threads();
}
```

**Key Concepts**:
- `move` closure transfers ownership to thread
- `join()` waits for thread completion and retrieves result
- Thread panics are caught by `join()`
- Use `thread::Builder` for thread configuration

---

### Recipe 2: Scoped Threads for Borrowing

**Problem**: Spawn threads that borrow data from the parent scope without requiring `'static` lifetime.

**Solution**:

```rust
use std::thread;

//==================================================
// Problem: This won't compile (non-static lifetime)
//==================================================
// fn broken_borrow() {
//==============================
//     let data = vec![1, 2, 3];
//==============================
//     thread::spawn(|| {
//==================================================================================
//         println!("{:?}", data); // Error: borrowed value doesn't live long enough
//==================================================================================
//     });
//==
// }
//==

//=============================
// Solution: Use scoped threads
//=============================
fn scoped_threads_borrowing() {
    let mut data = vec![1, 2, 3, 4, 5];

    thread::scope(|s| {
        //====================================
        // Spawn thread that borrows immutably
        //====================================
        s.spawn(|| {
            println!("Sum: {}", data.iter().sum::<i32>());
        });

        //============================================
        // Spawn another thread that borrows immutably
        //============================================
        s.spawn(|| {
            println!("Product: {}", data.iter().product::<i32>());
        });

        //===================================
        // Both threads can read concurrently
        //===================================
    }); // All scoped threads join here automatically

    //===========================
    // After scope, we can mutate
    //===========================
    data.push(6);
    println!("Extended data: {:?}", data);
}

//===================================================
// Pattern: Parallel processing with shared reference
//===================================================
fn parallel_search() {
    let haystack = vec![1, 5, 3, 8, 2, 9, 4, 7, 6];
    let needle = 8;
    let mut found_at = None;

    thread::scope(|s| {
        let chunk_size = haystack.len() / 4;

        for (i, chunk) in haystack.chunks(chunk_size).enumerate() {
            s.spawn(move || {
                if let Some(pos) = chunk.iter().position(|&x| x == needle) {
                    println!("Thread {} found at position {}", i, pos);
                }
            });
        }
    });
}

//=======================================
// Real-world: Parallel matrix operations
//=======================================
struct Matrix {
    data: Vec<Vec<f64>>,
    rows: usize,
    cols: usize,
}

impl Matrix {
    fn new(rows: usize, cols: usize) -> Self {
        Self {
            data: vec![vec![0.0; cols]; rows],
            rows,
            cols,
        }
    }

    fn parallel_row_operation<F>(&mut self, operation: F)
    where
        F: Fn(&mut [f64]) + Send + Sync,
    {
        thread::scope(|s| {
            for row in &mut self.data {
                //=============================================
                // Borrow each row mutably in different threads
                //=============================================
                s.spawn(|| {
                    operation(row);
                });
            }
        });
    }

    fn parallel_multiply(&self, scalar: f64) -> Matrix {
        let mut result = Matrix::new(self.rows, self.cols);

        thread::scope(|s| {
            for (i, row) in self.data.iter().enumerate() {
                let result_row = &mut result.data[i];
                s.spawn(move || {
                    for (j, &value) in row.iter().enumerate() {
                        result_row[j] = value * scalar;
                    }
                });
            }
        });

        result
    }
}

//================================================
// Pattern: Divide and conquer with scoped threads
//================================================
fn parallel_quicksort<T: Ord + Send>(arr: &mut [T]) {
    if arr.len() <= 1 {
        return;
    }

    let pivot_idx = partition(arr);
    let (left, right) = arr.split_at_mut(pivot_idx);

    thread::scope(|s| {
        s.spawn(|| parallel_quicksort(left));
        s.spawn(|| parallel_quicksort(&mut right[1..]));
    });
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

fn main() {
    println!("=== Scoped Threads ===\n");
    scoped_threads_borrowing();

    println!("\n=== Parallel Search ===\n");
    parallel_search();

    println!("\n=== Parallel Sort ===\n");
    let mut data = vec![5, 2, 8, 1, 9, 3, 7, 4, 6];
    println!("Before: {:?}", data);
    parallel_quicksort(&mut data);
    println!("After: {:?}", data);
}
```

**Scoped Thread Benefits**:
- Borrow data from parent scope (no `'static` required)
- Automatic joining at scope end
- Safe mutable access to different parts of data
- Ideal for divide-and-conquer algorithms

---

### Recipe 3: Thread Pool Pattern (Manual Implementation)

**Problem**: Reuse threads for multiple tasks to avoid the overhead of repeated thread creation.

**Solution**:

```rust
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

enum Message {
    NewJob(Job),
    Terminate,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

impl ThreadPool {
    fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }

    fn active_count(&self) -> usize {
        self.workers.len()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

//===========================================
// Real-world example: HTTP server simulation
//===========================================
use std::time::Duration;

fn simulate_http_server() {
    let pool = ThreadPool::new(4);

    //===========================
    // Simulate incoming requests
    //===========================
    for request_id in 0..10 {
        pool.execute(move || {
            println!("Handling request {}", request_id);
            //==============
            // Simulate work
            //==============
            thread::sleep(Duration::from_millis(500));
            println!("Request {} completed", request_id);
        });
    }

    //=====================================================
    // Pool will wait for all jobs to complete when dropped
    //=====================================================
}

//==================================
// Real-world: Batch data processing
//==================================
struct BatchProcessor {
    pool: ThreadPool,
}

impl BatchProcessor {
    fn new(num_threads: usize) -> Self {
        Self {
            pool: ThreadPool::new(num_threads),
        }
    }

    fn process_batch(&self, items: Vec<i32>) {
        for item in items {
            self.pool.execute(move || {
                //===============================
                // Simulate expensive computation
                //===============================
                let result = item * item;
                println!("Processed {}: result = {}", item, result);
            });
        }
    }
}

fn main() {
    println!("=== Thread Pool ===\n");
    simulate_http_server();

    println!("\n=== Batch Processing ===\n");
    let processor = BatchProcessor::new(3);
    processor.process_batch(vec![1, 2, 3, 4, 5, 6, 7, 8]);

    //================================
    // Wait for processing to complete
    //================================
    thread::sleep(Duration::from_secs(2));
}
```

**Thread Pool Advantages**:
- Amortize thread creation cost
- Limit concurrent resource usage
- Better CPU cache utilization
- Controlled parallelism

---

## Thread Pools and Work Stealing

Work stealing enables dynamic load balancing across threads, improving throughput for irregular workloads.

### Recipe 4: Rayon for Data Parallelism

**Problem**: Process large datasets in parallel with automatic work distribution.

**Solution**:

```rust
//========================================
// Note: Add `rayon = "1.8"` to Cargo.toml
//========================================
use rayon::prelude::*;

//==============================
// Pattern 1: Parallel iteration
//==============================
fn parallel_map_reduce() {
    let numbers: Vec<i32> = (1..=1_000_000).collect();

    //=============
    // Parallel sum
    //=============
    let sum: i32 = numbers.par_iter().sum();
    println!("Sum: {}", sum);

    //=============
    // Parallel map
    //=============
    let squares: Vec<i32> = numbers
        .par_iter()
        .map(|&x| x * x)
        .collect();

    println!("First 10 squares: {:?}", &squares[..10]);

    //================
    // Parallel filter
    //================
    let evens: Vec<i32> = numbers
        .par_iter()
        .filter(|&&x| x % 2 == 0)
        .copied()
        .collect();

    println!("Number of evens: {}", evens.len());
}

//============================
// Pattern 2: Parallel sorting
//============================
fn parallel_sorting() {
    let mut data: Vec<i32> = (0..1_000_000).rev().collect();

    //========================================================
    // Parallel sort (faster than sequential for large arrays)
    //========================================================
    data.par_sort();

    println!("Sorted data (first 10): {:?}", &data[..10]);
    println!("Sorted data (last 10): {:?}", &data[data.len() - 10..]);
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
            pixels: vec![0; width * height * 3], // RGB
            width,
            height,
        }
    }

    fn apply_filter_parallel(&mut self, filter: fn(u8) -> u8) {
        self.pixels.par_iter_mut().for_each(|pixel| {
            *pixel = filter(*pixel);
        });
    }

    fn brighten(&mut self, amount: u8) {
        self.apply_filter_parallel(|p| p.saturating_add(amount));
    }

    fn contrast(&mut self, factor: f32) {
        self.pixels.par_iter_mut().for_each(|pixel| {
            let adjusted = (*pixel as f32 - 128.0) * factor + 128.0;
            *pixel = adjusted.clamp(0.0, 255.0) as u8;
        });
    }

    fn parallel_blur(&mut self, radius: usize) {
        let width = self.width;
        let height = self.height;
        let old_pixels = self.pixels.clone();

        self.pixels
            .par_chunks_mut(width * 3)
            .enumerate()
            .for_each(|(y, row)| {
                for x in 0..width {
                    for c in 0..3 {
                        let mut sum = 0u32;
                        let mut count = 0u32;

                        for dy in -(radius as isize)..=radius as isize {
                            for dx in -(radius as isize)..=radius as isize {
                                let ny = (y as isize + dy).clamp(0, height as isize - 1) as usize;
                                let nx = (x as isize + dx).clamp(0, width as isize - 1) as usize;

                                sum += old_pixels[(ny * width + nx) * 3 + c] as u32;
                                count += 1;
                            }
                        }

                        row[x * 3 + c] = (sum / count) as u8;
                    }
                }
            });
    }
}

//=========================
// Real-world: Log analysis
//=========================
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct LogEntry {
    timestamp: u64,
    level: String,
    message: String,
}

fn analyze_logs_parallel(logs: Vec<LogEntry>) -> HashMap<String, usize> {
    //============================
    // Parallel group-by and count
    //============================
    logs.par_iter()
        .fold(
            || HashMap::new(),
            |mut map, entry| {
                *map.entry(entry.level.clone()).or_insert(0) += 1;
                map
            },
        )
        .reduce(
            || HashMap::new(),
            |mut a, b| {
                for (k, v) in b {
                    *a.entry(k).or_insert(0) += v;
                }
                a
            },
        )
}

//=================================================
// Pattern 3: Parallel reduce with custom operation
//=================================================
fn parallel_custom_reduce() {
    let numbers: Vec<i32> = (1..=100).collect();

    //=============================
    // Find min and max in parallel
    //=============================
    let (min, max) = numbers
        .par_iter()
        .fold(
            || (i32::MAX, i32::MIN),
            |(min, max), &x| (min.min(x), max.max(x)),
        )
        .reduce(
            || (i32::MAX, i32::MIN),
            |(min1, max1), (min2, max2)| (min1.min(min2), max1.max(max2)),
        );

    println!("Min: {}, Max: {}", min, max);
}

fn main() {
    println!("=== Parallel Map/Reduce ===\n");
    parallel_map_reduce();

    println!("\n=== Parallel Sorting ===\n");
    parallel_sorting();

    println!("\n=== Parallel Custom Reduce ===\n");
    parallel_custom_reduce();

    println!("\n=== Log Analysis ===\n");
    let logs = vec![
        LogEntry {
            timestamp: 1,
            level: "INFO".to_string(),
            message: "Started".to_string(),
        },
        LogEntry {
            timestamp: 2,
            level: "ERROR".to_string(),
            message: "Failed".to_string(),
        },
        LogEntry {
            timestamp: 3,
            level: "INFO".to_string(),
            message: "Retry".to_string(),
        },
    ];

    let stats = analyze_logs_parallel(logs);
    println!("Log level counts: {:?}", stats);
}
```

**Rayon Features**:
- **Work stealing**: Idle threads steal work from busy threads
- **Automatic chunking**: Divides work optimally
- **Recursive parallelism**: Nested parallel operations
- **Zero-cost abstraction**: Similar performance to manual threading

---

### Recipe 5: Custom Work Stealing Queue

**Problem**: Implement work stealing for task-based parallelism with dynamic load balancing.

**Solution**:

```rust
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

struct WorkStealingPool<T> {
    queues: Vec<Arc<Mutex<VecDeque<T>>>>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl<T> WorkStealingPool<T>
where
    T: Send + 'static,
{
    fn new<F>(num_threads: usize, worker_fn: F) -> Self
    where
        F: Fn(T) + Send + Sync + Clone + 'static,
    {
        let mut queues = Vec::new();
        let mut workers = Vec::new();

        //===============================
        // Create a queue for each worker
        //===============================
        for _ in 0..num_threads {
            queues.push(Arc::new(Mutex::new(VecDeque::new())));
        }

        //==============
        // Spawn workers
        //==============
        for i in 0..num_threads {
            let my_queue = Arc::clone(&queues[i]);
            let steal_queues: Vec<_> = queues
                .iter()
                .enumerate()
                .filter(|(idx, _)| *idx != i)
                .map(|(_, q)| Arc::clone(q))
                .collect();

            let worker_fn = worker_fn.clone();

            let worker = thread::spawn(move || {
                loop {
                    //===============================
                    // Try to get work from own queue
                    //===============================
                    let task = {
                        let mut queue = my_queue.lock().unwrap();
                        queue.pop_front()
                    };

                    if let Some(task) = task {
                        worker_fn(task);
                        continue;
                    }

                    //===============================
                    // Try to steal from other queues
                    //===============================
                    let mut stolen = false;
                    for steal_queue in &steal_queues {
                        let task = {
                            let mut queue = steal_queue.lock().unwrap();
                            queue.pop_back() // Steal from back
                        };

                        if let Some(task) = task {
                            worker_fn(task);
                            stolen = true;
                            break;
                        }
                    }

                    if !stolen {
                        //=================================
                        // No work available, sleep briefly
                        //=================================
                        thread::sleep(Duration::from_micros(100));
                    }
                }
            });

            workers.push(worker);
        }

        Self { queues, workers }
    }

    fn submit(&self, task: T, worker_id: usize) {
        let queue = &self.queues[worker_id % self.queues.len()];
        queue.lock().unwrap().push_back(task);
    }
}

//=====================================================
// Real-world: Recursive task decomposition (Fibonacci)
//=====================================================
use std::sync::atomic::{AtomicBool, Ordering};

fn fibonacci_work_stealing() {
    let running = Arc::new(AtomicBool::new(true));
    let results = Arc::new(Mutex::new(Vec::new()));

    let running_clone = Arc::clone(&running);
    let results_clone = Arc::clone(&results);

    let pool = WorkStealingPool::new(4, move |n: u64| {
        let fib = compute_fibonacci(n);
        println!("fib({}) = {}", n, fib);
        results_clone.lock().unwrap().push((n, fib));
    });

    //=============
    // Submit tasks
    //=============
    for i in 1..=20 {
        pool.submit(i, i as usize);
    }

    //====================
    // Let workers process
    //====================
    thread::sleep(Duration::from_secs(2));
}

fn compute_fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => compute_fibonacci(n - 1) + compute_fibonacci(n - 2),
    }
}

fn main() {
    println!("=== Work Stealing Pool ===\n");
    fibonacci_work_stealing();
}
```

**Work Stealing Benefits**:
- **Load balancing**: Idle threads help busy threads
- **Cache locality**: Workers process own queue first
- **Scalability**: Near-linear speedup for irregular workloads

---

## Message Passing with Channels

Channels enable safe communication between threads following "share memory by communicating" philosophy.

### Recipe 6: MPSC Channel Patterns

**Problem**: Coordinate multiple producer threads sending data to a single consumer.

**Solution**:

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

//===================================
// Pattern 1: Basic producer-consumer
//===================================
fn basic_mpsc() {
    let (tx, rx) = mpsc::channel();

    //===============
    // Spawn producer
    //===============
    thread::spawn(move || {
        for i in 0..5 {
            tx.send(i).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });

    //=========
    // Consumer
    //=========
    for received in rx {
        println!("Received: {}", received);
    }
}

//==============================
// Pattern 2: Multiple producers
//==============================
fn multiple_producers() {
    let (tx, rx) = mpsc::channel();

    for thread_id in 0..3 {
        let tx_clone = tx.clone();
        thread::spawn(move || {
            for i in 0..5 {
                let message = format!("Thread {} sends {}", thread_id, i);
                tx_clone.send(message).unwrap();
                thread::sleep(Duration::from_millis(50));
            }
        });
    }

    //=====================
    // Drop original sender
    //=====================
    drop(tx);

    //=====================
    // Receive all messages
    //=====================
    for received in rx {
        println!("{}", received);
    }
}

//=============================================================
// Pattern 3: Rendezvous channel (sync_channel with 0 capacity)
//=============================================================
fn rendezvous_channel() {
    let (tx, rx) = mpsc::sync_channel(0);

    thread::spawn(move || {
        println!("Sending...");
        tx.send("Hello").unwrap();
        println!("Sent! (receiver must have received)");
    });

    thread::sleep(Duration::from_secs(1));
    println!("Receiving...");
    let msg = rx.recv().unwrap();
    println!("Received: {}", msg);
}

//=============================
// Real-world: Pipeline pattern
//=============================
#[derive(Debug)]
struct RawData(String);

#[derive(Debug)]
struct ProcessedData {
    original: String,
    length: usize,
    uppercase: String,
}

#[derive(Debug)]
struct EnrichedData {
    data: ProcessedData,
    timestamp: u64,
}

fn data_pipeline() {
    let (raw_tx, raw_rx) = mpsc::channel::<RawData>();
    let (processed_tx, processed_rx) = mpsc::channel::<ProcessedData>();
    let (enriched_tx, enriched_rx) = mpsc::channel::<EnrichedData>();

    //========================
    // Stage 1: Data ingestion
    //========================
    thread::spawn(move || {
        for i in 0..5 {
            let data = RawData(format!("data_{}", i));
            println!("Ingested: {:?}", data);
            raw_tx.send(data).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });

    //====================
    // Stage 2: Processing
    //====================
    thread::spawn(move || {
        for raw in raw_rx {
            let processed = ProcessedData {
                length: raw.0.len(),
                uppercase: raw.0.to_uppercase(),
                original: raw.0,
            };
            println!("Processed: {:?}", processed);
            processed_tx.send(processed).unwrap();
        }
    });

    //====================
    // Stage 3: Enrichment
    //====================
    thread::spawn(move || {
        for data in processed_rx {
            let enriched = EnrichedData {
                data,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };
            println!("Enriched: {:?}", enriched);
            enriched_tx.send(enriched).unwrap();
        }
    });

    //================
    // Stage 4: Output
    //================
    for result in enriched_rx {
        println!("Final output: {:?}", result);
    }
}

//=====================================
// Real-world: Fan-out / Fan-in pattern
//=====================================
fn fan_out_fan_in() {
    let (input_tx, input_rx) = mpsc::channel::<i32>();
    let (output_tx, output_rx) = mpsc::channel::<i32>();

    //================
    // Input generator
    //================
    thread::spawn(move || {
        for i in 0..20 {
            input_tx.send(i).unwrap();
        }
    });

    //==========================
    // Fan-out: Multiple workers
    //==========================
    let num_workers = 4;
    for _ in 0..num_workers {
        let input_rx = input_rx.clone();
        let output_tx = output_tx.clone();

        thread::spawn(move || {
            for value in input_rx {
                //==============
                // Simulate work
                //==============
                thread::sleep(Duration::from_millis(10));
                let result = value * value;
                output_tx.send(result).unwrap();
            }
        });
    }

    drop(input_rx); // Close input channel
    drop(output_tx); // Close output channel

    //========================
    // Fan-in: Collect results
    //========================
    let results: Vec<i32> = output_rx.iter().collect();
    println!("Collected {} results", results.len());
    println!("First 10: {:?}", &results[..10.min(results.len())]);
}

fn main() {
    println!("=== Basic MPSC ===\n");
    basic_mpsc();

    println!("\n=== Multiple Producers ===\n");
    multiple_producers();

    println!("\n=== Rendezvous Channel ===\n");
    rendezvous_channel();

    println!("\n=== Data Pipeline ===\n");
    data_pipeline();

    println!("\n=== Fan-out / Fan-in ===\n");
    fan_out_fan_in();
}
```

**Channel Patterns**:
- **Pipeline**: Chain of processing stages
- **Fan-out**: Distribute work to multiple workers
- **Fan-in**: Collect results from multiple workers
- **Rendezvous**: Synchronous handoff

---

### Recipe 7: Crossbeam Channels for Advanced Patterns

**Problem**: Implement complex communication patterns with bounded channels, selection, and timeouts.

**Solution**:

```rust
//============================================
// Note: Add `crossbeam = "0.8"` to Cargo.toml
//============================================
use crossbeam::channel::{bounded, unbounded, select, Sender, Receiver};
use std::thread;
use std::time::Duration;

//=============================================
// Pattern 1: Bounded channel with backpressure
//=============================================
fn bounded_channel_backpressure() {
    let (tx, rx) = bounded(3);

    //==============
    // Fast producer
    //==============
    let producer = thread::spawn(move || {
        for i in 0..10 {
            println!("Trying to send {}", i);
            tx.send(i).unwrap(); // Blocks when channel is full
            println!("Sent {}", i);
        }
    });

    //==============
    // Slow consumer
    //==============
    let consumer = thread::spawn(move || {
        for value in rx {
            println!("Received {}", value);
            thread::sleep(Duration::from_millis(500));
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}

//=================================================
// Pattern 2: Select - waiting on multiple channels
//=================================================
fn channel_selection() {
    let (tx1, rx1) = unbounded();
    let (tx2, rx2) = unbounded();

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        tx1.send("from channel 1").unwrap();
    });

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        tx2.send("from channel 2").unwrap();
    });

    //================================
    // Select whichever is ready first
    //================================
    select! {
        recv(rx1) -> msg => println!("Received: {:?}", msg),
        recv(rx2) -> msg => println!("Received: {:?}", msg),
    }

    //===============================
    // Can select again for the other
    //===============================
    select! {
        recv(rx1) -> msg => println!("Received: {:?}", msg),
        recv(rx2) -> msg => println!("Received: {:?}", msg),
    }
}

//===================
// Pattern 3: Timeout
//===================
fn channel_with_timeout() {
    let (tx, rx) = unbounded();

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));
        tx.send("late message").unwrap();
    });

    select! {
        recv(rx) -> msg => println!("Received: {:?}", msg),
        default(Duration::from_secs(1)) => println!("Timeout!"),
    }
}

//========================
// Real-world: Actor model
//========================
enum ActorMessage {
    Process(String),
    GetState(Sender<String>),
    Shutdown,
}

struct Actor {
    inbox: Receiver<ActorMessage>,
    state: String,
}

impl Actor {
    fn new(inbox: Receiver<ActorMessage>) -> Self {
        Self {
            inbox,
            state: String::new(),
        }
    }

    fn run(mut self) {
        loop {
            select! {
                recv(self.inbox) -> msg => {
                    match msg {
                        Ok(ActorMessage::Process(data)) => {
                            self.state.push_str(&data);
                            println!("Actor processed: {}", data);
                        }
                        Ok(ActorMessage::GetState(reply)) => {
                            reply.send(self.state.clone()).unwrap();
                        }
                        Ok(ActorMessage::Shutdown) => {
                            println!("Actor shutting down");
                            break;
                        }
                        Err(_) => break,
                    }
                }
            }
        }
    }
}

fn actor_pattern() {
    let (tx, rx) = unbounded();

    let actor_handle = thread::spawn(move || {
        let actor = Actor::new(rx);
        actor.run();
    });

    //=======================
    // Send messages to actor
    //=======================
    tx.send(ActorMessage::Process("Hello ".to_string())).unwrap();
    tx.send(ActorMessage::Process("World!".to_string())).unwrap();

    //============
    // Query state
    //============
    let (reply_tx, reply_rx) = unbounded();
    tx.send(ActorMessage::GetState(reply_tx)).unwrap();
    let state = reply_rx.recv().unwrap();
    println!("Actor state: {}", state);

    //=========
    // Shutdown
    //=========
    tx.send(ActorMessage::Shutdown).unwrap();
    actor_handle.join().unwrap();
}

//=====================================
// Real-world: Request-response pattern
//=====================================
struct Request {
    id: u64,
    data: String,
    reply: Sender<Response>,
}

struct Response {
    id: u64,
    result: String,
}

fn request_response_pattern() {
    let (req_tx, req_rx) = unbounded::<Request>();

    //=======
    // Server
    //=======
    thread::spawn(move || {
        for request in req_rx {
            let response = Response {
                id: request.id,
                result: format!("Processed: {}", request.data),
            };
            request.reply.send(response).unwrap();
        }
    });

    //========
    // Clients
    //========
    for i in 0..5 {
        let req_tx = req_tx.clone();
        thread::spawn(move || {
            let (reply_tx, reply_rx) = unbounded();
            let request = Request {
                id: i,
                data: format!("request_{}", i),
                reply: reply_tx,
            };

            req_tx.send(request).unwrap();
            let response = reply_rx.recv().unwrap();
            println!("Client {} got response: {}", i, response.result);
        });
    }

    thread::sleep(Duration::from_secs(1));
}

fn main() {
    println!("=== Bounded Channel with Backpressure ===\n");
    bounded_channel_backpressure();

    println!("\n=== Channel Selection ===\n");
    channel_selection();

    println!("\n=== Timeout ===\n");
    channel_with_timeout();

    println!("\n=== Actor Pattern ===\n");
    actor_pattern();

    println!("\n=== Request-Response ===\n");
    request_response_pattern();
}
```

**Crossbeam Channel Features**:
- **Bounded/Unbounded**: Control memory usage
- **Select**: Wait on multiple channels (like Go's select)
- **Timeout**: Avoid indefinite blocking
- **MPMC**: Multiple producers and consumers

---

## Shared State with Arc/Mutex

Shared state enables multiple threads to safely access and modify common data.

### Recipe 8: Arc and Mutex Fundamentals

**Problem**: Share mutable state across threads safely with atomic reference counting and mutual exclusion.

**Solution**:

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

//==========================
// Pattern 1: Shared counter
//==========================
fn shared_counter() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let mut num = counter.lock().unwrap();
                *num += 1;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", *counter.lock().unwrap());
}

//================================
// Pattern 2: Lock guard and scope
//================================
fn lock_guard_scope() {
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));
    let data_clone = Arc::clone(&data);

    thread::spawn(move || {
        {
            let mut vec = data_clone.lock().unwrap();
            vec.push(4);
        } // Lock released here

        //=======================
        // Can acquire lock again
        //=======================
        let vec = data_clone.lock().unwrap();
        println!("Thread sees: {:?}", *vec);
    })
    .join()
    .unwrap();

    let vec = data.lock().unwrap();
    println!("Main sees: {:?}", *vec);
}

//===================================
// Pattern 3: Try-lock (non-blocking)
//===================================
fn try_lock_pattern() {
    let data = Arc::new(Mutex::new(0));
    let data_clone = Arc::clone(&data);

    let handle = thread::spawn(move || {
        let mut num = data_clone.lock().unwrap();
        *num += 1;
        thread::sleep(Duration::from_secs(2)); // Hold lock for a while
    });

    thread::sleep(Duration::from_millis(100));

    //=====================================
    // Try to acquire lock without blocking
    //=====================================
    match data.try_lock() {
        Ok(mut num) => {
            *num += 1;
            println!("Got lock!");
        }
        Err(_) => {
            println!("Lock is held by another thread");
        }
    }

    handle.join().unwrap();
}

//=========================
// Real-world: Shared cache
//=========================
use std::collections::HashMap;

struct Cache {
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl Cache {
    fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn get(&self, key: &str) -> Option<String> {
        let cache = self.data.lock().unwrap();
        cache.get(key).cloned()
    }

    fn set(&self, key: String, value: String) {
        let mut cache = self.data.lock().unwrap();
        cache.insert(key, value);
    }

    fn clone_handle(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}

fn shared_cache_example() {
    let cache = Cache::new();
    let mut handles = vec![];

    //===============
    // Writer threads
    //===============
    for i in 0..3 {
        let cache = cache.clone_handle();
        handles.push(thread::spawn(move || {
            cache.set(format!("key_{}", i), format!("value_{}", i));
        }));
    }

    //===============
    // Reader threads
    //===============
    for i in 0..3 {
        let cache = cache.clone_handle();
        handles.push(thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            if let Some(value) = cache.get(&format!("key_{}", i)) {
                println!("Read: {}", value);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

//========================================
// Real-world: Thread-safe connection pool
//========================================
struct Connection {
    id: usize,
}

struct ConnectionPool {
    connections: Arc<Mutex<Vec<Connection>>>,
}

impl ConnectionPool {
    fn new(size: usize) -> Self {
        let connections: Vec<_> = (0..size).map(|id| Connection { id }).collect();

        Self {
            connections: Arc::new(Mutex::new(connections)),
        }
    }

    fn acquire(&self) -> Option<Connection> {
        let mut pool = self.connections.lock().unwrap();
        pool.pop()
    }

    fn release(&self, conn: Connection) {
        let mut pool = self.connections.lock().unwrap();
        pool.push(conn);
    }

    fn clone_handle(&self) -> Self {
        Self {
            connections: Arc::clone(&self.connections),
        }
    }
}

fn connection_pool_example() {
    let pool = ConnectionPool::new(3);
    let mut handles = vec![];

    for i in 0..5 {
        let pool = pool.clone_handle();
        handles.push(thread::spawn(move || {
            if let Some(conn) = pool.acquire() {
                println!("Thread {} acquired connection {}", i, conn.id);
                thread::sleep(Duration::from_millis(100));
                pool.release(conn);
                println!("Thread {} released connection", i);
            } else {
                println!("Thread {} couldn't acquire connection", i);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn main() {
    println!("=== Shared Counter ===\n");
    shared_counter();

    println!("\n=== Lock Guard Scope ===\n");
    lock_guard_scope();

    println!("\n=== Try Lock ===\n");
    try_lock_pattern();

    println!("\n=== Shared Cache ===\n");
    shared_cache_example();

    println!("\n=== Connection Pool ===\n");
    connection_pool_example();
}
```

**Arc/Mutex Best Practices**:
- Keep critical sections small
- Avoid holding locks across await points
- Use `try_lock()` to avoid deadlocks
- Consider RwLock for read-heavy workloads

---

### Recipe 9: RwLock for Read-Heavy Workloads

**Problem**: Allow multiple concurrent readers while ensuring exclusive write access.

**Solution**:

```rust
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::HashMap;

//===========================================
// Pattern 1: Multiple readers, single writer
//===========================================
fn rwlock_basic() {
    let data = Arc::new(RwLock::new(vec![1, 2, 3]));
    let mut handles = vec![];

    //==============
    // Spawn readers
    //==============
    for i in 0..5 {
        let data = Arc::clone(&data);
        handles.push(thread::spawn(move || {
            let vec = data.read().unwrap();
            println!("Reader {} sees: {:?}", i, *vec);
            thread::sleep(Duration::from_millis(100));
        }));
    }

    //=============
    // Spawn writer
    //=============
    let data_clone = Arc::clone(&data);
    handles.push(thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        let mut vec = data_clone.write().unwrap();
        vec.push(4);
        println!("Writer added element");
    }));

    for handle in handles {
        handle.join().unwrap();
    }
}

//==================================
// Real-world: Configuration manager
//==================================
struct Config {
    settings: HashMap<String, String>,
}

struct ConfigManager {
    config: Arc<RwLock<Config>>,
}

impl ConfigManager {
    fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(Config {
                settings: HashMap::new(),
            })),
        }
    }

    fn get(&self, key: &str) -> Option<String> {
        let config = self.config.read().unwrap();
        config.settings.get(key).cloned()
    }

    fn set(&self, key: String, value: String) {
        let mut config = self.config.write().unwrap();
        config.settings.insert(key, value);
    }

    fn update_batch(&self, updates: HashMap<String, String>) {
        let mut config = self.config.write().unwrap();
        for (k, v) in updates {
            config.settings.insert(k, v);
        }
    }

    fn clone_handle(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
        }
    }
}

//===================================================
// Benchmark: RwLock vs Mutex for read-heavy workload
//===================================================
fn benchmark_rwlock_vs_mutex() {
    const READERS: usize = 10;
    const OPERATIONS: usize = 10000;

    //================
    // Test with Mutex
    //================
    println!("Testing with Mutex:");
    let mutex_data = Arc::new(std::sync::Mutex::new(0));
    let start = Instant::now();

    let handles: Vec<_> = (0..READERS)
        .map(|_| {
            let data = Arc::clone(&mutex_data);
            thread::spawn(move || {
                for _ in 0..OPERATIONS {
                    let _val = *data.lock().unwrap();
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let mutex_time = start.elapsed();
    println!("  Time: {:?}", mutex_time);

    //=================
    // Test with RwLock
    //=================
    println!("\nTesting with RwLock:");
    let rwlock_data = Arc::new(RwLock::new(0));
    let start = Instant::now();

    let handles: Vec<_> = (0..READERS)
        .map(|_| {
            let data = Arc::clone(&rwlock_data);
            thread::spawn(move || {
                for _ in 0..OPERATIONS {
                    let _val = *data.read().unwrap();
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let rwlock_time = start.elapsed();
    println!("  Time: {:?}", rwlock_time);

    println!(
        "\nRwLock speedup: {:.2}x",
        mutex_time.as_secs_f64() / rwlock_time.as_secs_f64()
    );
}

//===============================
// Real-world: In-memory database
//===============================
struct Database {
    data: Arc<RwLock<HashMap<String, String>>>,
}

impl Database {
    fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn get(&self, key: &str) -> Option<String> {
        let db = self.data.read().unwrap();
        db.get(key).cloned()
    }

    fn set(&self, key: String, value: String) {
        let mut db = self.data.write().unwrap();
        db.insert(key, value);
    }

    fn transaction<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HashMap<String, String>) -> R,
    {
        let mut db = self.data.write().unwrap();
        f(&mut db)
    }

    fn snapshot(&self) -> HashMap<String, String> {
        let db = self.data.read().unwrap();
        db.clone()
    }

    fn clone_handle(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}

fn database_example() {
    let db = Database::new();

    //=============
    // Initial data
    //=============
    db.transaction(|data| {
        data.insert("user:1".to_string(), "Alice".to_string());
        data.insert("user:2".to_string(), "Bob".to_string());
    });

    let mut handles = vec![];

    //=============
    // Many readers
    //=============
    for i in 1..=10 {
        let db = db.clone_handle();
        handles.push(thread::spawn(move || {
            if let Some(user) = db.get("user:1") {
                println!("Reader {} sees: {}", i, user);
            }
        }));
    }

    //==================
    // Occasional writer
    //==================
    let db_clone = db.clone_handle();
    handles.push(thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        db_clone.set("user:3".to_string(), "Charlie".to_string());
        println!("Writer added user:3");
    }));

    for handle in handles {
        handle.join().unwrap();
    }

    let snapshot = db.snapshot();
    println!("\nFinal database: {:?}", snapshot);
}

fn main() {
    println!("=== RwLock Basic ===\n");
    rwlock_basic();

    println!("\n=== Database Example ===\n");
    database_example();

    println!("\n=== Benchmark ===\n");
    benchmark_rwlock_vs_mutex();
}
```

**RwLock vs Mutex**:
- **RwLock**: Multiple readers OR one writer
- **Mutex**: Only one accessor at a time
- **Use RwLock** when: Read-heavy workload (>90% reads)
- **Use Mutex** when: Many writes or short critical sections

---

## Barrier and Condvar Patterns

Synchronization primitives enable coordinating thread execution at specific points.

### Recipe 10: Barrier for Phased Computation

**Problem**: Synchronize multiple threads at specific points, ensuring all threads reach a barrier before any proceed.

**Solution**:

```rust
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

//=========================================
// Pattern 1: Basic barrier synchronization
//=========================================
fn basic_barrier() {
    let barrier = Arc::new(Barrier::new(3));
    let mut handles = vec![];

    for i in 0..3 {
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            println!("Thread {} before barrier", i);
            thread::sleep(Duration::from_millis(i as u64 * 100));

            barrier.wait();

            println!("Thread {} after barrier", i);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

//===================================
// Pattern 2: Multi-phase computation
//===================================
fn multi_phase_computation() {
    let num_threads = 4;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];

    for id in 0..num_threads {
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            //====================
            // Phase 1: Initialize
            //====================
            println!("Thread {} initializing", id);
            thread::sleep(Duration::from_millis(50));
            barrier.wait();

            //=================
            // Phase 2: Process
            //=================
            println!("Thread {} processing", id);
            thread::sleep(Duration::from_millis(50));
            barrier.wait();

            //==================
            // Phase 3: Finalize
            //==================
            println!("Thread {} finalizing", id);
            thread::sleep(Duration::from_millis(50));
            barrier.wait();

            println!("Thread {} done", id);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

//===============================================
// Real-world: Parallel simulation with timesteps
//===============================================
struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
}

impl Particle {
    fn update(&mut self, dt: f64) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
    }
}

fn parallel_simulation() {
    let num_threads = 4;
    let particles_per_thread = 100;
    let timesteps = 5;

    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let barrier = Arc::clone(&barrier);

        handles.push(thread::spawn(move || {
            let mut particles: Vec<Particle> = (0..particles_per_thread)
                .map(|i| Particle {
                    x: (thread_id * particles_per_thread + i) as f64,
                    y: 0.0,
                    vx: 1.0,
                    vy: 1.0,
                })
                .collect();

            for t in 0..timesteps {
                //=================
                // Update particles
                //=================
                for particle in &mut particles {
                    particle.update(0.1);
                }

                //===============================================
                // Wait for all threads to complete this timestep
                //===============================================
                barrier.wait();

                if thread_id == 0 {
                    println!("Timestep {} completed", t);
                }
            }

            particles
        }));
    }

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    println!("Simulation complete. Total particles: {}", results.iter().map(|p| p.len()).sum::<usize>());
}

//=========================================================
// Real-world: Parallel matrix multiplication with barriers
//=========================================================
fn parallel_matrix_multiply() {
    const SIZE: usize = 1000;
    const NUM_THREADS: usize = 4;

    let a = vec![vec![1.0; SIZE]; SIZE];
    let b = vec![vec![2.0; SIZE]; SIZE];
    let c = Arc::new(std::sync::Mutex::new(vec![vec![0.0; SIZE]; SIZE]));
    let barrier = Arc::new(Barrier::new(NUM_THREADS));

    let mut handles = vec![];
    let rows_per_thread = SIZE / NUM_THREADS;

    for thread_id in 0..NUM_THREADS {
        let a = a.clone();
        let b = b.clone();
        let c = Arc::clone(&c);
        let barrier = Arc::clone(&barrier);

        handles.push(thread::spawn(move || {
            let start_row = thread_id * rows_per_thread;
            let end_row = if thread_id == NUM_THREADS - 1 {
                SIZE
            } else {
                (thread_id + 1) * rows_per_thread
            };

            println!("Thread {} computing rows {}-{}", thread_id, start_row, end_row);

            //======================
            // Compute assigned rows
            //======================
            let mut local_result = vec![vec![0.0; SIZE]; end_row - start_row];

            for i in 0..(end_row - start_row) {
                for j in 0..SIZE {
                    for k in 0..SIZE {
                        local_result[i][j] += a[start_row + i][k] * b[k][j];
                    }
                }
            }

            //==============
            // Write results
            //==============
            let mut c = c.lock().unwrap();
            for i in 0..(end_row - start_row) {
                c[start_row + i] = local_result[i].clone();
            }
            drop(c);

            //=====================
            // Wait for all threads
            //=====================
            barrier.wait();

            if thread_id == 0 {
                println!("Matrix multiplication complete");
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn main() {
    println!("=== Basic Barrier ===\n");
    basic_barrier();

    println!("\n=== Multi-Phase Computation ===\n");
    multi_phase_computation();

    println!("\n=== Parallel Simulation ===\n");
    parallel_simulation();

    println!("\n=== Parallel Matrix Multiply ===\n");
    parallel_matrix_multiply();
}
```

**Barrier Use Cases**:
- Phased algorithms (timestep simulations)
- Parallel matrix operations
- Synchronizing initialization/cleanup
- Iterative algorithms with dependencies

---

### Recipe 11: Condvar for Complex Synchronization

**Problem**: Wait for a condition to become true, enabling efficient thread coordination beyond simple barriers.

**Solution**:

```rust
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;
use std::collections::VecDeque;

//==========================================
// Pattern 1: Producer-Consumer with Condvar
//==========================================
fn producer_consumer_condvar() {
    let queue = Arc::new((Mutex::new(VecDeque::new()), Condvar::new()));
    let (queue_clone, queue_clone2) = (Arc::clone(&queue), Arc::clone(&queue));

    //=========
    // Producer
    //=========
    let producer = thread::spawn(move || {
        for i in 0..5 {
            thread::sleep(Duration::from_millis(100));

            let (lock, cvar) = &*queue_clone;
            let mut q = lock.lock().unwrap();
            q.push_back(i);
            println!("Produced: {}", i);
            cvar.notify_one(); // Wake up one waiting consumer
        }
    });

    //=========
    // Consumer
    //=========
    let consumer = thread::spawn(move || {
        let (lock, cvar) = &*queue_clone2;

        for _ in 0..5 {
            let mut q = lock.lock().unwrap();

            //==============================
            // Wait until queue is non-empty
            //==============================
            while q.is_empty() {
                q = cvar.wait(q).unwrap();
            }

            let item = q.pop_front().unwrap();
            println!("Consumed: {}", item);
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}

//=============================
// Pattern 2: Wait with timeout
//=============================
fn condvar_with_timeout() {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair_clone = Arc::clone(&pair);

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));
        let (lock, cvar) = &*pair_clone;
        let mut ready = lock.lock().unwrap();
        *ready = true;
        cvar.notify_one();
    });

    let (lock, cvar) = &*pair;
    let mut ready = lock.lock().unwrap();

    //========================
    // Wait for up to 1 second
    //========================
    let result = cvar
        .wait_timeout_while(ready, Duration::from_secs(1), |&mut ready| !ready)
        .unwrap();

    if result.1.timed_out() {
        println!("Timed out waiting for condition");
    } else {
        println!("Condition became true");
    }
}

//===================================
// Real-world: Bounded blocking queue
//===================================
struct BoundedQueue<T> {
    queue: Mutex<VecDeque<T>>,
    not_empty: Condvar,
    not_full: Condvar,
    capacity: usize,
}

impl<T> BoundedQueue<T> {
    fn new(capacity: usize) -> Self {
        Self {
            queue: Mutex::new(VecDeque::with_capacity(capacity)),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
            capacity,
        }
    }

    fn push(&self, item: T) {
        let mut queue = self.queue.lock().unwrap();

        //=============================
        // Wait until queue is not full
        //=============================
        while queue.len() >= self.capacity {
            queue = self.not_full.wait(queue).unwrap();
        }

        queue.push_back(item);
        self.not_empty.notify_one();
    }

    fn pop(&self) -> T {
        let mut queue = self.queue.lock().unwrap();

        //==============================
        // Wait until queue is not empty
        //==============================
        while queue.is_empty() {
            queue = self.not_empty.wait(queue).unwrap();
        }

        let item = queue.pop_front().unwrap();
        self.not_full.notify_one();
        item
    }

    fn try_pop(&self, timeout: Duration) -> Option<T> {
        let mut queue = self.queue.lock().unwrap();

        let result = self
            .not_empty
            .wait_timeout_while(queue, timeout, |q| q.is_empty())
            .unwrap();

        if result.1.timed_out() {
            None
        } else {
            let item = result.0.pop_front();
            if item.is_some() {
                self.not_full.notify_one();
            }
            item
        }
    }
}

fn bounded_queue_example() {
    let queue = Arc::new(BoundedQueue::new(3));

    //==============
    // Fast producer
    //==============
    let queue_clone = Arc::clone(&queue);
    let producer = thread::spawn(move || {
        for i in 0..10 {
            println!("Pushing {}", i);
            queue_clone.push(i);
            println!("Pushed {}", i);
        }
    });

    //==============
    // Slow consumer
    //==============
    let queue_clone = Arc::clone(&queue);
    let consumer = thread::spawn(move || {
        for _ in 0..10 {
            thread::sleep(Duration::from_millis(200));
            let item = queue_clone.pop();
            println!("Popped {}", item);
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}

//========================================
// Real-world: Thread pool with work queue
//========================================
struct WorkerPool {
    queue: Arc<(Mutex<VecDeque<Box<dyn FnOnce() + Send>>>, Condvar)>,
    workers: Vec<thread::JoinHandle<()>>,
    shutdown: Arc<Mutex<bool>>,
}

impl WorkerPool {
    fn new(num_workers: usize) -> Self {
        let queue = Arc::new((Mutex::new(VecDeque::new()), Condvar::new()));
        let shutdown = Arc::new(Mutex::new(false));
        let mut workers = Vec::new();

        for id in 0..num_workers {
            let queue = Arc::clone(&queue);
            let shutdown = Arc::clone(&shutdown);

            workers.push(thread::spawn(move || {
                loop {
                    let (lock, cvar) = &*queue;
                    let mut q = lock.lock().unwrap();

                    //==========================
                    // Wait for work or shutdown
                    //==========================
                    while q.is_empty() && !*shutdown.lock().unwrap() {
                        q = cvar.wait(q).unwrap();
                    }

                    if *shutdown.lock().unwrap() && q.is_empty() {
                        println!("Worker {} shutting down", id);
                        break;
                    }

                    if let Some(job) = q.pop_front() {
                        drop(q); // Release lock while executing
                        job();
                    }
                }
            }));
        }

        Self {
            queue,
            workers,
            shutdown,
        }
    }

    fn submit<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let (lock, cvar) = &*self.queue;
        let mut q = lock.lock().unwrap();
        q.push_back(Box::new(job));
        cvar.notify_one();
    }

    fn shutdown(self) {
        *self.shutdown.lock().unwrap() = true;
        let (_, cvar) = &*self.queue;
        cvar.notify_all();

        for worker in self.workers {
            worker.join().unwrap();
        }
    }
}

fn worker_pool_example() {
    let pool = WorkerPool::new(4);

    for i in 0..10 {
        pool.submit(move || {
            println!("Task {} executing", i);
            thread::sleep(Duration::from_millis(100));
            println!("Task {} done", i);
        });
    }

    thread::sleep(Duration::from_secs(2));
    pool.shutdown();
}

fn main() {
    println!("=== Producer-Consumer with Condvar ===\n");
    producer_consumer_condvar();

    println!("\n=== Condvar with Timeout ===\n");
    condvar_with_timeout();

    println!("\n=== Bounded Queue ===\n");
    bounded_queue_example();

    println!("\n=== Worker Pool ===\n");
    worker_pool_example();
}
```

**Condvar Patterns**:
- **Wait**: Block until condition is true
- **Notify**: Wake up waiting threads
- **Spurious wakeups**: Always check condition in loop
- **Timeout**: Avoid indefinite blocking

---

## Summary

This chapter covered essential threading patterns in Rust:

1. **Thread Spawn/Join**: Basic parallelism, scoped threads for borrowing, error handling
2. **Thread Pools**: Reuse threads, work stealing with Rayon, custom pools
3. **Message Passing**: MPSC channels, pipeline patterns, fan-out/fan-in, crossbeam channels
4. **Shared State**: Arc/Mutex fundamentals, RwLock for read-heavy workloads
5. **Synchronization**: Barriers for phased computation, Condvar for complex coordination

**Key Takeaways**:
- **Prefer message passing** over shared state when possible
- **Use scoped threads** to borrow data safely
- **Rayon** provides excellent ergonomics for data parallelism
- **RwLock** is faster than Mutex for read-heavy workloads (>90% reads)
- **Barriers** synchronize phases in parallel algorithms
- **Condvar** enables waiting for complex conditions

**Performance Guidelines**:
- Thread pools eliminate spawn/join overhead
- Work stealing balances irregular workloads
- Minimize lock hold time
- Use atomic operations for counters
- Profile before optimizing concurrency

**Safety Guarantees**:
- Rust's type system prevents data races at compile time
- Send/Sync traits ensure thread safety
- Ownership prevents use-after-free
- No deadlocks from forgetting to unlock
