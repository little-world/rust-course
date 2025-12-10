# Chapter 16: Atomic Operations - Project 3

## Project 3: Wait-Free Ring Buffer (Bounded Queue)

### Problem Statement

Implement a wait-free ring buffer (circular queue) for efficient producer-consumer communication. Start with SPSC (Single-Producer Single-Consumer), then extend to MPMC (Multi-Producer Multi-Consumer). The ring buffer uses fixed-size array with atomic head/tail pointers, avoiding allocations and providing bounded memory.

The implementation must handle:
- Full buffer detection (producer must wait or fail)
- Empty buffer detection (consumer returns None)
- Cache-line alignment to avoid false sharing
- Memory ordering for cross-thread visibility
- Proper wrapping arithmetic for circular indexing

### Use Cases

- Audio/video streaming pipelines (producer writes samples, consumer plays them)
- Network packet processing (NIC writes packets, application reads them)
- Async runtime task queues (spawn writes tasks, executor reads them)
- Game engine event queues (input thread writes events, game loop reads them)
- IPC (Inter-Process Communication) shared memory queues
- Logging systems (application threads write logs, background thread flushes them)

### Why It Matters

**Performance Comparison:**
- Mutex + VecDeque: ~100-500ns per operation (kernel involvement, allocation)
- SPSC Ring Buffer: ~10-30ns per operation (pure atomics, no allocation)
- MPMC Ring Buffer: ~50-150ns per operation (CAS contention)

**Wait-Free vs Lock-Free:**
- Lock-free: At least one thread makes progress (CAS loops can starve)
- Wait-free: Every operation completes in bounded steps (SPSC is wait-free!)

**False Sharing Problem:**
```
CPU 0 (Producer):          CPU 1 (Consumer):
Writes to head         →   Reads from tail
        ↓                       ↓
    [Cache Line]  ← Invalidated on every write!
    [head | tail]
```

Solution: Pad head and tail to separate cache lines (64 bytes on x86).

**Real-World Usage:**
- Linux kernel: kfifo (kernel FIFO queue)
- DPDK: rte_ring (high-performance packet queue)
- LMAX Disruptor: Java ring buffer for trading systems (millions of ops/sec)
- Crossbeam: `ArrayQueue` (Rust lock-free bounded queue)

---

## Key Concepts Explained

This project requires understanding ring buffer mechanics, circular indexing, memory ordering for producer-consumer patterns, cache-line optimization, and wait-free algorithms. These concepts enable building the highest-performance concurrent queues possible.

### Ring Buffer: The Circular Queue Data Structure

**What It Is**: A fixed-size queue implemented as a circular array with wrap-around indexing.

**Why Circular?**

Linear queue has a problem:

```
Linear Queue (grows to the right):
[_][_][_][A][B][C][_][_]
       ↑           ↑
     tail        head

After many enqueue/dequeue:
[_][_][_][_][_][_][_][X]  ← At end! Must shift all elements or reallocate
                       ↑
                     head

Problem: Eventually runs out of space despite empty slots at front
```

Ring buffer solves this with wrap-around:

```
Ring Buffer (wraps around):
[D][_][_][A][B][C][_][_]
 ↑       ↑
head    tail

Indices wrap: 7 → 0, allowing infinite enqueue/dequeue
```

**Visual Representation**:

```
Circular view:
       ┌───┬───┬───┬───┐
       │ 0 │ 1 │ 2 │ 3 │
       └───┴───┴───┴───┘
      ╱                 ╲
    ╱                     ╲
  ╱                         ╲
 │                           │
 │ 7                       4 │
 │                           │
  ╲                         ╱
    ╲                     ╱
      ╲                 ╱
       └───┴───┴───┴───┘
       │ 6 │ 5 │ 4 │   │

State: tail=2, head=6
Contains: [2][3][4][5]
```

**Three States**:

```
1. Empty: head == tail
   [_][_][_][_]
    ↑
  head,tail

2. Partially filled: head != tail
   [_][B][C][_]
    ↑       ↑
   tail    head

3. Full: (head + 1) % capacity == tail
   [D][B][C][_]  ← Reserve one slot to distinguish from empty
       ↑    ↑
      tail head
```

**Why Reserve One Slot?**

Without reservation:

```
Full state: head == tail (same as empty!)  ← AMBIGUOUS!
[A][B][C][D]
 ↑
head,tail

Can't tell if empty or full without extra state
```

With reservation:

```
Full: (head + 1) % capacity == tail
[D][B][C][_]  ← One empty slot
    ↑    ↑
   tail head

Empty: head == tail
[_][_][_][_]
 ↑
head,tail

No ambiguity! Cost: waste 1 slot
```

---

### Circular Indexing Arithmetic

**The Core Operations**: Map linear indices to circular array positions.

**Naive Modulo**:

```rust
let index = head % capacity;
```

**Problem**: Modulo is slow (~20-30 cycles on x86)

**Fast Modulo (Power-of-2 Optimization)**:

```rust
// If capacity is power of 2 (e.g., 8, 16, 32, 64):
let mask = capacity - 1;  // 16 → 0b1111
let index = head & mask;   // Bitwise AND (~1 cycle!)

// Example:
// capacity = 16 (0b10000)
// mask = 15 (0b01111)
// head = 23 (0b10111)
// index = 23 & 15 = 7 (0b00111)
```

**Wrapping Increment**:

```rust
// Naive:
head = (head + 1) % capacity;  // Slow

// Optimized (power-of-2):
head = (head + 1) & mask;  // Fast

// Even more optimized: let it wrap naturally
head = head.wrapping_add(1);  // u32 wraps at 2^32
let index = head & mask;
```

**Why Natural Wrapping Works**:

```rust
// With u32 head and capacity=16:
head: 0 → 1 → 2 → ... → 4,294,967,295 → 0 (wraps)
index = head & 15: Always in [0, 15]

// No need for explicit modulo!
// head never needs to be reset to 0
```

**Full Buffer Check**:

```rust
// Naive:
fn is_full(&self) -> bool {
    (self.head.load(Ordering::Relaxed) + 1) % self.capacity
        == self.tail.load(Ordering::Relaxed)
}

// Optimized (natural wrapping):
fn is_full(&self) -> bool {
    self.head.load(Ordering::Relaxed).wrapping_add(1) & self.mask
        == self.tail.load(Ordering::Relaxed) & self.mask
}
```

**Length Calculation**:

```rust
// Naive:
let len = if head >= tail {
    head - tail
} else {
    capacity - tail + head
};

// Optimized (wrapping subtraction):
let len = head.wrapping_sub(tail) & mask;

// Example:
// capacity = 16, mask = 15
// head = 3, tail = 14
// len = 3 - 14 = -11 (wraps to 4,294,967,285 in u32)
// len & 15 = 5 ✓ (correct: 14,15,0,1,2,3)
```

---

### MaybeUninit: Uninitialized Memory

**The Problem**: Ring buffer allocates fixed capacity upfront, but slots are unused until written.

**Wrong Approach**:

```rust
struct RingBuffer<T> {
    buffer: Vec<T>,  // ❌ Requires T: Default or complex initialization
}

impl<T> RingBuffer<T> {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![Default::default(); capacity],  // ❌ Unnecessary work
        }
    }
}
```

**Problems**:
- Requires `T: Default` (unnecessary constraint)
- Initializes all slots (wasted work)
- If `T` is expensive to create, this is very slow

