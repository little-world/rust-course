# Chapter 10: Vec & Slice Manipulation - Project 2

## Project 2: Time-Series Data Analyzer with Sliding Windows

### Problem Statement

Build a time-series data analyzer that computes statistics over sliding windows of data. The analyzer processes sensor readings, financial data, or metrics streams and computes aggregates (moving averages, min/max, standard deviation) using efficient windowing algorithms with zero-copy slicing.

Your analyzer should support:
- Multiple window sizes (e.g., 10-second, 1-minute, 5-minute windows)
- Computing statistics: moving average, min, max, median, percentiles
- Handling streaming data (process data as it arrives)
- Using efficient algorithms: O(n) sliding window, not O(n*w)
- Providing zero-copy views into data windows
- Detecting anomalies (values outside expected range)

Example workflow:
```
Input: Sensor readings stream (temperature, every 1 second)
Windows: [10s, 60s, 300s]
Operations: Compute average, min, max, std_dev per window
Anomaly detection: Flag readings > 3 std_dev from mean
Output: Real-time statistics and anomaly alerts
```

---

### Milestone 1: Basic Sliding Window with VecDeque

**Goal**: Implement fixed-size sliding window that maintains recent N elements.

**What to implement**:
- Use `VecDeque` for efficient push/pop from both ends
- `push()` adds element, `pop_front()` if window full
- Compute basic statistics (average, min, max)
- Provide slice view of window contents

**Architecture**:
- Structs: `SlidingWindow<T>`
- Fields: `window: VecDeque<T>`, `capacity: usize`
- Functions:
  - `new(capacity: usize) -> Self` - Create window
  - `push(value: T) -> Option<T>` - Add value, return evicted
  - `as_slice() -> &[T]` - Zero-copy view
  - `len() -> usize` - Current size
  - `is_full() -> bool` - Check capacity
  - `average() -> Option<f64>` - Compute mean (for f64 windows)
  - `min() -> Option<f64>` - Find minimum
  - `max() -> Option<f64>` - Find maximum

---

**Starter Code**:

```rust
use std::collections::VecDeque;

/// Fixed-size sliding window
/// Role: Maintain most recent N values
#[derive(Debug, Clone)]
pub struct SlidingWindow<T> {
    window: VecDeque<T>,               // Circular buffer  
    capacity: usize,                   // Maximum window size  
}

impl<T: Clone> SlidingWindow<T> {
    /// Create new sliding window
    /// Role: Initialize with capacity
    pub fn new(capacity: usize) -> Self {
        todo!("Create VecDeque with capacity")
    }

    /// Add value to window
    /// Role: Maintain FIFO ordering
    pub fn push(&mut self, value: T) -> Option<T> {
        todo!("Pop front if full, push back value")
    }

    /// Get slice view of window
    /// Role: Zero-copy access to data
    pub fn as_slice(&self) -> &[T] {
        todo!("Use make_contiguous or as_slices")
    }

    /// Current number of elements
    /// Role: Query window fill
    pub fn len(&self) -> usize {
        self.window.len()
    }

    /// Check if window is full
    /// Role: Determine if at capacity
    pub fn is_full(&self) -> bool {
        self.window.len() == self.capacity
    }

    /// Check if window is empty
    /// Role: Guard against empty statistics
    pub fn is_empty(&self) -> bool {
        self.window.is_empty()
    }
}

/// Statistics for numeric windows
impl SlidingWindow<f64> {
    /// Compute average
    /// Role: Basic statistic over window
    pub fn average(&self) -> Option<f64> {
        todo!("Sum all values, divide by length")
    }

    /// Find minimum value
    /// Role: Window minimum
    pub fn min(&self) -> Option<f64> {
        todo!("Use iterator min_by with partial_cmp")
    }

    /// Find maximum value
    /// Role: Window maximum
    pub fn max(&self) -> Option<f64> {
        todo!("Use iterator max_by with partial_cmp")
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
    fn test_create_window() {
        let window: SlidingWindow<f64> = SlidingWindow::new(10);
        assert_eq!(window.len(), 0);
        assert!(!window.is_full());
        assert!(window.is_empty());
    }

    #[test]
    fn test_push_values() {
        let mut window = SlidingWindow::new(3);

        assert_eq!(window.push(1.0), None);
        assert_eq!(window.len(), 1);

        assert_eq!(window.push(2.0), None);
        assert_eq!(window.len(), 2);

        assert_eq!(window.push(3.0), None);
        assert_eq!(window.len(), 3);
        assert!(window.is_full());
    }

    #[test]
    fn test_window_eviction() {
        let mut window = SlidingWindow::new(3);

        window.push(1.0);
        window.push(2.0);
        window.push(3.0);

        // Window is full, next push should evict oldest
        let evicted = window.push(4.0);
        assert_eq!(evicted, Some(1.0));
        assert_eq!(window.len(), 3);

        let evicted = window.push(5.0);
        assert_eq!(evicted, Some(2.0));
    }

    #[test]
    fn test_window_fifo_order() {
        let mut window = SlidingWindow::new(3);

        window.push(10.0);
        window.push(20.0);
        window.push(30.0);
        window.push(40.0); // Evicts 10.0

        let slice = window.as_slice();
        assert_eq!(slice, &[20.0, 30.0, 40.0]);
    }

    #[test]
    fn test_average() {
        let mut window = SlidingWindow::new(5);

        assert_eq!(window.average(), None); // Empty

        window.push(10.0);
        assert_eq!(window.average(), Some(10.0));

        window.push(20.0);
        assert_eq!(window.average(), Some(15.0));

        window.push(30.0);
        assert_eq!(window.average(), Some(20.0));
    }

    #[test]
    fn test_min_max() {
        let mut window = SlidingWindow::new(5);

        assert_eq!(window.min(), None);
        assert_eq!(window.max(), None);

        window.push(30.0);
        window.push(10.0);
        window.push(50.0);
        window.push(20.0);

        assert_eq!(window.min(), Some(10.0));
        assert_eq!(window.max(), Some(50.0));
    }

    #[test]
    fn test_min_max_after_eviction() {
        let mut window = SlidingWindow::new(3);

        window.push(10.0);
        window.push(50.0); // Max
        window.push(30.0);

        assert_eq!(window.max(), Some(50.0));

        window.push(20.0); // Evicts 10.0
        assert_eq!(window.max(), Some(50.0));

        window.push(25.0); // Evicts 50.0 (the max!)
        assert_eq!(window.max(), Some(30.0));
    }

    #[test]
    fn test_as_slice_zero_copy() {
        let mut window = SlidingWindow::new(100);

        for i in 0..50 {
            window.push(i as f64);
        }

        let slice1 = window.as_slice();
        let slice2 = window.as_slice();

        // Should be same pointer (zero-copy)
        assert_eq!(slice1.as_ptr(), slice2.as_ptr());
    }
}
```

