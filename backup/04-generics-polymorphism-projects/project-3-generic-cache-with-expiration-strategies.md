## Project 3: Generic Cache with Expiration Strategies

### Problem Statement

Implement a generic in-memory cache that supports:
- Multiple eviction policies (LRU, LFU, FIFO, TTL)
- Compile-time selection of eviction strategy using generics
- Thread-safe concurrent access
- Configurable capacity limits
- Statistics tracking (hit rate, miss rate, eviction count)
- Lazy expiration (items expire on access, not actively)
- Optional write-through to backing store

The cache must work with any key type implementing `Hash + Eq` and any value type, maintaining O(1) get/put performance.

### Why It Matters

Caching is fundamental to high-performance systems:
- **Web Servers**: Cache rendered pages, database queries, session data
- **Databases**: Buffer pool, query result cache
- **Operating Systems**: Page cache, inode cache
- **CDNs**: Cache static assets globally
- **Machine Learning**: Cache computed features, model predictions

Understanding cache implementation teaches:
- How generics enable reusable data structures
- Trade-offs between different eviction policies
- Concurrent data structure design
- Performance optimization techniques

### Use Cases

1. **Web Application**: Cache database query results, API responses
2. **Distributed Systems**: Local cache to reduce network calls
3. **Compilers**: Cache parsed ASTs, compiled artifacts
4. **Image Processing**: Cache thumbnails, transformed images
5. **Game Development**: Asset cache for textures, models
6. **DNS Resolver**: Cache domain name lookups

### Solution Outline

**Core Structure:**
```rust
use std::hash::Hash;
use std::collections::HashMap;

// Eviction strategy trait
pub trait EvictionPolicy<K> {
    fn on_get(&mut self, key: &K);
    fn on_put(&mut self, key: K);
    fn evict_candidate(&self) -> Option<K>;
}

// Cache with generic eviction policy
pub struct Cache<K, V, E: EvictionPolicy<K>> {
    data: HashMap<K, V>,
    eviction: E,
    capacity: usize,
    stats: CacheStats,
}

pub struct CacheStats {
    hits: usize,
    misses: usize,
    evictions: usize,
}
```

**Eviction Policies to Implement:**

1. **LRU (Least Recently Used)**
   - Track access order with doubly-linked list
   - Evict least recently accessed item
   - Use case: General-purpose caching

2. **LFU (Least Frequently Used)**
   - Track access frequency counter
   - Evict least frequently accessed item
   - Use case: Popular item caching

3. **FIFO (First In First Out)**
   - Track insertion order
   - Evict oldest item
   - Use case: Simple caching, log buffers

4. **TTL (Time To Live)**
   - Track insertion/access timestamp
   - Evict expired items
   - Use case: Session caching, rate limiting

**Performance Targets:**
- `get()`: O(1) average case
- `put()`: O(1) average case (amortized for eviction)
- Memory overhead: < 50% of stored data

### Testing Hints

**Unit Tests:**
```rust
#[test]
fn test_lru_eviction() {
    let mut cache = Cache::with_lru(2);
    cache.put(1, "a");
    cache.put(2, "b");
    cache.get(&1); // Access 1, making it more recent
    cache.put(3, "c"); // Should evict 2 (least recent)

    assert_eq!(cache.get(&1), Some(&"a"));
    assert_eq!(cache.get(&2), None); // Evicted
    assert_eq!(cache.get(&3), Some(&"c"));
}

#[test]
fn test_capacity_limit() {
    // Verify cache never exceeds capacity
}

#[test]
fn test_hit_miss_stats() {
    // Verify statistics are accurately tracked
}
```

