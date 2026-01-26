//! Pattern 1: String Type Selection
//! Demonstrates String, &str, Cow, OsString, and Path types
//!
//! Run with: cargo run --example p1_string_types

use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

fn main() {
    println!("=== String Type Selection ===\n");

    // String - Owned and Growable
    println!("=== String - Owned and Growable ===\n");
    string_example();

    // &str - Borrowed String Slice
    println!("\n=== &str - Borrowed String Slice ===\n");
    str_slice_example("Hello, World!");

    // Cow - Clone on Write
    println!("\n=== Cow - Clone on Write ===\n");
    let s1 = cow_example("hello", false);
    let s2 = cow_example("hello", true);
    println!("cow_example(\"hello\", false): {:?}", s1);
    println!("cow_example(\"hello\", true):  {:?}", s2);

    // OsString/OsStr - Platform-Native Strings
    println!("\n=== OsString/OsStr - Platform-Native Strings ===\n");
    os_string_example();

    // Path/PathBuf - Cross-Platform File Paths
    println!("\n=== Path/PathBuf - Cross-Platform File Paths ===\n");
    path_example();

    // Type Conversions
    println!("\n=== Type Conversions ===\n");
    type_conversions();

    println!("\n=== Key Points ===");
    println!("1. String owns data, &str borrows");
    println!("2. Cow optimizes by borrowing when possible");
    println!("3. OsString handles platform-specific encodings");
    println!("4. Path provides platform-independent path operations");
}

fn string_example() {
    let mut s = String::from("Hello");
    s.push_str(", World!");
    println!("{}", s);

    // Use when:
    // - Need to own the string
    // - Building strings dynamically
    // - Returning strings from functions
}

fn str_slice_example(s: &str) {
    println!("Length: {}", s.len());

    // Use when:
    // - Read-only access needed
    // - Function parameters (most flexible)
    // - String literals
}

fn cow_example<'a>(data: &'a str, uppercase: bool) -> Cow<'a, str> {
    if uppercase {
        Cow::Owned(data.to_uppercase())  // Allocates
    } else {
        Cow::Borrowed(data)  // No allocation
    }
}

fn os_string_example() {
    use std::env;

    // Print first 3 environment variables
    for (i, (key, value)) in env::vars_os().take(3).enumerate() {
        println!("{}. {:?} = {:?}", i + 1, key, value);
    }

    // Use when:
    // - Dealing with file system
    // - Environment variables
    // - FFI with OS APIs
}

fn path_example() {
    let path = Path::new("/tmp/foo.txt");

    println!("Extension: {:?}", path.extension());
    println!("Parent: {:?}", path.parent());
    println!("File name: {:?}", path.file_name());

    // Building paths
    let mut path_buf = PathBuf::from("/tmp");
    path_buf.push("subdir");
    path_buf.push("file.txt");
    println!("Built path: {:?}", path_buf);

    // Use when:
    // - Working with file paths
    // - Cross-platform path manipulation
}

fn type_conversions() {
    // Demonstrate type conversions
    let string = String::from("Hello");
    let str_slice: &str = &string;  // String -> &str (deref coercion)
    let cow: Cow<str> = Cow::Borrowed(str_slice);

    // String from &str
    let owned: String = str_slice.to_string();

    // Path conversions
    let path = Path::new("file.txt");
    let os_str: &OsStr = path.as_os_str();

    println!("String: {}", string);
    println!("&str: {}", str_slice);
    println!("Cow: {:?}", cow);
    println!("Owned: {}", owned);
    println!("Path: {:?}", path);
    println!("OsStr: {:?}", os_str);
}
