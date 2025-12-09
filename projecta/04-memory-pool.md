## Project 4: Memory Pool Allocator

### Problem Statement

Build a custom memory pool allocator that pre-allocates a large block of memory and manages allocations within it. This allocator should efficiently handle fixed-size allocations, track memory usage, and provide better performance than the system allocator for specific workloads.

Your memory pool should support:
- Pre-allocating a fixed-size pool
- Allocating and deallocating fixed-size blocks
- Tracking used/free blocks
- Preventing fragmentation
- Detecting memory leaks and double-frees


## Understanding Memory Pool Allocators

Before implementing the memory pool, let's understand the fundamental concepts that make custom allocators powerful and when they're appropriate.

### What is a Memory Pool Allocator?

A **memory pool allocator** (also called **object pool** or **fixed-size allocator**) pre-allocates a large block of memory and divides it into fixed-size chunks. Instead of asking the operating system for memory on each allocation, you request a chunk from your pool.

**The Problem with System Allocators**:
```
Application requests 64 bytes:
1. System allocator searches free list for suitable block (~100ns)
2. May need to split larger block or coalesce smaller blocks
3. Adds metadata (size, alignment, guard bytes) ~16 bytes overhead
4. Returns pointer to allocated memory
5. On free: reverses process, may coalesce adjacent blocks

Total cost: 50-200ns per allocation (non-deterministic!)
Overhead: ~20% of allocated memory
Fragmentation: Can waste 10-30% of heap over time
```

**Memory Pool Solution**:
```
Pre-allocate 64KB divided into 100 blocks of 640 bytes:

[Block 0][Block 1][Block 2]...[Block 99]
   ↑        ↑        ↑           ↑
  Free    Free    Free        Free

Allocation:
1. Pop index from free list: O(1), ~5ns
2. Return pre-calculated pointer: base + (index * block_size)
3. No metadata, no fragmentation, no system calls

Deallocation:
1. Push index back to free list: O(1), ~5ns
2. No coalescing needed

Total cost: ~10ns per allocation (deterministic!)
Overhead: 0 bytes per allocation
Fragmentation: 0% (fixed-size blocks)
```

---

### Why Use Memory Pool Allocators?

**1. Predictable Performance (Real-Time Systems)**

System allocators are non-deterministic:
```rust
// System allocator (malloc/free)
for i in 0..1000 {
    let data = Box::new([0u8; 64]);
    // Time: anywhere from 50ns to 10,000ns
    // Depends on fragmentation, cache state, system load
}
```

Memory pools are deterministic:
```rust
// Memory pool
let pool = MemoryPool::new(100 * 64, 64);
for i in 0..1000 {
    let ptr = pool.allocate();
    // Time: consistently 5-10ns
    // Always just pop from free list
}
```

**Real-World Example**: Audio processing at 48kHz must process each sample in 20.8μs. A single 10μs malloc stall would cause audible glitches (pops/clicks).

**2. No Fragmentation**

System allocator fragmentation:
```
Initial heap:
[              Free Space                    ]

After many random allocations and frees:
[Used][Free][Used][Free][Free][Used][Free][Used]
  8kb  2kb  16kb  1kb   4kb   32kb  512b  64kb

Request 8kb:
❌ Can't satisfy! Largest contiguous block is 4kb
✓ But 7.5kb total free space exists (wasted!)
```

Memory pool (no fragmentation):
```
Pool with 64-byte blocks:
[Free][Used][Free][Free][Used][Free][Used][Free]

Request 64 bytes:
✓ Always succeeds if any block is free
✓ All blocks same size, no holes
✓ 0% fragmentation by design
```

**3. Improved Cache Locality**

System allocator:
```
malloc(64):  Returns address 0x1000
malloc(64):  Returns address 0x5F80  (24kb away!)
malloc(64):  Returns address 0x2A40  (10kb away!)

Result: Poor cache locality
- Each allocation might be on different cache line
- Walking through objects causes cache misses
- Performance: 100+ cycles per access (RAM)
```

Memory pool:
```
Pool allocates contiguous blocks:
Block 0: 0x10000
Block 1: 0x10040  (+64 bytes)
Block 2: 0x10080  (+64 bytes)

Result: Excellent cache locality
- Sequential objects are adjacent in memory
- Likely share same cache line
- Performance: 4-10 cycles per access (L1 cache)
```

**Benchmark**: Iterating through 1000 objects:
- System allocator: ~50,000 cycles (cache misses)
- Memory pool: ~5,000 cycles (cache hits)
- **10x speedup** from locality alone!

**4. Reduced System Call Overhead**

System allocator:
```rust
// Each allocation might trigger system call
for i in 0..10000 {
    let data = Box::new(Data { ... });
    // May call brk()/sbrk() or mmap()
    // System call: ~1000ns overhead
}
```

Memory pool:
```rust
// Single system call to allocate pool
let pool = MemoryPool::new(64 * 10000, 64);  // One allocation

// All subsequent allocations are user-space only
for i in 0..10000 {
    let ptr = pool.allocate();
    // Pure user-space: ~5ns
    // No syscalls, no kernel involvement
}
```

---

### Memory Pool Architecture

**Core Components**:

```rust
struct MemoryPool {
    memory: Vec<u8>,         // The pre-allocated memory block
    block_size: usize,       // Size of each allocatable chunk
    free_list: Vec<usize>,   // Stack of available block indices
}

Memory Layout:
┌───────────────────────────────────────────┐
│              memory: Vec<u8>              │
├──────┬──────┬──────┬──────┬──────┬────────┤
│Block │Block │Block │Block │Block │...     │
│  0   │  1   │  2   │  3   │  4   │        │
│64 B  │64 B  │64 B  │64 B  │64 B  │        │
└──────┴──────┴──────┴──────┴──────┴────────┘

Free List: [4, 3, 1, 0]  (Block 2 is allocated)
             ↑ top
Next allocation returns pointer to Block 4
```

