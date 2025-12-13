# Matrix Multiplication

### Problem Statement

Build a production-grade matrix multiplication library, implementing progressively optimized algorithms from naive O(n³) to GPU-accelerated compute shaders. The implementations must handle matrices of sizes from 64×64 to 4096×4096, achieving performance within 2-5x of BLAS libraries through cache optimization, parallelization, SIMD vectorization, and GPU acceleration.

The system must:
- Multiply dense matrices: C = A × B where A is m×k, B is k×n, C is m×n
- Support both square and rectangular matrices
- Handle different data types (f32, f64)
- Provide accurate results (numerical stability)
- Scale from single-core to multi-core to GPU
- Achieve 100+ GFLOPS on modern hardware

### Use Cases

- **Deep Learning**: Neural network forward/backward passes (80% of training time)
- **Scientific Computing**: Physics simulations, fluid dynamics, climate models
- **Computer Graphics**: 3D transformations, rendering pipelines
- **Signal Processing**: Convolution, filtering, spectral analysis
- **Computer Vision**: Image transformations, feature extraction
- **Recommendation Systems**: Collaborative filtering, matrix factorization

---

## Core Concepts in Performance Optimization

Before diving into the implementation, let's understand the fundamental concepts that enable high-performance matrix multiplication. These concepts progressively build upon each other to achieve orders of magnitude speedup.

### Memory Hierarchy and Cache Optimization

**The Memory Wall:**
Modern CPUs can execute billions of operations per second, but memory access is the bottleneck. The memory hierarchy exists to bridge this gap:

```
CPU Registers:   ~1 cycle,    32 KB    (fastest)
L1 Cache:       ~4 cycles,    32 KB
L2 Cache:      ~12 cycles,   256 KB
L3 Cache:      ~40 cycles,  8-32 MB
RAM:          ~200 cycles, 16-64 GB
SSD/Disk:  ~100,000+ cycles          (slowest)
```

**Why This Matters:**
A cache miss (accessing RAM instead of L1) is **50x slower** than a cache hit. For matrix multiplication:
- Naive algorithm: 99% cache misses on large matrices
- Optimized algorithm: 95% cache hits
- **Result: 40x speedup from cache optimization alone**

**Cache Lines and Spatial Locality:**
Data moves between cache and RAM in 64-byte chunks called **cache lines**. Sequential memory access loads entire cache lines efficiently, while random access wastes bandwidth.

```rust
// Good: Sequential access (spatial locality)
for i in 0..n {
    sum += array[i];  // Loads 16 i32s per cache line
}

// Bad: Strided access (poor locality)
for i in (0..n).step_by(1000) {
    sum += array[i];  // Each access likely a cache miss
}
```

**Blocking/Tiling:**
Divide matrices into small **tiles** that fit in L1/L2 cache. Process entire tiles before moving to the next, maximizing cache reuse:

```
Instead of:  Compute full result row-by-row (thrashes cache)
Do:          Compute 64×64 tile, reusing data in cache
Effect:      10-50x speedup
```

### Parallelism: Multi-Core and GPU

**Amdahl's Law:**
If 95% of your program can be parallelized, the theoretical speedup with N cores is:

```
Speedup = 1 / (0.05 + 0.95/N)

1 core:   1.0x
4 cores:  3.5x
8 cores:  5.9x
16 cores: 9.1x
```

**Why Matrix Multiplication is Embarrassingly Parallel:**
Each output element `C[i][j]` can be computed independently:

```rust
// Each C[i][j] = dot(A[i,:], B[:,j])
// No dependencies between elements!

// Sequential
for i in 0..m {
    for j in 0..n {
        C[i][j] = dot(A[i], B[j])
    }
}

// Parallel: Each thread computes different rows
thread 0: computes C[0..m/4]
thread 1: computes C[m/4..m/2]
thread 2: computes C[m/2..3m/4]
thread 3: computes C[3m/4..m]
```

**GPU Architecture:**
- **CPUs**: 8-16 powerful cores, complex control flow
- **GPUs**: 1000-10000 simple cores, optimized for data parallelism
- **Memory bandwidth**: GPU has 500+ GB/s vs CPU's 50 GB/s
- **Result**: 10-50x speedup for large matrices

**Thread Synchronization:**
Critical for parallel algorithms:
- **Data races**: Multiple threads writing to same location
- **Cache coherence**: Keeping caches consistent across cores
- **False sharing**: Threads modifying adjacent cache lines

```rust
// Safe parallelism in Rust
result.par_chunks_mut(n_cols).for_each(|row| {
    // Each thread owns disjoint memory
    // No synchronization needed!
});
```

### SIMD: Single Instruction Multiple Data

**Vector Instructions:**
Modern CPUs have special registers that hold multiple values:

```
Scalar:  a * b             (1 operation)
SIMD:    [a0,a1,a2,a3] * [b0,b1,b2,b3] = [a0*b0, a1*b1, a2*b2, a3*b3]
         (4 operations in parallel!)
```

**SIMD Instruction Sets:**
- **SSE**: 4× f32 or 2× f64 (128-bit registers)
- **AVX2**: 8× f32 or 4× f64 (256-bit registers)
- **AVX-512**: 16× f32 or 8× f64 (512-bit registers)

**SIMD for Matrix Multiplication:**
Vectorize the inner loop to compute multiple dot product elements simultaneously:

```rust
// Scalar (slow)
for k in 0..n {
    c[i][j] += a[i][k] * b[k][j];
}

// SIMD (fast)
let mut sum = f32x8::splat(0.0);
for k in (0..n).step_by(8) {
    let a_vec = f32x8::load(&a[i][k..]);
    let b_vec = f32x8::load(&b[k][j..]);
    sum += a_vec * b_vec;
}
c[i][j] = sum.horizontal_sum();
```

**Speedup:** 4-8x depending on instruction set

**Alignment:**
SIMD loads are fastest when data is aligned to 16/32-byte boundaries:

```rust
// Aligned load (fast): _mm256_load_ps
// Unaligned load (slower): _mm256_loadu_ps
// Can be 2x difference!
```

### Loop Optimization Techniques

**Loop Interchange:**
Reorder loops to improve cache locality:

```rust
// Bad: Accesses B column-wise (poor locality)
for i in 0..m {
    for j in 0..n {
        for k in 0..p {
            C[i][j] += A[i][k] * B[k][j]  // B[k][j] jumps by n elements
        }
    }
}

// Better: After transposing B
for i in 0..m {
    for j in 0..n {
        for k in 0..p {
            C[i][j] += A[i][k] * B_T[j][k]  // B_T[j][k] sequential
        }
    }
}
```

**Loop Unrolling:**
Reduce loop overhead and enable more instruction-level parallelism:

```rust
// Original
for i in 0..n {
    sum += a[i] * b[i];
}

// Unrolled
for i in (0..n).step_by(4) {
    sum0 += a[i] * b[i];
    sum1 += a[i+1] * b[i+1];
    sum2 += a[i+2] * b[i+2];
    sum3 += a[i+3] * b[i+3];
}
sum = sum0 + sum1 + sum2 + sum3;
```

**Loop Tiling (Blocking):**
Divide iteration space into tiles for cache reuse:

```rust
// Tiled matmul
for ii in (0..m).step_by(TILE) {
    for jj in (0..n).step_by(TILE) {
        for kk in (0..p).step_by(TILE) {
            // Process TILE×TILE sub-matrix
            for i in ii..min(ii+TILE, m) {
                for j in jj..min(jj+TILE, n) {
                    for k in kk..min(kk+TILE, p) {
                        C[i][j] += A[i][k] * B[k][j]
                    }
                }
            }
        }
    }
}
```

### Data Layout and Access Patterns

**Row-Major vs Column-Major:**

```
Matrix:     Row-Major Storage:      Column-Major Storage:
[1 2 3]     [1, 2, 3, 4, 5, 6]     [1, 4, 2, 5, 3, 6]
[4 5 6]
```

- **C/Rust**: Row-major by default
- **Fortran/MATLAB**: Column-major
- **Access pattern matters**: Row-wise access in row-major is fast, column-wise is slow

**Structure of Arrays (SoA) vs Array of Structures (AoS):**

