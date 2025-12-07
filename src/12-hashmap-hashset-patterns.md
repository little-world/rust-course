# HashMap & HashSet Patterns

`HashMap<K, V>` is Rust's primary hash table implementation, offering excellent average-case performance for lookups, insertions, and deletions. This chapter explores key patterns for using `HashMap` and `HashSet` effectively, from basic operations to advanced techniques for performance, memory optimization, and concurrency.

## Pattern 1: The Entry API

The Entry API is the most idiomatic and efficient way to handle complex conditional logic in `HashMap`, such as "insert if absent, update if present."

-   **Problem**:
    -   **Inefficiency**: A common but inefficient pattern is to first check if a key exists (`contains_key`), and then perform a separate operation to insert or update the value. This results in at least two separate hash lookups.
    -   **Verbosity**: Writing `if map.contains_key(key) { ... } else { ... }` is verbose, clutters the code, and can be difficult to read.
    -   **Complexity**: Implementing common patterns like incrementing a counter or appending to a vector stored as a map value requires careful and often awkward handling of `Option`.

-   **Solution**:
    -   **The Entry API**: Use `map.entry(key)`, which performs a single lookup and returns an `Entry` enum. This enum represents the state of the key's slot in the map.
    -   **`Entry::Occupied`**: If the key exists, you get an `OccupiedEntry`, which allows you to view or modify the existing value in place.
    -   **`Entry::Vacant`**: If the key does not exist, you get a `VacantEntry`, which allows you to insert a new value.
    -   **Convenience Methods**: The API provides powerful, chainable methods like `.or_insert(default_value)`, `.or_insert_with(|| default_fn())`, and `.and_modify(|value| { ... })` to handle common cases in a single, fluent line of code.

-   **Why It Matters**:
    -   **Performance**: The Entry API reduces hash lookups from two or more down to just one, which can double the performance of lookup-heavy operations like word counting or building aggregations.
    -   **Readability**: It produces much cleaner, more idiomatic Rust code that clearly expresses the intended logic.
    -   **Atomicity**: While `HashMap` itself is not thread-safe, the Entry API provides an atomic-like interface for single-threaded operations, preventing certain logical bugs that can arise from separate check-then-insert logic.

-   **Use Cases**:
    -   **Frequency Counting**: Counting occurrences of words, characters, or any other item.
    -   **Grouping and Aggregation**: Grouping a list of items by a key and calculating sums, counts, or averages for each group.
    -   **Cache Implementation**: Efficiently managing entries in a cache, like an LRU cache.
    -   **Building Adjacency Lists**: Constructing graph data structures by adding edges between nodes.
    -   **Default Value Initialization**: Ensuring a key has a default value before modifying it.

### Example 1: Word Frequency Counter

The classic use case for the Entry API is counting word frequencies. The `.or_insert(0)` method gets the current count for a word or inserts `0` if the word is new. We can then increment the count in place.

```rust
use std::collections::HashMap;

fn word_frequency_counter() {
    let text = "the quick brown fox jumps over the lazy dog";
    let mut counts: HashMap<String, usize> = HashMap::new();

    for word in text.split_whitespace() {
        // Get the entry for the word, insert 0 if it's vacant,
        // and then get a mutable reference to the value to increment it.
        *counts.entry(word.to_string()).or_insert(0) += 1;
    }

    println!("Word counts: {:?}", counts);
    // Top word is "the" with a count of 2.
    assert_eq!(counts.get("the"), Some(&2));
}
```

### Example 2: Grouping Items by Key

The Entry API is perfect for grouping items from a list into a `HashMap` where keys are a property of the item and values are a `Vec` of items sharing that property. `.or_insert_with(Vec::new)` is used here to lazily create a new vector only when a new key is encountered.

```rust
use std::collections::HashMap;

#[derive(Debug)]
struct Sale {
    category: String,
    amount: f64,
}

fn group_sales_by_category() {
    let sales = vec![
        Sale { category: "Electronics".to_string(), amount: 1200.0 },
        Sale { category: "Furniture".to_string(), amount: 450.0 },
        Sale { category: "Electronics".to_string(), amount: 25.0 },
    ];

    let mut sales_by_category: HashMap<String, Vec<Sale>> = HashMap::new();

    for sale in sales {
        // Find the vector for the category, creating a new one if it doesn't exist,
        // and then push the sale into it.
        sales_by_category
            .entry(sale.category.clone())
            .or_insert_with(Vec::new)
            .push(sale);
    }

    println!("Sales grouped by category: {:?}", sales_by_category);
    assert_eq!(sales_by_category.get("Electronics").unwrap().len(), 2);
}
```

