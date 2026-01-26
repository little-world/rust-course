//! Pattern 4b: Parallel Iteration with Rayon
//! Example: Joining Parallel Computations
//!
//! Run with: cargo run --example p4b_parallel_join

use rayon::prelude::*;

/// Run two independent computations in parallel and get both results.
fn parallel_join_example(data: &[i32]) -> (i32, i32) {
    rayon::join(
        || data.par_iter().sum(),     // Compute sum
        || data.par_iter().product(), // Compute product
    )
}

/// More complex join with different return types.
fn compute_stats(numbers: &[f64]) -> (f64, f64, f64) {
    let ((sum, count), (min, max)) = rayon::join(
        || {
            // Compute sum and count for average
            numbers.par_iter().fold(
                || (0.0, 0usize),
                |(s, c), &x| (s + x, c + 1),
            ).reduce(
                || (0.0, 0),
                |(s1, c1), (s2, c2)| (s1 + s2, c1 + c2),
            )
        },
        || {
            // Compute min and max
            let min = numbers.par_iter().cloned().reduce(|| f64::INFINITY, f64::min);
            let max = numbers.par_iter().cloned().reduce(|| f64::NEG_INFINITY, f64::max);
            (min, max)
        },
    );

    let avg = if count > 0 { sum / count as f64 } else { 0.0 };
    (avg, min, max)
}

/// Nested joins for more than two tasks.
fn triple_join<A, B, C, FA, FB, FC>(a: FA, b: FB, c: FC) -> (A, B, C)
where
    A: Send,
    B: Send,
    C: Send,
    FA: FnOnce() -> A + Send,
    FB: FnOnce() -> B + Send,
    FC: FnOnce() -> C + Send,
{
    let (a_result, (b_result, c_result)) = rayon::join(a, || rayon::join(b, c));
    (a_result, b_result, c_result)
}

/// Join with dependent computation (second uses first's result).
fn dependent_join(data: &[i32]) -> (i32, Vec<i32>) {
    let sum: i32 = data.par_iter().sum();

    // Now compute normalized values (depends on sum)
    let normalized: Vec<i32> = data.par_iter().map(|&x| x * 100 / sum).collect();

    (sum, normalized)
}

fn main() {
    println!("=== Joining Parallel Computations ===\n");

    // Basic join example
    let data: Vec<i32> = vec![1, 2, 3, 4, 5];
    println!("Data: {:?}", data);

    let (sum, product) = parallel_join_example(&data);
    println!("Sum: {}, Product: {}", sum, product);
    // Sum: 15, Product: 120

    println!("\n=== Computing Statistics in Parallel ===");
    let numbers: Vec<f64> = vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0];
    println!("Numbers: {:?}", numbers);

    let (avg, min, max) = compute_stats(&numbers);
    println!("Average: {:.2}, Min: {}, Max: {}", avg, min, max);

    println!("\n=== Triple Join ===");
    let result = triple_join(
        || "computation A",
        || 42,
        || vec![1, 2, 3],
    );
    println!("Results: {:?}", result);

    println!("\n=== Dependent Computation ===");
    let values = vec![10, 20, 30, 40];
    println!("Values: {:?}", values);
    let (total, percentages) = dependent_join(&values);
    println!("Total: {}", total);
    println!("Percentages (x100): {:?}", percentages);

    println!("\n=== How rayon::join Works ===");
    println!("rayon::join(a, b):");
    println!("  1. Schedules closure 'b' to run on another thread");
    println!("  2. Runs closure 'a' on the current thread");
    println!("  3. Waits for 'b' to complete");
    println!("  4. Returns (result_a, result_b)");
    println!("");
    println!("Key: The calling thread participates in the work!");

    println!("\n=== join vs scope ===");
    println!("join: exactly 2 tasks, returns both results");
    println!("scope: any number of tasks, returns ()");
    println!("");
    println!("Use join for:");
    println!("  - Divide-and-conquer algorithms");
    println!("  - Computing independent results");
    println!("  - When you need both return values");

    println!("\n=== Efficiency Note ===");
    println!("join is more efficient than spawning threads because:");
    println!("  - No thread creation overhead");
    println!("  - Uses Rayon's existing thread pool");
    println!("  - Work-stealing keeps all cores busy");

    println!("\n=== Key Points ===");
    println!("1. rayon::join() runs two closures in parallel");
    println!("2. Calling thread executes one closure (no waste!)");
    println!("3. Returns tuple with both results");
    println!("4. Nest joins for more than 2 parallel tasks");
}
