# Project 3: High-Performance Cache with Alternative Maps

## Problem Statement

Build a multi-tiered cache system using different map types (HashMap, BTreeMap, FxHashMap) optimized for different access patterns and data characteristics. The cache should demonstrate when to use each map type and measure performance differences.

Your cache should:
- LRU cache with bounded size (fast lookups, insertion-order tracking)
- Time-based expiration using BTreeMap (range deletions)
- Hot path caching with FxHashMap (maximum speed)
- Benchmark all three map types

## Why It Matters

HashMap isn't always optimal. BTreeMap enables range queries impossible with HashMap. FxHashMap is 10× faster for integer keys. Small maps (<10 entries) benefit from arrays. Choosing the right map affects performance by 10-100×.

This demonstrates: data structure selection based on access patterns, performance measurement, and practical trade-offs.

## Use Cases

- Application caching (Redis-style)
- CDN edge caching
- Database query result caching
- Session management
- API rate limiting
- Configuration caching

---

## Introduction to Map Types and Caching Strategies

Choosing the right map type and caching strategy can improve performance by 10-100×. HashMap, BTreeMap, and FxHashMap have vastly different characteristics, and understanding when to use each is critical. Caching adds another dimension: eviction policies, expiration strategies, and multi-tiered architectures all affect hit rates and memory efficiency.

### 1. HashMap vs BTreeMap vs FxHashMap Trade-offs

Rust provides three primary map types with different performance characteristics:

**HashMap (Default)**:
```rust
use std::collections::HashMap;
let mut map: HashMap<String, i32> = HashMap::new();
```
- **Hash function**: SipHash-1-3 (cryptographic, DoS-resistant)
- **Ordering**: Unordered (iteration order is random)
- **Performance**: O(1) average insert/lookup, ~100ns per operation
- **Memory**: ~24 bytes overhead per entry
- **Use case**: General-purpose, untrusted keys

**BTreeMap (Ordered)**:
```rust
use std::collections::BTreeMap;
let mut map: BTreeMap<String, i32> = BTreeMap::new();
```
- **Data structure**: B-Tree (balanced tree, not binary)
- **Ordering**: Keys sorted (iteration yields sorted order)
- **Performance**: O(log n) insert/lookup, ~150-300ns per operation
- **Memory**: ~40 bytes overhead per entry (tree nodes)
- **Use case**: Range queries, sorted iteration, min/max lookups

**FxHashMap (Fast Hash)**:
```rust
use rustc_hash::FxHashMap;
let mut map: FxHashMap<u64, i32> = FxHashMap::default();
```
- **Hash function**: FxHash (non-cryptographic, fast)
- **Ordering**: Unordered
- **Performance**: O(1) average, ~10ns per operation (10× faster than HashMap)
- **Memory**: Same as HashMap (~24 bytes per entry)
- **Use case**: Trusted integer keys, hot paths

**Comparison Table** (1M operations):
```
Insert:
- HashMap:     150ms
- BTreeMap:    300ms
- FxHashMap:    15ms ✓ Fastest

Lookup:
- HashMap:     120ms
- BTreeMap:    250ms
- FxHashMap:    12ms ✓ Fastest

Range Query (10K entries):
- HashMap:     N/A (not supported)
- BTreeMap:    5ms ✓ Only option
- FxHashMap:   N/A (not supported)

Sorted Iteration:
- HashMap:     150ms (requires sorting)
- BTreeMap:    10ms ✓ Already sorted
- FxHashMap:   150ms (requires sorting)
```

### 2. LRU (Least Recently Used) Cache Algorithm

LRU evicts the least recently accessed item when capacity is reached:

**The Problem**:
```rust
// Unbounded cache - memory grows forever
let mut cache = HashMap::new();
for i in 0..1_000_000 {
    cache.insert(i, expensive_computation(i));
}
// 1M entries × 1KB each = 1GB memory!
```

**LRU Solution**:
```rust
// Bounded cache - keeps only 1000 hottest items
let mut cache = LruCache::new(1000);
for i in 0..1_000_000 {
    cache.put(i, expensive_computation(i));
    // Automatically evicts oldest when > 1000
}
// Maximum: 1000 entries × 1KB = 1MB memory
```

**How LRU Works**:
```
Initial: []
Insert A: [A]
Insert B: [B, A]  // B is most recent
Access A: [A, B]  // A becomes most recent
Insert C (capacity=2): [C, A]  // Evicts B (least recent)
```

**Implementation Approaches**:
- **HashMap + Access Counter**: Store timestamp with each entry, scan for minimum on eviction (O(n) eviction)
- **HashMap + Doubly-Linked List**: O(1) eviction but complex (standard approach)
- **Simplified (this project)**: HashMap with access counter, O(n) scan on eviction (good enough for small caches)

### 3. Time-Based Expiration (TTL Cache)

Many cache entries have inherent expiration times: sessions expire, API responses go stale, auth tokens time out.

**TTL Pattern**:
```rust
cache.put("session123", user_data, ttl=3600); // Expires in 1 hour

// 30 minutes later
cache.get("session123"); // ✓ Still valid

// 2 hours later
cache.get("session123"); // ✗ Expired, returns None
```

**Naive Implementation** (O(n) cleanup):
```rust
// Scan entire cache to find expired entries
for (key, (value, expiry)) in cache.iter() {
    if now > expiry {
        to_remove.push(key);
    }
}
// O(n) - expensive for large caches
```

**BTreeMap Optimization** (O(log n + k) cleanup):
```rust
// BTreeMap sorted by expiry time
let expiry_index: BTreeMap<u64, Vec<Key>> = ...;

// Remove all entries expiring before now
let expired = expiry_index.range(..now);
for (expiry_time, keys) in expired {
    for key in keys {
        cache.remove(key);
    }
}
// O(log n + k) where k = number of expired entries
```

**Why BTreeMap**: Range queries (`range(..time)`) efficiently find all entries in a time range.

### 4. BTreeMap Range Queries

BTreeMap's ordered structure enables efficient range operations:

**Range Operations**:
```rust
let mut timestamps: BTreeMap<u64, String> = BTreeMap::new();
timestamps.insert(100, "event1".into());
timestamps.insert(200, "event2".into());
timestamps.insert(300, "event3".into());

// Get all entries with timestamp < 250
let recent = timestamps.range(..250);
// Returns: [(100, "event1"), (200, "event2")]

// Get entries between 150 and 250
let window = timestamps.range(150..250);
// Returns: [(200, "event2")]
```

**Time Complexity**:
- `range(start..end)`: O(log n + k) where k = number of entries in range
- HashMap equivalent: O(n) - must scan entire map

**Use Cases**:
- Time-series data: "Events in last hour"
- Expiration cleanup: "Entries expiring before now"
- Leaderboards: "Top 10 scores"
- Pagination: "Items from index 100 to 200"

### 5. Cache Hit Rate and Metrics

Cache effectiveness is measured by hit rate:

**Hit Rate Formula**:
```
hit_rate = hits / (hits + misses)

Example:
- 1000 requests
- 800 cache hits (served from cache)
- 200 cache misses (loaded from backing store)
- hit_rate = 800 / 1000 = 80%
```

**Impact of Hit Rate**:
```
Backing store latency: 100ms
Cache latency: 1ms

80% hit rate:
- Average latency = 0.8 × 1ms + 0.2 × 100ms = 20.8ms

50% hit rate:
- Average latency = 0.5 × 1ms + 0.5 × 100ms = 50.5ms

95% hit rate:
- Average latency = 0.95 × 1ms + 0.05 × 100ms = 5.95ms
```

**Improving Hit Rate**:
- **Increase cache size**: More items fit → fewer evictions
- **Better eviction policy**: LRU keeps hot items, LFU (Least Frequently Used) even better
- **Pre-warming**: Load predictable data before requests
- **Smarter TTLs**: Longer TTLs for stable data, shorter for volatile

### 6. Multi-Tiered Cache Architecture

Real systems use multiple cache tiers with different characteristics:

**Three-Tier Example**:
```
Request
  ↓
L1: Hot Cache (10 items, FxHashMap, ~10ns latency)
  ↓ miss
L2: LRU Cache (1000 items, HashMap, ~100ns latency)
  ↓ miss
L3: TTL Cache (100K items, BTreeMap+HashMap, ~1μs latency)
  ↓ miss
Database (10ms latency)
```

**Performance Breakdown**:
```
Assume:
- 50% of requests hit L1 (10ns)
- 30% hit L2 (100ns)
- 15% hit L3 (1000ns)
- 5% hit database (10,000,000ns)

Average latency:
= 0.50 × 10ns
+ 0.30 × 100ns
+ 0.15 × 1000ns
+ 0.05 × 10,000,000ns
= 5 + 30 + 150 + 500,000
= 500,185ns ≈ 0.5ms

Without caching: 10ms average
Speedup: 20×
```

**Promotion Strategy**: Frequently accessed items "bubble up" from lower to higher tiers.

### 7. Cache Eviction Policies

Different policies for different workloads:

**LRU (Least Recently Used)**:
- **Strategy**: Evict oldest accessed item
- **Good for**: Temporal locality (recently accessed → likely accessed again)
- **Example**: Web page caching, session data

**LFU (Least Frequently Used)**:
- **Strategy**: Evict least frequently accessed item
- **Good for**: Popularity-based (hot items stay regardless of recency)
- **Example**: Video streaming (popular videos always cached)

**FIFO (First In, First Out)**:
- **Strategy**: Evict oldest inserted item
- **Good for**: Time-based relevance (news feeds, logs)
- **Example**: Activity feeds

**Random**:
- **Strategy**: Evict random item
- **Good for**: Simplicity when no clear pattern
- **Surprisingly effective**: Often within 10% of LRU performance

**Comparison** (cache size = 100, workload = 1000 requests):
```
LRU:    85% hit rate
LFU:    82% hit rate
FIFO:   70% hit rate
Random: 75% hit rate
```

### 8. Read-Through and Write-Through Patterns

Cache integration patterns standardize backing store interaction:

**Cache-Aside** (application manages cache):
```rust
fn get_user(id: u64) -> User {
    if let Some(user) = cache.get(id) {
        return user;  // Cache hit
    }
    let user = database.load(id);  // Cache miss
    cache.put(id, user.clone());
    user
}
```
- **Pros**: Simple, flexible
- **Cons**: Application handles cache logic

**Read-Through** (cache handles misses):
```rust
// Cache automatically loads on miss
let user = cache.get(id);  // Loads from DB if not cached

impl ReadThroughCache {
    fn get(&mut self, key: K) -> V {
        if let Some(v) = self.cache.get(key) {
            return v;
        }
        let v = self.backing_store.load(key);
        self.cache.put(key, v.clone());
        v
    }
}
```
- **Pros**: Transparent, simplified application code
- **Cons**: Cache coupled to backing store

**Write-Through** (writes go to cache + store):
```rust
cache.put(id, user);
// Automatically writes to both cache and database

impl WriteThroughCache {
    fn put(&mut self, key: K, value: V) {
        self.cache.put(key, value.clone());
        self.backing_store.save(key, value);  // Synchronous write
    }
}
```
- **Pros**: Consistency guaranteed
- **Cons**: Write latency = cache + store (slower writes)

**Write-Behind** (async writes):
```rust
cache.put(id, user);  // Returns immediately
// Background thread flushes to database periodically
```
- **Pros**: Fast writes
- **Cons**: Risk of data loss if crash before flush

### 9. Cache Stampede and Thundering Herd

When many requests simultaneously miss the same cache entry, they all query the backing store:

**The Problem**:
```
Cache expires "popular_item"
↓
1000 concurrent requests arrive
↓
All 1000 check cache → miss
↓
All 1000 query database simultaneously
↓
Database overload!
```

**Solutions**:

**Request Coalescing**:
```rust
// Only first request queries DB, others wait
if cache.is_loading(key) {
    wait_for_load(key);
} else {
    mark_loading(key);
    value = database.load(key);
    cache.put(key, value);
    unmark_loading(key);
}
```

