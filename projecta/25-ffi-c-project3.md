# Chapter 25: FFI & C Interop — Programming Projects

## Project 3: Zero-Copy Byte Buffers Across C/Rust (Ownership, Allocators, and Safety)

### Problem Statement

Design and implement a C-compatible byte buffer ABI that allows Rust and C to exchange data without copies, while preventing double-frees and allocator mismatches. Provide functions to create, clone (shallow), slice, and destroy buffers; support both “borrowed” (non-owning) and “owned” buffers; and make it easy for C callers to return ownership to Rust safely.

### Why It Matters

- Cross-language data movement is commonly dominated by copying and memory bugs. A sound buffer ABI reduces copies and clarifies ownership.
- Avoids undefined behavior from freeing memory with the wrong allocator (Rust’s allocator vs. `free`).
- Sets a foundation for safe, high-performance interop in networking, compression, and multimedia pipelines.

### Use Cases

- Passing network frames from a C stack to Rust without copying.
- Returning compressed bytes from Rust to C with a clear destroy function owned by Rust.
- Sharing large read-only blobs (e.g., mmap’d files) between C and Rust code safely.

---

## Solution Outline (Didactic, not full implementation)

1) Define an FFI-safe `#[repr(C)]` buffer header with pointer, length, capacity, and ownership tag.
2) Implement constructors for owned buffers (Rust allocates) and borrowed views (Rust does not own).
3) Provide `destroy` that only frees owned buffers using Rust’s allocator; borrowed buffers are ignored.
4) Add `slice` and `clone_shallow` to derive subviews without copying; check bounds.
5) Add `from_vec` and `into_vec` to move memory across the boundary without reallocation.
6) Optimization & parallelism: pool allocations, align to cache lines, and discuss thread safety for shared immutable buffers.

---

## Milestone 1: ABI Definition and Ownership Model

### Introduction
Design the C-visible struct and ownership enum. Decide how C tells Rust who owns the memory and who must free it.

Why previous step is not enough: Without a precise ABI and ownership contract, any subsequent code is unsafe and ambiguous.

### Architecture

- Structs/Traits:
  - `#[repr(C)] pub struct FfiBuf { data: *mut u8, len: usize, cap: usize, tag: BufTag }`
  - `#[repr(C)] pub enum BufTag { Owned = 0, Borrowed = 1 }`
- Functions:
  - None implemented yet; this step finalizes the ABI layout and semantics.
- Roles: Provide stable memory layout and explicit ownership semantics.

### Checkpoint Tests

```rust
assert_eq!(core::mem::size_of::<usize>() * 3 + core::mem::size_of::<BufTag>(), core::mem::size_of::<FfiBuf>());
```

### Starter Code

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BufTag { Owned = 0, Borrowed = 1 }

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiBuf {
    pub data: *mut u8,
    pub len: usize,
    pub cap: usize,
    pub tag: BufTag,
}
```

---

## Milestone 2: Constructors for Owned and Borrowed Buffers

### Introduction
Implement `buf_new_owned(len)`, `buf_from_slice_borrowed(ptr,len)`, and a helper to check null/overflow.

Why previous step is not enough: We need concrete ways to create buffers under clear ownership rules.

### Architecture

- Functions:
  - `buf_new_owned(len: usize) -> FfiBuf` — allocates via `Vec<u8>` and leaks to C.
  - `buf_from_slice_borrowed(data: *mut u8, len: usize) -> FfiBuf` — non-owning view; `cap = len`.
- Roles: Enable creation paths for both ownership modes.

### Checkpoint Tests

```text
let b = unsafe { buf_new_owned(128) };
assert!(!b.data.is_null());
assert_eq!(b.len, 128);
assert_eq!(b.tag as i32, BufTag::Owned as i32);
```

### Starter Code

```rust
#[no_mangle]
pub extern "C" fn buf_new_owned(len: usize) -> FfiBuf {
    let mut v = Vec::<u8>::with_capacity(len);
    unsafe { v.set_len(len); }
    let b = FfiBuf { data: v.as_mut_ptr(), len, cap: v.capacity(), tag: BufTag::Owned };
    core::mem::forget(v);
    b
}

