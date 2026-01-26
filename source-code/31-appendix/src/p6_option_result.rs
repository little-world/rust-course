// Pattern 6: Option and Result - Error Handling
// Demonstrates Option<T>, Result<T, E>, combinators, and error propagation.

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use thiserror::Error as ThisError;

// ============================================================================
// Example: Option<T> - Handling Optional Values
// ============================================================================

fn option_basics() {
    // Creating Option values
    let some_value: Option<i32> = Some(42);
    let no_value: Option<i32> = None;

    println!("Some: {:?}, None: {:?}", some_value, no_value);

    // Pattern matching
    match some_value {
        Some(x) => println!("Value: {}", x),
        None => println!("No value"),
    }

    // if let (single pattern)
    if let Some(x) = some_value {
        println!("if let Some: {}", x);
    }
}

fn expensive_default() -> i32 {
    println!("  Computing expensive default...");
    999
}

fn option_unwrapping() {
    let some_value: Option<i32> = Some(42);
    let no_value: Option<i32> = None;

    // Unwrapping methods
    let _value = some_value.unwrap(); // Panics if None
    let _value = some_value.expect("No value"); // Panics with message
    let value = no_value.unwrap_or(0);
    println!("unwrap_or(0): {}", value);

    let value = no_value.unwrap_or_else(expensive_default);
    println!("unwrap_or_else: {}", value);

    let value: i32 = no_value.unwrap_or_default();
    println!("unwrap_or_default: {}", value);
}

fn option_checking() {
    let some_value: Option<i32> = Some(42);
    let no_value: Option<i32> = None;

    println!("is_some: {}, is_none: {}", some_value.is_some(), no_value.is_none());
}

fn option_transforming() {
    let some_value: Option<i32> = Some(42);
    let no_value: Option<i32> = None;

    // map
    let doubled = some_value.map(|x| x * 2);
    let none_doubled = no_value.map(|x| x * 2);
    println!("map: {:?}, {:?}", doubled, none_doubled);

    // and_then for chaining
    fn divide(a: i32, b: i32) -> Option<i32> {
        if b == 0 {
            None
        } else {
            Some(a / b)
        }
    }

    let result = Some(10).and_then(|x| divide(x, 2)).and_then(|x| divide(x, 0));
    println!("Chained divide: {:?}", result);

    // filter
    let value = Some(42);
    let filtered_fail = value.filter(|&x| x > 50);
    let filtered_pass = value.filter(|&x| x > 30);
    println!("filter (>50): {:?}, filter (>30): {:?}", filtered_fail, filtered_pass);
}

fn option_conversions() {
    // Option to Result
    let opt: Option<i32> = Some(42);
    let result: Result<i32, &str> = opt.ok_or("No value");
    println!("Option to Result: {:?}", result);

    // Result to Option
    let result: Result<i32, &str> = Err("Error");
    let opt: Option<i32> = result.ok();
    println!("Result to Option: {:?}", opt);
}

fn option_borrowing() {
    let value = Some(String::from("hello"));

    // Borrow inner value with as_ref
    let borrowed: Option<&String> = value.as_ref();
    let length: Option<usize> = value.as_ref().map(|s| s.len());
    println!("Borrowed: {:?}, length: {:?}", borrowed, length);

    // take and replace
    let mut value = Some(42);
    let taken = value.take();
    println!("Taken: {:?}, remaining: {:?}", taken, value);

    let mut value = Some(42);
    let old = value.replace(100);
    println!("Replaced: old={:?}, new={:?}", old, value);
}

fn option_zip() {
    let a = Some(1);
    let b = Some("hello");
    let c: Option<i32> = None;

    let zipped = a.zip(b);
    let zipped_none = a.zip(c);
    println!("zip: {:?}, zip with None: {:?}", zipped, zipped_none);
}

// ============================================================================
// Example: Result<T, E> - Handling Errors
// ============================================================================

fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

