//! Pattern 4: Error Handling Across FFI
//!
//! Demonstrates translating between C error conventions (return codes, errno)
//! and Rust's Result type. Shows panic safety with catch_unwind.

use std::os::raw::{c_int, c_char};
use std::ffi::CStr;
use std::panic;

// Error codes (matching C header)
const SUCCESS: c_int = 0;
const ERROR_NULL_POINTER: c_int = -1;
const ERROR_INVALID_INPUT: c_int = -2;
const ERROR_COMPUTATION_FAILED: c_int = -3;

extern "C" {
    fn c_divide(a: c_int, b: c_int, result: *mut c_int) -> c_int;
    fn c_error_message(error_code: c_int) -> *const c_char;
}

// ===========================================
// Safe Rust wrapper around C error handling
// ===========================================

#[derive(Debug)]
enum DivideError {
    NullPointer,
    DivisionByZero,
    Unknown(i32),
}

impl std::fmt::Display for DivideError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DivideError::NullPointer => write!(f, "Null pointer"),
            DivideError::DivisionByZero => write!(f, "Division by zero"),
            DivideError::Unknown(code) => write!(f, "Unknown error (code: {})", code),
        }
    }
}

fn safe_divide(a: i32, b: i32) -> Result<i32, DivideError> {
    let mut result: c_int = 0;

    let error_code = unsafe {
        c_divide(a, b, &mut result)
    };

    match error_code {
        SUCCESS => Ok(result),
        ERROR_NULL_POINTER => Err(DivideError::NullPointer),
        ERROR_INVALID_INPUT => Err(DivideError::DivisionByZero),
        code => Err(DivideError::Unknown(code)),
    }
}

fn get_c_error_message(code: c_int) -> String {
    unsafe {
        let ptr = c_error_message(code);
        if ptr.is_null() {
            return "Unknown error".to_string();
        }
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

// ===========================================
// Panic-safe FFI wrapper
// ===========================================

fn risky_computation(value: i32) -> i32 {
    if value < 0 {
        panic!("Negative value not allowed: {}", value);
    }
    if value > 100 {
        panic!("Value too large: {}", value);
    }
    value * 2
}

/// A function that would be called from C - must never panic!
#[no_mangle]
pub extern "C" fn safe_rust_compute(value: c_int) -> c_int {
    // Catch any panics before they cross FFI boundary
    let result = panic::catch_unwind(|| {
        risky_computation(value)
    });

    match result {
        Ok(v) => v,
        Err(_) => {
            eprintln!("  [Rust] Caught panic! Returning error code.");
            -1 // Error code
        }
    }
}

// ===========================================
// Thread-local error storage
// ===========================================

use std::cell::RefCell;

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

fn set_last_error(msg: String) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = Some(msg);
    });
}

fn get_last_error() -> Option<String> {
    LAST_ERROR.with(|e| {
        e.borrow().clone()
    })
}

fn clear_last_error() {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = None;
    });
}

fn computation_with_error_context(value: i32) -> c_int {
    clear_last_error();

    if value < 0 {
        set_last_error(format!("Invalid negative value: {}", value));
        return ERROR_INVALID_INPUT;
    }

    if value == 0 {
        set_last_error("Zero is not allowed".to_string());
        return ERROR_INVALID_INPUT;
    }

    // Success
    SUCCESS
}

fn main() {
    println!("=== Pattern 4: Error Handling Across FFI ===\n");

    // --- C Error Code Translation ---
    println!("--- C Error Code Translation ---");

    println!("Testing safe_divide wrapper:");
    let test_cases = [(10, 2), (15, 3), (10, 0), (100, 5)];

    for (a, b) in test_cases {
        match safe_divide(a, b) {
            Ok(result) => println!("  {} / {} = {}", a, b, result),
            Err(e) => println!("  {} / {} failed: {}", a, b, e),
        }
    }

    // --- Getting C Error Messages ---
    println!("\n--- C Error Messages ---");

    let error_codes = [SUCCESS, ERROR_NULL_POINTER, ERROR_INVALID_INPUT, ERROR_COMPUTATION_FAILED, -99];

    for code in error_codes {
        let msg = get_c_error_message(code);
        println!("  Error code {}: \"{}\"", code, msg);
    }

    // --- Panic Safety ---
    println!("\n--- Panic Safety (catch_unwind) ---");

    let test_values = [50, -10, 150, 25];

    for value in test_values {
        let result = safe_rust_compute(value);
        if result >= 0 {
            println!("  safe_rust_compute({}) = {}", value, result);
        } else {
            println!("  safe_rust_compute({}) returned error: {}", value, result);
        }
    }

    // --- Thread-Local Error Context ---
    println!("\n--- Thread-Local Error Context ---");

    let values = [42, -5, 0, 100];

    for value in values {
        let result = computation_with_error_context(value);

        if result == SUCCESS {
            println!("  computation_with_error_context({}) = SUCCESS", value);
        } else {
            let error_msg = get_last_error().unwrap_or_else(|| "No error message".to_string());
            println!("  computation_with_error_context({}) = {} ({})",
                value, result, error_msg);
        }
    }

    // --- Comprehensive Error Wrapper Example ---
    println!("\n--- Comprehensive Error Wrapper ---");

    /// Comprehensive wrapper that handles all error cases
    fn comprehensive_operation(a: i32, b: i32) -> Result<i32, String> {
        // Input validation
        if a < 0 || b < 0 {
            return Err(format!("Negative inputs not allowed: a={}, b={}", a, b));
        }

        // Call C with panic protection
        let result = panic::catch_unwind(|| {
            safe_divide(a, b)
        });

        match result {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(e)) => Err(format!("C function error: {}", e)),
            Err(_) => Err("Unexpected panic in computation".to_string()),
        }
    }

    let operations = [(20, 4), (10, 0), (-5, 2), (100, 10)];

    for (a, b) in operations {
        match comprehensive_operation(a, b) {
            Ok(result) => println!("  comprehensive_operation({}, {}) = {}", a, b, result),
            Err(msg) => println!("  comprehensive_operation({}, {}) failed: {}", a, b, msg),
        }
    }

    println!("\nAll error handling examples completed successfully!");
}
