//! Pattern 1: Derive Macro with Field Access
//!
//! Shows how to iterate over struct fields to generate field-aware implementations.
//! Handles named fields, tuple structs, and unit structs differently.

use my_macros::Describe;

trait Describe {
    fn describe(&self) -> String;
}

#[derive(Describe)]
struct Person {
    name: String,
    age: u32,
    email: String,
}

#[derive(Describe)]
struct Point(i32, i32);

#[derive(Describe)]
struct Unit;

fn main() {
    println!("=== Derive Macro with Field Access Demo ===\n");

    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    };
    println!("Named struct: {}", person.describe());

    let point = Point(10, 20);
    println!("Tuple struct: {}", point.describe());

    let unit = Unit;
    println!("Unit struct: {}", unit.describe());
}
