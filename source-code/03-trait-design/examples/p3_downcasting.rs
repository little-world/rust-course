//! Pattern 3: Trait Objects and Dynamic Dispatch
//! Example: Downcasting Trait Objects with Any
//!
//! Run with: cargo run --example p3_downcasting

use std::any::Any;

trait Shape: Any {
    fn area(&self) -> f64;
    fn name(&self) -> &'static str;

    // Provided method for downcasting
    fn as_any(&self) -> &dyn Any;
}

struct Circle {
    radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    fn name(&self) -> &'static str {
        "Circle"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }

    fn name(&self) -> &'static str {
        "Rectangle"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Helper functions for downcasting
fn try_as_circle(shape: &dyn Shape) -> Option<&Circle> {
    shape.as_any().downcast_ref::<Circle>()
}

fn try_as_rectangle(shape: &dyn Shape) -> Option<&Rectangle> {
    shape.as_any().downcast_ref::<Rectangle>()
}

fn process_shape(shape: &dyn Shape) {
    println!("Processing {}, area: {:.2}", shape.name(), shape.area());

    // Try to get concrete type information
    if let Some(circle) = try_as_circle(shape) {
        println!("  -> It's a circle with radius {}", circle.radius);
    } else if let Some(rect) = try_as_rectangle(shape) {
        println!("  -> It's a rectangle {}x{}", rect.width, rect.height);
    }
}

fn main() {
    // Usage: Downcast from trait object back to concrete type when needed.
    let circle = Circle { radius: 5.0 };
    let rectangle = Rectangle {
        width: 10.0,
        height: 20.0,
    };

    // Use as trait objects
    let shape1: &dyn Shape = &circle;
    let shape2: &dyn Shape = &rectangle;

    println!("=== Via Trait Object Interface ===");
    println!("{}: {:.2}", shape1.name(), shape1.area());
    println!("{}: {:.2}", shape2.name(), shape2.area());

    println!("\n=== Downcasting to Access Concrete Fields ===");
    if let Some(c) = try_as_circle(shape1) {
        println!("Circle radius: {}", c.radius);
    }

    if let Some(r) = try_as_rectangle(shape2) {
        println!("Rectangle dimensions: {}x{}", r.width, r.height);
    }

    println!("\n=== Processing Mixed Shapes ===");
    let shapes: Vec<&dyn Shape> = vec![&circle, &rectangle];
    for shape in shapes {
        process_shape(shape);
    }

    println!("\n=== Caution ===");
    println!("Downcasting breaks abstraction - use sparingly!");
    println!("Prefer adding methods to the trait when possible.");
}