**Allocation Algorithm**:
```rust
fn allocate(&mut self) -> Option<*mut u8> {
    // Pop an index from free list
    self.free_list.pop().map(|index| {
        // Calculate pointer: base + (index * block_size)
        let offset = index * self.block_size;
        unsafe { self.memory.as_mut_ptr().add(offset) }
    })
}
```

**Deallocation Algorithm**:
```rust
fn deallocate(&mut self, ptr: *mut u8) {
    // Calculate index: (ptr - base) / block_size
    let offset = ptr as usize - self.memory.as_ptr() as usize;
    let index = offset / self.block_size;

    // Push index back onto free list
    self.free_list.push(index);
}
```

**Time Complexity**:
- Allocation: O(1) - pop from Vec
- Deallocation: O(1) - push to Vec
- Both are just a few CPU instructions

---

### Raw Pointers and Unsafe Rust

Memory pools require **unsafe Rust** because we're manually managing memory. Let's understand what that means.

**Safe Rust** (what you normally write):
```rust
let x = Box::new(42);  // Compiler tracks ownership
let y = x;             // Ownership moved to y
// Can't use x anymore - compile error!
// Box automatically frees memory when y goes out of scope
```

**Unsafe Rust** (what memory pools need):
```rust
let ptr: *mut i32 = pool.allocate() as *mut i32;
// ptr is a "raw pointer" - no ownership tracking
// Compiler doesn't know if it's valid
// We must manually ensure:
// 1. Pointer is not null
// 2. Points to valid memory
// 3. Memory is properly aligned
// 4. No use-after-free
// 5. No double-free
// 6. Proper drop handling
```

**Raw Pointer Operations**:

```rust
// 1. Dereferencing (reading/writing)
let ptr: *mut i32 = ...;
unsafe {
    *ptr = 42;          // Write through pointer
    let value = *ptr;   // Read through pointer
}

// 2. Pointer arithmetic
let ptr: *mut u8 = base_ptr;
let ptr2 = unsafe { ptr.add(64) };  // Move pointer 64 bytes forward

// 3. Casting
let byte_ptr: *mut u8 = ...;
let int_ptr: *mut i32 = byte_ptr as *mut i32;

// 4. Writing/reading values
unsafe {
    std::ptr::write(ptr, value);     // Write without dropping old value
    let value = std::ptr::read(ptr); // Read without moving
}
```

**Why These Are Unsafe**:

```rust
// Example 1: Dangling pointer
let ptr: *mut i32 = {
    let x = Box::new(42);
    &mut *x as *mut i32
}; // x dropped, memory freed
unsafe { *ptr }  // ❌ UNDEFINED BEHAVIOR: Reading freed memory

// Example 2: Null pointer dereference
let ptr: *mut i32 = std::ptr::null_mut();
unsafe { *ptr }  // ❌ SEGMENTATION FAULT

// Example 3: Alignment violation
let bytes = vec![0u8; 10];
let ptr = bytes.as_ptr() as *const u64;
unsafe { *ptr }  // ❌ UNDEFINED BEHAVIOR: u64 needs 8-byte alignment

// Example 4: Data race
// Thread 1:
unsafe { *ptr = 42 }
// Thread 2 (simultaneously):
unsafe { *ptr = 100 }  // ❌ DATA RACE
```

**Our Responsibility in Unsafe Code**:

When we write `unsafe`, we're making a **contract** with the compiler:
```rust
unsafe {
    // I, the programmer, guarantee:
    // ✓ This pointer is non-null
    // ✓ Points to valid, initialized memory
    // ✓ Properly aligned for the type
    // ✓ No data races possible
    // ✓ Satisfies all Rust safety invariants
}
```

If we violate this contract: **undefined behavior** (crashes, corruption, security vulnerabilities).

---

### RAII: Resource Acquisition Is Initialization

**RAII** is a pattern where resource lifetime is tied to object lifetime. In Rust, this is implemented via the `Drop` trait.

**The Problem Without RAII**:
```rust
let ptr = pool.allocate();
// ... use ptr ...
pool.deallocate(ptr);  // Easy to forget!

// Or:
let ptr = pool.allocate();
if error_condition {
    return Err(...);  // ❌ MEMORY LEAK: forgot to deallocate!
}
pool.deallocate(ptr);
```

**RAII Solution**:
```rust
struct Block<'a, T> {
    data: *mut T,
    pool: &'a mut TypedPool<T>,
}

impl<T> Drop for Block<'_, T> {
    fn drop(&mut self) {
        // Automatically called when Block goes out of scope
        unsafe {
            std::ptr::drop_in_place(self.data);  // Drop T's contents
        }
        self.pool.deallocate(self.data);  // Return to pool
    }
}

// Usage:
{
    let block = pool.allocate(42).unwrap();  // Block owns the memory
    // ... use block ...
}  // Block dropped here → automatically returns memory to pool!

// Even with early returns:
{
    let block = pool.allocate(42).unwrap();
    if error_condition {
        return Err(...);  // ✓ Block's Drop runs, memory returned
    }
}
```

**Benefits**:
- **No leaks**: Memory automatically returned
- **Exception safe**: Works with panics
- **Composable**: Blocks can contain Blocks
- **Zero overhead**: Drop inlined at compile time

