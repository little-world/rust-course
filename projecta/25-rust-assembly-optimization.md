# Chapter 25: Rust and Assembly Programming

## Project: From Rust to Assembly - Performance Optimization Journey

### Problem Statement

Build a deep understanding of how Rust code translates to assembly and leverage low-level optimizations for performance-critical code. You'll start by examining compiler output, verify zero-cost abstractions, write inline assembly, implement SIMD operations, understand calling conventions, and finally hand-optimize hot paths.

### Core Concepts

This project bridges high-level Rust abstractions with low-level machine code, teaching you when and how to reach for assembly optimization.

#### 1. **Compilation Pipeline: Rust → LLVM IR → Assembly → Machine Code**

Rust doesn't compile directly to assembly. The pipeline is:

```
Rust Source Code
    ↓ [rustc frontend]
LLVM IR (Intermediate Representation)
    ↓ [LLVM optimizer]
Optimized LLVM IR
    ↓ [LLVM backend]
Assembly (.s file)
    ↓ [assembler]
Object Code (.o file)
    ↓ [linker]
Executable Binary
```

**Why this matters**:
- **LLVM IR**: Platform-independent, allows cross-platform optimizations
- **Assembly**: Platform-specific (x86_64, ARM, etc.)
- **Optimization levels**: `-O0` (debug), `-O1`, `-O2`, `-O3` (release), `-Oz` (size)

**Example transformation**:
```rust
// Rust code
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// LLVM IR (simplified)
define i32 @add(i32 %a, i32 %b) {
    %result = add i32 %a, %b
    ret i32 %result
}

// x86_64 Assembly
add:
    lea eax, [rdi + rsi]
    ret
```

#### 2. **Zero-Cost Abstractions: Iterator Chains vs Manual Loops**

**The Promise**: High-level abstractions (iterators, closures) should compile to the same assembly as hand-written loops.

**Example**:
```rust
// High-level iterator chain
let sum: i32 = vec.iter().filter(|&&x| x > 0).sum();

// Manual loop
let mut sum = 0;
for &x in &vec {
    if x > 0 {
        sum += x;
    }
}

// Both produce identical assembly after optimization!
```

**Why it works**:
- **Inlining**: Compiler inlines iterator methods
- **Dead code elimination**: Unused code paths removed
- **Loop unrolling**: Repetitive operations merged
- **LLVM optimizations**: 100+ optimization passes

**Performance numbers**:
- **Debug build**: Iterator ~5x slower (no inlining)
- **Release build**: Iterator = manual loop (same assembly)
- **Binary size**: Iterator may be slightly larger (more inlined code)

#### 3. **Inline Assembly: `asm!` Macro**

**When to use**:
- ✅ Access CPU-specific instructions not exposed by Rust
- ✅ Avoid function call overhead for single instruction
- ✅ Implement algorithms proven faster in assembly (rare!)
- ❌ NOT for premature optimization (compiler is smarter)

**Syntax**:
```rust
use std::arch::asm;

let result: u64;
unsafe {
    asm!(
        "add {0}, {1}",     // Assembly template
        inout(reg) a => result,  // Input/output operand
        in(reg) b,          // Input operand
        options(pure, nomem, nostack),  // Optimization hints
    );
}
```

**Constraints**:
- `in(reg)`: Input in any general-purpose register
- `out(reg)`: Output to any register
- `inout(reg)`: Same register for input and output
- `lateout(reg)`: Output written after all inputs read
- `const`: Compile-time constant
- `sym`: Symbol address

**Clobbers**: Tell compiler what gets modified
- `options(nostack)`: Doesn't touch stack
- `options(nomem)`: Doesn't read/write memory
- `options(pure)`: No side effects

#### 4. **SIMD (Single Instruction, Multiple Data)**

**The Idea**: Process multiple values in parallel using vector registers.

**Example - Adding 4 integers at once**:
```rust
// Scalar: 4 separate operations
let r1 = a1 + b1;
let r2 = a2 + b2;
let r3 = a3 + b3;
let r4 = a4 + b4;

// SIMD: 1 operation on 4 values
// Using AVX2 (256-bit registers)
use std::arch::x86_64::*;
unsafe {
    let a = _mm256_set_epi32(a1, a2, a3, a4, 0, 0, 0, 0);
    let b = _mm256_set_epi32(b1, b2, b3, b4, 0, 0, 0, 0);
    let result = _mm256_add_epi32(a, b);
}
```

**Performance**:
```
Scalar addition (1M elements):     ~2.5ms
Auto-vectorized (LLVM):            ~0.8ms (3x faster)
Manual SIMD (AVX2):                ~0.6ms (4x faster)
Manual SIMD + loop unroll:         ~0.4ms (6x faster)
```

**SIMD Instruction Sets**:
- **SSE2**: 128-bit (4×i32, 2×i64, 4×f32, 2×f64) - universal on x86_64
- **AVX2**: 256-bit (8×i32, 4×i64, 8×f32, 4×f64) - modern Intel/AMD
- **AVX-512**: 512-bit (16×i32, etc.) - high-end servers
- **NEON**: ARM SIMD (128-bit)

#### 5. **Calling Conventions and ABI**

**ABI (Application Binary Interface)**: Rules for function calls at assembly level.

**x86_64 System V ABI** (Linux, macOS):
- **Arguments**: First 6 in registers (`rdi`, `rsi`, `rdx`, `rcx`, `r8`, `r9`), rest on stack
- **Return value**: `rax` (integer), `xmm0` (float)
- **Caller-saved**: `rax`, `rcx`, `rdx`, `rsi`, `rdi`, `r8`-`r11`
- **Callee-saved**: `rbx`, `rbp`, `r12`-`r15`

**Windows x64 ABI**:
- **Arguments**: First 4 in registers (`rcx`, `rdx`, `r8`, `r9`)
- **Shadow space**: Caller allocates 32 bytes on stack

**Example**:
```rust
// Rust function signature
extern "C" fn add(a: i64, b: i64, c: i64) -> i64 {
    a + b + c
}

// x86_64 Linux assembly
add:
    lea rax, [rdi + rsi]  // a + b
    add rax, rdx          // + c
    ret

// Call from assembly:
mov rdi, 10   // First arg
mov rsi, 20   // Second arg
mov rdx, 30   // Third arg
call add      // Result in rax
```