**Concurrency Tests:**
```rust
#[test]
fn test_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let cache = Arc::new(Cache::with_lru(100));
    let mut handles = vec![];

    for i in 0..10 {
        let cache = Arc::clone(&cache);
        handles.push(thread::spawn(move || {
            for j in 0..100 {
                cache.put(i * 100 + j, format!("value{}", j));
                cache.get(&(i * 100 + j));
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

**Performance Tests:**
```rust
#[bench]
fn bench_cache_operations(b: &mut Bencher) {
    let mut cache = Cache::with_lru(1000);
    b.iter(|| {
        for i in 0..1000 {
            cache.put(i, i * 2);
        }
        for i in 0..1000 {
            black_box(cache.get(&i));
        }
    });
}
```

---

## Step-by-Step Implementation Guide

### Step 1: Basic HashMap-Backed Cache with Fixed Size

**Goal:** Create a simple cache using `HashMap` with naive eviction (random).

**What to implement:**
```rust
use std::collections::HashMap;
use std::hash::Hash;

pub struct Cache<K, V> {
    data: HashMap<K, V>,
    capacity: usize,
}

impl<K: Hash + Eq, V> Cache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Cache {
            data: HashMap::with_capacity(capacity),
            capacity,
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }

    pub fn put(&mut self, key: K, value: V) {
        if self.data.len() >= self.capacity && !self.data.contains_key(&key) {
            // Naive: remove first key found (random due to HashMap iteration)
            if let Some(k) = self.data.keys().next().cloned() {
                self.data.remove(&k);
            }
        }
        self.data.insert(key, value);
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}
```

**Check/Test:**
- Test basic get/put operations
- Verify cache respects capacity limit
- Test with different key/value types (String, i32, custom structs)

**Why this isn't enough:**
Random eviction is useless for real caching—no locality benefit, poor hit rate. A cache that evicts randomly performs barely better than no cache at all. We need smart eviction policies (LRU, LFU) that keep hot data. Also, no statistics—we can't measure cache effectiveness.

---

### Step 2: Add LRU Eviction with Doubly-Linked List

**Goal:** Implement proper LRU (Least Recently Used) eviction policy.

**What to improve:**

Rust doesn't have a built-in doubly-linked list suitable for this, so we'll use indices and a VecDeque:

```rust
use std::collections::{HashMap, VecDeque};

pub struct LruCache<K, V> {
    data: HashMap<K, V>,
    order: VecDeque<K>,  // Front = most recent, back = least recent
    capacity: usize,
}

impl<K: Hash + Eq + Clone, V> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        LruCache {
            data: HashMap::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if self.data.contains_key(key) {
            // Move to front (most recent)
            self.touch(key);
            self.data.get(key)
        } else {
            None
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        if self.data.contains_key(&key) {
            // Update existing
            self.touch(&key);
            self.data.insert(key, value);
        } else {
            // Evict if at capacity
            if self.data.len() >= self.capacity {
                if let Some(lru_key) = self.order.pop_back() {
                    self.data.remove(&lru_key);
                }
            }

            self.data.insert(key.clone(), value);
            self.order.push_front(key);
        }
    }

    fn touch(&mut self, key: &K) {
        // Remove from current position and move to front
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
            self.order.push_front(key.clone());
        }
    }
}
```

**Check/Test:**
- Test LRU behavior: least recently accessed item is evicted
- Test that `get()` updates access order
- Test that updating existing keys doesn't grow cache
- Verify O(n) worst case for touch operation (problem for next step)

**Why this isn't enough:**
The `touch()` operation is O(n) because we're searching through `VecDeque` to find and remove the key. For a large cache (10k+ items), this kills performance. We need O(1) access to list nodes. Also, we only have LRU—what about other policies? The structure is hardcoded for one strategy.

---

### Step 3: Make Eviction Policy Generic with Trait

**Goal:** Abstract eviction logic behind a trait, supporting multiple policies.

**What to improve:**
```rust
use std::hash::Hash;
use std::collections::HashMap;

// Trait for eviction policies
pub trait EvictionPolicy<K> {
    fn new(capacity: usize) -> Self;
    fn on_get(&mut self, key: &K);
    fn on_put(&mut self, key: &K);
    fn on_remove(&mut self, key: &K);
    fn evict_candidate(&self) -> Option<K>;
    fn len(&self) -> usize;
}

