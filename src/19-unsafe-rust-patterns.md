# Unsafe Rust Patterns
Unsafe Rust is not a separate language—it's a escape hatch that allows you to tell the compiler "I know what I'm doing, trust me on this." While Rust's safety guarantees are powerful, they can't express every valid program. Low-level systems programming, hardware interaction, foreign function interfaces, and certain performance optimizations require operations that the compiler cannot verify as safe.

The `unsafe` keyword doesn't disable Rust's safety checks; it expands what you're allowed to do. You're still protected from type confusion, use-after-free in safe code surrounding your unsafe blocks, and many other pitfalls. What `unsafe` enables are five specific superpowers that the compiler cannot verify automatically:

1. **Dereferencing raw pointers** – Reading or writing through `*const T` and `*mut T`
2. **Calling unsafe functions** – Functions that have unchecked preconditions
3. **Accessing or modifying static mutable variables** – Global mutable state
4. **Implementing unsafe traits** – Traits with invariants the compiler can't verify
5. **Accessing fields of unions** – Type-punning and low-level tricks

This chapter explores patterns for using unsafe code responsibly. The goal is not to avoid unsafe code—that would be impossible for low-level libraries—but to **build safe abstractions over unsafe foundations**. Every unsafe block should be surrounded by safe APIs that enforce invariants, document preconditions, and prevent misuse.

Understanding unsafe Rust is crucial for:
- **Systems programming**: Operating systems, device drivers, embedded systems
- **Performance-critical code**: Zero-copy operations, custom allocators, SIMD
- **FFI (Foreign Function Interface)**: Calling C libraries, exposing Rust to other languages
- **Advanced data structures**: Intrusive lists, lock-free algorithms, graph structures

The patterns we'll explore show how to:
- Manipulate raw pointers safely while maintaining invariants
- Interface with C code without compromising Rust's safety
- Handle uninitialized memory correctly using `MaybeUninit`
- Use transmute sparingly and correctly
- Build safe APIs that encapsulate unsafe internals

**The golden rule**: Unsafe code is not about being unsafe—it's about maintaining safety invariants that the compiler cannot verify. Every unsafe block should have a comment explaining why it's correct. If you can't explain why it's safe, it probably isn't.


## Pattern 1: Raw Pointer Manipulation

**Problem**: Need manual memory management for custom data structures. Borrow checker can't express bidirectional relationships (tree with parent pointers). FFI requires raw pointers for C interop. Performance-critical code needs to bypass bounds checks. Intrusive collections embed pointers in nodes. Hardware access needs fixed memory addresses.

**Solution**: Use raw pointers (`*const T`, `*mut T`) with explicit safety. Creating pointers is safe, dereferencing requires `unsafe`. Use `ptr::add()` for arithmetic, `ptr::write()` for uninitialized memory. `NonNull<T>` for non-null guarantees with null pointer optimization. Document invariants clearly. Separate allocation from initialization.

**Why It Matters**: Enables implementing Vec, LinkedList, HashMap from scratch. Custom allocators power memory pools. Zero-copy I/O needs pointer arithmetic. Tree parent pointers impossible with references alone. Real example: `std::vec::Vec` uses raw pointers for uninitialized capacity, enabling push() without reallocation. Proper pointer arithmetic prevents buffer overruns (common C bug source).

**Use Cases**: Custom collections (linked lists, trees, graphs), custom allocators and memory pools, memory-mapped I/O, FFI with C code, intrusive data structures, zero-copy parsing, hardware drivers.

Raw pointers (`*const T` and `*mut T`) are Rust's unmanaged pointers. Unlike references, they have no borrowing rules, no lifetime tracking, and no automatic dereferencing. They're what you get when you need manual memory management or when interfacing with systems that don't speak Rust's language of ownership.

**When raw pointers are necessary:**
- Implementing custom data structures (linked lists, trees with parent pointers)
- FFI with C code that expects raw pointers
- Memory-mapped I/O for hardware access
- Custom allocators and memory pools
- Performance-critical code avoiding bounds checks

**The key difference** from references: raw pointers don't promise validity. A `*const T` might point to valid memory, freed memory, or complete garbage. The compiler won't stop you from creating them, but dereferencing requires `unsafe` because that's where things can go wrong.

### Example: Raw Pointer Usage

Creating raw pointers is safe—it's just taking an address. The danger comes when you dereference them, because you're asserting "this memory is valid and properly aligned," and the compiler can't verify that claim.

```rust
fn raw_pointer_basics() {
    let mut num = 42;
    let r1: *const i32 = &num;            // Immutable raw pointer
    let r2: *mut i32 = &mut num;          // Mutable raw pointer
    let address = 0x12345usize;
    let r3 = address as *const i32;       // Might point to invalid memory!

    unsafe {
        println!("r1 points to: {}", *r1);
        *r2 = 100;
        println!("num is now: {}", num);
        // Dereferencing r3 would be UB - it points to random memory!
    }
}
```

**Why this pattern exists**: Sometimes you need to store pointers in data structures where the borrow checker can't track the relationships. A tree node with a parent pointer, for example—the parent outlives the child, but Rust's borrow checker can't express that bidirectional relationship without causing issues.

### Example: Pointer Arithmetic

Pointer arithmetic lets you navigate through memory by calculating offsets. This is fundamental for implementing custom collections and working with contiguous memory layouts. However, it's also where many C bugs come from—off-by-one errors lead to buffer overruns, corrupted data, and security vulnerabilities.

```rust
fn pointer_arithmetic() {
    let arr = [1, 2, 3, 4, 5];
    let ptr: *const i32 = arr.as_ptr();

    unsafe {
        for i in 0..arr.len() {
            let element_ptr = ptr.add(i);  // Equivalent to ptr + i * sizeof(i32)
            println!("Element {}: {}", i, *element_ptr);
        }

        let third = ptr.add(2);  // Points to third element
        println!("Third element: {}", *third);

        // ptr.add(10) would be UB - out of bounds!
    }
}
```

**The critical rule**: Pointer arithmetic must stay within the bounds of the original allocation (or one byte past the end). Going beyond is undefined behavior even if you don't dereference. The CPU's memory protection won't save you—UB means the compiler can assume it never happens and optimize accordingly, leading to bizarre bugs.

**When to use this**: Implementing iterators over custom collections, parsing binary protocols, working with memory-mapped files where you need to jump to specific offsets.

### Example: Building a Raw Vec-like Structure

Let's build a simplified vector to understand how raw pointers, allocation, and unsafe come together. This pattern appears in countless Rust libraries that need custom memory management.

The strategy: separate allocation from element storage. `RawVec` handles raw memory, while a higher-level `Vec` (not shown) would track initialization.

