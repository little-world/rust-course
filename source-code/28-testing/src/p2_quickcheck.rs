// Pattern 2: Property-Based Testing with QuickCheck
// Demonstrates QuickCheck as an alternative to proptest.

use quickcheck_macros::quickcheck;

// ============================================================================
// Example: QuickCheck
// ============================================================================

#[quickcheck]
fn reverse_twice_is_identity(vec: Vec<i32>) -> bool {
    let mut reversed = vec.clone();
    reversed.reverse();
    reversed.reverse();
    vec == reversed
}

#[quickcheck]
fn concat_length(a: Vec<i32>, b: Vec<i32>) -> bool {
    let mut c = a.clone();
    c.extend(b.iter());
    c.len() == a.len() + b.len()
}

// Additional quickcheck examples

#[quickcheck]
fn sort_is_idempotent(mut vec: Vec<i32>) -> bool {
    vec.sort();
    let mut sorted_again = vec.clone();
    sorted_again.sort();
    vec == sorted_again
}

#[quickcheck]
fn sort_preserves_length(vec: Vec<i32>) -> bool {
    let mut sorted = vec.clone();
    sorted.sort();
    sorted.len() == vec.len()
}

fn main() {
    println!("QuickCheck property-based testing - run with: cargo test");
    println!("QuickCheck's syntax is slightly different from proptest.");
}
