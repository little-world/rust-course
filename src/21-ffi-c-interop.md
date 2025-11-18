# FFI & C Interop

C ABI Compatibility

- Problem: Rust/C have different ABIs; struct layouts differ; calling conventions incompatible; passing Rust types to C corrupts data; need binary compatibility
- Solution: extern "C" for C calling convention; #[repr(C)] for C struct layout; use c_int/c_char types; raw pointers (*const T, *mut T); unsafe blocks
- Why It Matters: Enables using C libraries (OpenSSL, SQLite); OS APIs are C; rewrite incrementally (FFI boundary); 50+ years of C code accessible
- Use Cases: System calls, database drivers (SQLite, Postgres), graphics (OpenGL), audio/video codecs, crypto (OpenSSL), embedded systems, legacy integration

String Conversions

- Problem: Rust &str (UTF-8, length-prefix) incompatible with C *char (null-terminated); ownership unclear; lifetime issues; UTF-8 validation needed
- Solution: CString/CStr for null-terminated; as_ptr() for passing to C; from_ptr() for receiving from C; into_raw() transfers ownership; validate UTF-8
- Why It Matters: Strings are FFI's most error-prone area—wrong conversion causes use-after-free, buffer overruns, null pointer derefs. UTF-8 validation critical.
- Use Cases: Passing filenames, error messages, configuration, logging to C; parsing C strings; command-line args; environment variables

Callback Patterns

- Problem: C callbacks expect function pointers; closures capture state (not C-compatible); need Rust closure called from C; lifetime management complex
- Solution: extern "C" fn for callbacks; Box::into_raw() for closure state; trampoline pattern; static/global state; panic::catch_unwind() boundary
- Why It Matters: Async APIs need callbacks; signal handlers; GUI event loops; thread callbacks; plugin systems. Panics across FFI are UB—must catch.
- Use Cases: Event loops (GUI, async I/O), signal handlers, qsort comparators, thread spawn callbacks, plugin systems, C library hooks

Error Handling Across FFI

- Problem: Rust Result/panic vs C errno/return codes; panics across FFI are UB; can't propagate ? across boundary; error context lost
- Solution: Convert Result to i32/errno; catch_unwind() prevents unwinding into C; out-parameters for detailed errors; error codes enum; thread_local! for errno
- Why It Matters: Panicking into C is undefined behavior (crashes, corruption). C has no Result type. Must translate error models. Essential for reliability.
- Use Cases: All FFI functions (must not panic), library wrappers, system calls, C callbacks, plugin interfaces, language bindings

bindgen Patterns

- Problem: Manually writing extern blocks tedious; struct layouts error-prone; C headers have macros/complex types; keeping sync hard; #define constants inaccessible
- Solution: bindgen auto-generates bindings from C headers; cargo build.rs integration; allowlist/blocklist APIs; override types; generate at build time
- Why It Matters: Eliminates 90% manual FFI work; keeps bindings synced with C headers; handles complex C types (unions, bitfields); essential for large C APIs
- Use Cases: Wrapping C libraries (SQLite, libcurl), system APIs, OpenGL/Vulkan, audio/video libs, embedded HALs, automatic binding generation


This chapter covers FFI (Foreign Function Interface)—calling C from Rust and vice versa. Rust's safety model differs from C: must use unsafe, manage memory carefully, handle strings/callbacks/errors correctly. Essential for integrating existing C libraries and system APIs.

## Table of Contents

