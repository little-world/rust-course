# 15. Unsafe Rust Patterns

## Raw Pointer Manipulation

### Basic Raw Pointer Usage

```rust
fn raw_pointer_basics() {
    let mut num = 42;

    // Creating raw pointers from references
    let r1: *const i32 = &num;
    let r2: *mut i32 = &mut num;

    // Creating raw pointers from arbitrary addresses (dangerous!)
    let address = 0x012345usize;
    let r3 = address as *const i32;

    unsafe {
        // Dereferencing raw pointers requires unsafe
        println!("r1 points to: {}", *r1);

        // Mutating through raw pointer
        *r2 = 100;
        println!("num is now: {}", num);
    }
}
```

### Pointer Arithmetic

```rust
fn pointer_arithmetic() {
    let arr = [1, 2, 3, 4, 5];
    let ptr: *const i32 = arr.as_ptr();

    unsafe {
        // Manual iteration using pointer arithmetic
        for i in 0..arr.len() {
            let element_ptr = ptr.add(i);
            println!("Element {}: {}", i, *element_ptr);
        }

        // Pointer offset
        let third = ptr.offset(2);
        println!("Third element: {}", *third);
    }
}
```

### Building a Raw Vec-like Structure

```rust
use std::alloc::{alloc, dealloc, realloc, Layout};
use std::ptr;

pub struct RawVec<T> {
    ptr: *mut T,
    cap: usize,
}

impl<T> RawVec<T> {
    pub fn new() -> Self {
        assert!(std::mem::size_of::<T>() != 0, "Zero-sized types not supported");
        RawVec {
            ptr: std::ptr::null_mut(),
            cap: 0,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        let layout = Layout::array::<T>(cap).unwrap();
        let ptr = unsafe { alloc(layout) as *mut T };

        if ptr.is_null() {
            panic!("Allocation failed");
        }

        RawVec { ptr, cap }
    }

    pub fn grow(&mut self) {
        let new_cap = if self.cap == 0 { 1 } else { self.cap * 2 };

        let new_layout = Layout::array::<T>(new_cap).unwrap();

        let new_ptr = if self.cap == 0 {
            unsafe { alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            unsafe { realloc(self.ptr as *mut u8, old_layout, new_layout.size()) }
        };

        if new_ptr.is_null() {
            panic!("Allocation failed");
        }

        self.ptr = new_ptr as *mut T;
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

### Null Pointer Optimization

```rust
use std::ptr::NonNull;

// NonNull is a wrapper that guarantees the pointer is never null
struct Node<T> {
    value: T,
    next: Option<NonNull<Node<T>>>,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Node { value, next: None }
    }

    fn set_next(&mut self, next: NonNull<Node<T>>) {
        self.next = Some(next);
    }
}

// Example of building a linked list with NonNull
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
        let mut node = Box::new(Node::new(value));
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

## FFI Patterns and C Interop

### Basic C Function Binding

```rust
// Declaring external C functions
extern "C" {
    fn abs(input: i32) -> i32;
    fn strlen(s: *const std::os::raw::c_char) -> usize;
    fn malloc(size: usize) -> *mut std::os::raw::c_void;
    fn free(ptr: *mut std::os::raw::c_void);
}

fn use_c_functions() {
    unsafe {
        println!("abs(-42) = {}", abs(-42));

        let c_str = b"Hello, C!\0";
        let len = strlen(c_str.as_ptr() as *const std::os::raw::c_char);
        println!("String length: {}", len);
    }
}
```

### Working with C Strings

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Rust to C string
fn rust_to_c_string() -> *mut c_char {
    let rust_str = "Hello from Rust";
    let c_string = CString::new(rust_str).expect("CString creation failed");
    c_string.into_raw() // Transfer ownership to C
}

// C to Rust string
unsafe fn c_to_rust_string(c_str: *const c_char) -> String {
    let c_str = CStr::from_ptr(c_str);
    c_str.to_string_lossy().into_owned()
}

// Free a C string created by Rust
unsafe fn free_rust_c_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
        // CString is dropped here, freeing the memory
    }
}

// Example usage
fn c_string_example() {
    let c_str = rust_to_c_string();

    unsafe {
        println!("C string: {:?}", CStr::from_ptr(c_str));
        free_rust_c_string(c_str);
    }
}
```

### C Struct Interop

```rust
use std::os::raw::{c_int, c_char};

// Define a C-compatible struct
#[repr(C)]
struct Point {
    x: c_int,
    y: c_int,
}

#[repr(C)]
struct Person {
    name: *const c_char,
    age: c_int,
    height: f64,
}

// Enum with C-compatible representation
#[repr(C)]
enum Status {
    Success = 0,
    Error = 1,
    Pending = 2,
}

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

