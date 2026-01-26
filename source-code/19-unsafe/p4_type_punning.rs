// Pattern 4: Type Punning for Optimized Code
union FloatUnion {
    f: f32,
    u: u32,
}

fn fast_float_bits(f: f32) -> u32 {
    let union_val = FloatUnion { f };
    unsafe { union_val.u }  // Reading inactive union field is unsafe
}

// For real code, use the built-in method
fn correct_float_bits(f: f32) -> u32 {
    f.to_bits()
}

fn main() {
    let pi = 3.14159f32;

    let bits_union = fast_float_bits(pi);
    let bits_safe = correct_float_bits(pi);

    println!("Float: {}", pi);
    println!("Bits via union: 0x{:08x}", bits_union);
    println!("Bits via to_bits(): 0x{:08x}", bits_safe);
    assert_eq!(bits_union, bits_safe);

    println!("Type punning example completed");
}
