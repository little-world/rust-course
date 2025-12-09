# Chapter 25: FFI & C Interop — Programming Projects

## Project 2: Expose a Rust Library as a C API (Opaque Handles, Strings, and Callbacks)

### Problem Statement

Build a Rust library that can be consumed from C. Design a minimal but realistic C API that manages an opaque handle, accepts C strings, returns error codes, and supports a user callback. Provide a small header (.h) design, implement the Rust side with `#[no_mangle] extern "C"` functions, and guarantee memory safety across the boundary.

### Why It Matters

- Most existing ecosystems speak C, not Rust. Shipping a Rust library behind a C ABI unlocks usage from C/C++, Python (ctypes), Java (JNI via C), Swift, etc.
- You’ll learn how to design FFI-safe types, avoid unwinding across FFI, marshal strings safely, and expose callbacks without leaking memory.

### Use Cases

- Embedding a Rust algorithmic kernel in a legacy C codebase.
- Providing a cross-language SDK for a Rust service/client.
- Incremental rewrite where Rust replaces a C component while preserving the public C API.

---

## Solution Outline (Didactic, not full implementation)

1) Define the target C API as a header sketch: opaque handle type, create/destroy, process, register_callback, last_error.
2) Implement the opaque handle in Rust with `#[repr(C)]` for FFI boundary types and an internal Rust struct.
3) Add C string interop: accept `*const c_char`, return error codes, expose `last_error()` thread-local buffer.
4) Add a callback registration that stores a raw function pointer plus user data pointer; invoke it on `process`.
5) Robustness/optimization: prevent panic across FFI, use `catch_unwind`, avoid allocations on the critical path.
6) Concurrency: make the handle `Send + Sync` only if the design supports it; document thread-safety. Add tests using multiple threads.

### Testing Hints

- From Rust, you can call your `extern "C"` functions directly to simulate a C caller.
- Property-test string inputs (valid UTF-8 vs. invalid bytes) and ensure error handling paths are covered.
- Use `loom` or standard `std::thread` tests to validate concurrency contracts.

---

## Milestone 1: Sketch the C API (Header) and Error Model

### Introduction
We start by writing the API we wish to expose to C. This forces clarity around ownership, lifetimes, and error reporting.

Why previous step is not enough: There is no API surface yet; we need a stable ABI and simple contracts before coding.

### Architecture

- Structs/Traits: none in Rust yet.
- Functions (C header sketch):
  - `my_handle_t* my_create(void);`
  - `void my_destroy(my_handle_t* h);`
  - `int my_process(my_handle_t* h, const char* input); // 0=OK, nonzero=error`
  - `void my_register_callback(my_handle_t* h, void(*cb)(const char* msg, void* user), void* user);`
  - `const char* my_last_error(void); // thread-local error message`
  - Role: Minimal lifecycle, processing, callbacks, and diagnostics.

### Checkpoint Tests

```rust
// Ensure chosen error codes are stable and documented
const OK: i32 = 0; const ERR_NULL: i32 = 1; const ERR_UTF8: i32 = 2;
assert_eq!(OK, 0);
```

### Starter Code

```c
// mylib.h (sketch)
typedef struct my_handle_t my_handle_t;

int my_version_major(void);
my_handle_t* my_create(void);
void my_destroy(my_handle_t*);
int my_process(my_handle_t*, const char* input);
void my_register_callback(my_handle_t*, void(*cb)(const char* msg, void* user), void* user);
const char* my_last_error(void);
```

---

## Milestone 2: Opaque Handle Implementation in Rust

### Introduction
Create the internal Rust struct and expose `create/destroy` with `#[no_mangle] extern "C"`.

Why previous step is not enough: The header is aspirational; we need a concrete, memory-safe representation.

### Architecture

- Structs:
  - `struct Handle { last_msg: String, cb: Option<Callback>, }`
  - `#[repr(C)] struct Callback { func: extern "C" fn(*const i8, *mut core::ffi::c_void), user: *mut core::ffi::c_void }`
- Functions:
  - `my_create() -> *mut Handle`
  - `my_destroy(*mut Handle)`
  - Role: Manage allocation and deallocation safely.

### Checkpoint Tests

```text
let h = unsafe { my_create() };
assert!(!h.is_null());
unsafe { my_destroy(h) };
```

