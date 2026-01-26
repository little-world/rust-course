//! Pattern 5: Complete Example (ToJson Serialization)
//!
//! Demonstrates a complete, practical macro generating serialization code.
//! Similar to how serde generates optimized serialization without reflection.

use my_macros::ToJson;

#[derive(ToJson)]
struct Person {
    name: String,
    age: u32,
    active: bool,
}

#[derive(ToJson)]
struct Product {
    id: u64,
    name: String,
    price: f64,
    in_stock: bool,
}

#[derive(ToJson)]
struct ApiResponse {
    status: String,
    code: u16,
    message: String,
}

fn main() {
    println!("=== ToJson Derive Demo ===\n");

    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        active: true,
    };
    println!("Person JSON:");
    println!("{}\n", person.to_json());

    let product = Product {
        id: 12345,
        name: "Widget".to_string(),
        price: 29.99,
        in_stock: true,
    };
    println!("Product JSON:");
    println!("{}\n", product.to_json());

    let response = ApiResponse {
        status: "success".to_string(),
        code: 200,
        message: "Data retrieved successfully".to_string(),
    };
    println!("API Response JSON:");
    println!("{}", response.to_json());
}
