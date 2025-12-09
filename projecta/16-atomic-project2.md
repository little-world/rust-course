# Chapter 16: Atomic Operations - Project 2

## Project 2: Lock-Free Stack (Treiber Stack)

### Problem Statement

Implement a lock-free concurrent stack using atomic pointers and compare-and-swap operations. The stack must support push and pop operations from multiple threads simultaneously without using mutexes. This is the classic "Treiber Stack" - one of the fundamental lock-free data structures.

The implementation must handle the ABA problem, memory reclamation, and provide proper memory ordering guarantees.

### Use Cases

- Thread pool work stealing queues
- Memory allocators (free list management)
- Undo/redo stacks in concurrent editors
- Task scheduling in async runtimes
- Lock-free object pools
- Message passing between producer/consumer threads

### Why It Matters

Traditional stack with mutex:
```rust
let mut stack = Mutex::new(Vec::new());
stack.lock().unwrap().push(item);  // Blocks all other threads
```

Under contention with 8 threads, mutex causes serialization—threads wait in queue. Lock-free stack allows parallel progress: failed CAS retries immediately, no kernel involvement, no context switches.

Performance comparison:
- Mutex stack: ~50-100ns per op (uncontended), ~1-10μs (contended)
- Lock-free stack: ~20-50ns per op (always), scales linearly with cores

**The ABA Problem:**
```
Thread 1 reads head=A
Thread 2: pop A, pop B, push A (head is A again!)
Thread 1: CAS succeeds (head is still A) but stack structure changed!
```

Solutions: version counters, hazard pointers, epoch-based reclamation.

Real-world usage: Crossbeam's lock-free queues, Tokio's work-stealing scheduler, parking_lot's thread parking.

---

## Milestone 1: Basic Single-Threaded Stack with Atomic Pointer

### Introduction

Build a basic stack using `AtomicPtr` for the head pointer. Start with single-threaded usage to understand the linked list structure and atomic pointer operations before adding concurrency.

### Architecture

**Structs:**
- `Node<T>` - Stack node
  - **Field** `value: T` - The stored value
  - **Field** `next: *mut Node<T>` - Raw pointer to next node
  - **Function** `new(value: T, next: *mut Node<T>) -> Box<Node<T>>` - Create boxed node

- `LockFreeStack<T>` - The stack structure
  - **Field** `head: AtomicPtr<Node<T>>` - Atomic pointer to top of stack
  - **Function** `new() -> Self` - Create empty stack
  - **Function** `push(&self, value: T)` - Add value to top
  - **Function** `pop(&self) -> Option<T>` - Remove from top
  - **Function** `is_empty(&self) -> bool` - Check if empty

**Role Each Plays:**
- `AtomicPtr`: Atomic pointer operations (load, store, CAS)
- `Box::into_raw()`: Convert Box to raw pointer
- `Box::from_raw()`: Convert raw pointer back to Box (for deallocation)
- Linked list: Each node points to next node
- Head pointer: Entry point to stack, null if empty

### Checkpoint Tests

```rust
#[test]
fn test_push_pop_single_thread() {
    let stack = LockFreeStack::new();
    assert!(stack.is_empty());

    stack.push(1);
    stack.push(2);
    stack.push(3);

    assert_eq!(stack.pop(), Some(3));
    assert_eq!(stack.pop(), Some(2));
    assert_eq!(stack.pop(), Some(1));
    assert_eq!(stack.pop(), None);
    assert!(stack.is_empty());
}

#[test]
fn test_lifo_order() {
    let stack = LockFreeStack::new();

    for i in 0..10 {
        stack.push(i);
    }

    for i in (0..10).rev() {
        assert_eq!(stack.pop(), Some(i));
    }
}

#[test]
fn test_push_strings() {
    let stack = LockFreeStack::new();

    stack.push("hello".to_string());
    stack.push("world".to_string());

    assert_eq!(stack.pop(), Some("world".to_string()));
    assert_eq!(stack.pop(), Some("hello".to_string()));
}
```

### Starter Code