**Correct Approach: MaybeUninit**:

```rust
use std::mem::MaybeUninit;

struct RingBuffer<T> {
    buffer: Vec<MaybeUninit<T>>,  // ✅ Uninitialized memory
}

impl<T> RingBuffer<T> {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: (0..capacity)
                .map(|_| MaybeUninit::uninit())
                .collect(),
        }
    }

    fn push(&self, value: T) -> Result<(), T> {
        // ... get index ...
        unsafe {
            // Write to uninitialized slot
            (*self.buffer.get_unchecked(index).as_ptr()) = value;
        }
        Ok(())
    }

    fn pop(&self) -> Option<T> {
        // ... get index ...
        unsafe {
            // Read from initialized slot
            Some(self.buffer.get_unchecked(index).as_ptr().read())
        }
    }
}
```

**MaybeUninit Operations**:

```rust
// Create uninitialized
let mut slot: MaybeUninit<T> = MaybeUninit::uninit();

// Write (initialize)
slot.write(value);

// Read (assume initialized)
unsafe {
    let value = slot.assume_init();  // Consumes MaybeUninit
    // or
    let value = slot.assume_init_read();  // Reads without consuming
}

// Get raw pointer for manual management
let ptr: *mut T = slot.as_mut_ptr();
unsafe {
    ptr.write(value);  // Initialize via pointer
    let value = ptr.read();  // Read via pointer
}
```

**Safety Considerations**:

```rust
// ✅ SAFE: Write before read
let mut slot = MaybeUninit::uninit();
slot.write(42);
let value = unsafe { slot.assume_init() };  // OK

// ❌ UNSAFE: Read uninitialized
let slot = MaybeUninit::<i32>::uninit();
let value = unsafe { slot.assume_init() };  // ❌ UNDEFINED BEHAVIOR!

// ✅ SAFE: Track initialization state
let mut slot = MaybeUninit::uninit();
let initialized = false;

if some_condition {
    slot.write(42);
    initialized = true;
}

if initialized {
    let value = unsafe { slot.assume_init() };  // OK
}
```

**Why This Matters for Ring Buffer**:

```
Capacity 1024 ring buffer:
- With Vec<T>: Initialize all 1024 elements upfront
- With Vec<MaybeUninit<T>>: No initialization, O(1) creation

For expensive T (e.g., Vec<u8>):
- Vec<Vec<u8>>: 1024 allocations
- Vec<MaybeUninit<Vec<u8>>>: 0 allocations
```

---

### SPSC vs MPMC: Producer-Consumer Patterns

#### SPSC: Single-Producer Single-Consumer

**Characteristics**:
- One writer thread, one reader thread
- **No contention**: Writer owns head, reader owns tail
- **Wait-free**: Operations complete in bounded time
- **Fastest**: ~10-30ns per operation

**Why It's Wait-Free**:

```rust
// Producer:
fn push(&self, value: T) -> Result<(), T> {
    let head = self.head.load(Ordering::Relaxed);  // Only I write head
    let tail = self.tail.load(Ordering::Acquire);  // Reader updates tail

    if (head + 1) & self.mask == tail & self.mask {
        return Err(value);  // Full - deterministic failure
    }

    unsafe {
        self.buffer[head & self.mask].write(value);
    }

    self.head.store(head + 1, Ordering::Release);  // Publish
    Ok(())
}

// No CAS, no loops, no retries → Wait-free!
```

**Memory Ordering**:
- **Producer**: `Acquire` tail, `Release` head
- **Consumer**: `Acquire` head, `Release` tail
- Creates synchronization between producer and consumer

#### MPMC: Multi-Producer Multi-Consumer

**Characteristics**:
- Multiple writers, multiple readers
- **Contention**: Multiple threads compete for head/tail
- **Lock-free**: Uses CAS, can retry, but always progresses
- **Slower**: ~50-150ns per operation (CAS overhead)

**Why It Needs CAS**:

```rust
// Multiple producers competing:
fn push(&self, value: T) -> Result<(), T> {
    loop {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        if (head + 1) & self.mask == tail & self.mask {
            return Err(value);  // Full
        }

        // Try to claim slot via CAS
        if self.head.compare_exchange_weak(
            head,
            head + 1,
            Ordering::Release,
            Ordering::Relaxed,
        ).is_ok() {
            // Success! We claimed slot at index head
            unsafe {
                self.buffer[head & self.mask].write(value);
            }
            return Ok(());
        }
        // Failed - another producer claimed it, retry
    }
}

// CAS loop → Lock-free (not wait-free)
```

**Comparison**:

| Aspect | SPSC | MPMC |
|--------|------|------|
| **Threads** | 1 producer, 1 consumer | N producers, M consumers |
| **Contention** | None | High |
| **Operations** | Load/Store | CAS loops |
| **Progress** | Wait-free | Lock-free |
| **Latency** | ~10-30ns | ~50-150ns |
| **Throughput** | Excellent | Good |
| **Complexity** | Simple | Complex |

---

### Memory Ordering for Producer-Consumer

**The Critical Synchronization**: Producer writes data, consumer must see it.

**SPSC Memory Ordering**:

```rust
// Producer thread:
fn push(&self, value: T) -> Result<(), T> {
    let head = self.head.load(Ordering::Relaxed);
    let tail = self.tail.load(Ordering::Acquire);  // ← Acquire tail

    // ... full check ...

    unsafe {
        self.buffer[head & self.mask].write(value);  // Write data
    }

    self.head.store(head + 1, Ordering::Release);  // ← Release head
    Ok(())
}

// Consumer thread:
fn pop(&self) -> Option<T> {
    let tail = self.tail.load(Ordering::Relaxed);
    let head = self.head.load(Ordering::Acquire);  // ← Acquire head

    if head & self.mask == tail & self.mask {
        return None;  // Empty
    }

    unsafe {
        let value = self.buffer[tail & self.mask].read();  // Read data

        self.tail.store(tail + 1, Ordering::Release);  // ← Release tail
        Some(value)
    }
}
```

**Why This Ordering?**

**Producer**:
1. **Acquire tail**: See consumer's latest consumed index
2. Write data to buffer slot
3. **Release head**: Publish new data to consumer

**Consumer**:
1. **Acquire head**: See producer's latest produced data
2. Read data from buffer slot
3. **Release tail**: Publish consumed index to producer

**Synchronization Edges**:

```
Producer Thread                Consumer Thread
───────────────                ───────────────
Load tail (Acquire)  ←─────────┐
                               │ Synchronizes
Write data                     │
                               │
Store head (Release) ─────────→│
                               │
                               └──→ Load head (Acquire)

                                    Read data

                               ┌─── Store tail (Release)
                               │
                               │ Synchronizes
Load tail (Acquire)  ←─────────┘
```

**What Happens Without Correct Ordering?**

```rust
// BAD: Using Relaxed everywhere
self.buffer[head & self.mask].write(value);  // Write data
self.head.store(head + 1, Ordering::Relaxed);  // ❌ No synchronization!

// Consumer:
let head = self.head.load(Ordering::Relaxed);  // ❌ Might not see data write!
let value = self.buffer[tail & self.mask].read();  // ❌ GARBAGE DATA!
```

**Relaxed Optimization (when safe)**:

```rust
// Can use Relaxed for own index:
let head = self.head.load(Ordering::Relaxed);  // ✅ Only I write head
let tail = self.tail.load(Ordering::Acquire);   // ✅ Consumer writes tail
```

