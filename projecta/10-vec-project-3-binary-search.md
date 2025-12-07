## Project 3: Binary Search and Sorted Data Structures

### Problem Statement

Build efficient search and query systems leveraging binary search on sorted data. Implement various binary search variants (exact match, lower bound, upper bound, range queries) and create data structures that maintain sorted invariants for O(log n) operations.

Your project should include:
- Generic binary search implementation (exact, lower_bound, upper_bound)
- Database-like range queries on sorted data
- Auto-complete / prefix matching with binary search
- Efficient merging of sorted sequences (k-way merge)
- Maintaining sorted invariants for incremental updates
- Performance comparisons with linear search and hash-based approaches

Example use case:
```
Sorted log entries by timestamp (1M entries)
Query: Find all logs between 10:00 and 10:05
Linear scan: O(n) = 1M comparisons
Binary search range: O(log n + k) = 20 comparisons + k results
Speedup: 50,000x for k=1000 results
```

### Why It Matters

Binary search is one of the most fundamental algorithms: O(log n) vs O(n) is the difference between 20 operations and 1,000,000 operations for n=1M. Many production systems rely on sorted data: databases (B-trees), file systems, network routing tables, autocomplete systems.

Understanding binary search variants enables building efficient query systems without heavy database dependencies.

---

### Milestone 1: Implement Binary Search Variants

**Goal**: Implement exact match, lower_bound, upper_bound binary searches.

**What to implement**:
- `binary_search_exact()`: Find exact match, return index
- `binary_search_lower_bound()`: Find first element >= target
- `binary_search_upper_bound()`: Find first element > target
- Generic implementations that work with any ordered type

**Architecture**:
- Functions:
  - `binary_search_exact<T: Ord>(arr: &[T], target: &T) -> Option<usize>` - Exact match
  - `binary_search_lower_bound<T: Ord>(arr: &[T], target: &T) -> usize` - Lower bound
  - `binary_search_upper_bound<T: Ord>(arr: &[T], target: &T) -> usize` - Upper bound

---

**Starter Code**:

```rust
use std::cmp::Ordering;

/// Binary search for exact match
/// Role: O(log n) exact search
pub fn binary_search_exact<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
    todo!("Implement binary search with left/right pointers")
}

/// Binary search for lower bound
/// Role: Range query start point
pub fn binary_search_lower_bound<T: Ord>(arr: &[T], target: &T) -> usize {
    todo!("Find leftmost position where arr[i] >= target")
}

/// Binary search for upper bound
/// Role: Range query end point
pub fn binary_search_upper_bound<T: Ord>(arr: &[T], target: &T) -> usize {
    todo!("Find leftmost position where arr[i] > target")
}

/// Helper: Check if array is sorted
/// Role: Validate precondition
pub fn is_sorted<T: Ord>(arr: &[T]) -> bool {
    todo!("Check arr[i] <= arr[i+1] for all i")
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_search_found() {
        let arr = vec![1, 3, 5, 7, 9, 11, 13];

        assert_eq!(binary_search_exact(&arr, &5), Some(2));
        assert_eq!(binary_search_exact(&arr, &1), Some(0));
        assert_eq!(binary_search_exact(&arr, &13), Some(6));
    }

    #[test]
    fn test_exact_search_not_found() {
        let arr = vec![1, 3, 5, 7, 9];

        assert_eq!(binary_search_exact(&arr, &2), None);
        assert_eq!(binary_search_exact(&arr, &0), None);
        assert_eq!(binary_search_exact(&arr, &10), None);
    }

    #[test]
    fn test_exact_search_empty() {
        let arr: Vec<i32> = vec![];
        assert_eq!(binary_search_exact(&arr, &5), None);
    }

    #[test]
    fn test_exact_search_duplicates() {
        let arr = vec![1, 3, 3, 3, 5, 7];

        // Should find one of the 3's (any is valid)
        let result = binary_search_exact(&arr, &3);
        assert!(result.is_some());
        assert_eq!(arr[result.unwrap()], 3);
    }

    #[test]
    fn test_lower_bound() {
        let arr = vec![1, 3, 5, 7, 9];

        assert_eq!(binary_search_lower_bound(&arr, &5), 2); // Exact match
        assert_eq!(binary_search_lower_bound(&arr, &4), 2); // Between 3 and 5
        assert_eq!(binary_search_lower_bound(&arr, &0), 0); // Before all
        assert_eq!(binary_search_lower_bound(&arr, &10), 5); // After all
    }

    #[test]
    fn test_lower_bound_duplicates() {
        let arr = vec![1, 3, 3, 3, 5, 7];

        // Should return first 3
        assert_eq!(binary_search_lower_bound(&arr, &3), 1);
    }

    #[test]
    fn test_upper_bound() {
        let arr = vec![1, 3, 5, 7, 9];

        assert_eq!(binary_search_upper_bound(&arr, &5), 3); // After 5
        assert_eq!(binary_search_upper_bound(&arr, &4), 2); // Between 3 and 5
        assert_eq!(binary_search_upper_bound(&arr, &0), 0); // Before all
        assert_eq!(binary_search_upper_bound(&arr, &9), 5); // After all
    }

    #[test]
    fn test_upper_bound_duplicates() {
        let arr = vec![1, 3, 3, 3, 5, 7];

        // Should return index after last 3
        assert_eq!(binary_search_upper_bound(&arr, &3), 4);
    }

    #[test]
    fn test_bounds_with_strings() {
        let arr = vec!["apple", "banana", "cherry", "date"];

        assert_eq!(binary_search_lower_bound(&arr, &"banana"), 1);
        assert_eq!(binary_search_upper_bound(&arr, &"banana"), 2);
    }

    #[test]
    fn test_is_sorted() {
        assert!(is_sorted(&[1, 2, 3, 4, 5]));
        assert!(is_sorted(&[1, 1, 2, 3])); // Duplicates OK
        assert!(!is_sorted(&[1, 3, 2, 4]));
        assert!(is_sorted(&Vec::<i32>::new())); // Empty is sorted
    }
}
```

