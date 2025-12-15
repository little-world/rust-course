# Project 1: In-Memory Analytics Engine with Entry API

## Problem Statement

Build an in-memory analytics engine that processes streaming events (page views, clicks, purchases) and maintains real-time statistics using efficient HashMap patterns. The engine should handle millions of events per second, compute aggregates per user/product/category, and provide instant query results.

Your analytics engine should:
- Track metrics per dimension (user, product, category, time bucket)
- Compute running statistics (count, sum, average, min, max)
- Support efficient increment/update operations using Entry API
- Group events by multiple dimensions simultaneously
- Handle high-throughput event streams (1M+ events/sec)
- Query metrics by any dimension combination

Example events:
```rust
Event { user_id: "user123", product: "laptop", category: "electronics", value: 1299.99, timestamp: ... }
```

Queries:
- "Total revenue by category"
- "Top 10 users by purchase count"
- "Average transaction value per product"

## Why It Matters

Real-time analytics require efficient incremental updates. Naive approaches using `contains_key()` + `get_mut()` perform 2 hash lookups per event. The Entry API reduces this to 1 lookup, providing 2-3x throughput improvement. For systems processing millions of events/second, this difference determines whether you need 10 servers or 5.

This pattern is fundamental to: metrics aggregation (Prometheus, InfluxDB), session tracking, real-time dashboards, fraud detection, A/B testing analytics.

## Use Cases

- Real-time analytics dashboards
- User behavior tracking (session analytics, funnel analysis)
- E-commerce metrics (revenue tracking, inventory analytics)
- Application performance monitoring (APM)
- Fraud detection (transaction pattern analysis)
- A/B testing platforms

---

## Introduction to HashMap and Entry API Concepts

HashMaps are the workhorse of data aggregation, yet naive usage patterns can severely limit performance. Understanding Rust's Entry API—which eliminates redundant hash lookups—is essential for building high-throughput systems. The difference between checking-then-inserting versus using the Entry API can determine whether your system handles 100K or 1M events per second.

### 1. HashMap Fundamentals and Hash Functions

HashMap provides O(1) average-case lookup, insert, and delete through hashing:

**How It Works**:
```rust
// Key → Hash → Bucket Index → Value
let key = "user123";
let hash = hash_function(key);        // e.g., 0x8a3f9c2b
let index = hash % bucket_count;      // e.g., 43
let value = buckets[index];           // Get value from bucket 43
```

**Collision Handling** (when two keys hash to same bucket):
- **Separate Chaining**: Each bucket is a linked list of entries
- **Open Addressing**: Search for next empty bucket (linear probing)

Rust's HashMap uses a variant of **Robin Hood hashing** (open addressing with backward shift deletion).

**Hash Function Quality**:
```rust
// Good hash: evenly distributes keys
hash("user1") → 42
hash("user2") → 173
hash("user3") → 89

// Bad hash: clusters keys
hash("user1") → 10
hash("user2") → 10  // Collision!
hash("user3") → 11  // Clustering
```

**Why This Matters**: Poor hash distribution causes collisions → linear search within buckets → O(n) worst case instead of O(1).

### 2. The Double-Lookup Problem

Naive HashMap usage performs redundant lookups:

**Naive Pattern** (2 hash lookups):
```rust
if !map.contains_key(&key) {  // Lookup 1: compute hash, find bucket
    map.insert(key, 0);        // Lookup 2: compute hash again, find bucket
}
let count = map.get_mut(&key).unwrap();  // Lookup 3!
*count += 1;
```

**Cost Analysis** (1M increments):
- Hash computation: 3M hash operations
- Bucket lookups: 3M bucket searches
- Total overhead: 200% extra work

**Why It's Slow**: Hashing is expensive (string hashing = 10-100 CPU cycles per byte). Repeating it 3× triples the cost.

### 3. Entry API for Single-Lookup Operations

The Entry API combines check-and-modify into a single hash lookup:

**Entry Pattern** (1 hash lookup):
```rust
*map.entry(key).or_insert(0) += 1;
// Single hash computation, single bucket lookup, in-place update
```

**How It Works Internally**:
```rust
// Conceptual implementation
match map.entry(key) {
    Occupied(entry) => {
        // Key exists, entry holds mutable reference to value
        *entry.get_mut() += 1;
    }
    Vacant(entry) => {
        // Key absent, entry can insert
        entry.insert(0);
    }
}
```

**Performance Impact**:
- Naive: 3 hash operations = ~300 cycles
- Entry API: 1 hash operation = ~100 cycles (3× faster)

For 1M operations: 300M cycles vs 100M cycles = **2 seconds saved** on a 100MHz processor.

### 4. Entry API Variants

Rust provides multiple Entry API methods for different use cases:

**`or_insert(default)`**: Insert if absent, return mutable reference
```rust
let count = map.entry(key).or_insert(0);
*count += 1;
```

**`or_insert_with(|| default)`**: Lazy initialization (closure called only if absent)
```rust
map.entry(key).or_insert_with(|| expensive_computation());
// Computation only runs if key is absent
```

**`or_default()`**: Insert `T::default()` if absent
```rust
let vec = map.entry(key).or_default();  // T = Vec<i32>, default = empty vec
vec.push(value);
```

**`and_modify(|v| ...)`: Update if present
```rust
map.entry(key)
    .and_modify(|count| *count += 1)  // If key exists
    .or_insert(1);                     // If key absent
```

**When to Use Each**:
- Simple values: `or_insert(0)`
- Expensive initialization: `or_insert_with(|| ...)`
- Default trait available: `or_default()`
- Update existing: `and_modify(...).or_insert(...)`

### 5. Load Factor and Resizing

HashMaps grow dynamically to maintain performance as entries increase:

**Load Factor** = `entries / buckets`

```rust
// Start with 16 buckets
let mut map = HashMap::new();

// After 12 insertions (16 * 0.75 = 12)
// Load factor reaches 0.75 → triggers resize

// HashMap doubles capacity: 16 → 32 buckets
// Rehashes ALL entries to new bucket positions
```

**Resize Cost**:
- Allocate new bucket array
- Rehash every entry (compute hash % new_capacity)
- Insert into new locations
- Deallocate old array

**Amortized Analysis**:
- Inserting N elements causes ~log(N) resizes
- Total rehash operations: N + N/2 + N/4 + ... ≈ 2N
- Amortized cost: O(1) per insertion

**Why Pre-allocation Matters**: If you know you'll insert 100K entries, pre-allocating eliminates ~17 resize operations.

### 6. Capacity Pre-allocation

Pre-allocating capacity eliminates resize overhead:

**Without Pre-allocation**:
```rust
let mut map = HashMap::new();  // Capacity: 0

// Insert 100K entries
for i in 0..100_000 {
    map.insert(i, i);
}
// Triggers ~17 resizes, rehashing ~200K entries total
```

**With Pre-allocation**:
```rust
let mut map = HashMap::with_capacity(100_000);  // Capacity: 133,333

// Insert 100K entries
for i in 0..100_000 {
    map.insert(i, i);
}
// 0 resizes!
```

**Capacity Calculation**:
```rust
// To hold N entries without resizing:
let capacity = (N as f64 / 0.75).ceil() as usize;
// For 100K: (100K / 0.75) = 133,333 buckets
```

**Performance Impact** (100K insertions):
- No pre-allocation: ~50ms (includes resize overhead)
- Pre-allocated: ~15ms (pure insertion)
- **3× speedup**

### 7. Composite Keys and Tuples

Multi-dimensional aggregation requires composite keys:

**Tuple Keys**:
```rust
// Group by (user, product) combination
let mut sales: HashMap<(String, String), u64> = HashMap::new();

sales.insert(("alice".into(), "laptop".into()), 2);
sales.insert(("bob".into(), "laptop".into()), 1);

// Query by composite key
let alice_laptop_sales = sales.get(&("alice", "laptop"));
```

**Why Tuples Work**: Rust automatically derives `Hash` and `Eq` for tuples if all elements implement those traits.

**Memory Layout**:
```rust
// (String, String) = 48 bytes
// String = 24 bytes (ptr, capacity, length)
// Tuple = 24 + 24 = 48 bytes
```

**Custom Composite Keys**:
```rust
#[derive(Hash, Eq, PartialEq)]
struct SalesKey {
    user_id: String,
    product_id: String,
    region: String,
}
```

### 8. Multiple HashMap Views

Real analytics require querying data by different dimensions:

**Single Map Approach** (slow queries):
```rust
let events: Vec<Event> = ...;

// Query: "Total sales by user" requires O(n) scan
let user_total = events.iter()
    .filter(|e| e.user == "alice")
    .map(|e| e.value)
    .sum();
```

**Multi-Map Approach** (fast queries):
```rust
struct Analytics {
    by_user: HashMap<String, Stats>,
    by_product: HashMap<String, Stats>,
    by_category: HashMap<String, Stats>,
}

// Each event updates all relevant maps
impl Analytics {
    fn record(&mut self, event: Event) {
        update_map(&mut self.by_user, &event.user, event.value);
        update_map(&mut self.by_product, &event.product, event.value);
        update_map(&mut self.by_category, &event.category, event.value);
    }
}

// Query: O(1) hash lookup
let user_stats = analytics.by_user.get("alice");
```

**Trade-off**: Memory (5 maps vs 1) for speed (O(1) vs O(n) queries).

### 9. Top-K Queries with Heaps

Finding "top 10 users" from 100K users is common in analytics:

**Full Sort Approach** (slow):
```rust
let mut entries: Vec<_> = map.iter().collect();
entries.sort_by_key(|(_, count)| Reverse(*count));
let top10 = entries.into_iter().take(10).collect();
// O(n log n) = 100K × log(100K) ≈ 1.6M operations
```

**Min-Heap Approach** (fast):
```rust
let mut heap: BinaryHeap<Reverse<(u64, String)>> = BinaryHeap::new();

for (key, count) in map {
    if heap.len() < 10 {
        heap.push(Reverse((count, key)));
    } else if count > heap.peek().unwrap().0 {
        heap.pop();
        heap.push(Reverse((count, key)));
    }
}
// O(n log k) = 100K × log(10) ≈ 332K operations (5× faster)
```

**Why Min-Heap**: Keeps smallest of the top-K at the top. When new element is larger, evict smallest and insert new.

### 10. Concurrent HashMap with DashMap

Standard HashMap is not thread-safe. Multi-threaded aggregation requires synchronization:

**Mutex Approach** (doesn't scale):
```rust
let map = Arc::new(Mutex::new(HashMap::new()));

// Thread 1
map.lock().unwrap().insert(key1, value1);

// Thread 2
map.lock().unwrap().insert(key2, value2);  // Blocks on lock
```

**Problem**: Single lock = only one thread active at a time, regardless of CPU cores.

**DashMap Approach** (scales):
```rust
let map = Arc::new(DashMap::new());

// Thread 1
map.insert(key1, value1);  // No explicit locking

// Thread 2
map.insert(key2, value2);  // Can run concurrently!
```

**How It Works**: DashMap internally shards the HashMap into N segments, each with its own lock. Different keys likely hash to different segments, allowing concurrent access.

**Sharding Example**:
```
DashMap with 16 shards:
Shard 0: {user1, user17, user33, ...}  // Lock 0
Shard 1: {user2, user18, user34, ...}  // Lock 1
...
Shard 15: {user16, user32, user48, ...} // Lock 15

Thread A accessing user1 (Shard 0) doesn't block
Thread B accessing user2 (Shard 1) — concurrent!
```

**Performance Scaling**:
- Mutex<HashMap>: 1M ops/sec (single-threaded)
- DashMap (8 cores): 7M ops/sec (near-linear scaling)

### Connection to This Project

This analytics engine project demonstrates HashMap patterns essential for high-throughput data processing:

**Entry API (Step 1)**: The `*map.entry(key).or_insert(0) += 1` pattern eliminates double lookups. For 1M events, this reduces hash operations from 3M to 1M—a 3× performance improvement critical for real-time analytics.

**and_modify Pattern (Step 2)**: Updating multiple statistics (count, sum, min, max) atomically uses `and_modify(|s| s.update(value)).or_insert_with(|| Stats::new(value))`. This single-entry approach prevents race conditions and eliminates redundant lookups.

**Composite Keys (Step 3)**: Tuple keys like `(user_id, product_id)` automatically derive Hash and Eq, enabling multi-dimensional aggregation. Each event updates 5 HashMaps (by_user, by_product, by_category, by_user_product, by_time) for instant O(1) queries across any dimension.

**Capacity Pre-allocation (Step 4)**: Pre-allocating with `HashMap::with_capacity(estimated_size * 4 / 3)` eliminates ~17 resize operations for 100K users. Each resize rehashes all existing entries—eliminating this saves seconds for large-scale ingestion.

**Top-K with Heap (Step 5)**: Finding "top 10 users by revenue" uses a min-heap maintaining only 10 entries, achieving O(n log k) instead of O(n log n). For 100K users, this is 5× faster than full sorting.

**Concurrent DashMap (Step 6)**: Replacing HashMap with DashMap enables lock-free concurrent updates. Internal sharding (16+ segments) allows 8 threads to achieve ~7× throughput improvement, utilizing all CPU cores for high-volume event streams.

By the end of this project, you'll have built a **production-ready analytics engine** achieving the same performance characteristics as Prometheus, InfluxDB, and other real-time metrics systems—handling millions of events per second through efficient HashMap usage.

---

## Build The Project

## Step 1: Basic Event Counter with Entry API

### Introduction

Build a simple event counter that tracks event counts per key using the Entry API. This establishes the foundation for all subsequent aggregations by demonstrating the core pattern: "increment if exists, initialize if absent."

### Architecture

**Structs:**
- `EventCounter<K>` - Generic counter tracking counts by key
  - **Field** `counts: HashMap<K, u64>` - Stores count for each key

**Key Functions:**
- `new()` - Creates empty counter
- `increment(key: K)` - Increments count for key (inserts 0 then increments if absent)
- `get(key: &K) -> u64` - Returns count for key (0 if absent)
- `top_k(k: usize) -> Vec<(K, u64)>` - Returns top K keys by count

**Role Each Plays:**
- Entry API eliminates double lookup (contains + insert/update)
- `or_insert(0)` provides default value in single operation
- `*entry += 1` updates in-place without additional lookup

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment_new_key() {
        let mut counter = EventCounter::new();
        counter.increment("page_view");
        assert_eq!(counter.get(&"page_view"), 1);
    }

    #[test]
    fn test_increment_existing_key() {
        let mut counter = EventCounter::new();
        counter.increment("click");
        counter.increment("click");
        counter.increment("click");
        assert_eq!(counter.get(&"click"), 3);
    }

    #[test]
    fn test_multiple_keys() {
        let mut counter = EventCounter::new();
        counter.increment("event_a");
        counter.increment("event_b");
        counter.increment("event_a");
        assert_eq!(counter.get(&"event_a"), 2);
        assert_eq!(counter.get(&"event_b"), 1);
    }

    #[test]
    fn test_get_nonexistent() {
        let counter: EventCounter<&str> = EventCounter::new();
        assert_eq!(counter.get(&"missing"), 0);
    }

    #[test]
    fn test_top_k() {
        let mut counter = EventCounter::new();
        counter.increment("a");
        counter.increment("b");
        counter.increment("b");
        counter.increment("c");
        counter.increment("c");
        counter.increment("c");

        let top = counter.top_k(2);
        assert_eq!(top[0], ("c", 3));
        assert_eq!(top[1], ("b", 2));
    }
}
```

### Starter Code

```rust
use std::collections::HashMap;
use std::hash::Hash;

pub struct EventCounter<K> {
    counts: HashMap<K, u64>,
}