### Example 3: Implementing an LRU Cache

The Entry API can be used to implement more complex data structures like a Least Recently Used (LRU) cache. Here, we use `Entry::Occupied` and `Entry::Vacant` to handle the logic for existing and new cache entries separately.

```rust
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

struct LruCache<K, V> {
    capacity: usize,
    map: HashMap<K, V>,
    order: VecDeque<K>, // Tracks usage order, from least to most recent.
}

impl<K: Eq + Hash + Clone> LruCache<K, V> {
    fn new(capacity: usize) -> Self {
        Self { capacity, map: HashMap::new(), order: VecDeque::new() }
    }

    fn put(&mut self, key: K, value: V) {
        use std::collections::hash_map::Entry;

        match self.map.entry(key.clone()) {
            Entry::Occupied(mut entry) => {
                // Key already exists, update the value.
                entry.insert(value);
                // Move it to the front of the usage queue.
                self.order.retain(|k| k != &key);
                self.order.push_back(key);
            }
            Entry::Vacant(entry) => {
                // Key is new. First, check if we need to evict an old entry.
                if self.map.len() >= self.capacity {
                    if let Some(lru_key) = self.order.pop_front() {
                        self.map.remove(&lru_key);
                    }
                }
                // Insert the new value and add it to the front of the usage queue.
                entry.insert(value);
                self.order.push_back(key);
            }
        }
    }
}
```

## Pattern 2: Custom Hashing and Equality

By default, `HashMap` uses a secure but slower hashing algorithm (SipHash) and relies on the standard `Eq` and `Hash` traits. For many use cases, providing a custom implementation is necessary for correctness or performance.

-   **Problem**:
    -   **Semantic Equality**: The default `Hash` and `PartialEq` derived for a type may not match the desired semantic equality. For example, you might want string keys to be case-insensitive, or floating-point keys to have a tolerance.
    -   **Performance**: The default SipHash algorithm is designed to be resistant to denial-of-service attacks, but it's slow. For keys that are trusted (e.g., internal integer IDs), this is unnecessary overhead.
    -   **Unsupported Types**: Some types, like `f64`, don't implement `Hash` by default because of their properties (e.g., `NaN != NaN`), making them unusable as keys without a wrapper.

-   **Solution**:
    -   **Implement `Hash` and `PartialEq`**: Manually implement the `Hash` and `PartialEq` traits for your key type. It is a critical invariant that if `a == b`, then `hash(a) == hash(b)`.
    -   **Newtype Wrappers**: Create a newtype wrapper (e.g., `struct CaseInsensitiveString(String)`) around an existing type and implement the custom hashing and equality logic on the wrapper. This is a clean way to add custom behavior without modifying the original type.
    -   **Alternative Hashers**: For performance-critical scenarios with trusted keys, swap out the default hasher for a faster, non-cryptographic one like `FxHasher` (from `rustc-hash`) or `AHasher` (from `ahash`).

-   **Why It Matters**:
    -   **Correctness**: Custom equality and hashing are essential for creating maps that behave correctly according to your application's domain logic. An incorrect `Hash` implementation can lead to keys being lost or not found in the map.
    -   **Major Performance Gains**: Switching to a faster hasher like `FxHasher` for integer keys can result in a 2-10x speedup for `HashMap`-heavy operations, which is critical in hot loops, compilers, or game engines.
    -   **Enabling New Key Types**: By implementing `Hash`, you can use complex custom types, composite keys, or even floating-point numbers as `HashMap` keys.

-   **Use Cases**:
    -   **Case-Insensitive Keys**: For usernames, HTTP headers, or configuration keys.
    -   **Performance-Critical Maps**: In compilers (symbol tables), game engines (entity IDs), or any hot path that heavily uses a map with trusted integer keys.
    -   **Composite Keys**: Using a struct with multiple fields as a single key.
    -   **Floating-Point Keys**: For spatial indexing or scientific computing, typically by wrapping floats to handle `NaN` and rounding for approximate equality.

### Example 1: Case-Insensitive String Keys

To make a `HashMap` treat string keys as case-insensitive, we can create a newtype wrapper around `String`. We then implement `PartialEq` to compare strings case-insensitively and `Hash` to hash their lowercase versions.

