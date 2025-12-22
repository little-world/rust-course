use std::cmp::Ordering;
use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap, HashSet};

// =============================================================================
// Milestone 1: Binary Search Variants
// =============================================================================

pub fn binary_search_exact<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
    let mut left = 0;
    let mut right = arr.len();

    while left < right {
        let mid = left + (right - left) / 2;
        match arr[mid].cmp(target) {
            Ordering::Equal => return Some(mid),
            Ordering::Less => left = mid + 1,
            Ordering::Greater => right = mid,
        }
    }
    None
}

pub fn binary_search_lower_bound<T: Ord>(arr: &[T], target: &T) -> usize {
    let mut left = 0;
    let mut right = arr.len();

    while left < right {
        let mid = left + (right - left) / 2;
        if arr[mid] < *target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    left
}

pub fn binary_search_upper_bound<T: Ord>(arr: &[T], target: &T) -> usize {
    let mut left = 0;
    let mut right = arr.len();

    while left < right {
        let mid = left + (right - left) / 2;
        if arr[mid] <= *target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    left
}

pub fn is_sorted<T: Ord>(arr: &[T]) -> bool {
    arr.windows(2).all(|w| w[0] <= w[1])
}

// =============================================================================
// Milestone 2: Range Queries with Binary Search
// =============================================================================

pub fn range_query<'a, T: Ord>(arr: &'a [T], start: &T, end: &T) -> &'a [T] {
    if arr.is_empty() || start > end {
        return &arr[0..0];
    }
    let begin = binary_search_lower_bound(arr, start);
    let end_idx = binary_search_upper_bound(arr, end);
    &arr[begin.min(arr.len())..end_idx.min(arr.len())]
}

pub fn count_in_range<T: Ord>(arr: &[T], start: &T, end: &T) -> usize {
    if start > end {
        return 0;
    }
    let begin = binary_search_lower_bound(arr, start);
    let end_idx = binary_search_upper_bound(arr, end);
    end_idx.saturating_sub(begin)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: String,
    pub message: String,
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

pub fn query_logs_by_time(logs: &[LogEntry], start_time: u64, end_time: u64) -> &[LogEntry] {
    let start = LogEntry {
        timestamp: start_time,
        level: String::new(),
        message: String::new(),
    };
    let end = LogEntry {
        timestamp: end_time,
        level: String::new(),
        message: String::new(),
    };
    range_query(logs, &start, &end)
}

// =============================================================================
// Milestone 3: Auto-Complete with Prefix Matching
// =============================================================================

pub fn prefix_search<'a>(words: &'a [String], prefix: &str) -> &'a [String] {
    if prefix.is_empty() {
        return words;
    }
    let start = words.partition_point(|word| word.as_str() < prefix);
    let mut end = start;
    while end < words.len() && words[end].starts_with(prefix) {
        end += 1;
    }
    &words[start..end]
}

#[derive(Debug)]
pub struct AutoComplete {
    words: Vec<String>,
}

impl AutoComplete {
    pub fn new(mut words: Vec<String>) -> Self {
        words.sort();
        words.dedup();
        Self { words }
    }

    pub fn suggest(&self, prefix: &str) -> Vec<&str> {
        let matches = prefix_search(&self.words, prefix);
        matches.iter().take(10).map(|s| s.as_str()).collect()
    }

    pub fn suggest_all(&self, prefix: &str) -> Vec<&str> {
        prefix_search(&self.words, prefix)
            .iter()
            .map(|s| s.as_str())
            .collect()
    }

    pub fn word_count(&self) -> usize {
        self.words.len()
    }
}

// =============================================================================
// Milestone 4: Merge Sorted Sequences (K-Way Merge)
// =============================================================================

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
    if i < left.len() {
        result.extend_from_slice(&left[i..]);
    }
    if j < right.len() {
        result.extend_from_slice(&right[j..]);
    }
    result
}