**Probabilistic Early Expiration**:
```rust
// Refresh before expiry with some probability
if time_to_expiry < random(0..60) {
    refresh_cache(key);  // Only one request likely to hit this
}
```

### 10. Memory-Efficient Small Maps

For tiny maps (< 10 entries), array-based maps can be faster than hash-based:

**SmallVec Pattern**:
```rust
enum SmallMap<K, V> {
    Array([(K, V); 8]),  // Up to 8 entries
    HashMap(HashMap<K, V>),  // 8+ entries
}
```

**Performance**:
```
Map size | Array lookup | HashMap lookup
1        | 2ns          | 100ns (50× slower)
5        | 10ns         | 100ns (10× slower)
10       | 20ns         | 100ns (5× slower)
100      | 200ns        | 100ns (2× faster HashMap)
```

**Why Arrays Win for Small N**:
- No hashing overhead
- No pointer chasing
- Better cache locality
- Linear scan of 8 entries ≈ 2ns each = 16ns total

**Trade-off**: Arrays become slower than HashMap around 10-15 entries.

### Connection to This Project

This multi-tier cache project demonstrates map selection and caching strategies essential for production systems:

**LRU Cache (Step 1)**: The HashMap-based LRU cache demonstrates bounded memory with intelligent eviction. Using an access counter for recency tracking, it achieves O(1) get/put with O(n) eviction—acceptable for small caches. This pattern prevents unbounded memory growth while keeping hot data.

**TTL Cache (Step 2)**: BTreeMap's range queries enable efficient bulk expiration. Instead of scanning all entries (O(n)), `range(..now)` finds expired entries in O(log n + k) where k = expired count. For cleaning 1000 expired entries from 1M total, this is 1000× faster than linear scan.

**FxHashMap Hot Cache (Step 3)**: Switching from HashMap's SipHash to FxHashMap's FxHash achieves 10× speedup for trusted integer keys. For hot paths handling millions of requests/second, this 90% latency reduction (100ns → 10ns) is the difference between scaling and not scaling.

**Multi-Level Architecture (Step 4)**: Combining all three map types creates a tiered cache matching access patterns. Hot tier (FxHashMap) serves 50% of requests in 10ns, LRU tier (HashMap) serves 30% in 100ns, TTL tier (BTreeMap) serves 15% in 1μs. Average latency is dominated by the 95% cache hit rate, not the 5% database misses.

**Benchmarking (Step 5)**: Comprehensive measurements reveal real-world performance differences. Claims like "FxHashMap is 10× faster" are validated with actual data, guiding optimization decisions based on evidence rather than intuition.

**Cache Patterns (Step 6)**: Read-through and write-through abstractions demonstrate production integration. These patterns separate caching concerns from application logic, making code more maintainable while ensuring consistent behavior across the system.

By the end of this project, you'll have built a **production-grade caching system** matching the architecture of Redis, Memcached, and CDN edge caches—understanding both the algorithms (LRU, TTL) and engineering decisions (map selection, tiering strategies) that enable high-performance, memory-efficient caching.

---

## Build The Project

## Step 1: Basic LRU Cache with HashMap

### Introduction

Implement Least Recently Used (LRU) cache using HashMap for O(1) lookups and a doubly-linked list for O(1) eviction tracking.

### Architecture

**Structs:**
- `LruCache<K, V>` - LRU cache with bounded capacity
  - **Field** `map: HashMap<K, (V, usize)>` - Value + access order
  - **Field** `access_order: Vec<K>` - Keys in access order
  - **Field** `capacity: usize` - Maximum entries
  - **Field** `access_counter: usize` - Monotonic access counter

**Key Functions:**
- `new(capacity: usize)` - Creates cache with max capacity
- `get(&mut self, key: &K) -> Option<&V>` - Get value, mark as recently used
- `put(&mut self, key: K, value: V)` - Insert value, evict if needed
- `len() -> usize` - Current size

**Role Each Plays:**
- HashMap provides O(1) get/put
- Access counter tracks recency
- Eviction removes oldest when capacity reached

### Checkpoint Tests

```rust
#[test]
fn test_basic_insertion() {
    let mut cache = LruCache::new(2);
    cache.put("a", 1);
    cache.put("b", 2);

    assert_eq!(cache.get(&"a"), Some(&1));
    assert_eq!(cache.get(&"b"), Some(&2));
}

#[test]
fn test_eviction() {
    let mut cache = LruCache::new(2);
    cache.put("a", 1);
    cache.put("b", 2);
    cache.put("c", 3); // Should evict "a"

    assert_eq!(cache.get(&"a"), None);
    assert_eq!(cache.get(&"b"), Some(&2));
    assert_eq!(cache.get(&"c"), Some(&3));
}

#[test]
fn test_update_existing() {
    let mut cache = LruCache::new(2);
    cache.put("a", 1);
    cache.put("b", 2);
    cache.put("a", 10); // Update "a"

    assert_eq!(cache.get(&"a"), Some(&10));
    assert_eq!(cache.len(), 2);
}

#[test]
fn test_access_updates_recency() {
    let mut cache = LruCache::new(2);
    cache.put("a", 1);
    cache.put("b", 2);
    cache.get(&"a"); // Make "a" most recent
    cache.put("c", 3); // Should evict "b", not "a"

    assert_eq!(cache.get(&"a"), Some(&1));
    assert_eq!(cache.get(&"b"), None);
    assert_eq!(cache.get(&"c"), Some(&3));
}
```

### Starter Code

```rust
use std::collections::HashMap;
use std::hash::Hash;

pub struct LruCache<K, V> {
    map: HashMap<K, (V, usize)>, // (value, access_time)
    capacity: usize,
    access_counter: usize,
}

impl<K, V> LruCache<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn new(capacity: usize) -> Self {
        // TODO: Create cache with given capacity
        unimplemented!()
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        // TODO: Get value and update access time
        // Increment access_counter
        // Update entry's access time
        unimplemented!()
    }

    pub fn put(&mut self, key: K, value: V) {
        // TODO: Insert or update entry
        // If at capacity and inserting new key, evict LRU entry
        // Hint: Find entry with minimum access_time
        unimplemented!()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    fn evict_lru(&mut self) {
        // TODO: Find and remove entry with oldest access_time
        unimplemented!()
    }
}
```

