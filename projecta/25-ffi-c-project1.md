# Chapter 25: FFI & C Interop — Programming Projects

## Project 1: Safe Wrapper over C qsort/bsearch (Callbacks across the FFI boundary)

### Introduction to FFI Concepts

Foreign Function Interface (FFI) enables Rust code to call functions written in other languages (primarily C) and vice versa. This capability is crucial for integrating with existing C libraries, operating system APIs, and legacy codebases. Understanding FFI requires mastering several interconnected concepts:

#### 1. The `extern` Keyword and ABI Compatibility

The `extern "C"` syntax specifies that a function follows the C calling convention (Application Binary Interface). This ensures that Rust and C code agree on how arguments are passed, how the stack is managed, and how return values are handled. Without matching ABIs, function calls would corrupt memory and crash.

```rust
extern "C" {
    fn qsort(base: *mut c_void, nmemb: usize, size: usize,
             compar: extern "C" fn(*const c_void, *const c_void) -> i32);
}
```

The `extern "C"` block declares functions implemented elsewhere (in C libraries), while `extern "C" fn` defines Rust functions callable from C.

#### 2. Raw Pointers and Memory Safety

FFI requires raw pointers (`*const T` and `*mut T`) instead of Rust references because:
- C has no concept of Rust's borrowing rules or lifetimes
- Raw pointers can be null, which Rust references cannot
- Raw pointers don't enforce aliasing guarantees

Working with raw pointers is inherently `unsafe` because the compiler cannot verify:
- The pointer is valid and properly aligned
- The memory it points to is initialized
- No data races occur
- Lifetimes are respected

#### 3. Memory Layout and `#[repr(C)]`

Rust's default memory layout may differ from C's expectations. The `#[repr(C)]` attribute forces Rust to lay out a struct exactly as C would, ensuring field order and padding match C conventions. This is critical when:
- Passing structs across the FFI boundary
- Casting between pointer types
- Reading C-formatted binary data

```rust
#[repr(C)]
struct Point {
    x: f64,
    y: f64,
}
```

#### 4. Function Pointers and Callbacks

Function pointers allow passing executable code as data. In FFI contexts, callbacks enable C code to call back into Rust. The key challenges are:
- Type signatures must exactly match C expectations (return type, argument types, calling convention)
- Callbacks must be `extern "C" fn` types, not closures (closures capture environment and have incompatible layouts)
- Panicking in a callback crosses the FFI boundary and causes undefined behavior

#### 5. Name Mangling and `#[no_mangle]`

Rust "mangles" function names by encoding type information and module paths into the symbol name. This prevents accidental name collisions but makes functions invisible to C. The `#[no_mangle]` attribute preserves the original function name in the compiled binary:

```rust
#[no_mangle]
pub extern "C" fn process_data(ptr: *mut u8, len: usize) -> i32 {
    // C can call this as "process_data"
}
```

#### 6. Building C-Compatible Libraries with `cdylib`

The `cdylib` crate type produces a C-compatible dynamic library (`.so` on Linux, `.dylib` on macOS, `.dll` on Windows). Unlike `rlib` (Rust library) or `dylib` (Rust dynamic library), `cdylib` exports functions with C ABI and can be loaded by any language with C FFI support.

#### 7. Void Pointers and Type Erasure

C uses `void*` for generic pointers that can point to any type. Rust's equivalent is `*mut c_void` or `*const c_void` from `core::ffi`. Converting between typed pointers and void pointers requires casting:

```rust
let typed: *mut i32 = data.as_mut_ptr();
let erased: *mut c_void = typed as *mut c_void;
let restored: *mut i32 = erased as *mut i32;
```

This pattern is common in C APIs that work with generic data (like `qsort`).

#### 8. Panic Safety Across FFI

Rust panics unwind the stack by default, which is incompatible with C's error handling model. Unwinding into C code causes undefined behavior. Safe FFI code must either:
- Use `catch_unwind` to prevent panics from crossing the boundary
- Ensure callbacks cannot panic (using `extern "C" fn` instead of closures helps)
- Document panic behavior and mark functions as `unsafe` if they can panic

#### 9. Lifetimes at the FFI Boundary

