## Project 2: Parallel Image Processor with Thread Pool

### Problem Statement

Build a parallel image processing application using a thread pool to process multiple images concurrently. The system should resize, filter, and save images using worker threads, with task distribution and result collection.

### Use Cases

- Image/video processing pipelines
- Web server request handling
- Batch data processing
- Parallel compilation systems
- Database query execution
- Scientific simulations

---

### Why It Matters

Thread pools amortize thread creation overhead and limit resource usage. Creating threads per task is expensive (1-2ms per spawn) and unbounded. Thread pool reuses threads and queues excess work.

For 10,000 small tasks:
- Spawn per task: 10-20 seconds (thread creation overhead)
- Thread pool (8 workers): 1-2 seconds (reuse threads)

Your image processor should:
- Load images from directory
- Distribute processing across worker threads
- Apply transformations (resize, blur, brightness adjustment)
- Save processed images to output directory
- Report progress and completion status
- Handle errors gracefully (corrupted images, disk full)


## Milestone 1: Basic Thread Pool Implementation

Implement a simple thread pool with fixed number of worker threads. Workers pull tasks from shared queue and execute them.

### Architecture

**Structs:**
- `ThreadPool` - Manages worker threads
    - **Field** `workers: Vec<JoinHandle<()>>` - Worker thread handles
    - **Field** `sender: Sender<Job>` - Task submission channel
    - **Field** `shutdown: Arc<AtomicBool>` - Shutdown signal

- `Job` - Unit of work
    - Type alias: `Box<dyn FnOnce() + Send + 'static>`

**Key Functions:**
- `new(size: usize) -> ThreadPool` - Create pool with N workers
- `execute<F>(&self, f: F)` where `F: FnOnce() + Send + 'static` - Submit task
- `shutdown(self)` - Stop all workers gracefully

**Role Each Plays:**
- Worker threads: Loop receiving and executing jobs
- Shared channel: Distributes work across workers
- Shutdown flag: Coordinates graceful termination

### Checkpoint Tests

```rust
#[test]
fn test_thread_pool_execution() {
    use std::sync::{Arc, Mutex};

    let pool = ThreadPool::new(4);
    let counter = Arc::new(Mutex::new(0));

    for _ in 0..100 {
        let c = counter.clone();
        pool.execute(move || {
            let mut num = c.lock().unwrap();
            *num += 1;
        });
    }

    pool.shutdown();

    assert_eq!(*counter.lock().unwrap(), 100);
}

#[test]
fn test_parallel_speedup() {
    use std::time::Instant;

    let pool = ThreadPool::new(4);
    let start = Instant::now();

    for _ in 0..8 {
        pool.execute(|| {
            thread::sleep(Duration::from_millis(100));
        });
    }

    pool.shutdown();
    let elapsed = start.elapsed();

    // 8 tasks @ 100ms each on 4 workers ≈ 200ms total
    assert!(elapsed < Duration::from_millis(300));
}
```

### Starter Code

```rust
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<JoinHandle<()>>,
    sender: Sender<Job>,
    shutdown: Arc<AtomicBool>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        // TODO: Create channel for jobs
        // Spawn 'size' worker threads
        // Each worker loops: recv job -> execute -> repeat
        // Return ThreadPool with workers and sender
        unimplemented!()
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // TODO: Box closure and send through channel
        // Hint: self.sender.send(Box::new(f)).unwrap()
        unimplemented!()
    }

    pub fn shutdown(self) {
        // TODO: Set shutdown flag
        // Drop sender to close channel
        // Join all worker threads
        unimplemented!()
    }
}

fn worker_loop(receiver: Arc<Mutex<Receiver<Job>>>, shutdown: Arc<AtomicBool>) {
    // TODO: Loop while !shutdown:
    //   - Lock receiver
    //   - Try to recv job (with timeout to check shutdown)
    //   - If job received, execute it
    //   - Drop lock
    unimplemented!()
}
```

#### Why previous Milestone is not enough: N/A - Foundation Milestone.

**What's the improvement:** Thread pool vs spawn-per-task:
- Spawn-per-task: 1000 tasks × 1ms spawn = 1 second overhead
- Thread pool: 0 overhead (threads pre-spawned)

For high-frequency tasks (web requests, image tiles), thread pool is mandatory.