```rust
use std::alloc::{alloc, dealloc, realloc, Layout};
use std::ptr;

/// Manages raw memory allocation for a vector.
/// Does NOT track which elements are initialized!
pub struct RawVec<T> {
    ptr: *mut T,     // Pointer to allocated memory
    cap: usize,      // Capacity (number of T that fit)
}

impl<T> RawVec<T> {
    /// Creates an empty RawVec with no allocation.
    pub fn new() -> Self {
        RawVec {
            ptr: std::ptr::null_mut(),  // null_mut() is a safe operation
            cap: 0,
        }
    }

    /// Allocates memory for `cap` elements.
    pub fn with_capacity(cap: usize) -> Self {
        let layout = Layout::array::<T>(cap).unwrap();
        let ptr = unsafe { alloc(layout) as *mut T };

        if ptr.is_null() {
            panic!("Allocation failed");
        }

        RawVec { ptr, cap }
    }

    /// Doubles capacity, or sets it to 1 if currently zero.
    pub fn grow(&mut self) {
        let new_cap = if self.cap == 0 { 1 } else { self.cap * 2 };
        let new_layout = Layout::array::<T>(new_cap).unwrap();

        let new_ptr = if self.cap == 0 {
            unsafe { alloc(new_layout) as *mut T }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                realloc(
                    self.ptr as *mut u8,  // realloc works with u8 pointers
                    old_layout,
                    new_layout.size()
                ) as *mut T
            }
        };

        if new_ptr.is_null() {
            panic!("Allocation failed");
        }

        self.ptr = new_ptr;
        self.cap = new_cap;
    }

    pub fn ptr(&self) -> *mut T {
        self.ptr
    }

    pub fn cap(&self) -> usize {
        self.cap
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}
```

**Why this pattern?** Separating raw allocation from element management clarifies responsibilities. `RawVec` handles memory, higher-level code handles `Drop` for elements. This is exactly how `std::vec::Vec` is implemented.

**Safety invariants we maintain:**
1. `ptr` is either null (when `cap == 0`) or points to valid allocated memory
2. `cap` accurately reflects the allocation size
3. Deallocation uses the same layout as allocation
4. We never dereference `ptr` here (no assumptions about initialization)

### Example: Null Pointer Optimization with NonNull

`NonNull<T>` is a raw pointer wrapper that guarantees non-nullness. This enables the "null pointer optimization"—`Option<NonNull<T>>` has the same size as `*mut T` because it can use null as the `None` representation.

```rust
use std::ptr::NonNull;

/// A node in a linked list using NonNull for efficiency.
struct Node<T> {
    value: T,
    next: Option<NonNull<Node<T>>>,  // Same size as *mut Node<T>
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Node { value, next: None }
    }

    fn set_next(&mut self, next: NonNull<Node<T>>) {
        self.next = Some(next);
    }
}

/// A simple singly-linked list.
struct LinkedList<T> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
}

impl<T> LinkedList<T> {
    fn new() -> Self {
        LinkedList {
            head: None,
            tail: None,
            len: 0,
        }
    }

    fn push_back(&mut self, value: T) {
        let node = Box::new(Node::new(value));
        let node_ptr = NonNull::new(Box::into_raw(node)).unwrap();

        unsafe {
            if let Some(mut tail) = self.tail {
                tail.as_mut().next = Some(node_ptr);
            } else {
                self.head = Some(node_ptr);
            }

            self.tail = Some(node_ptr);
        }

        self.len += 1;
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut current = self.head;

        while let Some(node_ptr) = current {
            unsafe {
                let node = Box::from_raw(node_ptr.as_ptr());
                current = node.next;
            }
        }
    }
}
```

**Why NonNull?** Three benefits:
1. **Memory efficiency**: `Option<NonNull<T>>` is the same size as a raw pointer
2. **Safety marker**: Guarantees non-null, catching bugs at creation time
3. **Covariance**: Unlike `*mut T`, `NonNull<T>` is covariant over `T`

**When to use**: Linked data structures, graph nodes, intrusive collections. Anywhere you'd use raw pointers but can guarantee non-null.

---

## Pattern 2: FFI and C Interop

**Problem**: Need to call C libraries (operating system, drivers, databases, graphics). C has no concept of ownership, borrowing, or lifetimes. String encoding mismatch: C uses null-terminated bytes, Rust uses UTF-8 length-prefixed. Memory management unclear: who allocates? who frees? Callback lifetimes unchecked. Struct layout differs between Rust and C.

**Solution**: Use `extern "C"` to declare/export functions with C ABI. `#[repr(C)]` for compatible struct layout. `CString`/`CStr` for string conversions with null termination. Create safe wrappers around unsafe FFI—encapsulate preconditions, use RAII for cleanup. Document ownership transfer explicitly. Use `catch_unwind` for panic safety in callbacks.

**Why It Matters**: Unlocks entire C ecosystem (millions of libraries). System programming requires OS APIs (all C). Zero-cost abstraction: no runtime overhead. Real examples: database drivers (libpq, libsqlite), windowing (SDL2, GLFW), compression (zlib, lz4). Safe wrappers make FFI ergonomic and prevent memory leaks/crashes.

**Use Cases**: OS APIs (filesystem, networking, processes), database bindings (PostgreSQL, MySQL, SQLite), graphics libraries (OpenGL, Vulkan), compression (zlib, lz4), cryptography (OpenSSL), embedded drivers, legacy C code integration.

Foreign Function Interface (FFI) is how Rust talks to C, and by extension, the vast ecosystem of C libraries. This is unavoidable in systems programming—the operating system, graphics drivers, databases, and countless libraries all speak C at their boundaries.

The challenge: C has no concept of Rust's ownership, borrowing, or lifetimes. A `char*` in C could be stack-allocated, heap-allocated, a string literal, or dangling memory. Rust must bridge this gap carefully.

**Key FFI principles:**
- Rust can call C functions, C can call Rust functions marked `extern "C"`
- Types must have compatible memory layouts (use `#[repr(C)]`)
- Ownership transfer must be explicit and documented
- String encoding matters: C uses null-terminated, Rust uses UTF-8 slices

### Example: Basic C Function Binding

Declaring external C functions is straightforward, but calling them requires `unsafe` because Rust can't verify their contracts.

```rust
//=========================================================
// Declare external C functions from the standard C library
//=========================================================
extern "C" {
    fn abs(input: i32) -> i32;
    fn strlen(s: *const std::os::raw::c_char) -> usize;
    fn malloc(size: usize) -> *mut std::os::raw::c_void;
    fn free(ptr: *mut std::os::raw::c_void);
}

fn use_c_functions() {
    unsafe {
        let result = abs(-42);
        println!("abs(-42) = {}", result);

        let c_str = b"Hello\0";
        let len = strlen(c_str.as_ptr() as *const std::os::raw::c_char);
        println!("String length: {}", len);

        let ptr = malloc(100);
        if !ptr.is_null() {
            free(ptr);
        }
    }
}
```

