//! Pattern 4b: Parallel Iteration with Rayon
//! Example: Controlling Parallelism with Scope
//!
//! Run with: cargo run --example p4b_parallel_scope

/// Process data with explicit parallel scope.
/// split_at_mut ensures exclusive access for each spawn.
fn parallel_with_scope(data: &mut [i32]) {
    rayon::scope(|s| {
        let mid = data.len() / 2;
        let (left, right) = data.split_at_mut(mid);

        s.spawn(|_| {
            for x in left.iter_mut() {
                *x *= 2;
            }
        });

        s.spawn(|_| {
            for x in right.iter_mut() {
                *x *= 3;
            }
        });
    });
    // Both spawned tasks complete before scope returns
}

/// Recursive parallel processing with scope.
fn parallel_sum(data: &[i32]) -> i32 {
    const THRESHOLD: usize = 100;

    if data.len() <= THRESHOLD {
        // Base case: sequential
        data.iter().sum()
    } else {
        // Recursive case: split and parallel
        let mid = data.len() / 2;
        let (left, right) = data.split_at(mid);

        let (left_sum, right_sum) = rayon::join(
            || parallel_sum(left),
            || parallel_sum(right),
        );

        left_sum + right_sum
    }
}

/// Nested scopes for complex parallelism.
fn nested_parallel_processing(data: &mut [Vec<i32>]) {
    rayon::scope(|s| {
        for row in data.iter_mut() {
            s.spawn(move |_| {
                // Each row processed in parallel
                for x in row.iter_mut() {
                    *x = *x * *x;
                }
            });
        }
    });
}

fn main() {
    println!("=== Controlling Parallelism with Scope ===\n");

    // Basic scope example
    println!("=== Basic Scope ===");
    let mut data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    println!("Before: {:?}", data);

    parallel_with_scope(&mut data);
    println!("After (left*2, right*3): {:?}", data);
    // [2, 4, 6, 8, 10, 18, 21, 24, 27, 30]

    // Recursive parallel sum
    println!("\n=== Recursive Parallel Sum ===");
    let large_data: Vec<i32> = (1..=1000).collect();
    let sum = parallel_sum(&large_data);
    println!("Sum of 1..=1000: {}", sum);
    // 500500

    // Nested parallelism
    println!("\n=== Nested Parallel Processing ===");
    let mut matrix = vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ];
    println!("Before:");
    for row in &matrix {
        println!("  {:?}", row);
    }

    nested_parallel_processing(&mut matrix);
    println!("After (squared):");
    for row in &matrix {
        println!("  {:?}", row);
    }

    // Scope guarantees
    println!("\n=== Scope Guarantees ===");
    println!("1. All spawned tasks complete before scope returns");
    println!("2. Borrowed data can be safely accessed after scope");
    println!("3. Nested scopes work correctly");

    // How scope differs from spawn
    println!("\n=== scope vs thread::spawn ===");
    println!("thread::spawn:");
    println!("  - Returns JoinHandle, must explicitly join");
    println!("  - Data must be 'static or owned");
    println!("");
    println!("rayon::scope:");
    println!("  - Automatically joins all spawned tasks");
    println!("  - Can borrow data (with split_at_mut for exclusivity)");
    println!("  - Integrates with Rayon's work-stealing");

    // split_at_mut for exclusive access
    println!("\n=== split_at_mut for Exclusive Access ===");
    let mut arr = [1, 2, 3, 4, 5, 6];
    let (left, right) = arr.split_at_mut(3);
    println!("Original: [1, 2, 3, 4, 5, 6]");
    println!("Left:  {:?}", left);
    println!("Right: {:?}", right);
    println!("Each half can be modified independently!");

    println!("\n=== Key Points ===");
    println!("1. rayon::scope() for explicit parallel tasks");
    println!("2. All tasks complete before scope returns");
    println!("3. split_at_mut provides exclusive mutable access");
    println!("4. rayon::join() for exactly two parallel tasks");
}
