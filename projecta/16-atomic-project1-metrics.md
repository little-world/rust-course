
# Lock-Free Metrics Collector

### Problem Statement

Build a lock-free metrics collection system that tracks application performance statistics from multiple threads without using mutexes or locks. The system should collect counters (requests served, errors), gauges (active connections, memory usage), and histograms (response times) with minimal contention and overhead.

The metrics collector must handle concurrent updates from hundreds of threads while allowing periodic snapshots for monitoring dashboards, without blocking writers or causing data races.

### Use Cases

- High-throughput web servers tracking request metrics
- Database connection pools monitoring active connections
- Real-time trading systems recording transaction latencies
- Game servers tracking player statistics
- Microservices exporting Prometheus/StatsD metrics
- Performance monitoring in hot paths where locks are too expensive

### Why It Matters

Locks create contention bottlenecks in high-concurrency scenarios. When 100 threads increment a mutex-protected counter, they serialize—only one thread proceeds while 99 wait. This destroys parallelism.

Atomics provide lock-free progress guarantees:
- **Lock-free**: At least one thread always makes progress (no deadlocks)
- **Wait-free**: Every thread makes progress in bounded time (strongest guarantee)
- **Obstruction-free**: Thread makes progress if it runs in isolation

Memory ordering matters for performance and correctness:
```
Relaxed: ~1-2 CPU cycles (no synchronization overhead)
Acquire/Release: ~10-20 cycles (cross-thread visibility)
SeqCst: ~30-50 cycles (total ordering across all threads)
```

For a counter incremented 1 million times/sec, using SeqCst vs Relaxed costs ~30-50 million extra cycles/sec.

Real-world impact: Prometheus client library uses atomics for metrics collection, enabling millions of observations/sec with negligible overhead. Mutex-based approach would cause 10-100x slowdown under contention.

---

## Key Concepts Explained

This project requires understanding atomic operations, memory ordering, and lock-free programming. These concepts enable building high-performance concurrent data structures that avoid the overhead and contention of traditional locks.

### What Are Atomic Operations?

**Definition**: Operations that execute as a single, indivisible unit—they either complete entirely or don't happen at all. No other thread can observe intermediate states.

**The Problem Atomics Solve**:

```rust
// NON-ATOMIC (Race condition):
static mut COUNTER: usize = 0;

fn increment() {
    unsafe {
        let temp = COUNTER;      // Thread A reads 0
        // Context switch here!
        // Thread B reads 0, increments to 1, writes 1
        COUNTER = temp + 1;      // Thread A writes 1 (should be 2!)
    }
}

// Two increments → COUNTER = 1 (WRONG! Lost update)
```

**The Atomic Solution**:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn increment() {
    COUNTER.fetch_add(1, Ordering::SeqCst);  // Atomic: no race
}

// Two increments → COUNTER = 2 (CORRECT!)
```

**How Atomics Work (Hardware Level)**:

Modern CPUs provide atomic instructions:
```assembly
# x86-64 LOCK prefix makes instruction atomic
lock incq (%rax)      # Atomically increment memory at address rax

# ARM64 Load-Exclusive/Store-Exclusive
1: ldxr x1, [x0]      # Load-exclusive (mark cache line)
   add  x1, x1, #1    # Increment
   stxr w2, x1, [x0]  # Store-exclusive (fails if cache line changed)
   cbnz w2, 1b        # Retry if store failed
```

**Key Point**: Atomics use hardware support (cache coherency protocols, memory barriers) to ensure operations are indivisible even across CPU cores.

---

### Atomic Types in Rust

Rust provides atomic versions of primitive types:

```rust
use std::sync::atomic::*;

// Integer atomics
AtomicU8, AtomicU16, AtomicU32, AtomicU64, AtomicUsize
AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize

// Boolean atomic
AtomicBool

// Pointer atomic
AtomicPtr<T>
```

**Common Operations**:

```rust
let counter = AtomicUsize::new(0);

// Load (read)
let value = counter.load(Ordering::SeqCst);

// Store (write)
counter.store(42, Ordering::SeqCst);

// Fetch-and-modify (read-modify-write)
let old = counter.fetch_add(5, Ordering::SeqCst);    // old = 42, counter = 47
let old = counter.fetch_sub(3, Ordering::SeqCst);    // old = 47, counter = 44
let old = counter.swap(100, Ordering::SeqCst);       // old = 44, counter = 100

// Compare-and-swap (CAS) - foundation of lock-free algorithms
let result = counter.compare_exchange(
    100,                    // Expected current value
    200,                    // New value to write
    Ordering::SeqCst,       // Success ordering
    Ordering::SeqCst,       // Failure ordering
);
// If counter == 100: counter becomes 200, returns Ok(100)
// If counter != 100: counter unchanged, returns Err(actual_value)
```

**Why Different Types?**:

Size matters for cache line packing:
```rust
struct Metrics {
    requests: AtomicU64,     // 8 bytes
    errors: AtomicU32,       // 4 bytes
    warnings: AtomicU16,     // 2 bytes
    flags: AtomicU8,         // 1 byte
    // Total: 15 bytes (fits in one 64-byte cache line)
}

// vs.

struct WastefulMetrics {
    requests: AtomicUsize,   // 8 bytes
    errors: AtomicUsize,     // 8 bytes (wasted 4 bytes!)
    warnings: AtomicUsize,   // 8 bytes (wasted 6 bytes!)
    flags: AtomicUsize,      // 8 bytes (wasted 7 bytes!)
    // Total: 32 bytes (wasted 17 bytes)
}
```

---

### Memory Ordering: The Critical Detail

**The Challenge**: Modern CPUs and compilers reorder memory operations for performance. Atomics need to control this reordering.

**Example of Reordering**:

```rust
// Thread A
x.store(1, Ordering::Relaxed);
y.store(1, Ordering::Relaxed);

// Thread B
let b = y.load(Ordering::Relaxed);
let a = x.load(Ordering::Relaxed);

// Possible outcome: b = 1, a = 0 (y write seen before x write!)
// CPU reordered the stores or loads
```

**The Five Memory Orderings**:

#### 1. Relaxed - No Synchronization

```rust
counter.fetch_add(1, Ordering::Relaxed);
```

**Guarantees**:
- ✅ Operation itself is atomic (no torn reads/writes)
- ❌ NO ordering guarantees with other operations
- ❌ NO cross-thread visibility guarantees (except eventual consistency)

**Use case**: Simple counters where only final value matters, not order

**Performance**: Fastest (~1-2 cycles)

**Example**:
```rust
// Thread A
REQUESTS.fetch_add(1, Ordering::Relaxed);
BYTES.fetch_add(1024, Ordering::Relaxed);

// Thread B
let r = REQUESTS.load(Ordering::Relaxed);
let b = BYTES.load(Ordering::Relaxed);
// Possible: r and b are inconsistent (one updated, not the other yet)
// That's OK for metrics! Eventually consistent.
```

#### 2. Acquire - Read Synchronization

```rust
let value = flag.load(Ordering::Acquire);
```

**Guarantees**:
- ✅ All operations AFTER this load cannot move BEFORE it
- ✅ If another thread used Release, see all its prior writes

**Use case**: Reading a flag/pointer set by another thread

**Example**:
```rust
// Thread A (producer)
DATA.store(42, Ordering::Relaxed);
READY.store(true, Ordering::Release);  // Ensures DATA write visible

