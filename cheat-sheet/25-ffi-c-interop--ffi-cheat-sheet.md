
### FFI Cheat Sheet
```rust
// ===== FFI BASICS =====
// Calling C functions from Rust

// Declare external C functions
extern "C" {
    fn abs(input: i32) -> i32;                      // Standard C library
    fn strlen(s: *const i8) -> usize;
    fn malloc(size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
}

// Call C functions (unsafe)
unsafe {
    let result = abs(-5);                           // 5
    println!("abs(-5) = {}", result);
}

// ===== EXPOSING RUST TO C =====
// Make Rust functions callable from C

#[no_mangle]                                        // Prevent name mangling
pub extern "C" fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern "C" fn rust_strlen(s: *const i8) -> usize {
    unsafe {
        let mut len = 0;
        let mut ptr = s;
        while *ptr != 0 {
            len += 1;
            ptr = ptr.offset(1);
        }
        len
    }
}

// ===== REPR ATTRIBUTE =====
// Control memory layout for C compatibility

#[repr(C)]                                          // C-compatible layout
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[repr(C)]
enum Status {
    Success = 0,
    Error = 1,
    Pending = 2,
}

#[repr(C)]
union Data {
    i: i32,
    f: f32,
}

// Packed structs (no padding)
#[repr(C, packed)]
struct PackedStruct {
    a: u8,
    b: u32,                                         // No padding between fields
}

// Align structs
#[repr(C, align(16))]
struct AlignedStruct {
    data: [u8; 16],
}

// ===== RAW POINTERS =====
// Working with C pointers

// Raw pointer types
let ptr: *const i32;                                // Immutable raw pointer
let ptr: *mut i32;                                  // Mutable raw pointer

// Create raw pointers
let x = 5;
let ptr = &x as *const i32;                        // From reference
let ptr = &x as *const i32 as *mut i32;            // Cast to mutable

// Dereference raw pointers (unsafe)
unsafe {
    let value = *ptr;                               // Read value
    *ptr = 10;                                      // Write value (mutable ptr)
}

// Pointer arithmetic
unsafe {
    let ptr = ptr.offset(1);                        // Move forward
    let ptr = ptr.offset(-1);                       // Move backward
    let ptr = ptr.add(5);                          // Add offset
    let ptr = ptr.sub(2);                          // Subtract offset
}

// Null pointers
use std::ptr;
let null_ptr: *const i32 = ptr::null();            // Null immutable
let null_ptr: *mut i32 = ptr::null_mut();          // Null mutable

if ptr.is_null() {
    println!("Pointer is null");
}

// ===== C STRINGS =====
use std::ffi::{CStr, CString};

// Rust String to C string
let rust_str = "hello";
let c_string = CString::new(rust_str).unwrap();    // CString (owned, null-terminated)
let c_ptr = c_string.as_ptr();                     // *const c_char

// C string to Rust String
unsafe {
    let c_str = CStr::from_ptr(c_ptr);             // &CStr (borrowed)
    let rust_str = c_str.to_str().unwrap();        // &str
    let owned = c_str.to_string_lossy();           // Cow<str> (lossy conversion)
}

// Create from bytes
let bytes = b"hello\0";
let c_str = unsafe { CStr::from_bytes_with_nul_unchecked(bytes) };

// Get as bytes
let bytes = c_str.to_bytes();                      // Without null terminator
let bytes = c_str.to_bytes_with_nul();            // With null terminator

// ===== OPAQUE TYPES =====
// Working with opaque C types

// Define opaque type
#[repr(C)]
pub struct OpaqueHandle {
    _private: [u8; 0],                             // Zero-sized, prevents construction
}

extern "C" {
    fn create_handle() -> *mut OpaqueHandle;
    fn use_handle(handle: *mut OpaqueHandle);
    fn destroy_handle(handle: *mut OpaqueHandle);
}

// Safe wrapper
pub struct Handle(*mut OpaqueHandle);

impl Handle {
    pub fn new() -> Self {
        unsafe { Handle(create_handle()) }
    }
    
    pub fn use_it(&self) {
        unsafe { use_handle(self.0) }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { destroy_handle(self.0) }
    }
}

// ===== CALLBACKS =====
// Passing Rust functions to C

// C callback type
type Callback = extern "C" fn(i32) -> i32;

extern "C" {
    fn register_callback(cb: Callback);
    fn call_callback(cb: Callback, value: i32) -> i32;
}

// Rust callback function
extern "C" fn my_callback(x: i32) -> i32 {
    x * 2
}

// Register callback
unsafe {
    register_callback(my_callback);
    let result = call_callback(my_callback, 5);
}

// Closure as callback (requires Box)
extern "C" fn trampoline<F>(data: *mut std::ffi::c_void, value: i32) -> i32
where
    F: FnMut(i32) -> i32,
{
    let closure: &mut F = unsafe { &mut *(data as *mut F) };
    closure(value)
}

// ===== VARIADICS =====
// Calling variadic C functions

extern "C" {
    fn printf(format: *const i8, ...) -> i32;
}

// Call variadic function
unsafe {
    let format = CString::new("Hello %s, number: %d\n").unwrap();
    let name = CString::new("World").unwrap();
    printf(format.as_ptr(), name.as_ptr(), 42);
}

// ===== LINKING =====
// Link to C libraries

// Link to system library
#[link(name = "m")]                                 // Link to libm (math library)
extern "C" {
    fn sqrt(x: f64) -> f64;
    fn sin(x: f64) -> f64;
    fn cos(x: f64) -> f64;
}

// Link to custom library
#[link(name = "mylib", kind = "static")]           // Static library
extern "C" {
    fn my_function();
}

#[link(name = "mylib", kind = "dylib")]            // Dynamic library
extern "C" {
    fn another_function();
}

// Specify search path in build.rs
/*
fn main() {
    println!("cargo:rustc-link-search=/path/to/libs");
    println!("cargo:rustc-link-lib=mylib");
}
*/

// ===== MEMORY MANAGEMENT =====
// Manual memory management for C interop

// Allocate from C
unsafe {
    let ptr = malloc(100);                          // Allocate 100 bytes
    if !ptr.is_null() {
        // Use ptr
        free(ptr);                                  // Free memory
    }
}

// Pass Rust-allocated memory to C
let mut vec = vec![1, 2, 3, 4, 5];
let ptr = vec.as_mut_ptr();
let len = vec.len();
unsafe {
    // Pass ptr and len to C function
    c_function(ptr, len);
}
std::mem::forget(vec);                             // Prevent Rust from freeing

// Convert Box to raw pointer
let boxed = Box::new(42);
let ptr = Box::into_raw(boxed);                    // Transfer ownership
unsafe {
    let value = *ptr;
    let boxed = Box::from_raw(ptr);                // Reclaim ownership
}

// ===== ARRAYS AND SLICES =====
// Passing arrays to C

#[no_mangle]
pub extern "C" fn process_array(arr: *const i32, len: usize) -> i32 {
    unsafe {
        let slice = std::slice::from_raw_parts(arr, len);
        slice.iter().sum()
    }
}

// Receive array from C
extern "C" {
    fn get_array(out: *mut i32, len: usize);
}

let mut buffer = vec![0i32; 10];
unsafe {
    get_array(buffer.as_mut_ptr(), buffer.len());
}

// ===== STRUCTS WITH CALLBACKS =====
#[repr(C)]
struct Callbacks {
    on_event: extern "C" fn(*const i8),
    on_error: extern "C" fn(i32),
}

#[no_mangle]
pub extern "C" fn register_callbacks(callbacks: Callbacks) {
    // Store and use callbacks
}

// ===== ERRNO AND ERROR HANDLING =====
use std::io;

extern "C" {
    fn open(path: *const i8, flags: i32) -> i32;
    #[link_name = "__errno_location"]              // Linux
    fn errno_location() -> *mut i32;
}

fn get_errno() -> i32 {
    unsafe { *errno_location() }
}

fn open_file(path: &str) -> io::Result<i32> {
    let c_path = CString::new(path).unwrap();
    let fd = unsafe { open(c_path.as_ptr(), 0) };
    
    if fd < 0 {
        Err(io::Error::from_raw_os_error(get_errno()))
    } else {
        Ok(fd)
    }
}

// ===== BINDGEN =====
// Generate Rust bindings from C headers
// build.rs:
/*
use bindgen;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    
    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");
}
*/

// Include generated bindings
// mod bindings;
// use bindings::*;

// ===== CC CRATE =====
// Compile C code with Rust
// build.rs:
/*
fn main() {
    cc::Build::new()
        .file("src/helper.c")
        .compile("helper");
}
*/

// ===== PLATFORM-SPECIFIC CODE =====
#[cfg(unix)]
extern "C" {
    fn unix_specific_function() -> i32;
}

#[cfg(windows)]
extern "C" {
    fn windows_specific_function() -> i32;
}

// ===== COMMON PATTERNS =====

// Pattern 1: Safe wrapper for C library
pub struct SafeHandle {
    handle: *mut OpaqueHandle,
}

impl SafeHandle {
    pub fn new() -> Result<Self, String> {
        let handle = unsafe { create_handle() };
        if handle.is_null() {
            Err("Failed to create handle".to_string())
        } else {
            Ok(SafeHandle { handle })
        }
    }
    
    pub fn do_something(&self) -> i32 {
        unsafe { c_do_something(self.handle) }
    }
}

impl Drop for SafeHandle {
    fn drop(&mut self) {
        unsafe { destroy_handle(self.handle) }
    }
}

unsafe impl Send for SafeHandle {}
unsafe impl Sync for SafeHandle {}

// Pattern 2: String conversion helper
fn to_c_string(s: &str) -> Result<CString, std::ffi::NulError> {
    CString::new(s)
}

fn from_c_string(ptr: *const i8) -> String {
    unsafe {
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

// Pattern 3: Array passing
fn pass_slice_to_c(slice: &[i32]) {
    unsafe {
        c_process_array(slice.as_ptr(), slice.len());
    }
}

fn receive_array_from_c(len: usize) -> Vec<i32> {
    let mut buffer = vec![0; len];
    unsafe {
        c_fill_array(buffer.as_mut_ptr(), len);
    }
    buffer
}

// Pattern 4: Callback wrapper
struct CallbackWrapper<F> {
    closure: F,
}

impl<F> CallbackWrapper<F>
where
    F: FnMut(i32) -> i32,
{
    fn new(closure: F) -> Box<Self> {
        Box::new(CallbackWrapper { closure })
    }
    
    extern "C" fn trampoline(data: *mut std::ffi::c_void, value: i32) -> i32 {
        let wrapper: &mut CallbackWrapper<F> = unsafe { &mut *(data as *mut _) };
        (wrapper.closure)(value)
    }
}

// Pattern 5: Error conversion
fn convert_c_error(code: i32) -> Result<(), String> {
    match code {
        0 => Ok(()),
        -1 => Err("General error".to_string()),
        -2 => Err("Invalid argument".to_string()),
        _ => Err(format!("Unknown error: {}", code)),
    }
}

// Pattern 6: Reference counting for C
use std::sync::Arc;

#[no_mangle]
pub extern "C" fn create_shared_data() -> *mut std::ffi::c_void {
    let data = Arc::new(MyData::new());
    Arc::into_raw(data) as *mut std::ffi::c_void
}

#[no_mangle]
pub extern "C" fn clone_shared_data(ptr: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    let data = unsafe { Arc::from_raw(ptr as *const MyData) };
    let cloned = Arc::clone(&data);
    std::mem::forget(data);                         // Don't drop original
    Arc::into_raw(cloned) as *mut std::ffi::c_void
}

#[no_mangle]
pub extern "C" fn release_shared_data(ptr: *mut std::ffi::c_void) {
    unsafe {
        Arc::from_raw(ptr as *const MyData);       // Drop reference
    }
}

// ===== WORKING WITH C++ =====
// Use extern "C++" for C++ interop

extern "C++" {
    // Declare C++ functions with C linkage
    fn cpp_function(x: i32) -> i32;
}

// Or use CXX crate for safer C++ interop
/*
#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn rust_function(x: i32) -> i32;
    }
    
    unsafe extern "C++" {
        include!("mylib.h");
        fn cpp_function(x: i32) -> i32;
    }
}
*/

// ===== PANIC HANDLING =====
// Catch panics at FFI boundary

#[no_mangle]
pub extern "C" fn safe_rust_function(x: i32) -> i32 {
    std::panic::catch_unwind(|| {
        // Function body that might panic
        risky_operation(x)
    })
    .unwrap_or(-1)                                  // Return error code on panic
}

// ===== THREAD SAFETY =====
// Ensure thread safety across FFI

static INIT: std::sync::Once = std::sync::Once::new();

#[no_mangle]
pub extern "C" fn initialize() {
    INIT.call_once(|| {
        // One-time initialization
    });
}

// ===== EXAMPLE: COMPLETE C LIBRARY WRAPPER =====
mod sys {
    use std::ffi::c_void;
    
    #[repr(C)]
    pub struct CContext {
        _private: [u8; 0],
    }
    
    extern "C" {
        pub fn context_create() -> *mut CContext;
        pub fn context_destroy(ctx: *mut CContext);
        pub fn context_process(ctx: *mut CContext, data: *const u8, len: usize) -> i32;
    }
}

pub struct Context {
    ctx: *mut sys::CContext,
}

impl Context {
    pub fn new() -> Result<Self, String> {
        let ctx = unsafe { sys::context_create() };
        if ctx.is_null() {
            Err("Failed to create context".to_string())
        } else {
            Ok(Context { ctx })
        }
    }
    
    pub fn process(&self, data: &[u8]) -> Result<(), String> {
        let result = unsafe {
            sys::context_process(self.ctx, data.as_ptr(), data.len())
        };
        
        if result == 0 {
            Ok(())
        } else {
            Err(format!("Process failed with code: {}", result))
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { sys::context_destroy(self.ctx) }
    }
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}
```