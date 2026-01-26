//! Build script that compiles the C library and links it with Rust.
//!
//! This demonstrates how to integrate C code into a Rust project using the `cc` crate.

fn main() {
    // Tell Cargo to rerun this build script if the C source changes
    println!("cargo:rerun-if-changed=c_src/mylib.c");
    println!("cargo:rerun-if-changed=c_src/mylib.h");

    // Compile the C library
    cc::Build::new()
        .file("c_src/mylib.c")
        .include("c_src")
        .compile("mylib");

    // The cc crate automatically:
    // 1. Compiles mylib.c to mylib.a (static library)
    // 2. Tells Cargo to link against it
    // 3. Adds the library to the link search path
}
