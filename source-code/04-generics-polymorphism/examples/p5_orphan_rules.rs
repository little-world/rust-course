//! Pattern 5: Blanket Implementations
//! Example: Orphan Rules for Blanket Impls
//!
//! Run with: cargo run --example p5_orphan_rules

use std::fmt::{Debug, Display};

// ========================================
// ALLOWED: Your trait with blanket impl
// ========================================
trait MyTrait {
    fn my_method(&self);
}

// Blanket impl: your trait on any Debug type
impl<T: Debug> MyTrait for T {
    fn my_method(&self) {
        println!("MyTrait::my_method: {:?}", self);
    }
}

// ========================================
// ALLOWED: Foreign trait on your type
// ========================================
struct MyType {
    value: i32,
}

impl Display for MyType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "MyType({})", self.value)
    }
}

impl Debug for MyType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "MyType {{ value: {} }}", self.value)
    }
}

// ========================================
// Another example: Process trait
// ========================================
trait Process {
    fn process(&self) -> String;
}

impl Process for i32 {
    fn process(&self) -> String {
        format!("processing i32: {}", self)
    }
}

// Blanket impl for references
impl<T: Process> Process for &T {
    fn process(&self) -> String {
        (*self).process()
    }
}

// Blanket impl for Box
impl<T: Process> Process for Box<T> {
    fn process(&self) -> String {
        (**self).process()
    }
}

// ========================================
// NOT ALLOWED (would cause compiler error):
// ========================================
// impl Display for Vec<i32> { ... }
// ^ Foreign trait (Display) on foreign type (Vec<i32>)

fn main() {
    println!("=== Your Trait on Foreign Types (ALLOWED) ===");
    // Usage: Your trait via blanket impl works on any Debug type
    42.my_method();
    "hello".my_method();
    vec![1, 2, 3].my_method();

    println!("\n=== Foreign Trait on Your Type (ALLOWED) ===");
    let mt = MyType { value: 42 };
    // Display (foreign) on MyType (yours) - allowed
    println!("Display: {}", mt);
    println!("Debug: {:?}", mt);

    println!("\n=== Process with Reference/Box Blanket Impls ===");
    let num = 42;
    println!("{}", num.process()); // Direct call
    println!("{}", (&num).process()); // Through reference (blanket impl)
    println!("{}", Box::new(42).process()); // Through Box (blanket impl)

    println!("\n=== Orphan Rules Summary ===");
    println!("✓ ALLOWED:");
    println!("  - Your trait + blanket impl on foreign types");
    println!("    impl<T: Debug> MyTrait for T {{ ... }}");
    println!("  - Foreign trait on your type");
    println!("    impl Display for MyType {{ ... }}");
    println!("  - Your trait on foreign type");
    println!("    impl MyTrait for Vec<i32> {{ ... }}");
    println!();
    println!("✗ NOT ALLOWED:");
    println!("  - Foreign trait on foreign type");
    println!("    impl Display for Vec<i32> {{ ... }}");
    println!();
    println!("Why? To prevent conflicting implementations across crates.");
    println!("If two crates could both impl Display for Vec<i32>,");
    println!("which implementation should be used?");

    println!("\n=== Coherence ===");
    println!("Rust enforces coherence: one impl per type-trait combination globally.");
    println!("Blanket impls must be in trait's crate or type's crate.");
}
