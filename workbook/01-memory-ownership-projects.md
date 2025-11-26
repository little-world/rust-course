# Chapter 01: 

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
1. **Web application request handlers** - Multiple handlers share a cache
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

### Milestone 1: Basic Statistics Tracker 

1. **LRU Caches Need Metrics**: In production, an LRU cache without metrics is blind. You need to know:
   - **Hit rate** (hits / total accesses) - Is the cache effective?
   - **Miss rate** - Should you increase capacity?
   - Whether the cache is worth the memory cost

**The Core Challenge**: How do we increment counters through `&self` instead of `&mut self`? This milestone teaches you the solution: `Cell<T>`.


**Goal**: Create a simple counter

**Key concepts**:
- Structs: `StatsTracker`,
- Fields:  `hits`, `misses`
- Functions:
   - `new(value: T) -> StatsTracker<` - Allocates and initializes
   - `record_hit`  - Increments the hits
   - `record_miss`  - Increments the misses 
   - `get_stats` - Returns both

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

**Check Your Understanding**:
- Why can we call `record_hit(&self)` without `&mut self`?
- What is `Cell<T>` and why is it useful here??
- Why do we need `Cell::get()` before incrementing?

---

### Why Milestone 1 Isn't Enough → Moving to Milestone 2

**Limitation**: `Cell<T>` only works with `Copy` types (like `usize`, `bool`, `i32`). For caching, we need to store complex data structures like `HashMap<K, V>`, which don't implement `Copy`.

**What we're adding**: `RefCell<T>` allows interior mutability for *any* type, not just `Copy` types. Trade-off: `Cell` has zero overhead, while `RefCell` performs runtime borrow checking.

**Improvement**:
- **Capability**: Can now wrap collections (HashMap, Vec, etc.)
- **Cost**: Small runtime overhead for borrow checking (~1-2 CPU cycles)
- **Safety**: Panics if borrow rules violated at runtime vs compile errors with plain borrows

---

### Milestone 2: Simple HashMap Cache (No Eviction)

**Goal**: Create a cache using `RefCell<HashMap>` that can insert and retrieve values through `&self`.

**Key concepts**:
- Structs: `SimpleCache`
- Fields: `data: RefCell<HashMap<K, V>>`,
- Functions:
   - `new() -> SimpleCache<K, V>` - Allocates and initializes
   - `get(&self, key: &K)`  - return a value if present
   - `put(&self, key: K, value: V)` - insert the key-value pair
   - `len(&self)` - return number of items in cache

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

**Key Concepts** 
Change `SimpleCache` to `LRUCache` below
```rust
struct LRUCache<K, V> {
  capacity: usize,
  data: RefCell<HashMap<K, V>>,
  order: RefCell<VecDeque<K>>,  // Most recent at back
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


**Check Your Understanding**:
- Why do we need to `drop(data)` before modifying `order`?
- What happens if we try to hold both borrows simultaneously?
- How does `VecDeque` help us track LRU order?
- Why do we check `contains_key` before checking capacity?

---

### Why Milestone 3 Isn't Enough → Moving to Milestone 4

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

**Task**: Modify your `LRUCache` to include a `StatsTracker` field and update `get()` to record hits/misses. Add two functions to `LRUCache`: `stats` and `clear`


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

**Check Your Understanding**:
- Why don't we clear stats when clearing the cache?
- Could we use `Cell<(usize, usize)>` instead of separate `Cell` fields?

---

### Why Milestone 4 Isn't Enough → Moving to Milestone 5

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

## Project 2: Arena-Based Expression Parser

### Problem Statement

Build a parser for arithmetic expressions that uses arena (bump) allocation. This demonstrates how arena allocation can dramatically speed up programs that create many small objects.

### Why It Matters

**Real-World Impact**: Compilers, parsers, and interpreters allocate millions of small objects (AST nodes, tokens, symbols). Traditional `malloc`/`free` becomes a bottleneck:

**Performance Disaster with Box<T>**:
- Parsing 10,000 expressions with `Box<Expr>`: Each node = 1 malloc call
- Expression `(1+2)*(3+4)` = 7 nodes = 7 malloc calls
- 10,000 expressions × 7 nodes average = **70,000 allocations**
- Each malloc: ~50-100ns (involves locks, metadata, fragmentation)
- Total time: 70,000 × 75ns = **5.25ms just for allocation**

**Arena Allocation Solution**:
- Pre-allocate 4KB chunk, bump pointer for each node
- Per-allocation cost: **~2-5ns** (pointer increment + write)
- Same 70,000 nodes: 70,000 × 3ns = **0.21ms** for allocation
- **25x faster allocation**, plus better cache locality

**Real Production Examples**:
- **Rust compiler**: Uses arenas for AST, HIR, MIR. Parsing 1M LOC project creates ~10M AST nodes in seconds.
- **V8 JavaScript**: Zone allocation (arena) for parser—parsed millions of nodes per second.
- **LLVM**: BumpPtrAllocator for IR nodes, symbol tables.
- **Databases**: Query plan nodes allocated in per-query arenas.

### Use Cases

**When you need this pattern**:
1. **Compiler frontends**: Lexer tokens, AST nodes, symbol table entries
2. **Web request handlers**: Per-request temporary objects (template AST, JSON parsing)
3. **Game engines**: Per-frame allocations (particle systems, AI pathfinding nodes)
4. **Database query execution**: Query plan nodes, temporary expression trees
5. **Text editors**: Syntax tree for incremental parsing
6. **JSON/XML parsers**: DOM nodes, parsing state

**Key characteristic**: Objects have the same lifetime—allocate many, free all at once.

**Counter-examples** (DON'T use arenas):
- Long-lived objects with individual lifetimes
- Objects that need to outlive the arena
- Memory that needs individual deallocation

### Learning Goals

- Understand arena/bump allocation and when it's appropriate
- Work with lifetimes in AST structures (`'arena` lifetime)
- Experience 10-100x allocation speedup
- Practice recursive descent parsing
- Understand memory layout and alignment requirements

---

### Milestone 1: Define AST Types

**Goal**: Create the expression tree data structures that represent arithmetic expressions.

**Key Concepts**:

1. **Expression Type** (`Expr`):
   - Should represent either a literal number or a binary operation
   - For binary operations, needs to store: the operator type and references to left and right sub-expressions

2. **Operator Type** (`OpType`):
   - Should represent the four arithmetic operations: addition, subtraction, multiplication, and division

3. **Evaluation**:
   - Implement an `eval()` method on `OpType` that takes two numbers and returns the result
   - Handle division by zero by returning a `Result<i64, String>`
   - Implement an `eval()` method on `Expr` that recursively evaluates the expression tree
   - For binary operations, evaluate both sides first, then apply the operator

**Design Hints**:
- Think about what variants your `Expr` enum needs
- Consider what data each variant should hold
- Remember that references in the tree need a lifetime annotation
- Binary operations need to store three pieces of information

**Implementation Hints**:
- Use pattern matching to handle different expression types
- For recursive evaluation, use the `?` operator to propagate errors
- Return appropriate error messages for invalid operations


**Checkpoint Tests**:
```rust
#[test]
fn test_literal_eval() {
    let expr = Expr::Literal(42);
    assert_eq!(expr.eval(), Ok(42));
}

#[test]
fn test_binop_eval() {
    let left = Expr::Literal(10);
    let right = Expr::Literal(5);
    let expr = Expr::BinOp {
        op: OpType::Add,
        left: &left,
        right: &right,
    };
    assert_eq!(expr.eval(), Ok(15));
}

#[test]
fn test_nested_eval() {
    // (2 + 3) * 4 = 20
    let two = Expr::Literal(2);
    let three = Expr::Literal(3);
    let four = Expr::Literal(4);

    let add = Expr::BinOp {
        op: OpType::Add,
        left: &two,
        right: &three,
    };

    let mul = Expr::BinOp {
        op: OpType::Mul,
        left: &add,
        right: &four,
    };

    assert_eq!(mul.eval(), Ok(20));
}

#[test]
fn test_division_by_zero() {
    let ten = Expr::Literal(10);
    let zero = Expr::Literal(0);
    let expr = Expr::BinOp {
        op: OpType::Div,
        left: &ten,
        right: &zero,
    };
    assert!(expr.eval().is_err());
}
```

**Check Your Understanding**:
- What does the `'arena` lifetime mean?
- Why do we use `&'arena Expr<'arena>` instead of `Box<Expr>`?
- How does the recursive `eval()` work?


**Solution**:
```rust
#[derive(Debug, PartialEq)]
enum Expr<'arena> {
    Literal(i64),
    BinOp {
        op: OpType,
        left: &'arena Expr<'arena>,
        right: &'arena Expr<'arena>,
    },
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum OpType {
    Add,
    Sub,
    Mul,
    Div,
}

impl OpType {
    fn eval(&self, left: i64, right: i64) -> Result<i64, String> {
        match self {
            OpType::Add => Ok(left + right),
            OpType::Sub => Ok(left - right),
            OpType::Mul => Ok(left * right),
            OpType::Div => {
                if right == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(left / right)
                }
            }
        }
    }
}
```

**Add evaluation**:
```rust
impl<'arena> Expr<'arena> {
    fn eval(&self) -> Result<i64, String> {
        match self {
            Expr::Literal(n) => Ok(*n),
            Expr::BinOp { op, left, right } => {
                let left_val = left.eval()?;
                let right_val = right.eval()?;
                op.eval(left_val, right_val)
            }
        }
    }
}
```

---

### Why Milestone 1 Isn't Enough → Moving to Milestone 2

**Limitation**: We've defined the types, but how do we actually create these AST nodes? Using stack allocation limits us to small, fixed-size trees. We need heap allocation.

**What we're adding**: First, we'll implement the traditional `Box` approach to understand the baseline, then optimize with arena allocation.

---

### Milestone 2: Box-Based Expression Trees

**Goal**: Implement expressions using `Box<Expr>` to understand traditional heap allocation.

**Key Concepts**:
- Each AST node gets its own heap allocation via `Box::new()`
- Each node has its own drop when the tree is freed
- This is the "normal" approach used in many programming languages

**Design Changes**:
Instead of using references with lifetimes, we'll use `Box` pointers:


**Checkpoint Tests**:
```rust
#[test]
fn test_box_expr_literal() {
    let expr = BoxExprBuilder::literal(42);
    assert_eq!(expr.eval(), Ok(42));
}

#[test]
fn test_box_expr_addition() {
    let expr = BoxExprBuilder::add(
        BoxExprBuilder::literal(10),
        BoxExprBuilder::literal(5),
    );
    assert_eq!(expr.eval(), Ok(15));
}

#[test]
fn test_box_expr_nested() {
    // Build: (2 + 3) * 4 = 20
    let expr = BoxExprBuilder::mul(
        BoxExprBuilder::add(
            BoxExprBuilder::literal(2),
            BoxExprBuilder::literal(3),
        ),
        BoxExprBuilder::literal(4),
    );
    assert_eq!(expr.eval(), Ok(20));
}

#[test]
fn test_box_expr_complex() {
    // Build: ((10 - 5) * 2) + (8 / 4) = 12
    let expr = BoxExprBuilder::add(
        BoxExprBuilder::mul(
            BoxExprBuilder::sub(
                BoxExprBuilder::literal(10),
                BoxExprBuilder::literal(5),
            ),
            BoxExprBuilder::literal(2),
        ),
        BoxExprBuilder::div(
            BoxExprBuilder::literal(8),
            BoxExprBuilder::literal(4),
        ),
    );
    assert_eq!(expr.eval(), Ok(12));
}
```