fn result_basics() {
    // Pattern matching
    match divide(10, 2) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }

    // Checking status
    println!("10/2 is_ok: {}", divide(10, 2).is_ok());
    println!("10/0 is_err: {}", divide(10, 0).is_err());
}

fn result_unwrapping() {
    // Providing defaults
    let value = divide(10, 0).unwrap_or(0);
    println!("unwrap_or(0): {}", value);

    let value = divide(10, 0).unwrap_or_else(|_| expensive_default());
    println!("unwrap_or_else: {}", value);

    let value: i32 = divide(10, 0).unwrap_or_default();
    println!("unwrap_or_default: {}", value);
}

fn result_transforming() {
    // map
    let doubled = divide(10, 2).map(|x| x * 2);
    println!("map: {:?}", doubled);

    // map_err
    let err_mapped = divide(10, 0).map_err(|e| format!("Fatal: {}", e));
    println!("map_err: {:?}", err_mapped);

    // and_then for chaining
    let result = divide(10, 2)
        .and_then(|x| divide(x, 2))
        .and_then(|x| divide(x, 0));
    println!("Chained divide: {:?}", result);
}

fn result_conversion_to_option() {
    let result: Result<i32, String> = Ok(42);
    let opt: Option<i32> = result.ok();
    println!("Result::ok(): {:?}", opt);

    let result: Result<i32, String> = Err("error".to_string());
    let err_opt: Option<String> = result.err();
    println!("Result::err(): {:?}", err_opt);
}

// ============================================================================
// Example: The ? Operator
// ============================================================================

fn read_file_manual(path: &str) -> Result<String, io::Error> {
    let f = File::open(path);
    let mut f = match f {
        Ok(file) => file,
        Err(e) => return Err(e),
    };

    let mut s = String::new();
    match f.read_to_string(&mut s) {
        Ok(_) => Ok(s),
        Err(e) => Err(e),
    }
}

fn read_file_question_mark(path: &str) -> Result<String, io::Error> {
    let mut f = File::open(path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

fn read_file_chained(path: &str) -> Result<String, io::Error> {
    let mut s = String::new();
    File::open(path)?.read_to_string(&mut s)?;
    Ok(s)
}

fn demonstrate_question_mark() {
    // These will all fail with a file not found error
    let result1 = read_file_manual("/nonexistent");
    let result2 = read_file_question_mark("/nonexistent");
    let result3 = read_file_chained("/nonexistent");

    println!("Manual: {:?}", result1.err().map(|e| e.kind()));
    println!("? operator: {:?}", result2.err().map(|e| e.kind()));
    println!("Chained: {:?}", result3.err().map(|e| e.kind()));
}

// ============================================================================
// Example: Custom Error Types
// ============================================================================

#[derive(Debug, PartialEq)]
enum MathError {
    DivisionByZero,
    NegativeSquareRoot,
}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MathError::DivisionByZero => write!(f, "Division by zero"),
            MathError::NegativeSquareRoot => write!(f, "Square root of negative number"),
        }
    }
}

impl Error for MathError {}

fn safe_divide(a: f64, b: f64) -> Result<f64, MathError> {
    if b == 0.0 {
        Err(MathError::DivisionByZero)
    } else {
        Ok(a / b)
    }
}

fn safe_sqrt(x: f64) -> Result<f64, MathError> {
    if x < 0.0 {
        Err(MathError::NegativeSquareRoot)
    } else {
        Ok(x.sqrt())
    }
}

fn demonstrate_custom_error() {
    println!("safe_divide(10, 2): {:?}", safe_divide(10.0, 2.0));
    println!("safe_divide(10, 0): {:?}", safe_divide(10.0, 0.0));
    println!("safe_sqrt(16): {:?}", safe_sqrt(16.0));
    println!("safe_sqrt(-1): {:?}", safe_sqrt(-1.0));
}

// ============================================================================
// Example: Using thiserror
// ============================================================================