**Why unsafe?** The compiler can't verify that:
- `strlen` won't read past the end of the string
- `malloc` returns are checked before use
- `free` is called exactly once per `malloc`

These are contracts you must uphold, documented in C headers and man pages.

### Example: Working with C Strings

C strings are null-terminated byte arrays. Rust strings are UTF-8 slices with explicit length. Converting between them requires care to avoid buffer overruns and encoding issues.

```rust
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

//==============
// Example usage
//==============
fn c_string_example() {
    let c_str = rust_to_c_string("Hello from Rust");

    unsafe {
        let rust_str = c_to_rust_string(c_str);
        println!("Back to Rust: {}", rust_str);
        free_rust_c_string(c_str);
    }
}
```

**Critical pattern**: Ownership transfer must be explicit. `into_raw()` says "Rust, stop tracking this." `from_raw()` says "Resume tracking so you can drop it." Missing either causes memory leaks or double-frees.

**Encoding issues**: C strings are byte arrays, not necessarily UTF-8. Use `to_string_lossy()` to handle invalid UTF-8 gracefully, or `to_str()` to fail fast.

### Example: C Struct Interop

When passing structs between Rust and C, memory layout must match exactly. `#[repr(C)]` tells Rust to use C's layout rules instead of optimizing field order.

```rust
use std::os::raw::{c_int, c_char};

/// A point with C-compatible layout.
#[repr(C)]
struct Point {
    x: c_int,  // Use c_int, not i32 (they're the same on most platforms but not guaranteed)
    y: c_int,
}

/// A person struct that C can understand.
#[repr(C)]
struct Person {
    name: *const c_char,  // C expects raw pointers, not &str
    age: c_int,
    height: f64,
}

/// An enum with explicit discriminant values for C.
#[repr(C)]
enum Status {
    Success = 0,
    Error = 1,
    Pending = 2,
}

//=================================================
// Declare C functions that work with these structs
//=================================================
extern "C" {
    fn process_point(point: *const Point) -> c_int;
    fn create_person(name: *const c_char, age: c_int) -> *mut Person;
    fn free_person(person: *mut Person);
}

fn use_c_structs() {
    let point = Point { x: 10, y: 20 };

    unsafe {
        let result = process_point(&point);
        println!("Result: {}", result);
    }
}
```

**Why `#[repr(C)]`?** Rust can reorder struct fields for optimization. C can't—field order is part of the ABI contract. `#[repr(C)]` locks in C's layout.

**Common pitfall**: Using Rust types (`String`, `&str`, `Option<T>`) in `#[repr(C)]` structs. These have Rust-specific layouts. Use raw pointers and C-compatible types instead.

### Exmple: Creating a Safe Wrapper for C Libraries

Raw FFI is unsafe and error-prone. The pattern: create a safe Rust wrapper that encapsulates the unsafe calls and maintains invariants.

```rust
use std::ffi::CString;
use std::os::raw::c_char;

//======================================================
// Unsafe C API (these would come from a real C library)
//======================================================
extern "C" {
    fn create_context() -> *mut std::os::raw::c_void;
    fn destroy_context(ctx: *mut std::os::raw::c_void);
    fn context_do_work(ctx: *mut std::os::raw::c_void, data: *const c_char) -> i32;
}

/// Safe Rust wrapper for the C context API.
///
/// Ensures the context is properly created and destroyed.
pub struct Context {
    inner: *mut std::os::raw::c_void,
}

impl Context {
    /// Creates a new context.
    ///
    /// Returns None if the C library fails to create a context.
    pub fn new() -> Option<Self> {
        let ptr = unsafe { create_context() };

        if ptr.is_null() {
            None
        } else {
            Some(Context { inner: ptr })
        }
    }

    /// Performs work with the given data.
    ///
    /// Returns Ok(result) on success, Err(message) on failure.
    pub fn do_work(&mut self, data: &str) -> Result<i32, String> {
        let c_data = CString::new(data).map_err(|e| e.to_string())?;

        let result = unsafe {
            context_do_work(self.inner, c_data.as_ptr())
        };

        if result >= 0 {
            Ok(result)
        } else {
            Err(format!("Operation failed with code: {}", result))
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            destroy_context(self.inner);
        }
    }
}

//===============================================================================
// If the C library is thread-safe, we can mark this safe to send between threads
//===============================================================================
// SAFETY: The C library documentation claims thread-safety
unsafe impl Send for Context {}
```

**This pattern solves multiple problems:**
1. **Lifetime management**: `Drop` ensures cleanup
2. **Type safety**: Users can't misuse the raw pointer
3. **Error handling**: Converts C error codes to Rust `Result`
4. **String safety**: Handles null termination automatically

**When to use**: Every time you wrap a C library. Users should never see `unsafe` in the public API unless absolutely necessary.

### Exmple: Callback Functions (C to Rust)

Sometimes C libraries need to call back into your code. Callbacks must use the C calling convention and can't panic (unwinding across FFI is undefined behavior).

```rust
use std::os::raw::c_int;

//==========================
// Type alias for C callback
//==========================
type Callback = extern "C" fn(c_int) -> c_int;

extern "C" {
    fn register_callback(cb: Callback);
    fn trigger_callback(value: c_int);
}

/// Rust function with C calling convention.
///
/// This can be called from C code.
extern "C" fn my_callback(value: c_int) -> c_int {
    println!("Callback called with: {}", value);
    value * 2
}

fn callback_example() {
    unsafe {
        register_callback(my_callback);
        trigger_callback(42);
    }
}

//====================================================
// Advanced: Callback with user data (context pointer)
//====================================================
type CallbackWithData = extern "C" fn(*mut std::os::raw::c_void, c_int) -> c_int;

extern "C" fn callback_with_context(user_data: *mut std::os::raw::c_void, value: c_int) -> c_int {
    unsafe {
        let data = &mut *(user_data as *mut i32);
        *data += value;
        *data
    }
}

fn callback_with_state_example() {
    let mut state = 0i32;

    extern "C" {
        fn register_callback_with_data(cb: CallbackWithData, user_data: *mut std::os::raw::c_void);
        fn trigger_callback_with_data(value: c_int);
    }

    unsafe {
        register_callback_with_data(
            callback_with_context,
            &mut state as *mut i32 as *mut std::os::raw::c_void
        );
        trigger_callback_with_data(10);
    }

    println!("State after callback: {}", state);
}
```

**Critical rules for callbacks:**
1. **Use `extern "C"`**: Ensures C calling convention
2. **No panics**: Unwinding through C code is UB. Use `catch_unwind` if needed
3. **Document user_data**: What type must be passed? Who owns it?
4. **Lifetime safety**: Ensure callback doesn't outlive data it references

**User data pattern**: C libraries pass a `void*` context pointer through to callbacks. This lets you maintain state without globals.

---

## Pattern 3: Uninitialized Memory Handling