---

### False Sharing and Cache Line Alignment

**The Problem**: Head and tail in same cache line causes ping-pong.

**False Sharing Scenario**:

```rust
struct RingBuffer {
    head: AtomicUsize,  // Bytes 0-7
    tail: AtomicUsize,  // Bytes 8-15  ← Same 64-byte cache line!
    buffer: Vec<MaybeUninit<T>>,
}

// CPU 0 (Producer):
self.head.store(new_head, Ordering::Release);
// → Marks entire cache line as modified
// → CPU 1's cache line invalidated

// CPU 1 (Consumer):
let head = self.head.load(Ordering::Acquire);
// → Must reload cache line from CPU 0
// → Then stores to tail...
self.tail.store(new_tail, Ordering::Release);
// → Marks entire cache line as modified
// → CPU 0's cache line invalidated

// Ping-pong continues → 10-100x slowdown!
```

**Performance Impact**:

```
Without padding (false sharing):
  SPSC throughput: 50 million ops/sec

With padding (separate cache lines):
  SPSC throughput: 500 million ops/sec

10x improvement from cache line alignment!
```

**Solution: Cache Line Padding**:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

// Cache line size: 64 bytes on x86, 128 bytes on some ARM
const CACHE_LINE_SIZE: usize = 64;

#[repr(align(64))]
struct CacheAligned<T> {
    value: T,
    _padding: [u8; CACHE_LINE_SIZE - std::mem::size_of::<T>()],
}

struct RingBuffer<T> {
    head: CacheAligned<AtomicUsize>,  // Bytes 0-63
    tail: CacheAligned<AtomicUsize>,  // Bytes 64-127  ← Different cache line!
    buffer: Vec<MaybeUninit<T>>,
}

// Now producer and consumer operate on separate cache lines
// No invalidation ping-pong!
```

**Alternative: Separate Structs**:

```rust
#[repr(align(64))]
struct ProducerData {
    head: AtomicUsize,
    _padding: [u8; 56],  // 64 - 8 = 56
}

#[repr(align(64))]
struct ConsumerData {
    tail: AtomicUsize,
    _padding: [u8; 56],
}

struct RingBuffer<T> {
    producer: ProducerData,
    consumer: ConsumerData,
    buffer: Vec<MaybeUninit<T>>,
}
```

**When Padding Matters**:
- ✅ High-frequency updates (SPSC/MPMC)
- ✅ Different threads updating different fields
- ❌ Single-threaded access
- ❌ Infrequent updates

---

### Power-of-2 Capacity Optimization

**Why Power of 2?**

Fast bitwise operations instead of slow division/modulo.

**Comparison**:

```rust
// Non-power-of-2 capacity (e.g., 100):
let index = head % capacity;  // ~20-30 cycles (division)

// Power-of-2 capacity (e.g., 128):
let mask = capacity - 1;  // 128 - 1 = 127 = 0b01111111
let index = head & mask;  // ~1 cycle (bitwise AND)

// 20-30x faster!
```

**Implementation**:

```rust
impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        // Round up to next power of 2
        let capacity = capacity.next_power_of_two();
        let mask = capacity - 1;

        Self {
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            buffer: (0..capacity).map(|_| MaybeUninit::uninit()).collect(),
            mask,
        }
    }

    fn push(&self, value: T) -> Result<(), T> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        // Fast modulo using bitwise AND
        if (head.wrapping_add(1) & self.mask) == (tail & self.mask) {
            return Err(value);  // Full
        }

        unsafe {
            self.buffer.get_unchecked(head & self.mask).write(value);
        }

        self.head.store(head.wrapping_add(1), Ordering::Release);
        Ok(())
    }
}
```

**Why `wrapping_add`?**

```rust
// Let head naturally wrap at u32/u64 boundary:
let head: u32 = 4_294_967_295;  // Max u32
let new_head = head.wrapping_add(1);  // → 0 (wraps)

// Index is always correct due to mask:
let index = new_head & mask;  // Works correctly after wrap
```

**Benefits**:
- No explicit modulo operations
- No manual wrap checking
- Simpler code
- Faster execution

---

### Wait-Free Guarantees

**Definitions**:

**Lock-Free**: System-wide progress (at least one thread always advances)
```rust
// Lock-free (CAS loop):
loop {
    let current = atomic.load(Ordering::Relaxed);
    if atomic.compare_exchange_weak(...).is_ok() {
        break;  // Success
    }
    // Individual thread might loop forever (starvation)
    // But at least one thread always succeeds → system progresses
}
```

**Wait-Free**: Per-thread progress (every thread completes in bounded steps)
```rust
// Wait-free (no loops):
let head = self.head.load(Ordering::Relaxed);  // Step 1
let tail = self.tail.load(Ordering::Acquire);  // Step 2

if is_full(head, tail) {
    return Err(value);  // Step 3 - deterministic failure
}

write_buffer(value);  // Step 4
self.head.store(new_head, Ordering::Release);  // Step 5

// Exactly 5 steps, no loops, no retries → Wait-free!
```

**Why SPSC is Wait-Free**:

```
Producer owns head:
  - Only producer writes head
  - Only consumer reads head
  - No contention → No CAS needed

Consumer owns tail:
  - Only consumer writes tail
  - Only producer reads tail
  - No contention → No CAS needed

Result: Simple load/store operations, bounded steps
```

**Why MPMC is NOT Wait-Free**:

```
Multiple producers compete for head:
  - Thread A: CAS(head, 5 → 6)
  - Thread B: CAS(head, 5 → 6)

Only one succeeds:
  - Winner: Advances in 1 attempt
  - Loser: Must retry with updated head

Retry unbounded → Lock-free, not wait-free
```

**Performance Implications**:

| Property | Lock-Free (MPMC) | Wait-Free (SPSC) |
|----------|------------------|------------------|
| **Latency** | Variable (CAS retries) | Constant |
| **Worst-case** | Unbounded retries | Bounded steps |
| **Best-case** | 1 CAS (~10ns) | 1 load/store (~5ns) |
| **Predictability** | Low | High |
| **Real-time** | Not suitable | Suitable |

---

### Connection to This Project

Now that you understand the core concepts, here's how they map to the milestones:

**Milestone 1: Basic SPSC Ring Buffer**
- **Concepts Used**: Ring buffer structure, circular indexing, `MaybeUninit`, atomic load/store
- **Why**: Establish foundation of circular queue and atomic indices
- **Key Insight**: SPSC is simple—no CAS needed, just load/store with proper ordering

**Milestone 2: Memory Ordering Optimization**
- **Concepts Used**: Acquire/Release ordering, producer-consumer synchronization
- **Why**: Correct ordering ensures consumer sees producer's data
- **Key Insight**: Release on write index, Acquire on read index creates sync edge

**Milestone 3: Cache Line Alignment**
- **Concepts Used**: False sharing, cache line padding, `#[repr(align(64))]`
- **Why**: Separate cache lines eliminate ping-pong between producer and consumer
- **Key Insight**: 64-byte padding can give 10x performance improvement

**Milestone 4: Power-of-2 Capacity Optimization**
- **Concepts Used**: Bitwise AND for fast modulo, `wrapping_add`, mask calculation
- **Why**: Avoid slow division operations
- **Key Insight**: `head & mask` is 20-30x faster than `head % capacity`