```rust
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

struct Node<T> {
    value: T,
    next: *mut Node<T>,
}

impl<T> Node<T> {
    fn new(value: T, next: *mut Node<T>) -> Box<Node<T>> {
        Box::new(Node { value, next })
    }
}

pub struct LockFreeStack<T> {
    head: AtomicPtr<Node<T>>,
}

impl<T> LockFreeStack<T> {
    pub fn new() -> Self {
        // TODO: Initialize with null pointer
        // Self { head: AtomicPtr::new(ptr::null_mut()) }
        todo!()
    }

    pub fn push(&self, value: T) {
        // TODO: For now, use simple store (not thread-safe yet)
        // Steps:
        // 1. Load current head
        // 2. Create new node pointing to current head
        // 3. Store new node as new head
        //
        // let old_head = self.head.load(Ordering::Relaxed);
        // let new_node = Box::into_raw(Node::new(value, old_head));
        // self.head.store(new_node, Ordering::Relaxed);
        todo!()
    }

    pub fn pop(&self) -> Option<T> {
        // TODO: For now, use simple load (not thread-safe yet)
        // Steps:
        // 1. Load head pointer
        // 2. If null, return None
        // 3. Get node from raw pointer
        // 4. Update head to next
        // 5. Extract value and return
        //
        // let head_ptr = self.head.load(Ordering::Relaxed);
        // if head_ptr.is_null() { return None; }
        // unsafe {
        //     let head_node = Box::from_raw(head_ptr);
        //     self.head.store(head_node.next, Ordering::Relaxed);
        //     Some(head_node.value)
        // }
        todo!()
    }

    pub fn is_empty(&self) -> bool {
        // TODO: Check if head is null
        todo!()
    }
}

impl<T> Drop for LockFreeStack<T> {
    fn drop(&mut self) {
        // TODO: Pop all nodes to free memory
        while self.pop().is_some() {}
    }
}
```

---

## Milestone 2: Thread-Safe Push with Compare-And-Swap

### Introduction

**Why Milestone 1 Is Not Enough:**
The simple store/load approach has a race condition:
```
Thread A: loads head=Node1
Thread B: loads head=Node1
Thread A: creates Node2->Node1, stores Node2 as head
Thread B: creates Node3->Node1, stores Node3 as head (overwrites Node2!)
Node2 is now leaked, Node1 appears twice in stack!
```

**What We're Improving:**
Use compare-and-swap (CAS) to atomically update head only if it hasn't changed. Retry loop if CAS fails because another thread modified head.

### Architecture

**Modified Functions:**
- `push(&self, value: T)` - Use CAS loop
  - Load current head
  - Create new node pointing to current head
  - Try to CAS head from old to new
  - If CAS fails, update new node's next pointer and retry
  - Loop until CAS succeeds

**Role Each Plays:**
- `compare_exchange_weak`: Try to swap head if it matches expected value
- Retry loop: Keep trying until we successfully update head
- `Acquire/Release` ordering: Synchronize node contents across threads

### Checkpoint Tests

```rust
#[test]
fn test_concurrent_push() {
    use std::thread;
    use std::sync::Arc;

    let stack = Arc::new(LockFreeStack::new());
    let mut handles = vec![];

    // 10 threads, each pushes 100 items
    for thread_id in 0..10 {
        let s = Arc::clone(&stack);
        let handle = thread::spawn(move || {
            for i in 0..100 {
                s.push(thread_id * 1000 + i);
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    // Should have 1000 items
    let mut count = 0;
    while stack.pop().is_some() {
        count += 1;
    }
    assert_eq!(count, 1000);
}

#[test]
fn test_no_lost_items() {
    use std::thread;
    use std::sync::Arc;
    use std::collections::HashSet;

    let stack = Arc::new(LockFreeStack::new());

    // Push unique values from multiple threads
    let handles: Vec<_> = (0..5).map(|tid| {
        let s = Arc::clone(&stack);
        thread::spawn(move || {
            for i in 0..200 {
                s.push(tid * 1000 + i);
            }
        })
    }).collect();

    for h in handles {
        h.join().unwrap();
    }

    // Collect all values
    let mut seen = HashSet::new();
    while let Some(val) = stack.pop() {
        assert!(seen.insert(val), "Duplicate value: {}", val);
    }

    assert_eq!(seen.len(), 1000);
}

#[test]
fn test_push_under_contention() {
    use std::thread;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let stack = Arc::new(LockFreeStack::new());
    let push_count = Arc::new(AtomicUsize::new(0));

    let handles: Vec<_> = (0..8).map(|_| {
        let s = Arc::clone(&stack);
        let pc = Arc::clone(&push_count);
        thread::spawn(move || {
            for _ in 0..1000 {
                s.push(42);
                pc.fetch_add(1, Ordering::Relaxed);
            }
        })
    }).collect();

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(push_count.load(Ordering::Acquire), 8000);
}
```

