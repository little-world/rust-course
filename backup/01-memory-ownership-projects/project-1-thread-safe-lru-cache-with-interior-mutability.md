## Project 1: Thread-Safe LRU Cache with Interior Mutability

### Problem Statement

Implement a Least Recently Used (LRU) cache that tracks the most recently accessed items and evicts the least recently used item when capacity is reached. You'll start with a simple single-threaded version, then progress to thread-safe variants.

### Why It Matters

**Real-World Impact**: LRU caches are everywhere in production systems:
- **Web servers** cache database query results, reducing load by 50-90%
- **CDNs** use LRU to keep popular content in fast storage
- **Operating systems** use LRU for page replacement in virtual memory
- **Browsers** cache DNS lookups, images, and compiled JavaScript

**Performance Numbers**: A well-implemented cache can turn a 100ms database query into a 0.1ms memory lookup—a 1000x speedup. For a web server handling 10,000 requests/second, this means the difference between needing 100 database servers vs 1.

**Rust-Specific Challenge**: Traditional caches in other languages use mutable references everywhere. Rust's ownership system forces us to think differently—we need interior mutability patterns to mutate cache contents through shared references. This project teaches you how to work *with* Rust's ownership rather than fighting it.

### Use Cases

**When you need this pattern**:
1. **Web application request handlers** - Multiple handlers share a cache, each needs read/write access without `&mut self`
2. **Multithreaded servers** - Threads need concurrent access to shared cache (hits: read, misses: write, evictions: write)
3. **Game engines** - Asset cache shared across systems, texture cache for renderer
4. **Build systems** - Compilation result cache, dependency resolution cache
5. **Database query caching** - Prepared statement cache, result set cache

**Real Examples**:
- Redis (in-memory cache) uses similar eviction strategies
- Linux kernel page cache uses LRU for memory management
- Chrome V8 uses LRU for inline caches
- Rust compiler uses LRU for incremental compilation cache

### Learning Goals

- Understand interior mutability with `RefCell` and `Mutex`/`RwLock`
- Practice working with shared references that allow mutation
- Learn proper lock scope management
- Build intuition for when to use different concurrency primitives
- Experience the performance trade-offs: `Cell` vs `RefCell` vs `Mutex` vs `RwLock`

---

### Milestone 1: Basic Statistics Tracker (Warmup)

**Why Start Here?**

Before diving into the full complexity of an LRU cache, we begin with a **statistics tracker** (hit/miss counter) for several important reasons:

1. **Introduces Interior Mutability Gently**: A cache needs to track statistics through shared references (`&self`), but counters need to be incremented. This creates a fundamental Rust challenge: how do you mutate through an immutable reference? Starting with simple counters lets you understand `Cell<T>` without the complexity of data structures.

2. **LRU Caches Need Metrics**: In production, an LRU cache without metrics is blind. You need to know:
   - **Hit rate** (hits / total accesses) - Is the cache effective?
   - **Miss rate** - Should you increase capacity?
   - Whether the cache is worth the memory cost

3. **Real-World Pattern**: Every production cache has a statistics layer. Redis, Memcached, and CDN caches all expose hit/miss metrics. This isn't academic—it's essential for operations.

4. **Demonstrates the "Shared but Mutable" Problem**: When multiple parts of your code hold `&cache`, they all need to update stats independently. This is impossible with normal Rust references, making it the perfect introduction to interior mutability patterns.

**The Core Challenge**: How do we increment counters through `&self` instead of `&mut self`? This milestone teaches you the solution: `Cell<T>`.

**Goal**: Create a simple counter that can be incremented through a shared reference.

**Starter Code**:
```rust
use std::cell::Cell;

struct StatsTracker {
    hits: Cell<usize>,
    misses: Cell<usize>,
}

impl StatsTracker {
    fn new() -> Self {
        // TODO: Initialize with zero hits and misses
        todo!()
    }

    fn record_hit(&self) {
        // TODO: Increment hits using Cell::get and Cell::set
        todo!()
    }

    fn record_miss(&self) {
        // TODO: Increment misses
        todo!()
    }

    fn get_stats(&self) -> (usize, usize) {
        // TODO: Return (hits, misses)
        todo!()
    }
}
```