Raw pointers have no lifetime tracking, so it's your responsibility to ensure:
- Data outlives all pointers to it
- No use-after-free occurs
- Mutable pointers don't alias with other pointers to the same data

Common patterns include:
- Converting `&[T]` to `*const T` for the duration of a C function call
- Ensuring Rust owns data until C is done with it
- Using pinning for data that must not move in memory

#### 10. Size and Alignment Guarantees

When interfacing with C, verify that:
- `size_of::<T>()` matches C's `sizeof(T)`
- `align_of::<T>()` meets C's alignment requirements
- Zero-sized types (ZSTs) are handled correctly (C has no equivalent)

### Connection to This Project

This project exercises FFI fundamentals by wrapping C standard library functions `qsort` and `bsearch`. Here's how each concept applies:

**ABI Compatibility**: You'll declare `extern "C"` function signatures matching the C standard library and define `extern "C" fn` comparators that C's `qsort` can call back into.

**Raw Pointers**: The wrapper converts Rust slices (`&mut [T]`) to raw pointers (`*mut c_void`) for C consumption, then safely reconstructs them after C operations complete.

**Memory Layout**: The `#[repr(C)]` attribute ensures custom structs (like `Pair`) have C-compatible layout when passed to `qsort`.

**Callbacks**: Writing comparator functions as `extern "C" fn` types demonstrates the constraints of FFI callbacks—no closures, no panic unwinding, exact type signatures.

**Void Pointers**: You'll implement the double-cast pattern: typed Rust pointer → void pointer for C → typed pointer in the callback, mirroring how C generic algorithms work.

**Panic Safety**: The project highlights why `extern "C" fn` comparators are safer than closures—they cannot capture environments or accidentally panic across the FFI boundary.

**Python Bindings**: The final milestone uses `#[no_mangle]` and `cdylib` to expose functions to Python via `ctypes`, demonstrating how one Rust library can serve multiple language ecosystems.

By the end of this project, you'll have created a **safe abstraction** over unsafe FFI, understanding both the low-level mechanics and the design principles for sound wrapper APIs.

---

### Problem Statement

Implement a safe, idiomatic Rust wrapper around the C standard library’s `qsort` and `bsearch` that can sort and search primitive types and C-compatible structs via user-supplied comparison functions. Provide zero-copy interop from Rust slices to C pointers, preserve Rust’s aliasing guarantees, and avoid UB around lifetimes, alignment, and callback trampolines.

### Why It Matters

- Real projects often need to integrate with mature C algorithms or APIs. A safe wrapper lets a Rust codebase benefit from C functionality without sacrificing Rust’s safety.
- You’ll learn how to design safe abstractions on top of `unsafe` blocks, how to cross the ABI boundary with function pointers, and how to manage lifetimes and ownership at the boundary.

### Use Cases

- Sorting C-ABI data blocks received from a C library while staying in Rust.
- Providing a Rust-friendly API for legacy C routines in an existing codebase.
- Building a platform interop layer where some operations are offloaded to the system C library.

---

## Solution Outline (Didactic, not full implementation)

We’ll start from a purely safe Rust baseline, then progressively introduce FFI and safety wrappers:

1) Baseline in pure Rust using `slice.sort_unstable_by(...)` to define the target behavior and tests.
2) Minimal FFI call to `libc::qsort` on a `&mut [T]` of C-compatible items using a raw pointer comparator.
3) Safe wrapper `qsort_slice` ensuring type/layout invariants (`#[repr(C)]` when needed) and preventing panics across FFI.
4) Add `bsearch` wrapper returning an index in the slice (or `None`).
5) Optimize: remove per-call closures, use `extern "C" fn` trampolines, precompute element size, and eliminate branches in the comparator when possible.
6) Parallelism discussion: compare with Rust’s native `sort_unstable` and when to choose one vs. the other; how to test performance soundly.

---

## Milestone 1: Baseline Sorting in Pure Rust

### Introduction
Before touching FFI, define the behavior and tests using idiomatic Rust. This gives you a correctness oracle.

Why previous step is not enough: We haven’t used C at all yet. We need the baseline to ensure our FFI path matches Rust’s behavior.

