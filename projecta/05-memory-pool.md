# Memory Pool Allocator

### Problem Statement

Build a custom memory pool allocator that pre-allocates a large block of memory and manages allocations within it. This allocator should efficiently handle fixed-size allocations, track memory usage, and provide better performance than the system allocator for specific workloads.

Your memory pool should support:
- Pre-allocating a fixed-size pool
- Allocating and deallocating fixed-size blocks
- Tracking used/free blocks
- Preventing fragmentation
- Detecting memory leaks and double-frees


## How System Allocators Work

Before we dive into memory pools, it's essential to understand how general-purpose system allocators work and why they struggle with certain workloads. This knowledge will help you appreciate when and why specialized allocators like memory pools are superior.

### System Allocator Overview

A **system allocator** (like `malloc`/`free`, `jemalloc`, `tcmalloc`) is a general-purpose memory management system that handles arbitrary-sized allocations. When your program calls `malloc(size)`, the allocator must:

1. Find a suitable block of free memory
2. Mark it as used
3. Return a pointer to the caller
4. Track metadata for eventual deallocation

**The Core Challenge**: Must handle **any size**, **any pattern**, **any timing** efficiently.

```
Typical program workload:
malloc(8)     → need tiny block
malloc(1024)  → need medium block
malloc(64)    → need small block
malloc(8192)  → need large block
free(small)   → creates hole
malloc(32)    → reuse hole or find new space?
```

This unpredictability forces system allocators to use complex strategies that trade performance for flexibility.

---

### Internal Data Structures

System allocators maintain several data structures to track free and allocated memory:

#### 1. Free Lists (Linked Lists of Available Blocks)

**Concept**: Chain together all free memory blocks using pointers stored **within the free blocks themselves**.

```
Heap memory with free list:

Address    Data
0x1000   ┌─────────────────┐
         │ [Used Block]    │  Allocated to user
0x1040   ├─────────────────┤
         │ Size: 128       │  Metadata
         │ Next: 0x20C0 ───┼──┐  Pointer to next free block
         │ [Free Space]    │  │  (stored in the free space!)
0x10C0   ├─────────────────┤  │
         │ [Used Block]    │  │
0x2000   ├─────────────────┤  │
         │ [Used Block]    │  │
0x20C0   ├─────────────────┤◄─┘
         │ Size: 256       │
         │ Next: NULL      │
         │ [Free Space]    │
         └─────────────────┘

Free list: 0x1040 → 0x20C0 → NULL
```

**Key Insight**: No extra memory needed for the free list—the list lives in the free blocks themselves!

**Search Strategies**:

| Strategy | How It Works | Time | Fragmentation |
|----------|-------------|------|---------------|
| **First Fit** | Use first block large enough | O(n) | Medium - leaves small holes at front |
| **Best Fit** | Use smallest block that fits | O(n) | High - leaves tiny unusable holes |
| **Worst Fit** | Use largest available block | O(n) | Low - keeps large blocks available |
| **Next Fit** | Resume search from last allocation | O(n) | Medium - distributes holes evenly |

**Example: First Fit Search**
```
Request: malloc(64)
Free list: [32] → [128] → [48] → [256]

Scan:
- 32 bytes? Too small, skip
- 128 bytes? Large enough! Use it
  - Return pointer to start of block
  - If 128 - 64 = 64 remaining, split into new free block
  - Update free list
```

#### 2. Size Classes / Bins (Segregated Free Lists)

**Problem with Single Free List**: Searching for the right size is slow O(n).

**Solution**: Maintain **multiple free lists**, one for each size range.

```
Bins (size classes):
┌──────────┬─────────────────┐
│ Bin 0    │ 8-16 bytes      │ → [Free 16] → [Free 8] → NULL
├──────────┼─────────────────┤
│ Bin 1    │ 17-32 bytes     │ → [Free 32] → [Free 24] → NULL
├──────────┼─────────────────┤
│ Bin 2    │ 33-64 bytes     │ → [Free 64] → [Free 48] → NULL
├──────────┼─────────────────┤
│ Bin 3    │ 65-128 bytes    │ → NULL (all allocated)
├──────────┼─────────────────┤
│ Bin 4    │ 129-256 bytes   │ → [Free 256] → NULL
├──────────┼─────────────────┤
│ ...      │ ...             │
└──────────┴─────────────────┘

malloc(50):
1. Calculate bin: 50 bytes → Bin 2 (33-64 bytes)
2. Check Bin 2: found [Free 64]
3. Return immediately - O(1)!
```