**Checkpoint Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_tracker() {
        let tracker = StatsTracker::new();
        assert_eq!(tracker.get_stats(), (0, 0));

        tracker.record_hit();
        tracker.record_hit();
        assert_eq!(tracker.get_stats(), (2, 0));

        tracker.record_miss();
        assert_eq!(tracker.get_stats(), (2, 1));
    }

    #[test]
    fn test_multiple_references() {
        let tracker = StatsTracker::new();
        let ref1 = &tracker;
        let ref2 = &tracker;

        ref1.record_hit();
        ref2.record_miss();

        assert_eq!(tracker.get_stats(), (1, 1));
    }
}
```

**Check Your Understanding**:
- Why can we call `record_hit(&self)` without `&mut self`?
- What is `Cell<T>` and why is it useful here??
- Why do we need `Cell::get()` before incrementing?

---

### 🔄 Why Milestone 1 Isn't Enough → Moving to Milestone 2

**Limitation**: `Cell<T>` only works with `Copy` types (like `usize`, `bool`, `i32`). For caching, we need to store complex data structures like `HashMap<K, V>`, which don't implement `Copy`.

**What we're adding**: `RefCell<T>` allows interior mutability for *any* type, not just `Copy` types. Trade-off: `Cell` has zero overhead, while `RefCell` performs runtime borrow checking.

**Improvement**:
- **Capability**: Can now wrap collections (HashMap, Vec, etc.)
- **Cost**: Small runtime overhead for borrow checking (~1-2 CPU cycles)
- **Safety**: Panics if borrow rules violated at runtime vs compile errors with plain borrows

---

### Milestone 2: Simple HashMap Cache (No Eviction)

**Goal**: Create a cache using `RefCell<HashMap>` that can insert and retrieve values through `&self`.

**Starter Code**:
```rust
use std::cell::RefCell;
use std::collections::HashMap;

struct SimpleCache<K, V> {
    data: RefCell<HashMap<K, V>>,
}

impl<K, V> SimpleCache<K, V>
where
    K: Eq + std::hash::Hash,
    V: Clone,
{
    fn new() -> Self {
        // TODO: Create cache with empty HashMap wrapped in RefCell
        todo!()
    }

    fn get(&self, key: &K) -> Option<V> {
        // TODO: Use borrow() to get read access to HashMap
        // Return cloned value if present
        todo!()
    }

    fn put(&self, key: K, value: V) {
        // TODO: Use borrow_mut() to get write access to HashMap
        // Insert the key-value pair
        todo!()
    }

    fn len(&self) -> usize {
        // TODO: Return number of items in cache
        todo!()
    }
}
```

**Checkpoint Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_cache() {
        let cache = SimpleCache::new();
        assert_eq!(cache.len(), 0);

        cache.put("key1", "value1");
        assert_eq!(cache.len(), 1);

        assert_eq!(cache.get(&"key1"), Some("value1"));
        assert_eq!(cache.get(&"key2"), None);
    }

    #[test]
    fn test_update_existing() {
        let cache = SimpleCache::new();
        cache.put("key", "value1");
        cache.put("key", "value2");

        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get(&"key"), Some("value2"));
    }

    #[test]
    #[should_panic]
    fn test_borrow_violation() {
        let cache: SimpleCache<i32, i32> = SimpleCache::new();
        cache.put(1, 100);

        // This should panic: holding borrow across another borrow_mut
        let data = cache.data.borrow();
        cache.put(2, 200);  // This will panic!
        drop(data);
    }
}
```

**Check Your Understanding**:
- What's the difference between `borrow()` and `borrow_mut()`?
- When is the borrow automatically released?
- Why did the `test_borrow_violation` panic?
- Why do we need `V: Clone`?

---

### 🔄 Why Milestone 2 Isn't Enough → Moving to Milestone 3

**Limitation**: Our cache grows unbounded! A cache without eviction will eventually consume all memory. In production, this causes OOM (Out Of Memory) kills.

**What we're adding**:
- **Capacity limits** - Prevent unbounded memory growth
- **LRU eviction policy** - When full, remove least recently used item
- **Access tracking** - `VecDeque` tracks access order (most recent at back)

**Improvement**:
- **Memory**: Bounded memory usage (capacity × item_size)
- **Predictability**: Cache size never exceeds capacity
- **Algorithm**: O(1) eviction (remove front of VecDeque)
- **Complexity**: Need to manage two data structures in sync (HashMap + VecDeque)