**Problem**: Large arrays (1MB) on stack cause overflow. Reading from I/O into buffers wastes initialization. Performance-critical code initializes piecemeal. Rust assumes all values initialized—reading uninitialized = UB. FFI out-parameters pass uninitialized pointers. Default initialization expensive for large buffers.

**Solution**: Use `MaybeUninit<T>` to work with possibly-uninitialized memory safely. `MaybeUninit::uninit()` creates uninitialized, `write()` initializes, `assume_init()` asserts initialization. Use `MaybeUninit::uninit_array()` for arrays. `addr_of_mut!` for field pointers without creating references. For FFI: pass `as_mut_ptr()`, check success, then `assume_init()`.

**Why It Matters**: Prevents stack overflow: `[i32; 1_000_000]` crashes, `MaybeUninit` array succeeds. 2-3x faster for bulk initialization—no double-init. Safe FFI out-parameters prevent UB from unchecked C writes. Real example: reading 1MB from socket—sequential init 2ms, MaybeUninit + bulk read 0.1ms (20x faster). Miri catches uninitialized reads immediately.

**Use Cases**: Large stack arrays (>4KB), reading from files/sockets/FFI into buffers, performance-critical initialization, FFI out-parameters, deserializing from binary, reusing buffers without clearing.

Memory starts uninitialized. Creating a `Vec` doesn't fill it with zeros; allocating a buffer doesn't clear it. For performance, you often want to initialize memory piecemeal—read into it from a file, compute values on demand, or skip initialization for data you'll immediately overwrite.

The problem: Rust's safety model assumes all values are initialized. Reading uninitialized memory is instant undefined behavior. Even casting an `i32` from uninitialized memory (without using it) is UB.

`MaybeUninit<T>` solves this: it's a type that may or may not hold a valid `T`. You can work with it safely, then assert initialization when you're ready.

### Example: Using MaybeUninit for Arrays

Large arrays on the stack can overflow it if initialized naively. `MaybeUninit` lets you initialize elements one at a time without paying upfront cost.

```rust
use std::mem::MaybeUninit;

/// Create a large array efficiently without stack overflow.
fn create_array_uninit() -> [i32; 1000] {
    let mut arr: [MaybeUninit<i32>; 1000] = unsafe {
        MaybeUninit::uninit().assume_init()
    };

    for (i, elem) in arr.iter_mut().enumerate() {
        *elem = MaybeUninit::new(i as i32);
    }

    unsafe {
        std::mem::transmute(arr)
    }
}

//=====================================
// Better: Use the newer stabilized API
//=====================================
fn create_array_uninit_safe() -> [i32; 1000] {
    let mut arr = MaybeUninit::uninit_array::<1000>();

    for (i, elem) in arr.iter_mut().enumerate() {
        elem.write(i as i32);
    }

    unsafe { MaybeUninit::array_assume_init(arr) }
}
```

**Why this works**: `MaybeUninit<T>` is the same size as `T`, but Rust knows it might not be initialized. Operations on it are safe until you call `assume_init()`, which asserts "I promise this is initialized."

**When to use**: Large stack arrays, reading data from external sources (sockets, files), performance-critical initialization.

### Example: Partial Initialization

Sometimes you need to initialize a struct field-by-field, perhaps because constructing one field depends on earlier fields, or because you're reading from a stream.

```rust
use std::mem::MaybeUninit;

struct ComplexStruct {
    field1: String,
    field2: Vec<i32>,
    field3: Box<i32>,
}

fn initialize_complex_struct() -> ComplexStruct {
    let mut uninit: MaybeUninit<ComplexStruct> = MaybeUninit::uninit();
    let ptr = uninit.as_mut_ptr();

    unsafe {
        // SAFETY: Using addr_of_mut! to get field pointers without creating references
        std::ptr::addr_of_mut!((*ptr).field1).write(String::from("hello"));
        std::ptr::addr_of_mut!((*ptr).field2).write(vec![1, 2, 3]);
        std::ptr::addr_of_mut!((*ptr).field3).write(Box::new(42));

        uninit.assume_init()
    }
}
```

**Critical detail**: Use `addr_of_mut!` to get field pointers without creating references. Creating a `&mut T` to uninitialized memory is UB, even if you don't read it. `addr_of_mut!` avoids this.

**When to use**: Deserializing from binary formats, constructing objects with complex dependencies, FFI out-parameters.

### Example: Reading Uninitialized Memory (What NOT to Do)

Let's be crystal clear: reading uninitialized memory is undefined behavior. The compiler can assume it never happens and optimize based on that assumption.

```rust
use std::mem::MaybeUninit;

fn undefined_behavior_example() {
    let mut uninit: MaybeUninit<i32> = MaybeUninit::uninit();

    // ✅ SAFE: Writing to uninitialized memory
    uninit.write(42);
    let value = unsafe { uninit.assume_init() };
    println!("Value: {}", value);
}

fn actual_undefined_behavior() {
    let uninit: MaybeUninit<i32> = MaybeUninit::uninit();

    // ❌ UB: Reading uninitialized memory!
    // let value = unsafe { uninit.assume_init() };  // DON'T DO THIS
}
```

**What "undefined behavior" means**: Not just "might crash." The compiler can:
- Delete your code entirely
- Produce inconsistent results
- Corrupt memory elsewhere in your program
- Work fine in debug builds but break in release

