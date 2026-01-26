//! Pattern 2: Associated Types vs Generics
//! Example: Combining Both - Generics for Input, Associated for Output
//!
//! Run with: cargo run --example p2_combining

// Generic Input (caller chooses), Associated Output (implementor fixes)
trait Converter<Input> {
    type Output;
    type Error;

    fn convert(&self, input: Input) -> Result<Self::Output, Self::Error>;
}

// Temperature converter: Celsius -> Fahrenheit
struct CelsiusToFahrenheit;

impl Converter<f64> for CelsiusToFahrenheit {
    type Output = f64;
    type Error = String;

    fn convert(&self, celsius: f64) -> Result<f64, String> {
        if celsius < -273.15 {
            Err("Temperature below absolute zero".to_string())
        } else {
            Ok(celsius * 9.0 / 5.0 + 32.0)
        }
    }
}

// Also accept i32 input
impl Converter<i32> for CelsiusToFahrenheit {
    type Output = f64;
    type Error = String;

    fn convert(&self, celsius: i32) -> Result<f64, String> {
        self.convert(celsius as f64)
    }
}

// String length converter
struct StringToLength;

impl Converter<&str> for StringToLength {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn convert(&self, input: &str) -> Result<usize, Self::Error> {
        Ok(input.len())
    }
}

impl Converter<String> for StringToLength {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn convert(&self, input: String) -> Result<usize, Self::Error> {
        Ok(input.len())
    }
}

fn main() {
    // Usage: Generic Input chosen by caller; Output fixed by implementation.
    let temp_conv = CelsiusToFahrenheit;

    // Can convert f64
    let f1 = temp_conv.convert(100.0_f64);
    println!("100°C = {:?}°F", f1);

    // Can also convert i32 (different Input type)
    let f2 = temp_conv.convert(0_i32);
    println!("0°C = {:?}°F", f2);

    // Error case
    let err = temp_conv.convert(-300.0_f64);
    println!("-300°C = {:?}", err);

    println!("\n=== String to Length ===");
    let len_conv = StringToLength;

    // Works with &str
    let len1 = len_conv.convert("hello");
    println!("'hello' length: {:?}", len1);

    // Works with String
    let len2 = len_conv.convert(String::from("hello world"));
    println!("'hello world' length: {:?}", len2);

    println!("\n=== Design Pattern ===");
    println!("- Generic<Input>: Multiple input types possible");
    println!("- type Output: Fixed by implementor, inferred by compiler");
    println!("- type Error: Fixed by implementor, enables Result ergonomics");
}