#[derive(ThisError, Debug)]
enum DataError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error at line {line}: {msg}")]
    Parse { line: usize, msg: String },

    #[error("Invalid format")]
    InvalidFormat,
}

fn demonstrate_thiserror() {
    let err = DataError::Parse {
        line: 42,
        msg: "unexpected token".to_string(),
    };
    println!("thiserror Display: {}", err);

    let err = DataError::InvalidFormat;
    println!("thiserror Display: {}", err);
}

// ============================================================================
// Example: Combinators Reference
// ============================================================================

fn combinators_demo() {
    let opt = Some(42);

    println!("\nOption combinators:");
    println!("  map:      {:?}", opt.map(|x| x * 2));
    println!("  and_then: {:?}", opt.and_then(|x| Some(x * 2)));
    println!("  or:       {:?}", opt.or(Some(0)));
    println!("  filter:   {:?}", opt.filter(|&x| x > 50));
    println!("  zip:      {:?}", opt.zip(Some(10)));

    let res: Result<i32, &str> = Ok(42);

    println!("\nResult combinators:");
    println!("  map:      {:?}", res.map(|x| x * 2));
    println!("  and_then: {:?}", res.and_then(|x| Ok(x * 2) as Result<i32, &str>));
    println!("  or:       {:?}", res.or(Ok(0) as Result<i32, &str>));
    println!("  map_err:  {:?}", res.map_err(|e| format!("Error: {}", e)));
}

// ============================================================================
// Example: Collecting Results
// ============================================================================