impl<K> EventCounter<K>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        // TODO: Create new EventCounter with empty HashMap
        unimplemented!()
    }

    pub fn increment(&mut self, key: K) {
        // TODO: Use entry API to increment count
        // Hint: entry(key).or_insert(0)
        // Then increment the value
        unimplemented!()
    }

    pub fn get(&self, key: &K) -> u64 {
        // TODO: Return count for key, or 0 if not present
        // Hint: Use .get().copied().unwrap_or(0)
        unimplemented!()
    }

    pub fn top_k(&self, k: usize) -> Vec<(K, u64)>
    where
        K: Clone + Ord,
    {
        // TODO: Return top k entries by count
        // Hint: collect into Vec, sort by count descending, take k
        unimplemented!()
    }

    pub fn len(&self) -> usize {
        self.counts.len()
    }
}
```

**Why previous step is not enough:** N/A - This is the foundation.

**What's the improvement:** Entry API (`entry().or_insert()`) performs single hash lookup instead of 2 lookups with `contains_key()` + `insert()`. For 1M increments:
- Naive (contains + insert): ~2M hash operations
- Entry API: ~1M hash operations (2x faster)

---

## Step 2: Multi-Metric Aggregator with and_modify

### Introduction

Extend beyond simple counting to track multiple statistics (count, sum, min, max) per key. This requires updating multiple fields atomically, which `and_modify()` enables efficiently.

### Architecture

**Structs:**
- `Stats` - Aggregated statistics
  - **Field** `count: u64` - Number of events
  - **Field** `sum: f64` - Sum of values
  - **Field** `min: f64` - Minimum value seen
  - **Field** `max: f64` - Maximum value seen

- `MetricAggregator<K>` - Aggregates metrics by key
  - **Field** `metrics: HashMap<K, Stats>` - Stats per key

**Key Functions:**
- `new()` - Creates empty aggregator
- `record(key: K, value: f64)` - Records event value for key
- `get_stats(key: &K) -> Option<&Stats>` - Returns stats for key
- `average(key: &K) -> Option<f64>` - Computes average (sum/count)

**Stats Methods:**
- `new(value: f64)` - Initialize stats with first value
- `update(&mut self, value: f64)` - Update stats with new value

**Role Each Plays:**
- `Stats` encapsulates all metrics for a single key
- `and_modify()` updates existing stats without re-insertion
- Entry API ensures single lookup for check-then-insert or update

### Checkpoint Tests

```rust
#[test]
fn test_record_single_value() {
    let mut agg = MetricAggregator::new();
    agg.record("product_a", 100.0);

    let stats = agg.get_stats(&"product_a").unwrap();
    assert_eq!(stats.count, 1);
    assert_eq!(stats.sum, 100.0);
    assert_eq!(stats.min, 100.0);
    assert_eq!(stats.max, 100.0);
}

#[test]
fn test_record_multiple_values() {
    let mut agg = MetricAggregator::new();
    agg.record("user1", 10.0);
    agg.record("user1", 20.0);
    agg.record("user1", 15.0);

    let stats = agg.get_stats(&"user1").unwrap();
    assert_eq!(stats.count, 3);
    assert_eq!(stats.sum, 45.0);
    assert_eq!(stats.min, 10.0);
    assert_eq!(stats.max, 20.0);
}

#[test]
fn test_average_calculation() {
    let mut agg = MetricAggregator::new();
    agg.record("test", 10.0);
    agg.record("test", 20.0);
    agg.record("test", 30.0);

    assert_eq!(agg.average(&"test"), Some(20.0));
}

#[test]
fn test_multiple_keys() {
    let mut agg = MetricAggregator::new();
    agg.record("key1", 100.0);
    agg.record("key2", 200.0);
    agg.record("key1", 150.0);

    assert_eq!(agg.get_stats(&"key1").unwrap().count, 2);
    assert_eq!(agg.get_stats(&"key2").unwrap().count, 1);
}
```

### Starter Code

```rust
#[derive(Debug, Clone)]
pub struct Stats {
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
}

impl Stats {
    pub fn new(value: f64) -> Self {
        // TODO: Initialize all fields with first value
        unimplemented!()
    }

    pub fn update(&mut self, value: f64) {
        // TODO: Update count, sum, min, max with new value
        unimplemented!()
    }

    pub fn average(&self) -> f64 {
        // TODO: Return sum / count
        unimplemented!()
    }
}

pub struct MetricAggregator<K> {
    metrics: HashMap<K, Stats>,
}

impl<K> MetricAggregator<K>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        unimplemented!()
    }

    pub fn record(&mut self, key: K, value: f64) {
        // TODO: Use entry API with or_insert_with() and and_modify()
        // If absent: create new Stats with value
        // If present: update existing Stats with value
        // Hint: entry(key).and_modify(|s| s.update(value)).or_insert_with(|| Stats::new(value))
        unimplemented!()
    }

    pub fn get_stats(&self, key: &K) -> Option<&Stats> {
        // TODO: Return stats for key
        unimplemented!()
    }

    pub fn average(&self, key: &K) -> Option<f64> {
        // TODO: Return average for key (sum/count)
        unimplemented!()
    }
}
```

**Why previous step is not enough:** Step 1 only counts events. Real analytics need aggregates: sums (revenue), averages (transaction size), min/max (price ranges).

**What's the improvement:** `and_modify()` enables atomic update of all statistics in single entry lookup. Without it, you'd need:
1. Check if key exists
2. If yes: get mutable reference, update
3. If no: insert new

With Entry API: Single lookup, branch on Occupied vs Vacant, update in place. For 1M events, this eliminates 1M extra lookups.

---

## Step 3: Multi-Dimensional Grouping with Composite Keys

### Introduction

Real analytics require grouping by multiple dimensions simultaneously: "revenue per product per category" or "clicks per user per hour." This requires composite keys that hash correctly and efficiently.

### Architecture

**Structs:**
- `DimensionKey` - Composite key for multi-dimensional grouping
  - **Field** `user_id: String`
  - **Field** `product: String`
  - **Field** `time_bucket: u64` - Hour/day bucket for time-series

- `MultiDimAggregator` - Aggregates across multiple dimension combinations
  - **Field** `by_user: HashMap<String, Stats>` - Stats per user
  - **Field** `by_product: HashMap<String, Stats>` - Stats per product
  - **Field** `by_category: HashMap<String, Stats>` - Stats per category
  - **Field** `by_user_product: HashMap<(String, String), Stats>` - Stats per user+product
  - **Field** `by_time: HashMap<u64, Stats>` - Stats per time bucket

**Key Functions:**
- `new()` - Creates aggregator
- `record_event(user: String, product: String, category: String, value: f64, timestamp: u64)` - Records event across all dimensions
- `query_by_user(user: &str) -> Option<&Stats>` - Get stats for user
- `query_by_product(product: &str) -> Option<&Stats>` - Get stats for product
- `query_by_user_product(user: &str, product: &str) -> Option<&Stats>` - Get stats for combination

**Role Each Plays:**
- Tuple keys `(String, String)` automatically derive Hash and Eq
- Each HashMap represents a different "view" of the data
- Single event updates all relevant dimensions

### Checkpoint Tests

```rust
#[test]
fn test_single_dimension_aggregation() {
    let mut agg = MultiDimAggregator::new();
    agg.record_event("user1".into(), "laptop".into(), "electronics".into(), 1000.0, 0);

    assert_eq!(agg.query_by_user("user1").unwrap().sum, 1000.0);
    assert_eq!(agg.query_by_product("laptop").unwrap().sum, 1000.0);
}

#[test]
fn test_multi_dimensional_grouping() {
    let mut agg = MultiDimAggregator::new();

    // User1 buys 2 laptops
    agg.record_event("user1".into(), "laptop".into(), "electronics".into(), 1000.0, 0);
    agg.record_event("user1".into(), "laptop".into(), "electronics".into(), 1200.0, 0);

    // User2 buys 1 laptop
    agg.record_event("user2".into(), "laptop".into(), "electronics".into(), 900.0, 0);

    // Check user dimension
    assert_eq!(agg.query_by_user("user1").unwrap().count, 2);
    assert_eq!(agg.query_by_user("user2").unwrap().count, 1);

    // Check product dimension
    assert_eq!(agg.query_by_product("laptop").unwrap().count, 3);
    assert_eq!(agg.query_by_product("laptop").unwrap().sum, 3100.0);

    // Check composite dimension
    assert_eq!(agg.query_by_user_product("user1", "laptop").unwrap().count, 2);
}

#[test]
fn test_category_aggregation() {
    let mut agg = MultiDimAggregator::new();
    agg.record_event("user1".into(), "laptop".into(), "electronics".into(), 1000.0, 0);
    agg.record_event("user2".into(), "phone".into(), "electronics".into(), 500.0, 0);

    let stats = agg.query_by_category("electronics").unwrap();
    assert_eq!(stats.count, 2);
    assert_eq!(stats.sum, 1500.0);
}
```

### Starter Code

```rust
pub struct MultiDimAggregator {
    by_user: HashMap<String, Stats>,
    by_product: HashMap<String, Stats>,
    by_category: HashMap<String, Stats>,
    by_user_product: HashMap<(String, String), Stats>,
    by_time: HashMap<u64, Stats>,
}

impl MultiDimAggregator {
    pub fn new() -> Self {
        // TODO: Initialize all HashMaps
        unimplemented!()
    }

