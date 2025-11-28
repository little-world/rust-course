# Chapter 17: Parallel Programming - Project 1

## Project 1: Parallel Sorting Algorithms Suite

### Problem Statement

Build a comprehensive parallel sorting library that demonstrates fork-join parallelism and divide-and-conquer strategies. Implement multiple sorting algorithms (merge sort, quicksort, radix sort) with progressive parallelization optimizations, achieving 4-8x speedup on multi-core systems while learning when parallelism helps versus hurts performance.

The system must:
- Sort arrays of 1 million to 100 million elements
- Implement sequential baselines for comparison
- Use fork-join parallelism effectively
- Handle various data distributions (random, sorted, reverse, duplicates)
- Optimize with sequential cutoffs and work stealing
- Demonstrate Amdahl's Law and scalability limits
- Achieve 60-80% parallel efficiency on 8 cores

### Use Cases

- **Database Systems**: Sorting query results, index building, external merge sort
- **Data Analytics**: Sorting large datasets (logs, metrics, time series)
- **Scientific Computing**: Sorting particle positions, mesh vertices
- **Search Engines**: Index construction, ranking scores
- **Financial Systems**: Transaction sorting, order book management
- **Operating Systems**: Process scheduling, file system operations

### Why It Matters

**Sequential Sorting Limits:**
```
Single core: ~100M comparisons/sec
Sorting 100M elements: ~2 seconds (O(n log n))
```

**Parallel Potential:**
```
8 cores: ~600M comparisons/sec (ideal)
Sorting 100M elements: ~0.3 seconds (6-7x speedup)
16 cores: ~1000M comparisons/sec (diminishing returns)
```

**Amdahl's Law Reality:**
```
If 5% of sorting is sequential (final merge):
Max speedup with infinite cores = 1 / 0.05 = 20x
With 8 cores: Speedup ≈ 6-7x (not 8x!)
```

**When Parallelism Helps vs Hurts:**

Helps:
- Large arrays (>10K elements)
- Random/unordered data
- Multi-core machines
- Memory bandwidth available

Hurts:
- Small arrays (<1K elements) - overhead dominates
- Already sorted data - minimal work
- Thread creation cost > sorting cost
- Cache thrashing from excessive parallelism

**Real-World Performance:**
- GNU sort (parallel): 8x speedup on 8 cores for 100GB files
- PostgreSQL parallel sort: 4-6x speedup on index building
- Java's Arrays.parallelSort(): 3-5x speedup typical

**Why Each Algorithm:**
1. **Merge Sort**: Easily parallelizable, stable, predictable performance
2. **Quicksort**: In-place, cache-friendly, good average case
3. **Radix Sort**: Linear time for integers, different parallelism model

---

## Milestone 1: Sequential Baseline Implementations

### Introduction

