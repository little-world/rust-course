// Pattern 4: Memory Layout Optimization - Optimizing Enum Size

// Bad: 1024+ bytes for every instance
#[allow(dead_code)]
enum Large {
    Small(u8),
    Big([u8; 1024]),
}

// Good: ~16 bytes (pointer + discriminant)
#[allow(dead_code)]
enum Optimized {
    Small(u8),
    Big(Box<[u8; 1024]>),
}

fn main() {
    assert!(std::mem::size_of::<Large>() > 1024);
    assert!(std::mem::size_of::<Optimized>() <= 16);

    println!("Large enum size: {} bytes", std::mem::size_of::<Large>());
    println!("Optimized enum size: {} bytes", std::mem::size_of::<Optimized>());
    println!("Enum size optimization example completed");
}
