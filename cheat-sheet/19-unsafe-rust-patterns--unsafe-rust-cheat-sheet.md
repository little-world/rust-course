### Unsafe Rust Cheat Sheet

```rust
// ============================================================================
// THE FIVE UNSAFE SUPERPOWERS
// ============================================================================

// 1. Dereference raw pointers
let ptr: *const i32 = &42;
unsafe { println!("{}", *ptr); }

// 2. Call unsafe functions
unsafe { some_unsafe_fn(); }

// 3. Access mutable statics
static mut COUNTER: i32 = 0;
unsafe { COUNTER += 1; }

// 4. Implement unsafe traits
unsafe impl Send for MyType {}
unsafe impl Sync for MyType {}

// 5. Access union fields
union MyUnion { i: i32, f: f32 }
let u = MyUnion { i: 42 };
unsafe { println!("{}", u.f); }

// ============================================================================
// RAW POINTERS
// ============================================================================

// Creating raw pointers (SAFE - no unsafe needed)
let x = 42;
let ptr_const: *const i32 = &x;           // Immutable raw pointer
let ptr_mut: *mut i32 = &x as *const i32 as *mut i32;  // Mutable raw pointer
let null_ptr: *const i32 = std::ptr::null();
let null_mut: *mut i32 = std::ptr::null_mut();

// Dereferencing (UNSAFE)
unsafe {
    let val = *ptr_const;                  // Read through pointer
    *ptr_mut = 100;                        // Write through pointer
}

// Pointer arithmetic (UNSAFE to dereference result)
unsafe {
    let arr = [1, 2, 3, 4, 5];
    let ptr = arr.as_ptr();
    let third = ptr.add(2);                // Points to arr[2]
    let first = third.sub(2);              // Back to arr[0]
    let offset = ptr.offset(3);            // Points to arr[3] (signed offset)
}

// NonNull - guaranteed non-null pointer
use std::ptr::NonNull;
let x = 42;
let nn: NonNull<i32> = NonNull::new(&x as *const _ as *mut _).unwrap();
// Option<NonNull<T>> has same size as *mut T (null pointer optimization)

// Pointer methods
ptr.is_null()                              // Check for null
ptr.as_ref()                               // Option<&T> (unsafe)
ptr_mut.as_mut()                           // Option<&mut T> (unsafe)
std::ptr::read(ptr)                        // Read without moving (unsafe)
std::ptr::write(ptr, val)                  // Write without dropping old (unsafe)
std::ptr::copy(src, dst, count)            // memcpy (unsafe)
std::ptr::copy_nonoverlapping(src, dst, n) // memcpy non-overlapping (unsafe)
std::ptr::swap(a, b)                       // Swap values at pointers (unsafe)
std::ptr::drop_in_place(ptr)               // Drop value at pointer (unsafe)

// ============================================================================
// FFI (FOREIGN FUNCTION INTERFACE)
// ============================================================================

// Declare external C functions
extern "C" {
    fn abs(input: i32) -> i32;
    fn strlen(s: *const std::os::raw::c_char) -> usize;
    fn malloc(size: usize) -> *mut std::os::raw::c_void;
    fn free(ptr: *mut std::os::raw::c_void);
}

// Export Rust function for C
#[no_mangle]
pub extern "C" fn rust_function(x: i32) -> i32 {
    x * 2
}

// C-compatible struct layout
#[repr(C)]
struct Point { x: i32, y: i32 }

// C-compatible enum
#[repr(C)]
enum Status { Ok = 0, Error = 1 }

// C-compatible union
#[repr(C)]
union Value { i: i32, f: f32 }

// repr options
#[repr(C)]              // C-compatible layout
#[repr(transparent)]    // Same layout as single field
#[repr(packed)]         // No padding (may cause UB on some platforms)
#[repr(align(N))]       // Minimum alignment of N bytes
#[repr(u8)]             // Enum discriminant is u8
#[repr(i32)]            // Enum discriminant is i32

// ============================================================================
// C STRING CONVERSIONS
// ============================================================================

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Rust String → C string (allocates, adds null terminator)
let rust_str = "hello";
let c_string = CString::new(rust_str).unwrap();
let c_ptr: *const c_char = c_string.as_ptr();

// Transfer ownership to C (caller must free)
let owned_ptr = c_string.into_raw();
// Later: retake ownership and free
let _ = unsafe { CString::from_raw(owned_ptr) };

// C string → Rust (borrowing, no allocation)
unsafe {
    let c_str = CStr::from_ptr(c_ptr);
    let rust_str: &str = c_str.to_str().unwrap();        // Fails if not UTF-8
    let rust_string: String = c_str.to_string_lossy().into_owned(); // Replaces invalid UTF-8
}

// ============================================================================
// MAYBEUNINIT - UNINITIALIZED MEMORY
// ============================================================================

use std::mem::MaybeUninit;

// Create uninitialized value
let mut uninit: MaybeUninit<i32> = MaybeUninit::uninit();

// Initialize it
uninit.write(42);

// Extract initialized value (UNSAFE - must be initialized first!)
let value: i32 = unsafe { uninit.assume_init() };

// Uninitialized array
let mut arr: [MaybeUninit<i32>; 100] = unsafe { MaybeUninit::uninit().assume_init() };
for (i, elem) in arr.iter_mut().enumerate() {
    elem.write(i as i32);
}
let arr: [i32; 100] = unsafe { std::mem::transmute(arr) };

// Field-by-field initialization
struct MyStruct { a: String, b: Vec<i32> }
let mut uninit: MaybeUninit<MyStruct> = MaybeUninit::uninit();
let ptr = uninit.as_mut_ptr();
unsafe {
    std::ptr::addr_of_mut!((*ptr).a).write(String::new());
    std::ptr::addr_of_mut!((*ptr).b).write(vec![]);
    uninit.assume_init()
};

// Out-parameter pattern for FFI
extern "C" { fn get_value(out: *mut i32) -> i32; }
fn call_ffi() -> Option<i32> {
    let mut out = MaybeUninit::uninit();
    let result = unsafe { get_value(out.as_mut_ptr()) };
    if result == 0 { Some(unsafe { out.assume_init() }) } else { None }
}

// ============================================================================
// TRANSMUTE - TYPE REINTERPRETATION
// ============================================================================

use std::mem::transmute;

// Basic transmute (sizes MUST match - compile error otherwise)
let x: u32 = 0x12345678;
let bytes: [u8; 4] = unsafe { transmute(x) };

// Float bits (PREFER the safe alternatives!)
let f: f32 = 3.14;
let bits: u32 = unsafe { transmute(f) };  // ❌ Works but prefer:
let bits: u32 = f.to_bits();               // ✅ Safe!
let f: f32 = f32::from_bits(bits);         // ✅ Safe!

// ❌ DANGEROUS - DON'T DO THESE:
// Extending lifetimes (creates dangling references)
// fn bad<'a>(x: &'a str) -> &'static str { unsafe { transmute(x) } }

// Changing mutability (violates aliasing)
// fn bad(x: &i32) -> &mut i32 { unsafe { transmute(x) } }

// Different sizes (compile error, but attempted = bug)
// let x: u32 = 42; let y: u64 = unsafe { transmute(x) };

// ============================================================================
// SAFE ALTERNATIVES TO TRANSMUTE
// ============================================================================

// Integer conversions
let x: i32 = -1;
let y: u32 = x as u32;                     // as cast (may change bits for signed)
let y: u32 = x.to_ne_bytes().try_into().map(u32::from_ne_bytes).unwrap();

// Float ↔ bits
let f: f32 = 3.14;
let bits = f.to_bits();                    // f32 → u32
let f = f32::from_bits(bits);              // u32 → f32

// Slice reinterpretation (use bytemuck crate for safety)
use std::slice;
let data: &[u32] = &[1, 2, 3];
let bytes: &[u8] = unsafe {
    slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4)
};

// ============================================================================
// BUILDING SAFE ABSTRACTIONS
// ============================================================================

// Pattern: Safe wrapper around unsafe FFI
pub struct SafeWrapper {
    ptr: *mut std::os::raw::c_void,
}

impl SafeWrapper {
    pub fn new() -> Option<Self> {
        extern "C" { fn create() -> *mut std::os::raw::c_void; }
        let ptr = unsafe { create() };
        if ptr.is_null() { None } else { Some(Self { ptr }) }
    }

    pub fn do_something(&mut self) -> Result<(), String> {
        extern "C" { fn operation(p: *mut std::os::raw::c_void) -> i32; }
        let result = unsafe { operation(self.ptr) };
        if result == 0 { Ok(()) } else { Err("Operation failed".into()) }
    }
}

impl Drop for SafeWrapper {
    fn drop(&mut self) {
        extern "C" { fn destroy(p: *mut std::os::raw::c_void); }
        unsafe { destroy(self.ptr); }
    }
}

// ============================================================================
// PHANTOMDATA - TYPE MARKERS
// ============================================================================

use std::marker::PhantomData;

// Indicate ownership without storing value
struct MyBox<T> {
    ptr: *mut T,
    _marker: PhantomData<T>,  // "I own a T" for drop check
}

// Indicate lifetime dependency
struct Ref<'a, T> {
    ptr: *const T,
    _marker: PhantomData<&'a T>,  // "I borrow a T for 'a"
}

// Variance control
struct Invariant<T> {
    _marker: PhantomData<fn(T) -> T>,  // Invariant over T
}

struct Covariant<T> {
    _marker: PhantomData<T>,           // Covariant over T
}

struct Contravariant<T> {
    _marker: PhantomData<fn(T)>,       // Contravariant over T
}

// ============================================================================
// UNSAFE TRAITS
// ============================================================================

// Send: Safe to transfer between threads
// Sync: Safe to share between threads (&T is Send)

// Implement when your type is thread-safe
// SAFETY: Document why it's safe!
unsafe impl<T: Send> Send for MyBox<T> {}
unsafe impl<T: Sync> Sync for MyBox<T> {}

// Common cases where manual impl needed:
// - Raw pointers (not Send/Sync by default)
// - Types with interior mutability via UnsafeCell
// - FFI types wrapping thread-safe C libraries

// ============================================================================
// INTERIOR MUTABILITY PRIMITIVES
// ============================================================================

use std::cell::UnsafeCell;

// UnsafeCell - the primitive for interior mutability
struct MyCell<T> {
    value: UnsafeCell<T>,
}

impl<T> MyCell<T> {
    fn new(value: T) -> Self {
        MyCell { value: UnsafeCell::new(value) }
    }

    fn get(&self) -> *mut T {
        self.value.get()  // Safe to get pointer
    }

    fn set(&self, value: T) {
        unsafe { *self.value.get() = value; }  // Must ensure no aliasing!
    }
}

// ============================================================================
// COMMON UNSAFE PATTERNS
// ============================================================================

// Pattern: Split slice into non-overlapping parts
fn split_at_mut<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    let len = slice.len();
    let ptr = slice.as_mut_ptr();
    assert!(mid <= len);
    unsafe {
        (
            std::slice::from_raw_parts_mut(ptr, mid),
            std::slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}

// Pattern: Reinterpret bytes as struct (careful with alignment!)
#[repr(C)]
struct Header { magic: u32, version: u16, flags: u16 }

fn parse_header(bytes: &[u8]) -> Option<&Header> {
    if bytes.len() < std::mem::size_of::<Header>() { return None; }
    let ptr = bytes.as_ptr();
    if ptr.align_offset(std::mem::align_of::<Header>()) != 0 { return None; }
    Some(unsafe { &*(ptr as *const Header) })
}

// ============================================================================
// DEBUGGING & TESTING UNSAFE CODE
// ============================================================================

// Run with Miri to detect UB
// cargo +nightly miri test

// Enable debug assertions in unsafe code
unsafe fn risky_operation(ptr: *const i32) -> i32 {
    debug_assert!(!ptr.is_null(), "null pointer passed");
    debug_assert!(ptr.align_offset(std::mem::align_of::<i32>()) == 0, "misaligned");
    *ptr
}

// Clippy lint for undocumented unsafe
// #![warn(clippy::undocumented_unsafe_blocks)]

// Address sanitizer (detects memory errors)
// RUSTFLAGS="-Z sanitizer=address" cargo +nightly test

// Thread sanitizer (detects data races)
// RUSTFLAGS="-Z sanitizer=thread" cargo +nightly test

// ============================================================================
// SAFETY DOCUMENTATION TEMPLATE
// ============================================================================

/// Brief description of the function.
///
/// # Safety
///
/// The caller must ensure that:
/// - `ptr` is valid for reads of `len * size_of::<T>()` bytes
/// - `ptr` is properly aligned for type `T`
/// - The memory is initialized with valid `T` values
/// - No other references to the same memory exist
/// - [Any other preconditions]
pub unsafe fn example_unsafe_fn<T>(ptr: *const T, len: usize) -> &'static [T] {
    // SAFETY: Caller guarantees ptr validity and initialization
    std::slice::from_raw_parts(ptr, len)
}

// ============================================================================
// QUICK REFERENCE: WHEN TO USE UNSAFE
// ============================================================================

// ✅ USE UNSAFE FOR:
// - Implementing fundamental data structures (Vec, HashMap, etc.)
// - FFI to call C libraries
// - Performance-critical code (after profiling proves necessity)
// - Hardware-level programming (embedded, drivers)
// - Building safe abstractions over inherently unsafe operations

// ❌ AVOID UNSAFE FOR:
// - Bypassing borrow checker in application code (redesign instead)
// - Micro-optimizations without benchmarks
// - When safe abstractions exist (use crates like bytemuck, zerocopy)
// - If you can't explain WHY it's safe
```
