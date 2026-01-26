//! Pattern 2: Zero-Allocation Iteration
//! Example: Chaining Adapters without Intermediate Collections
//!
//! Run with: cargo run --example p2_chaining

/// Process numbers by filtering, mapping, filtering again, and summing.
/// All in a single pass with no intermediate Vec created.
fn process_numbers(input: &[i32]) -> i32 {
    input
        .iter()
        .filter(|&&x| x > 0)      // Keep positive numbers
        .map(|&x| x * x)           // Square them
        .filter(|&x| x < 1000)     // Keep squares less than 1000
        .sum()                     // The iterator is consumed only at the end
}

fn main() {
    println!("=== Zero-Allocation Iteration ===\n");

    // Usage: filter, map, filter, and sum in a single pass
    let result = process_numbers(&[-1, 2, 3, 50]); // 2*2 + 3*3 = 4 + 9 = 13
    println!("process_numbers(&[-1, 2, 3, 50]) = {}", result);
    assert_eq!(result, 13);

    // Demonstrate that no intermediate allocations occur
    println!("\n=== Understanding Lazy Evaluation ===");
    let data = vec![-5, -3, 1, 2, 3, 4, 5, 40, 50];

    // This creates an iterator chain, not a Vec
    let iterator = data
        .iter()
        .filter(|&&x| {
            println!("  filter 1: checking {}", x);
            x > 0
        })
        .map(|&x| {
            println!("  map: squaring {}", x);
            x * x
        })
        .filter(|&x| {
            println!("  filter 2: checking {} < 1000", x);
            x < 1000
        });

    // At this point, nothing has been computed!
    println!("Iterator created, no work done yet.");
    println!("\nNow consuming with .sum():");

    // Work happens here, one element at a time:
    let result: i32 = iterator.sum();
    println!("\nResult: {}", result);

    // Show the memory efficiency
    println!("\n=== Memory Efficiency ===");
    let large_data: Vec<i32> = (0..1_000_000).collect();

    // This processes 1 million elements without allocating any intermediate Vec
    let sum: i64 = large_data
        .iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x as i64 * x as i64)
        .take(1000)
        .sum();

    println!("Sum of first 1000 squared even numbers: {}", sum);

    println!("\n=== Key Points ===");
    println!("1. Iterator adapters are lazy - they don't allocate");
    println!("2. Each adapter wraps the previous one (zero-cost abstraction)");
    println!("3. Work happens only when a consuming method is called");
    println!("4. Compiler often optimizes chains into a single loop");
}
