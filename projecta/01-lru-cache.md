## Project 1: Thread-Safe LRU Cache with Interior Mutability

### Problem Statement

Implement a Least Recently Used (LRU) cache that tracks the most recently accessed items and evicts the least recently used item when capacity is reached. You'll start with a simple single-threaded version, then progress to thread-safe variants.


## Understanding Caches and the LRU Algorithm

### What is a Cache?

A **cache** is a high-speed data storage layer that stores a subset of data, enabling faster retrieval than accessing the data from its primary storage location. Caches exploit the principle of **locality**: programs tend to access the same data repeatedly (temporal locality) and data stored near recently accessed data (spatial locality).

**The fundamental trade-off**:
- **Speed vs Capacity**: Caches are fast but small; primary storage is slow but large
- **Memory hierarchy**: CPU registers (fastest, ~1 cycle) → L1 cache (~4 cycles) → L2 cache (~12 cycles) → L3 cache (~40 cycles) → RAM (~100 cycles) → Disk (~10,000,000 cycles)

**Real-world example**:
```rust
// Without cache: Every request hits database (100ms each)
fn get_user_profile(id: u64) -> User {
    database.query("SELECT * FROM users WHERE id = ?", id)  // 100ms
}
// 1000 requests = 100 seconds total

// With cache: First request hits DB, subsequent hits cache (0.1ms)
fn get_user_profile_cached(id: u64) -> User {
    if let Some(user) = cache.get(id) {
        return user;  // 0.1ms - 1000x faster!
    }
    let user = database.query("SELECT * FROM users WHERE id = ?", id);
    cache.put(id, user.clone());
    user
}
// 1000 requests for same user = 100ms + 999 × 0.1ms = 200ms total
// Speedup: 500x faster
```

---

### Cache Fundamentals

#### 1. Cache Hit vs Cache Miss

**Cache Hit**: Requested data is found in the cache
- **Benefit**: Fast access (microseconds)
- **Example**: Browser loading an image already in cache

**Cache Miss**: Requested data is NOT in cache, must fetch from slower storage
- **Cost**: Slow access (milliseconds to seconds)
- **Example**: First time loading a webpage image

**Hit Rate**: The percentage of requests served from cache
```
Hit Rate = Hits / (Hits + Misses)

Example: 900 hits, 100 misses → Hit Rate = 900/1000 = 90%
```

**Why hit rate matters**:
- **90% hit rate**: 90% of requests take 0.1ms, 10% take 100ms → Average: 10.09ms
- **99% hit rate**: 99% take 0.1ms, 1% take 100ms → Average: 1.099ms
- **9x improvement** from 90% to 99% hit rate!

#### 2. Cache Capacity Limits

Caches must have **bounded size** to avoid consuming all memory. This creates the **cache eviction problem**: when the cache is full and a new item needs to be stored, which existing item should be removed?

**Without eviction** (unbounded cache):
```rust
// Memory grows without bound - will eventually crash!
let cache = HashMap::new();
loop {
    let data = fetch_from_api();
    cache.insert(data.id, data);  // Grows forever
}
// After 1M entries of 1KB each = 1GB memory used
// After 10M entries = 10GB memory used → OOM crash
```

**With eviction** (bounded cache):
```rust
let cache = LRUCache::new(1000);  // Maximum 1000 entries
loop {
    let data = fetch_from_api();
    cache.put(data.id, data);
    // When cache reaches 1000 entries, least recently used item is removed
}
// Memory: Always ≤ 1000 entries (predictable, safe)
```

---

### Cache Eviction Policies

When a cache is full, an **eviction policy** determines which item to remove. Different policies optimize for different access patterns.

#### Common Eviction Policies