pub fn merge_k<T: Ord + Clone>(sequences: &[&[T]]) -> Vec<T> {
    if sequences.is_empty() {
        return Vec::new();
    }
    let total_len: usize = sequences.iter().map(|seq| seq.len()).sum();
    let mut result = Vec::with_capacity(total_len);
    let mut heap: BinaryHeap<Reverse<(T, usize, usize)>> = BinaryHeap::new();

    for (seq_idx, seq) in sequences.iter().enumerate() {
        if let Some(first) = seq.first() {
            heap.push(Reverse((first.clone(), seq_idx, 0)));
        }
    }

    while let Some(Reverse((value, seq_idx, elem_idx))) = heap.pop() {
        result.push(value);
        let next_idx = elem_idx + 1;
        if let Some(next_val) = sequences[seq_idx].get(next_idx) {
            heap.push(Reverse((next_val.clone(), seq_idx, next_idx)));
        }
    }

    result
}

pub struct MergeIterator<'a, T> {
    sequences: Vec<&'a [T]>,
    indices: Vec<usize>,
    heap: BinaryHeap<Reverse<(T, usize)>>,
}

impl<'a, T: Ord + Clone> MergeIterator<'a, T> {
    pub fn new(sequences: Vec<&'a [T]>) -> Self {
        let mut heap = BinaryHeap::new();
        let mut indices = vec![0; sequences.len()];
        for (seq_idx, seq) in sequences.iter().enumerate() {
            if let Some(first) = seq.first() {
                heap.push(Reverse((first.clone(), seq_idx)));
            }
        }
        Self {
            sequences,
            indices,
            heap,
        }
    }
}

impl<'a, T: Ord + Clone> Iterator for MergeIterator<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let Reverse((value, seq_idx)) = self.heap.pop()?;
        self.indices[seq_idx] += 1;
        if let Some(next_val) = self.sequences[seq_idx].get(self.indices[seq_idx]) {
            self.heap.push(Reverse((next_val.clone(), seq_idx)));
        }
        Some(value)
    }
}

// =============================================================================
// Milestone 5: Sorted Set with Incremental Updates
// =============================================================================

#[derive(Debug, Clone)]
pub struct SortedVec<T> {
    data: Vec<T>,
}

impl<T: Ord> SortedVec<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn insert(&mut self, value: T) -> bool {
        match self.data.binary_search(&value) {
            Ok(_) => false,
            Err(idx) => {
                self.data.insert(idx, value);
                true
            }
        }
    }

    pub fn remove(&mut self, value: &T) -> bool {
        if let Ok(idx) = self.data.binary_search(value) {
            self.data.remove(idx);
            true
        } else {
            false
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        self.data.binary_search(value).is_ok()
    }

    pub fn range(&self, start: &T, end: &T) -> &[T] {
        range_query(&self.data, start, end)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data
    }
}

// =============================================================================
// Milestone 6: Performance Optimization and Trade-offs
// =============================================================================

pub struct CollectionBenchmark {
    sizes: Vec<usize>,
}

impl CollectionBenchmark {
    pub fn new(sizes: Vec<usize>) -> Self {
        Self { sizes }
    }

    pub fn benchmark_inserts(&self) {
        for &size in &self.sizes {
            let mut sorted_vec = SortedVec::new();
            let mut btree = BTreeSet::new();
            let mut hash = HashSet::new();
            for i in 0..size {
                sorted_vec.insert(i);
                btree.insert(i);
                hash.insert(i);
            }
            println!("Inserted {} elements", size);
            assert_eq!(sorted_vec.len(), size);
            assert_eq!(btree.len(), size);
            assert_eq!(hash.len(), size);
        }
    }

    pub fn benchmark_lookups(&self) {
        for &size in &self.sizes {
            let data: Vec<i32> = (0..size as i32).collect();
            let mut sorted_vec = SortedVec::new();
            for &v in &data {
                sorted_vec.insert(v);
            }
            let btree: BTreeSet<_> = data.iter().copied().collect();
            let hash: HashSet<_> = data.iter().copied().collect();
            for &key in &[0, size as i32 / 2, size as i32 - 1] {
                sorted_vec.contains(&key);
                btree.contains(&key);
                hash.contains(&key);
            }
        }
    }

