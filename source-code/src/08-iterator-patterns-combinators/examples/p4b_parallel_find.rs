//! Pattern 4b: Parallel Iteration with Rayon
//! Example: Parallel Find with Early Exit
//!
//! Run with: cargo run --example p4b_parallel_find

use rayon::prelude::*;

/// Find position of target using parallel search.
/// Returns an arbitrary match position (not necessarily first).
fn parallel_find_first(numbers: &[i32], target: i32) -> Option<usize> {
    numbers.par_iter().position_any(|&x| x == target)
}

/// Find first element matching a predicate.
fn parallel_find<T: Sync>(items: &[T], predicate: impl Fn(&T) -> bool + Sync) -> Option<&T> {
    items.par_iter().find_any(|item| predicate(item))
}

/// Check if any element matches (short-circuit).
fn parallel_any<T: Sync>(items: &[T], predicate: impl Fn(&T) -> bool + Sync) -> bool {
    items.par_iter().any(|item| predicate(item))
}

/// Check if all elements match (short-circuit on false).
fn parallel_all<T: Sync>(items: &[T], predicate: impl Fn(&T) -> bool + Sync) -> bool {
    items.par_iter().all(|item| predicate(item))
}

fn main() {
    println!("=== Parallel Find with Early Exit ===\n");

    // Usage: find element position using parallel search
    let numbers: Vec<i32> = (0..1000).collect();
    let idx = parallel_find_first(&numbers, 500);
    println!("Position of 500 in 0..1000: {:?}", idx);

    // Not found
    let missing = parallel_find_first(&numbers, 9999);
    println!("Position of 9999: {:?}", missing);

    println!("\n=== position_any vs position_first ===");
    let data: Vec<i32> = (0..100).collect();

    // position_any returns ANY matching position (fastest)
    let any_pos = data.par_iter().position_any(|&x| x > 50);
    println!("position_any(x > 50): {:?}", any_pos);

    // position_first returns FIRST matching position (slower, maintains order)
    let first_pos = data.par_iter().position_first(|&x| x > 50);
    println!("position_first(x > 50): {:?}", first_pos);
    // Should be Some(51)

    println!("\n=== find_any vs find_first ===");
    let words = vec!["apple", "banana", "apricot", "cherry", "avocado"];

    // find_any - fastest, any match
    let any_a = parallel_find(&words, |s| s.starts_with('a'));
    println!("find_any starting with 'a': {:?}", any_a);

    // find_first - first match in order
    let first_a = words.par_iter().find_first(|s| s.starts_with('a'));
    println!("find_first starting with 'a': {:?}", first_a);

    println!("\n=== Parallel any() and all() ===");
    let numbers2: Vec<i32> = (1..=1000).collect();

    // any - short-circuits on first true
    let has_big = parallel_any(&numbers2, |&x| x > 900);
    println!("Any > 900? {}", has_big);

    // all - short-circuits on first false
    let all_positive = parallel_all(&numbers2, |&x| x > 0);
    println!("All positive? {}", all_positive);

    let all_small = parallel_all(&numbers2, |&x| x < 500);
    println!("All < 500? {}", all_small);

    println!("\n=== Early Exit Behavior ===");
    println!("Parallel search can find matches faster than sequential:");
    println!("  - Multiple threads search different parts");
    println!("  - First thread to find stops all others");
    println!("");
    println!("Important: position_any/find_any return ANY match");
    println!("Use position_first/find_first if order matters");

    println!("\n=== Performance Note ===");
    println!("Early exit is more beneficial when:");
    println!("  - Matches are rare (need to search more)");
    println!("  - Match might be anywhere in the data");
    println!("  - Data is large enough to benefit from parallelism");

    println!("\n=== Key Points ===");
    println!("1. position_any/find_any: fastest, any match OK");
    println!("2. position_first/find_first: ordered, first match");
    println!("3. any()/all(): parallel short-circuit evaluation");
    println!("4. Early exit stops other workers on match");
}