| Policy | Evicts | Best For | Worst For |
|--------|--------|----------|-----------|
| **FIFO** (First-In-First-Out) | Oldest inserted item | Simple streaming data | Items accessed repeatedly |
| **LRU** (Least Recently Used) | Least recently accessed item | Temporal locality (repeated access) | Sequential scans |
| **LFU** (Least Frequently Used) | Least frequently accessed item | Repeated popular items | Changing access patterns |
| **Random** | Random item | Uniform access, low overhead | Predictable patterns |
| **MRU** (Most Recently Used) | Most recently accessed item | Sequential scans | Temporal locality |

**Comparison Example**:

```
Cache capacity: 3 items
Access sequence: A, B, C, D, A, E

┌─────────┬──────────┬────────────┬───────────────┬────────────┐
│ Access  │ FIFO     │ LRU        │ LFU           │ Random     │
├─────────┼──────────┼────────────┼───────────────┼────────────┤
│ A       │ [A]      │ [A]        │ [A:1]         │ [A]        │
│ B       │ [A,B]    │ [A,B]      │ [A:1,B:1]     │ [A,B]      │
│ C       │ [A,B,C]  │ [A,B,C]    │ [A:1,B:1,C:1] │ [A,B,C]    │
│ D       │ [B,C,D]  │ [B,C,D]    │ [B:1,C:1,D:1] │ [A,C,D]    │
│         │ (evict A)│ (evict A)  │ (evict A)     │ (evict B)  │
│ A       │ [B,C,D]  │ [B,D,A]    │ [C:1,D:1,A:1] │ [A,C,D]    │
│         │ MISS     │ (evict C)  │ (evict B)     │ MISS       │
│         │          │ HIT via A  │               │            │
│ E       │ [C,D,E]  │ [D,A,E]    │ [D:1,A:1,E:1] │ [A,E,D]    │
│         │ (evict B)│ (evict B)  │ (evict C)     │ (evict C)  │
└─────────┴──────────┴────────────┴───────────────┴────────────┘

Hit rates for this sequence:
- FIFO: 0% (0 hits, 6 misses)
- LRU: 16.7% (1 hit, 5 misses)  ← Best for this pattern
- LFU: 0% (0 hits, 6 misses)
- Random: 16.7% (1 hit, 5 misses) - depends on random choice
```

---

### Deep Dive: The LRU Algorithm

**LRU (Least Recently Used)** evicts the item that hasn't been accessed for the longest time. It's based on the assumption that recently accessed data is more likely to be accessed again soon.

#### Why LRU Works Well

**Temporal Locality Principle**: If data was accessed recently, it's likely to be accessed again soon.

**Real-world examples**:
1. **Web browser cache**: Recently viewed images/pages are likely to be viewed again
2. **Database query cache**: Same queries run repeatedly (dashboards, APIs)
3. **File system cache**: Editing a file → repeated reads/writes to same blocks
4. **Game asset cache**: Current level assets used repeatedly; old level assets not needed

#### How LRU Tracking Works

**Core idea**: Maintain access order, with most recently used at one end and least recently used at the other.

```
Initial state (capacity: 3):
Cache: []
Order: []

Access A:
Cache: {A: "data_a"}
Order: [A]  ← LRU                                   MRU →

Access B:
Cache: {A: "data_a", B: "data_b"}
Order: [A, B]  ← LRU                                MRU →

Access C:
Cache: {A: "data_a", B: "data_b", C: "data_c"}
Order: [A, B, C]  ← LRU                             MRU →

Access A again (move to most recent):
Cache: {A: "data_a", B: "data_b", C: "data_c"}
Order: [B, C, A]  ← LRU                             MRU →
       └─ Now B is least recently used

Access D (cache full, evict B):
Cache: {A: "data_a", C: "data_c", D: "data_d"}
Order: [C, A, D]  ← LRU                             MRU →
       └─ B was removed (least recently used)
```

#### LRU Operations

**Every cache operation updates the access order**:

1. **get(key)**:
   - If found: Move key to most recent position, return value (HIT)
   - If not found: Return None (MISS)

