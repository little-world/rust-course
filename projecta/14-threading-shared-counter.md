
## Project 3: Shared Counter Service with Arc/Mutex

### Problem Statement

Build a multi-threaded counter service using shared state synchronization with Arc and Mutex. The service provides thread-safe increment, decrement, and query operations with high concurrency.

Your counter service should:
- Support concurrent increment/decrement from multiple threads
- Provide atomic read operations
- Track operation statistics (total operations, contention events)
- Optimize for read-heavy workloads using RwLock
- Implement deadlock-free complex operations
- Compare performance: Mutex vs RwLock vs Atomics

### Why It Matters

Shared state is unavoidable in many systems: caches, connection pools, metrics. Mutexes ensure safety but create contention. Understanding when to use Mutex vs RwLock vs Atomics is critical for performance.

For 1M operations with 8 threads:
- Naive Mutex: 500ms (serialized)
- RwLock (90% reads): 100ms (parallel reads)
- AtomicU64: 50ms (lock-free)

Critical for: metrics systems, caches, connection pools, resource managers.

### Use Cases

- Metrics and monitoring systems
- Rate limiters
- Connection pool managers
- Cache implementations
- Resource quota tracking
- Distributed counters

---

### Milestone 1: Basic Arc/Mutex Counter

### Introduction

Implement a thread-safe counter using Arc<Mutex<T>>. Multiple threads can safely increment/decrement through the mutex.

### Architecture

**Structs:**
- `Counter` - Thread-safe counter
    - **Field** `value: Arc<Mutex<i64>>` - Protected counter value

**Key Functions:**
- `new() -> Counter` - Create counter at 0
- `increment(&self)` - Add 1
- `decrement(&self)` - Subtract 1
- `get(&self) -> i64` - Read current value

**Role Each Plays:**
- Arc: Shared ownership across threads
- Mutex: Ensures exclusive access for modifications
- Lock guard: Automatic unlock when dropped

### Checkpoint Tests

```rust
#[test]
fn test_concurrent_increments() {
    let counter = Arc::new(Counter::new());
    let mut handles = vec![];

    for _ in 0..10 {
        let c = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                c.increment();
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(counter.get(), 10_000);
}

#[test]
fn test_mixed_operations() {
    let counter = Arc::new(Counter::new());

    let c1 = counter.clone();
    let h1 = thread::spawn(move || {
        for _ in 0..100 {
            c1.increment();
        }
    });

    let c2 = counter.clone();
    let h2 = thread::spawn(move || {
        for _ in 0..50 {
            c2.decrement();
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();

    assert_eq!(counter.get(), 50);
}
```

### Starter Code

```rust
use std::sync::{Arc, Mutex};

pub struct Counter {
    value: Arc<Mutex<i64>>,
}

impl Counter {
    pub fn new() -> Self {
        // TODO: Initialize with Arc<Mutex<i64>> at 0
        unimplemented!()
    }

    pub fn increment(&self) {
        // TODO: Lock mutex, increment value, unlock automatically
        // Hint: let mut val = self.value.lock().unwrap();
        //       *val += 1;
        unimplemented!()
    }

    pub fn decrement(&self) {
        // TODO: Lock mutex, decrement value
        unimplemented!()
    }

    pub fn get(&self) -> i64 {
        // TODO: Lock mutex, read value, return
        unimplemented!()
    }

    pub fn add(&self, amount: i64) {
        // TODO: Lock once and add amount
        unimplemented!()
    }
}

impl Clone for Counter {
    fn clone(&self) -> Self {
        Counter {
            value: self.value.clone(), // Clone Arc, not Mutex
        }
    }
}
```

**Why previous Milestone is not enough:** N/A - Foundation Milestone.

**What's the improvement:** Arc/Mutex provides safe shared state:
- Unsafe: `static mut COUNTER` - data races, undefined behavior
- Safe: `Arc<Mutex<T>>` - compiler-enforced mutual exclusion

For concurrent counters, Arc/Mutex is the safe default.

---

### Milestone 2: Contention Monitoring

### Introduction

Add metrics to track mutex contention: lock acquisition time, waiting threads, lock hold duration.

### Architecture