```rust
// AoS (bad for SIMD)
struct Point { x: f32, y: f32, z: f32 }
let points: Vec<Point> = ...;

// SoA (good for SIMD)
struct Points {
    x: Vec<f32>,  // All x values contiguous
    y: Vec<f32>,
    z: Vec<f32>,
}
```

**Prefetching:**
Hint to CPU to load data before it's needed:

```rust
unsafe {
    _mm_prefetch(ptr.add(64) as *const i8, _MM_HINT_T0);
}
// Loads cache line at ptr+64 into L1 cache
// Hides memory latency when used correctly
```

### Performance Measurement

**FLOPS (Floating Point Operations Per Second):**
For matrix multiply C = A×B where A is m×k, B is k×n:
- Operations: `2×m×n×k` (multiply + add for each element)
- GFLOPS = Operations / (time_seconds × 10^9)

**Roofline Model:**
Determines performance ceiling based on:
- **Compute bound**: Limited by FLOPs (ALU throughput)
- **Memory bound**: Limited by bandwidth
- **Formula**: `Performance = min(Peak_FLOPS, Bandwidth × Arithmetic_Intensity)`

**Benchmarking Best Practices:**
```rust
// Warm-up
for _ in 0..10 { f(); }

// Measure
let start = Instant::now();
for _ in 0..100 {
    f();
    black_box(&result);  // Prevent optimization
}
let avg = start.elapsed() / 100;
```

---

## Connection to This Project

This project progressively applies all these concepts to matrix multiplication, demonstrating **compound optimization** where techniques combine multiplicatively:

### Milestone 1: Naive Implementation (Baseline)
- **Goal**: Establish correctness and baseline performance
- **Concepts**: Basic O(n³) algorithm, row-major layout
- **Performance**: 0.1-0.5 GFLOPS (~0.1% of peak)
- **Bottleneck**: Poor cache locality (99% miss rate), no optimization

### Milestone 2: Cache-Optimized Tiling
- **Goal**: Eliminate cache misses through blocking
- **Concepts Applied**:
  - Cache hierarchy understanding
  - Blocking/tiling to fit in L1/L2
  - Spatial locality optimization
- **Performance**: 5-10 GFLOPS (10-50x faster)
- **Why It Works**: 95% cache hit rate reduces memory latency by 40x
- **Trade-off**: More complex code, but worth it for 10x+ speedup

### Milestone 3: Parallel Multi-Core
- **Goal**: Utilize all CPU cores
- **Concepts Applied**:
  - Amdahl's Law and embarrassing parallelism
  - Rayon for work-stealing parallelism
  - Cache coherence considerations
- **Performance**: 40-80 GFLOPS (4-8x over tiled)
- **Why It Works**: Matrix multiplication has zero dependencies between output elements
- **Trade-off**: Thread overhead (~10µs per spawn), so only beneficial for large matrices

### Milestone 4: SIMD Vectorization
- **Goal**: Process 4-8 elements per instruction
- **Concepts Applied**:
  - Vector instructions (AVX2/AVX-512)
  - Data alignment for optimal SIMD performance
  - Horizontal reduction for accumulation
- **Performance**: 100-200 GFLOPS (2-4x over parallel)
- **Why It Works**: CPU has dedicated SIMD ALUs, unlocking 4-8x more compute
- **Trade-off**: Complex intrinsics, platform-specific code

### Milestone 5: Combined Optimization
- **Goal**: Apply ALL techniques together
- **Concepts Applied**:
  - Hierarchical tiling (L1, L2, L3 caches)
  - Parallel + SIMD (nested parallelism)
  - Micro-kernels optimized for register reuse
  - Loop unrolling and prefetching
- **Performance**: 150-250 GFLOPS (1500-2500x over naive!)
- **Why It Works**: Multiplicative effect: 10x (cache) × 8x (cores) × 4x (SIMD) = 320x theoretical
- **Reality**: 1500x achieved due to overhead and non-perfect scaling

### Milestone 6: GPU Acceleration
- **Goal**: Leverage massively parallel GPU architecture
- **Concepts Applied**:
  - GPGPU programming (WebGPU/wgpu)
  - Shared memory tiling on GPU
  - Workgroup cooperation
  - PCIe transfer overhead management
- **Performance**: 1000-5000 GFLOPS (10-25x over optimized CPU)
- **Why It Works**:
  - 1000s of cores vs 8-16 on CPU
  - 500+ GB/s bandwidth vs 50 GB/s
  - Specialized hardware for parallel workloads
- **Trade-offs**:
  - Data transfer overhead (can dominate for small matrices)
  - Complex programming model
  - Debugging difficulty

### Performance Journey Summary

| Milestone | Technique | GFLOPS | Speedup | Cumulative |
|-----------|-----------|--------|---------|------------|
| 1. Naive | None | 0.1 | 1x | 1x |
| 2. Tiled | Cache blocking | 5 | 50x | 50x |
| 3. Parallel | Multi-core | 40 | 8x | 400x |
| 4. SIMD | Vectorization | 120 | 3x | 1,200x |
| 5. Combined | All CPU opts | 200 | 1.7x | 2,000x |
| 6. GPU | Massively parallel | 3000 | 15x | 30,000x |

**Key Insight**: Optimizations compound! Going from naive (0.1 GFLOPS) to GPU (3000 GFLOPS) represents a **30,000x speedup** through progressive optimization.

### Real-World Impact

For a 2048×2048 matrix multiply:
- **Operations**: 2 × 2048³ ≈ 17 billion FLOPs
- **Naive**: 17s (0.1 GFLOPS)
- **Optimized CPU**: 85ms (200 GFLOPS)
- **GPU**: 5.7ms (3000 GFLOPS)

**Scaling to Deep Learning:**
- GPT-3 training: 3.14 × 10²³ FLOPS
- At naive speed: 99,563 years
- At GPU speed: 33 years (still need 10,000 GPUs!)

This is why **performance optimization matters** in real systems.

---

### Why It Matters

**Performance Impact:**
- Naive implementation: ~0.1 GFLOPS (billion floating-point operations per second)
- Cache-optimized: ~5-10 GFLOPS (50-100x speedup)
- Parallel: ~40-80 GFLOPS (400-800x speedup on 8 cores)
- SIMD + Parallel: ~100-200 GFLOPS (1000-2000x speedup)
- GPU: ~1000-5000 GFLOPS (10,000-50,000x speedup on modern GPU)

**Real-World Scale:**
- Training GPT-3: Performs 3.14 × 10²³ FLOPS (314 zettaFLOPS)
- 1ms improvement per matrix multiply × 1 billion operations = 11.5 days saved
- Cloud cost: $1.00/hr GPU, 10,000x speedup = $9,999 saved per 10,000 hours

**Why Matrix Multiplication Matters:**
Matrix multiplication is the fundamental operation in:
- Linear algebra (80% of NumPy/SciPy operations)
- Machine learning (transformers, CNNs, RNNs all use matmul)
- Graphics (every 3D transformation is a matrix multiply)
- Data science (PCA, SVD, recommendation systems)

**Memory Hierarchy Impact:**
```
CPU Register:     ~1 cycle,    32 KB
L1 Cache:        ~4 cycles,    32 KB
L2 Cache:       ~12 cycles,   256 KB
L3 Cache:       ~40 cycles,  8-32 MB
RAM:           ~200 cycles,  16-64 GB
GPU Memory:   ~400 cycles,  8-24 GB
```

Naive algorithm: 99% cache misses → 200 cycles per load
Optimized: 95% cache hits → 5 cycles per load = **40x speedup**

---

## Milestone 1: Naive Matrix Multiplication

### Introduction

Implement the textbook O(n³) matrix multiplication algorithm. This establishes correctness and provides a baseline for measuring optimizations.

For matrices A (m×k) and B (k×n), compute C (m×n) where:
```
C[i][j] = Σ(A[i][k] × B[k][j]) for k = 0..k
```

This is the simplest implementation: three nested loops, no optimizations. Expected performance: ~0.1-0.5 GFLOPS.

### Architecture

