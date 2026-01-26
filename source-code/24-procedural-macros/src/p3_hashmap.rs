//! Pattern 3: Complex Function-like Macro (HashMap Literal)
//!
//! Demonstrates parsing custom syntax with operators and collections.
//! The hashmap! macro uses key => value syntax with comma separators.

use my_macros::hashmap;

fn main() {
    println!("=== HashMap Literal Macro Demo ===\n");

    let colors = hashmap! {
        "red" => "#FF0000",
        "green" => "#00FF00",
        "blue" => "#0000FF"
    };
    println!("Colors: {:?}", colors);

    let scores = hashmap! {
        "Alice" => 95,
        "Bob" => 87,
        "Charlie" => 92,
        "Diana" => 98
    };
    println!("\nScores: {:?}", scores);

    let config = hashmap! {
        "host" => "localhost",
        "port" => "8080",
        "debug" => "true"
    };
    println!("\nConfig: {:?}", config);

    // Access values
    println!("\nAlice's score: {}", scores.get("Alice").unwrap());
    println!("Red color code: {}", colors.get("red").unwrap());
}
