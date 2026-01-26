//! Pattern 2: Generic Structs and Enums
//! Example: Basic Generic Structs
//!
//! Run with: cargo run --example p2_generic_struct

use std::ops::Add;

#[derive(Debug, PartialEq, Clone)]
struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn new(x: T, y: T) -> Self {
        Point { x, y }
    }
}

impl<T: Copy + Add<Output = T>> Point<T> {
    fn add(&self, other: &Point<T>) -> Point<T> {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

// Multiple type parameters
#[derive(Debug)]
struct Pair<T, U> {
    first: T,
    second: U,
}

impl<T, U> Pair<T, U> {
    fn new(first: T, second: U) -> Self {
        Pair { first, second }
    }

    fn swap(self) -> Pair<U, T> {
        Pair {
            first: self.second,
            second: self.first,
        }
    }

    fn mix<V, W>(self, other: Pair<V, W>) -> Pair<T, W> {
        Pair {
            first: self.first,
            second: other.second,
        }
    }
}

// Generic struct with lifetime
struct Ref<'a, T> {
    value: &'a T,
}

impl<'a, T> Ref<'a, T> {
    fn new(value: &'a T) -> Self {
        Ref { value }
    }

    fn get(&self) -> &T {
        self.value
    }
}

fn main() {
    println!("=== Basic Generic Struct ===");
    // Usage: Same struct works with i32, f64, or any addable type.
    let p1 = Point::new(1, 2);
    let p2 = Point::new(3, 4);
    let sum = p1.add(&p2);
    println!("Point<i32>: {:?} + {:?} = {:?}", p1, p2, sum);

    let fp1 = Point::new(1.5, 2.5);
    let fp2 = Point::new(0.5, 0.5);
    let fsum = fp1.add(&fp2);
    println!("Point<f64>: {:?} + {:?} = {:?}", fp1, fp2, fsum);

    println!("\n=== Multiple Type Parameters ===");
    // Usage: Different type parameters allow heterogeneous pairs.
    let p = Pair::new(42, "hello");
    println!("Pair<i32, &str>: {:?}", p);

    let swapped = p.swap();
    println!("After swap: Pair<&str, i32>: {:?}", swapped);

    let p1 = Pair::new("first", 100);
    let p2 = Pair::new(3.14, "second");
    let mixed = p1.mix(p2);
    println!("mix(Pair(\"first\", 100), Pair(3.14, \"second\")) = {:?}", mixed);

    println!("\n=== Generic Struct with Lifetime ===");
    // Usage: Lifetime ensures Ref doesn't outlive referenced data.
    let num = 42;
    let r = Ref::new(&num);
    let value = r.get();
    println!("Ref {{ value: &{} }}.get() = {}", num, value);

    let text = String::from("hello");
    let r2 = Ref::new(&text);
    println!("Ref {{ value: &\"{}\" }}.get() = {}", text, r2.get());
}