**Why this matters**:
- **FFI**: Calling C libraries requires matching ABI
- **Inline assembly**: Must preserve callee-saved registers
- **Performance**: Understanding register allocation helps optimize

#### 6. **System Calls: Kernel Interface**

**System calls**: Request kernel services (file I/O, networking, etc.)

**Mechanism**:
```rust
// Rust wrapper (libc)
unsafe { libc::write(1, b"Hello\n".as_ptr(), 6) };

// Under the hood (x86_64 Linux)
mov rax, 1      // syscall number (write)
mov rdi, 1      // fd (stdout)
mov rsi, msg    // buffer pointer
mov rdx, 6      // length
syscall         // Invoke kernel
```

**Syscall numbers** (x86_64 Linux):
- `0`: read
- `1`: write
- `2`: open
- `3`: close
- `60`: exit

**Performance**:
- **Syscall overhead**: ~100-500ns (context switch to kernel)
- **Comparison**: Function call ~1-5ns
- **Implication**: Batch operations to minimize syscalls

#### 7. **CPU Performance Features**

**Branch Prediction**:
```rust
// Predictable branches (loop)
for i in 0..1000 {
    sum += i;  // Branch at loop end is predicted
}
// Cost: ~1 cycle per iteration

// Unpredictable branches (random data)
for &x in data {
    if x > threshold {  // Unpredictable!
        sum += x;
    }
}
// Cost: ~10-20 cycles on misprediction
```

**Cache Locality**:
```rust
// Bad: Random access (cache misses)
for &i in indices {
    sum += array[i];  // Each access may miss cache
}
// ~100 cycles per miss

// Good: Sequential access (cache hits)
for &x in array {
    sum += x;  // Data prefetched into cache
}
// ~1 cycle per access (L1 cache)
```

**Instruction-Level Parallelism (ILP)**:
```rust
// Poor ILP: Data dependency chain
let mut x = 1;
for _ in 0..100 {
    x = x * 2;  // Must wait for previous iteration
}

// Good ILP: Independent operations
let mut x1 = 1;
let mut x2 = 1;
for _ in 0..100 {
    x1 = x1 * 2;  // Can execute in parallel
    x2 = x2 * 3;
}
```

#### 8. **Profiling and Benchmarking**

**Tools**:
- **perf** (Linux): CPU counters, cache misses, branch mispredictions
- **Instruments** (macOS): Time profiler, allocations
- **criterion**: Rust benchmarking with statistical rigor
- **cargo-asm**: View assembly for specific functions

**Key metrics**:
```
perf stat ./program

Performance counter stats:
    1,234,567,890  instructions          #  2.34  insn per cycle
      567,890,123  cycles
       12,345,678  cache-misses          #  1.23% of all refs
          123,456  branch-misses         #  0.12% of all branches
```

**Optimization workflow**:
1. **Profile**: Identify hot paths (80/20 rule)
2. **Measure**: Benchmark current performance
3. **Optimize**: Try improvements (algorithm, then micro-optimizations)
4. **Verify**: Re-benchmark, ensure improvement
5. **Repeat**: Focus on next bottleneck

---

### Connection to This Project

This project takes you through a complete optimization journey:

1. **Milestone 1**: Understand the compilation process by examining assembly output
2. **Milestone 2**: Verify zero-cost abstractions by comparing iterator chains to manual loops
3. **Milestone 3**: Write your first inline assembly for simple operations
4. **Milestone 4**: Leverage SIMD for parallel data processing (4-8x speedup)
5. **Milestone 5**: Master calling conventions and make raw system calls
6. **Milestone 6**: Apply all techniques to optimize a real hot path

**Performance progression**:
- **Baseline** (naive Rust): 100ms
- **Algorithm improvement**: 50ms (2x faster)
- **Zero-cost abstractions verified**: 50ms (no regression)
- **SIMD optimization**: 12ms (8x faster than baseline)
- **Hand-tuned assembly**: 10ms (10x faster than baseline)

**When to use each technique**:
- **99% of code**: Let the compiler optimize (trust LLVM)
- **0.9% of code**: Use high-level SIMD intrinsics
- **0.1% of code**: Hand-written inline assembly (only after profiling!)

**Real-world applications**:
- **Cryptography**: AES, SHA hashing with SIMD
- **Image processing**: Filter operations, color space conversion
- **Compression**: zlib, zstd use assembly hot paths
- **Databases**: Sorting, hashing, vectorized scans
- **Game engines**: Vector math, physics simulations

---

## Milestone 1: From Rust to Assembly

### Introduction

**Goal**: Learn to read Rust-generated assembly and understand how the compiler optimizes code.

**Why this matters**: You can't optimize what you don't measure. Before writing assembly, you must understand what the compiler already generates. Often, Rust's optimizer (LLVM) produces better assembly than hand-written code!

**Tools**:
- `cargo rustc -- --emit=asm`: Generate assembly files
- `cargo-asm`: View assembly for specific functions
- `objdump`: Disassemble binaries
- Compiler Explorer (godbolt.org): Online assembly viewer

### Key Concepts

**Assembly basics**:
```asm
; x86_64 AT&T syntax (default on Linux/macOS)
movq %rdi, %rax    ; Move rdi to rax (% prefix = register)
addq %rsi, %rax    ; Add rsi to rax
retq               ; Return (value in rax)

; Intel syntax (more readable, used in this project)
mov rax, rdi       ; Move rdi to rax
add rax, rsi       ; Add rsi to rax
ret                ; Return
```

**Common instructions**:
- `mov`: Move data between registers/memory
- `add`, `sub`, `imul`, `idiv`: Arithmetic
- `lea`: Load Effective Address (fast addition)
- `cmp`, `test`: Comparison (sets flags)
- `je`, `jne`, `jg`, `jl`: Conditional jumps
- `call`, `ret`: Function call/return
- `push`, `pop`: Stack operations

**Registers (x86_64)**:
- **General purpose**: `rax`, `rbx`, `rcx`, `rdx`, `rsi`, `rdi`, `r8`-`r15`
- **Special**: `rsp` (stack pointer), `rbp` (base pointer), `rip` (instruction pointer)
- **Smaller variants**: `eax` (32-bit), `ax` (16-bit), `al` (8-bit)

### Architecture