**Benefits**:
- **Fast allocation**: O(1) lookup for common sizes
- **Less fragmentation**: Similar-sized objects grouped together
- **Better cache locality**: Objects from same bin are nearby

**Real Allocator Bin Layouts**:

| Allocator | Small Bins | Large Bins |
|-----------|-----------|------------|
| **jemalloc** | 8, 16, 32, 48, 64, 80, 96, 112, 128... (fine-grained) | Powers of 2 |
| **tcmalloc** | 8, 16, 32, 48, 64, 80, 96... (8-byte increments) | Page-aligned |
| **ptmalloc2** | 16, 24, 32, 40, 48, 56, 64... (8-byte increments) | Powers of 2 |

#### 3. Block Metadata (Headers and Footers)

Every allocated block carries metadata for management:

```
Allocated Block Structure:

┌─────────────────────────────┐
│  HEADER                     │
│  ┌────────────────────────┐ │
│  │ Size: 128 bytes        │ │ ← Block size (includes header)
│  │ Flags: [IN_USE]        │ │ ← Status bits
│  │ Prev Size: 64          │ │ ← Previous block size (for coalescing)
│  └────────────────────────┘ │
├─────────────────────────────┤
│  USER DATA                  │ ← Pointer returned to user
│  (120 bytes usable)         │
│                             │
└─────────────────────────────┘
│  FOOTER (optional)          │
│  Size: 128 (duplicate)      │ ← Enables backward traversal
└─────────────────────────────┘

Total overhead: 8-16 bytes per allocation
```

**Metadata Uses**:
- **Size**: Know how much to free
- **Status flags**: In use, previous block free, etc.
- **Boundary tags**: Find adjacent blocks for coalescing

**Optimization: Size Class Pools**

Small allocations (≤ 256 bytes) often use **slab allocation** with no per-object metadata:

```
Slab for 64-byte objects:
┌───────┬───────┬───────┬───────┬───────┐
│ Obj 1 │ Obj 2 │ Obj 3 │ Obj 4 │ Obj 5 │
│ 64B   │ 64B   │ 64B   │ 64B   │ 64B   │
└───────┴───────┴───────┴───────┴───────┘

Bitmap: [1][1][0][1][0]  ← 1 = in use, 0 = free
         ↑  ↑     ↑
       Used Used  Used

No per-object headers! Just bitmap overhead.
```

**Savings**: 8-16 bytes per allocation for small objects = **10-20% memory savings**.

---

### Fragmentation: The Allocator's Nemesis

**Fragmentation** is wasted memory that's technically free but unusable. It's the primary problem system allocators try to minimize.

#### Types of Fragmentation

**1. External Fragmentation**

Free memory exists but is scattered in pieces too small to satisfy requests.

```
Memory state after many allocations/deallocations:

[Used 32][Free 16][Used 64][Free 8][Used 128][Free 32][Used 16][Free 4]

Request malloc(64):
❌ FAIL: Total free = 16 + 8 + 32 + 4 = 60 bytes
✓ But no single contiguous block ≥ 64 bytes!

Fragmentation ratio: 60 bytes free but unusable = wasted
```

**Real-World Impact**:
- Long-running servers: 20-40% of heap can become fragmented
- Embedded systems: May fail allocations despite having enough total memory
- Can trigger expensive OS memory operations (brk, mmap)

**2. Internal Fragmentation**

Allocated more than needed due to allocator constraints.

```
Request: malloc(50)
Allocator rounds up to size class: 64 bytes
Return: 64-byte block

Used:     50 bytes
Wasted:   14 bytes (internal fragmentation)
Overhead: 28% waste!

With many small allocations:
10,000 × 50-byte requests
Allocated: 10,000 × 64 = 640 KB
Needed:    10,000 × 50 = 500 KB
Waste:     140 KB (28%)
```

