//! Pattern 2: Associated Types vs Generics
//! Example: Generic Type Parameters
//!
//! Run with: cargo run --example p2_generic_parser

// With generics: Multiple implementations possible
trait GenericParser<Output> {
    fn parse(&self, input: &str) -> Result<Output, String>;
}

// A single type can implement GenericParser for multiple Output types
struct SimpleParser;

impl GenericParser<i32> for SimpleParser {
    fn parse(&self, input: &str) -> Result<i32, String> {
        input.trim().parse().map_err(|e| format!("{}", e))
    }
}

impl GenericParser<bool> for SimpleParser {
    fn parse(&self, input: &str) -> Result<bool, String> {
        match input.trim() {
            "true" | "1" | "yes" => Ok(true),
            "false" | "0" | "no" => Ok(false),
            _ => Err("invalid bool".to_string()),
        }
    }
}

impl GenericParser<f64> for SimpleParser {
    fn parse(&self, input: &str) -> Result<f64, String> {
        input.trim().parse().map_err(|e| format!("{}", e))
    }
}

fn main() {
    // Usage: Same parser, multiple Output types; caller specifies which.
    let parser = SimpleParser;

    // Must use turbofish or type annotation to specify Output
    let num: Result<i32, _> = GenericParser::<i32>::parse(&parser, "42");
    println!("Parsed i32: {:?}", num);

    let b: Result<bool, _> = GenericParser::<bool>::parse(&parser, "true");
    println!("Parsed bool: {:?}", b);

    let f: Result<f64, _> = GenericParser::<f64>::parse(&parser, "3.14");
    println!("Parsed f64: {:?}", f);

    // Alternative: type annotation on the binding
    let num2: Result<i32, _> = parser.parse("100");
    println!("Parsed via type annotation: {:?}", num2);

    // Error cases
    let err: Result<i32, _> = parser.parse("not a number");
    println!("Parse error: {:?}", err);

    println!("\nKey insight: One type (SimpleParser) implements the trait");
    println!("multiple times for different Output types!");
}
