//! Pattern 2: Associated Types vs Generics
//! Example: Ergonomics Comparison
//!
//! Run with: cargo run --example p2_ergonomics

// ========================================
// Generic version - requires type parameter
// ========================================
trait GenericParser<Output> {
    fn parse(&self, input: &str) -> Result<Output, String>;
}

struct MultiParser;

impl GenericParser<i32> for MultiParser {
    fn parse(&self, input: &str) -> Result<i32, String> {
        input.trim().parse().map_err(|e| format!("{}", e))
    }
}

impl GenericParser<bool> for MultiParser {
    fn parse(&self, input: &str) -> Result<bool, String> {
        match input.trim() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err("invalid bool".to_string()),
        }
    }
}

// With generic parameter, function needs extra type parameter
fn use_generic_parser<T, P: GenericParser<T>>(parser: &P, input: &str) -> Result<T, String> {
    parser.parse(input)
}

// ========================================
// Associated type version - cleaner API
// ========================================
trait Parser {
    type Output;
    fn parse(&self, input: &str) -> Result<Self::Output, String>;
}

struct IntParser;
struct BoolParser;

impl Parser for IntParser {
    type Output = i32;
    fn parse(&self, input: &str) -> Result<i32, String> {
        input.trim().parse().map_err(|e| format!("{}", e))
    }
}

impl Parser for BoolParser {
    type Output = bool;
    fn parse(&self, input: &str) -> Result<bool, String> {
        match input.trim() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err("invalid bool".to_string()),
        }
    }
}

// With associated type, no extra type parameter needed
fn use_associated_parser<P: Parser>(parser: &P, input: &str) -> Result<P::Output, String> {
    parser.parse(input)
}

fn main() {
    println!("=== Generic Parser (verbose) ===");
    let multi = MultiParser;

    // Caller must specify T with turbofish or type annotation
    let num = use_generic_parser::<i32, _>(&multi, "42");
    println!("Generic i32: {:?}", num);

    let b: Result<bool, _> = use_generic_parser(&multi, "true");
    println!("Generic bool: {:?}", b);

    println!("\n=== Associated Type Parser (ergonomic) ===");

    // Type is inferred from parser - no turbofish needed!
    let num = use_associated_parser(&IntParser, "42");
    println!("Associated i32: {:?}", num);

    let b = use_associated_parser(&BoolParser, "true");
    println!("Associated bool: {:?}", b);

    println!("\n=== Summary ===");
    println!("Generic: Flexible but verbose - caller specifies output type");
    println!("Associated: Ergonomic - output type determined by implementor");
}
