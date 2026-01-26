//! Pattern 1: Type-Safe Generic Functions
//! Example: Trait Bounds and Comparison
//!
//! Run with: cargo run --example p1_trait_bounds

use std::cmp::Ordering;
use std::fmt::{Debug, Display};

// Comparison functions with borrowing
fn compare<T: Ord>(a: &T, b: &T) -> Ordering {
    a.cmp(b)
}

fn min_ref<'a, T: Ord>(a: &'a T, b: &'a T) -> &'a T {
    if a <= b { a } else { b }
}

fn max_ref<'a, T: Ord>(a: &'a T, b: &'a T) -> &'a T {
    if a >= b { a } else { b }
}

// Where clause for complex bounds
fn complex_operation<T, U>(t: T, u: U)
where
    T: Clone + Debug + Default,
    U: AsRef<str> + Display,
{
    println!("T: {:?}, U: {}", t.clone(), u);
}

// Turbofish for explicit type specification
fn create_default<T: Default>() -> T {
    T::default()
}

fn main() {
    println!("=== Comparison Functions ===");
    // Usage: Borrow arguments for comparison without taking ownership.
    let ord = compare(&5, &10);
    println!("compare(&5, &10) = {:?}", ord);

    let min = min_ref(&"apple", &"banana");
    println!("min_ref(\"apple\", \"banana\") = {}", min);

    let max = max_ref(&100, &50);
    println!("max_ref(&100, &50) = {}", max);

    println!("\n=== Where Clause ===");
    // Usage: Multiple bounds combined with where clause.
    complex_operation(42, "hello");
    complex_operation(vec![1, 2, 3], String::from("world"));

    println!("\n=== Turbofish Syntax ===");
    // Usage: Turbofish specifies type when inference isn't enough.
    let parsed = "42".parse::<i32>().unwrap();
    let default_string = create_default::<String>();
    let default_vec: Vec<i32> = create_default();
    let collected: Vec<i32> = (0..5).collect();

    println!("\"42\".parse::<i32>() = {}", parsed);
    println!("create_default::<String>() = \"{}\"", default_string);
    println!("create_default::<Vec<i32>>() = {:?}", default_vec);
    println!("(0..5).collect::<Vec<i32>>() = {:?}", collected);
}