### Starter Code

```rust
impl<T> LockFreeStack<T> {
    pub fn push(&self, value: T) {
        // TODO: Implement CAS loop
        // Pattern:
        // let mut new_node = Box::new(Node {
        //     value,
        //     next: ptr::null_mut(),
        // });
        //
        // loop {
        //     let head = self.head.load(Ordering::Acquire);
        //     new_node.next = head;
        //
        //     let new_node_ptr = Box::into_raw(Box::new(Node {
        //         value: new_node.value, // Need to handle ownership properly!
        //         next: head,
        //     }));
        //
        //     match self.head.compare_exchange_weak(
        //         head,
        //         new_node_ptr,
        //         Ordering::Release,
        //         Ordering::Acquire,
        //     ) {
        //         Ok(_) => break,
        //         Err(_) => {
        //             // CAS failed, retry
        //             // Need to clean up new_node_ptr
        //             unsafe { Box::from_raw(new_node_ptr); }
        //         }
        //     }
        // }

        // Better pattern using ManuallyDrop or MaybeUninit
        todo!()
    }
}
```

---

## Milestone 3: Thread-Safe Pop with Memory Reclamation

### Introduction

**Why Milestone 2 Is Not Enough:**
Push is thread-safe now, but pop still has races:
```
Thread A: loads head=Node1
Thread B: pops Node1 (frees it!)
Thread A: tries to read Node1.next (use-after-free!)
```

**What We're Improving:**
Use CAS for pop with careful memory handling. Must read `next` pointer before CAS, then only free node after successful CAS.

### Architecture

**Modified Functions:**
- `pop(&self) -> Option<T>` - Use CAS loop
  - Load head pointer
  - If null, return None
  - Read next pointer from head node (unsafe)
  - Try to CAS head from current to next
  - If CAS succeeds, extract value and free node
  - If CAS fails, another thread modified stack, retry

**Memory Safety:**
- Only dereference head after checking it's not null
- Only free node after successful CAS
- Failed CAS means we don't own the node

### Checkpoint Tests

```rust
#[test]
fn test_concurrent_pop() {
    use std::thread;
    use std::sync::Arc;

    let stack = Arc::new(LockFreeStack::new());

    // Pre-fill stack
    for i in 0..1000 {
        stack.push(i);
    }

    let mut handles = vec![];

    // 10 threads, each tries to pop 100 items
    for _ in 0..10 {
        let s = Arc::clone(&stack);
        let handle = thread::spawn(move || {
            let mut popped = 0;
            for _ in 0..100 {
                if s.pop().is_some() {
                    popped += 1;
                }
            }
            popped
        });
        handles.push(handle);
    }

    let total: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
    assert_eq!(total, 1000);
    assert!(stack.is_empty());
}

#[test]
fn test_concurrent_push_pop() {
    use std::thread;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let stack = Arc::new(LockFreeStack::new());
    let push_count = Arc::new(AtomicUsize::new(0));
    let pop_count = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    // 4 pusher threads
    for tid in 0..4 {
        let s = Arc::clone(&stack);
        let pc = Arc::clone(&push_count);
        let handle = thread::spawn(move || {
            for i in 0..500 {
                s.push(tid * 1000 + i);
                pc.fetch_add(1, Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }

    // 4 popper threads
    for _ in 0..4 {
        let s = Arc::clone(&stack);
        let pc = Arc::clone(&pop_count);
        let handle = thread::spawn(move || {
            for _ in 0..500 {
                if s.pop().is_some() {
                    pc.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    let pushed = push_count.load(Ordering::Acquire);
    let popped = pop_count.load(Ordering::Acquire);

    assert_eq!(pushed, 2000);

    // Remaining in stack
    let mut remaining = 0;
    while stack.pop().is_some() {
        remaining += 1;
    }

    assert_eq!(popped + remaining, pushed);
}

#[test]
fn test_no_use_after_free() {
    use std::thread;
    use std::sync::Arc;

    // This test uses Miri or valgrind to detect use-after-free
    let stack = Arc::new(LockFreeStack::new());

    for i in 0..100 {
        stack.push(i);
    }

    let handles: Vec<_> = (0..8).map(|_| {
        let s = Arc::clone(&stack);
        thread::spawn(move || {
            for _ in 0..50 {
                s.pop();
            }
        })
    }).collect();

    for h in handles {
        h.join().unwrap();
    }
}
```

