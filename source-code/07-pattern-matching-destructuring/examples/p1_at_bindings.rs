//! Pattern 1: Advanced Match Patterns
//! Example: @ Bindings to Capture and Test
//!
//! Run with: cargo run --example p1_at_bindings

#[derive(Debug)]
enum Port {
    WellKnown(u16),
    Registered(u16),
    Dynamic(u16),
}

#[derive(Debug)]
struct ValidationError(String);

fn validate_port(port: u16) -> Result<Port, ValidationError> {
    match port {
        // @ binds the value while also testing it against a pattern
        p @ 1..=1023 => Ok(Port::WellKnown(p)),
        p @ 1024..=49151 => Ok(Port::Registered(p)),
        p @ 49152..=65535 => Ok(Port::Dynamic(p)),
        0 => Err(ValidationError("Port 0 is reserved".to_string())),
    }
}

#[derive(Debug)]
enum AgeGroup {
    Child(u8),
    Teen(u8),
    Adult(u8),
    Senior(u8),
}

fn categorize_age(age: u8) -> AgeGroup {
    match age {
        a @ 0..=12 => AgeGroup::Child(a),
        a @ 13..=19 => AgeGroup::Teen(a),
        a @ 20..=64 => AgeGroup::Adult(a),
        a @ 65..=255 => AgeGroup::Senior(a),
    }
}

#[derive(Debug)]
enum Message {
    Hello { id: u32 },
    Goodbye { id: u32 },
    Data { id: u32, payload: String },
}

fn process_message(msg: &Message) {
    match msg {
        // @ can bind to struct patterns too
        m @ Message::Hello { id } => {
            println!("Received hello from {}, full message: {:?}", id, m);
        }
        m @ Message::Goodbye { id } => {
            println!("Received goodbye from {}, full message: {:?}", id, m);
        }
        // Combine @ with guards
        Message::Data { id, payload } if payload.len() > 100 => {
            println!("Large data message from {}: {} bytes", id, payload.len());
        }
        Message::Data { id, payload } => {
            println!("Data from {}: {}", id, payload);
        }
    }
}

fn main() {
    println!("=== @ Bindings: Port Validation ===");
    // Usage: categorize ports while capturing their values
    let ports = [0, 80, 443, 8080, 50000, 65535];
    for port in ports {
        let result = validate_port(port);
        println!("  {} => {:?}", port, result);
    }

    println!("\n=== @ Bindings: Age Categorization ===");
    let ages = [5, 15, 30, 70];
    for age in ages {
        let group = categorize_age(age);
        println!("  {} => {:?}", age, group);
    }

    println!("\n=== @ Bindings with Structs ===");
    let messages = [
        Message::Hello { id: 1 },
        Message::Goodbye { id: 2 },
        Message::Data {
            id: 3,
            payload: "short".to_string(),
        },
        Message::Data {
            id: 4,
            payload: "x".repeat(150),
        },
    ];

    for msg in &messages {
        process_message(msg);
    }

    println!("\n=== @ Binding Syntax ===");
    println!("  name @ pattern => {{ ... }}");
    println!("\n`name` is bound to the ENTIRE matched value,");
    println!("while `pattern` specifies what to match.");

    println!("\n=== Why Use @ Bindings? ===");
    println!("Without @:");
    println!("  match port {{");
    println!("      1..=1023 => Port::WellKnown(port), // need to reference `port` again");
    println!("  }}");
    println!("\nWith @:");
    println!("  match port {{");
    println!("      p @ 1..=1023 => Port::WellKnown(p), // `p` is already the value");
    println!("  }}");
}
