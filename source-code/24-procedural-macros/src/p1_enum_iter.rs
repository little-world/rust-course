//! Pattern 1: Derive Macro for Enums
//!
//! Generates iteration helpers for enum variants.
//! Creates variants() and variant_names() methods for reflection-like behavior.

use my_macros::EnumIter;

#[derive(Debug, Copy, Clone, EnumIter)]
enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Debug, Copy, Clone, EnumIter)]
enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Copy, Clone, EnumIter)]
enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

fn main() {
    println!("=== Enum Iterator Derive Demo ===\n");

    println!("Colors:");
    for color in Color::variants() {
        println!("  {:?}", color);
    }
    println!("Color names: {:?}", Color::variant_names());

    println!("\nDirections:");
    for dir in Direction::variants() {
        println!("  {:?}", dir);
    }
    println!("Direction names: {:?}", Direction::variant_names());

    println!("\nLog Levels:");
    for (level, name) in LogLevel::variants().iter().zip(LogLevel::variant_names()) {
        println!("  {} -> {:?}", name, level);
    }
}