2. **put(key, value)**:
   - If key exists: Update value, move to most recent
   - If cache full: Remove least recent item, insert new item as most recent
   - If cache not full: Insert new item as most recent

**Time complexity requirements**:
- `get()`: O(1) - Must be fast for cache to be useful
- `put()`: O(1) - Including eviction
- Update order: O(1) - Move item to most recent position

#### Implementation Strategies

**Strategy 1: HashMap + VecDeque** (This project uses this)
```rust
struct LRUCache<K, V> {
    data: HashMap<K, V>,      // Fast lookup: O(1)
    order: VecDeque<K>,       // Track access order
}

// Get operation:
// 1. Lookup in HashMap: O(1)
// 2. Find key in VecDeque: O(n) ← SLOW!
// 3. Remove from current position: O(n)
// 4. Push to back: O(1)
// Total: O(n) - not ideal, but simple
```

**Trade-off**: Simple to implement, but updating order is O(n) because we need to find and remove the key from the VecDeque.

**Strategy 2: HashMap + Doubly-Linked List** (Optimal, used in production)
```rust
struct LRUCache<K, V> {
    data: HashMap<K, *mut Node<K, V>>,  // Value + pointer to node
    order: DoublyLinkedList<K, V>,      // Actual order
}

struct Node<K, V> {
    key: K,
    value: V,
    prev: *mut Node<K, V>,
    next: *mut Node<K, V>,
}

// Get operation:
// 1. Lookup in HashMap: O(1) - gives us pointer to node
// 2. Remove node from list: O(1) - just update pointers
// 3. Insert at tail: O(1)
// Total: O(1) ← OPTIMAL
```

**Trade-off**: O(1) operations, but requires unsafe code for pointer manipulation in Rust.

---

### Real-World Cache Examples

#### 1. CPU Caches (Hardware)

**L1 Cache**: 32-64 KB per core, ~4 CPU cycles latency
**L2 Cache**: 256-512 KB per core, ~12 cycles
**L3 Cache**: 8-32 MB shared, ~40 cycles
**RAM**: Gigabytes, ~100 cycles

```
// Without L1 cache:
for i in 0..1000 {
    sum += array[i];  // Each access: 100 cycles (RAM)
}
// Total: 100,000 cycles

// With L1 cache (after first access):
for i in 0..1000 {
    sum += array[i];  // First: 100 cycles, rest: 4 cycles
}
// Total: 100 + 999×4 = 4,096 cycles (24x faster!)
```

#### 2. Web Browser Cache

Browsers cache images, CSS, JavaScript, and HTML to avoid re-downloading.

```
First visit to website:
- Download: 2MB of assets (images, CSS, JS)
- Time: 2 seconds on 10 Mbps connection

Second visit (with cache):
- Check cache: 50 files, all HITs
- Time: 50ms to verify freshness
- Speedup: 40x faster
```

**Eviction**: LRU-based, typically 50-500 MB capacity

#### 3. Database Query Cache

```rust
// MySQL query cache example
// First execution:
let result = db.query("SELECT * FROM users WHERE age > 18");
// Time: 100ms (disk I/O, query execution)
// Cache: Store query → result mapping

// Second execution (same query):
let result = db.query("SELECT * FROM users WHERE age > 18");
// Time: 0.1ms (cache HIT)
// Speedup: 1000x faster
```

**Eviction**: LRU with automatic invalidation when table is modified

#### 4. CDN (Content Delivery Network)

CDNs cache website content geographically close to users.

```
User in Sydney requests image from US-based server:

Without CDN:
- Round trip: 200ms (speed of light limit!)
- Transfer: 50ms
- Total: 250ms

With CDN (cached in Sydney):
- Round trip: 10ms (local server)
- Transfer: 5ms
- Total: 15ms
- Speedup: 16x faster
```

**Eviction**: Mix of LRU and TTL (Time-To-Live)

#### 5. Operating System Page Cache

