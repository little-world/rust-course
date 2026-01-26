// Pattern 2: Copy Types
fn copy_semantics() {
    let x: i32 = 42;
    let y = x;  // x is COPIED to y

    println!("x = {}, y = {}", x, y);  // Both valid!

    // Primitives are Copy: i32, f64, bool, char, etc.
    // Tuples of Copy types are Copy: (i32, bool)
    // Arrays of Copy types are Copy: [i32; 10]
    // References are Copy: &T (but not &mut T)
}

// Make your own type Copy (only if all fields are Copy)
#[derive(Copy, Clone)]
struct Point {
    x: f64,
    y: f64,
}

fn copy_custom_type() {
    let p1 = Point { x: 1.0, y: 2.0 };
    let p2 = p1;  // Copied!
    println!("p1: ({}, {})", p1.x, p1.y);  // Both valid
    println!("p2: ({}, {})", p2.x, p2.y);
}

fn main() {
    copy_semantics();
    copy_custom_type();
    println!("Copy semantics example completed");
}