---

## Milestone 2: Image Processing Tasks

### Introduction

Add image processing functionality: load, resize, apply filters, save. Distribute tasks across thread pool workers.

### Architecture

**Structs:**
- `ImageTask` - Processing job
    - **Field** `input_path: PathBuf` - Source image
    - **Field** `output_path: PathBuf` - Destination
    - **Field** `operations: Vec<Operation>` - Transformations to apply

- `Operation` - Transformation enum
    - **Variant** `Resize(u32, u32)` - New dimensions
    - **Variant** `Blur(f32)` - Blur radius
    - **Variant** `Brighten(i32)` - Brightness delta

**Key Functions:**
- `process_image(task: ImageTask) -> Result<(), ImageError>`
- `load_image(path: &Path) -> Result<ImageBuffer, ImageError>`
- `save_image(image: &ImageBuffer, path: &Path) -> Result<(), ImageError>`

### Checkpoint Tests

```rust
#[test]
fn test_image_resize() {
    let task = ImageTask {
        input_path: PathBuf::from("test.png"),
        output_path: PathBuf::from("out.png"),
        operations: vec![Operation::Resize(100, 100)],
    };

    let result = process_image(task);
    assert!(result.is_ok());
}

#[test]
fn test_parallel_processing() {
    let pool = ThreadPool::new(4);
    let counter = Arc::new(AtomicUsize::new(0));

    for i in 0..10 {
        let c = counter.clone();
        pool.execute(move || {
            // Simulate image processing
            thread::sleep(Duration::from_millis(50));
            c.fetch_add(1, Ordering::SeqCst);
        });
    }

    pool.shutdown();
    assert_eq!(counter.load(Ordering::SeqCst), 10);
}
```

### Starter Code

```rust
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Operation {
    Resize(u32, u32),
    Blur(f32),
    Brighten(i32),
}

pub struct ImageTask {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub operations: Vec<Operation>,
}

pub fn process_image(task: ImageTask) -> Result<(), String> {
    // TODO: Load image from input_path
    // Apply each operation in sequence
    // Save to output_path
    // Hint: Use image crate for actual processing
    //   let mut img = image::open(&task.input_path)?;
    //   for op in task.operations {
    //     img = apply_operation(img, op);
    //   }
    //   img.save(&task.output_path)?;
    unimplemented!()
}

fn apply_operation(img: ImageBuffer, op: Operation) -> ImageBuffer {
    // TODO: Match on operation and apply transformation
    // Resize: image::resize()
    // Blur: image::blur()
    // Brighten: image::brighten()
    unimplemented!()
}
```

**Why previous Milestone is not enough:** Thread pool without real work is just overhead. Need actual tasks to process.

**What's the improvement:** Parallel image processing scales linearly:
- Sequential: 10 images × 500ms = 5 seconds
- Parallel (8 cores): 10 images / 8 = ~625ms

For batch processing (thousands of images), parallelism is essential.

---

## Milestone 3: Progress Tracking and Results

### Introduction

Track processing progress and collect results. Report completion percentage, failed tasks, and aggregate statistics.

### Architecture

**Enhanced Structs:**
- `ProcessingResult` - Task outcome
    - **Field** `task_id: usize`
    - **Field** `status: TaskStatus` - Success/Failed
    - **Field** `duration: Duration` - Processing time
    - **Field** `error: Option<String>` - Error message if failed

- `ProgressTracker` - Monitor progress
    - **Field** `total: usize` - Total tasks
    - **Field** `completed: AtomicUsize` - Finished count
    - **Field** `results: Mutex<Vec<ProcessingResult>>`

**Key Functions:**
- `track_progress(tracker: Arc<ProgressTracker>)` - Progress reporter thread
- `wait_for_completion(tracker: Arc<ProgressTracker>) -> Vec<ProcessingResult>`

### Checkpoint Tests

