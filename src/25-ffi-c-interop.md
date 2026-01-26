# FFI & C Interop
This chapter covers FFI (Foreign Function Interface)—calling C from Rust and vice versa. Rust's safety model differs from C: must use unsafe, manage memory carefully, handle strings/callbacks/errors correctly. Essential for integrating existing C libraries and system APIs.



## Pattern 1: C ABI Compatibility

**Problem**: Rust and C have incompatible ABIs—Rust optimizes struct layouts (reorders fields), uses different calling conventions, doesn't guarantee ABI stability between versions. Passing Rust struct to C corrupts data (wrong offsets).

**Solution**: Use extern "C" to declare C functions and mark Rust functions for C. Use #[repr(C)] to force C-compatible struct layout—fields in declaration order, C alignment rules.

**Why It Matters**: Enables using decades of C libraries—SQLite, OpenSSL, zlib, libcurl. OS APIs (POSIX, Win32) are C.

**Use Cases**: System calls (open, read, write), database drivers (SQLite, Postgres FFI), graphics APIs (OpenGL, Vulkan, DirectX), audio/video codecs (ffmpeg), crypto libraries (OpenSSL, libsodium), compression (zlib, lz4), embedded HALs, legacy C code integration.

### Example: C ABI Fundamentals
Without `#[repr(C)]`, Rust may reorder struct fields or add padding for optimization. The `#[repr(C)]` attribute forces C-compatible memory layout with fields in declaration order and standard C alignment rules, guaranteeing binary compatibility when passing structs across FFI boundaries.

```rust
struct RustStruct {  // Rust may reorder fields, add padding
    a: u8,
    b: u32,
    c: u16,
}

#[repr(C)]  // Guaranteed C-compatible layout
struct CCompatibleStruct {
    a: u8,    // 1B + 3B padding
    b: u32,   // 4B, 4-byte aligned
    c: u16,   // 2B
}

fn demonstrate_repr() {  // Both 12B, but only CCompatibleStruct guarantees C layout
    println!("RustStruct size: {}", std::mem::size_of::<RustStruct>());
    println!("CCompatibleStruct size: {}", std::mem::size_of::<CCompatibleStruct>());
}
demonstrate_repr(); // Shows size difference between Rust and C-compatible structs
```

The difference becomes critical when passing data to C libraries. If the layouts don't match exactly, you'll get corrupted data or crashes.

### Example: Calling C Functions from Rust
Declare external C functions in an `extern "C"` block with Rust types matching C's ABI. All calls require `unsafe` blocks since Rust cannot verify C code's memory safety, null-pointer handling, or bounds checking. Common libc functions like `abs`, `malloc`, and `sqrt` demonstrate this pattern.

```rust
extern "C" {  // Declare external C functions
    fn abs(n: i32) -> i32;                        // int abs(int)
    fn malloc(size: usize) -> *mut std::ffi::c_void;  // void* malloc(size_t)
    fn free(ptr: *mut std::ffi::c_void);              // void free(void*)
    fn sqrt(x: f64) -> f64;                           // double sqrt(double)
}

fn use_c_functions() {
    unsafe {  // All C calls unsafe—compiler can't verify C's guarantees
        let result = abs(-42);
        println!("abs(-42) = {}", result);

        let root = sqrt(16.0);
        println!("sqrt(16.0) = {}", root);
    }
}
use_c_functions(); // Calls C stdlib abs() and sqrt() functions
```

Notice that we must wrap C function calls in `unsafe` blocks. This is Rust's way of saying: "Beyond this point, the safety guarantees are up to you." C doesn't have Rust's borrow checker, null-safety, or bounds checking, so the compiler can't verify that the C code won't cause undefined behavior.

### Example: Exposing Rust Functions to C
Use `#[no_mangle]` with `extern "C"` to expose Rust functions to C. The `#[no_mangle]` attribute prevents Rust's name mangling, which encodes type information into symbol names. This ensures simple, predictable function names like `rust_add` that C linkers can find.

```rust
#[no_mangle]  // Preserve symbol name for C linking
pub extern "C" fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]  // Safety: values valid for count elements, aligned, not mutated
pub extern "C" fn rust_compute_average(values: *const f64, count: usize) -> f64 {
    unsafe {
        if values.is_null() || count == 0 {
            return 0.0;
        }

        let slice = std::slice::from_raw_parts(values, count);
        let sum: f64 = slice.iter().sum();
        sum / count as f64
    }
}
From C: double avg = rust_compute_average(arr, 5);
```

