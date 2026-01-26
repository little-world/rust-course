//! Pattern 3: Trait Bounds and Constraints
//! Example: Combining Bounds with impl Trait
//!
//! Run with: cargo run --example p3_impl_trait

// impl Trait in return position hides the concrete type
fn make_iterator() -> impl Iterator<Item = i32> {
    (0..10).filter(|x| x % 2 == 0)
}

fn make_counter(start: i32) -> impl Iterator<Item = i32> {
    std::iter::successors(Some(start), |n| Some(n + 1))
}

// impl Trait in argument position (sugar for generics)
fn transform<T: Clone>(items: impl IntoIterator<Item = T>) -> Vec<T> {
    items.into_iter().collect()
}

fn process_iterator(iter: impl Iterator<Item = i32>) -> i32 {
    iter.sum()
}

// Multiple impl Trait bounds
fn combine_iterators(
    a: impl Iterator<Item = i32>,
    b: impl Iterator<Item = i32>,
) -> impl Iterator<Item = i32> {
    a.chain(b)
}

// Returning closures with impl Trait
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

fn make_multiplier(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x * n
}

// impl Trait with additional bounds
fn debug_iterator() -> impl Iterator<Item = i32> + Clone {
    vec![1, 2, 3].into_iter()
}

fn main() {
    println!("=== impl Trait in Return Position ===");
    // Usage: impl Trait hides concrete type while exposing capabilities.
    let evens: Vec<_> = make_iterator().collect();
    println!("make_iterator().collect() = {:?}", evens);

    let first_5: Vec<_> = make_counter(1).take(5).collect();
    println!("make_counter(1).take(5) = {:?}", first_5);

    println!("\n=== impl Trait in Argument Position ===");
    // Works with Vec
    let v = transform(vec![1, 2, 3]);
    println!("transform(vec![1, 2, 3]) = {:?}", v);

    // Also works with array
    let v2 = transform([4, 5, 6]);
    println!("transform([4, 5, 6]) = {:?}", v2);

    // Also works with range
    let v3 = transform(0..5);
    println!("transform(0..5) = {:?}", v3);

    println!("\n=== Processing Iterators ===");
    let sum1 = process_iterator(vec![1, 2, 3].into_iter());
    println!("process_iterator([1, 2, 3]) = {}", sum1);

    let sum2 = process_iterator(0..=10);
    println!("process_iterator(0..=10) = {}", sum2);

    println!("\n=== Combining Iterators ===");
    let combined: Vec<_> = combine_iterators(
        vec![1, 2, 3].into_iter(),
        vec![4, 5, 6].into_iter(),
    ).collect();
    println!("combine_iterators([1,2,3], [4,5,6]) = {:?}", combined);

    println!("\n=== Returning Closures ===");
    let add_5 = make_adder(5);
    let times_3 = make_multiplier(3);

    println!("make_adder(5)(10) = {}", add_5(10));
    println!("make_multiplier(3)(10) = {}", times_3(10));

    // Composing closures
    let result = times_3(add_5(7)); // (7 + 5) * 3 = 36
    println!("times_3(add_5(7)) = {}", result);

    println!("\n=== impl Trait with Additional Bounds ===");
    let iter = debug_iterator();
    let iter_clone = iter.clone();

    let v1: Vec<_> = iter.collect();
    let v2: Vec<_> = iter_clone.collect();
    println!("Original: {:?}", v1);
    println!("Cloned: {:?}", v2);
}