    pub fn benchmark_ranges(&self) {
        for &size in &self.sizes {
            let mut sorted_vec = SortedVec::new();
            for i in 0..size {
                sorted_vec.insert(i);
            }
            let btree: BTreeSet<_> = (0..size).collect();
            let sv_range = sorted_vec.range(&10, &20);
            let bt_range: Vec<_> = btree.range(10..=20).collect();
            assert_eq!(sv_range.len(), bt_range.len());
        }
    }

    pub fn measure_memory(&self) {
        for &size in &self.sizes {
            println!(
                "Estimated memory for size {}: SortedVec={} bytes, BTreeSet={}, HashSet={}",
                size,
                size * std::mem::size_of::<usize>(),
                size * (std::mem::size_of::<usize>() * 2),
                size * (std::mem::size_of::<usize>() * 3)
            );
        }
    }

    pub fn generate_report(&self) {
        println!("Collection Benchmark Report");
        for &size in &self.sizes {
            println!("- Tested size {}", size);
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
        return "HashSet";
    }
    if size > 1000 || write_heavy {
        return "BTreeSet";
    }
    if needs_ranges {
        "SortedVec"
    } else {
        "BTreeSet"
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Milestone 1 tests
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
    fn test_lower_upper_bounds() {
        let arr = vec![1, 3, 5, 7, 9];
        assert_eq!(binary_search_lower_bound(&arr, &5), 2);
        assert_eq!(binary_search_lower_bound(&arr, &4), 2);
        assert_eq!(binary_search_upper_bound(&arr, &5), 3);
        assert_eq!(binary_search_upper_bound(&arr, &4), 2);
    }

    #[test]
    fn test_is_sorted() {
        assert!(is_sorted(&[1, 2, 3]));
        assert!(is_sorted(&[1, 1, 2]));
        assert!(!is_sorted(&[2, 1, 3]));
    }

    // Milestone 2 tests
    #[test]
    fn test_range_query_basic() {
        let arr = vec![1, 3, 5, 7, 9, 11, 13];
        let result = range_query(&arr, &5, &11);
        assert_eq!(result, &[5, 7, 9, 11]);
    }

    #[test]
    fn test_range_query_empty() {
        let arr = vec![1, 3, 5, 7, 9];
        let result = range_query(&arr, &20, &30);
        assert!(result.is_empty());
    }

    #[test]
    fn test_count_in_range() {
        let arr = vec![1, 3, 5, 7, 9, 11, 13, 15];
        assert_eq!(count_in_range(&arr, &5, &11), 4);
    }

    #[test]
    fn test_query_logs() {
        let logs = vec![
            LogEntry {
                timestamp: 100,
                level: "INFO".into(),
                message: "Msg1".into(),
            },
            LogEntry {
                timestamp: 200,
                level: "INFO".into(),
                message: "Msg2".into(),
            },
            LogEntry {
                timestamp: 300,
                level: "ERROR".into(),
                message: "Msg3".into(),
            },
        ];
        let result = query_logs_by_time(&logs, 150, 250);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].timestamp, 200);
    }