The `#[no_mangle]` attribute is crucial. Normally, Rust "mangles" function names to encode type information, which helps with overloading but makes the names unreadable. C expects simple, predictable names like `rust_add`, not something like `_ZN4rust8rust_add17h3b3c3d3e3f3g3hE`.

### Example: Type Mapping Between C and Rust
Use types from `std::os::raw` (`c_int`, `c_char`, `c_long`, `c_void`) for platform-independent C type mappings. These aliases automatically match C's type sizes on each platform, avoiding assumptions like "int is always i32" and ensuring correct behavior across 32-bit and 64-bit systems.

```rust
use std::os::raw::{c_char, c_int, c_long, c_ulong, c_void};
// c_char=i8/u8, c_int=i32, c_long=i32|i64, void*=*mut c_void, size_t=usize

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
use_c_types(); // Demonstrates C type mappings (c_int, c_char, etc.)
```

The `std::os::raw` module provides platform-independent type aliases that match C's types on the current platform. Always use these rather than assuming `int` is `i32`—on some platforms, it might not be.

### Example: Struct Padding and Alignment
The `#[repr(C)]` attribute adds padding bytes between fields to satisfy alignment requirements, making structs larger but ensuring proper memory access. Adding `#[repr(C, packed)]` removes all padding, reducing size but risking misaligned access. Use `std::mem::size_of` and `align_of` to verify layouts.

```rust
#[repr(C)]
struct Padded {
    a: u8,    // 1B + 3B padding
    b: u32,   // 4B (4-byte aligned)
    c: u8,    // 1B + 3B padding (struct size = 12)
}

#[repr(C, packed)]  // No padding, may cause misaligned access!
struct NoPadding {
    a: u8,    // 1B
    b: u32,   // 4B
    c: u8,    // 1B (struct size = 6)
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
examine_layout(); // Shows padding effects: 12 bytes vs 6 bytes
```

The `packed` attribute removes padding, which can save space but may cause performance issues or crashes on architectures that don't support misaligned access. Only use it when interfacing with C code that explicitly uses packed structs.

## Pattern 2: String Conversions

**Problem**: Rust &str (UTF-8, length-prefixed, can contain NUL) incompatible with C *char (null-terminated, no encoding, stops at \0). Ownership unclear: who frees the string?

**Solution**: Use CString to create owned null-terminated C strings. Use CStr to borrow C strings.

**Why It Matters**: Strings are most error-prone FFI aspect—wrong conversion causes use-after-free, buffer overrun, null pointer deref, memory leaks. FFI boundaries in production: file paths, error messages, configuration.

**Use Cases**: Passing filenames to C (fopen, stat), error messages from C (strerror), configuration strings, logging to C libraries, command-line arguments, environment variables, C string parsing, text processing across FFI.

### Example: CString/CStr Pattern
`CString` creates owned, null-terminated strings for passing to C. `CStr` borrows existing C strings without allocation. Key methods: `CString::new()` for creation, `as_ptr()` for C pointer access, `CStr::from_ptr()` for borrowing. Ensure `CString` outlives any pointers derived from it.

```rust
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

fn rust_string_to_c() {  // Rust to C
    let rust_string = "Hello, C!";
    let c_string = CString::new(rust_string).expect("CString::new failed");  // Alloc + null
    let c_ptr: *const c_char = c_string.as_ptr();

    unsafe { some_c_function(c_ptr); }
}  // c_string dropped here

unsafe fn c_string_to_rust(c_ptr: *const c_char) -> String {  // C to Rust
    if c_ptr.is_null() { return String::new(); }
    let c_str = CStr::from_ptr(c_ptr);  // Borrow C string
    c_str.to_string_lossy().into_owned()  // Lossy: invalid UTF-8 → �
}

extern "C" {
    fn some_c_function(s: *const c_char);
}
rust_string_to_c(); // Converts "Hello, C!" to null-terminated C string
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
demonstrate_null_bytes(); // Shows CString fails on embedded NUL bytes
```

### Example: OsString and OsStr
`OsString` and `OsStr` handle platform-specific path encodings that may not be valid UTF-8. Windows uses UTF-16; Unix allows arbitrary bytes. Use `to_str()` for fallible UTF-8 conversion or `to_string_lossy()` for lossy conversion. These types integrate with `Path` and `PathBuf` for cross-platform operations.