    pub fn record_event(
        &mut self,
        user: String,
        product: String,
        category: String,
        value: f64,
        timestamp: u64,
    ) {
        // TODO: Update all dimension maps with the event
        // Use entry API for each dimension
        // Calculate time_bucket from timestamp (e.g., timestamp / 3600 for hourly)

        // Update by_user
        // self.by_user.entry(user.clone()).and_modify(...).or_insert_with(...)

        // Update by_product
        // ...

        // Update by_category
        // ...

        // Update by_user_product with tuple key
        // ...

        // Update by_time
        // ...

        unimplemented!()
    }

    pub fn query_by_user(&self, user: &str) -> Option<&Stats> {
        // TODO: Return stats for user
        unimplemented!()
    }

    pub fn query_by_product(&self, product: &str) -> Option<&Stats> {
        unimplemented!()
    }

    pub fn query_by_category(&self, category: &str) -> Option<&Stats> {
        unimplemented!()
    }

    pub fn query_by_user_product(&self, user: &str, product: &str) -> Option<&Stats> {
        // TODO: Query with tuple key
        unimplemented!()
    }

    pub fn query_by_time(&self, time_bucket: u64) -> Option<&Stats> {
        unimplemented!()
    }
}
```

**Why previous step is not enough:** Single-dimension aggregation answers "total revenue" but not "revenue by product" or "top users per category." Business questions require slicing data by multiple dimensions.

**What's the improvement:** Multiple HashMaps enable O(1) queries across any dimension. Alternative (storing all events and filtering) would be O(n) per query. For 1M events:
- Filtering approach: 1M comparisons per query
- Multi-map approach: 1 hash lookup per query (1M× faster)

Trade-off: Memory overhead (5 maps instead of 1), but enables instant queries.

---

## Step 4: Efficient Capacity Pre-allocation

### Introduction

As event volume scales to millions, HashMap resizing becomes a bottleneck. Pre-allocating capacity eliminates resize overhead, providing 3-10x faster ingestion.

### Architecture

**Enhance `MultiDimAggregator`:**
- Add `with_capacity(estimated_users: usize, estimated_products: usize, ...)` constructor
- Track resize events for monitoring
- Add `reserve()` method for incremental capacity increases

**New Structs:**
- `ResizeStats` - Tracks HashMap resize operations
  - **Field** `resize_count: usize` - Number of resizes that occurred
  - **Field** `total_rehash_time_us: u64` - Total time spent rehashing

**Key Functions:**
- `MultiDimAggregator::with_capacity(...)` - Pre-allocates all maps
- `track_resize(&mut self, map_name: &str, new_capacity: usize)` - Logs resize events
- `get_resize_stats() -> ResizeStats` - Returns resize statistics

**Role Each Plays:**
- `with_capacity()` sets initial buckets = estimated_size / 0.75 (accounting for load factor)
- Tracking resizes helps identify capacity estimation accuracy
- Monitoring resize timing reveals performance impact

### Checkpoint Tests

```rust
#[test]
fn test_pre_allocated_capacity() {
    let agg = MultiDimAggregator::with_capacity(1000, 500, 100);

    // Verify maps were pre-allocated (capacity should be > 0)
    // Note: exact capacity depends on HashMap implementation
    assert!(agg.by_user.capacity() >= 1000);
    assert!(agg.by_product.capacity() >= 500);
}

#[test]
fn test_no_resize_within_capacity() {
    let mut agg = MultiDimAggregator::with_capacity(100, 100, 10);

    // Insert within capacity
    for i in 0..100 {
        agg.record_event(
            format!("user{}", i),
            format!("product{}", i % 50),
            "category".into(),
            100.0,
            0
        );
    }

    // Should have 0 or minimal resizes
    // In practice, monitor actual resize count
}

#[test]
fn test_reserve_additional_capacity() {
    let mut agg = MultiDimAggregator::with_capacity(10, 10, 10);
    agg.reserve_additional(1000);

    assert!(agg.by_user.capacity() >= 1000);
}
```

### Starter Code

```rust
impl MultiDimAggregator {
    pub fn with_capacity(
        estimated_users: usize,
        estimated_products: usize,
        estimated_categories: usize,
    ) -> Self {
        // TODO: Calculate appropriate capacity accounting for load factor
        // capacity = (estimated_size / 0.75).ceil() as usize
        // Or use estimated_size * 4 / 3

        let user_capacity = ((estimated_users as f64 / 0.75).ceil() as usize);
        let product_capacity = ((estimated_products as f64 / 0.75).ceil() as usize);
        let category_capacity = ((estimated_categories as f64 / 0.75).ceil() as usize);

        // TODO: Create HashMaps with calculated capacities
        // HashMap::with_capacity(user_capacity)

        unimplemented!()
    }

    pub fn reserve_additional(&mut self, additional: usize) {
        // TODO: Reserve additional capacity in all maps
        // self.by_user.reserve(additional);
        unimplemented!()
    }

    pub fn capacity_stats(&self) -> CapacityStats {
        // TODO: Return current capacity of each map
        CapacityStats {
            user_capacity: self.by_user.capacity(),
            product_capacity: self.by_product.capacity(),
            category_capacity: self.by_category.capacity(),
            user_count: self.by_user.len(),
            product_count: self.by_product.len(),
            category_count: self.by_category.len(),
        }
    }
}

#[derive(Debug)]
pub struct CapacityStats {
    pub user_capacity: usize,
    pub product_capacity: usize,
    pub category_capacity: usize,
    pub user_count: usize,
    pub product_count: usize,
    pub category_count: usize,
}
```

**Why previous step is not enough:** Without pre-allocation, inserting 100K users causes ~17 HashMap resizes, each rehashing all existing entries. This can add seconds of overhead.

**What's the improvement:** Pre-allocation eliminates resize overhead:
- Default (no capacity): ~17 resizes for 100K entries, rehashing ~200K total entries
- With capacity: 0 resizes, 0 rehashing

For 1M events across 10K users:
- Default: ~14 resizes, 20K entries rehashed ≈ 200ms overhead
- Pre-allocated: 0 resizes ≈ 0ms overhead

Load factor of 0.75 means HashMap allocates `capacity / 0.75 = capacity * 1.33` buckets internally.

---

## Step 5: Top-K Queries with Heap

### Introduction

Analytics often needs "top 10 users by revenue" or "top 5 products by count." Sorting entire HashMap is O(n log n). Using a min-heap of size K achieves O(n log k), and for K << n, this is much faster.

### Architecture

**New Functions in `MultiDimAggregator`:**
- `top_k_users(k: usize) -> Vec<(String, Stats)>` - Top K users by revenue
- `top_k_products(k: usize) -> Vec<(String, Stats)>` - Top K products by count
- `top_k_by<F>(map: &HashMap<K, Stats>, k: usize, metric: F) -> Vec<(K, Stats)>` - Generic top-K using provided metric extractor

**Helper:**
- Use `BinaryHeap` with `Reverse` for min-heap (keep smallest K, evict when > K)

**Role Each Plays:**
- Min-heap maintains K largest elements efficiently
- Generic `top_k_by()` allows sorting by any metric (count, sum, average, etc.)
- Returns sorted results (largest first)

### Checkpoint Tests

```rust
#[test]
fn test_top_k_users() {
    let mut agg = MultiDimAggregator::new();

    agg.record_event("user1".into(), "p1".into(), "c1".into(), 100.0, 0);
    agg.record_event("user2".into(), "p1".into(), "c1".into(), 500.0, 0);
    agg.record_event("user3".into(), "p1".into(), "c1".into(), 300.0, 0);
    agg.record_event("user2".into(), "p1".into(), "c1".into(), 200.0, 0); // user2 total: 700

    let top2 = agg.top_k_users_by_revenue(2);

    assert_eq!(top2.len(), 2);
    assert_eq!(top2[0].0, "user2"); // Highest revenue
    assert_eq!(top2[0].1.sum, 700.0);
    assert_eq!(top2[1].0, "user3");
}

#[test]
fn test_top_k_products_by_count() {
    let mut agg = MultiDimAggregator::new();

    // Product A: 5 purchases
    for i in 0..5 {
        agg.record_event(format!("user{}", i), "product_a".into(), "cat".into(), 10.0, 0);
    }

    // Product B: 3 purchases
    for i in 0..3 {
        agg.record_event(format!("user{}", i), "product_b".into(), "cat".into(), 10.0, 0);
    }

    // Product C: 1 purchase
    agg.record_event("user0".into(), "product_c".into(), "cat".into(), 10.0, 0);

    let top2 = agg.top_k_products_by_count(2);
    assert_eq!(top2[0].0, "product_a");
    assert_eq!(top2[0].1.count, 5);
}

