# Performance Optimization
This chapter explores performance optimization: profiling to find bottlenecks, allocation reduction techniques, cache-friendly data structures, zero-cost abstractions, and compiler optimizations for maximum performance.

## Pattern 1: Profiling Strategies

**Problem**: Guessing at bottlenecks leads to wasted optimization—without data you tweak the wrong code and still miss production hotspots.

**Solution**: Profile first: perf/cargo-flamegraph/Instruments for CPU, heaptrack/dhat for allocations, Criterion for microbenchmarks; always build in release with symbols and re-measure after each change.

**Why It Matters**: Profiling exposes the surprise 20% of code that burns 80% of time, shows call stacks responsible, and proves whether an optimization actually helped.

**Use Cases**: Locating hot functions, tracking allocations, validating optimizations, investigating production regressions, and comparing algorithm variants or scaling behavior.


### Example: Which bottleneck

Why does profiling matter. A data processing pipeline has multiple potential bottlenecks: validation iterating characters, transformation allocating strings, or collection resizing. Without measurement, optimization guesses are usually wrong.

```rust
// Which is the bottleneck?
fn process_data(items: Vec<String>) -> Vec<String> {
    items.iter()
        .filter(|s| validate(s))      // Is it validation?
        .map(|s| transform(s))         // Is it transformation?
        .collect()                      // Is it allocation?
}

fn validate(s: &str) -> bool {
    s.len() > 10 && s.chars().all(|c| c.is_alphanumeric())
}

fn transform(s: &str) -> String {
    s.to_uppercase()
}
```

You might guess `transform` is slow because it allocates. Or maybe `validate` because it iterates characters. Only profiling tells you the truth. Maybe `collect()` dominates because the vector is huge. Or maybe `validate` is called millions of times with tiny strings, making the overhead of `chars()` matter.

### Example: CPU Profiling with perf

 Linux CPU profiling with perf and flamegraph generation. Build with debug symbols in release mode, record samples with call graphs, then visualize. Wide flamegraph bars indicate expensive functions; the y-axis shows call stack depth.

```bash
# Build with debug symbols in release mode
# Add to Cargo.toml:
# [profile.release]
# debug = true

cargo build --release

# Record profiling data
perf record --call-graph dwarf ./target/release/myapp

# View report
perf report

# Generate flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg
```

The flamegraph visualizes where time is spent. Wide bars are expensive functions. The stack shows the call chain that led there.

Reading a flamegraph:
- **X-axis**: Percentage of samples (wider = more time)
- **Y-axis**: Call stack depth (bottom = entry point, top = current function)
- **Color**: Meaningless, just for visibility

Look for wide bars at the top—those are your bottlenecks.

### Example: Using cargo-flamegraph

 cargo-flamegraph example for simplified profiling. One command generates an interactive SVG flamegraph showing CPU time distribution. It wraps perf/dtrace automatically, making profiling accessible without manual tool configuration.

```bash
# Install
cargo install flamegraph

# Generate flamegraph (requires root on Linux)
cargo flamegraph

# Or without root (less accurate)
cargo flamegraph --dev
```

This generates `flamegraph.svg` automatically.

### Example: Profiling with Instruments (macOS)

 MacOS profiling using Instruments Time Profiler. The GUI provides interactive call trees, hotspot visualization, and timeline analysis. Build with debug symbols in release mode for meaningful function names in the profile.

```bash
# Build with debug symbols
cargo build --release

# Open in Instruments
instruments -t "Time Profiler" ./target/release/myapp
```

Instruments provides a GUI for exploring hotspots, viewing call trees, and drilling into specific functions.

### Example: Profiling in Code with Benchmarks

This example uses Criterion to benchmark individual pipeline components separately. By measuring validate, transform, and the full pipeline independently, you identify which specific function dominates execution time rather than guessing.
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn process_data(items: &[String]) -> Vec<String> {
    items.iter()
        .filter(|s| validate(s))
        .map(|s| transform(s))
        .collect()
}

fn validate(s: &str) -> bool {
    s.len() > 10 && s.chars().all(|c| c.is_alphanumeric())
}

fn transform(s: &str) -> String {
    s.to_uppercase()
}

