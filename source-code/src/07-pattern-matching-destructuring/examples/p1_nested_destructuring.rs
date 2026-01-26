//! Pattern 1: Advanced Match Patterns
//! Example: Nested Destructuring
//!
//! Run with: cargo run --example p1_nested_destructuring

#[derive(Debug, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug)]
enum Shape {
    Circle { center: Point, radius: f64 },
    Rectangle { top_left: Point, bottom_right: Point },
    Triangle { a: Point, b: Point, c: Point },
}

fn contains_origin(shape: &Shape) -> bool {
    match shape {
        // Destructure directly to the inner `x` and `y` fields.
        // Match only circles centered at origin.
        Shape::Circle {
            center: Point { x: 0, y: 0 },
            ..
        } => true,

        // Destructure rectangle corners and use guard to check.
        // Note: x1, y1, x2, y2 are &i32 due to match ergonomics on &Shape.
        Shape::Rectangle {
            top_left: Point { x: x1, y: y1 },
            bottom_right: Point { x: x2, y: y2 },
        } if *x1 <= 0 && *x2 >= 0 && *y1 >= 0 && *y2 <= 0 => true,

        // Check if origin is inside triangle (simplified: just check if any vertex is origin)
        Shape::Triangle {
            a: Point { x: 0, y: 0 },
            ..
        }
        | Shape::Triangle {
            b: Point { x: 0, y: 0 },
            ..
        }
        | Shape::Triangle {
            c: Point { x: 0, y: 0 },
            ..
        } => true,

        _ => false,
    }
}

fn describe_shape(shape: &Shape) -> String {
    match shape {
        Shape::Circle {
            center: Point { x, y },
            radius,
        } => format!("Circle at ({}, {}) with radius {}", x, y, radius),

        Shape::Rectangle {
            top_left: Point { x: x1, y: y1 },
            bottom_right: Point { x: x2, y: y2 },
        } => {
            let width = x2 - x1;
            let height = y1 - y2;
            format!("Rectangle {}x{} from ({}, {}) to ({}, {})", width, height, x1, y1, x2, y2)
        }

        Shape::Triangle { a, b, c } => {
            format!("Triangle with vertices {:?}, {:?}, {:?}", a, b, c)
        }
    }
}

#[derive(Debug)]
enum Config {
    Simple(String),
    WithOptions {
        name: String,
        options: Options,
    },
}

#[derive(Debug)]
struct Options {
    timeout: u32,
    retries: u32,
}

fn get_timeout(config: &Config) -> u32 {
    match config {
        Config::Simple(_) => 30, // Default timeout
        Config::WithOptions {
            options: Options { timeout, .. },
            ..
        } => *timeout,
    }
}

fn main() {
    println!("=== Nested Destructuring: Shapes ===");
    // Usage: check if shapes contain the origin point
    let shapes = [
        Shape::Circle {
            center: Point { x: 0, y: 0 },
            radius: 5.0,
        },
        Shape::Circle {
            center: Point { x: 5, y: 5 },
            radius: 3.0,
        },
        Shape::Rectangle {
            top_left: Point { x: -5, y: 5 },
            bottom_right: Point { x: 5, y: -5 },
        },
        Shape::Rectangle {
            top_left: Point { x: 1, y: 5 },
            bottom_right: Point { x: 5, y: 1 },
        },
        Shape::Triangle {
            a: Point { x: 0, y: 0 },
            b: Point { x: 3, y: 0 },
            c: Point { x: 0, y: 3 },
        },
    ];

    for shape in &shapes {
        println!("  {} - contains origin: {}", describe_shape(shape), contains_origin(shape));
    }

    println!("\n=== Nested Destructuring: Config ===");
    let configs = [
        Config::Simple("basic".to_string()),
        Config::WithOptions {
            name: "advanced".to_string(),
            options: Options {
                timeout: 60,
                retries: 3,
            },
        },
    ];

    for config in &configs {
        println!("  {:?} - timeout: {}", config, get_timeout(config));
    }

    println!("\n=== Key Points ===");
    println!("1. Destructure multiple levels in one pattern:");
    println!("   Shape::Circle {{ center: Point {{ x, y }}, .. }}");
    println!();
    println!("2. Use `..` to ignore fields you don't need:");
    println!("   Options {{ timeout, .. }}");
    println!();
    println!("3. Guards can reference any destructured variable:");
    println!("   }} if *x1 <= 0 && *x2 >= 0");
    println!();
    println!("4. Or-patterns combine multiple shapes:");
    println!("   Shape::Triangle {{ a: Point {{ x: 0, y: 0 }}, .. }}");
    println!("   | Shape::Triangle {{ b: Point {{ x: 0, y: 0 }}, .. }}");
}
