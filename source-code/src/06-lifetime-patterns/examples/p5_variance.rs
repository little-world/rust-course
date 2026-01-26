//! Pattern 5: Variance and Subtyping
//! Example: Variance Categories and Lifetime Subtyping
//!
//! Run with: cargo run --example p5_variance

use std::cell::Cell;

// Covariant: &'a T is covariant over 'a
// Longer lifetime can substitute for shorter
fn demonstrate_covariance() {
    println!("=== Covariance (&'a T) ===");

    // Usage: Longer lifetime ('static) substitutes for shorter; subtyping in action.
    let long: &'static str = "hello";
    let short: &str = long; // OK: 'static is subtype of any shorter lifetime
    println!("long: {}, short: {}", long, short);

    // Function expecting &str accepts &'static str
    fn take_short(x: &str) {
        println!("Received: {}", x);
    }

    let s: &'static str = "static string";
    take_short(s); // OK: 'static works where shorter expected
}

// Invariant: &'a mut T is invariant over 'a
// No lifetime substitution allowed
fn demonstrate_invariance() {
    println!("\n=== Invariance (&'a mut T) ===");

    fn needs_exact_lifetime<'a>(_x: &'a mut i32, _y: &'a mut i32) {
        // Both must have exactly the same lifetime
    }

    let mut a = 1;
    let mut b = 2;
    needs_exact_lifetime(&mut a, &mut b);
    println!("a: {}, b: {}", a, b);

    // This demonstrates why invariance is necessary:
    // If mutable references were covariant, you could do this:
    // fn bad<'a, 'b>(x: &'a mut &'static str, y: &'b mut &'a str) {
    //     *x = *y; // Would allow assigning shorter lifetime to longer!
    // }
    println!("Invariance prevents unsound lifetime substitutions");
}

// Producer/Consumer variance example
struct Producer<T> {
    produce: fn() -> T, // Covariant over T
}

struct Consumer<T> {
    consume: fn(T), // Contravariant over T
}

fn demonstrate_function_variance() {
    println!("\n=== Function Variance ===");

    // Producer is covariant: can produce longer-lived where shorter expected
    let p: Producer<&'static str> = Producer {
        produce: || "hello",
    };
    let _p2: Producer<&str> = p; // OK: covariant
    println!("Producer<&'static str> usable as Producer<&str>");

    // Consumer is contravariant: can consume shorter-lived where longer expected
    let c: Consumer<&str> = Consumer { consume: |_s| {} };
    // Note: Can't directly demonstrate contravariant assignment in simple code
    // because Rust's type inference handles this automatically
    println!("Consumer uses contravariance for function arguments");

    // This is why fn(T) -> U is contravariant in T, covariant in U
    println!("fn(T) -> U: contravariant in T, covariant in U");
}

// Interior mutability and invariance
fn demonstrate_cell_invariance() {
    println!("\n=== Cell Invariance ===");

    let cell: Cell<i32> = Cell::new(42);
    cell.set(100);
    println!("Cell value: {}", cell.get());

    // Cell<T> is invariant over T
    // If it were covariant, this would be unsound:
    // fn bad() {
    //     let cell: Cell<&'static str> = Cell::new("hello");
    //     let short_cell: Cell<&str> = cell; // Hypothetically allowed
    //     let local = String::from("local");
    //     short_cell.set(&local); // Store short-lived reference
    //     // Now cell (Cell<&'static str>) contains a non-static reference!
    // }

    println!("Cell<T> is invariant to prevent unsound mutations");
}

// Variance in practice
fn variance_in_apis() {
    println!("\n=== Variance in API Design ===");

    // Good: covariant, flexible
    struct GoodReader<'a> {
        data: &'a [u8],
    }

    impl<'a> GoodReader<'a> {
        fn read(&self) -> &'a [u8] {
            self.data
        }
    }

    // Can use GoodReader<'static> where GoodReader<'a> expected
    let static_data: &'static [u8] = b"hello";
    let reader = GoodReader { data: static_data };
    let _bytes = reader.read();
    println!("GoodReader is covariant - flexible for callers");

    // Less flexible: invariant (mutable reference)
    struct MutReader<'a> {
        data: &'a mut [u8],
    }

    let mut buffer = [1u8, 2, 3];
    let _mut_reader = MutReader { data: &mut buffer };
    println!("MutReader is invariant - less flexible");
}

fn main() {
    println!("=== Variance and Subtyping in Rust ===\n");

    demonstrate_covariance();
    demonstrate_invariance();
    demonstrate_function_variance();
    demonstrate_cell_invariance();
    variance_in_apis();

    println!("\n=== Summary of Variance Rules ===");
    println!("Covariant (longer → shorter OK):");
    println!("  - &'a T");
    println!("  - *const T");
    println!("  - fn() -> T (return position)");
    println!("  - Vec<T>, Box<T>, Rc<T>, Arc<T>");

    println!("\nInvariant (no substitution):");
    println!("  - &'a mut T");
    println!("  - *mut T");
    println!("  - Cell<T>, RefCell<T>, UnsafeCell<T>");

    println!("\nContravariant (shorter → longer OK):");
    println!("  - fn(T) (argument position)");

    println!("\n=== Why Variance Matters ===");
    println!("- Determines which lifetime substitutions are safe");
    println!("- Covariance enables ergonomic APIs");
    println!("- Invariance prevents unsound mutations");
    println!("- Understanding variance helps debug lifetime errors");
}