```rust
use std::ffi::{OsString, OsStr};
use std::path::{Path, PathBuf};

fn working_with_os_strings() {
    let os_string = OsString::from("my_file.txt");
    let path = Path::new("/home/user/document.txt");
    let os_str: &OsStr = path.as_os_str();

    match os_str.to_str() {  // May fail if not valid UTF-8
        Some(s) => println!("Valid UTF-8: {}", s),
        None => println!("Path contains invalid UTF-8"),
    }

    let string = os_str.to_string_lossy();  // Invalid UTF-8 → �
    println!("Path (lossy): {}", string);
}
working_with_os_strings(); // Platform-independent path string handling
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

### Example: String Ownership Across FFI
Use `CString::into_raw()` to transfer ownership to C, providing a corresponding `rust_free_string()` that calls `CString::from_raw()`. Use `CStr::from_ptr()` to borrow C-owned strings. Golden rule: whoever allocates must free. Mixing allocators causes heap corruption.

```rust
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

#[no_mangle]  // Rust allocates, C must call rust_free_string()
pub extern "C" fn rust_creates_string() -> *mut c_char {
    CString::new("Hello from Rust").unwrap().into_raw()  // Transfer ownership
}

#[no_mangle]
pub unsafe extern "C" fn rust_free_string(s: *mut c_char) {
    if !s.is_null() { let _ = CString::from_raw(s); }  // Reclaim and drop
}

#[no_mangle]  // C owns string, Rust borrows
pub unsafe extern "C" fn rust_uses_c_string(s: *const c_char) {
    if s.is_null() { return; }
    let c_str = CStr::from_ptr(s);  // Borrow only
    println!("C string: {}", c_str.to_string_lossy());
}  // s still valid, C frees
From C: rust_uses_c_string("hello"); // Borrows C string, C frees it

// WRONG: This leaks memory!
#[no_mangle]
pub extern "C" fn leaked_string() -> *const c_char {
    let s = CString::new("This will leak").unwrap();
    s.as_ptr() // s is dropped here, but pointer escapes!
}
```

The golden rule: **whoever allocates the memory must free it**. If Rust allocates, Rust must free (even if C holds the pointer temporarily). If C allocates, C must free.

### Example: Practical Example: File Path Handling
This example wraps C's `fopen` and `fclose` in a safe Rust API. The wrapper converts `Path` to `CString`, validates that `fopen` doesn't return NULL, and returns `Option` for success or failure. This pattern encapsulates unsafe code while presenting an idiomatic Rust interface.

```rust
use std::ffi::{CString, CStr, OsStr};
use std::os::raw::c_char;
use std::path::Path;

// C API for opening a file
extern "C" {
    fn fopen(filename: *const c_char, mode: *const c_char) -> *mut std::ffi::c_void;
    fn fclose(file: *mut std::ffi::c_void) -> i32;
}

