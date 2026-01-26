// Pattern 1: Unit Test Patterns
// Demonstrates basic unit testing, assertion macros, error testing, panic testing,
// test organization, setup/teardown, ignoring tests, and testing private functions.

// ============================================================================
// Example: The Basics of Rust Testing
// ============================================================================

#[cfg(test)]
mod basic_tests {
    #[test]
    fn test_addition() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_subtraction() {
        let result = 10 - 3;
        assert_eq!(result, 7);
    }
}

// ============================================================================
// Example: Assertion Macros
// ============================================================================

#[cfg(test)]
mod assertion_tests {
    #[test]
    fn assertion_examples() {
        // Basic equality
        assert_eq!(5, 2 + 3);

        // Inequality
        assert_ne!(5, 6);

        // Boolean assertions
        assert!(true);
        assert!(5 > 3, "5 should be greater than 3");

        // Custom error messages
        let x = 10;
        assert_eq!(x, 10, "x should be 10, but was {}", x);
    }
}

// ============================================================================
// Example: Testing Error Cases
// ============================================================================

fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_divide_success() {
        assert_eq!(divide(10, 2), Ok(5));
    }

    #[test]
    fn test_divide_by_zero() {
        assert!(divide(10, 0).is_err());
    }

    #[test]
    fn test_divide_error_message() {
        match divide(10, 0) {
            Err(msg) => assert_eq!(msg, "Division by zero"),
            Ok(_) => panic!("Expected error, got Ok"),
        }
    }
}

// ============================================================================
// Example: Testing Panics
// ============================================================================

fn validate_age(age: u32) {
    if age > 150 {
        panic!("Age {} is unrealistic", age);
    }
}

#[cfg(test)]
mod panic_tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_invalid_age_panics() {
        validate_age(200);
    }

    #[test]
    #[should_panic(expected = "unrealistic")]
    fn test_panic_message() {
        validate_age(200);
    }
}

// ============================================================================
// Example: Organizing Tests
// ============================================================================

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

#[cfg(test)]
mod organized_tests {
    use super::*;

    mod addition_tests {
        use super::*;

        #[test]
        fn test_positive_numbers() {
            assert_eq!(add(2, 3), 5);
        }

        #[test]
        fn test_negative_numbers() {
            assert_eq!(add(-2, -3), -5);
        }

        #[test]
        fn test_mixed_signs() {
            assert_eq!(add(-2, 3), 1);
        }
    }

    mod multiplication_tests {
        use super::*;

        #[test]
        fn test_positive_numbers() {
            assert_eq!(multiply(2, 3), 6);
        }

        #[test]
        fn test_by_zero() {
            assert_eq!(multiply(5, 0), 0);
        }
    }
}

// ============================================================================
// Example: Test Setup and Teardown (RAII)
// ============================================================================

struct TestContext {
    temp_dir: std::path::PathBuf,
}

impl TestContext {
    fn new() -> Self {
        let temp_dir = std::env::temp_dir().join(format!("test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        TestContext { temp_dir }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Cleanup happens automatically when TestContext is dropped
        let _ = std::fs::remove_dir_all(&self.temp_dir);
    }
}

#[cfg(test)]
mod setup_teardown_tests {
    use super::*;

    #[test]
    fn test_with_temp_directory() {
        let ctx = TestContext::new();

        // Use ctx.temp_dir for testing
        let test_file = ctx.temp_dir.join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();

        assert!(test_file.exists());

        // ctx is dropped here, cleaning up temp_dir
    }
}

// ============================================================================
// Example: Ignoring and Filtering Tests
// ============================================================================

#[cfg(test)]
mod ignore_tests {
    #[test]
    #[ignore]
    fn expensive_test() {
        // This test takes a long time
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    #[test]
    #[ignore = "Not yet implemented"]
    fn todo_test() {
        // Placeholder for future implementation
    }
}

// ============================================================================
// Example: Testing Private Functions
// ============================================================================

fn internal_helper(x: i32) -> i32 {
    x * 2
}

pub fn public_api(x: i32) -> i32 {
    internal_helper(x) + 1
}

#[cfg(test)]
mod private_function_tests {
    use super::*;

    #[test]
    fn test_internal_helper() {
        // Can test private functions
        assert_eq!(internal_helper(5), 10);
    }

    #[test]
    fn test_public_api() {
        assert_eq!(public_api(5), 11);
    }
}

fn main() {
    println!("Unit test patterns - run with: cargo test");
    println!("Run ignored tests with: cargo test -- --ignored");
    println!("Filter tests by name: cargo test addition");
}