// Thread B (consumer)
if READY.load(Ordering::Acquire) {     // Synchronizes with Release
    let data = DATA.load(Ordering::Relaxed);  // Guaranteed to see 42
    assert_eq!(data, 42);  // Always passes
}
```

#### 3. Release - Write Synchronization

```rust
flag.store(true, Ordering::Release);
```

**Guarantees**:
- ✅ All operations BEFORE this store cannot move AFTER it
- ✅ Makes all prior writes visible to threads that Acquire this value

**Use case**: Publishing data for other threads

**Pair with Acquire**: Release-Acquire creates synchronization edge

#### 4. AcqRel - Combined Acquire + Release

```rust
let old = counter.fetch_add(1, Ordering::AcqRel);
```

**Guarantees**:
- ✅ Acquire semantics for the read part
- ✅ Release semantics for the write part

**Use case**: Read-modify-write operations in lock-free algorithms

#### 5. SeqCst - Sequential Consistency

```rust
counter.fetch_add(1, Ordering::SeqCst);
```

**Guarantees**:
- ✅ All SeqCst operations have a single total order across all threads
- ✅ Strongest guarantee, easiest to reason about
- ❌ Slowest (~30-50 cycles due to memory fences)

**Use case**: When in doubt, start with SeqCst; optimize later

**Example showing difference**:
```rust
// With SeqCst: Total order guaranteed
// Thread A: X.store(1, SeqCst); Y.store(1, SeqCst);
// Thread B: a = Y.load(SeqCst); b = X.load(SeqCst);
// Thread C: c = X.load(SeqCst); d = Y.load(SeqCst);
// Impossible: a=1,b=0,c=1,d=0 (would violate total order)

// With Relaxed: This outcome IS possible (no total order)
```

**Visual Summary**:

```
Ordering:     Relaxed   Acquire   Release   AcqRel   SeqCst
Speed:        Fastest   ------>   ------>   ------>  Slowest
Guarantees:   Minimal   ------>   ------>   ------>  Strongest
Reordering:   Most      ------>   ------>   ------>  None
Use case:     Counters  Locks     Locks     RMW      Default
```

---

### Compare-and-Swap (CAS): The Lock-Free Primitive

**CAS** is the foundation of lock-free algorithms. It atomically performs:
```
if current_value == expected:
    current_value = new
    return success
else:
    return failure (with actual current value)
```

**Two Variants**:

**1. compare_exchange - Strong CAS**:
```rust
let result = counter.compare_exchange(
    expected,           // What I think the value is
    new,                // What I want it to be
    Ordering::SeqCst,   // Ordering if successful
    Ordering::SeqCst,   // Ordering if failed
);

match result {
    Ok(old_value) => println!("Success! Was {}", old_value),
    Err(actual) => println!("Failed! Actually {}", actual),
}
```

**2. compare_exchange_weak - Weak CAS**:
```rust
// May spuriously fail even if values match (hardware limitation on some platforms)
// Must use in a loop
loop {
    let current = counter.load(Ordering::Relaxed);
    let new = current + 1;

    match counter.compare_exchange_weak(
        current,
        new,
        Ordering::Relaxed,
        Ordering::Relaxed,
    ) {
        Ok(_) => break,  // Success
        Err(_) => continue,  // Retry (spurious or actual failure)
    }
}
```

**When to use each**:
- **Strong CAS**: One-shot attempts, complex retry logic
- **Weak CAS**: Always in loops (faster on some architectures like ARM)

**Lock-Free Stack Example**:

```rust
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;

struct Node<T> {
    value: T,
    next: *mut Node<T>,
}

struct LockFreeStack<T> {
    head: AtomicPtr<Node<T>>,
}

impl<T> LockFreeStack<T> {
    fn push(&self, value: T) {
        let new_node = Box::into_raw(Box::new(Node {
            value,
            next: ptr::null_mut(),
        }));

        loop {
            // Read current head
            let old_head = self.head.load(Ordering::Relaxed);

            // Point new node to current head
            unsafe { (*new_node).next = old_head; }

            // Try to swing head to new node
            match self.head.compare_exchange_weak(
                old_head,
                new_node,
                Ordering::Release,  // Success: publish new node
                Ordering::Relaxed,  // Failure: retry
            ) {
                Ok(_) => break,  // Success!
                Err(_) => continue,  // Another thread modified head, retry
            }
        }
    }
}
```

**Why CAS Works**:
1. Multiple threads can attempt push simultaneously
2. Only one CAS succeeds per head change
3. Failed threads retry with updated head value
4. No locks, no blocking—always progress

---

### Lock-Free vs Wait-Free vs Blocking

**Progress Guarantees** (from weakest to strongest):

#### Blocking (Locks/Mutexes):
```rust
let mut data = mutex.lock().unwrap();
data.counter += 1;
// If thread holding lock dies → deadlock
// If thread holding lock is slow → all waiters blocked
```

**Guarantee**: None (thread can be permanently blocked)

**Example**: `Mutex<T>`, `RwLock<T>`

#### Obstruction-Free:
**Guarantee**: Thread makes progress if it runs without interference

**Rarely used in practice** (too weak)

#### Lock-Free:
```rust
loop {
    let current = counter.load(Ordering::Relaxed);
    let new = current + 1;

    if counter.compare_exchange_weak(
        current, new,
        Ordering::Relaxed,
        Ordering::Relaxed,
    ).is_ok() {
        break;
    }
}
```

**Guarantee**: At least one thread always makes progress (system-wide)

**Tradeoff**: Individual thread might retry many times (but won't deadlock)

**Example**: Most CAS-based algorithms, `Arc<T>`

#### Wait-Free:
```rust
counter.fetch_add(1, Ordering::Relaxed);  // Never retries
```

**Guarantee**: Every thread makes progress in bounded steps

**Tradeoff**: Hardest to implement for complex data structures

**Example**: `fetch_add`, `fetch_sub`, simple atomic operations

**Comparison Table**:

| Property | Blocking | Lock-Free | Wait-Free |
|----------|----------|-----------|-----------|
| **Can deadlock?** | Yes | No | No |
| **Can starve?** | Yes | Individual threads, yes | No |
| **System progress?** | No guarantee | Always | Always |
| **Per-thread progress?** | No guarantee | No guarantee | Always |
| **Complexity** | Simple | Medium | Hard |
| **Performance** | Good (low contention) | Excellent | Excellent |

---

### False Sharing: The Hidden Performance Killer

**The Problem**: Different threads updating different variables can still contend if those variables share a cache line.

**Cache Line Size**: 64 bytes on most modern CPUs

**Example of False Sharing**:

```rust
struct BadMetrics {
    thread1_counter: AtomicUsize,  // Bytes 0-7
    thread2_counter: AtomicUsize,  // Bytes 8-15  ← Same cache line!
    thread3_counter: AtomicUsize,  // Bytes 16-23 ← Same cache line!
    thread4_counter: AtomicUsize,  // Bytes 24-31 ← Same cache line!
}

// Thread 1 increments thread1_counter
// → Entire cache line marked modified on Thread 1's core
// → Thread 2's cache line invalidated
// → Thread 2 must reload cache line to increment thread2_counter
// → Thread 1's cache line invalidated
// → Ping-pong continues → 10-100x slowdown!
```

**The Solution: Padding**:

```rust
use std::sync::atomic::AtomicUsize;

#[repr(align(64))]  // Force 64-byte alignment
struct PaddedAtomic {
    value: AtomicUsize,
    _padding: [u8; 64 - 8],  // Pad to 64 bytes
}

struct GoodMetrics {
    thread1_counter: PaddedAtomic,  // Bytes 0-63 (cache line 0)
    thread2_counter: PaddedAtomic,  // Bytes 64-127 (cache line 1)
    thread3_counter: PaddedAtomic,  // Bytes 128-191 (cache line 2)
    thread4_counter: PaddedAtomic,  // Bytes 192-255 (cache line 3)
}

// Now each counter in its own cache line
// No invalidation ping-pong
// Full parallel performance
```

**Performance Impact**:

```
Without padding (false sharing):
4 threads, 1M increments each: 2000ms (cache line bouncing)