### Starter Code

```rust
use core::ffi::c_void;

#[repr(C)]
pub struct Callback {
    pub func: extern "C" fn(*const i8, *mut c_void),
    pub user: *mut c_void,
}

pub struct Handle {
    last_msg: String,
    cb: Option<Callback>,
}

#[no_mangle]
pub extern "C" fn my_create() -> *mut Handle {
    Box::into_raw(Box::new(Handle { last_msg: String::new(), cb: None }))
}

#[no_mangle]
pub extern "C" fn my_destroy(h: *mut Handle) {
    if h.is_null() { return; }
    unsafe { drop(Box::from_raw(h)); }
}
```

---

## Milestone 3: C String Interop and Error Codes

### Introduction
Implement `my_process` that accepts `const char*`, validates UTF-8 (or handles bytes), and stores messages/errors.

Why previous step is not enough: Without string marshaling and errors, the API is not usable from C.

### Architecture

- Functions:
  - `my_process(h: *mut Handle, input: *const i8) -> i32` — returns error codes.
  - `my_last_error() -> *const i8` — thread-local buffer for the last error message.
- Roles: Safe conversion with `CStr`, prevent panics, consistent diagnostics.

### Checkpoint Tests

```text
use std::ffi::CString;
let h = unsafe { my_create() };
let ok = unsafe { my_process(h, CString::new("hello").unwrap().as_ptr()) };
assert_eq!(ok, 0);
unsafe { my_destroy(h) };
```

### Starter Code

```rust
use std::ffi::{CStr, CString};
use std::cell::RefCell;

thread_local! { static ERR: RefCell<Option<CString>> = RefCell::new(None); }

fn set_err(msg: &str) -> i32 {
    ERR.with(|e| *e.borrow_mut() = Some(CString::new(msg).unwrap()));
    1 // nonzero = error
}

#[no_mangle]
pub extern "C" fn my_last_error() -> *const i8 {
    ERR.with(|e| e.borrow().as_ref().map(|s| s.as_ptr()).unwrap_or(core::ptr::null()))
}

#[no_mangle]
pub extern "C" fn my_process(h: *mut Handle, input: *const i8) -> i32 {
    if h.is_null() { return set_err("null handle"); }
    if input.is_null() { return set_err("null input"); }
    let s = unsafe { CStr::from_ptr(input) };
    let s = match s.to_str() { Ok(v) => v, Err(_) => return set_err("invalid utf8") };
    let handle = unsafe { &mut *h };
    handle.last_msg = format!("processed:{}", s);
    0
}
```

---

## Milestone 4: Register and Invoke Callbacks

### Introduction
Enable C callers to provide a callback to be invoked on `my_process`.

Why previous step is not enough: Many real APIs are event-driven; we need callback support and user data passthrough.

### Architecture

- Functions:
  - `my_register_callback(h: *mut Handle, cb: Option<extern "C" fn(*const i8, *mut c_void)>, user: *mut c_void)`
  - Role: Store the function pointer and user data; call it with a message.

### Checkpoint Tests

```text
extern "C" fn cb(msg: *const i8, user: *mut core::ffi::c_void) {
    let s = unsafe { std::ffi::CStr::from_ptr(msg) }.to_str().unwrap();
    assert!(s.contains("processed:"));
    assert!(!user.is_null());
}
let h = unsafe { my_create() };
unsafe { my_register_callback(h, Some(cb), 0x1 as *mut _) };
let ok = unsafe { my_process(h, std::ffi::CString::new("x").unwrap().as_ptr()) };
assert_eq!(ok, 0);
unsafe { my_destroy(h) };
```

### Starter Code

```rust
#[no_mangle]
pub extern "C" fn my_register_callback(h: *mut Handle, cb: Option<extern "C" fn(*const i8, *mut c_void)>, user: *mut c_void) {
    if h.is_null() { return; }
    let handle = unsafe { &mut *h };
    handle.cb = cb.map(|func| Callback { func, user });
}

fn invoke_cb(handle: &Handle, msg: &str) {
    if let Some(cb) = &handle.cb {
        if let Ok(cs) = std::ffi::CString::new(msg) {
            (cb.func)(cs.as_ptr(), cb.user);
        }
    }
}
```

---

