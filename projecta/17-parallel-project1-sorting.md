
# Parallel Sorting Algorithms

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

## Key Concepts Explained

This project requires understanding divide-and-conquer algorithms, fork-join parallelism, work stealing, Amdahl's Law, cache effects, and parallel efficiency metrics. These concepts enable building scalable parallel algorithms that achieve near-linear speedup on multi-core systems.

### Divide-and-Conquer: The Foundation of Parallel Sorting

**What It Is**: Recursively split a problem into smaller subproblems, solve them independently, then combine results.

**The Pattern**:

```rust
fn divide_and_conquer<T>(problem: Problem<T>) -> Solution<T> {
    // Base case: problem small enough to solve directly
    if problem.is_small() {
        return solve_directly(problem);
    }

    // Divide: Split into subproblems
    let (left, right) = problem.split();

    // Conquer: Solve subproblems recursively
    let left_solution = divide_and_conquer(left);
    let right_solution = divide_and_conquer(right);

    // Combine: Merge solutions
    combine(left_solution, right_solution)
}
```

**Why Perfect for Parallelism**:

```
Sequential execution:
    [Full Problem]
         ↓
    Split (Divide)
    ↓         ↓
  [Left]    [Right]
    ↓         ↓
  Solve     Solve  ← These are independent!
    ↓         ↓
    Combine Results

Parallel execution:
    [Full Problem]
         ↓
    Split (Divide)
    ↙          ↘
  [Left]      [Right]
    ↓           ↓
  Thread 1   Thread 2  ← Execute simultaneously!
    ↓           ↓
    Combine Results

Speedup: 2x (with 2 cores, assuming equal work)
```

**Key Property**: Subproblems are **independent** (no shared state) → Can execute in parallel safely.

---

### Merge Sort: The Most Parallelizable Algorithm

**How It Works**:

```
Input: [38, 27, 43, 3, 9, 82, 10]

Divide phase (split in half recursively):
[38, 27, 43, 3, 9, 82, 10]
    ↙                    ↘
[38, 27, 43]          [3, 9, 82, 10]
  ↙      ↘              ↙         ↘
[38]  [27, 43]      [3, 9]     [82, 10]
       ↙   ↘         ↙  ↘       ↙   ↘
     [27] [43]     [3] [9]   [82] [10]

Conquer phase (merge sorted halves):
     [27] [43]     [3] [9]   [10] [82]
       ↘   ↙         ↘  ↙       ↘   ↙
     [27, 43]      [3, 9]     [10, 82]
         ↘            ↙         ↙
       [27, 43]   [3, 9, 10, 82]
              ↘        ↙
      [3, 9, 10, 27, 43, 82]
```

**Merge Algorithm**:

```rust
fn merge(left: &[T], right: &[T], output: &mut [T]) {
    let (mut i, mut j, mut k) = (0, 0, 0);

    // Compare and copy smaller element
    while i < left.len() && j < right.len() {
        if left[i] <= right[j] {
            output[k] = left[i];
            i += 1;
        } else {
            output[k] = right[j];
            j += 1;
        }
        k += 1;
    }

    // Copy remaining elements
    output[k..].copy_from_slice(&left[i..]);
    output[k..].copy_from_slice(&right[j..]);
}

// Example:
// left = [3, 27, 43], right = [9, 10, 82]
// Step 1: Compare 3 vs 9 → output[0] = 3
// Step 2: Compare 27 vs 9 → output[1] = 9
// Step 3: Compare 27 vs 10 → output[2] = 10
// Step 4: Compare 27 vs 82 → output[3] = 27
// Step 5: Compare 43 vs 82 → output[4] = 43
// Step 6: Copy remaining [82] → output[5] = 82
```

**Complexity**:
- **Time**: O(n log n) (always, regardless of input)
- **Space**: O(n) (requires temporary array)
- **Stable**: Yes (preserves relative order of equal elements)

**Why Parallelizable**:

```
Divide phase:
       [Original Array]
            ↓ Split
    ┌───────┴───────┐
  [Left]          [Right]   ← Independent!
    ↓                ↓
Split more       Split more
    ↓                ↓
Thread Pool:
  [Task 1]  [Task 2]  [Task 3]  [Task 4]

Each recursive call is independent → Fork-join parallelism!
```

**Parallel Merge Sort Pseudocode**:

```rust
fn parallel_merge_sort(arr: &mut [T], cutoff: usize) {
    if arr.len() <= cutoff {
        // Base case: sequential sort
        sequential_merge_sort(arr);
        return;
    }

    let mid = arr.len() / 2;
    let (left, right) = arr.split_at_mut(mid);

    // Fork: Spawn parallel tasks
    rayon::join(
        || parallel_merge_sort(left, cutoff),   // Thread 1
        || parallel_merge_sort(right, cutoff),  // Thread 2
    );

    // Join: Merge results
    merge(left, right, arr);
}
```

---

### Quicksort: In-Place Divide-and-Conquer

**How It Works**:

```
Input: [38, 27, 43, 3, 9, 82, 10]

Step 1: Choose pivot (e.g., last element: 10)
[38, 27, 43, 3, 9, 82, 10]
                        ↑ pivot

Step 2: Partition (elements < pivot on left, ≥ pivot on right)
[3, 9, 10, 27, 43, 82, 38]
      ↑ pivot now in correct position

Step 3: Recursively sort left and right partitions
Left: [3, 9]  →  [3, 9] (already sorted)
Right: [27, 43, 82, 38]
  Pick pivot: 38
  Partition: [27, 38, 82, 43]
  Left: [27] (done)
  Right: [82, 43] → [43, 82]

Final: [3, 9, 10, 27, 38, 43, 82]
```

