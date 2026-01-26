// Pattern 1: Profiling Strategies
// Demonstrates profiling techniques for finding performance bottlenecks.

use std::time::Instant;

// ============================================================================
// Example: Which bottleneck
// ============================================================================

fn process_data(items: Vec<String>) -> Vec<String> {
    items.iter()
        .filter(|s| validate(s))
        .map(|s| transform(s))
        .collect()
}

fn validate(s: &str) -> bool {
    s.len() > 10 && s.chars().all(|c| c.is_alphanumeric())
}

fn transform(s: &str) -> String {
    s.to_uppercase()
}

// ============================================================================
// Example: Micro-Benchmarking Best Practices
// ============================================================================

fn sum_loop(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data {
        sum += x;
    }
    sum
}

fn sum_iter(data: &[i32]) -> i32 {
    data.iter().sum()
}

fn sum_fold(data: &[i32]) -> i32 {
    data.iter().fold(0, |acc, &x| acc + x)
}

// ============================================================================
// Demonstration
// ============================================================================

fn benchmark_processing() {
    let data: Vec<String> = (0..1000)
        .map(|i| format!("item_number_{}", i))
        .collect();

    let start = Instant::now();
    for _ in 0..100 {
        let _ = process_data(data.clone());
    }
    println!("process_data (100 iterations): {:?}", start.elapsed());

    // Benchmark individual components
    let start = Instant::now();
    for _ in 0..100 {
        for item in &data {
            let _ = validate(item);
        }
    }
    println!("validate only (100 iterations): {:?}", start.elapsed());

    let start = Instant::now();
    for _ in 0..100 {
        for item in &data {
            let _ = transform(item);
        }
    }
    println!("transform only (100 iterations): {:?}", start.elapsed());
}

fn benchmark_sum_methods() {
    let data: Vec<i32> = (0..10000).collect();

    let start = Instant::now();
    for _ in 0..1000 {
        let _ = sum_loop(&data);
    }
    println!("sum_loop (1000 iterations): {:?}", start.elapsed());

    let start = Instant::now();
    for _ in 0..1000 {
        let _ = sum_iter(&data);
    }
    println!("sum_iter (1000 iterations): {:?}", start.elapsed());

    let start = Instant::now();
    for _ in 0..1000 {
        let _ = sum_fold(&data);
    }
    println!("sum_fold (1000 iterations): {:?}", start.elapsed());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate() {
        assert!(validate("alphanumeric123"));
        assert!(!validate("short"));
        assert!(!validate("has spaces here"));
    }

    #[test]
    fn test_transform() {
        assert_eq!(transform("hello"), "HELLO");
        assert_eq!(transform("MiXeD"), "MIXED");
    }

    #[test]
    fn test_process_data() {
        let input = vec![
            "short".to_string(),
            "validstring123".to_string(),
            "anothervalid456".to_string(),
        ];
        let result = process_data(input);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "VALIDSTRING123");
        assert_eq!(result[1], "ANOTHERVALID456");
    }

    #[test]
    fn test_sum_methods_equivalent() {
        let data: Vec<i32> = (0..100).collect();
        let expected: i32 = (0..100).sum();

        assert_eq!(sum_loop(&data), expected);
        assert_eq!(sum_iter(&data), expected);
        assert_eq!(sum_fold(&data), expected);
    }
}

fn main() {
    println!("Pattern 1: Profiling Strategies");
    println!("================================\n");

    println!("Benchmarking process_data components:");
    benchmark_processing();

    println!("\nBenchmarking sum implementations:");
    benchmark_sum_methods();

    println!("\nRun with: cargo flamegraph (for CPU profiling)");
    println!("Run with: cargo bench (for Criterion benchmarks)");
}
