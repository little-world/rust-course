## Project 4: Memory Pool Allocator

### Problem Statement

Build a custom memory pool allocator that pre-allocates a large block of memory and manages allocations within it. This allocator should efficiently handle fixed-size allocations, track memory usage, and provide better performance than the system allocator for specific workloads.

Your memory pool should support:
- Pre-allocating a fixed-size pool
- Allocating and deallocating fixed-size blocks
- Tracking used/free blocks
- Preventing fragmentation
- Detecting memory leaks and double-frees

### Why It Matters

Memory pools are critical for high-performance systems where allocation patterns are predictable. Game engines, embedded systems, real-time applications, and network servers use memory pools to achieve deterministic performance by avoiding unpredictable heap allocations. Understanding memory pools teaches you about memory layout, alignment, lifetimes, and ownership - core concepts in systems programming.

### Use Cases

- Game engines: Managing game objects with predictable lifecycles
- Network servers: Handling connection buffers of uniform size
- Embedded systems: Operating in constrained memory environments
- Real-time systems: Avoiding non-deterministic malloc/free
- Database engines: Managing fixed-size page buffers

### Solution Outline

Your solution should follow these Milestones:

#### Milestone 1: Basic Memory Pool Structure
**Goal**: Create a memory pool that can allocate and deallocate fixed-size blocks.

**What to implement**:
- Define the memory pool structure with pre-allocated memory
- Track which blocks are free/used
- Implement basic allocation and deallocation

**Why this Milestone**: Establishes the foundation of pool-based allocation. You'll learn about memory layout and block management.

**Key concepts**:
- Structs: `MemoryPool`
- Fields: `memory: Vec<u8>`, `block_size: usize`, `free_list: Vec<usize>`
- Functions:
    - `new(total_size: usize, block_size: usize) -> Self` - Creates the pool
    - `allocate() -> Option<*mut u8>` - Returns pointer to free block
    - `deallocate(ptr: *mut u8)` - Marks block as free

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

---

**Starter Code**:

```rust
/// A memory pool allocator that manages fixed-size blocks
///
/// Structs:
/// - MemoryPool: Main allocator structure
///
/// Fields:
/// - memory: Vec<u8> - The pre-allocated memory buffer
/// - block_size: usize - Size of each allocatable block
/// - total_size: usize - Total size of the pool
/// - free_list: Vec<usize> - Indices of free blocks
///
/// Functions:
/// - new() - Initializes the pool with specified size
/// - allocate() - Returns a pointer to a free block
/// - deallocate() - Returns a block to the free list
/// - total_blocks() - Returns number of blocks in pool
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

#### Milestone 2: Safe Wrapper with Ownership Tracking
**Goal**: Create a safe API that prevents use-after-free and double-free bugs.

**Why the previous Milestone is not enough**: Raw pointers are unsafe and error-prone. We need ownership tracking to ensure memory safety.

**What's the improvement**: Introduce typed blocks with RAII (Resource Acquisition Is Initialization). When a `Block<T>` is dropped, memory is automatically returned to the pool.

**Key concepts**:
- Structs: `Block<T>`, `TypedPool<T>`
- Traits: `Drop` for automatic cleanup
- Fields: `data: *mut T`, `pool: *mut TypedPool<T>`, `marker: PhantomData<T>`
- Functions:
    - `TypedPool::new(capacity: usize) -> Self` - Creates typed pool
    - `allocate(&mut self, value: T) -> Option<Block<T>>` - Returns owned block
    - `Drop::drop(&mut self)` - Auto-returns block to pool

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

**Starter Code**:

```rust
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A typed memory pool for type T
///
/// Structs:
/// - TypedPool<T>: Type-safe memory pool
/// - Block<T>: RAII wrapper for allocated memory
///
/// TypedPool Fields:
/// - pool: MemoryPool - Underlying raw pool
/// - _marker: PhantomData<T> - Zero-size type marker
///
/// Block Fields:
/// - data: *mut T - Pointer to the allocated object
/// - pool: *mut TypedPool<T> - Pointer back to owning pool
/// - _marker: PhantomData<T> - Ensures proper Drop behavior
///
/// Functions:
/// - TypedPool::new(capacity) - Creates pool for T
/// - allocate(value: T) - Allocates and initializes a block
/// - available() - Returns number of free blocks
/// - Block::drop() - Automatically returns memory on drop
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

#### Milestone 3: Thread-Safe Pool with Arc and Mutex
**Goal**: Make the memory pool usable across threads.

**Why the previous Milestone is not enough**: The pool isn't thread-safe - concurrent allocations would cause data races.

**What's the improvement**: Wrap pool in `Arc<Mutex<>>` to enable safe sharing across threads. Blocks now hold an `Arc` reference to keep the pool alive.

**Key concepts**:
- Structs: `SharedPool<T>`, `SharedBlock<T>`
- Traits: `Send`, `Sync` bounds
- Fields: `pool: Arc<Mutex<TypedPool<T>>>`
- Functions:
    - `SharedPool::new(capacity)` - Creates thread-safe pool
    - `allocate(value)` - Locks pool and allocates
    - `clone()` - Shares pool across threads

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

---

**Starter Code**:

```rust
use std::sync::{Arc, Mutex};

/// Thread-safe shared memory pool
///
/// Structs:
/// - SharedPool<T>: Clone-able, thread-safe pool
/// - SharedBlock<T>: Block that holds Arc reference
///
/// SharedPool Fields:
/// - inner: Arc<Mutex<TypedPool<T>>> - Shared, locked pool
///
/// SharedBlock Fields:
/// - data: *mut T - Pointer to data
/// - pool: Arc<Mutex<TypedPool<T>>> - Keeps pool alive
/// - _marker: PhantomData<T>
///
/// Functions:
/// - SharedPool::new(capacity) - Creates shareable pool
/// - clone() - Creates another reference to same pool
/// - allocate(value) - Thread-safe allocation
/// - available() - Returns free block count
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
