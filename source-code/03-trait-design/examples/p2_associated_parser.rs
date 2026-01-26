//! Pattern 2: Associated Types vs Generics
//! Example: Associated Types - One Implementation
//!
//! Run with: cargo run --example p2_associated_parser

// With associated types: Only one implementation possible
trait Parser {
    type Output;
    fn parse(&self, input: &str) -> Result<Self::Output, String>;
}

struct IntParser;

impl Parser for IntParser {
    type Output = i32;

    fn parse(&self, input: &str) -> Result<Self::Output, String> {
        input.trim().parse().map_err(|e| format!("{}", e))
    }
}

struct FloatParser;

impl Parser for FloatParser {
    type Output = f64;

    fn parse(&self, input: &str) -> Result<Self::Output, String> {
        input.trim().parse().map_err(|e| format!("{}", e))
    }
}

struct BoolParser;

impl Parser for BoolParser {
    type Output = bool;

    fn parse(&self, input: &str) -> Result<Self::Output, String> {
        match input.trim() {
            "true" | "1" | "yes" => Ok(true),
            "false" | "0" | "no" => Ok(false),
            _ => Err("invalid bool".to_string()),
        }
    }
}

// Generic function using Parser trait - no need for extra type parameter
fn parse_and_print<P: Parser>(parser: &P, input: &str)
where
    P::Output: std::fmt::Debug,
{
    match parser.parse(input) {
        Ok(value) => println!("Parsed: {:?}", value),
        Err(e) => println!("Error: {}", e),
    }
}

fn main() {
    // Usage: Each type has exactly one Output; no turbofish needed.
    let int_parser = IntParser;
    let float_parser = FloatParser;
    let bool_parser = BoolParser;

    // Output type is inferred from the parser type
    let num = int_parser.parse("42"); // Output inferred as i32
    let flt = float_parser.parse("3.14"); // Output inferred as f64
    let b = bool_parser.parse("true"); // Output inferred as bool

    println!("Int result: {:?}", num);
    println!("Float result: {:?}", flt);
    println!("Bool result: {:?}", b);

    // Using generic function - type inference works smoothly
    println!("\nUsing generic function:");
    parse_and_print(&int_parser, "100");
    parse_and_print(&float_parser, "2.718");
    parse_and_print(&bool_parser, "yes");

    println!("\nKey insight: Each parser type has exactly ONE Output type.");
    println!("No turbofish needed - the compiler infers everything!");
}
