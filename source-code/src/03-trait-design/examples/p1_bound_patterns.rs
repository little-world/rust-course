//! Pattern 1: Trait Inheritance and Bounds
//! Example: Trait Bound Patterns
//!
//! Run with: cargo run --example p1_bound_patterns

// Builder pattern with trait bounds
struct Query<T> {
    data: T,
}

impl<T> Query<T> {
    fn new(data: T) -> Self {
        Query { data }
    }

    fn get(&self) -> &T {
        &self.data
    }
}

impl<T: Clone> Query<T> {
    // Only available if T is Clone
    fn duplicate(&self) -> Self {
        Query {
            data: self.data.clone(),
        }
    }
}

impl<T: std::fmt::Debug> Query<T> {
    // Only available if T is Debug
    fn debug_print(&self) {
        println!("Query data: {:?}", self.data);
    }
}

impl<T: Default> Query<T> {
    // Only available if T is Default
    fn reset(&mut self) {
        self.data = T::default();
    }
}

// Higher-rank trait bounds (for all lifetimes)
fn process_with_lifetime<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    let owned = String::from("hello world");
    let result = f(&owned);
    println!("Result: {}", result);
}

fn identity(s: &str) -> &str {
    s
}

fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or(s)
}

fn main() {
    // Usage: Methods appear based on type parameter's capabilities.
    println!("=== Query with Vec<i32> ===");
    let q = Query::new(vec![1, 2, 3]);
    q.debug_print(); // Works: Vec<i32> is Debug
    let q_dup = q.duplicate(); // Works: Vec<i32> is Clone
    q_dup.debug_print();

    println!("\n=== Query with String ===");
    let mut sq = Query::new(String::from("hello"));
    sq.debug_print();
    sq.reset(); // Works: String is Default
    sq.debug_print(); // Now empty

    println!("\n=== Higher-rank trait bounds ===");
    process_with_lifetime(identity);
    process_with_lifetime(first_word);

    // Using closures
    process_with_lifetime(|s| s);
    process_with_lifetime(|s| {
        if s.len() > 5 {
            &s[..5]
        } else {
            s
        }
    });
}