---

### Milestone 2: Incremental Statistics (Avoid Re-Scanning)

**Goal**: Maintain running sum to compute average in O(1) instead of O(n).

**Why the previous milestone is not enough**: Milestone 1 computes average by summing entire window on every call (O(n)). For a stream of 1M values with window size 1000, this is 1 billion operations.

**What's the improvement**: Incremental updates reduce average computation from O(n) to O(1). Instead of summing 1000 values per update, we add one and subtract one. For 1M updates:
- Before: 1M × 1000 = 1 billion operations
- After: 1M × 2 = 2 million operations (500x faster)

**Optimization focus**: Speed through algorithmic improvement (O(n) → O(1)).

**Architecture**:
- Structs: `IncrementalWindow`
- Fields: `window: VecDeque<f64>`, `capacity: usize`, `running_sum: f64`, `running_sum_sq: f64`
- Functions:
  - `new(capacity: usize) -> Self` - Create window
  - `push(value: f64)` - Update with new value
  - `average() -> Option<f64>` - O(1) mean
  - `variance() -> Option<f64>` - O(1) variance
  - `std_dev() -> Option<f64>` - O(1) standard deviation

---

**Starter Code**:

```rust
/// Sliding window with incremental statistics
/// Role: O(1) statistics computation
#[derive(Debug, Clone)]
pub struct IncrementalWindow {
    window: VecDeque<f64>,                // Data storage                          
    capacity: usize,                      // Maximum size                                
    running_sum: f64,                     // Sum of all values                          
    running_sum_sq: f64,                  // Sum of squared values (for variance)    
}

impl IncrementalWindow {
    /// Create new incremental window
    /// Role: Initialize with zero statistics
    pub fn new(capacity: usize) -> Self {
        todo!("Initialize all fields to zero/empty")
    }

    /// Add value with incremental update
    /// Role: Maintain running statistics
    pub fn push(&mut self, value: f64) {
        todo!("Evict old, update running sums, push new")
    }

    /// Get mean in O(1)
    /// Role: Fast average computation
    pub fn average(&self) -> Option<f64> {
        todo!("Return running_sum / len")
    }

    /// Get variance in O(1)
    /// Role: Fast variance using sum of squares
    pub fn variance(&self) -> Option<f64> {
        todo!("Use Var(X) = E[X²] - E[X]²")
    }

    /// Get standard deviation in O(1)
    /// Role: Square root of variance
    pub fn std_dev(&self) -> Option<f64> {
        todo!("Return sqrt(variance)")
    }

    /// Get current length
    /// Role: Query window size
    pub fn len(&self) -> usize {
        self.window.len()
    }

    /// Check if empty
    /// Role: Guard for statistics
    pub fn is_empty(&self) -> bool {
        self.window.is_empty()
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
    fn test_incremental_average() {
        let mut window = IncrementalWindow::new(5);

        window.push(10.0);
        assert_eq!(window.average(), Some(10.0));

        window.push(20.0);
        assert_eq!(window.average(), Some(15.0));

        window.push(30.0);
        assert_eq!(window.average(), Some(20.0));
    }

    #[test]
    fn test_incremental_average_after_eviction() {
        let mut window = IncrementalWindow::new(3);

        window.push(10.0);
        window.push(20.0);
        window.push(30.0);
        assert_eq!(window.average(), Some(20.0));

        window.push(40.0); // Evicts 10.0
        // Window now: [20, 30, 40], avg = 30
        assert_eq!(window.average(), Some(30.0));
    }

    #[test]
    fn test_variance_calculation() {
        let mut window = IncrementalWindow::new(5);

        window.push(2.0);
        window.push(4.0);
        window.push(4.0);
        window.push(4.0);
        window.push(5.0);

        // Mean = 3.8, Variance = 0.96
        let variance = window.variance().unwrap();
        assert!((variance - 0.96).abs() < 0.01);
    }

    #[test]
    fn test_std_dev_calculation() {
        let mut window = IncrementalWindow::new(5);

        window.push(2.0);
        window.push(4.0);
        window.push(4.0);
        window.push(4.0);
        window.push(5.0);

        // Std dev = sqrt(0.96) ≈ 0.98
        let std_dev = window.std_dev().unwrap();
        assert!((std_dev - 0.98).abs() < 0.01);
    }

    #[test]
    fn test_incremental_vs_naive() {
        // Verify incremental matches naive computation
        let mut window = IncrementalWindow::new(100);

        let values: Vec<f64> = (0..100).map(|i| i as f64 * 1.5).collect();

        for &v in &values {
            window.push(v);
        }

        let incremental_avg = window.average().unwrap();

        // Naive average
        let naive_avg = values.iter().sum::<f64>() / values.len() as f64;

        assert!((incremental_avg - naive_avg).abs() < 0.0001);
    }

    #[test]
    fn test_performance_incremental() {
        use std::time::Instant;

        let mut window = IncrementalWindow::new(1000);

        // Fill window
        for i in 0..1000 {
            window.push(i as f64);
        }

        // Measure average computation time
        let iterations = 100000;
        let start = Instant::now();

        for _ in 0..iterations {
            let _ = window.average();
        }

        let elapsed = start.elapsed();

        println!("Time for {} incremental averages: {:?}", iterations, elapsed);

        // Should be very fast (microseconds for 100K operations)
        assert!(elapsed.as_millis() < 100);
    }

    #[test]
    fn test_variance_empty_window() {
        let window = IncrementalWindow::new(10);
        assert_eq!(window.variance(), None);
    }

    #[test]
    fn test_variance_single_value() {
        let mut window = IncrementalWindow::new(10);
        window.push(5.0);

        // Variance of single value is undefined or zero
        // (Implementation choice - we return None for < 2 values)
        assert_eq!(window.variance(), None);
    }
}
```

