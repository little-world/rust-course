//! Pattern 3: Higher-Ranked Trait Bounds (HRTBs)
//! Example: for<'a> Fn(&'a str) for Lifetime-Polymorphic Closures
//!
//! Run with: cargo run --example p3_hrtb

// HRTB `for<'a> Fn(&'a str)` ensures `f` works for ANY lifetime.
// The closure must handle references with any lifetime the caller provides.
fn call_on_hello<F>(f: F)
where
    F: for<'a> Fn(&'a str),
{
    let s = String::from("hello");
    f(&s); // Closure called with reference local to this function.
}

// Without HRTB, you'd need to specify a concrete lifetime.
// This is less flexible - the lifetime is fixed.
fn call_with_lifetime<'a, F>(s: &'a str, f: F)
where
    F: Fn(&'a str),
{
    f(s);
}

// HRTB with return value
fn transform_strings<F>(strings: &[String], f: F) -> Vec<String>
where
    F: for<'a> Fn(&'a str) -> String,
{
    strings.iter().map(|s| f(s)).collect()
}

// Trait with a higher-ranked method
trait Processor {
    // This method must work for any input lifetime 'a.
    fn process<'a>(&self, data: &'a str) -> &'a str;
}

struct Trimmer;

impl Processor for Trimmer {
    fn process<'a>(&self, data: &'a str) -> &'a str {
        data.trim()
    }
}

struct Uppercaser;

// Different implementation - returns owned data
impl Processor for Uppercaser {
    fn process<'a>(&self, data: &'a str) -> &'a str {
        // Note: This just returns the original since we can't return owned String as &str
        // In real code, you'd use a different signature for transformations
        data
    }
}

// HRTB in a trait bound
trait StringCallback {
    fn call<F>(&self, f: F)
    where
        F: for<'a> Fn(&'a str);
}

struct StringHolder {
    value: String,
}

impl StringCallback for StringHolder {
    fn call<F>(&self, f: F)
    where
        F: for<'a> Fn(&'a str),
    {
        f(&self.value);
    }
}

// Storing closures that work with any lifetime
struct CallbackStorage<F>
where
    F: for<'a> Fn(&'a str) -> usize,
{
    callback: F,
}

impl<F> CallbackStorage<F>
where
    F: for<'a> Fn(&'a str) -> usize,
{
    fn new(callback: F) -> Self {
        CallbackStorage { callback }
    }

    fn invoke(&self, s: &str) -> usize {
        (self.callback)(s)
    }
}

fn main() {
    println!("=== Basic HRTB Example ===");
    // Usage: for<'a> means closure handles any lifetime the function provides.
    let print_it = |s: &str| println!("Received: {}", s);
    call_on_hello(print_it);

    // The closure works with a reference that only lives inside call_on_hello.
    // Without for<'a>, this wouldn't compile because the lifetime would be ambiguous.

    println!("\n=== HRTB with Transform ===");
    let strings = vec![
        String::from("  hello  "),
        String::from("  world  "),
    ];
    let trimmed = transform_strings(&strings, |s| s.trim().to_string());
    println!("Trimmed: {:?}", trimmed);

    let uppercased = transform_strings(&strings, |s| s.to_uppercase());
    println!("Uppercased: {:?}", uppercased);

    println!("\n=== Processor Trait ===");
    // Usage: Method is generic over 'a; works with any input lifetime.
    let trimmer = Trimmer;
    let input = "  hello world  ";
    let result = trimmer.process(input);
    println!("Trimmed: '{}'", result);

    println!("\n=== StringCallback with HRTB ===");
    let holder = StringHolder {
        value: String::from("stored value"),
    };
    holder.call(|s| println!("Callback received: {}", s));

    println!("\n=== Callback Storage ===");
    let storage = CallbackStorage::new(|s: &str| s.len());
    println!("Length of 'hello': {}", storage.invoke("hello"));
    println!("Length of 'hello world': {}", storage.invoke("hello world"));

    println!("\n=== Why HRTBs Matter ===");
    println!("Without HRTBs:");
    println!("  - Closures would need a specific lifetime parameter");
    println!("  - Can't pass closures that work with local references");
    println!("  - Iterator adapters like map/filter wouldn't work with references");
    println!("\nWith HRTBs (for<'a>):");
    println!("  - Closure works with ANY lifetime");
    println!("  - Enables flexible, composable APIs");
    println!("  - Powers iterator adapters and callbacks");

    println!("\n=== Comparison: With vs Without HRTB ===");
    // Fixed lifetime - less flexible
    let external = String::from("external");
    call_with_lifetime(&external, |s| println!("Fixed: {}", s));

    // HRTB - works with any lifetime
    call_on_hello(|s| println!("Any: {}", s));

    println!("\n=== Common HRTB Patterns ===");
    println!("for<'a> Fn(&'a T) -> U     // Transform borrowed data");
    println!("for<'a> Fn(&'a T)          // Process borrowed data");
    println!("for<'a> FnMut(&'a mut T)   // Mutate borrowed data");
}