#[test]
fn test_top_k_less_than_total() {
    let mut agg = MultiDimAggregator::new();

    agg.record_event("user1".into(), "p".into(), "c".into(), 100.0, 0);
    agg.record_event("user2".into(), "p".into(), "c".into(), 200.0, 0);

    // Ask for top 5 when only 2 exist
    let top5 = agg.top_k_users_by_revenue(5);
    assert_eq!(top5.len(), 2);
}
```

### Starter Code

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

impl MultiDimAggregator {
    pub fn top_k_users_by_revenue(&self, k: usize) -> Vec<(String, Stats)> {
        // TODO: Use top_k_by helper with revenue metric
        // self.top_k_by(&self.by_user, k, |stats| stats.sum as i64)
        unimplemented!()
    }

    pub fn top_k_products_by_count(&self, k: usize) -> Vec<(String, Stats)> {
        // TODO: Use top_k_by helper with count metric
        unimplemented!()
    }

    fn top_k_by<K, F>(
        map: &HashMap<K, Stats>,
        k: usize,
        metric: F,
    ) -> Vec<(K, Stats)>
    where
        K: Clone + Ord,
        F: Fn(&Stats) -> i64,
    {
        // TODO: Implement top-K using min-heap
        // 1. Create BinaryHeap with Reverse wrapper (min-heap)
        // 2. Iterate through map entries
        // 3. If heap.len() < k, push (Reverse(metric), key, stats)
        // 4. Else if metric > heap.peek().0, pop and push new entry
        // 5. Extract from heap, reverse order, return

        // Hint: BinaryHeap<Reverse<(i64, K, Stats)>>
        // Reverse makes it a min-heap

        unimplemented!()
    }
}
```

**Why previous step is not enough:** Capacity optimization speeds up ingestion, but queries need optimization too. Finding top 10 from 100K entries by sorting all is wasteful.

**What's the improvement:** Min-heap approach:
- Full sort: O(n log n) ≈ 100K × log(100K) ≈ 1.6M operations
- Heap approach: O(n log k) ≈ 100K × log(10) ≈ 332K operations (5× faster)

For top 10 from 1M entries:
- Full sort: ~20M operations
- Heap: ~3.3M operations (6× faster)

Memory: O(k) instead of O(n) for sorting.

---

## Step 6: Concurrent Analytics with DashMap

### Introduction

Scale to multi-threaded event ingestion. Multiple threads recording events simultaneously requires thread-safe aggregation. `DashMap` provides lock-free concurrent HashMap with automatic sharding.

### Architecture

**New Implementation:**
- Replace `HashMap` with `DashMap` for concurrent access
- `DashMap` API is similar to `HashMap` but thread-safe
- Entry API works across threads

**Structs:**
- `ConcurrentAnalytics` - Thread-safe version using DashMap
  - **Field** `by_user: DashMap<String, Stats>` - Concurrent user stats
  - **Field** `by_product: DashMap<String, Stats>` - Concurrent product stats
  - ... (all dimensions)

**Key Functions:**
- `record_event(...)` - Thread-safe recording (can be called from multiple threads)
- `snapshot() -> MultiDimAggregator` - Create snapshot of current state
- `merge(&mut self, other: Self)` - Merge two aggregators

**Role Each Plays:**
- `DashMap` shards HashMap internally (multiple locks for different buckets)
- Automatic sharding prevents contention
- Entry API remains same, but now thread-safe

### Checkpoint Tests

```rust
use std::sync::Arc;
use std::thread;

#[test]
fn test_concurrent_recording() {
    let analytics = Arc::new(ConcurrentAnalytics::new());
    let mut handles = vec![];

    // Spawn 4 threads, each recording 1000 events
    for thread_id in 0..4 {
        let analytics_clone = Arc::clone(&analytics);
        let handle = thread::spawn(move || {
            for i in 0..1000 {
                analytics_clone.record_event(
                    format!("user{}", thread_id),
                    "product".into(),
                    "category".into(),
                    10.0,
                    0,
                );
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify totals
    // 4 threads × 1000 events = 4000 total
    let snapshot = analytics.snapshot();
    let product_stats = snapshot.query_by_product("product").unwrap();
    assert_eq!(product_stats.count, 4000);
}

#[test]
fn test_concurrent_users() {
    let analytics = Arc::new(ConcurrentAnalytics::new());
    let mut handles = vec![];

    for thread_id in 0..8 {
        let analytics_clone = Arc::clone(&analytics);
        let handle = thread::spawn(move || {
            analytics_clone.record_event(
                format!("user{}", thread_id),
                "p".into(),
                "c".into(),
                100.0,
                0,
            );
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let snapshot = analytics.snapshot();
    assert_eq!(snapshot.by_user.len(), 8);
}
```

### Starter Code

```rust
use dashmap::DashMap;
use std::sync::Arc;

pub struct ConcurrentAnalytics {
    by_user: DashMap<String, Stats>,
    by_product: DashMap<String, Stats>,
    by_category: DashMap<String, Stats>,
    by_user_product: DashMap<(String, String), Stats>,
    by_time: DashMap<u64, Stats>,
}

impl ConcurrentAnalytics {
    pub fn new() -> Self {
        // TODO: Initialize all DashMaps
        unimplemented!()
    }

    pub fn with_capacity(
        estimated_users: usize,
        estimated_products: usize,
        estimated_categories: usize,
    ) -> Self {
        // TODO: DashMap::with_capacity() for pre-allocation
        unimplemented!()
    }

    pub fn record_event(
        &self, // Note: &self, not &mut self (DashMap allows interior mutability)
        user: String,
        product: String,
        category: String,
        value: f64,
        timestamp: u64,
    ) {
        // TODO: Update all DashMaps using entry API
        // DashMap entry API is similar to HashMap
        // self.by_user.entry(user.clone()).and_modify(...).or_insert_with(...)

        unimplemented!()
    }

    pub fn snapshot(&self) -> MultiDimAggregator {
        // TODO: Convert DashMaps to regular HashMaps
        // Create new MultiDimAggregator and copy data
        // Iterate: self.by_user.iter().map(|entry| (entry.key().clone(), entry.value().clone()))

        unimplemented!()
    }

    pub fn query_by_user(&self, user: &str) -> Option<Stats> {
        // TODO: Return cloned stats for user
        // DashMap doesn't return references directly
        self.by_user.get(user).map(|entry| entry.value().clone())
    }
}
```

**Why previous step is not enough:** Single-threaded analytics can't utilize multiple CPU cores. For high-throughput systems receiving events from many sources, sequential processing is a bottleneck.

**What's the improvement:** Concurrent processing provides linear scaling with cores:
- Single-threaded: 1M events/sec
- 8-core concurrent: ~7M events/sec (7× throughput)

`DashMap` achieves this through automatic sharding:
- Mutex<HashMap>: Single lock = 1-core performance regardless of available cores
- DashMap: Internal sharding (16+ segments) = near-linear scaling

Trade-off: Slightly higher overhead per operation (~20% slower than HashMap for single-threaded), but massive gains for concurrent workloads.

---

## Complete Working Example

