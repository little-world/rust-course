# Cache-Aware Data Structures

### Problem Statement

Build a library of cache-optimized data structures that outperform standard implementations through better memory layout, prefetching, and cache-line awareness. Your library should include a cache-friendly vector, hash map, and priority queue, each demonstrating specific cache optimization techniques. Benchmark your implementations against std library to validate performance improvements and understand when custom structures provide value.

Your data structure library should support:
- CacheVec: Vector with prefetching and cache-line-aligned storage
- CacheHashMap: Open-addressing hash map with linear probing
- CachePriorityQueue: D-ary heap optimized for cache lines
- SmallVec integration for stack-allocated small collections
- Comprehensive benchmarks comparing against std
- Documentation of when to use each structure

## Why Cache-Aware Structures Matter

Modern computer systems are defined by a complex **memory hierarchy**. While CPUs have become incredibly fast, the speed of accessing main memory (RAM) has not kept pace. This growing "CPU-memory gap" makes efficient cache utilization paramount for high-performance applications.

### 1. The Critical Role of the Memory Hierarchy

The memory hierarchy is a tiered system designed to provide the CPU with data as quickly as possible. Data is moved between these tiers based on locality principles (temporal and spatial).

**The Performance Gap Quantified**:
The penalty for missing a cache and going to a lower level can be enormous.

```
Operation           Latency      Relative Cost (to L1)    Impact
L1 cache hit        0.5ns        1x                       CPU register speed
L2 cache hit        ~7ns         ~14x                     Minor stall
L3 cache hit        ~20ns        ~40x                     Noticeable stall
RAM access          ~100ns       ~200x                    Major stall, pipeline flush
Disk (SSD)          ~100,000ns   ~200,000x                Application unresponsive
```

**Impact Example**: Accessing data randomly can quickly turn a sub-millisecond operation into a hundreds-of-milliseconds nightmare.

```rust
// Sequential access (cache-friendly - data is contiguous, few cache misses)
let mut sum = 0;
for i in 0..1_000_000 {
    sum += array[i];  // Each access: ~1ns (L1/L2 cache hit)
}
// Total: ~1ms - CPU spends most time computing, not waiting

// Random access (cache-unfriendly - data is scattered, many cache misses)
let mut sum = 0;
for i in random_indices { // indices are random, so array[i] jumps around memory
    sum += array[i];  // Each access: ~200ns (RAM access due to cache miss)
}
// Total: ~200ms - CPU spends most time waiting for data from RAM

// The same computation can be 200x slower purely due to memory access patterns!
```

### 2. CPU Architecture and Pipelining

Modern CPUs use deep pipelines and speculative execution. A **cache miss** often means the CPU has to **stall** its pipeline, discard speculative work, and wait for data from slower memory. This is incredibly wasteful. Cache-aware design helps:
*   **Reduce pipeline stalls**: CPU has data when it needs it.
*   **Improve branch prediction**: Fewer random jumps, more predictable instruction flow.
*   **Utilize SIMD**: Contiguous data is essential for Single Instruction Multiple Data (SIMD) operations.

### 3. Cache Coherence and Concurrency

In multi-core systems, each CPU core has its own private L1/L2 caches.
*   **Cache Coherence Protocols (e.g., MESI)**: These protocols ensure all cores see a consistent view of memory. However, modifying shared data (even implicitly) can lead to:
    *   **Cache line invalidations**: One core modifies a cache line, invalidating it in others, forcing them to refetch from L3 or RAM.
    *   **False Sharing**: Two independent variables in different threads map to the same cache line. Modifying one causes constant invalidation of the other, leading to performance degradation.
Cache-aware structures can reduce false sharing by ensuring data frequently accessed together is placed on the same cache line, or data accessed independently is on different lines.

### 4. Overcoming Standard Library Limitations

While Rust's standard library data structures are robust and generally efficient, they are designed for broad utility, not extreme cache optimization.

**`std::collections::HashMap` Issues**:
```rust
use std::collections::HashMap;

// std HashMap typically uses separate chaining (linked lists for collisions)
// Each entry in the linked list can be separately allocated on the heap.
// This leads to "pointer chasing" where each step might be a cache miss.

let mut map = HashMap::new();
for i in 0..1000 {
    map.insert(i, i * 2);
}

// A lookup involves: hash → bucket → (potentially) iterate linked list.
// Following pointers from RAM to RAM to find elements means many expensive cache misses!
```