fn benchmark(c: &mut Criterion) {
    let data: Vec<String> = (0..1000)
        .map(|i| format!("item_number_{}", i))
        .collect();

    c.bench_function("process_data", |b| {
        b.iter(|| process_data(black_box(&data)))
    });

    // Profile individual components
    c.bench_function("validate", |b| {
        b.iter(|| {
            for item in &data {
                black_box(validate(black_box(item)));
            }
        })
    });

    c.bench_function("transform", |b| {
        b.iter(|| {
            for item in &data {
                black_box(transform(black_box(item)));
            }
        })
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
```

Run with `cargo bench`. Criterion shows which component is slow.

### Example: Memory Profiling with Valgrind

 Memory profiling with Valgrind's massif tool and heaptrack. These tools track heap allocations over time, identify allocation hotspots, and detect memory leaks. Heaptrack provides a GUI for exploring allocation patterns visually.

```bash
# Install valgrind
# Ubuntu: sudo apt install valgrind
# macOS: brew install valgrind

# Profile memory usage
valgrind --tool=massif ./target/release/myapp

# Visualize with ms_print
ms_print massif.out.* > massif.txt
```

Or use `heaptrack` for more detailed allocation tracking:

```bash
# Install heaptrack
# Ubuntu: sudo apt install heaptrack

# Profile
heaptrack ./target/release/myapp

# View results
heaptrack_gui heaptrack.myapp.*.gz
```

### Example: Profiling Allocations in Rust

 dhat for Rust-native allocation profiling. The custom global allocator tracks every allocation with stack traces. The JSON output can be viewed in Firefox's DHAT viewer, showing exactly where allocations occur and how much memory they consume.

```rust
// Add to Cargo.toml:
// [dependencies]
// dhat = "0.3"

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    // Your code here
    run_application();
}
```

Run with:
```bash
cargo run --features dhat-heap --release
```

This generates `dhat-heap.json`, viewable in Firefox's DHAT viewer.

### Example: Micro-Benchmarking Best Practices

 Micro-benchmarking with black_box to prevent dead code elimination. Comparing loop, iterator, and fold implementations reveals that the compiler often optimizes all three to identical machine code in release builds.

```rust
use criterion::{black_box, Criterion};

fn benchmark_alternatives(c: &mut Criterion) {
    let data: Vec<i32> = (0..10000).collect();

    c.bench_function("sum_loop", |b| {
        b.iter(|| {
            let mut sum = 0;
            for &x in black_box(&data) {
                sum += black_box(x);
            }
            black_box(sum)
        })
    });

    c.bench_function("sum_iter", |b| {
        b.iter(|| {
            black_box(&data).iter().sum::<i32>()
        })
    });

    c.bench_function("sum_fold", |b| {
        b.iter(|| {
            black_box(&data).iter().fold(0, |acc, &x| acc + x)
        })
    });
}
```

Use `black_box` to prevent the optimizer from eliminating code. Without it, the compiler might optimize away the entire computation.

## Pattern 2: Allocation Reduction

**Problem**: Heap allocations cost mutexes, cache misses, and copies; doing them inside hot loops or string builders dominates runtime and fragments memory.

**Solution**: Reuse buffers with `clear`, pre-size collections via `with_capacity`, lean on `SmallVec`, `Cow`, arenas, and `String::push_str` instead of repeated `format!`.

**Why It Matters**: Eliminating redundant allocations routinely yields multi-x speedups, keeps data cache-friendly, and avoids allocator contention.

**Use Cases**: Parsers, networking buffers, per-frame game loops, log formatting, small temporary collections, and any tight loop building strings or vectors.

### Example: Allocation vs Stack Allocation

Benchmark heap allocation versus stack allocation. Creating a Vec allocates heap memory with potential mutex contention, while arrays live on the stack. The difference is often 10-100x, making allocation reduction a high-impact optimization.

```rust
use std::time::Instant;

fn allocation_benchmark() {
    let iterations = 1_000_000;

    // Allocating
    let start = Instant::now();
    for _ in 0..iterations {
        let _v: Vec<i32> = vec![1, 2, 3, 4, 5];
    }
    println!("Allocating: {:?}", start.elapsed());

    // Stack only
    let start = Instant::now();
    for _ in 0..iterations {
        let _arr = [1, 2, 3, 4, 5];
    }
    println!("Stack: {:?}", start.elapsed());
}
```

Allocating is often 10-100x slower than stack allocation. Reducing allocations can dramatically improve performance.

### Example: Reusing Allocations

 Progressive optimization from per-iteration allocation to buffer reuse to pre-allocation. Using clear() retains capacity, with_capacity() prevents resizing, and iterators enable the compiler to optimize the entire pipeline efficiently.

```rust
fn process_bad(items: &[String]) -> Vec<String> {  // Bad: allocates per iteration
    let mut results = Vec::new();
    for item in items {
        let mut buffer = String::new();  // Allocates each time!
        buffer.push_str("processed_"); buffer.push_str(item);
        results.push(buffer);
    }
    results
}

fn process_good(items: &[String]) -> Vec<String> {  // Good: reuses buffer
    let mut results = Vec::new();
    let mut buffer = String::new();  // Allocate once
    for item in items {
        buffer.clear();  // Retain capacity
        buffer.push_str("processed_"); buffer.push_str(item);
        results.push(buffer.clone());
    }
    results
}

fn process_better(items: &[String]) -> Vec<String> {  // Better: pre-allocate
    let mut results = Vec::with_capacity(items.len());
    for item in items { results.push(format!("processed_{}", item)); }
    results
}

fn process_best(items: &[String]) -> Vec<String> {  // Best: iterators
    items.iter().map(|item| format!("processed_{}", item)).collect()
}
```

### Example: SmallVec: Stack-Allocated Small Collections

 SmallVec for stack-allocated small collections. When element count stays under the inline capacity (4 here), no heap allocation occurs. For larger sizes, it spills to heap transparently. Ideal when collections are usually small.

```rust
// Add to Cargo.toml:
// smallvec = "1.11"

use smallvec::SmallVec;

// Stores up to 4 elements on stack, spills to heap if larger
type SmallVec4<T> = SmallVec<[T; 4]>;

fn process_items(items: &[i32]) -> SmallVec4<i32> {
    let mut result = SmallVec4::new();
    for &item in items {
        if item % 2 == 0 {
            result.push(item);
        }
        if result.len() >= 4 {
            break;
        }
    }
    result
}

// If result has ≤4 elements, no heap allocation!
```

Use `SmallVec` when collections are usually small. The stack storage avoids allocation in the common case.

### Example: Cow: Clone-On-Write

 Cow (Clone-on-Write) for conditional allocation. When no modification is needed, it borrows the original data without allocation. When changes are required, it allocates an owned copy. This pattern avoids unnecessary cloning in read-heavy workloads.

```rust
use std::borrow::Cow;

fn process_string(input: &str) -> Cow<str> {
    if input.contains("bad") {
        // Must modify - allocates
        Cow::Owned(input.replace("bad", "good"))
    } else {
        // No modification needed - no allocation
        Cow::Borrowed(input)
    }
}

fn example() {
    let s1 = "good text";
    let s2 = "bad text";

    let r1 = process_string(s1);  // No allocation
    let r2 = process_string(s2);  // Allocates

    println!("{}, {}", r1, r2);
}
```

This pattern is common in APIs that sometimes need to modify data and sometimes don't.

### Example: Arena Allocation

 Typed-arena batch allocation. Arena allocation is a simple pointer bump, individual deallocation is a no-op, and all memory frees when the arena drops. Ideal for tree structures, parsers, and graph algorithms.

```rust
// Add to Cargo.toml:
// typed-arena = "2.0"

use typed_arena::Arena;

struct Node<'a> {
    value: i32,
    children: Vec<&'a Node<'a>>,
}

fn build_tree<'a>(arena: &'a Arena<Node<'a>>) -> &'a Node<'a> {
    let child1 = arena.alloc(Node {
        value: 1,
        children: vec![],
    });

    let child2 = arena.alloc(Node {
        value: 2,
        children: vec![],
    });

    arena.alloc(Node {
        value: 0,
        children: vec![child1, child2],
    })
}

