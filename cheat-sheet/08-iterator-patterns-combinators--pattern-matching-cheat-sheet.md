### Pattern Matching Cheat Sheet
```rust
// ===== CREATING ITERATORS =====
// From collections
let vec = vec![1, 2, 3, 4, 5];
vec.iter()                                          // Iterator over &T
vec.iter_mut()                                      // Iterator over &mut T
vec.into_iter()                                     // Iterator over T (consumes)

let arr = [1, 2, 3, 4, 5];
arr.iter()                                          // Iterator over &T
arr.into_iter()                                     // Iterator over &T (arrays)

let slice = &[1, 2, 3, 4, 5][..];
slice.iter()                                        // Iterator over &T

// From ranges
(0..10)                                             // Range: 0 to 9
(0..=10)                                            // Inclusive range: 0 to 10
(0..)                                               // Infinite range from 0
(..10)                                              // Range from start to 9

// Iterate strings
let s = "hello";
s.chars()                                           // Iterator over chars
s.bytes()                                           // Iterator over bytes
s.lines()                                           // Iterator over lines
s.split_whitespace()                                // Iterator over words

// HashMap iteration
use std::collections::HashMap;
let map = HashMap::from([("a", 1), ("b", 2)]);
map.iter()                                          // Iterator over (&K, &V)
map.keys()                                          // Iterator over &K
map.values()                                        // Iterator over &V
map.into_iter()                                     // Iterator over (K, V)

// ===== CONSUMING ITERATORS =====
let vec = vec![1, 2, 3, 4, 5];

// Collect into collection
vec.iter().collect::<Vec<_>>()                     // Collect to Vec
vec.iter().collect::<HashSet<_>>()                 // Collect to HashSet
vec.iter().cloned().collect::<Vec<i32>>()         // Clone elements

// Count elements
vec.iter().count()                                  // Number of elements
vec.iter().filter(|x| **x > 2).count()            // Count matching

// Find elements
vec.iter().find(|x| **x > 3)                       // First matching: Option<&T>
vec.iter().position(|x| *x > 3)                    // Index of first match: Option<usize>
vec.iter().rposition(|x| *x > 3)                   // Last matching index

// Any/all predicates
vec.iter().any(|x| *x > 3)                         // true if any match
vec.iter().all(|x| *x > 0)                         // true if all match

// Nth element
vec.iter().nth(2)                                   // Get element at index: Option<&T>
vec.iter().last()                                   // Get last element

// Max/min
vec.iter().max()                                    // Maximum: Option<&T>
vec.iter().min()                                    // Minimum: Option<&T>
vec.iter().max_by(|a, b| a.cmp(b))                // Max with comparator
vec.iter().max_by_key(|x| x.abs())                // Max by key function

// Sum/product
vec.iter().sum::<i32>()                            // Sum all elements
vec.iter().product::<i32>()                        // Product of all elements

// Fold/reduce
vec.iter().fold(0, |acc, x| acc + x)               // Fold with initial value
vec.iter().reduce(|acc, x| acc + x)                // Reduce (no initial): Option<T>

// For each
vec.iter().for_each(|x| println!("{}", x))        // Apply function to each

// Partition
let (even, odd): (Vec<_>, Vec<_>) = vec.iter()
    .partition(|x| *x % 2 == 0);                   // Split by predicate

// ===== ADAPTER METHODS (LAZY) =====
// Map - transform elements
vec.iter().map(|x| x * 2)                          // Transform each element
vec.iter().map(|x| x.to_string())                  // Change type

// Filter - keep matching elements
vec.iter().filter(|x| **x > 2)                     // Keep elements matching predicate
vec.iter().filter(|x| **x % 2 == 0)               // Keep even numbers

// Filter map - filter and map combined
vec.iter().filter_map(|x| {
    if *x > 2 { Some(x * 2) } else { None }
})

// Take - limit number of elements
vec.iter().take(3)                                  // First 3 elements
vec.iter().take_while(|x| **x < 4)                 // Take while condition true

// Skip - skip elements
vec.iter().skip(2)                                  // Skip first 2 elements
vec.iter().skip_while(|x| **x < 3)                 // Skip while condition true

// Chain - concatenate iterators
vec.iter().chain([6, 7, 8].iter())                 // Concatenate two iterators

// Zip - combine two iterators
let names = vec!["Alice", "Bob"];
let ages = vec![25, 30];
names.iter().zip(ages.iter())                      // Iterator over pairs

// Enumerate - add index
vec.iter().enumerate()                              // Iterator over (index, value)

// Cycle - repeat infinitely
vec.iter().cycle()                                  // Infinite repetition
vec.iter().cycle().take(10)                        // Repeat with limit

// Rev - reverse iterator
vec.iter().rev()                                    // Reverse order

// Cloned/copied - convert &T to T
vec.iter().cloned()                                 // Clone each element
vec.iter().copied()                                 // Copy each element (Copy trait)

// Step by - take every nth element
(0..10).step_by(2)                                  // 0, 2, 4, 6, 8

// Scan - stateful map
(1..5).scan(0, |acc, x| {
    *acc += x;
    Some(*acc)
})                                                  // 1, 3, 6, 10 (running sum)

// Flat map - map and flatten
vec![vec![1, 2], vec![3, 4]]
    .iter()
    .flat_map(|v| v.iter())                        // Iterator over all elements

// Flatten - flatten nested structures
vec![vec![1, 2], vec![3, 4]]
    .into_iter()
    .flatten()                                      // 1, 2, 3, 4

// Inspect - peek at elements (for debugging)
vec.iter()
    .inspect(|x| println!("About to filter: {}", x))
    .filter(|x| **x > 2)
    .inspect(|x| println!("After filter: {}", x))
    .collect::<Vec<_>>()

// Peekable - look ahead without consuming
let mut iter = vec.iter().peekable();
if let Some(&&first) = iter.peek() {               // Look at first without consuming
    println!("First: {}", first);
}
iter.next();                                        // Now consume first

// Fuse - stop after first None
let vec = vec![Some(1), Some(2), None, Some(4)];
vec.into_iter().fuse().collect::<Vec<_>>()        // Stops at None

// ===== CHAINING COMBINATORS =====
// Complex pipeline
let result: Vec<_> = vec
    .iter()
    .filter(|x| **x > 1)                           // Keep > 1
    .map(|x| x * 2)                                // Double
    .filter(|x| *x < 10)                           // Keep < 10
    .collect();

// Multiple transformations
let result: i32 = (1..10)
    .filter(|x| x % 2 == 0)                        // Even numbers
    .map(|x| x * x)                                // Square
    .take(3)                                        // First 3
    .sum();                                         // Sum

// ===== IMPLEMENTING ITERATOR =====
struct Counter {
    count: u32,
}

impl Counter {
    fn new() -> Counter {
        Counter { count: 0 }
    }
}

impl Iterator for Counter {
    type Item = u32;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.count < 5 {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}

let counter = Counter::new();
for num in counter {
    println!("{}", num);
}

// ===== ITERATOR TRAITS =====
// Iterator - basic iteration
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

// DoubleEndedIterator - iterate from both ends
let vec = vec![1, 2, 3, 4, 5];
let mut iter = vec.iter();
iter.next()                                         // Some(&1) from front
iter.next_back()                                    // Some(&5) from back

// ExactSizeIterator - known size
vec.iter().len()                                    // Exact remaining length

// FusedIterator - always None after first None

// ===== INFINITE ITERATORS =====
// Repeat value
std::iter::repeat(5).take(3)                       // [5, 5, 5]

// Repeat with function
std::iter::repeat_with(|| rand::random::<i32>())
    .take(5)                                        // 5 random numbers

// Successors - generate sequence
std::iter::successors(Some(1), |n| Some(n * 2))
    .take(5)                                        // [1, 2, 4, 8, 16]

// Once - single element iterator
std::iter::once(5)                                  // Single element

// Empty - empty iterator
std::iter::empty::<i32>()                          // No elements

// From function
std::iter::from_fn(|| Some(42)).take(3)            // Custom generator

// ===== WINDOWS AND CHUNKS =====
let slice = &[1, 2, 3, 4, 5];

// Windows - overlapping chunks
slice.windows(3)                                    // [[1,2,3], [2,3,4], [3,4,5]]

// Chunks - non-overlapping chunks
slice.chunks(2)                                     // [[1,2], [3,4], [5]]
slice.chunks_exact(2)                              // [[1,2], [3,4]] (drops remainder)

// Mutable versions
let mut vec = vec![1, 2, 3, 4, 5];
vec.chunks_mut(2)                                   // Mutable chunks
    .for_each(|chunk| chunk[0] *= 2);

// ===== ITERATOR METHODS WITH CLOSURES =====
// Find with closure
vec.iter().find(|&&x| x > 3)

// Find map - find and transform
vec.iter().find_map(|&x| {
    if x > 3 { Some(x * 2) } else { None }
})

// Map while - map until condition fails
(1..10).map_while(|x| {
    if x < 5 { Some(x * 2) } else { None }
})

// Try fold - fold that can short-circuit
vec.iter().try_fold(0, |acc, &x| {
    if x > 0 { Some(acc + x) } else { None }
})

// Try for each - for each that can short-circuit
vec.iter().try_for_each(|&x| {
    if x > 0 { Some(()) } else { None }
})

// ===== COMMON PATTERNS =====
// Pattern 1: Collect with type inference
let doubled: Vec<i32> = vec.iter().map(|x| x * 2).collect();

// Pattern 2: Filter and collect
let evens: Vec<_> = vec.iter()
    .filter(|&&x| x % 2 == 0)
    .collect();

// Pattern 3: Sum after transformation
let sum: i32 = vec.iter()
    .map(|x| x * 2)
    .sum();

// Pattern 4: Find first matching
if let Some(&value) = vec.iter().find(|&&x| x > 3) {
    println!("Found: {}", value);
}

// Pattern 5: Group by (requires itertools crate)
use itertools::Itertools;
let grouped: Vec<_> = vec.iter()
    .group_by(|&&x| x % 2)
    .into_iter()
    .map(|(key, group)| (key, group.collect::<Vec<_>>()))
    .collect();

// Pattern 6: Cartesian product
let a = vec![1, 2];
let b = vec!['a', 'b'];
for x in &a {
    for y in &b {
        println!("{}{}", x, y);
    }
}

// Or with itertools:
use itertools::Itertools;
a.iter().cartesian_product(b.iter())

// Pattern 7: Chunking with collect
let chunks: Vec<Vec<_>> = vec.chunks(2)
    .map(|chunk| chunk.to_vec())
    .collect();

// Pattern 8: Conditional iteration
let result = if condition {
    vec.iter().map(|x| x * 2).collect()
} else {
    vec.iter().map(|x| x * 3).collect()
};

// Pattern 9: Iterator over Option/Result
let vec = vec![Some(1), None, Some(3)];
let values: Vec<_> = vec.into_iter()
    .flatten()                                      // Skip None values
    .collect();

let results = vec![Ok(1), Err("error"), Ok(3)];
let values: Result<Vec<_>, _> = results.into_iter().collect(); // Collect Result

// Pattern 10: Custom step iteration
(0..10)
    .step_by(2)
    .collect::<Vec<_>>()                           // [0, 2, 4, 6, 8]

// Pattern 11: Zip with index
vec.iter()
    .enumerate()
    .map(|(i, x)| format!("{}: {}", i, x))
    .collect::<Vec<_>>()

// Pattern 12: Parallel iteration with Rayon
use rayon::prelude::*;
vec.par_iter()
    .map(|x| x * 2)
    .collect::<Vec<_>>()

// Pattern 13: Sorted iteration (requires itertools)
use itertools::Itertools;
vec.iter().sorted().collect::<Vec<_>>()

// Pattern 14: Unique elements (requires itertools)
vec.iter().unique().collect::<Vec<_>>()

// Pattern 15: Intersperse elements (requires itertools)
vec.iter().intersperse(&0).collect::<Vec<_>>()    // Insert 0 between elements

// Pattern 16: Take n largest/smallest
use itertools::Itertools;
vec.iter().k_largest(3).collect::<Vec<_>>()

// Pattern 17: Fold with early exit using try_fold
let result = vec.iter().try_fold(0, |acc, &x| {
    if x < 0 {
        None                                        // Stop early
    } else {
        Some(acc + x)
    }
});

// Pattern 18: Create HashMap from iterator
let map: HashMap<_, _> = vec.iter()
    .enumerate()
    .map(|(i, &x)| (i, x))
    .collect();

// Pattern 19: Batching (requires itertools)
use itertools::Itertools;
vec.iter().chunks(3)
    .into_iter()
    .map(|chunk| chunk.collect::<Vec<_>>())
    .collect::<Vec<_>>()

// Pattern 20: Cycle through values
let cycle_iter = vec.iter().cycle().take(10);
```
