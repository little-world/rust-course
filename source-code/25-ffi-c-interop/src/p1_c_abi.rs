//! Pattern 1: C ABI Compatibility
//!
//! Demonstrates #[repr(C)] for struct layout and extern "C" for calling conventions.
//! Shows how Rust and C types map to each other and how to call C functions.

use std::os::raw::{c_int, c_void};

// FFI declarations matching our C library
#[repr(C)]
pub struct CStruct {
    pub a: u8,
    pub b: u32,
    pub c: u16,
}

extern "C" {
    fn process_struct(s: *const CStruct) -> i32;
    fn c_add(a: i32, b: i32) -> i32;
    fn c_sqrt(x: f64) -> f64;
    fn c_abs(n: i32) -> i32;
}

// For comparison: Rust's default layout (may differ from C)
struct RustStruct {
    a: u8,
    b: u32,
    c: u16,
}

fn main() {
    println!("=== Pattern 1: C ABI Compatibility ===\n");

    // Demonstrate struct layout differences
    println!("Struct Layout Comparison:");
    println!("  RustStruct size: {} bytes, align: {}",
        std::mem::size_of::<RustStruct>(),
        std::mem::align_of::<RustStruct>());
    println!("  CStruct size: {} bytes, align: {}",
        std::mem::size_of::<CStruct>(),
        std::mem::align_of::<CStruct>());

    // Show field offsets for CStruct
    println!("\nCStruct field offsets (C-compatible):");
    println!("  a (u8) at offset 0");
    println!("  b (u32) at offset 4 (after 3 bytes padding)");
    println!("  c (u16) at offset 8");

    // Create a C-compatible struct and pass to C
    let my_struct = CStruct {
        a: 10,
        b: 100,
        c: 50,
    };

    println!("\n--- Calling C Functions ---");

    unsafe {
        // Pass struct to C function
        let sum = process_struct(&my_struct);
        println!("process_struct({{a: 10, b: 100, c: 50}}) = {}", sum);

        // Call simple C math functions
        let add_result = c_add(25, 17);
        println!("c_add(25, 17) = {}", add_result);

        let sqrt_result = c_sqrt(144.0);
        println!("c_sqrt(144.0) = {}", sqrt_result);

        let abs_result = c_abs(-42);
        println!("c_abs(-42) = {}", abs_result);
    }

    // Demonstrate type mappings
    println!("\n--- C Type Mappings ---");
    println!("  c_int size: {} bytes", std::mem::size_of::<c_int>());
    println!("  c_void size: {} bytes", std::mem::size_of::<c_void>());
    println!("  i32 size: {} bytes", std::mem::size_of::<i32>());
    println!("  f64 size: {} bytes (same as C double)", std::mem::size_of::<f64>());
    println!("  usize size: {} bytes (same as C size_t)", std::mem::size_of::<usize>());

    println!("\nAll C ABI examples completed successfully!");
}