#[no_mangle]
pub extern "C" fn buf_from_slice_borrowed(data: *mut u8, len: usize) -> FfiBuf {
    FfiBuf { data, len, cap: len, tag: BufTag::Borrowed }
}
```

---

## Milestone 3: Destroy Function (Free Only If Owned)

### Introduction
Provide `buf_destroy` that frees memory only when the tag is `Owned`. Avoid double-free and allocator mismatch.

Why previous step is not enough: Buffers allocated by Rust must be freed by Rust; borrowed buffers must not be freed.

### Architecture

- Functions:
  - `buf_destroy(buf: *mut FfiBuf)` — if `Owned`, reconstruct `Vec` and drop; then null out fields.
- Role: Enforce ownership and prevent memory errors.

### Checkpoint Tests

```text
let mut b = unsafe { buf_new_owned(64) };
unsafe { buf_destroy(&mut b as *mut _) };
assert!(b.data.is_null());
assert_eq!(b.len, 0);
```

### Starter Code

```rust
#[no_mangle]
pub extern "C" fn buf_destroy(buf: *mut FfiBuf) {
    if buf.is_null() { return; }
    let b = unsafe { &mut *buf };
    if b.tag as i32 == BufTag::Owned as i32 && !b.data.is_null() {
        unsafe { drop(Vec::from_raw_parts(b.data, b.len, b.cap)); }
    }
    b.data = core::ptr::null_mut();
    b.len = 0; b.cap = 0; b.tag = BufTag::Borrowed;
}
```

---

## Milestone 4: Slicing and Shallow Cloning Without Copies

### Introduction
Enable creating subviews and clones that share the underlying memory; ensure bounds checks and preserve ownership tag.

Why previous step is not enough: Callers need ergonomics to work with subranges without copying.

### Architecture

- Functions:
  - `buf_slice(buf: FfiBuf, offset: usize, len: usize) -> FfiBuf` — returns borrowed view.
  - `buf_clone_shallow(buf: FfiBuf) -> FfiBuf` — returns another view with same tag.
- Role: Zero-copy composition.

### Checkpoint Tests

```text
let b = unsafe { buf_new_owned(10) };
let s = unsafe { buf_slice(b, 2, 4) };
assert_eq!(s.len, 4);
```

### Starter Code

```rust
#[no_mangle]
pub extern "C" fn buf_slice(buf: FfiBuf, offset: usize, len: usize) -> FfiBuf {
    if offset.checked_add(len).map(|e| e <= buf.len).unwrap_or(false) {
        FfiBuf { data: unsafe { buf.data.add(offset) }, len, cap: len, tag: BufTag::Borrowed }
    } else { FfiBuf { data: core::ptr::null_mut(), len: 0, cap: 0, tag: BufTag::Borrowed } }
}

#[no_mangle]
pub extern "C" fn buf_clone_shallow(buf: FfiBuf) -> FfiBuf { buf }
```

---

## Milestone 5: Zero-Copy Moves: Vec <-> FfiBuf

### Introduction
Allow moving a `Vec<u8>` into an `FfiBuf` (without copy) and reconstituting it back into a `Vec<u8>` if still owned by Rust.

Why previous step is not enough: Many Rust APIs produce/consume `Vec<u8>`; zero-copy conversion avoids costly memmoves.

### Architecture

- Functions:
  - `ffi_buf_from_vec(v: Vec<u8>) -> FfiBuf`
  - `ffi_buf_into_vec(buf: *mut FfiBuf) -> *mut Vec<u8>` — only if `Owned`.
- Role: Efficient interop on Rust side while exposing a stable C ABI.

### Checkpoint Tests

```text
let v = vec![1,2,3];
let mut b = unsafe { ffi_buf_from_vec(v) };
let p = unsafe { ffi_buf_into_vec(&mut b) };
assert!(!p.is_null());
unsafe { drop(Box::from_raw(p)); }
```

### Starter Code

```rust
#[no_mangle]
pub extern "C" fn ffi_buf_from_vec(mut v: Vec<u8>) -> FfiBuf {
    let b = FfiBuf { data: v.as_mut_ptr(), len: v.len(), cap: v.capacity(), tag: BufTag::Owned };
    core::mem::forget(v); b
}