```rust
use std::collections::HashMap;
use std::hash::Hash;
use std::collections::BinaryHeap;
use std::cmp::{Reverse, Ordering};
use dashmap::DashMap;
use std::sync::Arc;
use std::thread;

// Stats structure for aggregation
#[derive(Debug, Clone)]
pub struct Stats {
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
}

impl Stats {
    pub fn new(value: f64) -> Self {
        Stats {
            count: 1,
            sum: value,
            min: value,
            max: value,
        }
    }

    pub fn update(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    pub fn average(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }
}

// Step 1: Basic Event Counter
pub struct EventCounter<K> {
    counts: HashMap<K, u64>,
}

impl<K> EventCounter<K>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        EventCounter {
            counts: HashMap::new(),
        }
    }

    pub fn increment(&mut self, key: K) {
        *self.counts.entry(key).or_insert(0) += 1;
    }

    pub fn get(&self, key: &K) -> u64 {
        self.counts.get(key).copied().unwrap_or(0)
    }

    pub fn top_k(&self, k: usize) -> Vec<(K, u64)>
    where
        K: Clone + Ord,
    {
        let mut entries: Vec<_> = self.counts.iter()
            .map(|(key, &count)| (key.clone(), count))
            .collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.into_iter().take(k).collect()
    }

    pub fn len(&self) -> usize {
        self.counts.len()
    }
}

// Step 2: Metric Aggregator
pub struct MetricAggregator<K> {
    metrics: HashMap<K, Stats>,
}

impl<K> MetricAggregator<K>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        MetricAggregator {
            metrics: HashMap::new(),
        }
    }

    pub fn record(&mut self, key: K, value: f64) {
        self.metrics
            .entry(key)
            .and_modify(|stats| stats.update(value))
            .or_insert_with(|| Stats::new(value));
    }

    pub fn get_stats(&self, key: &K) -> Option<&Stats> {
        self.metrics.get(key)
    }

    pub fn average(&self, key: &K) -> Option<f64> {
        self.get_stats(key).map(|s| s.average())
    }
}

// Step 3 & 4: Multi-dimensional aggregator with capacity
pub struct MultiDimAggregator {
    pub by_user: HashMap<String, Stats>,
    pub by_product: HashMap<String, Stats>,
    pub by_category: HashMap<String, Stats>,
    pub by_user_product: HashMap<(String, String), Stats>,
    pub by_time: HashMap<u64, Stats>,
}

impl MultiDimAggregator {
    pub fn new() -> Self {
        MultiDimAggregator {
            by_user: HashMap::new(),
            by_product: HashMap::new(),
            by_category: HashMap::new(),
            by_user_product: HashMap::new(),
            by_time: HashMap::new(),
        }
    }

    pub fn with_capacity(
        estimated_users: usize,
        estimated_products: usize,
        estimated_categories: usize,
    ) -> Self {
        let user_capacity = (estimated_users as f64 / 0.75).ceil() as usize;
        let product_capacity = (estimated_products as f64 / 0.75).ceil() as usize;
        let category_capacity = (estimated_categories as f64 / 0.75).ceil() as usize;

        MultiDimAggregator {
            by_user: HashMap::with_capacity(user_capacity),
            by_product: HashMap::with_capacity(product_capacity),
            by_category: HashMap::with_capacity(category_capacity),
            by_user_product: HashMap::with_capacity(user_capacity * 10),
            by_time: HashMap::with_capacity(1000),
        }
    }

    pub fn record_event(
        &mut self,
        user: String,
        product: String,
        category: String,
        value: f64,
        timestamp: u64,
    ) {
        let time_bucket = timestamp / 3600; // Hourly buckets

        // Update all dimensions
        self.by_user
            .entry(user.clone())
            .and_modify(|s| s.update(value))
            .or_insert_with(|| Stats::new(value));

        self.by_product
            .entry(product.clone())
            .and_modify(|s| s.update(value))
            .or_insert_with(|| Stats::new(value));

        self.by_category
            .entry(category)
            .and_modify(|s| s.update(value))
            .or_insert_with(|| Stats::new(value));

        self.by_user_product
            .entry((user, product))
            .and_modify(|s| s.update(value))
            .or_insert_with(|| Stats::new(value));

        self.by_time
            .entry(time_bucket)
            .and_modify(|s| s.update(value))
            .or_insert_with(|| Stats::new(value));
    }

    pub fn query_by_user(&self, user: &str) -> Option<&Stats> {
        self.by_user.get(user)
    }

    pub fn query_by_product(&self, product: &str) -> Option<&Stats> {
        self.by_product.get(product)
    }

    pub fn query_by_category(&self, category: &str) -> Option<&Stats> {
        self.by_category.get(category)
    }

    pub fn query_by_user_product(&self, user: &str, product: &str) -> Option<&Stats> {
        self.by_user_product.get(&(user.to_string(), product.to_string()))
    }

    // Step 5: Top-K queries
    pub fn top_k_users_by_revenue(&self, k: usize) -> Vec<(String, Stats)> {
        Self::top_k_by(&self.by_user, k, |stats| stats.sum as i64)
    }

    pub fn top_k_products_by_count(&self, k: usize) -> Vec<(String, Stats)> {
        Self::top_k_by(&self.by_product, k, |stats| stats.count as i64)
    }

    fn top_k_by<K, F>(
        map: &HashMap<K, Stats>,
        k: usize,
        metric: F,
    ) -> Vec<(K, Stats)>
    where
        K: Clone + Ord,
        F: Fn(&Stats) -> i64,
    {
        if k == 0 {
            return Vec::new();
        }

        // Use min-heap to maintain top K
        let mut heap: BinaryHeap<Reverse<(i64, K, Stats)>> = BinaryHeap::new();

        for (key, stats) in map.iter() {
            let value = metric(stats);

            if heap.len() < k {
                heap.push(Reverse((value, key.clone(), stats.clone())));
            } else if let Some(&Reverse((min_value, _, _))) = heap.peek() {
                if value > min_value {
                    heap.pop();
                    heap.push(Reverse((value, key.clone(), stats.clone())));
                }
            }
        }

        // Extract and reverse order (largest first)
        let mut results: Vec<_> = heap
            .into_iter()
            .map(|Reverse((_, key, stats))| (key, stats))
            .collect();
        results.reverse();
        results
    }
}

// Step 6: Concurrent Analytics
pub struct ConcurrentAnalytics {
    by_user: DashMap<String, Stats>,
    by_product: DashMap<String, Stats>,
    by_category: DashMap<String, Stats>,
    by_user_product: DashMap<(String, String), Stats>,
    by_time: DashMap<u64, Stats>,
}

impl ConcurrentAnalytics {
    pub fn new() -> Self {
        ConcurrentAnalytics {
            by_user: DashMap::new(),
            by_product: DashMap::new(),
            by_category: DashMap::new(),
            by_user_product: DashMap::new(),
            by_time: DashMap::new(),
        }
    }

    pub fn record_event(
        &self,
        user: String,
        product: String,
        category: String,
        value: f64,
        timestamp: u64,
    ) {
        let time_bucket = timestamp / 3600;

        // Update all dimensions concurrently
        self.by_user
            .entry(user.clone())
            .and_modify(|s| s.update(value))
            .or_insert_with(|| Stats::new(value));

        self.by_product
            .entry(product.clone())
            .and_modify(|s| s.update(value))
            .or_insert_with(|| Stats::new(value));

        self.by_category
            .entry(category)
            .and_modify(|s| s.update(value))
            .or_insert_with(|| Stats::new(value));

        self.by_user_product
            .entry((user, product))
            .and_modify(|s| s.update(value))
            .or_insert_with(|| Stats::new(value));

        self.by_time
            .entry(time_bucket)
            .and_modify(|s| s.update(value))
            .or_insert_with(|| Stats::new(value));
    }

    pub fn snapshot(&self) -> MultiDimAggregator {
        let by_user = self.by_user.iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        let by_product = self.by_product.iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        let by_category = self.by_category.iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        let by_user_product = self.by_user_product.iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        let by_time = self.by_time.iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        MultiDimAggregator {
            by_user,
            by_product,
            by_category,
            by_user_product,
            by_time,
        }
    }
}

// Example usage demonstrating all features
fn main() {
    println!("=== Analytics Engine Demo ===\n");

    // Step 1: Simple counter
    println!("Step 1: Event Counter");
    let mut counter = EventCounter::new();
    counter.increment("page_view");
    counter.increment("click");
    counter.increment("page_view");
    counter.increment("purchase");
    counter.increment("click");
    counter.increment("click");

    println!("Event counts:");
    for (event, count) in counter.top_k(10) {
        println!("  {}: {}", event, count);
    }
    println!();

    // Step 2: Metric aggregation
    println!("Step 2: Metric Aggregator");
    let mut metrics = MetricAggregator::new();
    metrics.record("product_a", 99.99);
    metrics.record("product_a", 149.99);
    metrics.record("product_b", 29.99);

    if let Some(stats) = metrics.get_stats(&"product_a") {
        println!("Product A stats:");
        println!("  Count: {}", stats.count);
        println!("  Total: ${:.2}", stats.sum);
        println!("  Average: ${:.2}", stats.average());
        println!("  Min: ${:.2}, Max: ${:.2}", stats.min, stats.max);
    }
    println!();

    // Step 3-5: Multi-dimensional aggregation
    println!("Step 3-5: Multi-dimensional Analytics");
    let mut analytics = MultiDimAggregator::with_capacity(100, 50, 10);

    // Simulate events
    analytics.record_event("alice".into(), "laptop".into(), "electronics".into(), 1299.99, 1000);
    analytics.record_event("bob".into(), "phone".into(), "electronics".into(), 899.99, 1000);
    analytics.record_event("alice".into(), "mouse".into(), "electronics".into(), 29.99, 2000);
    analytics.record_event("charlie".into(), "laptop".into(), "electronics".into(), 1499.99, 2000);
    analytics.record_event("alice".into(), "keyboard".into(), "electronics".into(), 89.99, 3000);

    // Query by user
    if let Some(stats) = analytics.query_by_user("alice") {
        println!("Alice's purchases:");
        println!("  Count: {}", stats.count);
        println!("  Total: ${:.2}", stats.sum);
        println!("  Average: ${:.2}", stats.average());
    }

    // Query by product
    if let Some(stats) = analytics.query_by_product("laptop") {
        println!("\nLaptop sales:");
        println!("  Units sold: {}", stats.count);
        println!("  Revenue: ${:.2}", stats.sum);
    }

    // Top users by revenue
    println!("\nTop 3 users by revenue:");
    for (i, (user, stats)) in analytics.top_k_users_by_revenue(3).iter().enumerate() {
        println!("  {}. {}: ${:.2}", i + 1, user, stats.sum);
    }
    println!();

    // Step 6: Concurrent analytics
    println!("Step 6: Concurrent Analytics");
    let concurrent = Arc::new(ConcurrentAnalytics::new());
    let mut handles = vec![];

    // Spawn 4 threads
    for thread_id in 0..4 {
        let analytics_clone = Arc::clone(&concurrent);
        let handle = thread::spawn(move || {
            for i in 0..100 {
                analytics_clone.record_event(
                    format!("user_{}", thread_id),
                    format!("product_{}", i % 10),
                    "category".into(),
                    (thread_id as f64 + 1.0) * 10.0,
                    (i * 1000) as u64,
                );
            }
        });
        handles.push(handle);
    }

    // Wait for completion
    for handle in handles {
        handle.join().unwrap();
    }

    let snapshot = concurrent.snapshot();
    println!("Concurrent processing complete:");
    println!("  Users: {}", snapshot.by_user.len());
    println!("  Products: {}", snapshot.by_product.len());
    println!("  Total events: {}", snapshot.by_category.get("category").map(|s| s.count).unwrap_or(0));
}
```

