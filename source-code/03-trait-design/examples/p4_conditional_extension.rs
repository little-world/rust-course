//! Pattern 4: Extension Traits
//! Example: Conditional Extensions Based on Type Capabilities
//!
//! Run with: cargo run --example p4_conditional_extension

use std::fmt::Debug;

// Extension for all Debug types
trait DebugExt {
    fn debug_print(&self);
    fn debug_string(&self) -> String;
}

impl<T: Debug> DebugExt for T {
    fn debug_print(&self) {
        println!("{:?}", self);
    }

    fn debug_string(&self) -> String {
        format!("{:?}", self)
    }
}

// Extension for Clone types
trait CloneExt: Clone {
    fn clone_n(&self, n: usize) -> Vec<Self>;
}

impl<T: Clone> CloneExt for T {
    fn clone_n(&self, n: usize) -> Vec<Self> {
        (0..n).map(|_| self.clone()).collect()
    }
}

// Extension for numeric types using trait bounds
trait NumericExt {
    fn is_pos(&self) -> bool;
    fn is_neg(&self) -> bool;
    fn difference(&self, other: &Self) -> Self;
}

impl NumericExt for i32 {
    fn is_pos(&self) -> bool {
        *self > 0
    }

    fn is_neg(&self) -> bool {
        *self < 0
    }

    fn difference(&self, other: &Self) -> Self {
        (self - other).abs()
    }
}

impl NumericExt for f64 {
    fn is_pos(&self) -> bool {
        *self > 0.0
    }

    fn is_neg(&self) -> bool {
        *self < 0.0
    }

    fn difference(&self, other: &Self) -> Self {
        (self - other).abs()
    }
}

// Extension that requires multiple bounds
trait ComparableDebug: Debug + PartialOrd {
    fn debug_compare(&self, other: &Self) -> String;
}

impl<T: Debug + PartialOrd> ComparableDebug for T {
    fn debug_compare(&self, other: &Self) -> String {
        let cmp = if self < other {
            "less than"
        } else if self > other {
            "greater than"
        } else {
            "equal to"
        };
        format!("{:?} is {} {:?}", self, cmp, other)
    }
}

fn main() {
    // Usage: All Debug types automatically get debug_print() method.
    println!("=== DebugExt (for all Debug types) ===");
    let numbers = vec![1, 2, 3];
    numbers.debug_print();

    let tuple = (1, "hello", 3.14);
    println!("Debug string: {}", tuple.debug_string());

    42.debug_print();
    "hello".debug_print();

    println!("\n=== CloneExt (for all Clone types) ===");
    let original = "test";
    let clones = original.clone_n(3);
    println!("Cloned 3 times: {:?}", clones);

    let num = 42;
    let num_clones = num.clone_n(5);
    println!("Number clones: {:?}", num_clones);

    println!("\n=== NumericExt (for numeric types) ===");
    let a: i32 = -5;
    let b: i32 = 10;
    println!("{} is_pos: {}", a, a.is_pos());
    println!("{} is_neg: {}", a, a.is_neg());
    println!("difference({}, {}): {}", a, b, a.difference(&b));

    let x: f64 = 3.14;
    let y: f64 = 2.0;
    println!("\n{} is_pos: {}", x, x.is_pos());
    println!("difference({}, {}): {}", x, y, x.difference(&y));

    println!("\n=== ComparableDebug (requires Debug + PartialOrd) ===");
    println!("{}", 5.debug_compare(&10));
    println!("{}", "banana".debug_compare(&"apple"));
    println!("{}", 3.14.debug_compare(&3.14));
}