**Structs:**
- `Matrix<T>` - Dense matrix representation
  - **Field** `data: Vec<T>` - Flattened row-major storage
  - **Field** `rows: usize` - Number of rows (m)
  - **Field** `cols: usize` - Number of columns (n)
  - **Function** `new(rows: usize, cols: usize) -> Self` - Create zero matrix
  - **Function** `from_vec(data: Vec<T>, rows: usize, cols: usize) -> Self` - Create from data
  - **Function** `get(&self, i: usize, j: usize) -> &T` - Access element
  - **Function** `get_mut(&mut self, i: usize, j: usize) -> &mut T` - Mutable access
  - **Function** `set(&mut self, i: usize, j: usize, value: T)` - Set element

**Key Functions:**
- `naive_matmul(a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32>` - Basic multiplication
- `check_dimensions(a: &Matrix<f32>, b: &Matrix<f32>)` - Validate dimensions
- `transpose(m: &Matrix<f32>) -> Matrix<f32>` - Transpose matrix

**Role Each Plays:**
- Row-major layout: `matrix[i][j]` stored at `data[i * cols + j]`
- Three nested loops: i (rows of A), j (cols of B), k (inner dimension)
- Dot product: Each C[i][j] is dot product of row i of A and column j of B

**Memory Layout:**
```
Matrix A (2×3):     Matrix B (3×2):
[1 2 3]             [7  8]
[4 5 6]             [9  10]
                    [11 12]

Stored as: [1,2,3,4,5,6]  Stored as: [7,8,9,10,11,12]
```

### Checkpoint Tests

```rust
#[test]
fn test_matrix_creation() {
    let m = Matrix::new(3, 4);
    assert_eq!(m.rows, 3);
    assert_eq!(m.cols, 4);
    assert_eq!(m.data.len(), 12);
}

#[test]
fn test_matrix_indexing() {
    let mut m = Matrix::new(2, 2);
    m.set(0, 0, 1.0);
    m.set(0, 1, 2.0);
    m.set(1, 0, 3.0);
    m.set(1, 1, 4.0);

    assert_eq!(*m.get(0, 0), 1.0);
    assert_eq!(*m.get(1, 1), 4.0);
}

#[test]
fn test_naive_matmul_small() {
    // 2×2 matrices
    let a = Matrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let b = Matrix::from_vec(vec![5.0, 6.0, 7.0, 8.0], 2, 2);

    let c = naive_matmul(&a, &b);

    // Expected: [19, 22]
    //           [43, 50]
    assert_eq!(*c.get(0, 0), 19.0);
    assert_eq!(*c.get(0, 1), 22.0);
    assert_eq!(*c.get(1, 0), 43.0);
    assert_eq!(*c.get(1, 1), 50.0);
}

#[test]
fn test_naive_matmul_rectangular() {
    // A: 2×3, B: 3×2
    let a = Matrix::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 2, 3);
    let b = Matrix::from_vec(vec![7.0, 8.0, 9.0, 10.0, 11.0, 12.0], 3, 2);

    let c = naive_matmul(&a, &b);

    assert_eq!(c.rows, 2);
    assert_eq!(c.cols, 2);

    // C[0][0] = 1*7 + 2*9 + 3*11 = 7 + 18 + 33 = 58
    assert_eq!(*c.get(0, 0), 58.0);
}

#[test]
fn test_identity_multiply() {
    let a = Matrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let identity = Matrix::from_vec(vec![1.0, 0.0, 0.0, 1.0], 2, 2);

    let c = naive_matmul(&a, &identity);

    // A * I = A
    assert_eq!(*c.get(0, 0), 1.0);
    assert_eq!(*c.get(0, 1), 2.0);
    assert_eq!(*c.get(1, 0), 3.0);
    assert_eq!(*c.get(1, 1), 4.0);
}

#[test]
#[should_panic]
fn test_dimension_mismatch() {
    let a = Matrix::new(2, 3);
    let b = Matrix::new(4, 2); // Mismatch: a.cols != b.rows

    naive_matmul(&a, &b);
}
```

### Starter Code

```rust
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct Matrix<T> {
    data: Vec<T>,
    rows: usize,
    cols: usize,
}

impl<T: Default + Clone> Matrix<T> {
    pub fn new(rows: usize, cols: usize) -> Self {
        // TODO: Create matrix filled with default values
        // Self {
        //     data: vec![T::default(); rows * cols],
        //     rows,
        //     cols,
        // }
        todo!()
    }

    pub fn from_vec(data: Vec<T>, rows: usize, cols: usize) -> Self {
        // TODO: Validate data.len() == rows * cols
        // assert_eq!(data.len(), rows * cols);
        // Self { data, rows, cols }
        todo!()
    }

    pub fn get(&self, i: usize, j: usize) -> &T {
        // TODO: Convert 2D index to 1D
        // &self.data[i * self.cols + j]
        todo!()
    }

    pub fn get_mut(&mut self, i: usize, j: usize) -> &mut T {
        // TODO: Mutable version
        // &mut self.data[i * self.cols + j]
        todo!()
    }

    pub fn set(&mut self, i: usize, j: usize, value: T) {
        // TODO: Set element
        // self.data[i * self.cols + j] = value;
        todo!()
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }
}

pub fn naive_matmul(a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32> {
    // TODO: Implement naive matrix multiplication
    //
    // 1. Check dimensions: a.cols must equal b.rows
    // 2. Create result matrix: rows=a.rows, cols=b.cols
    // 3. Triple nested loop:
    //    for i in 0..a.rows {
    //        for j in 0..b.cols {
    //            for k in 0..a.cols {
    //                result[i][j] += a[i][k] * b[k][j]
    //            }
    //        }
    //    }
    //
    // assert_eq!(a.cols, b.rows, "Dimension mismatch");
    //
    // let mut result = Matrix::new(a.rows, b.cols);
    //
    // for i in 0..a.rows {
    //     for j in 0..b.cols {
    //         let mut sum = 0.0;
    //         for k in 0..a.cols {
    //             sum += a.get(i, k) * b.get(k, j);
    //         }
    //         result.set(i, j, sum);
    //     }
    // }
    //
    // result
    todo!()
}

pub fn benchmark_naive(size: usize) -> (f64, f64) {
    use std::time::Instant;

    // TODO: Create random matrices and benchmark
    // 1. Create two size×size matrices with random data
    // 2. Measure time for multiplication
    // 3. Calculate GFLOPS: (2 * n³) / (time_in_seconds * 1e9)
    //
    // GFLOPS = (2 * size³) floating point operations per second
    todo!()
}
```

---

## Milestone 2: Cache-Optimized Matrix Multiplication (Tiling/Blocking)

### Introduction

**Why Milestone 1 Is Not Enough:**
Naive algorithm has terrible cache performance. For 1000×1000 matrices (4MB each), accessing B[k][j] in the innermost loop causes cache misses because columns are not contiguous in row-major layout.

**Cache Miss Analysis:**
```
Naive access pattern for B:
B[0][j], B[1][j], B[2][j], ... (stride = 1000 elements = 4KB)
Each access likely misses L1 cache (32KB)
```

**What We're Improving:**
Use tiling/blocking to improve cache locality. Process matrix in small tiles that fit in L1/L2 cache. This reduces cache misses from 99% to <10%.

**Blocking Strategy:**
```
Instead of: Multiply entire rows × columns
Do:         Multiply row_tile × column_tile

Split matrices into 64×64 tiles (16KB each, fits in L1)
Process tile-by-tile, keeping data in cache
```

**Expected Speedup:** 10-50x over naive (from 0.1 to 5-10 GFLOPS)

### Architecture

**Constants:**
- `BLOCK_SIZE: usize = 64` - Tile size for cache blocking

**Key Functions:**
- `blocked_matmul(a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32>` - Tiled multiplication
- `multiply_block(...)` - Multiply single tile
- `transpose_b(b: &Matrix<f32>) -> Matrix<f32>` - Pre-transpose B for better locality

**Blocking Algorithm:**
```
for i_block in (0..n).step_by(BLOCK_SIZE) {
    for j_block in (0..n).step_by(BLOCK_SIZE) {
        for k_block in (0..n).step_by(BLOCK_SIZE) {
            // Multiply tiles
            for i in i_block..min(i_block+BLOCK, n) {
                for j in j_block..min(j_block+BLOCK, n) {
                    for k in k_block..min(k_block+BLOCK, n) {
                        c[i][j] += a[i][k] * b[k][j]
                    }
                }
            }
        }
    }
}
```