---

### Low Fragmentation Strategies

Modern allocators use sophisticated techniques to minimize fragmentation. Let's examine the most important ones.

#### 1. Coalescing (Merging Adjacent Free Blocks)

**Problem**: After many frees, free list becomes littered with tiny adjacent blocks.

```
Before coalescing:
[Used][Free 16][Free 32][Used][Free 8][Free 24][Used]
       ↑        ↑              ↑        ↑
   Can't satisfy malloc(64)!

After coalescing:
[Used][Free 48][Used][Free 32][Used]
       ↑              ↑
   Now malloc(64) fails, but malloc(48) or malloc(32) works
```

**Implementation: Boundary Tags**

Use metadata to find adjacent blocks:

```rust
fn coalesce_free_block(ptr: *mut u8) {
    let mut block = Block::from_ptr(ptr);

    // Check previous block
    if block.prev_free() {
        let prev = block.prev_block();
        // Remove prev from free list
        // Merge: block = [prev][block]
        block.size += prev.size;
    }

    // Check next block
    let next = block.next_block();
    if next.is_free() {
        // Remove next from free list
        // Merge: block = [block][next]
        block.size += next.size;
    }

    // Add coalesced block to free list
    add_to_free_list(block);
}
```

**Cost**: O(1) per free operation (just check neighbors)

**Benefit**: Maintains larger contiguous blocks, reduces external fragmentation by 30-50%

#### 2. Splitting (Breaking Large Blocks)

**Problem**: Allocating small object from large block wastes space.

```
Free block: 1024 bytes
Request: malloc(64)

Without splitting:
[Used 64 + wasted 960] ← 960 bytes internal fragmentation!

With splitting:
[Used 64][Free 960]
         └── Return to free list
```

**Implementation**:

```rust
fn allocate_with_split(block: FreeBlock, requested_size: usize) -> *mut u8 {
    let total_size = block.size;
    let min_split_size = 32; // Don't split if remainder too small

    if total_size >= requested_size + min_split_size {
        // Split the block
        let remainder_size = total_size - requested_size;

        // First part: allocate
        let allocated = block.ptr;
        set_block_size(allocated, requested_size);
        set_used(allocated);

        // Second part: free
        let remainder = allocated.add(requested_size);
        set_block_size(remainder, remainder_size);
        set_free(remainder);
        add_to_free_list(remainder);

        return allocated;
    } else {
        // Too small to split, use whole block
        set_used(block.ptr);
        return block.ptr;
    }
}
```

**Threshold**: Only split if remainder ≥ minimum useful size (16-32 bytes)
- Prevents creating useless tiny blocks
- Balances internal vs external fragmentation

#### 3. Size Classes with Power-of-Two Rounding

**Strategy**: Round allocations to power-of-two or specific size classes.

**Benefits**:
- **Fast bin lookup**: `bin = log2(size)` or bit manipulation
- **Predictable splitting**: 512-byte block splits perfectly into 2×256, 4×128, etc.
- **Better reuse**: More likely to find exact size match

**Example Size Classes**:

```
Tiny:    8, 16, 24, 32, 40, 48, 56, 64  (8-byte increments)
Small:   64, 80, 96, 112, 128, 160, 192, 224, 256  (16-byte increments)
Medium:  256, 320, 384, 448, 512, 640, 768, 896, 1024  (64-byte increments)
Large:   1K, 2K, 4K, 8K, 16K, 32K, 64K...  (powers of 2)
Huge:    Direct mmap, page-aligned
```

**Internal Fragmentation Trade-off**:

```
Request: 65 bytes
Allocated: 80 bytes (next size class)
Waste: 15 bytes (23%)

But:
✓ Fast O(1) allocation (direct bin lookup)
✓ Reduces external fragmentation (standard sizes)
✓ Better cache utilization (aligned sizes)

Typical waste: 10-15% internal fragmentation
Savings from reduced external fragmentation: 20-40%
Net win: 5-30% less total waste
```

