# Atomic Operations & Lock-Free Programming

Memory Ordering Semantics

- Problem: CPU reordering and compiler optimizations break lock-free
  algorithms; wrong ordering causes races
- Solution: Acquire for reads, Release for writes, AcqRel for RMW, Relaxed
  for counters
- Why It Matters: 10x performance difference (Relaxed vs SeqCst); wrong
  ordering = random production failures
- Use Cases: Lock-free structures, Arc, flags, synchronization, atomic
  counters, wait-free algorithms

Compare-and-Swap Patterns

- Problem: Implementing lock-free operations requires atomic updates; ABA
  problem breaks naive CAS
- Solution: CAS loops with weak/strong variants; version counters or hazard
  pointers for ABA
- Why It Matters: Foundation of all lock-free algorithms; ABA causes silent
  corruption without protection
- Use Cases: Lock-free stacks, atomic max tracking, conditional updates,
  version tracking

Lock-Free Data Structures

- Problem: Mutex serializes access (80% time waiting); deadlocks; priority
  inversion; no real parallelism
- Solution: Treiber stack (CAS-based), MPSC queues, SPSC ring buffers;
  crossbeam for production
- Why It Matters: True parallelism: 8 cores → 8x vs 1x with Mutex;
  100-1000x better under contention
- Use Cases: Work-stealing queues, MPMC queues, real-time systems,
  high-throughput servers

Hazard Pointers

- Problem: Lock-free structures need memory reclamation; can't free nodes
  (use-after-free risk)
- Solution: Mark pointers as "in-use"; defer deletion until safe; epoch-based
  or hazard pointer schemes
- Why It Matters: Prevents crashes in production lock-free code; solves ABA
  problem completely
- Use Cases: Production lock-free stacks/queues, concurrent data structures,
  safe memory management

Seqlock Pattern

- Problem: Frequent reads of small data; locks too expensive; atomics
  insufficient for multi-field updates
- Solution: Sequence counter (odd=writing); readers retry on mismatch;
  optimistic lock-free reads
- Why It Matters: 10-100x faster than locks for read-heavy workloads;
  predictable latency; no blocking
- Use Cases: Coordinates, statistics, sensor data, game state, network
  metrics, configuration


This chapter explores low-level concurrent programming using atomic operations and lock-free data structures. We'll cover memory ordering semantics, compare-and-swap patterns, lock-free algorithms, memory reclamation strategies, and specialized synchronization patterns through practical, production-ready examples.

## Why Use Lock-Free

### **1. Avoid Blocking**
Locks can block threads. When a thread tries to acquire a lock that's held, it *stops* and waits.

Lock-free operations never block:
- A stalled or slow thread cannot prevent others from making progress.
- Ideal for real-time or latency-critical applications.

**Example:** In an audio processing thread, blocking even briefly can cause audible glitches.


### **2. Progress Guarantees**
Lock-free structures guarantee **system-wide progress**:
- *Lock-free*: at least **one** thread always makes progress.
- *Wait-free*: **every** thread makes progress within a bounded time.

Locks guarantee nothing under contention—threads can starve.

---

### **3. No Deadlocks, No Priority Inversion**
Locks come with hazards:
- Deadlocks
- Priority inversion (low-priority thread holds lock; high-priority thread waits)
- Convoys (one slow thread causes others to queue)

Lock-free code avoids all of these because it never "holds" exclusive access.

### **4. Better Scalability Under High Contention**
With many CPUs hammering the same lock, performance collapses:
- Threads constantly block, sleep, wake up (expensive operations)
- Cache lines bounce between cores like crazy

Lock-free operations often scale much better because:
- They use atomic instructions (CAS, fetch_add) that avoid kernel involvement
- They allow optimistic concurrency—many threads proceed in parallel


### **5. Lower Latency, Not Just Higher Throughput**
Under load, locks often collapse into long queues, causing:
- Latency spikes
- Tail latency problems (p99, p999)

Lock-free structures often offer:
- Predictable latency
- Fewer outlier delays


### **6. Useful for Specialized Cases (e.g., SPSC queues)**
Some lock-free patterns are extremely simple and efficient:
- Single-producer, single-consumer queue (SPSC)
- Atomic counters
- Seqlock reads

These outperform lock-based versions by avoiding all synchronization except atomic loads/stores.


# ❌ **Why *NOT* Use Lock-Free Everywhere?**

Lock-free is **hard**:
- Complex to design
- Easy to introduce subtle memory-ordering bugs
- ABA problem must be handled (with hazard pointers or epoch GC)
- Debugging is more difficult
- Unsafe code is often required

Locks are:
- Simple
- Correct by default
- Good enough for most workloads

**Rule of thumb:**
> Use locks unless you have a *measurable* reason not to.


### Summary

| Feature | Locks | Lock-Free |
|--------|-------|-----------|
| Blocking | Yes | No |
| Deadlocks | Possible | Impossible |
| Priority inversion | Possible | Impossible |
| Contention behavior | Poor | Often good |
| Latency | Can spike | More stable |
| Complexity | Low | High |
| Safety | Easy | Hard |


## Table of Contents