**Functions**:
```rust
// Simple addition
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Expected assembly (x86_64 Intel syntax):
// add:
//     lea eax, [rdi + rsi]  ; Use LEA for addition (faster)
//     ret

// Array sum
fn sum_array(arr: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in arr {
        sum += x;
    }
    sum
}

// Expected: Loop with auto-vectorization (SIMD)
```

### Starter Code

```rust
// Add to Cargo.toml:
// [profile.release]
// opt-level = 3
// lto = true

fn add(a: i32, b: i32) -> i32 {
    // TODO: Implement simple addition
    todo!()
}

fn multiply(a: i32, b: i32) -> i32 {
    // TODO: Implement multiplication
    todo!()
}

fn factorial(n: u64) -> u64 {
    // TODO: Implement factorial (iterative, not recursive)
    // This will show loop assembly
    todo!()
}

fn sum_array(arr: &[i32]) -> i32 {
    // TODO: Sum all elements
    // Check if compiler auto-vectorizes this!
    todo!()
}

fn main() {
    println!("add(5, 3) = {}", add(5, 3));
    println!("multiply(4, 7) = {}", multiply(4, 7));
    println!("factorial(10) = {}", factorial(10));

    let arr = vec![1, 2, 3, 4, 5];
    println!("sum([1,2,3,4,5]) = {}", sum_array(&arr));
}
```

**Generate assembly**:
```bash
# Method 1: Cargo (generates multiple .s files)
cargo rustc --release -- --emit=asm -C "llvm-args=-x86-asm-syntax=intel"

# Method 2: rustc directly
rustc --emit=asm -C opt-level=3 -C "llvm-args=-x86-asm-syntax=intel" src/main.rs

# Method 3: cargo-asm (install: cargo install cargo-asm)
cargo asm --rust milestone_1::add

# Method 4: Compiler Explorer
# Visit https://godbolt.org, select Rust, paste code, view assembly
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(5, 3), 8);
        assert_eq!(add(-10, 20), 10);
        assert_eq!(add(0, 0), 0);
    }

    #[test]
    fn test_multiply() {
        assert_eq!(multiply(4, 7), 28);
        assert_eq!(multiply(0, 100), 0);
        assert_eq!(multiply(-3, 5), -15);
    }

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
        assert_eq!(factorial(10), 3628800);
    }

    #[test]
    fn test_sum_array() {
        assert_eq!(sum_array(&[]), 0);
        assert_eq!(sum_array(&[1, 2, 3, 4, 5]), 15);
        assert_eq!(sum_array(&[-1, -2, -3]), -6);
    }
}
```

**Assembly Analysis Tasks**:

1. **Examine `add` function**:
   - Look for `lea` instruction (Load Effective Address)
   - Note: `lea eax, [rdi + rsi]` is faster than `mov + add`
   - Count instructions (should be 2-3 lines)

2. **Examine `factorial` loop**:
   - Find loop label and jump instructions
   - Look for loop unrolling (compiler may optimize)
   - Count iterations if manually traced

3. **Examine `sum_array`**:
   - Check for SIMD instructions (`paddd`, `vpaddd`, etc.)
   - Compare debug vs release builds
   - Note: Compiler may auto-vectorize!

### Check Your Understanding

- **What is `lea` and why is it used for addition?**
- **How does the compiler optimize `factorial`? Is the loop unrolled?**
- **Did the compiler auto-vectorize `sum_array`? How can you tell?**
- **What's the difference between debug and release assembly?**
- **How many instructions does `add` compile to?**

---

## Why Milestone 1 Isn't Enough → Moving to Milestone 2

**Limitation**: We've seen assembly, but haven't verified if Rust's high-level abstractions (iterators, closures) have zero overhead.

**Skepticism**: "Iterators look nice, but they must be slower than manual loops, right?"

**What we're proving**: Iterator chains compile to **identical assembly** as manual loops.

**Improvement**:
- **Verification**: Prove zero-cost abstractions aren't just marketing
- **Confidence**: Use iterators without guilt
- **Clarity**: Choose readable code over premature manual optimization

---

## Milestone 2: Zero-Cost Abstractions - Iterator Combinators

### Introduction

**The Claim**: Rust's iterators are "zero-cost abstractions" - they compile to the same assembly as hand-written loops.

**Your Mission**: Prove it! Write the same algorithm using iterators and manual loops, then compare assembly.

**Examples**:
1. Sum of filtered values
2. Map-reduce pipeline
3. Chained transformations

### Key Concepts

**Iterator methods**:
```rust
vec.iter()           // Iterate over references
   .filter(|x| ...)  // Keep elements matching predicate
   .map(|x| ...)     // Transform each element
   .sum()            // Fold into sum
```

**How inlining works**:
```rust
// High-level code
let sum: i32 = vec.iter().filter(|&&x| x > 0).sum();

// After inlining (conceptual, not actual IR)
let mut sum = 0;
let mut iter = vec.iter();
loop {
    match iter.next() {
        Some(&x) if x > 0 => sum += x,
        Some(_) => {},
        None => break,
    }
}

// After optimization
let mut sum = 0;
for &x in &vec {
    if x > 0 { sum += x; }
}

// Final assembly: Same as if you wrote the loop manually!
```

### Architecture

**Comparison pairs**:

```rust
// Pair 1: Simple sum
fn sum_iterator(v: &[i32]) -> i32 {
    v.iter().sum()
}

fn sum_manual(v: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in v {
        sum += x;
    }
    sum
}

// Pair 2: Filter and sum
fn sum_positive_iterator(v: &[i32]) -> i32 {
    v.iter().filter(|&&x| x > 0).sum()
}

fn sum_positive_manual(v: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in v {
        if x > 0 {
            sum += x;
        }
    }
    sum
}

// Pair 3: Map and sum
fn sum_squares_iterator(v: &[i32]) -> i32 {
    v.iter().map(|&x| x * x).sum()
}

fn sum_squares_manual(v: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in v {
        sum += x * x;
    }
    sum
}

// Pair 4: Complex chain
fn complex_iterator(v: &[i32]) -> i32 {
    v.iter()
        .filter(|&&x| x > 0)
        .map(|&x| x * 2)
        .filter(|&x| x < 100)
        .sum()
}

fn complex_manual(v: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in v {
        if x > 0 {
            let doubled = x * 2;
            if doubled < 100 {
                sum += doubled;
            }
        }
    }
    sum
}
```

