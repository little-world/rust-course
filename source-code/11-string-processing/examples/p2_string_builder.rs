//! Pattern 2: String Builder Pattern
//! Basic StringBuilder with Capacity Pre-allocation
//!
//! Run with: cargo run --example p2_string_builder

fn main() {
    println!("=== String Builder Pattern ===\n");

    // Simple string building
    println!("=== Basic StringBuilder ===\n");

    let mut sb = StringBuilder::with_capacity(100);
    sb.append("Hello")
      .append(", ")
      .append("World")
      .append("!");
    println!("Built: {}", sb.as_str());

    // Using append_line
    println!("\n=== StringBuilder with Lines ===\n");

    let mut sb = StringBuilder::with_capacity(200);
    sb.append_line("First line")
      .append_line("Second line")
      .append_line("Third line");
    println!("Multi-line:\n{}", sb.as_str());

    // Using append_fmt
    println!("=== StringBuilder with Formatting ===\n");

    let mut sb = StringBuilder::with_capacity(100);
    for i in 1..=3 {
        sb.append_fmt(format_args!("Item {}: value = {}\n", i, i * 10));
    }
    println!("Formatted:\n{}", sb.build());

    println!("=== Key Points ===");
    println!("1. Pre-allocate capacity when size known");
    println!("2. Method chaining with &mut self returns");
    println!("3. build(self) transfers ownership without copying");
}

struct StringBuilder {
    buffer: String,
}

impl StringBuilder {
    fn new() -> Self {
        StringBuilder {
            buffer: String::new(),
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        StringBuilder {
            buffer: String::with_capacity(capacity),
        }
    }

    fn append(&mut self, s: &str) -> &mut Self {
        self.buffer.push_str(s);
        self
    }

    fn append_line(&mut self, s: &str) -> &mut Self {
        self.buffer.push_str(s);
        self.buffer.push('\n');
        self
    }

    fn append_fmt(&mut self, args: std::fmt::Arguments) -> &mut Self {
        use std::fmt::Write;
        let _ = write!(&mut self.buffer, "{}", args);
        self
    }

    fn build(self) -> String {
        self.buffer
    }

    fn as_str(&self) -> &str {
        &self.buffer
    }
}