**RAII in Rust Standard Library**:
```rust
// File handle
let file = File::open("data.txt")?;
// ... use file ...
// Drop automatically closes file descriptor

// Mutex guard
let guard = mutex.lock().unwrap();
// ... critical section ...
// Drop automatically releases lock

// Database transaction
let tx = db.transaction()?;
// ... queries ...
tx.commit()?;
// Drop rolls back if commit wasn't called
```

---

### PhantomData: Zero-Sized Type Markers

`PhantomData<T>` tells the compiler about types we use, even though we don't store them directly.

**The Problem**:
```rust
struct TypedPool<T> {
    pool: MemoryPool,
    // We don't actually store any T!
    // But we allocate T and return *mut T
}

// Compiler sees no T field, so:
// - Doesn't enforce T's variance rules
// - Doesn't track T for Send/Sync
// - Optimizes away T entirely
```

**Solution with PhantomData**:
```rust
struct TypedPool<T> {
    pool: MemoryPool,
    _marker: PhantomData<T>,  // "Pretend" we own a T
}

// Now compiler knows:
// - This type is generic over T
// - If T: !Send, then TypedPool<T>: !Send
// - If T: Drop, we need to handle T's drops
// - Variance rules apply
```

**Key Properties**:

1. **Zero Size**:
```rust
assert_eq!(std::mem::size_of::<PhantomData<String>>(), 0);
// Compiles to nothing, no runtime cost!
```

2. **Ownership Semantics**:
```rust
struct Owner<T> {
    data: *mut T,
    _marker: PhantomData<T>,  // Acts like we own T
}

// If T is not Send, Owner<T> is not Send
// If T has Drop, Owner must handle it
```

3. **Different Kinds**:
```rust
PhantomData<T>         // Own T (invariant)
PhantomData<&'a T>     // Borrow &'a T (covariant)
PhantomData<&'a mut T> // Mutably borrow &'a mut T (invariant)
PhantomData<fn(T)>     // Contravariant over T
```

**Real Example**:
```rust
struct TypedPool<T> {
    pool: MemoryPool,
    _marker: PhantomData<T>,
}

// Without PhantomData:
// TypedPool<String> could be Send even if String wasn't
// ❌ UNSOUND: could send non-Send types across threads

// With PhantomData:
// TypedPool<T>: Send only if T: Send
// ✓ SOUND: compiler enforces correct Send/Sync bounds
```

---

### Thread Safety: Arc, Mutex, Send, and Sync

To share memory pools across threads, we need to understand Rust's concurrency primitives.

**Arc: Atomic Reference Counting**

`Arc<T>` is a thread-safe reference-counted pointer. Multiple threads can hold references; memory is freed when the last reference is dropped.

```rust
// Without Arc (won't compile):
let pool = TypedPool::new(100);
thread::spawn(move || {
    pool.allocate(42);  // pool moved here
});
// pool.allocate(100);  // ❌ Error: pool was moved

// With Arc:
let pool = Arc::new(Mutex::new(TypedPool::new(100)));
let pool_clone = Arc::clone(&pool);

thread::spawn(move || {
    pool_clone.lock().unwrap().allocate(42);
});
pool.lock().unwrap().allocate(100);  // ✓ Both threads can access
```

**How Arc Works**:
```
Arc created with count = 1:
┌─────────────────┐
│  Reference      │
│  Count: 1       │
│  ┌──────────┐   │
│  │  Data    │   │
│  │ TypedPool│   │
│  └──────────┘   │
└─────────────────┘

After Arc::clone(&arc):
┌─────────────────┐
│  Reference      │
│  Count: 2  ←────┼─── Atomically incremented
│  ┌──────────┐   │
│  │  Data    │   │
│  │ TypedPool│   │
│  └──────────┘   │
└─────────────────┘
     ↑      ↑
   arc1   arc2

When arc1 drops: count decremented to 1
When arc2 drops: count reaches 0 → data freed
```

**Mutex: Mutual Exclusion**

`Mutex<T>` ensures only one thread at a time can access `T`.

```rust
let mutex = Mutex::new(pool);

// Thread 1:
{
    let guard = mutex.lock().unwrap();  // Blocks if locked
    guard.allocate(42);
    // Lock held
}  // guard dropped → lock released

// Thread 2:
{
    let guard = mutex.lock().unwrap();  // Can now acquire lock
    guard.allocate(100);
}
```

**How Mutex Prevents Data Races**:
```rust
// Without Mutex (won't compile):
let mut pool = TypedPool::new(100);
let pool_ref = &mut pool;  // Only ONE mutable reference allowed
// Can't create second reference → compile error

// With Mutex:
let mutex = Mutex::new(TypedPool::new(100));

// Thread 1:
let mut guard1 = mutex.lock().unwrap();
// Thread 2:
let mut guard2 = mutex.lock().unwrap();  // BLOCKS until thread 1 releases

// Only ONE guard exists at a time → no data races
```

**Send and Sync Traits**

These marker traits define thread safety:

```rust
// Send: Type can be transferred to another thread
// Examples: i32, String, Vec<T> where T: Send
unsafe impl Send for MyType {}

// Sync: Type can be referenced from multiple threads
// Equivalent to: &T is Send
// Examples: i32, AtomicUsize, Mutex<T>
unsafe impl Sync for MyType {}
```

**Rules**:
- `T: Send` means you can move `T` to another thread
- `T: Sync` means you can share `&T` across threads
- `T: Sync` ⟺ `&T: Send`

