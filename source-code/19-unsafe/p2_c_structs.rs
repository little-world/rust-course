// Pattern 2: C Struct Interop
use std::os::raw::c_int;

/// A point with C-compatible layout.
#[repr(C)]
struct Point {
    x: c_int,  // Use c_int, not i32 (they're the same on most platforms but not guaranteed)
    y: c_int,
}

/// An enum with explicit discriminant values for C.
#[repr(C)]
#[derive(Debug)]
enum Status {
    Success = 0,
    Error = 1,
    Pending = 2,
}

fn use_c_structs() {
    let point = Point { x: 10, y: 20 };
    println!("Point: ({}, {})", point.x, point.y);
    println!("Point size: {} bytes", std::mem::size_of::<Point>());

    let status = Status::Success;
    println!("Status: {:?}", status);
    println!("Status size: {} bytes", std::mem::size_of::<Status>());
}

fn main() {
    use_c_structs();

    // Usage: Create C-compatible struct
    #[repr(C)]
    struct Vec2 { x: f32, y: f32 }
    let v = Vec2 { x: 1.0, y: 2.0 };
    println!("Vec2: ({}, {})", v.x, v.y);

    println!("C structs example completed");
}