With padding (separate cache lines):
4 threads, 1M increments each: 250ms (8x faster!)
```

**When to Use Padding**:
- ✅ High-contention atomic updates from different threads
- ✅ Per-thread metrics that are updated frequently
- ❌ Low-contention scenarios (wastes memory)
- ❌ Read-mostly data (false sharing only affects writes)

---

### Relaxed Ordering for Metrics: When It's Safe

**Key Insight**: Metrics collection has unique properties that make Relaxed ordering safe:

1. **Commutative**: Order of increments doesn't matter (1+2+3 = 3+1+2)
2. **Eventually consistent**: Okay if readers see slightly stale values
3. **No causality**: Counter A doesn't depend on counter B's value

**Example: Safe Relaxed Usage**:

```rust
struct Metrics {
    requests: AtomicU64,
    errors: AtomicU64,
    bytes_sent: AtomicU64,
}

impl Metrics {
    fn record_request(&self, bytes: u64, is_error: bool) {
        // All Relaxed: safe because operations are independent
        self.requests.fetch_add(1, Ordering::Relaxed);
        if is_error {
            self.errors.fetch_add(1, Ordering::Relaxed);
        }
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }

    fn snapshot(&self) -> MetricsSnapshot {
        // Relaxed reads: values may be inconsistent snapshot
        // But that's OK for monitoring dashboards
        MetricsSnapshot {
            requests: self.requests.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
        }
    }
}
```

**Why This Works**:
- Dashboard reads snapshot at time T
- Some updates from time T-1ms might not be visible yet
- Some updates from time T+1ms might already be visible
- But **all updates are eventually visible**
- Totals are accurate over time

**When Relaxed is NOT Safe**:

```rust
// WRONG: Using Relaxed for synchronization
if READY.load(Ordering::Relaxed) {  // ❌ Need Acquire
    let data = DATA.load(Ordering::Relaxed);  // Might not see DATA update!
}

// CORRECT: Use Acquire-Release
if READY.load(Ordering::Acquire) {  // ✅
    let data = DATA.load(Ordering::Relaxed);  // Guaranteed to see DATA
}
```

**Rules of Thumb**:
- ✅ Relaxed for independent counters/metrics
- ✅ Relaxed for statistics where slight inconsistency is acceptable
- ❌ Relaxed for flags/pointers that protect other data
- ❌ Relaxed for operations with cross-variable dependencies

---

### Connection to This Project

Now that you understand the core concepts, here's how they map to the milestones:

**Milestone 1: Basic Atomic Counter**
- **Concepts Used**: `AtomicUsize`, `fetch_add`, `load/store`, SeqCst ordering
- **Why**: Establish foundation of atomic operations and memory ordering
- **Key Insight**: SeqCst is easiest to reason about but has performance cost

**Milestone 2: Multiple Metric Types**
- **Concepts Used**: Multiple atomic types (counter, gauge), Relaxed ordering
- **Why**: Real metrics systems track different kinds of measurements
- **Key Insight**: Relaxed ordering is safe for independent metrics (10-30x faster than SeqCst)

**Milestone 3: Histogram with Buckets**
- **Concepts Used**: Array of atomics, bucketing algorithm, cache line awareness
- **Why**: Latency distributions need bucketed measurements
- **Key Insight**: Padding prevents false sharing between bucket counters

**Milestone 4: Thread-Local Aggregation**
- **Concepts Used**: Thread-local storage, periodic aggregation, reduced contention
- **Why**: Per-thread counters eliminate contention entirely
- **Key Insight**: Aggregate infrequently to balance memory vs synchronization cost

**Milestone 5: Lock-Free Snapshot**
- **Concepts Used**: Atomic loads for consistent snapshots, generation counters
- **Why**: Export metrics without blocking writers
- **Key Insight**: Relaxed loads give eventually-consistent snapshot without coordination

**Milestone 6: High-Resolution Timestamps**
- **Concepts Used**: Atomic timestamps, CAS for concurrent updates, temporal ordering
- **Why**: Track when metrics change for time-series analysis
- **Key Insight**: CAS enables lock-free timestamp updates with conflict resolution

**Putting It All Together**:

The complete metrics collector demonstrates:
1. **Lock-free counters** using `fetch_add` (wait-free)
2. **Memory ordering optimization** (Relaxed for metrics, Acquire/Release for synchronization)
3. **False sharing mitigation** (padding hot counters)
4. **Hybrid approach** (thread-local + periodic aggregation)
5. **Snapshot consistency** (atomic reads without blocking writers)

This architecture achieves:
- **Millions of updates/second** per thread with ~2ns overhead
- **Zero contention** in common case (thread-local)
- **Lock-free reads** for monitoring dashboards
- **Minimal memory** compared to per-thread histograms

Each milestone builds practical understanding of atomic operations, from simple counters to production-ready metrics infrastructure.

---

## Milestone 1: Basic Atomic Counter

### Introduction

Implement a thread-safe counter using `AtomicUsize` with `fetch_add`. This establishes understanding of atomic operations and memory ordering. Start with `SeqCst` ordering (strongest, simplest) before optimizing.

### Architecture

**Structs:**
- `AtomicCounter` - Thread-safe counter
  - **Field** `count: AtomicUsize` - The counter value
  - **Function** `new() -> Self` - Create counter initialized to 0
  - **Function** `increment(&self)` - Add 1 to counter
  - **Function** `add(&self, value: usize)` - Add arbitrary value
  - **Function** `get(&self) -> usize` - Read current value
  - **Function** `reset(&self) -> usize` - Reset to 0, return old value

**Role Each Plays:**
- `AtomicUsize`: Hardware-level atomic integer operations
- `fetch_add`: Atomically adds and returns previous value
- `load/store`: Read/write atomic value with memory ordering
- `SeqCst`: Sequential consistency - all threads see same order of operations

### Checkpoint Tests

```rust
#[test]
fn test_counter_increment() {
    let counter = AtomicCounter::new();
    assert_eq!(counter.get(), 0);

    counter.increment();
    assert_eq!(counter.get(), 1);

    counter.add(5);
    assert_eq!(counter.get(), 6);
}

#[test]
fn test_counter_reset() {
    let counter = AtomicCounter::new();
    counter.add(42);

    let old_value = counter.reset();
    assert_eq!(old_value, 42);
    assert_eq!(counter.get(), 0);
}

#[test]
fn test_concurrent_increments() {
    use std::thread;
    use std::sync::Arc;

    let counter = Arc::new(AtomicCounter::new());
    let mut handles = vec![];

    // 10 threads, each increments 1000 times
    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                counter_clone.increment();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(counter.get(), 10_000);
}
```

### Starter Code

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct AtomicCounter {
    count: AtomicUsize,
}

impl AtomicCounter {
    pub fn new() -> Self {
        // TODO: Initialize with AtomicUsize::new(0)
        todo!()
    }

    pub fn increment(&self) {
        // TODO: Use fetch_add(1, Ordering::SeqCst)
        todo!()
    }

    pub fn add(&self, value: usize) {
        // TODO: Use fetch_add(value, Ordering::SeqCst)
        todo!()
    }

    pub fn get(&self) -> usize {
        // TODO: Use load(Ordering::SeqCst)
        todo!()
    }

    pub fn reset(&self) -> usize {
        // TODO: Use swap(0, Ordering::SeqCst) to atomically replace with 0
        todo!()
    }
}
```

---

## Milestone 2: Multiple Metric Types with Relaxed Ordering

### Introduction

**Why Milestone 1 Is Not Enough:**
Using `SeqCst` for every operation is correct but slow. For independent counters (no cross-counter dependencies), we only need atomicity per-counter, not global ordering. `Relaxed` ordering is ~10-30x faster while maintaining single-variable atomicity.

**What We're Improving:**
Add support for multiple counter types (requests, errors, bytes) with optimized memory ordering. Introduce `Relaxed` ordering for increments and `Acquire` for reads where cross-thread visibility matters.

### Architecture

