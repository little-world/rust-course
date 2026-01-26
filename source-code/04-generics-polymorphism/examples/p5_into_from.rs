//! Pattern 5: Blanket Implementations
//! Example: Into from From (Std Library Pattern)
//!
//! Run with: cargo run --example p5_into_from

// Custom Into/From traits to demonstrate the pattern
trait MyFrom<T> {
    fn my_from(value: T) -> Self;
}

trait MyInto<T> {
    fn my_into(self) -> T;
}

// Blanket impl: If U can be created from T, then T can be converted into U
impl<T, U: MyFrom<T>> MyInto<U> for T {
    fn my_into(self) -> U {
        U::my_from(self)
    }
}

// Custom types
#[derive(Debug)]
struct Meters(f64);

#[derive(Debug)]
struct Feet(f64);

#[derive(Debug)]
struct Inches(f64);

// Implement MyFrom for conversions
impl MyFrom<f64> for Meters {
    fn my_from(value: f64) -> Self {
        Meters(value)
    }
}

impl MyFrom<Feet> for Meters {
    fn my_from(feet: Feet) -> Self {
        Meters(feet.0 * 0.3048)
    }
}

impl MyFrom<Inches> for Meters {
    fn my_from(inches: Inches) -> Self {
        Meters(inches.0 * 0.0254)
    }
}

impl MyFrom<Meters> for Feet {
    fn my_from(meters: Meters) -> Self {
        Feet(meters.0 / 0.3048)
    }
}

// Example with strings
#[derive(Debug)]
struct UserId(String);

impl MyFrom<&str> for UserId {
    fn my_from(s: &str) -> Self {
        UserId(s.to_string())
    }
}

impl MyFrom<i32> for UserId {
    fn my_from(id: i32) -> Self {
        UserId(format!("user_{}", id))
    }
}

// Generic function using MyInto
fn process_length<T: MyInto<Meters>>(value: T) -> f64 {
    let meters: Meters = value.my_into();
    meters.0
}

fn main() {
    println!("=== Basic MyFrom Usage ===");
    let m1 = Meters::my_from(5.0);
    println!("Meters::my_from(5.0) = {:?}", m1);

    let feet = Feet(10.0);
    let m2 = Meters::my_from(feet);
    println!("Meters::my_from(Feet(10.0)) = {:?}", m2);

    println!("\n=== MyInto via Blanket Impl ===");
    // Usage: Implement From, get Into free via blanket impl.
    let m3: Meters = 5.0.my_into();
    println!("5.0.my_into::<Meters>() = {:?}", m3);

    let m4: Meters = Inches(100.0).my_into();
    println!("Inches(100.0).my_into::<Meters>() = {:?}", m4);

    let f: Feet = Meters(1.0).my_into();
    println!("Meters(1.0).my_into::<Feet>() = {:?}", f);

    println!("\n=== UserId Conversions ===");
    let id1: UserId = "alice".my_into();
    let id2: UserId = 42.my_into();
    println!("\"alice\".my_into::<UserId>() = {:?}", id1);
    println!("42.my_into::<UserId>() = {:?}", id2);

    println!("\n=== Generic Function with MyInto ===");
    let len1 = process_length(10.0_f64);
    println!("process_length(10.0) = {} meters", len1);

    let len2 = process_length(Feet(100.0));
    println!("process_length(Feet(100.0)) = {:.2} meters", len2);

    let len3 = process_length(Inches(39.37));
    println!("process_length(Inches(39.37)) = {:.2} meters", len3);

    println!("\n=== How the Blanket Impl Works ===");
    println!("impl<T, U: MyFrom<T>> MyInto<U> for T {{");
    println!("    fn my_into(self) -> U {{ U::my_from(self) }}");
    println!("}}");
    println!("\nThis means:");
    println!("  - If you impl MyFrom<T> for U");
    println!("  - You automatically get MyInto<U> for T");
    println!("  - Just implement From, and Into comes free!");
}