1. [Memory Ordering Semantics](#memory-ordering-semantics)
2. [Compare-and-Swap Patterns](#compare-and-swap-patterns)
3. [Lock-Free Queues and Stacks](#lock-free-queues-and-stacks)
4. [Hazard Pointers](#hazard-pointers)
5. [Seqlock Pattern](#seqlock-pattern)

---

## Memory Ordering Semantics

**Problem**: CPU reordering and compiler optimizations can break lock-free algorithms—writes may be visible in different order than written. `Relaxed` ordering is fast but provides no guarantees, causing race conditions. `SeqCst` (sequentially consistent) is slow—acts like global lock on all atomics. Wrong ordering causes subtle bugs: ABA problem, data races, lost updates. Memory fence placement is complex and error-prone.

**Solution**: Use `Acquire` for reads that need to see all previous writes. Use `Release` for writes that make previous operations visible. Combine `AcqRel` for read-modify-write. Use `Relaxed` only for counters where ordering doesn't matter. Use `SeqCst` when correctness is unclear or for debugging. Understand happens-before relationships to reason about correctness.

**Why It Matters**: Ordering determines correctness and performance. Wrong ordering: lock-free queue corrupts data, appears to work in tests, fails randomly in production. `Relaxed` is 1-2 cycles, `Acquire`/`Release` is 5-10 cycles, `SeqCst` is 20-50 cycles—10x performance difference. Spinlock with `Release`/`Acquire`: correct and fast. With `Relaxed`: broken. With `SeqCst`: correct but slow. Understanding memory ordering is essential for lock-free programming.

**Use Cases**: Lock-free data structures (queues, stacks, maps), reference counting (Arc), flags and signals, atomic counters, synchronization primitives, wait-free algorithms.

### Atomic Orderings Are not Locks

Locks provide: Mutual exclusion (only one thread in critical section)
Atomics provide: Memory visibility ordering (when writes become visible)

Acquire semantics: When a thread performs a read operation (like get()), it ensures that all previous writes (by other threads) are visible. This prevents reordering of operations that follow the read.
Release semantics: When a thread performs a write operation (like set()), it ensures that all previous operations are completed before the write. This prevents reordering of operations that precede the write.



| Ordering | What It Does                                                  | Use For |
|----------|---------------------------------------------------------------|---------|
| **Relaxed** | Atomicity only, no ordering                                   | Counters where order doesn't matter |
| **Acquire** | that all previous writes (by other threads) are visible       | Reading after another thread signals "done" |
| **Release** | all writes (by other threads) are completed before this write | Signaling "I'm done writing" |
| **AcqRel** | Both Acquire + Release                                        | Read-modify-write operations (CAS, fetch_add) |
| **SeqCst** | Total global order of all operations                          | When you need strongest guarantees |

### Pattern 1: Memory Ordering Fundamentals

**Problem**: Understand different memory orderings and their performance/correctness trade-offs.

**Solution**:

```rust
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

//======================================================
// Pattern 1: Relaxed - No ordering guarantees (fastest)
//======================================================
// Use for: Counters where exact ordering doesn't matter
fn relaxed_ordering_example() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                // Relaxed: no synchronization, just atomicity
                counter.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Final value is guaranteed to be correct
    // but intermediate values may have appeared in any order
    println!("Counter (Relaxed): {}", counter.load(Ordering::Relaxed));
}

//============================================================================
// Pattern 2: Acquire/Release - Synchronization without sequential consistency
//============================================================================
// Use for: Producer-consumer, message passing
fn acquire_release_ordering() {
    let data = Arc::new(AtomicUsize::new(0));
    let ready = Arc::new(AtomicBool::new(false));

    let data_clone = Arc::clone(&data);
    let ready_clone = Arc::clone(&ready);

    // Producer
    let producer = thread::spawn(move || {
        // Write data
        data_clone.store(42, Ordering::Relaxed);

        // Release: all previous writes visible to thread that Acquires
        ready_clone.store(true, Ordering::Release);
    });

    // Consumer
    let consumer = thread::spawn(move || {
        // Acquire: see all writes before the Release
        while !ready.load(Ordering::Acquire) {
            thread::yield_now();
        }

        // Guaranteed to see data == 42
        let value = data.load(Ordering::Relaxed);
        println!("Consumer sees: {}", value);
        assert_eq!(value, 42);
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}

//==============================================================================
// Pattern 3: SeqCst - Sequential consistency (slowest, easiest to reason about)
//==============================================================================
// Use for: When correctness is critical and performance is secondary
fn seq_cst_ordering() {
    let x = Arc::new(AtomicBool::new(false));
    let y = Arc::new(AtomicBool::new(false));
    let z1 = Arc::new(AtomicBool::new(false));
    let z2 = Arc::new(AtomicBool::new(false));

    let x1 = Arc::clone(&x);
    let y1 = Arc::clone(&y);
    let z1_clone = Arc::clone(&z1);

    let t1 = thread::spawn(move || {
        x1.store(true, Ordering::SeqCst);
        if !y1.load(Ordering::SeqCst) {
            z1_clone.store(true, Ordering::SeqCst);
        }
    });

    let x2 = Arc::clone(&x);
    let y2 = Arc::clone(&y);
    let z2_clone = Arc::clone(&z2);

    let t2 = thread::spawn(move || {
        y2.store(true, Ordering::SeqCst);
        if !x2.load(Ordering::SeqCst) {
            z2_clone.store(true, Ordering::SeqCst);
        }
    });

    t1.join().unwrap();
    t2.join().unwrap();

    // With SeqCst: cannot have both z1 and z2 true
    // Without SeqCst: theoretically possible (hardware reordering)
    let both = z1.load(Ordering::SeqCst) && z2.load(Ordering::SeqCst);
    println!("Both flags set: {} (should be false with SeqCst)", both);
}

//================================================
// Pattern 4: AcqRel - Combine Acquire and Release
//================================================
// Use for: Read-modify-write operations
fn acq_rel_ordering() {
    let counter = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];
    for _ in 0..5 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                // AcqRel: Acts as Acquire for load, Release for store
                counter.fetch_add(1, Ordering::AcqRel);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Counter (AcqRel): {}", counter.load(Ordering::Acquire));
}

//==========================================
// Real-world: Spinlock with proper ordering
//==========================================
struct Spinlock {
    locked: AtomicBool,
}

impl Spinlock {
    fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }

    fn lock(&self) {
        while self
            .locked
            .compare_exchange_weak(
                false,
                true,
                Ordering::Acquire, // Success: acquire lock
                Ordering::Relaxed, // Failure: just retry
            )
            .is_err()
        {
            // Hint to CPU we're spinning
            while self.locked.load(Ordering::Relaxed) {
                std::hint::spin_loop();
            }
        }
    }

    fn unlock(&self) {
        // Release: make all previous writes visible
        self.locked.store(false, Ordering::Release);
    }
}

//===========================================================
// Real-world: Double-checked locking for lazy initialization
//===========================================================
struct LazyInit<T> {
    data: AtomicUsize, // Actually *mut T
    initialized: AtomicBool,
}

impl<T> LazyInit<T> {
    fn new() -> Self {
        Self {
            data: AtomicUsize::new(0),
            initialized: AtomicBool::new(false),
        }
    }

    fn get_or_init<F>(&self, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        // Fast path: already initialized (Acquire ensures we see the data)
        if self.initialized.load(Ordering::Acquire) {
            unsafe { &*(self.data.load(Ordering::Relaxed) as *const T) }
        } else {
            self.init_slow(init)
        }
    }

    fn init_slow<F>(&self, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        let ptr = Box::into_raw(Box::new(init()));

        // Try to publish (use SeqCst for correctness)
        match self.initialized.compare_exchange(
            false,
            true,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => {
                // We won the race
                self.data.store(ptr as usize, Ordering::Release);
                unsafe { &*ptr }
            }
            Err(_) => {
                // Someone else won, clean up our allocation
                unsafe { drop(Box::from_raw(ptr)) };
                unsafe { &*(self.data.load(Ordering::Acquire) as *const T) }
            }
        }
    }
}

fn main() {
    println!("=== Relaxed Ordering ===\n");
    relaxed_ordering_example();

    println!("\n=== Acquire/Release Ordering ===\n");
    acquire_release_ordering();

    println!("\n=== Sequential Consistency ===\n");
    seq_cst_ordering();

    println!("\n=== AcqRel Ordering ===\n");
    acq_rel_ordering();

    println!("\n=== Spinlock ===\n");
    let lock = Arc::new(Spinlock::new());
    let mut handles = vec![];

    for i in 0..3 {
        let lock = Arc::clone(&lock);
        handles.push(thread::spawn(move || {
            lock.lock();
            println!("Thread {} acquired lock", i);
            thread::sleep(Duration::from_millis(10));
            lock.unlock();
            println!("Thread {} released lock", i);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

**Memory Ordering Guide**:

| Ordering | Guarantees | Use Case | Performance |
|----------|-----------|----------|-------------|
| Relaxed | Atomicity only | Counters, flags (order doesn't matter) | Fastest |
| Acquire | See writes before Release | Consumer in producer-consumer | Fast |
| Release | Publish writes to Acquire | Producer in producer-consumer | Fast |
| AcqRel | Both Acquire and Release | RMW operations | Medium |
| SeqCst | Total order across all threads | When correctness is critical | Slowest |

---

### Pattern 2: Fence Operations

**Problem**: Establish memory ordering without atomic operations, or strengthen ordering of existing atomics.

**Solution**:

```rust
use std::sync::atomic::{fence, AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

//=====================================
// Pattern 1: Fence for non-atomic data
//=====================================
fn fence_with_non_atomic() {
    let mut data = 0u64;
    let ready = Arc::new(AtomicBool::new(false));
    let ready_clone = Arc::clone(&ready);

    // Producer
    let producer = thread::spawn(move || {
        unsafe {
            let data_ptr = &mut data as *mut u64;

            // Write non-atomic data
            *data_ptr = 42;

            // Fence ensures all previous writes are visible
            fence(Ordering::Release);

            // Signal ready
            ready_clone.store(true, Ordering::Relaxed);
        }
    });

    // Consumer
    thread::sleep(std::time::Duration::from_millis(10));

    if ready.load(Ordering::Relaxed) {
        // Fence ensures we see all writes before the Release fence
        fence(Ordering::Acquire);

        // Now safe to read data
        println!("Data: {}", data);
    }

    producer.join().unwrap();
}

//==============================================================
// Pattern 2: Compiler fence (prevents compiler reordering only)
//==============================================================
fn compiler_fence_example() {
    let x = AtomicUsize::new(0);
    let y = AtomicUsize::new(0);

    x.store(1, Ordering::Relaxed);

    // Prevent compiler from reordering (hardware can still reorder)
    std::sync::atomic::compiler_fence(Ordering::SeqCst);

    y.store(2, Ordering::Relaxed);

    // Ensures compiler sees x=1 before y=2
}

//========================================
// Real-world: Memory barrier for DMA/MMIO
//========================================
#[repr(C)]
struct DmaBuffer {
    data: [u8; 4096],
    ready: AtomicBool,
}

impl DmaBuffer {
    fn write_for_dma(&mut self, data: &[u8]) {
        self.data[..data.len()].copy_from_slice(data);

        // Ensure all writes complete before signaling device
        fence(Ordering::Release);

        self.ready.store(true, Ordering::Relaxed);
    }

    fn read_from_dma(&mut self) -> Option<&[u8]> {
        if !self.ready.load(Ordering::Relaxed) {
            return None;
        }

        // Ensure we see all device writes
        fence(Ordering::Acquire);

        Some(&self.data)
    }
}

fn main() {
    println!("=== Fence with Non-Atomic ===\n");
    fence_with_non_atomic();

    println!("\n=== Compiler Fence ===\n");
    compiler_fence_example();
}
```

**Fence Types**:
- **fence(Ordering)**: Hardware and compiler barrier
- **compiler_fence(Ordering)**: Compiler-only barrier (no CPU fence)
- Use for: MMIO, DMA, FFI boundaries

---

## Compare-and-Swap Patterns

**Problem**: Implementing lock-free operations requires atomic read-modify-write. Naive approaches have race conditions when multiple threads update simultaneously. Need to detect when value changed between read and write. ABA problem: value changes A→B→A, CAS succeeds but intermediate state was different. Conditional updates require careful retry logic.

**Solution**: Use compare-and-swap (CAS) as fundamental building block. Load current value, compute new value, CAS to update only if unchanged. Use `compare_exchange_weak` in loops (allows spurious failure). Use `compare_exchange` outside loops (stronger guarantee). Handle ABA with version counters or hazard pointers. Implement exponential backoff for contention.

**Why It Matters**: CAS is foundation of all lock-free algorithms. Without proper CAS loops, concurrent updates are lost. ABA problem causes silent data corruption—tests pass, production fails mysteriously. Lock-free max tracker: naive store loses updates, CAS ensures correctness. Conditional counter with CAS prevents overflow bugs. Performance: proper backoff reduces CPU waste during contention.

**Use Cases**: Lock-free stacks and queues, atomic max/min tracking, conditional increments (rate limiting), version tracking, optimistic updates, retry logic.

### Pattern 3: CAS Basics and Patterns

**Problem**: Use compare-and-swap to implement lock-free operations correctly.

**Solution**:

```rust
use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};
use std::sync::Arc;
use std::thread;
use std::ptr;

//==========================
// Pattern 1: Basic CAS loop
//==========================
fn cas_increment(counter: &AtomicUsize) {
    loop {
        let current = counter.load(Ordering::Relaxed);
        let new_value = current + 1;

        // Try to update: succeeds only if value hasn't changed
        if counter
            .compare_exchange_weak(
                current,
                new_value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_ok()
        {
            break;
        }

        // Spurious failure or actual contention - retry
    }
}

//=====================================================
// Pattern 2: compare_exchange vs compare_exchange_weak
//=====================================================
fn compare_exchange_variants() {
    let value = AtomicUsize::new(0);

    // compare_exchange: never spurious failure, use in non-loop
    let result = value.compare_exchange(
        0,
        1,
        Ordering::SeqCst,
        Ordering::SeqCst,
    );
    assert!(result.is_ok());

    // compare_exchange_weak: may spuriously fail, use in loop
    loop {
        if value
            .compare_exchange_weak(1, 2, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            break;
        }
    }

    println!("Final value: {}", value.load(Ordering::SeqCst));
}

//========================================
// Pattern 3: CAS with data transformation
//========================================
fn cas_update<F>(counter: &AtomicUsize, f: F)
where
    F: Fn(usize) -> usize,
{
    let mut current = counter.load(Ordering::Relaxed);

    loop {
        let new_value = f(current);

        match counter.compare_exchange_weak(
            current,
            new_value,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(actual) => current = actual, // Update current and retry
        }
    }
}

//===================================
// Real-world: Lock-free max tracking
//===================================
struct MaxTracker {
    max: AtomicUsize,
}

impl MaxTracker {
    fn new() -> Self {
        Self {
            max: AtomicUsize::new(0),
        }
    }

    fn update(&self, value: usize) {
        let mut current = self.max.load(Ordering::Relaxed);

        loop {
            if value <= current {
                // Already have a larger max
                break;
            }

            match self.max.compare_exchange_weak(
                current,
                value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }

    fn get(&self) -> usize {
        self.max.load(Ordering::Relaxed)
    }
}

//==================================
// Real-world: Lock-free accumulator
//==================================
struct Accumulator {
    sum: AtomicUsize,
    count: AtomicUsize,
}

impl Accumulator {
    fn new() -> Self {
        Self {
            sum: AtomicUsize::new(0),
            count: AtomicUsize::new(0),
        }
    }

    fn add(&self, value: usize) {
        self.sum.fetch_add(value, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    fn average(&self) -> f64 {
        let sum = self.sum.load(Ordering::Relaxed);
        let count = self.count.load(Ordering::Relaxed);

        if count == 0 {
            0.0
        } else {
            sum as f64 / count as f64
        }
    }

    fn reset(&self) -> (usize, usize) {
        let sum = self.sum.swap(0, Ordering::Relaxed);
        let count = self.count.swap(0, Ordering::Relaxed);
        (sum, count)
    }
}

//===============================
// Real-world: Conditional update
//===============================
struct ConditionalCounter {
    value: AtomicUsize,
}

impl ConditionalCounter {
    fn new(initial: usize) -> Self {
        Self {
            value: AtomicUsize::new(initial),
        }
    }

    fn increment_if_below(&self, threshold: usize) -> bool {
        let mut current = self.value.load(Ordering::Relaxed);

        loop {
            if current >= threshold {
                return false; // Can't increment
            }

            match self.value.compare_exchange_weak(
                current,
                current + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }

    fn get(&self) -> usize {
        self.value.load(Ordering::Relaxed)
    }
}

fn main() {
    println!("=== CAS Increment ===\n");

    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                cas_increment(&counter);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", counter.load(Ordering::Relaxed));

    println!("\n=== Max Tracker ===\n");

    let tracker = Arc::new(MaxTracker::new());
    let mut handles = vec![];

    for i in 0..10 {
        let tracker = Arc::clone(&tracker);
        handles.push(thread::spawn(move || {
            for j in 0..100 {
                tracker.update(i * 100 + j);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Max value: {}", tracker.get());

    println!("\n=== Accumulator ===\n");

    let acc = Accumulator::new();

    for i in 1..=100 {
        acc.add(i);
    }

    println!("Average: {:.2}", acc.average());
    println!("Reset: {:?}", acc.reset());
    println!("After reset: {:.2}", acc.average());

    println!("\n=== Conditional Counter ===\n");

    let counter = ConditionalCounter::new(0);

    for _ in 0..15 {
        if counter.increment_if_below(10) {
            println!("Incremented to {}", counter.get());
        } else {
            println!("Threshold reached: {}", counter.get());
        }
    }
}
```

**CAS Patterns**:
- **Basic loop**: Load, compute, CAS, retry on failure
- **compare_exchange_weak**: Use in loops (may spuriously fail)
- **compare_exchange**: Use outside loops (stronger guarantee)
- **Failure handling**: Update current value on failure

---

### Pattern 4: ABA Problem and Solutions

**Problem**: Detect and prevent the ABA problem where a value changes from A to B back to A, fooling CAS.

**Solution**:

```rust
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

//================================
// Problem: ABA without protection
//================================
struct NaiveStack<T> {
    head: AtomicUsize, // *mut Node<T>
    _phantom: std::marker::PhantomData<T>,
}

struct Node<T> {
    data: T,
    next: *mut Node<T>,
}

//=============================================
// This is unsafe and suffers from ABA problem!
//=============================================
impl<T> NaiveStack<T> {
    unsafe fn pop_unsafe(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire) as *mut Node<T>;

            if head.is_null() {
                return None;
            }

            let next = (*head).next;

            // ABA PROBLEM: Between load and CAS, another thread could:
            // 1. Pop this node
            // 2. Free it
            // 3. Push new nodes
            // 4. Push this node back (same address!)
            // CAS succeeds but we're in trouble!

            if self
                .head
                .compare_exchange(
                    head as usize,
                    next as usize,
                    Ordering::Release,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                let data = std::ptr::read(&(*head).data);
                drop(Box::from_raw(head));
                return Some(data);
            }
        }
    }
}

//==============================================
// Solution 1: Tagged pointers (version counter)
//==============================================
struct TaggedPtr {
    value: AtomicU64, // Upper 16 bits: tag, Lower 48 bits: pointer
}

impl TaggedPtr {
    fn new(ptr: *mut u8) -> Self {
        Self {
            value: AtomicU64::new(ptr as u64),
        }
    }

    fn load(&self, ordering: Ordering) -> (*mut u8, u16) {
        let packed = self.value.load(ordering);
        let ptr = (packed & 0x0000_FFFF_FFFF_FFFF) as *mut u8;
        let tag = (packed >> 48) as u16;
        (ptr, tag)
    }

    fn store(&self, ptr: *mut u8, tag: u16, ordering: Ordering) {
        let packed = ((tag as u64) << 48) | ((ptr as u64) & 0x0000_FFFF_FFFF_FFFF);
        self.value.store(packed, ordering);
    }

    fn compare_exchange(
        &self,
        current_ptr: *mut u8,
        current_tag: u16,
        new_ptr: *mut u8,
        new_tag: u16,
        success: Ordering,
        failure: Ordering,
    ) -> Result<(), ()> {
        let current = ((current_tag as u64) << 48) | ((current_ptr as u64) & 0x0000_FFFF_FFFF_FFFF);
        let new = ((new_tag as u64) << 48) | ((new_ptr as u64) & 0x0000_FFFF_FFFF_FFFF);

        self.value
            .compare_exchange(current, new, success, failure)
            .map(|_| ())
            .map_err(|_| ())
    }
}

//=====================================
// Solution 2: Version counter approach
//=====================================
struct VersionedStack<T> {
    head: AtomicU64, // Upper 32 bits: version, Lower 32 bits: index
    nodes: Vec<Option<VersionedNode<T>>>,
}

struct VersionedNode<T> {
    data: T,
    next: u32,
    version: u32,
}

impl<T> VersionedStack<T> {
    fn pack(index: u32, version: u32) -> u64 {
        ((version as u64) << 32) | (index as u64)
    }

    fn unpack(packed: u64) -> (u32, u32) {
        let index = (packed & 0xFFFF_FFFF) as u32;
        let version = (packed >> 32) as u32;
        (index, version)
    }
}

//=================================================
// Solution 3: Epoch-based reclamation (simplified)
//=================================================
struct EpochGC {
    global_epoch: AtomicUsize,
}

impl EpochGC {
    fn new() -> Self {
        Self {
            global_epoch: AtomicUsize::new(0),
        }
    }

    fn pin(&self) -> usize {
        self.global_epoch.load(Ordering::Acquire)
    }

    fn try_advance(&self) {
        self.global_epoch.fetch_add(1, Ordering::Release);
    }

    fn is_safe_to_free(&self, allocation_epoch: usize) -> bool {
        let current = self.global_epoch.load(Ordering::Acquire);
        current > allocation_epoch + 2 // Conservative: 2 epochs old
    }
}

//=============================
// Real-world: ABA-safe counter
//=============================
struct ABACounter {
    value: AtomicU64, // Upper 32 bits: version, Lower 32 bits: count
}

impl ABACounter {
    fn new(initial: u32) -> Self {
        Self {
            value: AtomicU64::new(initial as u64),
        }
    }

    fn increment(&self) {
        loop {
            let current = self.value.load(Ordering::Relaxed);
            let count = (current & 0xFFFF_FFFF) as u32;
            let version = (current >> 32) as u32;

            let new_count = count.wrapping_add(1);
            let new_version = version.wrapping_add(1);
            let new_value = ((new_version as u64) << 32) | (new_count as u64);

            if self
                .value
                .compare_exchange_weak(current, new_value, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    fn get(&self) -> u32 {
        let packed = self.value.load(Ordering::Relaxed);
        (packed & 0xFFFF_FFFF) as u32
    }

    fn get_with_version(&self) -> (u32, u32) {
        let packed = self.value.load(Ordering::Relaxed);
        let count = (packed & 0xFFFF_FFFF) as u32;
        let version = (packed >> 32) as u32;
        (count, version)
    }
}

fn main() {
    println!("=== ABA Counter ===\n");

    let counter = Arc::new(ABACounter::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                counter.increment();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let (count, version) = counter.get_with_version();
    println!("Count: {}, Version: {}", count, version);

    println!("\n=== Epoch GC ===\n");

    let gc = EpochGC::new();

    let epoch1 = gc.pin();
    println!("Pinned at epoch {}", epoch1);

    gc.try_advance();
    gc.try_advance();
    gc.try_advance();

    println!("Safe to free epoch 1? {}", gc.is_safe_to_free(epoch1));
}
```

**ABA Solutions**:
1. **Tagged pointers**: Add version counter to pointer
2. **Double-width CAS**: CAS on (pointer, version) pair
3. **Epoch-based reclamation**: Defer deletion until safe
4. **Hazard pointers**: Track active pointers (next Pattern)

---

## Lock-Free Queues and Stacks

**Problem**: Mutex-based data structures serialize all access—threads wait even when operating on different elements. Lock contention causes 80% of multi-threaded time spent waiting. Priority inversion: low-priority thread holds lock, blocking high-priority thread. Deadlocks from lock ordering mistakes. Panics while holding lock poison the mutex. Real-time systems can't tolerate lock-induced latency spikes.

**Solution**: Use Treiber stack (lock-free stack with CAS-based push/pop). Implement MPSC queue (multi-producer single-consumer) with atomic operations. Use SPSC ring buffer for bounded single-producer single-consumer. Leverage `crossbeam::queue` for production-ready implementations. Handle memory reclamation with hazard pointers or epoch-based GC. Use atomic operations with proper ordering for correctness.

**Why It Matters**: Lock-free structures enable true parallelism. Multi-threaded counter with Mutex: serialized updates = 1 core performance. Lock-free stack: linear scaling = 8 cores → 8x throughput. MPMC queue with crossbeam: 100-1000x better than Mutex<VecDeque> under contention. Real-time audio processing requires lock-free queues to prevent dropouts. Database systems use lock-free structures for transaction processing at millions/second. Work-stealing schedulers power async runtimes.

**Use Cases**: Work-stealing task queues (tokio, rayon), MPMC message passing, real-time audio/video processing, high-frequency trading, concurrent data structure building blocks, actor system mailboxes.

### Pattern 5: Treiber Stack (Lock-Free Stack)

**Problem**: Implement a lock-free stack that allows concurrent push/pop operations.

**Solution**:

```rust
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;
use std::sync::Arc;
use std::thread;

struct Node<T> {
    data: T,
    next: *mut Node<T>,
}

pub struct TreiberStack<T> {
    head: AtomicPtr<Node<T>>,
}

impl<T> TreiberStack<T> {
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(Node {
            data,
            next: ptr::null_mut(),
        }));

        loop {
            let head = self.head.load(Ordering::Relaxed);
            unsafe {
                (*new_node).next = head;
            }

            if self
                .head
                .compare_exchange_weak(head, new_node, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);

            if head.is_null() {
                return None;
            }

            unsafe {
                let next = (*head).next;

                if self
                    .head
                    .compare_exchange_weak(head, next, Ordering::Release, Ordering::Acquire)
                    .is_ok()
                {
                    let data = ptr::read(&(*head).data);
                    // WARNING: This is unsafe! We should use hazard pointers or epoch-based GC
                    // For now, we leak the node to avoid use-after-free
                    // drop(Box::from_raw(head));
                    return Some(data);
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Acquire).is_null()
    }
}

unsafe impl<T: Send> Send for TreiberStack<T> {}
unsafe impl<T: Send> Sync for TreiberStack<T> {}

//=============================================
// Real-world: Work-stealing deque (simplified)
//=============================================
pub struct WorkStealingDeque<T> {
    bottom: AtomicPtr<Node<T>>,
    top: AtomicPtr<Node<T>>,
}

impl<T> WorkStealingDeque<T> {
    pub fn new() -> Self {
        Self {
            bottom: AtomicPtr::new(ptr::null_mut()),
            top: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(Node {
            data,
            next: ptr::null_mut(),
        }));

        loop {
            let bottom = self.bottom.load(Ordering::Relaxed);
            unsafe {
                (*new_node).next = bottom;
            }

            if self
                .bottom
                .compare_exchange_weak(bottom, new_node, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        // Owner pops from bottom (LIFO - better cache locality)
        loop {
            let bottom = self.bottom.load(Ordering::Acquire);

            if bottom.is_null() {
                return None;
            }

            unsafe {
                let next = (*bottom).next;

                if self
                    .bottom
                    .compare_exchange_weak(bottom, next, Ordering::Release, Ordering::Acquire)
                    .is_ok()
                {
                    let data = ptr::read(&(*bottom).data);
                    return Some(data);
                }
            }
        }
    }

    pub fn steal(&self) -> Option<T> {
        // Thieves steal from top (FIFO - oldest work)
        loop {
            let top = self.top.load(Ordering::Acquire);

            if top.is_null() {
                return None;
            }

            unsafe {
                let next = (*top).next;

                if self
                    .top
                    .compare_exchange_weak(top, next, Ordering::Release, Ordering::Acquire)
                    .is_ok()
                {
                    let data = ptr::read(&(*top).data);
                    return Some(data);
                }
            }
        }
    }
}

unsafe impl<T: Send> Send for WorkStealingDeque<T> {}
unsafe impl<T: Send> Sync for WorkStealingDeque<T> {}

fn main() {
    println!("=== Treiber Stack ===\n");

    let stack = Arc::new(TreiberStack::new());
    let mut handles = vec![];

    // Producers
    for i in 0..5 {
        let stack = Arc::clone(&stack);
        handles.push(thread::spawn(move || {
            for j in 0..100 {
                stack.push(i * 100 + j);
            }
        }));
    }

    // Consumers
    for _ in 0..5 {
        let stack = Arc::clone(&stack);
        handles.push(thread::spawn(move || {
            let mut count = 0;
            while let Some(_) = stack.pop() {
                count += 1;
            }
            count
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Stack empty: {}", stack.is_empty());

    println!("\n=== Work Stealing Deque ===\n");

    let deque = Arc::new(WorkStealingDeque::new());

    // Owner thread
    let owner_deque = Arc::clone(&deque);
    let owner = thread::spawn(move || {
        for i in 0..100 {
            owner_deque.push(i);
        }

        let mut popped = 0;
        while owner_deque.pop().is_some() {
            popped += 1;
        }
        println!("Owner popped: {}", popped);
    });

    // Thief threads
    let mut thieves = vec![];
    for id in 0..3 {
        let thief_deque = Arc::clone(&deque);
        thieves.push(thread::spawn(move || {
            thread::sleep(std::time::Duration::from_millis(10));
            let mut stolen = 0;
            while thief_deque.steal().is_some() {
                stolen += 1;
            }
            println!("Thief {} stole: {}", id, stolen);
        }));
    }

    owner.join().unwrap();
    for thief in thieves {
        thief.join().unwrap();
    }
}
```

**Treiber Stack Properties**:
- **Lock-free**: At least one thread makes progress
- **Push**: O(1) average case
- **Pop**: O(1) average case
- **ABA problem**: Requires protection (hazard pointers or epoch GC)

---

### Pattern 6: Lock-Free Queue (MPSC)

**Problem**: Implement a multi-producer single-consumer lock-free queue.

**Solution**:

```rust
use std::sync::atomic::{AtomicPtr, AtomicBool, Ordering, AtomicUsize};
use std::ptr;
use std::sync::Arc;
use std::thread;

struct QueueNode<T> {
    data: Option<T>,
    next: AtomicPtr<QueueNode<T>>,
}

pub struct MpscQueue<T> {
    head: AtomicPtr<QueueNode<T>>,
    tail: AtomicPtr<QueueNode<T>>,
}

impl<T> MpscQueue<T> {
    pub fn new() -> Self {
        let sentinel = Box::into_raw(Box::new(QueueNode {
            data: None,
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        Self {
            head: AtomicPtr::new(sentinel),
            tail: AtomicPtr::new(sentinel),
        }
    }

    pub fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(QueueNode {
            data: Some(data),
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        // Insert at tail
        loop {
            let tail = self.tail.load(Ordering::Acquire);

            unsafe {
                let next = (*tail).next.load(Ordering::Acquire);

                if next.is_null() {
                    // Tail is actually the last node
                    if (*tail)
                        .next
                        .compare_exchange(
                            ptr::null_mut(),
                            new_node,
                            Ordering::Release,
                            Ordering::Acquire,
                        )
                        .is_ok()
                    {
                        // Try to update tail (optional, helps next push)
                        let _ = self.tail.compare_exchange(
                            tail,
                            new_node,
                            Ordering::Release,
                            Ordering::Acquire,
                        );
                        break;
                    }
                } else {
                    // Help other threads by updating tail
                    let _ = self.tail.compare_exchange(
                        tail,
                        next,
                        Ordering::Release,
                        Ordering::Acquire,
                    );
                }
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        unsafe {
            let head = self.head.load(Ordering::Acquire);
            let next = (*head).next.load(Ordering::Acquire);

            if next.is_null() {
                return None;
            }

            // Move head forward
            self.head.store(next, Ordering::Release);

            // Take data from old sentinel
            let data = (*next).data.take();

            // Drop old sentinel (safe because we're single consumer)
            drop(Box::from_raw(head));

            data
        }
    }
}

unsafe impl<T: Send> Send for MpscQueue<T> {}
unsafe impl<T: Send> Sync for MpscQueue<T> {}

//=================================================================
// Real-world: Bounded SPSC queue (Single Producer Single Consumer)
//=================================================================
pub struct BoundedSpscQueue<T> {
    buffer: Vec<Option<T>>,
    head: AtomicUsize,
    tail: AtomicUsize,
    capacity: usize,
}

impl<T> BoundedSpscQueue<T> {
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(None);
        }

        Self {
            buffer,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            capacity,
        }
    }

    pub fn push(&mut self, data: T) -> Result<(), T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let next_tail = (tail + 1) % self.capacity;
        let head = self.head.load(Ordering::Acquire);

        if next_tail == head {
            return Err(data); // Queue full
        }

        unsafe {
            let slot = self.buffer.get_unchecked_mut(tail);
            *slot = Some(data);
        }

        self.tail.store(next_tail, Ordering::Release);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        if head == tail {
            return None; // Queue empty
        }

        unsafe {
            let slot = self.buffer.get_unchecked_mut(head);
            let data = slot.take();

            let next_head = (head + 1) % self.capacity;
            self.head.store(next_head, Ordering::Release);

            data
        }
    }

    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);

        if tail >= head {
            tail - head
        } else {
            self.capacity - head + tail
        }
    }
}

unsafe impl<T: Send> Send for BoundedSpscQueue<T> {}

fn main() {
    println!("=== MPSC Queue ===\n");

    let queue = Arc::new(MpscQueue::new());

    // Multiple producers
    let mut producers = vec![];
    for i in 0..5 {
        let queue = Arc::clone(&queue);
        producers.push(thread::spawn(move || {
            for j in 0..100 {
                queue.push(i * 100 + j);
            }
        }));
    }

    for p in producers {
        p.join().unwrap();
    }

    // Single consumer
    let mut count = 0;
    while queue.pop().is_some() {
        count += 1;
    }

    println!("Consumed {} items", count);

    println!("\n=== SPSC Queue ===\n");

    let mut producer_queue = BoundedSpscQueue::new(32);
    let mut consumer_queue = unsafe {
        // This is safe because we ensure only one thread accesses each
        std::ptr::read(&producer_queue as *const _)
    };

    let producer = thread::spawn(move || {
        for i in 0..100 {
            while producer_queue.push(i).is_err() {
                thread::yield_now();
            }
        }
    });

    let consumer = thread::spawn(move || {
        let mut sum = 0;
        let mut received = 0;

        while received < 100 {
            if let Some(value) = consumer_queue.pop() {
                sum += value;
                received += 1;
            } else {
                thread::yield_now();
            }
        }

        sum
    });

    producer.join().unwrap();
    let sum = consumer.join().unwrap();
    println!("Sum of 0..100: {}", sum);
}
```

**Queue Variants**:
- **MPSC**: Multi-producer, single-consumer
- **SPSC**: Single-producer, single-consumer (fastest)
- **MPMC**: Multi-producer, multi-consumer (hardest)
- **Bounded**: Fixed size, cache-friendly

---

## Hazard Pointers

**Problem**: Lock-free structures need memory reclamation—can't immediately free nodes because other threads might access them. Naive deletion causes use-after-free. Reference counting (Arc) adds overhead and doesn't solve the problem (threads can hold pointer after refcount=0). Garbage collection would solve it but Rust doesn't have GC. Memory leaks accumulate if nodes are never freed. ABA problem makes safe reclamation even harder.

**Solution**: Use hazard pointers to mark nodes as "in-use". Each thread announces pointers it's accessing. Before freeing a node, check if any thread has it in their hazard list. If protected, defer deletion to retired list. Periodically scan and reclaim nodes not in any hazard list. Alternative: epoch-based reclamation (like `crossbeam-epoch`) for better performance. Provides memory safety for lock-free structures.

**Why It Matters**: Prevents crashes in production lock-free code. Without proper reclamation, lock-free queue either leaks memory or crashes with use-after-free. Hazard pointers add ~10-20% overhead but enable correct lock-free algorithms. Crossbeam's epoch-based approach is faster (5-10% overhead). Real systems (databases, async runtimes) rely on this for correctness. Solves ABA problem completely—node can't be reused while protected.

**Use Cases**: Production lock-free stacks and queues, concurrent hash maps, lock-free linked lists, RCU-style updates, safe memory management without GC, building blocks for complex concurrent data structures.

### Pattern 7: Hazard Pointer Implementation

**Problem**: Safely reclaim memory in lock-free structures without use-after-free.

**Solution**:

```rust
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::ptr;
use std::collections::HashSet;

const MAX_HAZARDS: usize = 128;

struct HazardPointer {
    pointer: AtomicPtr<u8>,
}

impl HazardPointer {
    fn new() -> Self {
        Self {
            pointer: AtomicPtr::new(ptr::null_mut()),
        }
    }

    fn protect(&self, ptr: *mut u8) {
        self.pointer.store(ptr, Ordering::Release);
    }

    fn clear(&self) {
        self.pointer.store(ptr::null_mut(), Ordering::Release);
    }

    fn get(&self) -> *mut u8 {
        self.pointer.load(Ordering::Acquire)
    }
}

struct HazardPointerDomain {
    hazards: Vec<HazardPointer>,
    retired: AtomicPtr<RetiredNode>,
    retired_count: AtomicUsize,
}

struct RetiredNode {
    ptr: *mut u8,
    next: *mut RetiredNode,
    deleter: unsafe fn(*mut u8),
}

impl HazardPointerDomain {
    fn new() -> Self {
        let mut hazards = Vec::new();
        for _ in 0..MAX_HAZARDS {
            hazards.push(HazardPointer::new());
        }

        Self {
            hazards,
            retired: AtomicPtr::new(ptr::null_mut()),
            retired_count: AtomicUsize::new(0),
        }
    }

    fn acquire(&self) -> Option<usize> {
        for (i, hp) in self.hazards.iter().enumerate() {
            let current = hp.get();
            if current.is_null() {
                // Try to claim this hazard pointer
                if hp
                    .pointer
                    .compare_exchange(
                        ptr::null_mut(),
                        1 as *mut u8, // Non-null marker
                        Ordering::Acquire,
                        Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    return Some(i);
                }
            }
        }
        None
    }

    fn protect(&self, index: usize, ptr: *mut u8) {
        self.hazards[index].protect(ptr);
    }

    fn release(&self, index: usize) {
        self.hazards[index].clear();
    }

    fn retire(&self, ptr: *mut u8, deleter: unsafe fn(*mut u8)) {
        let node = Box::into_raw(Box::new(RetiredNode {
            ptr,
            next: ptr::null_mut(),
            deleter,
        }));

        // Add to retired list
        loop {
            let head = self.retired.load(Ordering::Acquire);
            unsafe {
                (*node).next = head;
            }

            if self
                .retired
                .compare_exchange_weak(head, node, Ordering::Release, Ordering::Acquire)
                .is_ok()
            {
                break;
            }
        }

        let count = self.retired_count.fetch_add(1, Ordering::Relaxed);

        // Trigger reclamation if too many retired
        if count > MAX_HAZARDS * 2 {
            self.scan();
        }
    }

    fn scan(&self) {
        // Collect all protected pointers
        let mut protected = HashSet::new();
        for hp in &self.hazards {
            let ptr = hp.get();
            if !ptr.is_null() && ptr != 1 as *mut u8 {
                protected.insert(ptr);
            }
        }

        // Try to reclaim retired nodes
        let mut current = self.retired.swap(ptr::null_mut(), Ordering::Acquire);
        let mut kept = Vec::new();

        unsafe {
            while !current.is_null() {
                let next = (*current).next;

                if protected.contains(&(*current).ptr) {
                    // Still protected, keep it
                    kept.push(current);
                } else {
                    // Safe to delete
                    ((*current).deleter)((*current).ptr);
                    drop(Box::from_raw(current));
                    self.retired_count.fetch_sub(1, Ordering::Relaxed);
                }

                current = next;
            }
        }

        // Re-add kept nodes
        for node in kept {
            loop {
                let head = self.retired.load(Ordering::Acquire);
                unsafe {
                    (*node).next = head;
                }

                if self
                    .retired
                    .compare_exchange_weak(head, node, Ordering::Release, Ordering::Acquire)
                    .is_ok()
                {
                    break;
                }
            }
        }
    }
}

//====================================
// Example: Stack with hazard pointers
//====================================
struct SafeNode<T> {
    data: T,
    next: *mut SafeNode<T>,
}

struct SafeStack<T> {
    head: AtomicPtr<SafeNode<T>>,
    hp_domain: HazardPointerDomain,
}

impl<T> SafeStack<T> {
    fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
            hp_domain: HazardPointerDomain::new(),
        }
    }

    fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(SafeNode {
            data,
            next: ptr::null_mut(),
        }));

        loop {
            let head = self.head.load(Ordering::Relaxed);
            unsafe {
                (*new_node).next = head;
            }

            if self
                .head
                .compare_exchange_weak(head, new_node, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    fn pop(&self) -> Option<T> {
        let hp_index = self.hp_domain.acquire()?;

        loop {
            let head = self.head.load(Ordering::Acquire);

            if head.is_null() {
                self.hp_domain.release(hp_index);
                return None;
            }

            // Protect head from deletion
            self.hp_domain.protect(hp_index, head as *mut u8);

            // Verify head hasn't changed (avoid ABA)
            if self.head.load(Ordering::Acquire) != head {
                continue;
            }

            unsafe {
                let next = (*head).next;

                if self
                    .head
                    .compare_exchange_weak(head, next, Ordering::Release, Ordering::Acquire)
                    .is_ok()
                {
                    let data = ptr::read(&(*head).data);

                    // Retire the node for later deletion
                    self.hp_domain.retire(head as *mut u8, |ptr| {
                        drop(Box::from_raw(ptr as *mut SafeNode<T>));
                    });

                    self.hp_domain.release(hp_index);
                    return Some(data);
                }
            }
        }
    }
}

unsafe impl<T: Send> Send for SafeStack<T> {}
unsafe impl<T: Send> Sync for SafeStack<T> {}

fn main() {
    println!("=== Safe Stack with Hazard Pointers ===\n");

    let stack = std::sync::Arc::new(SafeStack::new());
    let mut handles = vec![];

    // Producers
    for i in 0..5 {
        let stack = std::sync::Arc::clone(&stack);
        handles.push(std::thread::spawn(move || {
            for j in 0..1000 {
                stack.push(i * 1000 + j);
            }
        }));
    }

    // Consumers
    for _ in 0..5 {
        let stack = std::sync::Arc::clone(&stack);
        handles.push(std::thread::spawn(move || {
            let mut count = 0;
            while stack.pop().is_some() {
                count += 1;
            }
            count
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Stack operations completed safely");
}
```

**Hazard Pointer Benefits**:
- **Safe reclamation**: No use-after-free
- **Lock-free**: No blocking
- **ABA protection**: Version checking via protection
- **Memory efficient**: Bounded overhead

---

## Seqlock Pattern

**Problem**: Frequent reads of small data with occasional writes. Mutex too expensive (blocks readers). Reader-writer lock still has overhead and priority issues. Atomics insufficient for multi-field updates (coordinates, stats). Need consistency: read all fields from same write. CAS-based approaches complex for multiple fields. Want zero-cost reads in common case (no writes).

**Solution**: Use sequence counter incremented on writes. Writers: increment (odd), write data, increment (even). Readers: read sequence, read data, verify sequence unchanged. Retry if sequence odd (write in progress) or changed. Works only for `Copy` types (small data). Single writer model—multiple concurrent writers break it. Optimistic reads with validation. No locks, no CAS, just memory barriers.

**Why It Matters**: 10-100x faster than locks for read-heavy workloads. Game coordinates updated 60fps, read 10,000x/sec: seqlock enables this. Statistics dashboard: writes 1/sec, reads 1000/sec—perfect for seqlock. No blocking ever: predictable latency for real-time systems. Readers never wait: always make progress even during writes. Cache-friendly: sequential reads, minimal memory traffic. Powers Linux kernel read-mostly data structures.

**Use Cases**: Game entity positions/state, real-time sensor data, network statistics and metrics, configuration that changes rarely, performance counters, dashboard data, time-series snapshots, read-heavy caches.

### Pattern 8: Seqlock Implementation

**Problem**: Allow fast, lock-free reads with occasional writes for small data structures.

**Solution**:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::cell::UnsafeCell;

pub struct SeqLock<T> {
    seq: AtomicUsize,
    data: UnsafeCell<T>,
}

impl<T: Copy> SeqLock<T> {
    pub fn new(data: T) -> Self {
        Self {
            seq: AtomicUsize::new(0),
            data: UnsafeCell::new(data),
        }
    }

    pub fn read(&self) -> T {
        loop {
            // Read sequence number (even = not writing)
            let seq1 = self.seq.load(Ordering::Acquire);

            if seq1 % 2 == 1 {
                // Writer is active, spin
                std::hint::spin_loop();
                continue;
            }

            // Read data
            let data = unsafe { *self.data.get() };

            // Verify sequence hasn't changed
            std::sync::atomic::fence(Ordering::Acquire);
            let seq2 = self.seq.load(Ordering::Acquire);

            if seq1 == seq2 {
                return data;
            }

            // Sequence changed during read, retry
        }
    }

    pub fn write(&self, data: T) {
        // Increment sequence (makes it odd = writing)
        let seq = self.seq.fetch_add(1, Ordering::Acquire);
        debug_assert!(seq % 2 == 0, "Concurrent writes detected");

        // Write data
        unsafe {
            *self.data.get() = data;
        }

        // Increment again (makes it even = readable)
        self.seq.fetch_add(1, Ordering::Release);
    }

    pub fn try_read(&self) -> Option<T> {
        let seq1 = self.seq.load(Ordering::Acquire);

        if seq1 % 2 == 1 {
            return None; // Writer active
        }

        let data = unsafe { *self.data.get() };

        std::sync::atomic::fence(Ordering::Acquire);
        let seq2 = self.seq.load(Ordering::Acquire);

        if seq1 == seq2 {
            Some(data)
        } else {
            None // Data changed
        }
    }
}

unsafe impl<T: Copy + Send> Send for SeqLock<T> {}
unsafe impl<T: Copy + Send> Sync for SeqLock<T> {}

//=====================================
// Real-world: Coordinates with seqlock
//=====================================
#[derive(Copy, Clone, Debug)]
struct Coordinates {
    x: f64,
    y: f64,
    z: f64,
}

fn seqlock_coordinates_example() {
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    let position = Arc::new(SeqLock::new(Coordinates {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }));

    // Writer thread (updates position)
    let writer_pos = Arc::clone(&position);
    let writer = thread::spawn(move || {
        for i in 0..100 {
            let coords = Coordinates {
                x: i as f64,
                y: (i * 2) as f64,
                z: (i * 3) as f64,
            };
            writer_pos.write(coords);
            thread::sleep(Duration::from_millis(10));
        }
    });

    // Reader threads (read position frequently)
    let mut readers = vec![];
    for id in 0..5 {
        let reader_pos = Arc::clone(&position);
        readers.push(thread::spawn(move || {
            for _ in 0..1000 {
                let coords = reader_pos.read();
                if id == 0 && coords.x as usize % 10 == 0 {
                    println!("Reader {}: {:?}", id, coords);
                }
            }
        }));
    }

    writer.join().unwrap();
    for reader in readers {
        reader.join().unwrap();
    }
}

//================================
// Real-world: Statistics snapshot
//================================
#[derive(Copy, Clone, Debug)]
struct Stats {
    count: u64,
    sum: u64,
    min: u64,
    max: u64,
}

impl Stats {
    fn new() -> Self {
        Self {
            count: 0,
            sum: 0,
            min: u64::MAX,
            max: 0,
        }
    }

    fn add(&mut self, value: u64) {
        self.count += 1;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    fn average(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum as f64 / self.count as f64
        }
    }
}

fn seqlock_stats_example() {
    use std::sync::Arc;
    use std::thread;

    let stats = Arc::new(SeqLock::new(Stats::new()));

    // Writer thread
    let writer_stats = Arc::clone(&stats);
    let writer = thread::spawn(move || {
        for i in 0..1000 {
            let mut current = writer_stats.read();
            current.add(i);
            writer_stats.write(current);
        }
    });

    // Reader threads (monitor stats)
    let mut readers = vec![];
    for id in 0..3 {
        let reader_stats = Arc::clone(&stats);
        readers.push(thread::spawn(move || {
            for _ in 0..100 {
                thread::sleep(std::time::Duration::from_millis(10));
                let snapshot = reader_stats.read();
                if id == 0 {
                    println!(
                        "Stats - Count: {}, Avg: {:.2}, Min: {}, Max: {}",
                        snapshot.count,
                        snapshot.average(),
                        snapshot.min,
                        snapshot.max
                    );
                }
            }
        }));
    }

    writer.join().unwrap();
    for reader in readers {
        reader.join().unwrap();
    }
}

//==========================================
// Pattern: Versioned seqlock (track writes)
//==========================================
pub struct VersionedSeqLock<T> {
    seqlock: SeqLock<T>,
}

impl<T: Copy> VersionedSeqLock<T> {
    pub fn new(data: T) -> Self {
        Self {
            seqlock: SeqLock::new(data),
        }
    }

    pub fn read_with_version(&self) -> (T, usize) {
        let seq1 = self.seqlock.seq.load(Ordering::Acquire);
        let data = self.seqlock.read();
        let version = seq1 / 2;
        (data, version)
    }

    pub fn write(&self, data: T) {
        self.seqlock.write(data);
    }

    pub fn version(&self) -> usize {
        self.seqlock.seq.load(Ordering::Acquire) / 2
    }
}

fn main() {
    println!("=== Seqlock Coordinates ===\n");
    seqlock_coordinates_example();

    println!("\n=== Seqlock Statistics ===\n");
    seqlock_stats_example();

    println!("\n=== Versioned Seqlock ===\n");

    let data = VersionedSeqLock::new(0u64);

    for i in 0..5 {
        data.write(i * 10);
        let (value, version) = data.read_with_version();
        println!("Value: {}, Version: {}", value, version);
    }
}
```

**Seqlock Characteristics**:
- **Optimistic reads**: No locks for readers
- **Single writer**: Only one writer at a time
- **Small data**: Works best with Copy types
- **Retry on write**: Readers retry if writer was active
- **Use case**: Frequently read, rarely written data (coordinates, stats)

---

### Pattern 9: Advanced Atomic Patterns

**Problem**: Implement specialized concurrent patterns using atomics.

**Solution**:

```rust
use std::sync::atomic::{AtomicU64, AtomicUsize, AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

//===============================================
// Pattern 1: Striped counter (reduce contention)
//===============================================
struct StripedCounter {
    stripes: Vec<AtomicUsize>,
}

impl StripedCounter {
    fn new(num_stripes: usize) -> Self {
        let mut stripes = Vec::new();
        for _ in 0..num_stripes {
            stripes.push(AtomicUsize::new(0));
        }

        Self { stripes }
    }

    fn increment(&self) {
        let thread_id = std::thread::current().id();
        let index = format!("{:?}", thread_id).len() % self.stripes.len();
        self.stripes[index].fetch_add(1, Ordering::Relaxed);
    }

    fn get(&self) -> usize {
        self.stripes
            .iter()
            .map(|s| s.load(Ordering::Relaxed))
            .sum()
    }
}

//===============================
// Pattern 2: Exponential backoff
//===============================
struct Backoff {
    current: Duration,
    max: Duration,
}

impl Backoff {
    fn new() -> Self {
        Self {
            current: Duration::from_nanos(1),
            max: Duration::from_micros(1000),
        }
    }

    fn spin(&mut self) {
        for _ in 0..(self.current.as_nanos() / 10) {
            std::hint::spin_loop();
        }

        self.current = (self.current * 2).min(self.max);
    }

    fn reset(&mut self) {
        self.current = Duration::from_nanos(1);
    }
}

fn cas_with_backoff(counter: &AtomicUsize) {
    let mut backoff = Backoff::new();

    loop {
        let current = counter.load(Ordering::Relaxed);

        match counter.compare_exchange_weak(
            current,
            current + 1,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => {
                backoff.reset();
                break;
            }
            Err(_) => {
                backoff.spin();
            }
        }
    }
}

//==========================
// Pattern 3: Atomic min/max
//==========================
struct AtomicMinMax {
    min: AtomicU64,
    max: AtomicU64,
}

impl AtomicMinMax {
    fn new() -> Self {
        Self {
            min: AtomicU64::new(u64::MAX),
            max: AtomicU64::new(0),
        }
    }

    fn update(&self, value: u64) {
        // Update min
        let mut current_min = self.min.load(Ordering::Relaxed);
        while value < current_min {
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
        while value > current_max {
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

    fn get(&self) -> (u64, u64) {
        (
            self.min.load(Ordering::Relaxed),
            self.max.load(Ordering::Relaxed),
        )
    }
}

//========================================
// Pattern 4: Once flag for initialization
//========================================
struct OnceFlag {
    state: AtomicUsize,
}

const INCOMPLETE: usize = 0;
const RUNNING: usize = 1;
const COMPLETE: usize = 2;

impl OnceFlag {
    fn new() -> Self {
        Self {
            state: AtomicUsize::new(INCOMPLETE),
        }
    }

    fn call_once<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        if self.state.load(Ordering::Acquire) == COMPLETE {
            return;
        }

        match self.state.compare_exchange(
            INCOMPLETE,
            RUNNING,
            Ordering::Acquire,
            Ordering::Acquire,
        ) {
            Ok(_) => {
                // We won the race
                f();
                self.state.store(COMPLETE, Ordering::Release);
            }
            Err(RUNNING) => {
                // Someone else is running, wait
                while self.state.load(Ordering::Acquire) == RUNNING {
                    std::hint::spin_loop();
                }
            }
            Err(COMPLETE) => {
                // Already done
            }
            _ => unreachable!(),
        }
    }

    fn is_completed(&self) -> bool {
        self.state.load(Ordering::Acquire) == COMPLETE
    }
}

//=============================
// Pattern 5: Atomic swap chain
//=============================
struct SwapChain<T> {
    value: AtomicUsize, // Actually *mut T
    _phantom: std::marker::PhantomData<T>,
}

impl<T> SwapChain<T> {
    fn new(initial: T) -> Self {
        let ptr = Box::into_raw(Box::new(initial));
        Self {
            value: AtomicUsize::new(ptr as usize),
            _phantom: std::marker::PhantomData,
        }
    }

    fn swap(&self, new_value: T) -> T {
        let new_ptr = Box::into_raw(Box::new(new_value));
        let old_ptr = self.value.swap(new_ptr as usize, Ordering::AcqRel) as *mut T;

        unsafe {
            let old_value = std::ptr::read(old_ptr);
            drop(Box::from_raw(old_ptr));
            old_value
        }
    }

    fn load(&self) -> T
    where
        T: Clone,
    {
        let ptr = self.value.load(Ordering::Acquire) as *mut T;
        unsafe { (*ptr).clone() }
    }
}

impl<T> Drop for SwapChain<T> {
    fn drop(&mut self) {
        let ptr = self.value.load(Ordering::Acquire) as *mut T;
        if !ptr.is_null() {
            unsafe {
                drop(Box::from_raw(ptr));
            }
        }
    }
}

fn main() {
    println!("=== Striped Counter ===\n");

    let counter = Arc::new(StripedCounter::new(16));
    let mut handles = vec![];

    let start = Instant::now();

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..100_000 {
                counter.increment();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Count: {} in {:?}", counter.get(), start.elapsed());

    println!("\n=== Backoff ===\n");

    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..10_000 {
                cas_with_backoff(&counter);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Count: {}", counter.load(Ordering::Relaxed));

    println!("\n=== Atomic Min/Max ===\n");

    let minmax = Arc::new(AtomicMinMax::new());
    let mut handles = vec![];

    for i in 0..10 {
        let minmax = Arc::clone(&minmax);
        handles.push(thread::spawn(move || {
            for j in 0..1000 {
                minmax.update(i * 1000 + j);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let (min, max) = minmax.get();
    println!("Min: {}, Max: {}", min, max);

    println!("\n=== Once Flag ===\n");

    let once = Arc::new(OnceFlag::new());
    let mut handles = vec![];

    for i in 0..10 {
        let once = Arc::clone(&once);
        handles.push(thread::spawn(move || {
            once.call_once(|| {
                println!("Initialization by thread {}", i);
            });
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Completed: {}", once.is_completed());
}
```

**Advanced Patterns**:
- **Striped counter**: Reduce contention by spreading across multiple atomics
- **Backoff**: Exponential delay on contention
- **Atomic min/max**: Track extremes lock-free
- **Once flag**: One-time initialization
- **Swap chain**: Atomic value replacement

---

## Summary

This chapter covered atomic operations and lock-free programming:

1. **Memory Ordering**: Relaxed, Acquire/Release, AcqRel, SeqCst with use cases
2. **Compare-and-Swap**: CAS loops, weak vs strong, ABA problem and solutions
3. **Lock-Free Structures**: Treiber stack, MPSC/SPSC queues, work-stealing deques
4. **Hazard Pointers**: Safe memory reclamation without garbage collection
5. **Seqlock**: Optimistic reads for small, frequently-read data

**Key Takeaways**:
- **Memory ordering** is critical for correctness
- **Relaxed** for counters, **Acquire/Release** for synchronization, **SeqCst** for simplicity
- **CAS** is the foundation of lock-free algorithms
- **ABA problem** requires version counters or hazard pointers
- **Lock-free** != faster always (measure performance)
- **Seqlock** excels for read-heavy small data

**Performance Guidelines**:
- Use **Relaxed** when order doesn't matter (fastest)
- **Acquire/Release** for most synchronization (good balance)
- **SeqCst** when correctness is critical (slowest)
- Striped counters reduce contention
- Backoff reduces CPU waste during contention
- Lock-free shines under high contention

**Common Pitfalls**:
- Wrong memory ordering (too weak = race, too strong = slow)
- ABA problem (use versioning or hazard pointers)
- Memory leaks (need reclamation strategy)
- Assuming lock-free = faster (profile!)
- Over-using atomics (sometimes locks are simpler)

**When to Use**:
- **Atomics**: Counters, flags, simple synchronization
- **Lock-free**: High contention, real-time constraints
- **Locks**: Complex operations, simplicity, most cases
- **Seqlock**: Coordinates, stats, small frequently-read data

**Safety**:
- Rust's type system prevents most data races
- Atomic operations are safe
- Raw pointers in lock-free structures require unsafe
- Use existing libraries (crossbeam) when possible