1. [C ABI Compatibility](#pattern-1-c-abi-compatibility)
2. [String Conversions](#pattern-2-string-conversions)
3. [Callback Patterns](#pattern-3-callback-patterns)
4. [Error Handling Across FFI](#pattern-4-error-handling-across-ffi)
5. [bindgen Patterns](#pattern-5-bindgen-patterns)

---

## Pattern 1: C ABI Compatibility

**Problem**: Rust and C have incompatible ABIs—Rust optimizes struct layouts (reorders fields), uses different calling conventions, doesn't guarantee ABI stability between versions. Passing Rust struct to C corrupts data (wrong offsets). C expects specific memory layout. Function calls fail (wrong calling convention). Can't link against C libraries without ABI compatibility. Rust's Vec/String layout incompatible with C arrays/pointers.

**Solution**: Use extern "C" to declare C functions and mark Rust functions for C. Use #[repr(C)] to force C-compatible struct layout—fields in declaration order, C alignment rules. Use raw pointers (*const T, *mut T) for C pointers. Use c_int, c_char, c_void from std::ffi. All FFI calls in unsafe blocks. Link with #[link(name = "library")].

**Why It Matters**: Enables using decades of C libraries—SQLite, OpenSSL, zlib, libcurl. OS APIs (POSIX, Win32) are C. Can't rewrite everything: FFI is boundary between Rust and C. Incremental migration possible (gradual rewrite). Wrong ABI means silent corruption or crashes. Essential for systems programming, embedded, game engines, crypto.

**Use Cases**: System calls (open, read, write), database drivers (SQLite, Postgres FFI), graphics APIs (OpenGL, Vulkan, DirectX), audio/video codecs (ffmpeg), crypto libraries (OpenSSL, libsodium), compression (zlib, lz4), embedded HALs, legacy C code integration.

### C ABI Fundamentals

**Problem**: Understand ABI compatibility requirements between Rust and C.

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

## Pattern 2: String Conversions

**Problem**: Rust &str (UTF-8, length-prefixed, can contain NUL) incompatible with C *char (null-terminated, no encoding, stops at \0). Ownership unclear: who frees the string? Lifetime mismatches cause use-after-free. Rust String contains internal NUL—crashes C. C string has invalid UTF-8—Rust validation fails. Double-free or memory leak if ownership unclear.

**Solution**: Use CString to create owned null-terminated C strings. Use CStr to borrow C strings. CString::new() validates no internal NULs, adds terminator. as_ptr() for passing to C (borrow). into_raw() transfers ownership to C. CStr::from_ptr() for receiving from C (unsafe). to_string_lossy() handles invalid UTF-8. Validate encoding carefully.

**Why It Matters**: Strings are most error-prone FFI aspect—wrong conversion causes use-after-free, buffer overrun, null pointer deref, memory leaks. FFI boundaries in production: file paths, error messages, configuration. C's null-termination + Rust's UTF-8 = impedance mismatch. Essential for any C library accepting strings. Security: improper handling exploitable.

**Use Cases**: Passing filenames to C (fopen, stat), error messages from C (strerror), configuration strings, logging to C libraries, command-line arguments, environment variables, C string parsing, text processing across FFI.

### CString/CStr Pattern

**Problem**: Convert between Rust strings and C null-terminated strings safely.

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

## Pattern 3: Callback Patterns

**Problem**: C callbacks expect function pointers (extern "C" fn), but Rust closures capture environment (not C-compatible). Need stateful callbacks—closure with captured state called from C. Lifetime management: callback outlives state. Panics in callbacks unwind into C—undefined behavior. C has no concept of closures/borrowing. Thread safety issues.

**Solution**: Use extern "C" fn for stateless callbacks. For stateful: Box::into_raw() passes closure as void*, trampoline function extracts state. Store state in static/thread_local for global access. catch_unwind() prevents panics crossing FFI boundary. Use ManuallyDrop or forget for lifetime extension. Context pointer pattern (userdata).

**Why It Matters**: Async APIs need callbacks—C libraries can't block Rust futures. Signal handlers require callbacks. GUI frameworks event-driven. Thread APIs pass callbacks. Plugin systems. Panics crossing FFI are UB (crashes, corruption). Essential for event-driven C libraries. Must handle carefully or undefined behavior.

**Use Cases**: Event loops (GUI toolkits—GTK, Qt), async I/O libraries, signal handlers (SIGINT, SIGTERM), qsort comparators, thread spawn callbacks (pthread_create), plugin systems, C library hooks, timer callbacks.

### Function Pointer Callback Pattern

**Problem**: Register Rust function as C callback for simple stateless cases.

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

## Pattern 4: Error Handling Across FFI

**Problem**: Rust Result<T, E> and panic incompatible with C's errno/return codes/NULL. Panics unwinding into C are undefined behavior (crashes, memory corruption). Can't use ? operator across FFI boundary. C has no Result type—must translate. Error context lost crossing boundary. Errno is thread-local but C API unclear. No way to propagate detailed Rust errors to C.

**Solution**: Convert Result to i32 return codes (0 = success, <0 = error). Set errno for POSIX compatibility. Use catch_unwind() to prevent panics crossing FFI. Out-parameters for detailed error info (*mut c_int for errno). Error enum maps to C codes. thread_local! for per-thread error state. Document error contracts clearly.

**Why It Matters**: Panicking into C is undefined behavior—absolutely must prevent. Production FFI must never panic. C libraries expect specific error conventions—violating them breaks clients. Essential for reliability and safety. Without proper error handling, silent corruption or crashes. C API users need clear error semantics. Critical for all FFI.

**Use Cases**: All FFI functions (must handle errors properly), library wrappers (translate Result to errno), system calls, C callbacks (cannot panic), plugin interfaces, language bindings (Python, Ruby calling Rust), error propagation.

### Error Translation Pattern

**Problem**: Convert Rust Result to C error codes and prevent panics.

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

## Pattern 5: bindgen Patterns

**Problem**: Manually writing FFI bindings tedious—100+ functions, 50+ structs, error-prone. Struct layouts wrong (misaligned fields). C headers change—bindings out of sync. #define constants inaccessible from Rust. Complex C types (bitfields, unions, packed structs) hard to translate. Function pointer types verbose. Keeping bindings synchronized with C headers painful.

**Solution**: bindgen auto-generates Rust bindings from C headers. Parses with libclang (actual compiler). Generates extern "C" blocks, #[repr(C)] structs, constants. cargo build.rs integration—rebuilds when headers change. Allowlist/blocklist specific APIs. Override types for better Rust ergonomics. Handles complex C (unions, bitfields, macros).

**Why It Matters**: Eliminates 90% manual FFI work—hundreds of lines auto-generated. Keeps bindings in sync with C headers automatically. Handles complex C types correctly (unions, bitfields, packed). Catches type mismatches at build time. Essential for large C libraries. Industry standard (used by Firefox, Servo, many crates). Without bindgen, wrapping large C APIs impractical.

**Use Cases**: Wrapping C libraries (SQLite, libcurl, OpenSSL, SDL), system API bindings (libc, Win32), graphics APIs (OpenGL, Vulkan), audio/video libraries (ffmpeg), embedded HALs, automatic binding generation, maintaining C library wrappers.

### bindgen Setup Pattern

**Problem**: Automatically generate Rust bindings from C headers at build time.

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

---

## Summary

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

**ABI Compatibility**:
```rust
#[repr(C)]  // C struct layout
extern "C" fn foo() { }  // C calling convention
#[link(name = "mylib")]  // Link C library
```

**String Conversions**:
```rust
// Rust → C (borrow)
let c_str = CString::new("hello")?;
let ptr = c_str.as_ptr();

// C → Rust (unsafe)
let c_str = unsafe { CStr::from_ptr(ptr) };
let rust_str = c_str.to_str()?;
```

**Error Handling**:
```rust
#[no_mangle]
pub extern "C" fn my_fn() -> i32 {
    match std::panic::catch_unwind(|| {
        // ... work ...
        Ok(())
    }) {
        Ok(Ok(())) => 0,  // Success
        _ => -1,  // Error
    }
}
```

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