**Enhanced Counter:**
- Track lock wait times
- Count contention events (when lock is already held)
- Measure critical section duration

**Structs:**
- `ContentionStats` - Metrics
    - **Field** `total_locks: AtomicU64`
    - **Field** `contention_events: AtomicU64`
    - **Field** `total_wait_time_us: AtomicU64`

### Checkpoint Tests

```rust
#[test]
fn test_contention_tracking() {
    let counter = Arc::new(MonitoredCounter::new());
    let mut handles = vec![];

    // High contention workload
    for _ in 0..8 {
        let c = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                c.increment();
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    let stats = counter.stats();
    println!("Contention events: {}", stats.contention_events);
    println!("Avg wait time: {}μs", stats.avg_wait_time_us());
}
```

### Starter Code

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub struct ContentionStats {
    pub total_locks: AtomicU64,
    pub contention_events: AtomicU64,
    pub total_wait_time_us: AtomicU64,
}

impl ContentionStats {
    pub fn avg_wait_time_us(&self) -> u64 {
        let total = self.total_locks.load(Ordering::Relaxed);
        if total == 0 {
            0
        } else {
            self.total_wait_time_us.load(Ordering::Relaxed) / total
        }
    }
}

pub struct MonitoredCounter {
    value: Arc<Mutex<i64>>,
    stats: Arc<ContentionStats>,
}

impl MonitoredCounter {
    pub fn increment(&self) {
        let start = Instant::now();

        // Try to lock - if contended, record it
        let mut val = self.value.lock().unwrap();

        let wait_time = start.elapsed();
        self.stats.total_locks.fetch_add(1, Ordering::Relaxed);
        self.stats.total_wait_time_us.fetch_add(
            wait_time.as_micros() as u64,
            Ordering::Relaxed
        );

        if wait_time > Duration::from_micros(1) {
            self.stats.contention_events.fetch_add(1, Ordering::Relaxed);
        }

        *val += 1;
    }

    pub fn stats(&self) -> &ContentionStats {
        &self.stats
    }
}
```

**Why previous Milestone is not enough:** Can't optimize without measuring. Contention metrics reveal bottlenecks.

**What's the improvement:** Monitoring enables optimization:
- High contention → Use RwLock or sharding
- Long hold times → Reduce critical section
- Identify hotspots → Targeted optimization

---

### Milestone 3: RwLock for Read-Heavy Workloads

### Introduction

Optimize for read-heavy access patterns using RwLock. Multiple readers can access concurrently, writers get exclusive access.

### Architecture

**RwLock Semantics:**
- Multiple readers simultaneously (shared access)
- Single writer exclusively (exclusive access)
- Readers block writers, writers block everyone

**Comparison:**
- Mutex: All operations serialized
- RwLock: Reads parallel, writes exclusive

### Checkpoint Tests

```rust
#[test]
fn test_concurrent_reads() {
    let counter = Arc::new(RwCounter::new());

    // Set value
    counter.set(42);

    // Spawn many readers
    let mut handles = vec![];
    for _ in 0..10 {
        let c = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                assert_eq!(c.get(), 42);
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn test_read_write_mix() {
    let counter = Arc::new(RwCounter::new());

    // 9 readers, 1 writer
    let mut handles = vec![];

    for _ in 0..9 {
        let c = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                c.get();
            }
        });
        handles.push(handle);
    }

    let c = counter.clone();
    let writer = thread::spawn(move || {
        for _ in 0..100 {
            c.increment();
        }
    });

    handles.push(writer);

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(counter.get(), 100);
}
```

### Starter Code

```rust
use std::sync::RwLock;

pub struct RwCounter {
    value: Arc<RwLock<i64>>,
}

impl RwCounter {
    pub fn new() -> Self {
        // TODO: Initialize with Arc<RwLock<i64>>
        unimplemented!()
    }

    pub fn increment(&self) {
        // TODO: Acquire write lock, increment
        // Hint: let mut val = self.value.write().unwrap();
        //       *val += 1;
        unimplemented!()
    }

    pub fn get(&self) -> i64 {
        // TODO: Acquire read lock, return value
        // Hint: let val = self.value.read().unwrap();
        //       *val
        unimplemented!()
    }

