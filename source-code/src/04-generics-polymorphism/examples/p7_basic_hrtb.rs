//! Pattern 7: Higher-Ranked Trait Bounds (HRTBs)
//! Example: Basic HRTB for Closure with References
//!
//! Run with: cargo run --example p7_basic_hrtb

// HRTB: for<'a> means "for all possible lifetimes 'a"
// The closure must work for ANY lifetime the function provides
fn call_with_ref<F>(f: F) -> usize
where
    F: for<'a> Fn(&'a str) -> usize,
{
    let local = String::from("hello");
    f(&local) // 'a is lifetime of local
}

// Another HRTB example with transformation
fn transform_local<F>(f: F) -> String
where
    F: for<'a> Fn(&'a str) -> String,
{
    let local = String::from("hello world");
    f(&local)
}

// HRTB with multiple parameters
fn call_with_two<F>(f: F) -> bool
where
    F: for<'a, 'b> Fn(&'a str, &'b str) -> bool,
{
    let s1 = String::from("hello");
    let s2 = String::from("world");
    f(&s1, &s2) // Different lifetimes
}

// Fn(&str) is SUGAR for for<'a> Fn(&'a str)
fn sugar_version<F>(f: F) -> usize
where
    F: Fn(&str) -> usize,
{
    let local = String::from("rust");
    f(&local)
}

fn main() {
    println!("=== Basic HRTB ===");
    // Usage: Closure must handle any lifetime the function provides.
    let len = call_with_ref(|s| s.len());
    println!("call_with_ref(|s| s.len()) = {}", len);

    let count = call_with_ref(|s| s.chars().count());
    println!("call_with_ref(|s| s.chars().count()) = {}", count);

    let words = call_with_ref(|s| s.split_whitespace().count());
    println!("call_with_ref(|s| s.split_whitespace().count()) = {}", words);

    println!("\n=== Transform with HRTB ===");
    let upper = transform_local(|s| s.to_uppercase());
    println!("transform_local(|s| s.to_uppercase()) = \"{}\"", upper);

    let reversed = transform_local(|s| s.chars().rev().collect());
    println!("transform_local(|s| s.chars().rev().collect()) = \"{}\"", reversed);

    println!("\n=== Multiple Lifetimes ===");
    let equal = call_with_two(|a, b| a == b);
    println!("call_with_two(|a, b| a == b) = {}", equal);

    let longer = call_with_two(|a, b| a.len() > b.len());
    println!("call_with_two(|a, b| a.len() > b.len()) = {}", longer);

    let combined_len = call_with_two(|a, b| a.len() + b.len() > 5);
    println!("call_with_two(|a, b| a.len() + b.len() > 5) = {}", combined_len);

    println!("\n=== Fn(&str) is Sugar for HRTB ===");
    let result = sugar_version(|s| s.len());
    println!("sugar_version(|s| s.len()) = {}", result);
    println!("Fn(&str) automatically desugars to for<'a> Fn(&'a str)");

    println!("\n=== Why HRTB Matters ===");
    println!("Without HRTB, you'd need to specify a lifetime:");
    println!("  fn call_with_ref<'x, F: Fn(&'x str)>(f: F)");
    println!("But then the caller chooses 'x, not the function!");
    println!();
    println!("HRTB says: \"The closure works for ANY lifetime\"");
    println!("This lets the function create local values and pass");
    println!("references to the closure with whatever lifetime it needs.");
}