---

### Milestone 2: Range Queries with Binary Search

**Goal**: Implement efficient range queries: find all elements in [start, end].

**Why the previous milestone is not enough**: Single element lookup is useful, but range queries are essential for time-series, databases, and filtering operations.

**What's the improvement**: Range queries using two binary searches are O(log n + k) where k is result size. Naive linear scan is O(n). For finding 100 elements in 1M element array:
- Linear scan: ~1,000,000 comparisons
- Binary search range: ~40 comparisons + 100 results

This is a 10,000x speedup for the search phase.

**Optimization focus**: Speed through binary search (O(n) → O(log n + k)).

**Architecture**:
- Functions:
  - `range_query<T: Ord>(arr: &[T], start: &T, end: &T) -> &[T]` - Get slice in range
  - `count_in_range<T: Ord>(arr: &[T], start: &T, end: &T) -> usize` - Count without materializing
  - Example types: `LogEntry` with timestamp ordering

---

**Starter Code**:

```rust
/// Range query on sorted array
/// Role: Zero-copy range extraction
pub fn range_query<T: Ord>(arr: &[T], start: &T, end: &T) -> &[T] {
    todo!("Use lower_bound(start) and upper_bound(end)")
}

/// Count elements in range
/// Role: Efficient counting
pub fn count_in_range<T: Ord>(arr: &[T], start: &T, end: &T) -> usize {
    todo!("Return upper_bound(end) - lower_bound(start)")
}

/// Log entry with timestamp
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub timestamp: u64,             // Unix timestamp                 
    pub level: String,              // Log level (INFO, ERROR, etc.)   
    pub message: String,            // Log message                   
}

impl Ord for LogEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl PartialOrd for LogEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Query logs by time range
/// Role: Time-series query
pub fn query_logs_by_time(logs: &[LogEntry], start_time: u64, end_time: u64) -> &[LogEntry] {
    todo!("Create dummy entries for bounds, use range_query")
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_query_basic() {
        let arr = vec![1, 3, 5, 7, 9, 11, 13, 15];

        let result = range_query(&arr, &5, &11);
        assert_eq!(result, &[5, 7, 9, 11]);
    }

    #[test]
    fn test_range_query_empty() {
        let arr = vec![1, 3, 5, 7, 9];

        let result = range_query(&arr, &20, &30);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_range_query_all() {
        let arr = vec![1, 3, 5, 7, 9];

        let result = range_query(&arr, &0, &10);
        assert_eq!(result, &arr[..]);
    }

    #[test]
    fn test_count_in_range() {
        let arr = vec![1, 3, 5, 7, 9, 11, 13, 15];

        assert_eq!(count_in_range(&arr, &5, &11), 4); // 5, 7, 9, 11
        assert_eq!(count_in_range(&arr, &0, &20), 8); // All
        assert_eq!(count_in_range(&arr, &20, &30), 0); // None
    }

    #[test]
    fn test_range_query_duplicates() {
        let arr = vec![1, 3, 3, 3, 5, 7, 7, 9];

        let result = range_query(&arr, &3, &7);
        assert_eq!(result, &[3, 3, 3, 5, 7, 7]);
    }

    #[test]
    fn test_log_entry_ordering() {
        let log1 = LogEntry {
            timestamp: 100,
            level: "INFO".to_string(),
            message: "Message 1".to_string(),
        };

        let log2 = LogEntry {
            timestamp: 200,
            level: "ERROR".to_string(),
            message: "Message 2".to_string(),
        };

        assert!(log1 < log2);
    }

    #[test]
    fn test_query_logs_by_time() {
        let logs = vec![
            LogEntry { timestamp: 100, level: "INFO".to_string(), message: "Msg 1".to_string() },
            LogEntry { timestamp: 200, level: "INFO".to_string(), message: "Msg 2".to_string() },
            LogEntry { timestamp: 300, level: "ERROR".to_string(), message: "Msg 3".to_string() },
            LogEntry { timestamp: 400, level: "INFO".to_string(), message: "Msg 4".to_string() },
        ];

        let result = query_logs_by_time(&logs, 200, 300);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].timestamp, 200);
        assert_eq!(result[1].timestamp, 300);
    }

    #[test]
    fn test_range_performance_vs_linear() {
        use std::time::Instant;

        let arr: Vec<i32> = (0..1_000_000).collect();

        // Binary search range query
        let start = Instant::now();
        let result1 = range_query(&arr, &100_000, &100_100);
        let binary_time = start.elapsed();

        // Linear scan
        let start = Instant::now();
        let result2: Vec<&i32> = arr.iter()
            .filter(|&&x| x >= 100_000 && x <= 100_100)
            .collect();
        let linear_time = start.elapsed();

        assert_eq!(result1.len(), result2.len());

        println!("Binary search: {:?}", binary_time);
        println!("Linear scan: {:?}", linear_time);

        // Binary search should be dramatically faster
        assert!(binary_time < linear_time);
    }
}
```

