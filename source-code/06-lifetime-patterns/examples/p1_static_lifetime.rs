//! Pattern 1: Named Lifetimes and Elision
//! Example: The 'static Lifetime
//!
//! Run with: cargo run --example p1_static_lifetime

// String literals have 'static lifetime - they're in the program binary.
const GREETING: &'static str = "Hello, World!";

// Static data with const
const APP_NAME: &'static str = "Lifetime Demo";
const VERSION: &'static str = "1.0.0";

// Function returning a 'static reference
fn get_greeting() -> &'static str {
    "Welcome to Rust!" // String literal is 'static
}

// Function that requires 'static bound
fn print_static(s: &'static str) {
    println!("Static string: {}", s);
}

// Generic function with 'static bound
fn store_static<T: 'static>(value: T) -> T {
    // T: 'static means T contains no non-static references
    value
}

// Struct that can only hold 'static references
struct StaticHolder {
    data: &'static str,
}

impl StaticHolder {
    fn new(data: &'static str) -> Self {
        StaticHolder { data }
    }

    fn get(&self) -> &'static str {
        self.data
    }
}

fn main() {
    println!("=== The 'static Lifetime ===");
    // Usage: 'static references are valid for the entire program duration.
    let s: &'static str = "I have a static lifetime.";
    println!("{}", s);
    println!("{} and {}", GREETING, APP_NAME);

    println!("\n=== String Literals are 'static ===");
    let greeting = get_greeting();
    println!("{}", greeting);
    // String literals live in the program binary, so they're always valid.

    println!("\n=== Functions Requiring 'static ===");
    print_static("This is a string literal"); // OK: literal is 'static
    print_static(GREETING); // OK: const is 'static

    // This would NOT work:
    // let owned = String::from("owned string");
    // print_static(&owned); // ERROR: &owned is not 'static

    println!("\n=== 'static Bound on Generics ===");
    // T: 'static means T owns its data (no borrowed references)
    let num = store_static(42); // i32 is 'static (no references)
    println!("Stored number: {}", num);

    let text = store_static(String::from("owned")); // String is 'static
    println!("Stored string: {}", text);

    let vec = store_static(vec![1, 2, 3]); // Vec<i32> is 'static
    println!("Stored vec: {:?}", vec);

    println!("\n=== StaticHolder Struct ===");
    let holder = StaticHolder::new("static data");
    println!("Holder contains: {}", holder.get());

    // This would NOT work:
    // let owned = String::from("owned");
    // let holder = StaticHolder::new(&owned); // ERROR: &owned not 'static

    println!("\n=== When to Use 'static ===");
    println!("- String literals and constants");
    println!("- Data that truly lives for the program's duration");
    println!("- Thread spawning (requires Send + 'static)");
    println!("- Lazy static initialization");

    println!("\n=== Common Misconceptions ===");
    println!("'static does NOT mean 'allocated at compile time'");
    println!("'static means 'can live for entire program duration'");
    println!("Owned types like String, Vec<T> satisfy T: 'static");
    println!("because they contain no borrowed references.");

    println!("\n=== 'static vs Owned Data ===");
    // These are different concepts:
    let literal: &'static str = "literal"; // Reference to static data
    let owned: String = String::from("owned"); // Owned data (satisfies 'static bound)

    println!("Literal (reference): {}", literal);
    println!("Owned (value): {}", owned);

    // Both can be used where 'static is required, but differently:
    // - literal IS a &'static str
    // - owned SATISFIES T: 'static bound (no references inside)
}