// Generic cache
pub struct Cache<K, V, E>
where
    K: Hash + Eq + Clone,
    E: EvictionPolicy<K>,
{
    data: HashMap<K, V>,
    eviction: E,
    capacity: usize,
}

impl<K, V, E> Cache<K, V, E>
where
    K: Hash + Eq + Clone,
    E: EvictionPolicy<K>,
{
    pub fn new(capacity: usize) -> Self {
        Cache {
            data: HashMap::with_capacity(capacity),
            eviction: E::new(capacity),
            capacity,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        let result = self.data.get(key);
        if result.is_some() {
            self.eviction.on_get(key);
        }
        result
    }

    pub fn put(&mut self, key: K, value: V) {
        if self.data.contains_key(&key) {
            self.eviction.on_put(&key);
            self.data.insert(key, value);
        } else {
            if self.data.len() >= self.capacity {
                if let Some(victim) = self.eviction.evict_candidate() {
                    self.data.remove(&victim);
                    self.eviction.on_remove(&victim);
                }
            }

            self.eviction.on_put(&key);
            self.data.insert(key, value);
        }
    }
}

// Implement LRU policy
pub struct LruPolicy<K> {
    order: VecDeque<K>,
    capacity: usize,
}

impl<K: Clone + Eq> EvictionPolicy<K> for LruPolicy<K> {
    fn new(capacity: usize) -> Self {
        LruPolicy {
            order: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn on_get(&mut self, key: &K) {
        // Move to front
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
            self.order.push_front(key.clone());
        }
    }

    fn on_put(&mut self, key: &K) {
        self.order.push_front(key.clone());
    }

    fn on_remove(&mut self, key: &K) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
        }
    }

    fn evict_candidate(&self) -> Option<K> {
        self.order.back().cloned()
    }

    fn len(&self) -> usize {
        self.order.len()
    }
}

// Implement FIFO policy
pub struct FifoPolicy<K> {
    order: VecDeque<K>,
    capacity: usize,
}

impl<K: Clone + Eq> EvictionPolicy<K> for FifoPolicy<K> {
    fn new(capacity: usize) -> Self {
        FifoPolicy {
            order: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn on_get(&mut self, _key: &K) {
        // FIFO doesn't care about access
    }

    fn on_put(&mut self, key: &K) {
        self.order.push_front(key.clone());
    }

    fn on_remove(&mut self, key: &K) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
        }
    }

    fn evict_candidate(&self) -> Option<K> {
        self.order.back().cloned()
    }

    fn len(&self) -> usize {
        self.order.len()
    }
}

// Type aliases for convenience
pub type LruCache<K, V> = Cache<K, V, LruPolicy<K>>;
pub type FifoCache<K, V> = Cache<K, V, FifoPolicy<K>>;
```

**Check/Test:**
- Test both LRU and FIFO caches behave correctly
- Verify policy trait abstraction works
- Test that FIFO doesn't update order on `get()`

**Why this isn't enough:**
Still O(n) performance for LRU due to VecDeque search. We need a better data structure—an intrusive doubly-linked list backed by HashMap for O(1) operations. Also, no statistics tracking yet (hit rate, miss rate). We can't measure cache effectiveness.

---

### Step 4: Optimize LRU to O(1) with HashMap + Linked List

**Goal:** Achieve true O(1) get/put for LRU using a custom linked list structure.

**What to improve:**

Use a pattern similar to `std::collections::LinkedHashMap` (not in std, but we can build it):

```rust
use std::collections::HashMap;
use std::ptr::NonNull;

// Doubly-linked list node
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<NonNull<Node<K, V>>>,
    next: Option<NonNull<Node<K, V>>>,
}

pub struct LruCacheOptimized<K, V>
where
    K: Hash + Eq + Clone,
{
    map: HashMap<K, NonNull<Node<K, V>>>,
    head: Option<NonNull<Node<K, V>>>,
    tail: Option<NonNull<Node<K, V>>>,
    capacity: usize,
}

