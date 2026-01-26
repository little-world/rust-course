//! Pattern 3: Trait Objects and Dynamic Dispatch
//! Example: Heterogeneous Collections with Trait Objects
//!
//! Run with: cargo run --example p3_heterogeneous

trait Drawable {
    fn draw(&self);
    fn name(&self) -> &str;
    fn area(&self) -> f64;
}

struct Circle {
    radius: f64,
}

impl Drawable for Circle {
    fn draw(&self) {
        println!("Drawing circle with radius {}", self.radius);
    }

    fn name(&self) -> &str {
        "Circle"
    }

    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Drawable for Rectangle {
    fn draw(&self) {
        println!("Drawing rectangle {}x{}", self.width, self.height);
    }

    fn name(&self) -> &str {
        "Rectangle"
    }

    fn area(&self) -> f64 {
        self.width * self.height
    }
}

struct Triangle {
    base: f64,
    height: f64,
}

impl Drawable for Triangle {
    fn draw(&self) {
        println!("Drawing triangle with base {} and height {}", self.base, self.height);
    }

    fn name(&self) -> &str {
        "Triangle"
    }

    fn area(&self) -> f64 {
        0.5 * self.base * self.height
    }
}

fn draw_all(shapes: &[Box<dyn Drawable>]) {
    for shape in shapes {
        shape.draw();
    }
}

fn total_area(shapes: &[Box<dyn Drawable>]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

fn main() {
    // Usage: Vec holds different concrete types via shared trait interface.
    let shapes: Vec<Box<dyn Drawable>> = vec![
        Box::new(Circle { radius: 5.0 }),
        Box::new(Rectangle { width: 10.0, height: 20.0 }),
        Box::new(Triangle { base: 8.0, height: 6.0 }),
        Box::new(Circle { radius: 3.0 }),
    ];

    println!("=== Drawing All Shapes ===");
    draw_all(&shapes);

    println!("\n=== Shape Details ===");
    for shape in &shapes {
        println!("{}: area = {:.2}", shape.name(), shape.area());
    }

    println!("\n=== Total Area ===");
    println!("Total area of all shapes: {:.2}", total_area(&shapes));

    // Can add shapes dynamically
    println!("\n=== Dynamic Collection ===");
    let mut dynamic_shapes: Vec<Box<dyn Drawable>> = Vec::new();

    dynamic_shapes.push(Box::new(Circle { radius: 1.0 }));
    println!("Added circle, count: {}", dynamic_shapes.len());

    dynamic_shapes.push(Box::new(Rectangle { width: 2.0, height: 3.0 }));
    println!("Added rectangle, count: {}", dynamic_shapes.len());

    draw_all(&dynamic_shapes);
}
