//! Pattern 5: Recoverable vs Unrecoverable Errors
//! Example: Debug Assertions
//!
//! Run with: cargo run --example p5_debug_assertions
//! Release:  cargo run --example p5_debug_assertions --release

/// Compute checksum with debug assertion for non-empty input.
fn compute_checksum(data: &[u8]) -> u32 {
    // Only checked in debug builds
    debug_assert!(!data.is_empty(), "Data must not be empty for checksum");
    data.iter().map(|&b| b as u32).sum()
}

/// Binary search with debug assertions for sorted input.
fn binary_search(arr: &[i32], target: i32) -> Option<usize> {
    // Expensive check only in debug builds
    debug_assert!(
        arr.windows(2).all(|w| w[0] <= w[1]),
        "Array must be sorted for binary search"
    );

    let mut low = 0;
    let mut high = arr.len();

    while low < high {
        let mid = (low + high) / 2;
        debug_assert!(mid < arr.len(), "BUG: mid out of bounds");

        if arr[mid] < target {
            low = mid + 1;
        } else if arr[mid] > target {
            high = mid;
        } else {
            return Some(mid);
        }
    }
    None
}

/// Matrix multiplication with dimension assertions.
fn matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    debug_assert!(!a.is_empty() && !b.is_empty(), "Matrices must not be empty");
    debug_assert!(
        a[0].len() == b.len(),
        "Matrix dimensions must match: a cols ({}) != b rows ({})",
        a[0].len(),
        b.len()
    );

    let rows = a.len();
    let cols = b[0].len();
    let inner = b.len();

    let mut result = vec![vec![0.0; cols]; rows];
    for i in 0..rows {
        for j in 0..cols {
            for k in 0..inner {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
}

fn main() {
    println!("=== Debug Assertions ===\n");

    #[cfg(debug_assertions)]
    println!("Running in DEBUG mode (assertions enabled)\n");

    #[cfg(not(debug_assertions))]
    println!("Running in RELEASE mode (assertions disabled)\n");

    // Checksum example
    println!("=== Checksum ===");
    let data = vec![1u8, 2, 3, 4, 5];
    let checksum = compute_checksum(&data);
    println!("  Checksum of {:?}: {}", data, checksum);

    // Binary search example
    println!("\n=== Binary Search ===");
    let sorted = vec![1, 3, 5, 7, 9, 11];
    for target in [5, 6, 1, 11] {
        match binary_search(&sorted, target) {
            Some(idx) => println!("  Found {} at index {}", target, idx),
            None => println!("  {} not found", target),
        }
    }

    // Matrix multiplication example
    println!("\n=== Matrix Multiplication ===");
    let a = vec![
        vec![1.0, 2.0],
        vec![3.0, 4.0],
    ];
    let b = vec![
        vec![5.0, 6.0],
        vec![7.0, 8.0],
    ];
    let c = matrix_multiply(&a, &b);
    println!("  A = {:?}", a);
    println!("  B = {:?}", b);
    println!("  A * B = {:?}", c);

    println!("\n=== debug_assert! vs assert! ===");
    println!("assert!:");
    println!("  - Always checked (debug and release)");
    println!("  - Use for critical invariants");
    println!("  - Example: bounds that must never be violated");
    println!();
    println!("debug_assert!:");
    println!("  - Only checked in debug builds");
    println!("  - Compiles to nothing in release");
    println!("  - Use for expensive checks");
    println!("  - Example: 'array must be sorted' (O(n) to verify)");

    println!("\n=== Variants ===");
    println!("debug_assert!(condition)");
    println!("debug_assert!(condition, \"message\")");
    println!("debug_assert!(condition, \"formatted: {{}}\", value)");
    println!("debug_assert_eq!(a, b)");
    println!("debug_assert_ne!(a, b)");

    println!("\n=== Use Cases ===");
    println!("1. Preconditions (input must be sorted/non-empty)");
    println!("2. Postconditions (result is valid)");
    println!("3. Loop invariants (index always in bounds)");
    println!("4. Expensive validation (full structure check)");

    println!("\n=== Key Points ===");
    println!("1. debug_assert! = zero cost in release builds");
    println!("2. Catches bugs during development");
    println!("3. Don't use for input validation (use Result)");
    println!("4. Great for documenting expectations");
}
