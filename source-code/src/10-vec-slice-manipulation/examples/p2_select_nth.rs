//! Pattern 2: Slice Algorithms
//! Example: Finding Median and Top-K Elements
//!
//! Run with: cargo run --example p2_select_nth

fn main() {
    println!("=== Finding Median and Top-K Elements ===\n");

    // Find median
    fn find_median(values: &mut [f64]) -> f64 {
        let mid = values.len() / 2;
        let (_, median, _) = values.select_nth_unstable_by(mid, |a, b| {
            a.partial_cmp(b).unwrap()
        });
        *median
    }

    let mut data = vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0, 5.0];
    println!("Data: {:?}", data);

    let median = find_median(&mut data);
    println!("Median: {}", median);

    // Note: data is now partially sorted around median
    println!("After select_nth: {:?}", data);

    // Find top-K elements
    println!("\n=== Top-K Elements ===\n");

    fn top_k_elements(values: &mut [i32], k: usize) -> &[i32] {
        let idx = values.len() - k;
        let (_, _, right) = values.select_nth_unstable(idx);
        right
    }

    let mut data = vec![1, 5, 2, 8, 3, 9, 4, 7, 6];
    println!("Data: {:?}", data);

    let top3 = top_k_elements(&mut data, 3);
    println!("Top 3 elements: {:?}", top3);

    // Performance comparison with full sort
    println!("\n=== Performance: select_nth vs Full Sort ===\n");

    let n = 100_000;

    // Using select_nth_unstable (O(n))
    let mut data1: Vec<i32> = (0..n).map(|_| rand_simple() as i32).collect();
    let start = std::time::Instant::now();
    data1.select_nth_unstable(n / 2);
    let select_time = start.elapsed();

    // Using full sort (O(n log n))
    let mut data2: Vec<i32> = (0..n).map(|_| rand_simple() as i32).collect();
    let start = std::time::Instant::now();
    data2.sort_unstable();
    let _median = data2[n / 2];
    let sort_time = start.elapsed();

    println!("Finding median of {} elements:", n);
    println!("  select_nth_unstable: {:?}", select_time);
    println!("  sort then index:     {:?}", sort_time);

    // Percentiles
    println!("\n=== Computing Percentiles ===\n");

    fn percentile(values: &mut [f64], p: f64) -> f64 {
        assert!((0.0..=100.0).contains(&p));
        let idx = ((values.len() - 1) as f64 * p / 100.0).round() as usize;
        let (_, value, _) = values.select_nth_unstable_by(idx, |a, b| {
            a.partial_cmp(b).unwrap()
        });
        *value
    }

    let mut data: Vec<f64> = (0..100).map(|i| i as f64).collect();

    for p in [0.0, 25.0, 50.0, 75.0, 100.0] {
        let mut data_copy = data.clone();
        let value = percentile(&mut data_copy, p);
        println!("  {}th percentile: {}", p, value);
    }

    // Find K smallest
    println!("\n=== K Smallest Elements ===\n");

    fn k_smallest(values: &mut [i32], k: usize) -> &mut [i32] {
        if k >= values.len() {
            return values;
        }
        let (left, _, _) = values.select_nth_unstable(k);
        left
    }

    let mut data = vec![9, 3, 7, 1, 5, 8, 2, 6, 4];
    println!("Data: {:?}", data);

    let smallest3 = k_smallest(&mut data, 3);
    println!("3 smallest: {:?}", smallest3);

    println!("\n=== Key Points ===");
    println!("1. select_nth_unstable is O(n), sorting is O(n log n)");
    println!("2. Returns (less_than, nth_element, greater_than)");
    println!("3. Elements are partially sorted around the nth element");
    println!("4. Perfect for median, percentiles, top-K queries");
}

/// Simple pseudo-random for demo.
fn rand_simple() -> u64 {
    use std::time::SystemTime;
    static mut SEED: u64 = 0;
    unsafe {
        if SEED == 0 {
            SEED = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;
        }
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        SEED
    }
}
