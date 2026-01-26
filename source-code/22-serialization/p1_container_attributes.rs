// Pattern 1: Container Attributes and Enum Representations
use serde::{Deserialize, Serialize};

// Rename all fields to camelCase automatically
// Rust: status_code → JSON: statusCode
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ApiResponse {
    status_code: u32,
    error_message: Option<String>,
    response_data: Vec<String>,
}

fn rename_all_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Rename All Demo ===\n");

    let response = ApiResponse {
        status_code: 200,
        error_message: None,
        response_data: vec!["item1".to_string(), "item2".to_string()],
    };

    let json = serde_json::to_string_pretty(&response)?;
    println!("ApiResponse (camelCase):\n{}", json);

    Ok(())
}

// Deny unknown fields during deserialization
// Fails if JSON contains fields not defined in the struct
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct StrictConfig {
    host: String,
    port: u16,
}

fn strict_config_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Strict Config Demo ===\n");

    // Valid JSON
    let valid_json = r#"{"host": "localhost", "port": 8080}"#;
    let config: StrictConfig = serde_json::from_str(valid_json)?;
    println!("Valid config: {:?}", config);

    // Invalid JSON with extra field
    let invalid_json = r#"{"host": "localhost", "port": 8080, "extra": "field"}"#;
    match serde_json::from_str::<StrictConfig>(invalid_json) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected error with extra field: {}", e),
    }

    Ok(())
}

// Tagged enum: adds a "type" field to identify the variant
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Message {
    Text { content: String },
    Image { url: String, width: u32, height: u32 },
    Video { url: String, duration: u32 },
}

fn tagged_enum_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Tagged Enum Demo ===\n");

    let messages = vec![
        Message::Text {
            content: "Hello, world!".to_string(),
        },
        Message::Image {
            url: "https://example.com/image.jpg".to_string(),
            width: 1920,
            height: 1080,
        },
        Message::Video {
            url: "https://example.com/video.mp4".to_string(),
            duration: 120,
        },
    ];

    for msg in &messages {
        let json = serde_json::to_string_pretty(msg)?;
        println!("{}\n", json);
    }

    // Deserialize
    let json = r#"{"type": "Image", "url": "https://example.com/pic.jpg", "width": 800, "height": 600}"#;
    let msg: Message = serde_json::from_str(json)?;
    println!("Deserialized: {:?}", msg);

    Ok(())
}

// Adjacently tagged enum: tag and content in separate fields
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
enum Event {
    Click { x: i32, y: i32 },
    KeyPress { key: String },
    Scroll { delta: i32 },
}

fn adjacent_tagged_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Adjacently Tagged Enum Demo ===\n");

    let event = Event::Click { x: 100, y: 200 };
    let json = serde_json::to_string_pretty(&event)?;
    println!("Adjacent tagged:\n{}", json);
    // {"type": "Click", "data": {"x": 100, "y": 200}}

    Ok(())
}

// Untagged enum: no type field, variant determined by structure
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Value {
    Integer(i64),
    Float(f64),
    Text(String),
    Bool(bool),
}

fn untagged_enum_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Untagged Enum Demo ===\n");

    let values = vec![
        Value::Integer(42),
        Value::Float(3.14),
        Value::Text("hello".to_string()),
        Value::Bool(true),
    ];

    for val in &values {
        let json = serde_json::to_string(val)?;
        println!("Value: {} -> JSON: {}", format!("{:?}", val), json);
    }

    // Deserialize - serde tries each variant in order
    let json_values = vec!["42", "3.14", "\"hello\"", "true"];
    println!("\nDeserializing:");
    for json in json_values {
        let val: Value = serde_json::from_str(json)?;
        println!("  {} -> {:?}", json, val);
    }

    Ok(())
}

// Externally tagged enum (default)
#[derive(Serialize, Deserialize, Debug)]
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle { base: f64, height: f64 },
}

fn external_tagged_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Externally Tagged Enum Demo (Default) ===\n");

    let shapes = vec![
        Shape::Circle { radius: 5.0 },
        Shape::Rectangle {
            width: 10.0,
            height: 20.0,
        },
    ];

    for shape in &shapes {
        let json = serde_json::to_string_pretty(shape)?;
        println!("{}\n", json);
    }
    // {"Circle": {"radius": 5.0}}
    // {"Rectangle": {"width": 10.0, "height": 20.0}}

    Ok(())
}

// Enum with renamed variants
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    #[serde(rename = "PATCH")]
    Patch,
}

fn renamed_variants_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Renamed Variants Demo ===\n");

    let methods = vec![
        HttpMethod::Get,
        HttpMethod::Post,
        HttpMethod::Patch,
    ];

    for method in &methods {
        let json = serde_json::to_string(method)?;
        println!("{:?} -> {}", method, json);
    }

    Ok(())
}

// Transparent wrapper
#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
struct UserId(u64);

#[derive(Serialize, Deserialize, Debug)]
struct UserWithId {
    id: UserId,
    name: String,
}

fn transparent_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Transparent Demo ===\n");

    let user = UserWithId {
        id: UserId(12345),
        name: "Alice".to_string(),
    };

    let json = serde_json::to_string_pretty(&user)?;
    println!("User with transparent UserId:\n{}", json);
    // id appears as 12345, not {"0": 12345}

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Container Attributes Demo ===\n");

    rename_all_demo()?;
    strict_config_demo()?;
    tagged_enum_demo()?;
    adjacent_tagged_demo()?;
    untagged_enum_demo()?;
    external_tagged_demo()?;
    renamed_variants_demo()?;
    transparent_demo()?;

    println!("\nContainer attributes demo completed");
    Ok(())
}