fn example() {
    let arena = Arena::new();
    let tree = build_tree(&arena);
    // All nodes deallocated together when arena drops
}
```

Arenas are fast because:
1. Allocation is a simple pointer bump
2. Individual deallocation is free (no-op)
3. Bulk deallocation is fast (drop the arena)

### Example: String Interning

Implement string interning to deduplicate strings. Identical strings map to the same ID, eliminating duplicate allocations. Compilers use this for identifiers, and databases for repeated column values. Memory savings and comparison speed both improve.

```rust
use std::collections::HashMap;

struct StringInterner {
    strings: HashMap<String, usize>,
    reverse: Vec<String>,
}

impl StringInterner {
    fn new() -> Self { StringInterner { strings: HashMap::new(), reverse: Vec::new() } }

    fn intern(&mut self, s: &str) -> usize {
        if let Some(&id) = self.strings.get(s) { return id; }
        let id = self.reverse.len();
        self.reverse.push(s.to_string());
        self.strings.insert(s.to_string(), id);
        id
    }

    fn get(&self, id: usize) -> &str { &self.reverse[id] }
}

fn example() {
    let mut interner = StringInterner::new();
    let id1 = interner.intern("hello");
    let id2 = interner.intern("hello");
    assert_eq!(id1, id2);  // Same string = same ID, no duplicate allocation
}
```

Use interning when you have many duplicate strings (like identifiers in a compiler).

## Pattern 3: Cache-Friendly Data Structures

**Problem**: Pointer-chasing data structures thrash caches—RAM misses are ~100× slower than L1 hits and false sharing stalls multi-threaded code.

**Solution**: Favor contiguous storage (`Vec`, SoA layouts, arenas), group hot fields together, pad or align to avoid false sharing, and prefetch predictable strides.

**Why It Matters**: Cache-friendly layouts let hardware prefetch and keep hot data in L1, yielding 2–10× faster loops with less coherency traffic.

**Use Cases**: ECS/game data, parsers/ASTs, graph and numerical kernels, big data scans, multi-threaded counters, and storage engines choosing row vs column layouts.

### Example: Array-of-Structs vs Struct-of-Arrays

This example contrasts AoS versus SoA memory layouts. AoS loads entire particles even when accessing only positions, wasting cache bandwidth. SoA stores positions contiguously, enabling efficient prefetching and SIMD operations. Often 2-3x faster for bulk operations.

```rust
struct ParticleAoS { x: f32, y: f32, z: f32, vx: f32, vy: f32, vz: f32 }  // AoS

