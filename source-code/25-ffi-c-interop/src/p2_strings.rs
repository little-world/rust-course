//! Pattern 2: String Conversions
//!
//! Demonstrates CString/CStr for converting between Rust strings and C strings.
//! Shows ownership rules and proper memory management across FFI boundaries.

use std::ffi::{CString, CStr};
use std::os::raw::c_char;

extern "C" {
    fn c_string_length(s: *const c_char) -> usize;
    fn c_string_concat(a: *const c_char, b: *const c_char) -> *mut c_char;
    fn c_string_free(s: *mut c_char);
    fn c_print_message(msg: *const c_char);
}

fn main() {
    println!("=== Pattern 2: String Conversions ===\n");

    // --- Rust String to C String ---
    println!("--- Rust to C String Conversion ---");

    let rust_string = "Hello from Rust!";
    println!("Original Rust string: \"{}\"", rust_string);

    // CString::new() adds null terminator and validates no internal nulls
    let c_string = CString::new(rust_string).expect("CString::new failed");

    unsafe {
        // Get raw pointer for C
        let ptr = c_string.as_ptr();

        // Pass to C function
        let len = c_string_length(ptr);
        println!("C reports length: {} bytes", len);

        // C can print the message
        c_print_message(ptr);
    }
    // c_string is automatically freed here when it goes out of scope

    // --- C String to Rust String ---
    println!("\n--- C to Rust String Conversion ---");

    let part1 = CString::new("Hello, ").unwrap();
    let part2 = CString::new("World!").unwrap();

    unsafe {
        // C allocates and returns a new string
        let concatenated = c_string_concat(part1.as_ptr(), part2.as_ptr());

        if !concatenated.is_null() {
            // Borrow the C string without taking ownership
            let c_str = CStr::from_ptr(concatenated);

            // Convert to Rust String (handles UTF-8)
            let rust_result = c_str.to_string_lossy();
            println!("Concatenated result: \"{}\"", rust_result);

            // IMPORTANT: Free the C-allocated string
            c_string_free(concatenated);
            println!("C string freed successfully");
        }
    }

    // --- Demonstrating CString Failure ---
    println!("\n--- CString with Embedded Null ---");

    let string_with_null = "Hello\0World";
    match CString::new(string_with_null) {
        Ok(_) => println!("Unexpected success!"),
        Err(e) => println!("CString::new failed (expected): {}", e),
    }

    // Workaround: filter out null bytes
    let filtered: String = string_with_null.chars()
        .filter(|&c| c != '\0')
        .collect();
    let c_filtered = CString::new(filtered).unwrap();
    println!("Filtered string: \"{}\"", c_filtered.to_str().unwrap());

    // --- String Ownership Transfer ---
    println!("\n--- Ownership Transfer to C ---");

    // Create string that will be owned by C
    let owned = CString::new("Owned by C").unwrap();

    // Transfer ownership - C is now responsible for freeing
    let raw_ptr = owned.into_raw();
    println!("Transferred ownership to C (ptr: {:?})", raw_ptr);

    unsafe {
        // C uses the string
        c_print_message(raw_ptr);

        // Take ownership back before program ends
        let reclaimed = CString::from_raw(raw_ptr);
        println!("Reclaimed ownership, will auto-free");
        // reclaimed is dropped here, freeing the memory
    }

    // --- Safe Wrapper Pattern ---
    println!("\n--- Safe Wrapper Pattern ---");

    fn safe_print(message: &str) -> Result<(), &'static str> {
        let c_str = CString::new(message)
            .map_err(|_| "String contains null byte")?;

        unsafe {
            c_print_message(c_str.as_ptr());
        }
        Ok(())
    }

    safe_print("This is a safe wrapper!").unwrap();
    println!("Safe wrapper pattern demonstrated");

    println!("\nAll string conversion examples completed successfully!");
}
