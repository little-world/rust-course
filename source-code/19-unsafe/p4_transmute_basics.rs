// Pattern 4: Basic Transmute
use std::mem;

fn transmute_basics() {
    let a: u32 = 0x12345678;
    let b: [u8; 4] = unsafe { mem::transmute(a) };
    println!("Bytes: {:?}", b);  // Depends on endianness!

    let f: f32 = 3.14;
    let bits: u32 = unsafe { mem::transmute(f) };
    println!("Float bits: 0x{:08x}", bits);

    // BETTER: Use safe built-in methods
    let bits_safe = f.to_bits();
    assert_eq!(bits, bits_safe);

    let f2 = f32::from_bits(bits);
    assert_eq!(f, f2);
}

fn main() {
    transmute_basics();

    // Usage: Get raw float bits (prefer to_bits() in real code)
    let pi: f32 = 3.14159;
    let bits = pi.to_bits(); // Safe alternative to transmute
    println!("Pi bits: 0x{:08x}", bits);

    println!("Transmute basics example completed");
}
