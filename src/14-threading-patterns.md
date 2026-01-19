# Threading Patterns

This chapter explores concurrent programming patterns in Rust using threads. We'll cover thread lifecycle management, parallel work distribution, message passing, shared state synchronization, and coordination primitives through practical, production-ready examples.

## Pattern 1: Thread Spawn and Join Patterns

This pattern is the foundation of multi-threaded programming in Rust. It covers how to create threads, transfer data to them, and get results back safely.

-   **Problem**: - **Underutilized CPU Cores**: A single-threaded program can't take advantage of multi-core processors, leaving expensive hardware idle. - **Data Ownership**: Moving data into a thread is tricky due to Rust's ownership rules.

-   **Solution**: - **`thread::spawn` with `move` Closures**: Use `thread::spawn` to create a new thread. The `move` keyword before the closure forces it to take ownership of the variables it captures, safely transferring them to the new thread.

-   **Why It Matters**: - **True Parallelism**: Threads allow your program to execute multiple operations at the same time on different CPU cores, dramatically improving performance for CPU-bound tasks. - **Compile-Time Safety**: Rust's ownership model prevents data races at compile time, one of the most common and difficult types of concurrency bugs.


### Example: Spawning a Thread with Owned Data

To move data into a thread, use a `move` closure. This transfers ownership of the `data` vector from the parent thread to the new thread. The parent can no longer access it, preventing data races. `thread::spawn` returns a `JoinHandle`, which we use to wait for the thread to finish and get its result.

```rust
use std::thread;

fn spawn_with_owned_data() {
    let data = vec![1, 2, 3, 4, 5];

    // The 'move' keyword transfers ownership of 'data' to the new thread.
    let handle = thread::spawn(move || {
        let sum: i32 = data.iter().sum();
        println!("Sum calculated by thread: {}", sum);
        sum // The thread returns the sum.
    });

    // The join() method waits for the thread to finish and returns a Result.
    let result = handle.join().unwrap();
    println!("Result received from thread: {}", result);

    // This would fail to compile, as 'data' has been moved:
    // println!("Data in main thread: {:?}", data);
}
```

### Example: Parallel Computations with Multiple Threads

You can spawn multiple threads to perform different computations in parallel. If they need to work on the same initial data, you must clone it to give each thread its own owned copy.

```rust
use std::thread;

fn parallel_computations() {
    let numbers = vec![1, 2, 3, 4, 5];

    // Clone data for the first thread.
    let numbers_clone1 = numbers.clone();
    let sum_handle = thread::spawn(move || {
        numbers_clone1.iter().sum::<i32>()
    });

    // Clone data for the second thread.
    let numbers_clone2 = numbers.clone();
    let product_handle = thread::spawn(move || {
        numbers_clone2.iter().product::<i32>()
    });

    // Wait for both threads to complete and collect their results.
    let sum = sum_handle.join().unwrap();
    let product = product_handle.join().unwrap();

    println!("Original data: {:?}", numbers);
    println!("Parallel Sum: {}, Parallel Product: {}", sum, product);
}
```

### Example: Handling Errors and Panics in Threads

The `join()` method returns a `Result`. An `Err` indicates the thread panicked. If the thread completes successfully, its own return value might also be a `Result`, so you often need to handle a nested `Result`.

```rust
use std::thread;

fn thread_with_error_handling() {
    let handle = thread::spawn(|| {
        // Simulate a computation that might fail.
        if rand::random::<bool>() {
            Ok(42)
        } else {
            Err("Computation failed in thread!")
        }
    });

    match handle.join() {
        Ok(Ok(value)) => {
            println!("Thread completed with value: {}", value)
        }
        Ok(Err(e)) => {
            println!("Thread returned error: {}", e)
        }
        Err(_) => println!("Thread panicked!"),
    }
}
```

### Example: Naming Threads for Better Debugging

For easier debugging, especially when you have many threads, you can give them names using the `thread::Builder`. The thread name will appear in panic messages and profiling tools.

```rust
use std::thread;
use std::time::Duration;

fn named_threads() {
    let handles: Vec<_> = (0..3)
        .map(|i| {
            thread::Builder::new()
                .name(format!("worker-{}", i))
                .spawn(move || {
                    let name = thread::current().name()
                        .unwrap_or("unnamed").to_string();
                    println!("Thread '{}' starting", name);
                    thread::sleep(Duration::from_millis(100));
                    println!("Thread '{}' finished", i);
                    i * 2
                })
                .unwrap()
        })
        .collect();

    let results: Vec<i32> = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect();

    println!("Results from named threads: {:?}", results);
}
```