**`std::vec::Vec` Limitations**:
```rust
// Vec is highly cache-friendly for sequential access due to contiguous storage.
// However, advanced optimizations are still possible:
// - No explicit prefetching hints: Manual prefetching can further reduce latency.
// - No guaranteed cache-line alignment: Custom allocators can ensure data starts on a cache line boundary.
// - Growth strategy: While usually good, a fixed small buffer (SmallVec) can eliminate allocations entirely for small collections.
```

### 5. Quantifiable Performance Gains

By understanding and designing for the memory hierarchy, custom data structures can yield significant, measurable speedups:

| Structure         | vs. `std` (Typical)             | Best For                  | Key Technique                                      |
|-------------------|---------------------------------|---------------------------|----------------------------------------------------|
| `CacheVec`        | +10-20% sequential iteration    | Predictable sequential read | Manual prefetching, cache-line alignment           |
| `CacheHashMap`    | +2-5x lookups                   | Read-heavy, small keys    | Open addressing, contiguous storage                |
| `SmallVec`        | +5-10x for small collections    | Short-lived, < ~32 elements | Stack allocation (zero-cost creation/destruction)  |
| `CachePriorityQueue` | +30-50% push/pop               | High-throughput priority ops | D-ary heap, improved cache locality                |

## Use Cases

### 1. High-Frequency Lookups
- **Game engines**: Entity component systems with fast lookups
- **Databases**: Index structures, hash joins
- **Caching layers**: LRU caches, memoization
- **Networking**: Connection tables, routing tables

### 2. Predictable Access Patterns
- **Sequential processing**: Log analysis, data pipelines
- **Batch operations**: Bulk inserts, mass updates
- **Scientific computing**: Matrix operations, simulations

### 3. Memory-Constrained Systems
- **Embedded systems**: Limited RAM, cache matters more
- **Mobile devices**: Battery life (cache hits = less power)
- **High-performance computing**: Maximize throughput

### 4. Real-Time Systems
- **Trading systems**: Microsecond-level latency requirements
- **Game loops**: 60 FPS = 16ms budget per frame
- **Audio/video processing**: Real-time streaming

---

## Building the Project

### Milestone 1: Cache-Friendly Vector with Prefetching

**Goal**: Build a vector that uses manual prefetching to reduce cache miss latency.

**Why we start here**: Vectors are fundamental. Understanding prefetching teaches cache optimization principles.

#### Architecture

**Structs:**
- `CacheVec<T>` - Cache-optimized vector
  - **Field**: `data: *mut T` - Raw pointer to data
  - **Field**: `len: usize` - Number of elements
  - **Field**: `capacity: usize` - Allocated capacity
  - **Field**: `_marker: PhantomData<T>` - Ownership marker

**Functions:**
- `new() -> CacheVec<T>` - Create empty vector
- `with_capacity(cap: usize) -> CacheVec<T>` - Pre-allocate
- `push(&mut self, value: T)` - Add element
- `iter_with_prefetch(&self) -> PrefetchIter<T>` - Iterator with prefetching
- `get_prefetch(&self, index: usize) -> Option<&T>` - Get with prefetch hint

**Starter Code**:

```rust
use std::marker::PhantomData;
use std::ptr;
use std::alloc::{alloc, dealloc, realloc, Layout};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub struct CacheVec<T> {
    data: *mut T,
    len: usize,
    capacity: usize,
    _marker: PhantomData<T>,
}

impl<T> CacheVec<T> {
    pub fn new() -> Self {
        // TODO: Initialize empty vector
        todo!("Create empty CacheVec")
    }

    pub fn with_capacity(capacity: usize) -> Self {
        // TODO: Allocate capacity elements
        // TODO: Ensure cache-line alignment (64 bytes)
        todo!("Create with capacity")
    }

    pub fn push(&mut self, value: T) {
        // TODO: Check if need to grow
        // TODO: Write value at end
        // TODO: Increment len
        todo!("Push element")
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe { Some(&*self.data.add(index)) }
        } else {
            None
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn get_prefetch(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe {
                // Prefetch next cache line
                const PREFETCH_DISTANCE: usize = 8;  // 64 bytes / 8 bytes per T
                if index + PREFETCH_DISTANCE < self.len {
                    let prefetch_ptr = self.data.add(index + PREFETCH_DISTANCE);
                    _mm_prefetch(prefetch_ptr as *const i8, _MM_HINT_T0);
                }

                Some(&*self.data.add(index))
            }
        } else {
            None
        }
    }

    fn grow(&mut self) {
        // TODO: Double capacity
        // TODO: Reallocate and copy data
        // TODO: Ensure alignment
        todo!("Grow vector")
    }

    pub fn iter_with_prefetch(&self) -> PrefetchIter<T> {
        PrefetchIter {
            vec: self,
            index: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

pub struct PrefetchIter<'a, T> {
    vec: &'a CacheVec<T>,
    index: usize,
}

impl<'a, T> Iterator for PrefetchIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.len() {
            let item = self.vec.get_prefetch(self.index);
            self.index += 1;
            item
        } else {
            None
        }
    }
}

impl<T> Drop for CacheVec<T> {
    fn drop(&mut self) {
        // TODO: Drop all elements
        // TODO: Deallocate memory
        todo!("Drop CacheVec")
    }
}

unsafe impl<T: Send> Send for CacheVec<T> {}
unsafe impl<T: Sync> Sync for CacheVec<T> {}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_push_and_get() {
        let mut vec = CacheVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        assert_eq!(vec.len(), 3);
        assert_eq!(*vec.get(0).unwrap(), 1);
        assert_eq!(*vec.get(2).unwrap(), 3);
    }

    #[test]
    fn test_growth() {
        let mut vec = CacheVec::with_capacity(2);
        for i in 0..10 {
            vec.push(i);
        }

        assert_eq!(vec.len(), 10);
        assert!(vec.capacity() >= 10);
    }

    #[test]
    fn test_iterator() {
        let mut vec = CacheVec::new();
        for i in 0..5 {
            vec.push(i);
        }

        let collected: Vec<_> = vec.iter_with_prefetch().copied().collect();
        assert_eq!(collected, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    #[ignore]
    fn benchmark_prefetch() {
        // Create large vector
        let mut vec = CacheVec::with_capacity(10_000_000);
        for i in 0..10_000_000 {
            vec.push(i);
        }

        // Without prefetch
        let start = Instant::now();
        let mut sum1 = 0;
        for i in 0..vec.len() {
            sum1 += vec.get(i).unwrap();
        }
        let no_prefetch = start.elapsed();

        // With prefetch
        let start = Instant::now();
        let mut sum2 = 0;
        for i in 0..vec.len() {
            sum2 += vec.get_prefetch(i).unwrap();
        }
        let with_prefetch = start.elapsed();

        println!("No prefetch: {:?}", no_prefetch);
        println!("With prefetch: {:?}", with_prefetch);
        println!("Speedup: {:.2}x", no_prefetch.as_secs_f64() / with_prefetch.as_secs_f64());

        assert_eq!(sum1, sum2);
    }
}
```

**Check Your Understanding**:
- Why does prefetching help?
- What is the optimal prefetch distance?
- When does prefetching hurt performance?

---

#### Why Milestone 1 Isn't Enough

**Limitation**: Prefetching helps sequential access, but hash maps have random access patterns—need different optimization.

**What we're adding**: Open-addressing hash map that keeps data contiguous for better cache usage.

**Improvement**:
- **Locality**: All data in one allocation, not scattered
- **Cache lines**: Fill cache lines efficiently
- **Predictability**: Linear probing is cache-friendly
- **Speed**: 2-5x faster lookups than chaining

---

### Milestone 2: Open-Addressing Hash Map

**Goal**: Build a hash map using open addressing (linear probing) for better cache performance than separate chaining.

**Why this matters**: std::HashMap uses separate chaining—each entry is a separate allocation. Open addressing keeps everything contiguous.

#### Architecture

**Concepts:**
- **Open addressing**: Store collisions in same array
- **Linear probing**: Check next slot if occupied
- **Tombstones**: Mark deleted entries
- **Load factor**: Resize when 70% full

**Structs:**
- `CacheHashMap<K, V>` - Cache-friendly hash map
  - **Field**: `buckets: Vec<Bucket<K, V>>` - Contiguous storage
  - **Field**: `len: usize` - Number of entries
  - **Field**: `capacity: usize` - Bucket count