OS caches disk blocks in RAM to speed up file access.

```
Reading a 1GB file:

First read (cold cache):
- Disk: 1GB at 100 MB/s = 10 seconds

Second read (warm cache):
- RAM: 1GB at 10 GB/s = 0.1 seconds
- Speedup: 100x faster
```

**Eviction**: LRU-based (Linux uses LRU with active/inactive lists)

---

### Performance Characteristics

#### Memory Usage

```rust
// Memory = (capacity × item_size) + overhead

// Example: LRU cache with capacity 10,000
struct Entry {
    key: String,    // ~24 bytes (avg)
    value: User,    // ~200 bytes
}

// HashMap: ~224 bytes per entry
// VecDeque: ~8 bytes per key (pointer/index)
// Total per entry: ~232 bytes
// Total memory: 10,000 × 232 = 2.32 MB
```

#### Time Complexity

| Operation | HashMap + VecDeque | HashMap + LinkedList |
|-----------|-------------------|----------------------|
| get() | O(n) | O(1) |
| put() | O(n) | O(1) |
| evict() | O(1) | O(1) |

**Why VecDeque is O(n)**:
```rust
// Must find and remove key from middle of VecDeque
order.retain(|k| k != key);  // O(n) - scans entire VecDeque
order.push_back(key);         // O(1)
```

**Why LinkedList is O(1)**:
```rust
// HashMap stores pointer to node, can remove directly
let node = map.get(key);      // O(1)
list.remove(node);            // O(1) - just update pointers
list.push_back(node);         // O(1)
```

#### Cache Hit Rate Impact

**Measurement**: How hit rate affects average latency

```
Cache latency: 0.1ms
Database latency: 100ms

Hit Rate    Avg Latency    Calculation
50%         50.05ms        0.5×0.1 + 0.5×100
70%         30.07ms        0.7×0.1 + 0.3×100
90%         10.09ms        0.9×0.1 + 0.1×100
95%         5.095ms        0.95×0.1 + 0.05×100
99%         1.099ms        0.99×0.1 + 0.01×100

Insight: Going from 90% → 99% hit rate = 9x improvement!
```

---

### When to Use LRU vs Other Policies

#### Use LRU When:

✅ **Access patterns show temporal locality**
- Web sessions (users browse multiple pages)
- Database queries (dashboards run same queries repeatedly)
- File editing (same files accessed multiple times)

✅ **Recent access predicts future access**
- News website (recent articles accessed more)
- E-commerce (viewed products likely to be viewed again)

✅ **Memory is limited**
- Need automatic eviction with bounded size
- Predictable memory usage is critical

#### Avoid LRU When:

❌ **Sequential scans** (each item accessed once)
```rust
// LRU performs poorly here
for i in 0..1_000_000 {
    cache.get(i);  // Each item accessed once, never again
}
// Every access is a MISS, cache constantly evicts
// Better: No cache, or MRU policy
```

❌ **Popular items accessed infrequently**
- Video streaming (popular movies accessed monthly)
- Better: LFU (Least Frequently Used)

❌ **Need time-based expiration**
- Session tokens (expire after 30 minutes)
- Better: TTL (Time-To-Live) cache

---

## Rust Programming Concepts for This Project

This project requires understanding several Rust-specific concepts that enable safe and efficient cache implementation. These concepts address challenges that don't exist in garbage-collected languages.

### Interior Mutability: The Core Challenge

**The Problem**: Rust's borrow checker normally requires `&mut self` to modify data. But caches need to update internal state (access tracking, statistics) during read operations that only have `&self`.

```rust
// This doesn't work - get() only has &self, can't modify!
impl Cache {
    fn get(&self, key: &K) -> Option<V> {
        self.hits += 1;  // ❌ Error: cannot mutate through &self
        // ...
    }
}

// Requiring &mut self doesn't work either - prevents sharing!
impl Cache {
    fn get(&mut self, key: &K) -> Option<V> {
        self.hits += 1;  // ✅ Compiles
        // ...
    }
}

let cache = Cache::new();
let hits = cache.get(&"key1");  // ❌ Error: need mutable borrow
let misses = cache.get(&"key2"); // Can't have multiple mutable refs!
```