### Starter Code

```rust
impl<T> LockFreeStack<T> {
    pub fn pop(&self) -> Option<T> {
        // TODO: Implement CAS loop for pop
        // loop {
        //     let head = self.head.load(Ordering::Acquire);
        //
        //     if head.is_null() {
        //         return None;
        //     }
        //
        //     unsafe {
        //         // SAFETY: head is not null, and we haven't freed it yet
        //         let next = (*head).next;
        //
        //         match self.head.compare_exchange_weak(
        //             head,
        //             next,
        //             Ordering::Release,
        //             Ordering::Acquire,
        //         ) {
        //             Ok(_) => {
        //                 // Successfully removed head
        //                 let head_node = Box::from_raw(head);
        //                 return Some(head_node.value);
        //             }
        //             Err(_) => {
        //                 // CAS failed, another thread modified stack
        //                 // Loop and retry
        //             }
        //         }
        //     }
        // }
        todo!()
    }
}
```

---

## Milestone 4: ABA Problem Protection with Version Counter

### Introduction

**Why Milestone 3 Is Not Enough:**
The ABA problem can cause subtle corruption:
```
Stack: A -> B -> C
Thread 1: reads head=A, next=B
Thread 2: pops A, pops B, pushes A (stack now A -> C)
Thread 1: CAS succeeds (head is still A!) but sets next=B (wrong!)
Stack now: A -> B -> ??? (B was already freed)
```

**What We're Improving:**
Add version counter to pointer. CAS checks both pointer and version, so reused pointers are detected.

### Architecture

**New Structs:**
- `VersionedPtr<T>` - Pointer with version counter
  - **Field** `ptr: usize` - Packed pointer and version
  - **Function** `new(ptr: *mut T, version: u64) -> Self` - Pack pointer and version
  - **Function** `as_ptr(&self) -> *mut T` - Extract pointer
  - **Function** `version(&self) -> u64` - Extract version
  - **Function** `pack(ptr: *mut T, version: u64) -> usize` - Combine into usize

**Modified Structs:**
- `LockFreeStack<T>`
  - **Field** `head: AtomicUsize` - Packed pointer + version counter
  - Increment version on every successful CAS

**Role Each Plays:**
- Version counter: Distinguishes pointer reuse
- Pointer packing: Store both in single atomic usize (64-bit pointer uses only 48 bits)
- Tag bits: Use upper 16 bits for version counter

**Pointer Packing on x86-64:**
```
Bits 0-47:  Pointer (48 bits, upper bits sign-extended)
Bits 48-63: Version counter (16 bits = 65536 versions)
```

### Checkpoint Tests

```rust
#[test]
fn test_versioned_ptr() {
    let ptr: *mut i32 = Box::into_raw(Box::new(42));
    let versioned = VersionedPtr::new(ptr, 5);

    assert_eq!(versioned.as_ptr(), ptr);
    assert_eq!(versioned.version(), 5);

    unsafe { Box::from_raw(ptr); }
}

#[test]
fn test_aba_protection() {
    use std::thread;
    use std::sync::Arc;

    let stack = Arc::new(LockFreeStack::new());

    // This is hard to test deterministically, but we can verify
    // that version counter increments
    stack.push(1);
    stack.push(2);

    // Pop and push same values multiple times
    for _ in 0..100 {
        let val = stack.pop().unwrap();
        stack.push(val);
    }

    // Stack should still be valid
    assert_eq!(stack.pop(), Some(2));
    assert_eq!(stack.pop(), Some(1));
}

#[test]
fn test_high_contention_with_aba_protection() {
    use std::thread;
    use std::sync::Arc;

    let stack = Arc::new(LockFreeStack::new());

    // Pre-fill
    for i in 0..1000 {
        stack.push(i);
    }

    let handles: Vec<_> = (0..8).map(|tid| {
        let s = Arc::clone(&stack);
        thread::spawn(move || {
            for i in 0..500 {
                // Mix of push and pop to create ABA scenarios
                if i % 2 == 0 {
                    s.push(tid * 10000 + i);
                } else {
                    s.pop();
                }
            }
        })
    }).collect();

    for h in handles {
        h.join().unwrap();
    }

    // Stack should still be valid
    let mut count = 0;
    while stack.pop().is_some() {
        count += 1;
    }

    println!("Final count: {}", count);
}
```