- `Bucket<K, V>` - One hash map slot
  - **Variants**:
    - `Empty` - Unused slot
    - `Occupied(K, V)` - Contains key-value pair
    - `Tombstone` - Deleted entry

**Functions:**
- `new() -> CacheHashMap<K, V>` - Create map
- `insert(&mut self, key: K, value: V) -> Option<V>` - Insert or update
- `get(&self, key: &K) -> Option<&V>` - Lookup value
- `remove(&mut self, key: &K) -> Option<V>` - Delete entry
- `resize(&mut self)` - Grow capacity

**Starter Code**:

```rust
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Clone)]
enum Bucket<K, V> {
    Empty,
    Occupied(K, V),
    Tombstone,
}

pub struct CacheHashMap<K, V> {
    buckets: Vec<Bucket<K, V>>,
    len: usize,
    capacity: usize,
}

impl<K: Hash + Eq, V> CacheHashMap<K, V> {
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        // TODO: Allocate buckets
        // TODO: Round up to power of 2 for fast modulo
        // TODO: Initialize all to Empty
        todo!("Create with capacity")
    }

    fn hash(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % self.capacity
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // TODO: Check if need resize (load factor > 0.7)
        // TODO: Hash key to find starting bucket
        // TODO: Linear probe to find empty/matching slot
        // TODO: Insert or update
        todo!("Insert key-value pair")
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        // TODO: Hash key
        // TODO: Linear probe to find key or Empty
        // TODO: Return reference to value
        todo!("Get value")
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        // TODO: Find key with linear probing
        // TODO: Replace with Tombstone
        // TODO: Return old value
        todo!("Remove entry")
    }

    fn resize(&mut self) {
        // TODO: Double capacity
        // TODO: Rehash all entries
        // TODO: Skip tombstones
        todo!("Resize map")
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn load_factor(&self) -> f64 {
        self.len as f64 / self.capacity as f64
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_insert_and_get() {
        let mut map = CacheHashMap::new();

        map.insert("key1", 100);
        map.insert("key2", 200);

        assert_eq!(map.get(&"key1"), Some(&100));
        assert_eq!(map.get(&"key2"), Some(&200));
        assert_eq!(map.get(&"key3"), None);
    }

    #[test]
    fn test_update() {
        let mut map = CacheHashMap::new();

        map.insert("key", 100);
        let old = map.insert("key", 200);

        assert_eq!(old, Some(100));
        assert_eq!(map.get(&"key"), Some(&200));
    }

    #[test]
    fn test_remove() {
        let mut map = CacheHashMap::new();

        map.insert("key", 100);
        let removed = map.remove(&"key");

        assert_eq!(removed, Some(100));
        assert_eq!(map.get(&"key"), None);
    }

    #[test]
    fn test_resize() {
        let mut map = CacheHashMap::with_capacity(4);

        // Insert enough to trigger resize
        for i in 0..10 {
            map.insert(i, i * 10);
        }

        assert!(map.capacity() > 4);

        // All values still accessible
        for i in 0..10 {
            assert_eq!(map.get(&i), Some(&(i * 10)));
        }
    }

    #[test]
    #[ignore]
    fn benchmark_vs_std() {
        use std::time::Instant;

        const N: usize = 1_000_000;

        // CacheHashMap
        let mut cache_map = CacheHashMap::with_capacity(N);
        let start = Instant::now();
        for i in 0..N {
            cache_map.insert(i, i * 2);
        }
        let cache_insert = start.elapsed();

        let start = Instant::now();
        for i in 0..N {
            let _ = cache_map.get(&i);
        }
        let cache_lookup = start.elapsed();

        // std HashMap
        let mut std_map = HashMap::with_capacity(N);
        let start = Instant::now();
        for i in 0..N {
            std_map.insert(i, i * 2);
        }
        let std_insert = start.elapsed();

        let start = Instant::now();
        for i in 0..N {
            let _ = std_map.get(&i);
        }
        let std_lookup = start.elapsed();

        println!("\n=== HashMap Benchmark ({} elements) ===", N);
        println!("Insert - Cache: {:?}, Std: {:?}", cache_insert, std_insert);
        println!("Lookup - Cache: {:?}, Std: {:?}", cache_lookup, std_lookup);
        println!("Speedup - Insert: {:.2}x, Lookup: {:.2}x",
            std_insert.as_secs_f64() / cache_insert.as_secs_f64(),
            std_lookup.as_secs_f64() / cache_lookup.as_secs_f64());
    }
}
```

