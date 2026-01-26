//! Pattern 4: Associated Types vs Generic Parameters
//! Example: Associated Type - One Impl Per Type
//!
//! Run with: cargo run --example p4_associated_types

// Associated types declare type inside trait; implementors specify concrete type
trait Container {
    type Item;
    fn get(&self, index: usize) -> Option<&Self::Item>;
    fn len(&self) -> usize;
}

impl<T> Container for Vec<T> {
    type Item = T;

    fn get(&self, index: usize) -> Option<&T> {
        self.as_slice().get(index)
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl<T, const N: usize> Container for [T; N] {
    type Item = T;

    fn get(&self, index: usize) -> Option<&T> {
        <[T]>::get(self, index)
    }

    fn len(&self) -> usize {
        N
    }
}

// Generic function using associated type
fn first_item<C: Container>(c: &C) -> Option<&C::Item> {
    c.get(0)
}

fn print_length<C: Container>(c: &C) {
    println!("Container length: {}", c.len());
}

// Associated type with bounds
use std::ops::Add;

trait Summable {
    type Item: Add<Output = Self::Item> + Default + Copy;
    fn items(&self) -> &[Self::Item];

    fn sum(&self) -> Self::Item {
        self.items()
            .iter()
            .copied()
            .fold(Self::Item::default(), |acc, x| acc + x)
    }
}

struct Numbers(Vec<i32>);

impl Summable for Numbers {
    type Item = i32;

    fn items(&self) -> &[i32] {
        &self.0
    }
}

struct Floats(Vec<f64>);

impl Summable for Floats {
    type Item = f64;

    fn items(&self) -> &[f64] {
        &self.0
    }
}

fn main() {
    println!("=== Container Trait with Associated Type ===");
    // Usage: Associated type inferred from container; no turbofish needed.
    let v = vec![1, 2, 3, 4, 5];
    let first = first_item(&v);
    println!("first_item(&vec![1, 2, 3, 4, 5]) = {:?}", first);
    print_length(&v);

    let arr = [10, 20, 30];
    let first_arr = first_item(&arr);
    println!("\nfirst_item(&[10, 20, 30]) = {:?}", first_arr);
    print_length(&arr);

    println!("\n=== Summable with Bounded Associated Type ===");
    // Usage: Bounded associated type enables default sum() implementation.
    let nums = Numbers(vec![1, 2, 3, 4, 5]);
    let total = nums.sum();
    println!("Numbers([1, 2, 3, 4, 5]).sum() = {}", total);

    let floats = Floats(vec![1.5, 2.5, 3.5]);
    let total_f = floats.sum();
    println!("Floats([1.5, 2.5, 3.5]).sum() = {}", total_f);

    println!("\n=== Associated Type vs Generic Parameter ===");
    println!("Associated types: ONE implementation per type");
    println!("  - Vec<T> implements Container with Item = T");
    println!("  - Iterator has Item as associated type");
    println!("  - User doesn't choose the type; it's determined by implementation");
}