#### 4. Segregated Free Lists (Per-Size Bins)

**Advanced Strategy**: Separate free lists for each size class.

```
Allocation Request: 48 bytes

Step 1: Calculate bin index
bin = size_to_bin(48)  // bin 2 (33-64 bytes)

Step 2: Check exact bin
if bins[2] not empty:
    return bins[2].pop()  // O(1)!

Step 3: Check larger bins
for i in 3..MAX_BINS:
    if bins[i] not empty:
        block = bins[i].pop()
        split_and_return(block, 48)
        return block

Step 4: Request more memory from OS
return expand_heap(48)
```

**Fast Path**: O(1) when exact size available (90%+ of allocations in practice)

**Benefits**:
- Eliminates search time for common sizes
- Objects of similar size grouped together (better cache locality)
- Reduces fragmentation (similar-sized objects live/die together)

#### 5. Arena/Region-Based Allocation

**Strategy**: Large allocations (>128KB) go directly to OS via `mmap()` instead of using heap.

```
malloc(200,000):
  ↓
Check if > threshold (typically 128KB)
  ↓ YES
Call mmap() → get dedicated memory region
  ↓
Return pointer
  ↓
free(): munmap() entire region

Benefits:
✓ No heap fragmentation (separate region)
✓ Can return memory to OS immediately
✓ Page-aligned automatically
```

**jemalloc Arena Structure**:

```
Per-Thread Arena:
┌────────────────────────┐
│ Small bins  (0-1KB)    │ ← Thread-local, no locking
├────────────────────────┤
│ Large bins  (1KB-128KB)│ ← Shared, requires lock
├────────────────────────┤
│ Huge allocations       │ ← Direct mmap
└────────────────────────┘

Multiple arenas reduce lock contention:
Thread 1 → Arena 1
Thread 2 → Arena 2
Thread 3 → Arena 1  (reuse)
Thread 4 → Arena 2  (reuse)
```

#### 6. Deferred Coalescing

**Problem**: Coalescing on every free() is expensive (traversing neighbors, updating lists).

**Solution**: Batch coalesce operations.

```
Strategy 1: Lazy coalescing
- free(): Just add to free list, don't coalesce
- malloc(): If allocation fails, run coalescing pass, then retry

Strategy 2: Periodic coalescing
- Every N allocations, run full coalescing pass
- Amortized cost: O(1) per operation

Strategy 3: Opportunistic coalescing
- Coalesce only when freeing blocks adjacent to already-free blocks
- Detected via boundary tags
```

**Trade-off**:
- Less coalescing = faster free()
- More fragmentation temporarily
- Batch processing reduces overhead

**Benchmark Impact**:
```
Immediate coalescing:
  - free(): 50ns per call
  - malloc success rate: 95%

Deferred coalescing:
  - free(): 10ns per call (5x faster!)
  - malloc success rate: 92%
  - Periodic cleanup: 500ns every 1000 ops

Net: 2-3x overall throughput improvement
```

---

### Real Allocator Implementations

Let's see how production allocators combine these strategies:

#### jemalloc (Used by Rust, FreeBSD, Firefox, Meta)

**Architecture**:
```
┌───────────────────────────────────────┐
│ Per-Thread Caches (TCaches)          │
│  - Small allocations (<= 14KB)       │
│  - Zero contention                    │
├───────────────────────────────────────┤
│ Arenas (Shared regions)               │
│  - Multiple arenas reduce contention  │
│  - 4 cores × 4 arenas = 16 arenas    │
├───────────────────────────────────────┤
│ Size Classes                          │
│  - Tiny: 8-256 bytes                 │
│  - Small: 512B-14KB                  │
│  - Large: 16KB-4MB                   │
│  - Huge: >4MB (direct mmap)          │
└───────────────────────────────────────┘
```

**Key Features**:
- **Immediate coalescing** for small/medium
- **Deferred coalescing** for large
- **96 size classes** (fine-grained)
- **4MB chunks** (slab units)
- **Typical fragmentation: 10-15%**

#### tcmalloc (Used by Chrome, Golang)

