# Performance Optimization

[Profiling Strategies](#pattern-1-profiling-strategies)

- Problem: Intuition about bottlenecks wrong; optimize wrong code; no data on hotspots; time wasted on non-bottlenecks
- Solution: CPU profiling (perf, flamegraph, Instruments); memory profiling (valgrind, heaptrack, dhat); Criterion benchmarks
- Why It Matters: Profiling reveals actual bottlenecks; intuition often wrong; 80/20 rule (80% time in 20% code); measure first saves effort
- Use Cases: Finding hotspots, allocation tracking, comparing implementations, regression detection, production profiling

[Allocation Reduction](#pattern-2-allocation-reduction)

- Problem: Allocations expensive (10-100x slower than stack); repeated allocations waste CPU; large allocations fragment memory
- Solution: Reuse buffers (clear() not new); SmallVec for small collections; Cow for conditional cloning; pre-allocate with_capacity
- Why It Matters: Allocation = mutex contention + heap access; reducing allocations often 2-10x speedup; cache-friendly
- Use Cases: Hot loops, repeated string building, temporary buffers, small collections, parser state, networking buffers

[Cache-Friendly Data Structures](#pattern-3-cache-friendly-data-structures)

- Problem: Cache misses 100x slower than hits; pointer chasing kills performance; scattered allocations waste cache lines
- Solution: Contiguous memory (Vec not linked list); struct-of-arrays for iteration; arena allocation; inline small data
- Why It Matters: Modern CPUs cache-bound not CPU-bound; cache miss = 200+ cycles; contiguous access = prefetch; 10x speedup possible
- Use Cases: Game engines, parsers, numerical computing, graph algorithms, large data processing, tight loops

[Zero-Cost Abstractions](#pattern-4-zero-cost-abstractions)

- Problem: Abstractions seem to cost performance; iterator overhead unclear; generics bloat binary; inline hints unclear
- Solution: Iterators compile to loops; generics monomorphize; #[inline] for small functions; const for compile-time; release optimizations
- Why It Matters: Abstractions free when used right; iterators as fast as loops; generics zero runtime cost; compiler optimizes aggressively
- Use Cases: Iterator chains, generic algorithms, small wrapper functions, compile-time computation, abstraction layers

[Compiler Optimizations](#pattern-5-compiler-optimizations)

- Problem: Compiler optimization levels unclear; PGO/LTO benefits unknown; target-cpu unused; codegen-units affect speed
- Solution: Release profile (opt-level=3); LTO for cross-crate inline; PGO for branch prediction; target-cpu=native; codegen-units=1
- Why It Matters: Release mode 10-100x faster than debug; LTO enables cross-crate optimization; PGO 10-30% speedup; target-cpu uses SIMD
- Use Cases: Production builds, benchmarking, CPU-intensive code, binary size reduction, maximum performance



[Performance Optimization Cheat Sheet](#performance-optimization-cheat-sheet)
- This comprehensive guide covers memory optimization, CPU optimization, cache optimization, I/O optimization, and profiling techniques for Rust performance optimization!


## Overview
This chapter explores performance optimization: profiling to find bottlenecks, allocation reduction techniques, cache-friendly data structures, zero-cost abstractions, and compiler optimizations for maximum performance.

## Pattern 1: Profiling Strategies

**Problem**: Intuition about performance bottlenecks is usually wrong—developers optimize the wrong code. No concrete data about where time/memory spent. Optimizing without measurement wastes effort on non-critical paths. Hotspots in surprising places (80/20 rule: 80% time in 20% code). Can't compare optimization attempts objectively. Production performance issues hard to diagnose. Allocation patterns invisible without tooling.

**Solution**: CPU profiling with perf (Linux), Instruments (macOS), or cargo-flamegraph (cross-platform). Flamegraphs visualize time spent (wide bars = hotspots). Memory profiling with valgrind/massif, heaptrack, or dhat for Rust-specific allocation tracking. Criterion benchmarks for micro-benchmarking with statistical analysis. Profile in release mode with debug symbols (debug = true in profile). Use black_box to prevent compiler from optimizing away benchmarks. Identify hotspots, then optimize, then re-profile to verify.

**Why It Matters**: Profiling reveals actual bottlenecks—often not where expected. Measure first principle: saves time by focusing optimization effort correctly. 80/20 rule applies: optimizing 20% of code improves 80% of runtime. Flamegraphs show call stacks: understand why function slow (who called it). Memory profiling reveals allocation hotspots: reducing allocations often yields 2-10x speedup. Criterion provides statistical confidence: know if optimization actually helped. Production profiling (perf) finds real-world bottlenecks missed in development.

**Use Cases**: Finding performance hotspots (which function slow?), allocation tracking (where are we allocating?), comparing algorithm implementations (A vs B which faster?), regression detection (did recent change slow things down?), production profiling (diagnose live performance issues), optimization validation (did optimization help?), understanding scaling behavior (how does performance change with input size?), identifying cache misses.


## Example: Which bottleneck
```rust
//=========================
// Which is the bottleneck?
//=========================
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

On Linux, `perf` is the gold standard for CPU profiling:

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

For easier flamegraph generation:

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

On macOS, use Instruments:

```bash
# Build with debug symbols
cargo build --release

# Open in Instruments
instruments -t "Time Profiler" ./target/release/myapp
```

Instruments provides a GUI for exploring hotspots, viewing call trees, and drilling into specific functions.

### Example: Profiling in Code with Benchmarks

Criterion benchmarks provide detailed performance data:

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

Find memory allocations and leaks:

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

Use `dhat` for Rust-specific allocation profiling:

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

When benchmarking specific code:

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

**Problem**: Allocations expensive—10-100x slower than stack allocation. Each allocation involves allocator mutex (contention in multi-threaded), heap access (cache miss likely), bookkeeping overhead, later deallocation. Repeated allocations in hot loops dominate runtime. String building allocates per concatenation. Temporary buffers allocated/freed repeatedly. Large allocations cause fragmentation. Vec reallocations when capacity exceeded (copy all elements). Short-lived allocations thrash allocator.

**Solution**: Reuse buffers: use clear() not new(), keep buffer across iterations. Pre-allocate with Vec::with_capacity(n) when size known. SmallVec for small collections (stack-allocated until threshold, then heap). Cow for conditional cloning (borrow when unchanged, allocate only when modified). Arena allocation for related objects (bump allocator, free all at once). String building: use String::with_capacity, push_str instead of format! in loops. Avoid collect() when unnecessary (iterator lazy evaluation).

**Why It Matters**: Allocation = slow: mutex contention in allocator, 100+ cycles, cache miss likely. Reducing allocations often yields 2-10x speedup for allocation-heavy code. Stack allocation nearly free: just adjust stack pointer. Reusing buffers eliminates churn: allocator sees fewer requests. Pre-allocation prevents reallocations: Vec won't copy when growing. SmallVec perfect for small collections: avoid heap entirely. Memory fragmentation reduced: fewer allocations = less fragmentation. Cache-friendly: less heap means more stack/cache hits.

**Use Cases**: Hot loops (process millions of items without allocating each), repeated string building (building JSON/HTML/SQL in loop), temporary buffers (parser state, networking buffers), small collections (function returning small Vec), parser state (reuse token buffer across tokens), networking (reuse read/write buffers), game loops (per-frame allocations eliminated), log formatting (buffer pool for log messages).

### Example: Allocation vs Stack Allocation

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

Instead of allocating repeatedly, reuse buffers:

```rust
//===============================
// Bad: Allocates every iteration
//===============================
fn process_bad(items: &[String]) -> Vec<String> {
    let mut results = Vec::new();
    for item in items {
        let mut buffer = String::new();  // Allocates!
        buffer.push_str("processed_");
        buffer.push_str(item);
        results.push(buffer);
    }
    results
}

//====================
// Good: Reuses buffer
//====================
fn process_good(items: &[String]) -> Vec<String> {
    let mut results = Vec::new();
    let mut buffer = String::new();  // Allocate once
    for item in items {
        buffer.clear();  // Reuse allocation
        buffer.push_str("processed_");
        buffer.push_str(item);
        results.push(buffer.clone());  // Still allocates, but see next example
    }
    results
}

//================================================
// Better: Pre-allocate and avoid unnecessary work
//================================================
fn process_better(items: &[String]) -> Vec<String> {
    let mut results = Vec::with_capacity(items.len());  // Pre-allocate
    for item in items {
        let processed = format!("processed_{}", item);  // One allocation
        results.push(processed);
    }
    results
}

//==================================
// Best for this case: Use iterators
//==================================
fn process_best(items: &[String]) -> Vec<String> {
    items
        .iter()
        .map(|item| format!("processed_{}", item))
        .collect()
}
```

### Example: SmallVec: Stack-Allocated Small Collections

`SmallVec` stores small collections on the stack:

```rust
// Add to Cargo.toml:
// smallvec = "1.11"

use smallvec::SmallVec;

//===========================================================
// Stores up to 4 elements on stack, spills to heap if larger
//===========================================================
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

//=================================================
// If result has ≤4 elements, no heap allocation!
//=================================================
```

Use `SmallVec` when collections are usually small. The stack storage avoids allocation in the common case.

### Example: Cow: Clone-On-Write

`Cow` defers allocation until mutation:

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

Arena as batch allocations for better performance:

```rust
//===================
// Add to Cargo.toml:
//===================
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

    // All nodes allocated from arena
    // Deallocated together when arena drops
}
```

Arenas are fast because:
1. Allocation is a simple pointer bump
2. Individual deallocation is free (no-op)
3. Bulk deallocation is fast (drop the arena)

### Example: String Interning

Deduplicate strings to save memory:

```rust
use std::collections::HashMap;

struct StringInterner {
    strings: HashMap<String, usize>,
    reverse: Vec<String>,
}

impl StringInterner {
    fn new() -> Self {
        StringInterner {
            strings: HashMap::new(),
            reverse: Vec::new(),
        }
    }

    fn intern(&mut self, s: &str) -> usize {
        if let Some(&id) = self.strings.get(s) {
            id
        } else {
            let id = self.reverse.len();
            self.reverse.push(s.to_string());
            self.strings.insert(s.to_string(), id);
            id
        }
    }

    fn get(&self, id: usize) -> &str {
        &self.reverse[id]
    }
}

fn example() {
    let mut interner = StringInterner::new();

    // Same strings map to same ID
    let id1 = interner.intern("hello");
    let id2 = interner.intern("hello");

    assert_eq!(id1, id2);  // No duplicate allocation

    println!("{}", interner.get(id1));
}
```

Use interning when you have many duplicate strings (like identifiers in a compiler).

## Pattern 3: Cache-Friendly Data Structures

**Problem**: Cache misses 100x+ slower than cache hits (RAM access ~200 cycles, L1 cache ~1 cycle). Pointer chasing kills performance—linked lists traverse pointers (each node separate allocation, random memory locations). Scattered allocations waste cache lines (64 bytes fetched, use 8). Array-of-structs loads unused fields when iterating. Hot data mixed with cold data. False sharing in multi-threaded (two threads accessing same cache line). Structure padding wastes cache space.

**Solution**: Contiguous memory: Vec not linked list, array not tree when possible. Struct-of-arrays (SoA) for bulk iteration—separate Vec per field, access only needed fields. Arena allocation: allocate related objects together (bump allocator), locality of reference. Inline small data in struct (avoid pointer to small allocation). Pack hot fields together, cold fields separate. Align structs to cache lines (64 bytes). Use #[repr(C)] for layout control. Prefetch hints for predictable access patterns.

**Why It Matters**: Modern CPUs cache-bound not CPU-bound—memory bandwidth is bottleneck. Cache miss = 200+ cycles wasted, cache hit = 1 cycle. Contiguous access = hardware prefetching: CPU fetches next cache line speculatively. SoA can be 2-10x faster than AoS for bulk operations. Arena allocation improves locality: related objects nearby in memory. Fewer cache misses = more CPU cycles doing useful work. False sharing eliminated: threads don't contend for cache lines.

**Use Cases**: Game engines (entities, particles, physics—thousands of objects), parsers (tokens, AST nodes—sequential access), numerical computing (matrices, vectors—SIMD-friendly), graph algorithms (adjacency lists—BFS/DFS traversal), large data processing (millions of records), tight loops (inner loop dominates runtime), multi-threaded workloads (avoid false sharing), database engines (row vs column storage).

### Example: Array-of-Structs vs Struct-of-Arrays

```rust
//=======================================
// Array of Structs (AoS) - bad for cache
//=======================================
struct ParticleAoS {
    x: f32,
    y: f32,
    z: f32,
    vx: f32,
    vy: f32,
    vz: f32,
}

fn update_positions_aos(particles: &mut [ParticleAoS], dt: f32) {
    for p in particles {
        // Loads entire particle (24 bytes)
        // But we only need x, y, z (12 bytes)
        // Wastes cache bandwidth
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        p.z += p.vz * dt;
    }
}

//========================================
// Struct of Arrays (SoA) - cache-friendly
//========================================
struct ParticlesSoA {
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
    vx: Vec<f32>,
    vy: Vec<f32>,
    vz: Vec<f32>,
}

fn update_positions_soa(particles: &mut ParticlesSoA, dt: f32) {
    for i in 0..particles.x.len() {
        // Loads contiguous x, y, z values
        // Much better cache usage
        particles.x[i] += particles.vx[i] * dt;
        particles.y[i] += particles.vy[i] * dt;
        particles.z[i] += particles.vz[i] * dt;
    }
}
```

SoA can be 2-3x faster for this access pattern because it uses cache lines efficiently.

### Example: Cache Line Awareness

Cache lines are typically 64 bytes. Accessing any byte in a cache line loads the entire line:

```rust
#[repr(C, align(64))]
struct CacheLineAligned {
    value: i64,
    padding: [u8; 56],
}

//===================
// Bad: False sharing
//===================
struct CounterBad {
    thread1_counter: i64,  // Same cache line
    thread2_counter: i64,  // Same cache line
}

//=======================
// Good: No false sharing
//=======================
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

Sequential access is dramatically faster than random access:

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

Linked lists are almost always slower than vectors due to poor cache behavior:

```rust
//=================
// Bad: Linked list
//=================
struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
}

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

//=============
// Good: Vector
//=============
struct VecList<T> {
    items: Vec<T>,
}

//================================================
// Iterating a vector loads contiguous cache lines
//================================================
// Iterating a linked list causes a cache miss per node
```

Use vectors unless you need O(1) insertion in the middle (rare).

### HashMap Optimization

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

//==================
// Pre-size HashMaps
//==================
fn presized_hashmap() {
    let items = vec![(1, "a"), (2, "b"), (3, "c")];

    // Bad: Allocates multiple times as it grows
    let mut map1 = HashMap::new();
    for (k, v) in &items {
        map1.insert(*k, *v);
    }

    // Good: Allocates once
    let mut map2 = HashMap::with_capacity(items.len());
    for (k, v) in &items {
        map2.insert(*k, *v);
    }

    // Better: Use FromIterator
    let map3: HashMap<_, _> = items.iter().copied().collect();
}
```

## Pattern 4: Zero-Cost Abstractions

**Problem**: Abstractions seem to cost performance—iterators look like overhead compared to raw loops. Generic code appears to create bloat. Function calls have overhead. Wrapper types seem expensive. Unclear when compiler optimizes abstractions away. Iterator chains look slow. Trait objects require vtable dispatch. Inline hints unclear.

**Solution**: Iterators compile to same code as loops—no overhead, often faster due to optimization opportunities. Generics monomorphize: separate copy per type, zero runtime cost. Small functions inlined with #[inline]—no call overhead. Newtypes are zero-cost: same representation as wrapped type. Release mode optimizations aggressive: inlining, dead code elimination, constant propagation. Use const fn for compile-time computation. LLVM optimizes aggressively: trust the compiler.

**Why It Matters**: Abstractions are free when used correctly—no performance penalty for clean code. Iterators as fast as loops: compiler sees intent, optimizes better. Generics zero runtime cost: monomorphization at compile-time, no dynamic dispatch. Newtype pattern zero-cost: UserId(u64) same as u64 at runtime. Release mode transformative: 10-100x faster than debug. Const evaluation moves work to compile-time. Zero-cost philosophy: can have nice things without paying.

**Use Cases**: Iterator chains (map/filter/collect as fast as loops), generic algorithms (HashMap<K, V> monomorphized per type), small wrapper functions (#[inline] eliminates overhead), newtype pattern (UserId type safety, u64 performance), compile-time computation (const fn, const generics), abstraction layers (trait boundaries with inlining), DSLs (zero-cost builder patterns).

### Example: Understanding Branch Misprediction

```rust
use std::time::Instant;

fn with_unpredictable_branch(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data {
        // Unpredictable - depends on data
        if x % 2 == 0 {
            sum += x;
        }
    }
    sum
}

fn without_branch(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data {
        // Branchless - uses arithmetic
        sum += x * (x % 2 == 0) as i32;
    }
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

```rust
//============
// With branch
//============
fn max_with_branch(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

//===========
// Branchless
//===========
fn max_branchless(a: i32, b: i32) -> i32 {
    let diff = a - b;
    let sign = diff >> 31;  // -1 if negative, 0 if positive
    a - (diff & sign)
}

//=======================================================
// Or use LLVM's select (compiler does this optimization)
//=======================================================
fn max_select(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }  // LLVM converts to select instruction
}
```

The compiler often optimizes simple `if` expressions to branchless code automatically.

### Example: Likely and Unlikely Hints

```rust
//====================================
// Unstable feature - requires nightly
//====================================
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

//==============================
// Stable alternative using cold
//==============================
#[cold]
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

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(u8, u8, u8),
}

//==========================================
// Less optimizable - many possible branches
//==========================================
fn process_message_bad(msg: &Message) -> String {
    match msg {
        Message::Quit => "quit".to_string(),
        Message::Move { x, y } => format!("move {} {}", x, y),
        Message::Write(s) => s.clone(),
        Message::ChangeColor(r, g, b) => format!("color {} {} {}", r, g, b),
    }
}

//=====================================
// More optimizable - common case first
//=====================================
fn process_message_good(msg: &Message) -> String {
    match msg {
        Message::Write(s) => s.clone(),  // Assume this is most common
        Message::Move { x, y } => format!("move {} {}", x, y),
        Message::ChangeColor(r, g, b) => format!("color {} {} {}", r, g, b),
        Message::Quit => "quit".to_string(),
    }
}
```

Put common cases first in match statements to improve branch prediction.

## Pattern 5: Compiler Optimizations

`**Problem**: Compiler optimization levels unclear—debug vs release massive difference. Link-time optimization (LTO) benefits unknown. Profile-guided optimization (PGO) not used. target-cpu=generic misses CPU-specific instructions (SIMD). codegen-units affects optimization. Binary size vs speed trade-off. Optimization flags scattered, unclear which matter.

**Solution**: Release profile: opt-level=3 for maximum speed, opt-level='z' for size. LTO=true enables cross-crate inlining and optimization. PGO: collect profile (instrumentation build), then optimize based on hot paths. target-cpu=native enables all CPU features (AVX, SSE). codegen-units=1 for max optimization (slower compile). panic='abort' for smaller binaries. strip=true removes debug symbols. Incremental=false for release builds (faster runtime, slower compile).

**Why It Matters**: Release mode 10-100x faster than debug—opt-level=3 enables aggressive optimizations. LTO enables whole-program optimization: inline across crates, dead code elimination globally. PGO 10-30% speedup: optimizes for actual hot paths, better branch prediction. target-cpu=native uses SIMD: AVX2 can be 4-8x faster for numerical code. codegen-units=1 vs 16: better optimization but slower compile. Understanding flags = max performance from compiler.

**Use Cases**: Production builds (max opt-level, LTO, target-cpu=native), benchmarking (release mode mandatory, consistent codegen-units), CPU-intensive code (numerical computing benefits from SIMD via target-cpu), binary size reduction (embedded, WASM—use opt-level='z', LTO), CI/CD (PGO for production artifacts), maximum performance (combine all: LTO + PGO + target-cpu=native + codegen-units=1).`


### Example: Const Functions

Const functions execute at compile time when possible:

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

Pre-compute lookup tables at compile time:

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

Verify invariants at compile time:

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

Use build scripts to generate code:

```rust
//=========
// build.rs
//=========
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
// src/lib.rs
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

fn is_prime_fast(n: u32) -> bool {
    PRIMES.binary_search(&n).is_ok()
}
```

### Example: Compile-Time String Processing

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

### Type-Level Computation

Use the type system for compile-time computation:

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



### SIMD (Single Instruction Multiple Data)

Process multiple values simultaneously:

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

### Inline Assembly

For absolute control (rarely needed):

```rust
#![feature(asm)]

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

### Link-Time Optimization

Enable in Cargo.toml:

```toml
[profile.release]
lto = "fat"        # Full LTO
codegen-units = 1  # Single codegen unit for better optimization
```

LTO enables cross-crate inlining and optimization, often yielding 10-20% speedup at the cost of longer compile times.

## Summary

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

**Common Optimizations**:
```rust
// Pre-allocate capacity
let mut vec = Vec::with_capacity(1000);  // vs Vec::new()

// Reuse buffers
buffer.clear();  // vs let mut buffer = String::new()

// SmallVec for small collections
let small: SmallVec<[i32; 4]> = SmallVec::new();  // Stack if ≤4 elements

// Cow for conditional cloning
fn process(s: &str) -> Cow<str> {
    if needs_change { Cow::Owned(modified) } else { Cow::Borrowed(s) }
}

// Struct-of-arrays
struct Particles {
    x: Vec<f32>, y: Vec<f32>, z: Vec<f32>  // vs Vec<Particle>
}

// Iterator chains (zero-cost)
items.iter().filter(|x| x.is_valid()).map(|x| x.process()).collect()

// Compiler optimizations in Cargo.toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
```

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


## Performance Optimization Cheat Sheet

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


