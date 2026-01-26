// Pattern 1: Serde Basics - Deriving Serialize and Deserialize
use serde::{Deserialize, Serialize};

// Basic struct with derive macros
#[derive(Serialize, Deserialize, Debug)]
struct Person {
    name: String,
    age: u32,
    email: String,
}

fn basic_serialization() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Serialization Demo ===\n");

    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    };

    // Serialize to JSON (compact representation)
    let json = serde_json::to_string(&person)?;
    println!("JSON: {}", json);
    // Output: {"name":"Alice","age":30,"email":"alice@example.com"}

    // Pretty print (human-readable with indentation)
    let json_pretty = serde_json::to_string_pretty(&person)?;
    println!("\nPretty JSON:\n{}", json_pretty);

    // Deserialize from JSON back to a Rust struct
    // Serde validates types: wrong types or missing fields → error
    let deserialized: Person = serde_json::from_str(&json)?;
    println!("\nDeserialized: {:?}", deserialized);

    Ok(())
}

// Nested structures
#[derive(Serialize, Deserialize, Debug)]
struct Company {
    name: String,
    employees: Vec<Employee>,
    headquarters: Address,
}

#[derive(Serialize, Deserialize, Debug)]
struct Employee {
    name: String,
    role: String,
    salary: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Address {
    street: String,
    city: String,
    country: String,
}

fn nested_structures() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Nested Structures Demo ===\n");

    let company = Company {
        name: "TechCorp".to_string(),
        employees: vec![
            Employee {
                name: "Alice".to_string(),
                role: "Engineer".to_string(),
                salary: 100000.0,
            },
            Employee {
                name: "Bob".to_string(),
                role: "Manager".to_string(),
                salary: 120000.0,
            },
        ],
        headquarters: Address {
            street: "123 Tech Ave".to_string(),
            city: "San Francisco".to_string(),
            country: "USA".to_string(),
        },
    };

    let json = serde_json::to_string_pretty(&company)?;
    println!("Company JSON:\n{}", json);

    // Deserialize back
    let deserialized: Company = serde_json::from_str(&json)?;
    println!("\nDeserialized: {:?}", deserialized);

    Ok(())
}

// Optional and default values
#[derive(Serialize, Deserialize, Debug)]
struct Config {
    host: String,
    port: u16,
    #[serde(default)]
    debug: bool,
    #[serde(default)]
    max_connections: Option<u32>,
}

fn optional_values() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Optional Values Demo ===\n");

    // JSON missing optional fields
    let json = r#"{"host": "localhost", "port": 8080}"#;

    let config: Config = serde_json::from_str(json)?;
    println!("Config from minimal JSON: {:?}", config);
    println!("  debug (defaulted to false): {}", config.debug);
    println!("  max_connections (defaulted to None): {:?}", config.max_connections);

    // JSON with all fields
    let json_full = r#"{"host": "0.0.0.0", "port": 3000, "debug": true, "max_connections": 100}"#;
    let config_full: Config = serde_json::from_str(json_full)?;
    println!("\nConfig from full JSON: {:?}", config_full);

    Ok(())
}

// Enums
#[derive(Serialize, Deserialize, Debug)]
enum Status {
    Active,
    Inactive,
    Pending,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    name: String,
    status: Status,
}

fn enum_serialization() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Enum Serialization Demo ===\n");

    let user = User {
        name: "Alice".to_string(),
        status: Status::Active,
    };

    let json = serde_json::to_string_pretty(&user)?;
    println!("User JSON:\n{}", json);

    // Deserialize
    let deserialized: User = serde_json::from_str(&json)?;
    println!("\nDeserialized: {:?}", deserialized);

    Ok(())
}

// Tuple structs and unit structs
#[derive(Serialize, Deserialize, Debug)]
struct Point(f64, f64);

#[derive(Serialize, Deserialize, Debug)]
struct Color(u8, u8, u8);

fn tuple_structs() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Tuple Structs Demo ===\n");

    let point = Point(10.5, 20.3);
    let color = Color(255, 128, 0);

    let point_json = serde_json::to_string(&point)?;
    let color_json = serde_json::to_string(&color)?;

    println!("Point JSON: {}", point_json);  // [10.5, 20.3]
    println!("Color JSON: {}", color_json);  // [255, 128, 0]

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Serde Basics Demo ===\n");

    basic_serialization()?;
    nested_structures()?;
    optional_values()?;
    enum_serialization()?;
    tuple_structs()?;

    println!("\nSerde basics demo completed");
    Ok(())
}