**Real-world importance**: Redis has maxmemory settings with eviction policies. Without eviction, caches become memory leaks that crash servers.

---

### Milestone 3: LRU Cache with Fixed Capacity

**Goal**: Add capacity limit and eviction logic. Use `VecDeque` to track access order.

**Starter Code**:
```rust
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

struct LRUCache<K, V> {
    capacity: usize,
    data: RefCell<HashMap<K, V>>,
    order: RefCell<VecDeque<K>>,  // Most recent at back
}

impl<K, V> LRUCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be greater than 0");
        // TODO: Initialize with given capacity
        todo!()
    }

    fn get(&self, key: &K) -> Option<V> {
        let data = self.data.borrow();
        if let Some(value) = data.get(key) {
            // TODO: Update access order - move key to back of VecDeque
            // Hint: First remove the key from its current position,
            // then push it to the back
            drop(data);  // Release borrow before mutating order
            let mut order = self.order.borrow_mut();
            // ... your code here ...

            // Return cloned value
            todo!()
        } else {
            None
        }
    }

    fn put(&self, key: K, value: V) {
        let mut data = self.data.borrow_mut();

        // Case 1: Key already exists - update value and move to back
        if data.contains_key(&key) {
            // TODO: Update value in HashMap
            // TODO: Move key to back in order VecDeque
            todo!()
        }
        // Case 2: At capacity - evict LRU item first
        else if data.len() >= self.capacity {
            // TODO: Remove front item from order (least recently used)
            // TODO: Remove that key from HashMap
            // TODO: Insert new key-value
            // TODO: Add new key to back of order
            todo!()
        }
        // Case 3: Under capacity - just insert
        else {
            // TODO: Insert new key-value
            // TODO: Add key to back of order
            todo!()
        }
    }

    fn len(&self) -> usize {
        self.data.borrow().len()
    }
}
```

