
### Advanced Collections Cheat Sheet
```rust
use std::collections::*;

// ===== HASHMAP =====
// Creating
let mut map: HashMap<String, i32> = HashMap::new();
let mut map = HashMap::with_capacity(10);          // Pre-allocate
let map = HashMap::from([("a", 1), ("b", 2)]);    // From array

// Inserting
map.insert("key".to_string(), 42);                 // Insert/update, returns old value
map.entry("key".to_string()).or_insert(42);        // Insert if missing
map.entry("key".to_string()).or_insert_with(|| 42); // With closure
map.entry("key".to_string()).or_default();         // Insert Default::default()

// Entry API - modify or insert
map.entry("counter".to_string())
    .and_modify(|v| *v += 1)
    .or_insert(0);

// Accessing
map.get("key")                                      // Get Option<&V>
map.get_mut("key")                                  // Get Option<&mut V>
map["key"]                                          // Direct access (panics if missing)
map.get_key_value("key")                           // Get Option<(&K, &V)>

// Removing
map.remove("key")                                   // Remove, return Option<V>
map.remove_entry("key")                            // Remove, return Option<(K, V)>
map.retain(|k, v| *v > 10)                         // Keep only matching

// Checking
map.contains_key("key")                            // Check if key exists
map.is_empty()                                      // Check if empty
map.len()                                           // Number of entries

// Iteration
for (key, value) in &map {                         // Iterate over (&K, &V)
    println!("{}: {}", key, value);
}
for key in map.keys() {                            // Iterate over &K
    println!("{}", key);
}
for value in map.values() {                        // Iterate over &V
    println!("{}", value);
}
for value in map.values_mut() {                    // Iterate over &mut V
    *value *= 2;
}

// Draining
for (k, v) in map.drain() {                        // Remove all, iterate over (K, V)
    println!("{}: {}", k, v);
}

// Capacity
map.capacity()                                      // Current capacity
map.reserve(100)                                    // Reserve additional space
map.shrink_to_fit()                                // Reduce capacity

// ===== BTREEMAP (SORTED MAP) =====
let mut map: BTreeMap<String, i32> = BTreeMap::new();
map.insert("c".to_string(), 3);
map.insert("a".to_string(), 1);
map.insert("b".to_string(), 2);

// Range queries
map.range("a".to_string().."c".to_string())        // Range of entries
    .for_each(|(k, v)| println!("{}: {}", k, v));

map.range(.."b".to_string())                       // Up to key
map.range("b".to_string()..)                       // From key onwards

// Split off
let right = map.split_off("b");                    // Split at key

// First/last
map.first_entry()                                   // First entry (Option)
map.last_entry()                                    // Last entry (Option)
map.first_key_value()                              // First (&K, &V)
map.last_key_value()                               // Last (&K, &V)
map.pop_first()                                     // Remove first
map.pop_last()                                      // Remove last

// ===== HASHSET =====
// Creating
let mut set: HashSet<i32> = HashSet::new();
let mut set = HashSet::with_capacity(10);
let set = HashSet::from([1, 2, 3, 4, 5]);

// Inserting
set.insert(42)                                      // Insert, returns bool (true if new)

// Removing
set.remove(&42)                                     // Remove, returns bool
set.take(&42)                                       // Remove and return Option<T>
set.retain(|x| *x > 10)                            // Keep only matching

// Checking
set.contains(&42)                                   // Check if contains
set.is_empty()                                      // Check if empty
set.len()                                           // Number of elements

// Set operations
let a = HashSet::from([1, 2, 3, 4]);
let b = HashSet::from([3, 4, 5, 6]);

a.union(&b)                                         // Union iterator
a.intersection(&b)                                  // Intersection iterator
a.difference(&b)                                    // Difference iterator (a - b)
a.symmetric_difference(&b)                         // Symmetric difference

a.is_disjoint(&b)                                  // No common elements
a.is_subset(&b)                                     // All elements in b
a.is_superset(&b)                                   // Contains all elements of b

// Collect set operations
let union: HashSet<_> = a.union(&b).collect();
let intersection: HashSet<_> = a.intersection(&b).cloned().collect();

// Iteration
for item in &set {
    println!("{}", item);
}

// Drain
for item in set.drain() {
    println!("{}", item);
}

// ===== BTREESET (SORTED SET) =====
let mut set: BTreeSet<i32> = BTreeSet::new();
set.insert(3);
set.insert(1);
set.insert(2);

// Range queries
set.range(1..3)                                     // Range of elements
    .for_each(|x| println!("{}", x));

set.range(..3)                                      // Up to value
set.range(2..)                                      // From value onwards

// Split off
let right = set.split_off(&2);                     // Split at value

// First/last
set.first()                                         // First element (Option<&T>)
set.last()                                          // Last element
set.pop_first()                                     // Remove first
set.pop_last()                                      // Remove last

// ===== VECDEQUE (DOUBLE-ENDED QUEUE) =====
let mut deque: VecDeque<i32> = VecDeque::new();
let mut deque = VecDeque::with_capacity(10);
let deque = VecDeque::from([1, 2, 3, 4, 5]);

// Adding elements
deque.push_back(42)                                 // Add to back
deque.push_front(0)                                 // Add to front

// Removing elements
deque.pop_back()                                    // Remove from back: Option<T>
deque.pop_front()                                   // Remove from front: Option<T>

// Accessing
deque.front()                                       // First element: Option<&T>
deque.back()                                        // Last element: Option<&T>
deque.front_mut()                                   // Mutable first
deque.back_mut()                                    // Mutable last
deque[0]                                            // Index access
deque.get(0)                                        // Safe access: Option<&T>

// Inserting/removing at index
deque.insert(2, 99)                                 // Insert at index
deque.remove(2)                                     // Remove at index: Option<T>

// Rotation
deque.rotate_left(2)                               // Rotate left
deque.rotate_right(2)                              // Rotate right

// Splitting
let (first, second) = deque.as_slices();           // Get as two slices
let (first, second) = deque.as_mut_slices();       // Mutable slices

// Iteration
for item in &deque {
    println!("{}", item);
}

// Range operations
deque.range(1..3)                                   // Range of elements
deque.drain(1..3)                                   // Drain range

// Make contiguous
deque.make_contiguous()                            // Rearrange to single slice

// ===== BINARYHEAP (PRIORITY QUEUE) =====
use std::collections::BinaryHeap;
let mut heap = BinaryHeap::new();
let heap = BinaryHeap::from([3, 1, 4, 1, 5]);

// Adding
heap.push(42)                                       // Add element

// Removing
heap.pop()                                          // Remove max: Option<T>

// Peeking
heap.peek()                                         // View max: Option<&T>
heap.peek_mut()                                     // Mutable view of max

// Min-heap (using Reverse)
use std::cmp::Reverse;
let mut min_heap = BinaryHeap::new();
min_heap.push(Reverse(5));
min_heap.push(Reverse(3));
min_heap.push(Reverse(7));
min_heap.pop()                                      // Pops min: Reverse(3)

// Iteration (unordered)
for item in &heap {
    println!("{}", item);
}

// Into sorted vec
let sorted = heap.into_sorted_vec();               // Consume heap, get sorted Vec

// Drain
for item in heap.drain() {
    println!("{}", item);
}

// ===== LINKEDLIST =====
let mut list: LinkedList<i32> = LinkedList::new();
let list = LinkedList::from([1, 2, 3]);

// Adding
list.push_back(42)                                  // Add to back
list.push_front(0)                                  // Add to front

// Removing
list.pop_back()                                     // Remove from back: Option<T>
list.pop_front()                                    // Remove from front: Option<T>

// Accessing
list.front()                                        // First element: Option<&T>
list.back()                                         // Last element: Option<&T>
list.front_mut()                                    // Mutable first
list.back_mut()                                     // Mutable last

// Splitting
let back = list.split_off(2);                      // Split at index

// Append
list.append(&mut other_list)                       // Append other list (moves)

// Cursor (advanced iteration and mutation)
let mut cursor = list.cursor_front_mut();          // Cursor at front
cursor.insert_before(99);                           // Insert before cursor
cursor.insert_after(88);                           // Insert after cursor

// ===== CUSTOM HASHERS =====
// Using different hasher
use std::collections::hash_map::RandomState;
let map: HashMap<String, i32, RandomState> = HashMap::default();

// Fast hasher (requires ahash crate)
// use ahash::AHashMap;
// let map: AHashMap<String, i32> = AHashMap::new();

// FxHash (requires rustc-hash crate)
// use rustc_hash::FxHashMap;
// let map: FxHashMap<String, i32> = FxHashMap::default();

// ===== CUSTOM COMPARATORS =====
// BTreeMap with custom ordering
use std::cmp::Ordering;
let mut map = BTreeMap::new();
// Must use wrapper type with custom Ord

#[derive(Eq, PartialEq)]
struct CustomKey(i32);

impl Ord for CustomKey {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)                       // Reverse order
    }
}

impl PartialOrd for CustomKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// ===== COMMON PATTERNS =====
// Pattern 1: Frequency counter
let text = "hello world";
let mut freq: HashMap<char, usize> = HashMap::new();
for c in text.chars() {
    *freq.entry(c).or_insert(0) += 1;
}

// Pattern 2: Group by key
let items = vec![("a", 1), ("b", 2), ("a", 3)];
let mut grouped: HashMap<&str, Vec<i32>> = HashMap::new();
for (key, val) in items {
    grouped.entry(key).or_insert_with(Vec::new).push(val);
}

// Pattern 3: Merge two maps
let mut map1 = HashMap::from([("a", 1), ("b", 2)]);
let map2 = HashMap::from([("b", 3), ("c", 4)]);
for (k, v) in map2 {
    *map1.entry(k).or_insert(0) += v;
}

// Pattern 4: Default HashMap with entry
let mut scores = HashMap::new();
let score = scores.entry("player1").or_insert(0);
*score += 10;

// Pattern 5: Cache with HashMap
use std::collections::HashMap;
let mut cache: HashMap<String, String> = HashMap::new();
let result = cache.entry(key.clone())
    .or_insert_with(|| expensive_computation(&key));

// Pattern 6: Set operations - union
let set1 = HashSet::from([1, 2, 3]);
let set2 = HashSet::from([3, 4, 5]);
let union: HashSet<_> = set1.union(&set2).cloned().collect();

// Pattern 7: Set operations - intersection
let intersection: HashSet<_> = set1.intersection(&set2).cloned().collect();

// Pattern 8: Deque as sliding window
let mut window: VecDeque<i32> = VecDeque::with_capacity(5);
for value in data {
    if window.len() == 5 {
        window.pop_front();
    }
    window.push_back(value);
    let sum: i32 = window.iter().sum();
}

// Pattern 9: Priority queue with custom priority
#[derive(Eq, PartialEq)]
struct Task {
    priority: i32,
    name: String,
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

let mut tasks = BinaryHeap::new();
tasks.push(Task { priority: 5, name: "task1".into() });

// Pattern 10: LRU Cache (requires lru crate)
// use lru::LruCache;
// let mut cache = LruCache::new(100);
// cache.put("key", "value");
// cache.get("key");

// Pattern 11: Multi-map (one key, multiple values)
let mut multimap: HashMap<String, Vec<i32>> = HashMap::new();
multimap.entry("key".to_string()).or_default().push(1);
multimap.entry("key".to_string()).or_default().push(2);

// Pattern 12: Bidirectional map
let mut forward: HashMap<String, i32> = HashMap::new();
let mut reverse: HashMap<i32, String> = HashMap::new();
forward.insert("key".to_string(), 42);
reverse.insert(42, "key".to_string());

// Pattern 13: Index map (ordered HashMap - requires indexmap crate)
// use indexmap::IndexMap;
// let mut map = IndexMap::new();
// map.insert("a", 1);
// map.insert("b", 2);
// Maintains insertion order

// Pattern 14: Topological sort with BTreeSet
let mut in_degree: HashMap<i32, usize> = HashMap::new();
let mut ready: BTreeSet<i32> = BTreeSet::new();
for node in nodes {
    if in_degree[&node] == 0 {
        ready.insert(node);
    }
}

// Pattern 15: Graph adjacency list
let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
graph.entry(1).or_default().push(2);
graph.entry(1).or_default().push(3);
graph.entry(2).or_default().push(4);

// Pattern 16: Remove duplicates maintaining order
let items = vec![1, 2, 2, 3, 1, 4];
let mut seen = HashSet::new();
let unique: Vec<_> = items.into_iter()
    .filter(|x| seen.insert(*x))
    .collect();

// Pattern 17: Find mode (most frequent)
let numbers = vec![1, 2, 2, 3, 3, 3];
let freq: HashMap<i32, usize> = numbers.iter()
    .fold(HashMap::new(), |mut map, &n| {
        *map.entry(n).or_insert(0) += 1;
        map
    });
let mode = freq.iter()
    .max_by_key(|(_, &count)| count)
    .map(|(&num, _)| num);

// Pattern 18: Two-way lookup
use std::collections::HashMap;
struct BiMap<K, V> {
    forward: HashMap<K, V>,
    reverse: HashMap<V, K>,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone + Eq + std::hash::Hash> BiMap<K, V> {
    fn insert(&mut self, k: K, v: V) {
        self.forward.insert(k.clone(), v.clone());
        self.reverse.insert(v, k);
    }
}
```