```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, Eq)]
struct CaseInsensitiveString(String);

impl Hash for CaseInsensitiveString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the lowercase version of the string to ensure "A" and "a" have the same hash.
        for byte in self.0.bytes().map(|b| b.to_ascii_lowercase()) {
            byte.hash(state);
        }
    }
}

impl PartialEq for CaseInsensitiveString {
    fn eq(&self, other: &Self) -> bool {
        // Compare the strings case-insensitively.
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

fn case_insensitive_headers() {
    let mut headers = HashMap::new();

    headers.insert(CaseInsensitiveString("Content-Type".to_string()), "application/json");
    headers.insert(CaseInsensitiveString("X-Request-ID".to_string()), "12345");

    // Lookup is case-insensitive.
    let key = CaseInsensitiveString("content-type".to_string());
    assert_eq!(headers.get(&key), Some(&"application/json"));
}
```

### Example 2: Faster Hashing with FxHashMap

For performance-critical code paths where the keys are trusted (not controlled by a potential attacker), you can replace the standard `HashMap` with `FxHashMap` from the `rustc-hash` crate. It uses a much faster, non-cryptographic hash function.

```rust
// Add `rustc-hash = "1.1"` to Cargo.toml
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::time::Instant;

fn benchmark_fxhashmap() {
    const SIZE: usize = 1_000_000;
    
    // Benchmark FxHashMap
    let start = Instant::now();
    let mut fx_map = FxHashMap::default();
    for i in 0..SIZE {
        fx_map.insert(i, i);
    }
    println!("FxHashMap insertion time: {:?}", start.elapsed());

    // Benchmark standard HashMap
    let start = Instant::now();
    let mut std_map = std::collections::HashMap::new();
    for i in 0..SIZE {
        std_map.insert(i, i);
    }
    println!("Standard HashMap insertion time: {:?}", start.elapsed());
}
```

## Pattern 3: Capacity and Memory Management

Failing to manage `HashMap` capacity can lead to poor performance due to repeated resizing, or wasted memory from over-allocation.

-   **Problem**:
    -   **Latency Spikes**: When a `HashMap` reaches its capacity, it must resize, which involves allocating a new, larger backing array and re-hashing and moving every single element. This can cause significant latency spikes in performance-sensitive applications.
    -   **Wasted Memory**: A `HashMap` that grows and then shrinks may hold on to a large, mostly empty allocation. For long-lived maps, this is a form of memory leak.
    -   **Inefficient Construction**: Building a large `HashMap` from scratch without setting an initial capacity is very inefficient, as it will trigger many resize operations.

-   **Solution**:
    -   **Pre-allocation**: If you know the final size of the map, use `HashMap::with_capacity(n)` to pre-allocate the required memory upfront, avoiding all intermediate resizes.
    -   **`reserve`**: Before inserting a batch of items, use `map.reserve(additional_items)` to ensure there is enough capacity, preventing a resize during the insertion loop.
    -   **`shrink_to_fit`**: After a map has been built or has had many items removed, call `map.shrink_to_fit()` to release any unused memory back to the system. This is useful for long-lived, read-mostly maps.
    -   **String Interning**: For maps with many duplicate string values, use a string interner to store each unique string only once and use a cheap, lightweight ID in the map.

-   **Why It Matters**:
    -   **Performance**: Pre-allocating capacity can make the construction of large `HashMap`s 3-10x faster. A map of 1 million entries could trigger ~20 resize operations if not pre-allocated.
    -   **Memory Efficiency**: `shrink_to_fit` and string interning can dramatically reduce the memory footprint of your application, which is critical in memory-constrained environments like embedded systems or for services running at massive scale.
    -   **Predictable Performance**: Avoiding resizes in hot paths ensures that your application has more predictable, lower-latency performance.

-   **Use Cases**:
    -   **Batch Data Loading**: When loading a large dataset from a file or database, pre-allocate the map with the size of the dataset.
    -   **Configuration Maps**: A global configuration map that is loaded at startup and then read frequently should be shrunk to fit after loading.
    -   **High-Frequency Trading / Real-Time Systems**: Any system where latency spikes are unacceptable must carefully manage capacity to avoid resizes.
    -   **Memory-Constrained Environments**: Any application where memory usage is a primary concern.

### Example 1: Pre-allocating for Batch Processing

When you know roughly how many items you're going to insert, using `HashMap::with_capacity` can dramatically speed up insertion by avoiding repeated resizing and re-hashing.

```rust
use std::collections::HashMap;
use std::time::Instant;

fn batch_processing_with_capacity() {
    const BATCH_SIZE: usize = 500_000;

    // Without pre-allocation
    let start = Instant::now();
    let mut map1 = HashMap::new();
    for i in 0..BATCH_SIZE {
        map1.insert(i, i);
    }
    println!("Without pre-allocation: {:?}, final capacity: {}", start.elapsed(), map1.capacity());

    // With pre-allocation
    let start = Instant::now();
    let mut map2 = HashMap::with_capacity(BATCH_SIZE);
    for i in 0..BATCH_SIZE {
        map2.insert(i, i);
    }
    println!("With pre-allocation: {:?}, final capacity: {}", start.elapsed(), map2.capacity());
}
```