---

### Milestone 3: Auto-Complete with Prefix Matching

**Goal**: Implement auto-complete using binary search on sorted strings.

**Why the previous milestone is not enough**: Exact and range queries work for known values, but prefix matching is needed for search, auto-complete, and fuzzy finding.

**What's the improvement**: Binary search + prefix scan is O(log n + k) where k is matches. Building a trie would be O(n) space and complex. For moderate-sized dictionaries (10K-1M words), sorted array + binary search is simpler and faster.

**Optimization focus**: Simplicity and speed for moderate datasets.

**Architecture**:
- Structs: `AutoComplete`
- Functions:
  - `prefix_search<'a>(words: &'a [String], prefix: &str) -> &'a [String]` - Find prefix matches
  - `AutoComplete::new(words: Vec<String>) -> Self` - Create with sorted words
  - `AutoComplete::suggest(&self, prefix: &str) -> Vec<&str>` - Get suggestions

---

**Starter Code**:

```rust
/// Find all strings with given prefix
/// Role: Efficient prefix matching
pub fn prefix_search<'a>(words: &'a [String], prefix: &str) -> &'a [String] {
    todo!("Use partition_point to find start, scan while prefix matches")
}

/// Auto-complete system
/// Role: Fast prefix suggestions
#[derive(Debug)]
pub struct AutoComplete {
    words: Vec<String>,                 // Sorted, deduplicated words 
}

impl AutoComplete {
    /// Create auto-complete with word list
    /// Role: Initialize and sort
    pub fn new(mut words: Vec<String>) -> Self {
        todo!("Sort and deduplicate words")
    }

    /// Get suggestions for prefix
    /// Role: Return top N matches
    pub fn suggest(&self, prefix: &str) -> Vec<&str> {
        todo!("Use prefix_search, take top 10")
    }

    /// Get all matches (no limit)
    /// Role: Complete result set
    pub fn suggest_all(&self, prefix: &str) -> Vec<&str> {
        todo!("Return all prefix matches")
    }

    /// Get word count
    /// Role: Query dictionary size
    pub fn word_count(&self) -> usize {
        self.words.len()
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_search_basic() {
        let words = vec![
            "apple".to_string(),
            "application".to_string(),
            "apply".to_string(),
            "banana".to_string(),
            "band".to_string(),
        ];

        let result = prefix_search(&words, "app");
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"apple".to_string()));
        assert!(result.contains(&"application".to_string()));
        assert!(result.contains(&"apply".to_string()));
    }

    #[test]
    fn test_prefix_search_empty_prefix() {
        let words = vec!["apple".to_string(), "banana".to_string()];

        let result = prefix_search(&words, "");
        assert_eq!(result.len(), 2); // All words
    }

    #[test]
    fn test_prefix_search_no_matches() {
        let words = vec!["apple".to_string(), "banana".to_string()];

        let result = prefix_search(&words, "xyz");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_autocomplete_creation() {
        let words = vec![
            "banana".to_string(),
            "apple".to_string(),
            "apple".to_string(), // Duplicate
            "cherry".to_string(),
        ];

        let ac = AutoComplete::new(words);

        // Should be sorted and deduplicated
        assert_eq!(ac.word_count(), 3);
    }

    #[test]
    fn test_autocomplete_suggestions() {
        let words = vec![
            "apple".to_string(),
            "application".to_string(),
            "apply".to_string(),
            "appreciate".to_string(),
            "banana".to_string(),
        ];

        let ac = AutoComplete::new(words);
        let suggestions = ac.suggest("app");

        assert!(suggestions.len() > 0);
        assert!(suggestions.len() <= 10); // Limited to 10
    }

    #[test]
    fn test_autocomplete_suggest_all() {
        let words = vec![
            "test1".to_string(),
            "test2".to_string(),
            "test3".to_string(),
            "other".to_string(),
        ];

        let ac = AutoComplete::new(words);
        let all_suggestions = ac.suggest_all("test");

        assert_eq!(all_suggestions.len(), 3);
    }

    #[test]
    fn test_autocomplete_case_sensitive() {
        let words = vec![
            "Apple".to_string(),
            "apple".to_string(),
            "APPLE".to_string(),
        ];

        let ac = AutoComplete::new(words);

        // Should treat as different words
        assert_eq!(ac.word_count(), 3);
    }

    #[test]
    fn test_autocomplete_performance() {
        use std::time::Instant;

        // Create large dictionary
        let words: Vec<String> = (0..100_000)
            .map(|i| format!("word{:06}", i))
            .collect();

        let ac = AutoComplete::new(words);

        // Benchmark suggestions
        let start = Instant::now();

        for _ in 0..1000 {
            let _ = ac.suggest("word1");
        }

        let elapsed = start.elapsed();

        println!("Time for 1000 lookups: {:?}", elapsed);

        // Should be very fast
        assert!(elapsed.as_millis() < 100);
    }

    #[test]
    fn test_autocomplete_real_world() {
        let words = vec![
            "javascript".to_string(),
            "java".to_string(),
            "python".to_string(),
            "rust".to_string(),
            "ruby".to_string(),
            "go".to_string(),
        ];

        let ac = AutoComplete::new(words);

        assert_eq!(ac.suggest("ja").len(), 2); // java, javascript
        assert_eq!(ac.suggest("r").len(), 2);  // ruby, rust
        assert_eq!(ac.suggest("xyz").len(), 0); // No matches
    }
}
```