**Milestone 5: MPMC with CAS**
- **Concepts Used**: CAS loops for contention, lock-free (not wait-free), fetch_add
- **Why**: Multiple producers/consumers require atomic claim of slots
- **Key Insight**: CAS enables lock-free MPMC but loses wait-free guarantee

**Milestone 6: Benchmarking and Validation**
- **Concepts Used**: Throughput measurement, latency histograms, contention testing
- **Why**: Validate performance claims and understand trade-offs
- **Key Insight**: SPSC 10x faster than MPMC, both much faster than Mutex

**Putting It All Together**:

The complete ring buffer demonstrates:
1. **Circular indexing** with power-of-2 optimization
2. **Uninitialized memory** with `MaybeUninit`
3. **Memory ordering** (Acquire/Release) for producer-consumer sync
4. **Cache line alignment** to eliminate false sharing
5. **Wait-free SPSC** vs **lock-free MPMC** trade-offs
6. **Unsafe Rust** for uninitialized slot access

This architecture achieves:
- **SPSC: ~10-30ns per operation** (wait-free, no contention)
- **MPMC: ~50-150ns per operation** (lock-free, CAS overhead)
- **10-50x faster than Mutex + VecDeque**
- **Zero allocations** after initialization
- **Bounded memory** (fixed capacity)

Each milestone builds from simple SPSC to high-performance MPMC with proper cache optimization and memory ordering.

---

## Milestone 1: Basic SPSC Ring Buffer with Naive Atomics

### Introduction

Implement a single-producer single-consumer ring buffer using atomics for head/tail indices. This is the simplest concurrent queue: one thread writes, one thread reads, no contention. We'll use `Relaxed` ordering initially (will optimize in later milestones).

### Architecture

**Structs:**
- `RingBuffer<T>` - Fixed-size circular queue
  - **Field** `buffer: Vec<MaybeUninit<T>>` - Pre-allocated storage
  - **Field** `head: AtomicUsize` - Write index (producer increments)
  - **Field** `tail: AtomicUsize` - Read index (consumer increments)
  - **Field** `capacity: usize` - Buffer size (power of 2 for fast modulo)
  - **Function** `new(capacity: usize) -> Self` - Create buffer
  - **Function** `push(&self, value: T) -> Result<(), T>` - Producer writes
  - **Function** `pop(&self) -> Option<T>` - Consumer reads
  - **Function** `len(&self) -> usize` - Current element count
  - **Function** `is_empty(&self) -> bool` - Check if empty
  - **Function** `is_full(&self) -> bool` - Check if full

**Key Concepts:**
- Circular indexing: `index % capacity` (use bitwise AND if power of 2)
- Full condition: `(head + 1) % capacity == tail` (reserve one slot)
- Empty condition: `head == tail`
- `MaybeUninit`: Avoid initializing unused slots

**Role Each Plays:**
- Head: Producer's write position
- Tail: Consumer's read position
- Capacity: Fixed size (never changes)
- MaybeUninit: Uninitialized memory for performance

### Checkpoint Tests

```rust
#[test]
fn test_single_threaded_push_pop() {
    let rb = RingBuffer::new(4);

    assert_eq!(rb.push(1), Ok(()));
    assert_eq!(rb.push(2), Ok(()));
    assert_eq!(rb.push(3), Ok(()));

    assert_eq!(rb.len(), 3);

    assert_eq!(rb.pop(), Some(1));
    assert_eq!(rb.pop(), Some(2));
    assert_eq!(rb.pop(), Some(3));
    assert_eq!(rb.pop(), None);
}

#[test]
fn test_wrap_around() {
    let rb = RingBuffer::new(4);

    // Fill buffer
    rb.push(1).unwrap();
    rb.push(2).unwrap();
    rb.push(3).unwrap();

    // Pop one
    assert_eq!(rb.pop(), Some(1));

    // Push one (wraps around)
    rb.push(4).unwrap();

    assert_eq!(rb.pop(), Some(2));
    assert_eq!(rb.pop(), Some(3));
    assert_eq!(rb.pop(), Some(4));
}

#[test]
fn test_full_buffer() {
    let rb = RingBuffer::new(4);

    rb.push(1).unwrap();
    rb.push(2).unwrap();
    rb.push(3).unwrap();

    // Buffer capacity is 4, but we reserve 1 slot
    assert!(rb.is_full());
    assert_eq!(rb.push(4), Err(4)); // Should fail
}

#[test]
fn test_spsc_producer_consumer() {
    use std::thread;
    use std::sync::Arc;

    let rb = Arc::new(RingBuffer::new(128));
    let rb_clone = Arc::clone(&rb);

    let producer = thread::spawn(move || {
        for i in 0..100 {
            while rb_clone.push(i).is_err() {
                // Spin until space available
                std::hint::spin_loop();
            }
        }
    });

    let consumer = thread::spawn(move || {
        let mut received = vec![];
        for _ in 0..100 {
            loop {
                if let Some(val) = rb.pop() {
                    received.push(val);
                    break;
                }
                std::hint::spin_loop();
            }
        }
        received
    });

    producer.join().unwrap();
    let received = consumer.join().unwrap();

    assert_eq!(received.len(), 100);
    assert_eq!(received, (0..100).collect::<Vec<_>>());
}
```

### Starter Code

```rust
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct RingBuffer<T> {
    buffer: Vec<MaybeUninit<T>>,
    head: AtomicUsize,
    tail: AtomicUsize,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        // Ensure capacity is power of 2 for efficient modulo
        assert!(capacity.is_power_of_two(), "Capacity must be power of 2");
        assert!(capacity > 1, "Capacity must be > 1");

        // TODO: Create buffer with uninitialized memory
        // let buffer = (0..capacity).map(|_| MaybeUninit::uninit()).collect();
        todo!()
    }

    pub fn push(&self, value: T) -> Result<(), T> {
        // TODO: Implement push
        // 1. Load head and tail
        // 2. Calculate next_head = (head + 1) % capacity
        // 3. Check if full: next_head == tail
        // 4. If full, return Err(value)
        // 5. Write value to buffer[head]
        // 6. Update head to next_head
        // 7. Return Ok(())

        // let head = self.head.load(Ordering::Relaxed);
        // let tail = self.tail.load(Ordering::Relaxed);
        // let next_head = (head + 1) & (self.capacity - 1); // Fast modulo for power of 2
        todo!()
    }

    pub fn pop(&self) -> Option<T> {
        // TODO: Implement pop
        // 1. Load head and tail
        // 2. Check if empty: head == tail
        // 3. If empty, return None
        // 4. Read value from buffer[tail]
        // 5. Update tail to (tail + 1) % capacity
        // 6. Return Some(value)
        todo!()
    }

    pub fn len(&self) -> usize {
        // TODO: Calculate length
        // (head - tail) % capacity (handle wrapping)
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        todo!()
    }

    pub fn is_empty(&self) -> bool {
        // TODO: head == tail
        todo!()
    }

    pub fn is_full(&self) -> bool {
        // TODO: (head + 1) % capacity == tail
        todo!()
    }

    pub fn capacity(&self) -> usize {
        self.capacity - 1 // Reserve one slot
    }
}

impl<T> Drop for RingBuffer<T> {
    fn drop(&mut self) {
        // TODO: Drop all valid elements
        // Elements between tail and head are initialized
        while self.pop().is_some() {}
    }
}
```

---

## Milestone 2: Correct Memory Ordering for SPSC

### Introduction