fn open_file(path: &Path, mode: &str) -> Option<*mut std::ffi::c_void> {
    let path_str = path.to_str()?;  // Fails if path has null bytes
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
file_handling_example(); // Safe Rust wrapper around C's fopen/fclose
```

This pattern—wrapping unsafe C calls in safe Rust functions—is the key to good FFI code. You contain the unsafety in small, well-tested functions and expose safe APIs to the rest of your code.

## Pattern 3: Callback Patterns

**Problem**: C callbacks expect function pointers (extern "C" fn), but Rust closures capture environment (not C-compatible). Need stateful callbacks—closure with captured state called from C.

**Solution**: Use extern "C" fn for stateless callbacks. For stateful: Box::into_raw() passes closure as void*, trampoline function extracts state.

**Why It Matters**: Async APIs need callbacks—C libraries can't block Rust futures. Signal handlers require callbacks.

**Use Cases**: Event loops (GUI toolkits—GTK, Qt), async I/O libraries, signal handlers (SIGINT, SIGTERM), qsort comparators, thread spawn callbacks (pthread_create), plugin systems, C library hooks, timer callbacks.

### Example: Function Pointer Callback Pattern
Stateless `extern "C"` functions can be passed directly as C callback function pointers. These functions cannot capture environment state since C function pointers are simple addresses without closure context. The callback signature must match exactly what C expects. For stateful callbacks, use the user data pattern.

```rust
use std::os::raw::c_int;

// C library API
extern "C" {
    fn register_callback(callback: extern "C" fn(c_int));
    fn trigger_callbacks();
}

// Our callback function
extern "C" fn my_callback(value: c_int) {
    println!("Callback received: {}", value);
}

fn simple_callback_example() {
    unsafe {
        register_callback(my_callback);
        trigger_callbacks();
    }
}
simple_callback_example(); // Registers extern "C" fn as C callback
```

This works because `my_callback` is a simple function pointer. No state, no closures, just a function. But what if you need state?

### Example: Callbacks with State (User Data Pattern)
Pass Rust state to C as a `*mut c_void` user data pointer, then cast back inside the callback. Critical requirements: state must outlive callback registration, must not be moved while registered, and must not be mutated from other code during execution to avoid data races.

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
        let state = &mut *(user_data as *mut CallbackState);  // Cast back
        state.count += 1;
        println!("{}: Callback #{} received {}", state.name, state.count, value);
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
stateful_callback_example(); // Passes Rust state via void* user_data
```

This pattern is powerful but dangerous. You must ensure:
1. The state outlives the callback registration
2. The state isn't moved (moving would invalidate the pointer)
3. No other code mutates the state during callbacks (data race!)

### Example: Thread-Safe Callbacks
For C libraries invoking callbacks from multiple threads, wrap shared state in `Arc<Mutex<T>>` for thread-safe access. The mutex prevents data races when concurrent callbacks modify state. Use `std::mem::forget` to leak state intentionally, keeping it alive when the callback's lifetime is unbounded.

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
        if let Ok(mut data) = state.data.lock() { data.push(value); }  // Lock before access
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

    std::mem::forget(state);  // Intentional leak—callback may fire anytime
}
threadsafe_example(); // Thread-safe callback with Arc<Mutex<T>> state
```

Notice the `std::mem::forget` at the end. This is necessary because we're giving C a pointer to Rust-owned data. If `state` were dropped, the pointer would become invalid. `forget` leaks the memory, which is usually wrong, but here it's intentional—the callback might be called at any time in the future.

### Example: Cleaning Up Callbacks
Implement RAII-style cleanup with a struct holding both callback handle and state. Implement `Drop` to automatically unregister the callback and free resources when out of scope. This requires the C API to provide an unregister function. `Box` ensures stable memory addresses for state pointers.

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
        unsafe { unregister_callback(self.handle); }  // state auto-dropped after
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
ManagedCallback::new("test".into()); // RAII callback with auto-cleanup
```

Now we have RAII-style cleanup. When `ManagedCallback` is dropped, it automatically unregisters the callback and cleans up the state. This is much safer.

### Example: Closures as Callbacks (Advanced)
Non-capturing closures can be coerced to `extern "C" fn` pointers because they have no state. Define an inner `extern "C"` wrapper function for the work. Capturing closures cannot work directly as C callbacks since they are fat pointers with code and environment. Use user data pattern instead.

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
closure_callback_example(); // Non-capturing closure as C callback
```

But capturing closures don't work directly—they have state, and C function pointers can't carry state. You need the user data pattern for that.

## Pattern 4: Error Handling Across FFI

**Problem**: Rust Result<T, E> and panic incompatible with C's errno/return codes/NULL. Panics unwinding into C are undefined behavior (crashes, memory corruption).

**Solution**: Convert Result to i32 return codes (0 = success, <0 = error). Set errno for POSIX compatibility.

**Why It Matters**: Panicking into C is undefined behavior—absolutely must prevent. Production FFI must never panic.

**Use Cases**: All FFI functions (must handle errors properly), library wrappers (translate Result to errno), system calls, C callbacks (cannot panic), plugin interfaces, language bindings (Python, Ruby calling Rust), error propagation.

### Example: Error Translation Pattern
Wrap POSIX-style C functions that return -1 on error and set `errno`. Check return value, read thread-local `errno` via platform-specific functions like `__errno_location`, then convert to Rust's `io::Error` using `from_raw_os_error`. This transforms C error conventions into idiomatic `Result` types.

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
error_handling_example(); // Converts C errno to Rust io::Result
```

This pattern—checking the return value and converting errno to a Rust error—is common when wrapping POSIX functions.

### Example: Exposing Rust Errors to C
Translate Rust `Result` to C conventions: return integer error codes where 0 means success and negative values indicate errors. Validate all pointers for null before dereferencing. Use out-parameters for computed values. Provide `rust_error_message()` to map error codes to human-readable C strings.

```rust
use std::os::raw::{c_int, c_char};
use std::ffi::CString;

// Error codes for C
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

// Helper for getting error messages
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
From C: int err = rust_compute(in, &out); const char* msg = rust_error_message(err);
```

This provides a C-friendly error API: integer codes with a function to get error messages. The pattern of using out-parameters (the `output` pointer) is also very C-friendly.