**Structs:**
- `MetricsCollector` - Collection of typed metrics
  - **Field** `requests: AtomicUsize` - Total requests
  - **Field** `errors: AtomicUsize` - Total errors
  - **Field** `bytes_sent: AtomicUsize` - Total bytes sent
  - **Field** `active_connections: AtomicUsize` - Current connections (gauge)
  - **Function** `new() -> Self` - Create with all counters at 0
  - **Function** `record_request(&self)` - Increment request counter
  - **Function** `record_error(&self)` - Increment error counter
  - **Function** `record_bytes(&self, bytes: usize)` - Add bytes sent
  - **Function** `connection_opened(&self)` - Increment active connections
  - **Function** `connection_closed(&self)` - Decrement active connections
  - **Function** `snapshot(&self) -> MetricsSnapshot` - Get consistent snapshot

- `MetricsSnapshot` - Point-in-time metrics
  - **Field** `requests: usize`
  - **Field** `errors: usize`
  - **Field** `bytes_sent: usize`
  - **Field** `active_connections: usize`
  - **Function** `error_rate(&self) -> f64` - errors / requests

**Role Each Plays:**
- `Relaxed` ordering: Fastest, no cross-thread synchronization
- `Acquire` ordering: Ensures we see all previous writes
- Snapshot: Provides consistent point-in-time view
- Gauge vs Counter: Gauge can go up/down, counter only increases

### Checkpoint Tests

```rust
#[test]
fn test_multiple_metrics() {
    let metrics = MetricsCollector::new();

    metrics.record_request();
    metrics.record_request();
    metrics.record_error();
    metrics.record_bytes(1024);

    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.requests, 2);
    assert_eq!(snapshot.errors, 1);
    assert_eq!(snapshot.bytes_sent, 1024);
    assert_eq!(snapshot.error_rate(), 0.5);
}

#[test]
fn test_gauge_operations() {
    let metrics = MetricsCollector::new();

    metrics.connection_opened();
    metrics.connection_opened();
    assert_eq!(metrics.snapshot().active_connections, 2);

    metrics.connection_closed();
    assert_eq!(metrics.snapshot().active_connections, 1);
}

#[test]
fn test_concurrent_mixed_operations() {
    use std::thread;
    use std::sync::Arc;

    let metrics = Arc::new(MetricsCollector::new());
    let mut handles = vec![];

    for _ in 0..5 {
        let m = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                m.record_request();
                if rand::random::<bool>() {
                    m.record_error();
                }
                m.record_bytes(256);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.requests, 500);
    assert_eq!(snapshot.bytes_sent, 500 * 256);
}
```

### Starter Code

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct MetricsCollector {
    requests: AtomicUsize,
    errors: AtomicUsize,
    bytes_sent: AtomicUsize,
    active_connections: AtomicUsize,
}

impl MetricsCollector {
    pub fn new() -> Self {
        // TODO: Initialize all atomics to 0
        todo!()
    }

    pub fn record_request(&self) {
        // TODO: Use Relaxed ordering - no cross-metric dependencies
        // self.requests.fetch_add(1, Ordering::Relaxed);
        todo!()
    }

    pub fn record_error(&self) {
        // TODO: Relaxed ordering
        todo!()
    }

    pub fn record_bytes(&self, bytes: usize) {
        // TODO: Relaxed ordering
        todo!()
    }

    pub fn connection_opened(&self) {
        // TODO: fetch_add for gauge
        todo!()
    }