---

### Milestone 3: Min/Max with Monotonic Deque

**Goal**: Maintain min/max in O(1) amortized time using monotonic deque.

**Why the previous milestone is not enough**: Finding min/max requires scanning window (O(n)). For 1M updates with window 1000, this is 1 billion operations.

**What's the improvement**: Monotonic deque maintains min/max in O(1) amortized time. Algorithm keeps only elements that could be min/max in future:
- For min: if new element is smaller than back of deque, pop back (it can never be min)
- Front of deque is always current min

Complexity: Each element pushed once, popped at most once → O(1) amortized.

**Optimization focus**: Speed through clever data structure (O(n) → O(1) amortized).

**Architecture**:
- Structs: `MinMaxWindow`
- Fields: `window: VecDeque<(usize, f64)>`, `min_deque: VecDeque<(usize, f64)>`, `max_deque: VecDeque<(usize, f64)>`, `capacity: usize`, `index: usize`
- Functions:
  - `new(capacity: usize) -> Self` - Create window
  - `push(value: f64)` - Add value with deque maintenance
  - `min() -> Option<f64>` - O(1) minimum
  - `max() -> Option<f64>` - O(1) maximum

---

**Starter Code**:

```rust
/// Sliding window with O(1) min/max
/// Role: Efficient min/max tracking
#[derive(Debug)]
pub struct MinMaxWindow {
    window: VecDeque<(usize, f64)>,             // Values with indices    
    min_deque: VecDeque<(usize, f64)>,          // Monotonic increasing
    max_deque: VecDeque<(usize, f64)>,          // Monotonic decreasing
    capacity: usize,                            // Maximum size                          
    index: usize,                               // Global index counter                     
}

impl MinMaxWindow {
    /// Create new min/max window
    /// Role: Initialize deques
    pub fn new(capacity: usize) -> Self {
        todo!("Initialize all deques")
    }

    /// Add value with monotonic deque update
    /// Role: Maintain min/max invariants
    pub fn push(&mut self, value: f64) {
        todo!("Evict old, update min/max deques, push new")
    }

    /// Get minimum in O(1)
    /// Role: Return front of min_deque
    pub fn min(&self) -> Option<f64> {
        todo!("Return min_deque front value")
    }

    /// Get maximum in O(1)
    /// Role: Return front of max_deque
    pub fn max(&self) -> Option<f64> {
        todo!("Return max_deque front value")
    }

    /// Get current length
    /// Role: Query window size
    pub fn len(&self) -> usize {
        self.window.len()
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
    fn test_min_max_basic() {
        let mut window = MinMaxWindow::new(5);

        window.push(30.0);
        assert_eq!(window.min(), Some(30.0));
        assert_eq!(window.max(), Some(30.0));

        window.push(10.0);
        assert_eq!(window.min(), Some(10.0));
        assert_eq!(window.max(), Some(30.0));

        window.push(50.0);
        assert_eq!(window.min(), Some(10.0));
        assert_eq!(window.max(), Some(50.0));
    }

    #[test]
    fn test_min_max_after_eviction() {
        let mut window = MinMaxWindow::new(3);

        window.push(10.0); // min
        window.push(30.0);
        window.push(20.0);

        assert_eq!(window.min(), Some(10.0));

        window.push(40.0); // Evicts 10.0
        // Window: [30, 20, 40]
        assert_eq!(window.min(), Some(20.0));
        assert_eq!(window.max(), Some(40.0));
    }

    #[test]
    fn test_max_eviction() {
        let mut window = MinMaxWindow::new(3);

        window.push(50.0); // max
        window.push(30.0);
        window.push(20.0);

        assert_eq!(window.max(), Some(50.0));

        window.push(25.0); // Evicts 50.0
        // Window: [30, 20, 25]
        assert_eq!(window.max(), Some(30.0));
    }

    #[test]
    fn test_monotonic_sequence() {
        let mut window = MinMaxWindow::new(5);

        // Increasing sequence
        for i in 1..=5 {
            window.push(i as f64);
        }

        assert_eq!(window.min(), Some(1.0));
        assert_eq!(window.max(), Some(5.0));

        // Continue increasing
        window.push(6.0); // Evicts 1.0
        assert_eq!(window.min(), Some(2.0));
        assert_eq!(window.max(), Some(6.0));
    }

    #[test]
    fn test_all_same_values() {
        let mut window = MinMaxWindow::new(5);

        for _ in 0..10 {
            window.push(42.0);
        }

        assert_eq!(window.min(), Some(42.0));
        assert_eq!(window.max(), Some(42.0));
    }

    #[test]
    fn test_alternating_values() {
        let mut window = MinMaxWindow::new(4);

        window.push(10.0);
        window.push(50.0);
        window.push(10.0);
        window.push(50.0);

        assert_eq!(window.min(), Some(10.0));
        assert_eq!(window.max(), Some(50.0));
    }

    #[test]
    fn test_performance_vs_naive() {
        use std::time::Instant;

        let values: Vec<f64> = (0..10000).map(|i| (i % 100) as f64).collect();

        // Monotonic deque approach
        let mut window = MinMaxWindow::new(100);
        let start = Instant::now();

        for &v in &values {
            window.push(v);
            let _ = window.min();
            let _ = window.max();
        }

        let deque_time = start.elapsed();

        // Naive approach (for comparison)
        let mut naive_window = SlidingWindow::new(100);
        let start = Instant::now();

        for &v in &values {
            naive_window.push(v);
            let _ = naive_window.min();
            let _ = naive_window.max();
        }

        let naive_time = start.elapsed();

        println!("Monotonic deque: {:?}", deque_time);
        println!("Naive approach: {:?}", naive_time);

        // Monotonic deque should be significantly faster
        assert!(deque_time < naive_time);
    }

    #[test]
    fn test_empty_window() {
        let window = MinMaxWindow::new(10);
        assert_eq!(window.min(), None);
        assert_eq!(window.max(), None);
    }
}
```