### Starter Code

```rust
// TODO: Implement all 8 functions above

fn main() {
    let data: Vec<i32> = (1..=100).collect();

    println!("sum_iterator: {}", sum_iterator(&data));
    println!("sum_manual: {}", sum_manual(&data));

    println!("sum_positive_iterator: {}", sum_positive_iterator(&data));
    println!("sum_positive_manual: {}", sum_positive_manual(&data));

    println!("sum_squares_iterator: {}", sum_squares_iterator(&data));
    println!("sum_squares_manual: {}", sum_squares_manual(&data));

    println!("complex_iterator: {}", complex_iterator(&data));
    println!("complex_manual: {}", complex_manual(&data));
}
```

**Compare assembly**:
```bash
cargo asm --rust sum_iterator > iterator.asm
cargo asm --rust sum_manual > manual.asm
diff iterator.asm manual.asm
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum() {
        let v = vec![1, 2, 3, 4, 5];
        assert_eq!(sum_iterator(&v), sum_manual(&v));
        assert_eq!(sum_iterator(&v), 15);
    }

    #[test]
    fn test_sum_positive() {
        let v = vec![-5, -2, 0, 3, 7, -1, 10];
        assert_eq!(sum_positive_iterator(&v), sum_positive_manual(&v));
        assert_eq!(sum_positive_iterator(&v), 20);
    }

    #[test]
    fn test_sum_squares() {
        let v = vec![1, 2, 3, 4];
        assert_eq!(sum_squares_iterator(&v), sum_squares_manual(&v));
        assert_eq!(sum_squares_iterator(&v), 30);
    }

    #[test]
    fn test_complex() {
        let v = vec![-10, 5, 30, 60, 100, 120];
        assert_eq!(complex_iterator(&v), complex_manual(&v));
        assert_eq!(complex_iterator(&v), 10 + 60);
    }

    // IMPORTANT: Verify assembly is identical!
    #[test]
    #[ignore] // Manual check
    fn verify_zero_cost_abstraction() {
        // Run: cargo asm --rust sum_iterator
        // Run: cargo asm --rust sum_manual
        // Compare: Both should produce nearly identical assembly
        panic!("Manual verification required - check assembly output");
    }
}
```

### Check Your Understanding

- **Do iterator and manual versions produce the same assembly?**
- **Are there any differences? If so, why?**
- **What happens in debug mode vs release mode?**
- **Can you find the inlined iterator methods in the assembly?**
- **Is there any performance difference? (Benchmark with criterion)**

---

## Why Milestone 2 Isn't Enough → Moving to Milestone 3

**Limitation**: We've verified the compiler is smart, but sometimes we need features not exposed by Rust.

**Examples**:
- CPU-specific instructions (RDTSC, CPUID)
- Atomics with exotic memory orderings
- Bit manipulation tricks
- Single-instruction operations (BSF, BSR, POPCNT)

**What we're adding**: Direct inline assembly using the `asm!` macro.

**Improvement**: Access the full power of the CPU without function call overhead.

---

## Milestone 3: Inline Assembly Basics

### Introduction

**Goal**: Write inline assembly for simple operations and understand the `asm!` macro syntax.

**Use cases**:
- CPU feature detection (`cpuid`)
- Timestamp counters (`rdtsc`)
- Bit manipulation (`bsf`, `bsr`, `popcnt`)
- Memory barriers (`mfence`, `lfence`, `sfence`)

**Syntax overview**:
```rust
use std::arch::asm;

unsafe {
    asm!(
        "instruction {out}, {in}",  // Assembly template
        out = out(reg) output_var,  // Output operand
        in = in(reg) input_var,     // Input operand
        options(...),               // Optimization hints
    );
}
```

### Key Concepts

**Constraints**:
- `reg`: Any general-purpose register
- `reg_abcd`: Only `rax`, `rbx`, `rcx`, `rdx`
- `xmm_reg`: SIMD register
- `const`: Compile-time constant
- `sym`: Function or static symbol

**Options**:
- `pure`: No side effects, can be optimized away if unused
- `nomem`: Doesn't read or write memory
- `readonly`: Only reads memory
- `preserves_flags`: Doesn't modify CPU flags
- `nostack`: Doesn't touch stack
- `att_syntax`: Use AT&T syntax instead of Intel

### Architecture

**Functions to implement**:

```rust
use std::arch::asm;

// 1. CPU timestamp counter (for micro-benchmarking)
pub fn rdtsc() -> u64 {
    // TODO: Use RDTSC instruction
    // Returns number of CPU cycles since boot
    todo!()
}

// 2. CPU feature detection
pub fn cpuid(leaf: u32) -> (u32, u32, u32, u32) {
    // TODO: Use CPUID instruction
    // Returns (eax, ebx, ecx, edx)
    todo!()
}

// 3. Count trailing zeros (BSF - Bit Scan Forward)
pub fn count_trailing_zeros(x: u64) -> u32 {
    // TODO: Use BSF instruction
    // Returns index of first set bit
    todo!()
}

// 4. Count leading zeros (BSR - Bit Scan Reverse)
pub fn count_leading_zeros(x: u64) -> u32 {
    // TODO: Use BSR instruction
    // Returns index of last set bit
    todo!()
}

// 5. Population count (number of 1 bits)
pub fn popcnt(x: u64) -> u32 {
    // TODO: Use POPCNT instruction (requires SSE4.2)
    todo!()
}

// 6. Byte swap (endianness conversion)
pub fn bswap(x: u64) -> u64 {
    // TODO: Use BSWAP instruction
    todo!()
}
```

### Starter Code