**Partition Algorithm (Hoare's)**:

```rust
fn partition(arr: &mut [T]) -> usize {
    let pivot_index = arr.len() / 2;
    let pivot = arr[pivot_index];

    let mut i = 0;
    let mut j = arr.len() - 1;

    loop {
        // Find element >= pivot from left
        while arr[i] < pivot { i += 1; }

        // Find element < pivot from right
        while arr[j] > pivot { j -= 1; }

        if i >= j {
            return j;  // Partition index
        }

        // Swap elements
        arr.swap(i, j);
        i += 1;
        j -= 1;
    }
}

// Example:
// [38, 27, 43, 3, 9, 82, 10], pivot = 43
// i=0, j=6: swap 38 ↔ 10: [10, 27, 43, 3, 9, 82, 38]
// i=2, j=4: 43 >= 43, 9 < 43: swap: [10, 27, 9, 3, 43, 82, 38]
// i=4, j=3: i >= j, return 3
// Result: [10, 27, 9, 3 | 43, 82, 38]
//                      ↑ partition point
```

**Complexity**:
- **Time**: O(n log n) average, O(n²) worst case (already sorted)
- **Space**: O(log n) (recursion stack)
- **Stable**: No (relative order not preserved)
- **In-place**: Yes (no extra array needed)

**Why Harder to Parallelize**:

Problem: Partition phase is **sequential** (single-threaded bottleneck).

```
Sequential bottleneck:
[Large Array]
      ↓
   Partition  ← Single thread, O(n) work
      ↓
[Left] | [Right]
   ↓        ↓
 Parallel sorting works here

Amdahl's Law applies:
- Partition: 30% of time (sequential)
- Recursive sorts: 70% of time (parallel)
- Max speedup with infinite cores: 1 / 0.3 = 3.3x
```

**Parallel Quicksort Strategy**:

```rust
fn parallel_quicksort(arr: &mut [T], cutoff: usize) {
    if arr.len() <= cutoff {
        sequential_quicksort(arr);
        return;
    }

    // Sequential partition (unavoidable)
    let pivot = partition(arr);

    let (left, right) = arr.split_at_mut(pivot);

    // Fork: Parallel sort of partitions
    rayon::join(
        || parallel_quicksort(left, cutoff),
        || parallel_quicksort(right, cutoff),
    );
}
```

---

### Fork-Join Parallelism with Rayon

**What Is Fork-Join?**

A parallel programming model where:
1. **Fork**: Spawn parallel tasks
2. **Work**: Execute tasks concurrently
3. **Join**: Wait for all tasks to complete

**Rayon's `join` Function**:

```rust
use rayon::join;

let (result_a, result_b) = join(
    || compute_a(),  // Task A (may run on any thread)
    || compute_b(),  // Task B (may run on any thread)
);
// Blocks until both complete

// One of the tasks runs on the current thread (no overhead)
// The other may be stolen by an idle worker
```

**How It Works Internally**:

```
Thread 1 (current):              Work-Stealing Thread Pool:
  join(task_a, task_b)
    ↓
  Push task_b to deque  ────────→ Thread 2 (idle) steals task_b
    ↓                                        ↓
  Execute task_a                       Execute task_b
    ↓                                        ↓
  Check if task_b done? ←──────────────── task_b finishes
    ↓ No
  Help execute task_b (work stealing)
    ↓
  Both done, return (result_a, result_b)
```

**Work Stealing**:

```
Thread Pool (4 threads):
Thread 0: [Task1][Task2][Task3]  ← Busy
Thread 1: [Task4]                ← Finished early
Thread 2: [Task5][Task6]         ← Busy
Thread 3: []                     ← Idle

Thread 1 steals from Thread 0:
Thread 0: [Task1][Task2]         ← Lost Task3
Thread 1: [Task4][Task3]         ← Stole Task3
Thread 2: [Task5][Task6]
Thread 3: []                     ← Steals next...

Result: Balanced load, high CPU utilization
```

**Parallel Recursion with Rayon**:

```rust
use rayon::join;

fn parallel_sum(arr: &[i32]) -> i32 {
    const CUTOFF: usize = 1000;

    if arr.len() <= CUTOFF {
        return arr.iter().sum();  // Sequential base case
    }

    let mid = arr.len() / 2;
    let (left, right) = arr.split_at(mid);

    let (left_sum, right_sum) = join(
        || parallel_sum(left),   // Fork left
        || parallel_sum(right),  // Fork right
    );

    left_sum + right_sum  // Join results
}

// Recursion tree (8 elements, cutoff=2):
//        [0..8]
//       /      \
//    [0..4]    [4..8]
//    /   \      /   \
// [0..2][2..4][4..6][6..8]

// With 4 cores: All leaf tasks execute in parallel
```

---

### Sequential Cutoff: When to Stop Parallelizing

**The Problem**: Parallelism has overhead (task creation, scheduling, synchronization).

**Overhead Breakdown**:

```
Parallel task overhead:
- Task creation: ~100-500ns
- Thread wake-up: ~1-10μs
- Cache synchronization: ~10-100ns

Sequential quicksort:
- 1000 elements: ~50μs
- 100 elements: ~5μs
- 10 elements: ~0.5μs

If overhead > work, parallelism is slower!
```

**Cutoff Strategy**:

```rust
const CUTOFF: usize = 10_000;  // Empirically determined

fn parallel_merge_sort(arr: &mut [T]) {
    if arr.len() <= CUTOFF {
        // Too small: use fast sequential sort
        arr.sort_unstable();  // ~50μs for 10K elements
        return;
    }

    // Large enough: parallelize
    let mid = arr.len() / 2;
    let (left, right) = arr.split_at_mut(mid);

    rayon::join(
        || parallel_merge_sort(left),   // Overhead: ~500ns
        || parallel_merge_sort(right),  // Work: ~1ms
    );                                   // Overhead << Work ✓

    merge(left, right, arr);
}
```

**Finding Optimal Cutoff**:

```
Benchmark results (8-core machine):
Cutoff     Time      Speedup
100        2.5s      0.8x  (overhead dominates)
1,000      1.2s      1.7x
10,000     0.35s     5.7x  ← Sweet spot!
100,000    0.45s     4.4x  (insufficient parallelism)
1,000,000  1.8s      1.1x  (almost sequential)

Optimal cutoff: 10,000 elements
```

**Rule of Thumb**:
- **Cutoff too small**: Overhead dominates, slowdown
- **Cutoff too large**: Insufficient parallelism, underutilization
- **Optimal**: Work >> overhead, enough tasks to saturate cores

---

### Amdahl's Law: The Speedup Ceiling

**The Law**: Maximum speedup limited by sequential portion.

**Formula**:

```
Speedup = 1 / (S + P / N)

Where:
  S = Sequential fraction (0 to 1)
  P = Parallel fraction (= 1 - S)
  N = Number of cores
```

**Example**:

```
Algorithm with 10% sequential code (S = 0.1):

1 core:   Speedup = 1 / (0.1 + 0.9 / 1)  = 1.0x
2 cores:  Speedup = 1 / (0.1 + 0.9 / 2)  = 1.8x
4 cores:  Speedup = 1 / (0.1 + 0.9 / 4)  = 3.1x
8 cores:  Speedup = 1 / (0.1 + 0.9 / 8)  = 4.7x
∞ cores:  Speedup = 1 / 0.1              = 10x  ← Maximum!

With 10% sequential code, max speedup = 10x (regardless of cores)
```

**Visual Representation**:

```
Execution time breakdown:
Sequential (1 core):
[████████████████████████████████] 100% time

Parallel (8 cores):
[██████] Sequential (10% time)
[███] Parallel (90% / 8 = 11.25% time)
Total: 21.25% time → Speedup = 4.7x

Cannot eliminate the sequential portion!
```

**Merge Sort Amdahl's Law**:

```
Merge sort phases:
- Divide: O(log n) - Sequential (splitting array)
- Conquer: O(n log n) - Parallelizable (sorting subproblems)
- Merge: O(n) - Sequential (combining results)

For 100M elements:
- Divide: ~0.05s (2%)
- Conquer: 1.8s (86%)
- Merge: 0.25s (12%)

Sequential fraction S = 0.02 + 0.12 = 0.14

Max speedup = 1 / 0.14 ≈ 7.1x

With 8 cores:
Actual speedup = 1 / (0.14 + 0.86 / 8) ≈ 5.6x

Close to maximum, but not 8x!
```

---

### Parallel Efficiency and Scalability

**Parallel Efficiency**: How well we use additional cores.

**Formula**:

```
Efficiency = Speedup / Cores

Example:
8 cores, 6x speedup:
Efficiency = 6 / 8 = 75%

Meaning: 75% of potential parallel speedup achieved
25% lost to overhead, synchronization, sequential work
```

**Scalability Measures**:

**Strong Scaling**: Fixed problem size, increase cores.

```
Problem: Sort 100M elements
1 core:  2.0s  → Speedup 1.0x, Efficiency 100%
2 cores: 1.1s  → Speedup 1.8x, Efficiency 90%
4 cores: 0.6s  → Speedup 3.3x, Efficiency 82%
8 cores: 0.35s → Speedup 5.7x, Efficiency 71%

Efficiency decreases with more cores (Amdahl's Law)
```

**Weak Scaling**: Problem size grows with cores.

```
Keep work per core constant (12.5M elements each):
1 core:  12.5M elements → 0.25s
2 cores: 25M elements   → 0.26s  (1.04x time)
4 cores: 50M elements   → 0.28s  (1.12x time)
8 cores: 100M elements  → 0.32s  (1.28x time)

Near-constant time → Good weak scaling
```

**Target Metrics**:
- **60-80% efficiency**: Excellent for real algorithms
- **80-90% efficiency**: Ideal (rare in practice)
- **<50% efficiency**: Poor (overhead too high)

---

### Cache Effects in Parallel Sorting

**Why Caches Matter**:

```
Memory hierarchy:
L1 Cache:  32KB,  ~1ns latency   (per core)
L2 Cache:  256KB, ~3ns latency   (per core)
L3 Cache:  8MB,   ~10ns latency  (shared)
RAM:       16GB,  ~100ns latency (shared)

Sorting is memory-intensive:
- 100M integers = 400MB
- Doesn't fit in any cache!
- Cache misses dominate performance
```

**Cache Behavior**:

**Sequential Access (Cache-Friendly)**:
```rust
// Good: Sequential scan
for i in 0..arr.len() {
    sum += arr[i];  // Predictable, prefetcher works
}

// Cache hit rate: 95%+
// Time: 0.1s for 100M elements
```

**Random Access (Cache-Hostile)**:
```rust
// Bad: Random jumps
for _ in 0..arr.len() {
    let i = random_index();
    sum += arr[i];  // Unpredictable, cache misses
}

// Cache hit rate: <50%
// Time: 2-3s for 100M elements (20-30x slower!)
```

**Merge Sort Cache Behavior**:

```
Problem: Merge requires reading entire left and right arrays

Cache-unfriendly merge (100M elements):
[Left 50M] [Right 50M]
   ↓           ↓
  [Merged 100M]

50M elements = 200MB (exceeds L3 cache)
→ Many cache misses
→ RAM bandwidth bottleneck

Optimization: Block-based merge
- Merge in cache-sized chunks (256KB = 64K elements)
- Reduces cache misses by 80%
```

**Quicksort Cache Behavior**:

```
Advantage: In-place, better locality

Partitioning scans array once:
[38, 27, 43, 3, 9, 82, 10]
 ↑→→→→→→→→→→→→→→→→→→→→→↑
  Sequential scan, cache-friendly

After partition:
[3, 9, 10 | 27, 43, 82, 38]
    ↓          ↓
  Smaller    Smaller
  chunks     chunks

Smaller chunks fit in cache → Faster
```

**False Sharing (Parallel Sorting)**:

```
Bad: Threads writing to adjacent array elements

Thread 0: arr[0..1000]  }
Thread 1: arr[1000..2000] } ← Same cache line!

Cache line size: 64 bytes = 16 integers

Thread 0 writes arr[1000]:
→ Invalidates Thread 1's cache line
→ Thread 1 must reload
→ Ping-pong continues
→ 10-100x slowdown

Solution: Padding or working on larger chunks
```

---

### Radix Sort: A Different Parallelism Model

**How It Works**: Sort by digit, from least significant to most significant.

```
Input: [170, 45, 75, 90, 802, 24, 2, 66]

Pass 1: Sort by ones digit
  0: [170, 90]
  2: [802, 2]
  4: [24]
  5: [45, 75]
  6: [66]
→ [170, 90, 802, 2, 24, 45, 75, 66]

Pass 2: Sort by tens digit
  0: [802, 2]
  2: [24]
  4: [45]
  6: [66]
  7: [170, 75]
  9: [90]
→ [802, 2, 24, 45, 66, 170, 75, 90]

Pass 3: Sort by hundreds digit
  0: [2, 24, 45, 66, 75, 90]
  1: [170]
  8: [802]
→ [2, 24, 45, 66, 75, 90, 170, 802]  ✓
```

**Complexity**:
- **Time**: O(d × n), where d = number of digits
- **Space**: O(n + k), where k = range (0-9 for decimal)
- **Stable**: Yes
- **Best for**: Integers, strings with bounded length

**Parallel Radix Sort**:

```
Challenge: Each pass depends on previous pass (sequential dependency)

Pass 1: [All threads]    Sort by ones digit
   ↓ Barrier (must complete)
Pass 2: [All threads]    Sort by tens digit
   ↓ Barrier
Pass 3: [All threads]    Sort by hundreds digit

Within each pass:
1. Parallel counting (histogram)
2. Sequential prefix sum (small, fast)
3. Parallel placement

Speedup limited by sequential portions (barriers, prefix sum)
```

---

### Connection to This Project

Now that you understand the core concepts, here's how they map to the milestones:

**Milestone 1: Sequential Baseline**
- **Concepts Used**: Divide-and-conquer, merge algorithm, partition algorithm
- **Why**: Establish baseline performance before parallelization
- **Key Insight**: Understanding sequential bottlenecks guides parallelization strategy

**Milestone 2: Basic Fork-Join Parallelism**
- **Concepts Used**: Rayon `join`, fork-join model, work stealing
- **Why**: Simplest parallelization—split work, join results
- **Key Insight**: `join` runs one task inline, other may be stolen (no wasted thread)

**Milestone 3: Sequential Cutoff Optimization**
- **Concepts Used**: Overhead analysis, cutoff threshold, sequential fallback
- **Why**: Avoid parallelizing small tasks where overhead > work
- **Key Insight**: Empirical benchmarking finds optimal cutoff (typically 1K-10K elements)

**Milestone 4: Amdahl's Law Validation**
- **Concepts Used**: Speedup calculation, parallel efficiency, sequential fraction measurement
- **Why**: Understand theoretical limits of parallelization
- **Key Insight**: 10% sequential code limits speedup to 10x, regardless of cores

**Milestone 5: Cache Optimization**
- **Concepts Used**: Cache hierarchy, locality, false sharing, blocking
- **Why**: Memory bandwidth often bottleneck, not CPU
- **Key Insight**: Cache-friendly algorithms (quicksort) scale better than cache-hostile (merge sort)

**Milestone 6: Multi-Algorithm Comparison**
- **Concepts Used**: Merge sort, quicksort, radix sort trade-offs
- **Why**: Different algorithms excel in different scenarios
- **Key Insight**: No "best" algorithm—choose based on data characteristics and constraints

**Putting It All Together**:

The complete parallel sorting library demonstrates:
1. **Divide-and-conquer** enables natural parallelism
2. **Fork-join with Rayon** provides simple, efficient parallelization
3. **Sequential cutoffs** balance parallelism vs overhead
4. **Amdahl's Law** explains why 8 cores → 5-6x speedup (not 8x)
5. **Cache effects** make quicksort faster than merge sort despite worse parallelism
6. **Work stealing** automatically balances load across cores

This architecture achieves:
- **5-7x speedup on 8 cores** (60-80% efficiency)
- **100M elements sorted in 0.3s** (vs 2s sequential)
- **Scalability to 100M+ elements** with bounded memory
- **Robust performance** across random, sorted, reverse data distributions

Each milestone builds understanding from sequential baselines to production-ready parallel sorting with proper tuning and performance analysis.

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