fn collecting_results() {
    // All must succeed
    let r1: Result<i32, &str> = Ok(1);
    let r2: Result<i32, &str> = Ok(2);
    let r3: Result<i32, &str> = Ok(3);

    let combined: Result<Vec<i32>, &str> = vec![r1, r2, r3].into_iter().collect();
    println!("All Ok: {:?}", combined);

    // One fails
    let r1: Result<i32, &str> = Ok(1);
    let r2: Result<i32, &str> = Err("failed");
    let r3: Result<i32, &str> = Ok(3);

    let combined: Result<Vec<i32>, &str> = vec![r1, r2, r3].into_iter().collect();
    println!("One Err: {:?}", combined);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_map() {
        assert_eq!(Some(42).map(|x| x * 2), Some(84));
        assert_eq!(None::<i32>.map(|x| x * 2), None);
    }

    #[test]
    fn test_option_and_then() {
        let result = Some(10)
            .and_then(|x| if x > 5 { Some(x * 2) } else { None });
        assert_eq!(result, Some(20));

        let result = Some(3)
            .and_then(|x| if x > 5 { Some(x * 2) } else { None });
        assert_eq!(result, None);
    }

    #[test]
    fn test_option_filter() {
        assert_eq!(Some(42).filter(|&x| x > 40), Some(42));
        assert_eq!(Some(42).filter(|&x| x > 50), None);
    }

    #[test]
    fn test_option_unwrap_or() {
        assert_eq!(Some(42).unwrap_or(0), 42);
        assert_eq!(None::<i32>.unwrap_or(0), 0);
    }

    #[test]
    fn test_option_zip() {
        assert_eq!(Some(1).zip(Some(2)), Some((1, 2)));
        assert_eq!(Some(1).zip(None::<i32>), None);
    }

    #[test]
    fn test_option_to_result() {
        let opt: Option<i32> = Some(42);
        let res: Result<i32, &str> = opt.ok_or("no value");
        assert_eq!(res, Ok(42));

        let opt: Option<i32> = None;
        let res: Result<i32, &str> = opt.ok_or("no value");
        assert_eq!(res, Err("no value"));
    }

    #[test]
    fn test_result_map() {
        assert_eq!(Ok::<_, &str>(42).map(|x| x * 2), Ok(84));
        assert_eq!(Err::<i32, _>("error").map(|x| x * 2), Err("error"));
    }

    #[test]
    fn test_result_map_err() {
        let res: Result<i32, &str> = Err("error");
        let mapped = res.map_err(|e| format!("wrapped: {}", e));
        assert_eq!(mapped, Err("wrapped: error".to_string()));
    }

    #[test]
    fn test_result_and_then() {
        let result = divide(10, 2).and_then(|x| divide(x, 2));
        assert_eq!(result, Ok(2));

        let result = divide(10, 2).and_then(|x| divide(x, 0));
        assert!(result.is_err());
    }

    #[test]
    fn test_result_unwrap_or() {
        assert_eq!(Ok::<_, &str>(42).unwrap_or(0), 42);
        assert_eq!(Err::<i32, _>("error").unwrap_or(0), 0);
    }

    #[test]
    fn test_result_ok_err() {
        assert_eq!(Ok::<i32, &str>(42).ok(), Some(42));
        assert_eq!(Err::<i32, &str>("error").ok(), None);
        assert_eq!(Ok::<i32, &str>(42).err(), None);
        assert_eq!(Err::<i32, &str>("error").err(), Some("error"));
    }

    #[test]
    fn test_divide() {
        assert_eq!(divide(10, 2), Ok(5));
        assert!(divide(10, 0).is_err());
    }

    #[test]
    fn test_safe_divide() {
        assert_eq!(safe_divide(10.0, 2.0), Ok(5.0));
        assert!(matches!(safe_divide(10.0, 0.0), Err(MathError::DivisionByZero)));
    }

    #[test]
    fn test_safe_sqrt() {
        assert_eq!(safe_sqrt(16.0), Ok(4.0));
        assert!(matches!(safe_sqrt(-1.0), Err(MathError::NegativeSquareRoot)));
    }

    #[test]
    fn test_collect_results_success() {
        let results: Vec<Result<i32, &str>> = vec![Ok(1), Ok(2), Ok(3)];
        let collected: Result<Vec<i32>, &str> = results.into_iter().collect();
        assert_eq!(collected, Ok(vec![1, 2, 3]));
    }

    #[test]
    fn test_collect_results_failure() {
        let results: Vec<Result<i32, &str>> = vec![Ok(1), Err("fail"), Ok(3)];
        let collected: Result<Vec<i32>, &str> = results.into_iter().collect();
        assert_eq!(collected, Err("fail"));
    }

    #[test]
    fn test_thiserror_display() {
        let err = DataError::Parse {
            line: 10,
            msg: "invalid".to_string(),
        };
        assert_eq!(format!("{}", err), "Parse error at line 10: invalid");
    }
}

fn main() {
    println!("Pattern 6: Option and Result - Error Handling");
    println!("==============================================\n");

    println!("=== Option Basics ===");
    option_basics();
    println!();

    println!("=== Option Unwrapping ===");
    option_unwrapping();
    println!();

    println!("=== Option Checking ===");
    option_checking();
    println!();

    println!("=== Option Transforming ===");
    option_transforming();
    println!();

    println!("=== Option Conversions ===");
    option_conversions();
    println!();

    println!("=== Option Borrowing ===");
    option_borrowing();
    println!();

    println!("=== Option Zip ===");
    option_zip();
    println!();

    println!("=== Result Basics ===");
    result_basics();
    println!();

    println!("=== Result Unwrapping ===");
    result_unwrapping();
    println!();

    println!("=== Result Transforming ===");
    result_transforming();
    println!();

    println!("=== Result to Option ===");
    result_conversion_to_option();
    println!();

    println!("=== The ? Operator ===");
    demonstrate_question_mark();
    println!();

    println!("=== Custom Error Types ===");
    demonstrate_custom_error();
    println!();

    println!("=== Using thiserror ===");
    demonstrate_thiserror();
    println!();

    println!("=== Combinators Demo ===");
    combinators_demo();
    println!();

    println!("=== Collecting Results ===");
    collecting_results();
}
