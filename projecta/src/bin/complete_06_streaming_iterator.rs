// Complete Streaming Iterator with HRTB Implementation
// Demonstrates GATs, HRTBs, and zero-copy iteration

//==============================================================================
// Milestone 1: Basic StreamingIterator Trait with GATs
//==============================================================================

/// StreamingIterator trait using Generic Associated Types
/// Unlike standard Iterator, items can borrow from the iterator
pub trait StreamingIterator {
    /// Item type constructor - takes a lifetime parameter
    /// The `where Self: 'a` clause ensures items can't outlive the iterator
    type Item<'a>
    where
        Self: 'a;

    /// Advances the iterator and returns the next item
    /// Item borrows from &mut self, so lifetime is tied to this call
    fn next(&mut self) -> Option<Self::Item<'_>>;

    /// Returns bounds on remaining length
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

//==============================================================================
// Iter: Simple streaming iterator over slice elements
//==============================================================================

/// Iterator that yields references to slice elements
pub struct Iter<'data, T> {
    data: &'data [T],
    position: usize,
}

impl<'data, T> Iter<'data, T> {
    pub fn new(data: &'data [T]) -> Self {
        Self { data, position: 0 }
    }
}

impl<'data, T> StreamingIterator for Iter<'data, T> {
    /// Each call to next() returns &'a T where 'a is the lifetime of that call
    type Item<'a>
        = &'a T
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.position < self.data.len() {
            let item = &self.data[self.position];
            self.position += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.data.len() - self.position;
        (remaining, Some(remaining))
    }
}

//==============================================================================
// Milestone 2: Windows Iterator with Zero-Copy Slices
//==============================================================================

/// Iterator that yields overlapping windows of fixed size
/// Returns borrowed slices - zero allocations!
pub struct Windows<'data, T> {
    data: &'data [T],
    window_size: usize,
    position: usize,
}

impl<'data, T> Windows<'data, T> {
    pub fn new(data: &'data [T], window_size: usize) -> Self {
        assert!(window_size > 0, "Window size must be > 0");
        Self {
            data,
            window_size,
            position: 0,
        }
    }
}

impl<'data, T> StreamingIterator for Windows<'data, T> {
    /// Yields slices borrowing from the data
    type Item<'a>
        = &'a [T]
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.position + self.window_size <= self.data.len() {
            let window = &self.data[self.position..self.position + self.window_size];
            self.position += 1;
            Some(window)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = if self.position + self.window_size <= self.data.len() {
            self.data.len() - self.position - self.window_size + 1
        } else {
            0
        };
        (remaining, Some(remaining))
    }
}

/// Helper function to create windows
pub fn windows<T>(data: &[T], size: usize) -> Windows<'_, T> {
    Windows::new(data, size)
}

//==============================================================================
// StepBy: Adapter for non-overlapping iteration
//==============================================================================

/// Adapter that advances by step items each time
pub struct StepBy<I> {
    iter: I,
    step: usize,
    first: bool,
}

impl<I: StreamingIterator> StreamingIterator for StepBy<I> {
    /// Forward the Item type from the inner iterator
    type Item<'a>
        = I::Item<'a>
    where
        Self: 'a,
        I: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        // On first call, just return the item
        if self.first {
            self.first = false;
            return self.iter.next();
        }

        // Otherwise, skip (step - 1) items, then return next
        for _ in 0..self.step - 1 {
            self.iter.next()?;
        }

        self.iter.next()
    }
}

/// Extension trait for adapter methods
pub trait StreamingIteratorExt: StreamingIterator {
    /// Creates an iterator that advances by `step` elements each time
    fn step_by(self, step: usize) -> StepBy<Self>
    where
        Self: Sized,
    {
        assert!(step > 0, "Step must be > 0");
        StepBy {
            iter: self,
            step,
            first: true,
        }
    }
}

// Implement for all StreamingIterators
impl<I: StreamingIterator> StreamingIteratorExt for I {}

//==============================================================================
// Milestone 3: Higher-Ranked Trait Bounds for Generic Functions
//==============================================================================