#[no_mangle]
pub extern "C" fn ffi_buf_into_vec(buf: *mut FfiBuf) -> *mut Vec<u8> {
    if buf.is_null() { return core::ptr::null_mut(); }
    let b = unsafe { &mut *buf };
    if b.tag as i32 != BufTag::Owned as i32 || b.data.is_null() { return core::ptr::null_mut(); }
    let v = unsafe { Vec::from_raw_parts(b.data, b.len, b.cap) };
    b.data = core::ptr::null_mut(); b.len = 0; b.cap = 0; b.tag = BufTag::Borrowed;
    Box::into_raw(Box::new(v))
}
```

---

## Milestone 6: Optimization, Memory Pools, and Thread Safety

### Introduction
Discuss pooling, alignment, and sharing. For read-mostly workloads, share immutable borrowed buffers across threads; for owned buffers, consider a lock-free freelist.

Why previous step is not enough: We have correctness, but not optimal performance or concurrency semantics.

Improvement: Reduce allocations using pools, align large buffers to cache lines or page size, and document `Send`/`Sync` guarantees (e.g., `FfiBuf` is `Send + Sync` when treated as raw bytes).

### Testing Hints

- Benchmark copy vs. zero-copy for 1 KiB, 64 KiB, and 4 MiB payloads.
- Use threads to read borrowed buffers concurrently; use `Miri` to catch UB in pointer math.

---

## Complete Working Example

```rust
use core::ffi::c_void;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BufTag { Owned = 0, Borrowed = 1 }

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiBuf {
    pub data: *mut u8,
    pub len: usize,
    pub cap: usize,
    pub tag: BufTag,
}

#[no_mangle]
pub extern "C" fn buf_new_owned(len: usize) -> FfiBuf {
    let mut v = Vec::<u8>::with_capacity(len);
    unsafe { v.set_len(len); }
    let b = FfiBuf { data: v.as_mut_ptr(), len, cap: v.capacity(), tag: BufTag::Owned };
    core::mem::forget(v);
    b
}

#[no_mangle]
pub extern "C" fn buf_from_slice_borrowed(data: *mut u8, len: usize) -> FfiBuf {
    FfiBuf { data, len, cap: len, tag: BufTag::Borrowed }
}

#[no_mangle]
pub extern "C" fn buf_destroy(buf: *mut FfiBuf) {
    if buf.is_null() { return; }
    let b = unsafe { &mut *buf };
    if b.tag as i32 == BufTag::Owned as i32 && !b.data.is_null() {
        unsafe { drop(Vec::from_raw_parts(b.data, b.len, b.cap)); }
    }
    b.data = core::ptr::null_mut();
    b.len = 0; b.cap = 0; b.tag = BufTag::Borrowed;
}

#[no_mangle]
pub extern "C" fn buf_slice(buf: FfiBuf, offset: usize, len: usize) -> FfiBuf {
    if offset.checked_add(len).map(|e| e <= buf.len).unwrap_or(false) {
        FfiBuf { data: unsafe { buf.data.add(offset) }, len, cap: len, tag: BufTag::Borrowed }
    } else { FfiBuf { data: core::ptr::null_mut(), len: 0, cap: 0, tag: BufTag::Borrowed } }
}

#[no_mangle]
pub extern "C" fn buf_clone_shallow(buf: FfiBuf) -> FfiBuf { buf }

#[no_mangle]
pub extern "C" fn ffi_buf_from_vec(mut v: Vec<u8>) -> FfiBuf {
    let b = FfiBuf { data: v.as_mut_ptr(), len: v.len(), cap: v.capacity(), tag: BufTag::Owned };
    core::mem::forget(v); b
}

#[no_mangle]
pub extern "C" fn ffi_buf_into_vec(buf: *mut FfiBuf) -> *mut Vec<u8> {
    if buf.is_null() { return core::ptr::null_mut(); }
    let b = unsafe { &mut *buf };
    if b.tag as i32 != BufTag::Owned as i32 || b.data.is_null() { return core::ptr::null_mut(); }
    let v = unsafe { Vec::from_raw_parts(b.data, b.len, b.cap) };
    b.data = core::ptr::null_mut(); b.len = 0; b.cap = 0; b.tag = BufTag::Borrowed;
    Box::into_raw(Box::new(v))
}