**Architecture**:
```
┌───────────────────────────────────────┐
│ Per-Thread Caches                     │
│  - 60 size classes                    │
│  - Max cache size: 4MB per thread     │
├───────────────────────────────────────┤
│ Central Free List                     │
│  - Spans (groups of pages)            │
│  - Lock-protected                     │
├───────────────────────────────────────┤
│ Page Heap                             │
│  - 4KB pages                          │
│  - Buddy allocation for large         │
└───────────────────────────────────────┘
```

**Key Features**:
- **No coalescing** for small allocations (fixed spans)
- **Buddy system** for large (efficient coalescing)
- **Lock-free** for thread cache hits (99%+ of allocations)
- **Typical fragmentation: 12-18%**

#### dlmalloc/ptmalloc2 (Used by glibc, Linux default)

**Architecture**:
```
┌───────────────────────────────────────┐
│ Fast Bins (LIFO, no coalescing)      │
│  - 10 bins for very small sizes      │
│  - 16, 24, 32...80 bytes              │
├───────────────────────────────────────┤
│ Small Bins (size classes)             │
│  - 62 bins                            │
│  - FIFO order                         │
├───────────────────────────────────────┤
│ Large Bins (sorted by size)           │
│  - 63 bins                            │
│  - Best-fit search                    │
├───────────────────────────────────────┤
│ Unsorted Bin (recent frees)           │
│  - Staging area for coalescing        │
└───────────────────────────────────────┘
```

**Key Features**:
- **Immediate coalescing** except fast bins
- **Best-fit** for large allocations
- **Wilderness preservation** (keep large tail block)
- **Typical fragmentation: 15-25%** (higher than jemalloc/tcmalloc)

---

### Fragmentation Benchmarks

Real-world fragmentation measurements from long-running programs:

| Allocator | Redis (24h) | MySQL (24h) | Chrome (4h) | Average |
|-----------|-------------|-------------|-------------|---------|
| **dlmalloc** | 28% | 32% | 41% | 34% |
| **ptmalloc2** | 22% | 26% | 35% | 28% |
| **jemalloc** | 12% | 15% | 18% | 15% |
| **tcmalloc** | 14% | 17% | 22% | 18% |
| **Memory Pool** | 0% | 0% | 0% | 0% |

**Workload characteristics that cause fragmentation**:
- **Mixed sizes**: Allocating 8, 64, 1024, 8, 64, 1024 repeatedly
- **Long lifetime variance**: Some objects live milliseconds, others live hours
- **Phase transitions**: Allocate 1000 objects, free 500, allocate 1000 more
- **Random free order**: Free in different order than allocated

---

### Why Memory Pools Win for Specific Workloads

After understanding system allocators, we can now see exactly **why** and **when** memory pools are superior:

| Aspect | System Allocator | Memory Pool |
|--------|------------------|-------------|
| **Fragmentation** | 10-40% (varies) | 0% (fixed sizes) |
| **Allocation time** | 50-200ns (varies) | 5-10ns (constant) |
| **Metadata overhead** | 8-16 bytes/block | 0 bytes/block |
| **Cache locality** | Poor (scattered) | Excellent (contiguous) |
| **Predictability** | Poor (depends on history) | Perfect (always O(1)) |
| **Flexibility** | Any size | Fixed size only |

**When to use system allocator**:
- Variable-sized allocations
- Long-lived objects with complex lifetime patterns
- Small number of allocations (<1000/sec)
- General-purpose code

**When to use memory pool**:
- Fixed-size or narrow size range
- Short-lived objects (create/destroy cycles)
- High allocation rate (>10,000/sec)
- Real-time constraints (audio, games, trading systems)
- Embedded systems with limited memory

---
## Rust Programming Concepts for This Project


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

## Connection to This Project

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

## Build The Project


### Milestone 1: Basic Memory Pool Structure
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
### Milestone 2: Safe Wrapper with Ownership Tracking
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

### Milestone 3: Thread-Safe Pool with Arc and Mutex
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

