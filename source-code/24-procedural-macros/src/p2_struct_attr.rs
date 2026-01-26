//! Pattern 2: Struct Attribute Macro
//!
//! Demonstrates modifying struct definitions by injecting new fields.
//! The macro adds a _debug_info field and a getter method.

use my_macros::add_debug_info;

#[add_debug_info]
struct MyStruct {
    pub value: i32,
}

#[add_debug_info]
struct Config {
    pub host: String,
    pub port: u16,
}

fn main() {
    println!("=== Struct Attribute Macro Demo ===\n");

    let s = MyStruct {
        value: 42,
        _debug_info: "Created at startup".to_string(),
    };
    println!("MyStruct value: {}", s.value);
    println!("MyStruct debug_info: {}", s.debug_info());

    let config = Config {
        host: "localhost".to_string(),
        port: 8080,
        _debug_info: "Production config loaded from env".to_string(),
    };
    println!("\nConfig host: {}", config.host);
    println!("Config port: {}", config.port);
    println!("Config debug_info: {}", config.debug_info());
}
