// Pattern 1: Type Conversion Cheatsheet
// Demonstrates From, Into, TryFrom, TryInto, AsRef, AsMut patterns.

use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};

// ============================================================================
// Example: Implementing From (Into comes free)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct UserId(u64);

#[derive(Debug, Clone)]
struct DatabaseId(u64);

impl From<DatabaseId> for UserId {
    fn from(db_id: DatabaseId) -> Self {
        UserId(db_id.0)
    }
}

// ============================================================================
// Example: Implementing TryFrom for validated conversions
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct Port(u16);

impl TryFrom<u32> for Port {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value <= 65535 {
            Ok(Port(value as u16))
        } else {
            Err("Port number exceeds maximum (65535)")
        }
    }
}

fn parse_port(input: u32) -> Result<Port, &'static str> {
    Port::try_from(input)
}

// ============================================================================
// Example: AsRef for flexible APIs
// ============================================================================

fn log_message<S: AsRef<str>>(msg: S) {
    println!("[LOG] {}", msg.as_ref());
}

// ============================================================================
// Example: AsMut for generic mutation
// ============================================================================

fn clear_buffer<T: AsMut<[u8]>>(buffer: &mut T) {
    for byte in buffer.as_mut() {
        *byte = 0;
    }
}

// ============================================================================
// Example: String Conversions
// ============================================================================

fn string_conversion_examples() {
    // &str -> String (allocation required)
    let owned1: String = "borrowed".to_string();
    let owned2: String = "borrowed".to_owned();
    let owned3: String = String::from("borrowed");
    let owned4: String = "borrowed".into();

    println!("String conversions: {}, {}, {}, {}", owned1, owned2, owned3, owned4);

    // String -> &str (cheap borrow)
    let s = String::from("hello");
    let borrowed1: &str = &s;
    let borrowed2: &str = s.as_str();

    println!("Borrowed: {}, {}", borrowed1, borrowed2);
}

// ============================================================================
// Example: Cow for zero-copy when possible
// ============================================================================

fn maybe_uppercase(s: &str, should_uppercase: bool) -> Cow<str> {
    if should_uppercase {
        Cow::Owned(s.to_uppercase())
    } else {
        Cow::Borrowed(s)
    }
}

// ============================================================================
// Example: Numeric Conversions
// ============================================================================

fn numeric_conversion_examples() {
    // Widening: use From/Into (infallible)
    let x: u8 = 255;
    let y: u32 = x.into();
    let z: u32 = u32::from(x);
    println!("Widening: {} -> {} or {}", x, y, z);

    // Narrowing: use TryFrom/TryInto (fallible)
    let big: u32 = 300;
    let small: Result<u8, _> = big.try_into();
    println!("Narrowing 300 to u8: {:?}", small);

    // Lossy: use as for explicit truncation
    let truncated = big as u8;
    println!("Truncated 300 as u8: {}", truncated);

    // Floating point conversions
    let precise: f64 = 3.14159;
    let rough = precise as f32;
    let integer = precise as i32;
    println!("f64 {} -> f32 {} -> i32 {}", precise, rough, integer);
}

// ============================================================================
// Example: Collection Conversions
// ============================================================================

fn collection_conversion_examples() {
    use std::collections::{BTreeSet, HashMap, HashSet};

    // Vec -> HashSet (deduplication)
    let numbers = vec![1, 2, 2, 3, 3, 3];
    let unique: HashSet<_> = numbers.into_iter().collect();
    println!("Deduplicated: {:?}", unique);

    // Vec<(K, V)> -> HashMap
    let pairs = vec![("a", 1), ("b", 2)];
    let map: HashMap<_, _> = pairs.into_iter().collect();
    println!("HashMap: {:?}", map);

    // HashSet -> BTreeSet (ordered)
    let hash_set: HashSet<_> = [3, 1, 2].iter().cloned().collect();
    let tree_set: BTreeSet<_> = hash_set.into_iter().collect();
    println!("BTreeSet (sorted): {:?}", tree_set);

    // Array -> Vec
    let array = [1, 2, 3, 4, 5];
    let vec = array.to_vec();
    println!("Array to Vec: {:?}", vec);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_into() {
        let db_id = DatabaseId(42);
        let user_id: UserId = db_id.into();
        assert_eq!(user_id, UserId(42));

        let user_id2 = UserId::from(DatabaseId(43));
        assert_eq!(user_id2, UserId(43));
    }

    #[test]
    fn test_try_from_valid() {
        let port = Port::try_from(8080u32);
        assert!(port.is_ok());
        assert_eq!(port.unwrap(), Port(8080));
    }

    #[test]
    fn test_try_from_invalid() {
        let port = Port::try_from(100000u32);
        assert!(port.is_err());
    }

    #[test]
    fn test_parse_port() {
        assert!(parse_port(80).is_ok());
        assert!(parse_port(70000).is_err());
    }

    #[test]
    fn test_as_ref() {
        // This compiles because AsRef<str> is flexible
        log_message("literal");
        log_message(String::from("owned"));
    }

    #[test]
    fn test_as_mut() {
        let mut vec_buffer = vec![1, 2, 3];
        let mut array_buffer = [4, 5, 6];

        clear_buffer(&mut vec_buffer);
        clear_buffer(&mut array_buffer);

        assert_eq!(vec_buffer, vec![0, 0, 0]);
        assert_eq!(array_buffer, [0, 0, 0]);
    }

    #[test]
    fn test_cow_borrowed() {
        let result = maybe_uppercase("hello", false);
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_cow_owned() {
        let result = maybe_uppercase("hello", true);
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_numeric_widening() {
        let x: u8 = 255;
        let y: u32 = x.into();
        assert_eq!(y, 255);
    }

    #[test]
    fn test_numeric_narrowing() {
        let big: u32 = 100;
        let small: Result<u8, _> = big.try_into();
        assert_eq!(small.unwrap(), 100);

        let too_big: u32 = 300;
        let result: Result<u8, _> = too_big.try_into();
        assert!(result.is_err());
    }
}

fn main() {
    println!("Pattern 1: Type Conversion Cheatsheet");
    println!("=====================================\n");

    // From/Into
    println!("From/Into conversions:");
    let db_id = DatabaseId(42);
    let user_id: UserId = db_id.into();
    println!("  DatabaseId(42) -> UserId({:?})", user_id);

    // TryFrom/TryInto
    println!("\nTryFrom/TryInto conversions:");
    println!("  Port::try_from(8080): {:?}", Port::try_from(8080u32));
    println!("  Port::try_from(100000): {:?}", Port::try_from(100000u32));

    // AsRef
    println!("\nAsRef for flexible APIs:");
    log_message("string literal");
    log_message(String::from("String value"));

    // AsMut
    println!("\nAsMut for generic mutation:");
    let mut buf = vec![1, 2, 3];
    println!("  Before clear_buffer: {:?}", buf);
    clear_buffer(&mut buf);
    println!("  After clear_buffer: {:?}", buf);

    // String conversions
    println!("\nString conversions:");
    string_conversion_examples();

    // Cow
    println!("\nCow (Clone-on-Write):");
    println!("  maybe_uppercase('hello', false): {:?}", maybe_uppercase("hello", false));
    println!("  maybe_uppercase('hello', true): {:?}", maybe_uppercase("hello", true));

    // Numeric conversions
    println!("\nNumeric conversions:");
    numeric_conversion_examples();

    // Collection conversions
    println!("\nCollection conversions:");
    collection_conversion_examples();
}