fn update_positions_aos(particles: &mut [ParticleAoS], dt: f32) {
    for p in particles {  // Loads 24B per particle, wastes cache bandwidth
        p.x += p.vx * dt; p.y += p.vy * dt; p.z += p.vz * dt;
    }
}

struct ParticlesSoA { x: Vec<f32>, y: Vec<f32>, z: Vec<f32>, vx: Vec<f32>, vy: Vec<f32>, vz: Vec<f32> }  // SoA

fn update_positions_soa(particles: &mut ParticlesSoA, dt: f32) {
    for i in 0..particles.x.len() {  // Contiguous access, cache-friendly
        particles.x[i] += particles.vx[i] * dt;
        particles.y[i] += particles.vy[i] * dt;
        particles.z[i] += particles.vz[i] * dt;
    }
}
```

SoA can be 2-3x faster for this access pattern because it uses cache lines efficiently.

### Example: Cache Line Awareness

 False sharing prevention with cache line alignment. When two threads write to the same 64-byte cache line, each write invalidates the other core's cache. Padding counters to separate cache lines eliminates this contention.

```rust
#[repr(C, align(64))]
struct CacheLineAligned {
    value: i64,
    padding: [u8; 56],
}

// Bad: False sharing
struct CounterBad {
    thread1_counter: i64,  // Same cache line
    thread2_counter: i64,  // Same cache line
}

// Good: No false sharing
#[repr(C, align(64))]
struct CounterGood {
    thread1_counter: i64,
    _padding: [u8; 56],
}

#[repr(C, align(64))]
struct CounterGood2 {
    thread2_counter: i64,
    _padding: [u8; 56],
}
```

False sharing occurs when two threads write to different variables in the same cache line, causing cache invalidation and performance degradation.

### Example: Prefetching and Sequential Access

Benchmarking sequential versus random memory access. Sequential access enables hardware prefetching, keeping data in cache. Random access causes cache misses on nearly every read. The difference is often 5-10x, dominating algorithm performance.

```rust
use std::time::Instant;

fn sequential_access(data: &[i32]) -> i64 {
    let mut sum = 0i64;
    for &x in data {
        sum += x as i64;
    }
    sum
}

fn random_access(data: &[i32], indices: &[usize]) -> i64 {
    let mut sum = 0i64;
    for &idx in indices {
        sum += data[idx] as i64;
    }
    sum
}

fn benchmark() {
    let data: Vec<i32> = (0..1_000_000).collect();
    let mut indices: Vec<usize> = (0..data.len()).collect();

    // Shuffle for random access
    use rand::seq::SliceRandom;
    indices.shuffle(&mut rand::thread_rng());

    let start = Instant::now();
    let sum1 = sequential_access(&data);
    println!("Sequential: {:?}", start.elapsed());

    let start = Instant::now();
    let sum2 = random_access(&data, &indices);
    println!("Random: {:?}", start.elapsed());

    // Random access is often 5-10x slower!
}
```

Design data structures for sequential access when possible.

### Example: Linked Lists vs Vectors

Illustrating why vectors outperform linked lists. Vector iteration loads contiguous cache lines, while linked list traversal causes a cache miss per node. Use vectors unless you need O(1) mid-insertion, which is rare in practice.

```rust
// Bad: Linked list
struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
}

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