### Architecture

- Structs/Traits: none yet.
- Functions:
  - `fn baseline_sort<T: Ord>(v: &mut [T])` — Sorts in place using Rust.
  - `fn baseline_sort_by<T, F: FnMut(&T, &T) -> core::cmp::Ordering>(v: &mut [T], f: F)` — Custom comparator.
  - Role: Define behavior, produce expected outputs for testing.

### Checkpoint Tests

```rust
#[test]
fn baseline_sorts_numbers() {
    let mut v = vec![5, 1, 4, 2, 3];
    baseline_sort(&mut v);
    assert_eq!(v, [1,2,3,4,5]);
}
```

### Starter Code

```rust
pub fn baseline_sort<T: Ord>(v: &mut [T]) {
    v.sort_unstable();
}

pub fn baseline_sort_by<T, F: FnMut(&T, &T) -> core::cmp::Ordering>(v: &mut [T], cmp: F) {
    v.sort_unstable_by(cmp);
}
```

---

## Milestone 2: Minimal qsort via unsafe FFI

### Introduction
Call `libc::qsort` directly. You’ll convert a Rust slice into a raw pointer, pass a comparator, and ensure memory layout compatibility.

Why previous step is not enough: Rust-only solution doesn’t teach FFI. We must cross the ABI boundary and handle raw pointers and callbacks correctly.

### Architecture

- Structs/Traits: none required.
- Functions:
  - `unsafe fn qsort_raw<T>(data: *mut core::ffi::c_void, len: usize, elem_size: usize, cmp: extern "C" fn(*const core::ffi::c_void, *const core::ffi::c_void) -> i32)` — Thin wrapper around `libc::qsort`.
  - Role: Show the bare-minimum C call shape.

### Checkpoint Tests (conceptual)

```rust
// This test will only check that qsort can be invoked without crashing on basic inputs.
// Compare results with baseline afterwards.
```

### Starter Code

```rust
extern "C" {
    fn qsort(
        base: *mut core::ffi::c_void,
        nmemb: usize,
        size: usize,
        compar: extern "C" fn(*const core::ffi::c_void, *const core::ffi::c_void) -> i32,
    );
}

pub unsafe fn qsort_raw(
    data: *mut core::ffi::c_void,
    len: usize,
    elem_size: usize,
    cmp: extern "C" fn(*const core::ffi::c_void, *const core::ffi::c_void) -> i32,
) {
    qsort(data, len, elem_size, cmp);
}
```

---

## Milestone 3: Safe qsort wrapper for slices

### Introduction
Expose `fn qsort_slice<T>(v: &mut [T], cmp: extern "C" fn(*const T, *const T) -> core::cmp::Ordering)` that guarantees soundness by construction.

Why previous step is not enough: Raw `void*` and `i32` comparators are error-prone. We need a typed, sound API that ordinary Rust callers can’t misuse.

### Architecture

- Structs/Traits:
  - `trait CComparable<T>: Sized { fn as_bytes(&self) -> &[u8]; }` (optional if you want to enforce repr/size).
- Functions:
  - `fn qsort_slice<T>(v: &mut [T], cmp: extern "C" fn(*const T, *const T) -> i32)` — Internally casts the typed comparator to the `void*` flavor, converts `Ordering` to `i32` (-1/0/1), asserts `T: Copy` or `#[repr(C)]`.
  - Role: Provide safety and better ergonomics.

### Checkpoint Tests

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
struct Pair { a: i32, b: i32 }

extern "C" fn cmp_pair(a: *const Pair, b: *const Pair) -> i32 {
    // Safety: qsort only calls comparator with valid pointers to elements
    let (a, b) = unsafe { (&*a, &*b) };
    a.a.cmp(&b.a) as i32
}