**Why previous step is not enough:** N/A - Foundation step.

**What's the improvement:** LRU cache provides bounded memory with intelligent eviction:
- Unbounded cache: Memory grows indefinitely
- LRU cache: Bounded memory, keeps hot data

---

## Step 2: Time-Based Expiration with BTreeMap

### Introduction

Implement TTL (Time-To-Live) cache using BTreeMap to enable efficient range-based expiration. BTreeMap's ordered keys allow O(log n) range deletions.

### Architecture

**Structs:**
- `TtlCache<K, V>` - Time-based expiration cache
  - **Field** `data: HashMap<K, V>` - Actual data
  - **Field** `expiry: BTreeMap<u64, Vec<K>>` - Expiry time → keys
  - **Field** `key_expiry: HashMap<K, u64>` - Key → expiry time
  - **Field** `default_ttl: u64` - Default TTL in seconds

**Key Functions:**
- `new(default_ttl: u64)` - Creates cache with TTL
- `put(&mut self, key: K, value: V)` - Insert with TTL
- `get(&mut self, key: &K) -> Option<&V>` - Get if not expired
- `cleanup(&mut self, now: u64)` - Remove expired entries
- `cleanup_range(&mut self, until: u64)` - Remove entries expiring before time

**Role Each Plays:**
- BTreeMap enables efficient range queries (all entries expiring before T)
- HashMap provides O(1) data access
- Dual-index (expiry → keys, keys → expiry) for efficient cleanup

### Checkpoint Tests

```rust
#[test]
fn test_ttl_expiration() {
    let mut cache = TtlCache::new(10); // 10 second TTL

    cache.put("key1", "value1", 100); // Inserted at time 100

    // Before expiry
    assert_eq!(cache.get(&"key1", 105), Some(&"value1"));

    // After expiry
    assert_eq!(cache.get(&"key1", 111), None);
}

#[test]
fn test_cleanup() {
    let mut cache = TtlCache::new(10);

    cache.put("a", 1, 100); // Expires at 110
    cache.put("b", 2, 100); // Expires at 110
    cache.put("c", 3, 105); // Expires at 115

    cache.cleanup(112); // Clean up entries expiring <= 112

    assert_eq!(cache.get(&"a", 112), None);
    assert_eq!(cache.get(&"b", 112), None);
    assert_eq!(cache.get(&"c", 112), Some(&3));
}

#[test]
fn test_range_cleanup() {
    let mut cache = TtlCache::new(10);

    for i in 0..100 {
        cache.put(i, i * 2, i as u64);
    }

    // Cleanup all entries expiring before time 50
    cache.cleanup_range(50);

    assert!(cache.len() >= 50);
}
```

### Starter Code

```rust
use std::collections::{HashMap, BTreeMap};

pub struct TtlCache<K, V> {
    data: HashMap<K, V>,
    expiry: BTreeMap<u64, Vec<K>>,
    key_expiry: HashMap<K, u64>,
    default_ttl: u64,
}

impl<K, V> TtlCache<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn new(default_ttl: u64) -> Self {
        // TODO: Initialize cache
        unimplemented!()
    }

    pub fn put(&mut self, key: K, value: V, now: u64) {
        // TODO: Insert value with expiry time
        // Calculate expiry = now + default_ttl
        // Update all three maps
        unimplemented!()
    }

    pub fn get(&mut self, key: &K, now: u64) -> Option<&V> {
        // TODO: Check if key exists and not expired
        // If expired, remove from all maps
        unimplemented!()
    }

    pub fn cleanup(&mut self, now: u64) {
        // TODO: Remove all expired entries
        // Use BTreeMap range query
        unimplemented!()
    }

    pub fn cleanup_range(&mut self, until: u64) {
        // TODO: Remove entries expiring before 'until'
        // Use BTreeMap::range() for efficient iteration
        unimplemented!()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}
```

**Why previous step is not enough:** LRU evicts by access recency, not time. Many caches need time-based expiration (sessions, API responses with TTL).

**What's the improvement:** BTreeMap enables efficient bulk expiration:
- HashMap only: Must scan all entries O(n) to find expired
- BTreeMap + HashMap: Range query O(log n + k) where k = expired entries

For cleaning 1000 expired entries from 1M total:
- HashMap scan: 1M checks
- BTreeMap range: ~20 tree operations + 1000 removals

---

## Step 3: Hot Path Cache with FxHashMap

### Introduction

Use FxHashMap for performance-critical integer-keyed caches where maximum throughput is essential.

### Architecture

**Structs:**
- `HotCache<V>` - Ultra-fast integer key cache
  - **Field** `cache: FxHashMap<u64, V>` - Fast integer hashing
  - **Field** `hits: u64` - Cache hit counter
  - **Field** `misses: u64` - Cache miss counter

**Key Functions:**
- `new()` - Creates cache
- `get(&mut self, key: u64) -> Option<&V>` - Get with hit/miss tracking
- `put(&mut self, key: u64, value: V)` - Insert value
- `hit_rate() -> f64` - Calculate hit ratio
- `stats() -> CacheStats` - Return statistics

**Role Each Plays:**
- FxHashMap provides 10× faster hashing for u64 keys
- Statistics track cache effectiveness
- Used for request IDs, user IDs, timestamps

### Checkpoint Tests

