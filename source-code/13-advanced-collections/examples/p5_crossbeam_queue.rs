//! Pattern 5: Lock-Free Data Structures
//! Lock-Free Queue with Crossbeam
//!
//! Run with: cargo run --example p5_crossbeam_queue

use crossbeam::queue::{ArrayQueue, SegQueue};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

//===================
// Bounded MPMC queue
//===================
struct BoundedWorkQueue<T> {
    queue: Arc<ArrayQueue<T>>,
}

impl<T> BoundedWorkQueue<T> {
    fn new(capacity: usize) -> Self {
        Self {
            queue: Arc::new(ArrayQueue::new(capacity)),
        }
    }

    fn push(&self, item: T) -> Result<(), T> {
        self.queue.push(item)
    }

    fn pop(&self) -> Option<T> {
        self.queue.pop()
    }

    fn len(&self) -> usize {
        self.queue.len()
    }

    fn is_full(&self) -> bool {
        self.queue.is_full()
    }

    fn clone_handle(&self) -> Self {
        Self {
            queue: Arc::clone(&self.queue),
        }
    }
}

//=====================
// Unbounded MPMC queue
//=====================
struct UnboundedWorkQueue<T> {
    queue: Arc<SegQueue<T>>,
}

impl<T> UnboundedWorkQueue<T> {
    fn new() -> Self {
        Self {
            queue: Arc::new(SegQueue::new()),
        }
    }

    fn push(&self, item: T) {
        self.queue.push(item);
    }

    fn pop(&self) -> Option<T> {
        self.queue.pop()
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn clone_handle(&self) -> Self {
        Self {
            queue: Arc::clone(&self.queue),
        }
    }
}

//==================================================
// Real-world: Thread pool with lock-free task queue
//==================================================
type Task = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    task_queue: UnboundedWorkQueue<Task>,
    workers: Vec<thread::JoinHandle<()>>,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
}

impl ThreadPool {
    fn new(num_threads: usize) -> Self {
        use std::sync::atomic::AtomicBool;
        let task_queue: UnboundedWorkQueue<Task> = UnboundedWorkQueue::new();
        let shutdown = Arc::new(AtomicBool::new(false));
        let mut workers = Vec::new();

        for _id in 0..num_threads {
            let queue_clone = task_queue.clone_handle();
            let shutdown_clone = Arc::clone(&shutdown);

            workers.push(thread::spawn(move || {
                use std::sync::atomic::Ordering::Acquire;
                while !shutdown_clone.load(Acquire) {
                    if let Some(task) = queue_clone.pop() {
                        task();
                    } else {
                        thread::sleep(Duration::from_micros(100));
                    }
                }
            }));
        }

        Self {
            task_queue,
            workers,
            shutdown,
        }
    }

    fn execute<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.task_queue.push(Box::new(task));
    }

    fn shutdown(self) {
        use std::sync::atomic::Ordering::Release;
        self.shutdown.store(true, Release);

        for worker in self.workers {
            worker.join().unwrap();
        }
    }
}

//==============================
// Benchmark: Lock-free vs Mutex
//==============================
use std::sync::Mutex;

fn benchmark_lockfree_vs_mutex() {
    const ITEMS: usize = 100_000;
    const THREADS: usize = 4;

    // Lock-free queue
    println!("Lock-free queue:");
    let start = Instant::now();
    let lockfree_queue = UnboundedWorkQueue::new();

    let mut producers = vec![];
    for _ in 0..THREADS {
        let queue = lockfree_queue.clone_handle();
        producers.push(thread::spawn(move || {
            for i in 0..ITEMS {
                queue.push(i);
            }
        }));
    }

    let mut consumers = vec![];
    for _ in 0..THREADS {
        let queue = lockfree_queue.clone_handle();
        consumers.push(thread::spawn(move || {
            let mut count = 0;
            loop {
                if queue.pop().is_some() {
                    count += 1;
                    if count >= ITEMS {
                        break;
                    }
                }
            }
        }));
    }

    for p in producers {
        p.join().unwrap();
    }
    for c in consumers {
        c.join().unwrap();
    }

    let lockfree_time = start.elapsed();
    println!("  Time: {:?}", lockfree_time);

    // Mutex-based queue
    println!("\nMutex-based queue:");
    let start = Instant::now();
    let deque = std::collections::VecDeque::new();
    let mutex_queue = Arc::new(Mutex::new(deque));

    let mut producers = vec![];
    for _ in 0..THREADS {
        let queue = Arc::clone(&mutex_queue);
        producers.push(thread::spawn(move || {
            for i in 0..ITEMS {
                queue.lock().unwrap().push_back(i);
            }
        }));
    }

    let mut consumers = vec![];
    for _ in 0..THREADS {
        let queue = Arc::clone(&mutex_queue);
        consumers.push(thread::spawn(move || {
            let mut count = 0;
            loop {
                if queue.lock().unwrap().pop_front().is_some() {
                    count += 1;
                    if count >= ITEMS {
                        break;
                    }
                }
            }
        }));
    }

    for p in producers {
        p.join().unwrap();
    }
    for c in consumers {
        c.join().unwrap();
    }

    let mutex_time = start.elapsed();
    println!("  Time: {:?}", mutex_time);

    println!(
        "\nSpeedup: {:.2}x",
        mutex_time.as_secs_f64() / lockfree_time.as_secs_f64()
    );
}

fn main() {
    println!("=== Lock-Free Queue ===\n");

    let queue = UnboundedWorkQueue::new();

    // Producer thread
    let producer = queue.clone_handle();
    let p = thread::spawn(move || {
        for i in 0..1000 {
            producer.push(i);
        }
    });

    // Consumer threads
    let mut consumers = vec![];
    for _ in 0..3 {
        let consumer = queue.clone_handle();
        consumers.push(thread::spawn(move || {
            let mut sum = 0;
            while let Some(val) = consumer.pop() {
                sum += val;
            }
            sum
        }));
    }

    p.join().unwrap();

    let total: i32 = consumers
        .into_iter()
        .map(|h| h.join().unwrap())
        .sum();
    println!("Total consumed: {}", total);

    println!("\n=== Thread Pool ===\n");

    let pool = ThreadPool::new(4);

    for i in 0..10 {
        pool.execute(move || {
            println!("Task {} executing", i);
            thread::sleep(Duration::from_millis(100));
        });
    }

    thread::sleep(Duration::from_secs(2));
    pool.shutdown();

    println!("\n=== Performance Benchmark ===\n");
    benchmark_lockfree_vs_mutex();

    println!("\n=== Key Points ===");
    println!("1. Crossbeam provides production-ready lock-free queues");
    println!("2. ArrayQueue: bounded, faster for fixed capacity");
    println!("3. SegQueue: unbounded, grows dynamically");
    println!("4. 2-10x faster than mutex-based queues under contention");
}
