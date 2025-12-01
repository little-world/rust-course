### Vector and Slices Cheat Sheet
```rust
// ===== CREATING VECTORS =====
// Empty vector
let v: Vec<i32> = Vec::new();                       // Empty Vec
let v = Vec::<i32>::new();                          // With turbofish
let v: Vec<i32> = vec![];                           // Empty with macro

// With initial values
let v = vec![1, 2, 3, 4, 5];                        // Vec macro
let v = vec![0; 5];                                 // [0, 0, 0, 0, 0]

// With capacity
let mut v = Vec::with_capacity(10);                 // Pre-allocate space

// From array
let v = Vec::from([1, 2, 3]);                       // From array
let v: Vec<i32> = [1, 2, 3].to_vec();              // Array to Vec

// From iterator
let v: Vec<_> = (0..10).collect();                  // Collect from range
let v: Vec<_> = "hello".chars().collect();         // Collect chars

// From slice
let slice = &[1, 2, 3];
let v = slice.to_vec();                             // Clone slice to Vec

// ===== VECTOR CAPACITY =====
let mut v = Vec::with_capacity(10);
v.len()                                             // Number of elements
v.capacity()                                        // Allocated capacity
v.is_empty()                                        // Check if empty

v.reserve(20)                                       // Reserve additional space
v.reserve_exact(20)                                 // Reserve exact space
v.shrink_to_fit()                                   // Reduce capacity to len
v.shrink_to(5)                                      // Shrink to at least n capacity

// ===== ADDING ELEMENTS =====
let mut v = vec![1, 2, 3];

v.push(4)                                           // Add to end
v.append(&mut vec![5, 6])                          // Append another Vec (moves)
v.extend([7, 8, 9])                                // Extend from iterable
v.extend_from_slice(&[10, 11])                     // Extend from slice

v.insert(0, 0)                                      // Insert at index
v.insert(v.len(), 99)                              // Insert at end

// ===== REMOVING ELEMENTS =====
let mut v = vec![1, 2, 3, 4, 5];

v.pop()                                             // Remove last: Option<T>
v.remove(2)                                         // Remove at index: T (shifts elements)
v.swap_remove(2)                                    // Remove at index: T (swaps with last, O(1))

v.clear()                                           // Remove all elements
v.truncate(3)                                       // Keep only first n elements

v.retain(|x| *x > 2)                               // Keep elements matching predicate
v.retain_mut(|x| { *x *= 2; *x > 4 })            // Retain with mutation

v.dedup()                                           // Remove consecutive duplicates
v.dedup_by(|a, b| a == b)                          // Dedup with custom comparison
v.dedup_by_key(|x| *x)                             // Dedup by key function

// Drain - remove and return iterator
let drained: Vec<_> = v.drain(1..3).collect();     // Remove range
let all: Vec<_> = v.drain(..).collect();           // Remove all (empties vec)
v.drain_filter(|x| *x % 2 == 0);                   // Remove matching (nightly)

// ===== ACCESSING ELEMENTS =====
let v = vec![1, 2, 3, 4, 5];

v[0]                                                // Index access (panics if out of bounds)
v.get(0)                                            // Safe access: Option<&T>
v.get(10)                                           // Returns None if out of bounds
v.get_mut(0)                                        // Mutable access: Option<&mut T>

v.first()                                           // First element: Option<&T>
v.last()                                            // Last element: Option<&T>
v.first_mut()                                       // Mutable first: Option<&mut T>
v.last_mut()                                        // Mutable last: Option<&mut T>

// ===== SLICING =====
let v = vec![1, 2, 3, 4, 5];

&v[..]                                              // Full slice
&v[1..4]                                            // Range: [2, 3, 4]
&v[..3]                                             // First 3: [1, 2, 3]
&v[2..]                                             // From index 2: [3, 4, 5]
&v[1..=3]                                           // Inclusive range: [2, 3, 4]

v.get(1..4)                                         // Safe slice: Option<&[T]>
v.get_mut(1..4)                                     // Mutable slice: Option<&mut [T]>

// ===== SLICE OPERATIONS =====
let slice = &[1, 2, 3, 4, 5];

slice.len()                                         // Length
slice.is_empty()                                    // Check if empty
slice.first()                                       // First element: Option<&T>
slice.last()                                        // Last element: Option<&T>

slice.split_first()                                 // (first, rest): Option<(&T, &[T])>
slice.split_last()                                  // (last, rest): Option<(&T, &[T])>

// Splitting
slice.split_at(2)                                   // Split at index: (&[T], &[T])
slice.split(|x| *x == 3)                           // Split by predicate: Iterator
slice.splitn(2, |x| *x == 3)                       // Split n times
slice.split_inclusive(|x| *x == 3)                 // Include delimiter in chunks
slice.rsplit(|x| *x == 3)                          // Split from right

// Chunks
slice.chunks(2)                                     // [[1,2], [3,4], [5]]
slice.chunks_exact(2)                              // [[1,2], [3,4]] (no remainder)
slice.rchunks(2)                                    // Chunks from right
slice.windows(3)                                    // [[1,2,3], [2,3,4], [3,4,5]]

// Mutable versions
let mut v = vec![1, 2, 3, 4, 5];
let slice = &mut v[..];
slice.split_at_mut(2)                              // Mutable split
slice.chunks_mut(2)                                // Mutable chunks
slice.split_first_mut()                            // Mutable split first

// ===== SEARCHING =====
let v = vec![1, 2, 3, 4, 5];

v.contains(&3)                                      // Check if contains value
v.binary_search(&3)                                // Binary search: Result<usize, usize>
v.binary_search_by(|x| x.cmp(&3))                 // Binary search with comparator
v.binary_search_by_key(&3, |x| *x)                // Binary search by key

v.starts_with(&[1, 2])                             // Check prefix
v.ends_with(&[4, 5])                               // Check suffix

// ===== SORTING =====
let mut v = vec![3, 1, 4, 1, 5, 9];

v.sort()                                            // Sort in place (stable)
v.sort_unstable()                                   // Unstable sort (faster)
v.sort_by(|a, b| a.cmp(b))                         // Sort with comparator
v.sort_by(|a, b| b.cmp(a))                         // Reverse sort
v.sort_by_key(|x| *x)                              // Sort by key function
v.sort_by_cached_key(|x| expensive(*x))           // Cache key computations

v.reverse()                                         // Reverse in place

// Check if sorted
v.is_sorted()                                       // Check if sorted
v.is_sorted_by(|a, b| a <= b)                     // Check with comparator
v.is_sorted_by_key(|x| *x)                        // Check by key

// ===== REORDERING =====
let mut v = vec![1, 2, 3, 4, 5];

v.swap(0, 4)                                        // Swap elements at indices
v.rotate_left(2)                                    // Rotate left: [3,4,5,1,2]
v.rotate_right(2)                                   // Rotate right
v.reverse()                                         // Reverse in place

// ===== FILLING =====
let mut v = vec![0; 5];

v.fill(42)                                          // Fill with value: [42,42,42,42,42]
v.fill_with(|| rand::random())                     // Fill with function

// ===== COPYING AND CLONING =====
let v = vec![1, 2, 3];

v.clone()                                           // Deep clone
v.to_vec()                                          // Clone to new Vec (for slices)

// Copy from slice
let mut dest = vec![0; 5];
let src = [1, 2, 3];
dest[..3].copy_from_slice(&src)                    // Copy exact slice
dest.clone_from_slice(&src)                        // Clone from slice

// ===== COMPARING =====
let v1 = vec![1, 2, 3];
let v2 = vec![1, 2, 3];

v1 == v2                                            // Equality
v1 != v2                                            // Inequality
v1 < v2                                             // Lexicographic comparison
v1.cmp(&v2)                                         // Ordering

// ===== ITERATION =====
let v = vec![1, 2, 3, 4, 5];

for x in &v {                                       // Borrow elements
    println!("{}", x);
}

for x in &mut v {                                   // Mutable borrow
    *x *= 2;
}

for x in v {                                        // Consume vector
    println!("{}", x);
}

// Iterator methods
v.iter()                                            // Iterator over &T
v.iter_mut()                                        // Iterator over &mut T
v.into_iter()                                       // Iterator over T (consume)

// ===== TRANSFORMING =====
let v = vec![1, 2, 3, 4, 5];

// Map to new vector
let doubled: Vec<_> = v.iter().map(|x| x * 2).collect();

// Filter
let evens: Vec<_> = v.iter().filter(|&&x| x % 2 == 0).collect();

// Partition
let (even, odd): (Vec<_>, Vec<_>) = v.iter()
    .partition(|&&x| x % 2 == 0);

// ===== JOINING AND SPLITTING =====
let v = vec![vec![1, 2], vec![3, 4], vec![5]];

// Flatten
let flat: Vec<_> = v.into_iter().flatten().collect(); // [1,2,3,4,5]
let flat: Vec<_> = v.concat();                     // Concatenate all

// Join with separator (for slices)
let v = vec![vec![1, 2], vec![3, 4]];
let joined = v.join(&0);                           // [1,2,0,3,4]

// ===== CONVERTING =====
let v = vec![1, 2, 3];

// To array (requires const generic)
let arr: [i32; 3] = v.try_into().unwrap();         // Vec to array
let arr: [i32; 3] = v[..3].try_into().unwrap();   // Slice to array

// To boxed slice
let boxed: Box<[i32]> = v.into_boxed_slice();      // Vec to Box<[T]>
let v: Vec<i32> = boxed.into_vec();                // Box<[T]> to Vec

// To string (for u8)
let bytes = vec![72, 101, 108, 108, 111];
let s = String::from_utf8(bytes).unwrap();         // Vec<u8> to String
let bytes = s.into_bytes();                        // String to Vec<u8>

// ===== SPECIAL OPERATIONS =====
let mut v = vec![1, 2, 3, 4, 5];

// Splice - replace range
v.splice(1..3, vec![10, 20]);                      // Replace [2,3] with [10,20]

// Leak - get static slice
let leak: &'static [i32] = v.leak();               // Leak memory, get static ref

// Resize
v.resize(10, 0)                                     // Resize to length 10, fill with 0
v.resize_with(10, Default::default)                // Resize with function

// Split off
let mut v = vec![1, 2, 3, 4, 5];
let v2 = v.split_off(3);                           // Split at index: v=[1,2,3], v2=[4,5]

// ===== RAW POINTERS =====
let v = vec![1, 2, 3];

v.as_ptr()                                          // Get raw pointer: *const T
v.as_mut_ptr()                                      // Get mutable raw pointer: *mut T

// Construct from raw parts (unsafe)
let mut v = vec![1, 2, 3];
let ptr = v.as_mut_ptr();
let len = v.len();
let cap = v.capacity();
std::mem::forget(v);
let v = unsafe { Vec::from_raw_parts(ptr, len, cap) };

// ===== COMMON PATTERNS =====
// Pattern 1: Remove duplicates (requires sorting)
let mut v = vec![1, 2, 2, 3, 3, 3, 4];
v.sort();
v.dedup();                                          // [1, 2, 3, 4]

// Pattern 2: Remove duplicates (preserve order)
use std::collections::HashSet;
let v = vec![1, 2, 2, 3, 1];
let unique: Vec<_> = v.into_iter()
    .collect::<HashSet<_>>()
    .into_iter()
    .collect();

// Pattern 3: Find and remove
let mut v = vec![1, 2, 3, 4, 5];
if let Some(pos) = v.iter().position(|&x| x == 3) {
    v.remove(pos);
}

// Pattern 4: Swap remove multiple
let mut v = vec![1, 2, 3, 4, 5];
let indices = vec![1, 3];
for &i in indices.iter().rev() {                   // Remove from back to preserve indices
    v.swap_remove(i);
}

// Pattern 5: Group consecutive
let v = vec![1, 1, 2, 2, 2, 3];
let groups: Vec<Vec<_>> = v.chunk_by(|a, b| a == b)
    .map(|chunk| chunk.to_vec())
    .collect();

// Pattern 6: Sliding window processing
let v = vec![1, 2, 3, 4, 5];
for window in v.windows(3) {
    let sum: i32 = window.iter().sum();
    println!("{:?} -> {}", window, sum);
}

// Pattern 7: Matrix (Vec<Vec<T>>)
let matrix: Vec<Vec<i32>> = vec![
    vec![1, 2, 3],
    vec![4, 5, 6],
];
let element = matrix[0][1];                        // 2

// Pattern 8: Flatten matrix
let flat: Vec<_> = matrix.into_iter().flatten().collect();

// Pattern 9: Transpose matrix
fn transpose<T: Clone>(matrix: Vec<Vec<T>>) -> Vec<Vec<T>> {
    if matrix.is_empty() { return vec![]; }
    let rows = matrix.len();
    let cols = matrix[0].len();
    (0..cols)
        .map(|col| (0..rows).map(|row| matrix[row][col].clone()).collect())
        .collect()
}

// Pattern 10: Ring buffer using rotate
let mut buffer = vec![1, 2, 3, 4, 5];
buffer.rotate_left(1);                             // [2,3,4,5,1]
buffer[4] = 6;                                      // [2,3,4,5,6]

// Pattern 11: Conditional push
let mut v = vec![];
let value = 42;
if condition {
    v.push(value);
}

// Pattern 12: Extend conditionally
let mut v = vec![1, 2, 3];
if condition {
    v.extend([4, 5, 6]);
}

// Pattern 13: Remove while iterating (drain_filter)
let mut v = vec![1, 2, 3, 4, 5];
let removed: Vec<_> = v.extract_if(|x| *x % 2 == 0).collect(); // Nightly

// Pattern 14: Safe index with get
let v = vec![1, 2, 3];
match v.get(index) {
    Some(&value) => println!("Found: {}", value),
    None => println!("Index out of bounds"),
}

// Pattern 15: Batch operations
let mut v = vec![1, 2, 3, 4, 5, 6];
for chunk in v.chunks_mut(2) {
    chunk[0] *= 10;
}
```
