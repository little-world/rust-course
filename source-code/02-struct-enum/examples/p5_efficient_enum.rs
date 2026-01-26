//! Pattern 5: Advanced Enum Techniques
//! Example: Memory-Efficient Large Variants
//!
//! Run with: cargo run --example p5_efficient_enum

use std::mem::size_of;

// Without Box: enum size = size of largest variant (LargeData)
#[allow(dead_code)]
enum Inefficient {
    Small(u8),
    Large([u8; 1024]), // 1KB - every variant takes this space
}

// With Box: enum size = size of pointer (8 bytes on 64-bit)
enum Efficient {
    Small(u8),
    Large(Box<[u8; 1024]>), // Only allocates when this variant is used
}

// Real-world example: Error types with large diagnostic info
enum AppError {
    NotFound,
    InvalidInput(String),
    // Large variant with detailed stack trace - boxed for efficiency
    InternalError(Box<DetailedError>),
}

struct DetailedError {
    message: String,
    stack_trace: Vec<String>,
    context: std::collections::HashMap<String, String>,
}

fn main() {
    // Size comparison
    assert!(size_of::<Inefficient>() >= 1024);
    assert!(size_of::<Efficient>() <= 16);

    println!("Size comparison:");
    println!("  Inefficient enum: {} bytes", size_of::<Inefficient>());
    println!("  Efficient enum: {} bytes", size_of::<Efficient>());
    println!(
        "  Savings: {} bytes per instance",
        size_of::<Inefficient>() - size_of::<Efficient>()
    );

    // Using the efficient version
    let small = Efficient::Small(42);
    let large = Efficient::Large(Box::new([0u8; 1024]));

    match &small {
        Efficient::Small(v) => println!("\nSmall variant: {}", v),
        Efficient::Large(_) => println!("Large variant"),
    }

    match &large {
        Efficient::Small(v) => println!("Small variant: {}", v),
        Efficient::Large(arr) => println!("Large variant: {} bytes", arr.len()),
    }

    // AppError example
    println!("\nAppError sizes:");
    println!("  AppError: {} bytes", size_of::<AppError>());
    println!("  DetailedError: {} bytes", size_of::<DetailedError>());

    let errors: Vec<AppError> = vec![
        AppError::NotFound,
        AppError::InvalidInput("bad input".to_string()),
        AppError::InternalError(Box::new(DetailedError {
            message: "Something went wrong".to_string(),
            stack_trace: vec!["fn main".to_string(), "fn process".to_string()],
            context: std::collections::HashMap::new(),
        })),
    ];

    println!("\nVector of {} errors takes {} bytes on stack",
        errors.len(),
        size_of::<Vec<AppError>>()
    );
}
