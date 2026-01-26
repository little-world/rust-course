//! Pattern 4: Associated Types vs Generic Parameters
//! Example: Generic Parameter - Multiple Impls Per Type
//!
//! Run with: cargo run --example p4_generic_params

// Generic parameters allow one type to implement the trait multiple times
trait Convertible<T> {
    fn convert(&self) -> T;
}

impl Convertible<String> for i32 {
    fn convert(&self) -> String {
        self.to_string()
    }
}

impl Convertible<f64> for i32 {
    fn convert(&self) -> f64 {
        *self as f64
    }
}

impl Convertible<bool> for i32 {
    fn convert(&self) -> bool {
        *self != 0
    }
}

// Another example: Parseable from different types
trait Parseable<From> {
    fn parse_from(from: From) -> Option<Self>
    where
        Self: Sized;
}

impl Parseable<&str> for i32 {
    fn parse_from(from: &str) -> Option<Self> {
        from.parse().ok()
    }
}

impl Parseable<f64> for i32 {
    fn parse_from(from: f64) -> Option<Self> {
        Some(from as i32)
    }
}

impl Parseable<bool> for i32 {
    fn parse_from(from: bool) -> Option<Self> {
        Some(if from { 1 } else { 0 })
    }
}

// Combining with functions
fn convert_to_string<T: Convertible<String>>(value: &T) -> String {
    value.convert()
}

fn convert_to_f64<T: Convertible<f64>>(value: &T) -> f64 {
    value.convert()
}

fn main() {
    println!("=== Convertible Trait (Multiple Impls Per Type) ===");
    // Usage: Same type converts to multiple targets; turbofish selects which.
    let n: i32 = 42;

    let s: String = Convertible::<String>::convert(&n);
    println!("42.convert::<String>() = \"{}\"", s);

    let f: f64 = Convertible::<f64>::convert(&n);
    println!("42.convert::<f64>() = {}", f);

    let b: bool = Convertible::<bool>::convert(&n);
    println!("42.convert::<bool>() = {}", b);

    let zero: i32 = 0;
    let b_zero: bool = Convertible::<bool>::convert(&zero);
    println!("0.convert::<bool>() = {}", b_zero);

    println!("\n=== Parseable Trait (Multiple From Types) ===");
    let from_str: Option<i32> = Parseable::parse_from("123");
    println!("i32::parse_from(\"123\") = {:?}", from_str);

    let from_float: Option<i32> = Parseable::parse_from(45.67);
    println!("i32::parse_from(45.67) = {:?}", from_float);

    let from_bool: Option<i32> = Parseable::parse_from(true);
    println!("i32::parse_from(true) = {:?}", from_bool);

    let invalid: Option<i32> = Parseable::parse_from("not a number");
    println!("i32::parse_from(\"not a number\") = {:?}", invalid);

    println!("\n=== Using Generic Bounds ===");
    let value = 100_i32;
    println!("convert_to_string(&100) = \"{}\"", convert_to_string(&value));
    println!("convert_to_f64(&100) = {}", convert_to_f64(&value));

    println!("\n=== Associated Type vs Generic Parameter ===");
    println!("Generic parameters: MULTIPLE implementations per type");
    println!("  - i32 implements Convertible<String>, Convertible<f64>, Convertible<bool>");
    println!("  - From<T>, Into<T>, Add<Rhs> use generic parameters");
    println!("  - User chooses the type at call site with turbofish");
}