**The Solution**: **Interior mutability** - types that provide mutable access through shared references (`&self`).

---

### Cell<T>: Zero-Cost Interior Mutability for Copy Types

**What It Is**: A container that allows mutating the value inside through `&self`, but only for types that implement `Copy` (integers, booleans, small structs).

**How It Works**:
```rust
use std::cell::Cell;

let counter = Cell::new(0);
counter.set(counter.get() + 1);  // Mutate through &self!
println!("{}", counter.get());   // 1
```

**Key Properties**:
- **Zero runtime cost**: Just moves bytes around
- **Only for Copy types**: Can't use with `String`, `Vec`, `HashMap`
- **No borrowing**: Values are copied in/out, never borrowed
- **Not thread-safe**: Only works in single-threaded code

**Why We Use It**: Perfect for counters (hits, misses) - they're just `usize` values.

**Limitations**:
```rust
let cache_data = Cell::new(HashMap::new());
// ❌ Error: HashMap doesn't implement Copy
```

---

### RefCell<T>: Runtime-Checked Interior Mutability

**What It Is**: Like `Cell`, but works with **any type**. Enforces borrow rules at runtime instead of compile time.

**How It Works**:
```rust
use std::cell::RefCell;
use std::collections::HashMap;

let cache = RefCell::new(HashMap::new());

// Borrow for reading
{
    let data = cache.borrow();  // Returns Ref<HashMap>
    println!("{:?}", data.get(&"key"));
}  // Borrow released here

// Borrow for writing
{
    let mut data = cache.borrow_mut();  // Returns RefMut<HashMap>
    data.insert("key", "value");
}  // Borrow released here
```

**Key Properties**:
- **Works with any type**: `HashMap`, `Vec`, custom structs, etc.
- **Runtime borrow checking**: Panics if borrow rules violated
- **Small overhead**: Maintains borrow counters (~2-3 CPU instructions)
- **Not thread-safe**: Panics if used across threads

**Borrow Rules** (checked at runtime):
- **Multiple readers OR one writer**: Can have many `borrow()` or one `borrow_mut()`, not both
- **Borrows must be released**: The `Ref`/`RefMut` guards must drop before next borrow

**Common Pitfalls**:
```rust
let cache = RefCell::new(HashMap::new());
let data = cache.borrow();          // Acquire read borrow
cache.borrow_mut().insert(1, 2);    // ❌ PANIC: already borrowed!
drop(data);                          // Must release first
cache.borrow_mut().insert(1, 2);    // ✅ Now it works
```

**Why We Use It**: Our cache needs to store `HashMap<K, V>` and `VecDeque<K>` - both require `RefCell` for interior mutability.

---

### Thread Safety: From RefCell to Mutex

**The Problem**: `RefCell` is **NOT thread-safe**. If two threads access it simultaneously, you get undefined behavior (data races).

```rust
let cache = RefCell::new(HashMap::new());
let cache_ref = &cache;

// Thread 1
std::thread::spawn(move || {
    cache_ref.borrow_mut().insert(1, 100);  // ❌ DATA RACE!
});

// Thread 2 (simultaneously)
cache.borrow_mut().insert(2, 200);  // ❌ DATA RACE!
```

**The Solution**: `Mutex<T>` - thread-safe interior mutability using OS-level locks.

---

### Mutex<T>: Thread-Safe Interior Mutability

**What It Is**: A mutual exclusion lock that ensures only one thread can access the data at a time.

**How It Works**:
```rust
use std::sync::Mutex;

let cache = Mutex::new(HashMap::new());

// Acquire lock (blocks if another thread holds it)
let mut data = cache.lock().unwrap();  // Returns MutexGuard
data.insert("key", "value");
// Lock automatically released when guard drops
```

