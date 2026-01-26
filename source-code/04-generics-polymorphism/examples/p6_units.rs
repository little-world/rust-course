//! Pattern 6: Phantom Types and Type-Level State
//! Example: Units of Measure
//!
//! Run with: cargo run --example p6_units

use std::marker::PhantomData;
use std::ops::{Add, Mul, Sub};

// Unit marker types
struct Meters;
struct Feet;
struct Seconds;
struct MetersPerSecond;

// Quantity with phantom unit type
// Note: PhantomData<Unit> is Copy, so Quantity is Copy if T is Copy
#[derive(Debug, Clone, Copy)]
struct Quantity<T, Unit> {
    value: T,
    _unit: PhantomData<Unit>,
}

impl<T, Unit> Quantity<T, Unit> {
    fn new(value: T) -> Self {
        Quantity {
            value,
            _unit: PhantomData,
        }
    }

    fn value(&self) -> &T {
        &self.value
    }
}

// Addition only works with same units
impl<T: Add<Output = T>, Unit> Add for Quantity<T, Unit> {
    type Output = Quantity<T, Unit>;

    fn add(self, other: Self) -> Self::Output {
        Quantity::new(self.value + other.value)
    }
}

// Subtraction only works with same units
impl<T: Sub<Output = T>, Unit> Sub for Quantity<T, Unit> {
    type Output = Quantity<T, Unit>;

    fn sub(self, other: Self) -> Self::Output {
        Quantity::new(self.value - other.value)
    }
}

// Scalar multiplication
impl<T: Mul<Output = T> + Copy, Unit> Quantity<T, Unit> {
    fn scale(self, factor: T) -> Self {
        Quantity::new(self.value * factor)
    }
}

// Unit conversion (explicit, type-safe)
impl Quantity<f64, Meters> {
    fn to_feet(self) -> Quantity<f64, Feet> {
        Quantity::new(self.value * 3.28084)
    }
}

impl Quantity<f64, Feet> {
    fn to_meters(self) -> Quantity<f64, Meters> {
        Quantity::new(self.value * 0.3048)
    }
}

// Derived units: distance / time = velocity
impl Quantity<f64, Meters> {
    fn divide_by_time(self, time: Quantity<f64, Seconds>) -> Quantity<f64, MetersPerSecond> {
        Quantity::new(self.value / time.value)
    }
}

// Type aliases for convenience
type Distance = Quantity<f64, Meters>;
type DistanceFt = Quantity<f64, Feet>;
type Time = Quantity<f64, Seconds>;
type Velocity = Quantity<f64, MetersPerSecond>;

fn main() {
    println!("=== Basic Quantity Operations ===");
    // Usage: Phantom unit type prevents adding incompatible quantities.
    // Create fresh quantities for each operation
    let d_sum = Quantity::<f64, Meters>::new(10.0) + Quantity::new(5.0);
    let d_diff = Quantity::<f64, Meters>::new(10.0) - Quantity::new(5.0);
    println!("10m + 5m = {}m", d_sum.value());
    println!("10m - 5m = {}m", d_diff.value());

    println!("\n=== Scalar Multiplication ===");
    let d3 = Quantity::<f64, Meters>::new(10.0).scale(3.0);
    println!("10m * 3 = {}m", d3.value());

    println!("\n=== Unit Conversion ===");
    let meters: Distance = Quantity::new(100.0);
    let feet: DistanceFt = meters.to_feet();
    println!("100m = {:.2}ft", feet.value());

    let feet2: DistanceFt = Quantity::new(328.084);
    let meters2: Distance = feet2.to_meters();
    println!("328.084ft = {:.2}m", meters2.value());

    println!("\n=== Derived Units (Velocity) ===");
    let distance: Distance = Quantity::new(100.0);
    let time: Time = Quantity::new(10.0);
    let velocity: Velocity = distance.divide_by_time(time);
    println!("100m / 10s = {}m/s", velocity.value());

    println!("\n=== Type Safety Demonstration ===");
    println!("These operations are allowed:");
    println!("  meters + meters ✓");
    println!("  feet + feet ✓");
    println!("  meters.to_feet() ✓");
    println!();
    println!("These would NOT compile:");
    println!("  meters + feet ✗ (type mismatch)");
    println!("  meters + seconds ✗ (type mismatch)");
    println!("  velocity + distance ✗ (type mismatch)");

    // This would NOT compile:
    // let invalid = d1 + feet; // ERROR: mismatched types

    println!("\n=== Same Units, Different Values ===");
    let a: Quantity<f64, Meters> = Quantity::new(3.0);
    let b: Quantity<f64, Meters> = Quantity::new(4.0);
    // Pythagorean calculation (safe because same unit)
    let hypotenuse = (a.value().powi(2) + b.value().powi(2)).sqrt();
    println!("√(3² + 4²) = {}m", hypotenuse);
}