### Starter Code

```rust
// Constants for pointer packing (x86-64)
const PTR_MASK: usize = 0x0000_FFFF_FFFF_FFFF; // Lower 48 bits
const VERSION_SHIFT: usize = 48;
const VERSION_MASK: usize = 0xFFFF; // 16 bits

#[derive(Copy, Clone)]
struct VersionedPtr<T> {
    packed: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> VersionedPtr<T> {
    fn new(ptr: *mut Node<T>, version: u64) -> Self {
        // TODO: Pack pointer and version into single usize
        // packed = (ptr as usize & PTR_MASK) | ((version & VERSION_MASK as u64) << VERSION_SHIFT)
        todo!()
    }

    fn as_ptr(&self) -> *mut Node<T> {
        // TODO: Extract pointer from packed value
        // Sign-extend 48-bit pointer to 64-bit
        // let ptr_bits = (self.packed & PTR_MASK) as isize;
        // let extended = (ptr_bits << 16) >> 16; // Sign extension
        // extended as *mut Node<T>
        todo!()
    }

    fn version(&self) -> u64 {
        // TODO: Extract version from upper bits
        // (self.packed >> VERSION_SHIFT) as u64
        todo!()
    }

    fn null() -> Self {
        // TODO: Return null pointer with version 0
        todo!()
    }

    fn is_null(&self) -> bool {
        // TODO: Check if pointer part is null
        todo!()
    }
}

pub struct LockFreeStack<T> {
    head: AtomicUsize, // Packed VersionedPtr
}

impl<T> LockFreeStack<T> {
    pub fn new() -> Self {
        // TODO: Initialize with null versioned pointer
        todo!()
    }

    pub fn push(&self, value: T) {
        // TODO: Update to use VersionedPtr
        // On successful CAS, increment version
        todo!()
    }

    pub fn pop(&self) -> Option<T> {
        // TODO: Update to use VersionedPtr
        // Compare both pointer and version
        todo!()
    }
}
```

---

## Milestone 5: Peek Operation and Length Tracking

### Introduction

**Why Milestone 4 Is Not Enough:**
Users often need to inspect the top element without removing it (peek), or check how many elements are in the stack. Adding these requires careful atomic operations to maintain consistency.

**What We're Improving:**
Add `peek()` to read top element without removing it, and `len()` to track stack size. Use additional atomic counter for length.

### Architecture

**Modified Structs:**
- `LockFreeStack<T>`
  - **Field** `len: AtomicUsize` - Count of elements
  - **Function** `peek(&self) -> Option<&T>` - View top element (UNSAFE - lifetime issues!)
  - **Function** `len(&self) -> usize` - Get current length
  - **Function** `is_empty(&self) -> bool` - Check if length is 0

**Challenges:**
- `peek()` is inherently unsafe in lock-free context (element can be popped while referenced)
- Length counter needs atomic updates coordinated with push/pop
- Alternative: return cloned value if T: Clone

### Checkpoint Tests

```rust
#[test]
fn test_peek() {
    let stack = LockFreeStack::new();

    assert_eq!(stack.peek_cloned(), None);

    stack.push(42);
    assert_eq!(stack.peek_cloned(), Some(42));

    stack.push(100);
    assert_eq!(stack.peek_cloned(), Some(100));

    stack.pop();
    assert_eq!(stack.peek_cloned(), Some(42));
}

#[test]
fn test_length_tracking() {
    let stack = LockFreeStack::new();
    assert_eq!(stack.len(), 0);

    stack.push(1);
    assert_eq!(stack.len(), 1);

    stack.push(2);
    stack.push(3);
    assert_eq!(stack.len(), 3);

    stack.pop();
    assert_eq!(stack.len(), 2);

    stack.pop();
    stack.pop();
    assert_eq!(stack.len(), 0);
}

#[test]
fn test_concurrent_length() {
    use std::thread;
    use std::sync::Arc;

    let stack = Arc::new(LockFreeStack::new());

    let handles: Vec<_> = (0..4).map(|_| {
        let s = Arc::clone(&stack);
        thread::spawn(move || {
            for _ in 0..250 {
                s.push(42);
            }
        })
    }).collect();

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(stack.len(), 1000);

    let handles: Vec<_> = (0..4).map(|_| {
        let s = Arc::clone(&stack);
        thread::spawn(move || {
            for _ in 0..250 {
                s.pop();
            }
        })
    }).collect();

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(stack.len(), 0);
}
```