###  Complete Working Example

```rust
use dashmap::DashMap;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::hash::Hash;

// =============================================================================
// Milestone 1: Event Counter with Entry API
// =============================================================================

pub struct EventCounter<K> {
    counts: HashMap<K, u64>,
}

impl<K> EventCounter<K>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        EventCounter {
            counts: HashMap::new(),
        }
    }

    pub fn increment(&mut self, key: K) {
        *self.counts.entry(key).or_insert(0) += 1;
    }

    pub fn get(&self, key: &K) -> u64 {
        self.counts.get(key).copied().unwrap_or(0)
    }

    pub fn top_k(&self, k: usize) -> Vec<(K, u64)>
    where
        K: Clone + Ord,
    {
        let mut entries: Vec<_> = self
            .counts
            .iter()
            .map(|(key, &count)| (key.clone(), count))
            .collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        entries.into_iter().take(k).collect()
    }

    pub fn len(&self) -> usize {
        self.counts.len()
    }
}

// =============================================================================
// Milestone 2: Multi-Metric Aggregator with and_modify
// =============================================================================

#[derive(Debug, Clone)]
pub struct Stats {
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
}

impl Stats {
    pub fn new(value: f64) -> Self {
        Stats {
            count: 1,
            sum: value,
            min: value,
            max: value,
        }
    }

    pub fn update(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    pub fn average(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }
}

pub struct MetricAggregator<K> {
    metrics: HashMap<K, Stats>,
}

impl<K> MetricAggregator<K>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        MetricAggregator {
            metrics: HashMap::new(),
        }
    }

    pub fn record(&mut self, key: K, value: f64) {
        self.metrics
            .entry(key)
            .and_modify(|stats| stats.update(value))
            .or_insert_with(|| Stats::new(value));
    }

    pub fn get_stats(&self, key: &K) -> Option<&Stats> {
        self.metrics.get(key)
    }

    pub fn average(&self, key: &K) -> Option<f64> {
        self.get_stats(key).map(|stats| stats.average())
    }
}

// =============================================================================
// Milestone 3 & 4: Multi-Dimensional Aggregation with Capacity Management
// =============================================================================

pub struct MultiDimAggregator {
    pub by_user: HashMap<String, Stats>,
    pub by_product: HashMap<String, Stats>,
    pub by_category: HashMap<String, Stats>,
    pub by_user_product: HashMap<(String, String), Stats>,
    pub by_time: HashMap<u64, Stats>,
}

impl MultiDimAggregator {
    pub fn new() -> Self {
        MultiDimAggregator {
            by_user: HashMap::new(),
            by_product: HashMap::new(),
            by_category: HashMap::new(),
            by_user_product: HashMap::new(),
            by_time: HashMap::new(),
        }
    }

    pub fn with_capacity(
        estimated_users: usize,
        estimated_products: usize,
        estimated_categories: usize,
    ) -> Self {
        let user_capacity = ((estimated_users as f64 / 0.75).ceil() as usize).max(1);
        let product_capacity = ((estimated_products as f64 / 0.75).ceil() as usize).max(1);
        let category_capacity = ((estimated_categories as f64 / 0.75).ceil() as usize).max(1);

        MultiDimAggregator {
            by_user: HashMap::with_capacity(user_capacity),
            by_product: HashMap::with_capacity(product_capacity),
            by_category: HashMap::with_capacity(category_capacity),
            by_user_product: HashMap::with_capacity(user_capacity.saturating_mul(product_capacity).max(1)),
            by_time: HashMap::with_capacity(1024),
        }
    }

    pub fn reserve_additional(&mut self, additional: usize) {
        self.by_user.reserve(additional);
        self.by_product.reserve(additional);
        self.by_category.reserve(additional);
        self.by_user_product.reserve(additional);
        self.by_time.reserve(additional);
    }

    pub fn capacity_stats(&self) -> CapacityStats {
        CapacityStats {
            user_capacity: self.by_user.capacity(),
            product_capacity: self.by_product.capacity(),
            category_capacity: self.by_category.capacity(),
            user_count: self.by_user.len(),
            product_count: self.by_product.len(),
            category_count: self.by_category.len(),
        }
    }

    pub fn record_event(
        &mut self,
        user: String,
        product: String,
        category: String,
        value: f64,
        timestamp: u64,
    ) {
        let time_bucket = timestamp / 3600;
        let user_for_product = user.clone();
        let product_for_user = product.clone();
        let category_clone = category.clone();
        let user_for_tuple = user_for_product.clone();
        let product_for_tuple = product_for_user.clone();

        self.by_user
            .entry(user_for_product)
            .and_modify(|stats| stats.update(value))
            .or_insert_with(|| Stats::new(value));

        self.by_product
            .entry(product_for_user)
            .and_modify(|stats| stats.update(value))
            .or_insert_with(|| Stats::new(value));

        self.by_category
            .entry(category_clone)
            .and_modify(|stats| stats.update(value))
            .or_insert_with(|| Stats::new(value));

        self.by_user_product
            .entry((user_for_tuple, product_for_tuple))
            .and_modify(|stats| stats.update(value))
            .or_insert_with(|| Stats::new(value));

        self.by_time
            .entry(time_bucket)
            .and_modify(|stats| stats.update(value))
            .or_insert_with(|| Stats::new(value));
    }

    pub fn query_by_user(&self, user: &str) -> Option<&Stats> {
        self.by_user.get(user)
    }

    pub fn query_by_product(&self, product: &str) -> Option<&Stats> {
        self.by_product.get(product)
    }

    pub fn query_by_category(&self, category: &str) -> Option<&Stats> {
        self.by_category.get(category)
    }

    pub fn query_by_user_product(&self, user: &str, product: &str) -> Option<&Stats> {
        self.by_user_product
            .get(&(user.to_string(), product.to_string()))
    }

    pub fn query_by_time(&self, time_bucket: u64) -> Option<&Stats> {
        self.by_time.get(&time_bucket)
    }

    // =============================================================================
    // Milestone 5: Top-K Queries with Heap
    // =============================================================================

    pub fn top_k_users_by_revenue(&self, k: usize) -> Vec<(String, Stats)> {
        Self::top_k_by(&self.by_user, k, |stats| stats.sum as i64)
    }

    pub fn top_k_products_by_count(&self, k: usize) -> Vec<(String, Stats)> {
        Self::top_k_by(&self.by_product, k, |stats| stats.count as i64)
    }

    fn top_k_by<KF, F>(map: &HashMap<KF, Stats>, k: usize, metric: F) -> Vec<(KF, Stats)>
    where
        KF: Clone + Ord + Eq + Hash,
        F: Fn(&Stats) -> i64,
    {
        if k == 0 {
            return Vec::new();
        }

        let mut heap: BinaryHeap<Reverse<(i64, usize, KF)>> = BinaryHeap::new();
        let mut idx = 0usize;

        for (key, stats) in map.iter() {
            let score = metric(stats);
            if heap.len() < k {
                heap.push(Reverse((score, idx, key.clone())));
            } else if let Some(Reverse((min_score, _, _))) = heap.peek() {
                if score > *min_score {
                    heap.pop();
                    heap.push(Reverse((score, idx, key.clone())));
                }
            }
            idx += 1;
        }

        let mut ordered = Vec::new();
        while let Some(Reverse((score, _, key))) = heap.pop() {
            ordered.push((score, key));
        }
        ordered.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));

        ordered
            .into_iter()
            .filter_map(|(_, key)| map.get(&key).map(|stats| (key, stats.clone())))
            .collect()
    }
}

#[derive(Debug)]
pub struct CapacityStats {
    pub user_capacity: usize,
    pub product_capacity: usize,
    pub category_capacity: usize,
    pub user_count: usize,
    pub product_count: usize,
    pub category_count: usize,
}

// =============================================================================
// Milestone 6: Concurrent Analytics with DashMap
// =============================================================================

pub struct ConcurrentAnalytics {
    by_user: DashMap<String, Stats>,
    by_product: DashMap<String, Stats>,
    by_category: DashMap<String, Stats>,
    by_user_product: DashMap<(String, String), Stats>,
    by_time: DashMap<u64, Stats>,
}

impl ConcurrentAnalytics {
    pub fn new() -> Self {
        ConcurrentAnalytics {
            by_user: DashMap::new(),
            by_product: DashMap::new(),
            by_category: DashMap::new(),
            by_user_product: DashMap::new(),
            by_time: DashMap::new(),
        }
    }

    pub fn with_capacity(
        estimated_users: usize,
        estimated_products: usize,
        estimated_categories: usize,
    ) -> Self {
        ConcurrentAnalytics {
            by_user: DashMap::with_capacity(estimated_users),
            by_product: DashMap::with_capacity(estimated_products),
            by_category: DashMap::with_capacity(estimated_categories),
            by_user_product: DashMap::with_capacity(estimated_users.saturating_mul(estimated_products).max(1)),
            by_time: DashMap::with_capacity(1024),
        }
    }

    pub fn record_event(
        &self,
        user: String,
        product: String,
        category: String,
        value: f64,
        timestamp: u64,
    ) {
        let time_bucket = timestamp / 3600;
        let user_for_product = user.clone();
        let product_for_user = product.clone();
        let category_clone = category.clone();
        let user_for_tuple = user_for_product.clone();
        let product_for_tuple = product_for_user.clone();

        self.by_user
            .entry(user_for_product)
            .and_modify(|stats| stats.update(value))
            .or_insert(Stats::new(value));

        self.by_product
            .entry(product_for_user)
            .and_modify(|stats| stats.update(value))
            .or_insert(Stats::new(value));

        self.by_category
            .entry(category_clone)
            .and_modify(|stats| stats.update(value))
            .or_insert(Stats::new(value));

        self.by_user_product
            .entry((user_for_tuple, product_for_tuple))
            .and_modify(|stats| stats.update(value))
            .or_insert(Stats::new(value));

        self.by_time
            .entry(time_bucket)
            .and_modify(|stats| stats.update(value))
            .or_insert(Stats::new(value));
    }

    pub fn snapshot(&self) -> MultiDimAggregator {
        MultiDimAggregator {
            by_user: self
                .by_user
                .iter()
                .map(|entry| (entry.key().clone(), entry.value().clone()))
                .collect(),
            by_product: self
                .by_product
                .iter()
                .map(|entry| (entry.key().clone(), entry.value().clone()))
                .collect(),
            by_category: self
                .by_category
                .iter()
                .map(|entry| (entry.key().clone(), entry.value().clone()))
                .collect(),
            by_user_product: self
                .by_user_product
                .iter()
                .map(|entry| (entry.key().clone(), entry.value().clone()))
                .collect(),
            by_time: self
                .by_time
                .iter()
                .map(|entry| (entry.key().clone(), entry.value().clone()))
                .collect(),
        }
    }

    pub fn query_by_user(&self, user: &str) -> Option<Stats> {
        self.by_user.get(user).map(|entry| entry.value().clone())
    }
}

fn main() {}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_event_counter() {
        let mut counter = EventCounter::new();
        counter.increment("view");
        counter.increment("click");
        counter.increment("view");
        assert_eq!(counter.get(&"view"), 2);
        assert_eq!(counter.get(&"click"), 1);
        assert_eq!(counter.get(&"purchase"), 0);
        let top = counter.top_k(1);
        assert_eq!(top[0], ("view", 2));
    }

    #[test]
    fn test_metric_aggregator() {
        let mut agg = MetricAggregator::new();
        agg.record("product", 10.0);
        agg.record("product", 20.0);
        let stats = agg.get_stats(&"product").unwrap();
        assert_eq!(stats.count, 2);
        assert_eq!(stats.sum, 30.0);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 20.0);
        assert_eq!(agg.average(&"product"), Some(15.0));
    }

    #[test]
    fn test_multi_dimensional_updates() {
        let mut agg = MultiDimAggregator::new();
        agg.record_event("user1".into(), "laptop".into(), "electronics".into(), 1000.0, 0);
        agg.record_event("user1".into(), "laptop".into(), "electronics".into(), 1200.0, 0);
        agg.record_event("user2".into(), "phone".into(), "electronics".into(), 500.0, 3600);

        assert_eq!(agg.query_by_user("user1").unwrap().count, 2);
        assert_eq!(agg.query_by_product("laptop").unwrap().sum, 2200.0);
        assert_eq!(agg.query_by_category("electronics").unwrap().count, 3);
        assert_eq!(
            agg.query_by_user_product("user1", "laptop").unwrap().count,
            2
        );
        assert_eq!(agg.query_by_time(1).unwrap().count, 1);
    }

    #[test]
    fn test_capacity_management() {
        let mut agg = MultiDimAggregator::with_capacity(100, 50, 20);
        let stats = agg.capacity_stats();
        assert!(stats.user_capacity >= 100);
        assert!(stats.product_capacity >= 50);
        agg.reserve_additional(500);
        let stats_after = agg.capacity_stats();
        assert!(stats_after.user_capacity >= stats.user_capacity);
    }

    #[test]
    fn test_top_k_queries() {
        let mut agg = MultiDimAggregator::new();
        agg.record_event("user1".into(), "a".into(), "c".into(), 100.0, 0);
        agg.record_event("user2".into(), "a".into(), "c".into(), 300.0, 0);
        agg.record_event("user3".into(), "a".into(), "c".into(), 200.0, 0);
        agg.record_event("user4".into(), "b".into(), "c".into(), 50.0, 0);

        let top_users = agg.top_k_users_by_revenue(2);
        assert_eq!(top_users[0].0, "user2");
        assert_eq!(top_users[0].1.sum, 300.0);

        let top_products = agg.top_k_products_by_count(1);
        assert_eq!(top_products[0].0, "a");
        assert_eq!(top_products[0].1.count, 3);
    }

    #[test]
    fn test_concurrent_recording() {
        let analytics = Arc::new(ConcurrentAnalytics::new());
        let mut handles = vec![];

        for thread_id in 0..4 {
            let analytics_clone = Arc::clone(&analytics);
            let handle = thread::spawn(move || {
                for _ in 0..1000 {
                    analytics_clone.record_event(
                        format!("user{}", thread_id),
                        "product".into(),
                        "category".into(),
                        10.0,
                        0,
                    );
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let snapshot = analytics.snapshot();
        assert_eq!(snapshot.by_product.get("product").unwrap().count, 4000);
    }

    #[test]
    fn test_concurrent_unique_users() {
        let analytics = Arc::new(ConcurrentAnalytics::new());
        let mut handles = vec![];

        for thread_id in 0..8 {
            let analytics_clone = Arc::clone(&analytics);
            let handle = thread::spawn(move || {
                analytics_clone.record_event(
                    format!("user{}", thread_id),
                    "p".into(),
                    "c".into(),
                    100.0,
                    0,
                );
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let snapshot = analytics.snapshot();
        assert_eq!(snapshot.by_user.len(), 8);
    }
}

```