## Milestone 5: Robustness, Unwind Safety, and Performance

### Introduction
Harden the API: don’t unwind across FFI, minimize allocations, and document thread-safety.

Why previous step is not enough: Panics across FFI are UB; extra allocations and unclear sync rules harm reliability and speed.

### Architecture

- Functions:
  - Wrap `my_process` body with `catch_unwind` and map panic to error.
  - Pre-allocate buffers or reuse `last_msg` to avoid repeated allocations.
- Role: Safety and performance improvements.

### Checkpoint Tests

- Simulate a panic path and verify it returns an error code and sets `last_error`.
- Benchmark `my_process` with and without callbacks.

### Starter Code

```rust
#[no_mangle]
pub extern "C" fn my_process_safe(h: *mut Handle, input: *const i8) -> i32 {
    let res = std::panic::catch_unwind(|| unsafe { my_process(h, input) });
    match res { Ok(code) => code, Err(_) => set_err("panic across FFI") }
}
```

---

## Milestone 6: Concurrency and Thread Safety

### Introduction
Make decisions about `Send`/`Sync` for your handle. Protect interior state with `Mutex` if sharing across threads.

Why previous step is not enough: Without a concurrency contract, multithreaded C callers may cause races or UB.

Improvement: Add `Arc<Mutex<...>>` inside the handle or document single-threaded usage only. Optimize with `parking_lot` if needed.

### Testing Hints

- Spawn several Rust threads calling the C API concurrently; use random inputs; look for races and deadlocks.

---

## Complete Working Example

```rust
use core::ffi::c_void;
use std::ffi::{CStr, CString};
use std::cell::RefCell;

#[repr(C)]
pub struct Callback {
    pub func: extern "C" fn(*const i8, *mut c_void),
    pub user: *mut c_void,
}

pub struct Handle {
    last_msg: String,
    cb: Option<Callback>,
}

thread_local! { static ERR: RefCell<Option<CString>> = RefCell::new(None); }

fn set_err(msg: &str) -> i32 {
    ERR.with(|e| *e.borrow_mut() = Some(CString::new(msg).unwrap()));
    1
}

#[no_mangle]
pub extern "C" fn my_last_error() -> *const i8 {
    ERR.with(|e| e.borrow().as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()))
}

#[no_mangle]
pub extern "C" fn my_create() -> *mut Handle {
    Box::into_raw(Box::new(Handle { last_msg: String::new(), cb: None }))
}

#[no_mangle]
pub extern "C" fn my_destroy(h: *mut Handle) { if !h.is_null() { unsafe { drop(Box::from_raw(h)); } } }

#[no_mangle]
pub extern "C" fn my_register_callback(h: *mut Handle, cb: Option<extern "C" fn(*const i8, *mut c_void)>, user: *mut c_void) {
    if h.is_null() { return; }
    unsafe { &mut *h }.cb = cb.map(|func| Callback { func, user });
}

fn invoke_cb(handle: &Handle, msg: &str) {
    if let Some(cb) = &handle.cb { if let Ok(cs) = CString::new(msg) { (cb.func)(cs.as_ptr(), cb.user); } }
}

#[no_mangle]
pub extern "C" fn my_process(h: *mut Handle, input: *const i8) -> i32 {
    if h.is_null() { return set_err("null handle"); }
    if input.is_null() { return set_err("null input"); }
    let s = unsafe { CStr::from_ptr(input) };
    let s = match s.to_str() { Ok(v) => v, Err(_) => return set_err("invalid utf8") };
    let handle = unsafe { &mut *h };
    handle.last_msg.clear();
    handle.last_msg.push_str("processed:");
    handle.last_msg.push_str(s);
    invoke_cb(handle, &handle.last_msg);
    0
}

#[no_mangle]
pub extern "C" fn my_process_safe(h: *mut Handle, input: *const i8) -> i32 {
    match std::panic::catch_unwind(|| unsafe { my_process(h, input) }) { Ok(code) => code, Err(_) => set_err("panic across FFI") }
}

fn main() {
    extern "C" fn cb(msg: *const i8, _user: *mut c_void) {
        let s = unsafe { CStr::from_ptr(msg) }.to_str().unwrap();
        println!("CALLBACK: {}", s);
    }
    let h = unsafe { my_create() };
    unsafe { my_register_callback(h, Some(cb), std::ptr::null_mut()) };
    let input = CString::new("hello").unwrap();
    let code = unsafe { my_process_safe(h, input.as_ptr()) };
    assert_eq!(code, 0);
    unsafe { my_destroy(h) };
}
```