#[test]
fn sorts_structs_by_field() {
    let mut v = vec![Pair{a:2,b:9}, Pair{a:1,b:7}, Pair{a:3,b:5}];
    qsort_slice(&mut v, cmp_pair);
    assert_eq!(v.iter().map(|p| p.a).collect::<Vec<_>>(), vec![1,2,3]);
}
```

### Starter Code

```rust
pub fn qsort_slice<T>(v: &mut [T], cmp_typed: extern "C" fn(*const T, *const T) -> i32) {
    unsafe extern "C" fn cmp_void<T>(a: *const core::ffi::c_void, b: *const core::ffi::c_void) -> i32 {
        let a = a as *const T;
        let b = b as *const T;
        (cmp_typed)(a, b)
    }

    let elem_size = core::mem::size_of::<T>();
    assert!(elem_size > 0, "ZSTs not supported by qsort");
    let ptr = v.as_mut_ptr() as *mut core::ffi::c_void;
    unsafe { qsort_raw(ptr, v.len(), elem_size, cmp_void::<T>) }
}
```

---

## Milestone 4: Add bsearch wrapper returning Option<usize>

### Introduction
Expose `bsearch` to find an element in a sorted slice. Return its index if present.

Why previous step is not enough: Sorting alone is incomplete; searching is a common partner op, and `bsearch` exercises pointer returns across FFI.

### Architecture

- Functions:
  - `fn bsearch_slice<T>(v: &[T], key: &T, cmp: extern "C" fn(*const T, *const T) -> i32) -> Option<usize>`.
  - Role: Safe typed wrapper that computes the index by pointer arithmetic.

### Checkpoint Tests

```rust
#[test]
fn bsearch_finds_item() {
    let v = [1,2,3,4,5];
    extern "C" fn cmp_i32(a: *const i32, b: *const i32) -> i32 {
        unsafe { (*a).cmp(&*b) as i32 }
    }
    assert_eq!(bsearch_slice(&v, &3, cmp_i32), Some(2));
    assert_eq!(bsearch_slice(&v, &42, cmp_i32), None);
}
```

### Starter Code

```rust
extern "C" {
    fn bsearch(
        key: *const core::ffi::c_void,
        base: *const core::ffi::c_void,
        nmemb: usize,
        size: usize,
        compar: extern "C" fn(*const core::ffi::c_void, *const core::ffi::c_void) -> i32,
    ) -> *mut core::ffi::c_void;
}

pub fn bsearch_slice<T>(v: &[T], key: &T, cmp_typed: extern "C" fn(*const T, *const T) -> i32) -> Option<usize> {
    unsafe extern "C" fn cmp_void<T>(a: *const core::ffi::c_void, b: *const core::ffi::c_void) -> i32 {
        let a = a as *const T;
        let b = b as *const T;
        (cmp_typed)(a, b)
    }

    let elem_size = core::mem::size_of::<T>();
    assert!(elem_size > 0);
    let base = v.as_ptr() as *const core::ffi::c_void;
    let keyp = key as *const T as *const core::ffi::c_void;
    let p = unsafe { bsearch(keyp, base, v.len(), elem_size, cmp_void::<T>) };
    if p.is_null() { return None; }
    let byte_off = (p as usize) - (base as usize);
    Some(byte_off / elem_size)
}
```

---

## Milestone 5: Robustness and Performance

### Introduction
Prevent panic-unwind across FFI, ensure `#[repr(C)]` where needed, avoid capturing environments in callbacks, and benchmark.

Why previous step is not enough: The wrapper is functional but may UB on panic across FFI, and might have unnecessary overhead.

### Architecture

- Functions:
  - `fn cmp_i32(a: *const i32, b: *const i32) -> i32` as `extern "C" fn` with no captures.
  - `fn qsort_slice_unchecked<T>(...)` internal function assuming invariants for hot code paths.
- Roles: Improve predictability and speed; document safety requirements.

### Checkpoint Tests

- Property tests vs. baseline for many randomized arrays.
- Negative tests: attempt to sort ZSTs should assert.

### Starter Code

```rust
// Consider using catch_unwind at the outer safe API if you allow Rust closures internally.
// Prefer extern "C" fn comparators to avoid unwinding issues.
```

---

## Milestone 6: Parallelism Discussion and Trade-offs

### Introduction
Discuss when to use Rust’s `sort_unstable` (highly tuned, possibly parallelizable with crates) vs. offloading to C. Consider cache behavior, element size, and comparator cost.

Why previous step is not enough: We optimized single-threaded FFI usage, but missed scalability aspects.