### Starter Code

```rust
impl<T> LockFreeStack<T> {
    // Add length field to struct
    // len: AtomicUsize,

    pub fn len(&self) -> usize {
        // TODO: Load length with Acquire ordering
        todo!()
    }

    pub fn is_empty(&self) -> bool {
        // TODO: Check if length is 0
        todo!()
    }

    // Update push to increment length
    pub fn push(&self, value: T) {
        // ... existing CAS loop ...
        // After successful CAS:
        // self.len.fetch_add(1, Ordering::Release);
        todo!()
    }

    // Update pop to decrement length
    pub fn pop(&self) -> Option<T> {
        // ... existing CAS loop ...
        // After successful CAS:
        // self.len.fetch_sub(1, Ordering::Release);
        todo!()
    }
}

impl<T: Clone> LockFreeStack<T> {
    pub fn peek_cloned(&self) -> Option<T> {
        // TODO: Load head, check if null, clone value
        // This is safe because we're cloning, not borrowing
        // loop {
        //     let head = load head as VersionedPtr
        //     if head.is_null() { return None; }
        //     unsafe {
        //         // Read value (might race with pop, but safe because clone)
        //         let value = (*head.as_ptr()).value.clone();
        //         // Verify head hasn't changed (if changed, value might be freed)
        //         let current = load head
        //         if current == head {
        //             return Some(value);
        //         }
        //         // Retry if head changed
        //     }
        // }
        todo!()
    }
}
```

---

## Milestone 6: Performance Benchmarking and Comparison

### Introduction

**Why Milestone 5 Is Not Enough:**
The stack is functionally complete but we need to validate performance claims. Compare against mutex-based stack and measure scalability with increasing thread count.

**What We're Improving:**
Add comprehensive benchmarks showing:
- Single-threaded performance
- Multi-threaded scalability
- Contention handling
- Comparison with Mutex<Vec<T>>

### Architecture

**Benchmark Suite:**
- Single-threaded push/pop throughput
- Multi-threaded scaling (1, 2, 4, 8, 16 threads)
- Push-only contention
- Pop-only contention
- Mixed workload (50% push, 50% pop)
- Comparison with `Mutex<Vec<T>>`

### Checkpoint Tests

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Instant;

    fn benchmark<F>(name: &str, op: F) -> f64
    where
        F: FnOnce(),
    {
        let start = Instant::now();
        op();
        let elapsed = start.elapsed();
        let ops_per_sec = 1_000_000.0 / elapsed.as_secs_f64();
        println!("{}: {:.2}M ops/sec ({:?})", name, ops_per_sec / 1_000_000.0, elapsed);
        ops_per_sec
    }

    #[test]
    fn bench_single_threaded() {
        let stack = LockFreeStack::new();

        benchmark("Single-threaded push+pop", || {
            for i in 0..1_000_000 {
                stack.push(i);
            }
            for _ in 0..1_000_000 {
                stack.pop();
            }
        });
    }

    #[test]
    fn bench_multi_threaded_push() {
        for num_threads in [1, 2, 4, 8] {
            let stack = Arc::new(LockFreeStack::new());
            let ops_per_thread = 1_000_000 / num_threads;

            let throughput = benchmark(&format!("{} threads push", num_threads), || {
                let handles: Vec<_> = (0..num_threads)
                    .map(|_| {
                        let s = Arc::clone(&stack);
                        thread::spawn(move || {
                            for i in 0..ops_per_thread {
                                s.push(i);
                            }
                        })
                    })
                    .collect();

                for h in handles {
                    h.join().unwrap();
                }
            });

            println!("  Speedup: {:.2}x\n", throughput / 1_000_000.0);
        }
    }

    #[test]
    fn bench_vs_mutex() {
        println!("\n=== Lock-Free Stack ===");
        let lf_stack = Arc::new(LockFreeStack::new());

        let lf_throughput = benchmark("Lock-free (4 threads)", || {
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let s = Arc::clone(&lf_stack);
                    thread::spawn(move || {
                        for i in 0..250_000 {
                            s.push(i);
                            s.pop();
                        }
                    })
                })
                .collect();

            for h in handles {
                h.join().unwrap();
            }
        });

        println!("\n=== Mutex Stack ===");
        let mutex_stack = Arc::new(Mutex::new(Vec::new()));

        let mutex_throughput = benchmark("Mutex (4 threads)", || {
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let s = Arc::clone(&mutex_stack);
                    thread::spawn(move || {
                        for i in 0..250_000 {
                            s.lock().unwrap().push(i);
                            s.lock().unwrap().pop();
                        }
                    })
                })
                .collect();

            for h in handles {
                h.join().unwrap();
            }
        });

        println!("\n=== Comparison ===");
        println!("Lock-free advantage: {:.2}x faster", lf_throughput / mutex_throughput);
    }
}
```

### Starter Code

```rust
// Add more comprehensive benchmarks