**Examples**:
```rust
// i32: Send + Sync
let x = 42;
thread::spawn(move || {
    println!("{}", x);  // ✓ Can send i32
});

// Rc<T>: !Send (not thread-safe reference counting)
let rc = Rc::new(42);
thread::spawn(move || {
    println!("{}", rc);  // ❌ Compile error: Rc is not Send
});

// Arc<T>: Send + Sync (atomic reference counting)
let arc = Arc::new(42);
thread::spawn(move || {
    println!("{}", arc);  // ✓ Can send Arc
});

// Cell<T>: !Sync (interior mutability without synchronization)
let cell = Cell::new(42);
let cell_ref = &cell;
thread::spawn(move || {
    cell_ref.set(100);  // ❌ Compile error: &Cell is not Send
});
```

**Our SharedBlock Implementation**:
```rust
struct SharedBlock<T> {
    data: *mut T,
    pool: Arc<Mutex<TypedPool<T>>>,
    _marker: PhantomData<T>,
}

// SAFETY: We must manually implement Send/Sync
// because raw pointers are !Send and !Sync by default

unsafe impl<T: Send> Send for SharedBlock<T> {}
// Safe because:
// - *mut T is owned uniquely by this SharedBlock
// - Arc<Mutex<...>> is Send when T: Send
// - We never share *mut T across threads

unsafe impl<T: Send> Sync for SharedBlock<T> {}
// Safe because:
// - All access to *mut T requires &mut self
// - Arc<Mutex<...>> synchronizes pool access
```

---

### Deref and DerefMut: Smart Pointer Pattern

`Deref` and `DerefMut` enable automatic dereferencing, making smart pointers transparent.

**Without Deref**:
```rust
struct Block<T> {
    data: *mut T,
}

let block = Block { data: ... };
// To access data:
unsafe { (*block.data).method() }  // Ugly!
```

**With Deref**:
```rust
impl<T> Deref for Block<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.data }
    }
}

let block = Block { data: ... };
block.method();  // Automatic deref! Calls (*block).method()
```

**Deref Coercion**:
```rust
impl Deref for Block<T> {
    type Target = T;
    fn deref(&self) -> &T { ... }
}

fn takes_ref(x: &String) { ... }

let block: Block<String> = ...;
takes_ref(&block);  // Automatically coerces Block<String> to &String
// Compiler inserts: takes_ref(&*block)
```

**DerefMut for Mutability**:
```rust
impl<T> DerefMut for Block<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data }
    }
}

let mut block = Block { data: ... };
block.push_str("hello");  // Calls (*block).push_str("hello")
*block = new_value;       // Direct assignment through DerefMut
```

**Standard Library Examples**:
```rust
// Box<T> implements Deref
let boxed = Box::new(String::from("hello"));
boxed.len();  // Calls String::len through Deref

// String implements Deref<Target = str>
let s = String::from("hello");
let len: usize = s.len();  // Calls str::len through Deref

// Vec<T> implements Deref<Target = [T]>
let vec = vec![1, 2, 3];
let slice: &[i32] = &vec;  // Deref coercion
```

---

### Connection to This Project

In this project, you'll implement all these concepts:

1. **Milestone 1**: Raw memory pool with unsafe pointer operations
   - Learn pointer arithmetic, free lists, block management
   - Understand why unsafe code is necessary

2. **Milestone 2**: Type-safe RAII wrappers
   - Implement Drop for automatic cleanup
   - Use PhantomData for type safety
   - Implement Deref/DerefMut for ergonomics

3. **Milestone 3**: Thread-safe shared pools
   - Wrap with Arc<Mutex<>> for concurrency
   - Implement Send/Sync correctly
   - Understand why manual Send/Sync impls are needed for raw pointers

    

#### Milestone 1: Basic Memory Pool Structure
**Goal**: Create a memory pool that can allocate and deallocate fixed-size blocks.

**What to implement**:
- Define the memory pool structure with pre-allocated memory
- Track which blocks are free/used
- Implement basic allocation and deallocation


**Architecture**:
- **Struct**: `MemoryPool`   - Main allocator structure   
  - **field**: `memory: Vec<u8>`- The pre-allocated memory buffer
  - **field**: `block_size: usize` - Size of each allocatable block
  - **field**: `total_size: usize` - Total size of the pool
  - **field**: `free_list: Vec<usize>` - Indices of free blocks
**Functions**:
- `new()` - Initializes the pool with specified size 
- `allocate()` - Returns a pointer to a free block   
- `deallocate()` - Returns a block to the free list  
- `total_blocks()` - Returns number of blocks in pool
---



**Starter Code**:

```rust
/// A memory pool allocator that manages fixed-size blocks
pub struct MemoryPool {
    memory: Vec<u8>,
    block_size: usize,
    total_size: usize,
    free_list: Vec<usize>,
}

impl MemoryPool {
    /// Creates a new memory pool
    /// Role: Initialize pre-allocated memory and free list
    pub fn new(total_size: usize, block_size: usize) -> Self {
        todo!("Implement pool creation")
    }

    /// Allocates a block from the pool
    /// Role: Find and return a free block, update free list
    pub fn allocate(&mut self) -> Option<*mut u8> {
        todo!("Implement allocation logic")
    }

    /// Returns a block to the pool
    /// Role: Mark block as free and add to free list
    pub fn deallocate(&mut self, ptr: *mut u8) {
        todo!("Implement deallocation logic")
    }

    /// Returns total number of blocks
    /// Role: Calculate capacity of the pool
    pub fn total_blocks(&self) -> usize {
        todo!("Implement block counting")
    }
}
```