```rust
#[test]
fn test_hot_cache_basic() {
    let mut cache = HotCache::new();
    cache.put(1, "value1");
    cache.put(2, "value2");

    assert_eq!(cache.get(1), Some(&"value1"));
    assert_eq!(cache.get(2), Some(&"value2"));
}

#[test]
fn test_hit_miss_tracking() {
    let mut cache = HotCache::new();
    cache.put(1, "a");

    cache.get(1); // hit
    cache.get(2); // miss
    cache.get(1); // hit

    let stats = cache.stats();
    assert_eq!(stats.hits, 2);
    assert_eq!(stats.misses, 1);
    assert_eq!(cache.hit_rate(), 2.0 / 3.0);
}

#[test]
fn benchmark_fxhash_vs_hashmap() {
    use std::collections::HashMap;
    use std::time::Instant;

    const N: u64 = 1_000_000;

    // Standard HashMap
    let start = Instant::now();
    let mut std_cache = HashMap::new();
    for i in 0..N {
        std_cache.insert(i, i * 2);
    }
    for i in 0..N {
        std_cache.get(&i);
    }
    let std_time = start.elapsed();

    // FxHashMap
    let start = Instant::now();
    let mut fx_cache = HotCache::new();
    for i in 0..N {
        fx_cache.put(i, i * 2);
    }
    for i in 0..N {
        fx_cache.get(i);
    }
    let fx_time = start.elapsed();

    println!("HashMap: {:?}", std_time);
    println!("FxHashMap: {:?}", fx_time);
    println!("Speedup: {:.2}x", std_time.as_secs_f64() / fx_time.as_secs_f64());
}
```

### Starter Code

```rust
use rustc_hash::FxHashMap;

#[derive(Debug)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
}

pub struct HotCache<V> {
    cache: FxHashMap<u64, V>,
    hits: u64,
    misses: u64,
}

impl<V> HotCache<V> {
    pub fn new() -> Self {
        // TODO: Initialize cache
        unimplemented!()
    }

    pub fn get(&mut self, key: u64) -> Option<&V> {
        // TODO: Get value and track hit/miss
        unimplemented!()
    }

    pub fn put(&mut self, key: u64, value: V) {
        // TODO: Insert value
        unimplemented!()
    }

    pub fn hit_rate(&self) -> f64 {
        // TODO: Calculate hits / (hits + misses)
        unimplemented!()
    }

    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits,
            misses: self.misses,
            size: self.cache.len(),
        }
    }
}
```

**Why previous step is not enough:** BTreeMap and HashMap use SipHash (secure but slow). For trusted integer keys in hot paths, speed matters more than security.

**What's the improvement:** FxHashMap for integer keys:
- HashMap with SipHash: ~150ns per operation
- FxHashMap: ~15ns per operation (10× faster)

For 1M cache operations:
- HashMap: 150ms
- FxHashMap: 15ms (savings add up in high-traffic systems)

---

## Step 4: Multi-Level Cache Strategy

### Introduction

Combine all three cache types in a tiered architecture: hot cache (FxHashMap) → LRU (HashMap) → TTL (BTreeMap).

### Architecture

**Structs:**
- `MultiLevelCache<K, V>` - Three-tier cache
  - **Field** `hot: HotCache<V>` - Level 1: Hot integer keys
  - **Field** `lru: LruCache<K, V>` - Level 2: LRU bounded cache
  - **Field** `ttl: TtlCache<K, V>` - Level 3: Long-term with expiry

**Key Functions:**
- `get(&mut self, key: &K, now: u64) -> Option<&V>` - Check all levels
- `put(&mut self, key: K, value: V, now: u64)` - Insert to appropriate level
- `promote(&mut self, key: &K)` - Move from lower to higher tier
- `stats() -> MultiLevelStats` - Statistics from all tiers

**Role Each Plays:**
- Hot cache: Frequently accessed integer keys (user sessions)
- LRU: Medium-frequency access, bounded size
- TTL: Infrequent access, time-based expiration

### Checkpoint Tests

```rust
#[test]
fn test_multi_level_lookup() {
    let mut cache = MultiLevelCache::new(10, 100);

    cache.put("key1", "value1", 1000);

    // Should be found
    assert_eq!(cache.get(&"key1", 1005), Some(&"value1"));
}

#[test]
fn test_promotion() {
    let mut cache = MultiLevelCache::new(5, 10);

    cache.put("key1", "value1", 1000);

    // Access multiple times to trigger promotion
    for _ in 0..10 {
        cache.get(&"key1", 1001);
    }

    // Verify it moved to hot tier
    let stats = cache.stats();
    assert!(stats.hot_size > 0);
}

#[test]
fn test_tier_eviction() {
    let mut cache = MultiLevelCache::new(2, 5);

    // Fill hot tier
    for i in 0..10 {
        cache.put(i, i * 2, 1000);
    }

    let stats = cache.stats();
    assert_eq!(stats.hot_size, 2);
    assert!(stats.lru_size > 0);
}
```

### Starter Code

```rust
#[derive(Debug)]
pub struct MultiLevelStats {
    pub hot_size: usize,
    pub hot_hits: u64,
    pub lru_size: usize,
    pub ttl_size: usize,
}

pub struct MultiLevelCache<K, V> {
    hot: HotCache<V>,
    lru: LruCache<K, V>,
    ttl: TtlCache<K, V>,
    hot_capacity: usize,
    lru_capacity: usize,
}

impl<K, V> MultiLevelCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(hot_capacity: usize, lru_capacity: usize) -> Self {
        // TODO: Initialize all three caches
        unimplemented!()
    }

    pub fn get(&mut self, key: &K, now: u64) -> Option<&V> {
        // TODO: Check hot → lru → ttl
        // If found in lower tier, consider promotion
        unimplemented!()
    }

    pub fn put(&mut self, key: K, value: V, now: u64) {
        // TODO: Insert to appropriate tier
        // Start in TTL, promote based on access
        unimplemented!()
    }

    pub fn stats(&self) -> MultiLevelStats {
        // TODO: Aggregate stats from all tiers
        unimplemented!()
    }
}
```

**Why previous step is not enough:** Single-tier caches have fixed characteristics. Real systems benefit from tiered caching matching access patterns.

**What's the improvement:** Multi-level cache optimizes for different access patterns:
- Hot tier: Ultra-fast for 1% of keys accounting for 50% of traffic
- LRU tier: Moderate speed for 10% of keys, 30% of traffic
- TTL tier: Bulk storage for remaining 20% of traffic