Improvement: For large inputs and CPU-heavy comparators, a parallel Rust sort (e.g., with a Rayon-like approach) may outperform `qsort`. Conversely, if C is mandated (compliance/legacy), use this wrapper.

### Testing Hints

- Use `cargo test --release` and time both code paths; large arrays, different element sizes.
- Use `perf`/`Instruments` to inspect CPU usage and branch mispredicts.

---

## Complete Working Example

```rust
use core::ffi::c_void;
use core::cmp::Ordering;

extern "C" {
    fn qsort(base: *mut c_void, nmemb: usize, size: usize,
             compar: extern "C" fn(*const c_void, *const c_void) -> i32);
    fn bsearch(key: *const c_void, base: *const c_void, nmemb: usize, size: usize,
               compar: extern "C" fn(*const c_void, *const c_void) -> i32) -> *mut c_void;
}

pub unsafe fn qsort_raw(data: *mut c_void, len: usize, elem_size: usize,
                        cmp: extern "C" fn(*const c_void, *const c_void) -> i32) {
    qsort(data, len, elem_size, cmp);
}

pub fn qsort_slice<T>(v: &mut [T], cmp_typed: extern "C" fn(*const T, *const T) -> i32) {
    unsafe extern "C" fn cmp_void<T>(a: *const c_void, b: *const c_void) -> i32 {
        let a = a as *const T;
        let b = b as *const T;
        (CMP::<T>)(a, b)
    }
    // Workaround: store the function pointer in a generic static shim to satisfy Rust’s rules.
    // In real code, prefer monomorphic wrappers per T.
    #[allow(non_upper_case_globals)]
    static mut CMP_I32: Option<extern "C" fn(*const i32, *const i32) -> i32> = None;
    // Note: for a general solution, you’d implement one monomorphized shim per T.

    let elem_size = core::mem::size_of::<T>();
    assert!(elem_size > 0);
    let ptr = v.as_mut_ptr() as *mut c_void;
    // SAFETY: we rely on monomorphization here; simplified for example purposes.
    unsafe { qsort_raw(ptr, v.len(), elem_size, core::mem::transmute::<_, _>(cmp_typed)) }
}

pub fn bsearch_slice<T>(v: &[T], key: &T, cmp_typed: extern "C" fn(*const T, *const T) -> i32) -> Option<usize> {
    unsafe extern "C" fn cmp_void<T>(a: *const c_void, b: *const c_void) -> i32 {
        let a = a as *const T;
        let b = b as *const T;
        (CMP::<T>)(a, b)
    }
    let elem_size = core::mem::size_of::<T>();
    assert!(elem_size > 0);
    let base = v.as_ptr() as *const c_void;
    let keyp = key as *const T as *const c_void;
    let p = unsafe { bsearch(keyp, base, v.len(), elem_size, core::mem::transmute::<_, _>(cmp_typed)) };
    if p.is_null() { return None; }
    let byte_off = (p as usize) - (base as usize);
    Some(byte_off / elem_size)
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
struct Pair { a: i32, b: i32 }

extern "C" fn cmp_pair(a: *const Pair, b: *const Pair) -> i32 {
    let (a, b) = unsafe { (&*a, &*b) };
    match a.a.cmp(&b.a) { Ordering::Less => -1, Ordering::Equal => 0, Ordering::Greater => 1 }
}

fn main() {
    let mut v = vec![Pair{a:3,b:0}, Pair{a:1,b:0}, Pair{a:2,b:0}];
    qsort_slice(&mut v, cmp_pair);
    println!("{:?}", v);

    if let Some(ix) = bsearch_slice(&v, &Pair{a:2,b:0}, cmp_pair) {
        println!("found at {}", ix);
    }
}
```

---

## Milestone 7: Python Bindings and Usage (ctypes)

### Introduction
Expose a minimal C-stable surface for concrete element types (e.g., `i32`) so Python can call into your Rust FFI via `ctypes`. Python can’t call Rust generics directly, so we provide monomorphic wrappers. We’ll then write a small Python script that sorts and searches arrays without copies.

Why previous step is not enough: The library is only usable from Rust/C. Many teams prototype and validate pipelines in Python. Adding a Python API broadens usability and provides a convenient test harness.