```rust
#![feature(asm_const)]
use std::arch::asm;

pub fn rdtsc() -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!(
            "rdtsc",
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags),
        );
    }
    ((high as u64) << 32) | (low as u64)
}

pub fn cpuid(leaf: u32) -> (u32, u32, u32, u32) {
    let mut eax: u32;
    let mut ebx: u32;
    let mut ecx: u32 = 0;
    let mut edx: u32;

    unsafe {
        asm!(
            // TODO: Call CPUID with leaf in eax
            // Results in eax, ebx, ecx, edx
            "cpuid",
            inout("eax") leaf => eax,
            out("ebx") ebx,
            inout("ecx") ecx,
            out("edx") edx,
        );
    }
    (eax, ebx, ecx, edx)
}

pub fn count_trailing_zeros(x: u64) -> u32 {
    if x == 0 {
        return 64; // BSF is undefined for 0
    }
    let result: u64;
    unsafe {
        asm!(
            "bsf {result}, {input}",
            result = out(reg) result,
            input = in(reg) x,
            options(nomem, nostack),
        );
    }
    result as u32
}

// TODO: Implement remaining functions

fn main() {
    // Benchmark example
    let start = rdtsc();
    let mut sum = 0;
    for i in 0..1000 {
        sum += i;
    }
    let end = rdtsc();
    println!("Cycles: {}", end - start);
    println!("Result: {}", sum);

    // CPU info
    let (eax, ebx, ecx, edx) = cpuid(0);
    println!("Max CPUID leaf: {}", eax);

    // Bit manipulation
    println!("Trailing zeros of 0b1000: {}", count_trailing_zeros(0b1000));
}
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rdtsc() {
        let t1 = rdtsc();
        let t2 = rdtsc();
        assert!(t2 > t1, "Timestamp should increase");
    }

    #[test]
    fn test_cpuid() {
        let (eax, _ebx, _ecx, _edx) = cpuid(0);
        assert!(eax > 0, "Should support at least CPUID leaf 0");
    }

    #[test]
    fn test_count_trailing_zeros() {
        assert_eq!(count_trailing_zeros(0b1000), 3);
        assert_eq!(count_trailing_zeros(0b1), 0);
        assert_eq!(count_trailing_zeros(0b10), 1);
        assert_eq!(count_trailing_zeros(0), 64);
    }

    #[test]
    fn test_count_leading_zeros() {
        assert_eq!(count_leading_zeros(0b1), 63);
        assert_eq!(count_leading_zeros(0b1000), 60);
        assert_eq!(count_leading_zeros(1u64 << 63), 0);
    }

    #[test]
    fn test_popcnt() {
        assert_eq!(popcnt(0b1010), 2);
        assert_eq!(popcnt(0b1111), 4);
        assert_eq!(popcnt(0), 0);
        assert_eq!(popcnt(u64::MAX), 64);
    }

    #[test]
    fn test_bswap() {
        assert_eq!(bswap(0x0123456789ABCDEF), 0xEFCDAB8967452301);
        assert_eq!(bswap(0x1122334455667788), 0x8877665544332211);
    }
}
```

### Check Your Understanding

- **What does `rdtsc` measure? Is it accurate for micro-benchmarking?**
- **Why does `cpuid` clobber multiple registers?**
- **What happens if you call `bsf` with input 0?**
- **When should you use inline assembly vs Rust's `leading_zeros()` method?**
- **What do the `options()` tell the compiler?**

---

## Why Milestone 3 Isn't Enough → Moving to Milestone 4

**Limitation**: Single operations are nice, but modern CPUs can process multiple values simultaneously using SIMD.

**Example**: Adding two arrays element-wise
- Scalar: 1 add per cycle
- SIMD (SSE2): 4 adds per cycle (4x faster)
- SIMD (AVX2): 8 adds per cycle (8x faster)

**What we're adding**: SIMD intrinsics and vectorized algorithms.

**Improvement**: 4-8x performance for data-parallel operations.

---

## Milestone 4: SIMD Operations with Assembly

### Introduction

**Goal**: Use SIMD (Single Instruction, Multiple Data) to process arrays in parallel.

**SIMD widths**:
- **SSE2** (128-bit): 4×f32, 2×f64, 4×i32, 2×i64
- **AVX2** (256-bit): 8×f32, 4×f64, 8×i32, 4×i64
- **AVX-512** (512-bit): 16×f32, 8×f64, 16×i32, 8×i64

**Approach**:
1. **Let compiler auto-vectorize** (check assembly)
2. **Use intrinsics** (portable, safe-ish)
3. **Write inline assembly** (last resort, platform-specific)

### Key Concepts

**SIMD intrinsics** (platform-specific):
```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

unsafe {
    // Load 4 floats into 128-bit register
    let a = _mm_set_ps(1.0, 2.0, 3.0, 4.0);
    let b = _mm_set_ps(5.0, 6.0, 7.0, 8.0);

    // Add all 4 pairs in parallel
    let result = _mm_add_ps(a, b);

    // Store back to memory
    let mut out = [0.0f32; 4];
    _mm_storeu_ps(out.as_mut_ptr(), result);
}
```

**Auto-vectorization**:
```rust
// Compiler may auto-vectorize this!
fn add_arrays(a: &[f32], b: &[f32], out: &mut [f32]) {
    for i in 0..a.len() {
        out[i] = a[i] + b[i];
    }
}

// Check assembly for `vaddps` or `addps` instructions
```

### Architecture

**Functions to implement**:

```rust
// 1. Scalar baseline
pub fn add_arrays_scalar(a: &[f32], b: &[f32]) -> Vec<f32> {
    a.iter().zip(b).map(|(&x, &y)| x + y).collect()
}

// 2. Auto-vectorization (let compiler try)
pub fn add_arrays_auto(a: &[f32], b: &[f32]) -> Vec<f32> {
    let mut result = vec![0.0; a.len()];
    for i in 0..a.len() {
        result[i] = a[i] + b[i];
    }
    result
}

// 3. Explicit SIMD (SSE2)
#[cfg(target_arch = "x86_64")]
pub fn add_arrays_sse2(a: &[f32], b: &[f32]) -> Vec<f32> {
    // TODO: Process 4 elements at a time using _mm_add_ps
    todo!()
}

// 4. Explicit SIMD (AVX2)
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn add_arrays_avx2(a: &[f32], b: &[f32]) -> Vec<f32> {
    // TODO: Process 8 elements at a time using _mm256_add_ps
    todo!()
}

// 5. Dot product (sum of element-wise products)
pub fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(&x, &y)| x * y).sum()
}

#[cfg(target_arch = "x86_64")]
pub fn dot_product_simd(a: &[f32], b: &[f32]) -> f32 {
    // TODO: Use SIMD multiplication + horizontal sum
    todo!()
}

// 6. Find maximum value
pub fn max_value_scalar(arr: &[f32]) -> f32 {
    arr.iter().copied().fold(f32::NEG_INFINITY, f32::max)
}

#[cfg(target_arch = "x86_64")]
pub fn max_value_simd(arr: &[f32]) -> f32 {
    // TODO: Use _mm_max_ps or _mm256_max_ps
    todo!()
}
```