    pub fn set(&self, new_value: i64) {
        // TODO: Acquire write lock, set value
        unimplemented!()
    }
}
```

**Why previous Milestone is not enough:** Mutex serializes all access, even reads. For read-heavy workloads (90%+ reads), this wastes concurrency.

**What's the improvement:** RwLock enables parallel reads:
- Mutex (90% reads): 1× throughput (all serialized)
- RwLock (90% reads): 8× throughput (reads parallel)

For caches and metrics, RwLock is often 5-10× faster.

---

### Milestone 4: Lock-Free with Atomics

### Introduction

Eliminate locks entirely using atomic operations. AtomicU64 provides lock-free increment/decrement with fetch_add.

### Architecture

**Atomic Operations:**
- `fetch_add`: Atomically add and return old value
- `fetch_sub`: Atomically subtract
- `load`: Read current value
- `store`: Write new value

**Memory Ordering:**
- `Relaxed`: No synchronization (fastest)
- `Acquire/Release`: Synchronizes with other operations
- `SeqCst`: Strongest guarantees (slowest)

### Checkpoint Tests

```rust
#[test]
fn test_atomic_counter() {
    let counter = Arc::new(AtomicCounter::new());
    let mut handles = vec![];

    for _ in 0..10 {
        let c = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..10000 {
                c.increment();
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(counter.get(), 100_000);
}

#[test]
fn test_atomic_performance() {
    let counter = Arc::new(AtomicCounter::new());
    let start = Instant::now();

    let mut handles = vec![];
    for _ in 0..8 {
        let c = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100_000 {
                c.increment();
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    let elapsed = start.elapsed();
    println!("Atomic: {}μs per op", elapsed.as_micros() / 800_000);
}
```

### Starter Code

```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct AtomicCounter {
    value: Arc<AtomicU64>,
}

impl AtomicCounter {
    pub fn new() -> Self {
        // TODO: Initialize with Arc<AtomicU64>
        unimplemented!()
    }

    pub fn increment(&self) {
        // TODO: Use fetch_add with Relaxed ordering
        // Hint: self.value.fetch_add(1, Ordering::Relaxed);
        unimplemented!()
    }

    pub fn decrement(&self) {
        // TODO: Use fetch_sub
        unimplemented!()
    }

    pub fn get(&self) -> u64 {
        // TODO: Use load
        unimplemented!()
    }

    pub fn add(&self, amount: u64) {
        self.value.fetch_add(amount, Ordering::Relaxed);
    }
}
```

**Why previous Milestone is not enough:** Even RwLock has overhead (syscalls, context switches). Atomics are lock-free and fastest.

**What's the improvement:** Atomics provide maximum throughput:
- Mutex: 2-5μs per operation
- RwLock: 1-3μs per operation (reads)
- Atomic: 0.01-0.1μs per operation (100× faster!)

For high-frequency counters (metrics, rate limiters), atomics are mandatory.

---

### Milestone 5: Deadlock Prevention

### Introduction

Implement complex operations safely without deadlocks. Use lock ordering, try_lock, and timeout patterns.

### Architecture

**Deadlock Scenarios:**
1. **Lock ordering**: Thread A locks M1→M2, Thread B locks M2→M1
2. **Nested locks**: Function calls itself, tries to reacquire same lock
3. **Circular wait**: A waits for B, B waits for C, C waits for A

**Prevention Strategies:**
- **Lock ordering**: Always acquire locks in consistent order
- **Try-lock**: Don't block, retry later if lock unavailable
- **Timeout**: Give up after deadline

### Checkpoint Tests

```rust
#[test]
fn test_transfer_no_deadlock() {
    let counter1 = Arc::new(Counter::new());
    let counter2 = Arc::new(Counter::new());

    counter1.add(100);
    counter2.add(50);

    let c1 = counter1.clone();
    let c2 = counter2.clone();
    let h1 = thread::spawn(move || {
        for _ in 0..100 {
            transfer(&c1, &c2, 1);
        }
    });

    let c1 = counter1.clone();
    let c2 = counter2.clone();
    let h2 = thread::spawn(move || {
        for _ in 0..100 {
            transfer(&c2, &c1, 1);
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();

    // Total should be conserved
    assert_eq!(counter1.get() + counter2.get(), 150);
}
```

### Starter Code

```rust
pub fn transfer(from: &Counter, to: &Counter, amount: i64) -> Result<(), String> {
    // TODO: Implement deadlock-free transfer
    // Strategy 1: Lock ordering - always lock lower address first
    // Strategy 2: Try-lock with retry
    // Strategy 3: Use global lock for multi-resource operations

    // Lock ordering approach:
    let (first, second) = if (from as *const Counter) < (to as *const Counter) {
        (from, to)
    } else {
        (to, from)
    };

    // TODO: Lock first, then second
    // Subtract from 'from', add to 'to'
    // Check balance before transfer

    unimplemented!()
}

pub fn try_transfer_with_timeout(
    from: &Counter,
    to: &Counter,
    amount: i64,
    timeout: Duration,
) -> Result<(), String> {
    // TODO: Use try_lock with timeout
    // Retry until timeout expires
    // Hint: Use Instant::now() and loop with try_lock()
    unimplemented!()
}
```

**Why previous Milestone is not enough:** Simple operations don't reveal deadlock risks. Complex operations (transfers, swaps) need careful design.

**What's the improvement:** Deadlock prevention ensures progress:
- No prevention: System hangs, requires restart
- With prevention: Operations always complete (or fail gracefully)

For production systems, deadlock freedom is mandatory.

---

### Milestone 6: Performance Comparison

### Introduction

Benchmark all approaches: Mutex vs RwLock vs Atomic. Measure throughput under different read/write ratios.

### Architecture

**Benchmarks:**
- Vary read/write ratio: 50/50, 70/30, 90/10, 99/1
- Vary thread count: 1, 2, 4, 8, 16
- Fixed workload: 1M operations

### Starter Code

```rust
pub struct Benchmark;

impl Benchmark {
    pub fn benchmark_mutex(num_ops: usize, num_threads: usize, read_ratio: f64) -> Duration {
        let counter = Arc::new(Counter::new());
        let start = Instant::now();

        let mut handles = vec![];
        for _ in 0..num_threads {
            let c = counter.clone();
            let ops_per_thread = num_ops / num_threads;
            let handle = thread::spawn(move || {
                for _ in 0..ops_per_thread {
                    if rand::random::<f64>() < read_ratio {
                        c.get();
                    } else {
                        c.increment();
                    }
                }
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().unwrap();
        }

        start.elapsed()
    }

    pub fn benchmark_rwlock(num_ops: usize, num_threads: usize, read_ratio: f64) -> Duration {
        // TODO: Similar to mutex but use RwCounter
        unimplemented!()
    }

    pub fn benchmark_atomic(num_ops: usize, num_threads: usize) -> Duration {
        // TODO: Use AtomicCounter (no read/write distinction)
        unimplemented!()
    }

    pub fn run_comparison() {
        println!("=== Synchronization Performance Comparison ===\n");

        let num_ops = 1_000_000;
        let num_threads = 8;

        for read_ratio in [0.5, 0.7, 0.9, 0.99] {
            println!("Read ratio: {:.0}%", read_ratio * 100.0);

            let mutex_time = Self::benchmark_mutex(num_ops, num_threads, read_ratio);
            let rwlock_time = Self::benchmark_rwlock(num_ops, num_threads, read_ratio);
            let atomic_time = Self::benchmark_atomic(num_ops, num_threads);

            println!("  Mutex:   {:?}", mutex_time);
            println!("  RwLock:  {:?} ({:.2}x)", rwlock_time, mutex_time.as_secs_f64() / rwlock_time.as_secs_f64());
            println!("  Atomic:  {:?} ({:.2}x)\n", atomic_time, mutex_time.as_secs_f64() / atomic_time.as_secs_f64());
        }
    }
}
```

**Why previous Milestone is not enough:** Need empirical data to choose synchronization primitive.

**What's the improvement:** Measured performance guides design:
- 50% reads: Mutex ≈ RwLock (frequent writes block readers)
- 90% reads: RwLock 5× faster than Mutex
- 99% reads: RwLock 10× faster, Atomic 100× faster

For high-contention read-heavy workloads, atomics provide orders of magnitude improvement.

---

### Complete Working Example

```rust
use std::sync::{Arc, Mutex, RwLock, atomic::{AtomicU64, Ordering}};
use std::thread;
use std::time::{Duration, Instant};

// Mutex-based counter
pub struct MutexCounter {
    value: Arc<Mutex<i64>>,
}

impl MutexCounter {
    pub fn new() -> Self {
        MutexCounter {
            value: Arc::new(Mutex::new(0)),
        }
    }

    pub fn increment(&self) {
        let mut val = self.value.lock().unwrap();
        *val += 1;
    }

    pub fn get(&self) -> i64 {
        *self.value.lock().unwrap()
    }
}

impl Clone for MutexCounter {
    fn clone(&self) -> Self {
        MutexCounter {
            value: self.value.clone(),
        }
    }
}

// RwLock-based counter
pub struct RwCounter {
    value: Arc<RwLock<i64>>,
}

impl RwCounter {
    pub fn new() -> Self {
        RwCounter {
            value: Arc::new(RwLock::new(0)),
        }
    }

    pub fn increment(&self) {
        let mut val = self.value.write().unwrap();
        *val += 1;
    }

    pub fn get(&self) -> i64 {
        *self.value.read().unwrap()
    }
}

impl Clone for RwCounter {
    fn clone(&self) -> Self {
        RwCounter {
            value: self.value.clone(),
        }
    }
}

// Atomic counter
pub struct AtomicCounter {
    value: Arc<AtomicU64>,
}

impl AtomicCounter {
    pub fn new() -> Self {
        AtomicCounter {
            value: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn increment(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

impl Clone for AtomicCounter {
    fn clone(&self) -> Self {
        AtomicCounter {
            value: self.value.clone(),
        }
    }
}

fn main() {
    println!("=== Shared Counter Service Demo ===\n");

    // Mutex counter
    println!("1. Mutex Counter:");
    let mutex_counter = Arc::new(MutexCounter::new());
    let mut handles = vec![];

    for i in 0..4 {
        let c = mutex_counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                c.increment();
            }
            println!("  Thread {} completed", i);
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    println!("Final count: {}\n", mutex_counter.get());

    // RwLock counter
    println!("2. RwLock Counter (read-heavy):");
    let rw_counter = Arc::new(RwCounter::new());
    let mut handles = vec![];

    // 1 writer
    let c = rw_counter.clone();
    let writer = thread::spawn(move || {
        for _ in 0..1000 {
            c.increment();
        }
    });
    handles.push(writer);

    // 10 readers
    for i in 0..10 {
        let c = rw_counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                let _ = c.get();
            }
            println!("  Reader {} completed", i);
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    println!("Final count: {}\n", rw_counter.get());

    // Atomic counter
    println!("3. Atomic Counter:");
    let atomic_counter = Arc::new(AtomicCounter::new());
    let mut handles = vec![];

    let start = Instant::now();

    for i in 0..8 {
        let c = atomic_counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100_000 {
                c.increment();
            }
            println!("  Thread {} completed", i);
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    let elapsed = start.elapsed();

    println!("Final count: {}", atomic_counter.get());
    println!("Time: {:?}", elapsed);
    println!("Throughput: {:.0} ops/sec", 800_000.0 / elapsed.as_secs_f64());
}
```

### Testing Strategies

1. **Correctness Tests**: Verify final counter values
2. **Concurrency Tests**: High thread count, verify no races
3. **Deadlock Tests**: Complex operations (transfer, swap)
4. **Performance Tests**: Compare Mutex/RwLock/Atomic
5. **Stress Tests**: 1M+ operations, sustained load

---

This project comprehensively demonstrates shared state synchronization patterns, from basic Mutex through RwLock optimization, lock-free atomics, deadlock prevention, and performance benchmarks comparing all approaches.

---

**All three Chapter 14 projects demonstrate:**
1. Message passing with channels (Project 1)
2. Thread pools for parallel work (Project 2)
3. Shared state synchronization (Project 3)

Each includes 6 progressive Milestones, checkpoint tests, starter code, complete working examples, and performance benchmarks.
