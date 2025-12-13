
# Lock-Free Stack (Treiber Stack)

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

## Key Concepts Explained

This project requires understanding atomic pointers, compare-and-swap loops, the ABA problem, and memory reclamation in lock-free data structures. These concepts are fundamental to building correct concurrent data structures without locks.

### Atomic Pointers: AtomicPtr<T>

**What It Is**: An atomic reference to heap-allocated data, enabling lock-free manipulation of linked data structures.

**Why We Need It**:

```rust
// NON-ATOMIC (Race condition):
struct Stack {
    head: *mut Node,  // Regular raw pointer
}

impl Stack {
    fn push(&mut self, node: *mut Node) {
        unsafe {
            (*node).next = self.head;  // Thread A reads head
            // Context switch!
            // Thread B pushes, changes head
            self.head = node;  // Thread A writes stale head - LOST UPDATE!
        }
    }
}
```

**The Atomic Solution**:

```rust
use std::sync::atomic::{AtomicPtr, Ordering};

struct Stack {
    head: AtomicPtr<Node>,  // Atomic pointer
}

impl Stack {
    fn push(&self, node: *mut Node) {
        loop {
            let old_head = self.head.load(Ordering::Relaxed);
            unsafe { (*node).next = old_head; }

            // CAS: only succeeds if head unchanged
            if self.head.compare_exchange_weak(
                old_head,
                node,
                Ordering::Release,
                Ordering::Relaxed,
            ).is_ok() {
                break;  // Success!
            }
            // If failed, retry with updated head
        }
    }
}
```

**AtomicPtr Operations**:

```rust
let head = AtomicPtr::new(ptr::null_mut());

// Load (read the pointer)
let current = head.load(Ordering::Acquire);

// Store (write a new pointer)
head.store(new_ptr, Ordering::Release);

// Swap (exchange, return old)
let old = head.swap(new_ptr, Ordering::AcqRel);

// Compare-and-swap (conditional update)
let result = head.compare_exchange(
    expected_ptr,
    new_ptr,
    Ordering::Release,  // Success ordering
    Ordering::Relaxed,  // Failure ordering
);
```

**Memory Safety Consideration**:

```rust
// AtomicPtr stores *mut T (raw pointer)
// You must ensure:
// 1. Pointer validity (not dangling)
// 2. Proper deallocation (no memory leaks)
// 3. No data races on pointed-to data

// Safe pattern:
let node = Box::new(Node { value: 42, next: ptr::null_mut() });
let raw = Box::into_raw(node);  // Box → raw pointer
head.store(raw, Ordering::Release);

// Later:
let raw = head.load(Ordering::Acquire);
if !raw.is_null() {
    unsafe {
        let node = Box::from_raw(raw);  // raw → Box (deallocates on drop)
        println!("{}", node.value);
    }
}
```

---

### Compare-and-Swap Loops: The Lock-Free Pattern

**The Core Pattern**: Retry until CAS succeeds.

```rust
loop {
    // 1. Read current state
    let current = atomic.load(Ordering::Relaxed);

    // 2. Compute new state based on current
    let new = compute_new_state(current);

    // 3. Try to update (only if current unchanged)
    match atomic.compare_exchange_weak(
        current,
        new,
        Ordering::Release,  // If successful
        Ordering::Relaxed,  // If failed
    ) {
        Ok(_) => break,      // Success! Exit loop
        Err(_) => continue,  // Retry with updated current
    }
}
```

**Why Weak CAS in Loops**:

```rust
// Strong CAS: Never spuriously fails (but slower on some platforms)
compare_exchange(current, new, success_order, failure_order)

// Weak CAS: May spuriously fail even if values match (faster on ARM)
compare_exchange_weak(current, new, success_order, failure_order)

// In a loop, spurious failures just cause one extra iteration
// On ARM, weak CAS is significantly faster
// Always use weak CAS in loops!
```

**Stack Push with CAS Loop**:

```rust
fn push(&self, value: T) {
    let new_node = Box::into_raw(Box::new(Node {
        value,
        next: ptr::null_mut(),
    }));

    loop {
        // Read current head
        let old_head = self.head.load(Ordering::Relaxed);

        // Link new node to current head
        unsafe { (*new_node).next = old_head; }

        // Try to swing head to new node
        match self.head.compare_exchange_weak(
            old_head,
            new_node,
            Ordering::Release,  // Publish new node
            Ordering::Relaxed,  // Retry on failure
        ) {
            Ok(_) => return,  // Success!
            Err(_) => {
                // Another thread modified head
                // Loop will retry with updated old_head
            }
        }
    }
}
```