---

### Milestone 4: Merge Sorted Sequences (K-Way Merge)

**Goal**: Efficiently merge multiple sorted sequences.

**Why the previous milestone is not enough**: Individual sorted sequences are useful, but often we need to combine multiple sources (log files, database shards, sorted chunks).

**What's the improvement**: K-way merge with heap is O(n log k) where n is total elements, k is number of sequences. Repeated 2-way merge is O(nk). For k=100:
- Repeated 2-way: 100× slower
- K-way with heap: Optimal

**Optimization focus**: Speed through better algorithm.

**Architecture**:
- Functions:
  - `merge_two<T: Ord + Clone>(left: &[T], right: &[T]) -> Vec<T>` - Two-way merge
  - `merge_k<T: Ord + Clone>(sequences: &[&[T]]) -> Vec<T>` - K-way merge with heap

---

**Starter Code**:

```rust
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Merge two sorted slices
/// Role: Building block for merge sort
pub fn merge_two<T: Ord + Clone>(left: &[T], right: &[T]) -> Vec<T> {
    todo!("Two-pointer merge algorithm")
}

/// Merge K sorted sequences using heap
/// Role: Combine multiple sorted sources
pub fn merge_k<T: Ord + Clone>(sequences: &[&[T]]) -> Vec<T> {
    todo!("Use BinaryHeap with (value, seq_index, elem_index)")
}

/// Merge iterator (lazy evaluation)
/// Role: Zero-allocation merging
pub struct MergeIterator<'a, T> {
    sequences: Vec<&'a [T]>,
    indices: Vec<usize>,
    heap: BinaryHeap<Reverse<(T, usize)>>,
}

impl<'a, T: Ord + Clone> MergeIterator<'a, T> {
    /// Create merge iterator
    /// Role: Initialize heap with first elements
    pub fn new(sequences: Vec<&'a [T]>) -> Self {
        todo!("Initialize heap, indices")
    }
}

impl<'a, T: Ord + Clone> Iterator for MergeIterator<'a, T> {
    type Item = T;

    /// Get next merged element
    /// Role: Lazy merging
    fn next(&mut self) -> Option<Self::Item> {
        todo!("Pop from heap, push next from same sequence")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_two_basic() {
        let left = vec![1, 3, 5];
        let right = vec![2, 4, 6];

        let result = merge_two(&left, &right);

        assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_merge_two_empty() {
        let left = vec![1, 2, 3];
        let right: Vec<i32> = vec![];

        let result = merge_two(&left, &right);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_merge_two_overlapping() {
        let left = vec![1, 5, 9];
        let right = vec![3, 7, 11];

        let result = merge_two(&left, &right);
        assert_eq!(result, vec![1, 3, 5, 7, 9, 11]);
    }

    #[test]
    fn test_merge_k_basic() {
        let seq1 = vec![1, 4, 7];
        let seq2 = vec![2, 5, 8];
        let seq3 = vec![3, 6, 9];

        let sequences = vec![&seq1[..], &seq2[..], &seq3[..]];
        let result = merge_k(&sequences);

        assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_merge_k_different_lengths() {
        let seq1 = vec![1, 2];
        let seq2 = vec![3, 4, 5, 6];
        let seq3 = vec![7];

        let sequences = vec![&seq1[..], &seq2[..], &seq3[..]];
        let result = merge_k(&sequences);

        assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_merge_k_with_duplicates() {
        let seq1 = vec![1, 3, 5];
        let seq2 = vec![1, 3, 5];

        let sequences = vec![&seq1[..], &seq2[..]];
        let result = merge_k(&sequences);

        assert_eq!(result, vec![1, 1, 3, 3, 5, 5]);
    }

    #[test]
    fn test_merge_k_single_sequence() {
        let seq1 = vec![1, 2, 3];

        let sequences = vec![&seq1[..]];
        let result = merge_k(&sequences);

        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_merge_k_empty_sequences() {
        let empty: Vec<i32> = vec![];
        let sequences: Vec<&[i32]> = vec![&empty];

        let result = merge_k(&sequences);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_merge_performance() {
        use std::time::Instant;

        // Create 10 sorted sequences of 10000 elements each
        let sequences: Vec<Vec<i32>> = (0..10)
            .map(|i| (i..100000).step_by(10).collect())
            .collect();

        let seq_refs: Vec<&[i32]> = sequences.iter().map(|v| v.as_slice()).collect();

        // K-way merge
        let start = Instant::now();
        let result_k = merge_k(&seq_refs);
        let k_way_time = start.elapsed();

        // Repeated 2-way merge
        let start = Instant::now();
        let mut result_2way = sequences[0].clone();
        for seq in &sequences[1..] {
            result_2way = merge_two(&result_2way, seq);
        }
        let two_way_time = start.elapsed();

        println!("K-way merge: {:?}", k_way_time);
        println!("Repeated 2-way: {:?}", two_way_time);

        assert_eq!(result_k.len(), result_2way.len());

        // K-way should be faster
        assert!(k_way_time < two_way_time);
    }

    #[test]
    fn test_merge_iterator() {
        let seq1 = vec![1, 4, 7];
        let seq2 = vec![2, 5, 8];
        let seq3 = vec![3, 6, 9];

        let sequences = vec![&seq1[..], &seq2[..], &seq3[..]];
        let iter = MergeIterator::new(sequences);

        let result: Vec<i32> = iter.collect();

        assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
}
```