Implement sequential merge sort and quicksort to establish performance baselines. Understanding sequential algorithms is critical before parallelizing - you need to know what portion of work is parallelizable (Amdahl's Law).

**Merge Sort:**
- O(n log n) worst case
- Stable (preserves order of equal elements)
- Not in-place (requires O(n) extra space)
- Divide-and-conquer: Split, recurse, merge

**Quicksort:**
- O(n log n) average, O(n²) worst case
- Not stable
- In-place (O(log n) stack space)
- Divide-and-conquer: Partition, recurse

### Architecture

**Structs:**
- `SortStats` - Performance metrics
  - **Field** `comparisons: usize` - Number of comparisons
  - **Field** `swaps: usize` - Number of swaps
  - **Field** `duration: Duration` - Time taken
  - **Function** `new() -> Self` - Create new stats
  - **Function** `print(&self, name: &str)` - Display results

**Key Functions:**
- `merge_sort<T: Ord>(arr: &mut [T])` - Sequential merge sort
- `quicksort<T: Ord>(arr: &mut [T])` - Sequential quicksort
- `merge<T: Ord>(left: &[T], right: &[T], result: &mut [T])` - Merge two sorted arrays
- `partition<T: Ord>(arr: &mut [T]) -> usize` - Partition around pivot
- `is_sorted<T: Ord>(arr: &[T]) -> bool` - Verify sorted order

**Merge Sort Algorithm:**
```rust
fn merge_sort<T: Ord>(arr: &mut [T]) {
    if arr.len() <= 1 { return; }

    let mid = arr.len() / 2;
    merge_sort(&mut arr[..mid]);   // Sort left half
    merge_sort(&mut arr[mid..]);   // Sort right half
    merge(&arr[..mid], &arr[mid..], arr);  // Merge halves
}
```

**Quicksort Algorithm:**
```rust
fn quicksort<T: Ord>(arr: &mut [T]) {
    if arr.len() <= 1 { return; }

    let pivot = partition(arr);    // Partition around pivot
    quicksort(&mut arr[..pivot]);  // Sort left partition
    quicksort(&mut arr[pivot+1..]); // Sort right partition
}
```

**Role Each Plays:**
- Divide: Split array into subproblems
- Conquer: Recursively sort subproblems
- Combine: Merge sorted subproblems (merge sort) or do nothing (quicksort)

### Checkpoint Tests

```rust
#[test]
fn test_merge_sort_small() {
    let mut arr = vec![5, 2, 8, 1, 9];
    merge_sort(&mut arr);
    assert_eq!(arr, vec![1, 2, 5, 8, 9]);
}

#[test]
fn test_merge_sort_large() {
    let mut arr: Vec<i32> = (0..10000).rev().collect();
    merge_sort(&mut arr);
    assert!(is_sorted(&arr));
}

#[test]
fn test_quicksort_small() {
    let mut arr = vec![5, 2, 8, 1, 9];
    quicksort(&mut arr);
    assert_eq!(arr, vec![1, 2, 5, 8, 9]);
}

#[test]
fn test_quicksort_duplicates() {
    let mut arr = vec![5, 2, 8, 2, 5, 1];
    quicksort(&mut arr);
    assert_eq!(arr, vec![1, 2, 2, 5, 5, 8]);
}

#[test]
fn test_stability_merge_sort() {
    #[derive(Debug, PartialEq, Eq)]
    struct Item { key: i32, id: usize }

    impl Ord for Item {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.key.cmp(&other.key)
        }
    }
    impl PartialOrd for Item {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    let mut arr = vec![
        Item { key: 5, id: 0 },
        Item { key: 2, id: 1 },
        Item { key: 5, id: 2 },
    ];

    merge_sort(&mut arr);

    // Two items with key=5 should maintain original order (stable)
    assert_eq!(arr[1].id, 0);
    assert_eq!(arr[2].id, 2);
}

#[test]
fn benchmark_sequential() {
    use std::time::Instant;

    let sizes = vec![1_000, 10_000, 100_000, 1_000_000];

    for size in sizes {
        let mut arr1: Vec<i32> = (0..size).rev().collect();
        let mut arr2 = arr1.clone();

        let start = Instant::now();
        merge_sort(&mut arr1);
        let merge_time = start.elapsed();

        let start = Instant::now();
        quicksort(&mut arr2);
        let quick_time = start.elapsed();

        println!("Size {}: MergeSort {:?}, QuickSort {:?}",
            size, merge_time, quick_time);
    }
}
```

### Starter Code

```rust
use std::time::{Duration, Instant};

pub struct SortStats {
    pub comparisons: usize,
    pub swaps: usize,
    pub duration: Duration,
}

impl SortStats {
    pub fn new() -> Self {
        Self {
            comparisons: 0,
            swaps: 0,
            duration: Duration::ZERO,
        }
    }

    pub fn print(&self, name: &str) {
        println!("{}: {:?}, {} comparisons, {} swaps",
            name, self.duration, self.comparisons, self.swaps);
    }
}

pub fn merge_sort<T: Ord + Clone>(arr: &mut [T]) {
    // TODO: Implement sequential merge sort
    //
    // Base case: arrays of size 0 or 1 are already sorted
    // if arr.len() <= 1 { return; }
    //
    // Recursive case:
    // let mid = arr.len() / 2;
    //
    // // Sort left and right halves
    // merge_sort(&mut arr[..mid]);
    // merge_sort(&mut arr[mid..]);
    //
    // // Merge sorted halves
    // let mut temp = arr.to_vec();  // Temporary buffer
    // merge(&arr[..mid], &arr[mid..], &mut temp);
    // arr.copy_from_slice(&temp);
    todo!()
}

fn merge<T: Ord + Clone>(left: &[T], right: &[T], result: &mut [T]) {
    // TODO: Merge two sorted arrays
    //
    // Two-pointer technique:
    // let mut i = 0;  // left index
    // let mut j = 0;  // right index
    // let mut k = 0;  // result index
    //
    // while i < left.len() && j < right.len() {
    //     if left[i] <= right[j] {
    //         result[k] = left[i].clone();
    //         i += 1;
    //     } else {
    //         result[k] = right[j].clone();
    //         j += 1;
    //     }
    //     k += 1;
    // }
    //
    // // Copy remaining elements
    // while i < left.len() {
    //     result[k] = left[i].clone();
    //     i += 1;
    //     k += 1;
    // }
    //
    // while j < right.len() {
    //     result[k] = right[j].clone();
    //     j += 1;
    //     k += 1;
    // }
    todo!()
}

pub fn quicksort<T: Ord>(arr: &mut [T]) {
    // TODO: Implement sequential quicksort
    //
    // Base case
    // if arr.len() <= 1 { return; }
    //
    // Partition and get pivot index
    // let pivot = partition(arr);
    //
    // Recursively sort left and right partitions
    // quicksort(&mut arr[..pivot]);
    // quicksort(&mut arr[pivot + 1..]);
    todo!()
}

fn partition<T: Ord>(arr: &mut [T]) -> usize {
    // TODO: Partition array around pivot
    //
    // Lomuto partition scheme:
    // let pivot_index = arr.len() - 1;  // Use last element as pivot
    // let mut i = 0;
    //
    // for j in 0..arr.len() - 1 {
    //     if arr[j] <= arr[pivot_index] {
    //         arr.swap(i, j);
    //         i += 1;
    //     }
    // }
    //
    // arr.swap(i, pivot_index);
    // i
    todo!()
}

pub fn is_sorted<T: Ord>(arr: &[T]) -> bool {
    // TODO: Verify array is sorted
    // arr.windows(2).all(|w| w[0] <= w[1])
    todo!()
}
```

---

## Milestone 2: Naive Parallel Merge Sort (Fork-Join)

### Introduction

**Why Milestone 1 Is Not Enough:**
Sequential sorting uses only 1 core. Modern CPUs have 8-16 cores sitting idle. Merge sort is embarrassingly parallel: the two recursive calls are independent and can run concurrently.

**What We're Improving:**
Use Rayon's `join()` for fork-join parallelism. Split work into two tasks, run them in parallel, then merge results.

**Fork-Join Pattern:**
```
Thread 0: Sort left half  ┐
                          ├─> Join here
Thread 1: Sort right half ┘
          ↓
Both threads: Merge results
```

**Expected Speedup:** 2-4x on 8-core machine (naive approach has overhead)

### Architecture

**Key Functions:**
- `parallel_merge_sort<T: Ord + Send>(arr: &mut [T])` - Parallel merge sort
- Use `rayon::join()` for parallel recursion

**Rayon's join():**
```rust
rayon::join(
    || work_task_1(),  // Potentially runs on different thread
    || work_task_2(),  // Potentially runs on different thread
)
```

**Role Each Plays:**
- `rayon::join()`: Fork two tasks, join when both complete
- Work stealing: Idle threads steal work from busy threads
- Thread pool: Rayon manages thread pool automatically

### Checkpoint Tests

```rust
#[test]
fn test_parallel_merge_sort() {
    let mut arr: Vec<i32> = (0..10000).rev().collect();
    parallel_merge_sort(&mut arr);
    assert!(is_sorted(&arr));
}

#[test]
fn test_parallel_correctness() {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let mut arr: Vec<i32> = (0..10000).map(|_| rng.gen()).collect();
    let mut arr_seq = arr.clone();

    parallel_merge_sort(&mut arr);
    merge_sort(&mut arr_seq);

    assert_eq!(arr, arr_seq);
}

#[test]
fn benchmark_parallel_vs_sequential() {
    use std::time::Instant;

    let sizes = vec![10_000, 100_000, 1_000_000];

    for size in sizes {
        let mut arr: Vec<i32> = (0..size).rev().collect();
        let mut arr_par = arr.clone();

        let start = Instant::now();
        merge_sort(&mut arr);
        let seq_time = start.elapsed();

        let start = Instant::now();
        parallel_merge_sort(&mut arr_par);
        let par_time = start.elapsed();

        let speedup = seq_time.as_secs_f64() / par_time.as_secs_f64();

        println!("Size {}: Sequential {:?}, Parallel {:?}, Speedup: {:.2}x",
            size, seq_time, par_time, speedup);
    }
}

#[test]
fn test_small_array_overhead() {
    use std::time::Instant;

    // Small array - parallelism should hurt
    let mut arr: Vec<i32> = (0..100).rev().collect();
    let mut arr_par = arr.clone();

    let start = Instant::now();
    merge_sort(&mut arr);
    let seq_time = start.elapsed();

    let start = Instant::now();
    parallel_merge_sort(&mut arr_par);
    let par_time = start.elapsed();

    println!("Small array (100 elements):");
    println!("  Sequential: {:?}", seq_time);
    println!("  Parallel:   {:?}", par_time);

    // Parallel might be slower!
    if par_time > seq_time {
        println!("  Parallelism overhead dominates for small arrays!");
    }
}
```

### Starter Code

```rust
use rayon::prelude::*;

pub fn parallel_merge_sort<T: Ord + Clone + Send>(arr: &mut [T]) {
    // TODO: Implement parallel merge sort
    //
    // Base case
    // if arr.len() <= 1 { return; }
    //
    // let mid = arr.len() / 2;
    // let (left, right) = arr.split_at_mut(mid);
    //
    // // PARALLEL: Sort left and right concurrently
    // rayon::join(
    //     || parallel_merge_sort(left),
    //     || parallel_merge_sort(right),
    // );
    //
    // // Merge (still sequential)
    // let mut temp = arr.to_vec();
    // merge(&arr[..mid], &arr[mid..], &mut temp);
    // arr.copy_from_slice(&temp);
    todo!()
}
```

---

## Milestone 3: Sequential Cutoff Optimization

### Introduction

**Why Milestone 2 Is Not Enough:**
Naive parallelization creates too many threads. For a 1M element array with binary splits, we'd create ~1M tasks! Thread creation and synchronization costs dominate for small subarrays.

**Overhead Analysis:**
```
Thread creation: ~1000 cycles
Context switch: ~500 cycles
Sorting 10 elements: ~50 cycles

Parallel overhead > sorting work!
```

**What We're Improving:**
Add sequential cutoff: switch to sequential sorting when subarray is small. This is a critical optimization for all parallel divide-and-conquer algorithms.

**Cutoff Strategy:**
```rust
const SEQUENTIAL_CUTOFF: usize = 10_000;

if arr.len() < SEQUENTIAL_CUTOFF {
    merge_sort(arr);  // Sequential for small arrays
} else {
    rayon::join(...);  // Parallel for large arrays
}
```

**Expected Improvement:** 2-3x better than naive parallel (6-8x over sequential)

### Architecture

**Constants:**
- `SEQUENTIAL_CUTOFF: usize = 10_000` - Threshold for parallelization
- `INSERTION_SORT_CUTOFF: usize = 32` - Use insertion sort for tiny arrays

**Modified Functions:**
- `optimized_parallel_merge_sort<T>(arr: &mut [T])` - With cutoff
- `insertion_sort<T>(arr: &mut [T])` - For tiny subarrays

**Role Each Plays:**
- Sequential cutoff: Minimize parallel overhead
- Insertion sort: O(n²) but fast for small n due to low constant factors
- Granularity control: Balance parallelism and overhead

### Checkpoint Tests

```rust
#[test]
fn test_insertion_sort() {
    let mut arr = vec![5, 2, 8, 1, 9, 3];
    insertion_sort(&mut arr);
    assert_eq!(arr, vec![1, 2, 3, 5, 8, 9]);
}

#[test]
fn test_cutoff_optimization() {
    use std::time::Instant;

    let size = 1_000_000;
    let mut arr: Vec<i32> = (0..size).rev().collect();
    let mut arr_naive = arr.clone();
    let mut arr_opt = arr.clone();

    // Naive parallel
    let start = Instant::now();
    parallel_merge_sort(&mut arr_naive);
    let naive_time = start.elapsed();

    // With cutoff
    let start = Instant::now();
    optimized_parallel_merge_sort(&mut arr_opt);
    let opt_time = start.elapsed();

    println!("Naive parallel: {:?}", naive_time);
    println!("With cutoff:    {:?}", opt_time);
    println!("Improvement: {:.2}x", naive_time.as_secs_f64() / opt_time.as_secs_f64());

    assert!(opt_time < naive_time);
}

#[test]
fn benchmark_different_cutoffs() {
    use std::time::Instant;

    let size = 1_000_000;
    let cutoffs = vec![1_000, 5_000, 10_000, 20_000, 50_000];

    println!("\nCutoff optimization benchmark:");
    for cutoff in cutoffs {
        let mut arr: Vec<i32> = (0..size).rev().collect();

        let start = Instant::now();
        parallel_merge_sort_with_cutoff(&mut arr, cutoff);
        let time = start.elapsed();

        println!("  Cutoff {}: {:?}", cutoff, time);
    }
}
```

### Starter Code

```rust
const SEQUENTIAL_CUTOFF: usize = 10_000;
const INSERTION_SORT_CUTOFF: usize = 32;

pub fn insertion_sort<T: Ord>(arr: &mut [T]) {
    // TODO: Implement insertion sort for small arrays
    //
    // for i in 1..arr.len() {
    //     let mut j = i;
    //     while j > 0 && arr[j] < arr[j - 1] {
    //         arr.swap(j, j - 1);
    //         j -= 1;
    //     }
    // }
    todo!()
}

pub fn optimized_parallel_merge_sort<T: Ord + Clone + Send>(arr: &mut [T]) {
    // TODO: Parallel merge sort with sequential cutoff
    //
    // // Very small: use insertion sort
    // if arr.len() <= INSERTION_SORT_CUTOFF {
    //     insertion_sort(arr);
    //     return;
    // }
    //
    // // Small: use sequential merge sort
    // if arr.len() < SEQUENTIAL_CUTOFF {
    //     merge_sort(arr);
    //     return;
    // }
    //
    // // Large: use parallel
    // let mid = arr.len() / 2;
    // let (left, right) = arr.split_at_mut(mid);
    //
    // rayon::join(
    //     || optimized_parallel_merge_sort(left),
    //     || optimized_parallel_merge_sort(right),
    // );
    //
    // // Merge
    // let mut temp = arr.to_vec();
    // merge(&arr[..mid], &arr[mid..], &mut temp);
    // arr.copy_from_slice(&temp);
    todo!()
}

pub fn parallel_merge_sort_with_cutoff<T: Ord + Clone + Send>(arr: &mut [T], cutoff: usize) {
    // TODO: Parameterized cutoff for benchmarking
    todo!()
}
```

---

## Milestone 4: Parallel Quicksort with Partition Parallelism

### Introduction

**Why Milestone 3 Is Not Enough:**
Merge sort requires O(n) extra space for merging. Quicksort is in-place but harder to parallelize efficiently. The partition step seems sequential, but we can parallelize the recursive calls after partitioning.

**What We're Improving:**
Implement parallel quicksort using Rayon. After partitioning, the two recursive calls are independent and can run in parallel.

**Challenge:**
Partition is inherently sequential (Amdahl's Law). For large arrays, partition takes ~O(n) time sequentially, limiting parallel speedup.

**Expected Speedup:** 4-6x (worse than merge sort due to sequential partition)

### Architecture

**Key Functions:**
- `parallel_quicksort<T: Ord + Send>(arr: &mut [T])` - Parallel quicksort
- `parallel_quicksort_three_way<T: Ord + Send>(arr: &mut [T])` - Handle duplicates better
- `median_of_three<T: Ord>(arr: &[T]) -> usize` - Better pivot selection

**Three-Way Partitioning:**
```
[< pivot | = pivot | > pivot]

Handles duplicates efficiently
Important for real-world data
```

**Role Each Plays:**
- Partition: Sequential bottleneck (Amdahl's Law)
- Parallel recursion: Where speedup comes from
- Pivot selection: Affects balance and performance

### Checkpoint Tests

```rust
#[test]
fn test_parallel_quicksort() {
    let mut arr: Vec<i32> = (0..10000).rev().collect();
    parallel_quicksort(&mut arr);
    assert!(is_sorted(&arr));
}

#[test]
fn test_quicksort_duplicates() {
    let mut arr = vec![5, 2, 8, 2, 5, 2, 5];
    parallel_quicksort(&mut arr);
    assert_eq!(arr, vec![2, 2, 2, 5, 5, 5, 8]);
}

#[test]
fn test_median_of_three() {
    let arr = vec![5, 2, 8];
    let median_idx = median_of_three(&arr);
    assert_eq!(arr[median_idx], 5);  // Median value
}

#[test]
fn benchmark_quicksort_vs_mergesort() {
    use std::time::Instant;

    let sizes = vec![100_000, 1_000_000, 10_000_000];

    for size in sizes {
        let mut arr1: Vec<i32> = (0..size).rev().collect();
        let mut arr2 = arr1.clone();

        let start = Instant::now();
        optimized_parallel_merge_sort(&mut arr1);
        let merge_time = start.elapsed();

        let start = Instant::now();
        parallel_quicksort(&mut arr2);
        let quick_time = start.elapsed();

        println!("Size {}: MergeSort {:?}, QuickSort {:?}",
            size, merge_time, quick_time);
    }
}

#[test]
fn test_worst_case_quicksort() {
    // Already sorted - worst case for naive quicksort
    let mut arr: Vec<i32> = (0..10000).collect();

    let start = std::time::Instant::now();
    parallel_quicksort(&mut arr);
    let time = start.elapsed();

    println!("Worst case (sorted): {:?}", time);
    assert!(is_sorted(&arr));
}
```

### Starter Code

```rust
const QUICKSORT_CUTOFF: usize = 5_000;

pub fn parallel_quicksort<T: Ord + Send>(arr: &mut [T]) {
    // TODO: Implement parallel quicksort
    //
    // Base cases
    // if arr.len() <= 1 { return; }
    //
    // if arr.len() < QUICKSORT_CUTOFF {
    //     quicksort(arr);  // Sequential for small arrays
    //     return;
    // }
    //
    // // Partition
    // let pivot = partition(arr);
    //
    // // PARALLEL: Sort partitions
    // let (left, right) = arr.split_at_mut(pivot);
    // rayon::join(
    //     || parallel_quicksort(left),
    //     || parallel_quicksort(&mut right[1..]),  // Skip pivot
    // );
    todo!()
}

fn median_of_three<T: Ord>(arr: &[T]) -> usize {
    // TODO: Choose median of first, middle, last as pivot
    //
    // Better pivot selection reduces worst-case probability
    //
    // let first = 0;
    // let middle = arr.len() / 2;
    // let last = arr.len() - 1;
    //
    // if arr[first] <= arr[middle] && arr[middle] <= arr[last] {
    //     middle
    // } else if arr[first] <= arr[last] && arr[last] <= arr[middle] {
    //     last
    // } else {
    //     first
    // }
    todo!()
}

pub fn parallel_quicksort_three_way<T: Ord + Send + Clone>(arr: &mut [T]) {
    // TODO: Three-way partitioning for duplicate handling
    //
    // Partition into [< pivot | = pivot | > pivot]
    //
    // Better for data with many duplicates
    // Common in real-world data
    todo!()
}
```

---

## Milestone 5: Parallel Radix Sort (Bucket Parallelism)

### Introduction

**Why Milestone 4 Is Not Enough:**
Both merge sort and quicksort are comparison-based: Ω(n log n) lower bound. For integers, we can do better with radix sort: O(d × n) where d is number of digits.

**What We're Improving:**
Implement parallel radix sort using bucket parallelism. Each thread processes different buckets independently.

**Radix Sort:**
```
Sort by least significant digit first
[170, 45, 75, 90, 2, 802, 24, 66]

Pass 1 (1s place): [170, 90, 2, 802, 24, 45, 75, 66]
Pass 2 (10s place): [2, 802, 24, 45, 66, 170, 75, 90]
Pass 3 (100s place): [2, 24, 45, 66, 75, 90, 170, 802]
```

**Parallelization:**
- Each pass can process multiple buckets in parallel
- Counting phase can be parallel (per-thread counts)
- Prefix sum for bucket offsets

**Expected Performance:** Linear O(n) for integers, 3-5x speedup parallel

### Architecture

**Key Functions:**
- `radix_sort_u32(arr: &mut [u32])` - Sequential radix sort
- `parallel_radix_sort_u32(arr: &mut [u32])` - Parallel radix sort
- `counting_sort_digit(arr: &mut [u32], digit: u32)` - Sort by single digit
- `parallel_bucket_sort(arr: &mut [u32], buckets: usize)` - Parallel bucketing

**Algorithm:**
```rust
for digit_position in 0..32 {  // 32 bits = 32 passes
    counting_sort_by_bit(arr, digit_position);
}
```

**Role Each Plays:**
- Counting sort: Stable sort for single digit/bit
- Multiple passes: One per digit/bit
- Bucket parallelism: Different threads process different buckets

### Checkpoint Tests

```rust
#[test]
fn test_radix_sort() {
    let mut arr = vec![170, 45, 75, 90, 2, 802, 24, 66];
    radix_sort_u32(&mut arr);
    assert_eq!(arr, vec![2, 24, 45, 66, 75, 90, 170, 802]);
}

#[test]
fn test_radix_large() {
    let mut arr: Vec<u32> = (0..100_000).rev().collect();
    radix_sort_u32(&mut arr);
    assert!(arr.windows(2).all(|w| w[0] <= w[1]));
}

#[test]
fn test_parallel_radix() {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let mut arr: Vec<u32> = (0..1_000_000).map(|_| rng.gen()).collect();
    let mut arr_seq = arr.clone();

    radix_sort_u32(&mut arr_seq);
    parallel_radix_sort_u32(&mut arr);

    assert_eq!(arr, arr_seq);
}

#[test]
fn benchmark_radix_vs_comparison() {
    use std::time::Instant;

    let sizes = vec![100_000, 1_000_000, 10_000_000];

    for size in sizes {
        let mut arr1: Vec<u32> = (0..size).rev().collect();
        let mut arr2 = arr1.clone();
        let mut arr3 = arr1.clone();

        let start = Instant::now();
        optimized_parallel_merge_sort(&mut arr1);
        let merge_time = start.elapsed();

        let start = Instant::now();
        radix_sort_u32(&mut arr2);
        let radix_time = start.elapsed();

        let start = Instant::now();
        parallel_radix_sort_u32(&mut arr3);
        let par_radix_time = start.elapsed();

        println!("Size {}:", size);
        println!("  MergeSort: {:?}", merge_time);
        println!("  Radix:     {:?}", radix_time);
        println!("  Par Radix: {:?}", par_radix_time);
    }
}
```

### Starter Code

```rust
const RADIX_BITS: u32 = 8;  // Process 8 bits at a time
const RADIX_BASE: usize = 1 << RADIX_BITS;  // 256 buckets

pub fn radix_sort_u32(arr: &mut [u32]) {
    // TODO: Implement sequential radix sort
    //
    // Process 8 bits at a time (4 passes for 32-bit integers)
    //
    // let mut temp = vec![0u32; arr.len()];
    //
    // for shift in (0..32).step_by(RADIX_BITS as usize) {
    //     counting_sort_by_bits(arr, &mut temp, shift);
    //     std::mem::swap(arr, &mut temp);
    // }
    todo!()
}

fn counting_sort_by_bits(arr: &[u32], output: &mut [u32], shift: u32) {
    // TODO: Counting sort for specific bit range
    //
    // 1. Count occurrences of each bucket
    // let mut counts = vec![0usize; RADIX_BASE];
    // for &val in arr {
    //     let bucket = ((val >> shift) & ((RADIX_BASE - 1) as u32)) as usize;
    //     counts[bucket] += 1;
    // }
    //
    // 2. Prefix sum to get positions
    // let mut positions = vec![0usize; RADIX_BASE];
    // for i in 1..RADIX_BASE {
    //     positions[i] = positions[i - 1] + counts[i - 1];
    // }
    //
    // 3. Place elements in output
    // for &val in arr {
    //     let bucket = ((val >> shift) & ((RADIX_BASE - 1) as u32)) as usize;
    //     output[positions[bucket]] = val;
    //     positions[bucket] += 1;
    // }
    todo!()
}

pub fn parallel_radix_sort_u32(arr: &mut [u32]) {
    // TODO: Parallel radix sort
    //
    // Parallelize the counting phase:
    // - Each thread counts its portion of array
    // - Merge thread-local counts
    // - Parallel prefix sum
    // - Parallel placement
    //
    // use rayon::prelude::*;
    //
    // let num_threads = rayon::current_num_threads();
    // let chunk_size = (arr.len() + num_threads - 1) / num_threads;
    //
    // for shift in (0..32).step_by(RADIX_BITS as usize) {
    //     // Parallel counting
    //     let thread_counts: Vec<_> = arr.par_chunks(chunk_size)
    //         .map(|chunk| {
    //             let mut counts = vec![0usize; RADIX_BASE];
    //             for &val in chunk {
    //                 let bucket = ((val >> shift) & ((RADIX_BASE - 1) as u32)) as usize;
    //                 counts[bucket] += 1;
    //             }
    //             counts
    //         })
    //         .collect();
    //
    //     // Merge and sort
    //     // ...
    // }
    todo!()
}
```

---

## Milestone 6: Hybrid Algorithm and Scalability Analysis

### Introduction

**Why Milestone 5 Is Not Enough:**
No single algorithm is best for all inputs. Production systems use hybrid approaches:
- Small arrays: Insertion sort
- Medium arrays: Quicksort or merge sort
- Integers: Radix sort
- Nearly sorted: Adaptive algorithms

**What We're Improving:**
Create adaptive hybrid sorter that chooses algorithm based on input characteristics. Measure and analyze scalability using Amdahl's Law and strong/weak scaling.

**Expected Performance:** Best-in-class for all input types

### Architecture

**Enum:**
- `SortAlgorithm` - Algorithm selection
  - `MergeSort`, `QuickSort`, `RadixSort`, `InsertionSort`, `Adaptive`

**Key Functions:**
- `hybrid_sort<T>(arr: &mut [T])` - Adaptive algorithm selection
- `analyze_scalability(sizes: &[usize])` - Measure speedup vs cores
- `strong_scaling_test(size: usize)` - Fixed problem size, vary cores
- `weak_scaling_test(size_per_core: usize)` - Scale problem with cores

**Hybrid Strategy:**
```rust
fn hybrid_sort<T>(arr: &mut [T]) {
    if arr.len() < 32 {
        insertion_sort(arr);
    } else if is_nearly_sorted(arr) {
        adaptive_merge_sort(arr);
    } else if T is integer {
        radix_sort(arr);
    } else {
        parallel_quicksort(arr);
    }
}
```

**Scalability Metrics:**
- **Strong scaling**: Speedup = T(1) / T(p) where p = cores
- **Weak scaling**: Time should stay constant as problem and cores scale
- **Parallel efficiency**: Speedup / Cores (ideal = 100%)

### Checkpoint Tests

```rust
#[test]
fn test_hybrid_sort() {
    let test_cases = vec![
        vec![5, 2, 8, 1, 9],  // Small
        (0..1000).rev().collect::<Vec<_>>(),  // Medium reversed
        vec![1, 2, 3, 5, 4, 6, 7],  // Nearly sorted
        vec![5, 5, 5, 2, 2, 8, 8],  // Duplicates
    ];

    for mut arr in test_cases {
        hybrid_sort(&mut arr);
        assert!(is_sorted(&arr));
    }
}

#[test]
fn test_strong_scaling() {
    use rayon::ThreadPoolBuilder;

    let size = 10_000_000;
    let core_counts = vec![1, 2, 4, 8];

    println!("\nStrong Scaling (fixed size: {}):", size);
    println!("Cores | Time      | Speedup | Efficiency");
    println!("------|-----------|---------|------------");

    let mut baseline_time = None;

    for cores in core_counts {
        let pool = ThreadPoolBuilder::new()
            .num_threads(cores)
            .build()
            .unwrap();

        let mut arr: Vec<i32> = (0..size).rev().collect();

        let time = pool.install(|| {
            let start = std::time::Instant::now();
            optimized_parallel_merge_sort(&mut arr);
            start.elapsed()
        });

        if baseline_time.is_none() {
            baseline_time = Some(time);
        }

        let speedup = baseline_time.unwrap().as_secs_f64() / time.as_secs_f64();
        let efficiency = (speedup / cores as f64) * 100.0;

        println!("{:5} | {:?} | {:7.2}x | {:6.1}%",
            cores, time, speedup, efficiency);
    }
}

#[test]
fn test_weak_scaling() {
    use rayon::ThreadPoolBuilder;

    let size_per_core = 1_000_000;
    let core_counts = vec![1, 2, 4, 8];

    println!("\nWeak Scaling (size per core: {}):", size_per_core);
    println!("Cores | Total Size | Time      | Efficiency");
    println!("------|------------|-----------|------------");

    let mut baseline_time = None;

    for cores in core_counts {
        let pool = ThreadPoolBuilder::new()
            .num_threads(cores)
            .build()
            .unwrap();

        let size = size_per_core * cores;
        let mut arr: Vec<i32> = (0..size).rev().collect();

        let time = pool.install(|| {
            let start = std::time::Instant::now();
            optimized_parallel_merge_sort(&mut arr);
            start.elapsed()
        });

        if baseline_time.is_none() {
            baseline_time = Some(time);
        }

        let efficiency = (baseline_time.unwrap().as_secs_f64() / time.as_secs_f64()) * 100.0;

        println!("{:5} | {:10} | {:?} | {:6.1}%",
            cores, size, time, efficiency);
    }
}

#[test]
fn test_amdahl_law() {
    // Measure sequential fraction
    //
    // Amdahl's Law: Speedup = 1 / (s + (1-s)/p)
    // where s = sequential fraction, p = cores
    //
    // From measured speedups, compute implied sequential fraction

    let size = 1_000_000;
    let cores = vec![1, 2, 4, 8];
    let mut speedups = vec![];

    for &core_count in &cores {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(core_count)
            .build()
            .unwrap();

        let mut arr: Vec<i32> = (0..size).rev().collect();

        let time = pool.install(|| {
            let start = std::time::Instant::now();
            optimized_parallel_merge_sort(&mut arr);
            start.elapsed()
        });

        if core_count == 1 {
            speedups.push(1.0);
        } else {
            let speedup = speedups[0] * (time.as_secs_f64() / speedups[0]);
            speedups.push(speedup);
        }
    }

    // Calculate implied sequential fraction from 8-core speedup
    let p = 8.0;
    let actual_speedup = speedups[3];
    let s = (p - actual_speedup) / (p * actual_speedup - actual_speedup);

    println!("\nAmdahl's Law Analysis:");
    println!("Measured 8-core speedup: {:.2}x", actual_speedup);
    println!("Implied sequential fraction: {:.1}%", s * 100.0);
    println!("Theoretical max speedup (∞ cores): {:.2}x", 1.0 / s);
}
```

### Starter Code

```rust
#[derive(Debug, Copy, Clone)]
pub enum SortAlgorithm {
    MergeSort,
    QuickSort,
    RadixSort,
    InsertionSort,
    Adaptive,
}

pub fn hybrid_sort<T: Ord + Clone + Send>(arr: &mut [T]) {
    // TODO: Adaptive algorithm selection
    //
    // Choose best algorithm based on:
    // - Array size
    // - Data type
    // - Presortedness
    //
    // if arr.len() < 32 {
    //     insertion_sort(arr);
    // } else if arr.len() < 10_000 {
    //     quicksort(arr);
    // } else {
    //     optimized_parallel_merge_sort(arr);
    // }
    todo!()
}

pub fn is_nearly_sorted<T: Ord>(arr: &[T], threshold: f64) -> bool {
    // TODO: Check if array is nearly sorted
    //
    // Count inversions or runs
    // If < threshold%, consider nearly sorted
    //
    // let mut inversions = 0;
    // for i in 0..arr.len() - 1 {
    //     if arr[i] > arr[i + 1] {
    //         inversions += 1;
    //     }
    // }
    //
    // let inversion_rate = inversions as f64 / arr.len() as f64;
    // inversion_rate < threshold
    todo!()
}

pub fn analyze_scalability(sizes: &[usize], algorithms: &[SortAlgorithm]) {
    // TODO: Comprehensive scalability analysis
    //
    // For each size and algorithm:
    // - Run on 1, 2, 4, 8 cores
    // - Measure time and speedup
    // - Calculate parallel efficiency
    // - Print results table
    todo!()
}

pub struct ScalabilityReport {
    pub algorithm: SortAlgorithm,
    pub size: usize,
    pub cores: usize,
    pub time: Duration,
    pub speedup: f64,
    pub efficiency: f64,
}

impl ScalabilityReport {
    pub fn print_table(reports: &[Self]) {
        // TODO: Pretty-print scalability results
        println!("Algorithm    | Size       | Cores | Time      | Speedup | Efficiency");
        println!("-------------|------------|-------|-----------|---------|------------");
        // ...
    }
}
```

---

## Complete Working Example

```rust
use rayon::prelude::*;
use std::time::{Duration, Instant};

// ============================================================================
// SEQUENTIAL MERGE SORT
// ============================================================================

pub fn merge_sort<T: Ord + Clone>(arr: &mut [T]) {
    if arr.len() <= 1 {
        return;
    }

    let mid = arr.len() / 2;
    let mut left = arr[..mid].to_vec();
    let mut right = arr[mid..].to_vec();

    merge_sort(&mut left);
    merge_sort(&mut right);

    merge(&left, &right, arr);
}

fn merge<T: Ord + Clone>(left: &[T], right: &[T], result: &mut [T]) {
    let mut i = 0;
    let mut j = 0;
    let mut k = 0;

    while i < left.len() && j < right.len() {
        if left[i] <= right[j] {
            result[k] = left[i].clone();
            i += 1;
        } else {
            result[k] = right[j].clone();
            j += 1;
        }
        k += 1;
    }

    while i < left.len() {
        result[k] = left[i].clone();
        i += 1;
        k += 1;
    }

    while j < right.len() {
        result[k] = right[j].clone();
        j += 1;
        k += 1;
    }
}

// ============================================================================
// PARALLEL MERGE SORT WITH CUTOFF
// ============================================================================

const SEQUENTIAL_CUTOFF: usize = 10_000;

pub fn parallel_merge_sort<T: Ord + Clone + Send>(arr: &mut [T]) {
    if arr.len() < SEQUENTIAL_CUTOFF {
        merge_sort(arr);
        return;
    }

    let mid = arr.len() / 2;
    let (left, right) = arr.split_at_mut(mid);

    rayon::join(
        || parallel_merge_sort(left),
        || parallel_merge_sort(right),
    );

    let mut temp = arr.to_vec();
    merge(&arr[..mid], &arr[mid..], &mut temp);
    arr.copy_from_slice(&temp);
}

// ============================================================================
// QUICKSORT
// ============================================================================

pub fn quicksort<T: Ord>(arr: &mut [T]) {
    if arr.len() <= 1 {
        return;
    }

    let pivot = partition(arr);
    quicksort(&mut arr[..pivot]);
    quicksort(&mut arr[pivot + 1..]);
}

fn partition<T: Ord>(arr: &mut [T]) -> usize {
    let pivot_index = arr.len() - 1;
    let mut i = 0;

    for j in 0..arr.len() - 1 {
        if arr[j] <= arr[pivot_index] {
            arr.swap(i, j);
            i += 1;
        }
    }

    arr.swap(i, pivot_index);
    i
}

// ============================================================================
// PARALLEL QUICKSORT
// ============================================================================

const QUICKSORT_CUTOFF: usize = 5_000;

pub fn parallel_quicksort<T: Ord + Send>(arr: &mut [T]) {
    if arr.len() <= 1 {
        return;
    }

    if arr.len() < QUICKSORT_CUTOFF {
        quicksort(arr);
        return;
    }

    let pivot = partition(arr);
    let (left, right) = arr.split_at_mut(pivot);

    rayon::join(
        || parallel_quicksort(left),
        || parallel_quicksort(&mut right[1..]),
    );
}

// ============================================================================
// UTILITIES
// ============================================================================

pub fn is_sorted<T: Ord>(arr: &[T]) -> bool {
    arr.windows(2).all(|w| w[0] <= w[1])
}

// ============================================================================
// BENCHMARKING
// ============================================================================

fn main() {
    println!("=== Parallel Sorting Benchmark ===\n");

    let sizes = vec![100_000, 1_000_000, 10_000_000];

    for size in sizes {
        println!("Array size: {}", size);

        let mut arr: Vec<i32> = (0..size).rev().collect();
        let mut arr_par = arr.clone();
        let mut arr_quick = arr.clone();

        // Sequential merge sort
        let start = Instant::now();
        merge_sort(&mut arr);
        let seq_time = start.elapsed();
        println!("  Sequential merge sort: {:?}", seq_time);

        // Parallel merge sort
        let start = Instant::now();
        parallel_merge_sort(&mut arr_par);
        let par_time = start.elapsed();
        let speedup = seq_time.as_secs_f64() / par_time.as_secs_f64();
        println!("  Parallel merge sort:   {:?} ({:.2}x speedup)", par_time, speedup);

        // Parallel quicksort
        let start = Instant::now();
        parallel_quicksort(&mut arr_quick);
        let quick_time = start.elapsed();
        let speedup_quick = seq_time.as_secs_f64() / quick_time.as_secs_f64();
        println!("  Parallel quicksort:    {:?} ({:.2}x speedup)", quick_time, speedup_quick);

        println!();
    }

    // Scalability test
    println!("=== Strong Scaling (10M elements) ===\n");
    strong_scaling_test(10_000_000);
}

fn strong_scaling_test(size: usize) {
    use rayon::ThreadPoolBuilder;

    let cores = vec![1, 2, 4, 8];
    println!("Cores | Time      | Speedup | Efficiency");
    println!("------|-----------|---------|------------");

    let mut baseline = None;

    for &core_count in &cores {
        let pool = ThreadPoolBuilder::new()
            .num_threads(core_count)
            .build()
            .unwrap();

        let mut arr: Vec<i32> = (0..size).rev().collect();

        let time = pool.install(|| {
            let start = Instant::now();
            parallel_merge_sort(&mut arr);
            start.elapsed()
        });

        if baseline.is_none() {
            baseline = Some(time);
            println!("{:5} | {:?} | {:7.2}x | {:6.1}%",
                core_count, time, 1.0, 100.0);
        } else {
            let speedup = baseline.unwrap().as_secs_f64() / time.as_secs_f64();
            let efficiency = (speedup / core_count as f64) * 100.0;
            println!("{:5} | {:?} | {:7.2}x | {:6.1}%",
                core_count, time, speedup, efficiency);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_sort() {
        let mut arr = vec![5, 2, 8, 1, 9];
        merge_sort(&mut arr);
        assert_eq!(arr, vec![1, 2, 5, 8, 9]);
    }

    #[test]
    fn test_parallel_merge_sort() {
        let mut arr: Vec<i32> = (0..10000).rev().collect();
        parallel_merge_sort(&mut arr);
        assert!(is_sorted(&arr));
    }

    #[test]
    fn test_quicksort() {
        let mut arr = vec![5, 2, 8, 1, 9];
        quicksort(&mut arr);
        assert_eq!(arr, vec![1, 2, 5, 8, 9]);
    }

    #[test]
    fn test_parallel_quicksort() {
        let mut arr: Vec<i32> = (0..10000).rev().collect();
        parallel_quicksort(&mut arr);
        assert!(is_sorted(&arr));
    }
}
```

This completes Project 1: Parallel Sorting Algorithms Suite!