### Creating a Safe Wrapper for C Library

```rust
use std::ffi::CString;
use std::os::raw::c_char;

// Unsafe C API (simulated)
extern "C" {
    fn create_context() -> *mut std::os::raw::c_void;
    fn destroy_context(ctx: *mut std::os::raw::c_void);
    fn context_do_work(ctx: *mut std::os::raw::c_void, data: *const c_char) -> i32;
}

// Safe Rust wrapper
pub struct Context {
    inner: *mut std::os::raw::c_void,
}

impl Context {
    pub fn new() -> Option<Self> {
        let ptr = unsafe { create_context() };
        if ptr.is_null() {
            None
        } else {
            Some(Context { inner: ptr })
        }
    }

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

// Safe to send between threads if the C library is thread-safe
unsafe impl Send for Context {}
```

### Callback Functions (C to Rust)

```rust
use std::os::raw::c_int;

// Type alias for C callback
type Callback = extern "C" fn(c_int) -> c_int;

extern "C" {
    fn register_callback(cb: Callback);
    fn trigger_callback(value: c_int);
}

// Rust function with C calling convention
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

// Advanced: Callback with user data
type CallbackWithData = extern "C" fn(*mut std::os::raw::c_void, c_int) -> c_int;

extern "C" fn callback_with_context(user_data: *mut std::os::raw::c_void, value: c_int) -> c_int {
    unsafe {
        let data = &mut *(user_data as *mut i32);
        *data += value;
        *data
    }
}
```

## Uninitialized Memory Handling

### Using MaybeUninit

```rust
use std::mem::MaybeUninit;

fn create_array_uninit() -> [i32; 1000] {
    // Efficient initialization of large arrays
    let mut arr: [MaybeUninit<i32>; 1000] = unsafe {
        MaybeUninit::uninit().assume_init()
    };

    for (i, elem) in arr.iter_mut().enumerate() {
        *elem = MaybeUninit::new(i as i32);
    }

    unsafe {
        // Transmute to initialized array
        std::mem::transmute(arr)
    }
}

// Better pattern with newer Rust
fn create_array_uninit_safe() -> [i32; 1000] {
    let mut arr: [MaybeUninit<i32>; 1000] = MaybeUninit::uninit_array();

    for (i, elem) in arr.iter_mut().enumerate() {
        elem.write(i as i32);
    }

    unsafe { MaybeUninit::array_assume_init(arr) }
}
```

### Partial Initialization

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
        // Initialize fields one by one
        std::ptr::addr_of_mut!((*ptr).field1).write(String::from("hello"));
        std::ptr::addr_of_mut!((*ptr).field2).write(vec![1, 2, 3]);
        std::ptr::addr_of_mut!((*ptr).field3).write(Box::new(42));

        // All fields initialized, safe to assume_init
        uninit.assume_init()
    }
}
```

### Reading Uninitialized Memory (What NOT to Do)

```rust
use std::mem::MaybeUninit;

fn undefined_behavior_example() {
    let uninit: MaybeUninit<i32> = MaybeUninit::uninit();

    // DON'T DO THIS - Undefined behavior!
    // let value = unsafe { uninit.assume_init() };

    // CORRECT: Initialize first
    let mut uninit = MaybeUninit::uninit();
    uninit.write(42);
    let value = unsafe { uninit.assume_init() };
    println!("Value: {}", value);
}
```

### Out-Parameter Pattern for C FFI

```rust
use std::mem::MaybeUninit;
use std::os::raw::c_int;

