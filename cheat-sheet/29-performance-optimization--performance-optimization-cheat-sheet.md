
### Performance Optimization Cheat Sheet

```rust
// ===== MEMORY ALLOCATION OPTIMIZATION =====

// Bad: Allocating on every call
fn bad_allocation(n: usize) -> Vec<i32> {
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i as i32);
    }
    vec
}

// Good: Pre-allocate with capacity
fn good_allocation(n: usize) -> Vec<i32> {
    let mut vec = Vec::with_capacity(n);
    for i in 0..n {
        vec.push(i as i32);
    }
    vec
}

// Best: Use iterator and collect (single allocation)
fn best_allocation(n: usize) -> Vec<i32> {
    (0..n).map(|i| i as i32).collect()
}

// Reuse allocations
fn reuse_allocation() {
    let mut buffer = Vec::with_capacity(1000);
    
    for _ in 0..100 {
        buffer.clear(); // Keeps capacity
        // Fill buffer with new data
        for i in 0..1000 {
            buffer.push(i);
        }
        // Process buffer
    }
}

// ===== STRING OPTIMIZATION =====

// Bad: Multiple allocations
fn bad_string_concat(parts: &[&str]) -> String {
    let mut result = String::new();
    for part in parts {
        result = result + part; // Creates new String each time
    }
    result
}

// Good: Pre-allocate capacity
fn good_string_concat(parts: &[&str]) -> String {
    let total_len: usize = parts.iter().map(|s| s.len()).sum();
    let mut result = String::with_capacity(total_len);
    for part in parts {
        result.push_str(part);
    }
    result
}

// Best: Use join
fn best_string_concat(parts: &[&str]) -> String {
    parts.join("")
}

// String formatting optimization
fn string_formatting() {
    // Bad: Multiple allocations
    let bad = String::from("Hello") + " " + "World";
    
    // Good: format! macro (single allocation)
    let good = format!("Hello {}", "World");
    
    // Better: Write to existing buffer
    let mut buffer = String::with_capacity(100);
    use std::fmt::Write;
    write!(buffer, "Hello {}", "World").unwrap();
}

// ===== CLONE AVOIDANCE =====

// Bad: Unnecessary clones
fn bad_clone_usage(data: &Vec<i32>) -> i32 {
    let cloned = data.clone(); // Unnecessary
    cloned.iter().sum()
}

// Good: Use references
fn good_clone_usage(data: &Vec<i32>) -> i32 {
    data.iter().sum()
}

// Use Cow (Clone on Write) for conditional ownership
use std::borrow::Cow;

fn process_string(s: Cow<str>) -> Cow<str> {
    if s.contains("error") {
        // Need to modify, so clone
        Cow::Owned(s.replace("error", "warning"))
    } else {
        // No modification needed, return borrowed
        s
    }
}

fn cow_example() {
    let borrowed = process_string(Cow::Borrowed("Hello"));
    let owned = process_string(Cow::Borrowed("This is an error"));
}

// ===== ITERATOR OPTIMIZATION =====

// Bad: Intermediate collections
fn bad_iteration(data: &[i32]) -> Vec<i32> {
    let doubled: Vec<_> = data.iter().map(|x| x * 2).collect();
    let filtered: Vec<_> = doubled.iter().filter(|x| *x > 10).collect();
    let squared: Vec<_> = filtered.iter().map(|x| x * x).collect();
    squared.iter().copied().collect()
}

// Good: Chain iterators (zero-cost abstraction)
fn good_iteration(data: &[i32]) -> Vec<i32> {
    data.iter()
        .map(|x| x * 2)
        .filter(|x| *x > 10)
        .map(|x| x * x)
        .collect()
}

// Use iterator methods instead of manual loops
fn iterator_methods(data: &[i32]) -> i32 {
    // Bad: Manual loop
    let mut sum = 0;
    for &x in data {
        if x > 0 {
            sum += x * 2;
        }
    }
    
    // Good: Iterator chain
    data.iter()
        .filter(|&&x| x > 0)
        .map(|x| x * 2)
        .sum()
}

// ===== BOUNDS CHECKING ELIMINATION =====

// Bad: Multiple bounds checks
fn bad_bounds_check(data: &[i32], indices: &[usize]) -> i32 {
    let mut sum = 0;
    for &idx in indices {
        sum += data[idx]; // Bounds check on each access
    }
    sum
}

// Good: Use get_unchecked (when safe)
fn good_bounds_check(data: &[i32], indices: &[usize]) -> i32 {
    let mut sum = 0;
    for &idx in indices {
        if idx < data.len() {
            sum += unsafe { *data.get_unchecked(idx) };
        }
    }
    sum
}

// Better: Iterator with chunks
fn better_bounds_check(data: &[i32]) -> i32 {
    data.chunks(4)
        .map(|chunk| chunk.iter().sum::<i32>())
        .sum()
}

// ===== SMALL STRING OPTIMIZATION =====
use std::borrow::Cow;

// Use stack allocation for small strings
enum SmallString {
    Stack([u8; 23], u8), // 23 bytes + 1 length
    Heap(String),
}

impl SmallString {
    fn new(s: &str) -> Self {
        if s.len() <= 23 {
            let mut arr = [0u8; 23];
            arr[..s.len()].copy_from_slice(s.as_bytes());
            SmallString::Stack(arr, s.len() as u8)
        } else {
            SmallString::Heap(s.to_string())
        }
    }
    
    fn as_str(&self) -> &str {
        match self {
            SmallString::Stack(arr, len) => {
                std::str::from_utf8(&arr[..*len as usize]).unwrap()
            }
            SmallString::Heap(s) => s.as_str(),
        }
    }
}

// Or use smallvec/smartstring crates
// use smartstring::alias::String as SmartString;

// ===== ZERO-COPY PARSING =====

// Bad: Allocating strings while parsing
fn bad_parse(input: &str) -> Vec<String> {
    input.split(',')
        .map(|s| s.trim().to_string())
        .collect()
}

// Good: Return references (zero-copy)
fn good_parse(input: &str) -> Vec<&str> {
    input.split(',')
        .map(|s| s.trim())
        .collect()
}

// Use bytes for non-UTF8 parsing
fn parse_bytes(input: &[u8]) -> Vec<&[u8]> {
    input.split(|&b| b == b',').collect()
}

// ===== INLINE OPTIMIZATION =====

// Force inlining for hot paths
#[inline(always)]
fn hot_function(x: i32, y: i32) -> i32 {
    x * x + y * y
}

// Prevent inlining for cold paths
#[inline(never)]
fn cold_error_path(msg: &str) {
    eprintln!("Error: {}", msg);
}

// Let compiler decide (default)
#[inline]
fn normal_function(x: i32) -> i32 {
    x * 2
}

// ===== BRANCH PREDICTION =====

// Use likely/unlikely hints (nightly)
// #![feature(core_intrinsics)]
// use std::intrinsics::{likely, unlikely};

fn branch_prediction(x: i32) -> i32 {
    // Hot path first
    if x > 0 {
        x * 2
    } else {
        // Cold path
        x * 3
    }
}

// Avoid branches in hot loops
fn avoid_branches(data: &[i32]) -> Vec<i32> {
    data.iter()
        .map(|&x| {
            // Bad: Branch in loop
            // if x > 0 { x * 2 } else { x * 3 }
            
            // Good: Branchless
            let mask = (x > 0) as i32;
            x * (2 * mask + 3 * (1 - mask))
        })
        .collect()
}

// ===== SIMD OPTIMIZATION =====
use std::arch::x86_64::*;

// Manual SIMD (unsafe)
#[cfg(target_arch = "x86_64")]
unsafe fn simd_sum(data: &[f32]) -> f32 {
    if data.len() < 4 {
        return data.iter().sum();
    }
    
    let mut sum = _mm_setzero_ps();
    let chunks = data.chunks_exact(4);
    let remainder = chunks.remainder();
    
    for chunk in chunks {
        let values = _mm_loadu_ps(chunk.as_ptr());
        sum = _mm_add_ps(sum, values);
    }
    
    let mut result = [0f32; 4];
    _mm_storeu_ps(result.as_mut_ptr(), sum);
    
    result.iter().sum::<f32>() + remainder.iter().sum::<f32>()
}

// Auto-vectorization friendly code
fn auto_vectorize(a: &[f32], b: &[f32], c: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), c.len());
    
    for i in 0..a.len() {
        c[i] = a[i] + b[i]; // Compiler can vectorize
    }
}

// ===== CACHE OPTIMIZATION =====

// Bad: Cache-unfriendly access (column-major)
fn bad_cache_access(matrix: &Vec<Vec<i32>>) -> i32 {
    let mut sum = 0;
    for col in 0..matrix[0].len() {
        for row in 0..matrix.len() {
            sum += matrix[row][col];
        }
    }
    sum
}

// Good: Cache-friendly access (row-major)
fn good_cache_access(matrix: &Vec<Vec<i32>>) -> i32 {
    let mut sum = 0;
    for row in matrix {
        for &val in row {
            sum += val;
        }
    }
    sum
}

// Structure of Arrays (SoA) vs Array of Structures (AoS)
// Bad: Array of Structures (poor cache locality)
#[derive(Clone)]
struct PointAoS {
    x: f32,
    y: f32,
    z: f32,
}

fn process_aos(points: &[PointAoS]) -> f32 {
    points.iter().map(|p| p.x + p.y + p.z).sum()
}

// Good: Structure of Arrays (better cache locality)
struct PointsSoA {
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
}

fn process_soa(points: &PointsSoA) -> f32 {
    points.x.iter()
        .zip(&points.y)
        .zip(&points.z)
        .map(|((&x, &y), &z)| x + y + z)
        .sum()
}

// ===== DATA STRUCTURE OPTIMIZATION =====

// Use appropriate data structures
use std::collections::{HashMap, BTreeMap, HashSet};

// HashMap for O(1) lookups
fn use_hashmap(data: &[(String, i32)]) -> HashMap<String, i32> {
    data.iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect()
}

// BTreeMap for sorted keys
fn use_btreemap(data: &[(i32, String)]) -> BTreeMap<i32, String> {
    data.iter()
        .map(|(k, v)| (*k, v.clone()))
        .collect()
}

// SmallVec for small vectors on stack
// use smallvec::{SmallVec, smallvec};
// type SmallVector = SmallVec<[i32; 8]>;

// ArrayVec for fixed-size on stack
// use arrayvec::ArrayVec;

// ===== LAZY EVALUATION =====

// Bad: Eager computation
fn eager_computation(data: &[i32]) -> Vec<i32> {
    let doubled: Vec<_> = data.iter().map(|x| x * 2).collect();
    let filtered: Vec<_> = doubled.iter().filter(|&&x| x > 10).collect();
    filtered.iter().take(5).copied().collect()
}

// Good: Lazy with iterators
fn lazy_computation(data: &[i32]) -> Vec<i32> {
    data.iter()
        .map(|x| x * 2)
        .filter(|&x| x > 10)
        .take(5)
        .collect()
}

// OnceCell for lazy initialization
use std::sync::OnceLock;

static EXPENSIVE: OnceLock<Vec<i32>> = OnceLock::new();

fn get_expensive_data() -> &'static Vec<i32> {
    EXPENSIVE.get_or_init(|| {
        // Expensive computation
        (0..1000).collect()
    })
}

// ===== MEMORY LAYOUT OPTIMIZATION =====

// Use #[repr(C)] for predictable layout
#[repr(C)]
struct OptimizedStruct {
    small: u8,
    large: u64,
    medium: u32,
}

// Pack struct to reduce size
#[repr(packed)]
struct PackedStruct {
    a: u8,
    b: u64,
    c: u8,
}

// Align for better performance
#[repr(align(64))] // Cache line alignment
struct AlignedStruct {
    data: [u8; 64],
}

// Order fields by size (largest first) to minimize padding
struct WellOrdered {
    large: u64,     // 8 bytes
    medium: u32,    // 4 bytes
    small: u16,     // 2 bytes
    tiny: u8,       // 1 byte
    // 1 byte padding
} // Total: 16 bytes

struct PoorlyOrdered {
    tiny: u8,       // 1 byte + 7 padding
    large: u64,     // 8 bytes
    small: u16,     // 2 bytes + 2 padding
    medium: u32,    // 4 bytes
} // Total: 24 bytes

// ===== AVOIDING ALLOCATIONS =====

// Use references instead of owned values
fn no_allocation(data: &[i32], threshold: i32) -> Vec<&i32> {
    data.iter().filter(|&&x| x > threshold).collect()
}

// Return iterators instead of collections
fn return_iterator(data: &[i32]) -> impl Iterator<Item = i32> + '_ {
    data.iter().map(|&x| x * 2).filter(|&x| x > 10)
}

// Use stack allocation for small arrays
fn stack_array() {
    let arr = [0i32; 100]; // On stack
    // Process arr
}

// ===== CONST EVALUATION =====

// Compute at compile time
const fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

const FIB_10: u32 = fibonacci(10);

// Const generics for zero-cost abstractions
fn const_generic_array<const N: usize>(arr: [i32; N]) -> i32 {
    arr.iter().sum()
}

// ===== PARALLEL PROCESSING =====
use std::thread;
use std::sync::{Arc, Mutex};

// Parallel processing with rayon
// use rayon::prelude::*;

fn parallel_sum(data: &[i32]) -> i32 {
    // data.par_iter().sum()
    
    // Manual threading
    let chunk_size = data.len() / 4;
    let chunks: Vec<_> = data.chunks(chunk_size).collect();
    
    let handles: Vec<_> = chunks.into_iter()
        .map(|chunk| {
            let chunk = chunk.to_vec();
            thread::spawn(move || {
                chunk.iter().sum::<i32>()
            })
        })
        .collect();
    
    handles.into_iter()
        .map(|h| h.join().unwrap())
        .sum()
}

// ===== BUFFERED I/O =====
use std::io::{BufReader, BufWriter, Read, Write};
use std::fs::File;

fn buffered_io() -> std::io::Result<()> {
    // Bad: Unbuffered
    // let mut file = File::open("data.txt")?;
    
    // Good: Buffered
    let file = File::open("data.txt")?;
    let mut reader = BufReader::new(file);
    
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;
    
    // Buffered writing
    let file = File::create("output.txt")?;
    let mut writer = BufWriter::new(file);
    writer.write_all(b"Hello, world!")?;
    
    Ok(())
}

// ===== PROFILING HELPERS =====

// Time measurement
use std::time::Instant;

fn measure_time<F: FnOnce() -> R, R>(f: F) -> (R, std::time::Duration) {
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

fn timing_example() {
    let (result, duration) = measure_time(|| {
        // Expensive operation
        (0..1000000).sum::<i32>()
    });
    println!("Result: {}, Time: {:?}", result, duration);
}

// Memory usage tracking
fn memory_usage() {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    struct CountingAllocator;
    
    static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
    
    unsafe impl GlobalAlloc for CountingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
            System.alloc(layout)
        }
        
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
            System.dealloc(ptr, layout)
        }
    }
}

// ===== COMPILER HINTS =====

// Hint that value is unlikely to be zero
fn assume_nonzero(x: i32) -> i32 {
    if x == 0 {
        unsafe { std::hint::unreachable_unchecked() }
    }
    x
}

// Cold attribute for error paths
#[cold]
fn error_handler() {
    panic!("Fatal error");
}

// ===== OPTIMIZATION FLAGS =====
/*
In Cargo.toml:

[profile.release]
opt-level = 3              # Maximum optimization
lto = true                 # Link-time optimization
codegen-units = 1          # Better optimization, slower compile
panic = 'abort'            # Smaller binary
strip = true               # Strip symbols

[profile.bench]
inherits = "release"
debug = true               # Keep debug info for profiling
*/

// ===== BENCHMARKING =====
#[cfg(test)]
mod benches {
    use super::*;
    
    // Use criterion crate for accurate benchmarking
    // use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    // fn benchmark(c: &mut Criterion) {
    //     c.bench_function("allocation", |b| {
    //         b.iter(|| good_allocation(black_box(1000)))
    //     });
    // }
}
```