Result: Better average latency and memory efficiency.

---

## Step 5: Benchmark Suite

### Introduction

Comprehensive benchmarks comparing all map types and cache strategies.

### Architecture

**Benchmarks:**
1. HashMap vs BTreeMap vs FxHashMap insertion
2. HashMap vs BTreeMap vs FxHashMap lookups
3. LRU vs TTL vs Hot cache hit rates
4. Multi-level vs single-tier performance

### Starter Code

```rust
use std::time::Instant;

pub struct CacheBenchmarks;

impl CacheBenchmarks {
    pub fn run_all() {
        Self::bench_map_types();
        Self::bench_cache_strategies();
        Self::bench_multi_level();
    }

    fn bench_map_types() {
        println!("=== Map Type Comparison ===");

        const N: usize = 1_000_000;

        // TODO: Benchmark HashMap
        // TODO: Benchmark BTreeMap
        // TODO: Benchmark FxHashMap

        // Measure insertion, lookup, iteration
    }

    fn bench_cache_strategies() {
        println!("=== Cache Strategy Comparison ===");

        // TODO: Simulate workload on LRU
        // TODO: Simulate workload on TTL
        // TODO: Simulate workload on Hot cache

        // Compare hit rates, throughput
    }

    fn bench_multi_level() {
        println!("=== Multi-Level Cache ===");

        // TODO: Compare single-tier vs multi-tier
        // Measure average latency, memory usage
    }
}
```

---

## Step 6: Real-World Cache Patterns

### Introduction

Implement common caching patterns: read-through, write-through, cache-aside.

### Architecture

**Patterns:**
- **Cache-aside**: Application manages cache explicitly
- **Read-through**: Cache loads from backing store on miss
- **Write-through**: Writes go to cache and backing store

### Starter Code

```rust
pub trait BackingStore<K, V> {
    fn load(&self, key: &K) -> Option<V>;
    fn save(&mut self, key: K, value: V);
}

pub struct ReadThroughCache<K, V, S> {
    cache: LruCache<K, V>,
    store: S,
}

impl<K, V, S> ReadThroughCache<K, V, S>
where
    K: Eq + Hash + Clone,
    V: Clone,
    S: BackingStore<K, V>,
{
    pub fn get(&mut self, key: &K) -> Option<V> {
        // TODO: Check cache first
        // On miss, load from store and populate cache
        unimplemented!()
    }
}

pub struct WriteThroughCache<K, V, S> {
    cache: LruCache<K, V>,
    store: S,
}

impl<K, V, S> WriteThroughCache<K, V, S>
where
    K: Eq + Hash + Clone,
    V: Clone,
    S: BackingStore<K, V>,
{
    pub fn put(&mut self, key: K, value: V) {
        // TODO: Write to both cache and store
        unimplemented!()
    }
}
```

**Why previous step is not enough:** Benchmarks show performance but not integration patterns. Real caches interact with databases, APIs, file systems.

**What's the improvement:** Cache patterns standardize integration:
- Cache-aside: Flexible but requires explicit cache management
- Read-through: Simplifies reads, automatic population
- Write-through: Guarantees consistency between cache and store

---

## Complete Working Example

```rust
fn main() {
    println!("=== Multi-Tier Cache Demo ===\n");

    // Step 1: LRU Cache
    println!("Step 1: LRU Cache");
    let mut lru = LruCache::new(3);
    lru.put("a", 1);
    lru.put("b", 2);
    lru.put("c", 3);
    lru.put("d", 4); // Evicts "a"
    println!("After inserting a,b,c,d with capacity 3:");
    println!("  Contains 'a': {}", lru.get(&"a").is_some());
    println!("  Contains 'd': {}", lru.get(&"d").is_some());

    // Step 2: TTL Cache
    println!("\nStep 2: TTL Cache");
    let mut ttl = TtlCache::new(10);
    ttl.put("session1", "user123", 1000);
    println!("At time 1005: {:?}", ttl.get(&"session1", 1005));
    println!("At time 1015: {:?}", ttl.get(&"session1", 1015));

    // Step 3: Hot Cache
    println!("\nStep 3: Hot Cache");
    let mut hot = HotCache::new();
    hot.put(1, "fast");
    hot.put(2, "cache");
    hot.get(1);
    hot.get(1);
    hot.get(3); // miss
    println!("Hit rate: {:.2}", hot.hit_rate());

    // Step 4: Multi-Level
    println!("\nStep 4: Multi-Level Cache");
    let mut multi = MultiLevelCache::new(2, 10);
    for i in 0..20 {
        multi.put(i, i * 2, 1000);
    }
    let stats = multi.stats();
    println!("Hot tier: {} entries", stats.hot_size);
    println!("LRU tier: {} entries", stats.lru_size);
    println!("TTL tier: {} entries", stats.ttl_size);

    // Step 5: Benchmarks
    println!("\nStep 5: Running Benchmarks");
    CacheBenchmarks::run_all();
}
```

### Complete Working Example