---

#### Why Milestone 2 Isn't Enough

**Limitation**: Small collections (< 100 elements) shouldn't allocate at all—use stack storage.

**What we're adding**: SmallVec integration for stack-allocated small collections.

**Improvement**:
- **Zero allocations**: Small collections on stack
- **Speed**: Stack access faster than heap
- **Simplicity**: Same API for small and large
- **Memory**: No allocator overhead for small cases

---

### Milestone 3: SmallVec for Stack-Allocated Collections

**Goal**: Implement SmallVec that stores small collections on the stack, spilling to heap when necessary.

**Why this matters**: Most collections are small. Stack storage avoids allocations entirely.

#### Architecture

**Concepts:**
- **Inline storage**: Fixed-size array on stack
- **Spilling**: Move to heap when exceeding capacity
- **Tagged union**: Discriminate inline vs heap

**Structs:**
- `SmallVec<T, const N: usize>` - Stack or heap vec
  - **Field**: `data: SmallVecData<T, N>` - Storage
  - **Field**: `len: usize` - Element count

- `SmallVecData<T, const N: usize>` - Storage union
  - **Variants**:
    - `Inline([MaybeUninit<T>; N])` - Stack array
    - `Heap(*mut T, usize)` - Heap pointer + capacity

**Functions:**
- `new() -> SmallVec<T, N>` - Create empty (inline)
- `push(&mut self, value: T)` - Add element, spill if needed
- `spill_to_heap(&mut self)` - Convert inline to heap

**Starter Code**:

```rust
use std::mem::MaybeUninit;

enum SmallVecData<T, const N: usize> {
    Inline([MaybeUninit<T>; N]),
    Heap(*mut T, usize),  // pointer, capacity
}

pub struct SmallVec<T, const N: usize> {
    data: SmallVecData<T, N>,
    len: usize,
}

impl<T, const N: usize> SmallVec<T, N> {
    pub fn new() -> Self {
        // TODO: Initialize with inline storage
        todo!("Create SmallVec")
    }

    pub fn push(&mut self, value: T) {
        // TODO: Check if inline and at capacity
        // TODO: If so, spill to heap first
        // TODO: Then push value
        todo!("Push element")
    }

    fn spill_to_heap(&mut self) {
        // TODO: Allocate heap storage
        // TODO: Move inline elements to heap
        // TODO: Switch to Heap variant
        todo!("Spill to heap")
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            match &self.data {
                SmallVecData::Inline(arr) => {
                    unsafe { Some(arr[index].assume_init_ref()) }
                }
                SmallVecData::Heap(ptr, _) => {
                    unsafe { Some(&*ptr.add(index)) }
                }
            }
        } else {
            None
        }
    }

    pub fn is_inline(&self) -> bool {
        matches!(self.data, SmallVecData::Inline(_))
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl<T, const N: usize> Drop for SmallVec<T, N> {
    fn drop(&mut self) {
        // TODO: Drop all elements
        // TODO: If heap, deallocate
        todo!("Drop SmallVec")
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_storage() {
        let mut vec: SmallVec<i32, 4> = SmallVec::new();

        vec.push(1);
        vec.push(2);
        vec.push(3);

        assert!(vec.is_inline());
        assert_eq!(vec.len(), 3);
    }

    #[test]
    fn test_spill_to_heap() {
        let mut vec: SmallVec<i32, 4> = SmallVec::new();

        for i in 0..10 {
            vec.push(i);
        }

        assert!(!vec.is_inline());  // Should have spilled
        assert_eq!(vec.len(), 10);

        // All values still accessible
        for i in 0..10 {
            assert_eq!(*vec.get(i).unwrap(), i);
        }
    }

    #[test]
    #[ignore]
    fn benchmark_smallvec() {
        use std::time::Instant;

        // Small collections (fits inline)
        let start = Instant::now();
        for _ in 0..1_000_000 {
            let mut vec: SmallVec<i32, 8> = SmallVec::new();
            for i in 0..5 {
                vec.push(i);
            }
            // vec drops here - no deallocation needed!
        }
        let small_time = start.elapsed();

        // Regular Vec (always allocates)
        let start = Instant::now();
        for _ in 0..1_000_000 {
            let mut vec = Vec::new();
            for i in 0..5 {
                vec.push(i);
            }
            // vec drops here - deallocation required
        }
        let vec_time = start.elapsed();

        println!("SmallVec (inline): {:?}", small_time);
        println!("Vec (heap): {:?}", vec_time);
        println!("Speedup: {:.2}x", vec_time.as_secs_f64() / small_time.as_secs_f64());
    }
}
```