    pub fn connection_closed(&self) {
        // TODO: fetch_sub for gauge (can use wrapping arithmetic)
        todo!()
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        // TODO: Use Acquire ordering to see all previous writes
        // Load all atomics with Ordering::Acquire
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub requests: usize,
    pub errors: usize,
    pub bytes_sent: usize,
    pub active_connections: usize,
}

impl MetricsSnapshot {
    pub fn error_rate(&self) -> f64 {
        if self.requests == 0 {
            0.0
        } else {
            self.errors as f64 / self.requests as f64
        }
    }
}
```

---

## Milestone 3: Histogram with Lock-Free Buckets

### Introduction

**Why Milestone 2 Is Not Enough:**
Counters only track totals. To understand latency distribution (p50, p95, p99), we need histograms. A histogram bins measurements into buckets (0-10ms, 10-50ms, 50-100ms, etc.). Each bucket is an atomic counter.

**What We're Improving:**
Add lock-free histogram for tracking response time distributions. Use array of atomic buckets with binary search to find correct bucket. Enable percentile calculations from snapshot.

### Architecture

**Structs:**
- `AtomicHistogram` - Lock-free latency histogram
  - **Field** `buckets: [AtomicUsize; N]` - Fixed bucket array
  - **Field** `bucket_boundaries: [u64; N]` - Upper bounds in microseconds
  - **Function** `new(boundaries: [u64; N]) -> Self` - Create with boundaries
  - **Function** `record(&self, value_us: u64)` - Record measurement
  - **Function** `snapshot(&self) -> HistogramSnapshot` - Get bucket counts
  - **Function** `find_bucket(&self, value: u64) -> usize` - Binary search for bucket

- `HistogramSnapshot` - Point-in-time histogram
  - **Field** `buckets: Vec<usize>` - Count per bucket
  - **Field** `boundaries: Vec<u64>` - Bucket upper bounds
  - **Function** `total(&self) -> usize` - Total observations
  - **Function** `percentile(&self, p: f64) -> u64` - Calculate percentile
  - **Function** `mean(&self) -> f64` - Approximate mean

**Role Each Plays:**
- Fixed buckets: Avoid dynamic allocation in hot path
- Binary search: O(log N) bucket lookup
- Percentile: Find bucket containing Nth percentile observation
- Snapshot: Convert atomic array to owned data

### Checkpoint Tests

```rust
#[test]
fn test_histogram_basic() {
    // Buckets: 0-10ms, 10-50ms, 50-100ms, 100-500ms, 500+ms
    let hist = AtomicHistogram::new([10_000, 50_000, 100_000, 500_000, u64::MAX]);

    hist.record(5_000);   // 5ms -> bucket 0
    hist.record(25_000);  // 25ms -> bucket 1
    hist.record(75_000);  // 75ms -> bucket 2

    let snapshot = hist.snapshot();
    assert_eq!(snapshot.buckets[0], 1);
    assert_eq!(snapshot.buckets[1], 1);
    assert_eq!(snapshot.buckets[2], 1);
    assert_eq!(snapshot.total(), 3);
}

#[test]
fn test_percentile_calculation() {
    let hist = AtomicHistogram::new([10_000, 50_000, 100_000, 500_000, u64::MAX]);

    // Record 100 samples: 50 in bucket 0, 30 in bucket 1, 20 in bucket 2
    for _ in 0..50 {
        hist.record(5_000);
    }
    for _ in 0..30 {
        hist.record(25_000);
    }
    for _ in 0..20 {
        hist.record(75_000);
    }

    let snapshot = hist.snapshot();

    // p50 should be in bucket 0 (first 50%)
    assert!(snapshot.percentile(0.5) <= 10_000);

    // p90 should be in bucket 1 (after 80 samples)
    let p90 = snapshot.percentile(0.9);
    assert!(p90 > 10_000 && p90 <= 50_000);
}

#[test]
fn test_concurrent_histogram() {
    use std::thread;
    use std::sync::Arc;

    let hist = Arc::new(AtomicHistogram::new([10_000, 50_000, 100_000, 500_000, u64::MAX]));
    let mut handles = vec![];

    for thread_id in 0..10 {
        let h = Arc::clone(&hist);
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let value = (thread_id * 1000 + i * 100) as u64;
                h.record(value);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(hist.snapshot().total(), 1000);
}
```

### Starter Code

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct AtomicHistogram<const N: usize> {
    buckets: [AtomicUsize; N],
    bucket_boundaries: [u64; N],
}

impl<const N: usize> AtomicHistogram<N> {
    pub fn new(boundaries: [u64; N]) -> Self {
        // TODO: Create array of AtomicUsize::new(0) for buckets
        // Use std::array::from_fn or manual initialization
        todo!()
    }

    pub fn record(&self, value_us: u64) {
        // TODO:
        // 1. Find bucket index using binary search
        // 2. Increment that bucket with Relaxed ordering
        let bucket_idx = self.find_bucket(value_us);
        todo!()
    }

    fn find_bucket(&self, value: u64) -> usize {
        // TODO: Binary search to find first boundary >= value
        // Use slice::binary_search or manual implementation
        todo!()
    }

    pub fn snapshot(&self) -> HistogramSnapshot {
        // TODO: Load all buckets with Acquire ordering
        todo!()
    }
}

pub struct HistogramSnapshot {
    pub buckets: Vec<usize>,
    pub boundaries: Vec<u64>,
}

impl HistogramSnapshot {
    pub fn total(&self) -> usize {
        // TODO: Sum all bucket counts
        todo!()
    }

    pub fn percentile(&self, p: f64) -> u64 {
        // TODO:
        // 1. Calculate target count: total * p
        // 2. Iterate buckets, accumulating count
        // 3. Return boundary when accumulated >= target
        todo!()
    }

    pub fn mean(&self) -> f64 {
        // TODO: Approximate mean using bucket midpoints
        // For bucket[i], use (boundaries[i-1] + boundaries[i]) / 2
        todo!()
    }
}
```

---

## Milestone 4: Compare-and-Swap for Atomic Max/Min

### Introduction

**Why Milestone 3 Is Not Enough:**
Histograms show distribution but sometimes we need exact min/max values (fastest/slowest request). `fetch_add` doesn't work—we need conditional updates: "update if new value is larger." This requires `compare_and_swap` (CAS).

**What We're Improving:**
Add atomic min/max tracking using compare-and-swap loop. This is a fundamental lock-free primitive: read-modify-write with retry until success.

### Architecture

**Structs:**
- `AtomicMinMax` - Track min and max values
  - **Field** `min: AtomicU64` - Minimum observed
  - **Field** `max: AtomicU64` - Maximum observed
  - **Function** `new() -> Self` - Initialize min=u64::MAX, max=0
  - **Function** `update(&self, value: u64)` - Update min and max
  - **Function** `get_min(&self) -> u64` - Read current minimum
  - **Function** `get_max(&self) -> u64` - Read current maximum
  - **Function** `reset(&self)` - Reset to initial state

**Key Functions:**
- `compare_exchange_weak()` - Try to swap if current value matches expected
- CAS loop pattern: `loop { read current, compute new, try swap, break if success }`

**Role Each Plays:**
- CAS: Atomic test-and-set operation
- `compare_exchange_weak`: May spuriously fail but faster than `strong`
- Retry loop: Keep trying until CAS succeeds (lock-free)
- `Relaxed` ordering: Safe here because single-variable updates

### Checkpoint Tests

```rust
#[test]
fn test_minmax_basic() {
    let minmax = AtomicMinMax::new();

    minmax.update(100);
    assert_eq!(minmax.get_min(), 100);
    assert_eq!(minmax.get_max(), 100);

    minmax.update(50);
    assert_eq!(minmax.get_min(), 50);
    assert_eq!(minmax.get_max(), 100);

    minmax.update(150);
    assert_eq!(minmax.get_min(), 50);
    assert_eq!(minmax.get_max(), 150);
}

#[test]
fn test_concurrent_minmax() {
    use std::thread;
    use std::sync::Arc;

    let minmax = Arc::new(AtomicMinMax::new());
    let mut handles = vec![];

    for thread_id in 0..10 {
        let mm = Arc::clone(&minmax);
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let value = (thread_id * 100 + i) as u64;
                mm.update(value);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(minmax.get_min(), 0);
    assert_eq!(minmax.get_max(), 999);
}

#[test]
fn test_reset() {
    let minmax = AtomicMinMax::new();
    minmax.update(50);
    minmax.update(150);

    minmax.reset();
    assert_eq!(minmax.get_min(), u64::MAX);
    assert_eq!(minmax.get_max(), 0);
}
```

### Starter Code

```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct AtomicMinMax {
    min: AtomicU64,
    max: AtomicU64,
}

impl AtomicMinMax {
    pub fn new() -> Self {
        // TODO: Initialize min to u64::MAX, max to 0
        todo!()
    }

    pub fn update(&self, value: u64) {
        // TODO: Update min using CAS loop
        // Pattern:
        // let mut current_min = self.min.load(Ordering::Relaxed);
        // loop {
        //     if value >= current_min { break; } // Already smaller
        //     match self.min.compare_exchange_weak(
        //         current_min, value, Ordering::Relaxed, Ordering::Relaxed
        //     ) {
        //         Ok(_) => break,
        //         Err(actual) => current_min = actual, // Retry with new value
        //     }
        // }

        // TODO: Same for max (but opposite comparison)
        todo!()
    }

    pub fn get_min(&self) -> u64 {
        // TODO: Load with Acquire ordering
        todo!()
    }

    pub fn get_max(&self) -> u64 {
        // TODO: Load with Acquire ordering
        todo!()
    }

    pub fn reset(&self) {
        // TODO: Store initial values with Release ordering
        todo!()
    }
}
```

---

## Milestone 5: Full Metrics System with Periodic Export

### Introduction

**Why Milestone 4 Is Not Enough:**
Individual components work but real systems need coordinated collection and export. Metrics are useless if not exported to monitoring systems (Prometheus, Grafana, CloudWatch).

**What We're Improving:**
Combine all metric types into unified system with periodic export. Add snapshot-and-reset for delta metrics. Implement background thread for periodic collection without blocking writers.

### Architecture

**Structs:**
- `MetricsRegistry` - Central metrics collection
  - **Field** `collectors: Vec<Arc<MetricsCollector>>` - All metric collectors
  - **Field** `histograms: Vec<Arc<AtomicHistogram<8>>>` - All histograms
  - **Field** `minmax_trackers: Vec<Arc<AtomicMinMax>>` - All min/max trackers
  - **Field** `export_interval: Duration` - How often to export
  - **Field** `running: AtomicBool` - Export thread control
  - **Function** `new(interval: Duration) -> Self` - Create registry
  - **Function** `register_collector(&mut self, name: String) -> Arc<MetricsCollector>`
  - **Function** `register_histogram(&mut self, name: String) -> Arc<AtomicHistogram<8>>`
  - **Function** `start_export_thread(&self, callback: F)` - Start exporter
  - **Function** `stop(&self)` - Stop export thread
  - **Function** `snapshot_all(&self) -> FullSnapshot` - Get all metrics

- `FullSnapshot` - Complete metrics snapshot
  - **Field** `timestamp: SystemTime` - When snapshot was taken
  - **Field** `metrics: HashMap<String, MetricsSnapshot>`
  - **Field** `histograms: HashMap<String, HistogramSnapshot>`
  - **Function** `to_prometheus_format(&self) -> String` - Export format

**Role Each Plays:**
- Registry: Central coordination point
- Arc: Share metrics across threads
- Background thread: Periodic export without blocking
- AtomicBool: Signal thread shutdown
- Callback: Custom export logic (stdout, HTTP, file)

### Checkpoint Tests

```rust
#[test]
fn test_registry_registration() {
    let mut registry = MetricsRegistry::new(Duration::from_secs(10));

    let collector1 = registry.register_collector("http".to_string());
    let collector2 = registry.register_collector("db".to_string());

    collector1.record_request();
    collector2.record_request();
    collector2.record_request();

    let snapshot = registry.snapshot_all();
    assert_eq!(snapshot.metrics["http"].requests, 1);
    assert_eq!(snapshot.metrics["db"].requests, 2);
}

#[test]
fn test_periodic_export() {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    let mut registry = MetricsRegistry::new(Duration::from_millis(100));
    let collector = registry.register_collector("test".to_string());

    let export_count = Arc::new(Mutex::new(0));
    let count_clone = Arc::clone(&export_count);

    registry.start_export_thread(move |snapshot| {
        *count_clone.lock().unwrap() += 1;
        println!("Exported at {:?}", snapshot.timestamp);
    });

    // Generate some metrics
    for _ in 0..10 {
        collector.record_request();
        std::thread::sleep(Duration::from_millis(50));
    }

    registry.stop();

    // Should have exported at least once
    assert!(*export_count.lock().unwrap() >= 1);
}

#[test]
fn test_prometheus_format() {
    let mut registry = MetricsRegistry::new(Duration::from_secs(60));
    let collector = registry.register_collector("http".to_string());

    collector.record_request();
    collector.record_request();
    collector.record_error();
    collector.record_bytes(1024);

    let snapshot = registry.snapshot_all();
    let prom = snapshot.to_prometheus_format();

    assert!(prom.contains("http_requests 2"));
    assert!(prom.contains("http_errors 1"));
    assert!(prom.contains("http_bytes_sent 1024"));
}
```

### Starter Code

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::thread;

pub struct MetricsRegistry {
    collectors: HashMap<String, Arc<MetricsCollector>>,
    histograms: HashMap<String, Arc<AtomicHistogram<8>>>,
    export_interval: Duration,
    running: Arc<AtomicBool>,
}

impl MetricsRegistry {
    pub fn new(interval: Duration) -> Self {
        // TODO: Initialize with empty HashMaps
        todo!()
    }

    pub fn register_collector(&mut self, name: String) -> Arc<MetricsCollector> {
        // TODO: Create collector, wrap in Arc, insert into map, return clone
        todo!()
    }

    pub fn register_histogram(&mut self, name: String, boundaries: [u64; 8]) -> Arc<AtomicHistogram<8>> {
        // TODO: Similar to register_collector
        todo!()
    }

    pub fn start_export_thread<F>(&self, callback: F)
    where
        F: Fn(FullSnapshot) + Send + 'static,
    {
        // TODO:
        // 1. Set running flag to true
        // 2. Clone collectors/histograms for thread
        // 3. Spawn thread that:
        //    - Loops while running is true
        //    - Sleeps for export_interval
        //    - Takes snapshot
        //    - Calls callback
        todo!()
    }

    pub fn stop(&self) {
        // TODO: Set running flag to false
        todo!()
    }

    pub fn snapshot_all(&self) -> FullSnapshot {
        // TODO: Collect all snapshots into FullSnapshot
        todo!()
    }
}

pub struct FullSnapshot {
    pub timestamp: SystemTime,
    pub metrics: HashMap<String, MetricsSnapshot>,
    pub histograms: HashMap<String, HistogramSnapshot>,
}

impl FullSnapshot {
    pub fn to_prometheus_format(&self) -> String {
        // TODO: Format as Prometheus text format
        // Example:
        // # TYPE http_requests counter
        // http_requests 42
        // # TYPE http_errors counter
        // http_errors 3
        todo!()
    }
}
```

---

## Milestone 6: Memory Ordering Optimization and Benchmarking

### Introduction

**Why Milestone 5 Is Not Enough:**
The system works but may be slower than necessary. Different memory orderings have 10-30x performance differences. We need to verify our ordering choices through benchmarking and understand the trade-offs.

**What We're Improving:**
Add comprehensive benchmarks comparing memory orderings. Optimize hot paths using `Relaxed` where safe. Document when each ordering is required and measure performance impact.

### Architecture

**New Components:**
- Benchmark suite comparing orderings
- Performance documentation
- Ordering justification for each atomic operation

**Memory Ordering Rules:**
1. **Relaxed**: Single-variable atomicity only (counters, independent metrics)
2. **Acquire/Release**: Synchronize with other threads (snapshot reads need Acquire)
3. **SeqCst**: Total ordering across all threads (rarely needed)

**Optimization Targets:**
- Hot path: `record_request()`, `record()` - use `Relaxed`
- Read path: `snapshot()` - use `Acquire` to see all writes
- Control: shutdown flag - use `SeqCst` for visibility

### Checkpoint Tests

```rust
#[test]
fn benchmark_counter_increment_relaxed() {
    use std::time::Instant;

    let counter = AtomicCounter::new();
    let start = Instant::now();

    for _ in 0..1_000_000 {
        counter.increment(); // Should use Relaxed
    }

    let elapsed = start.elapsed();
    println!("1M increments (Relaxed): {:?}", elapsed);
    assert_eq!(counter.get(), 1_000_000);
}

#[test]
fn benchmark_concurrent_throughput() {
    use std::thread;
    use std::sync::Arc;
    use std::time::Instant;

    let metrics = Arc::new(MetricsCollector::new());
    let start = Instant::now();

    let handles: Vec<_> = (0..4).map(|_| {
        let m = Arc::clone(&metrics);
        thread::spawn(move || {
            for _ in 0..250_000 {
                m.record_request();
            }
        })
    }).collect();

    for h in handles {
        h.join().unwrap();
    }

    let elapsed = start.elapsed();
    let ops_per_sec = 1_000_000.0 / elapsed.as_secs_f64();

    println!("Throughput: {:.0} ops/sec", ops_per_sec);
    assert_eq!(metrics.snapshot().requests, 1_000_000);
}

#[test]
fn verify_snapshot_consistency() {
    use std::thread;
    use std::sync::Arc;

    let metrics = Arc::new(MetricsCollector::new());

    // Writer thread
    let m1 = Arc::clone(&metrics);
    let writer = thread::spawn(move || {
        for i in 0..1000 {
            m1.record_request();
            m1.record_bytes(i);
        }
    });

    // Reader thread - take many snapshots
    let m2 = Arc::clone(&metrics);
    let reader = thread::spawn(move || {
        for _ in 0..100 {
            let snap = m2.snapshot();
            // If we see N requests, bytes should be consistent
            // (not necessarily exact due to timing, but should be reasonable)
            assert!(snap.bytes_sent <= snap.requests * 1000);
        }
    });

    writer.join().unwrap();
    reader.join().unwrap();
}
```

### Starter Code

```rust
// Add to MetricsCollector implementation

impl MetricsCollector {
    // Optimized version with documented ordering
    pub fn record_request(&self) {
        // ORDERING: Relaxed is safe here because:
        // - Single variable (self.requests) is updated
        // - No dependencies on other variables
        // - Readers use Acquire to synchronize
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        // ORDERING: Acquire ensures we see all Relaxed writes
        // that happened-before this snapshot
        MetricsSnapshot {
            requests: self.requests.load(Ordering::Acquire),
            errors: self.errors.load(Ordering::Acquire),
            bytes_sent: self.bytes_sent.load(Ordering::Acquire),
            active_connections: self.active_connections.load(Ordering::Acquire),
        }
    }
}

// TODO: Add benchmark module
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    fn benchmark_operation<F>(name: &str, iterations: usize, mut op: F)
    where
        F: FnMut(),
    {
        // TODO: Run operation many times, measure time
        // Print results: ops/sec, ns/op
        todo!()
    }

    #[test]
    fn compare_orderings() {
        // TODO: Compare SeqCst vs Acquire vs Relaxed for same operation
        // Show performance difference
        todo!()
    }
}

// TODO: Add documentation module explaining ordering choices
```

---

## Complete Working Example

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

// ============================================================================
// ATOMIC COUNTER
// ============================================================================

pub struct AtomicCounter {
    count: AtomicUsize,
}

impl AtomicCounter {
    pub fn new() -> Self {
        Self {
            count: AtomicUsize::new(0),
        }
    }

    pub fn increment(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add(&self, value: usize) {
        self.count.fetch_add(value, Ordering::Relaxed);
    }

    pub fn get(&self) -> usize {
        self.count.load(Ordering::Acquire)
    }

    pub fn reset(&self) -> usize {
        self.count.swap(0, Ordering::AcqRel)
    }
}

// ============================================================================
// METRICS COLLECTOR
// ============================================================================

pub struct MetricsCollector {
    requests: AtomicUsize,
    errors: AtomicUsize,
    bytes_sent: AtomicUsize,
    active_connections: AtomicUsize,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            requests: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
            bytes_sent: AtomicUsize::new(0),
            active_connections: AtomicUsize::new(0),
        }
    }

    pub fn record_request(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_bytes(&self, bytes: usize) {
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn connection_opened(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn connection_closed(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            requests: self.requests.load(Ordering::Acquire),
            errors: self.errors.load(Ordering::Acquire),
            bytes_sent: self.bytes_sent.load(Ordering::Acquire),
            active_connections: self.active_connections.load(Ordering::Acquire),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub requests: usize,
    pub errors: usize,
    pub bytes_sent: usize,
    pub active_connections: usize,
}

impl MetricsSnapshot {
    pub fn error_rate(&self) -> f64 {
        if self.requests == 0 {
            0.0
        } else {
            self.errors as f64 / self.requests as f64
        }
    }
}

// ============================================================================
// ATOMIC HISTOGRAM
// ============================================================================

pub struct AtomicHistogram<const N: usize> {
    buckets: [AtomicUsize; N],
    bucket_boundaries: [u64; N],
}

impl<const N: usize> AtomicHistogram<N> {
    pub fn new(boundaries: [u64; N]) -> Self {
        Self {
            buckets: std::array::from_fn(|_| AtomicUsize::new(0)),
            bucket_boundaries: boundaries,
        }
    }

    pub fn record(&self, value_us: u64) {
        let bucket_idx = self.find_bucket(value_us);
        self.buckets[bucket_idx].fetch_add(1, Ordering::Relaxed);
    }

    fn find_bucket(&self, value: u64) -> usize {
        self.bucket_boundaries
            .iter()
            .position(|&boundary| value <= boundary)
            .unwrap_or(N - 1)
    }

    pub fn snapshot(&self) -> HistogramSnapshot {
        let buckets = self.buckets
            .iter()
            .map(|b| b.load(Ordering::Acquire))
            .collect();

        HistogramSnapshot {
            buckets,
            boundaries: self.bucket_boundaries.to_vec(),
        }
    }
}

pub struct HistogramSnapshot {
    pub buckets: Vec<usize>,
    pub boundaries: Vec<u64>,
}

impl HistogramSnapshot {
    pub fn total(&self) -> usize {
        self.buckets.iter().sum()
    }

    pub fn percentile(&self, p: f64) -> u64 {
        let total = self.total();
        if total == 0 {
            return 0;
        }

        let target = (total as f64 * p) as usize;
        let mut accumulated = 0;

        for (i, &count) in self.buckets.iter().enumerate() {
            accumulated += count;
            if accumulated >= target {
                return self.boundaries[i];
            }
        }

        *self.boundaries.last().unwrap()
    }

    pub fn mean(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            return 0.0;
        }

        let mut sum = 0.0;
        let mut prev_boundary = 0;

        for (i, &count) in self.buckets.iter().enumerate() {
            let midpoint = (prev_boundary + self.boundaries[i]) as f64 / 2.0;
            sum += midpoint * count as f64;
            prev_boundary = self.boundaries[i];
        }

        sum / total as f64
    }
}

// ============================================================================
// ATOMIC MIN/MAX
// ============================================================================

pub struct AtomicMinMax {
    min: AtomicU64,
    max: AtomicU64,
}

impl AtomicMinMax {
    pub fn new() -> Self {
        Self {
            min: AtomicU64::new(u64::MAX),
            max: AtomicU64::new(0),
        }
    }

    pub fn update(&self, value: u64) {
        // Update min
        let mut current_min = self.min.load(Ordering::Relaxed);
        loop {
            if value >= current_min {
                break;
            }
            match self.min.compare_exchange_weak(
                current_min,
                value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_min = actual,
            }
        }

        // Update max
        let mut current_max = self.max.load(Ordering::Relaxed);
        loop {
            if value <= current_max {
                break;
            }
            match self.max.compare_exchange_weak(
                current_max,
                value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_max = actual,
            }
        }
    }

    pub fn get_min(&self) -> u64 {
        self.min.load(Ordering::Acquire)
    }

    pub fn get_max(&self) -> u64 {
        self.max.load(Ordering::Acquire)
    }

    pub fn reset(&self) {
        self.min.store(u64::MAX, Ordering::Release);
        self.max.store(0, Ordering::Release);
    }
}

// ============================================================================
// METRICS REGISTRY
// ============================================================================

pub struct MetricsRegistry {
    collectors: HashMap<String, Arc<MetricsCollector>>,
    histograms: HashMap<String, Arc<AtomicHistogram<8>>>,
    minmax: HashMap<String, Arc<AtomicMinMax>>,
    export_interval: Duration,
    running: Arc<AtomicBool>,
}

impl MetricsRegistry {
    pub fn new(interval: Duration) -> Self {
        Self {
            collectors: HashMap::new(),
            histograms: HashMap::new(),
            minmax: HashMap::new(),
            export_interval: interval,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn register_collector(&mut self, name: String) -> Arc<MetricsCollector> {
        let collector = Arc::new(MetricsCollector::new());
        self.collectors.insert(name, Arc::clone(&collector));
        collector
    }

    pub fn register_histogram(&mut self, name: String, boundaries: [u64; 8]) -> Arc<AtomicHistogram<8>> {
        let hist = Arc::new(AtomicHistogram::new(boundaries));
        self.histograms.insert(name, Arc::clone(&hist));
        hist
    }

    pub fn register_minmax(&mut self, name: String) -> Arc<AtomicMinMax> {
        let mm = Arc::new(AtomicMinMax::new());
        self.minmax.insert(name, Arc::clone(&mm));
        mm
    }

    pub fn start_export_thread<F>(&mut self, callback: F) -> thread::JoinHandle<()>
    where
        F: Fn(FullSnapshot) + Send + 'static,
    {
        self.running.store(true, Ordering::SeqCst);

        let collectors = self.collectors.clone();
        let histograms = self.histograms.clone();
        let minmax = self.minmax.clone();
        let interval = self.export_interval;
        let running = Arc::clone(&self.running);

        thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                thread::sleep(interval);

                let snapshot = FullSnapshot {
                    timestamp: SystemTime::now(),
                    metrics: collectors
                        .iter()
                        .map(|(k, v)| (k.clone(), v.snapshot()))
                        .collect(),
                    histograms: histograms
                        .iter()
                        .map(|(k, v)| (k.clone(), v.snapshot()))
                        .collect(),
                    minmax: minmax
                        .iter()
                        .map(|(k, v)| (k.clone(), (v.get_min(), v.get_max())))
                        .collect(),
                };

                callback(snapshot);
            }
        })
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn snapshot_all(&self) -> FullSnapshot {
        FullSnapshot {
            timestamp: SystemTime::now(),
            metrics: self.collectors
                .iter()
                .map(|(k, v)| (k.clone(), v.snapshot()))
                .collect(),
            histograms: self.histograms
                .iter()
                .map(|(k, v)| (k.clone(), v.snapshot()))
                .collect(),
            minmax: self.minmax
                .iter()
                .map(|(k, v)| (k.clone(), (v.get_min(), v.get_max())))
                .collect(),
        }
    }
}

pub struct FullSnapshot {
    pub timestamp: SystemTime,
    pub metrics: HashMap<String, MetricsSnapshot>,
    pub histograms: HashMap<String, HistogramSnapshot>,
    pub minmax: HashMap<String, (u64, u64)>,
}

impl FullSnapshot {
    pub fn to_prometheus_format(&self) -> String {
        let mut output = String::new();

        for (name, snapshot) in &self.metrics {
            output.push_str(&format!("# TYPE {}_requests counter\n", name));
            output.push_str(&format!("{}_requests {}\n", name, snapshot.requests));

            output.push_str(&format!("# TYPE {}_errors counter\n", name));
            output.push_str(&format!("{}_errors {}\n", name, snapshot.errors));

            output.push_str(&format!("# TYPE {}_bytes_sent counter\n", name));
            output.push_str(&format!("{}_bytes_sent {}\n", name, snapshot.bytes_sent));

            output.push_str(&format!("# TYPE {}_active_connections gauge\n", name));
            output.push_str(&format!("{}_active_connections {}\n", name, snapshot.active_connections));
        }

        for (name, hist) in &self.histograms {
            output.push_str(&format!("# TYPE {}_latency_us histogram\n", name));
            for (i, count) in hist.buckets.iter().enumerate() {
                output.push_str(&format!(
                    "{}_latency_us_bucket{{le=\"{}\"}} {}\n",
                    name, hist.boundaries[i], count
                ));
            }
        }

        for (name, (min, max)) in &self.minmax {
            output.push_str(&format!("# TYPE {}_min gauge\n", name));
            output.push_str(&format!("{}_min {}\n", name, min));
            output.push_str(&format!("# TYPE {}_max gauge\n", name));
            output.push_str(&format!("{}_max {}\n", name, max));
        }

        output
    }
}

// ============================================================================
// EXAMPLE USAGE
// ============================================================================

fn main() {
    println!("=== Lock-Free Metrics Collector Demo ===\n");

    // Create registry with 2-second export interval
    let mut registry = MetricsRegistry::new(Duration::from_secs(2));

    // Register metrics
    let http_metrics = registry.register_collector("http".to_string());
    let latency_hist = registry.register_histogram(
        "http_latency".to_string(),
        [1000, 5000, 10_000, 50_000, 100_000, 500_000, 1_000_000, u64::MAX],
    );
    let latency_minmax = registry.register_minmax("http_latency".to_string());

    // Start export thread
    let _export_handle = registry.start_export_thread(|snapshot| {
        println!("\n--- Metrics Snapshot ---");
        println!("Timestamp: {:?}", snapshot.timestamp);

        for (name, metrics) in &snapshot.metrics {
            println!("\n{}:", name);
            println!("  Requests: {}", metrics.requests);
            println!("  Errors: {} ({:.2}% error rate)",
                metrics.errors,
                metrics.error_rate() * 100.0
            );
            println!("  Bytes sent: {}", metrics.bytes_sent);
            println!("  Active connections: {}", metrics.active_connections);
        }

        for (name, hist) in &snapshot.histograms {
            println!("\n{} histogram:", name);
            println!("  Total observations: {}", hist.total());
            println!("  Mean: {:.0}μs", hist.mean());
            println!("  p50: {}μs", hist.percentile(0.5));
            println!("  p95: {}μs", hist.percentile(0.95));
            println!("  p99: {}μs", hist.percentile(0.99));
        }

        for (name, (min, max)) in &snapshot.minmax {
            println!("\n{} range:", name);
            println!("  Min: {}μs", min);
            println!("  Max: {}μs", max);
        }
    });

    // Simulate workload with multiple threads
    println!("Simulating workload with 4 worker threads...\n");

    let workers: Vec<_> = (0..4)
        .map(|worker_id| {
            let metrics = Arc::clone(&http_metrics);
            let hist = Arc::clone(&latency_hist);
            let minmax = Arc::clone(&latency_minmax);

            thread::spawn(move || {
                use rand::Rng;
                let mut rng = rand::thread_rng();

                for i in 0..50 {
                    // Simulate request processing
                    let latency_us = rng.gen_range(100..100_000);

                    metrics.record_request();

                    // Simulate error rate ~5%
                    if rng.gen_bool(0.05) {
                        metrics.record_error();
                    }

                    metrics.record_bytes(rng.gen_range(100..10_000));

                    if i % 10 == 0 {
                        metrics.connection_opened();
                    }
                    if i % 15 == 0 && i > 0 {
                        metrics.connection_closed();
                    }

                    // Record latency
                    hist.record(latency_us);
                    minmax.update(latency_us);

                    thread::sleep(Duration::from_millis(rng.gen_range(10..50)));
                }

                println!("Worker {} completed", worker_id);
            })
        })
        .collect();

    // Wait for workers
    for worker in workers {
        worker.join().unwrap();
    }

    // Let final export happen
    thread::sleep(Duration::from_secs(3));

    // Stop export thread
    registry.stop();

    // Final snapshot
    println!("\n\n=== Final Prometheus Export ===\n");
    let final_snapshot = registry.snapshot_all();
    println!("{}", final_snapshot.to_prometheus_format());

    println!("\n=== Benchmarking ===\n");

    // Benchmark counter throughput
    let counter = AtomicCounter::new();
    let start = Instant::now();
    for _ in 0..1_000_000 {
        counter.increment();
    }
    let elapsed = start.elapsed();
    println!("1M counter increments: {:?}", elapsed);
    println!("Throughput: {:.0} ops/sec", 1_000_000.0 / elapsed.as_secs_f64());

    // Benchmark concurrent throughput
    let metrics = Arc::new(MetricsCollector::new());
    let start = Instant::now();

    let bench_workers: Vec<_> = (0..4)
        .map(|_| {
            let m = Arc::clone(&metrics);
            thread::spawn(move || {
                for _ in 0..250_000 {
                    m.record_request();
                }
            })
        })
        .collect();

    for worker in bench_workers {
        worker.join().unwrap();
    }

    let elapsed = start.elapsed();
    println!("\n1M concurrent increments (4 threads): {:?}", elapsed);
    println!("Throughput: {:.0} ops/sec", 1_000_000.0 / elapsed.as_secs_f64());

    println!("\n=== Done ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_counter() {
        let counter = AtomicCounter::new();
        assert_eq!(counter.get(), 0);

        counter.increment();
        assert_eq!(counter.get(), 1);

        counter.add(41);
        assert_eq!(counter.get(), 42);

        let old = counter.reset();
        assert_eq!(old, 42);
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_concurrent_counter() {
        let counter = Arc::new(AtomicCounter::new());
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let c = Arc::clone(&counter);
                thread::spawn(move || {
                    for _ in 0..1000 {
                        c.increment();
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(counter.get(), 10_000);
    }

    #[test]
    fn test_metrics_collector() {
        let metrics = MetricsCollector::new();

        metrics.record_request();
        metrics.record_request();
        metrics.record_error();
        metrics.record_bytes(1024);

        let snap = metrics.snapshot();
        assert_eq!(snap.requests, 2);
        assert_eq!(snap.errors, 1);
        assert_eq!(snap.bytes_sent, 1024);
        assert_eq!(snap.error_rate(), 0.5);
    }

    #[test]
    fn test_histogram() {
        let hist = AtomicHistogram::new([10, 50, 100, 500, u64::MAX, 0, 0, 0]);

        hist.record(5);
        hist.record(25);
        hist.record(75);
        hist.record(200);

        let snap = hist.snapshot();
        assert_eq!(snap.total(), 4);
        assert!(snap.percentile(0.5) <= 100);
    }

    #[test]
    fn test_minmax() {
        let mm = AtomicMinMax::new();

        mm.update(100);
        assert_eq!(mm.get_min(), 100);
        assert_eq!(mm.get_max(), 100);

        mm.update(50);
        mm.update(150);
        assert_eq!(mm.get_min(), 50);
        assert_eq!(mm.get_max(), 150);
    }
}
```

This completes the lock-free metrics collector project with all milestones!