// Good: Vector
struct VecList<T> {
    items: Vec<T>,
}

// Iterating a vector loads contiguous cache lines
// Iterating a linked list causes a cache miss per node
```

Use vectors unless you need O(1) insertion in the middle (rare).

### Example: HashMap Optimization

 HashMap optimizations: FxHashMap uses a faster hash function for integer keys (30% speedup but less DoS-resistant), and with_capacity pre-allocates to avoid resizing. FromIterator combines allocation and insertion efficiently.

```rust
use std::collections::HashMap;
use rustc_hash::FxHashMap;  // Faster hash for integer keys

fn compare_hashmaps() {
    let mut std_map = HashMap::new();
    let mut fx_map = FxHashMap::default();

    // FxHashMap is faster for integer keys
    for i in 0..10000 {
        std_map.insert(i, i * 2);
        fx_map.insert(i, i * 2);
    }

    // FxHashMap: ~30% faster for integer keys
    // But less secure (predictable hashes)
}

fn presized_hashmap() {
    let items = vec![(1, "a"), (2, "b"), (3, "c")];

    let mut map1 = HashMap::new();  // Bad: grows multiple times
    for (k, v) in &items { map1.insert(*k, *v); }

    let mut map2 = HashMap::with_capacity(items.len());  // Good: one allocation
    for (k, v) in &items { map2.insert(*k, *v); }

    let map3: HashMap<_, _> = items.iter().copied().collect();  // Best: FromIterator
}
```

## Pattern 4: Zero-Cost Abstractions

**Problem**: Abstractions look expensive—iterator chains, generics, and newtypes seem slower than hand-written loops or raw types.

**Solution**: Trust the optimizer: iterators inline to the same machine code, generics monomorphize per type, `#[inline]` removes tiny call overhead, and newtypes share representation with their inner type.

**Why It Matters**: Zero-cost abstractions let you write clear, reusable APIs without leaving performance on the table; release builds routinely match or beat manual code.

**Use Cases**: Iterator-heavy pipelines, generic data structures, type-safe ID wrappers, compile-time computation (const fn/generics), and layered DSL-style APIs.

### Example: Understanding Branch Misprediction

Compare branching versus branchless code with random data. When branch outcomes are unpredictable, the CPU's branch predictor fails frequently, causing 10-20 cycle penalties. Branchless arithmetic using conditionals as integers avoids this penalty.

```rust
use std::time::Instant;

fn with_unpredictable_branch(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data { if x % 2 == 0 { sum += x; } }  // Unpredictable branches
    sum
}

fn without_branch(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data { sum += x * (x % 2 == 0) as i32; }  // Branchless arithmetic
    sum
}

fn benchmark() {
    // Random data - unpredictable branches
    let random_data: Vec<i32> = (0..1_000_000)
        .map(|_| rand::random::<i32>())
        .collect();

    let start = Instant::now();
    let sum1 = with_unpredictable_branch(&random_data);
    println!("With branch: {:?}", start.elapsed());

    let start = Instant::now();
    let sum2 = without_branch(&random_data);
    println!("Branchless: {:?}", start.elapsed());

    // Branchless can be 2x faster with random data
}
```

### Example: Sorting for Branch Prediction

 How does sorting improve branch prediction. After sorting, conditional checks become predictable: all negatives come first, then positives. The O(n log n) sort cost is amortized when iterating multiple times over the same data.

```rust
fn sum_if_positive_unsorted(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data {
        if x > 0 {  // Unpredictable branches
            sum += x;
        }
    }
    sum
}

fn sum_if_positive_sorted(data: &mut [i32]) -> i32 {
    data.sort_unstable();  // Cost: O(n log n)

    let mut sum = 0;
    for &x in data {
        if x > 0 {  // Predictable after sorting
            sum += x;
        }
    }
    sum
}

// If you iterate many times, sorting once can be faster
```

### Example: Branch-Free Code with Bitwise Operations

Implement branchless max using bit manipulation. The sign bit extracts whether the difference is negative, then masks accordingly. However, LLVM often converts simple if-else to branchless select instructions automatically in release builds.

```rust
// With branch
fn max_with_branch(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

// Branchless
fn max_branchless(a: i32, b: i32) -> i32 {
    let diff = a - b;
    let sign = diff >> 31;  // -1 if negative, 0 if positive
    a - (diff & sign)
}

// Or use LLVM's select (compiler does this optimization)
fn max_select(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }  // LLVM converts to select instruction
}
```