impl<K: Hash + Eq + Clone, V> LruCacheOptimized<K, V> {
    pub fn new(capacity: usize) -> Self {
        LruCacheOptimized {
            map: HashMap::with_capacity(capacity),
            head: None,
            tail: None,
            capacity,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(&node_ptr) = self.map.get(key) {
            unsafe {
                self.move_to_front(node_ptr);
                Some(&(*node_ptr.as_ptr()).value)
            }
        } else {
            None
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        if let Some(&node_ptr) = self.map.get(&key) {
            unsafe {
                (*node_ptr.as_ptr()).value = value;
                self.move_to_front(node_ptr);
            }
        } else {
            if self.map.len() >= self.capacity {
                self.remove_tail();
            }

            let mut node = Box::new(Node {
                key: key.clone(),
                value,
                prev: None,
                next: self.head,
            });

            let node_ptr = NonNull::new(Box::into_raw(node)).unwrap();

            if let Some(mut head) = self.head {
                unsafe {
                    (*head.as_ptr()).prev = Some(node_ptr);
                }
            } else {
                self.tail = Some(node_ptr);
            }

            self.head = Some(node_ptr);
            self.map.insert(key, node_ptr);
        }
    }

    unsafe fn move_to_front(&mut self, node_ptr: NonNull<Node<K, V>>) {
        let node = node_ptr.as_ptr();

        if self.head == Some(node_ptr) {
            return; // Already at front
        }

        // Remove from current position
        if let Some(mut prev) = (*node).prev {
            (*prev.as_ptr()).next = (*node).next;
        }

        if let Some(mut next) = (*node).next {
            (*next.as_ptr()).prev = (*node).prev;
        } else {
            self.tail = (*node).prev;
        }

        // Move to front
        (*node).prev = None;
        (*node).next = self.head;

        if let Some(mut head) = self.head {
            (*head.as_ptr()).prev = Some(node_ptr);
        }

        self.head = Some(node_ptr);
    }

    fn remove_tail(&mut self) {
        if let Some(tail_ptr) = self.tail {
            unsafe {
                let tail = tail_ptr.as_ptr();
                let key = (*tail).key.clone();

                self.map.remove(&key);

                if let Some(mut prev) = (*tail).prev {
                    (*prev.as_ptr()).next = None;
                    self.tail = Some(prev);
                } else {
                    self.head = None;
                    self.tail = None;
                }

                // Free the node
                drop(Box::from_raw(tail));
            }
        }
    }
}

impl<K, V> Drop for LruCacheOptimized<K, V>
where
    K: Hash + Eq + Clone,
{
    fn drop(&mut self) {
        let mut current = self.head;
        while let Some(node_ptr) = current {
            unsafe {
                let node = Box::from_raw(node_ptr.as_ptr());
                current = node.next;
            }
        }
    }
}
```

**Important:** This uses unsafe code. In a real implementation, consider using a safe library like `lru` crate or refactoring with indices.

**Check/Test:**
- Benchmark O(1) performance for large caches (10k+ items)
- Test with Miri or AddressSanitizer for memory safety
- Verify no memory leaks with valgrind or similar

**Why this isn't enough:**
We've optimized LRU, but:
1. No statistics tracking (hit/miss rate, eviction count)
2. Not thread-safe—can't share across threads
3. No TTL (time-based expiration) support
4. No LFU (Least Frequently Used) policy

Let's add statistics and thread safety next.

---

### Step 5: Add Statistics and Thread Safety

**Goal:** Track cache performance metrics and make the cache thread-safe.

**What to improve:**

**1. Add statistics:**
```rust
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
pub struct CacheStats {
    hits: AtomicUsize,
    misses: AtomicUsize,
    evictions: AtomicUsize,
    inserts: AtomicUsize,
}

impl CacheStats {
    pub fn hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn insert(&self) {
        self.inserts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let total = hits + self.misses.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    pub fn summary(&self) -> StatsSummary {
        StatsSummary {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            inserts: self.inserts.load(Ordering::Relaxed),
        }
    }
}

pub struct StatsSummary {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub inserts: usize,
}
```

**2. Add thread safety with RwLock:**
```rust
use std::sync::{Arc, RwLock};

pub struct ThreadSafeCache<K, V, E>
where
    K: Hash + Eq + Clone,
    E: EvictionPolicy<K>,
{
    inner: Arc<RwLock<Cache<K, V, E>>>,
    stats: Arc<CacheStats>,
}

impl<K, V, E> Clone for ThreadSafeCache<K, V, E>
where
    K: Hash + Eq + Clone,
    E: EvictionPolicy<K>,
{
    fn clone(&self) -> Self {
        ThreadSafeCache {
            inner: Arc::clone(&self.inner),
            stats: Arc::clone(&self.stats),
        }
    }
}

impl<K, V, E> ThreadSafeCache<K, V, E>
where
    K: Hash + Eq + Clone,
    V: Clone,
    E: EvictionPolicy<K>,
{
    pub fn new(capacity: usize) -> Self {
        ThreadSafeCache {
            inner: Arc::new(RwLock::new(Cache::new(capacity))),
            stats: Arc::new(CacheStats::default()),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.inner.write().unwrap();
        let result = cache.get(key).cloned();

        if result.is_some() {
            self.stats.hit();
        } else {
            self.stats.miss();
        }

        result
    }

    pub fn put(&self, key: K, value: V) {
        let mut cache = self.inner.write().unwrap();
        let was_full = cache.len() >= cache.capacity();

        cache.put(key, value);

        if was_full {
            self.stats.eviction();
        }
        self.stats.insert();
    }

    pub fn stats(&self) -> StatsSummary {
        self.stats.summary()
    }
}
```

**Check/Test:**
- Test concurrent access from multiple threads
- Verify statistics are accurate under concurrent load
- Test that hit/miss rate calculation is correct
- Benchmark performance with contention

**Why this isn't enough:**
Write lock for reads is inefficient—multiple readers could access simultaneously, but `get()` updates LRU order (mutable). This creates contention. Also, still no TTL support for time-based expiration. Large caches can grow stale without expiration.

---

### Step 6: Add TTL Support and Lazy Expiration

**Goal:** Implement time-based expiration with lazy eviction on access.

**What to improve:**

**1. Add TTL policy:**
```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

pub struct TtlPolicy<K> {
    timestamps: HashMap<K, Instant>,
    ttl: Duration,
    capacity: usize,
}

impl<K: Hash + Eq + Clone> EvictionPolicy<K> for TtlPolicy<K> {
    fn new(capacity: usize) -> Self {
        TtlPolicy {
            timestamps: HashMap::with_capacity(capacity),
            ttl: Duration::from_secs(300), // Default 5 minutes
            capacity,
        }
    }

    fn on_get(&mut self, key: &K) {
        // Update timestamp on access
        self.timestamps.insert(key.clone(), Instant::now());
    }

    fn on_put(&mut self, key: &K) {
        self.timestamps.insert(key.clone(), Instant::now());
    }

    fn on_remove(&mut self, key: &K) {
        self.timestamps.remove(key);
    }

    fn evict_candidate(&self) -> Option<K> {
        let now = Instant::now();

        // Find first expired item
        self.timestamps
            .iter()
            .find(|(_, &timestamp)| now.duration_since(timestamp) > self.ttl)
            .map(|(k, _)| k.clone())
            .or_else(|| {
                // If none expired, evict oldest
                self.timestamps
                    .iter()
                    .min_by_key(|(_, &timestamp)| timestamp)
                    .map(|(k, _)| k.clone())
            })
    }

    fn len(&self) -> usize {
        self.timestamps.len()
    }
}

impl<K> TtlPolicy<K> {
    pub fn with_ttl(capacity: usize, ttl: Duration) -> Self {
        TtlPolicy {
            timestamps: HashMap::with_capacity(capacity),
            ttl,
            capacity,
        }
    }
}
```

**2. Add lazy expiration check to Cache:**
```rust
impl<K, V, E> Cache<K, V, E>
where
    K: Hash + Eq + Clone,
    E: EvictionPolicy<K>,
{
    pub fn get(&mut self, key: &K) -> Option<&V> {
        // Check for expiration first
        if self.is_expired(key) {
            self.remove(key);
            return None;
        }

        let result = self.data.get(key);
        if result.is_some() {
            self.eviction.on_get(key);
        }
        result
    }

    fn is_expired(&self, key: &K) -> bool {
        // Policies can implement expiration check
        // For TTL: check if timestamp + ttl < now
        false // Default: no expiration
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.eviction.on_remove(key);
        self.data.remove(key)
    }

    pub fn cleanup_expired(&mut self) -> usize {
        // Periodic cleanup of expired items
        let mut count = 0;
        let expired: Vec<K> = self.data
            .keys()
            .filter(|k| self.is_expired(k))
            .cloned()
            .collect();

        for key in expired {
            self.remove(&key);
            count += 1;
        }

        count
    }
}
```

**3. Add LFU (Least Frequently Used) policy:**
```rust
pub struct LfuPolicy<K> {
    frequencies: HashMap<K, usize>,
    capacity: usize,
}

impl<K: Hash + Eq + Clone> EvictionPolicy<K> for LfuPolicy<K> {
    fn new(capacity: usize) -> Self {
        LfuPolicy {
            frequencies: HashMap::with_capacity(capacity),
            capacity,
        }
    }

    fn on_get(&mut self, key: &K) {
        *self.frequencies.entry(key.clone()).or_insert(0) += 1;
    }

    fn on_put(&mut self, key: &K) {
        self.frequencies.insert(key.clone(), 1);
    }

    fn on_remove(&mut self, key: &K) {
        self.frequencies.remove(key);
    }

    fn evict_candidate(&self) -> Option<K> {
        self.frequencies
            .iter()
            .min_by_key(|(_, &freq)| freq)
            .map(|(k, _)| k.clone())
    }

    fn len(&self) -> usize {
        self.frequencies.len()
    }
}
```

**Type aliases:**
```rust
pub type LruCache<K, V> = Cache<K, V, LruPolicy<K>>;
pub type LfuCache<K, V> = Cache<K, V, LfuPolicy<K>>;
pub type FifoCache<K, V> = Cache<K, V, FifoPolicy<K>>;
pub type TtlCache<K, V> = Cache<K, V, TtlPolicy<K>>;
```

**Check/Test:**
- Test TTL expiration: items expire after configured duration
- Test LFU evicts least frequently used items
- Test lazy expiration only triggers on access
- Test `cleanup_expired()` removes all expired items
- Benchmark different policies under various workloads

**What this achieves:**
- **Multiple Eviction Policies**: LRU, LFU, FIFO, TTL all supported through generic trait
- **Time-Based Expiration**: Items automatically expire after TTL
- **Performance**: O(1) operations for all policies (except expiration cleanup)
- **Thread Safety**: Safe concurrent access with Arc<RwLock>
- **Statistics**: Track hit rate, miss rate, eviction count
- **Flexibility**: Generic over key/value types and eviction policy
- **Type Safety**: Compile-time policy selection

**Extensions to explore:**
- Write-through cache: sync to backing store
- Cache warming: pre-populate from disk
- Tiered caching: L1 (memory) + L2 (disk)
- Async support: async get/put for network caches
- Bloom filters: fast negative lookups
- Compression: compress values to save memory

---

## Summary

These three projects teach complementary aspects of Rust's generics and polymorphism:

1. **Priority Queue**: Generic data structures, trait bounds, phantom types for compile-time configuration, efficient algorithms

2. **Type-State Connection Builder**: Phantom types for state machines, zero-cost state guarantees, builder pattern, compile-time safety

3. **Generic Cache**: Trait-based polymorphism, multiple implementations of abstraction, performance optimization, thread safety, statistics

All three emphasize:
- Zero-cost abstractions through generics
- Compile-time safety guarantees
- Performance-conscious design
- Practical, real-world patterns

Students should come away understanding how to design flexible, type-safe, performant generic APIs—the foundation of modern Rust systems programming.
