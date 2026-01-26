//! Pattern 5: Combining syn and quote (GetterSetter)
//!
//! Showcases the standard syn+quote workflow: parse with syn, transform data,
//! generate with quote. Creates getters and setters for all struct fields.

use my_macros::GetterSetter;

#[derive(Debug, GetterSetter)]
struct Person {
    name: String,
    age: u32,
    email: String,
}

#[derive(Debug, GetterSetter)]
struct Rectangle {
    width: f64,
    height: f64,
}

fn main() {
    println!("=== GetterSetter Derive Demo ===\n");

    let mut person = Person {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    };

    println!("Initial person:");
    println!("  name: {}", person.name());
    println!("  age: {}", person.age());
    println!("  email: {}", person.email());

    // Use setters
    person.set_name("Bob".to_string());
    person.set_age(25);
    person.set_email("bob@example.com".to_string());

    println!("\nAfter setters:");
    println!("  name: {}", person.name());
    println!("  age: {}", person.age());
    println!("  email: {}", person.email());

    // Rectangle example
    let mut rect = Rectangle {
        width: 10.0,
        height: 5.0,
    };

    println!("\nRectangle:");
    println!("  width: {}", rect.width());
    println!("  height: {}", rect.height());
    println!("  area: {}", rect.width() * rect.height());

    rect.set_width(20.0);
    println!("\nAfter set_width(20.0):");
    println!("  new area: {}", rect.width() * rect.height());
}