---

### Milestone 5: Sorted Set with Incremental Updates

**Goal**: Maintain sorted collection with efficient insert/remove/search.

**Why the previous milestone is not enough**: Static sorted arrays are fast for queries but can't handle updates. Need dynamic sorted collection.

**What's the improvement**: Binary search for insertion point gives O(log n) search + O(n) shift. Still faster than hash table for small sets (<1000 elements) due to cache locality. Provides range queries and ordering that hash tables don't support.

**Optimization focus**: When to use SortedVec vs BTreeSet vs HashSet.

**Architecture**:
- Structs: `SortedVec<T>`
- Fields: `data: Vec<T>`
- Functions:
  - `new() -> Self` - Create empty set
  - `insert(value: T) -> bool` - Add maintaining order
  - `remove(value: &T) -> bool` - Remove if present
  - `contains(value: &T) -> bool` - O(log n) search
  - `range(start: &T, end: &T) -> &[T]` - Range query

---

**Starter Code**:

```rust
/// Sorted vector maintaining order invariant
/// Role: Efficient sorted set for small-medium collections
#[derive(Debug, Clone)]
pub struct SortedVec<T> {
    data: Vec<T>,                  // : Ordered set using Vec    
}

impl<T: Ord> SortedVec<T> {
    /// Create empty sorted vec
    /// Role: Initialize
    pub fn new() -> Self {
        todo!("Create empty Vec")
    }

    /// Insert value maintaining order
    /// Role: O(log n) search + O(n) insert
    pub fn insert(&mut self, value: T) -> bool {
        todo!("Binary search position, insert if not present")
    }

    /// Remove value if present
    /// Role: O(log n) search + O(n) remove
    pub fn remove(&mut self, value: &T) -> bool {
        todo!("Binary search, remove if found")
    }

    /// Check if contains value
    /// Role: O(log n) membership test
    pub fn contains(&self, value: &T) -> bool {
        todo!("Use binary_search")
    }

    /// Get range of values
    /// Role: Range query support
    pub fn range(&self, start: &T, end: &T) -> &[T] {
        todo!("Use range_query helper")
    }

    /// Get length
    /// Role: Query size
    pub fn len(&self) -> usize {
        todo!()
    }

    /// Check if empty
    /// Role: Query emptiness
    pub fn is_empty(&self) -> bool {
       todo!()
    }

    /// Get all elements as slice
    /// Role: Zero-copy access
    pub fn as_slice(&self) -> &[T] {
          todo!()
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeSet, HashSet};
    use std::time::Instant;

    #[test]
    fn test_sorted_vec_insert() {
        let mut sv = SortedVec::new();

        assert!(sv.insert(5));
        assert!(sv.insert(3));
        assert!(sv.insert(7));
        assert!(sv.insert(1));

        assert_eq!(sv.as_slice(), &[1, 3, 5, 7]);
    }

    #[test]
    fn test_sorted_vec_insert_duplicate() {
        let mut sv = SortedVec::new();

        assert!(sv.insert(5));
        assert!(!sv.insert(5)); // Duplicate

        assert_eq!(sv.len(), 1);
    }

    #[test]
    fn test_sorted_vec_remove() {
        let mut sv = SortedVec::new();

        sv.insert(1);
        sv.insert(3);
        sv.insert(5);

        assert!(sv.remove(&3));
        assert_eq!(sv.as_slice(), &[1, 5]);

        assert!(!sv.remove(&10)); // Not present
    }

    #[test]
    fn test_sorted_vec_contains() {
        let mut sv = SortedVec::new();

        sv.insert(1);
        sv.insert(3);
        sv.insert(5);

        assert!(sv.contains(&3));
        assert!(!sv.contains(&4));
    }

    #[test]
    fn test_sorted_vec_range() {
        let mut sv = SortedVec::new();

        for i in vec![1, 3, 5, 7, 9, 11, 13] {
            sv.insert(i);
        }

        let range = sv.range(&5, &11);
        assert_eq!(range, &[5, 7, 9, 11]);
    }

    #[test]
    fn test_sorted_vec_maintains_order() {
        let mut sv = SortedVec::new();

        // Insert in random order
        for i in vec![9, 3, 7, 1, 5] {
            sv.insert(i);
        }

        // Should be sorted
        assert_eq!(sv.as_slice(), &[1, 3, 5, 7, 9]);
    }

    #[test]
    fn test_benchmark_vs_btreeset() {
        let n = 1000;

        // SortedVec
        let mut sv = SortedVec::new();
        let start = Instant::now();
        for i in 0..n {
            sv.insert(i);
        }
        let sv_insert_time = start.elapsed();

        // BTreeSet
        let mut btree = BTreeSet::new();
        let start = Instant::now();
        for i in 0..n {
            btree.insert(i);
        }
        let btree_insert_time = start.elapsed();

        println!("SortedVec insert (n={}): {:?}", n, sv_insert_time);
        println!("BTreeSet insert (n={}): {:?}", n, btree_insert_time);

        // For small n, SortedVec might be competitive
        // For large n, BTreeSet should win
    }

    #[test]
    fn test_benchmark_vs_hashset() {
        let n = 1000;

        // SortedVec
        let mut sv = SortedVec::new();
        let start = Instant::now();
        for i in 0..n {
            sv.insert(i);
        }
        let sv_time = start.elapsed();

        // HashSet
        let mut hs = HashSet::new();
        let start = Instant::now();
        for i in 0..n {
            hs.insert(i);
        }
        let hs_time = start.elapsed();

        println!("SortedVec: {:?}", sv_time);
        println!("HashSet: {:?}", hs_time);

        // HashSet should be faster for insertion
        // But SortedVec provides ordering
    }

    #[test]
    fn test_sorted_vec_use_case() {
        // Use case: Maintain sorted list of active user IDs
        let mut active_users = SortedVec::new();

        active_users.insert(101);
        active_users.insert(105);
        active_users.insert(103);

        // Get users in range
        let users_100_to_104 = active_users.range(&100, &104);
        assert_eq!(users_100_to_104, &[101, 103]);

        // Remove user
        active_users.remove(&103);

        // Check membership
        assert!(!active_users.contains(&103));
        assert!(active_users.contains(&105));
    }
}
```

