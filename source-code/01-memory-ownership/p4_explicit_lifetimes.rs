// Pattern 4: When You Need Explicit Lifetimes

// Multiple input references - compiler can't guess which to use
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn use_longest() {
    let s1 = String::from("long string");
    let result;
    {
        let s2 = String::from("short");
        result = longest(&s1, &s2);
        println!("Longest: {}", result);  // Must use here, while s2 valid
    }
    // println!("{}", result);  // Error if uncommented: s2 dropped
}

// Different lifetimes for different relationships
fn first_or_default<'a, 'b>(first: &'a str, _default: &'b str) -> &'a str {
    if !first.is_empty() { first } else {
        // Can't return default - wrong lifetime!
        first
    }
}

fn main() {
    use_longest();

    let result = first_or_default("hello", "default");
    println!("Result: {}", result);

    println!("Explicit lifetimes example completed");
}
