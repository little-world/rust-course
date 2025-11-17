# 21. FFI & C Interop

Foreign Function Interface (FFI) is Rust's bridge to the vast ecosystem of existing C libraries. While Rust provides excellent safety guarantees, the real world is filled with battle-tested C libraries—database drivers, graphics libraries, operating system APIs, and embedded system interfaces. FFI allows you to leverage these libraries while maintaining Rust's safety where possible.

This integration comes with challenges. C and Rust have fundamentally different approaches to memory management, error handling, and type safety. Understanding these differences and how to bridge them safely is crucial for writing reliable FFI code.

## C ABI Compatibility

The Application Binary Interface (ABI) defines how functions are called at the machine level: how arguments are passed, how return values are handled, and how the stack is managed. For Rust and C to communicate, they must agree on these low-level details. This is where `extern "C"` and `#[repr(C)]` come in.

### Understanding the C ABI

C has a stable ABI that has remained largely consistent for decades. This stability is a double-edged sword—it enables interoperability but also constrains innovation. Rust, by contrast, doesn't guarantee ABI stability between versions. Rust can optimize struct layouts, change calling conventions, and rearrange data structures for performance. This is great for Rust-only code, but it means we need explicit markers when interfacing with C.

When you mark a Rust function with `extern "C"`, you're telling the compiler: "This function needs to follow C's calling convention." Similarly, `#[repr(C)]` tells Rust to lay out a struct exactly as C would, without any of Rust's optimizations.

Let's see this in practice:

```rust
//===============================================
// This struct uses Rust's default representation
//===============================================
// The compiler might reorder fields, add padding, or optimize layout
struct RustStruct {
    a: u8,
    b: u32,
    c: u16,
}

//==============================================
// This struct is guaranteed to match C's layout
//==============================================
// Fields appear in memory in the exact order declared
#[repr(C)]
struct CCompatibleStruct {
    a: u8,    // 1 byte, followed by 3 bytes of padding
    b: u32,   // 4 bytes, aligned to 4-byte boundary
    c: u16,   // 2 bytes
}

//=================
// Verify the sizes
//=================
fn demonstrate_repr() {
    println!("RustStruct size: {}", std::mem::size_of::<RustStruct>());
    println!("CCompatibleStruct size: {}", std::mem::size_of::<CCompatibleStruct>());

    // Both are likely 12 bytes, but only CCompatibleStruct guarantees
    // the exact layout C expects
}
```

The difference becomes critical when passing data to C libraries. If the layouts don't match exactly, you'll get corrupted data or crashes.

### Calling C Functions from Rust

To call a C function from Rust, you first declare it with `extern "C"`. This declaration acts as a promise to the compiler about what exists in the linked C library:

```rust
//=============================
// Declare external C functions
//=============================
extern "C" {
    // C: int abs(int n);
    fn abs(n: i32) -> i32;

    // C: void *malloc(size_t size);
    fn malloc(size: usize) -> *mut std::ffi::c_void;

    // C: void free(void *ptr);
    fn free(ptr: *mut std::ffi::c_void);

    // C: double sqrt(double x);
    fn sqrt(x: f64) -> f64;
}

fn use_c_functions() {
    unsafe {
        // All C function calls are unsafe
        // The Rust compiler can't verify C code's safety guarantees
        let result = abs(-42);
        println!("abs(-42) = {}", result);

        let root = sqrt(16.0);
        println!("sqrt(16.0) = {}", root);
    }
}
```

Notice that we must wrap C function calls in `unsafe` blocks. This is Rust's way of saying: "Beyond this point, the safety guarantees are up to you." C doesn't have Rust's borrow checker, null-safety, or bounds checking, so the compiler can't verify that the C code won't cause undefined behavior.

### Exposing Rust Functions to C

Sometimes you need to go the other direction—exposing Rust functions so C code can call them. This is common when embedding Rust in existing C applications or creating C-compatible libraries:

```rust
//================================
// A Rust function callable from C
//================================
#[no_mangle]  // Don't change the function name during compilation
pub extern "C" fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}

//=========================================
// Prevent name mangling for easier linking
//=========================================
#[no_mangle]
pub extern "C" fn rust_compute_average(values: *const f64, count: usize) -> f64 {
    // Safety: Caller must ensure:
    // 1. values is valid for count elements
    // 2. values is properly aligned
    // 3. values is not mutated during this call
    unsafe {
        if values.is_null() || count == 0 {
            return 0.0;
        }

        let slice = std::slice::from_raw_parts(values, count);
        let sum: f64 = slice.iter().sum();
        sum / count as f64
    }
}
```

The `#[no_mangle]` attribute is crucial. Normally, Rust "mangles" function names to encode type information, which helps with overloading but makes the names unreadable. C expects simple, predictable names like `rust_add`, not something like `_ZN4rust8rust_add17h3b3c3d3e3f3g3hE`.

### Type Mapping Between C and Rust

Understanding how C types map to Rust types is essential for correct FFI:

```rust
use std::os::raw::{c_char, c_int, c_long, c_ulong, c_void};

// C type          -> Rust equivalent
// char            -> c_char (usually i8 or u8, platform-dependent)
// int             -> c_int (usually i32)
// long            -> c_long (i32 on 32-bit, i64 on 64-bit)
// unsigned long   -> c_ulong
// void*           -> *mut c_void or *const c_void
// bool            -> bool (C99) or c_int (C89)
// size_t          -> usize
// float           -> f32
// double          -> f64

// Example: working with C types
extern "C" {
    fn process_data(
        buffer: *mut c_char,
        size: usize,
        flags: c_int,
    ) -> c_long;
}

fn use_c_types() {
    let mut buffer = vec![0u8; 100];

    unsafe {
        let result = process_data(
            buffer.as_mut_ptr() as *mut c_char,
            buffer.len(),
            42,
        );

        if result >= 0 {
            println!("Processed {} bytes", result);
        }
    }
}
```

The `std::os::raw` module provides platform-independent type aliases that match C's types on the current platform. Always use these rather than assuming `int` is `i32`—on some platforms, it might not be.

### Struct Padding and Alignment

C compilers insert padding between struct fields to satisfy alignment requirements. Rust does the same with `#[repr(C)]`, but understanding this is crucial for debugging layout issues:

```rust
#[repr(C)]
struct Padded {
    a: u8,    // 1 byte
    // 3 bytes padding
    b: u32,   // 4 bytes (must be 4-byte aligned)
    c: u8,    // 1 byte
    // 3 bytes padding (to make struct size multiple of largest alignment)
}

#[repr(C, packed)]
struct NoPadding {
    a: u8,    // 1 byte
    b: u32,   // 4 bytes (no padding, may be misaligned!)
    c: u8,    // 1 byte
}

fn examine_layout() {
    println!("Padded size: {}, align: {}",
        std::mem::size_of::<Padded>(),
        std::mem::align_of::<Padded>()
    );
    // Padded size: 12, align: 4

    println!("NoPadding size: {}, align: {}",
        std::mem::size_of::<NoPadding>(),
        std::mem::align_of::<NoPadding>()
    );
    // NoPadding size: 6, align: 1
}
```

The `packed` attribute removes padding, which can save space but may cause performance issues or crashes on architectures that don't support misaligned access. Only use it when interfacing with C code that explicitly uses packed structs.

## String Conversions

Strings are one of the trickiest aspects of FFI. C represents strings as null-terminated byte arrays (`char*`), while Rust uses length-prefixed UTF-8 (`String`). These fundamental differences require careful conversion to avoid crashes, memory leaks, and security vulnerabilities.

### The String Problem

Consider what happens when C and Rust exchange a string:

1. **Encoding**: C strings are byte arrays with no encoding guarantees. They might be ASCII, UTF-8, Latin-1, or any other encoding. Rust strings are always valid UTF-8.

2. **Termination**: C strings end with a null byte (`\0`). Rust strings store their length and can contain null bytes in the middle.

3. **Ownership**: C has manual memory management. Who owns a string pointer? Who's responsible for freeing it? These questions can cause memory leaks or double-frees.

Rust's standard library provides several types to bridge this gap, each suited for different scenarios.