The compiler often optimizes simple `if` expressions to branchless code automatically.

### Example: Likely and Unlikely Hints

 Branch prediction hints. The nightly likely/unlikely intrinsics tell the compiler which branch is common. The stable alternative uses #[cold] on error handlers, signaling that the error path is rare and optimizing the happy path.

```rust
// Unstable feature - requires nightly
#![feature(core_intrinsics)]

use std::intrinsics::{likely, unlikely};

fn process_with_hints(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data {
        unsafe {
            if likely(x > 0) {  // Hint: usually true
                sum += x;
            }
        }
    }
    sum
}

// Stable alternative using cold
#[inline(never)]
fn handle_error() {
    eprintln!("Error occurred!");
}

fn process_stable(x: i32) {
    if x < 0 {
        handle_error();  // Compiler knows this is rare
    }
}
```

The `#[cold]` attribute tells the compiler this code is rarely executed, improving branch prediction for the common path.

### Example: Pattern Matching Optimization

 Match arm ordering for performance. Placing the most common variant first improves branch prediction. The compiler may reorder arms, but explicit ordering documents intent and helps when the compiler lacks profile-guided optimization data.

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(u8, u8, u8),
}

fn process_message_bad(msg: &Message) -> String {  // Random order
    match msg {
        Message::Quit => "quit".to_string(),
        Message::Move { x, y } => format!("move {} {}", x, y),
        Message::Write(s) => s.clone(),
        Message::ChangeColor(r, g, b) => format!("color {} {} {}", r, g, b),
    }
}

fn process_message_good(msg: &Message) -> String {  // Common case first
    match msg {
        Message::Write(s) => s.clone(),  // Most common first
        Message::Move { x, y } => format!("move {} {}", x, y),
        Message::ChangeColor(r, g, b) => format!("color {} {} {}", r, g, b),
        Message::Quit => "quit".to_string(),
    }
}
```

Put common cases first in match statements to improve branch prediction.

## Pattern 5: Compiler Optimizations

**Problem**: Leaving builds at debug defaults or generic CPU targets forfeits huge speedups; many teams never flip LTO, PGO, or codegen knobs because the impact seems opaque.

**Solution**: Ship release builds with `opt-level=3`, enable LTO (and PGO when feasible), target the actual CPU (`target-cpu=native`), reduce `codegen-units` for deeper optimization, and tune for size with `opt-level="z"`/`panic="abort"` when needed.

**Why It Matters**: The right flags routinely deliver 10–30× faster binaries or much smaller artifacts by letting LLVM inline across crates, specialize for hot paths, and emit SIMD instructions your hardware already supports.

**Use Cases**: Production binaries, benchmarking harnesses, SIMD-heavy workloads, embedded/WASM targets chasing size, and CI pipelines that produce optimized artifacts via PGO/LTO combinations.


### Example: Const Functions

 Demo of const fn for compile-time computation. The factorial function executes at compile time when called in a const context, embedding the result directly in the binary. Runtime cost is zero since computation happens during compilation.

```rust
const fn factorial(n: u32) -> u32 {
    match n {
        0 => 1,
        _ => n * factorial(n - 1),
    }
}

const FACTORIAL_10: u32 = factorial(10);  // Computed at compile time!

fn example() {
    println!("{}", FACTORIAL_10);  // Just loads the constant
}
```

### Example: Const Generics for Compile-Time Values

Const generics for compile-time matrix dimensions. The size is part of the type, enabling stack allocation and loop unrolling. The identity matrix is computed at compile time and embedded directly in the binary.

```rust
struct Matrix<const N: usize> {
    data: [[f64; N]; N],
}

impl<const N: usize> Matrix<N> {
    const fn zeros() -> Self {
        Matrix {
            data: [[0.0; N]; N],
        }
    }

    const fn identity() -> Self {
        let mut data = [[0.0; N]; N];
        let mut i = 0;
        while i < N {
            data[i][i] = 1.0;
            i += 1;
        }
        Matrix { data }
    }
}

const IDENTITY_4X4: Matrix<4> = Matrix::identity();

fn example() {
    // Matrix size known at compile time
    // Enables better optimization
    let m = Matrix::<3>::zeros();
}
```

### Example: Lookup Tables

Pre-compute lookup tables at compile time using const fn. Instead of calculating trigonometric functions at runtime, values are embedded in the binary. Lookup is O(1) versus expensive floating-point operations, trading binary size for speed.

```rust
const fn generate_sin_table() -> [f64; 360] {
    let mut table = [0.0; 360];
    let mut i = 0;
    while i < 360 {
        // Simplified - actual sin computation would use series expansion
        table[i] = 0.0;  // Placeholder
        i += 1;
    }
    table
}