**Why This Works**:
- Multiple threads can push simultaneously
- Each CAS attempt is atomic
- Only one CAS succeeds per head change
- Failed threads see updated head and retry
- No locks, always progress

**Ordering Choice**:
- **Load**: `Relaxed` (just need current value, retry if stale)
- **Success CAS**: `Release` (publish new node to other threads)
- **Failure CAS**: `Relaxed` (no synchronization needed on retry)

---

### The ABA Problem: A Subtle Race Condition

**What It Is**: CAS succeeds because value matches, but the value changed and then changed back.

**Classic Example**:

```
Initial state: Stack = [A → B → C]

Thread 1:
  1. Read head = A
  2. Read A.next = B
  3. Prepare CAS(head, A → B)
  ... interrupted ...

Thread 2:
  4. Pop A  → Stack = [B → C]
  5. Pop B  → Stack = [C]
  6. Push A → Stack = [A → C]  ← A is back!

Thread 1 resumes:
  7. CAS(head, A → B)  ← SUCCEEDS! head == A
  8. Stack = [B → ???]  ← B.next is now invalid!
```

**The Problem Visualized**:

```
Before Thread 1's CAS:
head: A → C

Thread 1 thinks:
head: A → B → C

After Thread 1's CAS:
head: B → ??? (B.next was deallocated!)
```

**Result**: Undefined behavior (dangling pointer, use-after-free)

**Why It's Called ABA**:
- Value was **A**
- Changed to **B** (and possibly other values)
- Changed back to **A**
- CAS sees A → A and succeeds incorrectly

---

### ABA Problem Solutions

#### Solution 1: Version Counters (Tagged Pointers)

**Idea**: Combine pointer with a version counter. CAS checks both.

```rust
use std::sync::atomic::AtomicU128;

#[repr(C)]
struct TaggedPtr {
    ptr: *mut Node,      // 64 bits
    version: u64,        // 64 bits
}

// Pack into 128-bit atomic (requires nightly Rust or cmpxchg16b)
struct Stack {
    head: AtomicU128,  // Stores TaggedPtr as u128
}

impl Stack {
    fn push(&self, node: *mut Node) {
        loop {
            let current_u128 = self.head.load(Ordering::Relaxed);
            let current = TaggedPtr::from_u128(current_u128);

            unsafe { (*node).next = current.ptr; }

            let new = TaggedPtr {
                ptr: node,
                version: current.version + 1,  // Increment version!
            };

            if self.head.compare_exchange_weak(
                current_u128,
                new.to_u128(),
                Ordering::Release,
                Ordering::Relaxed,
            ).is_ok() {
                break;
            }
        }
    }
}

// ABA scenario:
// Thread 1 reads: (ptr=A, version=5)
// Thread 2: pop A, push A → (ptr=A, version=7)
// Thread 1 CAS: expect (A, 5), got (A, 7) → FAILS! (version mismatch)
```

**Pros**:
- ✅ Completely solves ABA problem
- ✅ Simple to understand

**Cons**:
- ❌ Requires 128-bit CAS (not all platforms support)
- ❌ Version can overflow (rare but possible)
- ❌ Larger memory footprint

#### Solution 2: Hazard Pointers

**Idea**: Threads announce which pointers they're using. Don't reclaim announced pointers.

```rust
struct HazardPointer {
    protected: AtomicPtr<Node>,
}

thread_local! {
    static HAZARD: HazardPointer = HazardPointer {
        protected: AtomicPtr::new(ptr::null_mut()),
    };
}

impl Stack {
    fn pop(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            if head.is_null() {
                return None;
            }

            // Announce: I'm using this pointer!
            HAZARD.with(|hp| hp.protected.store(head, Ordering::Release));

            // Re-check head hasn't changed
            if self.head.load(Ordering::Acquire) != head {
                continue;  // Retry
            }

            unsafe {
                let next = (*head).next;

                if self.head.compare_exchange_weak(
                    head,
                    next,
                    Ordering::Release,
                    Ordering::Relaxed,
                ).is_ok() {
                    let value = ptr::read(&(*head).value);

                    // Can't free head yet - another thread might have it in hazard
                    retire_node(head);  // Add to deferred free list

                    return Some(value);
                }
            }
        }
    }
}

fn retire_node(node: *mut Node) {
    // Check if any thread has node in their hazard pointer
    if no_hazards_for(node) {
        unsafe { Box::from_raw(node); }  // Free immediately
    } else {
        RETIRE_LIST.push(node);  // Defer until safe
    }
}
```