### Example 2: Shrinking to Reclaim Memory

For a long-lived `HashMap` that is populated once and then mostly read from, you can call `shrink_to_fit()` after population to release any excess memory capacity.

```rust
use std::collections::HashMap;

fn shrinking_to_fit() {
    let mut map = HashMap::with_capacity(1000);
    println!("Initial capacity: {}", map.capacity());

    for i in 0..100 {
        map.insert(i, i);
    }
    println!("Capacity after 100 insertions: {}", map.capacity());
    
    // Shrink the map to reclaim the unused capacity.
    map.shrink_to_fit();
    println!("Capacity after shrinking: {}", map.capacity());
}
```

## Pattern 4: Alternative Map Implementations

`HashMap` is a great default, but the Rust ecosystem offers several other map implementations that are better suited for specific use cases.

-   **Problem**:
    -   **Order**: `HashMap` does not preserve insertion order, and its iteration order is effectively random. This is problematic if you need deterministic output or want to iterate over items in the order they were added.
    -   **Sorted Keys**: `HashMap` does not keep its keys sorted, making range queries (e.g., "get all items with keys between X and Y") very inefficient (O(N)).
    -   **Overhead for Small Maps**: For very small maps (e.g., fewer than 10-20 entries), the overhead of `HashMap` can be significant.

-   **Solution**:
    -   **`BTreeMap`**: A map based on a B-Tree. It keeps its keys sorted at all times. Use this when you need sorted iteration or efficient range queries.
    -   **`IndexMap`**: A map that preserves the original insertion order of keys. It's a drop-in replacement for `HashMap` with slightly higher overhead.
    -   **Array / `SmallVec`**: For very small, fixed-size maps, a simple array of key-value pairs or a `smallvec` can be faster and more memory-efficient than a full-blown `HashMap`.

-   **Why It Matters**:
    -   **Choosing the Right Tool**: Using the right map for the job can lead to simpler code and better performance. Using a `BTreeMap` for range queries can turn an O(N) scan into an efficient O(log N) operation.
    -   **Deterministic Behavior**: `IndexMap` and `BTreeMap` provide deterministic iteration order, which is essential for reproducible tests, consistent serialization (e.g., JSON), and any logic that depends on order.
    -   **Performance Trade-offs**: While `HashMap` has O(1) average-case lookups, `BTreeMap` has O(log N) lookups. Understanding this trade-off is key. `BTreeMap` is slower for random lookups but enables features `HashMap` lacks.

-   **Use Cases**:
    -   **`BTreeMap`**:
        -   **Leaderboards / Rankings**: Keeping scores sorted.
        -   **Time-Series Data**: Querying data within a specific time range.
        -   **Deterministic Serialization**: Ensuring keys in a serialized format like JSON are always in the same order.
    -   **`IndexMap`**:
        -   **Ordered Configuration**: Preserving the order of keys from a config file.
        -   **LRU Caches**: `IndexMap` is often a great choice for LRU caches as it naturally tracks insertion order.
        -   **Remembering User Choices**: Displaying items in the order a user added them.

### Example 1: BTreeMap for Ordered Operations

`BTreeMap` keeps its keys sorted. This makes it ideal for use cases that require ordered iteration or range queries, like a leaderboard or time-series data.

```rust
use std::collections::BTreeMap;

fn leaderboard() {
    // Scores are the keys, so they are kept sorted.
    let mut scores = BTreeMap::new();
    scores.insert(1500, "Alice".to_string());
    scores.insert(2200, "David".to_string());
    scores.insert(1800, "Charlie".to_string());

    // `iter()` returns items in sorted key order. `.rev()` gets us descending order for a top-down leaderboard.
    println!("Leaderboard (Top 3):");
    for (score, name) in scores.iter().rev().take(3) {
        println!("- {}: {}", name, score);
    }
    
    // BTreeMap also supports efficient range queries.
    println!("\nPlayers with scores between 1500 and 2000:");
    for (score, name) in scores.range(1500..=2000) {
        println!("- {}: {}", name, score);
    }
}
```

### Example 2: IndexMap for Insertion Order Preservation

`IndexMap` is a drop-in replacement for `HashMap` that remembers the order in which keys were inserted. This is useful for creating ordered JSON objects or any other scenario where order matters.