---

#### Why Milestone 3 Isn't Enough

**Limitation**: Priority queues (binary heaps) have poor cache locality due to tree structure.

**What we're adding**: D-ary heap that packs more children per cache line.

**Improvement**:
- **Cache efficiency**: More nodes per cache line
- **Branching factor**: D=4 or D=8 works well
- **Predictability**: Better branch prediction
- **Speed**: 30-50% faster than binary heap

---

### Milestone 4: Cache-Optimized D-Ary Heap

**Goal**: Implement a priority queue using a d-ary heap (d=4) for better cache performance.

**Why this matters**: Binary heaps jump around memory. D-ary heaps keep related nodes close together.

#### Architecture

**Concepts:**
- **D-ary heap**: Each node has D children (not 2)
- **Array layout**: Children at indices `d*i+1` through `d*i+d`
- **Cache lines**: D=4 fits 4 children in one cache line
- **Fewer levels**: Tree is shallower

**Structs:**
- `CachePriorityQueue<T, const D: usize>` - D-ary heap
  - **Field**: `data: Vec<T>` - Heap array
  - **Field**: `_marker: PhantomData<T>`

**Functions:**
- `new() -> CachePriorityQueue<T, D>` - Create empty heap
- `push(&mut self, value: T)` - Insert element
- `pop(&mut self) -> Option<T>` - Remove min/max
- `bubble_up(&mut self, index: usize)` - Restore heap property upward
- `bubble_down(&mut self, index: usize)` - Restore heap property downward

**Starter Code**:

```rust
use std::cmp::Ord;

pub struct CachePriorityQueue<T: Ord, const D: usize> {
    data: Vec<T>,
}

impl<T: Ord, const D: usize> CachePriorityQueue<T, D> {
    pub fn new() -> Self {
        CachePriorityQueue { data: Vec::new() }
    }

    pub fn push(&mut self, value: T) {
        // TODO: Add to end
        // TODO: Bubble up to restore heap property
        self.data.push(value);
        let index = self.data.len() - 1;
        self.bubble_up(index);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.data.is_empty() {
            return None;
        }

        // TODO: Swap first and last
        // TODO: Remove last
        // TODO: Bubble down to restore heap property
        let last_index = self.data.len() - 1;
        self.data.swap(0, last_index);
        let result = self.data.pop();
        if !self.data.is_empty() {
            self.bubble_down(0);
        }
        result
    }

    fn parent(index: usize) -> usize {
        (index - 1) / D
    }

    fn first_child(index: usize) -> usize {
        D * index + 1
    }

    fn bubble_up(&mut self, mut index: usize) {
        // TODO: While not root and less than parent
        // TODO: Swap with parent
        while index > 0 {
            let parent = Self::parent(index);
            if self.data[index] < self.data[parent] {
                self.data.swap(index, parent);
                index = parent;
            } else {
                break;
            }
        }
    }

    fn bubble_down(&mut self, mut index: usize) {
        // TODO: While has children
        // TODO: Find smallest child
        // TODO: If smaller than current, swap
        loop {
            let first_child = Self::first_child(index);
            if first_child >= self.data.len() {
                break;
            }

            // Find smallest among D children
            let mut smallest = first_child;
            for i in 1..D {
                let child = first_child + i;
                if child < self.data.len() && self.data[child] < self.data[smallest] {
                    smallest = child;
                }
            }

            if self.data[smallest] < self.data[index] {
                self.data.swap(index, smallest);
                index = smallest;
            } else {
                break;
            }
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BinaryHeap;

    #[test]
    fn test_push_pop() {
        let mut pq: CachePriorityQueue<i32, 4> = CachePriorityQueue::new();

        pq.push(5);
        pq.push(2);
        pq.push(8);
        pq.push(1);

        assert_eq!(pq.pop(), Some(1));
        assert_eq!(pq.pop(), Some(2));
        assert_eq!(pq.pop(), Some(5));
        assert_eq!(pq.pop(), Some(8));
        assert_eq!(pq.pop(), None);
    }

    #[test]
    fn test_heap_property() {
        let mut pq: CachePriorityQueue<i32, 4> = CachePriorityQueue::new();

        for i in (0..100).rev() {
            pq.push(i);
        }

        let mut sorted = Vec::new();
        while let Some(val) = pq.pop() {
            sorted.push(val);
        }

        // Should be sorted
        for i in 0..sorted.len() - 1 {
            assert!(sorted[i] <= sorted[i + 1]);
        }
    }

    #[test]
    #[ignore]
    fn benchmark_d_ary_heap() {
        use std::time::Instant;

        const N: usize = 1_000_000;

        // Binary heap (std)
        let mut binary_heap = BinaryHeap::new();
        let start = Instant::now();
        for i in 0..N {
            binary_heap.push(N - i);  // Reverse order
        }
        for _ in 0..N {
            binary_heap.pop();
        }
        let binary_time = start.elapsed();

        // 4-ary heap (cache-friendly)
        let mut quad_heap: CachePriorityQueue<usize, 4> = CachePriorityQueue::new();
        let start = Instant::now();
        for i in 0..N {
            quad_heap.push(N - i);
        }
        for _ in 0..N {
            quad_heap.pop();
        }
        let quad_time = start.elapsed();

        println!("Binary heap: {:?}", binary_time);
        println!("4-ary heap: {:?}", quad_time);
        println!("Speedup: {:.2}x", binary_time.as_secs_f64() / quad_time.as_secs_f64());
    }
}
```