**Why Milestone 1 Is Not Enough:**
`Relaxed` ordering doesn't guarantee visibility across threads. Producer might write value but consumer doesn't see it due to CPU reordering. We need `Release/Acquire` ordering for correct synchronization.

**What We're Improving:**
Use proper memory ordering:
- Producer: `Release` on head update (publishes data)
- Consumer: `Acquire` on head load (sees data)
- This creates happens-before relationship

### Architecture

**Memory Ordering Rules:**
```rust
// Producer:
buffer[head] = value;        // Store to memory
head.store(Release);         // Release fence: all previous writes visible

// Consumer:
h = head.load(Acquire);      // Acquire fence: see all writes before Release
value = buffer[tail];        // Read sees producer's write
```

**Why This Works:**
- Release-Acquire creates synchronization point
- Producer's writes to buffer happen-before head update
- Consumer's head read happen-before buffer read
- Transitivity ensures consumer sees buffer writes

### Checkpoint Tests

```rust
#[test]
fn test_memory_ordering_visibility() {
    use std::thread;
    use std::sync::Arc;

    let rb = Arc::new(RingBuffer::new(16));

    // Producer writes complex data
    let rb_clone = Arc::clone(&rb);
    let producer = thread::spawn(move || {
        for i in 0..10 {
            let data = vec![i, i + 1, i + 2]; // Heap allocation
            while rb_clone.push(data.clone()).is_err() {
                std::hint::spin_loop();
            }
        }
    });

    // Consumer reads and validates
    let consumer = thread::spawn(move || {
        for i in 0..10 {
            let data = loop {
                if let Some(d) = rb.pop() {
                    break d;
                }
                std::hint::spin_loop();
            };

            assert_eq!(data, vec![i, i + 1, i + 2]);
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}

#[test]
fn test_high_throughput_spsc() {
    use std::thread;
    use std::sync::Arc;
    use std::time::Instant;

    let rb = Arc::new(RingBuffer::new(1024));
    let rb_clone = Arc::clone(&rb);

    let start = Instant::now();

    let producer = thread::spawn(move || {
        for i in 0..1_000_000 {
            while rb_clone.push(i).is_err() {
                std::hint::spin_loop();
            }
        }
    });

    let consumer = thread::spawn(move || {
        for _ in 0..1_000_000 {
            loop {
                if rb.pop().is_some() {
                    break;
                }
                std::hint::spin_loop();
            }
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();

    let elapsed = start.elapsed();
    let throughput = 1_000_000.0 / elapsed.as_secs_f64();
    println!("SPSC throughput: {:.2}M ops/sec", throughput / 1_000_000.0);
}
```

### Starter Code

```rust
impl<T> RingBuffer<T> {
    pub fn push(&self, value: T) -> Result<(), T> {
        let head = self.head.load(Ordering::Relaxed); // Can use Relaxed for read
        let tail = self.tail.load(Ordering::Acquire);  // Acquire to see consumer's updates

        let next_head = (head + 1) & (self.capacity - 1);

        if next_head == tail {
            return Err(value); // Full
        }

        // SAFETY: We own this slot (checked not full)
        unsafe {
            self.buffer[head].as_ptr().write(value);
        }

        // Release: Make value visible to consumer
        self.head.store(next_head, Ordering::Release);

        Ok(())
    }

    pub fn pop(&self) -> Option<T> {
        let tail = self.tail.load(Ordering::Relaxed); // Can use Relaxed for read
        let head = self.head.load(Ordering::Acquire);  // Acquire to see producer's writes

        if tail == head {
            return None; // Empty
        }

        // SAFETY: Producer wrote value, we synchronized via Acquire
        let value = unsafe { self.buffer[tail].as_ptr().read() };

        let next_tail = (tail + 1) & (self.capacity - 1);

        // Release: Make slot available to producer
        self.tail.store(next_tail, Ordering::Release);

        Some(value)
    }
}
```

---

## Milestone 3: Cache-Line Alignment to Avoid False Sharing

### Introduction

**Why Milestone 2 Is Not Enough:**
Head and tail are on same cache line, causing false sharing:
```
Producer writes head → Invalidates cache line
Consumer reads tail  → Cache miss, reload from memory
Result: 10-100x slowdown!
```

**What We're Improving:**
Align head and tail to separate cache lines (64 bytes). This eliminates false sharing and allows parallel access.

### Architecture

**Cache Line Padding:**
```rust
#[repr(align(64))]
struct Aligned<T>(T);

struct RingBuffer<T> {
    buffer: Vec<MaybeUninit<T>>,
    head: Aligned<AtomicUsize>,  // Separate cache line
    tail: Aligned<AtomicUsize>,  // Separate cache line
    capacity: usize,
}
```

**Why 64 Bytes:**
- x86 cache line = 64 bytes
- ARM cache line = 64-128 bytes
- 64 is safe default

### Checkpoint Tests

```rust
#[test]
fn test_cache_line_alignment() {
    use std::mem;

    let rb = RingBuffer::<i32>::new(16);

    // Check that head and tail are on different cache lines
    let head_addr = &rb.head as *const _ as usize;
    let tail_addr = &rb.tail as *const _ as usize;

    let cache_line_size = 64;
    let head_line = head_addr / cache_line_size;
    let tail_line = tail_addr / cache_line_size;

    assert_ne!(head_line, tail_line, "head and tail on same cache line!");
}

#[test]
fn benchmark_with_padding() {
    use std::thread;
    use std::sync::Arc;
    use std::time::Instant;

    let rb = Arc::new(RingBuffer::new(512));
    let rb_clone = Arc::clone(&rb);

    let start = Instant::now();

    let producer = thread::spawn(move || {
        for i in 0..10_000_000 {
            while rb_clone.push(i).is_err() {
                std::hint::spin_loop();
            }
        }
    });

    let consumer = thread::spawn(move || {
        for _ in 0..10_000_000 {
            loop {
                if rb.pop().is_some() {
                    break;
                }
                std::hint::spin_loop();
            }
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();

    let elapsed = start.elapsed();
    println!("10M ops in {:?} ({:.2}M ops/sec)",
        elapsed, 10.0 / elapsed.as_secs_f64());
}
```

### Starter Code

```rust
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};

#[repr(align(64))]
struct CacheLineAligned<T>(T);

pub struct RingBuffer<T> {
    buffer: Vec<MaybeUninit<T>>,
    head: CacheLineAligned<AtomicUsize>,
    tail: CacheLineAligned<AtomicUsize>,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity.is_power_of_two());
        assert!(capacity > 1);

        let buffer = (0..capacity).map(|_| MaybeUninit::uninit()).collect();

        Self {
            buffer,
            head: CacheLineAligned(AtomicUsize::new(0)),
            tail: CacheLineAligned(AtomicUsize::new(0)),
            capacity,
        }
    }

    pub fn push(&self, value: T) -> Result<(), T> {
        let head = self.head.0.load(Ordering::Relaxed);
        let tail = self.tail.0.load(Ordering::Acquire);

        let next_head = (head + 1) & (self.capacity - 1);

        if next_head == tail {
            return Err(value);
        }

        unsafe {
            self.buffer[head].as_ptr().write(value);
        }

        self.head.0.store(next_head, Ordering::Release);
        Ok(())
    }

    pub fn pop(&self) -> Option<T> {
        let tail = self.tail.0.load(Ordering::Relaxed);
        let head = self.head.0.load(Ordering::Acquire);

        if tail == head {
            return None;
        }

        let value = unsafe { self.buffer[tail].as_ptr().read() };
        let next_tail = (tail + 1) & (self.capacity - 1);

        self.tail.0.store(next_tail, Ordering::Release);
        Some(value)
    }

    pub fn len(&self) -> usize {
        let head = self.head.0.load(Ordering::Relaxed);
        let tail = self.tail.0.load(Ordering::Relaxed);

        if head >= tail {
            head - tail
        } else {
            self.capacity - tail + head
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.0.load(Ordering::Relaxed) == self.tail.0.load(Ordering::Relaxed)
    }

    pub fn is_full(&self) -> bool {
        let head = self.head.0.load(Ordering::Relaxed);
        let tail = self.tail.0.load(Ordering::Relaxed);
        let next_head = (head + 1) & (self.capacity - 1);
        next_head == tail
    }

    pub fn capacity(&self) -> usize {
        self.capacity - 1
    }
}

impl<T> Drop for RingBuffer<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

unsafe impl<T: Send> Send for RingBuffer<T> {}
unsafe impl<T: Send> Sync for RingBuffer<T> {}
```