```rust
// Add `indexmap = "2.0"` to Cargo.toml
use indexmap::IndexMap;

fn ordered_json() {
    let mut user_data = IndexMap::new();

    // The insertion order is preserved.
    user_data.insert("id", "123".to_string());
    user_data.insert("name", "Alice".to_string());
    user_data.insert("email", "alice@example.com".to_string());

    // When serialized (e.g., to JSON), the fields will appear in the order they were inserted.
    // This is not guaranteed with a standard HashMap.
    let as_vec: Vec<_> = user_data.iter().map(|(k,v)| (k.to_string(), v.clone())).collect();
    println!("Fields in insertion order: {:?}", as_vec);
}
```

## Pattern 5: Concurrent HashMaps

A standard `HashMap` cannot be safely shared across multiple threads. While `Arc<Mutex<HashMap>>` is a valid approach, it suffers from heavy lock contention.

-   **Problem**:
    -   **High Contention**: Wrapping a `HashMap` in a `Mutex` or `RwLock` means that only one thread (for `Mutex`) or one writer (for `RwLock`) can access the map at a time, regardless of which key they are trying to access. On a multi-core machine, this becomes a major performance bottleneck.
    -   **Complex Implementation**: Building a more granular, sharded map by hand (e.g., a `Vec<Mutex<HashMap>>`) is complex and error-prone.

-   **Solution**:
    -   **`DashMap`**: The `dashmap` crate provides a concurrent `HashMap` that is sharded internally. It stripes the map across many small, independent locks. This means that two threads trying to access different keys are very unlikely to block each other, allowing for high-throughput concurrent access.
    -   **`RwLock<HashMap>`**: For workloads that are overwhelmingly read-heavy (e.g., >90% reads), an `RwLock` can be a good choice. It allows unlimited concurrent readers, but a writer will block all other readers and writers.

-   **Why It Matters**:
    -   **Scalability**: A concurrent map like `DashMap` can scale almost linearly with the number of CPU cores for many workloads, whereas a `Mutex`-wrapped `HashMap` does not scale at all. This is critical for building high-performance, multi-threaded applications like web servers, databases, and caches.
    -   **Simplicity**: `DashMap` provides a simple, `HashMap`-like API, hiding the complexity of the internal sharding and locking.

-   **Use Cases**:
    -   **High-Concurrency Caches**: A shared in-memory cache in a web server that is read from and written to by many request-handling threads.
    -   **Session Stores**: Storing user session data that is accessed concurrently.
    -   **Request Counters / Metrics**: Atomically incrementing counters for different endpoints or metrics from many threads.
    -   **Any shared, mutable key-value store in a multi-threaded application.**

### Example: Concurrent Request Counter with DashMap

`DashMap` provides an API similar to `HashMap` but is designed for high-concurrency scenarios. It allows multiple threads to read and write to the map at the same time with minimal blocking.

```rust
// Note: Add `dashmap = "5.5"` and `rayon = "1.8"` to Cargo.toml
use dashmap::DashMap;
use std::sync::Arc;
use rayon::prelude::*

fn concurrent_request_counter() {
    let counters = Arc::new(DashMap::new());

    // Simulate 1000 concurrent requests to different endpoints.
    (0..1000).into_par_iter().for_each(|i| {
        let endpoint = format!("/endpoint_{}", i % 10);
        // DashMap's entry API is similar to HashMap's and is thread-safe.
        *counters.entry(endpoint).or_insert(0) += 1;
    });

    println!("Request counts per endpoint:");
    for entry in counters.iter() {
        println!("- {}: {}", entry.key(), entry.value());
    }
}
```

## Summary

This chapter covered essential HashMap and HashSet patterns: 

1.  **Entry API**: Efficient single-lookup operations (or_insert, and_modify)
2.  **Custom Hash**: Case-insensitive keys, composite keys, spatial hashing, floating-point keys
3.  **Capacity Optimization**: Pre-allocation, load factor management, memory efficiency
4.  **Alternative Maps**: BTreeMap for ordering, FxHashMap for speed
5.  **Concurrent Maps**: DashMap for high-performance multi-threaded access

**Key Takeaways**:
-   Use Entry API to avoid double lookups
-   Pre-allocate capacity when size is known
-   Choose the right map type for your use case
-   FxHashMap is 2-3x faster for integer keys
-   DashMap is essential for high-concurrency scenarios
-   BTreeMap when you need ordering or range queries

**Performance Tips**:
-   Reserve capacity before batch insertions
-   Use compact key types to reduce memory
-   Consider string interning for repeated strings
-   Profile before choosing hash function
-   DashMap for >4 concurrent writers