const SIN_TABLE: [f64; 360] = generate_sin_table();

fn fast_sin(degrees: usize) -> f64 {
    SIN_TABLE[degrees % 360]  // O(1) lookup vs expensive calculation
}
```

### Example: Static Assertions

`const` assertions to enforce invariants at compile time. The ring buffer requires power-of-two size for efficient modulo operations. Invalid sizes cause compilation failures, catching bugs before runtime without any performance cost.

```rust
const fn is_power_of_two(n: usize) -> bool {
    n != 0 && (n & (n - 1)) == 0
}

struct RingBuffer<T, const N: usize> {
    data: [Option<T>; N],
    head: usize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    const VALID_SIZE: () = assert!(is_power_of_two(N), "Size must be power of two");

    fn new() -> Self {
        let _ = Self::VALID_SIZE;  // Force compile-time check
        RingBuffer {
            data: std::array::from_fn(|_| None),
            head: 0,
        }
    }
}

// This won't compile:
// let buffer = RingBuffer::<i32, 7>::new();

// This compiles:
let buffer = RingBuffer::<i32, 8>::new();
```

### Example: Build-Time Code Generation

 `build.rs` to generate code at build time. Complex computations like prime number generation run once during compilation, producing a static lookup table. The generated code is included via include! and compiled into the final binary.

```rust
// build.rs
fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("generated.rs");

    // Generate lookup table
    let mut code = String::from("const PRIMES: &[u32] = &[\n");
    for i in 2..10000 {
        if is_prime(i) {
            code.push_str(&format!("    {},\n", i));
        }
    }
    code.push_str("];\n");

    std::fs::write(dest_path, code).unwrap();
}

fn is_prime(n: u32) -> bool {
    if n < 2 { return false; }
    for i in 2..=(n as f64).sqrt() as u32 {
        if n % i == 0 { return false; }
    }
    true
}
```

```rust
// examples/lib.rs
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

fn is_prime_fast(n: u32) -> bool {
    PRIMES.binary_search(&n).is_ok()
}
```

### Example: Compile-Time String Processing

 `const fn` for compile-time string operations. String length is computed during compilation, enabling fixed-size buffer allocation without runtime overhead. Useful for embedded systems and performance-critical code requiring static buffer sizes.

```rust
const fn const_strlen(s: &str) -> usize {
    s.len()
}

const fn const_concat_len(s1: &str, s2: &str) -> usize {
    s1.len() + s2.len()
}

const HELLO_LEN: usize = const_strlen("Hello, World!");

fn example() {
    // Length computed at compile time
    let mut buffer = [0u8; HELLO_LEN];
}
```

### Example: Type-Level Computation

Encode natural numbers in the type system using Peano arithmetic. Succ and Zero types represent numbers at compile time, with NatNum trait computing values. Useful for type-safe dimensional analysis and compile-time verified arithmetic.

```rust
use std::marker::PhantomData;

struct Succ<N>(PhantomData<N>);
struct Zero;

type One = Succ<Zero>;
type Two = Succ<One>;
type Three = Succ<Two>;

trait NatNum {
    const VALUE: usize;
}

impl NatNum for Zero {
    const VALUE: usize = 0;
}

impl<N: NatNum> NatNum for Succ<N> {
    const VALUE: usize = N::VALUE + 1;
}

fn example() {
    assert_eq!(Three::VALUE, 3);  // Computed at compile time
}
```


## Advanced Optimization Techniques



### Example: SIMD (Single Instruction Multiple Data)

 SIMD intrinsics for parallel data processing. AVX2 instructions load, add, and store 8 floats simultaneously. SIMD achieves 4-8x speedup for data-parallel operations. The remainder loop handles non-aligned array lengths.

```rust
// Requires nightly and target features
// Add to .cargo/config.toml:
// [build]
// rustflags = ["-C", "target-cpu=native"]

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[inline]
fn add_arrays_scalar(a: &[f32], b: &[f32], result: &mut [f32]) {
    for i in 0..a.len() {
        result[i] = a[i] + b[i];
    }
}