**Pros**:
- ✅ Works on all platforms
- ✅ Prevents use-after-free

**Cons**:
- ❌ Complex implementation
- ❌ Memory overhead (hazard pointers per thread)
- ❌ Deferred reclamation (memory stays allocated)

#### Solution 3: Epoch-Based Reclamation (EBR)

**Idea**: Track global "epochs". Threads announce current epoch. Only reclaim memory from old epochs.

```rust
static EPOCH: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static LOCAL_EPOCH: Cell<usize> = Cell::new(0);
}

struct Stack {
    head: AtomicPtr<Node>,
}

impl Stack {
    fn pop(&self) -> Option<T> {
        // Enter current epoch
        let epoch = EPOCH.load(Ordering::Acquire);
        LOCAL_EPOCH.with(|e| e.set(epoch));

        loop {
            let head = self.head.load(Ordering::Acquire);
            if head.is_null() {
                return None;
            }

            unsafe {
                let next = (*head).next;

                if self.head.compare_exchange_weak(
                    head,
                    next,
                    Ordering::Release,
                    Ordering::Relaxed,
                ).is_ok() {
                    let value = ptr::read(&(*head).value);

                    // Retire node in current epoch
                    retire_in_epoch(head, epoch);

                    return Some(value);
                }
            }
        }
    }
}

// Periodically advance epoch
fn advance_epoch() {
    EPOCH.fetch_add(1, Ordering::Release);

    // Free nodes from epochs where no threads are active
    reclaim_old_epochs();
}
```

**Pros**:
- ✅ Excellent performance
- ✅ Amortized reclamation (batch frees)
- ✅ Used by Crossbeam

**Cons**:
- ❌ Complex implementation
- ❌ Requires global coordination
- ❌ Memory usage spikes (delayed reclamation)

---

### Memory Reclamation: The Fundamental Challenge

**The Problem**: Can't free nodes immediately—another thread might be accessing them.

```rust
// UNSAFE - DON'T DO THIS:
fn pop(&self) -> Option<T> {
    loop {
        let head = self.head.load(Ordering::Acquire);
        if head.is_null() {
            return None;
        }

        unsafe {
            let next = (*head).next;

            if self.head.compare_exchange_weak(
                head,
                next,
                Ordering::Release,
                Ordering::Relaxed,
            ).is_ok() {
                let value = ptr::read(&(*head).value);
                drop(Box::from_raw(head));  // ❌ FREE IMMEDIATELY - WRONG!
                // Another thread might have read head and is about to access it!
                return Some(value);
            }
        }
    }
}
```

**Race Condition**:

```
Thread 1:
  1. let head = self.head.load(...)  // head = A
  2. let next = (*head).next         // Read A.next = B
  ... interrupted ...

Thread 2:
  3. Pop A successfully
  4. drop(Box::from_raw(A))  ← A deallocated!

Thread 1 resumes:
  5. CAS(head, A → B)  ← Accessing freed memory!
```

**Safe Strategies**:

**1. Leak Memory** (Simplest for learning):
```rust
// Never free - acceptable for educational projects
if self.head.compare_exchange_weak(...).is_ok() {
    let value = ptr::read(&(*head).value);
    std::mem::forget(head);  // Leak the node
    return Some(value);
}
```

**2. Reference Counting**:
```rust
struct Node {
    value: T,
    next: *mut Node,
    ref_count: AtomicUsize,  // Track references
}

// Increment on access, decrement when done
// Free when ref_count reaches 0
// Problem: Overhead of atomic increments
```

**3. Deferred Reclamation** (Hazard Pointers, EBR):
- Don't free immediately
- Add to "retire list"
- Periodically scan and free safe nodes
- Balances safety and performance

**4. Garbage Collection**:
```rust
// In Java/Go: Let GC handle it
// In Rust: Not available (manual memory management)
```

---

### Memory Ordering for Linked Structures

**Key Insight**: Publication and consumption require synchronization.

**Push Operation**:

```rust
fn push(&self, value: T) {
    let new_node = Box::into_raw(Box::new(Node {
        value,  // Initialize value
        next: ptr::null_mut(),
    }));

    loop {
        let old_head = self.head.load(Ordering::Relaxed);
        unsafe { (*new_node).next = old_head; }

        if self.head.compare_exchange_weak(
            old_head,
            new_node,
            Ordering::Release,  // ← CRITICAL: Publish new node
            Ordering::Relaxed,
        ).is_ok() {
            break;
        }
    }
}
```