### CString and CStr

`CString` and `CStr` are Rust's primary tools for C string interop. `CString` owns a null-terminated C string, while `CStr` borrows one:

```rust
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

//==========
// Rust to C
//==========
fn rust_string_to_c() {
    let rust_string = "Hello, C!";

    // Create a CString (allocates, adds null terminator)
    let c_string = CString::new(rust_string).expect("CString::new failed");

    // Get a pointer suitable for C
    let c_ptr: *const c_char = c_string.as_ptr();

    unsafe {
        // Pass to C function
        some_c_function(c_ptr);
    }

    // c_string is dropped here, freeing the memory
}

//==========
// C to Rust
//==========
unsafe fn c_string_to_rust(c_ptr: *const c_char) -> String {
    // Safety: Caller must ensure c_ptr is valid and null-terminated

    if c_ptr.is_null() {
        return String::new();
    }

    // Create a CStr (borrows the C string)
    let c_str = CStr::from_ptr(c_ptr);

    // Convert to Rust String
    // to_string_lossy replaces invalid UTF-8 with �
    c_str.to_string_lossy().into_owned()
}

extern "C" {
    fn some_c_function(s: *const c_char);
}
```

The key insight here is that `CString::new()` can fail. Why? Because Rust strings can contain null bytes, but C strings can't (null terminates the string). If you try to create a `CString` from a Rust string containing `\0`, you'll get an error:

```rust
use std::ffi::CString;

fn demonstrate_null_bytes() {
    let with_null = "Hello\0World";

    match CString::new(with_null) {
        Ok(_) => println!("Success"),
        Err(e) => println!("Failed: {}", e),
        // Output: Failed: nul byte found in provided data at position: 5
    }

    // If you need to handle this, strip null bytes first:
    let without_null: String = with_null.chars()
        .filter(|&c| c != '\0')
        .collect();

    let c_string = CString::new(without_null).unwrap();
}
```

### OsString and OsStr

While `CString` handles null-terminated strings, operating system paths present a different challenge. File paths aren't always valid UTF-8—Windows uses UTF-16, and Unix allows arbitrary bytes (except null). This is where `OsString` and `OsStr` come in:

```rust
use std::ffi::{OsString, OsStr};
use std::path::{Path, PathBuf};

fn working_with_os_strings() {
    // Creating an OsString
    let os_string = OsString::from("my_file.txt");

    // Converting between Path and OsStr
    let path = Path::new("/home/user/document.txt");
    let os_str: &OsStr = path.as_os_str();

    // Attempting UTF-8 conversion (may fail on Windows or Unix)
    match os_str.to_str() {
        Some(s) => println!("Valid UTF-8: {}", s),
        None => println!("Path contains invalid UTF-8"),
    }

    // Lossy conversion (replaces invalid UTF-8 with �)
    let string = os_str.to_string_lossy();
    println!("Path (lossy): {}", string);
}
```

`OsString` is particularly important when writing cross-platform code:

```rust
use std::ffi::{OsString, OsStr};

#[cfg(windows)]
fn platform_specific_path() -> OsString {
    use std::os::windows::ffi::OsStringExt;

    // Windows uses UTF-16
    let wide: Vec<u16> = vec![0x0048, 0x0065, 0x006C, 0x006C, 0x006F]; // "Hello"
    OsString::from_wide(&wide)
}

#[cfg(unix)]
fn platform_specific_path() -> OsString {
    use std::os::unix::ffi::OsStringExt;

    // Unix allows arbitrary bytes
    let bytes: Vec<u8> = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello"
    OsString::from_vec(bytes)
}
```

### String Ownership Across FFI

One of the most common bugs in FFI code is getting ownership wrong. Consider these scenarios:

```rust
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

//==============================
// CORRECT: Rust owns the string
//==============================
#[no_mangle]
pub extern "C" fn rust_creates_string() -> *mut c_char {
    let s = CString::new("Hello from Rust").unwrap();

    // Transfer ownership to C
    // C must call rust_free_string() when done
    s.into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn rust_free_string(s: *mut c_char) {
    if !s.is_null() {
        // Take ownership back and drop
        let _ = CString::from_raw(s);
    }
}

//===========================
// CORRECT: C owns the string
//===========================
#[no_mangle]
pub unsafe extern "C" fn rust_uses_c_string(s: *const c_char) {
    if s.is_null() {
        return;
    }

    // Borrow the string, don't take ownership
    let c_str = CStr::from_ptr(s);
    println!("C string: {}", c_str.to_string_lossy());

    // s is still valid here; C will free it
}

//==========================
// WRONG: This leaks memory!
//==========================
#[no_mangle]
pub extern "C" fn leaked_string() -> *const c_char {
    let s = CString::new("This will leak").unwrap();
    s.as_ptr() // s is dropped here, but pointer escapes!
}
```

The golden rule: **whoever allocates the memory must free it**. If Rust allocates, Rust must free (even if C holds the pointer temporarily). If C allocates, C must free.

### Practical Example: File Path Handling

Here's a complete example showing proper string handling across FFI boundaries:

```rust
use std::ffi::{CString, CStr, OsStr};
use std::os::raw::c_char;
use std::path::Path;

//=========================
// C API for opening a file
//=========================
extern "C" {
    fn fopen(filename: *const c_char, mode: *const c_char) -> *mut std::ffi::c_void;
    fn fclose(file: *mut std::ffi::c_void) -> i32;
}

/// Safe wrapper around C's fopen
fn open_file(path: &Path, mode: &str) -> Option<*mut std::ffi::c_void> {
    // Convert Path to CString
    // This can fail if the path contains null bytes
    let path_str = path.to_str()?;
    let c_path = CString::new(path_str).ok()?;

    let c_mode = CString::new(mode).ok()?;

    unsafe {
        let file = fopen(c_path.as_ptr(), c_mode.as_ptr());

        if file.is_null() {
            None
        } else {
            Some(file)
        }
    }
}

fn file_handling_example() {
    let path = Path::new("test.txt");

    if let Some(file) = open_file(path, "r") {
        println!("File opened successfully");

        unsafe {
            fclose(file);
        }
    } else {
        println!("Failed to open file");
    }
}
```

This pattern—wrapping unsafe C calls in safe Rust functions—is the key to good FFI code. You contain the unsafety in small, well-tested functions and expose safe APIs to the rest of your code.

## Callback Patterns

Callbacks allow C libraries to call back into your Rust code. This is common in event-driven systems, async I/O libraries, and GUI frameworks. However, callbacks introduce complexity: function pointers, state management, and lifetime issues.

### Understanding C Callbacks

In C, a callback is simply a function pointer. The C library stores this pointer and calls it when an event occurs. The challenge for Rust is that callbacks often need access to state, but C has no concept of closures. We need to bridge this gap carefully.

### Simple Function Pointer Callbacks

The simplest case is a callback that doesn't need any state:

```rust
use std::os::raw::c_int;

//==============
// C library API
//==============
extern "C" {
    fn register_callback(callback: extern "C" fn(c_int));
    fn trigger_callbacks();
}

//======================
// Our callback function
//======================
extern "C" fn my_callback(value: c_int) {
    println!("Callback received: {}", value);
}

fn simple_callback_example() {
    unsafe {
        register_callback(my_callback);
        trigger_callbacks();
    }
}
```

This works because `my_callback` is a simple function pointer. No state, no closures, just a function. But what if you need state?

### Callbacks with State (User Data Pattern)

Most C libraries support a "user data" or "context" pointer—an opaque `void*` that gets passed back to your callback:

```rust
use std::os::raw::{c_int, c_void};

extern "C" {
    fn register_callback_with_data(
        callback: extern "C" fn(*mut c_void, c_int),
        user_data: *mut c_void,
    );
    fn trigger_callbacks_with_data();
}

struct CallbackState {
    count: i32,
    name: String,
}

extern "C" fn stateful_callback(user_data: *mut c_void, value: c_int) {
    unsafe {
        // Cast the void pointer back to our state
        let state = &mut *(user_data as *mut CallbackState);

        state.count += 1;
        println!("{}: Callback #{} received value {}",
            state.name, state.count, value);
    }
}

fn stateful_callback_example() {
    let mut state = CallbackState {
        count: 0,
        name: "MyCallback".to_string(),
    };

    unsafe {
        register_callback_with_data(
            stateful_callback,
            &mut state as *mut _ as *mut c_void,
        );

        trigger_callbacks_with_data();
    }

    println!("Final count: {}", state.count);
}
```