---

## Milestone 4: MPSC (Multi-Producer Single-Consumer)

### Introduction

**Why Milestone 3 Is Not Enough:**
SPSC only allows one producer. Real systems often have multiple producers (e.g., multiple threads logging to single file writer). Need atomic head increment for multiple producers.

**What We're Improving:**
Use `fetch_add` for head to allow multiple producers. Each producer atomically claims a slot, then writes to it. Consumer remains unchanged (single consumer).

### Architecture

**Modified Push:**
```rust
// SPSC:
head = load head
buffer[head] = value
store head + 1

// MPSC:
slot = fetch_add(head, 1)  // Atomically claim slot
buffer[slot] = value
```

**Challenge:**
Slots may be written out of order! Consumer must handle this.

**Solution:**
Add sequence numbers to track which slots are ready.

### Checkpoint Tests

```rust
#[test]
fn test_mpsc_multiple_producers() {
    use std::thread;
    use std::sync::Arc;
    use std::collections::HashSet;

    let rb = Arc::new(RingBuffer::new(512));

    // 4 producers
    let producers: Vec<_> = (0..4).map(|tid| {
        let rb_clone = Arc::clone(&rb);
        thread::spawn(move || {
            for i in 0..250 {
                let value = tid * 1000 + i;
                while rb_clone.push(value).is_err() {
                    std::hint::spin_loop();
                }
            }
        })
    }).collect();

    // 1 consumer
    let rb_clone = Arc::clone(&rb);
    let consumer = thread::spawn(move || {
        let mut received = HashSet::new();
        for _ in 0..1000 {
            loop {
                if let Some(val) = rb_clone.pop() {
                    received.insert(val);
                    break;
                }
                std::hint::spin_loop();
            }
        }
        received
    });

    for p in producers {
        p.join().unwrap();
    }

    let received = consumer.join().unwrap();
    assert_eq!(received.len(), 1000);
}

#[test]
fn test_mpsc_no_duplicates() {
    use std::thread;
    use std::sync::Arc;

    let rb = Arc::new(RingBuffer::new(256));

    let producers: Vec<_> = (0..8).map(|tid| {
        let rb_clone = Arc::clone(&rb);
        thread::spawn(move || {
            for i in 0..100 {
                while rb_clone.push((tid, i)).is_err() {
                    std::hint::spin_loop();
                }
            }
        })
    }).collect();

    let rb_clone = Arc::clone(&rb);
    let consumer = thread::spawn(move || {
        let mut received = vec![];
        for _ in 0..800 {
            loop {
                if let Some(val) = rb_clone.pop() {
                    received.push(val);
                    break;
                }
                std::hint::spin_loop();
            }
        }
        received
    });

    for p in producers {
        p.join().unwrap();
    }

    let received = consumer.join().unwrap();

    // Check no duplicates
    let mut sorted = received.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), 800);
}
```

### Starter Code

```rust
// For MPSC, we need sequence numbers to track slot readiness

#[repr(align(64))]
struct CacheLineAligned<T>(T);

struct Slot<T> {
    value: MaybeUninit<T>,
    sequence: AtomicUsize,
}

pub struct RingBuffer<T> {
    buffer: Vec<Slot<T>>,
    head: CacheLineAligned<AtomicUsize>,
    tail: CacheLineAligned<AtomicUsize>,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity.is_power_of_two());

        let buffer = (0..capacity)
            .map(|i| Slot {
                value: MaybeUninit::uninit(),
                sequence: AtomicUsize::new(i),
            })
            .collect();

        Self {
            buffer,
            head: CacheLineAligned(AtomicUsize::new(0)),
            tail: CacheLineAligned(AtomicUsize::new(0)),
            capacity,
        }
    }

    pub fn push(&self, value: T) -> Result<(), T> {
        loop {
            let head = self.head.0.load(Ordering::Relaxed);
            let slot_idx = head & (self.capacity - 1);
            let slot = &self.buffer[slot_idx];

            let seq = slot.sequence.load(Ordering::Acquire);

            // Check if slot is ready for writing
            if seq == head {
                // Try to claim this slot
                match self.head.0.compare_exchange_weak(
                    head,
                    head + 1,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // We claimed the slot, write value
                        unsafe {
                            slot.value.as_ptr().write(value);
                        }

                        // Mark slot as ready for reading
                        slot.sequence.store(head + 1, Ordering::Release);
                        return Ok(());
                    }
                    Err(_) => {
                        // Another producer claimed it, retry
                    }
                }
            } else if seq < head {
                // Slot not ready yet (being written by another producer)
                std::hint::spin_loop();
            } else {
                // Buffer full
                return Err(value);
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        loop {
            let tail = self.tail.0.load(Ordering::Relaxed);
            let slot_idx = tail & (self.capacity - 1);
            let slot = &self.buffer[slot_idx];

            let seq = slot.sequence.load(Ordering::Acquire);

            if seq == tail + 1 {
                // Slot is ready for reading
                let value = unsafe { slot.value.as_ptr().read() };

                // Mark slot as available for writing
                slot.sequence.store(tail + self.capacity, Ordering::Release);

                self.tail.0.store(tail + 1, Ordering::Release);
                return Some(value);
            } else if seq < tail + 1 {
                // Slot not ready yet (being written)
                return None; // Or spin?
            } else {
                // Buffer empty
                return None;
            }
        }
    }
}
```

---

## Milestone 5: MPMC (Multi-Producer Multi-Consumer)

### Introduction

**Why Milestone 4 Is Not Enough:**
Single consumer is limiting. Many systems need multiple consumers (e.g., thread pool with multiple workers pulling tasks). Need atomic tail increment.

**What We're Improving:**
Use `fetch_add` for both head and tail. Both producers and consumers use CAS loops to claim slots.

### Architecture

**MPMC Complexity:**
- Multiple producers claim slots with head
- Multiple consumers claim slots with tail
- Both need sequence number coordination
- More contention, slower than SPSC/MPSC

### Checkpoint Tests