**Role Each Plays:**
- Blocking: Improve spatial locality
- Tile size: Balance cache capacity and computation
- Loop reordering: Maximize cache hits

### Checkpoint Tests

```rust
#[test]
fn test_blocked_matmul_correctness() {
    let a = Matrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let b = Matrix::from_vec(vec![5.0, 6.0, 7.0, 8.0], 2, 2);

    let naive_result = naive_matmul(&a, &b);
    let blocked_result = blocked_matmul(&a, &b);

    // Results should be identical
    for i in 0..2 {
        for j in 0..2 {
            assert_eq!(*naive_result.get(i, j), *blocked_result.get(i, j));
        }
    }
}

#[test]
fn test_blocked_matmul_large() {
    use rand::Rng;

    let size = 256;
    let mut rng = rand::thread_rng();

    let a_data: Vec<f32> = (0..size * size).map(|_| rng.gen()).collect();
    let b_data: Vec<f32> = (0..size * size).map(|_| rng.gen()).collect();

    let a = Matrix::from_vec(a_data, size, size);
    let b = Matrix::from_vec(b_data, size, size);

    let naive_result = naive_matmul(&a, &b);
    let blocked_result = blocked_matmul(&a, &b);

    // Check results are close (floating point tolerance)
    for i in 0..size {
        for j in 0..size {
            let diff = (*naive_result.get(i, j) - *blocked_result.get(i, j)).abs();
            assert!(diff < 0.01, "Difference too large at ({}, {}): {}", i, j, diff);
        }
    }
}

#[test]
fn test_blocking_performance() {
    use std::time::Instant;

    let size = 512;
    let a = Matrix::from_vec(vec![1.0; size * size], size, size);
    let b = Matrix::from_vec(vec![2.0; size * size], size, size);

    // Naive
    let start = Instant::now();
    let _ = naive_matmul(&a, &b);
    let naive_time = start.elapsed();

    // Blocked
    let start = Instant::now();
    let _ = blocked_matmul(&a, &b);
    let blocked_time = start.elapsed();

    println!("Naive:   {:?}", naive_time);
    println!("Blocked: {:?}", blocked_time);
    println!("Speedup: {:.2}x", naive_time.as_secs_f64() / blocked_time.as_secs_f64());

    // Blocked should be faster
    assert!(blocked_time < naive_time);
}

#[test]
fn test_non_square_blocking() {
    let a = Matrix::from_vec(vec![1.0; 128 * 256], 128, 256);
    let b = Matrix::from_vec(vec![2.0; 256 * 128], 256, 128);

    let result = blocked_matmul(&a, &b);

    assert_eq!(result.rows(), 128);
    assert_eq!(result.cols(), 128);
}
```

### Starter Code

```rust
const BLOCK_SIZE: usize = 64;

pub fn blocked_matmul(a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32> {
    // TODO: Implement blocked/tiled matrix multiplication
    //
    // Algorithm:
    // 1. Divide matrices into BLOCK_SIZE × BLOCK_SIZE tiles
    // 2. For each tile combination:
    //    - Load tile into cache
    //    - Multiply tiles (nested loops)
    //    - Accumulate result
    //
    // Pseudocode:
    // for i_block in (0..a.rows).step_by(BLOCK_SIZE) {
    //     for j_block in (0..b.cols).step_by(BLOCK_SIZE) {
    //         for k_block in (0..a.cols).step_by(BLOCK_SIZE) {
    //             // Process block
    //             let i_end = min(i_block + BLOCK_SIZE, a.rows);
    //             let j_end = min(j_block + BLOCK_SIZE, b.cols);
    //             let k_end = min(k_block + BLOCK_SIZE, a.cols);
    //
    //             for i in i_block..i_end {
    //                 for j in j_block..j_end {
    //                     let mut sum = *result.get(i, j);
    //                     for k in k_block..k_end {
    //                         sum += a.get(i, k) * b.get(k, j);
    //                     }
    //                     result.set(i, j, sum);
    //                 }
    //             }
    //         }
    //     }
    // }
    todo!()
}

// Optional optimization: transpose B for better cache locality
pub fn transpose(m: &Matrix<f32>) -> Matrix<f32> {
    // TODO: Transpose matrix
    // Converts row-major to column-major access
    //
    // let mut result = Matrix::new(m.cols, m.rows);
    // for i in 0..m.rows {
    //     for j in 0..m.cols {
    //         result.set(j, i, *m.get(i, j));
    //     }
    // }
    // result
    todo!()
}

pub fn blocked_matmul_transposed(a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32> {
    // TODO: Multiply using transposed B
    // Transpose B first, then access B[j][k] instead of B[k][j]
    // This makes B access row-major (better cache locality)
    todo!()
}
```

---

## Milestone 3: Parallel Matrix Multiplication

### Introduction

**Why Milestone 2 Is Not Enough:**
Blocked multiplication uses only 1 core. Modern CPUs have 8-16 cores, leaving 87-93% of compute power idle. For large matrices, the outer loops are embarrassingly parallel.

**What We're Improving:**
Parallelize the computation across CPU cores using Rayon. Split rows of result matrix among threads. Each thread computes independent rows, no synchronization needed.

**Parallelization Strategy:**
```
Result matrix C (m×n):
Thread 0: Computes rows 0..m/4
Thread 1: Computes rows m/4..m/2
Thread 2: Computes rows m/2..3m/4
Thread 3: Computes rows 3m/4..m
```

**Expected Speedup:** 4-8x on 8-core machine (total 40-80 GFLOPS)

### Architecture

**Dependencies:**
```toml
[dependencies]
rayon = "1.8"
```

**Key Functions:**
- `parallel_matmul(a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32>` - Parallel multiplication
- Use Rayon's `par_chunks_mut()` for parallel row processing

**Parallelization Points:**
1. **Row-level parallelism**: Each thread computes different rows of C
2. **Work stealing**: Rayon automatically balances load
3. **No synchronization**: Each thread writes to separate memory locations

**Role Each Plays:**
- Rayon: Thread pool and work distribution
- par_chunks_mut: Split result into parallel chunks
- Read-only sharing: A and B are shared read-only (safe)

### Checkpoint Tests

```rust
#[test]
fn test_parallel_matmul_correctness() {
    let a = Matrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let b = Matrix::from_vec(vec![5.0, 6.0, 7.0, 8.0], 2, 2);

    let blocked = blocked_matmul(&a, &b);
    let parallel = parallel_matmul(&a, &b);

    for i in 0..2 {
        for j in 0..2 {
            assert_eq!(*blocked.get(i, j), *parallel.get(i, j));
        }
    }
}

#[test]
fn test_parallel_speedup() {
    use std::time::Instant;

    let size = 1024;
    let a = Matrix::from_vec(vec![1.0; size * size], size, size);
    let b = Matrix::from_vec(vec![2.0; size * size], size, size);

    // Sequential
    let start = Instant::now();
    let _ = blocked_matmul(&a, &b);
    let seq_time = start.elapsed();

    // Parallel
    let start = Instant::now();
    let _ = parallel_matmul(&a, &b);
    let par_time = start.elapsed();

    println!("Sequential: {:?}", seq_time);
    println!("Parallel:   {:?}", par_time);
    println!("Speedup:    {:.2}x", seq_time.as_secs_f64() / par_time.as_secs_f64());

    assert!(par_time < seq_time);
}

#[test]
fn test_parallel_large_matrix() {
    use rand::Rng;

    let size = 512;
    let mut rng = rand::thread_rng();

    let a_data: Vec<f32> = (0..size * size).map(|_| rng.gen()).collect();
    let b_data: Vec<f32> = (0..size * size).map(|_| rng.gen()).collect();

    let a = Matrix::from_vec(a_data, size, size);
    let b = Matrix::from_vec(b_data, size, size);

    let result = parallel_matmul(&a, &b);

    // Just check it completes without errors
    assert_eq!(result.rows(), size);
    assert_eq!(result.cols(), size);
}

#[test]
fn test_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    let a = Arc::new(Matrix::from_vec(vec![1.0; 256 * 256], 256, 256));
    let b = Arc::new(Matrix::from_vec(vec![2.0; 256 * 256], 256, 256));

    // Multiple threads can share matrices safely
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let a = Arc::clone(&a);
            let b = Arc::clone(&b);
            thread::spawn(move || {
                parallel_matmul(&a, &b)
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}
```

