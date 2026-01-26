//! Pattern 1: Struct Design Patterns
//! Example: Tuple Structs
//!
//! Run with: cargo run --example p1_tuple_struct

// Coordinates where position matters more than names
struct Point3D(f64, f64, f64);

// Type-safe wrappers (newtype pattern)
struct Kilometers(f64);
struct Miles(f64);

impl Point3D {
    fn origin() -> Self {
        Point3D(0.0, 0.0, 0.0)
    }

    fn distance_from_origin(&self) -> f64 {
        (self.0.powi(2) + self.1.powi(2) + self.2.powi(2)).sqrt()
    }
}

impl Kilometers {
    fn to_miles(&self) -> Miles {
        Miles(self.0 * 0.621371)
    }
}

fn main() {
    // Usage: Access tuple struct fields by index.
    let point = Point3D(3.0, 4.0, 0.0);
    println!("Point: ({}, {}, {})", point.0, point.1, point.2);
    println!("Distance from origin: {}", point.distance_from_origin());

    let origin = Point3D::origin();
    println!("Origin: ({}, {}, {})", origin.0, origin.1, origin.2);

    // Type-safe conversion
    let km = Kilometers(100.0);
    let mi = km.to_miles();
    println!("\n{} km = {} miles", km.0, mi.0);

    // The newtype pattern prevents mixing types:
    // fn drive(distance: Kilometers) { ... }
    // drive(Miles(50.0)); // Won't compile - type mismatch!
}