### Starter Code

```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub fn add_arrays_scalar(a: &[f32], b: &[f32]) -> Vec<f32> {
    a.iter().zip(b).map(|(&x, &y)| x + y).collect()
}

#[cfg(target_arch = "x86_64")]
pub fn add_arrays_sse2(a: &[f32], b: &[f32]) -> Vec<f32> {
    assert_eq!(a.len(), b.len());
    let len = a.len();
    let mut result = vec![0.0f32; len];

    unsafe {
        let chunks = len / 4;

        // Process 4 elements at a time
        for i in 0..chunks {
            let idx = i * 4;

            // Load 4 floats from each array
            let va = _mm_loadu_ps(a.as_ptr().add(idx));
            let vb = _mm_loadu_ps(b.as_ptr().add(idx));

            // Add in parallel
            let vresult = _mm_add_ps(va, vb);

            // Store back
            _mm_storeu_ps(result.as_mut_ptr().add(idx), vresult);
        }

        // Handle remainder (< 4 elements)
        for i in (chunks * 4)..len {
            result[i] = a[i] + b[i];
        }
    }

    result
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn add_arrays_avx2(a: &[f32], b: &[f32]) -> Vec<f32> {
    // TODO: Similar to SSE2, but process 8 elements at a time
    // Use _mm256_loadu_ps, _mm256_add_ps, _mm256_storeu_ps
    todo!()
}

#[cfg(target_arch = "x86_64")]
pub fn dot_product_simd(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    unsafe {
        let len = a.len();
        let chunks = len / 4;

        // Accumulator (4 partial sums)
        let mut acc = _mm_setzero_ps();

        for i in 0..chunks {
            let idx = i * 4;
            let va = _mm_loadu_ps(a.as_ptr().add(idx));
            let vb = _mm_loadu_ps(b.as_ptr().add(idx));

            // Multiply and accumulate
            let prod = _mm_mul_ps(va, vb);
            acc = _mm_add_ps(acc, prod);
        }

        // Horizontal sum of 4 elements in acc
        // acc = [a, b, c, d]
        // temp = [c, d, a, b]
        let temp = _mm_shuffle_ps(acc, acc, 0b_01_00_11_10);
        acc = _mm_add_ps(acc, temp);  // [a+c, b+d, c+a, d+b]

        let temp = _mm_shuffle_ps(acc, acc, 0b_00_00_00_01);
        acc = _mm_add_ps(acc, temp);  // [a+c+b+d, ...]

        let mut result = _mm_cvtss_f32(acc);

        // Add remainder
        for i in (chunks * 4)..len {
            result += a[i] * b[i];
        }

        result
    }
}

// TODO: Implement max_value_simd

fn main() {
    let a: Vec<f32> = (0..1000).map(|i| i as f32).collect();
    let b: Vec<f32> = (0..1000).map(|i| (i * 2) as f32).collect();

    let result_scalar = add_arrays_scalar(&a, &b);
    let result_sse2 = add_arrays_sse2(&a, &b);

    println!("Scalar result[0]: {}", result_scalar[0]);
    println!("SSE2 result[0]: {}", result_sse2[0]);

    let dot = dot_product_simd(&a, &b);
    println!("Dot product: {}", dot);
}
```

**Benchmarking**:
```bash
# Add to Cargo.toml:
# [dev-dependencies]
# criterion = "0.5"

# Then create benches/simd_bench.rs
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_arrays() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let b = vec![8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];

        let scalar = add_arrays_scalar(&a, &b);
        let sse2 = add_arrays_sse2(&a, &b);

        assert_eq!(scalar, sse2);
        assert_eq!(scalar[0], 9.0);
    }

    #[test]
    fn test_dot_product() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];

        let scalar = dot_product_scalar(&a, &b);
        let simd = dot_product_simd(&a, &b);

        assert_eq!(scalar, simd);
        assert_eq!(scalar, 70.0); // 1*5 + 2*6 + 3*7 + 4*8
    }

    #[test]
    fn test_max_value() {
        let arr = vec![3.0, 7.0, 2.0, 9.0, 1.0, 5.0];

        let scalar = max_value_scalar(&arr);
        let simd = max_value_simd(&arr);

        assert_eq!(scalar, simd);
        assert_eq!(scalar, 9.0);
    }
}
```

### Check Your Understanding

- **How many elements can SSE2 process simultaneously for f32?**
- **What's the speedup factor of SIMD over scalar code?**
- **How do you handle array lengths that aren't multiples of 4/8?**
- **What is a horizontal sum and why is it needed?**
- **Did the compiler auto-vectorize your scalar code?**

---

## Why Milestone 4 Isn't Enough → Moving to Milestone 5

**Limitation**: We've optimized computation, but haven't touched system interaction.

**Real-world bottleneck**: System calls (file I/O, networking) often dominate compute time.

**What we're learning**: How functions are called at the assembly level, and how to make raw system calls.

**Improvement**: Understand FFI, calling conventions, and eliminate libc overhead.

---

## Milestone 5: System Calls and ABI

### Introduction

**Goal**: Understand calling conventions and make raw system calls without libc.

**Calling Convention**: Rules for how functions pass arguments and return values in assembly.

**System Call**: Special CPU instruction to invoke kernel services.

### Key Concepts

**x86_64 Linux System V ABI**:
- Arguments: `rdi`, `rsi`, `rdx`, `rcx`, `r8`, `r9`, then stack
- Return: `rax`
- Caller-saved: `rax`, `rcx`, `rdx`, `rsi`, `rdi`, `r8-r11`
- Callee-saved: `rbx`, `rbp`, `r12-r15`

**System calls** (x86_64 Linux):
```asm
mov rax, syscall_number
mov rdi, arg1
mov rsi, arg2
mov rdx, arg3
mov r10, arg4  ; Note: r10, not rcx
mov r8, arg5
mov r9, arg6
syscall        ; Invoke kernel
; Result in rax
```

**Common syscalls**:
- 0: read
- 1: write
- 2: open
- 3: close
- 60: exit

### Architecture

