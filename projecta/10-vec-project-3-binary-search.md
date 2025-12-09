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

---

### Milestone 1: Implement Binary Search Variants

**Goal**: Implement exact match, lower_bound, upper_bound binary searches.

**What to implement**:
- `binary_search_exact()`: Find exact match, return index
- `binary_search_lower_bound()`: Find first element >= target
- `binary_search_upper_bound()`: Find first element > target
- Generic implementations that work with any ordered type

---

**Binary Search Explained**:

Binary search is a divide-and-conquer algorithm that finds an element in a **sorted** array in O(log n) time.

**How it works**:
1. Start with two pointers: `left = 0`, `right = array.len()`
2. Calculate middle: `mid = (left + right) / 2`
3. Compare `array[mid]` with `target`:
   - If `array[mid] == target`: Found! Return `mid`
   - If `array[mid] < target`: Search right half (`left = mid + 1`)
   - If `array[mid] > target`: Search left half (`right = mid`)
4. Repeat until `left >= right`

**Example**: Search for 7 in `[1, 3, 5, 7, 9, 11, 13]`
```
Step 1: left=0, right=7, mid=3 → arr[3]=7 → Found at index 3!
```

**Variants**:
- **Exact match**: Return index if found, None otherwise
- **Lower bound**: First position where `arr[i] >= target` (leftmost insertion point)
- **Upper bound**: First position where `arr[i] > target` (rightmost insertion point)

---


**Key differences**:
- Lower bound: `arr[mid] < target` → move right
- Upper bound: `arr[mid] <= target` → move right (includes equals)



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

**Range Query Explained**:

Range queries find all elements in `[start, end]` using two binary searches:
1. Find **lower bound** of `start` → first element >= start
2. Find **upper bound** of `end` → first element > end
3. Return slice `arr[lower..upper]`

**Visual example**: Find range [5, 11] in `[1, 3, 5, 7, 9, 11, 13, 15]`
```
Array:  [1, 3, 5, 7, 9, 11, 13, 15]
Index:   0  1  2  3  4   5   6   7

lower_bound(5) = 2  (first element >= 5)
upper_bound(11) = 6 (first element > 11)
Result: arr[2..6] = [5, 7, 9, 11]
```


**For LogEntry with custom ordering**:
- Implement `Ord` based on timestamp
- Create dummy entries with target timestamps for comparison
- Use range_query on the sorted log array

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

**Prefix Search Explained**:

Prefix matching finds all strings starting with a given prefix. On sorted strings:
1. Find first string >= prefix (lower bound)
2. Scan forward while strings start with prefix
3. Stop when prefix no longer matches

**Visual example**: Find prefix "app" in sorted words
```
Words: ["apple", "application", "apply", "banana", "band"]
         ^^^^^    ^^^^^^^^^^^    ^^^^^
         match    match          match

Step 1: Binary search for "app" → index 0 (first "apple")
Step 2: Scan forward: "apple".starts_with("app") ✓
                      "application".starts_with("app") ✓
                      "apply".starts_with("app") ✓
                      "banana".starts_with("app") ✗ (stop)
Result: ["apple", "application", "apply"]
```


**Alternative using lower/upper bound trick**:
- Lower bound: search for prefix "app"
- Upper bound: search for prefix + 1 char "apq" (next string after "app...")
- This gives exact range without scanning



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

**K-Way Merge Explained**:

Merging multiple sorted sequences efficiently is crucial for external sorting, log aggregation, and distributed systems.

**Two-way merge (simple but inefficient for k sequences)**:
```rust
// Merge [1,3,5] and [2,4,6]
pub fn merge_two<T: Ord + Clone>(left: &[T], right: &[T]) -> Vec<T> {
    let mut result = Vec::with_capacity(left.len() + right.len());
    let mut i = 0;
    let mut j = 0;

    while i < left.len() && j < right.len() {
        if left[i] <= right[j] {
            result.push(left[i].clone());
            i += 1;
        } else {
            result.push(right[j].clone());
            j += 1;
        }
    }

    result.extend_from_slice(&left[i..]);
    result.extend_from_slice(&right[j..]);
    result
}
```

**K-way merge with heap (optimal)**:

Problem: Merging 100 sequences with repeated 2-way merge is O(nk).

Solution: Use min-heap to track smallest element from each sequence.

**Algorithm**:
1. Create min-heap with first element from each sequence
2. Pop minimum (gives next merged element)
3. Push next element from same sequence to heap
4. Repeat until heap empty

