// Pattern 4: Memory Layout Optimization - Field Ordering

// Bad: 24 bytes with padding
#[repr(C)]
struct Unoptimized {
    a: u8,      // 1 byte + 7 padding
    b: u64,     // 8 bytes
    c: u8,      // 1 byte + 7 padding
}               // Total: 24 bytes

// Good: 16 bytes
#[repr(C)]
struct Optimized {
    b: u64,     // 8 bytes (largest first)
    a: u8,      // 1 byte
    c: u8,      // 1 byte + 6 padding
}               // Total: 16 bytes

fn main() {
    assert_eq!(std::mem::size_of::<Unoptimized>(), 24);
    assert_eq!(std::mem::size_of::<Optimized>(), 16);

    println!("Unoptimized size: {} bytes", std::mem::size_of::<Unoptimized>());
    println!("Optimized size: {} bytes", std::mem::size_of::<Optimized>());
    println!("Field ordering example completed");
}