---
**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation() {
        let pool = MemoryPool::new(1024, 64);
        assert_eq!(pool.block_size, 64);
        assert_eq!(pool.total_blocks(), 16);
    }

    #[test]
    fn test_single_allocation() {
        let mut pool = MemoryPool::new(1024, 64);
        let ptr = pool.allocate();
        assert!(ptr.is_some());
    }

    #[test]
    fn test_allocation_exhaustion() {
        let mut pool = MemoryPool::new(256, 64);
        let p1 = pool.allocate();
        let p2 = pool.allocate();
        let p3 = pool.allocate();
        let p4 = pool.allocate();
        let p5 = pool.allocate(); // Should fail - only 4 blocks available
        assert!(p5.is_none());
    }

    #[test]
    fn test_deallocation_and_reuse() {
        let mut pool = MemoryPool::new(256, 64);
        let ptr1 = pool.allocate().unwrap();
        pool.deallocate(ptr1);
        let ptr2 = pool.allocate();
        assert!(ptr2.is_some());
        // Should reuse the deallocated block
    }
}
```

---**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation() {
        let pool = MemoryPool::new(1024, 64);
        assert_eq!(pool.block_size, 64);
        assert_eq!(pool.total_blocks(), 16);
    }

    #[test]
    fn test_single_allocation() {
        let mut pool = MemoryPool::new(1024, 64);
        let ptr = pool.allocate();
        assert!(ptr.is_some());
    }

    #[test]
    fn test_allocation_exhaustion() {
        let mut pool = MemoryPool::new(256, 64);
        let p1 = pool.allocate();
        let p2 = pool.allocate();
        let p3 = pool.allocate();
        let p4 = pool.allocate();
        let p5 = pool.allocate(); // Should fail - only 4 blocks available
        assert!(p5.is_none());
    }

    #[test]
    fn test_deallocation_and_reuse() {
        let mut pool = MemoryPool::new(256, 64);
        let ptr1 = pool.allocate().unwrap();
        pool.deallocate(ptr1);
        let ptr2 = pool.allocate();
        assert!(ptr2.is_some());
        // Should reuse the deallocated block
    }
}
```

---
#### Milestone 2: Safe Wrapper with Ownership Tracking
**Goal**: Create a safe API that prevents use-after-free and double-free bugs.

**Why the previous Milestone is not enough**: Raw pointers are unsafe and error-prone. We need ownership tracking to ensure memory safety.

**What's the improvement**: Introduce typed blocks with RAII (Resource Acquisition Is Initialization). When a `Block<T>` is dropped, memory is automatically returned to the pool.

**Key concepts**:
- **Struct**: `TypedPool<T>` - Type-safe memory pool
  - **Field**: `pool: MemoryPool` - Underlying raw pool
  - **Field**: `_marker: PhantomData<T>` - Zero-size type marker
- **Struct**: `Block<T>` - RAII wrapper for allocated memory
  - **Field**: `data: *mut T` - Pointer to the allocated object
  - **Field**: `pool: *mut TypedPool<T>` - Pointer back to owning pool
- **Trait**: `Drop` - Ensures automatic cleanup on drop

- **Function**: `TypedPool::new(capacity: usize) -> Self` - Creates typed pool
- **Function**: `allocate(&mut self, value: T) -> Option<Block<T>>` - Returns owned block
- **Function**: `Drop::drop(&mut self)` - Auto-returns block to pool



**Starter Code**:

```rust
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A typed memory pool for type T
pub struct TypedPool<T> {
    pool: MemoryPool,
    _marker: PhantomData<T>,
}

pub struct Block<'a, T> {
    data: *mut T,
    pool: &'a mut TypedPool<T>,
}

impl<T> TypedPool<T> {
    /// Creates a typed pool for type T
    /// Role: Initialize pool with size based on sizeof(T)
    pub fn new(capacity: usize) -> Self {
        todo!("Create pool with appropriate block size for T")
    }

    /// Allocates and initializes a block
    /// Role: Get memory from pool and write value into it
    pub fn allocate(&mut self, value: T) -> Option<Block<T>> {
        todo!("Allocate block and initialize with value")
    }

    /// Returns number of available blocks
    /// Role: Query pool state
    pub fn available(&self) -> usize {
        todo!("Count free blocks")
    }
}

impl<T> Deref for Block<'_, T> {
    type Target = T;

    /// Allows treating Block<T> as &T
    /// Role: Enable transparent access to inner value
    fn deref(&self) -> &Self::Target {
        todo!("Return reference to stored value")
    }
}

impl<T> DerefMut for Block<'_, T> {
    /// Allows treating Block<T> as &mut T
    /// Role: Enable mutable access to inner value
    fn deref_mut(&mut self) -> &mut Self::Target {
        todo!("Return mutable reference to stored value")
    }
}

impl<T> Drop for Block<'_, T> {
    /// Automatically returns block to pool when dropped
    /// Role: RAII cleanup - ensures no memory leaks
    fn drop(&mut self) {
        todo!("Drop the value and return memory to pool")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_allocation() {
        let mut pool = TypedPool::<u64>::new(10);
        let block = pool.allocate(42);
        assert!(block.is_some());
        assert_eq!(*block.unwrap(), 42);
    }

    #[test]
    fn test_automatic_deallocation() {
        let mut pool = TypedPool::<u64>::new(2);
        {
            let _b1 = pool.allocate(10).unwrap();
            let _b2 = pool.allocate(20).unwrap();
            // Both blocks allocated
            assert_eq!(pool.available(), 0);
        } // Both blocks dropped here

        assert_eq!(pool.available(), 2);
    }

    #[test]
    fn test_block_access() {
        let mut pool = TypedPool::<String>::new(5);
        let mut block = pool.allocate(String::from("hello")).unwrap();
        block.push_str(" world");
        assert_eq!(&*block, "hello world");
    }

    #[test]
    fn test_prevents_double_free() {
        let mut pool = TypedPool::<i32>::new(5);
        let block = pool.allocate(100).unwrap();
        drop(block);
        // Block is already freed - can't free again
        // Rust's type system prevents this at compile time
    }
}
```

