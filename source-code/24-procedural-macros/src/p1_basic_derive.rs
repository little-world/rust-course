//! Pattern 1: Basic Derive Macro
//!
//! Demonstrates the fundamental structure of a derive macro.
//! The HelloWorld derive automatically implements the HelloWorld trait.

use my_macros::HelloWorld;

trait HelloWorld {
    fn hello_world();
}

#[derive(HelloWorld)]
struct MyStruct;

#[derive(HelloWorld)]
struct AnotherStruct;

#[derive(HelloWorld)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    println!("=== Basic Derive Macro Demo ===\n");

    MyStruct::hello_world();
    AnotherStruct::hello_world();
    Point::hello_world();

    println!("\nAll structs automatically got HelloWorld implementation!");
}