---

### Milestone 6: Performance Optimization and Trade-offs

**Goal**: Understand when to use different data structures and optimize critical paths.

**Why the previous milestone is not enough**: Having implementations is good, but understanding trade-offs is essential for making the right choice in production.

**What's the improvement**: This milestone focuses on measurement, comparison, and decision-making:
- SortedVec: Best for <1K elements, cache-friendly, supports ranges
- BTreeSet: Best for >1K elements, O(log n) all operations
- HashSet: Best for membership only, no ordering

**Optimization focus**: Making informed architectural decisions.

**Architecture**:
- Benchmarks and comparisons
- Trade-off analysis
- Memory usage measurements

---

**Starter Code**:

```rust
/// Benchmark framework for collection comparisons
/// Role: Compare data structures
pub struct CollectionBenchmark {
    sizes: Vec<usize>,
}

impl CollectionBenchmark {
    /// Create benchmark suite
    /// Role: Initialize test sizes
    pub fn new(sizes: Vec<usize>) -> Self {
        todo!("Store sizes to test")
    }

    /// Benchmark insertions
    /// Role: Measure insert performance
    pub fn benchmark_inserts(&self) {
        todo!("Test SortedVec, BTreeSet, HashSet insertions")
    }

    /// Benchmark lookups
    /// Role: Measure search performance
    pub fn benchmark_lookups(&self) {
        todo!("Test contains() performance")
    }

    /// Benchmark range queries
    /// Role: Measure range performance
    pub fn benchmark_ranges(&self) {
        todo!("Test range queries (SortedVec vs BTreeSet)")
    }

    /// Memory usage comparison
    /// Role: Measure space efficiency
    pub fn measure_memory(&self) {
        todo!("Estimate memory overhead")
    }

    /// Generate report
    /// Role: Summary of findings
    pub fn generate_report(&self) {
        todo!("Print comparison table")
    }
}

/// Trade-off analysis
/// Role: Decision support
pub fn recommend_collection(
    size: usize,
    needs_ordering: bool,
    needs_ranges: bool,
    write_heavy: bool,
) -> &'static str {
    todo!("Return recommendation based on requirements")
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommendation_small_ordered() {
        let rec = recommend_collection(100, true, true, false);
        assert_eq!(rec, "SortedVec");
    }

    #[test]
    fn test_recommendation_large_ordered() {
        let rec = recommend_collection(10000, true, false, false);
        assert_eq!(rec, "BTreeSet");
    }

    #[test]
    fn test_recommendation_unordered() {
        let rec = recommend_collection(10000, false, false, true);
        assert_eq!(rec, "HashSet");
    }

    #[test]
    fn test_benchmark_suite() {
        let benchmark = CollectionBenchmark::new(vec![100, 1000, 10000]);

        // Run benchmarks
        benchmark.benchmark_inserts();
        benchmark.benchmark_lookups();
        benchmark.benchmark_ranges();

        // Generate report
        benchmark.generate_report();
    }

    #[test]
    fn test_cache_locality() {
        use std::time::Instant;

        let n = 10000;

        // Sequential access (cache-friendly)
        let data: Vec<i32> = (0..n).collect();
        let start = Instant::now();
        let sum1: i32 = data.iter().sum();
        let sequential_time = start.elapsed();

        // Random access (cache-unfriendly simulation)
        let indices: Vec<usize> = (0..n).rev().collect();
        let start = Instant::now();
        let sum2: i32 = indices.iter().map(|&i| data[i]).sum();
        let random_time = start.elapsed();

        assert_eq!(sum1, sum2);

        println!("Sequential: {:?}", sequential_time);
        println!("Random: {:?}", random_time);

        // Sequential should be faster
        assert!(sequential_time < random_time);
    }
}
```