---

#### Milestone 3: Thread-Safe Pool with Arc and Mutex
**Goal**: Make the memory pool usable across threads.

**Why the previous Milestone is not enough**: The pool isn't thread-safe - concurrent allocations would cause data races.

**What's the improvement**: Wrap pool in `Arc<Mutex<>>` to enable safe sharing across threads. Blocks now hold an `Arc` reference to keep the pool alive.

**Architecture**:
**Structs**:
- `SharedPool<T>`: Clone-able, thread-safe pool
  - **field**: `inner: Arc<Mutex<TypedPool<T>>> `- Shared, locked pool           
- `SharedBlock<T>`: Block that holds Arc reference
  - **field**: `data: *mut T` - Pointer to data
  - *field**: `pool: Arc<Mutex<TypedPool<T>>>` - Keeps pool alive
  - *field**: `_marker: PhantomData<T> `

**Functions**:
- `SharedPool::new(capacity)` - Creates shareable pool
- `clone()` - Creates another reference to same pool
- `allocate(value)` - Thread-safe allocation
- `available()` - Returns free block count
---

---

**Starter Code**:

```rust
use std::sync::{Arc, Mutex};

/// Thread-safe shared memory pool
pub struct SharedPool<T> {
    inner: Arc<Mutex<TypedPool<T>>>,
}

pub struct SharedBlock<T> {
    data: *mut T,
    pool: Arc<Mutex<TypedPool<T>>>,
    _marker: PhantomData<T>,
}

// Safety: SharedBlock can be sent between threads if T can
unsafe impl<T: Send> Send for SharedBlock<T> {}

impl<T> SharedPool<T> {
    /// Creates a new thread-safe pool
    /// Role: Wrap TypedPool in Arc<Mutex<>>
    pub fn new(capacity: usize) -> Self {
        todo!("Create shared pool")
    }

    /// Allocates a block from the pool
    /// Role: Lock pool and allocate safely
    pub fn allocate(&self, value: T) -> Option<SharedBlock<T>> {
        todo!("Lock, allocate, wrap in SharedBlock")
    }

    /// Returns number of available blocks
    /// Role: Query pool state thread-safely
    pub fn available(&self) -> usize {
        todo!("Lock and check availability")
    }
}

impl<T> Clone for SharedPool<T> {
    /// Clones the Arc reference to the pool
    /// Role: Enable sharing across threads
    fn clone(&self) -> Self {
        todo!("Clone the Arc")
    }
}

impl<T> Deref for SharedBlock<T> {
    type Target = T;

    /// Role: Transparent access to value
    fn deref(&self) -> &Self::Target {
        todo!("Safe dereference")
    }
}

impl<T> DerefMut for SharedBlock<T> {
    /// Role: Mutable access to value
    fn deref_mut(&mut self) -> &mut Self::Target {
        todo!("Safe mutable dereference")
    }
}

impl<T> Drop for SharedBlock<T> {
    /// Returns block to pool when dropped
    /// Role: Thread-safe cleanup
    fn drop(&mut self) {
        todo!("Lock pool and deallocate")
    }
}
```

---


