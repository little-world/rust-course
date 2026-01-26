//! Pattern 2: Lifetime Bounds
//! Example: T: 'a Bounds and Where Clauses
//!
//! Run with: cargo run --example p2_lifetime_bounds

use std::fmt::Debug;

// `T: 'a` means `T` must outlive `'a`.
// Any references inside T must live at least as long as 'a.
struct Wrapper<'a, T: 'a> {
    value: &'a T,
}

impl<'a, T: 'a> Wrapper<'a, T> {
    fn new(value: &'a T) -> Self {
        Wrapper { value }
    }

    fn get(&self) -> &T {
        self.value
    }
}

// Where clause for complex bounds - more readable.
fn process_and_debug<'a, T>(items: &'a [T])
where
    T: Debug + 'a, // T must implement Debug AND outlive 'a
{
    for item in items {
        println!("Item: {:?}", item);
    }
}

// Multiple lifetime bounds
fn compare_and_return<'a, 'b, T>(x: &'a T, y: &'b T) -> &'a T
where
    T: PartialOrd + 'a + 'b,
    'b: 'a, // 'b outlives 'a
{
    // We can only return &'a T, not &'b T
    x
}

// Struct with multiple lifetime parameters
struct Context<'a, 'b> {
    data: &'a str,
    metadata: &'b str,
}

impl<'a, 'b> Context<'a, 'b> {
    fn new(data: &'a str, metadata: &'b str) -> Self {
        Context { data, metadata }
    }

    fn data(&self) -> &'a str {
        self.data
    }

    fn metadata(&self) -> &'b str {
        self.metadata
    }
}

// Trait with lifetime parameter
trait Parser<'a> {
    fn parse(&self, input: &'a str) -> Option<&'a str>;
}

struct WordParser;

impl<'a> Parser<'a> for WordParser {
    fn parse(&self, input: &'a str) -> Option<&'a str> {
        input.split_whitespace().next()
    }
}

// Generic with lifetime bound in impl
struct Cache<'a, T> {
    items: Vec<&'a T>,
}

impl<'a, T: 'a> Cache<'a, T> {
    fn new() -> Self {
        Cache { items: Vec::new() }
    }

    fn add(&mut self, item: &'a T) {
        self.items.push(item);
    }

    fn get(&self, index: usize) -> Option<&&T> {
        self.items.get(index)
    }
}

fn main() {
    println!("=== Lifetime Bound on Generic Struct ===");
    // Usage: T: 'a ensures wrapped reference doesn't outlive the data.
    let num = 42;
    let wrapped = Wrapper::new(&num);
    println!("Wrapped value: {}", wrapped.get());

    let text = String::from("hello");
    let wrapped_text = Wrapper::new(&text);
    println!("Wrapped text: {}", wrapped_text.get());

    println!("\n=== Where Clauses for Complex Bounds ===");
    // Usage: where clause combines trait bound (Debug) with lifetime bound ('a).
    let numbers = [1, 2, 3, 4, 5];
    process_and_debug(&numbers);

    let strings = ["hello", "world"];
    process_and_debug(&strings);

    println!("\n=== Multiple Lifetime Parameters ===");
    let data = "important data";
    let metadata = "created: 2024";
    let ctx = Context::new(data, metadata);
    println!("Data: {}", ctx.data());
    println!("Metadata: {}", ctx.metadata());

    println!("\n=== Trait with Lifetime Parameter ===");
    let parser = WordParser;
    let input = "hello world from rust";
    if let Some(word) = parser.parse(input) {
        println!("First word: {}", word);
    }

    println!("\n=== Cache with Lifetime Bounds ===");
    let items = vec![1, 2, 3, 4, 5];
    let mut cache: Cache<i32> = Cache::new();
    for item in &items {
        cache.add(item);
    }
    println!("Cache item 2: {:?}", cache.get(2));

    println!("\n=== Lifetime Bound Comparison ===");
    let x = 10;
    let y = 20;
    let result = compare_and_return(&x, &y);
    println!("Result: {}", result);

    println!("\n=== Key Points ===");
    println!("- T: 'a means T contains no references shorter than 'a");
    println!("- 'b: 'a means 'b outlives 'a ('b is longer)");
    println!("- Where clauses improve readability for complex bounds");
    println!("- Multiple lifetime params for independent lifetimes");
}