### Starter Code

```rust
use rayon::prelude::*;

pub fn parallel_matmul(a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32> {
    // TODO: Implement parallel matrix multiplication
    //
    // Strategy:
    // 1. Create result matrix
    // 2. Split result into row chunks
    // 3. Process each chunk in parallel
    // 4. Each thread computes its assigned rows
    //
    // use rayon::prelude::*;
    //
    // assert_eq!(a.cols, b.rows);
    //
    // let mut result = Matrix::new(a.rows, b.cols);
    //
    // // Process rows in parallel
    // result.data
    //     .par_chunks_mut(b.cols) // Each chunk is one row
    //     .enumerate()
    //     .for_each(|(i, row_chunk)| {
    //         for j in 0..b.cols {
    //             let mut sum = 0.0;
    //             for k in 0..a.cols {
    //                 sum += a.get(i, k) * b.get(k, j);
    //             }
    //             row_chunk[j] = sum;
    //         }
    //     });
    //
    // result
    todo!()
}

pub fn parallel_blocked_matmul(a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32> {
    // TODO: Combine blocking and parallelism
    // Parallelize the outer block loop
    //
    // Benefits:
    // - Cache optimization from blocking
    // - Multi-core utilization from parallelism
    // - Best of both worlds
    todo!()
}

pub fn benchmark_parallel(size: usize) -> (f64, f64) {
    use std::time::Instant;

    let a = Matrix::from_vec(vec![1.0; size * size], size, size);
    let b = Matrix::from_vec(vec![2.0; size * size], size, size);

    let start = Instant::now();
    let _ = parallel_matmul(&a, &b);
    let elapsed = start.elapsed();

    let flops = 2.0 * (size as f64).powi(3);
    let gflops = flops / (elapsed.as_secs_f64() * 1e9);

    (elapsed.as_secs_f64(), gflops)
}
```

---

## Milestone 4: SIMD Vectorization

### Introduction

**Why Milestone 3 Is Not Enough:**
Modern CPUs can perform 4-8 floating-point operations per instruction using SIMD (Single Instruction Multiple Data). AVX2 processes 8×f32 per instruction, AVX-512 does 16×f32. Without SIMD, we're using only 12.5% of CPU compute capability.

**What We're Improving:**
Use explicit SIMD instructions to vectorize the inner loop. Instead of processing one element at a time, process 4-8 elements simultaneously.

**SIMD Concept:**
```
Scalar:  a[i] * b[i]  (1 operation)
SIMD:    [a0,a1,a2,a3] * [b0,b1,b2,b3] = [a0*b0, a1*b1, a2*b2, a3*b3]
         (4 operations in parallel)
```

**Expected Speedup:** 2-4x over parallel (total 100-200 GFLOPS)

### Architecture

**Dependencies:**
```toml
# Use portable_simd (nightly) or packed_simd
[dependencies]
packed_simd = "0.3"
# Or use std::simd on nightly Rust
```

**Key Types:**
- `f32x4` / `f32x8` - SIMD vector of 4 or 8 floats
- `*mut f32x4` - Pointer to aligned SIMD data

**Key Functions:**
- `simd_matmul(a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32>` - SIMD-optimized
- `aligned_matrix(rows: usize, cols: usize) -> Matrix<f32>` - Ensure alignment
- `simd_dot_product(a: &[f32], b: &[f32]) -> f32` - Vectorized dot product

**SIMD Algorithm:**
```rust
// Process 4 elements at a time
for i in 0..rows {
    for j in (0..cols).step_by(4) {
        let mut sum = f32x4::splat(0.0);
        for k in 0..inner {
            let a_val = f32x4::splat(a[i][k]);
            let b_vec = f32x4::from_slice_unaligned(&b[k][j..j+4]);
            sum += a_val * b_vec;
        }
        sum.write_to_slice_unaligned(&mut result[i][j..j+4]);
    }
}
```

**Role Each Plays:**
- SIMD registers: Hold multiple values
- Vector operations: Parallel arithmetic
- Alignment: Performance critical for SIMD loads
- Horizontal sum: Reduce vector to scalar

### Checkpoint Tests

```rust
#[test]
fn test_simd_matmul_correctness() {
    let a = Matrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
    let b = Matrix::from_vec(vec![5.0, 6.0, 7.0, 8.0], 2, 2);

    let parallel = parallel_matmul(&a, &b);
    let simd = simd_matmul(&a, &b);

    for i in 0..2 {
        for j in 0..2 {
            let diff = (*parallel.get(i, j) - *simd.get(i, j)).abs();
            assert!(diff < 1e-5);
        }
    }
}

#[test]
fn test_simd_alignment() {
    use std::mem;

    let size = 256;
    let matrix = aligned_matrix(size, size);

    // Check alignment
    let ptr = matrix.data.as_ptr() as usize;
    assert_eq!(ptr % mem::align_of::<f32x8>(), 0);
}

#[test]
fn test_simd_dot_product() {
    let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let b = vec![8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];

    let result = simd_dot_product(&a, &b);

    // Manual: 1*8 + 2*7 + 3*6 + 4*5 + 5*4 + 6*3 + 7*2 + 8*1
    //       = 8 + 14 + 18 + 20 + 20 + 18 + 14 + 8 = 120
    assert!((result - 120.0).abs() < 1e-5);
}

#[test]
fn test_simd_performance() {
    use std::time::Instant;

    let size = 512;
    let a = Matrix::from_vec(vec![1.0; size * size], size, size);
    let b = Matrix::from_vec(vec![2.0; size * size], size, size);

    // Parallel
    let start = Instant::now();
    let _ = parallel_matmul(&a, &b);
    let par_time = start.elapsed();

    // SIMD
    let start = Instant::now();
    let _ = simd_matmul(&a, &b);
    let simd_time = start.elapsed();

    println!("Parallel: {:?}", par_time);
    println!("SIMD:     {:?}", simd_time);
    println!("Speedup:  {:.2}x", par_time.as_secs_f64() / simd_time.as_secs_f64());

    assert!(simd_time < par_time);
}

#[test]
fn test_simd_large_matrix() {
    let size = 1024;
    let a = aligned_matrix(size, size);
    let b = aligned_matrix(size, size);

    let result = simd_matmul(&a, &b);

    assert_eq!(result.rows(), size);
    assert_eq!(result.cols(), size);
}
```

### Starter Code

```rust
use std::arch::x86_64::*; // For intrinsics
// Or use portable_simd:
// use packed_simd::*;

const SIMD_WIDTH: usize = 8; // AVX2: 8 f32s

pub fn simd_dot_product(a: &[f32], b: &[f32]) -> f32 {
    // TODO: Implement SIMD dot product
    //
    // Using AVX2 intrinsics:
    // unsafe {
    //     let mut sum = _mm256_setzero_ps();
    //
    //     let chunks = a.len() / 8;
    //     for i in 0..chunks {
    //         let a_vec = _mm256_loadu_ps(a.as_ptr().add(i * 8));
    //         let b_vec = _mm256_loadu_ps(b.as_ptr().add(i * 8));
    //         let prod = _mm256_mul_ps(a_vec, b_vec);
    //         sum = _mm256_add_ps(sum, prod);
    //     }
    //
    //     // Horizontal sum
    //     let mut result = [0.0f32; 8];
    //     _mm256_storeu_ps(result.as_mut_ptr(), sum);
    //     result.iter().sum()
    // }
    //
    // Or using portable_simd:
    // use packed_simd::f32x8;
    //
    // let mut sum = f32x8::splat(0.0);
    // for i in (0..a.len()).step_by(8) {
    //     let a_vec = f32x8::from_slice_unaligned(&a[i..]);
    //     let b_vec = f32x8::from_slice_unaligned(&b[i..]);
    //     sum += a_vec * b_vec;
    // }
    // sum.sum()
    todo!()
}

pub fn simd_matmul(a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32> {
    // TODO: SIMD matrix multiplication
    //
    // Strategy:
    // 1. For each element of result:
    //    - Compute dot product using SIMD
    // 2. Process multiple columns simultaneously
    //
    // Optimization: Transpose B and use row-row dot products
    // (better for SIMD - both rows are contiguous)
    todo!()
}

pub fn aligned_matrix(rows: usize, cols: usize) -> Matrix<f32> {
    // TODO: Create matrix with SIMD-aligned memory
    //
    // Use std::alloc::alloc with alignment
    // Or Vec with capacity padding
    //
    // Alignment is important for _mm256_load_ps (aligned load)
    // vs _mm256_loadu_ps (unaligned load, slower)
    todo!()
}

pub fn simd_parallel_matmul(a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32> {
    // TODO: Combine SIMD and parallelism
    //
    // Use Rayon for parallelism
    // Use SIMD within each thread
    //
    // This gives best of both:
    // - Multi-core parallelism
    // - SIMD vectorization within cores
    todo!()
}
```