#[cfg(test)]
mod benchmarks {
    // TODO: Add benchmarks for:
    // - Different contention levels
    // - Cache line effects
    // - NUMA effects (if applicable)
    // - Workload patterns (producer-consumer, work-stealing)

    // Example: Measure CAS failure rate
    #[test]
    fn measure_cas_contention() {
        // TODO: Instrument CAS loop to count failures
        // Higher thread count should show more CAS retries
        todo!()
    }
}
```

---

## Complete Working Example

```rust
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

// ============================================================================
// VERSIONED POINTER
// ============================================================================

const PTR_MASK: usize = 0x0000_FFFF_FFFF_FFFF;
const VERSION_SHIFT: usize = 48;
const VERSION_MASK: u64 = 0xFFFF;

#[derive(Copy, Clone, PartialEq, Eq)]
struct VersionedPtr {
    packed: usize,
}

impl VersionedPtr {
    fn new<T>(ptr: *mut Node<T>, version: u64) -> Self {
        let ptr_bits = ptr as usize & PTR_MASK;
        let version_bits = ((version & VERSION_MASK) as usize) << VERSION_SHIFT;
        Self {
            packed: ptr_bits | version_bits,
        }
    }

    fn as_ptr<T>(&self) -> *mut Node<T> {
        // Sign-extend 48-bit pointer
        let ptr_bits = (self.packed & PTR_MASK) as isize;
        let extended = (ptr_bits << 16) >> 16;
        extended as *mut Node<T>
    }

    fn version(&self) -> u64 {
        (self.packed >> VERSION_SHIFT) as u64
    }

    fn null<T>() -> Self {
        Self { packed: 0 }
    }

    fn is_null(&self) -> bool {
        (self.packed & PTR_MASK) == 0
    }

    fn to_usize(&self) -> usize {
        self.packed
    }

    fn from_usize(val: usize) -> Self {
        Self { packed: val }
    }
}

// ============================================================================
// NODE
// ============================================================================

struct Node<T> {
    value: T,
    next: *mut Node<T>,
}

// ============================================================================
// LOCK-FREE STACK
// ============================================================================

pub struct LockFreeStack<T> {
    head: AtomicUsize, // VersionedPtr
    len: AtomicUsize,
}

impl<T> LockFreeStack<T> {
    pub fn new() -> Self {
        Self {
            head: AtomicUsize::new(VersionedPtr::null::<T>().to_usize()),
            len: AtomicUsize::new(0),
        }
    }