**Why Release**:
- All writes to `new_node` (value, next) happen before CAS
- `Release` ensures those writes visible to threads that Acquire
- Without Release, another thread might see uninitialized value!

**Pop Operation**:

```rust
fn pop(&self) -> Option<T> {
    loop {
        let head = self.head.load(Ordering::Acquire);  // ← CRITICAL
        if head.is_null() {
            return None;
        }

        unsafe {
            let next = (*head).next;
            let value = ptr::read(&(*head).value);  // Read value

            if self.head.compare_exchange_weak(
                head,
                next,
                Ordering::Release,
                Ordering::Relaxed,
            ).is_ok() {
                return Some(value);
            }
        }
    }
}
```

**Why Acquire**:
- Synchronizes with the Release store from push
- Ensures we see the fully initialized node
- Without Acquire, might see garbage in `value` or `next`!

**Ordering Summary**:

| Operation | Ordering | Reason |
|-----------|----------|--------|
| Load head (read) | Acquire | See published node data |
| Store head (push) | Release | Publish new node |
| CAS success | Release | Publish changes |
| CAS failure | Relaxed | No synchronization needed (retry) |

**Relaxed Reads in Loop**:

Some implementations use Relaxed for reads that will be validated by CAS:

```rust
// Read with Relaxed, CAS validates
let old_head = self.head.load(Ordering::Relaxed);
unsafe { (*new_node).next = old_head; }

// CAS with Acquire success ordering validates the read
if self.head.compare_exchange_weak(
    old_head,
    new_node,
    Ordering::Release,
    Ordering::Acquire,  // ← Acquire on success
).is_ok() { ... }
```

But for safety, many prefer Acquire reads to avoid subtle bugs.

---

### Lock-Free Stack Design Patterns

**Pattern 1: Try-Lock (Spin Until Success)**

```rust
pub fn push(&self, value: T) {
    let new_node = ...;

    loop {
        let old_head = self.head.load(Ordering::Relaxed);
        unsafe { (*new_node).next = old_head; }

        if self.head.compare_exchange_weak(...).is_ok() {
            break;  // Success
        }
        // Spin and retry
    }
}
```

**Characteristics**:
- Simple, clean code
- Burns CPU on contention
- Good for low contention scenarios

**Pattern 2: Backoff (Reduce Contention)**

```rust
use std::hint::spin_loop;

pub fn push(&self, value: T) {
    let new_node = ...;
    let mut backoff = 1;

    loop {
        let old_head = self.head.load(Ordering::Relaxed);
        unsafe { (*new_node).next = old_head; }

        if self.head.compare_exchange_weak(...).is_ok() {
            break;
        }

        // Exponential backoff
        for _ in 0..backoff {
            spin_loop();  // Hint to CPU: reduce power, let other threads run
        }
        backoff = (backoff * 2).min(64);  // Cap at 64 iterations
    }
}
```

**Characteristics**:
- Reduces cache line bouncing
- Better performance under contention
- More complex code

**Pattern 3: Try-With-Limit (Fallback)**

```rust
pub fn try_push(&self, value: T, max_attempts: usize) -> Result<(), T> {
    let new_node = ...;

    for _ in 0..max_attempts {
        let old_head = self.head.load(Ordering::Relaxed);
        unsafe { (*new_node).next = old_head; }

        if self.head.compare_exchange_weak(...).is_ok() {
            return Ok(());
        }
    }

    // Failed after max_attempts
    unsafe { Box::from_raw(new_node); }  // Clean up
    Err(value)
}
```

**Characteristics**:
- Bounded retry attempts
- Allows fallback strategy
- Useful for real-time systems

---

### Unsafe Rust in Lock-Free Structures

Lock-free data structures require `unsafe` for raw pointer manipulation. Understanding what makes it safe is critical.

**Unsafe Operations Used**:

1. **Dereferencing raw pointers**:
```rust
unsafe {
    let next = (*head).next;  // Dereference *mut Node
}
```

**Safety invariant**: `head` must be valid, non-null, properly aligned

2. **Creating references from raw pointers**:
```rust
unsafe {
    let node_ref = &*head;  // *mut Node → &Node
}
```

**Safety invariant**: No mutable aliasing, pointer valid for reference lifetime

3. **Pointer arithmetic** (in more complex structures):
```rust
unsafe {
    let next_ptr = head.offset(1);  // Move pointer
}
```

**Safety invariant**: Result must be in bounds or one-past-end