```rust
#[test]
fn test_progress_tracking() {
    let tracker = Arc::new(ProgressTracker::new(10));

    for i in 0..10 {
        let t = tracker.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            t.record_completion(i, TaskStatus::Success, Duration::from_millis(10), None);
        });
    }

    let results = tracker.wait_for_completion();
    assert_eq!(results.len(), 10);
    assert_eq!(tracker.completed.load(Ordering::SeqCst), 10);
}

#[test]
fn test_error_collection() {
    let tracker = Arc::new(ProgressTracker::new(5));

    for i in 0..5 {
        let t = tracker.clone();
        thread::spawn(move || {
            if i % 2 == 0 {
                t.record_completion(i, TaskStatus::Success, Duration::from_millis(10), None);
            } else {
                t.record_completion(
                    i,
                    TaskStatus::Failed,
                    Duration::from_millis(5),
                    Some("Processing error".to_string())
                );
            }
        });
    }

    let results = tracker.wait_for_completion();
    let failed = results.iter().filter(|r| matches!(r.status, TaskStatus::Failed)).count();
    assert_eq!(failed, 2);
}
```

### Starter Code

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum TaskStatus {
    Success,
    Failed,
}

pub struct ProcessingResult {
    pub task_id: usize,
    pub status: TaskStatus,
    pub duration: Duration,
    pub error: Option<String>,
}

pub struct ProgressTracker {
    total: usize,
    completed: AtomicUsize,
    results: Mutex<Vec<ProcessingResult>>,
}

impl ProgressTracker {
    pub fn new(total: usize) -> Self {
        // TODO: Initialize with total count and empty results
        unimplemented!()
    }

    pub fn record_completion(
        &self,
        task_id: usize,
        status: TaskStatus,
        duration: Duration,
        error: Option<String>,
    ) {
        // TODO: Increment completed counter
        // Add result to results vec (with lock)
        unimplemented!()
    }

    pub fn progress_percentage(&self) -> f64 {
        // TODO: Calculate completion percentage
        // Hint: (completed / total) * 100.0
        unimplemented!()
    }

    pub fn wait_for_completion(&self) -> Vec<ProcessingResult> {
        // TODO: Spin until completed == total
        // Return cloned results vec
        unimplemented!()
    }
}

pub fn progress_reporter(tracker: Arc<ProgressTracker>) {
    // TODO: Loop printing progress every 100ms
    // Example: "Progress: 45/100 (45%)"
    // Exit when completed == total
    unimplemented!()
}
```

**Why previous Milestone is not enough:** No visibility into processing status. Users want progress bars and error reports.

**What's the improvement:** Progress tracking enables UX and debugging:
- No tracking: Black box, no idea if hung or processing
- With tracking: Real-time progress, failed task identification

For long-running batch jobs, progress reporting is mandatory.

---

## Milestone 4: Dynamic Task Submission

### Introduction

Support submitting tasks dynamically while processing continues. Add tasks from multiple threads without blocking.

### Architecture

**Enhanced Pool:**
- Allow task submission from any thread
- Handle varying load (elastic work queue)
- Report queue depth for monitoring

**Key Functions:**
- `execute_with_timeout(&self, f: Job, timeout: Duration) -> Result<(), TimeoutError>`
- `queue_depth(&self) -> usize` - Number of pending tasks

### Checkpoint Tests

```rust
#[test]
fn test_dynamic_submission() {
    let pool = ThreadPool::new(4);

    // Submit initial batch
    for i in 0..10 {
        pool.execute(move || println!("Task {}", i));
    }

    // Submit more tasks while processing
    thread::sleep(Duration::from_millis(50));
    for i in 10..20 {
        pool.execute(move || println!("Task {}", i));
    }

    pool.shutdown();
}

#[test]
fn test_multi_threaded_submission() {
    let pool = Arc::new(ThreadPool::new(4));
    let mut submitters = vec![];

    for _ in 0..4 {
        let p = pool.clone();
        let handle = thread::spawn(move || {
            for i in 0..100 {
                p.execute(move || {
                    thread::sleep(Duration::from_micros(10));
                });
            }
        });
        submitters.push(handle);
    }

    for h in submitters {
        h.join().unwrap();
    }
}
```

### Starter Code

```rust
impl ThreadPool {
    pub fn execute_with_timeout<F>(
        &self,
        f: F,
        timeout: Duration,
    ) -> Result<(), String>
    where
        F: FnOnce() + Send + 'static,
    {
        // TODO: Try to send job with timeout
        // Use sync_channel with timeout instead of regular channel
        // Return Err if send times out
        unimplemented!()
    }