    // Milestone 3 tests
    #[test]
    fn test_prefix_search_basic() {
        let words = vec![
            "apple".to_string(),
            "application".to_string(),
            "apply".to_string(),
            "banana".to_string(),
        ];
        let result = prefix_search(&words, "app");
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_autocomplete() {
        let words = vec![
            "apple".to_string(),
            "application".to_string(),
            "apply".to_string(),
            "appreciate".to_string(),
            "banana".to_string(),
        ];
        let ac = AutoComplete::new(words);
        let suggestions = ac.suggest("app");
        assert!(suggestions.len() <= 10);
        assert!(suggestions.iter().all(|s| s.starts_with("app")));
    }

    #[test]
    fn test_autocomplete_dedup() {
        let words = vec!["apple".to_string(), "apple".to_string()];
        let ac = AutoComplete::new(words);
        assert_eq!(ac.word_count(), 1);
    }

    // Milestone 4 tests
    #[test]
    fn test_merge_two_basic() {
        let left = vec![1, 3, 5];
        let right = vec![2, 4, 6];
        let result = merge_two(&left, &right);
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
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
    fn test_merge_iterator() {
        let seq1 = vec![1, 4, 7];
        let seq2 = vec![2, 5, 8];
        let seq3 = vec![3, 6, 9];
        let sequences = vec![&seq1[..], &seq2[..], &seq3[..]];
        let iter = MergeIterator::new(sequences);
        let result: Vec<_> = iter.collect();
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_merge_performance() {
        use std::time::Instant;
        let sequences: Vec<Vec<i32>> = (0..10).map(|i| (i..1000).step_by(10).collect()).collect();
        let seq_refs: Vec<&[i32]> = sequences.iter().map(|seq| seq.as_slice()).collect();
        let start = Instant::now();
        let _ = merge_k(&seq_refs);
        let k_time = start.elapsed();
        let start = Instant::now();
        let mut merged = sequences[0].clone();
        for seq in &sequences[1..] {
            merged = merge_two(&merged, seq);
        }
        let two_time = start.elapsed();
        println!("K-way: {:?}, two-way: {:?}", k_time, two_time);
    }

    // Milestone 5 tests
    #[test]
    fn test_sorted_vec_insert() {
        let mut sv = SortedVec::new();
        assert!(sv.insert(5));
        assert!(sv.insert(3));
        assert!(sv.insert(7));
        assert_eq!(sv.as_slice(), &[3, 5, 7]);
    }

    #[test]
    fn test_sorted_vec_remove() {
        let mut sv = SortedVec::new();
        sv.insert(1);
        sv.insert(3);
        sv.insert(5);
        assert!(sv.remove(&3));
        assert_eq!(sv.as_slice(), &[1, 5]);
    }

    #[test]
    fn test_sorted_vec_range() {
        let mut sv = SortedVec::new();
        for value in &[1, 3, 5, 7, 9, 11] {
            sv.insert(*value);
        }
        assert_eq!(sv.range(&5, &9), &[5, 7, 9]);
    }

    #[test]
    fn test_sorted_vec_contains() {
        let mut sv = SortedVec::new();
        sv.insert(1);
        sv.insert(3);
        assert!(sv.contains(&3));
        assert!(!sv.contains(&5));
    }

    // Milestone 6 tests
    #[test]
    fn test_recommendations() {
        assert_eq!(recommend_collection(100, true, true, false), "SortedVec");
        assert_eq!(recommend_collection(10000, true, false, false), "BTreeSet");
        assert_eq!(recommend_collection(1000, false, false, true), "HashSet");
    }

    #[test]
    fn test_benchmark_suite() {
        let benchmark = CollectionBenchmark::new(vec![100, 1000]);
        benchmark.benchmark_inserts();
        benchmark.benchmark_lookups();
        benchmark.benchmark_ranges();
        benchmark.measure_memory();
        benchmark.generate_report();
    }

    #[test]
    fn test_cache_locality() {
        use std::time::Instant;
        let n: usize = 10000;
        let data: Vec<i32> = (0..n as i32).collect();
        let start = Instant::now();
        let sum_seq: i32 = data.iter().sum();
        let seq_time = start.elapsed();
        let indices: Vec<usize> = (0..n).rev().collect();
        let start = Instant::now();
        let sum_rand: i32 = indices.iter().map(|&i| data[i]).sum();
        let rand_time = start.elapsed();
        assert_eq!(sum_seq, sum_rand);
        println!("Sequential: {:?}, Random: {:?}", seq_time, rand_time);
    }
}

fn main() {}