4. **Box conversion**:
```rust
// Box → raw pointer
let raw = Box::into_raw(boxed_node);

// raw pointer → Box (takes ownership, will deallocate)
unsafe {
    let boxed_node = Box::from_raw(raw);
}
```

**Safety invariant**:
- Pointer came from `Box::into_raw`
- Only convert once (double-free otherwise)
- Pointer not accessed after conversion

**Safety Checklist for Lock-Free Stack**:

```rust
fn push(&self, value: T) {
    // ✅ SAFE: Box::new allocates valid memory
    let new_node = Box::into_raw(Box::new(Node {
        value,
        next: ptr::null_mut(),
    }));

    loop {
        let old_head = self.head.load(Ordering::Relaxed);

        // ✅ SAFE: new_node valid (just allocated)
        unsafe { (*new_node).next = old_head; }

        if self.head.compare_exchange_weak(...).is_ok() {
            // ✅ SAFE: Published to other threads via Release ordering
            break;
        }
    }
    // ✅ SAFE: new_node now owned by stack, won't be freed
}
```

**Common Unsafe Bugs**:

```rust
// ❌ WRONG: Double-free
let node = Box::into_raw(Box::new(...));
unsafe {
    drop(Box::from_raw(node));
    drop(Box::from_raw(node));  // CRASH: node already freed
}

// ❌ WRONG: Use-after-free
let node = Box::into_raw(Box::new(...));
unsafe {
    drop(Box::from_raw(node));
    let value = (*node).value;  // CRASH: accessing freed memory
}

// ❌ WRONG: Memory leak
let node = Box::into_raw(Box::new(...));
// Never freed - leaked!

// ❌ WRONG: Dangling pointer
let node = Box::into_raw(Box::new(...));
unsafe {
    drop(Box::from_raw(node));
}
head.store(node, Ordering::Release);  // Storing freed pointer!
```

---

### Connection to This Project

Now that you understand the core concepts, here's how they map to the milestones:

**Milestone 1: Basic Single-Threaded Stack**
- **Concepts Used**: `AtomicPtr`, `Box::into_raw/from_raw`, raw pointer manipulation
- **Why**: Establish foundation of atomic pointers and linked list structure
- **Key Insight**: Even single-threaded, using atomics prepares for concurrency

**Milestone 2: Thread-Safe Push with CAS**
- **Concepts Used**: CAS loops, memory ordering (Acquire/Release), concurrent push
- **Why**: Multiple threads must push without corrupting the stack
- **Key Insight**: CAS loop with Release ordering publishes new nodes safely

**Milestone 3: Thread-Safe Pop with Memory Leak**
- **Concepts Used**: CAS for pop, reading node data, accepting memory leaks
- **Why**: Pop is harder—must handle empty stack and concurrent modifications
- **Key Insight**: Leaking memory is acceptable for learning; production needs reclamation

**Milestone 4: ABA Problem Demonstration**
- **Concepts Used**: ABA scenario construction, version counters or tagged pointers
- **Why**: Understand the subtle race condition that can corrupt lock-free structures
- **Key Insight**: Simple CAS isn't enough; need version tracking or hazard pointers

**Milestone 5: Hazard Pointers (Basic)**
- **Concepts Used**: Thread-local hazard pointers, deferred reclamation
- **Why**: Safe memory reclamation without leaking
- **Key Insight**: Announce usage before access, defer frees of protected pointers

**Milestone 6: Performance Benchmarks**
- **Concepts Used**: Contention testing, backoff strategies, performance measurement
- **Why**: Validate lock-free benefits vs mutex under different workloads
- **Key Insight**: Lock-free excels under high contention; mutex has lower overhead when uncontended

**Putting It All Together**:

The complete Treiber stack demonstrates:
1. **Atomic pointer operations** for lock-free linked structures
2. **CAS loops** for concurrent modifications
3. **Memory ordering** (Acquire/Release) for safe publication
4. **ABA problem awareness** and mitigation strategies
5. **Memory reclamation** techniques (leak, hazard pointers, EBR)
6. **Unsafe code** with rigorous safety reasoning

This architecture achieves:
- **Lock-free progress**: No deadlocks, always forward progress
- **Linear scalability**: Performance improves with more cores
- **~20-50ns operations**: Faster than mutex under contention
- **Production-ready patterns**: Used in Crossbeam, Tokio, parking_lot

Each milestone builds understanding from basic atomic operations to production-ready lock-free data structures with proper memory management.

---

# Building The Project

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