```rust
#[test]
fn test_mpmc() {
    use std::thread;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let rb = Arc::new(RingBuffer::new(256));
    let push_count = Arc::new(AtomicUsize::new(0));
    let pop_count = Arc::new(AtomicUsize::new(0));

    // 4 producers
    let producers: Vec<_> = (0..4).map(|tid| {
        let rb_clone = Arc::clone(&rb);
        let pc = Arc::clone(&push_count);
        thread::spawn(move || {
            for i in 0..250 {
                while rb_clone.push(tid * 1000 + i).is_err() {
                    std::hint::spin_loop();
                }
                pc.fetch_add(1, Ordering::Relaxed);
            }
        })
    }).collect();

    // 4 consumers
    let consumers: Vec<_> = (0..4).map(|_| {
        let rb_clone = Arc::clone(&rb);
        let pc = Arc::clone(&pop_count);
        thread::spawn(move || {
            let mut count = 0;
            for _ in 0..250 {
                loop {
                    if rb_clone.pop().is_some() {
                        count += 1;
                        pc.fetch_add(1, Ordering::Relaxed);
                        break;
                    }
                    std::hint::spin_loop();
                }
            }
            count
        })
    }).collect();

    for p in producers {
        p.join().unwrap();
    }

    for c in consumers {
        c.join().unwrap();
    }

    assert_eq!(push_count.load(Ordering::Acquire), 1000);
    assert_eq!(pop_count.load(Ordering::Acquire), 1000);
}
```

### Starter Code

```rust
impl<T> RingBuffer<T> {
    // Push remains same as MPSC

    pub fn pop(&self) -> Option<T> {
        loop {
            let tail = self.tail.0.load(Ordering::Relaxed);
            let slot_idx = tail & (self.capacity - 1);
            let slot = &self.buffer[slot_idx];

            let seq = slot.sequence.load(Ordering::Acquire);

            if seq == tail + 1 {
                // Slot ready, try to claim it
                match self.tail.0.compare_exchange_weak(
                    tail,
                    tail + 1,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // We claimed the slot
                        let value = unsafe { slot.value.as_ptr().read() };
                        slot.sequence.store(tail + self.capacity, Ordering::Release);
                        return Some(value);
                    }
                    Err(_) => {
                        // Another consumer claimed it, retry
                    }
                }
            } else if seq < tail + 1 {
                // Slot not ready
                return None;
            } else {
                // Empty
                return None;
            }
        }
    }
}
```

---

## Milestone 6: Blocking Operations with Backoff Strategy

### Introduction

**Why Milestone 5 Is Not Enough:**
Spin loops waste CPU. In low-throughput scenarios, we want to block (sleep) when buffer is full/empty instead of spinning. Add backoff strategy: spin briefly, then yield, then sleep.

**What We're Improving:**
Add blocking push/pop variants with exponential backoff. Start with spin, escalate to yield, then sleep.

### Architecture

**Backoff Strategy:**
```
1-10 iterations:   Spin (std::hint::spin_loop)
11-100 iterations: Yield (thread::yield_now)
100+ iterations:   Sleep (thread::sleep)
```

**New Functions:**
- `push_blocking(&self, value: T)` - Block until space available
- `pop_blocking(&self) -> T` - Block until element available
- `try_push(&self, value: T, timeout: Duration) -> Result<(), T>` - Timeout
- `try_pop(&self, timeout: Duration) -> Option<T>` - Timeout

### Checkpoint Tests

```rust
#[test]
fn test_blocking_operations() {
    use std::thread;
    use std::sync::Arc;
    use std::time::Duration;

    let rb = Arc::new(RingBuffer::new(4));

    // Fill buffer
    for i in 0..3 {
        rb.push(i).unwrap();
    }

    let rb_clone = Arc::clone(&rb);
    let producer = thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        rb_clone.pop(); // Make space
    });

    // This should block until producer makes space
    let start = std::time::Instant::now();
    rb.push_blocking(100);
    let elapsed = start.elapsed();

    producer.join().unwrap();

    assert!(elapsed >= Duration::from_millis(100));
}

#[test]
fn test_timeout() {
    let rb = RingBuffer::new(4);

    for i in 0..3 {
        rb.push(i).unwrap();
    }

    // Should timeout
    let result = rb.try_push(100, Duration::from_millis(10));
    assert!(result.is_err());
}
```

### Starter Code

```rust
use std::time::{Duration, Instant};
use std::thread;

impl<T> RingBuffer<T> {
    pub fn push_blocking(&self, mut value: T) {
        let mut backoff = 1;
        loop {
            match self.push(value) {
                Ok(()) => return,
                Err(v) => {
                    value = v;

                    if backoff <= 10 {
                        for _ in 0..backoff {
                            std::hint::spin_loop();
                        }
                        backoff *= 2;
                    } else if backoff <= 100 {
                        thread::yield_now();
                        backoff += 1;
                    } else {
                        thread::sleep(Duration::from_micros(100));
                    }
                }
            }
        }
    }

    pub fn pop_blocking(&self) -> T {
        let mut backoff = 1;
        loop {
            if let Some(value) = self.pop() {
                return value;
            }

            if backoff <= 10 {
                for _ in 0..backoff {
                    std::hint::spin_loop();
                }
                backoff *= 2;
            } else if backoff <= 100 {
                thread::yield_now();
                backoff += 1;
            } else {
                thread::sleep(Duration::from_micros(100));
            }
        }
    }

    pub fn try_push(&self, mut value: T, timeout: Duration) -> Result<(), T> {
        let start = Instant::now();
        let mut backoff = 1;

        loop {
            match self.push(value) {
                Ok(()) => return Ok(()),
                Err(v) => {
                    if start.elapsed() >= timeout {
                        return Err(v);
                    }

                    value = v;

                    if backoff <= 10 {
                        for _ in 0..backoff {
                            std::hint::spin_loop();
                        }
                        backoff *= 2;
                    } else {
                        thread::yield_now();
                    }
                }
            }
        }
    }

    pub fn try_pop(&self, timeout: Duration) -> Option<T> {
        let start = Instant::now();
        let mut backoff = 1;

        loop {
            if let Some(value) = self.pop() {
                return Some(value);
            }

            if start.elapsed() >= timeout {
                return None;
            }

            if backoff <= 10 {
                for _ in 0..backoff {
                    std::hint::spin_loop();
                }
                backoff *= 2;
            } else {
                thread::yield_now();
            }
        }
    }
}
```

---

## Complete Working Example