### Example: Parallel File Processing

A real-world example of using threads to process multiple files in parallel. Each thread receives a file path, processes the file, and returns a result structure.

```rust
use std::thread;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct ProcessResult {
    path: PathBuf,
    line_count: usize,
    word_count: usize,
    byte_count: usize,
    error: Option<String>,
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

fn process_files_parallel(paths: Vec<PathBuf>) -> Vec<ProcessResult> {
    let handles: Vec<_> = paths
        .into_iter()
        .map(|path| {
            thread::spawn(move || {
                process_file(&path)
            })
        })
        .collect();

    handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect()
}
```

### Example: Borrowing Stack Data with Scoped Threads

A regular `thread::spawn` requires the closure to have a `'static` lifetime, meaning it cannot borrow data from the parent thread's stack. `thread::scope` solves this by guaranteeing that all threads spawned within the scope will finish before the scope ends, making it safe to borrow.

```rust
use std::thread;

fn scoped_threads_for_borrowing() {
    let mut data = vec![1, 2, 3, 4, 5];

    // 'thread::scope' creates a scope for spawning threads.
    // The scope guarantees that all threads within it will join before it exits.
    thread::scope(|s| {
        // This thread borrows 'data' immutably.
        s.spawn(|| {
            let sum: i32 = data.iter().sum();
            println!("Scoped thread sees sum: {}", sum);
        });

        // This thread also borrows 'data' immutably.
        s.spawn(|| {
            let product: i32 = data.iter().product();
            println!("Scoped thread sees product: {}", product);
        });
    }); // The scope blocks here until all spawned threads complete.

    // After the scope, we can mutate 'data' again.
    data.push(6);
    println!("After scope, data is: {:?}", data);
}
```

## Pattern 2: Thread Pools and Work Stealing

Spawning a new thread for every small task is inefficient. Thread pools and work-stealing are advanced patterns for managing a fixed set of worker threads to execute many tasks efficiently.

-   **Problem**: - **High Overhead**: Spawning thousands of OS threads for thousands of small tasks is slow and wastes memory. - **Resource Exhaustion**: An unbounded number of threads can exhaust system resources, leading to thrashing as the OS constantly switches between them.

-   **Solution**: - **Thread Pools**: Create a fixed number of worker threads (often equal to the number of CPU cores) and reuse them for multiple tasks. Tasks are submitted to a shared queue, and idle workers pull tasks from it.

-   **Why It Matters**: - **Efficiency**: Thread pools eliminate the overhead of thread creation, making it feasible to parallelize even small tasks. - **Automatic Load Balancing**: Work-stealing schedulers automatically distribute work, ensuring that all CPU cores are kept busy, leading to near-linear performance scaling for many parallel algorithms.


### Example: Data Parallelism with Rayon

Rayon is the de-facto standard for data parallelism in Rust. It provides a `par_iter()` method that turns a sequential iterator into a parallel one, automatically distributing the work across a work-stealing thread pool.

```rust
// Add `rayon = "1.8"` to Cargo.toml
use rayon::prelude::*;

fn parallel_map_reduce_with_rayon() {
    let numbers: Vec<i32> = (1..=1_000_000).collect();

    // Parallel sum
    let sum: i32 = numbers.par_iter().sum();
    println!("Parallel Sum (Rayon): {}", sum);

    // Parallel map
    let squares: Vec<i32> = numbers
        .par_iter()
        .map(|&x| x * x)
        .collect();
    println!("First 5 squares: {:?}", &squares[..5]);

    // Parallel filter and count
    let even_count = numbers
        .par_iter()
        .filter(|&&x| x % 2 == 0)
        .count();
    println!("Number of even numbers: {}", even_count);
}
```

### Example: Parallel Sorting with Rayon

Rayon also provides parallel implementations of common algorithms, like sorting. For large datasets, `par_sort()` can be significantly faster than the standard sequential sort.