---

#### Why Milestone 4 Isn't Enough

**Limitation**: Individual optimizations help, but we need to measure and validate the improvements systematically.

**What we're adding**: Comprehensive benchmarking suite comparing all structures against std library.

**Improvement**:
- **Validation**: Prove optimizations work
- **Regression detection**: Catch slowdowns
- **Guidance**: Know when to use what
- **Learning**: Understand trade-offs

---

### Milestone 5: Comprehensive Benchmark Suite

**Goal**: Create a thorough benchmarking framework that compares all custom structures against std library equivalents.

**Why this matters**: Claims need evidence. Benchmarks prove (or disprove) optimization value.

#### Architecture

**Benchmark Categories:**
- **Sequential access**: Where prefetching helps
- **Random access**: Where cache layout matters
- **Insertion**: Allocation patterns
- **Lookup**: Hash map performance
- **Priority operations**: Heap performance

**Functions:**
- `benchmark_sequential_access()` - Vec iteration
- `benchmark_random_access()` - Random lookups
- `benchmark_hash_map_operations()` - Insert/lookup/remove
- `benchmark_priority_queue()` - Push/pop sequences
- `benchmark_small_collections()` - SmallVec vs Vec

**Starter Code**:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_sequential_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_access");

    for size in [1000, 10000, 100000, 1000000] {
        // std Vec
        group.bench_with_input(BenchmarkId::new("std_vec", size), &size, |b, &size| {
            let vec: Vec<i32> = (0..size).collect();
            b.iter(|| {
                let mut sum = 0;
                for &x in &vec {
                    sum += black_box(x);
                }
                black_box(sum)
            });
        });

        // CacheVec with prefetch
        group.bench_with_input(BenchmarkId::new("cache_vec", size), &size, |b, &size| {
            let mut cache_vec = CacheVec::with_capacity(size as usize);
            for i in 0..size {
                cache_vec.push(i);
            }
            b.iter(|| {
                let mut sum = 0;
                for i in 0..cache_vec.len() {
                    sum += black_box(*cache_vec.get_prefetch(i).unwrap());
                }
                black_box(sum)
            });
        });
    }

    group.finish();
}

fn benchmark_hash_map_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_operations");

    for size in [1000, 10000, 100000] {
        // std HashMap
        group.bench_with_input(BenchmarkId::new("std_hashmap", size), &size, |b, &size| {
            b.iter(|| {
                let mut map = std::collections::HashMap::new();
                for i in 0..size {
                    map.insert(i, i * 2);
                }
                for i in 0..size {
                    black_box(map.get(&i));
                }
            });
        });

        // CacheHashMap
        group.bench_with_input(BenchmarkId::new("cache_hashmap", size), &size, |b, &size| {
            b.iter(|| {
                let mut map = CacheHashMap::new();
                for i in 0..size {
                    map.insert(i, i * 2);
                }
                for i in 0..size {
                    black_box(map.get(&i));
                }
            });
        });
    }

    group.finish();
}