---

## Milestone 7: Python Bindings and Usage (ctypes + callbacks)

### Introduction
Bind the exposed C API to Python using `ctypes`. We will load the shared library, declare function prototypes, and demonstrate calling `my_create`, `my_process`, `my_last_error`, and registering a Python callback that Rust invokes.

Why previous step is not enough: The API is only consumable from C/Rust. Many users want to script and test from Python, including receiving callbacks. This milestone shows how to bridge both directions safely.

### Architecture

- Python `ctypes` declarations for:
  - `my_create() -> *mut Handle`
  - `my_destroy(*mut Handle)`
  - `my_process(*mut Handle, *const i8) -> i32`
  - `my_last_error() -> *const i8`
  - `my_register_callback(*mut Handle, cb: extern "C" fn(*const i8, *mut c_void), user: *mut c_void)`
- Use `ctypes.CFUNCTYPE` to build a C-callable function pointer from a Python function.
- GIL: Python holds the GIL while your callback runs; avoid long computations or use threads.

### Checkpoint Tests (Python)

```python
# test_mylib_py.py
import ctypes as ct, os

def load() -> ct.CDLL:
    name = os.environ.get("RUST_FFI_LIB", "./target/release/libffi_c_examples.dylib")
    lib = ct.CDLL(name)
    # typedef struct Handle Handle; // opaque in C, use void* in ctypes
    void_p = ct.c_void_p
    c_char_p = ct.c_char_p
    c_int = ct.c_int
    c_void_p = ct.c_void_p

    lib.my_create.argtypes = ()
    lib.my_create.restype = void_p

    lib.my_destroy.argtypes = (void_p,)
    lib.my_destroy.restype = None

    lib.my_process.argtypes = (void_p, c_char_p)
    lib.my_process.restype = c_int

    lib.my_last_error.argtypes = ()
    lib.my_last_error.restype = ct.c_char_p  # const char* -> bytes

    CALLBACK = ct.CFUNCTYPE(None, ct.c_char_p, c_void_p)
    lib.my_register_callback.argtypes = (void_p, CALLBACK, c_void_p)
    lib.my_register_callback.restype = None

    return lib, CALLBACK

def test_process_and_callback():
    lib, CALLBACK = load()
    events = []
    def py_cb(msg_ptr, user):
        msg = ct.cast(msg_ptr, ct.c_char_p).value.decode('utf8')
        events.append((msg, user))

    cb = CALLBACK(py_cb)
    h = lib.my_create()
    try:
        lib.my_register_callback(h, cb, ct.c_void_p(0x1234))
        rc = lib.my_process(h, b"hello from python")
        assert rc == 0
        assert events and events[0][0].startswith("processed:")
    finally:
        lib.my_destroy(h)

def test_error_path():
    lib, _ = load()
    rc = lib.my_process(ct.c_void_p(), b"ignored")  # null handle
    assert rc != 0
    msg = lib.my_last_error()
    assert msg is not None and b"null handle" in msg
```

### Starter Code (No Rust changes required)

You already exposed the necessary `extern "C"` functions in previous milestones. Ensure the library is built as a `cdylib` and that symbols are exported.

```toml
[lib]
name = "ffi_c_examples"
crate-type = ["cdylib"]
```

Build and locate the shared library:
- macOS: `target/release/libffi_c_examples.dylib`
- Linux: `target/release/libffi_c_examples.so`
- Windows: `target\\release\\ffi_c_examples.dll`

### Why this step improves things
- Validates your C ABI from a high-level language.
- Demonstrates safe callback interop: Python function -> C function pointer -> called by Rust.
- Provides a fast feedback loop for testing, including error handling.

### Testing Hints
- Use `pytest` to run the Python tests above.
- Add a test that intentionally passes invalid UTF-8 (e.g., `b"\xff"`) and assert a nonzero error with `my_last_error` containing `invalid utf8`.
- For concurrency, register a callback and call `my_process` from multiple Python threads; verify no crashes and expected sequencing if documented.