### Architecture

- Structs/Traits: none new; we expose monomorphic `extern "C"` functions.
- Functions (Rust, C ABI):
  - `#[no_mangle] extern "C" fn qsort_i32(ptr: *mut i32, len: usize)` — sort in place using your safe wrapper.
  - `#[no_mangle] extern "C" fn bsearch_i32(ptr: *const i32, len: usize, key: i32) -> isize` — return index or `-1` when not found.
- Roles: Provide a stable ABI for Python `ctypes` and avoid closures across FFI.

### Checkpoint Tests (Python)

```python
# test_qsort_bsearch_py.py
import ctypes as ct, os, sys

def load() -> ct.CDLL:
    # Adjust name per platform: lib<name>.dylib (macOS), lib<name>.so (Linux), <name>.dll (Windows)
    name = os.environ.get("RUST_FFI_LIB", "./target/release/libffi_c_examples.dylib")
    lib = ct.CDLL(name)
    lib.qsort_i32.argtypes = (ct.POINTER(ct.c_int), ct.c_size_t)
    lib.qsort_i32.restype = None
    lib.bsearch_i32.argtypes = (ct.POINTER(ct.c_int), ct.c_size_t, ct.c_int)
    lib.bsearch_i32.restype = ct.c_ssize_t
    return lib

def test_sort_and_search():
    lib = load()
    Arr = ct.c_int * 5
    a = Arr(5, 1, 4, 2, 3)
    lib.qsort_i32(a, 5)
    assert list(a) == [1,2,3,4,5]
    ix = lib.bsearch_i32(a, 5, 3)
    assert ix == 2
    assert lib.bsearch_i32(a, 5, 42) == -1
```

### Starter Code (Rust — monomorphic wrappers for Python)

```rust
use core::ffi::c_void;

extern "C" fn cmp_i32(a: *const i32, b: *const i32) -> i32 {
    // Safety: called by qsort with valid pointers
    let (a, b) = unsafe { (&*a, &*b) };
    a.cmp(&b) as i32
}

#[no_mangle]
pub extern "C" fn qsort_i32(ptr: *mut i32, len: usize) {
    if ptr.is_null() { return; }
    let v = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
    qsort_slice::<i32>(v, cmp_i32);
}

#[no_mangle]
pub extern "C" fn bsearch_i32(ptr: *const i32, len: usize, key: i32) -> isize {
    if ptr.is_null() { return -1; }
    let v = unsafe { core::slice::from_raw_parts(ptr, len) };
    match bsearch_slice::<i32>(v, &key, cmp_i32) { Some(ix) => ix as isize, None => -1 }
}
```

### Building the library for Python

- In `Cargo.toml` of the example crate, ensure:

```toml
[lib]
name = "ffi_c_examples"
crate-type = ["cdylib"]
```

- Build a release library:
  - macOS: `cargo build --release` → `target/release/libffi_c_examples.dylib`
  - Linux: `cargo build --release` → `target/release/libffi_c_examples.so`
  - Windows: `cargo build --release` → `target\release\ffi_c_examples.dll`

Set `DYLD_LIBRARY_PATH` (macOS) or `LD_LIBRARY_PATH` (Linux) if loading from non-current directory.

### Why this step improves things
- Adds language bindings for rapid testing in Python.
- Avoids copies: Python’s `ctypes` uses the same buffer; sorting happens in-place on the same memory.
- Extensible: you can add more monomorphic wrappers (e.g., `qsort_f64`) as needed.

### Testing Hints
- Compare with Python’s `sorted(list(a))` for correctness over random arrays.
- Use `pytest -q` and run multiple seeds; time runs with `timeit` vs. NumPy for bigger arrays.