```rust

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::ptr;

//==============================================================================
// Milestone  1: Raw Memory Pool
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
// Milestone  2: Type-Safe Pool with RAII
//==============================================================================

/// A typed memory pool for allocating objects of type T
pub struct TypedPool<T> {
    pool: MemoryPool,
    _marker: PhantomData<T>,
}

/// An RAII wrapper for an allocated block
/// Automatically returns memory to pool when dropped
pub struct Block<T> {
    data: *mut T,
    pool_ptr: *mut TypedPool<T>,
    _marker: PhantomData<T>,
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
                pool_ptr: self as *mut TypedPool<T>,
                _marker: PhantomData,
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

impl<T> Deref for Block<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: data is valid for the lifetime of Block
        unsafe { &*self.data }
    }
}

impl<T> DerefMut for Block<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: data is valid and uniquely owned by Block
        unsafe { &mut *self.data }
    }
}

impl<T> Drop for Block<T> {
    fn drop(&mut self) {
        // SAFETY: data was initialized in allocate()
        unsafe {
            ptr::drop_in_place(self.data);
            (*self.pool_ptr).deallocate_raw(self.data);
        }
    }
}

// SAFETY: Block can be sent between threads if T can (pool is not thread-local)
unsafe impl<T: Send> Send for Block<T> {}

//==============================================================================
// Milestone  3: Thread-Safe Shared Pool
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

        let mut all_blocks = vec![];
        for handle in handles {
            let blocks = handle.join().unwrap();
            assert_eq!(blocks.len(), 10);
            all_blocks.extend(blocks);
        }

        // All blocks are still allocated (held by all_blocks)
        assert_eq!(pool.available(), 0);

        // Drop all blocks and verify they're returned to pool
        drop(all_blocks);
        assert_eq!(pool.available(), 100);
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
            let _b1 = pool
                .allocate(DropCounter {
                    count: drop_count.clone(),
                })
                .unwrap();
            let _b2 = pool
                .allocate(DropCounter {
                    count: drop_count.clone(),
                })
                .unwrap();
        }

        assert_eq!(drop_count.load(Ordering::SeqCst), 2);
    }

    // Additional tests from specification
    #[test]
    fn test_pool_creation() {
        let pool = MemoryPool::new(1024, 64);
        assert_eq!(pool.block_size, 64);
        assert_eq!(pool.total_blocks(), 16);
        assert_eq!(pool.available(), 16);
    }

    #[test]
    fn test_single_allocation() {
        let mut pool = MemoryPool::new(1024, 64);
        let ptr = pool.allocate();
        assert!(ptr.is_some());
        assert_eq!(pool.available(), 15);
    }

    #[test]
    fn test_allocation_exhaustion() {
        let mut pool = MemoryPool::new(256, 64);
        let p1 = pool.allocate();
        let p2 = pool.allocate();
        let p3 = pool.allocate();
        let p4 = pool.allocate();
        let p5 = pool.allocate(); // Should fail - only 4 blocks available

        assert!(p1.is_some());
        assert!(p2.is_some());
        assert!(p3.is_some());
        assert!(p4.is_some());
        assert!(p5.is_none());
    }

    #[test]
    fn test_deallocation_and_reuse() {
        let mut pool = MemoryPool::new(256, 64);
        let ptr1 = pool.allocate().unwrap();
        assert_eq!(pool.available(), 3);

        pool.deallocate(ptr1);
        assert_eq!(pool.available(), 4);

        let ptr2 = pool.allocate();
        assert!(ptr2.is_some());
        assert_eq!(pool.available(), 3);
    }

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
        assert_eq!(pool.available(), 2);

        {
            let _b1 = pool.allocate(10).unwrap();
            let _b2 = pool.allocate(20).unwrap();
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
    fn test_shared_pool_creation() {
        let pool = SharedPool::<i32>::new(10);
        let block = pool.allocate(42);
        assert!(block.is_some());
        assert_eq!(*block.unwrap(), 42);
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

        let mut all_blocks = vec![];
        for handle in handles {
            all_blocks.extend(handle.join().unwrap());
        }

        // All allocations should succeed
        assert_eq!(pool.available(), 0);
        drop(all_blocks);
        assert_eq!(pool.available(), 100);
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