**Key Properties**:
- **Thread-safe**: Safe to share between threads
- **Blocking**: If locked, other threads wait
- **Significant overhead**: System call (~20-100ns vs 0.2ns for `RefCell`)
- **Poisoning**: If thread panics while holding lock, mutex is "poisoned"

**Performance Impact**:
```
Operation          RefCell     Mutex      Slowdown
Simple increment   0.2ns       20ns       100x
HashMap lookup     10ns        30ns       3x
Complex operation  100ns       150ns      1.5x
```

**Why We Need It**: To make the cache usable from multiple threads safely.

---

### Arc<T>: Shared Ownership Across Threads

**The Problem**: Can't share `&Cache` across threads because threads might outlive the original owner.

```rust
let cache = LRUCache::new(100);
std::thread::spawn(|| {
    cache.put(1, 2);  // ❌ Error: cache may not live long enough
});
```

**The Solution**: `Arc<T>` (Atomic Reference Counted) - shared ownership with atomic counters.

**How It Works**:
```rust
use std::sync::Arc;

let cache = Arc::new(LRUCache::new(100));
let cache_clone = Arc::clone(&cache);  // Increment ref count

std::thread::spawn(move || {
    cache_clone.put(1, 2);  // ✅ Works! Thread owns a clone
});

cache.get(&1);  // ✅ Original still valid
// Last Arc drops -> cache is freed
```

**Key Properties**:
- **Atomic counters**: Thread-safe reference counting
- **Shared ownership**: Multiple owners, freed when last one drops
- **Clone is cheap**: Just increments counter (~5-10ns)
- **Immutable by default**: Need `Arc<Mutex<T>>` or `Arc<RwLock<T>>` for mutation

**Why We Need It**: Threads need independent ownership of the cache.

---

### Generics: Type-Agnostic Data Structures

**The Problem**: We want our cache to work with any key/value types, not just `String` → `i32`.

**The Solution**: Generic type parameters `<K, V>`.

**How It Works**:
```rust
struct LRUCache<K, V> {
    data: HashMap<K, V>,
    // ...
}

impl<K, V> LRUCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    fn new(capacity: usize) -> Self { /* ... */ }
    fn get(&self, key: &K) -> Option<V> { /* ... */ }
}

// Use with any types that satisfy the trait bounds:
let int_cache: LRUCache<i32, String> = LRUCache::new(100);
let str_cache: LRUCache<String, Vec<u8>> = LRUCache::new(50);
```

