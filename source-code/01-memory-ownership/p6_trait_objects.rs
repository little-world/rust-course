// Pattern 6: Trait Objects with Box
trait Drawable {
    fn draw(&self);
}

struct Circle { radius: f64 }
struct Rectangle { width: f64, height: f64 }

impl Drawable for Circle {
    fn draw(&self) { println!("Circle r={}", self.radius); }
}

impl Drawable for Rectangle {
    fn draw(&self) { println!("Rect {}x{}", self.width, self.height); }
}

fn main() {
    // Store different types in one collection
    let shapes: Vec<Box<dyn Drawable>> = vec![
        Box::new(Circle { radius: 5.0 }),
        Box::new(Rectangle { width: 10.0, height: 20.0 }),
    ];
    for shape in &shapes { shape.draw(); }
    println!("Trait objects example completed");
}
