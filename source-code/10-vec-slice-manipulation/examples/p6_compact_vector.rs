//! Pattern 6: Advanced Slice Patterns
//! Example: In-Place Vector Compaction
//!
//! Run with: cargo run --example p6_compact_vector

fn main() {
    println!("=== In-Place Vector Compaction ===\n");

    // Item struct with validity flag
    #[derive(Debug, Clone)]
    struct Item {
        id: usize,
        value: i32,
        active: bool,
    }

    impl Item {
        fn should_keep(&self) -> bool {
            self.active
        }
    }

    // Compact using swap technique (O(n) time, O(1) space)
    fn compact_vector(vec: &mut Vec<Item>) {
        let mut write_index = 0;

        for read_index in 0..vec.len() {
            if vec[read_index].should_keep() {
                if read_index != write_index {
                    vec.swap(read_index, write_index);
                }
                write_index += 1;
            }
        }

        vec.truncate(write_index);
    }

    let mut items: Vec<Item> = (0..10)
        .map(|i| Item {
            id: i,
            value: i as i32 * 10,
            active: i % 3 != 0, // 0, 3, 6, 9 are inactive
        })
        .collect();

    println!("Before compaction:");
    for item in &items {
        println!("  {:?}", item);
    }

    compact_vector(&mut items);

    println!("\nAfter compaction:");
    for item in &items {
        println!("  {:?}", item);
    }

    // Compare with retain
    println!("\n=== Comparison with retain() ===\n");

    let mut items1: Vec<i32> = (0..10).collect();
    let mut items2 = items1.clone();

    // Using retain
    items1.retain(|&x| x % 2 == 0);
    println!("retain (evens): {:?}", items1);

    // Manual compaction
    fn compact_evens(vec: &mut Vec<i32>) {
        let mut write = 0;
        for read in 0..vec.len() {
            if vec[read] % 2 == 0 {
                if read != write {
                    vec.swap(read, write);
                }
                write += 1;
            }
        }
        vec.truncate(write);
    }

    compact_evens(&mut items2);
    println!("compact (evens): {:?}", items2);

    // Extract matching elements
    println!("\n=== Extract Matching Elements ===\n");

    fn extract_matching<T, F>(vec: &mut Vec<T>, predicate: F) -> Vec<T>
    where
        T: Default,
        F: Fn(&T) -> bool,
    {
        let mut extracted = Vec::new();
        let mut i = 0;

        while i < vec.len() {
            if predicate(&vec[i]) {
                extracted.push(std::mem::take(&mut vec[i]));
                vec.swap_remove(i);
                // Don't increment i - swap_remove puts last element at i
            } else {
                i += 1;
            }
        }

        extracted
    }

    #[derive(Debug, Default)]
    struct Task {
        id: usize,
        priority: u8,
    }

    let mut tasks: Vec<Task> = (0..8)
        .map(|i| Task { id: i, priority: (i % 3) as u8 })
        .collect();

    println!("Before extraction:");
    for task in &tasks {
        println!("  Task {} (priority {})", task.id, task.priority);
    }

    let high_priority = extract_matching(&mut tasks, |t| t.priority >= 2);

    println!("\nExtracted (priority >= 2):");
    for task in &high_priority {
        println!("  Task {} (priority {})", task.id, task.priority);
    }

    println!("\nRemaining:");
    for task in &tasks {
        println!("  Task {} (priority {})", task.id, task.priority);
    }

    // Splice for range replacement
    println!("\n=== Splice for Range Replacement ===\n");

    fn replace_range(vec: &mut Vec<i32>, start: usize, end: usize, replacement: &[i32]) {
        vec.splice(start..end, replacement.iter().copied());
    }

    let mut data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    println!("Before: {:?}", data);

    replace_range(&mut data, 2, 5, &[30, 40]);
    println!("Replace [2..5] with [30, 40]: {:?}", data);

    replace_range(&mut data, 1, 2, &[20, 21, 22]);
    println!("Replace [1..2] with [20, 21, 22]: {:?}", data);

    // Efficient mid-vector insertion
    println!("\n=== Efficient Mid-Vector Insertion ===\n");

    fn insert_slice_at(vec: &mut Vec<u8>, index: usize, data: &[u8]) {
        let tail = vec.split_off(index);
        vec.extend_from_slice(data);
        vec.extend_from_slice(&tail);
    }

    let mut buffer = vec![1u8, 2, 3, 7, 8, 9];
    println!("Before: {:?}", buffer);

    insert_slice_at(&mut buffer, 3, &[4, 5, 6]);
    println!("Insert [4, 5, 6] at index 3: {:?}", buffer);

    println!("\n=== Key Points ===");
    println!("1. Swap-compact is O(n) with O(1) extra space");
    println!("2. retain() is built-in and optimized");
    println!("3. swap_remove is O(1) but changes order");
    println!("4. splice for range replacement");
    println!("5. split_off + extend for mid-vector insertion");
}
