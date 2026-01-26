//! Pattern 7: Higher-Ranked Trait Bounds (HRTBs)
//! Example: HRTB vs Regular Lifetime Parameter
//!
//! Run with: cargo run --example p7_hrtb_vs_lifetime

// REGULAR LIFETIME: Caller chooses the lifetime
// The function accepts a reference with lifetime 'a chosen by caller
fn with_lifetime<'a, F>(s: &'a str, f: F) -> &'a str
where
    F: Fn(&'a str) -> &'a str,
{
    f(s)
}

// HRTB: Function chooses the lifetime
// The closure must work for ANY lifetime the function provides
fn with_hrtb<F>(f: F) -> String
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    let local = String::from("hello world");
    f(&local).to_string() // f must work with local's lifetime
}

// Demonstration: transforming external vs internal data
fn process_external<'a, F>(data: &'a str, f: F) -> &'a str
where
    F: Fn(&'a str) -> &'a str,
{
    // The lifetime comes from the parameter
    f(data)
}

fn process_internal<F>(f: F) -> String
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    // Creating data inside the function
    let data = format!("generated-{}", 42);
    f(&data).to_string()
}

// Example showing when you DON'T need HRTB
fn map_ref<'a, T, U, F>(value: &'a T, f: F) -> U
where
    F: Fn(&'a T) -> U,
{
    f(value)
}

// Example showing when you DO need HRTB
fn create_and_process<F>(f: F) -> String
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    let local = String::from("temporary data");
    f(&local).to_string()
}

fn main() {
    println!("=== Regular Lifetime (Caller Chooses) ===");
    let external = "external string";
    // The lifetime 'a is tied to `external`
    let result = with_lifetime(external, |x| x);
    println!("with_lifetime(\"external string\", |x| x) = \"{}\"", result);

    // Can return a reference because lifetime is known
    let trimmed = with_lifetime("  padded  ", |x| x.trim());
    println!("with_lifetime(\"  padded  \", |x| x.trim()) = \"{}\"", trimmed);

    println!("\n=== HRTB (Function Chooses) ===");
    // The function creates internal data; closure handles any lifetime
    let result = with_hrtb(|x| x);
    println!("with_hrtb(|x| x) = \"{}\"", result);

    let result = with_hrtb(|x| x.trim());
    println!("with_hrtb(|x| x.trim()) = \"{}\"", result);

    println!("\n=== Processing External vs Internal Data ===");
    let data = "hello rust";
    let external_result = process_external(data, |s| &s[0..5]);
    println!("process_external(\"hello rust\", slice) = \"{}\"", external_result);

    let internal_result = process_internal(|s| s);
    println!("process_internal(|s| s) = \"{}\"", internal_result);

    println!("\n=== When NOT to Use HRTB ===");
    let value = 42;
    let doubled = map_ref(&value, |x| x * 2);
    println!("map_ref(&42, |x| x * 2) = {}", doubled);
    println!("^ Lifetime comes from parameter, no HRTB needed");

    println!("\n=== When to Use HRTB ===");
    let result = create_and_process(|s| s);
    println!("create_and_process(|s| s) = \"{}\"", result);
    println!("^ Function creates internal data, HRTB required");

    println!("\n=== Key Difference ===");
    println!("Regular lifetime <'a, F: Fn(&'a T)>:");
    println!("  - Caller provides data with lifetime 'a");
    println!("  - Function accepts that lifetime");
    println!();
    println!("HRTB <F: for<'a> Fn(&'a T)>:");
    println!("  - Function creates data internally");
    println!("  - Closure must work with whatever lifetime is given");
}