---

### Testing Strategies

1. **Correctness Tests**: Verify search results against linear scan
2. **Edge Case Tests**: Empty arrays, single element, duplicates
3. **Performance Tests**: Benchmark binary search vs linear scan
4. **Property Tests**: Verify sorted invariants maintained
5. **Stress Tests**: Test with millions of elements
6. **Comparison Tests**: SortedVec vs BTreeSet vs HashSet
7. **Memory Tests**: Measure space overhead

---

### Complete Working Example

```rust
use std::collections::VecDeque;

fn main() {
    println!("=== Binary Search & Sorted Structures ===\n");

    // Example 1: Log query system
    let mut logs = vec![
        LogEntry { timestamp: 100, level: "INFO".to_string(), message: "Server started".to_string() },
        LogEntry { timestamp: 150, level: "INFO".to_string(), message: "Request received".to_string() },
        LogEntry { timestamp: 200, level: "ERROR".to_string(), message: "Database error".to_string() },
        LogEntry { timestamp: 250, level: "INFO".to_string(), message: "Request completed".to_string() },
        LogEntry { timestamp: 300, level: "INFO".to_string(), message: "Shutdown".to_string() },
    ];

    // Query logs between timestamps
    let results = query_logs_by_time(&logs, 150, 250);
    println!("Logs between 150-250:");
    for log in results {
        println!("  [{}] {}: {}", log.timestamp, log.level, log.message);
    }

    // Example 2: Auto-complete
    let dictionary = vec![
        "rust".to_string(),
        "ruby".to_string(),
        "python".to_string(),
        "javascript".to_string(),
        "java".to_string(),
    ];

    let autocomplete = AutoComplete::new(dictionary);

    println!("\nAuto-complete for 'ru':");
    for suggestion in autocomplete.suggest("ru") {
        println!("  - {}", suggestion);
    }

    // Example 3: Sorted set for active sessions
    let mut sessions = SortedVec::new();

    sessions.insert(101);
    sessions.insert(105);
    sessions.insert(103);
    sessions.insert(107);

    println!("\nActive sessions: {:?}", sessions.as_slice());
    println!("Sessions 102-106: {:?}", sessions.range(&102, &106));
}
```

This project demonstrates binary search mastery:
- **Binary search variants** (exact, lower/upper bound)
- **Range queries** (O(log n + k) performance)
- **Prefix matching** for auto-complete
- **K-way merge** with heap
- **Sorted collections** with trade-off analysis
- **Performance optimization** and decision-making