```rust
#[derive(Debug, PartialEq)]
enum BoxExpr {
    Literal(i64),
    BinOp {
        op: OpType,
        left: Box<BoxExpr>,
        right: Box<BoxExpr>,
    },
}

impl BoxExpr {
    fn eval(&self) -> Result<i64, String> {
        match self {
            BoxExpr::Literal(n) => Ok(*n),
            BoxExpr::BinOp { op, left, right } => {
                let left_val = left.eval()?;
                let right_val = right.eval()?;
                op.eval(left_val, right_val)
            }
        }
    }
}
```

**Builder Pattern**:
```rust
struct BoxExprBuilder;

impl BoxExprBuilder {
    fn literal(n: i64) -> Box<BoxExpr> {
        // TODO: Allocate BoxExpr::Literal(n) using Box::new()
        todo!()
    }

    fn binary(op: OpType, left: Box<BoxExpr>, right: Box<BoxExpr>) -> Box<BoxExpr> {
        // TODO: Allocate BoxExpr::BinOp using Box::new()
        todo!()
    }

    fn add(left: Box<BoxExpr>, right: Box<BoxExpr>) -> Box<BoxExpr> {
        // TODO: Call binary() with OpType::Add
        todo!()
    }

    fn sub(left: Box<BoxExpr>, right: Box<BoxExpr>) -> Box<BoxExpr> {
        // TODO: Call binary() with OpType::Sub
        todo!()
    }

    fn mul(left: Box<BoxExpr>, right: Box<BoxExpr>) -> Box<BoxExpr> {
        // TODO: Call binary() with OpType::Mul
        todo!()
    }

    fn div(left: Box<BoxExpr>, right: Box<BoxExpr>) -> Box<BoxExpr> {
        // TODO: Call binary() with OpType::Div
        todo!()
    }
}
```


**Check Your Understanding**:
- How many heap allocations occur for the expression `(2 + 3) * 4`?
- What happens when a `Box<BoxExpr>` goes out of scope?
- Why does the builder consume (take ownership of) the `Box` parameters?
- What are the performance implications of many small allocations?

---

### Why Milestone 2 Isn't Enough → Moving to Milestone 3

**Performance Problem**: Every single AST node requires a separate heap allocation with `Box::new()`. Let's analyze the cost:

**Allocation Overhead**:
- Expression `(1+2)*(3+4)` = 7 nodes
- Each `Box::new()`: ~50-100ns (involves malloc, locks, metadata)
- Total allocation time: 7 × 75ns = **525ns just for allocations**
- Parsing 10,000 expressions: 70,000 allocations = **5.25ms**

**Memory Fragmentation**:
- Nodes scattered across heap memory
- Poor cache locality (next node likely in different cache line)
- Each allocation has ~16 bytes overhead for allocator metadata

**What we're adding**: **Arena allocator** - bump allocation strategy:
An arena (also called a bump allocator) is a simple memory allocator that hands out memory by continuously "bumping" a pointer forward inside a pre-allocated buffer. Individual allocations are extremely cheap (often just pointer arithmetic), and deallocation is even simpler: you free everything at once by dropping the arena.

- Mental model: Imagine a notebook. Each allocation writes on the next free line. You cannot erase individual lines; you throw away the whole notebook when you're done.
- Key trade-off: No per-object free. This only fits workloads where many objects share the same lifetime (they all become unreachable together).

How it works at a glance:
1. Reserve a big chunk of memory (e.g., 4 KB).
2. Keep an offset (the "bump" pointer) into that chunk.
3. To allocate `T`, round the offset up to `align_of::<T>()`, ensure there’s room, then return a pointer/reference to that slot and advance the offset by `size_of::<T>()`.
4. When the arena goes out of scope, the whole chunk is freed at once.

Why use it for ASTs and similar graphs:
- Many small nodes created together and dropped together at the end of parsing/evaluation.
- Significantly fewer calls to the global allocator → better performance and cache locality.

Safety and Rust lifetimes:
- Arena returns references with a lifetime tied to the arena itself (e.g., `&'arena T`). That prevents use-after-free because references cannot outlive the arena.
- Alignment must be respected; a `u64` must be placed at an 8-byte aligned address, etc.

Contrast with `Box<T>` per node:
- `Box<T>`: many small, scattered allocations; each `Box` is freed individually.
- Arena: one or few big allocations; trivial per-object allocation; single bulk free.

**Improvements**:
- **Speed**: Allocation is pointer increment (~2-5ns) vs malloc (~75ns) = **25x faster**
- **Memory**: Better cache locality (nodes allocated sequentially)
- **Simplicity**: No individual frees—drop arena, free everything
- **Alignment**: Must handle properly (u8 at any address, u64 needs 8-byte alignment)

**Complexity trade-off**: Can't free individual objects. Only works when all objects have same lifetime.

---

### Milestone 3: Simple Bump Allocator

**Goal**: Implement a basic arena that can allocate objects.


**Key Concepts**
struct: `Arena` 
fields: `storage`: RefCell<Vec<u8>>
functions: 
 - `new() `
 - `alloc()`


**Checkpoint Tests**:
```rust
#[test]
fn test_arena_alloc_int() {
    let arena = Arena::new();
    let x = arena.alloc(42);
    assert_eq!(*x, 42);

    *x = 100;
    assert_eq!(*x, 100);
}

#[test]
fn test_arena_multiple_allocs() {
    let arena = Arena::new();
    let x = arena.alloc(1);
    let y = arena.alloc(2);
    let z = arena.alloc(3);

    assert_eq!(*x, 1);
    assert_eq!(*y, 2);
    assert_eq!(*z, 3);
}

#[test]
fn test_arena_alloc_string() {
    let arena = Arena::new();
    let s = arena.alloc(String::from("hello"));
    assert_eq!(s, "hello");
}

#[test]
fn test_arena_alignment() {
    let arena = Arena::new();
    let _byte = arena.alloc(1u8);
    let num = arena.alloc(1234u64);  // Needs 8-byte alignment

    let ptr = num as *const u64 as usize;
    assert_eq!(ptr % 8, 0, "u64 should be 8-byte aligned");
}
```

**Starter Code**:
```rust
use std::cell::RefCell;
use std::ptr::NonNull;

struct Arena {
    storage: RefCell<Vec<u8>>,
}

impl Arena {
    fn new() -> Self {
        Arena {
            storage: RefCell::new(Vec::with_capacity(4096)),
        }
    }

    fn alloc<T>(&self, value: T) -> &mut T {
        let mut storage = self.storage.borrow_mut();

        // TODO: Calculate size and alignment using std::mem functions
        let size = todo!("Get size of T");
        let align = todo!("Get alignment of T");

        // TODO: Get current position in storage
        let current_len = todo!();

        // TODO: Calculate aligned position
        // Hint: padding = (align - (current_len % align)) % align
        let padding = todo!();
        let start = todo!("current_len + padding");

        // TODO: Ensure we have space in storage
        // Hint: Use storage.resize(start + size, 0)
        todo!();

        // TODO: Get pointer to allocated space
        // Hint: &mut storage[start] as *mut u8 as *mut T
        let ptr = todo!();

        unsafe {
            // TODO: Write value to allocated space using std::ptr::write
            todo!();
            // TODO: Return mutable reference with arena lifetime
            todo!()
        }
    }
}
```

**Check Your Understanding**:
- Why do we need alignment?
- What does `std::ptr::write` do?
- Why is the function marked `unsafe`?
- What lifetime does the returned reference have?

---

### Milestone 4: Build Expressions in Arena

**Goal**: Use the arena allocator to create expression trees with the builder pattern.

**Why This Milestone Matters**:

Now that we have a working arena allocator (Milestone 3), we need a clean API to use it. Directly calling `arena.alloc()` everywhere would be verbose and error-prone. The **Builder Pattern** provides a fluent, type-safe interface for constructing expression trees.

**What We're Building**:

The `ExprBuilder` wraps the arena and provides convenient methods like `literal()`, `add()`, `mul()` that hide the allocation details. Compare:

```rust
// Without builder (verbose, easy to mess up lifetimes):
let two = arena.alloc(Expr::Literal(2));
let three = arena.alloc(Expr::Literal(3));
let sum = arena.alloc(Expr::BinOp {
    op: OpType::Add,
    left: two,
    right: three,
});

// With builder (clean, fluent):
let two = builder.literal(2);
let three = builder.literal(3);
let sum = builder.add(two, three);
```

**Key Design Decisions**:

1. **Builder holds `&'arena Arena`**: The builder doesn't own the arena—it just borrows it. This allows multiple builders to share one arena if needed.

2. **All methods return `&'arena Expr<'arena>`**: Every expression we allocate lives in the arena, and the lifetime annotation ensures they can't outlive it.

3. **Convenience methods** (`add`, `mul`, etc.): These wrap the generic `binary()` method, making expression construction more readable.

**The Lifetime Dance**:

Notice the signature: `fn literal(&self, n: i64) -> &'arena Expr<'arena>`. We take `&self` (short borrow of builder), but return `&'arena` (long-lived reference tied to arena's lifetime). This works because:
- The builder holds `&'arena Arena`
- We allocate in that arena
- The returned reference lives as long as the arena, not the builder

**Real-World Applications**:

This pattern appears in many Rust projects:
- **Compiler AST builders**: rustc uses arena allocation with builder APIs
- **HTML/XML builders**: Construct DOM trees without individual allocations
- **Query builders**: SQL query DSLs that build AST nodes
- **Game scene graphs**: Building hierarchical scene trees efficiently

**Key Concepts**:

**Struct**: `ExprBuilder<'arena>`
- **Fields**: `arena: &'arena Arena`
- **Methods**:
  - `new(arena: &'arena Arena) -> Self` - Creates builder wrapping arena
  - `literal(&self, n: i64) -> &'arena Expr<'arena>` - Allocates literal expression
  - `binary(&self, op: OpType, left: &'arena Expr<'arena>, right: &'arena Expr<'arena>) -> &'arena Expr<'arena>` - Generic binary operation
  - `add(...)`, `sub(...)`, `mul(...)`, `div(...)` - Convenience wrappers


**Checkpoint Tests**:
```rust
#[test]
fn test_builder() {
    let arena = Arena::new();
    let builder = ExprBuilder::new(&arena);

    // Build: (2 + 3) * 4
    let two = builder.literal(2);
    let three = builder.literal(3);
    let four = builder.literal(4);

    let sum = builder.add(two, three);
    let product = builder.mul(sum, four);

    assert_eq!(product.eval(), Ok(20));
}

#[test]
fn test_complex_expression() {
    let arena = Arena::new();
    let builder = ExprBuilder::new(&arena);

    // Build: ((10 - 5) * 2) + (8 / 4)
    let expr = builder.add(
        builder.mul(
            builder.sub(builder.literal(10), builder.literal(5)),
            builder.literal(2)
        ),
        builder.div(builder.literal(8), builder.literal(4))
    );

    assert_eq!(expr.eval(), Ok(12)); // (5 * 2) + 2 = 12
}
```

**Starter Code**:
```rust
struct ExprBuilder<'arena> {
    arena: &'arena Arena,
}

impl<'arena> ExprBuilder<'arena> {
    fn new(arena: &'arena Arena) -> Self {
        // TODO: Create ExprBuilder with reference to arena
        todo!()
    }

    fn literal(&self, n: i64) -> &'arena Expr<'arena> {
        // TODO: Allocate Expr::Literal(n) in arena and return reference
        todo!()
    }

    fn binary(
        &self,
        op: OpType,
        left: &'arena Expr<'arena>,
        right: &'arena Expr<'arena>,
    ) -> &'arena Expr<'arena> {
        // TODO: Allocate Expr::BinOp in arena with given op, left, right
        todo!()
    }

    fn add(
        &self,
        left: &'arena Expr<'arena>,
        right: &'arena Expr<'arena>,
    ) -> &'arena Expr<'arena> {
        // TODO: Call binary() with OpType::Add
        todo!()
    }

    // TODO: Add methods for sub, mul, div following the same pattern
    fn sub(&self, left: &'arena Expr<'arena>, right: &'arena Expr<'arena>) -> &'arena Expr<'arena> {
        todo!()
    }

    fn mul(&self, left: &'arena Expr<'arena>, right: &'arena Expr<'arena>) -> &'arena Expr<'arena> {
        todo!()
    }

    fn div(&self, left: &'arena Expr<'arena>, right: &'arena Expr<'arena>) -> &'arena Expr<'arena> {
        todo!()
    }
}
```


**Check Your Understanding**:
- Why does the builder need a reference to the arena?
- Can expressions outlive the arena?
- How many heap allocations happen for a 3-node tree?

---

### Milestone 5: Lexer (Tokenizer)

**Goal**: Transform raw text input into a stream of tokens that the parser can work with.

**Why This Milestone Matters**:

So far, we've been manually constructing expression trees using the builder. But real parsers work with text input like `"(2 + 3) * 4"`. The **lexer** (also called tokenizer or scanner) is the first stage of parsing that breaks this text into meaningful chunks called **tokens**.

**The Two-Stage Pipeline**:

```
Text → Lexer → Tokens → Parser → AST
"2+3"  →       [Num(2), Plus, Num(3)]  →  BinOp{Add, 2, 3}
```

Separating lexing from parsing is a fundamental compiler design pattern because:
1. **Separation of concerns**: Lexing handles character-level details (whitespace, digits), parsing handles structure (precedence, grammar)
2. **Simplification**: Parser doesn't worry about whitespace or number parsing
3. **Reusability**: Same token stream can feed multiple parsers
4. **Performance**: Can optimize lexer separately (e.g., SIMD for digit scanning)

**What We're Building**:

A **Lexer** that walks through input text character-by-character and identifies:
- **Numbers**: Sequences of digits like `123`, `0`, `9876`
- **Operators**: `+`, `-`, `*`, `/`
- **Parentheses**: `(`, `)`
- **Whitespace**: Skipped (not significant in arithmetic)
- **End of input**: Special `End` token

**The Lexer State**:

```rust
struct Lexer {
    input: Vec<char>,   // Input text as characters
    position: usize,    // Current position in input
}
```

We convert the string to `Vec<char>` because:
- Easy indexing by character (not byte)
- Handles multi-byte Unicode properly (though our grammar is ASCII-only)
- Simple `position` counter tracks where we are

**Key Operations**:

1. **`peek()`**: Look at current character without moving forward
2. **`advance()`**: Move position forward by one character
3. **`skip_whitespace()`**: Skip spaces, tabs, newlines
4. **`read_number()`**: Consume consecutive digits and build an integer
5. **`next_token()`**: Return the next token from input
6. **`tokenize()`**: Convert entire input to `Vec<Token>`

**Example Tokenization**:

```rust
Input: "(10 + 5) * 2"

Steps:
1. Skip nothing, see '(' → Token::LeftParen, advance
2. Skip space, see '1' → read_number() → Token::Number(10), advance twice
3. Skip space, see '+' → Token::Plus, advance
4. Skip space, see '5' → read_number() → Token::Number(5), advance
5. Skip nothing, see ')' → Token::RightParen, advance
6. Skip space, see '*' → Token::Star, advance
7. Skip space, see '2' → read_number() → Token::Number(2), advance
8. At end → Token::End

Output: [LeftParen, Number(10), Plus, Number(5), RightParen, Star, Number(2), End]
```

**Error Handling**:

The lexer must detect invalid characters:
```rust
Input: "2 & 3"  // '&' is not a valid operator
Result: Err("Unexpected character '&'")
```

Returning `Result<Token, String>` allows propagating errors up to the caller.

**Real-World Lexers**:

- **Programming languages**: Every compiler has a lexer (rustc, clang, javac)
- **Configuration parsers**: JSON, YAML, TOML start with lexing
- **Text editors**: Syntax highlighting uses lexers to identify token types
- **Log parsers**: Extract timestamps, levels, messages from log lines

**Performance Considerations**:

Lexing is typically very fast (millions of tokens/second) because:
- Simple character-by-character scanning
- No recursion or complex logic
- Cache-friendly sequential access
- Can be parallelized for large files (split at whitespace boundaries)

**Design Decision: Why `Vec<Token>` instead of Iterator?**

We return `Vec<Token>` from `tokenize()` for simplicity. Production lexers often use iterators:
```rust
// Production style:
for token in lexer {
    match token? {
        Token::Number(n) => ...,
        ...
    }
}
```

This allows lazy evaluation (don't tokenize entire file upfront), but adds complexity. For learning, the vector approach is clearer.


**Checkpoint Tests**:
```rust
#[test]
fn test_lexer_numbers() {
    let mut lexer = Lexer::new("123 456");
    assert_eq!(lexer.next_token(), Ok(Token::Number(123)));
    assert_eq!(lexer.next_token(), Ok(Token::Number(456)));
    assert_eq!(lexer.next_token(), Ok(Token::End));
}

#[test]
fn test_lexer_operators() {
    let mut lexer = Lexer::new("+ - * /");
    assert_eq!(lexer.next_token(), Ok(Token::Plus));
    assert_eq!(lexer.next_token(), Ok(Token::Minus));
    assert_eq!(lexer.next_token(), Ok(Token::Star));
    assert_eq!(lexer.next_token(), Ok(Token::Slash));
}

#[test]
fn test_lexer_expression() {
    let mut lexer = Lexer::new("(2 + 3) * 4");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens, vec![
        Token::LeftParen,
        Token::Number(2),
        Token::Plus,
        Token::Number(3),
        Token::RightParen,
        Token::Star,
        Token::Number(4),
        Token::End,
    ]);
}

#[test]
fn test_lexer_error() {
    let mut lexer = Lexer::new("2 & 3");
    assert!(lexer.tokenize().is_err());
}
```

**Starter Code**:
```rust
#[derive(Debug, PartialEq, Clone)]
enum Token {
    Number(i64),
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,
    RightParen,
    End,
}

struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    fn new(input: &str) -> Self {
        // TODO: Create Lexer with input converted to Vec<char> and position 0
        todo!()
    }

    fn peek(&self) -> Option<char> {
        // TODO: Return the character at current position (or None if at end)
        // Hint: self.input.get(self.position).copied()
        todo!()
    }

    fn advance(&mut self) {
        // TODO: Increment position by 1
        todo!()
    }

    fn skip_whitespace(&mut self) {
        // TODO: Loop while current character is whitespace
        // Hint: Use peek() and ch.is_whitespace(), call advance() for each whitespace
        todo!()
    }

    fn read_number(&mut self) -> i64 {
        // TODO: Build up a number by reading consecutive digits
        // Hint: Start with num = 0, for each digit: num = num * 10 + digit_value
        // Use ch.is_ascii_digit() to check, convert with (ch as i64 - '0' as i64)
        todo!()
    }

    fn next_token(&mut self) -> Result<Token, String> {
        // TODO: Skip whitespace first
        todo!();

        // TODO: Match on peek() to determine token type
        // - None → Token::End
        // - '0'..='9' → Token::Number(self.read_number())
        // - '+' → advance and return Token::Plus
        // - '-' → advance and return Token::Minus
        // - '*' → advance and return Token::Star
        // - '/' → advance and return Token::Slash
        // - '(' → advance and return Token::LeftParen
        // - ')' → advance and return Token::RightParen
        // - anything else → Err with message
        todo!()
    }

    fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        // TODO: Create empty Vec for tokens
        // TODO: Loop calling next_token() until Token::End
        // TODO: Push each token to Vec (including End token), then break
        // TODO: Return Ok(tokens)
        todo!()
    }
}
```


**Check Your Understanding**:
- Why do we skip whitespace?
- How does `read_number()` build up the number?
- What happens if we forget to `advance()` after a token?

---

### Milestone 6: Recursive Descent Parser

**Goal**: Transform the token stream from the lexer into an Abstract Syntax Tree (AST) stored in the arena, respecting operator precedence and parentheses.

**Why This Milestone Matters**:

The parser is the **brain** of the compiler—it understands the **structure** and **meaning** of code. While the lexer breaks text into tokens, the parser answers questions like:
- Does `2 + 3 * 4` mean `(2 + 3) * 4` or `2 + (3 * 4)`? (Answer: second one, multiplication binds tighter)
- Are the parentheses balanced in `((1 + 2) * 3`? (Answer: no, missing closing paren)
- Is `+ + 3` valid? (Answer: no, can't have two operators in a row)

**Complete Pipeline**:

```
Text → Lexer → Tokens → Parser → AST → Evaluator → Result
"2+3*4" →  [Num(2), Plus, Num(3), Star, Num(4)]  →
           BinOp{Add, 2, BinOp{Mul, 3, 4}}  →  14
```

**Recursive Descent Parsing**:

We'll implement a **recursive descent parser**, which means:
1. Each grammar rule becomes a function
2. Functions call each other recursively to match nested structures
3. The call stack mirrors the parse tree structure

This is one of the simplest and most intuitive parsing techniques. Other approaches (LR, LALR, Pratt parsing) are more powerful but complex.

**The Grammar and Operator Precedence**:

Our grammar has **three levels** to encode operator precedence:

```
Expr   → Term (('+' | '-') Term)*      // Lowest precedence: addition/subtraction
Term   → Factor (('*' | '/') Factor)*  // Medium precedence: multiplication/division
Factor → Number | '(' Expr ')'         // Highest precedence: atoms and parens
```

**Why three levels?** This encodes the precedence rules:
- **Factor** (highest): Numbers and parenthesized expressions bind tightest
- **Term** (medium): `*` and `/` bind tighter than `+` and `-`
- **Expr** (lowest): `+` and `-` bind loosest

**How Precedence Works**:

For `2 + 3 * 4`:

```
parse_expr() calls:
  parse_term() for "2"
    parse_factor() returns Literal(2)
  Sees '+', continues
  parse_term() for "3 * 4"
    parse_factor() returns Literal(3)
    Sees '*', continues
    parse_factor() returns Literal(4)
    Returns Mul(3, 4)
  Returns Add(2, Mul(3, 4))
```

Notice: `parse_term()` consumed `3 * 4` as a unit **before** returning to `parse_expr()`. This is how multiplication binds tighter than addition!

**Parsing Strategy for Each Level**:

1. **`parse_expr()`**: Parse a term, then loop consuming `+` or `-` operators
   ```
   2 + 3 - 4  →  Sub(Add(2, 3), 4)
   ```

2. **`parse_term()`**: Parse a factor, then loop consuming `*` or `/` operators
   ```
   2 * 3 / 4  →  Div(Mul(2, 3), 4)
   ```

3. **`parse_factor()`**: Parse atomic elements
   - If number: return literal
   - If `(`: recursively parse expression, expect `)`
   - Otherwise: error

**The Parser State**:

```rust
struct Parser<'arena> {
    tokens: Vec<Token>,           // All tokens from lexer
    position: usize,              // Current position in token stream
    builder: ExprBuilder<'arena>, // For allocating AST nodes in arena
}
```

**Key Operations**:

1. **`peek()`**: Look at current token without advancing
2. **`advance()`**: Move to next token
3. **`expect(token)`**: Verify current token matches expected, advance, or error
4. **`parse_factor()`**: Parse numbers and parenthesized expressions
5. **`parse_term()`**: Parse multiplication and division
6. **`parse_expr()`**: Parse addition and subtraction
7. **`parse()`**: Entry point that parses and verifies we consumed all tokens

**Detailed Example: Parsing `(2 + 3) * 4`**:

```
Tokens: [LeftParen, Number(2), Plus, Number(3), RightParen, Star, Number(4), End]

parse() calls parse_expr():
  parse_expr() calls parse_term():
    parse_term() calls parse_factor():
      See '(' → advance, call parse_expr() recursively:
        parse_expr() calls parse_term():
          parse_term() calls parse_factor():
            See Number(2) → return Literal(2)
          No '*' or '/', return Literal(2)
        See '+', advance, call parse_term():
          parse_term() calls parse_factor():
            See Number(3) → return Literal(3)
          No '*' or '/', return Literal(3)
        Build Add(Literal(2), Literal(3))
      Expect ')' → found it, advance
      Return Add(2, 3)
    See '*', advance, call parse_factor():
      See Number(4) → return Literal(4)
    Build Mul(Add(2, 3), Literal(4))
  No '+' or '-', return Mul(...)
parse() verifies Token::End

Result: Mul(Add(2, 3), 4)
```

**Error Handling**:

The parser must catch:
- **Unexpected tokens**: `2 + + 3` (two operators)
- **Missing operands**: `2 +` (nothing after +)
- **Unbalanced parens**: `(2 + 3` (missing closing paren)
- **Trailing input**: `2 + 3 4` (unexpected 4 at end)

All parse functions return `Result<&'arena Expr<'arena>, String>` to propagate errors.

**Why Recursive Descent?**

**Advantages**:
- **Simple**: Each grammar rule = one function
- **Clear error messages**: Know exactly where parsing failed
- **Debuggable**: Can step through and see call stack
- **Hand-optimizable**: Can add special cases for performance
- **No external tools**: No parser generator needed

**Disadvantages**:
- **Left recursion**: Can't handle grammars like `Expr → Expr '+' Term` (infinite loop)
- **Backtracking**: Inefficient for ambiguous grammars (not our case)
- **Grammar restrictions**: Not all grammars work

**Real-World Recursive Descent Parsers**:

- **Rust compiler**: Uses recursive descent for parsing Rust syntax
- **Go compiler**: Hand-written recursive descent
- **JSON parsers**: Most use recursive descent (simple grammar)
- **Markdown parsers**: Often recursive descent with extensions
- **Configuration languages**: TOML, INI parsers

**The Connection to Arena Allocation**:

Notice: The parser allocates many AST nodes while parsing. With arena allocation:
- Each node: 1 arena bump (~3ns)
- Total for `(2+3)*4`: 5 nodes = ~15ns allocation time
- With Box: 5 mallocs = ~375ns

For complex expressions with hundreds of nodes, the arena speedup is dramatic!

**Grammar**:
```
Expr   → Term (('+' | '-') Term)*
Term   → Factor (('*' | '/') Factor)*
Factor → Number | '(' Expr ')'
```


**Checkpoint Tests**:
```rust
#[test]
fn test_parse_number() {
    assert_eq!(parse_and_eval("42"), Ok(42));
}

#[test]
fn test_parse_addition() {
    assert_eq!(parse_and_eval("2 + 3"), Ok(5));
}

#[test]
fn test_parse_precedence() {
    assert_eq!(parse_and_eval("2 + 3 * 4"), Ok(14)); // Not 20!
}

#[test]
fn test_parse_parentheses() {
    assert_eq!(parse_and_eval("(2 + 3) * 4"), Ok(20));
}

#[test]
fn test_parse_complex() {
    assert_eq!(parse_and_eval("(10 - 5) * 2 + 8 / 4"), Ok(12));
}

#[test]
fn test_parse_nested() {
    assert_eq!(parse_and_eval("((1 + 2) * (3 + 4)) / (5 - 2)"), Ok(7));
}

#[test]
fn test_parse_error() {
    assert!(parse_and_eval("2 + + 3").is_err());
    assert!(parse_and_eval("(2 + 3").is_err());  // Unclosed paren
}
```
**Starter Code**:
```rust
struct Parser<'arena> {
    tokens: Vec<Token>,
    position: usize,
    builder: ExprBuilder<'arena>,
}

impl<'arena> Parser<'arena> {
    fn new(tokens: Vec<Token>, arena: &'arena Arena) -> Self {
        Parser {
            tokens,
            position: 0,
            builder: ExprBuilder::new(arena),
        }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::End)
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if self.peek() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, found {:?}", expected, self.peek()))
        }
    }

    // Factor → Number | '(' Expr ')'
    fn parse_factor(&mut self) -> Result<&'arena Expr<'arena>, String> {
        match self.peek() {
            Token::Number(n) => {
                let n = *n;
                self.advance();
                Ok(self.builder.literal(n))
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            token => Err(format!("Expected number or '(', found {:?}", token)),
        }
    }

    // Term → Factor (('*' | '/') Factor)*
    fn parse_term(&mut self) -> Result<&'arena Expr<'arena>, String> {
        let mut left = self.parse_factor()?;

        loop {
            match self.peek() {
                Token::Star => {
                    self.advance();
                    let right = self.parse_factor()?;
                    left = self.builder.mul(left, right);
                }
                Token::Slash => {
                    self.advance();
                    let right = self.parse_factor()?;
                    left = self.builder.div(left, right);
                }
                _ => break,
            }
        }

        Ok(left)
    }

    // Expr → Term (('+' | '-') Term)*
    fn parse_expr(&mut self) -> Result<&'arena Expr<'arena>, String> {
        // TODO: Similar to parse_term but for + and -
        // Start with parse_term(), then loop handling + and -
        todo!()
    }

    fn parse(&mut self) -> Result<&'arena Expr<'arena>, String> {
        let expr = self.parse_expr()?;
        if self.peek() != &Token::End {
            return Err(format!("Unexpected token: {:?}", self.peek()));
        }
        Ok(expr)
    }
}

// Helper function
fn parse_and_eval(input: &str) -> Result<i64, String> {
    let arena = Arena::new();
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens, &arena);
    let expr = parser.parse()?;
    expr.eval()
}
```

**Check Your Understanding**:
- Why does the grammar have three levels (Expr, Term, Factor)?
- How does this handle operator precedence?
- Why do we parse Factor in Term and Term in Expr?
- When do we create nodes in the arena?

---

### Milestone 7: Performance Comparison

**Goal**: Compare arena allocation vs Box allocation using the implementations from Milestones 2 and 3.

**Benchmark Code**:
```rust
use std::time::Instant;

fn benchmark_arena() {
    let start = Instant::now();
    for _ in 0..10000 {
        let arena = Arena::new();
        // Build expression: (1+2)*(3+4)+(5-2)*7
        let builder = ExprBuilder::new(&arena);
        let expr = builder.add(
            builder.mul(
                builder.add(builder.literal(1), builder.literal(2)),
                builder.add(builder.literal(3), builder.literal(4)),
            ),
            builder.mul(
                builder.sub(builder.literal(5), builder.literal(2)),
                builder.literal(7),
            ),
        );
        let _ = expr.eval();
    }
    let duration = start.elapsed();
    println!("Arena: {:?}", duration);
}

fn benchmark_box() {
    let start = Instant::now();
    for _ in 0..10000 {
        // Build same expression with Box
        let expr = BoxExprBuilder::add(
            BoxExprBuilder::mul(
                BoxExprBuilder::add(BoxExprBuilder::literal(1), BoxExprBuilder::literal(2)),
                BoxExprBuilder::add(BoxExprBuilder::literal(3), BoxExprBuilder::literal(4)),
            ),
            BoxExprBuilder::mul(
                BoxExprBuilder::sub(BoxExprBuilder::literal(5), BoxExprBuilder::literal(2)),
                BoxExprBuilder::literal(7),
            ),
        );
        let _ = expr.eval();
    }
    let duration = start.elapsed();
    println!("Box: {:?}", duration);
}

fn main() {
    println!("Benchmarking expression allocation...");
    benchmark_box();
    benchmark_arena();
}
```

**Expected Results**: Arena should be 5-20x faster depending on expression complexity.

**Check Your Understanding**:
- Why is arena allocation faster?
- When would Box be better than arena?
- What's the memory trade-off?

---

### Complete Project Summary

**What You Built**:
1. AST types with arena lifetimes
2. Bump allocator with proper alignment
3. Expression builder using arena
4. Lexer for tokenization
5. Recursive descent parser
6. Performance comparison

**Key Concepts Practiced**:
- Lifetimes and arena allocation
- Recursive descent parsing
- Unsafe Rust for low-level allocation
- Performance measurement and trade-offs

---

## Project 3: Custom String Interning with Cow Patterns

### Problem Statement

Build a string interning system that stores unique strings once and reuses them. This demonstrates Clone-on-Write (Cow) patterns and zero-copy optimization.

### Why It Matters

**Real-World Impact**: String duplication wastes massive amounts of memory in real programs:

**The String Duplication Problem**:
- Compiler parsing 100K LOC: identifier "count" appears 5,000 times
- Without interning: 5,000 allocations × 6 bytes = **30KB** for one identifier
- With interning: 1 allocation × 6 bytes = **6 bytes**, 5,000 pointers (8 bytes each) = **40KB total**
- But: pointers are often stack-allocated or in structs, actual savings = **29.9KB per repeated identifier**
- Across thousands of identifiers: **Megabytes of savings**

**Real Production Examples**:
- **Rust compiler**: `Symbol` interning saves 40% memory on large codebases
- **Python**: All string literals interned, identifiers interned automatically
- **Java JVM**: String pool for literals, manual `intern()` for runtime strings
- **JavaScript V8**: Symbol table interning for property names
- **Databases**: Column names, table names, SQL keywords interned

**Performance Benefits**:
1. **Memory**: 10-40% reduction in string memory for identifier-heavy workloads
2. **Comparison**: `O(1)` pointer equality vs `O(n)` string comparison
3. **Hashing**: Hash once, reuse hash value (important for HashMaps)
4. **Cache**: Fewer unique strings = better cache locality

**Cow Pattern Benefits**:
- **Zero-copy**: If string already interned, return borrowed reference (no allocation)
- **Lazy allocation**: Only allocate when necessary
- **API clarity**: Caller knows if allocation happened by checking `Cow` variant

### Use Cases

**When you need this pattern**:
1. **Compilers/Interpreters**: Variable names, function names, keywords, string literals
2. **Configuration systems**: Keys in config files (often repeated)
3. **Web frameworks**: Route paths, template variable names, header field names
4. **Databases**: Table/column names, SQL keywords, username strings
5. **Game engines**: Asset names, entity tags, component type names
6. **Logging systems**: Log levels, logger names, common message patterns

**String Interning is Critical When**:
- Many duplicate strings (identifiers in code, repeated log messages)
- String comparison is frequent (symbol table lookups)
- Memory is constrained (embedded systems, large-scale deployments)

**Cow Pattern is Critical When**:
- Processing user input (may or may not need normalization)
- Path manipulation (may or may not need conversion)
- HTML escaping (most strings don't need escaping)

### Learning Goals

- Understand `Cow<'_, T>` and when to use it for zero-copy
- Implement string interning for memory optimization
- Build generational indices for safe handles (no lifetime issues)
- Measure memory savings and performance improvements
- Learn trade-offs: when interning helps vs hurts

---

### Milestone 1: Understand Cow Basics

**Goal**: Learn how `Cow` (Clone-on-Write) works through hands-on examples that demonstrate zero-copy optimization.

**Why This Milestone Matters**:

`Cow<'_, T>` is one of Rust's most elegant patterns for performance optimization. It solves a common dilemma: **"Should my function return a borrowed reference or an owned value?"**

The answer is often: **"It depends on the input!"**

**The Problem `Cow` Solves**:

Imagine writing a function that normalizes whitespace in text. Sometimes the input is already normalized (no work needed), sometimes it needs modification. What should the function signature be?

**Option 1: Always return `String` (always allocate)**
```rust
fn normalize(text: &str) -> String {
    text.replace("  ", " ")  // Always allocates, even if no changes!
}
```
❌ **Problem**: Wastes memory and time when input is already clean (90% of cases)

**Option 2: Return `&str` (never allocate)**
```rust
fn normalize(text: &str) -> &str {
    text  // Can't modify!
}
```
❌ **Problem**: Can't handle cases that need modification

**Option 3: Return `Cow<str>` (allocate only when needed)**
```rust
fn normalize(text: &str) -> Cow<str> {
    if text.contains("  ") {
        Cow::Owned(text.replace("  ", " "))  // Allocate when needed
    } else {
        Cow::Borrowed(text)  // Zero-copy when clean
    }
}
```
✅ **Perfect**: Zero overhead for clean input, handles modifications when needed!

**What is `Cow`?**

`Cow` stands for **Clone-on-Write** (or **Copy-on-Write**). It's an enum with two variants:

```rust
pub enum Cow<'a, B: ?Sized + 'a>
where
    B: ToOwned,
{
    Borrowed(&'a B),  // Borrowed reference (zero-copy)
    Owned(<B as ToOwned>::Owned),  // Owned value (allocated)
}
```

For strings:
- `Cow::Borrowed(&str)` - Points to existing string data
- `Cow::Owned(String)` - Owns heap-allocated string data

**Key Insights**:

1. **Caller's perspective**: `Cow<str>` acts like a string—you can read it, compare it, print it
2. **Zero-copy path**: When no modification needed, return `Cow::Borrowed` (no allocation!)
3. **Allocation path**: When modification needed, return `Cow::Owned` (allocate once)
4. **API clarity**: The type signature tells callers "might allocate, might not"

**Real-World Performance Impact**:

Consider processing 10,000 log lines, normalizing whitespace:
- 9,000 lines already clean (90%)
- 1,000 lines need normalization (10%)

**With `String` return (always allocate)**:
- 10,000 allocations
- ~75ns each = **750,000ns = 0.75ms**

**With `Cow<str>` return (allocate only when needed)**:
- 1,000 allocations (only for dirty lines)
- ~75ns each = **75,000ns = 0.075ms**
- **10x faster!**

**Common `Cow` Use Cases**:

1. **Text processing**: Escaping, normalization, case conversion
   - Most strings don't need escaping → return borrowed
   - HTML special chars found → return owned (escaped version)

2. **Path manipulation**: Canonicalization, directory separators
   - Already canonical → return borrowed
   - Needs normalization → return owned

3. **Configuration loading**: Environment variable expansion
   - No variables like `${FOO}` → return borrowed
   - Variables present → return owned (expanded version)

4. **Data validation**: Trimming, sanitization
   - Already valid → return borrowed
   - Needs fixes → return owned

**The Two Functions You'll Implement**:

1. **`normalize_whitespace(text: &str) -> Cow<str>`**
   - Checks for double spaces or tabs
   - If found: replace with single spaces (allocate)
   - If not found: return original (zero-copy)

2. **`maybe_escape_html(text: &str) -> Cow<str>`**
   - Checks for `<`, `>`, `&` characters
   - If found: escape to `&lt;`, `&gt;`, `&amp;` (allocate)
   - If not found: return original (zero-copy)

**Mental Model**:

Think of `Cow` as:
```rust
// Simplified mental model (not actual code)
match cow {
    Cow::Borrowed(borrowed_ref) => "I'm just pointing to existing data",
    Cow::Owned(owned_string) => "I allocated and own this data",
}
```

**Why Not Just Use `Option<String>`?**

You might think: "Why not return `Option<String>` where `None` means 'use original'?"

```rust
fn normalize(text: &str) -> Option<String> {
    if text.contains("  ") {
        Some(text.replace("  ", " "))
    } else {
        None  // Caller must use original
    }
}
```

**Problems**:
1. Caller must handle two cases manually (match or unwrap_or)
2. Can't use the result uniformly (need to access original separately)
3. Not composable (can't chain operations easily)

`Cow` solves all of these—it's usable as a string in both cases!

**Exercises**:
```rust
use std::borrow::Cow;

// Exercise 1: Function that sometimes modifies input
fn normalize_whitespace(text: &str) -> Cow<str> {
    if text.contains("  ") || text.contains('\t') {
        // Need to modify - return Owned
        let normalized = text.replace("  ", " ").replace('\t', " ");
        Cow::Owned(normalized)
    } else {
        // No modification needed - return Borrowed
        Cow::Borrowed(text)
    }
}

// Exercise 2: Function that might escape HTML
fn maybe_escape_html(text: &str) -> Cow<str> {
    if text.contains('<') || text.contains('>') || text.contains('&') {
        let escaped = text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        Cow::Owned(escaped)
    } else {
        Cow::Borrowed(text)
    }
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_normalize_no_change() {
    let result = normalize_whitespace("hello world");
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "hello world");
}

#[test]
fn test_normalize_with_change() {
    let result = normalize_whitespace("hello  world");
    assert!(matches!(result, Cow::Owned(_)));
    assert_eq!(result, "hello world");
}

#[test]
fn test_escape_no_html() {
    let result = maybe_escape_html("hello");
    assert!(matches!(result, Cow::Borrowed(_)));
}

#[test]
fn test_escape_with_html() {
    let result = maybe_escape_html("<div>");
    assert!(matches!(result, Cow::Owned(_)));
    assert_eq!(result, "&lt;div&gt;");
}
```

**Check Your Understanding**:
- When should you return `Cow::Borrowed` vs `Cow::Owned`?
- What's the benefit of returning `Cow` vs always returning `String`?
- How can the caller use a `Cow<str>`?

---

### 🔄 Why Milestone 1 Isn't Enough → Moving to Milestone 2

**Limitation**: `Cow` shows us *when* to avoid allocations, but doesn't actually *store* strings for reuse. Each call still checks/modifies independently.

**The Real Problem**: Consider processing 1 million log messages, many containing "ERROR: Connection timeout". Without interning:
- Each occurrence: Parse, check, maybe allocate
- No sharing between occurrences
- Memory: Thousands of copies of "ERROR: Connection timeout"

**What we're adding**: **String Interning** - global string pool:
- **HashSet<Box<str>>** - stores unique strings once
- **References** - return `&str` pointing into the set
- **Deduplication** - automatic string reuse

**Improvements**:
- **Memory**: One allocation per unique string (not per occurrence)
- **Comparison**: `ptr::eq()` for equality (vs strcmp)
- **Lifetime**: Strings live as long as interner exists
- **Cost**: HashSet lookup + occasional allocation

**Performance Numbers**:
- Without interning: 1M strings × 25 bytes average = **25MB**
- With interning (10K unique): 10K × 25 bytes = **250KB** (100x savings!)
- Lookup overhead: ~50ns per intern call (hash + comparison)
- Win when: duplicates > ~2x per unique string

---

### Milestone 2: Basic String Interner

**Goal**: Implement a string interner that stores each unique string once and returns references to deduplicated storage.

**Why This Milestone Matters**:

Now that we understand `Cow` for conditional allocation, let's tackle a bigger problem: **string duplication across your entire program**. String interning is a powerful technique that trades lookup time for dramatic memory savings.

**The String Duplication Crisis**:

Consider a compiler parsing a large codebase:
```rust
// file1.rs: variable "count" appears 100 times
// file2.rs: variable "count" appears 150 times
// file3.rs: variable "count" appears 200 times
// ... 50 more files ...
```

**Without interning**:
- 5,000 occurrences of "count"
- 5,000 separate `String` allocations
- 5,000 × (6 bytes + 24 bytes overhead) = **150KB** just for one identifier!
- Multiply by thousands of identifiers = **megabytes wasted**

**With interning**:
- 1 allocation for "count"
- 5,000 references (just pointers)
- **6 bytes total** + pointer overhead
- **Savings: 149.994KB per identifier!**

**What is String Interning?**

String interning is a technique where:
1. **Unique strings stored once**: First occurrence allocates and stores
2. **Duplicates return references**: Subsequent occurrences return pointer to existing storage
3. **Pointer equality works**: Can compare strings with `ptr::eq()` instead of `strcmp()`

**The Core Data Structure**:

```rust
struct StringInterner {
    strings: HashSet<Box<str>>,  // Set of unique strings
}
```

**Why `HashSet<Box<str>>`?**
- **HashSet**: Fast O(1) lookup to check if string already exists
- **Box<str>**: Fixed-size string (not growable like `String`), minimal overhead
- **Not Vec**: Can't use index (strings added in any order, need fast lookup)

**The `intern()` Algorithm**:

```rust
fn intern(&mut self, s: &str) -> &str {
    // 1. Check if string already in set
    if !self.strings.contains(s) {
        // 2. First time seeing this string - allocate and store
        self.strings.insert(Box::from(s));
    }
    // 3. Return reference to the string in the set
    self.strings.get(s).unwrap()
}
```

**Key Insight**: `HashSet::get()` returns a reference to the **stored value**, not the input! This is how we return `&str` with a longer lifetime.

**Lifetime Magic**:

Notice the signature: `fn intern(&mut self, s: &str) -> &str`

The returned `&str` is **not** tied to the input `s`—it's tied to `&mut self`! The string lives in the `HashSet`, so it lives as long as the interner.

```rust
let interner = StringInterner::new();
let interned: &str = interner.intern("hello");
// `interned` lives as long as `interner`, not the string literal
```

**Pointer Equality Optimization**:

With interning, you can compare strings by pointer:

```rust
let s1 = interner.intern("hello");
let s2 = interner.intern("hello");

// Fast pointer comparison (1 CPU cycle)
assert!(std::ptr::eq(s1, s2));

// Slow string comparison (N cycles, where N = string length)
// assert_eq!(s1, s2);  // Not needed anymore!
```

**Real-World Performance**:

**Example: Parsing 100,000 lines of logs**
- 10,000 unique strings
- 100,000 total string occurrences
- Average string length: 25 bytes

**Without interning**:
- Memory: 100,000 × 25 = **2.5MB**
- Allocations: 100,000

**With interning**:
- Memory: 10,000 × 25 = **250KB** (10x savings!)
- Allocations: 10,000 (90% fewer!)
- Cost: 100,000 × 50ns hash lookups = **5ms overhead**

**Trade-off analysis**:
- Win: Memory usage (10x reduction)
- Win: String comparison speed (`ptr::eq` vs `strcmp`)
- Cost: Lookup overhead (~50ns per intern call)
- Cost: Memory lives forever (can't free individual strings)

**When to Use String Interning**:

✅ **Good Use Cases**:
- **Compilers**: Identifiers, keywords, type names
- **Parsers**: Element names, attribute keys
- **Logging**: Log levels, logger names, repeated messages
- **Games**: Asset names, entity tags, component types
- **Databases**: Column names, table names, keywords

❌ **Bad Use Cases**:
- **Unique strings**: User-generated content (comments, messages)
- **Temporary strings**: Short-lived string operations
- **Mutable strings**: Strings that change frequently
- **Memory-constrained**: If 10,000s of unique strings would blow memory

**The Methods You'll Implement**:

1. **`new() -> Self`**: Create empty interner with empty HashSet
2. **`intern(&mut self, s: &str) -> &str`**: Add string to set if new, return reference
3. **`contains(&self, s: &str) -> bool`**: Check if string is interned
4. **`len(&self) -> usize`**: Number of unique strings stored
5. **`total_bytes(&self) -> usize`**: Total bytes used by all strings

**Design Decision: Why Not `HashMap<String, String>`?**

You might think: "Why not `HashMap<String, String>` to map inputs to stored values?"

**Problems**:
1. **Double storage**: Key and value are duplicates (wastes memory)
2. **Double allocation**: Allocates both key and value
3. **Complexity**: HashSet is simpler—we just need uniqueness

HashSet is perfect because the **value itself** is the key (string content determines uniqueness).

**Memory Layout**:

```
StringInterner:
  ├─ HashSet
      ├─ Box<str> "hello" [heap allocation #1: 5 bytes]
      ├─ Box<str> "world" [heap allocation #2: 5 bytes]
      └─ Box<str> "foo"   [heap allocation #3: 3 bytes]

Total: 3 heap allocations, 13 bytes of string data
```

Compare to without interning (if "hello" appears 1000 times):
- 1000 heap allocations
- 5000 bytes of string data (duplicated)


**Checkpoint Tests**:
```rust
#[test]
fn test_intern_basic() {
    let mut interner = StringInterner::new();

    let s1 = interner.intern("hello");
    let s2 = interner.intern("hello");

    // Should be same pointer (no second allocation)
    assert!(std::ptr::eq(s1, s2));
    assert_eq!(interner.len(), 1);
}

#[test]
fn test_intern_different() {
    let mut interner = StringInterner::new();

    let s1 = interner.intern("hello");
    let s2 = interner.intern("world");

    assert!(!std::ptr::eq(s1, s2));
    assert_eq!(interner.len(), 2);
}

#[test]
fn test_contains() {
    let mut interner = StringInterner::new();
    interner.intern("hello");

    assert!(interner.contains("hello"));
    assert!(!interner.contains("world"));
}

#[test]
fn test_total_bytes() {
    let mut interner = StringInterner::new();
    interner.intern("hi");     // 2 bytes
    interner.intern("hello");  // 5 bytes

    assert_eq!(interner.total_bytes(), 7);
}
```
**Starter Code**:
```rust
use std::collections::HashSet;

struct StringInterner {
    strings: HashSet<Box<str>>,
}

impl StringInterner {
    fn new() -> Self {
        // TODO: Create new StringInterner with empty HashSet
        todo!()
    }

    fn intern(&mut self, s: &str) -> &str {
        // TODO: Check if string already interned using contains()
        if todo!("Check if !self.strings.contains(s)") {
            // TODO: Insert Box::from(s) into self.strings
            todo!();
        }

        // TODO: Get reference to the interned string from HashSet
        // Hint: self.strings.get(s).unwrap()
        // This works because we just inserted it if it wasn't there
        todo!()
    }

    fn contains(&self, s: &str) -> bool {
        // TODO: Check if strings HashSet contains s
        todo!()
    }

    fn len(&self) -> usize {
        // TODO: Return length of strings HashSet
        todo!()
    }

    fn total_bytes(&self) -> usize {
        // TODO: Sum up the length of all strings
        // Hint: self.strings.iter().map(|s| s.len()).sum()
        todo!()
    }
}
```

**Check Your Understanding**:
- Why do we use `Box<str>` instead of `String`?
- Why can we return `&str` from intern even though it takes `&mut self`?
- What makes the pointers equal for the same string?

---

### Milestone 3: Add Cow-based API

**Goal**: Combine the `Cow` pattern from Milestone 1 with the interner from Milestone 2 to create an API that communicates allocation status.

**Why This Milestone Matters**:

In Milestone 1, we learned that `Cow` communicates **"did we allocate or not?"** to the caller. In Milestone 2, we built an interner but lost that information—`intern()` always returns `&str`, hiding whether allocation happened.

Let's bring these concepts together!

**The Problem with `intern()`**:

```rust
let s1 = interner.intern("hello");  // First time - allocates
let s2 = interner.intern("hello");  // Already there - no allocation
```

Both calls return `&str`, so the caller can't tell which one allocated. In performance-critical code, you might want to know:
- **For logging**: "Interned 1000 strings, 900 were hits (no allocation), 100 were misses"
- **For debugging**: Track allocation rate to optimize string reuse
- **For statistics**: Measure interner effectiveness

**The Solution: `get_or_intern()`**:

```rust
fn get_or_intern(&mut self, s: &str) -> Cow<str> {
    if self.contains(s) {
        Cow::Borrowed(self.strings.get(s).unwrap())  // Already there
    } else {
        self.strings.insert(Box::from(s));
        Cow::Borrowed(self.strings.get(s).unwrap())  // Just inserted
    }
}
```

**Wait, why always `Cow::Borrowed`?**

Good question! You might expect:
```rust
// Intuitive but WRONG approach
fn get_or_intern(&mut self, s: &str) -> Cow<str> {
    if self.contains(s) {
        Cow::Borrowed(self.strings.get(s).unwrap())
    } else {
        Cow::Owned(s.to_string())  // ❌ Wrong!
    }
}
```

**Why this is wrong**: The interner's job is to **store and return references to stored strings**. If we return `Cow::Owned(String)`, the string isn't in the interner—it's owned by the caller! That defeats the purpose.

**The Correct Pattern**:

Actually, for a string interner, `get_or_intern()` should **always** return `Cow::Borrowed` because:
1. Already interned → borrow from HashSet
2. Not interned → insert, then borrow from HashSet

The `Cow` variant isn't the right way to communicate allocation here (it's always `Borrowed`). In the next milestone, we'll add explicit statistics tracking instead.

**So Why Implement This?**

This milestone teaches:
1. **Combining patterns**: How `Cow` and interning interact
2. **API design**: Understanding when `Cow` makes sense vs statistics
3. **Lifetime reasoning**: How references from HashSet work with `Cow`
4. **The limitation**: Recognizing that `Cow` isn't perfect for all cases

**Alternative Design: Return `bool` for "was new"?**

```rust
fn intern_with_info(&mut self, s: &str) -> (&str, bool) {
    let was_new = !self.contains(s);
    if was_new {
        self.strings.insert(Box::from(s));
    }
    (self.strings.get(s).unwrap(), was_new)
}
```

This is actually more useful! But we'll go with the `Cow` approach to practice the pattern, then move to explicit statistics in Milestone 4.

**Real-World Usage**:

In production code, you typically see one of:
1. **Simple API**: Just `intern()` returning `&str` (no allocation info)
2. **Statistics API**: Separate methods for hit/miss counts
3. **Hybrid**: `intern()` for normal use, `statistics()` method for monitoring

The `Cow`-based API is rare for interners because both paths allocate internally (into the HashSet) even if they don't allocate a `String`.

**Key Takeaway**:

This milestone illustrates that **`Cow` isn't always the right tool**. It's perfect for "maybe modify the input" but awkward for "maybe store the input." This prepares you for Milestone 4's better solution: explicit statistics tracking.

**Add Method**:
```rust
impl StringInterner {
    fn get_or_intern(&mut self, s: &str) -> Cow<str> {
        // TODO: Check if string is already interned
        if todo!("self.contains(s)") {
            // TODO: Return Cow::Borrowed with reference from HashSet
            // Hint: Cow::Borrowed(self.strings.get(s).unwrap())
            todo!()
        } else {
            // TODO: Insert the string into HashSet
            todo!();
            // TODO: Return Cow::Borrowed with reference to newly inserted string
            todo!()
        }
    }
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_cow_already_interned() {
    let mut interner = StringInterner::new();
    interner.intern("hello");

    let result = interner.get_or_intern("hello");
    assert!(matches!(result, Cow::Borrowed(_)));
}

#[test]
fn test_cow_new_string() {
    let mut interner = StringInterner::new();

    let result = interner.get_or_intern("hello");
    // First time still returns Borrowed after interning
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(interner.len(), 1);
}
```

**Check Your Understanding**:
- Why does `get_or_intern` always return `Cow::Borrowed`?
- When would it return `Cow::Owned`?
- How does this API communicate whether allocation happened?

---

### Milestone 4: Add Statistics Tracking

**Goal**: Add comprehensive statistics to measure interner effectiveness and understand allocation patterns.

**Why This Milestone Matters**:

As we learned in Milestone 3, `Cow` isn't the ideal way to track string interner performance. What we really need is **aggregate statistics** that answer questions like:

- **Is the interner effective?** High hit rate (lookups/total) = good reuse!
- **Should we use interning?** If allocation rate is too low, overhead might not be worth it
- **Memory saved**: Compare `total_bytes` vs. `(allocations + lookups) × average_length`
- **Performance tuning**: Identify which strings are duplicated most

**The Problem Without Statistics**:

```rust
let mut interner = StringInterner::new();
// ... 10,000 intern calls later ...
// ❓ How many were duplicates?
// ❓ How much memory did we save?
// ❓ Is the interner actually helping?
```

Without stats, you're flying blind. You don't know if the interner is paying for itself!

**What We're Adding: `InternerStats` struct**:

```rust
struct InternerStats {
    total_strings: usize,   // Unique strings currently stored
    total_bytes: usize,     // Total bytes used by strings
    allocations: usize,     // How many new strings added
    lookups: usize,         // How many duplicate strings found
}
```

**Key Metrics**:

1. **Hit Rate**: `lookups / (allocations + lookups)`
   - High hit rate (>50%) = interner is valuable
   - Low hit rate (<10%) = mostly unique strings, overhead may not be worth it

2. **Memory Efficiency**: Compare actual memory vs. without interning
   - Without: `(allocations + lookups) × average_string_length`
   - With: `total_bytes` (only unique strings)
   - Savings: `(without - with) / without × 100%`

3. **Allocation Ratio**: `allocations / total_calls`
   - Low ratio = lots of reuse (good for interning)
   - High ratio = mostly unique (bad for interning)

**Real-World Example: Web Server Logs**:

Imagine processing 100,000 HTTP log entries:
```
GET /api/users 200
GET /api/users 200
GET /api/posts 404
GET /api/users 200
...
```

**Expected pattern**:
- 10,000 unique strings (paths, status codes, methods)
- 100,000 total strings
- Hit rate: 90% (90,000 lookups, 10,000 allocations)

**Statistics would show**:
```rust
InternerStats {
    total_strings: 10_000,     // 10K unique
    total_bytes: 250_000,      // ~25 bytes average
    allocations: 10_000,       // 10K new strings
    lookups: 90_000,           // 90K duplicates found!
}
```

**Analysis**:
- **Hit rate**: 90,000 / 100,000 = 90% ✅ Excellent!
- **Memory without interning**: 100,000 × 25 = 2.5MB
- **Memory with interning**: 250KB
- **Savings**: (2.5MB - 250KB) / 2.5MB = **90% memory saved!** 🎉

**When Statistics Show Interning Is NOT Worth It**:

```rust
// Processing unique user comments (no duplicates)
InternerStats {
    total_strings: 100_000,
    total_bytes: 5_000_000,    // 50 bytes average
    allocations: 100_000,      // Every string is new
    lookups: 0,                // No hits!
}
```

**Analysis**:
- **Hit rate**: 0% ❌ Terrible!
- **Overhead**: Hash computation + HashSet storage + lookup time
- **Verdict**: Remove the interner, just use `String` directly

**Implementing Statistics**:

The stats need to be updated in `intern()`:

```rust
fn intern(&mut self, s: &str) -> &str {
    if !self.strings.contains(s) {
        // New string - record allocation
        self.strings.insert(Box::from(s));
        self.stats.total_strings += 1;
        self.stats.total_bytes += s.len();
        self.stats.allocations += 1;
    } else {
        // Duplicate - record lookup
        self.stats.lookups += 1;
    }
    self.strings.get(s).unwrap()
}
```

**Production Monitoring**:

In real systems, you'd export these stats to monitoring:

```rust
// Export to Prometheus
gauge!("interner.total_strings", interner.stats.total_strings as f64);
gauge!("interner.total_bytes", interner.stats.total_bytes as f64);
counter!("interner.allocations", interner.stats.allocations as u64);
counter!("interner.lookups", interner.stats.lookups as u64);
```

This lets you graph hit rate over time, alert on low efficiency, etc.

**Why Track Both `total_strings` AND `allocations`?**

Good question! They're usually equal, but might differ if you add a `clear()` or `remove()` method:

```rust
interner.intern("hello");  // allocations=1, total_strings=1
interner.clear();          // allocations=1, total_strings=0 (cleared!)
interner.intern("world");  // allocations=2, total_strings=1
```

`allocations` is the **lifetime total**, `total_strings` is the **current count**.

**Performance Cost of Statistics**:

Adding stats is cheap:
- Increment counters: ~1ns each (just memory writes)
- String length: already computed for HashSet
- No allocations, no complex computation

The cost is negligible compared to the HashSet lookup (~50ns).

**Alternative: Separate `StatsInterner` Type?**

Some designs use generics to make statistics optional:

```rust
struct StringInterner<S = NoStats> {
    strings: HashSet<Box<str>>,
    stats: S,
}
```

This avoids overhead for users who don't need stats, but adds API complexity. For learning, we'll just always include stats (overhead is tiny anyway).


**Checkpoint Tests**:
```rust
#[test]
fn test_stats() {
    let mut interner = StringInterner::new();

    interner.intern("hello");  // allocation
    interner.intern("world");  // allocation
    interner.intern("hello");  // lookup

    let stats = interner.statistics();
    assert_eq!(stats.total_strings, 2);
    assert_eq!(stats.total_bytes, 10);  // 5 + 5
    assert_eq!(stats.allocations, 2);
    assert_eq!(stats.lookups, 1);
}

#[test]
fn test_stats_empty() {
    let interner = StringInterner::new();
    let stats = interner.statistics();
    assert_eq!(stats.total_strings, 0);
    assert_eq!(stats.allocations, 0);
}
```
**Starter Code**:
```rust
#[derive(Debug, PartialEq)]
struct InternerStats {
    total_strings: usize,
    total_bytes: usize,
    allocations: usize,  // How many times we allocated
    lookups: usize,      // How many times we just returned existing
}

impl StringInterner {
    fn new() -> Self {
        // TODO: Create StringInterner with empty HashSet and zero stats
        todo!()
    }

    fn intern(&mut self, s: &str) -> &str {
        // TODO: Check if string is not already interned
        if todo!("!self.strings.contains(s)") {
            // TODO: Insert string into HashSet
            todo!();
            // TODO: Update statistics: increment total_strings, add to total_bytes, increment allocations
            todo!();
        } else {
            // TODO: Increment lookups count (string was already interned)
            todo!();
        }

        // TODO: Return reference to interned string
        todo!()
    }

    fn statistics(&self) -> &InternerStats {
        // TODO: Return reference to stats
        todo!()
    }
}
```

**Check Your Understanding**:
- Why track both allocations and lookups?
- How does this help evaluate interner effectiveness?

---

### 🔄 Why Milestone 4 Isn't Enough → Moving to Milestone 5

**Critical Limitation**: Lifetimes! Returning `&str` from intern ties all references to the interner's lifetime. This causes problems:

```rust
let s: &str;
{
    let mut interner = StringInterner::new();
    s = interner.intern("hello");  // ❌ Error: s outlives interner
}
println!("{}", s);  // Dangling reference!
```

**Real-world pain points**:
- Can't store interned strings in long-lived structs without holding interner reference
- Compiler fights you with lifetime errors
- `&'static str` doesn't work for runtime strings

**What we're adding**: **Generational Indices** (AKA "slot map" pattern):
- **Symbol** handle: `{index: usize, generation: u32}` - Copy, 'static
- **Indirection**: Symbol → lookup in Vec → get string
- **Stale detection**: Generation mismatch = invalid symbol

**Improvements**:
- **Lifetime freedom**: Symbols are `Copy` + `'static`, store anywhere
- **Safety**: Stale symbols return `None` (not dangling pointers)
- **Memory reuse**: Freed slots recycled with incremented generation
- **Cost**: Extra indirection (Vec lookup) ~2-3ns

**Comparison**:
- `&'a str` approach: Zero runtime cost, lifetime complexity
- `Symbol` approach: Small runtime cost, no lifetime complexity
- **Choose Symbol when**: Need flexibility, store in multiple places, serialize/deserialize

**Real-world usage**:
- Game engines (entity IDs)
- GUI frameworks (widget handles)
- Compilers (symbol table indices)
- Databases (row IDs with generation for MVCC)

**Memory layout**:
- Symbol: 12 bytes (8 byte index + 4 byte generation)
- Reference: 8 bytes (just pointer)
- Trade-off: 50% more memory per handle, but no lifetime constraints

---

### Milestone 5: Symbol-Based Access with Generational Indices

**Goal**: Replace lifetime-bound references with `Copy` handles that work anywhere, using the generational index pattern to detect stale handles safely.

**Why This Milestone Matters**:

Our interner from Milestone 4 has a critical flaw: **lifetime hell**. Every interned string reference is tied to the interner's lifetime, making it nearly impossible to use in real applications.

**The Lifetime Problem**:

```rust
struct Compiler<'intern> {
    identifiers: Vec<&'intern str>,  // ❌ Lifetime everywhere!
    interner: &'intern StringInterner,  // ❌ Must hold reference
}

// Can't return identifiers without dragging 'intern lifetime along
fn parse<'intern>(source: &str, interner: &'intern mut StringInterner)
    -> Result<Vec<&'intern str>, Error> {  // ❌ Lifetime infected return type!
    // ...
}
```

**The pain gets worse**:
- Can't store identifiers in one struct and interner in another
- Can't serialize/deserialize (references can't be saved to disk)
- Can't send between threads easily (lifetimes don't cross thread boundaries cleanly)
- Can't build self-referential structures (compiler forbids them)

**The Solution: Handles Instead of References**:

Instead of returning `&str` (with lifetime), return a `Symbol` handle (no lifetime):

```rust
#[derive(Copy, Clone, PartialEq)]
struct Symbol {
    index: usize,      // Which slot in the interner?
    generation: u32,   // Which version of that slot?
}
```

Now your code looks like:

```rust
struct Compiler {
    identifiers: Vec<Symbol>,  // ✅ No lifetime!
    interner: SymbolInterner,  // ✅ Can own it
}

fn parse(source: &str, interner: &mut SymbolInterner) -> Result<Vec<Symbol>, Error> {
    // ✅ No lifetimes in return type!
}
```

**What Are Generational Indices?**

Generational indices (also called "slot maps" or "generational arena") solve two problems:

1. **Stable handles**: Index stays valid even if other items are removed
2. **Dangling detection**: Generation number catches stale references

**The Core Idea**:

```rust
struct Slot {
    string: Option<Box<str>>,  // None = slot is free
    generation: u32,           // Incremented each time slot is reused
}

struct SymbolInterner {
    slots: Vec<Slot>,           // All slots (some filled, some free)
    free_list: Vec<usize>,      // Indices of free slots to reuse
}
```

**How It Works**:

1. **Allocate**: Find free slot (or create new one), store string, return `Symbol{index, generation}`
2. **Resolve**: Look up `slots[index]`, check generation matches, return `&str` or `None`
3. **Remove**: Set `slots[index].string = None`, increment generation, add index to free list
4. **Reuse**: Next allocation reuses freed slot with new generation number

**Example Walkthrough**:

```rust
let mut interner = SymbolInterner::new();

// 1. Intern "hello" → creates slot 0
let sym1 = interner.intern("hello");  // Symbol{index: 0, generation: 0}
assert_eq!(interner.resolve(sym1), Some("hello"));

// 2. Remove "hello" → frees slot 0, increments generation
interner.remove(sym1);
// slots[0] = Slot{string: None, generation: 1}
// free_list = [0]

// 3. Try to resolve old symbol → generation mismatch!
assert_eq!(interner.resolve(sym1), None);  // sym1 has gen=0, slot has gen=1

// 4. Intern "world" → reuses slot 0 with new generation
let sym2 = interner.intern("world");  // Symbol{index: 0, generation: 1}
assert_eq!(interner.resolve(sym2), Some("world"));

// 5. Old symbol still doesn't work
assert_eq!(interner.resolve(sym1), None);  // Still stale!
```

**Why Generations?**

Without generations, you'd have a classic "dangling pointer" bug:

```rust
// Without generations (BAD!)
let sym1 = interner.intern("hello");  // Symbol{index: 0}
interner.remove(sym1);
let sym2 = interner.intern("world");  // Reuses slot 0

// BUG: sym1 resolves to "world" instead of None!
assert_eq!(interner.resolve(sym1), Some("world"));  // ❌ Wrong string!
```

With generations, stale symbols return `None` safely:

```rust
// With generations (GOOD!)
let sym1 = interner.intern("hello");  // Symbol{index: 0, gen: 0}
interner.remove(sym1);                 // Slot becomes {None, gen: 1}
let sym2 = interner.intern("world");  // Symbol{index: 0, gen: 1}

assert_eq!(interner.resolve(sym1), None);       // ✅ Detects stale!
assert_eq!(interner.resolve(sym2), Some("world"));  // ✅ Correct!
```

**Memory Layout**:

```
SymbolInterner:
  slots: [
    Slot{string: Some("hello"), generation: 0},   // index 0
    Slot{string: None, generation: 3},            // index 1 (freed 3 times)
    Slot{string: Some("world"), generation: 0},   // index 2
  ]
  free_list: [1]  // Slot 1 is available for reuse
```

**Performance Trade-Offs**:

| Aspect | `&str` Approach | `Symbol` Approach |
|--------|----------------|-------------------|
| **Resolve speed** | Direct pointer dereference (~1ns) | Vec lookup + generation check (~3ns) |
| **Handle size** | 8 bytes (pointer) | 12 bytes (index + generation) |
| **Lifetime complexity** | High (infects everything) | Zero (Copy, 'static) |
| **Safety** | Compiler enforced | Runtime checks |
| **Serialization** | Impossible | Easy (just two numbers) |
| **Thread safety** | Complex (lifetime bounds) | Simple (Copy, Send, Sync) |

**When to Use Which**:

✅ **Use `Symbol` (generational index) when**:
- Need to store in multiple places
- Need to serialize/deserialize
- Want to avoid lifetime annotations everywhere
- Building complex data structures (graphs, trees)
- Working with concurrent code

✅ **Use `&str` (reference) when**:
- Short-lived, local usage only
- Performance-critical tight loop (avoid indirection)
- Simple codebase where lifetimes aren't a burden

**Real-World Examples**:

1. **Rust compiler**: Uses `Symbol` for identifiers (from `rustc_span::symbol`)
   - 100,000s of identifiers across compilation
   - Stored in AST nodes, type tables, name resolution tables
   - Serialized to incremental compilation cache

2. **Game engines (Bevy, Amethyst)**: Entity IDs are generational indices
   - Entities can be despawned and IDs reused
   - Systems store entity references without lifetimes
   - Generation catches "use after despawn" bugs

3. **GUI frameworks (Druid, Iced)**: Widget IDs
   - Widgets destroyed and recreated frequently
   - Event handlers store widget IDs across frames
   - Stale IDs safely ignored

**Implementation Strategy**:

1. **`intern(s: &str) -> Symbol`**:
   - Check if string already exists (linear search through slots)
   - If found: return Symbol with that index/generation
   - If not found:
     - Try `free_list.pop()` for reusable slot
     - Otherwise push new slot
     - Return Symbol

2. **`resolve(symbol: Symbol) -> Option<&str>`**:
   - Look up `slots[symbol.index]`
   - Check `slot.generation == symbol.generation`
   - If match: return `Some(&string)`
   - If mismatch: return `None` (stale)

3. **`remove(symbol: Symbol)`**:
   - Check generation matches
   - Set `slot.string = None`
   - Increment `slot.generation`
   - Push index to `free_list`

**Optimization: HashMap for Fast Lookup**:

Our starter code uses linear search (slow for many strings). Production code would add:

```rust
struct SymbolInterner {
    slots: Vec<Slot>,
    free_list: Vec<usize>,
    lookup: HashMap<Box<str>, Symbol>,  // Fast find by string content
}
```

This makes `intern()` O(1) instead of O(n), but we'll skip it for simplicity.

**Starter Code**:
```rust
#[derive(Debug, Copy, Clone, PartialEq)]
struct Symbol {
    index: usize,
    generation: u32,
}

struct Slot {
    string: Option<Box<str>>,
    generation: u32,
}

struct SymbolInterner {
    slots: Vec<Slot>,
    free_list: Vec<usize>,
}

impl SymbolInterner {
    fn new() -> Self {
        // TODO: Create SymbolInterner with empty slots and free_list
        todo!()
    }

    fn intern(&mut self, s: &str) -> Symbol {
        // TODO: Check if string already exists in slots
        // Hint: Loop through slots, check if slot.string matches s
        for (index, slot) in self.slots.iter().enumerate() {
            if let Some(existing) = &slot.string {
                if existing.as_ref() == s {
                    // TODO: Return Symbol with this index and generation
                    todo!()
                }
            }
        }

        // Not found - allocate new slot
        // TODO: Check if there's a free slot to reuse
        if let Some(index) = self.free_list.pop() {
            // TODO: Reuse freed slot
            // - Get mutable reference to slot at index
            // - Increment generation
            // - Set string to Some(Box::from(s))
            // - Return Symbol with index and new generation
            todo!()
        } else {
            // TODO: Allocate new slot at end of Vec
            // - Get index (current slots.len())
            // - Push new Slot with string and generation 0
            // - Return Symbol with index and generation 0
            todo!()
        }
    }

    fn resolve(&self, symbol: Symbol) -> Option<&str> {
        // TODO: Get slot at symbol.index
        // TODO: Check if generation matches
        // TODO: If matches, return string as Option<&str>, else None
        // Hint: self.slots.get(symbol.index).and_then(|slot| ...)
        todo!()
    }

    fn remove(&mut self, symbol: Symbol) {
        // TODO: Get mutable reference to slot at symbol.index
        // TODO: Check if generation matches
        // TODO: If matches, set string to None and push index to free_list
        todo!()
    }

    fn clear(&mut self) {
        // TODO: Iterate through all slots
        // TODO: For each slot with a string:
        //   - Set string to None
        //   - Increment generation
        //   - Push index to free_list
        todo!()
    }
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_symbol_intern() {
    let mut interner = SymbolInterner::new();

    let sym1 = interner.intern("hello");
    let sym2 = interner.intern("hello");

    // Same string should have same symbol
    assert_eq!(sym1, sym2);
    assert_eq!(interner.resolve(sym1), Some("hello"));
}

#[test]
fn test_symbol_resolve() {
    let mut interner = SymbolInterner::new();

    let sym = interner.intern("test");
    assert_eq!(interner.resolve(sym), Some("test"));
}

#[test]
fn test_stale_symbol() {
    let mut interner = SymbolInterner::new();

    let sym1 = interner.intern("test");
    interner.clear();

    // sym1 is now stale
    assert_eq!(interner.resolve(sym1), None);
}

#[test]
fn test_generation_reuse() {
    let mut interner = SymbolInterner::new();

    let sym1 = interner.intern("test");
    let index1 = sym1.index;
    let gen1 = sym1.generation;

    interner.remove(sym1);

    // Interning again should reuse slot but increment generation
    let sym2 = interner.intern("test");
    assert_eq!(sym2.index, index1);  // Same slot
    assert_ne!(sym2.generation, gen1);  // Different generation
}

#[test]
fn test_symbol_lifetime_safety() {
    let mut interner = SymbolInterner::new();
    let sym = interner.intern("test");

    // Symbol can outlive the borrow of interner
    drop(interner);

    // This is safe - we just can't resolve it anymore
    let _copy = sym;  // Symbol is Copy
}
```

**Check Your Understanding**:
- Why use symbols instead of direct string references?
- How do generational indices detect stale references?
- What's the advantage of reusing slots with free_list?
- Why is Symbol Copy but still safe?

---

### Milestone 6: Performance Comparison

**Goal**: Measure the benefit of interning.

**Benchmark Code**:
```rust
use std::time::Instant;

fn benchmark_with_interner() {
    let mut interner = StringInterner::new();
    let words = vec!["hello", "world", "foo", "bar", "hello", "world"];

    let start = Instant::now();
    for _ in 0..100000 {
        for word in &words {
            let _ = interner.intern(word);
        }
    }
    let duration = start.elapsed();

    let stats = interner.statistics();
    println!("With interner: {:?}", duration);
    println!("Stats: {:?}", stats);
}

fn benchmark_without_interner() {
    let words = vec!["hello", "world", "foo", "bar", "hello", "world"];
    let mut strings = Vec::new();

    let start = Instant::now();
    for _ in 0..100000 {
        for word in &words {
            strings.push(word.to_string());  // Always allocate
        }
    }
    let duration = start.elapsed();

    println!("Without interner: {:?}", duration);
    println!("Allocations: {}", strings.len());
}
```

**Expected Results**: Interner should be much faster for duplicate-heavy workloads and use significantly less memory.

**Check Your Understanding**:
- When does interning help most?
- When might interning hurt performance?
- What's the memory trade-off?

---

### Complete Project Summary

**What You Built**:
1. Understanding of `Cow<T>` for zero-copy patterns
2. Basic string interner with HashSet
3. Statistics tracking for allocations
4. Symbol-based access with generational indices
5. Performance comparisons

**Key Concepts Practiced**:
- Clone-on-Write patterns
- String interning benefits
- Generational indices for safe handles
- Trade-offs between copying and interning

---

## Final Review Questions

After completing all three projects, review these concepts:

### Memory & Ownership Patterns

1. **Interior Mutability**:
   - When would you use `Cell` vs `RefCell` vs `Mutex` vs `RwLock`?
   - What's the runtime cost of each?
   - Why can interior mutability panic?

2. **Arena Allocation**:
   - What workloads benefit most from arena allocation?
   - What's the trade-off of arena vs individual allocations?
   - When does arena allocation hurt performance?

3. **Cow Patterns**:
   - When should a function return `Cow<T>` vs `T` vs `&T`?
   - How does `Cow` enable zero-copy optimization?
   - What's the caller's responsibility when receiving `Cow`?

4. **Lifetimes**:
   - Why do arena-allocated objects need lifetime annotations?
   - How do generational indices avoid lifetime issues?
   - When are lifetimes better than indices?

### Design Patterns

1. **When to use what**:
   - Single-threaded mutation: `RefCell`
   - Multi-threaded mutation: `Mutex` or `RwLock`
   - Bulk allocation/deallocation: `Arena`
   - Avoiding duplicate allocations: `Cow` or string interning
   - Stable handles: generational indices

2. **Performance Characteristics**:
   - `Cell::get/set`: zero cost
   - `RefCell`: runtime check overhead
   - `Mutex`: OS lock overhead + contention
   - `RwLock`: higher overhead than `Mutex`, but allows concurrent reads
   - Arena: extremely fast allocation, but can't free individual items

### Common Pitfalls

1. **Don't hold RefCell borrows across function calls** - causes panics
2. **Don't use `Arc<Mutex<T>>` for single-threaded code** - unnecessary overhead
3. **Don't intern everything** - has its own costs
4. **Don't ignore lock scope** - minimizing critical sections is important
5. **Don't assume arena is always faster** - measure for your workload

---

## Next Steps

- Implement the additional challenges from each project
- Read the corresponding chapter sections again
- Try combining patterns (e.g., thread-safe arena allocator)
- Profile your implementations to understand costs
- Explore real-world codebases using these patterns (Rust compiler, game engines, etc.)