This pattern is powerful but dangerous. You must ensure:
1. The state outlives the callback registration
2. The state isn't moved (moving would invalidate the pointer)
3. No other code mutates the state during callbacks (data race!)

### Thread-Safe Callbacks

What if callbacks can be triggered from different threads? Now you need thread-safe state:

```rust
use std::sync::{Arc, Mutex};
use std::os::raw::{c_int, c_void};

extern "C" {
    fn register_threadsafe_callback(
        callback: extern "C" fn(*mut c_void, c_int),
        user_data: *mut c_void,
    );
}

struct ThreadSafeState {
    data: Arc<Mutex<Vec<i32>>>,
}

extern "C" fn threadsafe_callback(user_data: *mut c_void, value: c_int) {
    unsafe {
        let state = &*(user_data as *const ThreadSafeState);

        // Lock the mutex before accessing shared data
        if let Ok(mut data) = state.data.lock() {
            data.push(value);
        }
    }
}

fn threadsafe_example() {
    let state = ThreadSafeState {
        data: Arc::new(Mutex::new(Vec::new())),
    };

    unsafe {
        register_threadsafe_callback(
            threadsafe_callback,
            &state as *const _ as *mut c_void,
        );
    }

    // Keep state alive for the duration of the program
    std::mem::forget(state);
}
```

Notice the `std::mem::forget` at the end. This is necessary because we're giving C a pointer to Rust-owned data. If `state` were dropped, the pointer would become invalid. `forget` leaks the memory, which is usually wrong, but here it's intentional—the callback might be called at any time in the future.

### Cleaning Up Callbacks

Of course, leaking memory isn't ideal. Better C libraries provide a way to unregister callbacks:

```rust
use std::os::raw::{c_int, c_void};

extern "C" {
    fn register_callback_managed(
        callback: extern "C" fn(*mut c_void, c_int),
        user_data: *mut c_void,
    ) -> c_int; // Returns handle

    fn unregister_callback(handle: c_int);
}

struct ManagedCallback {
    handle: c_int,
    state: Box<CallbackState>,
}

impl ManagedCallback {
    fn new(name: String) -> Self {
        let state = Box::new(CallbackState {
            count: 0,
            name,
        });

        unsafe {
            let handle = register_callback_managed(
                stateful_callback,
                &*state as *const _ as *mut c_void,
            );

            ManagedCallback { handle, state }
        }
    }
}

impl Drop for ManagedCallback {
    fn drop(&mut self) {
        unsafe {
            unregister_callback(self.handle);
        }
        // state is automatically dropped after this
    }
}

struct CallbackState {
    count: i32,
    name: String,
}

extern "C" fn stateful_callback(user_data: *mut c_void, value: c_int) {
    // Same as before
    unsafe {
        let state = &mut *(user_data as *mut CallbackState);
        state.count += 1;
    }
}
```

Now we have RAII-style cleanup. When `ManagedCallback` is dropped, it automatically unregisters the callback and cleans up the state. This is much safer.

### Closures as Callbacks (Advanced)

Can we use Rust closures as C callbacks? It's tricky, but possible for non-capturing closures:

```rust
use std::os::raw::c_int;

extern "C" {
    fn register_simple_callback(callback: extern "C" fn(c_int));
}

fn closure_callback_example() {
    // Non-capturing closure can be coerced to function pointer
    let callback: extern "C" fn(c_int) = {
        extern "C" fn wrapper(value: c_int) {
            println!("Closure callback: {}", value);
        }
        wrapper
    };

    unsafe {
        register_simple_callback(callback);
    }
}
```

But capturing closures don't work directly—they have state, and C function pointers can't carry state. You need the user data pattern for that.

## Error Handling Across FFI