    pub fn queue_depth(&self) -> usize {
        // TODO: Track pending tasks
        // Could use Arc<AtomicUsize> incremented on send, decremented on execute
        unimplemented!()
    }

    pub fn active_workers(&self) -> usize {
        // TODO: Track number of workers currently executing
        // Use Arc<AtomicUsize> incremented before execute, decremented after
        unimplemented!()
    }
}
```

**Why previous Milestone is not enough:** Static workload doesn't reflect reality. Real systems have dynamic, unpredictable task arrival.

**What's the improvement:** Dynamic submission enables real-world patterns:
- Web server: New requests arrive while processing existing
- Stream processing: Events arrive continuously
- Adaptive systems: Task generation based on results

---

## Milestone 5: Adaptive Pool Sizing

### Introduction

Automatically adjust worker count based on load. Scale up when queue grows, scale down when idle.

### Architecture

**Adaptive Logic:**
- Monitor queue depth and worker utilization
- Spawn workers if queue > threshold × current_workers
- Terminate idle workers after timeout

**Key Functions:**
- `scale_up(&mut self, count: usize)` - Add workers
- `scale_down(&mut self, count: usize)` - Remove workers
- `auto_scale(&self)` - Background thread monitoring and adjusting

### Checkpoint Tests

```rust
#[test]
fn test_scale_up() {
    let mut pool = ThreadPool::new(2);

    // Submit many tasks to trigger scaling
    for _ in 0..100 {
        pool.execute(|| thread::sleep(Duration::from_millis(10)));
    }

    thread::sleep(Duration::from_millis(50));

    // Pool should have scaled up
    assert!(pool.worker_count() > 2);
}

#[test]
fn test_scale_down() {
    let mut pool = ThreadPool::new(8);

    // Submit few tasks
    for _ in 0..4 {
        pool.execute(|| thread::sleep(Duration::from_millis(10)));
    }

    thread::sleep(Duration::from_secs(2)); // Wait for idle timeout

    // Pool should have scaled down
    assert!(pool.worker_count() < 8);
}
```

### Starter Code

```rust
pub struct AdaptiveThreadPool {
    workers: Arc<Mutex<Vec<JoinHandle<()>>>>,
    sender: Sender<Job>,
    min_workers: usize,
    max_workers: usize,
    queue_threshold: usize,
}

impl AdaptiveThreadPool {
    pub fn new(min: usize, max: usize) -> Self {
        // TODO: Initialize with min workers
        // Spawn monitoring thread for auto-scaling
        unimplemented!()
    }

    pub fn scale_up(&mut self, count: usize) {
        // TODO: Spawn 'count' new workers
        // Don't exceed max_workers
        unimplemented!()
    }

    pub fn scale_down(&mut self, count: usize) {
        // TODO: Signal 'count' workers to exit
        // Don't go below min_workers
        // Use special "exit" message in channel
        unimplemented!()
    }

    pub fn worker_count(&self) -> usize {
        self.workers.lock().unwrap().len()
    }
}

fn auto_scale_monitor(pool: Arc<AdaptiveThreadPool>) {
    // TODO: Loop checking queue depth
    // If queue_depth > threshold * workers: scale_up()
    // If workers idle for > 30s: scale_down()
    unimplemented!()
}
```

**Why previous Milestone is not enough:** Fixed pool size is inefficient. Overprovisioned when idle (waste resources), underprovisioned during peaks (high latency).

**What's the improvement:** Adaptive sizing optimizes resource usage:
- Fixed 100 workers: Wastes 95% resources during low load
- Adaptive 5-100 workers: Scales to load, saves resources

For cloud deployments, adaptive sizing reduces costs by 50-90%.

---

## Milestone 6: Benchmark vs Sequential

### Introduction

Benchmark thread pool against sequential processing. Measure speedup with varying worker counts and task sizes.

### Architecture

**Benchmarks:**
- Fixed workload (1000 tasks)
- Vary task duration: 1ms, 10ms, 100ms
- Vary worker count: 1, 2, 4, 8, 16
- Measure total time and tasks/sec

### Starter Code

```rust
pub struct Benchmark;