**Functions to implement**:

```rust
use std::arch::asm;

// 1. Exit program
pub fn sys_exit(code: i32) -> ! {
    // TODO: syscall 60
    todo!()
}

// 2. Write to file descriptor
pub fn sys_write(fd: i32, buf: &[u8]) -> isize {
    // TODO: syscall 1
    // Returns bytes written or -errno
    todo!()
}

// 3. Read from file descriptor
pub fn sys_read(fd: i32, buf: &mut [u8]) -> isize {
    // TODO: syscall 0
    todo!()
}

// 4. Get current time
pub fn sys_time() -> i64 {
    // TODO: syscall 201 (time) or use VDSO
    todo!()
}

// 5. FFI: Call C function
extern "C" {
    fn strlen(s: *const u8) -> usize;
}

pub fn call_c_strlen(s: &str) -> usize {
    unsafe {
        strlen(s.as_ptr())
    }
}

// 6. Implement strlen in assembly
pub fn asm_strlen(s: &str) -> usize {
    // TODO: Use inline assembly
    // Loop until null byte, count length
    todo!()
}
```

### Starter Code

```rust
use std::arch::asm;

pub fn sys_exit(code: i32) -> ! {
    unsafe {
        asm!(
            "mov rax, 60",      // syscall number for exit
            "syscall",
            in("rdi") code,     // exit code
            options(noreturn)
        );
    }
}

pub fn sys_write(fd: i32, buf: &[u8]) -> isize {
    let result: isize;
    unsafe {
        asm!(
            "mov rax, 1",       // syscall number for write
            "syscall",
            in("rdi") fd,
            in("rsi") buf.as_ptr(),
            in("rdx") buf.len(),
            lateout("rax") result,
            out("rcx") _,       // Clobbered by syscall
            out("r11") _,       // Clobbered by syscall
        );
    }
    result
}

pub fn sys_read(fd: i32, buf: &mut [u8]) -> isize {
    // TODO: Similar to sys_write, but syscall 0
    todo!()
}

pub fn asm_strlen(s: &str) -> usize {
    let len: usize;
    unsafe {
        asm!(
            "xor {len}, {len}",           // len = 0
            "2:",                          // Loop label
            "cmp byte ptr [{ptr} + {len}], 0",  // Check for null
            "je 3f",                       // If null, exit loop
            "inc {len}",                   // len++
            "jmp 2b",                      // Repeat
            "3:",                          // Exit label
            ptr = in(reg) s.as_ptr(),
            len = out(reg) len,
        );
    }
    len
}

fn main() {
    // Write to stdout
    let msg = b"Hello from raw syscall!\n";
    sys_write(1, msg);

    // String length
    let s = "Hello, world!";
    let len = asm_strlen(s);
    println!("Length: {}", len);

    // Exit
    // sys_exit(0);  // Uncomment to test
}
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_write() {
        let msg = b"test\n";
        let written = sys_write(1, msg);
        assert_eq!(written, 5);
    }

    #[test]
    fn test_asm_strlen() {
        assert_eq!(asm_strlen(""), 0);
        assert_eq!(asm_strlen("hello"), 5);
        assert_eq!(asm_strlen("hello world"), 11);
    }

    #[test]
    #[ignore] // Don't exit during tests
    fn test_sys_exit() {
        sys_exit(42);
    }
}
```

### Check Your Understanding

- **What registers are used for the first 6 arguments?**
- **Why does `syscall` clobber `rcx` and `r11`?**
- **What's the difference between caller-saved and callee-saved registers?**
- **How would you call a Windows API function? (Different ABI)**
- **What's the overhead of a system call vs a function call?**

---

## Why Milestone 5 Isn't Enough → Moving to Milestone 6

**Limitation**: We've learned individual techniques, but haven't applied them to a real optimization problem.

**What we're doing**: Take a real hot path, profile it, and optimize using all techniques learned.

**Improvement**: Practical experience optimizing production-like code.

---

## Milestone 6: Performance-Critical Assembly - Optimizing a Hot Path

### Introduction

**Goal**: Apply all learned techniques to optimize a realistic algorithm.

**Scenario**: Implement and optimize a base64 encoder (CPU-intensive, data-parallel).

**Optimization stages**:
1. Naive Rust implementation
2. Algorithm improvement
3. SIMD vectorization
4. Hand-tuned assembly for hot loop

**Expected speedup**: 5-10x from naive to fully optimized.

### Key Concepts

**Base64 encoding**:
```
Input:  3 bytes (24 bits)
Output: 4 base64 characters

Example:
Binary: 01001101 01100001 01101110  (Man)
Split:  010011 010110 000101 101110
Base64: T      W      F      u
```

**Optimization opportunities**:
- SIMD: Process 12 bytes → 16 base64 chars at once
- Lookup table: Fast character mapping
- Loop unrolling: Reduce branch overhead

### Architecture

**Implementation stages**:

```rust
// Stage 1: Naive scalar
pub fn base64_encode_naive(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut output = Vec::new();

    for chunk in input.chunks(3) {
        let b1 = chunk.get(0).copied().unwrap_or(0);
        let b2 = chunk.get(1).copied().unwrap_or(0);
        let b3 = chunk.get(2).copied().unwrap_or(0);

        let n = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);

        output.push(TABLE[((n >> 18) & 0x3F) as usize]);
        output.push(TABLE[((n >> 12) & 0x3F) as usize]);
        output.push(if chunk.len() > 1 { TABLE[((n >> 6) & 0x3F) as usize] } else { b'=' });
        output.push(if chunk.len() > 2 { TABLE[(n & 0x3F) as usize] } else { b'=' });
    }

    String::from_utf8(output).unwrap()
}

// Stage 2: Algorithmic improvement (process 4 bytes at a time)
pub fn base64_encode_optimized(input: &[u8]) -> String {
    // TODO: Reduce branching, optimize memory access
    todo!()
}

// Stage 3: SIMD vectorization
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "ssse3")]
pub unsafe fn base64_encode_simd(input: &[u8]) -> String {
    // TODO: Use SSSE3 pshufb for parallel shuffling
    // Process 12 bytes -> 16 base64 chars per iteration
    todo!()
}

// Stage 4: Hand-tuned assembly
#[cfg(target_arch = "x86_64")]
pub fn base64_encode_asm(input: &[u8]) -> String {
    // TODO: Critical loop in inline assembly
    todo!()
}

// Decoder for validation
pub fn base64_decode(input: &str) -> Vec<u8> {
    // TODO: Implement to verify correctness
    todo!()
}
```