---

## Milestone 5: Combined Optimization - Tiling + Parallel + SIMD

### Introduction

**Why Milestone 4 Is Not Enough:**
We've implemented three orthogonal optimizations:
1. Cache blocking (memory hierarchy)
2. Parallelism (multi-core)
3. SIMD (instruction-level parallelism)

But we've only combined some of them. The ultimate performance requires all three working together.

**What We're Improving:**
Create a unified implementation that combines:
- Cache-friendly tiling (reduces memory bandwidth)
- Parallel execution (utilizes all cores)
- SIMD vectorization (maximizes per-core throughput)

**Expected Performance:** 150-250 GFLOPS on modern 8-core CPU with AVX2

### Architecture

**Optimization Layers:**
```
Level 1: SIMD - Process 8 floats per instruction
Level 2: Cache blocking - Keep working set in L1/L2
Level 3: Parallelism - Distribute tiles across cores
```

**Algorithm Structure:**
```rust
// Parallel over row blocks
par_iter row_blocks {
    // Cache blocking
    for each tile {
        // SIMD innermost loops
        simd_process_tile()
    }
}
```

**Micro-kernels:**
Create optimized 8×8 or 16×16 micro-kernels that:
- Fit entirely in registers
- Use SIMD for all operations
- Minimize loads/stores

**Role Each Plays:**
- Tiling: Reduces DRAM bandwidth from ~100GB/s to ~10GB/s
- Parallelism: Increases compute from 40 GFLOPS to 200+ GFLOPS
- SIMD: Reduces instructions by 8x
- Together: Approach theoretical peak (400-800 GFLOPS for modern CPUs)

### Checkpoint Tests

```rust
#[test]
fn test_optimized_correctness() {
    use rand::Rng;

    let size = 256;
    let mut rng = rand::thread_rng();

    let a_data: Vec<f32> = (0..size * size).map(|_| rng.gen()).collect();
    let b_data: Vec<f32> = (0..size * size).map(|_| rng.gen()).collect();

    let a = Matrix::from_vec(a_data.clone(), size, size);
    let b = Matrix::from_vec(b_data.clone(), size, size);

    let naive = naive_matmul(&a, &b);
    let optimized = optimized_matmul(&a, &b);

    // Check results match within floating-point tolerance
    let mut max_error = 0.0;
    for i in 0..size {
        for j in 0..size {
            let error = (*naive.get(i, j) - *optimized.get(i, j)).abs();
            max_error = max_error.max(error);
        }
    }

    println!("Max error: {}", max_error);
    assert!(max_error < 0.1);
}

#[test]
fn benchmark_all_versions() {
    use std::time::Instant;

    let size = 1024;
    let a = Matrix::from_vec(vec![1.0; size * size], size, size);
    let b = Matrix::from_vec(vec![2.0; size * size], size, size);

    println!("\n=== Matrix Multiplication Benchmark ({}×{}) ===\n", size, size);

    // Naive
    let start = Instant::now();
    let _ = naive_matmul(&a, &b);
    let naive_time = start.elapsed();
    let naive_gflops = (2.0 * (size as f64).powi(3)) / (naive_time.as_secs_f64() * 1e9);
    println!("Naive:      {:?} ({:.2} GFLOPS)", naive_time, naive_gflops);

    // Blocked
    let start = Instant::now();
    let _ = blocked_matmul(&a, &b);
    let blocked_time = start.elapsed();
    let blocked_gflops = (2.0 * (size as f64).powi(3)) / (blocked_time.as_secs_f64() * 1e9);
    println!("Blocked:    {:?} ({:.2} GFLOPS, {:.1}x)",
        blocked_time, blocked_gflops,
        naive_time.as_secs_f64() / blocked_time.as_secs_f64());

    // Parallel
    let start = Instant::now();
    let _ = parallel_matmul(&a, &b);
    let par_time = start.elapsed();
    let par_gflops = (2.0 * (size as f64).powi(3)) / (par_time.as_secs_f64() * 1e9);
    println!("Parallel:   {:?} ({:.2} GFLOPS, {:.1}x)",
        par_time, par_gflops,
        naive_time.as_secs_f64() / par_time.as_secs_f64());

    // SIMD
    let start = Instant::now();
    let _ = simd_matmul(&a, &b);
    let simd_time = start.elapsed();
    let simd_gflops = (2.0 * (size as f64).powi(3)) / (simd_time.as_secs_f64() * 1e9);
    println!("SIMD:       {:?} ({:.2} GFLOPS, {:.1}x)",
        simd_time, simd_gflops,
        naive_time.as_secs_f64() / simd_time.as_secs_f64());

    // Optimized (all combined)
    let start = Instant::now();
    let _ = optimized_matmul(&a, &b);
    let opt_time = start.elapsed();
    let opt_gflops = (2.0 * (size as f64).powi(3)) / (opt_time.as_secs_f64() * 1e9);
    println!("Optimized:  {:?} ({:.2} GFLOPS, {:.1}x)",
        opt_time, opt_gflops,
        naive_time.as_secs_f64() / opt_time.as_secs_f64());

    println!("\nFinal speedup: {:.1}x over naive",
        naive_time.as_secs_f64() / opt_time.as_secs_f64());
}

#[test]
fn test_micro_kernel() {
    // Test small kernel optimization
    let kernel_size = 16;
    let a = Matrix::from_vec(vec![1.0; kernel_size * kernel_size], kernel_size, kernel_size);
    let b = Matrix::from_vec(vec![2.0; kernel_size * kernel_size], kernel_size, kernel_size);

    let result = micro_kernel_matmul(&a, &b, kernel_size);

    // Check all elements
    for i in 0..kernel_size {
        for j in 0..kernel_size {
            let expected = kernel_size as f32 * 2.0;
            assert_eq!(*result.get(i, j), expected);
        }
    }
}

#[test]
fn test_cache_performance() {
    // Measure cache hit rate indirectly via performance
    let sizes = [128, 256, 512, 1024, 2048];

    println!("\nCache performance scaling:");
    for &size in &sizes {
        let a = Matrix::from_vec(vec![1.0; size * size], size, size);
        let b = Matrix::from_vec(vec![2.0; size * size], size, size);

        let start = std::time::Instant::now();
        let _ = optimized_matmul(&a, &b);
        let elapsed = start.elapsed();

        let gflops = (2.0 * (size as f64).powi(3)) / (elapsed.as_secs_f64() * 1e9);
        println!("{}×{}: {:.2} GFLOPS", size, size, gflops);
    }
}
```

### Starter Code

