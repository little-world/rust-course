//! Pattern 2: Newtype and Wrapper Patterns
//! Example: Transparent Wrappers with Deref
//!
//! Run with: cargo run --example p2_deref_wrapper

use std::ops::Deref;

struct Validated<T> {
    value: T,
    validated_at: std::time::Instant,
}

impl<T> Validated<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            validated_at: std::time::Instant::now(),
        }
    }

    fn age(&self) -> std::time::Duration {
        self.validated_at.elapsed()
    }
}

impl<T> Deref for Validated<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

fn main() {
    // Usage: Deref lets wrapper use inner type's methods directly.
    let validated = Validated::new("hello".to_string());

    // String::len via Deref - no need to access .value
    assert_eq!(validated.len(), 5);
    println!("Length via Deref: {}", validated.len());

    // Other String methods work too
    println!("Uppercase: {}", validated.to_uppercase());
    println!("Contains 'ell': {}", validated.contains("ell"));

    // Wrapper-specific methods still available
    std::thread::sleep(std::time::Duration::from_millis(10));
    println!("Age of validated value: {:?}", validated.age());

    // Works with any type
    let validated_vec = Validated::new(vec![1, 2, 3, 4, 5]);
    println!("\nValidated vec length: {}", validated_vec.len());
    println!("First element: {:?}", validated_vec.first());

    // Can still access the wrapped value directly if needed
    println!("Direct access: {:?}", validated_vec.value);
}