Error handling is fundamentally different in C and Rust. C uses return codes, errno, and null pointers. Rust uses `Result` and `Option`. Bridging these paradigms requires careful thought about how errors propagate across the FFI boundary.

### C Error Conventions

C libraries typically signal errors in one of several ways:

1. **Return codes**: -1 or specific error codes for failure
2. **errno**: A global variable set when an error occurs
3. **Null pointers**: NULL indicates failure
4. **Out parameters**: Status written to a pointer parameter

Understanding which convention a library uses is crucial for correct error handling.

### Handling C Errors in Rust

Let's wrap a C function that uses multiple error conventions:

```rust
use std::os::raw::{c_int, c_char};
use std::ffi::CString;
use std::io;

extern "C" {
    // Returns -1 on error, sets errno
    fn c_function(path: *const c_char) -> c_int;

    // Gets the errno value
    fn __errno_location() -> *mut c_int;
}

fn errno() -> i32 {
    unsafe { *__errno_location() }
}

fn safe_c_function(path: &str) -> io::Result<i32> {
    // Convert string
    let c_path = CString::new(path)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid path"))?;

    // Call C function
    let result = unsafe { c_function(c_path.as_ptr()) };

    if result == -1 {
        // Error occurred, check errno
        let err = errno();
        Err(io::Error::from_raw_os_error(err))
    } else {
        Ok(result)
    }
}

fn error_handling_example() {
    match safe_c_function("/some/path") {
        Ok(result) => println!("Success: {}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

This pattern—checking the return value and converting errno to a Rust error—is common when wrapping POSIX functions.

### Exposing Rust Errors to C

Going the other direction is trickier. C can't understand `Result` or panics. You need to design a C-compatible error API:

```rust
use std::os::raw::{c_int, c_char};
use std::ffi::CString;

//==================
// Error codes for C
//==================
const SUCCESS: c_int = 0;
const ERROR_NULL_POINTER: c_int = -1;
const ERROR_INVALID_INPUT: c_int = -2;
const ERROR_COMPUTATION_FAILED: c_int = -3;

#[no_mangle]
pub extern "C" fn rust_compute(
    input: *const c_char,
    output: *mut f64,
) -> c_int {
    // Check for null pointers
    if input.is_null() || output.is_null() {
        return ERROR_NULL_POINTER;
    }

    unsafe {
        // Convert C string to Rust
        let c_str = std::ffi::CStr::from_ptr(input);
        let rust_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return ERROR_INVALID_INPUT,
        };

        // Do computation
        let result = match compute_internal(rust_str) {
            Ok(r) => r,
            Err(_) => return ERROR_COMPUTATION_FAILED,
        };

        // Write result
        *output = result;
        SUCCESS
    }
}

fn compute_internal(input: &str) -> Result<f64, &'static str> {
    input.parse().map_err(|_| "Invalid number")
}

//==================================
// Helper for getting error messages
//==================================
#[no_mangle]
pub extern "C" fn rust_error_message(error_code: c_int) -> *const c_char {
    let msg = match error_code {
        ERROR_NULL_POINTER => "Null pointer provided\0",
        ERROR_INVALID_INPUT => "Invalid input\0",
        ERROR_COMPUTATION_FAILED => "Computation failed\0",
        _ => "Unknown error\0",
    };

    msg.as_ptr() as *const c_char
}
```

This provides a C-friendly error API: integer codes with a function to get error messages. The pattern of using out-parameters (the `output` pointer) is also very C-friendly.

### Panic Safety

One critical consideration: **Rust must never panic across FFI boundaries**. If Rust code panics while called from C, the behavior is undefined. Always catch panics:

```rust
use std::panic;
use std::os::raw::c_int;

#[no_mangle]
pub extern "C" fn safe_rust_function(value: c_int) -> c_int {
    // Catch any panics
    let result = panic::catch_unwind(|| {
        // Code that might panic
        risky_computation(value)
    });

    match result {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Rust function panicked!");
            -1 // Error code
        }
    }
}