**Visual example**: Merge 3 sequences
```
Seq 0: [1, 4, 7]
Seq 1: [2, 5, 8]
Seq 2: [3, 6, 9]

Initial heap: [(1, seq=0), (2, seq=1), (3, seq=2)]

Step 1: Pop (1, seq=0), output 1, push (4, seq=0)
Heap: [(2, seq=1), (3, seq=2), (4, seq=0)]

Step 2: Pop (2, seq=1), output 2, push (5, seq=1)
Heap: [(3, seq=2), (4, seq=0), (5, seq=1)]

Step 3: Pop (3, seq=2), output 3, push (6, seq=2)
...
Result: [1, 2, 3, 4, 5, 6, 7, 8, 9]
```

**Complexity**:
- Two-way repeated: O(nk) where n = total elements, k = sequences
- K-way with heap: O(n log k)
- For k=100: K-way is ~50x faster!

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

---

**Sorted Vector Explained**:

A `SortedVec` maintains sorted order on insertion, providing:
- Fast lookups: O(log n) using binary search
- Range queries: Not possible with HashSet
- Ordered iteration: Always sorted
- Cache-friendly: Contiguous memory

**Trade-off**: Insert/remove is O(n) due to shifting, but for small-medium collections (<1000 elements), cache locality makes it faster than tree-based structures.

**Insert algorithm**
**Visual example**: Insert 6 into `[1, 3, 5, 7, 9]`
```
Binary search finds position 3 (between 5 and 7)
Before: [1, 3, 5, 7, 9]
After:  [1, 3, 5, 6, 7, 9]
```


**When to use SortedVec vs BTreeSet vs HashSet**:
- **SortedVec**: Small sets (<1K), need ranges, cache-friendly
- **BTreeSet**: Large sets (>1K), need ordering and ranges
- **HashSet**: Only need membership test, no ordering required

---
**Architecture**:
- Structs: `SortedVec<T>`
- Fields: `data: Vec<T>`
- Functions:
  - `new() -> Self` - Create empty set
  - `insert(value: T) -> bool` - Add maintaining order
  - `remove(value: &T) -> bool` - Remove if present
  - `contains(value: &T) -> bool` - O(log n) search
  - `range(start: &T, end: &T) -> &[T]` - Range query

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


---

**Performance Analysis Explained**:

Understanding when to choose each data structure is critical for production systems.

**Comparison matrix**:

| Operation     | SortedVec | BTreeSet | HashSet |
|--------------|-----------|----------|---------|
| Insert       | O(n)      | O(log n) | O(1)    |
| Remove       | O(n)      | O(log n) | O(1)    |
| Contains     | O(log n)  | O(log n) | O(1)    |
| Range query  | O(log n)  | O(log n) | ✗       |
| Ordered iter | ✓ Free    | ✓ Free   | ✗       |
| Memory       | Best      | Medium   | High    |
| Cache        | Excellent | Medium   | Poor    |

**When SortedVec wins** (despite O(n) inserts):
- Small collections (<1000 elements)
- Read-heavy workloads (90% reads, 10% writes)
- Need range queries
- Cache locality matters

**Real-world example**: Active user sessions
```rust
// 100 active users at any time
// 1000 session checks per second
// 10 new sessions per second

let mut sessions = SortedVec::new();

// Insert: 10/sec × O(100) = 1000 ops
// Contains: 1000/sec × O(log 100) = 7000 ops
// Range queries: Fast with zero overhead

// Total: ~8000 ops/sec → microseconds per operation
// SortedVec wins due to cache locality!
```


**Key insights**:
1. **Asymptotic complexity isn't everything**: O(n) can beat O(log n) for small n due to cache
2. **Memory layout matters**: Contiguous arrays (SortedVec) have better cache locality than trees
3. **Read/write ratio**: SortedVec excels when reads dominate
4. **Measure, don't guess**: Benchmark your specific workload

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

### Implementations
**Implementation Milestone 1**:

```rust
// Exact match implementation:
pub fn binary_search_exact<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
    let mut left = 0;
    let mut right = arr.len();

    while left < right {
        let mid = left + (right - left) / 2;  // Avoid overflow

        match arr[mid].cmp(target) {
            Ordering::Equal => return Some(mid),
            Ordering::Less => left = mid + 1,
            Ordering::Greater => right = mid,
        }
    }
    None
}

// Lower bound (first element >= target):
pub fn binary_search_lower_bound<T: Ord>(arr: &[T], target: &T) -> usize {
    let mut left = 0;
    let mut right = arr.len();

    while left < right {
        let mid = left + (right - left) / 2;

        if arr[mid] < target {
            left = mid + 1;  // Move right
        } else {
            right = mid;      // Could be answer, keep searching left
        }
    }
    left
}

// Upper bound (first element > target):
pub fn binary_search_upper_bound<T: Ord>(arr: &[T], target: &T) -> usize {
    let mut left = 0;
    let mut right = arr.len();

    while left < right {
        let mid = left + (right - left) / 2;

        if arr[mid] <= target {  // Note: <= not <
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    left
}
```