**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_shared_pool_creation() {
        let pool = SharedPool::<i32>::new(10);
        let block = pool.allocate(42);
        assert!(block.is_some());
    }

    #[test]
    fn test_concurrent_allocation() {
        let pool = SharedPool::<u64>::new(100);
        let mut handles = vec![];

        for i in 0..10 {
            let pool_clone = pool.clone();
            let handle = thread::spawn(move || {
                let mut blocks = vec![];
                for j in 0..10 {
                    if let Some(block) = pool_clone.allocate(i * 10 + j) {
                        blocks.push(block);
                    }
                }
                blocks
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // All allocations should succeed
        assert_eq!(pool.available(), 0);
    }

    #[test]
    fn test_pool_survives_block_thread() {
        let pool = SharedPool::<String>::new(5);

        let block = pool.allocate(String::from("thread-safe")).unwrap();
        let pool_clone = pool.clone();

        let handle = thread::spawn(move || {
            let _block2 = pool_clone.allocate(String::from("another")).unwrap();
            // pool_clone dropped here but pool still lives
        });

        handle.join().unwrap();

        // Original block still valid
        assert_eq!(&*block, "thread-safe");
    }
}
```


### Testing Strategies

1. **Correctness Tests**:
    - Verify allocation/deallocation work correctly
    - Test edge cases (empty pool, full pool)
    - Ensure no use-after-free or double-free

2. **Concurrency Tests**:
    - Spawn multiple threads allocating simultaneously
    - Use tools like `loom` for systematic concurrency testing
    - Verify no data races with `cargo test --features=sanitizer`

3. **Performance Tests**:
    - Benchmark pool allocator vs system allocator
    - Measure allocation/deallocation throughput
    - Test with various block sizes

4. **Memory Tests**:
    - Use Valgrind/Address Sanitizer to detect leaks
    - Verify all memory is freed on pool drop
    - Test alignment requirements

---

### Complete Working Example

Here's a complete, production-ready memory pool implementation:

```rust
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::ptr;

//==============================================================================
// Part 1: Raw Memory Pool
//==============================================================================

/// A memory pool allocator that manages fixed-size blocks
pub struct MemoryPool {
    memory: Vec<u8>,
    block_size: usize,
    total_size: usize,
    free_list: Vec<usize>,
}

impl MemoryPool {
    /// Creates a new memory pool with the specified total size and block size
    pub fn new(total_size: usize, block_size: usize) -> Self {
        assert!(block_size > 0, "Block size must be positive");
        assert!(total_size >= block_size, "Total size must be >= block size");

        let num_blocks = total_size / block_size;
        let actual_size = num_blocks * block_size;

        // Pre-allocate all memory
        let memory = vec![0u8; actual_size];

        // Initialize free list with all block indices
        let free_list = (0..num_blocks).collect();

        MemoryPool {
            memory,
            block_size,
            total_size: actual_size,
            free_list,
        }
    }

    /// Allocates a block from the pool, returning a pointer to it
    pub fn allocate(&mut self) -> Option<*mut u8> {
        self.free_list.pop().map(|index| {
            let offset = index * self.block_size;
            unsafe { self.memory.as_mut_ptr().add(offset) }
        })
    }

    /// Deallocates a block, returning it to the pool
    pub fn deallocate(&mut self, ptr: *mut u8) {
        let offset = unsafe {
            ptr.offset_from(self.memory.as_ptr())
        } as usize;

        assert!(offset % self.block_size == 0, "Invalid pointer alignment");
        let index = offset / self.block_size;
        assert!(index < self.total_blocks(), "Pointer out of bounds");

        self.free_list.push(index);
    }

    /// Returns the total number of blocks in the pool
    pub fn total_blocks(&self) -> usize {
        self.total_size / self.block_size
    }

    /// Returns the number of available (free) blocks
    pub fn available(&self) -> usize {
        self.free_list.len()
    }
}

//==============================================================================
// Part 2: Type-Safe Pool with RAII
//==============================================================================

/// A typed memory pool for allocating objects of type T
pub struct TypedPool<T> {
    pool: MemoryPool,
    _marker: PhantomData<T>,
}

/// An RAII wrapper for an allocated block
/// Automatically returns memory to pool when dropped
pub struct Block<'a, T> {
    data: *mut T,
    pool: &'a mut TypedPool<T>,
}

impl<T> TypedPool<T> {
    /// Creates a new typed pool with capacity for `capacity` objects
    pub fn new(capacity: usize) -> Self {
        let block_size = std::mem::size_of::<T>().max(1);
        let total_size = capacity * block_size;

        TypedPool {
            pool: MemoryPool::new(total_size, block_size),
            _marker: PhantomData,
        }
    }

    /// Allocates a block and initializes it with `value`
    pub fn allocate(&mut self, value: T) -> Option<Block<T>> {
        self.pool.allocate().map(|ptr| {
            let typed_ptr = ptr as *mut T;
            // SAFETY: We just allocated this memory and it's properly aligned
            unsafe {
                ptr::write(typed_ptr, value);
            }

            Block {
                data: typed_ptr,
                pool: self,
            }
        })
    }

    /// Returns the number of available blocks
    pub fn available(&self) -> usize {
        self.pool.available()
    }

    /// Internal function to deallocate a block
    fn deallocate_raw(&mut self, ptr: *mut T) {
        self.pool.deallocate(ptr as *mut u8);
    }
}

impl<T> Deref for Block<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: data is valid for the lifetime of Block
        unsafe { &*self.data }
    }
}

impl<T> DerefMut for Block<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: data is valid and uniquely owned by Block
        unsafe { &mut *self.data }
    }
}

impl<T> Drop for Block<'_, T> {
    fn drop(&mut self) {
        // SAFETY: data was initialized in allocate()
        unsafe {
            ptr::drop_in_place(self.data);
        }
        self.pool.deallocate_raw(self.data);
    }
}

//==============================================================================
// Part 3: Thread-Safe Shared Pool
//==============================================================================

/// A thread-safe, reference-counted memory pool
#[derive(Clone)]
pub struct SharedPool<T> {
    inner: Arc<Mutex<TypedPool<T>>>,
}

/// A block allocated from a shared pool
pub struct SharedBlock<T> {
    data: *mut T,
    pool: Arc<Mutex<TypedPool<T>>>,
    _marker: PhantomData<T>,
}

// SAFETY: SharedBlock can be sent between threads if T can
unsafe impl<T: Send> Send for SharedBlock<T> {}
unsafe impl<T: Send> Sync for SharedBlock<T> {}

impl<T> SharedPool<T> {
    /// Creates a new thread-safe shared pool
    pub fn new(capacity: usize) -> Self {
        SharedPool {
            inner: Arc::new(Mutex::new(TypedPool::new(capacity))),
        }
    }

    /// Allocates a block from the pool
    pub fn allocate(&self, value: T) -> Option<SharedBlock<T>> {
        let mut pool = self.inner.lock().unwrap();

        pool.pool.allocate().map(|ptr| {
            let typed_ptr = ptr as *mut T;
            // SAFETY: We just allocated this memory
            unsafe {
                ptr::write(typed_ptr, value);
            }

            SharedBlock {
                data: typed_ptr,
                pool: Arc::clone(&self.inner),
                _marker: PhantomData,
            }
        })
    }

    /// Returns the number of available blocks
    pub fn available(&self) -> usize {
        self.inner.lock().unwrap().available()
    }
}

impl<T> Deref for SharedBlock<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: data is valid for the lifetime of SharedBlock
        unsafe { &*self.data }
    }
}

impl<T> DerefMut for SharedBlock<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: We have exclusive access via &mut self
        unsafe { &mut *self.data }
    }
}