### Starter Code

```rust
const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode_naive(input: &[u8]) -> String {
    let mut output = Vec::with_capacity((input.len() + 2) / 3 * 4);

    for chunk in input.chunks(3) {
        let b1 = chunk.get(0).copied().unwrap_or(0);
        let b2 = chunk.get(1).copied().unwrap_or(0);
        let b3 = chunk.get(2).copied().unwrap_or(0);

        let n = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);

        output.push(TABLE[((n >> 18) & 0x3F) as usize]);
        output.push(TABLE[((n >> 12) & 0x3F) as usize]);
        output.push(if chunk.len() > 1 { TABLE[((n >> 6) & 0x3F) as usize] } else { b'=' });
        output.push(if chunk.len() > 2 { TABLE[(n & 0x3F) as usize] } else { b'=' });
    }

    String::from_utf8(output).unwrap()
}

// TODO: Implement optimized versions

fn main() {
    let data = b"Hello, World! This is a test of base64 encoding.";

    let encoded = base64_encode_naive(data);
    println!("Encoded: {}", encoded);

    // Benchmark
    use std::time::Instant;

    let iterations = 100_000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = base64_encode_naive(data);
    }
    let duration = start.elapsed();
    println!("Naive: {:?} per iteration", duration / iterations);
}
```

**Profiling**:
```bash
# Linux perf
cargo build --release
perf record --call-graph dwarf ./target/release/milestone_6
perf report

# Flamegraph
cargo install flamegraph
cargo flamegraph --bin milestone_6

# Criterion benchmark
cargo bench
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_encoding() {
        assert_eq!(base64_encode_naive(b"Man"), "TWFu");
        assert_eq!(base64_encode_naive(b"M"), "TQ==");
        assert_eq!(base64_encode_naive(b"Ma"), "TWE=");
    }

    #[test]
    fn test_all_implementations_match() {
        let data = b"The quick brown fox jumps over the lazy dog.";

        let naive = base64_encode_naive(data);
        let optimized = base64_encode_optimized(data);

        assert_eq!(naive, optimized);

        #[cfg(target_arch = "x86_64")]
        unsafe {
            let simd = base64_encode_simd(data);
            assert_eq!(naive, simd);
        }
    }

    #[test]
    fn test_round_trip() {
        let data = b"Round trip test with various characters: !@#$%^&*()";
        let encoded = base64_encode_naive(data);
        let decoded = base64_decode(&encoded);

        assert_eq!(data.to_vec(), decoded);
    }
}
```

**Benchmark setup** (`benches/base64_bench.rs`):
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_base64(c: &mut Criterion) {
    let data = vec![0u8; 1024]; // 1KB

    c.bench_function("naive_1kb", |b| {
        b.iter(|| base64_encode_naive(black_box(&data)))
    });

    c.bench_function("optimized_1kb", |b| {
        b.iter(|| base64_encode_optimized(black_box(&data)))
    });

    #[cfg(target_arch = "x86_64")]
    c.bench_function("simd_1kb", |b| {
        b.iter(|| unsafe { base64_encode_simd(black_box(&data)) })
    });
}

criterion_group!(benches, benchmark_base64);
criterion_main!(benches);
```

### Check Your Understanding

- **What is the bottleneck in the naive version?**
- **How much speedup did SIMD provide?**
- **What CPU instructions appear in the hot path?**
- **How do cache misses affect performance?**
- **Is the hand-tuned assembly faster than SIMD intrinsics?**

### Performance Analysis

**Expected results**:
```
Naive (scalar):         ~800 MB/s
Optimized (algorithm):  ~1.2 GB/s (1.5x)
SIMD (SSE/AVX2):        ~4 GB/s (5x)
Hand-tuned (ASM):       ~5 GB/s (6x)

Real-world libraries:
- base64 crate (SIMD):  ~6-8 GB/s
- Hardware (AES-NI):    N/A (base64 not hardware accelerated)
```

---

## Complete Working Example

```rust
// Minimal working milestone 1-3 example
#![feature(asm_const)]
use std::arch::asm;

// Milestone 1: Assembly inspection
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Milestone 2: Zero-cost abstractions
fn sum_iterator(v: &[i32]) -> i32 {
    v.iter().sum()
}

fn sum_manual(v: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in v {
        sum += x;
    }
    sum
}

// Milestone 3: Inline assembly
fn rdtsc() -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!(
            "rdtsc",
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags),
        );
    }
    ((high as u64) << 32) | (low as u64)
}

fn main() {
    println!("add(5, 3) = {}", add(5, 3));

    let v = vec![1, 2, 3, 4, 5];
    println!("sum_iterator: {}", sum_iterator(&v));
    println!("sum_manual: {}", sum_manual(&v));

    let t1 = rdtsc();
    let _ = add(10, 20);
    let t2 = rdtsc();
    println!("add() took {} cycles", t2 - t1);
}
```

---

## Summary

**What You Built**: A complete understanding of Rust's compilation pipeline, zero-cost abstractions, inline assembly, SIMD optimization, and system-level programming.

**Key Concepts Mastered**:
- **Rust → Assembly compilation**: Understand what the compiler generates
- **Zero-cost abstractions**: Iterators are free (after optimization)
- **Inline assembly**: Access CPU features not exposed by Rust
- **SIMD**: Process multiple values in parallel (4-8x speedup)
- **Calling conventions**: Understand function calls at assembly level
- **Performance optimization**: Profile → optimize → verify

**Performance Journey**:
- **Baseline** (naive): 100 ms
- **Algorithm improvement**: 50 ms (2x faster)
- **SIMD vectorization**: 12 ms (8x faster)
- **Hand-tuned assembly**: 10 ms (10x faster)

**When to Use Each Technique**:
- **99% of code**: Trust the compiler, use high-level Rust
- **0.9% of code**: SIMD intrinsics for data-parallel operations
- **0.1% of code**: Inline assembly for CPU-specific features
- **Never**: Premature optimization without profiling

**Real-World Applications**: Cryptography, compression, image processing, databases, game engines, scientific computing.
