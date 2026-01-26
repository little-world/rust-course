//! Pattern 6: Advanced Lifetime Patterns
//! Example: Closures with Lifetimes and Anonymous Lifetimes
//!
//! Run with: cargo run --example p6_advanced_lifetimes

use std::fmt::Debug;

// Lifetime bounds with closures
fn process_with_context<'a, F, T>(data: &'a str, context: &'a T, f: F) -> String
where
    F: Fn(&'a str, &'a T) -> String,
    T: Debug,
{
    f(data, context)
}

// Parser with lifetime elision in impl blocks
struct Parser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input, position: 0 }
    }

    // Elided: fn parse(&self) -> Option<&str>
    // Actual: fn parse(&'a self) -> Option<&'a str>
    fn parse(&self) -> Option<&str> {
        if self.position < self.input.len() {
            Some(&self.input[self.position..])
        } else {
            None
        }
    }

    // Multiple lifetimes when needed
    fn parse_with<'b>(&self, other: &'b str) -> (&'a str, &'b str) {
        (self.input, other)
    }

    // Anonymous lifetime: '_ = 'a in this context
    fn peek(&self) -> Option<&'_ str> {
        self.input.get(self.position..self.position + 1)
    }

    fn advance(&mut self, n: usize) {
        self.position = (self.position + n).min(self.input.len());
    }
}

// Anonymous lifetimes in function signatures
fn get_first<T>(vec: &Vec<T>) -> Option<&'_ T> {
    vec.first()
}

fn get_slice<T>(vec: &Vec<T>, start: usize, end: usize) -> Option<&'_ [T]> {
    vec.get(start..end)
}

// Struct with multiple lifetime parameters
struct MultiLifetime<'a, 'b> {
    first: &'a str,
    second: &'b str,
}

impl<'a, 'b> MultiLifetime<'a, 'b> {
    fn new(first: &'a str, second: &'b str) -> Self {
        MultiLifetime { first, second }
    }

    // Return tied to 'a
    fn get_first(&self) -> &'a str {
        self.first
    }

    // Return tied to 'b
    fn get_second(&self) -> &'b str {
        self.second
    }

    // Return tied to shorter of 'a and 'b
    fn get_shorter(&self) -> &str
    where
        'a: 'b, // If 'a outlives 'b, we can return 'b
    {
        if self.first.len() < self.second.len() {
            self.first
        } else {
            self.second
        }
    }
}

// Closures capturing references
fn demonstrate_closure_lifetimes() {
    println!("\n=== Closure Lifetime Capture ===");

    let data = String::from("captured data");

    // Closure captures &data
    let closure = |suffix: &str| format!("{} {}", data, suffix);

    println!("Closure result: {}", closure("here"));

    // The closure's lifetime is tied to `data`
    // It can't outlive `data`
}

// Higher-order function with lifetime constraints
fn apply_to_all<'a, F>(items: &'a [String], f: F) -> Vec<&'a str>
where
    F: Fn(&'a String) -> &'a str,
{
    items.iter().map(f).collect()
}

fn main() {
    println!("=== Lifetime Bounds with Closures ===");
    // Usage: Closure receives refs with same lifetime as function inputs.
    let ctx = "prefix";
    let result = process_with_context("data", &ctx, |d, c| format!("{}: {}", c, d));
    println!("Result: {}", result);

    println!("\n=== Parser with Lifetime Elision ===");
    // Usage: Elision ties method return lifetime to &self automatically.
    let mut parser = Parser::new("hello world");
    println!("Parse result: {:?}", parser.parse());
    println!("Peek result: {:?}", parser.peek());

    parser.advance(6);
    println!("After advance(6): {:?}", parser.parse());

    println!("\n=== Multiple Lifetimes in parse_with ===");
    let other = "another string";
    let (input, other_ref) = parser.parse_with(other);
    println!("Input: {}, Other: {}", input, other_ref);

    println!("\n=== Anonymous Lifetimes ('_) ===");
    // Usage: '_ placeholder lets compiler infer lifetime without naming it.
    let v = vec![1, 2, 3, 4, 5];
    let first = get_first(&v);
    println!("First element: {:?}", first);

    let slice = get_slice(&v, 1, 4);
    println!("Slice [1..4]: {:?}", slice);

    println!("\n=== Multiple Lifetime Parameters ===");
    let string1 = String::from("hello");
    let string2 = String::from("world!");

    let multi = MultiLifetime::new(&string1, &string2);
    println!("First: {}", multi.get_first());
    println!("Second: {}", multi.get_second());
    println!("Shorter: {}", multi.get_shorter());

    demonstrate_closure_lifetimes();

    println!("\n=== Higher-Order Functions with Lifetimes ===");
    let strings = vec![
        String::from("  hello  "),
        String::from("  world  "),
    ];

    let trimmed: Vec<&str> = apply_to_all(&strings, |s| s.trim());
    println!("Trimmed: {:?}", trimmed);

    println!("\n=== Lifetime Elision Summary ===");
    println!("Rule 1: Each elided input lifetime gets distinct parameter");
    println!("  fn foo(x: &i32) -> fn foo<'a>(x: &'a i32)");
    println!("\nRule 2: Single input lifetime -> all output lifetimes");
    println!("  fn foo(x: &str) -> &str => fn foo<'a>(x: &'a str) -> &'a str");
    println!("\nRule 3: &self lifetime -> all output lifetimes");
    println!("  fn get(&self) -> &T => fn get(&'a self) -> &'a T");

    println!("\n=== Anonymous Lifetime ('_) Usage ===");
    println!("- fn get_first<T>(v: &Vec<T>) -> Option<&'_ T>");
    println!("- impl Iterator for Foo<'_> {{ ... }}");
    println!("- let x: &'_ str = ...");
    println!("\nMeaning: 'infer this lifetime, I don't need to name it'");

    println!("\n=== Best Practices ===");
    println!("1. Let elision work when possible");
    println!("2. Use '_ for clarity without naming");
    println!("3. Name lifetimes when relationships matter");
    println!("4. Use multiple lifetime params for independent lifetimes");
    println!("5. Read compiler errors carefully - they're helpful!");
}