```rust
use rustc_hash::FxHashMap;
use std::collections::{hash_map::DefaultHasher, BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use std::time::Instant;

// =============================================================================
// Milestone 1: LRU Cache with HashMap
// =============================================================================

pub struct LruCache<K, V> {
    map: HashMap<K, (V, usize)>,
    capacity: usize,
    access_counter: usize,
}

impl<K, V> LruCache<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            capacity,
            access_counter: 0,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some((value, time)) = self.map.get_mut(key) {
            self.access_counter += 1;
            *time = self.access_counter;
            Some(value)
        } else {
            None
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        self.access_counter += 1;
        if self.map.contains_key(&key) {
            if let Some(entry) = self.map.get_mut(&key) {
                entry.0 = value;
                entry.1 = self.access_counter;
            }
            return;
        }
        if self.capacity > 0 && self.map.len() >= self.capacity {
            self.evict_lru();
        }
        self.map.insert(key, (value, self.access_counter));
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    fn evict_lru(&mut self) {
        if let Some((key, _)) = self
            .map
            .iter()
            .min_by_key(|(_, (_, time))| *time)
            .map(|(k, v)| (k.clone(), v.1))
        {
            self.map.remove(&key);
        }
    }
}

// =============================================================================
// Milestone 2: Time-Based Expiration with BTreeMap
// =============================================================================

pub struct TtlCache<K, V> {
    data: HashMap<K, V>,
    expiry: BTreeMap<u64, Vec<K>>,
    key_expiry: HashMap<K, u64>,
    default_ttl: u64,
}

impl<K, V> TtlCache<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn new(default_ttl: u64) -> Self {
        Self {
            data: HashMap::new(),
            expiry: BTreeMap::new(),
            key_expiry: HashMap::new(),
            default_ttl,
        }
    }

    pub fn put(&mut self, key: K, value: V, now: u64) {
        let expiry_time = now + self.default_ttl;
        if let Some(old_expiry) = self.key_expiry.insert(key.clone(), expiry_time) {
            if let Some(keys) = self.expiry.get_mut(&old_expiry) {
                keys.retain(|k| k != &key);
                if keys.is_empty() {
                    self.expiry.remove(&old_expiry);
                }
            }
        }
        self.data.insert(key.clone(), value);
        self.expiry
            .entry(expiry_time)
            .or_insert_with(Vec::new)
            .push(key);
    }

    pub fn get(&mut self, key: &K, now: u64) -> Option<&V> {
        if let Some(&expiry_time) = self.key_expiry.get(key) {
            if expiry_time <= now {
                if let Some(expiry_time) = self.key_expiry.remove(key) {
                    if let Some(keys) = self.expiry.get_mut(&expiry_time) {
                        keys.retain(|k| k != key);
                        if keys.is_empty() {
                            self.expiry.remove(&expiry_time);
                        }
                    }
                }
                self.data.remove(key);
                return None;
            }
            return self.data.get(key);
        }
        None
    }

    pub fn cleanup(&mut self, now: u64) {
        self.cleanup_range(now);
    }

    pub fn cleanup_range(&mut self, until: u64) {
        let expired: Vec<u64> = self.expiry.range(..=until).map(|(&time, _)| time).collect();
        for time in expired {
            if let Some(keys) = self.expiry.remove(&time) {
                for key in keys {
                    self.data.remove(&key);
                    self.key_expiry.remove(&key);
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

// =============================================================================
// Milestone 3: FxHash Hot Cache
// =============================================================================

#[derive(Debug)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
}

pub struct HotCache<V> {
    cache: FxHashMap<u64, V>,
    hits: u64,
    misses: u64,
}

impl<V> HotCache<V> {
    pub fn new() -> Self {
        Self {
            cache: FxHashMap::default(),
            hits: 0,
            misses: 0,
        }
    }

    pub fn get(&mut self, key: u64) -> Option<&V> {
        if let Some(value) = self.cache.get(&key) {
            self.hits += 1;
            Some(value)
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn put(&mut self, key: u64, value: V) {
        self.cache.insert(key, value);
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits,
            misses: self.misses,
            size: self.cache.len(),
        }
    }
}

// =============================================================================
// Milestone 4: Multi-Level Cache
// =============================================================================

#[derive(Debug)]
pub struct MultiLevelStats {
    pub hot_size: usize,
    pub hot_hits: u64,
    pub lru_size: usize,
    pub ttl_size: usize,
}

pub struct MultiLevelCache<K, V> {
    hot: HotCache<V>,
    lru: LruCache<K, V>,
    ttl: TtlCache<K, V>,
    hot_capacity: usize,
}

impl<K, V> MultiLevelCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(hot_capacity: usize, lru_capacity: usize) -> Self {
        Self {
            hot: HotCache::new(),
            lru: LruCache::new(lru_capacity),
            ttl: TtlCache::new(60),
            hot_capacity,
        }
    }

    pub fn get(&mut self, key: &K, now: u64) -> Option<&V> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hot_key = hasher.finish();
        let _hot_hit = self.hot.get(hot_key).is_some();
        let lru_clone = self.lru.get(key).map(|v| v.clone());
        if let Some(cloned) = lru_clone {
            if self.hot.stats().size < self.hot_capacity {
                self.hot.put(hot_key, cloned);
            }
            return self.lru.get(key);
        }
        if let Some(cloned) = self.ttl.get(key, now).map(|v| v.clone()) {
            let hot_copy = cloned.clone();
            self.lru.put(key.clone(), cloned);
            if self.hot.stats().size < self.hot_capacity {
                self.hot.put(hot_key, hot_copy);
            }
            return self.lru.get(key);
        }
        None
    }

    pub fn put(&mut self, key: K, value: V, now: u64) {
        self.ttl.put(key.clone(), value.clone(), now);
        self.lru.put(key.clone(), value.clone());
        if self.hot.stats().size < self.hot_capacity {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            let hot_key = hasher.finish();
            self.hot.put(hot_key, value);
        }
    }

    pub fn stats(&self) -> MultiLevelStats {
        let hot_stats = self.hot.stats();
        MultiLevelStats {
            hot_size: hot_stats.size,
            hot_hits: hot_stats.hits,
            lru_size: self.lru.len(),
            ttl_size: self.ttl.len(),
        }
    }
}

// =============================================================================
// Milestone 5: Benchmark Suite
// =============================================================================

pub struct CacheBenchmarks;

impl CacheBenchmarks {
    pub fn run_all() {
        Self::bench_map_types();
        Self::bench_cache_strategies();
        Self::bench_multi_level();
    }

    fn bench_map_types() {
        println!("=== Map Type Comparison ===");
        const N: usize = 100_000;

        let start = Instant::now();
        let mut hash_map = HashMap::new();
        for i in 0..N {
            hash_map.insert(i, i);
        }
        let hash_insert = start.elapsed();
        let start = Instant::now();
        for i in 0..N {
            let _ = hash_map.get(&i);
        }
        let hash_lookup = start.elapsed();

        let start = Instant::now();
        let mut btree = BTreeMap::new();
        for i in 0..N {
            btree.insert(i, i);
        }
        let bt_insert = start.elapsed();
        let start = Instant::now();
        for i in 0..N {
            let _ = btree.get(&i);
        }
        let bt_lookup = start.elapsed();

        let start = Instant::now();
        let mut fx = FxHashMap::default();
        for i in 0..N {
            fx.insert(i, i);
        }
        let fx_insert = start.elapsed();
        let start = Instant::now();
        for i in 0..N {
            let _ = fx.get(&i);
        }
        let fx_lookup = start.elapsed();

        println!(
            "HashMap insert {:?}, lookup {:?}\nBTreeMap insert {:?}, lookup {:?}\nFxHashMap insert {:?}, lookup {:?}",
            hash_insert, hash_lookup, bt_insert, bt_lookup, fx_insert, fx_lookup
        );
    }

    fn bench_cache_strategies() {
        println!("=== Cache Strategy Comparison ===");
        let mut lru = LruCache::new(1000);
        let mut ttl = TtlCache::new(60);
        let mut hot = HotCache::new();

        for i in 0..10_000 {
            lru.put(i, i);
            ttl.put(i, i, i as u64);
            hot.put(i as u64, i);
        }

        for i in 0..10_000 {
            let _ = lru.get(&i);
            let _ = ttl.get(&i, (i + 30) as u64);
            let _ = hot.get(i as u64);
        }

        println!(
            "LRU size {}, TTL size {}, Hot hit rate {:.2}",
            lru.len(),
            ttl.len(),
            hot.hit_rate()
        );
    }

    fn bench_multi_level() {
        println!("=== Multi-Level Cache ===");
        let mut single = LruCache::new(2000);
        let mut multi = MultiLevelCache::new(500, 1500);

        for i in 0..10_000 {
            single.put(i, i);
            multi.put(i, i, 0);
        }

        let start = Instant::now();
        for i in 0..10_000 {
            let _ = single.get(&i);
        }
        let single_time = start.elapsed();

        let start = Instant::now();
        for i in 0..10_000 {
            let _ = multi.get(&i, 0);
        }
        let multi_time = start.elapsed();

        println!(
            "Single-tier: {:?}, Multi-tier: {:?}",
            single_time, multi_time
        );
    }
}

// =============================================================================
// Milestone 6: Read-Through / Write-Through Caches
// =============================================================================

pub trait BackingStore<K, V> {
    fn load(&self, key: &K) -> Option<V>;
    fn save(&mut self, key: K, value: V);
}

pub struct ReadThroughCache<K, V, S> {
    cache: LruCache<K, V>,
    store: S,
}

impl<K, V, S> ReadThroughCache<K, V, S>
where
    K: Eq + Hash + Clone,
    V: Clone,
    S: BackingStore<K, V>,
{
    pub fn new(cache: LruCache<K, V>, store: S) -> Self {
        Self { cache, store }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(value) = self.cache.get(key) {
            return Some(value.clone());
        }
        if let Some(value) = self.store.load(key) {
            self.cache.put(key.clone(), value.clone());
            return Some(value);
        }
        None
    }
}

pub struct WriteThroughCache<K, V, S> {
    cache: LruCache<K, V>,
    store: S,
}

impl<K, V, S> WriteThroughCache<K, V, S>
where
    K: Eq + Hash + Clone,
    V: Clone,
    S: BackingStore<K, V>,
{
    pub fn new(cache: LruCache<K, V>, store: S) -> Self {
        Self { cache, store }
    }

    pub fn put(&mut self, key: K, value: V) {
        self.store.save(key.clone(), value.clone());
        self.cache.put(key, value);
    }
}

fn main() {
    CacheBenchmarks::run_all();
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lru_eviction() {
        let mut cache = LruCache::new(2);
        cache.put("a", 1);
        cache.put("b", 2);
        cache.get(&"a");
        cache.put("c", 3);
        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.get(&"b"), None);
    }

    #[test]
    fn ttl_expiration() {
        let mut cache = TtlCache::new(5);
        cache.put("key", "value", 100);
        assert_eq!(cache.get(&"key", 104), Some(&"value"));
        assert_eq!(cache.get(&"key", 106), None);
    }

    #[test]
    fn hot_cache_stats() {
        let mut cache = HotCache::new();
        cache.put(1, "a");
        cache.get(1);
        cache.get(2);
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(cache.hit_rate(), 0.5);
    }

    #[test]
    fn multi_level_operations() {
        let mut cache = MultiLevelCache::new(2, 5);
        cache.put("a", 1, 0);
        cache.put("b", 2, 0);
        assert_eq!(cache.get(&"a", 1), Some(&1));
        let stats = cache.stats();
        assert!(stats.ttl_size >= 2);
    }

    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct SharedStore<K, V>
    where
        K: Eq + Hash,
    {
        data: Arc<Mutex<HashMap<K, V>>>,
    }

    impl<K, V> SharedStore<K, V>
    where
        K: Eq + Hash,
    {
        fn new() -> Self {
            Self {
                data: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    impl<K, V> BackingStore<K, V> for SharedStore<K, V>
    where
        K: Eq + Hash + Clone,
        V: Clone,
    {
        fn load(&self, key: &K) -> Option<V> {
            self.data.lock().unwrap().get(key).cloned()
        }

        fn save(&mut self, key: K, value: V) {
            self.data.lock().unwrap().insert(key, value);
        }
    }

    #[test]
    fn read_through_cache() {
        let store = SharedStore::new();
        {
            let mut locked = store.data.lock().unwrap();
            locked.insert("k".to_string(), 42);
        }
        let cache = LruCache::new(2);
        let mut rtc = ReadThroughCache::new(cache, store);
        assert_eq!(rtc.get(&"k".to_string()), Some(42));
    }

    #[test]
    fn write_through_cache() {
        let store = SharedStore::new();
        let mirror = store.clone();
        let cache = LruCache::new(2);
        let mut wtc = WriteThroughCache::new(cache, store);
        wtc.put("k".to_string(), 5);
        assert_eq!(mirror.load(&"k".to_string()), Some(5));
    }
}

```
