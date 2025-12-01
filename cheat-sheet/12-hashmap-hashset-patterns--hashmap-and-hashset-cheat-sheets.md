### Hashmap and HashSet Cheat Sheets
```rust
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher, BuildHasher};
use std::collections::hash_map::{Entry, RandomState};

// ===== HASHMAP CREATION =====
// Empty HashMap
let mut map: HashMap<String, i32> = HashMap::new();
let mut map = HashMap::<String, i32>::new();        // Turbofish syntax
let mut map: HashMap<String, i32> = HashMap::default();

// With capacity
let mut map = HashMap::with_capacity(100);          // Pre-allocate for 100 entries

// From array/iterator
let map = HashMap::from([
    ("a", 1),
    ("b", 2),
    ("c", 3),
]);

let map: HashMap<_, _> = vec![("a", 1), ("b", 2)]
    .into_iter()
    .collect();

// With custom hasher
let map: HashMap<String, i32, RandomState> = HashMap::default();
let map = HashMap::with_hasher(RandomState::new());
let map = HashMap::with_capacity_and_hasher(100, RandomState::new());

// ===== HASHMAP INSERTION =====
let mut map = HashMap::new();

// Basic insert
map.insert("key".to_string(), 42);                  // Returns Option<V> (old value)

let old_value = map.insert("key".to_string(), 43); // old_value = Some(42)

// Insert only if absent
map.entry("new_key".to_string()).or_insert(100);   // Insert if missing
map.entry("key".to_string()).or_insert(0);         // Doesn't overwrite existing

// Insert with closure (lazy evaluation)
map.entry("computed".to_string())
    .or_insert_with(|| expensive_computation());

// Insert default value
map.entry("default".to_string()).or_default();     // Uses Default::default()

// ===== ENTRY API =====
// Basic entry manipulation
match map.entry("key".to_string()) {
    Entry::Occupied(e) => {
        println!("Exists: {}", e.get());
        *e.into_mut() += 1;                         // Modify value
    }
    Entry::Vacant(e) => {
        e.insert(0);                                 // Insert new value
    }
}

// Modify if exists, insert if not
map.entry("counter".to_string())
    .and_modify(|v| *v += 1)
    .or_insert(0);

// Get or insert, then modify
*map.entry("score".to_string()).or_insert(0) += 10;

// Get key-value pair from entry
let entry = map.entry("key".to_string());
if let Entry::Occupied(e) = entry {
    let (key, value) = e.remove_entry();           // Remove and return both
}

// ===== HASHMAP ACCESSING =====
let map = HashMap::from([("a", 1), ("b", 2), ("c", 3)]);

// Safe access
map.get("a")                                        // Returns Option<&V>
map.get_mut("a")                                    // Returns Option<&mut V>
map.get_key_value("a")                             // Returns Option<(&K, &V)>

if let Some(value) = map.get("a") {
    println!("Value: {}", value);
}

// Direct access (panics if key missing)
let value = map["a"];                               // Panics if "a" doesn't exist
// let value = map["missing"];                      // PANIC!

// Safe indexing pattern
if let Some(&value) = map.get("a") {
    println!("Value: {}", value);
}

// Get with default
let value = map.get("a").unwrap_or(&0);            // Default if missing
let value = map.get("a").copied().unwrap_or(0);    // Copy value

// ===== HASHMAP CHECKING =====
map.contains_key("a")                               // Check if key exists
map.is_empty()                                      // Check if empty
map.len()                                           // Number of entries

// ===== HASHMAP REMOVAL =====
let mut map = HashMap::from([("a", 1), ("b", 2)]);

map.remove("a")                                     // Remove, return Option<V>
map.remove_entry("a")                              // Remove, return Option<(K, V)>

// Retain only matching entries
map.retain(|key, value| *value > 5);               // Keep entries where value > 5

// Clear all entries
map.clear();                                        // Remove all, keep capacity

// ===== HASHMAP ITERATION =====
let map = HashMap::from([("a", 1), ("b", 2), ("c", 3)]);

// Iterate over references
for (key, value) in &map {                         // (&K, &V)
    println!("{}: {}", key, value);
}

// Iterate with mutable values
let mut map = HashMap::from([("a", 1), ("b", 2)]);
for (key, value) in &mut map {                     // (&K, &mut V)
    *value *= 2;
}

// Consume HashMap
for (key, value) in map {                          // (K, V) - map moved
    println!("{}: {}", key, value);
}

// Iterate keys only
for key in map.keys() {                            // Iterator<Item = &K>
    println!("Key: {}", key);
}

// Iterate values only
for value in map.values() {                        // Iterator<Item = &V>
    println!("Value: {}", value);
}

// Iterate mutable values
for value in map.values_mut() {                    // Iterator<Item = &mut V>
    *value *= 2;
}

// ===== HASHMAP DRAINING =====
let mut map = HashMap::from([("a", 1), ("b", 2)]);

// Drain all entries
for (key, value) in map.drain() {                  // Remove all, iterate (K, V)
    println!("{}: {}", key, value);
}
// map is now empty

// Filter drain (nightly)
// map.drain_filter(|k, v| *v < 10);               // Remove matching entries

// ===== HASHMAP CAPACITY =====
map.capacity()                                      // Current capacity
map.reserve(100)                                    // Reserve space for 100 more
map.shrink_to_fit()                                // Reduce capacity to len
map.shrink_to(10)                                  // Shrink to at least capacity

// ===== HASHMAP RAW ENTRY API =====
use std::collections::hash_map::RawEntryMut;

let mut map: HashMap<String, i32> = HashMap::new();

// Access without hashing key
// Useful for custom key comparison
// map.raw_entry_mut()
//     .from_key("key")
//     .or_insert_with(|| ("key".to_string(), 42));

// ===== HASHSET CREATION =====
// Empty HashSet
let mut set: HashSet<i32> = HashSet::new();
let mut set = HashSet::<i32>::new();
let mut set: HashSet<i32> = HashSet::default();

// With capacity
let mut set = HashSet::with_capacity(100);

// From array/iterator
let set = HashSet::from([1, 2, 3, 4, 5]);

let set: HashSet<_> = vec![1, 2, 3, 4, 5]
    .into_iter()
    .collect();

// With custom hasher
let set = HashSet::with_hasher(RandomState::new());
let set = HashSet::with_capacity_and_hasher(100, RandomState::new());

// ===== HASHSET INSERTION =====
let mut set = HashSet::new();

set.insert(42)                                      // Returns bool (true if new)
set.insert(42)                                      // Returns false (already exists)

// Insert multiple
set.extend([1, 2, 3, 4, 5]);
set.extend(vec![6, 7, 8]);

// ===== HASHSET CHECKING =====
set.contains(&42)                                   // Check if contains value
set.is_empty()                                      // Check if empty
set.len()                                           // Number of elements

// ===== HASHSET REMOVAL =====
let mut set = HashSet::from([1, 2, 3, 4, 5]);

set.remove(&3)                                      // Remove, return bool
set.take(&3)                                        // Remove and return Option<T>

// Retain only matching
set.retain(|x| *x > 2);                            // Keep elements > 2

// Clear all
set.clear();                                        // Remove all elements

// ===== HASHSET ITERATION =====
let set = HashSet::from([1, 2, 3, 4, 5]);

// Iterate over references
for item in &set {                                  // &T
    println!("{}", item);
}

// Consume set
for item in set {                                   // T - set moved
    println!("{}", item);
}

// ===== HASHSET DRAINING =====
let mut set = HashSet::from([1, 2, 3]);

for item in set.drain() {                          // Remove all, iterate T
    println!("{}", item);
}
// set is now empty

// ===== HASHSET SET OPERATIONS =====
let a = HashSet::from([1, 2, 3, 4]);
let b = HashSet::from([3, 4, 5, 6]);

// Union - all elements from both sets
let union: HashSet<_> = a.union(&b).cloned().collect();
for item in a.union(&b) {                          // Iterator<Item = &T>
    println!("{}", item);
}

// Intersection - common elements
let intersection: HashSet<_> = a.intersection(&b).cloned().collect();
for item in a.intersection(&b) {
    println!("{}", item);
}

// Difference - elements in a but not in b
let difference: HashSet<_> = a.difference(&b).cloned().collect();
for item in a.difference(&b) {
    println!("{}", item);
}

// Symmetric difference - elements in either but not both
let sym_diff: HashSet<_> = a.symmetric_difference(&b).cloned().collect();
for item in a.symmetric_difference(&b) {
    println!("{}", item);
}

// ===== HASHSET PREDICATES =====
let a = HashSet::from([1, 2, 3]);
let b = HashSet::from([2, 3, 4]);
let c = HashSet::from([1, 2]);

a.is_disjoint(&b)                                  // No common elements: false
a.is_subset(&b)                                     // All elements in b: false
a.is_superset(&c)                                   // Contains all of c: true
c.is_subset(&a)                                     // c is subset of a: true

// ===== HASHSET CAPACITY =====
set.capacity()                                      // Current capacity
set.reserve(100)                                    // Reserve space for 100 more
set.shrink_to_fit()                                // Reduce capacity to len
set.shrink_to(10)                                  // Shrink to at least capacity

// ===== CUSTOM HASH IMPLEMENTATION =====
#[derive(Debug, Eq, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

// Manual Hash implementation
impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

// Now Point can be used as HashMap key or HashSet element
let mut map: HashMap<Point, String> = HashMap::new();
map.insert(Point { x: 1, y: 2 }, "origin".to_string());

let mut set: HashSet<Point> = HashSet::new();
set.insert(Point { x: 1, y: 2 });

// ===== COMMON PATTERNS =====
// Pattern 1: Frequency counter
let text = "hello world";
let mut freq: HashMap<char, usize> = HashMap::new();
for c in text.chars() {
    *freq.entry(c).or_insert(0) += 1;
}

// Pattern 2: Group by key
let items = vec![("fruit", "apple"), ("veg", "carrot"), ("fruit", "banana")];
let mut grouped: HashMap<&str, Vec<&str>> = HashMap::new();
for (category, item) in items {
    grouped.entry(category).or_insert_with(Vec::new).push(item);
}

// Pattern 3: Default values with entry
let mut scores: HashMap<String, i32> = HashMap::new();
for player in players {
    scores.entry(player).or_insert(0);             // Initialize if missing
}

// Pattern 4: Accumulate values
let mut totals: HashMap<String, i32> = HashMap::new();
for (key, value) in data {
    *totals.entry(key).or_insert(0) += value;
}

// Pattern 5: Invert HashMap
let original = HashMap::from([("a", 1), ("b", 2)]);
let inverted: HashMap<_, _> = original.iter()
    .map(|(k, v)| (v, k))
    .collect();

// Pattern 6: Remove duplicates with HashSet
let numbers = vec![1, 2, 2, 3, 3, 3, 4];
let unique: Vec<_> = numbers.into_iter()
    .collect::<HashSet<_>>()
    .into_iter()
    .collect();

// Pattern 7: Remove duplicates preserving order
let numbers = vec![1, 2, 2, 3, 1, 4];
let mut seen = HashSet::new();
let unique: Vec<_> = numbers.into_iter()
    .filter(|x| seen.insert(*x))
    .collect();

// Pattern 8: Find missing elements
let a = HashSet::from([1, 2, 3, 4, 5]);
let b = HashSet::from([3, 4, 5, 6, 7]);
let missing: Vec<_> = a.difference(&b).collect();  // [1, 2]

// Pattern 9: Check for duplicates
let numbers = vec![1, 2, 3, 4, 5];
let has_duplicates = numbers.len() != numbers.iter().collect::<HashSet<_>>().len();

// Pattern 10: Cache/memoization
let mut cache: HashMap<i32, i32> = HashMap::new();
fn fibonacci(n: i32, cache: &mut HashMap<i32, i32>) -> i32 {
    if let Some(&result) = cache.get(&n) {
        return result;
    }
    let result = if n <= 1 { n } else { fibonacci(n - 1, cache) + fibonacci(n - 2, cache) };
    cache.insert(n, result);
    result
}

// Pattern 11: Two sum problem
fn two_sum(nums: Vec<i32>, target: i32) -> Option<(usize, usize)> {
    let mut map: HashMap<i32, usize> = HashMap::new();
    for (i, &num) in nums.iter().enumerate() {
        if let Some(&j) = map.get(&(target - num)) {
            return Some((j, i));
        }
        map.insert(num, i);
    }
    None
}

// Pattern 12: Count elements in ranges
let numbers = vec![1, 5, 10, 15, 20, 25];
let mut ranges: HashMap<String, usize> = HashMap::new();
for &num in &numbers {
    let range = match num {
        0..=10 => "0-10",
        11..=20 => "11-20",
        _ => "20+",
    };
    *ranges.entry(range.to_string()).or_insert(0) += 1;
}

// Pattern 13: Merge HashMaps
let mut map1 = HashMap::from([("a", 1), ("b", 2)]);
let map2 = HashMap::from([("b", 3), ("c", 4)]);
map1.extend(map2);                                  // Overwrites duplicates

// Or sum values:
for (k, v) in map2 {
    *map1.entry(k).or_insert(0) += v;
}

// Pattern 14: Multi-value HashMap (multimap)
let mut multimap: HashMap<String, Vec<i32>> = HashMap::new();
multimap.entry("key".to_string()).or_default().push(1);
multimap.entry("key".to_string()).or_default().push(2);

// Pattern 15: Lazy initialization with entry
let mut cache: HashMap<String, Vec<i32>> = HashMap::new();
let values = cache.entry("key".to_string())
    .or_insert_with(|| load_from_database());

// Pattern 16: Update all values
let mut map = HashMap::from([("a", 1), ("b", 2), ("c", 3)]);
for value in map.values_mut() {
    *value *= 2;
}

// Pattern 17: Filter HashMap
let map = HashMap::from([("a", 1), ("b", 2), ("c", 3)]);
let filtered: HashMap<_, _> = map.into_iter()
    .filter(|(_, &v)| v > 1)
    .collect();

// Pattern 18: Get or compute
let value = map.entry("key".to_string())
    .or_insert_with(|| expensive_computation());

// Pattern 19: Swap keys and values
let map = HashMap::from([("a", 1), ("b", 2)]);
let swapped: HashMap<_, _> = map.into_iter()
    .map(|(k, v)| (v, k))
    .collect();

// Pattern 20: Count occurrences
let words = vec!["apple", "banana", "apple", "cherry", "banana", "apple"];
let counts = words.iter()
    .fold(HashMap::new(), |mut map, &word| {
        *map.entry(word).or_insert(0) += 1;
        map
    });

// Pattern 21: Find mode (most frequent)
let numbers = vec![1, 2, 2, 3, 3, 3];
let freq: HashMap<i32, usize> = numbers.iter()
    .fold(HashMap::new(), |mut map, &n| {
        *map.entry(n).or_insert(0) += 1;
        map
    });
let mode = freq.iter()
    .max_by_key(|(_, &count)| count)
    .map(|(&num, _)| num);

// Pattern 22: Check if sets are equal
let set1 = HashSet::from([1, 2, 3]);
let set2 = HashSet::from([3, 2, 1]);
let are_equal = set1 == set2;                      // true

// Pattern 23: Conditional insert
let mut map = HashMap::new();
if !map.contains_key("key") {
    map.insert("key", expensive_computation());
}
// Better:
map.entry("key").or_insert_with(|| expensive_computation());

// Pattern 24: Batch operations
let updates = vec![("a", 10), ("b", 20), ("c", 30)];
let mut map = HashMap::new();
for (key, value) in updates {
    map.insert(key, value);
}

// Pattern 25: Graph adjacency list
let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
graph.entry(1).or_default().extend([2, 3]);
graph.entry(2).or_default().push(4);
graph.entry(3).or_default().extend([4, 5]);
```