impl Benchmark {
    pub fn benchmark_sequential(num_tasks: usize, task_duration: Duration) -> Duration {
        let start = Instant::now();

        for _ in 0..num_tasks {
            thread::sleep(task_duration);
        }

        start.elapsed()
    }

    pub fn benchmark_thread_pool(
        num_tasks: usize,
        num_workers: usize,
        task_duration: Duration,
    ) -> Duration {
        let pool = ThreadPool::new(num_workers);
        let start = Instant::now();

        for _ in 0..num_tasks {
            pool.execute(move || {
                thread::sleep(task_duration);
            });
        }

        pool.shutdown();
        start.elapsed()
    }

    pub fn run_comparison() {
        println!("=== Thread Pool vs Sequential Performance ===\n");

        let num_tasks = 100;
        let task_duration = Duration::from_millis(10);
        let thread_counts = [1, 2, 4, 8];

        let seq_time = Self::benchmark_sequential(num_tasks, task_duration);
        println!("Sequential: {:?}\n", seq_time);

        for &num_threads in &thread_counts {
            let pool_time = Self::benchmark_thread_pool(num_tasks, num_threads, task_duration);
            let speedup = seq_time.as_secs_f64() / pool_time.as_secs_f64();

            println!("Thread Pool ({} workers):", num_threads);
            println!("  Time: {:?}", pool_time);
            println!("  Speedup: {:.2}x\n", speedup);
        }
    }
}
```

**Why previous Milestone is not enough:** Performance claims need validation.

**What's the improvement:** Empirical speedup data:
- 1 worker: 1× (same as sequential)
- 4 workers: 3.8-4× speedup
- 8 workers: 7-8× speedup

Validates parallel efficiency and guides worker count selection.

---

### Complete Working Example

```rust
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<JoinHandle<()>>,
    sender: Sender<Job>,
    shutdown: Arc<AtomicBool>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let (sender, receiver) = channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let shutdown = Arc::new(AtomicBool::new(false));

        let mut workers = Vec::with_capacity(size);

        for _ in 0..size {
            let receiver = receiver.clone();
            let shutdown = shutdown.clone();

            let handle = thread::spawn(move || {
                worker_loop(receiver, shutdown);
            });

            workers.push(handle);
        }

        ThreadPool {
            workers,
            sender,
            shutdown,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(f)).unwrap();
    }

    pub fn shutdown(self) {
        self.shutdown.store(true, Ordering::SeqCst);
        drop(self.sender);

        for worker in self.workers {
            worker.join().unwrap();
        }
    }
}

fn worker_loop(receiver: Arc<Mutex<Receiver<Job>>>, shutdown: Arc<AtomicBool>) {
    loop {
        let job = {
            let receiver = receiver.lock().unwrap();
            receiver.recv()
        };

        match job {
            Ok(job) => job(),
            Err(_) => {
                if shutdown.load(Ordering::SeqCst) {
                    break;
                }
            }
        }
    }
}

fn main() {
    println!("=== Thread Pool Demo ===\n");

    let pool = ThreadPool::new(4);
    let counter = Arc::new(AtomicUsize::new(0));

    println!("Submitting 100 tasks to thread pool with 4 workers...");

    let start = Instant::now();

    for i in 0..100 {
        let c = counter.clone();
        pool.execute(move || {
            // Simulate work
            thread::sleep(Duration::from_millis(10));
            c.fetch_add(1, Ordering::SeqCst);

            if i % 20 == 0 {
                println!("Task {} completed", i);
            }
        });
    }

    pool.shutdown();
    let elapsed = start.elapsed();

    println!("\nAll tasks completed!");
    println!("Total tasks: {}", counter.load(Ordering::SeqCst));
    println!("Time elapsed: {:?}", elapsed);
    println!("Throughput: {:.0} tasks/sec", 100.0 / elapsed.as_secs_f64());
}
```

### Testing Strategies

1. **Concurrency Tests**: Verify thread safety with ThreadSanitizer
2. **Load Tests**: 10K+ tasks, verify no deadlocks
3. **Shutdown Tests**: Clean termination under load
4. **Performance Tests**: Measure speedup vs sequential
5. **Stress Tests**: Rapid task submission from many threads

---

This project comprehensively demonstrates thread pool patterns, from basic implementation through dynamic submission, adaptive sizing, and performance benchmarks.

---