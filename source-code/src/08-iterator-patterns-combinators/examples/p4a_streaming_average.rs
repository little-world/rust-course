//! Pattern 4a: Streaming Algorithms
//! Example: Streaming Average Calculation
//!
//! Run with: cargo run --example p4a_streaming_average

/// A lightweight accumulator for computing average in one pass.
/// Requires O(1) memory regardless of input size.
struct StreamingAverage {
    sum: f64,
    count: usize,
}

impl StreamingAverage {
    fn new() -> Self {
        StreamingAverage { sum: 0.0, count: 0 }
    }

    fn add(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;
    }

    fn average(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f64)
        }
    }

    fn count(&self) -> usize {
        self.count
    }

    fn sum(&self) -> f64 {
        self.sum
    }
}

/// Compute streaming average from any iterator.
fn compute_streaming_average(numbers: impl Iterator<Item = f64>) -> Option<f64> {
    let mut avg = StreamingAverage::new();
    for num in numbers {
        avg.add(num);
    }
    avg.average()
}

/// Streaming statistics: min, max, sum, count, average.
struct StreamingStats {
    min: Option<f64>,
    max: Option<f64>,
    sum: f64,
    count: usize,
}

impl StreamingStats {
    fn new() -> Self {
        StreamingStats {
            min: None,
            max: None,
            sum: 0.0,
            count: 0,
        }
    }

    fn add(&mut self, value: f64) {
        self.min = Some(self.min.map_or(value, |m| m.min(value)));
        self.max = Some(self.max.map_or(value, |m| m.max(value)));
        self.sum += value;
        self.count += 1;
    }

    fn average(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f64)
        }
    }
}

fn main() {
    println!("=== Streaming Average Calculation ===\n");

    // Usage: compute average in a single pass
    let avg = compute_streaming_average([1.0, 2.0, 3.0, 4.0, 5.0].into_iter());
    println!("Average of [1, 2, 3, 4, 5]: {:?}", avg);
    assert_eq!(avg, Some(3.0));

    // Empty iterator
    let empty_avg = compute_streaming_average(std::iter::empty());
    println!("Average of []: {:?}", empty_avg);
    assert_eq!(empty_avg, None);

    println!("\n=== Incremental Updates ===");
    let mut stats = StreamingAverage::new();
    let values = [10.0, 20.0, 30.0, 40.0, 50.0];

    for (i, &v) in values.iter().enumerate() {
        stats.add(v);
        println!(
            "After adding {}: count={}, sum={}, avg={:.2}",
            v,
            stats.count(),
            stats.sum(),
            stats.average().unwrap()
        );
    }

    println!("\n=== Full Statistics in Single Pass ===");
    let data = [15.0, 22.0, 8.0, 45.0, 33.0, 19.0, 27.0];
    let mut full_stats = StreamingStats::new();
    for &v in &data {
        full_stats.add(v);
    }

    println!("Data: {:?}", data);
    println!("Min: {:?}", full_stats.min);
    println!("Max: {:?}", full_stats.max);
    println!("Sum: {}", full_stats.sum);
    println!("Count: {}", full_stats.count);
    println!("Average: {:?}", full_stats.average());

    println!("\n=== Large Data Simulation ===");
    // Simulate processing a large stream
    let large_stream = (1..=1_000_000).map(|x| x as f64);
    let large_avg = compute_streaming_average(large_stream);
    println!("Average of 1..=1_000_000: {:?}", large_avg);
    // Expected: 500000.5

    println!("\n=== Memory Efficiency ===");
    println!("StreamingAverage uses only:");
    println!("  - 8 bytes for sum (f64)");
    println!("  - 8 bytes for count (usize)");
    println!("Total: 16 bytes, regardless of input size!");

    println!("\n=== Key Points ===");
    println!("1. O(1) memory regardless of input size");
    println!("2. Suitable for unbounded streams");
    println!("3. Option return handles empty input gracefully");
    println!("4. Can compute multiple statistics in single pass");
}
