//! Pattern 2: Generic Structs and Enums
//! Example: Generic Enum with Variants (BinaryTree)
//!
//! Run with: cargo run --example p2_binary_tree

use std::cmp::Ordering;

enum BinaryTree<T> {
    Empty,
    Node {
        value: T,
        left: Box<BinaryTree<T>>,
        right: Box<BinaryTree<T>>,
    },
}

impl<T: Ord> BinaryTree<T> {
    fn new() -> Self {
        BinaryTree::Empty
    }

    fn insert(&mut self, item: T) {
        match self {
            BinaryTree::Empty => {
                *self = BinaryTree::Node {
                    value: item,
                    left: Box::new(BinaryTree::Empty),
                    right: Box::new(BinaryTree::Empty),
                };
            }
            BinaryTree::Node { value, left, right } => {
                if item < *value {
                    left.insert(item);
                } else {
                    right.insert(item);
                }
            }
        }
    }

    fn contains(&self, target: &T) -> bool {
        match self {
            BinaryTree::Empty => false,
            BinaryTree::Node { value, left, right } => match target.cmp(value) {
                Ordering::Equal => true,
                Ordering::Less => left.contains(target),
                Ordering::Greater => right.contains(target),
            },
        }
    }

    fn in_order<'a>(&'a self, result: &mut Vec<&'a T>) {
        if let BinaryTree::Node { value, left, right } = self {
            left.in_order(result);
            result.push(value);
            right.in_order(result);
        }
    }
}

fn main() {
    println!("=== Generic Binary Tree with Integers ===");
    // Usage: Generic tree works with any Ord type.
    let mut tree = BinaryTree::new();
    tree.insert(5);
    tree.insert(3);
    tree.insert(7);
    tree.insert(1);
    tree.insert(9);

    println!("Inserted: 5, 3, 7, 1, 9");
    println!("contains(&5) = {}", tree.contains(&5));
    println!("contains(&3) = {}", tree.contains(&3));
    println!("contains(&100) = {}", tree.contains(&100));

    let mut sorted = Vec::new();
    tree.in_order(&mut sorted);
    println!("In-order traversal: {:?}", sorted);

    println!("\n=== Generic Binary Tree with Strings ===");
    let mut str_tree = BinaryTree::new();
    str_tree.insert("banana");
    str_tree.insert("apple");
    str_tree.insert("cherry");
    str_tree.insert("date");

    println!("Inserted: banana, apple, cherry, date");
    println!("contains(\"apple\") = {}", str_tree.contains(&"apple"));
    println!("contains(\"fig\") = {}", str_tree.contains(&"fig"));

    let mut sorted_str = Vec::new();
    str_tree.in_order(&mut sorted_str);
    println!("In-order traversal: {:?}", sorted_str);
}