```rust
const MICRO_KERNEL_SIZE: usize = 16;

pub fn optimized_matmul(a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32> {
    // TODO: Combine all optimizations
    //
    // Structure:
    // 1. Transpose B for better cache access
    // 2. Parallel outer loop (distribute row blocks)
    // 3. Cache blocking (tile the matrices)
    // 4. SIMD micro-kernels for innermost computation
    //
    // Pseudocode:
    // let b_transposed = transpose(b);
    //
    // result.data.par_chunks_mut(BLOCK_SIZE * b.cols)
    //     .enumerate()
    //     .for_each(|(block_i, result_block)| {
    //         for block_j in (0..b.cols).step_by(BLOCK_SIZE) {
    //             for block_k in (0..a.cols).step_by(BLOCK_SIZE) {
    //                 simd_multiply_block(
    //                     a, &b_transposed, result_block,
    //                     block_i, block_j, block_k
    //                 );
    //             }
    //         }
    //     });
    todo!()
}

pub fn micro_kernel_matmul(
    a: &Matrix<f32>,
    b: &Matrix<f32>,
    kernel_size: usize
) -> Matrix<f32> {
    // TODO: Highly optimized micro-kernel
    //
    // For 16×16 blocks:
    // - Load into SIMD registers
    // - Perform all operations in registers
    // - Minimize memory traffic
    //
    // This is the innermost kernel used by optimized_matmul
    todo!()
}

pub fn prefetch_block(ptr: *const f32, size: usize) {
    // TODO: Software prefetch for next block
    //
    // Use _mm_prefetch intrinsic to load next cache line
    // Hides memory latency
    //
    // unsafe {
    //     for i in (0..size).step_by(64) {
    //         _mm_prefetch(ptr.add(i) as *const i8, _MM_HINT_T0);
    //     }
    // }
    todo!()
}
```

---

## Milestone 6: GPU Acceleration with wgpu

### Introduction

**Why Milestone 5 Is Not Enough:**
Even with all CPU optimizations, we're limited by CPU cores (8-16) and memory bandwidth (~50 GB/s). Modern GPUs have thousands of cores and 500+ GB/s memory bandwidth.

**What We're Improving:**
Implement matrix multiplication on GPU using WebGPU (wgpu). GPUs excel at massively parallel workloads like matrix multiplication.

**GPU vs CPU:**
```
CPU: 8-16 cores, 200 GFLOPS, 50 GB/s bandwidth
GPU: 2000-10000 cores, 5000+ GFLOPS, 500+ GB/s bandwidth
```

**Expected Performance:** 1000-5000 GFLOPS (10-25x over optimized CPU)

### Architecture

**Dependencies:**
```toml
[dependencies]
wgpu = "0.18"
pollster = "0.3"
bytemuck = "1.14"
```

**GPU Concepts:**
- **Compute Shader**: Program that runs on GPU
- **Work Groups**: Threads organized in 3D grid
- **Shared Memory**: Fast on-chip memory shared by work group
- **Global Memory**: Device memory (VRAM)

**Tiled GPU Algorithm:**
```
Each work group computes one tile of C (e.g., 16×16)
Shared memory holds tiles of A and B
Threads cooperate to load tiles, then compute
```

**Key Components:**
- `GpuMatrixMultiplier` - GPU context and buffers
- Compute shader in WGSL (WebGPU Shading Language)
- Buffer management (host ↔ device transfers)

### Checkpoint Tests

```rust
#[test]
fn test_gpu_matmul_correctness() {
    let size = 128;
    let a = Matrix::from_vec(vec![1.0; size * size], size, size);
    let b = Matrix::from_vec(vec![2.0; size * size], size, size);

    let cpu_result = optimized_matmul(&a, &b);

    let mut gpu_multiplier = GpuMatrixMultiplier::new().unwrap();
    let gpu_result = gpu_multiplier.multiply(&a, &b).unwrap();

    // Check results match
    let mut max_error = 0.0;
    for i in 0..size {
        for j in 0..size {
            let error = (*cpu_result.get(i, j) - *gpu_result.get(i, j)).abs();
            max_error = max_error.max(error);
        }
    }

    println!("Max GPU error: {}", max_error);
    assert!(max_error < 0.1);
}

#[test]
fn test_gpu_performance() {
    use std::time::Instant;

    let size = 2048;
    let a = Matrix::from_vec(vec![1.0; size * size], size, size);
    let b = Matrix::from_vec(vec![2.0; size * size], size, size);

    // CPU
    let start = Instant::now();
    let _ = optimized_matmul(&a, &b);
    let cpu_time = start.elapsed();

    // GPU
    let mut gpu_multiplier = GpuMatrixMultiplier::new().unwrap();
    let start = Instant::now();
    let _ = gpu_multiplier.multiply(&a, &b).unwrap();
    let gpu_time = start.elapsed();

    let cpu_gflops = (2.0 * (size as f64).powi(3)) / (cpu_time.as_secs_f64() * 1e9);
    let gpu_gflops = (2.0 * (size as f64).powi(3)) / (gpu_time.as_secs_f64() * 1e9);

    println!("CPU: {:?} ({:.2} GFLOPS)", cpu_time, cpu_gflops);
    println!("GPU: {:?} ({:.2} GFLOPS)", gpu_time, gpu_gflops);
    println!("Speedup: {:.2}x", cpu_time.as_secs_f64() / gpu_time.as_secs_f64());
}

#[test]
fn test_gpu_large_matrix() {
    let size = 4096;
    let a = Matrix::from_vec(vec![1.0; size * size], size, size);
    let b = Matrix::from_vec(vec![2.0; size * size], size, size);

    let mut gpu_multiplier = GpuMatrixMultiplier::new().unwrap();
    let result = gpu_multiplier.multiply(&a, &b).unwrap();

    assert_eq!(result.rows(), size);
    assert_eq!(result.cols(), size);

    // Spot check a few values
    let expected = size as f32;
    assert_eq!(*result.get(0, 0), expected);
    assert_eq!(*result.get(100, 100), expected);
}

#[test]
fn test_gpu_transfer_overhead() {
    // Measure data transfer cost
    let size = 1024;
    let a = Matrix::from_vec(vec![1.0; size * size], size, size);
    let b = Matrix::from_vec(vec![2.0; size * size], size, size);

    let mut gpu_multiplier = GpuMatrixMultiplier::new().unwrap();

    // Warm up
    let _ = gpu_multiplier.multiply(&a, &b).unwrap();

    // Measure multiple runs
    let start = std::time::Instant::now();
    for _ in 0..10 {
        let _ = gpu_multiplier.multiply(&a, &b).unwrap();
    }
    let avg_time = start.elapsed() / 10;

    println!("Average GPU time: {:?}", avg_time);
}
```

### Starter Code

```rust
use wgpu::util::DeviceExt;

pub struct GpuMatrixMultiplier {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
}

impl GpuMatrixMultiplier {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // TODO: Initialize GPU
        //
        // 1. Request GPU device and queue
        // 2. Load compute shader
        // 3. Create compute pipeline
        //
        // let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        // let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))?;
        // let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))?;
        //
        // let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        //     label: Some("Matrix Multiply Shader"),
        //     source: wgpu::ShaderSource::Wgsl(include_str!("matmul.wgsl").into()),
        // });
        //
        // let pipeline = device.create_compute_pipeline(...);
        todo!()
    }

    pub fn multiply(&mut self, a: &Matrix<f32>, b: &Matrix<f32>) -> Result<Matrix<f32>, Box<dyn std::error::Error>> {
        // TODO: GPU matrix multiplication
        //
        // Steps:
        // 1. Create GPU buffers for A, B, C
        // 2. Copy A and B to GPU
        // 3. Dispatch compute shader
        // 4. Copy C back to CPU
        //
        // let a_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Matrix A"),
        //     contents: bytemuck::cast_slice(&a.data),
        //     usage: wgpu::BufferUsages::STORAGE,
        // });
        //
        // // Similar for B and C
        //
        // let mut encoder = self.device.create_command_encoder(&Default::default());
        // {
        //     let mut compute_pass = encoder.begin_compute_pass(&Default::default());
        //     compute_pass.set_pipeline(&self.pipeline);
        //     compute_pass.set_bind_group(0, &bind_group, &[]);
        //     compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
        // }
        //
        // self.queue.submit([encoder.finish()]);
        //
        // // Read back result
        todo!()
    }
}

// ============================================================================
// COMPUTE SHADER (matmul.wgsl)
// ============================================================================

const WGSL_SHADER: &str = r#"
// TODO: Write GPU compute shader
//
// @group(0) @binding(0) var<storage, read> a: array<f32>;
// @group(0) @binding(1) var<storage, read> b: array<f32>;
// @group(0) @binding(2) var<storage, read_write> c: array<f32>;
// @group(0) @binding(3) var<uniform> dims: vec3<u32>; // M, N, K
//
// const TILE_SIZE: u32 = 16u;
//
// var<workgroup> a_tile: array<array<f32, TILE_SIZE>, TILE_SIZE>;
// var<workgroup> b_tile: array<array<f32, TILE_SIZE>, TILE_SIZE>;
//
// @compute @workgroup_size(16, 16, 1)
// fn main(
//     @builtin(global_invocation_id) global_id: vec3<u32>,
//     @builtin(local_invocation_id) local_id: vec3<u32>,
// ) {
//     let row = global_id.x;
//     let col = global_id.y;
//
//     var sum = 0.0;
//
//     // Tiled multiplication
//     for (var tile = 0u; tile < (dims.z + TILE_SIZE - 1u) / TILE_SIZE; tile++) {
//         // Load tile into shared memory
//         let a_idx = row * dims.z + tile * TILE_SIZE + local_id.y;
//         a_tile[local_id.x][local_id.y] = a[a_idx];
//
//         let b_idx = (tile * TILE_SIZE + local_id.x) * dims.y + col;
//         b_tile[local_id.x][local_id.y] = b[b_idx];
//
//         workgroupBarrier();
//
//         // Compute partial dot product
//         for (var k = 0u; k < TILE_SIZE; k++) {
//             sum += a_tile[local_id.x][k] * b_tile[k][local_id.y];
//         }
//
//         workgroupBarrier();
//     }
//
//     // Write result
//     if (row < dims.x && col < dims.y) {
//         c[row * dims.y + col] = sum;
//     }
// }
"#;
```

