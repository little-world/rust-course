//! Pattern 2: Generic Structs and Enums
//! Example: Specialized Impl for Specific Types
//!
//! Run with: cargo run --example p2_specialized_impl

use std::ops::Mul;

#[derive(Debug, PartialEq, Clone, Copy)]
struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn new(x: T, y: T) -> Self {
        Point { x, y }
    }
}

// Specialized impl only for f64
impl Point<f64> {
    fn distance_from_origin(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    fn angle(&self) -> f64 {
        self.y.atan2(self.x)
    }

    fn rotate(&self, angle: f64) -> Point<f64> {
        let cos = angle.cos();
        let sin = angle.sin();
        Point {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }
}

// Generic impl with bounds for all multipliable types
impl<T: Mul<Output = T> + Copy> Point<T> {
    fn scale(&self, factor: T) -> Point<T> {
        Point {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
}

// Specialized impl only for i32
impl Point<i32> {
    fn manhattan_distance(&self) -> i32 {
        self.x.abs() + self.y.abs()
    }

    fn quadrant(&self) -> Option<u8> {
        match (self.x.signum(), self.y.signum()) {
            (1, 1) => Some(1),
            (-1, 1) => Some(2),
            (-1, -1) => Some(3),
            (1, -1) => Some(4),
            _ => None, // On an axis
        }
    }
}

fn main() {
    println!("=== f64-Specific Methods ===");
    // Usage: f64-specific method only available on Point<f64>.
    let p = Point::new(3.0_f64, 4.0_f64);
    let dist = p.distance_from_origin();
    println!("Point(3.0, 4.0).distance_from_origin() = {} (Pythagorean)", dist);

    let angle = p.angle();
    println!("Point(3.0, 4.0).angle() = {:.4} radians", angle);

    let rotated = p.rotate(std::f64::consts::PI / 2.0);
    println!("Point(3.0, 4.0).rotate(PI/2) = Point({:.2}, {:.2})", rotated.x, rotated.y);

    println!("\n=== Generic scale() Method ===");
    // Works for any type with Mul + Copy
    let pi32 = Point::new(2, 3);
    let scaled_i32 = pi32.scale(5);
    println!("Point<i32>(2, 3).scale(5) = {:?}", scaled_i32);

    let pf64 = Point::new(1.5, 2.5);
    let scaled_f64 = pf64.scale(2.0);
    println!("Point<f64>(1.5, 2.5).scale(2.0) = {:?}", scaled_f64);

    println!("\n=== i32-Specific Methods ===");
    let p = Point::new(-3, 4);
    println!("Point(-3, 4).manhattan_distance() = {}", p.manhattan_distance());
    println!("Point(-3, 4).quadrant() = {:?}", p.quadrant());

    let origin = Point::new(0, 5);
    println!("Point(0, 5).quadrant() = {:?} (on y-axis)", origin.quadrant());

    println!("\n=== Method Availability ===");
    let float_point = Point::new(1.0_f64, 1.0_f64);
    let int_point = Point::new(1_i32, 1_i32);

    println!("Point<f64> has: distance_from_origin, angle, rotate, scale");
    println!("Point<i32> has: manhattan_distance, quadrant, scale");

    // This wouldn't compile:
    // int_point.distance_from_origin(); // No such method for i32
    // float_point.manhattan_distance(); // No such method for f64

    // Both have scale:
    println!("float_point.scale(2.0) = {:?}", float_point.scale(2.0));
    println!("int_point.scale(2) = {:?}", int_point.scale(2));
}