```rust
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// ============================================================================
// CACHE LINE ALIGNED WRAPPER
// ============================================================================

#[repr(align(64))]
struct CacheLineAligned<T>(T);

// ============================================================================
// SLOT WITH SEQUENCE NUMBER
// ============================================================================

struct Slot<T> {
    value: MaybeUninit<T>,
    sequence: AtomicUsize,
}

// ============================================================================
// RING BUFFER
// ============================================================================

pub struct RingBuffer<T> {
    buffer: Vec<Slot<T>>,
    head: CacheLineAligned<AtomicUsize>,
    tail: CacheLineAligned<AtomicUsize>,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity.is_power_of_two(), "Capacity must be power of 2");
        assert!(capacity > 1);

        let buffer = (0..capacity)
            .map(|i| Slot {
                value: MaybeUninit::uninit(),
                sequence: AtomicUsize::new(i),
            })
            .collect();

        Self {
            buffer,
            head: CacheLineAligned(AtomicUsize::new(0)),
            tail: CacheLineAligned(AtomicUsize::new(0)),
            capacity,
        }
    }

    pub fn push(&self, value: T) -> Result<(), T> {
        loop {
            let head = self.head.0.load(Ordering::Relaxed);
            let slot_idx = head & (self.capacity - 1);
            let slot = &self.buffer[slot_idx];

            let seq = slot.sequence.load(Ordering::Acquire);

            if seq == head {
                match self.head.0.compare_exchange_weak(
                    head,
                    head.wrapping_add(1),
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        unsafe {
                            slot.value.as_ptr().write(value);
                        }
                        slot.sequence.store(head.wrapping_add(1), Ordering::Release);
                        return Ok(());
                    }
                    Err(_) => {}
                }
            } else if seq.wrapping_sub(head) < self.capacity {
                return Err(value); // Full
            } else {
                std::hint::spin_loop();
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        loop {
            let tail = self.tail.0.load(Ordering::Relaxed);
            let slot_idx = tail & (self.capacity - 1);
            let slot = &self.buffer[slot_idx];

            let seq = slot.sequence.load(Ordering::Acquire);

            if seq == tail.wrapping_add(1) {
                match self.tail.0.compare_exchange_weak(
                    tail,
                    tail.wrapping_add(1),
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        let value = unsafe { slot.value.as_ptr().read() };
                        slot.sequence.store(tail.wrapping_add(self.capacity), Ordering::Release);
                        return Some(value);
                    }
                    Err(_) => {}
                }
            } else if seq.wrapping_sub(tail.wrapping_add(1)) >= self.capacity {
                return None; // Empty
            } else {
                std::hint::spin_loop();
            }
        }
    }

    pub fn push_blocking(&self, mut value: T) {
        let mut backoff = 1;
        loop {
            match self.push(value) {
                Ok(()) => return,
                Err(v) => {
                    value = v;
                    if backoff <= 10 {
                        for _ in 0..backoff {
                            std::hint::spin_loop();
                        }
                        backoff *= 2;
                    } else if backoff <= 100 {
                        thread::yield_now();
                        backoff += 1;
                    } else {
                        thread::sleep(Duration::from_micros(100));
                    }
                }
            }
        }
    }

    pub fn pop_blocking(&self) -> T {
        let mut backoff = 1;
        loop {
            if let Some(value) = self.pop() {
                return value;
            }

            if backoff <= 10 {
                for _ in 0..backoff {
                    std::hint::spin_loop();
                }
                backoff *= 2;
            } else if backoff <= 100 {
                thread::yield_now();
                backoff += 1;
            } else {
                thread::sleep(Duration::from_micros(100));
            }
        }
    }

    pub fn len(&self) -> usize {
        let head = self.head.0.load(Ordering::Relaxed);
        let tail = self.tail.0.load(Ordering::Relaxed);
        head.wrapping_sub(tail)
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<T> Drop for RingBuffer<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

unsafe impl<T: Send> Send for RingBuffer<T> {}
unsafe impl<T: Send> Sync for RingBuffer<T> {}

// ============================================================================
// EXAMPLE USAGE
// ============================================================================

fn main() {
    println!("=== Wait-Free Ring Buffer Demo ===\n");

    // SPSC Example
    println!("--- SPSC (Single Producer, Single Consumer) ---");
    {
        let rb = Arc::new(RingBuffer::new(16));
        let rb_clone = Arc::clone(&rb);

        let producer = thread::spawn(move || {
            for i in 0..10 {
                rb_clone.push_blocking(i);
                println!("Produced: {}", i);
                thread::sleep(Duration::from_millis(50));
            }
        });

        let consumer = thread::spawn(move || {
            for _ in 0..10 {
                let val = rb.pop_blocking();
                println!("Consumed: {}", val);
                thread::sleep(Duration::from_millis(100));
            }
        });

        producer.join().unwrap();
        consumer.join().unwrap();
    }

    println!();

    // MPMC Example
    println!("--- MPMC (Multi Producer, Multi Consumer) ---");
    {
        let rb = Arc::new(RingBuffer::new(128));

        let producers: Vec<_> = (0..4)
            .map(|tid| {
                let rb_clone = Arc::clone(&rb);
                thread::spawn(move || {
                    for i in 0..25 {
                        rb_clone.push_blocking(tid * 100 + i);
                    }
                    println!("Producer {} done", tid);
                })
            })
            .collect();

        let consumers: Vec<_> = (0..4)
            .map(|tid| {
                let rb_clone = Arc::clone(&rb);
                thread::spawn(move || {
                    let mut count = 0;
                    for _ in 0..25 {
                        rb_clone.pop_blocking();
                        count += 1;
                    }
                    println!("Consumer {} consumed {} items", tid, count);
                })
            })
            .collect();

        for p in producers {
            p.join().unwrap();
        }
        for c in consumers {
            c.join().unwrap();
        }
    }

    println!();

    // Performance Benchmark
    println!("--- Performance Benchmark ---");
    {
        let rb = Arc::new(RingBuffer::new(1024));
        let rb_clone = Arc::clone(&rb);

        let start = Instant::now();

        let producer = thread::spawn(move || {
            for i in 0..1_000_000 {
                rb_clone.push_blocking(i);
            }
        });

        let consumer = thread::spawn(move || {
            for _ in 0..1_000_000 {
                rb.pop_blocking();
            }
        });

        producer.join().unwrap();
        consumer.join().unwrap();

        let elapsed = start.elapsed();
        let throughput = 1_000_000.0 / elapsed.as_secs_f64();

        println!("SPSC: 1M ops in {:?}", elapsed);
        println!("Throughput: {:.2}M ops/sec", throughput / 1_000_000.0);
    }

    println!("\n=== Done ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spsc() {
        let rb = Arc::new(RingBuffer::new(16));
        let rb_clone = Arc::clone(&rb);

        let producer = thread::spawn(move || {
            for i in 0..100 {
                rb_clone.push_blocking(i);
            }
        });

        let consumer = thread::spawn(move || {
            let mut received = vec![];
            for _ in 0..100 {
                received.push(rb.pop_blocking());
            }
            received
        });

        producer.join().unwrap();
        let received = consumer.join().unwrap();

        assert_eq!(received, (0..100).collect::<Vec<_>>());
    }

    #[test]
    fn test_mpmc() {
        let rb = Arc::new(RingBuffer::new(256));

        let producers: Vec<_> = (0..4)
            .map(|tid| {
                let rb_clone = Arc::clone(&rb);
                thread::spawn(move || {
                    for i in 0..250 {
                        rb_clone.push_blocking(tid * 1000 + i);
                    }
                })
            })
            .collect();

        let consumers: Vec<_> = (0..4)
            .map(|_| {
                let rb_clone = Arc::clone(&rb);
                thread::spawn(move || {
                    let mut count = 0;
                    for _ in 0..250 {
                        rb_clone.pop_blocking();
                        count += 1;
                    }
                    count
                })
            })
            .collect();

        for p in producers {
            p.join().unwrap();
        }

        let total: usize = consumers.into_iter().map(|c| c.join().unwrap()).sum();
        assert_eq!(total, 1000);
    }

    #[test]
    fn test_cache_alignment() {
        let rb = RingBuffer::<i32>::new(16);

        let head_addr = &rb.head as *const _ as usize;
        let tail_addr = &rb.tail as *const _ as usize;

        let diff = if head_addr > tail_addr {
            head_addr - tail_addr
        } else {
            tail_addr - head_addr
        };

        assert!(diff >= 64, "head and tail not properly separated");
    }
}
```

This completes the wait-free ring buffer project with SPSC, MPMC, and blocking operations!