### Example: Panic Safety
Use `std::panic::catch_unwind` to intercept panics before they cross FFI boundaries, which causes undefined behavior. This function runs a closure returning `Ok(value)` on success or `Err` on panic. Convert caught panics to error codes C can handle. Essential for any Rust code called from C.

```rust
use std::panic;
use std::os::raw::c_int;

#[no_mangle]
pub extern "C" fn safe_rust_function(value: c_int) -> c_int {
    let result = panic::catch_unwind(|| risky_computation(value));  // Catch panics

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
safe_rust_function(-1) returns -1 instead of unwinding into C
```

`catch_unwind` prevents panics from crossing the FFI boundary, giving you a chance to log the error and return a safe error code.

### Example: Error Context and Debugging
Thread-local storage provides detailed error messages beyond integer codes. Use `thread_local!` with `RefCell<Option<String>>` to store last error messages. Expose `rust_get_last_error()` for C to retrieve messages and `rust_clear_last_error()` for cleanup. This mimics how C uses errno with strerror.

```rust
use std::os::raw::c_int;
use std::sync::Mutex;
use std::cell::RefCell;

thread_local! {  // Thread-local error storage
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

#[no_mangle]  // Returns error msg; production code needs better lifetime management
pub extern "C" fn rust_get_last_error() -> *const std::os::raw::c_char {
    LAST_ERROR.with(|e| match &*e.borrow() {
        Some(err) => std::ffi::CString::new(err.as_str()).unwrap().into_raw(),
        None => std::ptr::null(),
    })
}

#[no_mangle]
pub unsafe extern "C" fn rust_clear_last_error() {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = None;
    });
}
rust_function_with_error(-1); const char* err = rust_get_last_error();
```

This provides detailed error messages while maintaining a C-compatible API.

## Pattern 5: bindgen Patterns

**Problem**: Manually writing FFI bindings tedious—100+ functions, 50+ structs, error-prone. Struct layouts wrong (misaligned fields).

**Solution**: bindgen auto-generates Rust bindings from C headers. Parses with libclang (actual compiler).

**Why It Matters**: Eliminates 90% manual FFI work—hundreds of lines auto-generated. Keeps bindings in sync with C headers automatically.

**Use Cases**: Wrapping C libraries (SQLite, libcurl, OpenSSL, SDL), system API bindings (libc, Win32), graphics APIs (OpenGL, Vulkan), audio/video libraries (ffmpeg), embedded HALs, automatic binding generation, maintaining C library wrappers.

### Example: bindgen Setup Pattern
Add bindgen as a build dependency, then create `build.rs` invoking `bindgen::Builder` to parse C headers and generate Rust bindings. Use `cargo:rerun-if-changed` for auto-regeneration. Include generated code via `include!(concat!(env!("OUT_DIR"), "/bindings.rs"))`. Link with `cargo:rustc-link-lib`.

```toml
# Cargo.toml
[build-dependencies]
bindgen = "0.69"
```

Then create a build script:

```rust
// build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");  // Rebuild on header change
    println!("cargo:rustc-link-lib=mylib");        // Link C library

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

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
// lib.rs or main.rs
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn use_bindings() {
    unsafe { let result = some_c_function(42); println!("Result: {}", result); }
}
use_bindings(); // Calls auto-generated FFI bindings from bindgen
```

The `allow` attributes silence warnings about the generated code not following Rust naming conventions.

### Example: Configuring bindgen
Use `allowlist_function`, `allowlist_type`, and `allowlist_var` with regex patterns to generate only needed bindings. Use `blocklist_*` to exclude internals. Enable `derive_default`, `derive_debug`, `derive_eq` for trait implementations. Use `use_core()` for no_std and `clang_arg` for C++ or custom includes.

```rust
// build.rs
use bindgen;

fn main() {
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .allowlist_function("my_.*")   // Regex: only my_* functions
        .allowlist_type("MyStruct")
        .allowlist_var("MY_CONSTANT")
        .blocklist_function("internal_.*")  // Exclude internals
        .generate_comments(true)       // Keep C docs
        .use_core()                    // For no_std
        .derive_default(true).derive_debug(true).derive_eq(true)  // Traits
        .clang_arg("-x").clang_arg("c++")  // C++ support
        .raw_line("use std::os::raw::c_char;")  // Custom imports
        .generate()
        .expect("Unable to generate bindings");
}
cargo build triggers build.rs to generate filtered bindings
```

This level of control is essential for large libraries where you only need a subset of functionality.

