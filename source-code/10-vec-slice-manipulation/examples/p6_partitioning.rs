//! Pattern 6: Advanced Slice Patterns
//! Example: Binary and Three-Way Partitioning
//!
//! Run with: cargo run --example p6_partitioning

fn main() {
    println!("=== Binary and Three-Way Partitioning ===\n");

    // Binary partition (Lomuto scheme)
    println!("=== Binary Partitioning (by sign) ===\n");

    fn partition_by_sign(values: &mut [i32]) -> usize {
        let mut left = 0;
        let mut right = values.len();

        while left < right {
            if values[left] >= 0 {
                left += 1;
            } else {
                right -= 1;
                values.swap(left, right);
            }
        }

        left
    }

    let mut data = vec![3, -1, 4, -1, 5, -9, 2, -6, 5];
    println!("Before: {:?}", data);

    let pivot = partition_by_sign(&mut data);
    println!("After:  {:?}", data);
    println!("Pivot:  {}", pivot);
    println!("Non-negative: {:?}", &data[..pivot]);
    println!("Negative:     {:?}", &data[pivot..]);

    // Partition by predicate
    println!("\n=== Partition by Predicate ===\n");

    fn partition_by<T, F>(values: &mut [T], predicate: F) -> usize
    where
        F: Fn(&T) -> bool,
    {
        let mut write = 0;
        for read in 0..values.len() {
            if predicate(&values[read]) {
                values.swap(read, write);
                write += 1;
            }
        }
        write
    }

    let mut data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    println!("Before: {:?}", data);

    let pivot = partition_by(&mut data, |&x| x % 2 == 0);
    println!("Partition by even:");
    println!("  After:  {:?}", data);
    println!("  Evens:  {:?}", &data[..pivot]);
    println!("  Odds:   {:?}", &data[pivot..]);

    // Three-way partition (Dutch National Flag)
    println!("\n=== Three-Way Partitioning (Dutch National Flag) ===\n");

    fn partition_three_way(values: &mut [i32], pivot: i32) -> (usize, usize) {
        let mut low = 0;
        let mut mid = 0;
        let mut high = values.len();

        while mid < high {
            if values[mid] < pivot {
                values.swap(low, mid);
                low += 1;
                mid += 1;
            } else if values[mid] > pivot {
                high -= 1;
                values.swap(mid, high);
            } else {
                mid += 1;
            }
        }

        (low, high)
    }

    let mut data = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
    let pivot_value = 5;
    println!("Before: {:?}", data);
    println!("Pivot value: {}", pivot_value);

    let (low, high) = partition_three_way(&mut data, pivot_value);
    println!("After:  {:?}", data);
    println!("Regions:");
    println!("  < {}: {:?} (indices 0..{})", pivot_value, &data[..low], low);
    println!("  = {}: {:?} (indices {}..{})", pivot_value, &data[low..high], low, high);
    println!("  > {}: {:?} (indices {}..{})", pivot_value, &data[high..], high, data.len());

    // Quickselect using partition
    println!("\n=== Quickselect (Find Kth Element) ===\n");

    fn quickselect(values: &mut [i32], k: usize) -> i32 {
        if values.len() == 1 {
            return values[0];
        }

        // Use middle element as pivot
        let pivot_idx = values.len() / 2;
        let pivot = values[pivot_idx];

        let (low, high) = partition_three_way(values, pivot);

        if k < low {
            quickselect(&mut values[..low], k)
        } else if k >= high {
            quickselect(&mut values[high..], k - high)
        } else {
            pivot
        }
    }

    let mut data = vec![9, 3, 7, 1, 5, 8, 2, 6, 4];
    let original = data.clone();
    println!("Data: {:?}", original);

    for k in [0, 4, 8] {
        let mut copy = data.clone();
        let kth = quickselect(&mut copy, k);
        let mut sorted = original.clone();
        sorted.sort();
        println!("{}th smallest: {} (sorted[{}] = {})", k, kth, k, sorted[k]);
    }

    // Stable partition
    println!("\n=== Stable Partition (Preserves Order) ===\n");

    fn stable_partition<T: Clone, F>(values: &[T], predicate: F) -> (Vec<T>, Vec<T>)
    where
        F: Fn(&T) -> bool,
    {
        let (matching, non_matching): (Vec<_>, Vec<_>) =
            values.iter().cloned().partition(|x| predicate(x));
        (matching, non_matching)
    }

    let data = vec!["apple", "banana", "avocado", "cherry", "apricot"];
    println!("Data: {:?}", data);

    let (a_words, other) = stable_partition(&data, |s| s.starts_with('a'));
    println!("Starts with 'a': {:?}", a_words);
    println!("Other:           {:?}", other);

    // Partition for load balancing
    println!("\n=== Use Case: Load Balancing ===\n");

    #[derive(Debug, Clone)]
    struct Task {
        id: usize,
        weight: u32,
    }

    fn partition_by_weight(tasks: &mut [Task], threshold: u32) -> usize {
        let mut write = 0;
        for read in 0..tasks.len() {
            if tasks[read].weight <= threshold {
                tasks.swap(read, write);
                write += 1;
            }
        }
        write
    }

    let mut tasks: Vec<Task> = (0..8)
        .map(|i| Task { id: i, weight: (i * 7 % 10 + 1) as u32 })
        .collect();

    println!("Tasks: {:?}", tasks);
    let threshold = 5;
    println!("Threshold: {}", threshold);

    let pivot = partition_by_weight(&mut tasks, threshold);
    println!("Light tasks (weight <= {}): {:?}", threshold, &tasks[..pivot]);
    println!("Heavy tasks (weight > {}):  {:?}", threshold, &tasks[pivot..]);

    println!("\n=== Key Points ===");
    println!("1. Binary partition: O(n) time, O(1) space");
    println!("2. Three-way partition: handles duplicates efficiently");
    println!("3. Foundation for quicksort/quickselect");
    println!("4. In-place but not stable (use iter().partition() for stable)");
    println!("5. Perfect for dividing work, filtering in place");
}
