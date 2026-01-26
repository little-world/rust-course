//! Pattern 4: Extension Traits
//! Example: Blanket Iterator Extensions
//!
//! Run with: cargo run --example p4_iterator_extension

use std::collections::HashMap;

// Extension trait for all iterators
trait IteratorExt: Iterator {
    // Count occurrences of each item
    fn counts(self) -> HashMap<Self::Item, usize>
    where
        Self: Sized,
        Self::Item: Eq + std::hash::Hash,
    {
        let mut map = HashMap::new();
        for item in self {
            *map.entry(item).or_insert(0) += 1;
        }
        map
    }

    // Collect into pairs
    fn pairs(self) -> Vec<(Self::Item, Self::Item)>
    where
        Self: Sized,
    {
        let mut result = Vec::new();
        let mut iter = self;
        while let (Some(a), Some(b)) = (iter.next(), iter.next()) {
            result.push((a, b));
        }
        result
    }

    // Take while predicate and also return the first failing element
    fn take_while_inclusive<P>(self, predicate: P) -> Vec<Self::Item>
    where
        Self: Sized,
        P: Fn(&Self::Item) -> bool,
    {
        let mut result = Vec::new();
        for item in self {
            let matches = predicate(&item);
            result.push(item);
            if !matches {
                break;
            }
        }
        result
    }

    // Interleave with another iterator
    fn interleave<I>(self, other: I) -> Vec<Self::Item>
    where
        Self: Sized,
        I: Iterator<Item = Self::Item>,
    {
        let mut result = Vec::new();
        let mut iter1 = self;
        let mut iter2 = other;
        loop {
            match (iter1.next(), iter2.next()) {
                (Some(a), Some(b)) => {
                    result.push(a);
                    result.push(b);
                }
                (Some(a), None) => result.push(a),
                (None, Some(b)) => result.push(b),
                (None, None) => break,
            }
        }
        result
    }
}

// Blanket impl: applies to any type that is an Iterator.
impl<I: Iterator> IteratorExt for I {}

fn main() {
    // Usage: Blanket impl gives counts() to all iterators automatically.
    println!("=== counts() ===");
    let words = vec!["apple", "banana", "apple", "cherry", "banana", "apple"];
    let counts = words.iter().counts();
    println!("Word counts: {:?}", counts);

    let numbers = vec![1, 2, 2, 3, 3, 3, 4, 4, 4, 4];
    let num_counts = numbers.iter().counts();
    println!("Number counts: {:?}", num_counts);

    println!("\n=== pairs() ===");
    let items = vec![1, 2, 3, 4, 5, 6];
    let pairs = items.into_iter().pairs();
    println!("Pairs: {:?}", pairs);

    println!("\n=== take_while_inclusive() ===");
    let nums = vec![1, 2, 3, 10, 4, 5];
    let taken = nums.into_iter().take_while_inclusive(|&x| x < 5);
    println!("Take while < 5 (inclusive): {:?}", taken);

    println!("\n=== interleave() ===");
    let a = vec![1, 3, 5];
    let b = vec![2, 4, 6, 8];
    let interleaved = a.into_iter().interleave(b.into_iter());
    println!("Interleaved: {:?}", interleaved);

    // Works with any iterator
    println!("\n=== Works with Range ===");
    let range_pairs = (0..8).pairs();
    println!("Range pairs: {:?}", range_pairs);
}
