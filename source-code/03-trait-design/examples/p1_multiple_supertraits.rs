//! Pattern 1: Trait Inheritance and Bounds
//! Example: Multiple Supertraits
//!
//! Run with: cargo run --example p1_multiple_supertraits

use std::fmt::{Debug, Display};

// Requires both Debug and Display
trait Loggable: Debug + Display {
    fn log(&self) {
        println!("[DEBUG] {:?}", self);
        println!("[INFO] {}", self);
    }
}

#[derive(Debug)]
struct User {
    name: String,
    id: u32,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "User {} (ID: {})", self.name, self.id)
    }
}

impl Loggable for User {}

fn use_loggable<T: Loggable>(item: &T) {
    item.log();
}

fn main() {
    // Usage: Loggable requires both Debug and Display implementations.
    let user = User {
        name: "Alice".to_string(),
        id: 42,
    };

    println!("=== Direct log() call ===");
    user.log(); // Prints both [DEBUG] and [INFO] lines

    println!("\n=== Via generic function ===");
    use_loggable(&user);

    // Both Debug and Display work independently too
    println!("\n=== Using traits directly ===");
    println!("Debug: {:?}", user);
    println!("Display: {}", user);
}