fn benchmark_small_collections(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_collections");

    // Small size (fits inline)
    group.bench_function("smallvec_inline", |b| {
        b.iter(|| {
            let mut vec: SmallVec<i32, 8> = SmallVec::new();
            for i in 0..5 {
                vec.push(black_box(i));
            }
            black_box(vec)
        });
    });

    group.bench_function("std_vec_small", |b| {
        b.iter(|| {
            let mut vec = Vec::new();
            for i in 0..5 {
                vec.push(black_box(i));
            }
            black_box(vec)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_sequential_access,
    benchmark_hash_map_operations,
    benchmark_small_collections
);
criterion_main!(benches);
```

---

#### Why Milestone 5 Isn't Enough

**Limitation**: Benchmarks show numbers, but developers need guidance on when to use what.

**What we're adding**: Decision framework and documentation explaining trade-offs.

**Improvement**:
- **Clarity**: Know when optimizations help
- **Trade-offs**: Understand costs
- **Guidance**: Make informed decisions
- **Completeness**: Full picture of performance

---

### Milestone 6: Decision Framework and Documentation

**Goal**: Provide clear guidelines on when to use each data structure based on access patterns and requirements.

**Why this matters**: The best optimization is using the right structure for the job.

#### Decision Framework

**When to Use CacheVec:**
- ✅ Sequential iteration with predictable access
- ✅ Large collections (> 10,000 elements)
- ❌ Random access (prefetching doesn't help)
- ❌ Small collections (overhead not worth it)

**When to Use CacheHashMap:**
- ✅ Many lookups (read-heavy workload)
- ✅ Integer or small keys
- ✅ Predictable size (can pre-allocate)
- ❌ Many deletions (tombstones accumulate)
- ❌ Very large keys (open addressing inefficient)

**When to Use SmallVec:**
- ✅ Usually small (< 8 elements)
- ✅ Short-lived collections
- ✅ Hot path (created frequently)
- ❌ Always large (just use Vec)
- ❌ Need to share (inline storage not thread-safe)

**When to Use Cache PriorityQueue:**
- ✅ Many push/pop operations
- ✅ Sorting-like workload
- ✅ Predictable access pattern
- ❌ Rare usage (overhead not worth it)
- ❌ Need stable sort (heap isn't stable)

**Benchmark Summary Table:**

```rust
pub fn print_recommendation_table() {
    println!(r#"
╔═══════════════════╦═══════════════╦═══════════════╦═════════════════════════╗
║ Data Structure    ║ vs std        ║ Best For      ║ Avoid When              ║
╠═══════════════════╬═══════════════╬═══════════════╬═════════════════════════╣
║ CacheVec          ║ +10-20%       ║ Sequential    ║ Random access           ║
║ CacheHashMap      ║ +2-5x         ║ Lookups       ║ Many deletions          ║
║ SmallVec          ║ +5-10x        ║ Small, temp   ║ Always large            ║
║ CachePriorityQ    ║ +30-50%       ║ Many push/pop ║ Rare usage              ║
╚═══════════════════╩═══════════════╩═══════════════╩═════════════════════════╝
    "#);
}
```

---

## Testing Strategies

### 1. Correctness Tests
- Verify same behavior as std equivalents
- Test edge cases (empty, single element, etc.)
- Property-based testing for invariants

### 2. Performance Tests
- Criterion benchmarks for all operations
- Compare against std library
- Test with various sizes

### 3. Memory Tests
- Verify no leaks with valgrind
- Check allocation counts
- Measure memory overhead

### 4. Cache Tests
- Use performance counters to measure cache misses
- Compare prefetch vs no prefetch
- Validate cache-line alignment

---

## Complete Working Example

The complete library demonstrates:
- **CacheVec**: 10-20% faster sequential access with prefetching
- **CacheHashMap**: 2-5x faster lookups with open addressing
- **SmallVec**: 5-10x faster for small collections via stack storage
- **CachePriorityQueue**: 30-50% faster with d-ary heap layout

Students learn when custom structures provide value and when std library is already optimal. The key lesson: measure, don't guess—optimizations are workload-specific.