#[cfg(target_arch = "x86_64")]
#[inline]
unsafe fn add_arrays_simd(a: &[f32], b: &[f32], result: &mut [f32]) {
    let chunks = a.len() / 8;

    for i in 0..chunks {
        let offset = i * 8;

        // Load 8 floats at once
        let a_vec = _mm256_loadu_ps(a.as_ptr().add(offset));
        let b_vec = _mm256_loadu_ps(b.as_ptr().add(offset));

        // Add 8 floats in one instruction
        let result_vec = _mm256_add_ps(a_vec, b_vec);

        // Store 8 floats at once
        _mm256_storeu_ps(result.as_mut_ptr().add(offset), result_vec);
    }

    // Handle remainder
    for i in (chunks * 8)..a.len() {
        result[i] = a[i] + b[i];
    }
}

// SIMD can be 4-8x faster for this operation
```

### Example: Inline Assembly

 Inline assembly for direct CPU instruction access. The asm! macro executes cpuid to query processor features. Inline assembly is rarely needed since LLVM generates excellent code, but enables access to special instructions unavailable through intrinsics.

```rust

unsafe fn cpuid(eax: u32) -> (u32, u32, u32, u32) {
    let mut ebx: u32;
    let mut ecx: u32;
    let mut edx: u32;

    std::arch::asm!(
        "cpuid",
        inout("eax") eax,
        out("ebx") ebx,
        out("ecx") ecx,
        out("edx") edx,
    );

    (eax, ebx, ecx, edx)
}
```

### Example: Link-Time Optimization

Enable LTO for cross-crate optimization. Full LTO with single codegen unit allows LLVM to inline across crate boundaries and eliminate dead code globally. Expect 10-20% speedup at the cost of significantly longer compile times.

```toml
[profile.release]
lto = "fat"        # Full LTO
codegen-units = 1  # Single codegen unit for better optimization
```

LTO enables cross-crate inlining and optimization, often yielding 10-20% speedup at the cost of longer compile times.

### Summary

This chapter covered performance optimization patterns for maximizing Rust code performance:

1. **Profiling Strategies**: CPU profiling (perf, flamegraph), memory profiling (dhat, valgrind), Criterion benchmarks
2. **Allocation Reduction**: Reuse buffers, SmallVec, Cow, pre-allocation, arena allocation
3. **Cache-Friendly Data Structures**: Contiguous memory, struct-of-arrays, arena allocation, inline data
4. **Zero-Cost Abstractions**: Iterators = loops, generics monomorphize, inline functions, newtype pattern
5. **Compiler Optimizations**: Release mode, LTO, PGO, target-cpu=native, codegen-units=1

**Key Takeaways**:
- Measure first: intuition about bottlenecks usually wrong, profiling reveals truth
- Allocation reduction: 2-10x speedup by reusing buffers, pre-allocating, using SmallVec
- Cache-friendly: cache miss = 100x slower than hit, contiguous memory = prefetching
- Zero-cost abstractions: iterators as fast as loops, generics free, newtypes free
- Compiler optimization: release 10-100x faster than debug, LTO + PGO + target-cpu = maximum speed

**Optimization Workflow**:
1. Profile to find hotspots (perf, flamegraph, Criterion)
2. Understand why slow (allocations? cache misses? branches?)
3. Optimize (reduce allocations, improve locality, eliminate branches)
4. Verify with benchmarks (did it actually help?)
5. Repeat for next hotspot


**Profiling Commands**:
```bash
# Flamegraph (all platforms)
cargo install flamegraph
cargo flamegraph

# Criterion benchmarks
cargo bench

# Memory profiling with dhat
cargo run --features dhat-heap --release

# Release mode with debug symbols
[profile.release]
debug = true
```

**Performance Guidelines**:
- Allocation: 10-100x slower than stack, mutex contention in multi-threaded
- Cache: L1 = 1 cycle, L2 = 10 cycles, L3 = 40 cycles, RAM = 200 cycles
- Branch misprediction: 10-20 cycles penalty
- Function call: inlined = free, not inlined = ~5 cycles
- SIMD: 4-8x speedup for data-parallel operations

**Anti-Patterns**:
- Premature optimization (measure first!)
- Optimizing cold code (focus on hotspots)
- Sacrificing readability for negligible gains (5% not worth complexity)
- Ignoring allocations (often biggest win)
- Not benchmarking changes (did it actually help?)
- Using debug mode for benchmarks (10-100x slower)
- Assuming cache doesn't matter (it's the bottleneck)

**Optimization Priority** (by typical impact):
1. Algorithmic complexity (O(N²) → O(N log N))
2. Reduce allocations (reuse buffers, SmallVec)
3. Cache-friendly data layout (contiguous, SoA)
4. Compiler flags (release, LTO, target-cpu)
5. Branch prediction (predictable branches)
6. Micro-optimizations (last resort, measure first)