---

## Complete Working Example

```rust
use std::time::Instant;

// ============================================================================
// MATRIX STRUCT
// ============================================================================

#[derive(Debug, Clone)]
pub struct Matrix<T> {
    data: Vec<T>,
    rows: usize,
    cols: usize,
}

impl<T: Default + Clone> Matrix<T> {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            data: vec![T::default(); rows * cols],
            rows,
            cols,
        }
    }

    pub fn from_vec(data: Vec<T>, rows: usize, cols: usize) -> Self {
        assert_eq!(data.len(), rows * cols);
        Self { data, rows, cols }
    }

    pub fn get(&self, i: usize, j: usize) -> &T {
        &self.data[i * self.cols + j]
    }

    pub fn get_mut(&mut self, i: usize, j: usize) -> &mut T {
        &mut self.data[i * self.cols + j]
    }

    pub fn set(&mut self, i: usize, j: usize, value: T) {
        self.data[i * self.cols + j] = value;
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }
}

// ============================================================================
// NAIVE IMPLEMENTATION
// ============================================================================

pub fn naive_matmul(a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32> {
    assert_eq!(a.cols, b.rows);

    let mut result = Matrix::new(a.rows, b.cols);

    for i in 0..a.rows {
        for j in 0..b.cols {
            let mut sum = 0.0;
            for k in 0..a.cols {
                sum += a.get(i, k) * b.get(k, j);
            }
            result.set(i, j, sum);
        }
    }

    result
}

// ============================================================================
// BLOCKED IMPLEMENTATION
// ============================================================================

const BLOCK_SIZE: usize = 64;

pub fn blocked_matmul(a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32> {
    assert_eq!(a.cols, b.rows);

    let mut result = Matrix::new(a.rows, b.cols);

    for i_block in (0..a.rows).step_by(BLOCK_SIZE) {
        for j_block in (0..b.cols).step_by(BLOCK_SIZE) {
            for k_block in (0..a.cols).step_by(BLOCK_SIZE) {
                let i_end = (i_block + BLOCK_SIZE).min(a.rows);
                let j_end = (j_block + BLOCK_SIZE).min(b.cols);
                let k_end = (k_block + BLOCK_SIZE).min(a.cols);

                for i in i_block..i_end {
                    for j in j_block..j_end {
                        let mut sum = *result.get(i, j);
                        for k in k_block..k_end {
                            sum += a.get(i, k) * b.get(k, j);
                        }
                        result.set(i, j, sum);
                    }
                }
            }
        }
    }

    result
}

// ============================================================================
// PARALLEL IMPLEMENTATION
// ============================================================================

use rayon::prelude::*;

pub fn parallel_matmul(a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32> {
    assert_eq!(a.cols, b.rows);

    let mut result = Matrix::new(a.rows, b.cols);

    result
        .data
        .par_chunks_mut(b.cols)
        .enumerate()
        .for_each(|(i, row_chunk)| {
            for j in 0..b.cols {
                let mut sum = 0.0;
                for k in 0..a.cols {
                    sum += a.get(i, k) * b.get(k, j);
                }
                row_chunk[j] = sum;
            }
        });

    result
}

// ============================================================================
// PARALLEL + BLOCKED
// ============================================================================

pub fn optimized_matmul(a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32> {
    assert_eq!(a.cols, b.rows);

    let mut result = Matrix::new(a.rows, b.cols);

    let row_blocks: Vec<_> = (0..a.rows).step_by(BLOCK_SIZE).collect();

    row_blocks.par_iter().for_each(|&i_block| {
        for j_block in (0..b.cols).step_by(BLOCK_SIZE) {
            for k_block in (0..a.cols).step_by(BLOCK_SIZE) {
                let i_end = (i_block + BLOCK_SIZE).min(a.rows);
                let j_end = (j_block + BLOCK_SIZE).min(b.cols);
                let k_end = (k_block + BLOCK_SIZE).min(a.cols);

                for i in i_block..i_end {
                    for j in j_block..j_end {
                        let mut sum = unsafe {
                            *result.data.get_unchecked(i * result.cols + j)
                        };

                        for k in k_block..k_end {
                            sum += a.get(i, k) * b.get(k, j);
                        }

                        unsafe {
                            *result.data.get_unchecked_mut(i * result.cols + j) = sum;
                        }
                    }
                }
            }
        }
    });

    result
}

// ============================================================================
// BENCHMARKING
// ============================================================================

fn benchmark(name: &str, size: usize, f: impl Fn(&Matrix<f32>, &Matrix<f32>) -> Matrix<f32>) {
    let a = Matrix::from_vec(vec![1.0; size * size], size, size);
    let b = Matrix::from_vec(vec![2.0; size * size], size, size);

    let start = Instant::now();
    let _ = f(&a, &b);
    let elapsed = start.elapsed();

    let flops = 2.0 * (size as f64).powi(3);
    let gflops = flops / (elapsed.as_secs_f64() * 1e9);

    println!("{:12} {:?} ({:.2} GFLOPS)", name, elapsed, gflops);
}

fn main() {
    println!("=== Matrix Multiplication Performance ===\n");

    for &size in &[128, 256, 512, 1024] {
        println!("Matrix size: {}×{}", size, size);

        benchmark("Naive", size, naive_matmul);
        benchmark("Blocked", size, blocked_matmul);
        benchmark("Parallel", size, parallel_matmul);
        benchmark("Optimized", size, optimized_matmul);

        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naive() {
        let a = Matrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
        let b = Matrix::from_vec(vec![5.0, 6.0, 7.0, 8.0], 2, 2);

        let c = naive_matmul(&a, &b);

        assert_eq!(*c.get(0, 0), 19.0);
        assert_eq!(*c.get(0, 1), 22.0);
        assert_eq!(*c.get(1, 0), 43.0);
        assert_eq!(*c.get(1, 1), 50.0);
    }

    #[test]
    fn test_all_match() {
        let size = 64;
        let a = Matrix::from_vec(vec![1.0; size * size], size, size);
        let b = Matrix::from_vec(vec![2.0; size * size], size, size);

        let naive = naive_matmul(&a, &b);
        let blocked = blocked_matmul(&a, &b);
        let parallel = parallel_matmul(&a, &b);

        for i in 0..size {
            for j in 0..size {
                assert_eq!(*naive.get(i, j), *blocked.get(i, j));
                assert_eq!(*naive.get(i, j), *parallel.get(i, j));
            }
        }
    }
}
```

This completes the comprehensive matrix multiplication project with all 6 milestones, from naive to GPU-accelerated!