```rust
// Add `rayon = "1.8"` to Cargo.toml
use rayon::prelude::*;

fn parallel_sorting_with_rayon() {
    let mut data: Vec<i32> = (0..1_000_000).rev().collect();
    println!("First 10 elements (before sort): {:?}", &data[..10]);

    // Parallel sort is much faster for large collections.
    data.par_sort();

    println!("First 10 elements (after sort): {:?}", &data[..10]);
}
```

### Example: Real-World - Parallel Image Processing

Rayon is perfect for tasks like image processing, where the same operation can be applied to millions of pixels independently. This example shows how to apply filters to an image in parallel.

```rust
// Add `rayon = "1.8"` to Cargo.toml
use rayon::prelude::*;

struct Image {
    pixels: Vec<u8>,
    width: usize,
    height: usize,
}

impl Image {
    fn apply_filter_parallel(&mut self, filter: impl Fn(u8) -> u8 + Sync) {
        self.pixels.par_iter_mut().for_each(|pixel| {
            *pixel = filter(*pixel);
        });
    }

    fn brighten(&mut self, amount: u8) {
        self.apply_filter_parallel(|p| p.saturating_add(amount));
    }
}
```

## Pattern 3: Message Passing with Channels

This pattern focuses on communication between threads by sending messages through channels, avoiding the complexities of shared memory and locks.

-   **Problem**: - **Complexity of Locks**: Using `Arc<Mutex<T>>` for every piece of shared data is verbose, and it's easy to make mistakes like holding a lock for too long (hurting performance) or causing deadlocks. - **Race Conditions**: Manually coordinating access to shared data is a major source of bugs that are hard to reproduce and debug.

-   **Solution**: - **Channels**: A channel provides a safe way to send data from one or more "producer" threads to one or more "consumer" threads. The channel handles all the necessary synchronization.

-   **Why It Matters**: - **Simplicity and Safety**: Channels transform a complex synchronization problem into a simple producer/consumer pattern. The type system ensures that you can't have data races.

-   **Use Cases**:
    -   **Producer-Consumer Pipelines**: A series of threads organized into stages, where each stage processes data and passes it to the next via a channel.
    -   **Event-Driven Architectures**: A central thread or task that receives events from multiple sources and dispatches them for processing.
    -   **Actor Systems**: A concurrency model where independent "actors" communicate with each other exclusively by sending messages.
    -   **Background Workers**: Offloading tasks like sending emails, logging, or processing analytics to a pool of workers that receive jobs via a channel.

### Example: Basic Producer-Consumer with MPSC Channel