extern "C" {
    // C function that writes to an out parameter
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

### Initializing Arrays from External Functions

```rust
use std::mem::MaybeUninit;

extern "C" {
    fn fill_buffer(buffer: *mut u8, size: usize) -> i32;
}

fn read_into_buffer(size: usize) -> Option<Vec<u8>> {
    let mut buffer: Vec<MaybeUninit<u8>> = Vec::with_capacity(size);
    unsafe {
        buffer.set_len(size);
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

## Transmute and Type Punning

### Basic Transmute

```rust
use std::mem;

fn transmute_basics() {
    // Convert between types of the same size
    let a: u32 = 0x12345678;
    let b: [u8; 4] = unsafe { mem::transmute(a) };
    println!("Bytes: {:?}", b);

    // Float to bits
    let f: f32 = 1.0;
    let bits: u32 = unsafe { mem::transmute(f) };
    println!("Float bits: 0x{:08x}", bits);

    // Use to_bits() instead!
    let bits_safe = f.to_bits();
    assert_eq!(bits, bits_safe);
}
```

### Transmuting References

```rust
use std::mem;

// DANGEROUS: Transmuting references
fn transmute_reference_unsafe() {
    let x: &i32 = &42;

    // DON'T DO THIS without extreme care
    let y: &u32 = unsafe { mem::transmute(x) };
    println!("Transmuted: {}", y);
}

// BETTER: Using as_ptr and casting
fn transmute_reference_safer() {
    let x: i32 = 42;
    let ptr = &x as *const i32 as *const u32;
    let y = unsafe { &*ptr };
    println!("Casted: {}", y);
}
```

### Converting Between Slice Types

```rust
use std::slice;

fn slice_transmute() {
    let data: Vec<u32> = vec![0x12345678, 0x9abcdef0];

    // Convert &[u32] to &[u8]
    let bytes: &[u8] = unsafe {
        slice::from_raw_parts(
            data.as_ptr() as *const u8,
            data.len() * std::mem::size_of::<u32>(),
        )
    };

    println!("Bytes: {:?}", bytes);

    // SAFER alternative using bytemuck or zerocopy crates
}
```

### Enum Discrimination

```rust
use std::mem;

#[repr(u8)]
enum MyEnum {
    A = 0,
    B = 1,
    C = 2,
}

fn get_discriminant(e: &MyEnum) -> u8 {
    // UNSAFE: Reading discriminant directly
    unsafe { *(e as *const MyEnum as *const u8) }
}

fn enum_discriminant_safe(e: &MyEnum) -> u8 {
    // SAFE: Using mem::discriminant
    match e {
        MyEnum::A => 0,
        MyEnum::B => 1,
        MyEnum::C => 2,
    }
}
```

### Type Punning for Optimized Code

```rust
union FloatUnion {
    f: f32,
    u: u32,
}

fn fast_float_bits(f: f32) -> u32 {
    // Union-based type punning
    let union = FloatUnion { f };
    unsafe { union.u }
}

fn fast_floor(f: f32) -> i32 {
    // Fast float to int conversion
    let union = FloatUnion { f };
    let bits = unsafe { union.u };

    // Extract exponent and handle edge cases
    // (simplified example - real implementation is more complex)
    f as i32
}
```

### Transmuting Closures (Advanced)

```rust
use std::mem;

// Transmuting function pointers
fn transmute_fn_pointer() {
    fn example(x: i32) -> i32 { x * 2 }

    let fn_ptr: fn(i32) -> i32 = example;
    let raw_ptr: *const () = unsafe { mem::transmute(fn_ptr) };
    let fn_ptr_back: fn(i32) -> i32 = unsafe { mem::transmute(raw_ptr) };

    println!("Result: {}", fn_ptr_back(21));
}
```

### When NOT to Use Transmute

```rust
// BAD: Extending lifetimes
fn extend_lifetime_bad<'a>(x: &'a str) -> &'static str {
    // NEVER DO THIS - Undefined behavior!
    // unsafe { std::mem::transmute(x) }
    x // Just return the original
}

// BAD: Converting between different sized types
fn different_sizes_bad() {
    let x: u32 = 42;
    // COMPILE ERROR: different sizes
    // let y: u64 = unsafe { std::mem::transmute(x) };

    // CORRECT:
    let y: u64 = x as u64;
}
```

## Writing Safe Abstractions Over Unsafe

### Building a Safe Vec

```rust
use std::ptr;
use std::mem;

pub struct MyVec<T> {
    ptr: *mut T,
    len: usize,
    cap: usize,
}

impl<T> MyVec<T> {
    pub fn new() -> Self {
        MyVec {
            ptr: ptr::null_mut(),
            len: 0,
            cap: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            ptr::write(self.ptr.add(self.len), value);
        }

        self.len += 1;
    }

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

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe {
                Some(&*self.ptr.add(index))
            }
        } else {
            None
        }
    }

    fn grow(&mut self) {
        use std::alloc::{alloc, realloc, Layout};

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
        use std::alloc::{dealloc, Layout};

        // Drop all elements
        while let Some(_) = self.pop() {}

        // Deallocate memory
        if self.cap != 0 {
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

// Safety: Send if T is Send
unsafe impl<T: Send> Send for MyVec<T> {}
unsafe impl<T: Sync> Sync for MyVec<T> {}
```

### Invariants and Documentation

```rust
/// A slice type that is guaranteed to be non-empty.
///
/// # Safety Invariants
/// - The inner slice must always have at least one element
/// - The pointer must be valid and properly aligned
/// - The data must be valid for 'a
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
        // SAFETY: Our invariant guarantees at least one element
        unsafe { self.slice.get_unchecked(0) }
    }

    /// Returns the last element (always exists).
    pub fn last(&self) -> &T {
        // SAFETY: Our invariant guarantees at least one element
        unsafe { self.slice.get_unchecked(self.slice.len() - 1) }
    }

    pub fn as_slice(&self) -> &[T] {
        self.slice
    }
}
```

### PhantomData for Type Safety

```rust
use std::marker::PhantomData;
use std::ptr::NonNull;

