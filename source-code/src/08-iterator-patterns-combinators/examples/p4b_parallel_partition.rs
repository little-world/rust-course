//! Pattern 4b: Parallel Iteration with Rayon
//! Example: Parallel Partition
//!
//! Run with: cargo run --example p4b_parallel_partition

use rayon::prelude::*;

/// Partition numbers into even and odd in parallel.
fn parallel_partition(numbers: Vec<i32>) -> (Vec<i32>, Vec<i32>) {
    numbers.into_par_iter().partition(|&x| x % 2 == 0)
}

/// Partition with custom predicate.
fn partition_by<T, F>(items: Vec<T>, predicate: F) -> (Vec<T>, Vec<T>)
where
    T: Send,
    F: Fn(&T) -> bool + Sync,
{
    items.into_par_iter().partition(|item| predicate(item))
}

/// Partition map: different transformation for each group.
fn partition_map<T, U, V, F, G>(
    items: Vec<T>,
    predicate: impl Fn(&T) -> bool + Sync,
    true_map: F,
    false_map: G,
) -> (Vec<U>, Vec<V>)
where
    T: Send,
    U: Send,
    V: Send,
    F: Fn(T) -> U + Sync,
    G: Fn(T) -> V + Sync,
{
    items.into_par_iter().partition_map(|item| {
        if predicate(&item) {
            rayon::iter::Either::Left(true_map(item))
        } else {
            rayon::iter::Either::Right(false_map(item))
        }
    })
}

fn main() {
    println!("=== Parallel Partition ===\n");

    // Usage: split numbers into even and odd in parallel
    let numbers: Vec<i32> = (1..=10).collect();
    println!("Numbers: {:?}", numbers);

    let (evens, odds) = parallel_partition(numbers);
    println!("Evens: {:?}", evens);
    println!("Odds: {:?}", odds);

    // Partition strings
    println!("\n=== Partition Strings by Length ===");
    let words = vec!["a", "bb", "ccc", "dddd", "ee", "f", "ggg"];
    println!("Words: {:?}", words);

    let (short, long) = partition_by(words, |s| s.len() <= 2);
    println!("Short (<=2): {:?}", short);
    println!("Long (>2): {:?}", long);

    // Partition with transformation
    println!("\n=== Partition Map ===");
    let mixed: Vec<i32> = vec![-3, 1, -2, 4, -5, 6, 7, -8];
    println!("Mixed numbers: {:?}", mixed);

    // Positive -> squared, Negative -> absolute value as string
    let (positive_squares, negative_abs): (Vec<i32>, Vec<String>) = partition_map(
        mixed,
        |&x| x > 0,
        |x| x * x,
        |x| format!("abs({})", x.abs()),
    );
    println!("Positive (squared): {:?}", positive_squares);
    println!("Negative (as string): {:?}", negative_abs);

    // Practical example: valid vs invalid data
    println!("\n=== Practical Example: Validation ===");
    #[derive(Debug, Clone)]
    struct User {
        name: String,
        age: i32,
    }

    let users = vec![
        User { name: "Alice".into(), age: 30 },
        User { name: "".into(), age: 25 },        // Invalid: empty name
        User { name: "Bob".into(), age: -5 },     // Invalid: negative age
        User { name: "Carol".into(), age: 35 },
        User { name: "Dave".into(), age: 150 },   // Invalid: unrealistic age
    ];

    fn is_valid_user(u: &User) -> bool {
        !u.name.is_empty() && u.age > 0 && u.age < 120
    }

    let (valid, invalid): (Vec<User>, Vec<User>) =
        users.into_par_iter().partition(is_valid_user);

    println!("Valid users:");
    for u in &valid {
        println!("  {:?}", u);
    }
    println!("Invalid users:");
    for u in &invalid {
        println!("  {:?}", u);
    }

    println!("\n=== Partition vs Filter ===");
    println!("filter: keeps only matching elements");
    println!("partition: keeps ALL elements, splits into two groups");
    println!("");
    println!("Use partition when you need both groups!");

    println!("\n=== Key Points ===");
    println!("1. partition() splits into two Vecs based on predicate");
    println!("2. into_par_iter() for ownership transfer");
    println!("3. partition_map() allows different transformations");
    println!("4. Useful for validation, categorization, split processing");
}