---

### Milestone 4: Median and Percentiles with select_nth_unstable

**Goal**: Compute median efficiently using quickselect algorithm.

**Why the previous milestone is not enough**: We have mean, min, max but not median or percentiles. Naive approach sorts entire window (O(n log n)).

**What's the improvement**: Quickselect finds k-th element in O(n) average time, faster than sorting:
- Sorting: O(n log n) ≈ 10,000 ops for n=1000
- Quickselect: O(n) ≈ 1,000 ops (10x faster)

For streaming percentiles, this is significant. Note: median requires copying window (can't be incremental like mean).

**Optimization focus**: Speed through better algorithm (O(n log n) → O(n)).

**Architecture**:
- Add methods to `IncrementalWindow`:
  - `median() -> Option<f64>` - 50th percentile
  - `percentile(p: f64) -> Option<f64>` - Any percentile

---

**Starter Code**:

```rust
impl IncrementalWindow {
    /// Compute median using quickselect
    /// Role: O(n) median calculation
    pub fn median(&self) -> Option<f64> {
        todo!("Copy to temp buffer, use select_nth_unstable")
    }

    /// Compute arbitrary percentile
    /// Role: Find p-th percentile (0-100)
    pub fn percentile(&self, p: f64) -> Option<f64> {
        todo!("Validate p, calculate index, use select_nth_unstable")
    }

    /// Get multiple percentiles efficiently
    /// Role: Compute p50, p95, p99 in one pass
    pub fn percentiles(&self, ps: &[f64]) -> Vec<Option<f64>> {
        todo!("Sort once, extract multiple percentiles")
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
    fn test_median_odd_count() {
        let mut window = IncrementalWindow::new(10);

        window.push(1.0);
        window.push(3.0);
        window.push(2.0);

        // Sorted: [1, 2, 3], median = 2
        assert_eq!(window.median(), Some(2.0));
    }

    #[test]
    fn test_median_even_count() {
        let mut window = IncrementalWindow::new(10);

        window.push(1.0);
        window.push(2.0);
        window.push(3.0);
        window.push(4.0);

        // Sorted: [1, 2, 3, 4], median = (2 + 3) / 2 = 2.5
        assert_eq!(window.median(), Some(2.5));
    }

    #[test]
    fn test_percentile_basic() {
        let mut window = IncrementalWindow::new(10);

        for i in 1..=10 {
            window.push(i as f64);
        }

        // p0 = 1, p50 = 5.5, p100 = 10
        assert_eq!(window.percentile(0.0), Some(1.0));
        assert_eq!(window.percentile(100.0), Some(10.0));

        let p50 = window.percentile(50.0).unwrap();
        assert!((p50 - 5.5).abs() < 0.1);
    }

    #[test]
    fn test_percentile_p95() {
        let mut window = IncrementalWindow::new(100);

        for i in 1..=100 {
            window.push(i as f64);
        }

        let p95 = window.percentile(95.0).unwrap();
        // p95 of 1..100 should be around 95
        assert!((p95 - 95.0).abs() < 2.0);
    }

    #[test]
    fn test_percentile_invalid_range() {
        let mut window = IncrementalWindow::new(10);
        window.push(5.0);

        assert_eq!(window.percentile(-1.0), None);
        assert_eq!(window.percentile(101.0), None);
    }

    #[test]
    fn test_percentile_empty() {
        let window = IncrementalWindow::new(10);
        assert_eq!(window.percentile(50.0), None);
    }

    #[test]
    fn test_multiple_percentiles() {
        let mut window = IncrementalWindow::new(100);

        for i in 1..=100 {
            window.push(i as f64);
        }

        let percentiles = window.percentiles(&[25.0, 50.0, 75.0, 95.0]);

        assert_eq!(percentiles.len(), 4);
        assert!(percentiles[0].is_some()); // p25
        assert!(percentiles[1].is_some()); // p50
        assert!(percentiles[2].is_some()); // p75
        assert!(percentiles[3].is_some()); // p95
    }

    #[test]
    fn test_median_vs_sort() {
        // Verify quickselect matches sort-based median
        let mut window = IncrementalWindow::new(1000);

        for i in 0..1000 {
            window.push((i * 7 % 1000) as f64); // Pseudo-random
        }

        let median = window.median().unwrap();

        // Manual calculation
        let mut values: Vec<f64> = window.window.iter().copied().collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let expected = (values[499] + values[500]) / 2.0;

        assert!((median - expected).abs() < 0.01);
    }

    #[test]
    fn test_performance_quickselect_vs_sort() {
        use std::time::Instant;

        let mut window = IncrementalWindow::new(10000);

        for i in 0..10000 {
            window.push(i as f64);
        }

        // Quickselect median
        let start = Instant::now();
        for _ in 0..100 {
            let _ = window.median();
        }
        let quickselect_time = start.elapsed();

        // Sort-based median
        let start = Instant::now();
        for _ in 0..100 {
            let mut temp: Vec<f64> = window.window.iter().copied().collect();
            temp.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let _ = temp[temp.len() / 2];
        }
        let sort_time = start.elapsed();

        println!("Quickselect: {:?}", quickselect_time);
        println!("Sort: {:?}", sort_time);

        // Quickselect should be faster
        assert!(quickselect_time < sort_time);
    }
}
```

---

### Milestone 5: Multiple Windows Simultaneously

**Goal**: Track multiple window sizes (1min, 5min, 1hour) with single pass.

**Why the previous milestone is not enough**: Often we need statistics at multiple time scales (short-term and long-term trends). Processing data separately for each window multiplies computational cost.

**What's the improvement**: Single-pass multi-window processing shares data ingestion cost. For 3 windows:
- Separate processing: 3 passes over data
- Combined processing: 1 pass over data (3x faster)

**Optimization focus**: Speed through single-pass processing.

**Architecture**:
- Structs: `MultiWindowAnalyzer`, `WindowStats`
- Fields: `windows: Vec<IncrementalWindow>`, `window_sizes: Vec<usize>`
- Functions:
  - `new(window_sizes: Vec<usize>) -> Self` - Create analyzer
  - `push(value: f64)` - Update all windows
  - `get_stats(window_index: usize) -> Option<WindowStats>` - Query specific window
  - `all_stats() -> Vec<WindowStats>` - Get all statistics

---

**Starter Code**:

```rust
/// Multi-window analyzer
/// Role: Track multiple time scales
#[derive(Debug)]
pub struct MultiWindowAnalyzer {
    windows: Vec<IncrementalWindow>,            // All windows       
    window_sizes: Vec<usize>,                   // Sizes for each window    
}

impl MultiWindowAnalyzer {
    /// Create multi-window analyzer
    /// Role: Initialize all windows
    pub fn new(window_sizes: Vec<usize>) -> Self {
        todo!("Create IncrementalWindow for each size")
    }

    /// Update all windows
    /// Role: Single-pass update
    pub fn push(&mut self, value: f64) {
        todo!("Call push on each window")
    }

    /// Get statistics for specific window
    /// Role: Query individual window
    pub fn get_stats(&self, window_index: usize) -> Option<WindowStats> {
        todo!("Extract stats from window at index")
    }

    /// Get statistics for all windows
    /// Role: Complete snapshot
    pub fn all_stats(&self) -> Vec<WindowStats> {
        todo!("Collect stats from all windows")
    }

    /// Get number of windows
    /// Role: Query configuration
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }
}

/// Statistics for a window
#[derive(Debug, Clone)]
pub struct WindowStats {
    pub average: Option<f64>,             // Mean                    
    pub std_dev: Option<f64>,             // Standard deviation      
    pub median: Option<f64>,              // Median                   
    pub min: Option<f64>,                 // Minimum                     
    pub max: Option<f64>,                 // maximum                     
    pub window_size: usize,               // Window configuration      
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_window_creation() {
        let analyzer = MultiWindowAnalyzer::new(vec![10, 60, 300]);

        assert_eq!(analyzer.window_count(), 3);
    }

    #[test]
    fn test_multi_window_push() {
        let mut analyzer = MultiWindowAnalyzer::new(vec![3, 5]);

        for i in 1..=10 {
            analyzer.push(i as f64);
        }

        let stats0 = analyzer.get_stats(0).unwrap();
        let stats1 = analyzer.get_stats(1).unwrap();

        // Window 0 (size 3): last 3 values [8, 9, 10]
        assert_eq!(stats0.average, Some(9.0));

        // Window 1 (size 5): last 5 values [6, 7, 8, 9, 10]
        assert_eq!(stats1.average, Some(8.0));
    }

    #[test]
    fn test_all_stats() {
        let mut analyzer = MultiWindowAnalyzer::new(vec![5, 10, 20]);

        for i in 1..=30 {
            analyzer.push(i as f64);
        }

        let all_stats = analyzer.all_stats();

        assert_eq!(all_stats.len(), 3);
        assert!(all_stats[0].average.is_some());
        assert!(all_stats[1].average.is_some());
        assert!(all_stats[2].average.is_some());
    }

    #[test]
    fn test_different_window_behaviors() {
        let mut analyzer = MultiWindowAnalyzer::new(vec![2, 5]);

        analyzer.push(10.0);
        analyzer.push(20.0);
        analyzer.push(30.0);
        analyzer.push(40.0);
        analyzer.push(50.0);

        let stats_small = analyzer.get_stats(0).unwrap(); // Window size 2
        let stats_large = analyzer.get_stats(1).unwrap(); // Window size 5

        // Small window: [40, 50]
        assert_eq!(stats_small.average, Some(45.0));

        // Large window: [10, 20, 30, 40, 50]
        assert_eq!(stats_large.average, Some(30.0));
    }

    #[test]
    fn test_single_pass_efficiency() {
        use std::time::Instant;

        let window_sizes = vec![10, 50, 100, 500, 1000];
        let data: Vec<f64> = (0..10000).map(|i| i as f64).collect();

        // Multi-window (single pass)
        let mut analyzer = MultiWindowAnalyzer::new(window_sizes.clone());
        let start = Instant::now();

        for &value in &data {
            analyzer.push(value);
        }

        let multi_time = start.elapsed();

        // Separate windows (multiple passes)
        let start = Instant::now();

        for &size in &window_sizes {
            let mut window = IncrementalWindow::new(size);
            for &value in &data {
                window.push(value);
            }
        }

        let separate_time = start.elapsed();

        println!("Multi-window (single pass): {:?}", multi_time);
        println!("Separate windows: {:?}", separate_time);

        // Multi-window should be faster or comparable
        // (In practice, might be slightly slower due to multiple window management,
        // but saves on data iteration)
    }

    #[test]
    fn test_empty_stats() {
        let analyzer = MultiWindowAnalyzer::new(vec![10]);

        let stats = analyzer.get_stats(0).unwrap();

        assert_eq!(stats.average, None);
        assert_eq!(stats.std_dev, None);
        assert_eq!(stats.median, None);
    }

    #[test]
    fn test_invalid_window_index() {
        let analyzer = MultiWindowAnalyzer::new(vec![10, 20]);

        assert!(analyzer.get_stats(2).is_none());
        assert!(analyzer.get_stats(10).is_none());
    }
}
```

---

### Milestone 6: Anomaly Detection with Z-Score

**Goal**: Detect anomalies using statistical thresholds.

**Why the previous milestone is not enough**: Statistics alone don't identify problems. Anomaly detection enables proactive monitoring and alerting.

**What's the improvement**: Automated anomaly detection catches issues in real-time. Instead of humans watching dashboards, systems alert on unusual patterns. Z-score method is simple yet effective: values more than 3 standard deviations from mean are flagged as anomalies.

**Optimization focus**: Practical application of streaming statistics.

**Architecture**:
- Structs: `AnomalyDetector`, `Anomaly`
- Fields: `analyzer: MultiWindowAnalyzer`, `threshold: f64`, `anomalies: Vec<Anomaly>`
- Functions:
  - `new(window_sizes, threshold) -> Self` - Create detector
  - `push(value, timestamp) -> Option<Anomaly>` - Check for anomaly
  - `anomaly_rate(total_points) -> f64` - Calculate percentage

---

**Starter Code**:

```rust
/// Anomaly detector using z-score
/// Role: Statistical outlier detection
#[derive(Debug)]
pub struct AnomalyDetector {
    analyzer: MultiWindowAnalyzer,           // Window statistics   
    threshold: f64,                          // Z-score threshold (typically 3.0)  
    anomalies: Vec<Anomaly>,                 // Detected anomalies        
}

/// Detected anomaly
#[derive(Debug, Clone)]
pub struct Anomaly {
    pub value: f64,                             // Anomalous value               
    pub z_score: f64,                           // How many std devs from mean 
    pub timestamp: usize,                       // When detected           
    pub window_stats: WindowStats,              // Context        
}

impl AnomalyDetector {
    /// Create anomaly detector
    /// Role: Initialize with configuration
    pub fn new(window_sizes: Vec<usize>, threshold: f64) -> Self {
        todo!("Create analyzer and empty anomaly list")
    }

    /// Add value and check for anomaly
    /// Role: Real-time detection
    pub fn push(&mut self, value: f64, timestamp: usize) -> Option<Anomaly> {
        todo!("Update analyzer, calculate z-score, check threshold")
    }

    /// Calculate anomaly rate
    /// Role: Summary statistic
    pub fn anomaly_rate(&self, total_points: usize) -> f64 {
        todo!("Return anomalies.len() / total_points")
    }

    /// Get all detected anomalies
    /// Role: Retrieve history
    pub fn get_anomalies(&self) -> &[Anomaly] {
        &self.anomalies
    }

    /// Clear anomaly history
    /// Role: Reset detector
    pub fn clear_anomalies(&mut self) {
        self.anomalies.clear();
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
    fn test_no_anomalies_in_normal_data() {
        let mut detector = AnomalyDetector::new(vec![100], 3.0);

        // Generate normal data (mean=50, std_dev small)
        for i in 0..200 {
            let value = 50.0 + ((i % 10) as f64 - 5.0);
            detector.push(value, i);
        }

        assert_eq!(detector.get_anomalies().len(), 0);
    }

    #[test]
    fn test_detect_obvious_outlier() {
        let mut detector = AnomalyDetector::new(vec![100], 3.0);

        // Normal values around 50
        for i in 0..100 {
            detector.push(50.0, i);
        }

        // Outlier
        let anomaly = detector.push(200.0, 100);

        assert!(anomaly.is_some());

        let anomaly = anomaly.unwrap();
        assert!(anomaly.z_score.abs() > 3.0);
        assert_eq!(anomaly.value, 200.0);
    }

    #[test]
    fn test_z_score_calculation() {
        let mut detector = AnomalyDetector::new(vec![10], 2.0);

        // Mean = 10, Std dev = 0
        for i in 0..10 {
            detector.push(10.0, i);
        }

        // Add value outside 2 std devs
        let anomaly = detector.push(15.0, 10);

        if let Some(anomaly) = anomaly {
            // Z-score = (15 - 10) / std_dev
            assert!(anomaly.z_score > 2.0);
        }
    }

    #[test]
    fn test_anomaly_rate() {
        let mut detector = AnomalyDetector::new(vec![50], 3.0);

        let total_points = 1000;

        for i in 0..total_points {
            let value = if i % 100 == 0 {
                // Every 100th point is anomaly
                1000.0
            } else {
                50.0
            };

            detector.push(value, i);
        }

        let rate = detector.anomaly_rate(total_points);

        // Should detect ~10 anomalies out of 1000
        assert!(rate > 0.005 && rate < 0.015); // Between 0.5% and 1.5%
    }

    #[test]
    fn test_anomaly_context() {
        let mut detector = AnomalyDetector::new(vec![20], 3.0);

        for i in 0..30 {
            detector.push(100.0, i);
        }

        // Anomaly
        let anomaly = detector.push(200.0, 30).unwrap();

        assert_eq!(anomaly.timestamp, 30);
        assert!(anomaly.window_stats.average.is_some());
        assert!(anomaly.window_stats.std_dev.is_some());
    }

    #[test]
    fn test_different_thresholds() {
        let data: Vec<f64> = (0..100).map(|i| 50.0 + (i % 20) as f64).collect();

        // Strict threshold (more sensitive)
        let mut strict = AnomalyDetector::new(vec![50], 2.0);
        for (i, &v) in data.iter().enumerate() {
            strict.push(v, i);
        }

        // Lenient threshold (less sensitive)
        let mut lenient = AnomalyDetector::new(vec![50], 4.0);
        for (i, &v) in data.iter().enumerate() {
            lenient.push(v, i);
        }

        // Strict should detect more anomalies
        assert!(strict.get_anomalies().len() >= lenient.get_anomalies().len());
    }

    #[test]
    fn test_clear_anomalies() {
        let mut detector = AnomalyDetector::new(vec![10], 2.0);

        for i in 0..10 {
            detector.push(10.0, i);
        }

        detector.push(50.0, 10); // Anomaly

        assert_eq!(detector.get_anomalies().len(), 1);

        detector.clear_anomalies();

        assert_eq!(detector.get_anomalies().len(), 0);
    }

    #[test]
    fn test_real_world_monitoring() {
        // Simulate server response time monitoring
        let mut detector = AnomalyDetector::new(
            vec![60, 300, 900], // 1min, 5min, 15min windows
            3.0
        );

        // Normal response times: 100-200ms
        for i in 0..1000 {
            let normal_time = 150.0 + ((i % 50) as f64 - 25.0);
            detector.push(normal_time, i);
        }

        // Spike: 2000ms response time
        let anomaly = detector.push(2000.0, 1000);

        assert!(anomaly.is_some());
        println!("Detected anomaly: {:?}", anomaly.unwrap());
    }
}
```

---

### Testing Strategies

1. **Unit Tests**: Test each window algorithm independently
2. **Property Tests**: Verify incremental stats equal batch stats
3. **Performance Tests**: Benchmark O(1) vs O(n) algorithms
4. **Correctness Tests**: Compare with reference implementations
5. **Stress Tests**: Process millions of data points
6. **Anomaly Tests**: Test with synthetic data (normal + outliers)
7. **Integration Tests**: Complete monitoring pipeline

---

### Complete Working Example

```rust
use std::collections::VecDeque;

fn main() {
    println!("=== Time-Series Analyzer ===\n");

    // Example: Monitor sensor data
    let mut detector = AnomalyDetector::new(
        vec![10, 60, 300], // 10s, 60s, 300s windows
        3.0 // 3 standard deviations
    );

    // Simulate sensor readings
    for i in 0..1000 {
        let value = if i == 500 {
            // Inject anomaly at timestamp 500
            150.0
        } else {
            // Normal readings: 20-25°C
            22.5 + ((i % 10) as f64 - 5.0) * 0.2
        };

        if let Some(anomaly) = detector.push(value, i) {
            println!(
                "🚨 ANOMALY at t={}: value={:.2}°C (z-score={:.2})",
                anomaly.timestamp,
                anomaly.value,
                anomaly.z_score
            );

            let stats = &anomaly.window_stats;
            println!(
                "   Context: mean={:.2}, std_dev={:.2}",
                stats.average.unwrap(),
                stats.std_dev.unwrap()
            );
        }
    }

    println!("\nAnomal rate: {:.2}%", detector.anomaly_rate(1000) * 100.0);
}
```

This project demonstrates advanced Vec/slice techniques:
- **VecDeque** for efficient sliding windows
- **Incremental algorithms** (500x speedup)
- **Monotonic deques** for O(1) min/max
- **Quickselect** for O(n) median
- **Multi-window** single-pass processing
- **Real-time anomaly detection**
