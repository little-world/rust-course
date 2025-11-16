# HashMap & HashSet Patterns

This chapter explores advanced patterns and techniques for working with hash-based collections in Rust. We'll cover the Entry API, custom hash functions, performance optimization, alternative map implementations, and concurrent access patterns through practical, real-world examples.

## Table of Contents

1. [Entry API Patterns](#entry-api-patterns)
2. [Custom Hash Functions](#custom-hash-functions)
3. [Capacity and Load Factor Optimization](#capacity-and-load-factor-optimization)
4. [Alternative Maps](#alternative-maps)
5. [Concurrent Maps](#concurrent-maps)

---

## Entry API Patterns

The Entry API provides efficient ways to work with HashMap entries, avoiding multiple lookups and enabling complex update patterns.

### Recipe 1: LRU Cache with Entry API

**Problem**: Implement a Least Recently Used (LRU) cache that efficiently tracks access order and evicts least recently used items.

**Solution**:

```rust
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

struct LruCache<K, V> {
    capacity: usize,
    map: HashMap<K, V>,
    order: VecDeque<K>,
}

impl<K, V> LruCache<K, V>
where
    K: Eq + Hash + Clone,
{
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            map: HashMap::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
        }
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        if self.map.contains_key(key) {
            // Move to front (most recently used)
            self.order.retain(|k| k != key);
            self.order.push_back(key.clone());
            self.map.get(key)
        } else {
            None
        }
    }

    fn put(&mut self, key: K, value: V) {
        use std::collections::hash_map::Entry;

        match self.map.entry(key.clone()) {
            Entry::Occupied(mut e) => {
                // Update existing entry
                e.insert(value);
                // Move to front
                self.order.retain(|k| k != &key);
                self.order.push_back(key);
            }
            Entry::Vacant(e) => {
                // Check capacity
                if self.map.len() >= self.capacity {
                    // Evict least recently used
                    if let Some(lru_key) = self.order.pop_front() {
                        self.map.remove(&lru_key);
                    }
                }
                e.insert(value);
                self.order.push_back(key);
            }
        }
    }

    fn len(&self) -> usize {
        self.map.len()
    }
}

// Example usage
fn main() {
    let mut cache = LruCache::new(3);

    cache.put("user:1".to_string(), "Alice");
    cache.put("user:2".to_string(), "Bob");
    cache.put("user:3".to_string(), "Charlie");

    assert_eq!(cache.get(&"user:1".to_string()), Some(&"Alice"));

    // This will evict "user:2" (least recently used)
    cache.put("user:4".to_string(), "David");

    assert_eq!(cache.get(&"user:2".to_string()), None);
    assert_eq!(cache.len(), 3);
}
```

**Why This Works**:
- Entry API avoids double lookup (check + insert)
- `Entry::Occupied` handles updates efficiently
- `Entry::Vacant` handles insertions with capacity checks
- Time complexity: O(1) average for get/put

---

### Recipe 2: Word Frequency Counter

**Problem**: Count word frequencies in a large text corpus efficiently, handling case-insensitive matching and Unicode.

**Solution**:

```rust
use std::collections::HashMap;

struct WordFrequency {
    counts: HashMap<String, usize>,
    total_words: usize,
}

impl WordFrequency {
    fn new() -> Self {
        Self {
            counts: HashMap::new(),
            total_words: 0,
        }
    }

    fn add_text(&mut self, text: &str) {
        for word in text.split_whitespace() {
            let word = word
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();

            if !word.is_empty() {
                // Entry API: increment or insert
                *self.counts.entry(word).or_insert(0) += 1;
                self.total_words += 1;
            }
        }
    }

    fn add_text_batch(&mut self, text: &str) {
        // More efficient: collect words first, then update
        let mut word_counts = HashMap::new();

        for word in text.split_whitespace() {
            let word = word
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();

            if !word.is_empty() {
                *word_counts.entry(word).or_insert(0) += 1;
            }
        }

        // Merge into main counts
        for (word, count) in word_counts {
            *self.counts.entry(word).or_insert(0) += count;
            self.total_words += count;
        }
    }

    fn frequency(&self, word: &str) -> f64 {
        let word = word.to_lowercase();
        let count = self.counts.get(&word).copied().unwrap_or(0);
        count as f64 / self.total_words as f64
    }

    fn top_words(&self, n: usize) -> Vec<(&str, usize)> {
        let mut words: Vec<_> = self.counts
            .iter()
            .map(|(w, c)| (w.as_str(), *c))
            .collect();

        words.sort_by(|a, b| b.1.cmp(&a.1));
        words.truncate(n);
        words
    }

    fn merge(&mut self, other: WordFrequency) {
        for (word, count) in other.counts {
            *self.counts.entry(word).or_insert(0) += count;
        }
        self.total_words += other.total_words;
    }
}

// Example usage
fn main() {
    let mut freq = WordFrequency::new();

    freq.add_text("The quick brown fox jumps over the lazy dog");
    freq.add_text("The dog was really lazy");

    println!("Top 5 words:");
    for (word, count) in freq.top_words(5) {
        println!("{}: {} ({:.2}%)", word, count, freq.frequency(word) * 100.0);
    }
}
```

**Key Patterns**:
- `or_insert()`: Provides default value for new entries
- `or_insert_with()`: Lazy initialization with closure
- `and_modify()`: Chain modification of existing entries
- Batch processing reduces HashMap resizing

---

### Recipe 3: Graph Adjacency List Construction

**Problem**: Build an adjacency list representation of a graph efficiently, supporting both directed and undirected edges.

**Solution**:

```rust
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EdgeType {
    Directed,
    Undirected,
}

struct Graph<T> {
    adjacency: HashMap<T, HashSet<T>>,
    edge_type: EdgeType,
}

impl<T> Graph<T>
where
    T: Eq + Hash + Clone,
{
    fn new(edge_type: EdgeType) -> Self {
        Self {
            adjacency: HashMap::new(),
            edge_type,
        }
    }

    fn add_edge(&mut self, from: T, to: T) {
        // Use entry API to avoid double lookup
        self.adjacency
            .entry(from.clone())
            .or_insert_with(HashSet::new)
            .insert(to.clone());

        if self.edge_type == EdgeType::Undirected {
            self.adjacency
                .entry(to)
                .or_insert_with(HashSet::new)
                .insert(from);
        } else {
            // Ensure 'to' node exists even if it has no outgoing edges
            self.adjacency.entry(to).or_insert_with(HashSet::new);
        }
    }

    fn add_edges(&mut self, edges: Vec<(T, T)>) {
        for (from, to) in edges {
            self.add_edge(from, to);
        }
    }

    fn neighbors(&self, node: &T) -> Option<&HashSet<T>> {
        self.adjacency.get(node)
    }

    fn degree(&self, node: &T) -> usize {
        self.adjacency
            .get(node)
            .map(|neighbors| neighbors.len())
            .unwrap_or(0)
    }

    fn remove_edge(&mut self, from: &T, to: &T) -> bool {
        let removed = self.adjacency
            .get_mut(from)
            .map(|neighbors| neighbors.remove(to))
            .unwrap_or(false);

        if self.edge_type == EdgeType::Undirected && removed {
            self.adjacency
                .get_mut(to)
                .map(|neighbors| neighbors.remove(from));
        }

        removed
    }

    fn vertices(&self) -> Vec<&T> {
        self.adjacency.keys().collect()
    }

    fn edge_count(&self) -> usize {
        let total: usize = self.adjacency.values().map(|s| s.len()).sum();

        match self.edge_type {
            EdgeType::Directed => total,
            EdgeType::Undirected => total / 2,
        }
    }
}

// Example: Social network
fn main() {
    let mut network = Graph::new(EdgeType::Undirected);

    network.add_edges(vec![
        ("Alice", "Bob"),
        ("Alice", "Charlie"),
        ("Bob", "David"),
        ("Charlie", "David"),
        ("Charlie", "Eve"),
    ]);

    println!("Alice's friends: {:?}", network.neighbors(&"Alice"));
    println!("David's degree: {}", network.degree(&"David"));
    println!("Total edges: {}", network.edge_count());
}
```

**Entry API Benefits**:
- `or_insert_with(HashSet::new)`: Lazy initialization of empty sets
- Avoids checking if key exists before inserting
- Cleaner code compared to manual if-let patterns

---

### Recipe 4: Grouping and Aggregation

**Problem**: Group items by a key and perform aggregations (count, sum, average) efficiently.

**Solution**:

```rust
use std::collections::HashMap;
use std::hash::Hash;

struct GroupBy<K, V> {
    groups: HashMap<K, Vec<V>>,
}

impl<K, V> GroupBy<K, V>
where
    K: Eq + Hash,
{
    fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    fn add(&mut self, key: K, value: V) {
        self.groups.entry(key).or_insert_with(Vec::new).push(value);
    }

    fn add_all<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = (K, V)>,
    {
        for (key, value) in items {
            self.add(key, value);
        }
    }

    fn get(&self, key: &K) -> Option<&Vec<V>> {
        self.groups.get(key)
    }

    fn keys(&self) -> impl Iterator<Item = &K> {
        self.groups.keys()
    }

    fn into_map(self) -> HashMap<K, Vec<V>> {
        self.groups
    }
}

// Specialized grouping operations
struct Aggregator;

impl Aggregator {
    fn count_by<T, K, F>(items: Vec<T>, key_fn: F) -> HashMap<K, usize>
    where
        K: Eq + Hash,
        F: Fn(&T) -> K,
    {
        let mut counts = HashMap::new();
        for item in items {
            *counts.entry(key_fn(&item)).or_insert(0) += 1;
        }
        counts
    }

    fn sum_by<T, K, F, V>(items: Vec<T>, key_fn: F, value_fn: fn(&T) -> V) -> HashMap<K, V>
    where
        K: Eq + Hash,
        F: Fn(&T) -> K,
        V: Default + std::ops::AddAssign,
    {
        let mut sums = HashMap::new();
        for item in items {
            let entry = sums.entry(key_fn(&item)).or_insert_with(Default::default);
            *entry += value_fn(&item);
        }
        sums
    }

    fn avg_by<T, K, F>(items: Vec<T>, key_fn: F, value_fn: fn(&T) -> f64) -> HashMap<K, f64>
    where
        K: Eq + Hash + Clone,
        F: Fn(&T) -> K,
    {
        let mut sums: HashMap<K, f64> = HashMap::new();
        let mut counts: HashMap<K, usize> = HashMap::new();

        for item in items {
            let key = key_fn(&item);
            *sums.entry(key.clone()).or_insert(0.0) += value_fn(&item);
            *counts.entry(key).or_insert(0) += 1;
        }

        sums.into_iter()
            .map(|(k, sum)| {
                let count = counts[&k];
                (k, sum / count as f64)
            })
            .collect()
    }
}

// Example: Sales data analysis
#[derive(Debug, Clone)]
struct Sale {
    product: String,
    category: String,
    amount: f64,
    quantity: usize,
}

fn main() {
    let sales = vec![
        Sale {
            product: "Laptop".to_string(),
            category: "Electronics".to_string(),
            amount: 1200.0,
            quantity: 1,
        },
        Sale {
            product: "Mouse".to_string(),
            category: "Electronics".to_string(),
            amount: 25.0,
            quantity: 3,
        },
        Sale {
            product: "Desk".to_string(),
            category: "Furniture".to_string(),
            amount: 450.0,
            quantity: 1,
        },
        Sale {
            product: "Chair".to_string(),
            category: "Furniture".to_string(),
            amount: 200.0,
            quantity: 2,
        },
    ];

    // Count by category
    let counts = Aggregator::count_by(sales.clone(), |s| s.category.clone());
    println!("Sales count by category: {:?}", counts);

    // Sum by category
    let totals = Aggregator::sum_by(
        sales.clone(),
        |s| s.category.clone(),
        |s| s.amount,
    );
    println!("Total revenue by category: {:?}", totals);

    // Average by category
    let averages = Aggregator::avg_by(
        sales.clone(),
        |s| s.category.clone(),
        |s| s.amount,
    );
    println!("Average sale by category: {:?}", averages);

    // Group all sales by category
    let mut groups = GroupBy::new();
    groups.add_all(sales.into_iter().map(|s| (s.category.clone(), s)));

    for category in groups.keys() {
        let items = groups.get(category).unwrap();
        println!("{}: {} items", category, items.len());
    }
}
```

**Entry API Patterns Demonstrated**:
- `or_insert(0)`: Initializing counters
- `or_insert_with(Vec::new)`: Lazy collection creation
- `or_insert_with(Default::default)`: Generic initialization
- Chaining entry access with modifications

---

## Custom Hash Functions

Custom hash functions allow you to use complex types as HashMap keys and optimize hashing for specific use cases.

### Recipe 5: Case-Insensitive String Keys

**Problem**: Create a HashMap where string keys are case-insensitive ("Hello" and "hello" are the same key).

**Solution**:

```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq)]
struct CaseInsensitiveString(String);

impl CaseInsensitiveString {
    fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl Hash for CaseInsensitiveString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the lowercase version
        for byte in self.0.bytes().map(|b| b.to_ascii_lowercase()) {
            byte.hash(state);
        }
    }
}

impl PartialEq for CaseInsensitiveString {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

// Wrapper type for easier usage
struct CaseInsensitiveMap<V> {
    map: HashMap<CaseInsensitiveString, V>,
}

impl<V> CaseInsensitiveMap<V> {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn insert(&mut self, key: impl Into<String>, value: V) -> Option<V> {
        self.map.insert(CaseInsensitiveString::new(key), value)
    }

    fn get(&self, key: &str) -> Option<&V> {
        self.map.get(&CaseInsensitiveString::new(key))
    }

    fn contains_key(&self, key: &str) -> bool {
        self.map.contains_key(&CaseInsensitiveString::new(key))
    }

    fn remove(&mut self, key: &str) -> Option<V> {
        self.map.remove(&CaseInsensitiveString::new(key))
    }
}

// Example: HTTP headers
fn main() {
    let mut headers = CaseInsensitiveMap::new();

    headers.insert("Content-Type", "application/json");
    headers.insert("content-length", "1234");
    headers.insert("AUTHORIZATION", "Bearer token123");

    // All these work regardless of case
    assert_eq!(headers.get("content-type"), Some(&"application/json"));
    assert_eq!(headers.get("Content-Length"), Some(&"1234"));
    assert_eq!(headers.get("authorization"), Some(&"Bearer token123"));

    assert!(headers.contains_key("CONTENT-TYPE"));
}
```

**Key Points**:
- Implement both `Hash` and `Eq` consistently
- Hash and equality must agree: `a == b` implies `hash(a) == hash(b)`
- Case-insensitive hashing requires transforming data during hashing

---

### Recipe 6: Composite Keys and Custom Types

**Problem**: Use complex composite keys (e.g., (user_id, timestamp, event_type)) efficiently in a HashMap.

**Solution**:

```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// Composite key for event tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EventKey {
    user_id: u64,
    timestamp: u64,
    event_type: EventType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum EventType {
    PageView,
    Click,
    Purchase,
    SignUp,
}

#[derive(Debug, Clone)]
struct Event {
    data: String,
    metadata: HashMap<String, String>,
}

struct EventStore {
    events: HashMap<EventKey, Event>,
}

impl EventStore {
    fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    fn record(&mut self, key: EventKey, event: Event) {
        self.events.insert(key, event);
    }

    fn get(&self, key: &EventKey) -> Option<&Event> {
        self.events.get(key)
    }

    fn get_user_events(&self, user_id: u64) -> Vec<(&EventKey, &Event)> {
        self.events
            .iter()
            .filter(|(k, _)| k.user_id == user_id)
            .collect()
    }

    fn get_events_by_type(&self, event_type: EventType) -> Vec<(&EventKey, &Event)> {
        self.events
            .iter()
            .filter(|(k, _)| k.event_type == event_type)
            .collect()
    }
}

// Optimized key for spatial indexing
#[derive(Debug, Clone, Copy)]
struct GridKey {
    x: i32,
    y: i32,
}

impl GridKey {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    // Spatial hash: interleave bits of x and y (Z-order curve)
    fn spatial_hash(&self) -> u64 {
        fn interleave(mut x: u32, mut y: u32) -> u64 {
            let mut result = 0u64;
            for i in 0..32 {
                result |= ((x & 1) as u64) << (2 * i);
                result |= ((y & 1) as u64) << (2 * i + 1);
                x >>= 1;
                y >>= 1;
            }
            result
        }

        interleave(self.x as u32, self.y as u32)
    }
}

impl PartialEq for GridKey {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for GridKey {}

impl Hash for GridKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Use spatial hash for better locality
        self.spatial_hash().hash(state);
    }
}

// Spatial grid for game entities
struct SpatialGrid<T> {
    grid: HashMap<GridKey, Vec<T>>,
    cell_size: i32,
}

impl<T> SpatialGrid<T> {
    fn new(cell_size: i32) -> Self {
        Self {
            grid: HashMap::new(),
            cell_size,
        }
    }

    fn insert(&mut self, x: i32, y: i32, entity: T) {
        let key = GridKey::new(x / self.cell_size, y / self.cell_size);
        self.grid.entry(key).or_insert_with(Vec::new).push(entity);
    }

    fn get_cell(&self, x: i32, y: i32) -> Option<&Vec<T>> {
        let key = GridKey::new(x / self.cell_size, y / self.cell_size);
        self.grid.get(&key)
    }

    fn get_nearby(&self, x: i32, y: i32, radius: i32) -> Vec<&T> {
        let cell_x = x / self.cell_size;
        let cell_y = y / self.cell_size;
        let cell_radius = (radius / self.cell_size) + 1;

        let mut result = Vec::new();
        for dy in -cell_radius..=cell_radius {
            for dx in -cell_radius..=cell_radius {
                let key = GridKey::new(cell_x + dx, cell_y + dy);
                if let Some(entities) = self.grid.get(&key) {
                    result.extend(entities);
                }
            }
        }
        result
    }
}

// Example usage
fn main() {
    // Event tracking
    let mut store = EventStore::new();

    let key = EventKey {
        user_id: 12345,
        timestamp: 1699564800,
        event_type: EventType::Purchase,
    };

    let event = Event {
        data: "product_id=789".to_string(),
        metadata: HashMap::from([
            ("amount".to_string(), "99.99".to_string()),
            ("currency".to_string(), "USD".to_string()),
        ]),
    };

    store.record(key, event);

    // Spatial grid
    let mut grid = SpatialGrid::new(100);
    grid.insert(150, 250, "Enemy1");
    grid.insert(180, 270, "Enemy2");
    grid.insert(500, 600, "Enemy3");

    let nearby = grid.get_nearby(150, 250, 150);
    println!("Nearby entities: {:?}", nearby);
}
```

**Custom Hash Benefits**:
- Spatial hashing improves cache locality for nearby coordinates
- Composite keys avoid creating wrapper types
- Domain-specific hashing can improve performance

---

### Recipe 7: Hashing Floating-Point Numbers

**Problem**: Use floating-point coordinates as HashMap keys (floats don't implement `Hash` due to NaN issues).

**Solution**:

```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// Wrapper for f64 that implements Hash
#[derive(Debug, Clone, Copy)]
struct OrderedFloat(f64);

impl OrderedFloat {
    fn new(value: f64) -> Self {
        if value.is_nan() {
            panic!("NaN is not allowed in OrderedFloat");
        }
        Self(value)
    }

    fn get(&self) -> f64 {
        self.0
    }
}

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for OrderedFloat {}

impl Hash for OrderedFloat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

// 2D point with floating coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Point {
    x: OrderedFloat,
    y: OrderedFloat,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Self {
            x: OrderedFloat::new(x),
            y: OrderedFloat::new(y),
        }
    }

    fn distance(&self, other: &Point) -> f64 {
        let dx = self.x.get() - other.x.get();
        let dy = self.y.get() - other.y.get();
        (dx * dx + dy * dy).sqrt()
    }
}

// Quantized point (rounds to grid for approximate matching)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct QuantizedPoint {
    x: i32,
    y: i32,
}

impl QuantizedPoint {
    fn from_float(x: f64, y: f64, precision: f64) -> Self {
        Self {
            x: (x / precision).round() as i32,
            y: (y / precision).round() as i32,
        }
    }

    fn to_float(&self, precision: f64) -> (f64, f64) {
        (self.x as f64 * precision, self.y as f64 * precision)
    }
}

// Approximate point lookup with tolerance
struct ApproximatePointMap<V> {
    map: HashMap<QuantizedPoint, Vec<(Point, V)>>,
    precision: f64,
}

impl<V> ApproximatePointMap<V> {
    fn new(precision: f64) -> Self {
        Self {
            map: HashMap::new(),
            precision,
        }
    }

    fn insert(&mut self, point: Point, value: V) {
        let key = QuantizedPoint::from_float(
            point.x.get(),
            point.y.get(),
            self.precision,
        );
        self.map
            .entry(key)
            .or_insert_with(Vec::new)
            .push((point, value));
    }

    fn get_exact(&self, point: &Point) -> Option<&V> {
        let key = QuantizedPoint::from_float(
            point.x.get(),
            point.y.get(),
            self.precision,
        );

        self.map
            .get(&key)
            .and_then(|items| {
                items
                    .iter()
                    .find(|(p, _)| p == point)
                    .map(|(_, v)| v)
            })
    }

    fn get_nearby(&self, point: &Point, tolerance: f64) -> Vec<(&Point, &V)> {
        let key = QuantizedPoint::from_float(
            point.x.get(),
            point.y.get(),
            self.precision,
        );

        let mut results = Vec::new();

        // Check the cell and neighboring cells
        for dy in -1..=1 {
            for dx in -1..=1 {
                let neighbor_key = QuantizedPoint {
                    x: key.x + dx,
                    y: key.y + dy,
                };

                if let Some(items) = self.map.get(&neighbor_key) {
                    for (p, v) in items {
                        if point.distance(p) <= tolerance {
                            results.push((p, v));
                        }
                    }
                }
            }
        }

        results
    }
}

// Example: GPS coordinate caching
fn main() {
    // Exact floating-point keys
    let mut exact_map: HashMap<Point, String> = HashMap::new();

    let p1 = Point::new(37.7749, -122.4194); // San Francisco
    let p2 = Point::new(40.7128, -74.0060);  // New York

    exact_map.insert(p1, "San Francisco".to_string());
    exact_map.insert(p2, "New York".to_string());

    assert_eq!(exact_map.get(&p1), Some(&"San Francisco".to_string()));

    // Approximate lookup (useful for GPS with measurement error)
    let mut approx_map = ApproximatePointMap::new(0.01); // ~1km precision

    approx_map.insert(p1, "San Francisco");
    approx_map.insert(p2, "New York");

    // Find points within 50km
    let nearby = approx_map.get_nearby(&Point::new(37.78, -122.42), 0.5);
    println!("Nearby cities: {:?}", nearby);
}
```

**Floating-Point Hash Strategies**:
1. **OrderedFloat**: Hash raw bits (`to_bits()`) for exact matching
2. **Quantization**: Round to grid for approximate matching
3. **Spatial indexing**: Group nearby points in same bucket

---

## Capacity and Load Factor Optimization

Understanding and optimizing HashMap capacity and load factor can significantly improve performance.

### Recipe 8: Pre-Allocated Collections for Batch Processing

**Problem**: Process large datasets efficiently by pre-allocating HashMap capacity to avoid resizing.

**Solution**:

```rust
use std::collections::HashMap;
use std::time::Instant;

struct BatchProcessor<K, V> {
    map: HashMap<K, V>,
}

impl<K, V> BatchProcessor<K, V>
where
    K: Eq + std::hash::Hash,
{
    // Pre-allocate for known size
    fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
        }
    }

    // Estimate capacity based on load factor
    fn with_expected_size(expected_size: usize) -> Self {
        // Default load factor is ~0.875
        // Allocate extra to avoid resizing
        let capacity = (expected_size as f64 / 0.75).ceil() as usize;
        Self::with_capacity(capacity)
    }

    fn process_batch(&mut self, items: Vec<(K, V)>) {
        // Reserve additional space if needed
        self.map.reserve(items.len());

        for (k, v) in items {
            self.map.insert(k, v);
        }
    }

    fn capacity(&self) -> usize {
        self.map.capacity()
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    fn load_factor(&self) -> f64 {
        self.len() as f64 / self.capacity() as f64
    }
}

// Benchmark different allocation strategies
fn benchmark_allocation_strategy() {
    const SIZE: usize = 1_000_000;

    // Strategy 1: No pre-allocation (worst)
    let start = Instant::now();
    let mut map1 = HashMap::new();
    for i in 0..SIZE {
        map1.insert(i, i * 2);
    }
    let duration1 = start.elapsed();
    println!("No pre-allocation: {:?}", duration1);
    println!("Final capacity: {}", map1.capacity());

    // Strategy 2: Exact pre-allocation
    let start = Instant::now();
    let mut map2 = HashMap::with_capacity(SIZE);
    for i in 0..SIZE {
        map2.insert(i, i * 2);
    }
    let duration2 = start.elapsed();
    println!("Exact pre-allocation: {:?}", duration2);
    println!("Final capacity: {}", map2.capacity());

    // Strategy 3: Over-allocation (best for future growth)
    let start = Instant::now();
    let mut map3 = HashMap::with_capacity((SIZE as f64 / 0.75) as usize);
    for i in 0..SIZE {
        map3.insert(i, i * 2);
    }
    let duration3 = start.elapsed();
    println!("Over-allocation: {:?}", duration3);
    println!("Final capacity: {}", map3.capacity());
    println!("Load factor: {:.2}", SIZE as f64 / map3.capacity() as f64);
}

// Real-world example: Log aggregation
#[derive(Debug, Clone)]
struct LogEntry {
    timestamp: u64,
    level: String,
    message: String,
}

struct LogAggregator {
    entries_by_level: HashMap<String, Vec<LogEntry>>,
    entries_by_hour: HashMap<u64, Vec<LogEntry>>,
}

impl LogAggregator {
    fn new(estimated_logs: usize, estimated_levels: usize) -> Self {
        Self {
            entries_by_level: HashMap::with_capacity(estimated_levels),
            entries_by_hour: HashMap::with_capacity(24), // 24 hours max
        }
    }

    fn process_logs(&mut self, logs: Vec<LogEntry>) {
        // Pre-allocate for estimated unique hours
        let estimated_hours = logs.len() / 1000; // Assume ~1000 logs per hour
        self.entries_by_hour.reserve(estimated_hours);

        for log in logs {
            // Group by level
            self.entries_by_level
                .entry(log.level.clone())
                .or_insert_with(|| Vec::with_capacity(100))
                .push(log.clone());

            // Group by hour
            let hour = log.timestamp / 3600;
            self.entries_by_hour
                .entry(hour)
                .or_insert_with(|| Vec::with_capacity(1000))
                .push(log);
        }
    }

    fn get_stats(&self) {
        println!("Logs by level:");
        for (level, entries) in &self.entries_by_level {
            println!("  {}: {}", level, entries.len());
        }

        println!("\nLoad factors:");
        println!("  By level: {:.2}",
                 self.entries_by_level.len() as f64 / self.entries_by_level.capacity() as f64);
        println!("  By hour: {:.2}",
                 self.entries_by_hour.len() as f64 / self.entries_by_hour.capacity() as f64);
    }
}

fn main() {
    println!("=== Allocation Strategy Benchmark ===\n");
    benchmark_allocation_strategy();

    println!("\n=== Log Aggregation Example ===\n");

    let logs: Vec<LogEntry> = (0..10000)
        .map(|i| LogEntry {
            timestamp: 1699564800 + (i * 10),
            level: match i % 4 {
                0 => "INFO",
                1 => "WARN",
                2 => "ERROR",
                _ => "DEBUG",
            }.to_string(),
            message: format!("Log message {}", i),
        })
        .collect();

    let mut aggregator = LogAggregator::new(10000, 4);
    aggregator.process_logs(logs);
    aggregator.get_stats();
}
```

**Performance Guidelines**:
- **Pre-allocate when size is known**: Use `with_capacity()` to avoid resizing
- **Reserve for batches**: Use `reserve()` before inserting batches
- **Over-allocate slightly**: Allocate `size / 0.75` to maintain low load factor
- **Monitor load factor**: Keep it below 0.75 for optimal performance

---

### Recipe 9: Memory-Efficient HashMap Configuration

**Problem**: Minimize memory usage for HashMaps while maintaining acceptable performance.

**Solution**:

```rust
use std::collections::HashMap;
use std::mem::size_of_val;

// Compact key types reduce memory overhead
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CompactKey {
    id: u32,      // Use smaller type (4 bytes vs 8 bytes for u64)
    category: u8,  // Use u8 instead of enum/string
}

// Interned strings to avoid duplication
struct StringInterner {
    map: HashMap<String, u32>,
    strings: Vec<String>,
}

impl StringInterner {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            strings: Vec::new(),
        }
    }

    fn intern(&mut self, s: &str) -> u32 {
        if let Some(&id) = self.map.get(s) {
            id
        } else {
            let id = self.strings.len() as u32;
            self.strings.push(s.to_string());
            self.map.insert(s.to_string(), id);
            id
        }
    }

    fn get(&self, id: u32) -> Option<&str> {
        self.strings.get(id as usize).map(|s| s.as_str())
    }

    fn memory_usage(&self) -> usize {
        let map_size = self.map.capacity() * (size_of_val(&"") + size_of_val(&0u32));
        let vec_size = self.strings.iter().map(|s| s.capacity()).sum::<usize>();
        map_size + vec_size
    }
}

// Comparison: memory usage with and without interning
fn compare_memory_usage() {
    const ENTRIES: usize = 100_000;
    const UNIQUE_STRINGS: usize = 1_000;

    // Without interning: store full strings
    let mut without_interning: HashMap<u32, String> = HashMap::with_capacity(ENTRIES);
    let strings: Vec<String> = (0..UNIQUE_STRINGS)
        .map(|i| format!("string_{}", i))
        .collect();

    for i in 0..ENTRIES {
        without_interning.insert(
            i as u32,
            strings[i % UNIQUE_STRINGS].clone(),
        );
    }

    // With interning: store string IDs
    let mut interner = StringInterner::new();
    let mut with_interning: HashMap<u32, u32> = HashMap::with_capacity(ENTRIES);

    for i in 0..ENTRIES {
        let string_id = interner.intern(&strings[i % UNIQUE_STRINGS]);
        with_interning.insert(i as u32, string_id);
    }

    println!("Memory comparison (approximate):");
    println!("Without interning: ~{} bytes",
             without_interning.capacity() * (size_of_val(&0u32) + 24)); // String is ~24 bytes
    println!("With interning: ~{} bytes",
             with_interning.capacity() * (size_of_val(&0u32) * 2) + interner.memory_usage());
}

// Shrink map after deletions
struct SelfOptimizingMap<K, V> {
    map: HashMap<K, V>,
    deletions: usize,
    shrink_threshold: usize,
}

impl<K, V> SelfOptimizingMap<K, V>
where
    K: Eq + std::hash::Hash,
{
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            deletions: 0,
            shrink_threshold: 1000,
        }
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.map.insert(key, value)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        let result = self.map.remove(key);

        if result.is_some() {
            self.deletions += 1;

            // Shrink if we've deleted many items
            if self.deletions >= self.shrink_threshold {
                self.shrink();
            }
        }

        result
    }

    fn shrink(&mut self) {
        // Shrink to fit current size
        self.map.shrink_to_fit();
        self.deletions = 0;
        println!("Shrunk map to capacity: {}", self.map.capacity());
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    fn capacity(&self) -> usize {
        self.map.capacity()
    }
}

fn main() {
    compare_memory_usage();

    println!("\n=== Self-Optimizing Map ===\n");

    let mut map = SelfOptimizingMap::new();

    // Insert many items
    for i in 0..10000 {
        map.insert(i, i * 2);
    }
    println!("After insertions - len: {}, capacity: {}", map.len(), map.capacity());

    // Delete most items
    for i in 0..9000 {
        map.remove(&i);
    }
    println!("After deletions - len: {}, capacity: {}", map.len(), map.capacity());

    // String interning example
    println!("\n=== String Interning ===\n");
    let mut interner = StringInterner::new();

    let id1 = interner.intern("hello");
    let id2 = interner.intern("world");
    let id3 = interner.intern("hello"); // Reuses existing

    assert_eq!(id1, id3);
    assert_eq!(interner.get(id1), Some("hello"));
}
```

**Memory Optimization Techniques**:
1. **Use compact types**: u32 instead of u64, u8 instead of String
2. **String interning**: Store unique strings once, use IDs
3. **Shrink after deletions**: Use `shrink_to_fit()` to reclaim memory
4. **Avoid over-allocation**: Don't pre-allocate more than needed

---

## Alternative Maps

Different map implementations offer trade-offs between performance, ordering, and features.

### Recipe 10: BTreeMap for Ordered Operations

**Problem**: Need to maintain sorted keys and perform range queries efficiently.

**Solution**:

```rust
use std::collections::BTreeMap;
use std::ops::Bound::{Included, Excluded, Unbounded};

// Time-series data with range queries
struct TimeSeries<V> {
    data: BTreeMap<u64, V>, // timestamp -> value
}

impl<V: Clone> TimeSeries<V> {
    fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    fn insert(&mut self, timestamp: u64, value: V) {
        self.data.insert(timestamp, value);
    }

    // Get all values in time range
    fn range(&self, start: u64, end: u64) -> Vec<(u64, V)> {
        self.data
            .range(start..=end)
            .map(|(&k, v)| (k, v.clone()))
            .collect()
    }

    // Get latest value before timestamp
    fn get_latest_before(&self, timestamp: u64) -> Option<(u64, V)> {
        self.data
            .range(..timestamp)
            .next_back()
            .map(|(&k, v)| (k, v.clone()))
    }

    // Get earliest value after timestamp
    fn get_earliest_after(&self, timestamp: u64) -> Option<(u64, V)> {
        self.data
            .range(timestamp..)
            .next()
            .map(|(&k, v)| (k, v.clone()))
    }

    // Get first and last
    fn first(&self) -> Option<(u64, V)> {
        self.data.first_key_value().map(|(&k, v)| (k, v.clone()))
    }

    fn last(&self) -> Option<(u64, V)> {
        self.data.last_key_value().map(|(&k, v)| (k, v.clone()))
    }

    // Remove old data (data retention)
    fn remove_before(&mut self, timestamp: u64) -> usize {
        let keys_to_remove: Vec<u64> = self.data
            .range(..timestamp)
            .map(|(&k, _)| k)
            .collect();

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            self.data.remove(&key);
        }
        count
    }

    // Downsample: get one value per interval
    fn downsample(&self, interval: u64) -> Vec<(u64, V)> {
        let mut result = Vec::new();
        let mut current_bucket = None;

        for (&timestamp, value) in &self.data {
            let bucket = timestamp / interval;

            if current_bucket != Some(bucket) {
                result.push((timestamp, value.clone()));
                current_bucket = Some(bucket);
            }
        }

        result
    }
}

// Leaderboard with ranking
struct Leaderboard {
    scores: BTreeMap<u64, Vec<String>>, // score -> list of names
    name_to_score: std::collections::HashMap<String, u64>,
}

impl Leaderboard {
    fn new() -> Self {
        Self {
            scores: BTreeMap::new(),
            name_to_score: std::collections::HashMap::new(),
        }
    }

    fn update_score(&mut self, name: String, score: u64) {
        // Remove old score
        if let Some(&old_score) = self.name_to_score.get(&name) {
            if let Some(names) = self.scores.get_mut(&old_score) {
                names.retain(|n| n != &name);
                if names.is_empty() {
                    self.scores.remove(&old_score);
                }
            }
        }

        // Add new score
        self.scores
            .entry(score)
            .or_insert_with(Vec::new)
            .push(name.clone());
        self.name_to_score.insert(name, score);
    }

    fn top(&self, n: usize) -> Vec<(String, u64)> {
        let mut result = Vec::new();

        for (&score, names) in self.scores.iter().rev() {
            for name in names {
                if result.len() >= n {
                    return result;
                }
                result.push((name.clone(), score));
            }
        }

        result
    }

    fn rank(&self, name: &str) -> Option<usize> {
        let score = self.name_to_score.get(name)?;

        let mut rank = 1;
        for (&s, names) in self.scores.iter().rev() {
            if s == *score {
                // Find position within same score
                if let Some(pos) = names.iter().position(|n| n == name) {
                    return Some(rank + pos);
                }
            }
            rank += names.len();
        }

        None
    }

    fn get_score(&self, name: &str) -> Option<u64> {
        self.name_to_score.get(name).copied()
    }
}

// Example usage
fn main() {
    println!("=== Time Series ===\n");

    let mut temps = TimeSeries::new();

    // Insert temperature readings
    temps.insert(1699564800, 20.5); // 12:00
    temps.insert(1699568400, 22.1); // 13:00
    temps.insert(1699572000, 23.8); // 14:00
    temps.insert(1699575600, 24.2); // 15:00
    temps.insert(1699579200, 22.9); // 16:00

    // Range query
    let afternoon = temps.range(1699568400, 1699575600);
    println!("Afternoon temps: {:?}", afternoon);

    // Latest before 14:30
    let before_1430 = temps.get_latest_before(1699573800);
    println!("Latest before 14:30: {:?}", before_1430);

    // Downsample to hourly
    let hourly = temps.downsample(3600);
    println!("Hourly samples: {:?}", hourly);

    println!("\n=== Leaderboard ===\n");

    let mut leaderboard = Leaderboard::new();

    leaderboard.update_score("Alice".to_string(), 1500);
    leaderboard.update_score("Bob".to_string(), 2000);
    leaderboard.update_score("Charlie".to_string(), 1800);
    leaderboard.update_score("David".to_string(), 2200);

    println!("Top 3: {:?}", leaderboard.top(3));
    println!("Charlie's rank: {:?}", leaderboard.rank("Charlie"));

    // Update score
    leaderboard.update_score("Alice".to_string(), 2500);
    println!("After Alice's update - Top 3: {:?}", leaderboard.top(3));
    println!("Alice's new rank: {:?}", leaderboard.rank("Alice"));
}
```

**BTreeMap Use Cases**:
- Time-series data with range queries
- Leaderboards with ranking
- Ordered iteration
- Range-based operations
- Maintaining sorted order

**Performance**: O(log n) for insert/get/remove vs O(1) average for HashMap

---

### Recipe 11: FxHashMap for Performance

**Problem**: Need faster hashing for integer keys or when cryptographic security isn't required.

**Solution**:

```rust
// Note: Add `rustc-hash = "1.1"` to Cargo.toml
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::time::Instant;

// Benchmark HashMap vs FxHashMap
fn benchmark_hash_maps() {
    const SIZE: usize = 1_000_000;

    // Standard HashMap (SipHash - cryptographically secure)
    let start = Instant::now();
    let mut std_map: HashMap<u64, u64> = HashMap::with_capacity(SIZE);
    for i in 0..SIZE as u64 {
        std_map.insert(i, i * 2);
    }
    let std_insert = start.elapsed();

    let start = Instant::now();
    let mut sum = 0u64;
    for i in 0..SIZE as u64 {
        sum += std_map.get(&i).unwrap();
    }
    let std_get = start.elapsed();
    println!("Standard HashMap:");
    println!("  Insert: {:?}", std_insert);
    println!("  Get: {:?}", std_get);
    println!("  Sum: {}", sum);

    // FxHashMap (FxHash - fast non-cryptographic)
    let start = Instant::now();
    let mut fx_map: FxHashMap<u64, u64> = FxHashMap::with_capacity_and_hasher(
        SIZE,
        Default::default(),
    );
    for i in 0..SIZE as u64 {
        fx_map.insert(i, i * 2);
    }
    let fx_insert = start.elapsed();

    let start = Instant::now();
    let mut sum = 0u64;
    for i in 0..SIZE as u64 {
        sum += fx_map.get(&i).unwrap();
    }
    let fx_get = start.elapsed();
    println!("\nFxHashMap:");
    println!("  Insert: {:?}", fx_insert);
    println!("  Get: {:?}", fx_get);
    println!("  Sum: {}", sum);

    println!("\nSpeedup:");
    println!("  Insert: {:.2}x", std_insert.as_secs_f64() / fx_insert.as_secs_f64());
    println!("  Get: {:.2}x", std_get.as_secs_f64() / fx_get.as_secs_f64());
}

// Graph algorithms benefit from FxHashMap
use std::hash::Hash;

struct FastGraph<T> {
    adjacency: FxHashMap<T, Vec<T>>,
}

impl<T> FastGraph<T>
where
    T: Eq + Hash + Clone,
{
    fn new() -> Self {
        Self {
            adjacency: FxHashMap::default(),
        }
    }

    fn add_edge(&mut self, from: T, to: T) {
        self.adjacency
            .entry(from)
            .or_insert_with(Vec::new)
            .push(to);
    }

    fn bfs(&self, start: &T) -> Vec<T> {
        use std::collections::VecDeque;

        let mut visited = FxHashMap::default();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        queue.push_back(start.clone());
        visited.insert(start.clone(), true);

        while let Some(node) = queue.pop_front() {
            result.push(node.clone());

            if let Some(neighbors) = self.adjacency.get(&node) {
                for neighbor in neighbors {
                    if !visited.contains_key(neighbor) {
                        visited.insert(neighbor.clone(), true);
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        result
    }
}

// Compiler symbol table (common use case for FxHashMap)
struct SymbolTable {
    symbols: FxHashMap<String, SymbolInfo>,
    scopes: Vec<FxHashMap<String, SymbolInfo>>,
}

#[derive(Debug, Clone)]
struct SymbolInfo {
    ty: String,
    defined_at: usize,
}

impl SymbolTable {
    fn new() -> Self {
        Self {
            symbols: FxHashMap::default(),
            scopes: vec![FxHashMap::default()],
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(FxHashMap::default());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define(&mut self, name: String, info: SymbolInfo) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, info);
        }
    }

    fn lookup(&self, name: &str) -> Option<&SymbolInfo> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }
}

fn main() {
    benchmark_hash_maps();

    println!("\n=== Graph BFS ===\n");

    let mut graph = FastGraph::new();
    graph.add_edge(1, 2);
    graph.add_edge(1, 3);
    graph.add_edge(2, 4);
    graph.add_edge(3, 4);
    graph.add_edge(4, 5);

    let traversal = graph.bfs(&1);
    println!("BFS from 1: {:?}", traversal);

    println!("\n=== Symbol Table ===\n");

    let mut symbols = SymbolTable::new();

    symbols.define("x".to_string(), SymbolInfo {
        ty: "int".to_string(),
        defined_at: 1,
    });

    symbols.push_scope();
    symbols.define("y".to_string(), SymbolInfo {
        ty: "string".to_string(),
        defined_at: 5,
    });

    println!("Lookup x: {:?}", symbols.lookup("x"));
    println!("Lookup y: {:?}", symbols.lookup("y"));

    symbols.pop_scope();
    println!("After pop - Lookup y: {:?}", symbols.lookup("y"));
}
```

**When to Use FxHashMap**:
- Integer keys (u32, u64, etc.)
- Compiler/interpreter internal data structures
- Graph algorithms
- Game engines (entity IDs)
- When DoS attacks via hash collision aren't a concern
- 2-3x faster than standard HashMap for integer keys

**Don't Use For**:
- User-controlled string keys (DoS vulnerability)
- Cryptographic applications
- When security is a priority

---

## Concurrent Maps

For multi-threaded scenarios, specialized concurrent maps provide thread-safe access without manual locking.

### Recipe 12: DashMap for Concurrent Access

**Problem**: Multiple threads need to read and write to a shared map concurrently with minimal contention.

**Solution**:

```rust
// Note: Add `dashmap = "5.5"` to Cargo.toml
use dashmap::DashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Concurrent cache with automatic expiration
struct ConcurrentCache<K, V> {
    map: Arc<DashMap<K, (V, u64)>>, // value + expiration timestamp
}

impl<K, V> ConcurrentCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    fn new() -> Self {
        Self {
            map: Arc::new(DashMap::new()),
        }
    }

    fn insert(&self, key: K, value: V, ttl_seconds: u64) {
        let expiration = current_timestamp() + ttl_seconds;
        self.map.insert(key, (value, expiration));
    }

    fn get(&self, key: &K) -> Option<V> {
        let entry = self.map.get(key)?;
        let (value, expiration) = entry.value();

        if current_timestamp() > *expiration {
            drop(entry); // Release read lock
            self.map.remove(key);
            None
        } else {
            Some(value.clone())
        }
    }

    fn remove_expired(&self) -> usize {
        let now = current_timestamp();
        let mut removed = 0;

        self.map.retain(|_, (_, expiration)| {
            if now > *expiration {
                removed += 1;
                false
            } else {
                true
            }
        });

        removed
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    fn clear(&self) {
        self.map.clear();
    }

    fn clone_handle(&self) -> Self {
        Self {
            map: Arc::clone(&self.map),
        }
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// Multi-threaded request counter
struct RequestCounter {
    counts: Arc<DashMap<String, usize>>,
}

impl RequestCounter {
    fn new() -> Self {
        Self {
            counts: Arc::new(DashMap::new()),
        }
    }

    fn increment(&self, endpoint: &str) {
        self.counts
            .entry(endpoint.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    fn get(&self, endpoint: &str) -> usize {
        self.counts.get(endpoint).map(|r| *r).unwrap_or(0)
    }

    fn total(&self) -> usize {
        self.counts.iter().map(|r| *r.value()).sum()
    }

    fn top(&self, n: usize) -> Vec<(String, usize)> {
        let mut entries: Vec<_> = self.counts
            .iter()
            .map(|r| (r.key().clone(), *r.value()))
            .collect();

        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.truncate(n);
        entries
    }

    fn reset(&self) {
        self.counts.clear();
    }

    fn clone_handle(&self) -> Self {
        Self {
            counts: Arc::clone(&self.counts),
        }
    }
}

// Concurrent task queue with status tracking
#[derive(Debug, Clone, PartialEq)]
enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}

struct TaskTracker {
    tasks: Arc<DashMap<u64, TaskStatus>>,
    next_id: Arc<std::sync::atomic::AtomicU64>,
}

impl TaskTracker {
    fn new() -> Self {
        Self {
            tasks: Arc::new(DashMap::new()),
            next_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
        }
    }

    fn create_task(&self) -> u64 {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.tasks.insert(id, TaskStatus::Pending);
        id
    }

    fn update_status(&self, id: u64, status: TaskStatus) -> bool {
        self.tasks.insert(id, status).is_some()
    }

    fn get_status(&self, id: u64) -> Option<TaskStatus> {
        self.tasks.get(&id).map(|r| r.value().clone())
    }

    fn get_pending(&self) -> Vec<u64> {
        self.tasks
            .iter()
            .filter(|r| *r.value() == TaskStatus::Pending)
            .map(|r| *r.key())
            .collect()
    }

    fn count_by_status(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();

        for entry in self.tasks.iter() {
            let status_name = match entry.value() {
                TaskStatus::Pending => "pending",
                TaskStatus::Running => "running",
                TaskStatus::Completed => "completed",
                TaskStatus::Failed(_) => "failed",
            };
            *counts.entry(status_name.to_string()).or_insert(0) += 1;
        }

        counts
    }

    fn clone_handle(&self) -> Self {
        Self {
            tasks: Arc::clone(&self.tasks),
            next_id: Arc::clone(&self.next_id),
        }
    }
}

// Example usage
fn main() {
    println!("=== Concurrent Cache ===\n");

    let cache = ConcurrentCache::new();

    // Spawn multiple writer threads
    let mut handles = vec![];
    for i in 0..4 {
        let cache_clone = cache.clone_handle();
        handles.push(thread::spawn(move || {
            for j in 0..100 {
                cache_clone.insert(
                    format!("key_{}_{}", i, j),
                    format!("value_{}_{}", i, j),
                    5, // 5 second TTL
                );
            }
        }));
    }

    // Wait for writers
    for handle in handles {
        handle.join().unwrap();
    }

    println!("Cache size after inserts: {}", cache.len());

    // Simulate time passing
    thread::sleep(Duration::from_secs(6));
    let removed = cache.remove_expired();
    println!("Removed {} expired entries", removed);
    println!("Cache size after expiration: {}", cache.len());

    println!("\n=== Request Counter ===\n");

    let counter = RequestCounter::new();

    // Simulate concurrent requests
    let mut handles = vec![];
    for _ in 0..10 {
        let counter_clone = counter.clone_handle();
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                counter_clone.increment("/api/users");
                counter_clone.increment("/api/posts");
                counter_clone.increment("/api/comments");
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Total requests: {}", counter.total());
    println!("Top endpoints: {:?}", counter.top(3));

    println!("\n=== Task Tracker ===\n");

    let tracker = TaskTracker::new();

    // Create tasks
    let task_ids: Vec<u64> = (0..10).map(|_| tracker.create_task()).collect();
    println!("Created {} tasks", task_ids.len());

    // Spawn worker threads
    let mut handles = vec![];
    for _ in 0..3 {
        let tracker_clone = tracker.clone_handle();
        handles.push(thread::spawn(move || {
            loop {
                let pending = tracker_clone.get_pending();
                if pending.is_empty() {
                    break;
                }

                if let Some(&id) = pending.first() {
                    tracker_clone.update_status(id, TaskStatus::Running);

                    // Simulate work
                    thread::sleep(Duration::from_millis(100));

                    // Complete or fail randomly
                    if id % 3 == 0 {
                        tracker_clone.update_status(
                            id,
                            TaskStatus::Failed("simulated error".to_string()),
                        );
                    } else {
                        tracker_clone.update_status(id, TaskStatus::Completed);
                    }
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Task status counts: {:?}", tracker.count_by_status());
}
```

**DashMap Benefits**:
- **Lock-free reads**: Multiple threads can read concurrently
- **Sharded locking**: Reduces write contention (16 shards by default)
- **API similar to HashMap**: Easy to use
- **No manual mutex management**: Safer than `Arc<Mutex<HashMap>>`

**Performance**: 10-100x faster than `Arc<Mutex<HashMap>>` for read-heavy workloads

---

### Recipe 13: Comparison of Concurrent Strategies

**Problem**: Choose the right concurrent map strategy for different scenarios.

**Solution**:

```rust
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Instant;

// Strategy 1: Mutex<HashMap>
fn benchmark_mutex_hashmap(iterations: usize, threads: usize) -> Duration {
    let map = Arc::new(Mutex::new(HashMap::new()));
    let start = Instant::now();

    let handles: Vec<_> = (0..threads)
        .map(|t| {
            let map = Arc::clone(&map);
            thread::spawn(move || {
                for i in 0..iterations {
                    let key = t * iterations + i;
                    map.lock().unwrap().insert(key, key * 2);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    start.elapsed()
}

// Strategy 2: RwLock<HashMap>
fn benchmark_rwlock_hashmap(iterations: usize, threads: usize) -> Duration {
    let map = Arc::new(RwLock::new(HashMap::new()));
    let start = Instant::now();

    let handles: Vec<_> = (0..threads)
        .map(|t| {
            let map = Arc::clone(&map);
            thread::spawn(move || {
                for i in 0..iterations {
                    let key = t * iterations + i;
                    map.write().unwrap().insert(key, key * 2);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    start.elapsed()
}

// Strategy 3: DashMap
fn benchmark_dashmap(iterations: usize, threads: usize) -> Duration {
    let map = Arc::new(DashMap::new());
    let start = Instant::now();

    let handles: Vec<_> = (0..threads)
        .map(|t| {
            let map = Arc::clone(&map);
            thread::spawn(move || {
                for i in 0..iterations {
                    let key = t * iterations + i;
                    map.insert(key, key * 2);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    start.elapsed()
}

// Read-heavy benchmark
fn benchmark_read_heavy_dashmap(iterations: usize, threads: usize) -> Duration {
    let map = Arc::new(DashMap::new());

    // Pre-populate
    for i in 0..1000 {
        map.insert(i, i * 2);
    }

    let start = Instant::now();

    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let map = Arc::clone(&map);
            thread::spawn(move || {
                for i in 0..iterations {
                    let _ = map.get(&(i % 1000));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    start.elapsed()
}

use std::time::Duration;

fn main() {
    const ITERATIONS: usize = 10_000;
    const THREADS: usize = 8;

    println!("=== Write-Heavy Benchmark ===");
    println!("({} iterations per thread, {} threads)\n", ITERATIONS, THREADS);

    let mutex_time = benchmark_mutex_hashmap(ITERATIONS, THREADS);
    println!("Mutex<HashMap>:   {:?}", mutex_time);

    let rwlock_time = benchmark_rwlock_hashmap(ITERATIONS, THREADS);
    println!("RwLock<HashMap>:  {:?}", rwlock_time);

    let dashmap_time = benchmark_dashmap(ITERATIONS, THREADS);
    println!("DashMap:          {:?}", dashmap_time);

    println!("\nSpeedup vs Mutex:");
    println!("RwLock:  {:.2}x", mutex_time.as_secs_f64() / rwlock_time.as_secs_f64());
    println!("DashMap: {:.2}x", mutex_time.as_secs_f64() / dashmap_time.as_secs_f64());

    println!("\n=== Read-Heavy Benchmark ===");
    let read_heavy_time = benchmark_read_heavy_dashmap(ITERATIONS * 10, THREADS);
    println!("DashMap (reads): {:?}", read_heavy_time);

    println!("\n=== Recommendations ===");
    println!("Mutex<HashMap>:   Simple, low contention scenarios");
    println!("RwLock<HashMap>:  Read-heavy workloads, moderate contention");
    println!("DashMap:          High contention, many concurrent writers");
}
```

**Strategy Selection Guide**:

| Scenario | Best Choice | Why |
|----------|-------------|-----|
| Low contention | `Mutex<HashMap>` | Simplest, lowest overhead |
| Read-heavy | `RwLock<HashMap>` | Multiple concurrent readers |
| Write-heavy | `DashMap` | Sharded locking reduces contention |
| High concurrency | `DashMap` | Lock-free reads, better scalability |
| Simple access patterns | `Mutex<HashMap>` | Easier to reason about |
| Complex operations | `Mutex<HashMap>` | Full control with guards |

---

## Summary

This chapter covered essential HashMap and HashSet patterns:

1. **Entry API**: Efficient single-lookup operations (or_insert, and_modify)
2. **Custom Hash**: Case-insensitive keys, composite keys, spatial hashing, floating-point keys
3. **Capacity Optimization**: Pre-allocation, load factor management, memory efficiency
4. **Alternative Maps**: BTreeMap for ordering, FxHashMap for speed
5. **Concurrent Maps**: DashMap for high-performance multi-threaded access

**Key Takeaways**:
- Use Entry API to avoid double lookups
- Pre-allocate capacity when size is known
- Choose the right map type for your use case
- FxHashMap is 2-3x faster for integer keys
- DashMap is essential for high-concurrency scenarios
- BTreeMap when you need ordering or range queries

**Performance Tips**:
- Reserve capacity before batch insertions
- Use compact key types to reduce memory
- Consider string interning for repeated strings
- Profile before choosing hash function
- DashMap for >4 concurrent writers