`std::sync::mpsc` provides a "Multiple Producer, Single Consumer" channel. Here, one producer thread sends data to a single consumer thread.

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn basic_mpsc_channel() {
    let (tx, rx) = mpsc::channel(); // tx = transmitter, rx = receiver

    // Spawn a producer thread.
    thread::spawn(move || {
        for i in 0..5 {
            println!("Sending: {}", i);
            tx.send(i).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });

    // The receiver can be used as an iterator that blocks until a message is received.
    for received in rx {
        println!("Received: {}", received);
    }
}
```

### Example: Multiple Producers

The transmitter (`tx`) can be cloned to allow multiple threads to send messages to the same receiver.

```rust
use std::sync::mpsc;
use std::thread;

fn multiple_producers() {
    let (tx, rx) = mpsc::channel();

    for thread_id in 0..3 {
        // Clone the transmitter for each new thread.
        let tx_clone = tx.clone();
        thread::spawn(move || {
            for i in 0..3 {
                let msg = format!("Thread {} msg {}", thread_id, i);
                tx_clone.send(msg).unwrap();
            }
        });
    }

    // Drop the original transmitter so the receiver knows when to stop waiting.
    drop(tx);

    // The receiver will automatically close when all transmitters have been dropped.
    for received in rx {
        println!("Received: {}", received);
    }
}
```

### Example: Crossbeam Bounded Channel for Backpressure

The `crossbeam` crate offers more powerful channels. A "bounded" channel has a fixed capacity. If a fast producer tries to send to a full channel, it will block until a slow consumer makes space. This is called backpressure.

```rust
// Add `crossbeam = "0.8"` to Cargo.toml
use crossbeam::channel::bounded;
use std::thread;
use std::time::Duration;

fn bounded_channel_backpressure() {
    // A channel with a capacity of 2.
    let (tx, rx) = bounded(2);

    // A fast producer.
    thread::spawn(move || {
        for i in 0..10 {
            println!("Producer: trying to send {}", i);
            tx.send(i).unwrap(); // This will block if the channel is full.
            println!("Producer: sent {}", i);
        }
    });

    // A slow consumer.
    thread::sleep(Duration::from_secs(1));
    for _ in 0..10 {
        let value = rx.recv().unwrap();
        println!("Consumer: received {}", value);
        thread::sleep(Duration::from_millis(500));
    }
}
```

### Example: The Actor Pattern

The actor model is a concurrency pattern where "actors" are isolated entities that communicate exclusively through messages. This example implements a simple actor that maintains an internal state.

```rust
// Add `crossbeam = "0.8"` to Cargo.toml
use crossbeam::channel::{unbounded, Sender, Receiver, select};
use std::thread;

// Messages the actor can receive.
enum ActorMessage {
    Process(String),
    GetState(Sender<String>), // Message to request the actor's state.
    Shutdown,
}

// The actor itself.
struct Actor {
    inbox: Receiver<ActorMessage>,
    state: String,
}

impl Actor {
    fn new(inbox: Receiver<ActorMessage>) -> Self {
        Self { inbox, state: String::new() }
    }

    // The actor's main loop.
    fn run(mut self) {
        while let Ok(msg) = self.inbox.recv() {
            match msg {
                ActorMessage::Process(data) => {
                    self.state.push_str(&data);
                    println!("Actor state updated.");
                }
                ActorMessage::GetState(reply_to) => {
                    reply_to.send(self.state.clone()).unwrap();
                }
                ActorMessage::Shutdown => {
                    println!("Actor shutting down.");
                    break;
                }
            }
        }
    }
}

fn actor_pattern_example() {
    let (tx, rx) = unbounded();
    let actor_handle = thread::spawn(move || Actor::new(rx).run());

    // Send messages to the actor.
    tx.send(ActorMessage::Process("Hello, ".to_string())).unwrap();
    tx.send(ActorMessage::Process("Actor!".to_string())).unwrap();

    // Send a message to get the actor's state.
    let (reply_tx, reply_rx) = unbounded();
    tx.send(ActorMessage::GetState(reply_tx)).unwrap();
    let state = reply_rx.recv().unwrap();
    println!("Retrieved actor state: '{}'", state);

    // Shut down the actor.
    tx.send(ActorMessage::Shutdown).unwrap();
    actor_handle.join().unwrap();
}
```

## Pattern 4: Shared State with Locks (Mutex & RwLock)

While message passing is preferred, sometimes you need multiple threads to access the same piece of data. This pattern uses `Arc`, `Mutex`, and `RwLock` to share memory safely.

-   **Problem**: - **Ownership Conflicts**: Rust's ownership rules prevent you from having multiple mutable references to the same data, which is exactly what you need when threads share state. - **Data Races**: Unsynchronized access to shared data can lead to data races, where the final state depends on the non-deterministic order of thread execution, causing subtle and difficult-to-reproduce bugs.

-   **Solution**: - **`Arc<T>` (Atomically Reference-Counted Pointer)**: `Arc` is a smart pointer that lets multiple threads have shared ownership of the same data. It keeps a count of active references, and when the last reference is dropped, the data is cleaned up.

-   **Why It Matters**: - **Compile-Time Safety**: Rust's type system ensures you use locks correctly. You cannot access the data without first acquiring the lock, and you cannot forget to release it, preventing entire classes of concurrency bugs.

### Example: Shared Counter with `Arc<Mutex<T>>`

This is the "Hello, World!" of shared state concurrency. Multiple threads increment a shared counter, using a `Mutex` to ensure that the increments don't interfere with each other.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn shared_counter() {
    // Arc<Mutex<T>> is the standard way to share mutable state.
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        // Clone the Arc to give each thread a reference to the Mutex.
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            // lock() acquires the mutex, blocking until it's available.
            // The returned "lock guard" provides access to the data.
            let mut num = counter_clone.lock().unwrap();
            *num += 1;
            // The lock is automatically released when 'num' goes out of scope.
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", *counter.lock().unwrap());
}
```

### Example: Read-Heavy Workloads with `RwLock`

An `RwLock` is ideal when data is read frequently but written to infrequently. It allows unlimited concurrent readers but only a single writer.

```rust
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

fn rwlock_for_read_heavy_data() {
    let config = Arc::new(RwLock::new("initial_config".to_string()));
    let mut handles = vec![];

    // Spawn multiple reader threads.
    for i in 0..5 {
        let config_clone = Arc::clone(&config);
        let handle = thread::spawn(move || {
            // read() acquires a read lock. Multiple threads can hold a read lock.
            let cfg = config_clone.read().unwrap();
            println!("Reader {}: Current config is '{}'", i, *cfg);
        });
        handles.push(handle);
    }

    // Wait a moment, then spawn a writer thread.
    thread::sleep(Duration::from_millis(10));
    let config_clone = Arc::clone(&config);
    let writer_handle = thread::spawn(move || {
        // write() acquires a write lock. This will wait until all read locks are released.
        // No new readers can acquire a lock while the writer is waiting.
        let mut cfg = config_clone.write().unwrap();
        *cfg = "updated_config".to_string();
        println!("Writer: Updated config.");
    });
    handles.push(writer_handle);

    for handle in handles {
        handle.join().unwrap();
    }
    println!("Final config: {}", *config.read().unwrap());
}
```

## Pattern 5: Synchronization Primitives (Barrier & Condvar)

Barriers and Condvars are lower-level primitives used to coordinate the timing of thread execution.

-   **Problem**: - **Phased Execution**: Some parallel algorithms require all threads to complete a certain phase of work before any thread can move on to the next phase. - **Inefficient Waiting**: A thread might need to wait for a specific condition to become true (e.g., for a queue to become non-empty).

-   **Solution**: - **`Barrier`**: A `Barrier` is created with a count. Threads that reach the barrier will call `.wait()` and block.

-   **Why It Matters**: - **Efficiency**: `Condvar` avoids busy-waiting, leading to much better CPU utilization. Threads that are waiting for a condition consume no CPU resources.

### Example: `Barrier` for Phased Computation

A `Barrier` is used to synchronize multiple threads at a specific point. All threads must reach the barrier before any of them can proceed.

```rust
use std::sync::{Arc, Barrier};
use std::thread;

fn barrier_for_phased_work() {
    let num_threads = 4;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];

    for id in 0..num_threads {
        let barrier_clone = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            println!("Thread {}: Performing phase 1", id);
            // ... do some work ...
            barrier_clone.wait(); // All threads wait here.

            println!("Thread {}: Phase 1 done. Starting phase 2.", id);
            // ... do some work ...
            barrier_clone.wait(); // Wait again.

            println!("Thread {}: Phase 2 done.", id);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

### Example: `Condvar` for a Bounded Queue

This example shows how to use a `Condvar` and a `Mutex` to build a thread-safe bounded queue. Producers wait if the queue is full, and consumers wait if it's empty.

```rust
use std::sync::{Arc, Mutex, Condvar};
use std::collections::VecDeque;
use std::thread;

struct BoundedQueue<T> {
    queue: Mutex<VecDeque<T>>,
    condvar: Condvar,
    capacity: usize,
}

impl<T> BoundedQueue<T> {
    fn new(capacity: usize) -> Self {
        Self {
            queue: Mutex::new(VecDeque::with_capacity(capacity)),
            condvar: Condvar::new(),
            capacity,
        }
    }

    fn push(&self, item: T) {
        let mut queue = self.queue.lock().unwrap();
        // Wait while the queue is full.
        while queue.len() >= self.capacity {
            queue = self.condvar.wait(queue).unwrap();
        }
        queue.push_back(item);
        // Notify one waiting consumer that there's new data.
        self.condvar.notify_one();
    }

    fn pop(&self) -> T {
        let mut queue = self.queue.lock().unwrap();
        // Wait while the queue is empty.
        while queue.is_empty() {
            queue = self.condvar.wait(queue).unwrap();
        }
        let item = queue.pop_front().unwrap();
        // Notify one waiting producer that there's new space.
        self.condvar.notify_one();
        item
    }
}

fn condvar_for_queue() {
    let queue = Arc::new(BoundedQueue::new(3));

    // Producer thread
    let queue_clone_p = Arc::clone(&queue);
    let producer = thread::spawn(move || {
        for i in 0..10 {
            println!("Producer: pushing {}", i);
            queue_clone_p.push(i);
        }
    });

    // Consumer thread
    let queue_clone_c = Arc::clone(&queue);
    let consumer = thread::spawn(move || {
        for _ in 0..10 {
            let item = queue_clone_c.pop();
            println!("Consumer: popped {}", item);
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
```