**Trait Bounds Explained**:
- **`K: Eq + Hash`**: Keys must be comparable and hashable (required for `HashMap`)
- **`K: Clone`**: Need to copy keys into `VecDeque` for order tracking
- **`V: Clone`**: Need to return cloned values (can't give away ownership)

**Why We Use It**: Makes the cache reusable for any data types.

---

### Understanding Trait Bounds

This project uses several trait bounds. Here's what they mean:

| Trait | Purpose | Example |
|-------|---------|---------|
| `Eq` | Equality comparison | Required for HashMap keys |
| `Hash` | Hash function | Required for HashMap keys |
| `Clone` | Deep copy | Needed to return values without moving |
| `Copy` | Bitwise copy | Only for `Cell<T>` types like `usize` |
| `Send` | Safe to send across threads | Required for thread-safe types |
| `Sync` | Safe to share refs across threads | Required for thread-safe types |

**Type Requirements Summary**:
```rust
// Milestone 1-4 (single-threaded with RefCell):
K: Eq + Hash + Clone
V: Clone

// Milestone 5 (thread-safe with Mutex):
K: Eq + Hash + Clone + Send
V: Clone + Send
```

---

### Performance Trade-offs: RefCell vs Mutex

Understanding when to use each is critical:

| Aspect | Cell<T> | RefCell<T> | Mutex<T> |
|--------|---------|-----------|----------|
| **Types** | `Copy` only | Any type | Any type |
| **Thread-safe** | ❌ No | ❌ No | ✅ Yes |
| **Overhead** | 0 cycles | ~2 cycles | ~50-100 cycles |
| **Borrow check** | None | Runtime | Runtime |
| **Failure mode** | Compile error | Panic | Deadlock/poison |
| **Use case** | Counters | Single-thread collections | Multi-thread collections |

**Decision Guide**:
1. **Need to share across threads?** → Must use `Arc<Mutex<T>>`
2. **Single thread + `Copy` type?** → Use `Cell<T>`
3. **Single thread + non-`Copy` type?** → Use `RefCell<T>`

---

### Why Multiple Data Structures?

**The Challenge**: LRU requires **both** fast lookup AND ordered access tracking.

| Requirement | Data Structure | Time Complexity |
|-------------|---------------|-----------------|
| Fast lookup by key | `HashMap<K, V>` | O(1) |
| Track access order | `VecDeque<K>` | O(1) evict, O(n) update |
| (Optimal version) | `LinkedList<K>` | O(1) all ops |

**Why We Use Both**:
- **HashMap**: Stores actual key-value pairs, enables O(1) lookup
- **VecDeque**: Tracks access order (back = most recent, front = least recent)

**Synchronization Challenge**:
```rust
struct LRUCache<K, V> {
    data: RefCell<HashMap<K, V>>,    // The actual cache data
    order: RefCell<VecDeque<K>>,     // The access order
}

// Every operation must keep them in sync!
fn put(&self, key: K, value: V) {
    let mut data = self.data.borrow_mut();
    let mut order = self.order.borrow_mut();

    data.insert(key.clone(), value);
    order.push_back(key);  // Must stay synchronized!
}
```

**Why VecDeque Instead of Vec**:
- `VecDeque` supports O(1) removal from front (LRU eviction)
- `Vec` would require O(n) shifting when removing front element

---

### Connection to This Project

This project implements an LRU cache with the following progression:

1. **Milestone 1-2**: Learn interior mutability patterns (`Cell`, `RefCell`)
2. **Milestone 3**: Implement LRU eviction logic with `HashMap + VecDeque`
3. **Milestone 4**: Add statistics tracking (hit rate, miss rate)
4. **Milestone 5**: Make thread-safe with `Mutex` for concurrent access

**Key learning points**:
- **O(n) is acceptable for learning**: The VecDeque approach is simpler to understand
- **Statistics matter**: Can't optimize what you don't measure
- **Thread safety has costs**: `Mutex` is ~50-100x slower than `RefCell`
- **Algorithm affects concurrency**: LRU requires write-on-read (updating order), limiting parallelism

---

### Milestone 1: Basic Statistics Tracker

**LRU Caches Need Metrics**: In production, an LRU cache without metrics is blind. You need to know:
    - **Hit rate** (hits / total accesses) - Is the cache effective?
    - **Miss rate** - Should you increase capacity?
    - Whether the cache is worth the memory cost

**The Core Challenge**: How do we increment counters through `&self` instead of `&mut self`? This milestone teaches you the solution: `Cell<T>`.


**Goal**: Create a simple counter

#### Architecture
**Structs:**
- `StatsTracker` - manages the counters
  - **Field**:  `hits` - counts the hits
  - **Field**:  `misses` - counts the misses
  
**Functions**:
- `new(value: T) -> StatsTracker<` - Allocates and initializes
- `record_hit`  - Count the hits
- `record_miss`  - Count the misses
- `get_stats` - Returns both


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

#### Why Milestone 1 Isn't Enough

**Limitation**: `Cell<T>` only works with `Copy` types (like `usize`, `bool`, `i32`). For caching, we need to store complex data structures like `HashMap<K, V>`, which don't implement `Copy`.

**What we're adding**: `RefCell<T>` allows interior mutability for *any* type, not just `Copy` types. Trade-off: `Cell` has zero overhead, while `RefCell` performs runtime borrow checking.

**Improvement**:
- **Capability**: Can now wrap collections (HashMap, Vec, etc.)
- **Cost**: Small runtime overhead for borrow checking (~1-2 CPU cycles)
- **Safety**: Panics if borrow rules violated at runtime vs compile errors with plain borrows

---

### Milestone 2: Simple HashMap Cache (No Eviction Yet)

**Goal**: Create a cache using `RefCell<HashMap>` that can insert and retrieve values.

**Architcture**:
- **Structs**: `SimpleCache`
  - **Field**: `data: RefCell<HashMap<K, V>>`

**Functions**:
- `new() -> SimpleCache<K, V>` - Allocates and initializes
- `get(&self, key: &K)`  - return a value if present
- `put(&self, key: K, value: V)` - insert the key-value pair
- `len(&self)` - return number of items in cache


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

#### Why Milestone 2 Isn't Enough

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


### Milestone 3: LRU Cache with Fixed Capacity

 Add capacity limit and eviction logic. Use `VecDeque` to track access order.

**Architecture**
**struct** `LRUCache` 
```rust
struct LRUCache<K, V> {
  capacity: usize,
  data: RefCell<HashMap<K, V>>,
  order: RefCell<VecDeque<K>>,  // Most recent at back
}
```
**functions** (different implementations)
- `new() -> LRUCache<K, V>` - Allocates and initializes
- `get(&self, key: &K)`  - return a value if present
- `put(&self, key: K, value: V)` - insert the key-value pair
- `len(&self)` - return number of items in cache


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

#### Why Milestone 3 Isn't Enough

**Limitation**: We have no visibility into cache performance! Without metrics, we can't answer:
- Is the cache effective? (high hit rate = good, low = wasting memory)
- Should we increase capacity? (too many misses)
- Is it worth the memory cost? (hit rate analysis)

**What we're adding**: the `StatsTracker`
- **Hit/Miss tracking** - Measure cache effectiveness
- **Statistics API** - Query performance metrics
- **Persistent metrics** - Stats survive cache clears

**Improvement**:
- **Observability**: Can measure cache effectiveness (hit rate = hits / (hits + misses))
- **Optimization guidance**: Low hit rate → increase capacity or change eviction policy
- **Production monitoring**: Export metrics to Prometheus/Grafana
- **Cost**: Minimal—just two `Cell<usize>` increments per access

---

### Milestone 4: Add Statistics Tracking

Integrate the stats tracker from Milestone 1.

**Architecture**: 
**struct**: Modify your `LRUCache` to include a field `StatsTracker` field and update 
**functions**: 
- `get()` to record hits/misses. 
- `stats()`  wrapper of `get_stats()`
- `clear()` clear `data` and `order`
 
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

#### Why Milestone 4 Isn't Enough

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


**When it's worth it**: When you have concurrent access. Single-threaded? Stick with `RefCell`.

---

### Milestone 5: Thread-Safe Version with Mutex

 Create a thread-safe version using `Mutex` instead of `RefCell`.


**Architecture**:
- Use `self.inner.lock().unwrap()` to get a `MutexGuard`
- The guard automatically releases when dropped
- For thread-safety, `StatsTracker` needs `AtomicUsize` instead of `Cell<usize>`


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
### Complete Project Summary

**What You Built**:
1. Stats tracker with `Cell` for interior mutability
2. Simple cache with `RefCell<HashMap>`
3. LRU eviction logic with `VecDeque`
4. Thread-safe version with `Mutex`

**Key Concepts Practiced**:
- Interior mutability: `Cell`, `RefCell`, `Mutex`, `RwLock`
- Borrow scope management
- Thread safety with `Arc`
- Performance trade-offs between different synchronization primitives

---
