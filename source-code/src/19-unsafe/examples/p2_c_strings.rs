// Pattern 2: Working with C Strings
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Converts a Rust string to a C-owned string.
/// Caller must free with free_rust_c_string().
fn rust_to_c_string(s: &str) -> *mut c_char {
    let c_string = CString::new(s).expect("CString::new failed");
    c_string.into_raw()  // Transfers ownership to caller
}

/// Converts a C string to a Rust String (copying data).
///
/// # Safety
/// - `c_str` must be a valid null-terminated C string
/// - The memory must remain valid for the duration of this call
unsafe fn c_to_rust_string(c_str: *const c_char) -> String {
    let c_str = CStr::from_ptr(c_str);
    c_str.to_string_lossy().into_owned()
}

/// Frees a C string created by rust_to_c_string().
///
/// # Safety
/// - `ptr` must have been created by rust_to_c_string()
/// - `ptr` must not be used after this call (use-after-free)
unsafe fn free_rust_c_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}

fn c_string_example() {
    let c_str = rust_to_c_string("Hello from Rust");

    unsafe {
        let rust_str = c_to_rust_string(c_str);
        println!("Back to Rust: {}", rust_str);
        free_rust_c_string(c_str);
    }
}

fn main() {
    c_string_example();

    // Usage: Convert Rust string to C
    let c_str = CString::new("hello").unwrap();
    println!("C string pointer: {:?}", c_str.as_ptr());

    println!("C strings example completed");
}
