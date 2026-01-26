//! Pattern 1: Trait Inheritance and Bounds
//! Example: Super Traits
//!
//! Run with: cargo run --example p1_supertraits

// Supertrait relationship: Printable requires Debug
trait Printable: std::fmt::Debug {
    fn print(&self) {
        println!("{:?}", self);
    }
}

// Any type implementing Printable must also implement Debug
#[derive(Debug)]
struct Document {
    title: String,
    content: String,
}

impl Printable for Document {}

#[derive(Debug)]
struct Report {
    name: String,
    pages: u32,
}

impl Printable for Report {}

fn main() {
    // Usage: Implementing Printable gives you print() via Debug supertrait.
    let doc = Document {
        title: "Rust Guide".to_string(),
        content: "Learning Rust".to_string(),
    };
    doc.print();

    let report = Report {
        name: "Q4 Results".to_string(),
        pages: 42,
    };
    report.print();

    // The Debug supertrait is automatically available
    println!("\nUsing Debug directly: {:?}", doc);
}