fn main() {
    // Create an owned buffer and fill it
    let mut b = buf_new_owned(8);
    assert!(!b.data.is_null());
    unsafe { core::slice::from_raw_parts_mut(b.data, b.len).copy_from_slice(&[1,2,3,4,5,6,7,8]); }

    // Borrow a sub-slice without copying
    let s = buf_slice(b, 2, 4);
    let view = unsafe { core::slice::from_raw_parts(s.data, s.len) };
    assert_eq!(view, &[3,4,5,6]);

    // Convert to Vec and take back ownership on Rust side
    let p = ffi_buf_into_vec(&mut b);
    let v = unsafe { Box::from_raw(p) };
    assert_eq!(&*v, &[1,2,3,4,5,6,7,8]);
}
```

---

## Milestone 7: Python Bindings and Usage (cffi for structs + zero-copy)

### Introduction
Bind the buffer ABI to Python using `cffi`, which is convenient for declaring C `struct` layouts like `FfiBuf`. We will demonstrate creating an owned buffer in Rust, exposing it to Python as a `memoryview` without copies, slicing/cloning views, and returning ownership to Rust for destruction.

Why previous step is not enough: We’ve only shown Rust/C interop. Many data pipelines integrate Python. Providing Python bindings enables zero-copy flows between Rust and Python while maintaining correct ownership and lifetime.

### Architecture

- Python uses `cffi.FFI()` to declare:
  - `enum BufTag { Owned = 0, Borrowed = 1 };`
  - `typedef struct { uint8_t* data; size_t len; size_t cap; enum BufTag tag; } FfiBuf;`
  - Functions: `buf_new_owned`, `buf_from_slice_borrowed`, `buf_slice`, `buf_clone_shallow`, `buf_destroy`, `ffi_buf_into_vec` (optional for Rust-only path), `ffi_buf_from_vec` (optional).
- Strategy:
  - Create an owned buffer in Rust, fill it from Python via `memoryview` without copies.
  - Create slices in Rust using `buf_slice` and read from Python as `bytes`/`memoryview`.
  - Ensure `buf_destroy` is called when done to avoid leaks.

### Checkpoint Tests (Python)

```python
# test_ffibuf_py.py
import os
from cffi import FFI

ffi = FFI()
ffi.cdef(
    """
    typedef enum { Owned = 0, Borrowed = 1 } BufTag;
    typedef struct { unsigned char* data; size_t len; size_t cap; BufTag tag; } FfiBuf;

    FfiBuf buf_new_owned(size_t len);
    FfiBuf buf_from_slice_borrowed(unsigned char* data, size_t len);
    void   buf_destroy(FfiBuf* buf);
    FfiBuf buf_slice(FfiBuf buf, size_t offset, size_t len);
    FfiBuf buf_clone_shallow(FfiBuf buf);
    """
)

libname = os.environ.get("RUST_FFI_LIB", "./target/release/libffi_c_examples.dylib")
lib = ffi.dlopen(libname)

def test_zero_copy_roundtrip():
    b = lib.buf_new_owned(8)
    assert b.data != ffi.NULL and b.len == 8
    # Write into the Rust-owned buffer from Python without copying
    mv = ffi.buffer(b.data, b.len)
    mv[:] = bytes([1,2,3,4,5,6,7,8])

    # Slice in Rust, view from Python
    s = lib.buf_slice(b, 2, 4)
    assert s.len == 4
    sv = bytes(ffi.buffer(s.data, s.len))
    assert sv == b"\x03\x04\x05\x06"

    # Clone shallow and check reads
    c = lib.buf_clone_shallow(b)
    cv = bytes(ffi.buffer(c.data, c.len))
    assert cv == bytes([1,2,3,4,5,6,7,8])

    # Destroy original owned buffer (frees memory); cloned borrowed views must not be freed.
    lib.buf_destroy(ffi.addressof(b))
    assert b.data == ffi.NULL and b.len == 0
```

### Starter Code (Rust: already provided C ABI)

You already exposed the `FfiBuf` ABI and functions. Ensure the shared library is built as `cdylib`:

```toml
[lib]
name = "ffi_c_examples"
crate-type = ["cdylib"]
```

Build outputs:
- macOS: `target/release/libffi_c_examples.dylib`
- Linux: `target/release/libffi_c_examples.so`
- Windows: `target\\release\\ffi_c_examples.dll`

### Why this step improves things
- Demonstrates zero-copy data exchange between Rust and Python.
- Maintains correct ownership: Rust frees Rust-allocated buffers; Python only borrows.
- Enables Python-based tests and prototyping for systems that will run in Rust.

### Testing Hints
- Use `pytest` to run the Python test; verify contents after slicing/cloning.
- Benchmark copying `bytes(b.data[:b.len])` vs. direct `memoryview` access to see zero-copy benefits.
- Add negative test: attempt out-of-bounds slice and verify returned `FfiBuf` has `data == NULL` and `len == 0`.