fn risky_computation(value: c_int) -> c_int {
    if value < 0 {
        panic!("Negative values not allowed");
    }
    value * 2
}
```

`catch_unwind` prevents panics from crossing the FFI boundary, giving you a chance to log the error and return a safe error code.

### Error Context and Debugging

For complex FFI code, maintaining error context is important:

```rust
use std::os::raw::c_int;
use std::sync::Mutex;
use std::cell::RefCell;

//===========================
// Thread-local error storage
//===========================
thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

fn set_last_error(error: String) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = Some(error);
    });
}

#[no_mangle]
pub extern "C" fn rust_function_with_error(value: c_int) -> c_int {
    if value < 0 {
        set_last_error(format!("Invalid value: {}", value));
        return -1;
    }

    // ... computation ...

    0
}

#[no_mangle]
pub extern "C" fn rust_get_last_error() -> *const std::os::raw::c_char {
    LAST_ERROR.with(|e| {
        match &*e.borrow() {
            Some(err) => {
                // Note: This is simplified. In production, you'd want to
                // manage the string's lifetime more carefully
                let c_str = std::ffi::CString::new(err.as_str()).unwrap();
                c_str.into_raw()
            }
            None => std::ptr::null(),
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn rust_clear_last_error() {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = None;
    });
}
```

This provides detailed error messages while maintaining a C-compatible API.

## bindgen Patterns

Manually declaring extern functions and types is tedious and error-prone. The `bindgen` tool automates this process, generating Rust FFI bindings from C header files. This is invaluable for large C libraries.

### How bindgen Works

bindgen parses C header files using libclang (the C compiler's parser) and generates corresponding Rust code. It handles:
- Function declarations
- Struct and union definitions
- Enums and constants
- Type aliases
- Preprocessor macros (to some extent)

The generated code follows the same patterns we've discussed: `extern "C"` blocks, `#[repr(C)]` structs, and raw pointers.

### Basic bindgen Usage

First, add bindgen to your build dependencies:

```toml
# Cargo.toml
[build-dependencies]
bindgen = "0.69"
```

Then create a build script:

```rust
//=========
// build.rs
//=========
use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // Link to C library
    println!("cargo:rustc-link-lib=mylib");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write bindings to $OUT_DIR/bindings.rs
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
```

Create a wrapper header that includes the library you want to bind:

```c
// wrapper.h
#include <mylib.h>
```

Then use the generated bindings in your Rust code:

```rust
//==================
// lib.rs or main.rs
//==================
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn use_bindings() {
    unsafe {
        // Use the generated bindings
        let result = some_c_function(42);
        println!("Result: {}", result);
    }
}
```

The `allow` attributes silence warnings about the generated code not following Rust naming conventions.

### Configuring bindgen

bindgen is highly configurable. You can control what gets generated:

```rust
//=========
// build.rs
//=========
use bindgen;

fn main() {
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")

        // Allowlist: only generate bindings for these
        .allowlist_function("my_.*")  // Regex: functions starting with my_
        .allowlist_type("MyStruct")
        .allowlist_var("MY_CONSTANT")

        // Blocklist: don't generate bindings for these
        .blocklist_function("internal_.*")

        // Generate comments from C documentation
        .generate_comments(true)

        // Use core instead of std (for no_std environments)
        .use_core()

        // Derive additional traits
        .derive_default(true)
        .derive_debug(true)
        .derive_eq(true)

        // Handle C++ (if needed)
        .clang_arg("-x")
        .clang_arg("c++")

        // Custom type mappings
        .raw_line("use std::os::raw::c_char;")

        .generate()
        .expect("Unable to generate bindings");

    // Write bindings...
}
```

This level of control is essential for large libraries where you only need a subset of functionality.

### Wrapping Generated Bindings

Generated bindings are completely unsafe. Best practice is to wrap them in safe Rust APIs:

```rust
//=====================
// Generated by bindgen
//=====================
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

//=============
// Safe wrapper
//=============
pub struct Database {
    handle: *mut ffi::db_t,
}

impl Database {
    pub fn open(path: &str) -> Result<Self, String> {
        let c_path = std::ffi::CString::new(path)
            .map_err(|_| "Invalid path")?;

        let handle = unsafe {
            ffi::db_open(c_path.as_ptr())
        };

        if handle.is_null() {
            Err("Failed to open database".to_string())
        } else {
            Ok(Database { handle })
        }
    }

    pub fn query(&self, sql: &str) -> Result<Vec<String>, String> {
        let c_sql = std::ffi::CString::new(sql)
            .map_err(|_| "Invalid SQL")?;

        unsafe {
            let result = ffi::db_query(self.handle, c_sql.as_ptr());

            if result.is_null() {
                return Err("Query failed".to_string());
            }

            // Process results...
            // (This is simplified; real code would parse the result)

            ffi::db_free_result(result);
            Ok(vec![])
        }
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        unsafe {
            ffi::db_close(self.handle);
        }
    }
}

//=========================================================
// Safe to send between threads if C library is thread-safe
//=========================================================
unsafe impl Send for Database {}
```

Now users of your library can work with `Database` using safe, idiomatic Rust, while all the FFI complexity is hidden.

### Handling Opaque Types

C libraries often use opaque pointers—pointers to types whose definition isn't in the header:

```c
// In C header
typedef struct db_connection db_connection_t;
db_connection_t* db_connect(const char* url);
void db_disconnect(db_connection_t* conn);
```

bindgen generates an opaque type:

```rust
//==========
// Generated
//==========
#[repr(C)]
pub struct db_connection {
    _unused: [u8; 0],
}
```

You can't construct this directly (it has zero size!), but you can hold pointers to it. This is actually perfect—it prevents you from incorrectly constructing instances:

```rust
pub struct Connection {
    ptr: *mut ffi::db_connection,
}

impl Connection {
    pub fn connect(url: &str) -> Result<Self, String> {
        let c_url = std::ffi::CString::new(url)
            .map_err(|_| "Invalid URL")?;

        let ptr = unsafe { ffi::db_connect(c_url.as_ptr()) };

        if ptr.is_null() {
            Err("Connection failed".to_string())
        } else {
            Ok(Connection { ptr })
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
            ffi::db_disconnect(self.ptr);
        }
    }
}
```

### Function Pointers and Callbacks with bindgen

bindgen handles C function pointers automatically:

```c
// C header
typedef void (*callback_t)(int value, void* user_data);
void register_callback(callback_t callback, void* user_data);
```

Generated Rust:

```rust
//==========
// Generated
//==========
pub type callback_t = Option<unsafe extern "C" fn(value: ::std::os::raw::c_int, user_data: *mut ::std::os::raw::c_void)>;

extern "C" {
    pub fn register_callback(callback: callback_t, user_data: *mut ::std::os::raw::c_void);
}
```

You can then implement callbacks using the patterns we discussed earlier:

```rust
extern "C" fn my_callback(value: i32, user_data: *mut std::ffi::c_void) {
    unsafe {
        let state = &mut *(user_data as *mut MyState);
        state.handle_event(value);
    }
}

struct MyState {
    count: i32,
}

impl MyState {
    fn handle_event(&mut self, value: i32) {
        self.count += value;
        println!("Event: {}, Total: {}", value, self.count);
    }
}
```

## Conclusion

FFI is where Rust's safety guarantees meet the unsafe reality of C code. The key to successful FFI is understanding what guarantees you can maintain and where you must carefully document assumptions.

**Key principles:**

1. **Encapsulate unsafety**: Wrap unsafe FFI calls in safe Rust APIs
2. **Document invariants**: Clearly state what the caller must ensure
3. **Handle errors gracefully**: Convert C error codes to Rust's Result
4. **Manage lifetimes carefully**: Ensure pointers remain valid
5. **Use bindgen**: Automate the tedious parts, but review the output
6. **Test thoroughly**: FFI bugs can be subtle and destructive

FFI is a powerful tool for leveraging existing C libraries while writing new code in Rust. By following these patterns and understanding the challenges, you can build reliable, safe wrappers around C code, bringing Rust's safety guarantees to the vast ecosystem of existing libraries.

The future of systems programming is incremental adoption—not rewriting everything from scratch, but gradually replacing components with safer alternatives. FFI makes this possible, allowing Rust and C to coexist peacefully while you transition at your own pace.