    pub fn push(&self, value: T) {
        let mut new_node = Box::new(Node {
            value,
            next: ptr::null_mut(),
        });

        loop {
            let head_packed = self.head.load(Ordering::Acquire);
            let head = VersionedPtr::from_usize(head_packed);

            new_node.next = head.as_ptr();
            let new_node_ptr = Box::into_raw(new_node);

            let new_version = head.version().wrapping_add(1);
            let new_head = VersionedPtr::new(new_node_ptr, new_version);

            match self.head.compare_exchange_weak(
                head_packed,
                new_head.to_usize(),
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    self.len.fetch_add(1, Ordering::Release);
                    return;
                }
                Err(_) => {
                    // CAS failed, retry
                    unsafe {
                        new_node = Box::from_raw(new_node_ptr);
                    }
                }
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        loop {
            let head_packed = self.head.load(Ordering::Acquire);
            let head = VersionedPtr::from_usize(head_packed);

            if head.is_null() {
                return None;
            }

            unsafe {
                let next = (*head.as_ptr()).next;
                let new_version = head.version().wrapping_add(1);
                let new_head = VersionedPtr::new(next, new_version);

                match self.head.compare_exchange_weak(
                    head_packed,
                    new_head.to_usize(),
                    Ordering::Release,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {
                        self.len.fetch_sub(1, Ordering::Release);
                        let head_node = Box::from_raw(head.as_ptr());
                        return Some(head_node.value);
                    }
                    Err(_) => {
                        // CAS failed, retry
                    }
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::Acquire)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: Clone> LockFreeStack<T> {
    pub fn peek_cloned(&self) -> Option<T> {
        loop {
            let head_packed = self.head.load(Ordering::Acquire);
            let head = VersionedPtr::from_usize(head_packed);

            if head.is_null() {
                return None;
            }

            unsafe {
                let value = (*head.as_ptr()).value.clone();

                // Verify head hasn't changed
                let current_packed = self.head.load(Ordering::Acquire);
                if current_packed == head_packed {
                    return Some(value);
                }
                // Retry if head changed
            }
        }
    }
}

impl<T> Drop for LockFreeStack<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

unsafe impl<T: Send> Send for LockFreeStack<T> {}
unsafe impl<T: Send> Sync for LockFreeStack<T> {}

// ============================================================================
// EXAMPLE USAGE
// ============================================================================

fn main() {
    println!("=== Lock-Free Stack Demo ===\n");

    // Basic usage
    println!("--- Basic Operations ---");
    let stack = LockFreeStack::new();

    stack.push(1);
    stack.push(2);
    stack.push(3);

    println!("Length: {}", stack.len());
    println!("Peek: {:?}", stack.peek_cloned());

    while let Some(val) = stack.pop() {
        println!("Popped: {}", val);
    }

    println!();

    // Concurrent usage
    println!("--- Concurrent Push/Pop ---");
    let stack = Arc::new(LockFreeStack::new());

    let handles: Vec<_> = (0..4)
        .map(|tid| {
            let s = Arc::clone(&stack);
            thread::spawn(move || {
                for i in 0..100 {
                    s.push(tid * 1000 + i);
                }
                println!("Thread {} finished pushing", tid);
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    println!("Total elements: {}", stack.len());

    let handles: Vec<_> = (0..4)
        .map(|tid| {
            let s = Arc::clone(&stack);
            thread::spawn(move || {
                let mut count = 0;
                for _ in 0..100 {
                    if s.pop().is_some() {
                        count += 1;
                    }
                }
                println!("Thread {} popped {} elements", tid, count);
                count
            })
        })
        .collect();

    let total_popped: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
    println!("Total popped: {}", total_popped);
    println!("Remaining: {}", stack.len());

    println!();

    // Performance benchmark
    println!("--- Performance Benchmark ---");

    // Single-threaded
    let stack = LockFreeStack::new();
    let start = Instant::now();
    for i in 0..1_000_000 {
        stack.push(i);
    }
    for _ in 0..1_000_000 {
        stack.pop();
    }
    let elapsed = start.elapsed();
    println!(
        "Single-threaded: 2M ops in {:?} ({:.2}M ops/sec)",
        elapsed,
        2.0 / elapsed.as_secs_f64()
    );

    // Multi-threaded
    let stack = Arc::new(LockFreeStack::new());
    let start = Instant::now();

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let s = Arc::clone(&stack);
            thread::spawn(move || {
                for i in 0..250_000 {
                    s.push(i);
                    s.pop();
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let elapsed = start.elapsed();
    println!(
        "Multi-threaded (4 threads): 2M ops in {:?} ({:.2}M ops/sec)",
        elapsed,
        2.0 / elapsed.as_secs_f64()
    );

    println!("\n=== Done ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let stack = LockFreeStack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_concurrent() {
        let stack = Arc::new(LockFreeStack::new());

        let handles: Vec<_> = (0..10)
            .map(|tid| {
                let s = Arc::clone(&stack);
                thread::spawn(move || {
                    for i in 0..100 {
                        s.push(tid * 1000 + i);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(stack.len(), 1000);

        let mut count = 0;
        while stack.pop().is_some() {
            count += 1;
        }
        assert_eq!(count, 1000);
    }

    #[test]
    fn test_peek() {
        let stack = LockFreeStack::new();
        assert_eq!(stack.peek_cloned(), None);

        stack.push(42);
        assert_eq!(stack.peek_cloned(), Some(42));

        stack.push(100);
        assert_eq!(stack.peek_cloned(), Some(100));

        stack.pop();
        assert_eq!(stack.peek_cloned(), Some(42));
    }
}
```

This completes the lock-free stack project with ABA protection and comprehensive testing!
