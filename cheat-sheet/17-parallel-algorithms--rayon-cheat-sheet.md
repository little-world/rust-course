### Rayon Cheat Sheet
```rust
// Basic parallel iterators
use rayon::prelude::*;

// Parallel iteration
vec.par_iter()                                       // Parallel immutable iterator
vec.par_iter_mut()                                   // Parallel mutable iterator
vec.into_par_iter()                                  // Parallel consuming iterator
(0..n).into_par_iter()                              // Parallel range iterator

// Map operations
vec.par_iter().map(|x| x * 2).collect()             // Parallel map
vec.par_iter_mut().for_each(|x| *x *= 2)           // Parallel mutation
vec.par_iter().map_with(init, |state, x| work)     // Map with thread-local state
vec.par_iter().map_init(|| init, |state, x| work)  // Map with lazy init state

// Filter operations
vec.par_iter().filter(|x| condition).collect()      // Parallel filter
vec.par_iter().filter_map(|x| option).collect()     // Filter and map

// Reduce operations
vec.par_iter().sum::<i32>()                         // Parallel sum
vec.par_iter().product::<i32>()                     // Parallel product
vec.par_iter().min()                                 // Parallel min
vec.par_iter().max()                                 // Parallel max
vec.par_iter().reduce(|| init, |a, b| combine)      // Custom reduce
vec.par_iter().reduce_with(|a, b| combine)          // Reduce without identity

// Fold operations
vec.par_iter().fold(|| init, |acc, x| update)       // Parallel fold with init
    .reduce(|| init, |a, b| combine)                 // Combine fold results
vec.par_iter().fold_with(init, |acc, x| update)     // Fold with cloned init

// Find operations
vec.par_iter().find_any(|x| condition)              // Find any matching (non-deterministic)
vec.par_iter().find_first(|x| condition)            // Find first matching (deterministic)
vec.par_iter().find_last(|x| condition)             // Find last matching
vec.par_iter().position_any(|x| condition)          // Find position of any match
vec.par_iter().position_first(|x| condition)        // Find position of first match

// Boolean operations
vec.par_iter().all(|x| condition)                   // Check if all match
vec.par_iter().any(|x| condition)                   // Check if any match

// Count operations
vec.par_iter().count()                               // Count elements
vec.par_iter().filter(|x| cond).count()             // Count matching

// Collect operations
vec.par_iter().collect::<Vec<_>>()                  // Collect into Vec
vec.par_iter().collect::<HashSet<_>>()              // Collect into HashSet
vec.par_iter().unzip::<_, _, Vec<_>, Vec<_>>()      // Unzip into two collections

// Partition operations
vec.par_iter().partition::<Vec<_>, _>(|x| cond)     // Partition into two Vecs
vec.par_iter().partition_map(|x| either)            // Partition with mapping

// Chunking
vec.par_chunks(size)                                 // Parallel iteration over chunks
vec.par_chunks_mut(size)                            // Parallel mutable chunks
vec.par_windows(size)                               // Parallel sliding windows

// Sorting
vec.par_sort()                                       // Parallel sort (unstable)
vec.par_sort_unstable()                             // Parallel unstable sort
vec.par_sort_by(|a, b| a.cmp(b))                    // Sort with comparator
vec.par_sort_by_key(|x| x.field)                    // Sort by key function

// Split operations
vec.par_split(|x| condition)                        // Split by predicate
vec.par_split_mut(|x| condition)                    // Split mutably

// Enumerate
vec.par_iter().enumerate()                          // Parallel enumerate (index, item)

// Zip
vec1.par_iter().zip(vec2.par_iter())               // Parallel zip

// Chain
iter1.par_chain(iter2)                              // Chain two parallel iterators

// Flat map
vec.par_iter().flat_map(|x| inner_iter)            // Parallel flat map
vec.par_iter().flat_map_iter(|x| sequential_iter)  // Flat map with sequential inner

// Interleave
iter1.par_interleave(iter2)                         // Interleave two iterators

// Take/skip
vec.par_iter().take_any(n)                          // Take n elements (non-deterministic)
vec.par_iter().skip_any(n)                          // Skip n elements (non-deterministic)

// Update
vec.par_iter().update(|x| *x *= 2)                 // Update in place

// Inspect
vec.par_iter().inspect(|x| println!("{}", x))      // Inspect elements

// Cloned/copied
vec.par_iter().cloned()                             // Clone elements
vec.par_iter().copied()                             // Copy elements

// While operations
vec.par_iter().take_any_while(|x| cond)            // Take while condition
vec.par_iter().skip_any_while(|x| cond)            // Skip while condition
```