**Checkpoint Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_basic() {
        let cache = LRUCache::new(2);
        cache.put("a", 1);
        cache.put("b", 2);

        assert_eq!(cache.get(&"a"), Some(1));
        assert_eq!(cache.get(&"b"), Some(2));
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_lru_eviction() {
        let cache = LRUCache::new(2);
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);  // Should evict "a"

        assert_eq!(cache.get(&"a"), None);  // "a" was evicted
        assert_eq!(cache.get(&"b"), Some(2));
        assert_eq!(cache.get(&"c"), Some(3));
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_lru_access_order() {
        let cache = LRUCache::new(2);
        cache.put("a", 1);
        cache.put("b", 2);

        // Access "a" to make it more recent
        assert_eq!(cache.get(&"a"), Some(1));

        // Insert "c" - should evict "b" (now least recent)
        cache.put("c", 3);

        assert_eq!(cache.get(&"a"), Some(1));
        assert_eq!(cache.get(&"b"), None);  // "b" was evicted
        assert_eq!(cache.get(&"c"), Some(3));
    }

    #[test]
    fn test_update_existing() {
        let cache = LRUCache::new(2);
        cache.put("a", 1);
        cache.put("a", 10);  // Update

        assert_eq!(cache.get(&"a"), Some(10));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_capacity_one() {
        let cache = LRUCache::new(1);
        cache.put("a", 1);
        cache.put("b", 2);

        assert_eq!(cache.get(&"a"), None);
        assert_eq!(cache.get(&"b"), Some(2));
    }
}
```

**Check Your Understanding**:
- Why do we need to `drop(data)` before modifying `order`?
- What happens if we try to hold both borrows simultaneously?
- How does `VecDeque` help us track LRU order?
- Why do we check `contains_key` before checking capacity?

---

### 🔄 Why Milestone 3 Isn't Enough → Moving to Milestone 4

**Limitation**: We have no visibility into cache performance! Without metrics, we can't answer:
- Is the cache effective? (high hit rate = good, low = wasting memory)
- Should we increase capacity? (too many misses)
- Is it worth the memory cost? (hit rate analysis)

**What we're adding**:
- **Hit/Miss tracking** - Measure cache effectiveness
- **Statistics API** - Query performance metrics
- **Persistent metrics** - Stats survive cache clears

**Improvement**:
- **Observability**: Can measure cache effectiveness (hit rate = hits / (hits + misses))
- **Optimization guidance**: Low hit rate → increase capacity or change eviction policy
- **Production monitoring**: Export metrics to Prometheus/Grafana
- **Cost**: Minimal—just two `Cell<usize>` increments per access

**Real-world example**: Redis `INFO stats` command shows hit/miss ratios. A 90% hit rate means the cache is doing its job; 10% means you need more capacity or better eviction.

---

### Milestone 4: Add Statistics Tracking

**Goal**: Integrate the stats tracker from Milestone 1.

**Task**: Modify your `LRUCache` to include a `StatsTracker` field and update `get()` to record hits/misses.

**New method to add**:
```rust
fn stats(&self) -> (usize, usize) {
    self.stats_tracker.get_stats()
}

fn clear(&self) {
    self.data.borrow_mut().clear();
    self.order.borrow_mut().clear();
    // Don't clear stats - they persist across clears
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_stats_tracking() {
    let cache = LRUCache::new(2);
    cache.put("a", 1);
    cache.put("b", 2);

    cache.get(&"a");  // Hit
    cache.get(&"b");  // Hit
    cache.get(&"c");  // Miss

    let (hits, misses) = cache.stats();
    assert_eq!(hits, 2);
    assert_eq!(misses, 1);
}

#[test]
fn test_clear() {
    let cache = LRUCache::new(2);
    cache.put("a", 1);
    cache.get(&"a");
    cache.get(&"b");  // miss

    cache.clear();
    assert_eq!(cache.len(), 0);

    let (hits, misses) = cache.stats();
    assert_eq!(hits, 1);  // Stats persist
    assert_eq!(misses, 1);
}
```

**Check Your Understanding**:
- Why don't we clear stats when clearing the cache?
- Could we use `Cell<(usize, usize)>` instead of separate `Cell` fields?

---

### 🔄 Why Milestone 4 Isn't Enough → Moving to Milestone 5

**Critical Limitation**: `RefCell` is **NOT thread-safe**! If two threads access it simultaneously, your program exhibits **undefined behavior** (data races, memory corruption).

**Why we need thread safety**:
- Web servers handle concurrent requests across multiple threads
- Game engines run rendering, physics, and AI on different threads
- Microservices need to share caches across async tasks

**What we're adding**:
- **`Mutex<T>`** instead of `RefCell<T>` - OS-level locking for thread safety
- **`Arc<T>`** - Atomic reference counting for sharing across threads
- **`AtomicUsize`** for stats - Thread-safe counters

**Performance Changes**:
- **Speed**: `Mutex` is ~50-100x slower than `RefCell` (10-20ns vs 0.2ns)
- **Why**: System calls, kernel context switches, CPU cache invalidation
- **Parallelism**: Multiple threads can now safely access cache (but serialized by lock)
- **Contention**: Under high concurrent load, threads wait for locks (reduced throughput)

**Real numbers**: Single-threaded `RefCell` cache: 50M ops/sec. Multi-threaded `Mutex` cache with 4 threads: 5M ops/sec per thread = 20M ops/sec total (worse due to contention).

**When it's worth it**: When you have concurrent access. Single-threaded? Stick with `RefCell`.

---

### Milestone 5: Thread-Safe Version with Mutex

**Goal**: Create a thread-safe version using `Mutex` instead of `RefCell`.

**Starter Code**:
```rust
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, VecDeque};

struct ThreadSafeLRUCache<K, V> {
    capacity: usize,
    inner: Mutex<CacheInner<K, V>>,
    // Stats need atomic operations or separate mutex
    stats: StatsTracker,  // From Milestone 1 - uses Cell (NOT thread-safe!)
}

struct CacheInner<K, V> {
    data: HashMap<K, V>,
    order: VecDeque<K>,
}

// TODO: Implement similar methods but using .lock().unwrap() instead of .borrow()
```

**Hints**:
- Use `self.inner.lock().unwrap()` to get a `MutexGuard`
- The guard automatically releases when dropped
- For thread-safety, `StatsTracker` needs `AtomicUsize` instead of `Cell<usize>`

**Checkpoint Tests**:
```rust
#[test]
fn test_thread_safe_basic() {
    let cache = Arc::new(ThreadSafeLRUCache::new(10));

    let cache_clone = Arc::clone(&cache);
    let handle = std::thread::spawn(move || {
        cache_clone.put("thread_key", 42);
    });

    handle.join().unwrap();
    assert_eq!(cache.get(&"thread_key"), Some(42));
}

#[test]
fn test_concurrent_access() {
    let cache = Arc::new(ThreadSafeLRUCache::new(100));
    let mut handles = vec![];

    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = std::thread::spawn(move || {
            for j in 0..10 {
                cache_clone.put(i * 10 + j, i * 100 + j);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(cache.len(), 100);
}
```

**Check Your Understanding**:
- What's the difference between `Mutex` and `RefCell`?
- Why do we need `Arc` to share the cache between threads?
- What happens if a thread panics while holding the lock?
- Why can't we use `Cell<usize>` for stats in the thread-safe version?

---

### 🔄 Why Milestone 5 Isn't Enough → Moving to Milestone 6

**Limitation**: `Mutex` allows only **one** thread at a time—even for reads! This serializes all cache access, wasting CPU cores.

**The problem with Mutex for caches**:
- Read-heavy workloads (common in caches: 90% reads, 10% writes)
- With `Mutex`: 4 threads doing reads wait in line, using only 25% of CPU capacity
- Ideal: Allow concurrent reads, exclusive writes

**What we're adding**:
- **`RwLock<T>`** - Multiple readers OR one writer
- **Read locks**: Multiple threads can read simultaneously
- **Write locks**: Exclusive access (like Mutex)

**Performance Improvement**:
- **Concurrent reads**: 4 threads reading simultaneously: 4x throughput on reads
- **Read-heavy workloads**: 90% reads → ~3.5x overall throughput improvement
- **Write penalty**: `RwLock` write is slightly slower than `Mutex` (more complex lock management)

**The LRU dilemma**:
- Problem: `get()` needs to update access order (move key to back) = **write operation**
- Can't use read lock for `get()` → must use write lock
- This negates RwLock's benefit for caches!

**Solutions**:
1. **Accept write-on-read**: Every get() takes write lock (defeats purpose)
2. **Approximate LRU**: Update order only occasionally (trade accuracy for parallelism)
3. **Two-lock design**: Separate RwLock for data + Mutex for order (complex)
4. **Lock-free LRU**: Use atomics (very complex, covered in advanced courses)

**Performance Comparison**:
- `Mutex` cache: 20M ops/sec with 4 threads (all operations serialized)
- `RwLock` cache (write-on-read): ~20M ops/sec (no benefit, LRU needs writes)
- `RwLock` cache (approximate LRU): ~60M ops/sec (3x faster, but less accurate eviction)

**Key Insight**: **Algorithm choice affects concurrency potential**. True LRU requires write-on-read, limiting parallelism. Alternative algorithms (LRU-K, segmented LRU) offer better concurrency.

---

### Milestone 6 (Advanced): Use RwLock for Better Concurrency

**Goal**: Replace `Mutex` with `RwLock` to allow multiple concurrent reads.

**Key Changes**:
```rust
use std::sync::RwLock;

struct ThreadSafeLRUCache<K, V> {
    capacity: usize,
    inner: RwLock<CacheInner<K, V>>,
    stats: AtomicStats,
}

// In get():
let data = self.inner.read().unwrap();
// Problem: We need write access to update order!
// Solution approaches:
// 1. Use separate RwLock for data and order
// 2. Upgrade read to write when needed (requires releasing read lock first)
// 3. Accept that get() needs write access in LRU (write-on-read)
```

**Challenge**: LRU caches have "write-on-read" behavior because accessing an item updates its position. Think about the trade-offs:
- Should `get()` take a write lock every time?
- Or use a more complex two-lock design?
- What are the performance implications?

---

### Complete Project Summary

**What You Built**:
1. Stats tracker with `Cell` for interior mutability
2. Simple cache with `RefCell<HashMap>`
3. LRU eviction logic with `VecDeque`
4. Thread-safe version with `Mutex`
5. (Optional) `RwLock` optimization with trade-offs

**Key Concepts Practiced**:
- Interior mutability: `Cell`, `RefCell`, `Mutex`, `RwLock`
- Borrow scope management
- Thread safety with `Arc`
- Performance trade-offs between different synchronization primitives

---
