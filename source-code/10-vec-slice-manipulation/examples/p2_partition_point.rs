//! Pattern 2: Slice Algorithms
//! Example: Partition Point for Range Queries
//!
//! Run with: cargo run --example p2_partition_point

fn main() {
    println!("=== Partition Point for Range Queries ===\n");

    // Basic partition_point
    let sorted = vec![1, 3, 5, 7, 9, 11, 13, 15];
    println!("Sorted array: {:?}", sorted);

    // Find first element >= 6
    let idx = sorted.partition_point(|&x| x < 6);
    println!("First element >= 6 is at index {}: {}", idx, sorted[idx]);

    // Find first element >= 20 (would be past the end)
    let idx = sorted.partition_point(|&x| x < 20);
    println!("First element >= 20 would be at index {} (past end)", idx);

    // Range queries
    println!("\n=== Range Queries ===\n");

    fn find_range(sorted: &[i32], min: i32, max: i32) -> &[i32] {
        let start = sorted.partition_point(|&x| x < min);
        let end = sorted.partition_point(|&x| x <= max);
        &sorted[start..end]
    }

    let data = vec![1, 3, 5, 7, 9, 11, 13, 15, 17, 19];
    println!("Data: {:?}", data);

    let ranges = [(3, 7), (5, 15), (0, 5), (16, 20), (10, 10)];
    for (min, max) in ranges {
        let range = find_range(&data, min, max);
        println!("  Range [{}, {}]: {:?}", min, max, range);
    }

    // Count elements in range
    println!("\n=== Count Elements in Range ===\n");

    fn count_in_range(sorted: &[i32], min: i32, max: i32) -> usize {
        let start = sorted.partition_point(|&x| x < min);
        let end = sorted.partition_point(|&x| x <= max);
        end - start
    }

    let large_data: Vec<i32> = (0..1000).collect();
    let test_ranges = [(100, 200), (0, 50), (500, 999), (1000, 2000)];

    for (min, max) in test_ranges {
        let count = count_in_range(&large_data, min, max);
        println!("  Elements in [{}, {}]: {}", min, max, count);
    }

    // Database-style queries
    println!("\n=== Database-Style Queries ===\n");

    #[derive(Debug)]
    struct Record {
        timestamp: u64,
        value: f64,
    }

    let mut records: Vec<Record> = (0..20)
        .map(|i| Record {
            timestamp: i * 100,
            value: (i as f64) * 1.5,
        })
        .collect();

    // Records are already sorted by timestamp

    fn query_time_range<'a>(
        records: &'a [Record],
        start_time: u64,
        end_time: u64,
    ) -> &'a [Record] {
        let start = records.partition_point(|r| r.timestamp < start_time);
        let end = records.partition_point(|r| r.timestamp <= end_time);
        &records[start..end]
    }

    println!("Records with timestamps 0, 100, 200, ..., 1900");
    let result = query_time_range(&records, 500, 1000);
    println!("Query [500, 1000]: {} records", result.len());
    for r in result {
        println!("  {:?}", r);
    }

    // Partitioning predicate
    println!("\n=== Partition by Predicate ===\n");

    // partition_point works when slice is partitioned (not just sorted)
    let partitioned = vec![2, 4, 6, 8, 1, 3, 5, 7]; // Even first, then odd
    println!("Partitioned (evens first): {:?}", partitioned);

    let first_odd = partitioned.partition_point(|&x| x % 2 == 0);
    println!("First odd at index {}: {}", first_odd, partitioned[first_odd]);

    let (evens, odds) = partitioned.split_at(first_odd);
    println!("Evens: {:?}", evens);
    println!("Odds: {:?}", odds);

    println!("\n=== Key Points ===");
    println!("1. partition_point finds where predicate transitions false->true");
    println!("2. Slice must be partitioned by the predicate");
    println!("3. Use two partition_points for range queries");
    println!("4. O(log n) performance for both endpoints");
}