**Implementation Milestone 2**
```rust
pub fn range_query<T: Ord>(arr: &[T], start: &T, end: &T) -> &[T] {
    let lower = binary_search_lower_bound(arr, start);
    let upper = binary_search_upper_bound(arr, end);
    &arr[lower..upper]
}
```


**Implementation Milestone 3**:
```rust
pub fn prefix_search<'a>(words: &'a [String], prefix: &str) -> &'a [String] {
    // Find start position using partition_point
    let start = words.partition_point(|word| word.as_str() < prefix);

    // Find end by counting matches
    let mut end = start;
    while end < words.len() && words[end].starts_with(prefix) {
        end += 1;
    }

    &words[start..end]
}
```

**Implementation Milestone 3**:
```rust
impl AutoComplete {
    pub fn new(mut words: Vec<String>) -> Self {
        words.sort_unstable();      // Sort words
        words.dedup();              // Remove duplicates
        Self { words }
    }

    pub fn suggest(&self, prefix: &str) -> Vec<&str> {
        prefix_search(&self.words, prefix)
            .iter()
            .take(10)               // Limit to 10 suggestions
            .map(|s| s.as_str())
            .collect()
    }
}
```

---


**Implementation Milestone 4**:
```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

pub fn merge_k<T: Ord + Clone>(sequences: &[&[T]]) -> Vec<T> {
    let total_size: usize = sequences.iter().map(|s| s.len()).sum();
    let mut result = Vec::with_capacity(total_size);

    // Heap stores: (value, sequence_index, element_index)
    let mut heap = BinaryHeap::new();

    // Initialize heap with first element from each sequence
    for (seq_idx, seq) in sequences.iter().enumerate() {
        if let Some(first) = seq.first() {
            heap.push(Reverse((first.clone(), seq_idx, 0)));
        }
    }

    // Extract minimum and push next from same sequence
    while let Some(Reverse((value, seq_idx, elem_idx))) = heap.pop() {
        result.push(value);

        let next_idx = elem_idx + 1;
        if next_idx < sequences[seq_idx].len() {
            let next_val = sequences[seq_idx][next_idx].clone();
            heap.push(Reverse((next_val, seq_idx, next_idx)));
        }
    }

    result
}
```


**Implementation Milestone 5**:
**Insert algorithm**:
```rust
pub fn insert(&mut self, value: T) -> bool {
    // Find insertion position using binary search
    match self.data.binary_search(&value) {
        Ok(_) => false,  // Already exists
        Err(pos) => {
            self.data.insert(pos, value);  // Insert at correct position
            true
        }
    }
}
```

**Remove algorithm**:
```rust
pub fn remove(&mut self, value: &T) -> bool {
    match self.data.binary_search(value) {
        Ok(pos) => {
            self.data.remove(pos);  // Found, remove it
            true
        }
        Err(_) => false  // Not found
    }
}
```

**Contains (fast O(log n) lookup)**:
```rust
pub fn contains(&self, value: &T) -> bool {
    self.data.binary_search(value).is_ok()
}
```

**Range query**:
```rust
pub fn range(&self, start: &T, end: &T) -> &[T] {
    let lower = binary_search_lower_bound(&self.data, start);
    let upper = binary_search_upper_bound(&self.data, end);
    &self.data[lower..upper]
}
```


**Implementation Milestone 6**:

```rust
pub struct CollectionBenchmark {
    sizes: Vec<usize>,
}

impl CollectionBenchmark {
    pub fn benchmark_inserts(&self) {
        for &size in &self.sizes {
            // Test SortedVec
            let start = Instant::now();
            let mut sv = SortedVec::new();
            for i in 0..size {
                sv.insert(i);
            }
            let sv_time = start.elapsed();

            // Test BTreeSet
            let start = Instant::now();
            let mut bt = BTreeSet::new();
            for i in 0..size {
                bt.insert(i);
            }
            let bt_time = start.elapsed();

            // Test HashSet
            let start = Instant::now();
            let mut hs = HashSet::new();
            for i in 0..size {
                hs.insert(i);
            }
            let hs_time = start.elapsed();

            println!("Size {}: SV={:?}, BT={:?}, HS={:?}",
                     size, sv_time, bt_time, hs_time);
        }
    }
}

pub fn recommend_collection(
    size: usize,
    needs_ordering: bool,
    needs_ranges: bool,
    write_heavy: bool,
) -> &'static str {
    if !needs_ordering && !needs_ranges {
        return "HashSet";  // Fast, no ordering needed
    }

    if needs_ranges {
        if size < 1000 && !write_heavy {
            return "SortedVec";  // Cache-friendly for small sizes
        }
        return "BTreeSet";  // Better for large or write-heavy
    }

    if size < 1000 {
        "SortedVec"
    } else {
        "BTreeSet"
    }
}
```
