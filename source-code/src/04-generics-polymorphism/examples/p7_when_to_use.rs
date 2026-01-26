//! Pattern 7: Higher-Ranked Trait Bounds (HRTBs)
//! Example: When HRTB Is and Isn't Needed
//!
//! Run with: cargo run --example p7_when_to_use

// ========================================
// DON'T need HRTB: Lifetime from parameter
// ========================================

fn map_ref<'a, T, U, F>(value: &'a T, f: F) -> U
where
    F: Fn(&'a T) -> U, // 'a from parameter
{
    f(value)
}

fn transform_slice<'a, T, F>(slice: &'a [T], f: F) -> Vec<T>
where
    T: Clone,
    F: Fn(&'a T) -> T, // 'a from parameter
{
    slice.iter().map(|x| f(x)).collect()
}

// ========================================
// DO need HRTB: Lifetime created inside
// ========================================

fn create_and_process<F>(f: F) -> String
where
    F: for<'a> Fn(&'a str) -> &'a str, // HRTB needed
{
    let local = String::from("temporary");
    f(&local).to_string() // local's lifetime unknown to caller
}

fn process_with_context<F>(context: &str, f: F) -> String
where
    F: for<'a> Fn(&'a str, &'a str) -> String, // HRTB needed
{
    let generated = format!("generated from {}", context);
    f(context, &generated) // generated's lifetime unknown
}

// ========================================
// Closure returning owned: No HRTB needed
// ========================================

fn apply_and_collect<T, U, F>(items: Vec<T>, f: F) -> Vec<U>
where
    F: Fn(T) -> U, // Takes ownership, returns owned
{
    items.into_iter().map(f).collect()
}

fn transform_to_string<T, F>(value: T, f: F) -> String
where
    F: Fn(T) -> String, // Returns owned String
{
    f(value)
}

// ========================================
// Summary function demonstrations
// ========================================

fn main() {
    println!("=== When NOT to Use HRTB ===\n");

    // Lifetime comes from parameter
    let v = 42;
    let doubled = map_ref(&v, |x| x * 2);
    println!("map_ref(&42, |x| x * 2) = {}", doubled);

    let nums = vec![1, 2, 3];
    let incremented = transform_slice(&nums, |x| x + 10);
    println!("transform_slice(&[1,2,3], |x| x+10) = {:?}", incremented);

    // Returns owned value, no references involved
    let strings = apply_and_collect(vec![1, 2, 3], |x| x.to_string());
    println!("apply_and_collect([1,2,3], to_string) = {:?}", strings);

    let result = transform_to_string(42, |x| format!("Number: {}", x));
    println!("transform_to_string(42, format) = \"{}\"", result);

    println!("\n=== When to Use HRTB ===\n");

    // Function creates local value
    let processed = create_and_process(|s| s);
    println!("create_and_process(|s| s) = \"{}\"", processed);

    let processed = create_and_process(|s| s.trim());
    println!("create_and_process(trim) = \"{}\"", processed);

    // Function generates internal data
    let result = process_with_context("input", |ctx, gen| {
        format!("context: {}, generated: {}", ctx, gen)
    });
    println!("process_with_context = \"{}\"", result);

    println!("\n=== Decision Guide ===\n");

    println!("Use REGULAR lifetime when:");
    println!("  ✓ Lifetime comes from function parameter");
    println!("  ✓ Caller provides the data");
    println!("  ✓ Closure returns owned value");
    println!("  ✓ No references involved");
    println!();
    println!("Use HRTB when:");
    println!("  ✓ Function creates values internally");
    println!("  ✓ Closure must work with unknown lifetime");
    println!("  ✓ Storing callbacks for later use");
    println!("  ✓ Parser combinators");
    println!();
    println!("Quick test: Does the function create data that the");
    println!("closure needs to reference? If yes, use HRTB.");

    println!("\n=== Common Patterns ===\n");

    println!("No HRTB needed:");
    println!("  fn foo<'a, F: Fn(&'a T) -> U>()     // lifetime from param");
    println!("  fn bar<F: Fn(T) -> U>()             // owned values");
    println!();
    println!("HRTB needed:");
    println!("  fn foo<F: for<'a> Fn(&'a T)>()      // internal temporaries");
    println!("  type Cb = Box<dyn for<'a> Fn(&'a T)> // stored callbacks");
}