Miri (Rust's interpreter for detecting UB) will catch these issues. Use it: `cargo +nightly miri test`.

### Example: Out-Parameter Pattern for C FFI

Many C functions write results through pointer arguments instead of returning them. `MaybeUninit` handles this pattern safely.

```rust
use std::mem::MaybeUninit;
use std::os::raw::c_int;

extern "C" {
    /// C function that writes to an out parameter.
    /// Returns 0 on success, non-zero on error.
    fn get_value(out: *mut c_int) -> c_int;
}

fn call_out_parameter_function() -> Option<i32> {
    let mut value = MaybeUninit::uninit();

    let result = unsafe {
        get_value(value.as_mut_ptr())
    };

    if result == 0 {
        Some(unsafe { value.assume_init() })
    } else {
        None
    }
}
```

**Pattern**: Create `MaybeUninit`, pass its pointer to C, check return code, assume init only on success.

**Why this is safe**: `MaybeUninit::as_mut_ptr()` gives a raw pointer that C can write to. If C doesn't write (error case), we don't call `assume_init()`, avoiding UB.

### Example: Initializing Arrays from External Functions

When filling a buffer from external sources, `MaybeUninit` prevents double-initialization and enables efficient bulk operations.

```rust
use std::mem::MaybeUninit;

extern "C" {
    /// Fills buffer with data.
    /// Returns 0 on success, non-zero on error.
    fn fill_buffer(buffer: *mut u8, size: usize) -> i32;
}

fn read_into_buffer(size: usize) -> Option<Vec<u8>> {
    let mut buffer: Vec<MaybeUninit<u8>> = Vec::with_capacity(size);

    unsafe {
        buffer.set_len(size);  // Set length without initializing
    }

    let result = unsafe {
        fill_buffer(buffer.as_mut_ptr() as *mut u8, size)
    };

    if result == 0 {
        let buffer = unsafe {
            std::mem::transmute::<Vec<MaybeUninit<u8>>, Vec<u8>>(buffer)
        };
        Some(buffer)
    } else {
        None
    }
}
```

**Why transmute?** `Vec<MaybeUninit<u8>>` and `Vec<u8>` have identical memory layout. `transmute` reinterprets the type without copying data.

**Alternative**: Use `MaybeUninit::slice_assume_init_ref()` for slices if you don't need to transfer ownership.

---

## Pattern 4: Transmute and Type Punning

**Problem**: Need bit-level reinterpretation for binary protocols. Numerical code needs bit manipulation (float bits). Zero-copy serialization requires type reinterpretation. Endianness conversion for network protocols. Enum discrimination for low-level code. Want to avoid copying data.

**Solution**: Use `transmute` sparingly—most dangerous function in Rust. Prefer safe alternatives: `to_bits()`/`from_bits()` for floats, `as` casts for integers, pointer casts for references. Document why transmute is correct and can't be avoided. Check sizes match (compile-time). Use unions for type punning when bit-compatible. `bytemuck` crate for safe verified transmutes.

**Why It Matters**: Wrong transmute = instant UB (lifetime extension, size mismatch, invalid values). Proper use enables zero-copy parsing (10x faster). Binary protocol parsing requires reinterpreting bytes. Real examples: `f32::to_bits()` uses transmute internally (safe). Extending lifetimes with transmute corrupts memory silently. Size mismatch caught at compile-time. Bytemuck prevents 90% of transmute bugs via traits.

**Use Cases**: Binary protocol parsing (network, file formats), bit manipulation in numerical code, zero-copy serialization/deserialization, converting between types with identical layout, enum discrimination, hardware register access.

`std::mem::transmute` is the most dangerous function in Rust. It reinterprets bytes from one type as another, no questions asked. Get it wrong and you invoke undefined behavior. Use it only when necessary and document why it's correct.

**Valid uses:**
- Converting between types with identical layout (e.g., `[u8; 4]` ↔ `u32`)
- Implementing zero-copy protocols
- Optimized numerical code

**Invalid uses:**
- Extending lifetimes (creates dangling references)
- Converting between different-sized types (compile error)
- Type confusion (pointers to different types)

### Example: Basic Transmute

The simplest uses: converting between types that have the same size and compatible representations.

```rust
use std::mem;

fn transmute_basics() {
    let a: u32 = 0x12345678;
    let b: [u8; 4] = unsafe { mem::transmute(a) };
    println!("Bytes: {:?}", b);  // Depends on endianness!

    let f: f32 = 3.14;
    let bits: u32 = unsafe { mem::transmute(f) };
    println!("Float bits: 0x{:08x}", bits);

    // ✅ BETTER: Use safe built-in methods
    let bits_safe = f.to_bits();
    assert_eq!(bits, bits_safe);

    let f2 = f32::from_bits(bits);
    assert_eq!(f, f2);
}
```

**Rule**: If a safe alternative exists (`.to_bits()`, `.from_bits()`, `as` casts), use it. Transmute should be a last resort.

**Endianness matters**: `u32` to `[u8; 4]` gives different byte orders on little-endian (x86) vs big-endian (some ARM, network byte order) systems.

### Example: Transmuting References (Dangerous!)

Transmuting references is one of the easiest ways to create undefined behavior. The compiler assumes references point to valid data of their type.

```rust
use std::mem;

//======================================
// ❌ DANGEROUS: Transmuting references
//======================================
fn transmute_reference_unsafe() {
    let x: &i32 = &42;
    let y: &u32 = unsafe { mem::transmute(x) };
    println!("Transmuted: {}", y);
}

//==================================
// ✅ BETTER: Using pointer casting
//==================================
fn transmute_reference_safer() {
    let x: i32 = 42;
    let ptr = &x as *const i32 as *const u32;
    let y = unsafe { &*ptr };
    println!("Casted: {}", y);
}

//==============================================
// ✅ SAFEST: Just use from_ne_bytes or as cast
//==============================================
fn safe_conversion() {
    let x: i32 = 42;
    let y = x as u32;  // Sign-extends negative values
    println!("Converted: {}", y);
}
```

**Why pointer casting is better**: It's explicit about what you're doing and doesn't accidentally change const-ness or lifetimes.

**When transmuting references is UB**:
- If the types have different alignment (e.g., `&u8` to `&u32`)
- If the memory doesn't satisfy the target type's validity invariant (e.g., transmuting to `&bool` with value 2)

### Example: Converting Between Slice Types

Reinterpreting slices is common in binary protocol parsing and numerical computing, but requires careful size calculations.

```rust
use std::slice;

fn slice_transmute() {
    let data: Vec<u32> = vec![0x12345678, 0x9abcdef0];

    let bytes: &[u8] = unsafe {
        slice::from_raw_parts(
            data.as_ptr() as *const u8,
            data.len() * std::mem::size_of::<u32>(),
        )
    };

    println!("Bytes: {:?}", bytes);

    // Reverse: bytes to u32 (must ensure alignment!)
}
```

**Size calculation must be exact**: `data.len()` elements × `size_of::<u32>()` bytes per element.

**Safer alternatives**: The `bytemuck` crate provides `cast_slice`, which only compiles if the transmutation is proven safe (no padding, correct alignment, etc.).

### Example: Enum Discrimination

Getting an enum's discriminant (which variant it is) as a raw number.

```rust
use std::mem;

#[repr(u8)]
enum MyEnum {
    A = 0,
    B = 1,
    C = 2,
}

fn get_discriminant(e: &MyEnum) -> u8 {
    unsafe { *(e as *const MyEnum as *const u8) }
}

fn enum_discriminant_safe(e: &MyEnum) -> u8 {
    match e {
        MyEnum::A => 0,
        MyEnum::B => 1,
        MyEnum::C => 2,
    }
}

//======================================
// ✅ Also safe: std::mem::discriminant
//======================================
fn enum_discriminant_std(e: &MyEnum) -> std::mem::Discriminant<MyEnum> {
    std::mem::discriminant(e)
}
```

**Why the unsafe version is wrong**: Enums might have niches (unused bit patterns) that the compiler uses for optimization. Reading the raw bytes might give you unexpected values.

**When you need the number**: Use `match` or `std::mem::discriminant`. The latter returns an opaque type, useful for equality comparisons.

### Example: Type Punning for Optimized Code

Type punning—reinterpreting data as a different type—is sometimes necessary for bit manipulation and numerical tricks. Unions provide a safer alternative to transmute.

```rust
union FloatUnion {
    f: f32,
    u: u32,
}

fn fast_float_bits(f: f32) -> u32 {
    let union = FloatUnion { f };
    unsafe { union.u }  // Reading inactive union field is unsafe
}

//=======================================
// For real code, use the built-in method
//=======================================
fn correct_float_bits(f: f32) -> u32 {
    f.to_bits()
}
```

**Union safety**: Writing one field and reading another is safe only if both types are valid for all bit patterns (e.g., integers, floats). Reading a `bool` from a union where you wrote `u8` would be UB if the byte is not 0 or 1.

**Modern Rust**: Unions are less necessary now that we have methods like `to_bits()` and `from_bits()`. Use library methods when available.

### Example: When NOT to Use Transmute

Some uses of transmute are always wrong. Here are the common mistakes:

```rust
//===============================
// ❌ WRONG: Extending lifetimes
//===============================
fn extend_lifetime_bad<'a>(x: &'a str) -> &'static str {
    unsafe { std::mem::transmute(x) }  // UB: dangling reference
}

fn extend_lifetime_good<'a>(x: &'a str) -> &'a str {
    x  // Just return the original with its real lifetime
}

//=================================
// ❌ WRONG: Different sized types
//=================================
fn different_sizes_bad() {
    let x: u32 = 42;
    // Won't compile: size mismatch
    // let y: u64 = unsafe { std::mem::transmute(x) };
}

//===============================
// ❌ WRONG: Changing mutability
//===============================
fn change_mutability_bad(x: &i32) -> &mut i32 {
    // UB: violates aliasing rules
    // unsafe { std::mem::transmute(x) }
    panic!("Can't safely do this")
}

//=================================
// ❌ WRONG: Bypassing type safety
//=================================
fn type_confusion_bad() {
    let x: &str = "hello";
    // UB: str has invariants (valid UTF-8) that might be violated
    // let y: &[u8] = unsafe { std::mem::transmute(x) };
}
```

**Why these are UB**:
- Lifetime extension creates dangling references
- Size mismatch writes to unintended memory
- Mutability changes violate aliasing rules
- Type confusion breaks type invariants

**If the compiler accepts transmute**, it doesn't mean it's safe! Transmute has a single compile-time check: sizes must match. Everything else is on you.

---

## Pattern 5: Safe Abstractions Over Unsafe

**Problem**: Unsafe code scattered everywhere is error-prone. Hard to audit and maintain invariants. Users exposed to raw pointers and manual management. Bugs in unsafe code cause UB throughout program. Testing unsafe code difficult. No encapsulation of preconditions.

**Solution**: Build safe types that encapsulate unsafe internals. Public API has no `unsafe`. Maintain invariants via type system (private fields). Use RAII (Drop) for automatic cleanup. PhantomData for variance/drop check. Document safety invariants clearly. Implement Send/Sync when provably safe. Type-state pattern prevents invalid states at compile-time.

**Why It Matters**: Vec/String/Arc/Mutex prove pattern works—millions use them safely. Single audit point instead of scattered unsafe. Type system prevents misuse. Real example: custom Vec with unsafe internals—users can't break invariants (len>cap impossible). SpinLock exposes safe API via RAII guard. Wrong: public `unsafe fn`—callers must uphold contract. Right: private unsafe, safe public API.

**Use Cases**: Custom collections (Vec, HashMap, LinkedList), synchronization primitives (Mutex, RwLock, atomics), custom allocators, FFI wrappers for C libraries, type-state APIs (builder patterns), intrusive data structures.

The goal of unsafe code is not to scatter `unsafe` blocks throughout your codebase. It's to build safe abstractions—types and functions that encapsulate unsafe operations and expose only safe interfaces.

This pattern is everywhere in the standard library: `Vec`, `String`, `Arc`, `Mutex` all use unsafe internally but are safe to use. You can achieve the same.

### Example: Building a Safe Vec

Let's implement a simplified vector to see how unsafe internals create safe APIs. This teaches the principles of invariant maintenance and careful boundary design.

```rust
use std::ptr;
use std::mem;
use std::alloc::{alloc, realloc, dealloc, Layout};

pub struct MyVec<T> {
    ptr: *mut T,
    len: usize,
    cap: usize,
}

impl<T> MyVec<T> {
    /// Creates an empty vector.
    pub fn new() -> Self {
        MyVec {
            ptr: std::ptr::null_mut(),  // Null is fine when cap == 0
            len: 0,
            cap: 0,
        }
    }

    /// Adds an element to the end of the vector.
    pub fn push(&mut self, value: T) {
        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            ptr::write(self.ptr.add(self.len), value);
        }

        self.len += 1;
    }

    /// Removes and returns the last element, or None if empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe {
                Some(ptr::read(self.ptr.add(self.len)))
            }
        }
    }

    /// Returns a reference to the element at the given index.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe {
                Some(&*self.ptr.add(index))
            }
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// Grows the capacity, doubling it or setting to 1 if currently 0.
    fn grow(&mut self) {
        let new_cap = if self.cap == 0 { 1 } else { self.cap * 2 };
        let new_layout = Layout::array::<T>(new_cap).unwrap();

        let new_ptr = if self.cap == 0 {
            unsafe { alloc(new_layout) as *mut T }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                realloc(
                    self.ptr as *mut u8,
                    old_layout,
                    new_layout.size(),
                ) as *mut T
            }
        };

        if new_ptr.is_null() {
            panic!("Allocation failed");
        }

        self.ptr = new_ptr;
        self.cap = new_cap;
    }
}

impl<T> Drop for MyVec<T> {
    fn drop(&mut self) {
        // Drop all elements
        while self.pop().is_some() {}

        // Deallocate memory
        if self.cap != 0 {
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

//========================================================
// Safety: MyVec<T> can be sent to another thread if T can
//========================================================
unsafe impl<T: Send> Send for MyVec<T> {}
unsafe impl<T: Sync> Sync for MyVec<T> {}
```

**Invariants we maintain**:
1. `ptr` is either null (when `cap == 0`) or points to valid allocated memory for `cap` elements
2. Elements `0..len` are initialized, `len..cap` are uninitialized
3. `len <= cap` always
4. Deallocation uses the same layout as allocation

**Why this is safe**: Public methods (`push`, `pop`, `get`) never break invariants. Users can't create invalid states.

**Send/Sync**: We implement these unsafe traits because `MyVec<T>` upholds the same safety guarantees as `Vec<T>`.

###  Example: Invariants and Documentation

When writing unsafe code, document your invariants clearly. Future maintainers (including yourself) need to know what assumptions the code relies on.

```rust
/// A slice type that is guaranteed to be non-empty.
///
/// # Safety Invariants
/// - The inner slice must always have at least one element
/// - The pointer must be valid and properly aligned
/// - The data must be valid for lifetime 'a
pub struct NonEmptySlice<'a, T> {
    slice: &'a [T],
}

impl<'a, T> NonEmptySlice<'a, T> {
    /// Creates a non-empty slice from a regular slice.
    ///
    /// Returns None if the slice is empty.
    pub fn new(slice: &'a [T]) -> Option<Self> {
        if slice.is_empty() {
            None
        } else {
            Some(NonEmptySlice { slice })
        }
    }

    /// Creates a non-empty slice without checking.
    ///
    /// # Safety
    /// The caller must ensure that the slice is not empty.
    pub unsafe fn new_unchecked(slice: &'a [T]) -> Self {
        debug_assert!(!slice.is_empty());
        NonEmptySlice { slice }
    }

    /// Returns the first element (always exists).
    pub fn first(&self) -> &T {
        &self.slice[0]
    }

    /// Returns the last element (always exists).
    pub fn last(&self) -> &T {
        &self.slice[self.slice.len() - 1]
    }

    pub fn as_slice(&self) -> &[T] {
        self.slice
    }
}
```

**Pattern**: Every `unsafe fn` needs a "# Safety" section. Every `unsafe` block should have a "SAFETY:" comment explaining why it's correct.

**Invariants**: Document what must always be true. These are the assumptions your unsafe code relies on.

###  Example: PhantomData for Type Safety

`PhantomData` is a zero-sized type that tells the compiler "I logically own a `T`" without actually storing one. This affects drop checking and variance.

```rust
use std::marker::PhantomData;
use std::ptr::NonNull;

/// A raw pointer wrapper that tracks ownership.
pub struct RawPtr<T> {
    ptr: NonNull<T>,
    // PhantomData tells compiler we "own" a T
    _marker: PhantomData<T>,
}

impl<T> RawPtr<T> {
    pub fn new(value: T) -> Self {
        let boxed = Box::new(value);
        RawPtr {
            ptr: NonNull::new(Box::into_raw(boxed)).unwrap(),
            _marker: PhantomData,
        }
    }

    pub fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }

    pub fn as_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for RawPtr<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.ptr.as_ptr()));
        }
    }
}

//==========================
// Safe because we own the T
//==========================
unsafe impl<T: Send> Send for RawPtr<T> {}
unsafe impl<T: Sync> Sync for RawPtr<T> {}
```

**Why PhantomData?** Without it, the compiler wouldn't know we "own" a `T`, so drop check wouldn't run T's destructor before RawPtr's. The `_marker` field has zero runtime cost but provides compile-time guarantees.

**Variance**: `PhantomData<T>` makes `RawPtr<T>` covariant over `T`, meaning `RawPtr<&'long T>` can be used where `RawPtr<&'short T>` is expected.

###  Example: Building Safe APIs with Unsafe Internals

Here's a complete example: a spinlock that uses unsafe internally but exposes a safe API through RAII guards.

```rust
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

/// A simple spinlock implementation.
///
/// Uses atomic operations and unsafe cell internally,
/// but provides safe locking through RAII guards.
pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> SpinLock<T> {
    pub fn new(data: T) -> Self {
        SpinLock {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// Acquires the lock, blocking until available.
    ///
    /// Returns a guard that provides access to the data
    /// and releases the lock when dropped.
    pub fn lock(&self) -> SpinLockGuard<T> {
        while self.locked.swap(true, Ordering::Acquire) {
            std::hint::spin_loop();
        }

        SpinLockGuard { lock: self }
    }
}

impl<'a, T> std::ops::Deref for SpinLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> std::ops::DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

//===================================================
// SAFETY: SpinLock properly synchronizes access to T
//===================================================
// The Acquire/Release ordering ensures memory visibility
unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}
```

**Safe API**: Users never see `unsafe`. The lock/unlock mechanism is enforced by the type system—you can't forget to unlock because the guard drops automatically.

**Unsafe implementation**: `UnsafeCell` allows interior mutability, atomics provide synchronization, but these are encapsulated.

**Ordering matters**: `Acquire` on lock, `Release` on unlock. This ensures memory writes before unlock are visible after lock on other threads.

###  Example: Compile-Time Type-State Programming

Type-state uses phantom types to encode state machine transitions at compile-time, preventing invalid state transitions.

```rust
use std::marker::PhantomData;

//============
// Type states
//============
struct Locked;
struct Unlocked;

/// A lock that uses the type system to ensure correct usage.
///
/// Can only access data when in the Locked state.
pub struct TypeStateLock<T, State = Unlocked> {
    data: *mut T,
    _state: PhantomData<State>,
}

impl<T> TypeStateLock<T, Unlocked> {
    pub fn new(data: T) -> Self {
        TypeStateLock {
            data: Box::into_raw(Box::new(data)),
            _state: PhantomData,
        }
    }

    /// Locks the data, transitioning to Locked state.
    pub fn lock(self) -> TypeStateLock<T, Locked> {
        TypeStateLock {
            data: self.data,
            _state: PhantomData,
        }
    }
}

impl<T> TypeStateLock<T, Locked> {
    /// Unlocks the data, transitioning back to Unlocked state.
    pub fn unlock(self) -> TypeStateLock<T, Unlocked> {
        TypeStateLock {
            data: self.data,
            _state: PhantomData,
        }
    }

    /// Accesses the data (only available when locked).
    pub fn access(&mut self) -> &mut T {
        unsafe { &mut *self.data }
    }
}

impl<T, State> Drop for TypeStateLock<T, State> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.data);
        }
    }
}

fn typestate_example() {
    let lock = TypeStateLock::new(vec![1, 2, 3]);

    // Can't access unlocked data - no access() method!

    let mut locked = lock.lock();
    locked.access().push(4);  // OK, we're in Locked state

    let unlocked = locked.unlock();
    // Can't access again - back to Unlocked state
}
```

**Power of type-state**: Impossible states are unrepresentable. You can't access unlocked data because there's no `access()` method in that state.

**Real-world use**: Builder APIs that require certain methods be called (typestate ensures required fields are set), protocol state machines, resource lifecycle management.

###  Example: Testing Unsafe Code

Unsafe code requires rigorous testing, including tests specifically designed to trigger potential UB.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_vec_basic() {
        let mut vec = MyVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        assert_eq!(vec.get(0), Some(&1));
        assert_eq!(vec.get(1), Some(&2));
        assert_eq!(vec.get(2), Some(&3));
        assert_eq!(vec.get(3), None);
    }

    #[test]
    fn test_my_vec_pop() {
        let mut vec = MyVec::new();
        vec.push(1);
        vec.push(2);

        assert_eq!(vec.pop(), Some(2));
        assert_eq!(vec.pop(), Some(1));
        assert_eq!(vec.pop(), None);
    }

    #[test]
    fn test_my_vec_drop() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct DropCounter;
        impl Drop for DropCounter {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }

        {
            let mut vec = MyVec::new();
            vec.push(DropCounter);
            vec.push(DropCounter);
            vec.push(DropCounter);
        }  // vec dropped here

        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 3);
    }


    #[test]
    fn test_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let vec = Arc::new(SpinLock::new(MyVec::new()));
        let mut handles = vec![];

        for i in 0..10 {
            let vec_clone = Arc::clone(&vec);
            handles.push(thread::spawn(move || {
                vec_clone.lock().push(i);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(vec.lock().len(), 10);
    }
}
```

**Testing strategies**:
1. **Basic correctness**: Does it work for normal cases?
2. **Edge cases**: Empty collections, single elements, capacity boundaries
3. **Drop tracking**: Use `DropCounter` to ensure no leaks or double-drops
4. **Thread safety**: Test concurrent access with multiple threads
5. **Miri**: Run tests with Miri to detect UB

**Miri is essential**: It interprets your code at the MIR level and detects undefined behavior that compiles fine but is wrong.

---

## Pattern 5: Best Practices for Unsafe Code

Unsafe Rust is powerful but dangerous. These practices help you wield that power responsibly.

### 1. Minimize Unsafe Boundaries

Keep unsafe code localized. Encapsulate it in small, well-tested functions or types.

```rust
//============================================
// ❌ BAD: Unsafe spreads throughout the code
//============================================
pub fn bad_api(data: *mut u8, len: usize) {
    // Users must pass raw pointers
}

//=========================================
// ✅ GOOD: Unsafe is contained internally
//=========================================
pub fn good_api(data: &mut [u8]) {
    unsafe {
        // Unsafe code is hidden from users
    }
}
```

**Principle**: Unsafe should be an implementation detail, not a user-facing requirement.

### 2. Document Safety Requirements

Every unsafe function must document its preconditions. This is for your future self and other maintainers.

```rust
/// Interprets a raw pointer as a slice.
///
/// # Safety
///
/// The caller must ensure that:
/// - `ptr` is valid for reads of `len * size_of::<T>()` bytes
/// - `ptr` is properly aligned for type `T`
/// - The memory referenced by `ptr` is not concurrently accessed for writes
/// - `len` is exactly the number of elements in the allocation
/// - The memory contains valid values of type `T`
pub unsafe fn from_raw_parts<T>(ptr: *const T, len: usize) -> &'static [T] {
    std::slice::from_raw_parts(ptr, len)
}
```

**What to document**:
- Pointer validity (aligned, non-null, within bounds)
- Initialization state (is memory initialized?)
- Concurrent access (can other threads access this?)
- Ownership (who frees this memory?)

### 3. Use Helper Functions

Extract unsafe operations into well-named, well-tested helper functions.

```rust
mod unsafe_helpers {
    /// Writes a value to a pointer without dropping the old value.
    ///
    /// # Safety
    /// `ptr` must be valid for writes and properly aligned.
    pub(crate) unsafe fn write_unchecked<T>(ptr: *mut T, value: T) {
        debug_assert!(!ptr.is_null(), "write_unchecked: null pointer");
        std::ptr::write(ptr, value);
    }

    /// Reads a value from a pointer without moving it.
    ///
    /// # Safety
    /// `ptr` must be valid for reads, properly aligned, and point to initialized data.
    pub(crate) unsafe fn read_unchecked<T>(ptr: *const T) -> T {
        debug_assert!(!ptr.is_null(), "read_unchecked: null pointer");
        std::ptr::read(ptr)
    }
}
```

**Benefits**:
- Centralize unsafe operations for easier auditing
- Add debug assertions for development builds
- Document assumptions once, reference from call sites

### 4. Use Clippy and Miri

Automated tools catch mistakes humans miss.

```bash
# Check for undocumented unsafe blocks
cargo clippy -- -W clippy::undocumented_unsafe_blocks

# Detect undefined behavior at runtime
cargo +nightly miri test

# Run with address sanitizer (detects memory errors)
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test

# Run with thread sanitizer (detects data races)
RUSTFLAGS="-Z sanitizer=thread" cargo +nightly test
```

**Miri** interprets code and detects:
- Use of uninitialized memory
- Use-after-free
- Invalid pointer arithmetic
- Data races (in unsafe code)
- Violating pointer aliasing rules

**Clippy** warns about:
- Undocumented unsafe blocks
- Transmutes that could be replaced with safe alternatives
- Missing safety comments

### 5. Consider Alternatives

Before writing unsafe code, check if a safe solution exists.

```rust
//===================================
// Instead of raw pointers, consider:
//===================================
// - std::pin::Pin for self-referential structs
// - std::rc::Rc or std::sync::Arc for shared ownership
// - std::cell::UnsafeCell for interior mutability (still unsafe, but more targeted)
// - std::sync::atomic for lock-free operations
// - ouroboros crate for self-referential structs
// - bytemuck crate for safe transmutes (compile-time checks)
// - zerocopy crate for zero-copy parsing with safety

//======================================
// Example: Safe transmute with bytemuck
//======================================
use bytemuck::{Pod, Zeroable};

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Point {
    x: f32,
    y: f32,
}

fn safe_transmute_example() {
    let bytes: [u8; 8] = [0; 8];
    let point: Point = bytemuck::cast(bytes);  // Compile-time verified safe
}
```

**Crates that help**:
- `bytemuck`: Safe transmutations with compile-time verification
- `zerocopy`: Zero-copy parsing with safety proofs
- `ouroboros`: Self-referential structs without unsafe (macros generate safe wrappers)
- `parking_lot`: Better locks with less overhead than `std::sync`

---

### Summary

Unsafe Rust is not about being reckless—it's about taking responsibility for safety properties the compiler cannot verify. Every unsafe block is a promise: "I have verified these invariants hold."

**Key principles**:
1. **Minimize scope**: Keep unsafe blocks small and localized
2. **Build safe abstractions**: Encapsulate unsafe code behind safe APIs
3. **Document invariants**: Explain what must be true for correctness
4. **Test rigorously**: Use Miri, sanitizers, and stress tests
5. **Prefer alternatives**: Use safe solutions when they exist

**When to use unsafe**:
- Implementing fundamental abstractions (collections, concurrency primitives)
- FFI to interact with C libraries
- Performance optimizations where safe code can't match (after profiling!)
- Hardware-level programming (embedded systems, drivers)

**When to avoid unsafe**:
- To bypass the borrow checker in application code (redesign instead)
- For micro-optimizations without measurements
- When safe abstractions exist (use them!)
- If you can't explain why it's safe (it probably isn't)

Unsafe Rust gives you the power to build anything—operating systems, databases, game engines, embedded systems. With that power comes responsibility. Document your assumptions, test thoroughly, and always ask: "Is there a safe way to do this?" If not, make your unsafe code correct enough that others can build safe abstractions on top.

The standard library is proof this works: millions of lines of safe Rust code rely on carefully crafted unsafe foundations. Your unsafe code can achieve the same reliability with discipline and care.