### Example: Wrapping Generated Bindings
Create safe Rust wrappers around raw bindgen output to hide FFI complexity. Validate inputs, check for null pointers, implement `Drop` for automatic cleanup, and expose idiomatic `Result`-returning APIs. This lets users work with safe Rust while unsafe code remains contained.

```rust
// Generated by bindgen
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// Safe wrapper
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
            if result.is_null() { return Err("Query failed".to_string()); }
            // Real code would parse result here
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

// Safe to send between threads if C library is thread-safe
unsafe impl Send for Database {}
let db = Database::open("test.db")?; db.query("SELECT *")?;
```

Now users of your library can work with `Database` using safe, idiomatic Rust, while all the FFI complexity is hidden.

### Example: Handling Opaque Types
bindgen represents opaque C types as zero-sized structs with `_unused: [u8; 0]`. You cannot construct these directly, only hold pointers to them. This prevents accidental construction of library-managed resources and ensures only the C library creates valid instances.

```c
// In C header
typedef struct db_connection db_connection_t;
db_connection_t* db_connect(const char* url);
void db_disconnect(db_connection_t* conn);
```

bindgen generates an opaque type:

```rust
// Generated
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
let conn = Connection::connect("postgres://...")?; // Auto-disconnects on drop
```

### Example: Function Pointers and Callbacks with bindgen
bindgen translates C function pointers to `Option<unsafe extern "C" fn(...)>` types. Implement callbacks using the user data pattern from earlier examples. bindgen ensures type signatures match, reducing manual errors when working with C callback APIs.

```c
// C header
typedef void (*callback_t)(int value, void* user_data);
void register_callback(callback_t callback, void* user_data);
```

Generated Rust:

```rust
// Generated
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


### Summary

This chapter covered FFI (Foreign Function Interface) patterns for C interop:

1. **C ABI Compatibility**: extern "C", #[repr(C)], raw pointers, calling conventions
2. **String Conversions**: CString/CStr, null-termination, UTF-8 validation, ownership
3. **Callback Patterns**: Function pointers, stateful callbacks, panic boundaries, context pointers
4. **Error Handling**: Result to errno, catch_unwind(), return codes, preventing UB
5. **bindgen Patterns**: Auto-generate bindings, build.rs integration, allowlist/blocklist

**Key Takeaways**:
- FFI bridges Rust safety with C's unsafety—careful handling required
- All FFI calls are unsafe—must verify C library guarantees
- extern "C" for C calling convention, #[repr(C)] for C struct layout
- Strings most error-prone: CString/CStr for null-terminated conversion
- Panics across FFI are undefined behavior—always catch_unwind()
- bindgen automates 90% of FFI work for large C libraries

**Critical Safety Rules**:
- **Never panic into C**: Use catch_unwind() at FFI boundary
- **Validate all pointers**: Check for NULL before dereferencing
- **Manage lifetimes**: Ensure pointers outlive their use
- **Match layouts exactly**: Use #[repr(C)] and verify with tests
- **Handle errors**: Translate Result to C conventions properly

**Common Patterns**:
- Wrap unsafe FFI in safe Rust APIs
- Use CString::new() → as_ptr() for Rust → C strings
- Use CStr::from_ptr() → to_str() for C → Rust strings
- Box::into_raw() for transferring ownership to C
- catch_unwind() in callbacks to prevent unwinding into C

**When to Use FFI**:
- Existing C libraries too valuable to rewrite (SQLite, OpenSSL)
- System APIs (OS interfaces are C)
- Incremental migration (rewrite gradually)
- Performance-critical code already optimized in C
- Ecosystem integration (Python/Ruby call Rust via C FFI)

**Best Practices**:
- Encapsulate unsafety in small, well-tested functions
- Document all invariants and safety requirements
- Use bindgen for large C APIs
- Test FFI boundaries extensively
- Validate all data crossing boundary
- Never trust C to uphold Rust's invariants

**Common Pitfalls**:
- Panicking across FFI boundary (undefined behavior)
- String lifetime issues (use-after-free)
- Struct layout mismatches (wrong #[repr])
- Forgetting to validate UTF-8 from C
- Double-free or memory leaks
- Integer overflow in size conversions

**Tools**:
- **bindgen**: Auto-generate bindings from C headers
- **cbindgen**: Generate C headers from Rust (reverse)
- **cargo expand**: View generated code
- **Miri**: Detect undefined behavior (limited FFI support)
- **Valgrind**: Memory errors at runtime
