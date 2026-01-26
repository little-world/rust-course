//! Pattern 3: Trait Bounds and Constraints
//! Example: Various Trait Bound Patterns
//!
//! Run with: cargo run --example p3_trait_bounds

use std::fmt::{Debug, Display};

// Single trait bound
fn print_debug<T: Debug>(value: T) {
    println!("{:?}", value);
}

fn clone_it<T: Clone>(value: &T) -> T {
    value.clone()
}

// Multiple trait bounds with +
fn compare_and_show<T: PartialOrd + Display>(a: T, b: T) {
    if a < b {
        println!("{} < {}", a, b);
    } else if a > b {
        println!("{} > {}", a, b);
    } else {
        println!("{} == {}", a, b);
    }
}

// Where clause for complex bounds
fn complex_operation<T, U>(t: T, u: U)
where
    T: Clone + Debug + Default,
    U: AsRef<str> + Display,
{
    let cloned = t.clone();
    println!("T (cloned): {:?}, U: {}", cloned, u);
}

// Bounds on associated types
fn sum_iterator<I: Iterator<Item = i32>>(iter: I) -> i32 {
    iter.sum()
}

fn print_all<I>(iter: I)
where
    I: Iterator,
    I::Item: Display,
{
    for item in iter {
        println!("  {}", item);
    }
}

fn main() {
    println!("=== Single Trait Bound ===");
    // Usage: Bounds unlock specific operations on generic types.
    print_debug(vec![1, 2, 3]);
    print_debug(("tuple", 42, 3.14));

    let original = vec![1, 2, 3];
    let cloned = clone_it(&original);
    println!("Original: {:?}, Cloned: {:?}", original, cloned);

    println!("\n=== Multiple Trait Bounds ===");
    // Usage: Multiple bounds combined with + or where clause.
    compare_and_show(1, 2);
    compare_and_show("banana", "apple");
    compare_and_show(3.14, 3.14);

    println!("\n=== Where Clause ===");
    complex_operation(42, "hello");
    complex_operation(vec![1, 2], String::from("world"));

    println!("\n=== Bounds on Associated Types ===");
    // Usage: Constrain iterator's Item type for specific operations.
    let sum = sum_iterator(vec![1, 2, 3, 4, 5].into_iter());
    println!("sum of [1, 2, 3, 4, 5] = {}", sum);

    let sum2 = sum_iterator([10, 20, 30].into_iter());
    println!("sum of [10, 20, 30] = {}", sum2);

    println!("\nprint_all for [\"a\", \"b\", \"c\"]:");
    print_all(vec!["a", "b", "c"].into_iter());

    println!("\nprint_all for [1, 2, 3]:");
    print_all([1, 2, 3].into_iter());
}