/// Process each item with a closure
/// HRTB: F must work for ANY lifetime 'a
pub fn for_each<'i, I, F>(mut iter: I, mut f: F)
where
    I: StreamingIterator + 'i,
    F: FnMut(I::Item<'_>),
{
    while let Some(item) = iter.next() {
        f(item);
    }
}

/// Check if all items satisfy predicate
pub fn all<'i, I, F>(mut iter: I, mut predicate: F) -> bool
where
    I: StreamingIterator + 'i,
    F: FnMut(I::Item<'_>) -> bool,
{
    while let Some(item) = iter.next() {
        if !predicate(item) {
            return false;
        }
    }
    true
}

/// Fold items into accumulator
pub fn fold<'i, I, B, F>(mut iter: I, init: B, mut f: F) -> B
where
    I: StreamingIterator + 'i,
    F: FnMut(B, I::Item<'_>) -> B,
{
    let mut acc = init;
    while let Some(item) = iter.next() {
        acc = f(acc, item);
    }
    acc
}

/// Count number of items
pub fn count<I>(mut iter: I) -> usize
where
    I: StreamingIterator,
{
    let mut count = 0;
    while iter.next().is_some() {
        count += 1;
    }
    count
}

/// Find if any item matches predicate
pub fn find<'i, I, F>(mut iter: I, mut predicate: F) -> bool
where
    I: StreamingIterator + 'i,
    F: FnMut(I::Item<'_>) -> bool,
{
    while let Some(item) = iter.next() {
        if predicate(item) {
            return true;
        }
    }
    false
}

//==============================================================================
// Milestone 4: GroupBy Iterator with Lifetime Variance
//==============================================================================

/// Groups consecutive equal elements
pub struct GroupBy<'data, T> {
    data: &'data [T],
    position: usize,
}

impl<'data, T> GroupBy<'data, T> {
    pub fn new(data: &'data [T]) -> Self {
        Self { data, position: 0 }
    }
}

impl<'data, T: PartialEq> StreamingIterator for GroupBy<'data, T> {
    type Item<'a>
        = &'a [T]
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.position >= self.data.len() {
            return None;
        }

        let start = self.position;
        let first = &self.data[start];
        let mut end = start + 1;

        // Find end of group (while elements are equal)
        while end < self.data.len() && &self.data[end] == first {
            end += 1;
        }

        self.position = end;
        Some(&self.data[start..end])
    }
}

//==============================================================================
// Milestone 5: Comparison with Standard Iterator
//==============================================================================

/// Standard Iterator version that MUST allocate
pub struct AllocatingWindows<T: Clone> {
    data: Vec<T>,
    window_size: usize,
    position: usize,
}

impl<T: Clone> AllocatingWindows<T> {
    pub fn new(data: Vec<T>, window_size: usize) -> Self {
        Self {
            data,
            window_size,
            position: 0,
        }
    }
}

impl<T: Clone> Iterator for AllocatingWindows<T> {
    type Item = Vec<T>; // Must return OWNED Vec

    fn next(&mut self) -> Option<Self::Item> {
        if self.position + self.window_size <= self.data.len() {
            // ALLOCATION: Must copy data into new Vec
            let window = self.data[self.position..self.position + self.window_size].to_vec();
            self.position += 1;
            Some(window)
        } else {
            None
        }
    }
}

/// Benchmark comparison between streaming and allocating iterators
pub fn benchmark_windows(data_size: usize, window_size: usize, iterations: usize) {
    use std::time::Instant;

    let data: Vec<i32> = (0..data_size as i32).collect();

    // Streaming iterator (zero-copy)
    let start = Instant::now();
    for _ in 0..iterations {
        let mut iter = windows(&data, window_size);
        let mut sum = 0;
        while let Some(window) = iter.next() {
            sum += window[0]; // Just access, no allocation
        }
        std::hint::black_box(sum); // Prevent optimization
    }
    let streaming_time = start.elapsed();

    // Standard iterator (allocating)
    let start = Instant::now();
    for _ in 0..iterations {
        let iter = AllocatingWindows::new(data.clone(), window_size);
        let mut sum = 0;
        for window in iter {
            sum += window[0]; // Each window allocated
        }
        std::hint::black_box(sum);
    }
    let allocating_time = start.elapsed();

    println!("\n=== Benchmark Results ===");
    println!(
        "Data size: {}, Window: {}, Iterations: {}",
        data_size, window_size, iterations
    );
    println!("Streaming:   {:?}", streaming_time);
    println!("Allocating:  {:?}", allocating_time);
    println!(
        "Speedup: {:.2}x",
        allocating_time.as_secs_f64() / streaming_time.as_secs_f64()
    );

    let num_windows = data_size - window_size + 1;
    println!("Allocations saved: {} per iteration", num_windows);
}

//==============================================================================
// Example Usage
//==============================================================================

fn main() {
    println!("=== Streaming Iterator Examples ===\n");

    // Example 1: Basic iteration
    println!("Example 1: Basic iteration");
    let data = vec![1, 2, 3, 4, 5];
    let mut iter = Iter::new(&data);

    print!("Elements: ");
    while let Some(&x) = iter.next() {
        print!("{} ", x);
    }
    println!("\n");

    // Example 2: Windows with zero-copy
    println!("Example 2: Overlapping windows (zero-copy)");
    let data = vec![1, 2, 3, 4, 5];
    let mut iter = windows(&data, 3);

    while let Some(window) = iter.next() {
        println!("Window: {:?}", window);
    }
    println!();

    // Example 3: Step by for non-overlapping
    println!("Example 3: Non-overlapping windows with step_by");
    let data = vec![1, 2, 3, 4, 5, 6];
    let data_clone = data.clone();
    let mut iter = windows(&data_clone, 2).step_by(2);

    while let Some(window) = iter.next() {
        println!("Window: {:?}", window);
    }
    println!();

    // Example 4: Higher-ranked trait bounds
    println!("Example 4: Using HRTB with for_each");
    {
        static DATA: &[i32] = &[1, 2, 3, 4, 5];
        let iter = windows(DATA, 2);

        for_each(iter, |window| {
            println!("Sum of window: {}", window.iter().sum::<i32>());
        });
    }
    println!();

    // Example 5: GroupBy consecutive elements
    println!("Example 5: GroupBy consecutive equal elements");
    let data = vec![1, 1, 1, 2, 2, 3, 3, 3, 3];
    let mut iter = GroupBy::new(&data);

    while let Some(group) = iter.next() {
        println!("Group: {:?} (length: {})", group, group.len());
    }
    println!();

    // Example 6: Fold with HRTB
    println!("Example 6: Fold to sum all windows");
    {
        static DATA: &[i32] = &[1, 2, 3, 4];
        let iter = windows(DATA, 2);

        let total = fold(iter, 0, |acc, window| acc + window[0] + window[1]);
        println!("Total: {}\n", total);
    }

    // Example 7: Performance comparison
    println!("Example 7: Performance comparison");
    benchmark_windows(1000, 10, 100);
}

//==============================================================================
// Tests
//==============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Milestone 1 Tests
    #[test]
    fn test_streaming_iter_basic() {
        let data = vec![1, 2, 3, 4, 5];
        let mut iter = Iter::new(&data);

        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), Some(&5));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_size_hint() {
        let data = vec![10, 20, 30];
        let mut iter = Iter::new(&data);

        assert_eq!(iter.size_hint(), (3, Some(3)));

        iter.next();
        assert_eq!(iter.size_hint(), (2, Some(2)));

        iter.next();
        assert_eq!(iter.size_hint(), (1, Some(1)));
    }

    // Milestone 2 Tests
    #[test]
    fn test_windows_overlapping() {
        let data = vec![1, 2, 3, 4, 5];
        let mut iter = windows(&data, 3);

        assert_eq!(iter.next(), Some(&[1, 2, 3][..]));
        assert_eq!(iter.next(), Some(&[2, 3, 4][..]));
        assert_eq!(iter.next(), Some(&[3, 4, 5][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_windows_size_2() {
        let data = vec!['a', 'b', 'c', 'd'];
        let mut iter = windows(&data, 2);

        assert_eq!(iter.next(), Some(&['a', 'b'][..]));
        assert_eq!(iter.next(), Some(&['b', 'c'][..]));
        assert_eq!(iter.next(), Some(&['c', 'd'][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_windows_exact_size() {
        let data = vec![10, 20, 30];
        let mut iter = windows(&data, 3);

        assert_eq!(iter.next(), Some(&[10, 20, 30][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_windows_too_large() {
        let data = vec![1, 2];
        let mut iter = windows(&data, 5);

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_step_by() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let mut iter = windows(&data, 2).step_by(2);

        assert_eq!(iter.next(), Some(&[1, 2][..]));
        assert_eq!(iter.next(), Some(&[3, 4][..]));
        assert_eq!(iter.next(), Some(&[5, 6][..]));
        assert_eq!(iter.next(), None);
    }

    // Milestone 3 Tests
    #[test]
    fn test_for_each() {
        static DATA: &[i32] = &[1, 2, 3, 4, 5];
        let iter = windows(DATA, 2);

        let mut sum = 0;
        for_each(iter, |window| {
            sum += window[0] + window[1];
        });

        // Windows: [1,2], [2,3], [3,4], [4,5]
        // Sum: 3 + 5 + 7 + 9 = 24
        assert_eq!(sum, 24);
    }

    #[test]
    fn test_all() {
        static DATA1: &[i32] = &[2, 4, 6, 8];
        let iter = Iter::new(DATA1);

        let all_even = all(iter, |&x| x % 2 == 0);
        assert!(all_even);

        static DATA2: &[i32] = &[2, 4, 5, 8];
        let iter = Iter::new(DATA2);
        let all_even = all(iter, |&x| x % 2 == 0);
        assert!(!all_even);
    }

    #[test]
    fn test_fold() {
        static DATA: &[i32] = &[1, 2, 3, 4, 5];
        let iter = Iter::new(DATA);

        let sum = fold(iter, 0, |acc, &x| acc + x);
        assert_eq!(sum, 15);
    }

    #[test]
    fn test_fold_windows() {
        static DATA: &[i32] = &[1, 2, 3, 4];
        let iter = windows(DATA, 2);

        // Concatenate all windows
        let result = fold(iter, Vec::new(), |mut acc, window| {
            acc.extend_from_slice(window);
            acc
        });

        // Windows: [1,2], [2,3], [3,4]
        assert_eq!(result, vec![1, 2, 2, 3, 3, 4]);
    }

    #[test]
    fn test_count() {
        let data = vec![10, 20, 30, 40, 50];
        let iter = windows(&data, 3);

        assert_eq!(count(iter), 3); // 3 windows of size 3
    }

    #[test]
    fn test_find() {
        static DATA: &[i32] = &[1, 2, 3, 4, 5];
        let iter = windows(DATA, 2);

        // Find window where first element is 3
        let has_window_starting_3 = find(iter, |window| window[0] == 3);
        assert!(has_window_starting_3);
    }

    // Milestone 4 Tests
    #[test]
    fn test_group_by_consecutive() {
        let data = vec![1, 1, 1, 2, 2, 3, 3, 3, 3];
        let mut iter = GroupBy::new(&data);

        assert_eq!(iter.next(), Some(&[1, 1, 1][..]));
        assert_eq!(iter.next(), Some(&[2, 2][..]));
        assert_eq!(iter.next(), Some(&[3, 3, 3, 3][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_group_by_single_elements() {
        let data = vec![1, 2, 3, 4];
        let mut iter = GroupBy::new(&data);

        assert_eq!(iter.next(), Some(&[1][..]));
        assert_eq!(iter.next(), Some(&[2][..]));
        assert_eq!(iter.next(), Some(&[3][..]));
        assert_eq!(iter.next(), Some(&[4][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_group_by_all_equal() {
        let data = vec!['a', 'a', 'a', 'a'];
        let mut iter = GroupBy::new(&data);

        assert_eq!(iter.next(), Some(&['a', 'a', 'a', 'a'][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_group_by_strings() {
        let data = vec!["a", "a", "b", "c", "c", "c"];
        let mut iter = GroupBy::new(&data);

        assert_eq!(iter.next(), Some(&["a", "a"][..]));
        assert_eq!(iter.next(), Some(&["b"][..]));
        assert_eq!(iter.next(), Some(&["c", "c", "c"][..]));
        assert_eq!(iter.next(), None);
    }

    // Milestone 5 Tests
    #[test]
    fn test_allocating_windows() {
        let data = vec![1, 2, 3, 4, 5];
        let mut iter = AllocatingWindows::new(data, 3);

        assert_eq!(iter.next(), Some(vec![1, 2, 3]));
        assert_eq!(iter.next(), Some(vec![2, 3, 4]));
        assert_eq!(iter.next(), Some(vec![3, 4, 5]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_allocating_can_collect() {
        let data = vec![1, 2, 3, 4];
        let iter = AllocatingWindows::new(data, 2);

        // CAN collect with standard Iterator
        let windows: Vec<Vec<i32>> = iter.collect();
        assert_eq!(windows, vec![vec![1, 2], vec![2, 3], vec![3, 4]]);
    }

    #[test]
    fn test_benchmark_small() {
        // Small benchmark to verify it runs
        benchmark_windows(100, 5, 10);
    }

    #[test]
    fn test_memory_comparison() {
        use std::mem::{size_of, size_of_val};

        let data: Vec<i32> = (0..100).collect();

        // Streaming: just slice metadata (pointer + length)
        let mut streaming = windows(&data, 10);
        let slice_ref_size = if let Some(window) = streaming.next() {
            // This measures the fat pointer (&[i32]) on the stack
            size_of_val(&window) // Size of the reference itself
        } else {
            0
        };

        // Allocating: full Vec (pointer + length + capacity)
        let mut allocating = AllocatingWindows::new(data.clone(), 10);
        let vec_ref_size = if let Some(window) = allocating.next() {
            // This measures Vec<i32> on the stack
            size_of_val(&window)
        } else {
            0
        };

        // Both &[i32] and Vec<i32> are stack structures
        // &[i32] is 16 bytes (ptr + len)
        // Vec<i32> is 24 bytes (ptr + len + cap)
        println!("Slice reference size: {}", slice_ref_size);
        println!("Vec size: {}", vec_ref_size);
        println!("Expected &[T] size: {}", size_of::<&[i32]>());
        println!("Expected Vec<T> size: {}", size_of::<Vec<i32>>());

        // Just verify they have reasonable sizes
        assert!(slice_ref_size > 0 && vec_ref_size > 0);
    }

    #[test]
    fn test_zero_cost_abstraction() {
        // Streaming iterator is just data pointer + position
        let data = vec![1, 2, 3];
        let iter = Iter::new(&data);

        // Should be small - just a reference and index
        assert!(std::mem::size_of_val(&iter) <= 24);
    }
}