impl<T> Drop for SharedBlock<T> {
    fn drop(&mut self) {
        // SAFETY: data was initialized in allocate()
        unsafe {
            ptr::drop_in_place(self.data);
        }

        let mut pool = self.pool.lock().unwrap();
        pool.pool.deallocate(self.data as *mut u8);
    }
}

//==============================================================================
// Example Usage and Tests
//==============================================================================

fn main() {
    println!("=== Memory Pool Examples ===\n");

    // Example 1: Basic pool usage
    println!("Example 1: Basic Pool");
    {
        let mut pool = TypedPool::<i32>::new(5);
        let mut block1 = pool.allocate(42).unwrap();
        let block2 = pool.allocate(100).unwrap();

        println!("Block1: {}", *block1);
        println!("Block2: {}", *block2);

        *block1 = 99;
        println!("Block1 modified: {}", *block1);
        println!("Available blocks: {}\n", pool.available());
    }

    // Example 2: Automatic cleanup
    println!("Example 2: RAII and Automatic Cleanup");
    {
        let mut pool = TypedPool::<String>::new(3);
        println!("Initial available: {}", pool.available());

        {
            let _b1 = pool.allocate(String::from("hello")).unwrap();
            let _b2 = pool.allocate(String::from("world")).unwrap();
            println!("After 2 allocations: {}", pool.available());
        } // b1 and b2 dropped here

        println!("After blocks dropped: {}\n", pool.available());
    }

    // Example 3: Thread-safe pool
    println!("Example 3: Thread-Safe Pool");
    {
        use std::thread;

        let pool = SharedPool::<u64>::new(20);
        let mut handles = vec![];

        for i in 0..4 {
            let pool_clone = pool.clone();
            let handle = thread::spawn(move || {
                let mut local_blocks = vec![];
                for j in 0..5 {
                    if let Some(block) = pool_clone.allocate(i * 5 + j) {
                        local_blocks.push(block);
                    }
                }
                println!("Thread {} allocated {} blocks", i, local_blocks.len());
                local_blocks
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        println!("Final available: {}\n", pool.available());
    }

    // Example 4: Complex type with Drop
    println!("Example 4: Complex Types");
    {
        #[derive(Debug)]
        struct Resource {
            id: usize,
            data: Vec<i32>,
        }

        impl Drop for Resource {
            fn drop(&mut self) {
                println!("Resource {} dropped", self.id);
            }
        }

        let mut pool = TypedPool::<Resource>::new(3);

        {
            let r1 = pool.allocate(Resource {
                id: 1,
                data: vec![1, 2, 3],
            }).unwrap();

            let r2 = pool.allocate(Resource {
                id: 2,
                data: vec![4, 5, 6],
            }).unwrap();

            println!("Resource 1: {:?}", *r1);
            println!("Resource 2: {:?}", *r2);
        } // Resources properly dropped

        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_basic_pool() {
        let mut pool = MemoryPool::new(1024, 64);
        assert_eq!(pool.total_blocks(), 16);
        assert_eq!(pool.available(), 16);

        let ptr = pool.allocate().unwrap();
        assert_eq!(pool.available(), 15);

        pool.deallocate(ptr);
        assert_eq!(pool.available(), 16);
    }

    #[test]
    fn test_typed_pool() {
        let mut pool = TypedPool::<u64>::new(10);
        let mut block = pool.allocate(42).unwrap();
        assert_eq!(*block, 42);

        *block = 100;
        assert_eq!(*block, 100);
    }

    #[test]
    fn test_automatic_cleanup() {
        let mut pool = TypedPool::<String>::new(5);
        assert_eq!(pool.available(), 5);

        {
            let _b1 = pool.allocate(String::from("test")).unwrap();
            let _b2 = pool.allocate(String::from("test2")).unwrap();
            assert_eq!(pool.available(), 3);
        }

        assert_eq!(pool.available(), 5);
    }

    #[test]
    fn test_shared_pool_threading() {
        let pool = SharedPool::<i32>::new(100);
        let mut handles = vec![];

        for i in 0..10 {
            let pool_clone = pool.clone();
            let handle = thread::spawn(move || {
                let mut blocks = vec![];
                for j in 0..10 {
                    blocks.push(pool_clone.allocate(i * 10 + j).unwrap());
                }
                blocks
            });
            handles.push(handle);
        }

        for handle in handles {
            let blocks = handle.join().unwrap();
            assert_eq!(blocks.len(), 10);
        }

        assert_eq!(pool.available(), 0);
    }

    #[test]
    fn test_drop_behavior() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let drop_count = Arc::new(AtomicUsize::new(0));

        struct DropCounter {
            count: Arc<AtomicUsize>,
        }

        impl Drop for DropCounter {
            fn drop(&mut self) {
                self.count.fetch_add(1, Ordering::SeqCst);
            }
        }

        {
            let mut pool = TypedPool::<DropCounter>::new(5);
            let _b1 = pool.allocate(DropCounter { count: drop_count.clone() }).unwrap();
            let _b2 = pool.allocate(DropCounter { count: drop_count.clone() }).unwrap();
        }

        assert_eq!(drop_count.load(Ordering::SeqCst), 2);
    }
}
```

This complete example demonstrates:
- **Part 1**: Raw memory pool with block management
- **Part 2**: Type-safe RAII wrappers preventing memory errors
- **Part 3**: Thread-safe shared pools with Arc/Mutex
- **Examples**: Real-world usage patterns
- **Tests**: Comprehensive validation of correctness and safety

The implementation progresses from unsafe low-level memory management to safe, ergonomic APIs that prevent common bugs through Rust's type system and ownership rules.

---