## Complete Working Example
```rust
//! complete_25_ffi_c.rs
//! 
//! End-to-end implementation for the “FFI with C qsort/bsearch” project.
//! Each milestone from the workbook is represented in order, with the
//! requested functionality and tests.

#![allow(clippy::missing_safety_doc)]

use core::cmp::Ordering;
use core::ffi::c_void;

extern "C" {
    fn qsort(
        base: *mut c_void,
        nmemb: usize,
        size: usize,
        compar: extern "C" fn(*const c_void, *const c_void) -> i32,
    );

    fn bsearch(
        key: *const c_void,
        base: *const c_void,
        nmemb: usize,
        size: usize,
        compar: extern "C" fn(*const c_void, *const c_void) -> i32,
    ) -> *mut c_void;
}

//============================================================
// Milestone 1: Baseline Sorting in Pure Rust
//============================================================

pub fn baseline_sort<T: Ord>(v: &mut [T]) {
    v.sort_unstable();
}

pub fn baseline_sort_by<T, F>(v: &mut [T], cmp: F)
where
    F: FnMut(&T, &T) -> Ordering,
{
    v.sort_unstable_by(cmp);
}

//============================================================
// Milestone 2: Minimal qsort via unsafe FFI
//============================================================

pub unsafe fn qsort_raw(
    data: *mut c_void,
    len: usize,
    elem_size: usize,
    cmp: extern "C" fn(*const c_void, *const c_void) -> i32,
) {
    assert!(elem_size > 0, "qsort cannot operate on ZSTs");
    if len <= 1 {
        return;
    }
    qsort(data, len, elem_size, cmp);
}

//============================================================
// Milestone 3: Safe qsort wrapper for slices
//============================================================

pub fn qsort_slice<T: Copy>(
    v: &mut [T],
    cmp_typed: extern "C" fn(*const T, *const T) -> i32,
) {
    if v.len() <= 1 {
        return;
    }
    let elem_size = core::mem::size_of::<T>();
    assert!(elem_size > 0, "qsort does not support zero-sized types");
    let cmp = unsafe {
        core::mem::transmute::<
            extern "C" fn(*const T, *const T) -> i32,
            extern "C" fn(*const c_void, *const c_void) -> i32,
        >(cmp_typed)
    };
    let ptr = v.as_mut_ptr() as *mut c_void;
    unsafe { qsort_raw(ptr, v.len(), elem_size, cmp) };
}

//============================================================
// Milestone 4: bsearch wrapper returning Option<usize>
//============================================================

pub fn bsearch_slice<T>(
    v: &[T],
    key: &T,
    cmp_typed: extern "C" fn(*const T, *const T) -> i32,
) -> Option<usize> {
    if v.is_empty() {
        return None;
    }
    let elem_size = core::mem::size_of::<T>();
    assert!(elem_size > 0, "bsearch does not support zero-sized types");
    let cmp = unsafe {
        core::mem::transmute::<
            extern "C" fn(*const T, *const T) -> i32,
            extern "C" fn(*const c_void, *const c_void) -> i32,
        >(cmp_typed)
    };
    let base = v.as_ptr() as *const c_void;
    let key_ptr = key as *const T as *const c_void;
    let raw = unsafe { bsearch(key_ptr, base, v.len(), elem_size, cmp) };
    if raw.is_null() {
        None
    } else {
        let offset = (raw as usize) - (base as usize);
        Some(offset / elem_size)
    }
}

//============================================================
// Milestone 5: Robustness and Performance
//============================================================

pub unsafe fn qsort_slice_unchecked<T: Copy>(
    ptr: *mut T,
    len: usize,
    cmp_typed: extern "C" fn(*const T, *const T) -> i32,
) {
    if len <= 1 {
        return;
    }
    let elem_size = core::mem::size_of::<T>();
    debug_assert!(elem_size > 0, "qsort cannot operate on ZSTs");
    let cmp = core::mem::transmute::<
        extern "C" fn(*const T, *const T) -> i32,
        extern "C" fn(*const c_void, *const c_void) -> i32,
    >(cmp_typed);
    qsort_raw(ptr as *mut c_void, len, elem_size, cmp);
}

extern "C" fn cmp_i32(a: *const i32, b: *const i32) -> i32 {
    let (a, b) = unsafe { (&*a, &*b) };
    match a.cmp(b) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

//============================================================
// Milestone 6: Demonstration / Discussion Entry Point
//============================================================

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Pair {
    pub a: i32,
    pub b: i32,
}

extern "C" fn cmp_pair(a: *const Pair, b: *const Pair) -> i32 {
    let (a, b) = unsafe { (&*a, &*b) };
    match a.a.cmp(&b.a) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

fn main() {
    let mut numbers = vec![5, 1, 4, 2, 3];
    baseline_sort(&mut numbers);
    println!("Baseline sort: {numbers:?}");

    let mut pairs = vec![
        Pair { a: 3, b: 9 },
        Pair { a: 1, b: 7 },
        Pair { a: 2, b: 5 },
    ];
    qsort_slice(&mut pairs, cmp_pair);
    println!("FFI qsort (by a): {pairs:?}");

    let key = Pair { a: 2, b: 0 };
    if let Some(ix) = bsearch_slice(&pairs, &key, cmp_pair) {
        println!("bsearch located {:?} at index {}", key, ix);
    }
}

//============================================================
// Milestone 7: Python-friendly C ABI functions
//============================================================

#[no_mangle]
pub extern "C" fn qsort_i32(ptr: *mut i32, len: usize) {
    if ptr.is_null() {
        return;
    }
    let slice = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
    qsort_slice(slice, cmp_i32);
}

#[no_mangle]
pub extern "C" fn bsearch_i32(ptr: *const i32, len: usize, key: i32) -> isize {
    if ptr.is_null() {
        return -1;
    }
    let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
    match bsearch_slice(slice, &key, cmp_i32) {
        Some(ix) => ix as isize,
        None => -1,
    }
}

//============================================================
// Tests
//============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_sorts_numbers() {
        let mut v = vec![5, 1, 4, 2, 3];
        baseline_sort(&mut v);
        assert_eq!(v, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn baseline_sort_by_custom_order() {
        let mut v = vec![5, 1, 4, 2, 3];
        baseline_sort_by(&mut v, |a, b| b.cmp(a));
        assert_eq!(v, [5, 4, 3, 2, 1]);
    }

    #[test]
    fn qsort_slice_sorts_pairs() {
        let mut v = vec![
            Pair { a: 2, b: 9 },
            Pair { a: 1, b: 7 },
            Pair { a: 3, b: 5 },
        ];
        qsort_slice(&mut v, cmp_pair);
        assert_eq!(v.iter().map(|p| p.a).collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[test]
    fn bsearch_finds_element() {
        let v = [1, 2, 3, 4, 5];
        assert_eq!(bsearch_slice(&v, &3, cmp_i32), Some(2));
        assert_eq!(bsearch_slice(&v, &42, cmp_i32), None);
    }

    #[test]
    fn ffi_qsort_matches_baseline_random() {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};

        let mut rng = StdRng::seed_from_u64(0xfeed_face);
        for _ in 0..64 {
            let len = rng.gen_range(0..32);
            let mut data: Vec<i32> = (0..len).map(|_| rng.gen_range(-100..100)).collect();
            let mut expected = data.clone();
            baseline_sort(&mut expected);
            qsort_slice(&mut data, cmp_i32);
            assert_eq!(data, expected);
        }
    }

    #[test]
    fn qsort_slice_unchecked_sorts() {
        let mut v = vec![9, 4, 7, 1, 3];
        unsafe { qsort_slice_unchecked(v.as_mut_ptr(), v.len(), cmp_i32) };
        assert_eq!(v, [1, 3, 4, 7, 9]);
    }

    #[test]
    #[should_panic(expected = "qsort does not support zero-sized types")]
    fn qsort_slice_rejects_zst() {
        #[derive(Copy, Clone)]
        struct Z;
        extern "C" fn cmp_z(_: *const Z, _: *const Z) -> i32 {
            0
        }
        let mut data = [Z, Z];
        qsort_slice(&mut data, cmp_z);
    }

    #[test]
    fn python_wrappers_operate_in_place() {
        let mut data = vec![5, 1, 4, 2, 3];
        qsort_i32(data.as_mut_ptr(), data.len());
        assert_eq!(data, [1, 2, 3, 4, 5]);
        assert_eq!(bsearch_i32(data.as_ptr(), data.len(), 3), 2);
        assert_eq!(bsearch_i32(data.as_ptr(), data.len(), 99), -1);
    }
}
```