/// A raw pointer wrapper that tracks ownership and variance.
pub struct RawPtr<T> {
    ptr: NonNull<T>,
    // PhantomData to:
    // 1. Mark that we "own" a T (affects drop check)
    // 2. Make the type covariant over T
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
            let _ = Box::from_raw(self.ptr.as_ptr());
        }
    }
}

// Safe because we own the T
unsafe impl<T: Send> Send for RawPtr<T> {}
unsafe impl<T: Sync> Sync for RawPtr<T> {}
```

### Building Safe APIs with Unsafe Internals

```rust
use std::cell::UnsafeCell;

/// A simple spinlock implementation.
pub struct SpinLock<T> {
    locked: std::sync::atomic::AtomicBool,
    data: UnsafeCell<T>,
}

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> SpinLock<T> {
    pub fn new(data: T) -> Self {
        SpinLock {
            locked: std::sync::atomic::AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SpinLockGuard<T> {
        use std::sync::atomic::Ordering;

        // Spin until we acquire the lock
        while self.locked.swap(true, Ordering::Acquire) {
            std::hint::spin_loop();
        }

        SpinLockGuard { lock: self }
    }
}

impl<'a, T> std::ops::Deref for SpinLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: We hold the lock
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> std::ops::DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: We hold the lock and have mutable access to guard
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) {
        use std::sync::atomic::Ordering;
        self.lock.locked.store(false, Ordering::Release);
    }
}

// SAFETY: SpinLock properly synchronizes access to T
unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}
```

### Compile-Time Type-State Programming

```rust
use std::marker::PhantomData;

// Type states
struct Locked;
struct Unlocked;

/// A lock that uses type-state programming to ensure correct usage.
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

    /// Lock the data, transitioning to Locked state.
    pub fn lock(self) -> TypeStateLock<T, Locked> {
        TypeStateLock {
            data: self.data,
            _state: PhantomData,
        }
    }
}

impl<T> TypeStateLock<T, Locked> {
    /// Unlock the data, transitioning back to Unlocked state.
    pub fn unlock(self) -> TypeStateLock<T, Unlocked> {
        TypeStateLock {
            data: self.data,
            _state: PhantomData,
        }
    }

    /// Access the data (only available when locked).
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

    // Can't access data when unlocked
    // lock.access(); // Compile error!

    let mut locked = lock.lock();
    locked.access().push(4); // OK

    let unlocked = locked.unlock();
    // unlocked.access(); // Compile error again!
}
```

### Testing Unsafe Code

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
        }

        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 3);
    }

    // Run with Miri for undefined behavior detection
    // cargo +nightly miri test
}
```

## Best Practices for Unsafe Code

### 1. Minimize Unsafe Boundaries

```rust
// BAD: Unsafe spreads throughout the code
pub fn bad_api(data: *mut u8, len: usize) {
    // Users must handle raw pointers
}

// GOOD: Unsafe is contained
pub fn good_api(data: &mut [u8]) {
    // Internally might use unsafe, but API is safe
}
```

### 2. Document Safety Requirements

```rust
/// # Safety
///
/// The caller must ensure that:
/// - `ptr` is valid for reads of `len` bytes
/// - `ptr` is properly aligned for type `T`
/// - The memory referenced by `ptr` is not accessed concurrently
/// - `len` is exactly the number of elements in the allocation
pub unsafe fn from_raw_parts<T>(ptr: *const T, len: usize) -> &'static [T] {
    std::slice::from_raw_parts(ptr, len)
}
```

### 3. Use Helper Functions

```rust
// Extract unsafe operations into well-tested helpers
mod unsafe_helpers {
    pub(crate) unsafe fn write_unchecked<T>(ptr: *mut T, value: T) {
        debug_assert!(!ptr.is_null());
        std::ptr::write(ptr, value);
    }

    pub(crate) unsafe fn read_unchecked<T>(ptr: *const T) -> T {
        debug_assert!(!ptr.is_null());
        std::ptr::read(ptr)
    }
}
```

### 4. Use Clippy and Miri

```bash
# Check for unsafe code issues
cargo clippy -- -W clippy::undocumented_unsafe_blocks

# Detect undefined behavior
cargo +nightly miri test

# Run with address sanitizer
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test
```

### 5. Consider Alternatives

```rust
// Instead of raw pointers, consider:
// - std::pin::Pin for self-referential structs
// - std::cell::UnsafeCell for interior mutability
// - std::sync::atomic for lock-free operations
// - ouroboros crate for self-referential structs
// - bytemuck crate for safe transmutes
```

This comprehensive guide covers the essential patterns for working with unsafe Rust while maintaining safety guarantees at API boundaries.
