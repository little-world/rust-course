//! Pattern 3: Struct Memory and Update Patterns
//! Example: Understanding Partial Moves
//!
//! Run with: cargo run --example p3_partial_moves

struct Data {
    copyable: i32,    // Implements Copy
    moveable: String, // Does not implement Copy
}

fn main() {
    // Usage: After moving non-Copy field, only Copy fields remain accessible.
    let data = Data {
        copyable: 42,
        moveable: "hello".to_string(),
    };

    // Move the String out
    let s = data.moveable;
    println!("Moved string: {}", s);

    // Copy field still accessible
    assert_eq!(data.copyable, 42);
    println!("Copy field still accessible: {}", data.copyable);

    // These would error - value partially moved:
    // println!("{:?}", data);        // Error: value partially moved
    // let d = data;                  // Error: cannot use `data` as a whole
    // println!("{}", data.moveable); // Error: value moved

    // Demonstrating with destructuring
    let data2 = Data {
        copyable: 100,
        moveable: "world".to_string(),
    };

    // Destructure - moves moveable, copies copyable
    let Data { copyable: c, moveable: m } = data2;
    println!("\nDestructured: copyable={}, moveable={}", c, m);

    // Pattern: Clone if you need to keep the original
    let data3 = Data {
        copyable: 200,
        moveable: "preserved".to_string(),
    };

    let s_cloned = data3.moveable.clone(); // Clone instead of move
    println!("\nCloned string: {}", s_cloned);
    println!("Original still accessible: {}", data3.moveable);
}
