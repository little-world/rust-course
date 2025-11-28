# Chapter 19: CPU Performance Optimization - Deep Dive

## Project: High-Performance Image Convolution with CPU Optimizations

### Problem Statement

Build a production-grade image convolution library that demonstrates fundamental CPU optimization techniques. Implement Gaussian blur, edge detection, and sharpening filters while progressively optimizing for heap vs stack allocation, cache efficiency, register usage, branch prediction, and assembly intrinsics.

The system must:
- Apply convolution kernels to images (3×3, 5×5, 7×7)
- Process images up to 8K resolution (7680×4320 pixels)
- Demonstrate measurable performance improvements from each optimization
- Handle edge cases (image boundaries, different pixel formats)
- Achieve 10-100x speedup through systematic optimization
- Reach 500+ million pixels/second throughput

### Use Cases

- **Real-Time Video Processing**: 4K@60fps video filters (blur, sharpen)
- **Photo Editing Applications**: Instagram-like filters, Photoshop operations
- **Computer Vision**: Feature detection (Sobel, Canny edge detection)
- **Medical Imaging**: Image enhancement, noise reduction
- **Scientific Visualization**: Data smoothing and filtering
- **Game Development**: Post-processing effects (bloom, depth of field)

### Why It Matters

**Performance Impact of Optimizations:**
```
Naive implementation:        ~10 Mpixels/sec
Stack allocation:           ~20 Mpixels/sec (2x)
Cache optimization:        ~100 Mpixels/sec (10x)
Register optimization:     ~200 Mpixels/sec (20x)
Branch-free code:          ~350 Mpixels/sec (35x)
Assembly/intrinsics:       ~500 Mpixels/sec (50x)
```

**Real-World Impact:**
- Processing 4K video (8.3 Mpixels/frame) at 60 fps = 500 Mpixels/sec
- Naive: Can't do real-time (10 Mpixels/sec)
- Optimized: Can process 4K@60fps with headroom (500 Mpixels/sec)

**CPU Architecture Understanding:**

```
Memory Hierarchy (typical latencies):
Register:        0 cycles     (32 registers × 64 bits)
L1 Cache:        4 cycles     (32 KB)
L2 Cache:       12 cycles     (256 KB)
L3 Cache:       40 cycles     (8-32 MB)
RAM:          ~200 cycles     (16-64 GB)
Heap alloc:  ~1000 cycles     (syscall overhead)
```

**Branch Misprediction Cost:**
- Correctly predicted: 0 cycles
- Mispredicted: 15-20 cycles (pipeline flush)
- 10% misprediction rate on 100M branches = 150-200M wasted cycles

**Why Each Optimization Matters:**

1. **Stack vs Heap**: malloc/free costs 100-1000 cycles, stack allocation is free
2. **Cache**: 99% L1 hit vs 50% L1 hit = 3-5x performance difference
3. **Registers**: Memory load = 4 cycles, register access = 0 cycles
4. **Branch Prediction**: Modern CPUs predict 95-99%, but 1-5% misses kill performance
5. **Assembly**: Hand-tuned code can be 2-5x faster than compiler output for hot paths

---

## Milestone 1: Naive Implementation with Heap Allocations

### Introduction

Implement straightforward convolution with no optimizations. Allocate temporary buffers on heap, use natural memory access patterns, include bounds checking and branches. This establishes a baseline for measuring improvements.

**Convolution Operation:**
```
For each pixel (x, y):
  result[x][y] = Σ Σ kernel[i][j] × image[x+i][y+j]
                 i j
```

For 3×3 kernel: 9 multiplications + 9 additions per pixel

### Architecture

**Structs:**
- `Image` - RGB image representation
  - **Field** `data: Vec<u8>` - Heap-allocated pixel data (RGB bytes)
  - **Field** `width: usize` - Image width in pixels
  - **Field** `height: usize` - Image height in pixels
  - **Function** `new(width: usize, height: usize) -> Self` - Allocate on heap
  - **Function** `get_pixel(&self, x: usize, y: usize) -> (u8, u8, u8)` - Get RGB
  - **Function** `set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8))` - Set RGB

- `Kernel` - Convolution kernel
  - **Field** `data: Vec<f32>` - Heap-allocated kernel weights
  - **Field** `size: usize` - Kernel dimension (3, 5, 7, etc.)
  - **Function** `gaussian_blur(size: usize, sigma: f32) -> Self` - Create Gaussian kernel
  - **Function** `edge_detection() -> Self` - Sobel operator
  - **Function** `sharpen() -> Self` - Sharpening kernel

**Key Functions:**
- `naive_convolve(image: &Image, kernel: &Kernel) -> Image` - Basic convolution
- `clamp(value: f32, min: f32, max: f32) -> u8` - Bounds checking with branches
- `safe_get_pixel(image: &Image, x: i32, y: i32) -> (u8, u8, u8)` - Boundary handling

**Role Each Plays:**
- Heap allocation: `Vec::new()` calls malloc for every operation
- Boundary checks: `if x < 0 || x >= width` branches in hot loop
- Natural access: Row-major access without cache consideration
- Separate channels: Process R, G, B separately (poor locality)

**Memory Layout:**
```
RGB pixels stored interleaved:
[R0, G0, B0, R1, G1, B1, R2, G2, B2, ...]

Access pattern for convolution:
Row 0: [x-1,y-1] [x,y-1] [x+1,y-1]
Row 1: [x-1,y]   [x,y]   [x+1,y]
Row 2: [x-1,y+1] [x,y+1] [x+1,y+1]
```

### Checkpoint Tests

```rust
#[test]
fn test_image_creation() {
    let img = Image::new(100, 100);
    assert_eq!(img.width, 100);
    assert_eq!(img.height, 100);
    assert_eq!(img.data.len(), 100 * 100 * 3); // RGB
}

#[test]
fn test_pixel_access() {
    let mut img = Image::new(10, 10);
    img.set_pixel(5, 5, (255, 128, 64));

    let (r, g, b) = img.get_pixel(5, 5);
    assert_eq!((r, g, b), (255, 128, 64));
}

#[test]
fn test_gaussian_kernel() {
    let kernel = Kernel::gaussian_blur(3, 1.0);

    // 3×3 kernel
    assert_eq!(kernel.size, 3);
    assert_eq!(kernel.data.len(), 9);

    // Sum of weights should be ≈1.0
    let sum: f32 = kernel.data.iter().sum();
    assert!((sum - 1.0).abs() < 0.01);
}

#[test]
fn test_naive_convolution() {
    // Create simple test image (white square on black background)
    let mut img = Image::new(5, 5);
    img.set_pixel(2, 2, (255, 255, 255)); // Center pixel white

    let kernel = Kernel::gaussian_blur(3, 1.0);
    let result = naive_convolve(&img, &kernel);

    // Blur should spread white to neighbors
    let (r, _, _) = result.get_pixel(2, 2);
    assert!(r > 0);

    let (r, _, _) = result.get_pixel(1, 2);
    assert!(r > 0); // Neighbor should have some white
}

#[test]
fn test_edge_detection() {
    let mut img = Image::new(10, 10);

    // Create vertical edge
    for y in 0..10 {
        for x in 0..5 {
            img.set_pixel(x, y, (0, 0, 0));
        }
        for x in 5..10 {
            img.set_pixel(x, y, (255, 255, 255));
        }
    }

    let kernel = Kernel::edge_detection();
    let result = naive_convolve(&img, &kernel);

    // Edge should have high values at x=5
    let (edge_val, _, _) = result.get_pixel(5, 5);
    assert!(edge_val > 100);
}

#[test]
fn test_heap_allocations() {
    // This test demonstrates heap overhead
    use std::time::Instant;

    let img = Image::new(1000, 1000);
    let kernel = Kernel::gaussian_blur(3, 1.0);

    let start = Instant::now();
    for _ in 0..10 {
        // Each iteration allocates new result on heap
        let _ = naive_convolve(&img, &kernel);
    }
    let elapsed = start.elapsed();

    println!("10 convolutions (with heap alloc): {:?}", elapsed);
}
```

### Starter Code

```rust
#[derive(Debug, Clone)]
pub struct Image {
    data: Vec<u8>,  // RGB bytes: [R, G, B, R, G, B, ...]
    width: usize,
    height: usize,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        // TODO: Allocate on heap
        // Self {
        //     data: vec![0u8; width * height * 3],
        //     width,
        //     height,
        // }
        todo!()
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> (u8, u8, u8) {
        // TODO: Extract RGB from interleaved array
        // let idx = (y * self.width + x) * 3;
        // (self.data[idx], self.data[idx + 1], self.data[idx + 2])
        todo!()
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        // TODO: Set RGB values
        // let idx = (y * self.width + x) * 3;
        // self.data[idx] = rgb.0;
        // self.data[idx + 1] = rgb.1;
        // self.data[idx + 2] = rgb.2;
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Kernel {
    data: Vec<f32>,
    size: usize,
}

impl Kernel {
    pub fn gaussian_blur(size: usize, sigma: f32) -> Self {
        // TODO: Create Gaussian kernel
        //
        // Gaussian function: G(x,y) = (1/(2πσ²)) × e^(-(x²+y²)/(2σ²))
        //
        // let mut data = vec![0.0; size * size];
        // let center = (size / 2) as i32;
        //
        // for y in 0..size {
        //     for x in 0..size {
        //         let dx = x as i32 - center;
        //         let dy = y as i32 - center;
        //         let dist_sq = (dx * dx + dy * dy) as f32;
        //         data[y * size + x] = (-dist_sq / (2.0 * sigma * sigma)).exp();
        //     }
        // }
        //
        // // Normalize so sum = 1.0
        // let sum: f32 = data.iter().sum();
        // for val in data.iter_mut() {
        //     *val /= sum;
        // }
        //
        // Self { data, size }
        todo!()
    }

    pub fn edge_detection() -> Self {
        // TODO: Sobel operator
        // Sobel X kernel:
        // [-1, 0, 1]
        // [-2, 0, 2]
        // [-1, 0, 1]
        todo!()
    }

    pub fn sharpen() -> Self {
        // TODO: Sharpening kernel
        // [ 0, -1,  0]
        // [-1,  5, -1]
        // [ 0, -1,  0]
        todo!()
    }
}

pub fn naive_convolve(image: &Image, kernel: &Kernel) -> Image {
    // TODO: Implement naive convolution
    //
    // Algorithm:
    // 1. Create result image (HEAP ALLOCATION)
    // 2. For each pixel (x, y):
    //    a. For each kernel element (kx, ky):
    //       - Get pixel at (x + kx - offset, y + ky - offset)
    //       - Check bounds (BRANCHES)
    //       - Multiply by kernel weight
    //       - Accumulate
    //    b. Clamp result to 0-255 (BRANCHES)
    //    c. Set result pixel
    //
    // let mut result = Image::new(image.width, image.height); // HEAP ALLOC
    // let offset = (kernel.size / 2) as i32;
    //
    // for y in 0..image.height {
    //     for x in 0..image.width {
    //         let mut r_sum = 0.0;
    //         let mut g_sum = 0.0;
    //         let mut b_sum = 0.0;
    //
    //         for ky in 0..kernel.size {
    //             for kx in 0..kernel.size {
    //                 let img_x = x as i32 + kx as i32 - offset;
    //                 let img_y = y as i32 + ky as i32 - offset;
    //
    //                 // BRANCHES for bounds checking
    //                 if img_x >= 0 && img_x < image.width as i32 &&
    //                    img_y >= 0 && img_y < image.height as i32 {
    //
    //                     let (r, g, b) = image.get_pixel(img_x as usize, img_y as usize);
    //                     let weight = kernel.data[ky * kernel.size + kx];
    //
    //                     r_sum += r as f32 * weight;
    //                     g_sum += g as f32 * weight;
    //                     b_sum += b as f32 * weight;
    //                 }
    //             }
    //         }
    //
    //         // BRANCHES for clamping
    //         let r = clamp(r_sum, 0.0, 255.0);
    //         let g = clamp(g_sum, 0.0, 255.0);
    //         let b = clamp(b_sum, 0.0, 255.0);
    //
    //         result.set_pixel(x, y, (r, g, b));
    //     }
    // }
    //
    // result
    todo!()
}

fn clamp(value: f32, min: f32, max: f32) -> u8 {
    // TODO: Clamp with branches
    // if value < min {
    //     min as u8
    // } else if value > max {
    //     max as u8
    // } else {
    //     value as u8
    // }
    todo!()
}

pub fn benchmark_naive(width: usize, height: usize) -> f64 {
    use std::time::Instant;

    let image = Image::new(width, height);
    let kernel = Kernel::gaussian_blur(3, 1.0);

    let start = Instant::now();
    let _ = naive_convolve(&image, &kernel);
    let elapsed = start.elapsed();

    let pixels_per_sec = (width * height) as f64 / elapsed.as_secs_f64();
    let mpixels_per_sec = pixels_per_sec / 1_000_000.0;

    println!("Naive: {:.2} Mpixels/sec", mpixels_per_sec);
    mpixels_per_sec
}
```

---

## Milestone 2: Stack Allocation and Buffer Reuse

### Introduction

**Why Milestone 1 Is Not Enough:**
Every `naive_convolve()` call allocates a new result image on the heap via `Vec::new()`. Heap allocation costs ~1000 cycles (malloc syscall, memory manager overhead). For small operations or repeated calls, this overhead dominates.

**Heap Allocation Costs:**
```
malloc(1MB):  ~10,000 cycles
free(1MB):    ~5,000 cycles
Total:        ~15,000 cycles per convolution

vs

Stack allocation: 0 cycles (just SP adjustment)
```

**What We're Improving:**
- Use stack allocation for small images/buffers
- Pre-allocate and reuse buffers for large images
- Use fixed-size arrays on stack where possible
- Reduce allocator pressure

**Stack vs Heap Tradeoffs:**
- Stack: Fast (free), limited size (~8MB), automatic cleanup
- Heap: Unlimited size, slow allocation, manual management

### Architecture

**Modified Structs:**
- `ImageBuffer` - Reusable buffer
  - **Field** `buffer: Vec<u8>` - Pre-allocated, reused across calls
  - **Function** `with_capacity(width: usize, height: usize) -> Self` - Pre-allocate
  - **Function** `convolve_into(&mut self, image: &Image, kernel: &Kernel)` - Reuse buffer

**New Functions:**
- `convolve_small_stack(image: &Image, kernel: &Kernel) -> Image` - Stack allocation for small images
- `create_buffer_pool(count: usize, width: usize, height: usize) -> Vec<ImageBuffer>` - Buffer pool

**Stack Allocation Pattern:**
```rust
// Small image: use stack
let mut pixel_buffer: [u8; 64 * 64 * 3] = [0; 64 * 64 * 3];

// Large image: pre-allocate once, reuse
let mut buffer = ImageBuffer::with_capacity(width, height);
for image in images {
    buffer.convolve_into(&image, &kernel); // No allocation
}
```

**Role Each Plays:**
- Stack arrays: Zero allocation cost for small working sets
- Buffer reuse: Amortize allocation cost across operations
- Buffer pool: Pre-allocate for multi-threaded scenarios

### Checkpoint Tests

```rust
#[test]
fn test_stack_small_image() {
    let img = Image::new(64, 64);
    let kernel = Kernel::gaussian_blur(3, 1.0);

    let result = convolve_small_stack(&img, &kernel);

    assert_eq!(result.width, 64);
    assert_eq!(result.height, 64);
}

#[test]
fn test_buffer_reuse() {
    let img = Image::new(100, 100);
    let kernel = Kernel::gaussian_blur(3, 1.0);

    let mut buffer = ImageBuffer::with_capacity(100, 100);

    // Multiple convolutions without new allocations
    for _ in 0..100 {
        buffer.convolve_into(&img, &kernel);
    }
}

#[test]
fn test_allocation_overhead() {
    use std::time::Instant;

    let img = Image::new(500, 500);
    let kernel = Kernel::gaussian_blur(3, 1.0);

    // Naive (allocates every time)
    let start = Instant::now();
    for _ in 0..20 {
        let _ = naive_convolve(&img, &kernel);
    }
    let naive_time = start.elapsed();

    // Reuse buffer
    let mut buffer = ImageBuffer::with_capacity(500, 500);
    let start = Instant::now();
    for _ in 0..20 {
        buffer.convolve_into(&img, &kernel);
    }
    let reuse_time = start.elapsed();

    println!("Naive (20 allocs): {:?}", naive_time);
    println!("Reuse (1 alloc):   {:?}", reuse_time);
    println!("Speedup: {:.2}x", naive_time.as_secs_f64() / reuse_time.as_secs_f64());

    assert!(reuse_time < naive_time);
}

#[test]
fn test_stack_vs_heap() {
    use std::time::Instant;

    let img = Image::new(64, 64);
    let kernel = Kernel::gaussian_blur(3, 1.0);

    // Heap allocation
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = naive_convolve(&img, &kernel);
    }
    let heap_time = start.elapsed();

    // Stack allocation
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = convolve_small_stack(&img, &kernel);
    }
    let stack_time = start.elapsed();

    println!("Heap:  {:?}", heap_time);
    println!("Stack: {:?}", stack_time);
    println!("Speedup: {:.2}x", heap_time.as_secs_f64() / stack_time.as_secs_f64());
}
```

### Starter Code

```rust
const MAX_STACK_IMAGE_SIZE: usize = 64; // 64×64 max for stack

pub fn convolve_small_stack(image: &Image, kernel: &Kernel) -> Image {
    // TODO: Use stack allocation for small images
    //
    // assert!(image.width <= MAX_STACK_IMAGE_SIZE);
    // assert!(image.height <= MAX_STACK_IMAGE_SIZE);
    //
    // const BUFFER_SIZE: usize = MAX_STACK_IMAGE_SIZE * MAX_STACK_IMAGE_SIZE * 3;
    // let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    //
    // // Perform convolution directly into stack buffer
    // // ... (same logic as naive, but write to buffer instead of Vec)
    //
    // // Copy buffer to result image
    // let mut result = Image::new(image.width, image.height);
    // result.data.copy_from_slice(&buffer[..image.width * image.height * 3]);
    // result
    todo!()
}

pub struct ImageBuffer {
    buffer: Vec<u8>,
    width: usize,
    height: usize,
}

impl ImageBuffer {
    pub fn with_capacity(width: usize, height: usize) -> Self {
        // TODO: Pre-allocate buffer
        // Self {
        //     buffer: vec![0u8; width * height * 3],
        //     width,
        //     height,
        // }
        todo!()
    }

    pub fn convolve_into(&mut self, image: &Image, kernel: &Kernel) {
        // TODO: Convolve into pre-allocated buffer
        // No new allocation, just reuse self.buffer
        //
        // Same convolution logic as naive, but write to self.buffer
        // This avoids allocation overhead
        todo!()
    }

    pub fn as_image(&self) -> Image {
        // TODO: Create Image view of buffer (no copy if possible)
        todo!()
    }
}

pub fn create_buffer_pool(count: usize, width: usize, height: usize) -> Vec<ImageBuffer> {
    // TODO: Pre-allocate multiple buffers for parallel processing
    // (0..count)
    //     .map(|_| ImageBuffer::with_capacity(width, height))
    //     .collect()
    todo!()
}
```

---

## Milestone 3: Cache-Optimized Memory Access

### Introduction

**Why Milestone 2 Is Not Enough:**
Even with stack allocation, memory access patterns are cache-inefficient. Convolving accesses image in scattered pattern, causing cache misses.

**Cache Miss Analysis:**
```
Naive access for 3×3 kernel at pixel (x, y):
(x-1, y-1), (x, y-1), (x+1, y-1)  ← Row y-1
(x-1, y),   (x, y),   (x+1, y)    ← Row y
(x-1, y+1), (x, y+1), (x+1, y+1)  ← Row y+1

If image width = 1920 pixels × 3 bytes = 5760 bytes/row
Cache line = 64 bytes = ~21 pixels

Accessing row y-1, then y, then y+1 → poor temporal locality
```

**What We're Improving:**
- Tiled/blocked processing to keep working set in cache
- Prefetch next tiles to hide memory latency
- SOA vs AOS: Structure of Arrays (separate R, G, B planes)
- Align data to cache line boundaries

**Cache-Friendly Patterns:**
```
Blocking: Process image in 64×64 tiles
Prefetching: Load next tile while processing current
SOA layout: [RRR...][GGG...][BBB...] instead of [RGB][RGB][RGB]
```

**Expected Improvement:** 5-10x speedup from better cache utilization

### Architecture

**New Concepts:**
- `TILE_SIZE = 64` - Process in cache-friendly tiles
- `#[repr(align(64))]` - Cache line alignment
- Software prefetch hints

**Modified Functions:**
- `tiled_convolve(image: &Image, kernel: &Kernel) -> Image` - Blocked processing
- `prefetch_tile(data: &[u8], offset: usize)` - Software prefetch
- `convert_to_soa(image: &Image) -> ImageSOA` - Planar format

**Tiled Algorithm:**
```
for tile_y in (0..height).step_by(TILE_SIZE) {
    for tile_x in (0..width).step_by(TILE_SIZE) {
        prefetch_tile(image, next_tile_offset);

        // Process tile (fits in L1/L2 cache)
        for y in tile_y..min(tile_y + TILE_SIZE, height) {
            for x in tile_x..min(tile_x + TILE_SIZE, width) {
                // Convolve pixel
            }
        }
    }
}
```

**Role Each Plays:**
- Tiling: Maximize cache reuse
- Prefetch: Hide memory latency
- SOA: Better vectorization and cache use
- Alignment: Avoid cache line splits

### Checkpoint Tests

```rust
#[test]
fn test_tiled_correctness() {
    let img = Image::new(256, 256);
    let kernel = Kernel::gaussian_blur(3, 1.0);

    let naive_result = naive_convolve(&img, &kernel);
    let tiled_result = tiled_convolve(&img, &kernel);

    // Results should match
    for y in 0..256 {
        for x in 0..256 {
            let (r1, g1, b1) = naive_result.get_pixel(x, y);
            let (r2, g2, b2) = tiled_result.get_pixel(x, y);

            assert!((r1 as i32 - r2 as i32).abs() <= 1);
        }
    }
}

#[test]
fn test_cache_performance() {
    use std::time::Instant;

    let img = Image::new(1920, 1080);
    let kernel = Kernel::gaussian_blur(3, 1.0);

    // No tiling
    let mut buffer = ImageBuffer::with_capacity(1920, 1080);
    let start = Instant::now();
    buffer.convolve_into(&img, &kernel);
    let no_tile_time = start.elapsed();

    // With tiling
    let start = Instant::now();
    let _ = tiled_convolve(&img, &kernel);
    let tiled_time = start.elapsed();

    println!("No tiling: {:?}", no_tile_time);
    println!("Tiled:     {:?}", tiled_time);
    println!("Speedup: {:.2}x", no_tile_time.as_secs_f64() / tiled_time.as_secs_f64());

    assert!(tiled_time < no_tile_time);
}

#[test]
fn test_soa_layout() {
    let img = Image::new(100, 100);
    let soa = convert_to_soa(&img);

    assert_eq!(soa.r_plane.len(), 100 * 100);
    assert_eq!(soa.g_plane.len(), 100 * 100);
    assert_eq!(soa.b_plane.len(), 100 * 100);
}

#[test]
fn test_alignment() {
    let aligned_buffer = create_aligned_buffer(1024);

    // Check 64-byte alignment
    let ptr = aligned_buffer.as_ptr() as usize;
    assert_eq!(ptr % 64, 0);
}
```

### Starter Code

```rust
const TILE_SIZE: usize = 64;
const CACHE_LINE_SIZE: usize = 64;

pub fn tiled_convolve(image: &Image, kernel: &Kernel) -> Image {
    // TODO: Implement tiled convolution
    //
    // Process image in TILE_SIZE × TILE_SIZE blocks
    // Each tile fits in L1/L2 cache for better performance
    //
    // let mut result = Image::new(image.width, image.height);
    // let offset = (kernel.size / 2) as i32;
    //
    // for tile_y in (0..image.height).step_by(TILE_SIZE) {
    //     for tile_x in (0..image.width).step_by(TILE_SIZE) {
    //
    //         // Prefetch next tile
    //         if tile_x + TILE_SIZE < image.width {
    //             prefetch_tile(image, tile_x + TILE_SIZE, tile_y);
    //         }
    //
    //         let tile_end_y = (tile_y + TILE_SIZE).min(image.height);
    //         let tile_end_x = (tile_x + TILE_SIZE).min(image.width);
    //
    //         // Process tile
    //         for y in tile_y..tile_end_y {
    //             for x in tile_x..tile_end_x {
    //                 // Convolve pixel (same as naive)
    //             }
    //         }
    //     }
    // }
    //
    // result
    todo!()
}

pub fn prefetch_tile(image: &Image, tile_x: usize, tile_y: usize) {
    // TODO: Software prefetch for next tile
    //
    // Use std::arch::x86_64::_mm_prefetch or similar
    // Brings next cache lines into L1/L2
    //
    // #[cfg(target_arch = "x86_64")]
    // unsafe {
    //     use std::arch::x86_64::*;
    //     for y in tile_y..(tile_y + TILE_SIZE).min(image.height) {
    //         let row_start = (y * image.width + tile_x) * 3;
    //         let ptr = image.data.as_ptr().add(row_start);
    //         _mm_prefetch(ptr as *const i8, _MM_HINT_T0);
    //     }
    // }
    todo!()
}

#[repr(align(64))]
pub struct AlignedBuffer {
    data: Vec<u8>,
}

pub fn create_aligned_buffer(size: usize) -> AlignedBuffer {
    // TODO: Create cache-line aligned buffer
    // Alignment prevents cache line splits
    todo!()
}

#[derive(Debug)]
pub struct ImageSOA {
    r_plane: Vec<u8>,
    g_plane: Vec<u8>,
    b_plane: Vec<u8>,
    width: usize,
    height: usize,
}

pub fn convert_to_soa(image: &Image) -> ImageSOA {
    // TODO: Convert RGB interleaved to planar format
    //
    // AOS (Array of Structures): [RGB][RGB][RGB]...
    // SOA (Structure of Arrays):  [RRR...][GGG...][BBB...]
    //
    // SOA benefits:
    // - Better cache utilization (process one channel at a time)
    // - Easier vectorization
    //
    // let mut r_plane = vec![0u8; image.width * image.height];
    // let mut g_plane = vec![0u8; image.width * image.height];
    // let mut b_plane = vec![0u8; image.width * image.height];
    //
    // for i in 0..image.width * image.height {
    //     r_plane[i] = image.data[i * 3];
    //     g_plane[i] = image.data[i * 3 + 1];
    //     b_plane[i] = image.data[i * 3 + 2];
    // }
    //
    // ImageSOA {
    //     r_plane,
    //     g_plane,
    //     b_plane,
    //     width: image.width,
    //     height: image.height,
    // }
    todo!()
}

pub fn convolve_soa(image: &ImageSOA, kernel: &Kernel) -> ImageSOA {
    // TODO: Convolve planar image
    // Process each plane separately (better cache locality)
    todo!()
}
```

---

## Milestone 4: Register Optimization and Loop Unrolling

### Introduction

**Why Milestone 3 Is Not Enough:**
Inner loops still load values from memory repeatedly. Modern CPUs have 16-32 general-purpose registers and 16-32 SIMD registers, but we're not utilizing them effectively.

**Register Pressure:**
```
Typical convolution inner loop:
for kx in 0..3 {
    sum += pixel[kx] * kernel[kx];  // Load pixel, load kernel, multiply
}

Each iteration: 2 loads, 1 multiply, 1 add
Registers used: ~3-4
Available registers: 16 (GPR) + 16 (XMM) = 32 total
```

**What We're Improving:**
- Loop unrolling: Reduce loop overhead, expose parallelism
- Register blocking: Keep frequently used values in registers
- Minimize memory traffic: Load once, use many times
- Compiler hints: `#[inline(always)]`, `likely/unlikely`

**Loop Unrolling Example:**
```rust
// Before:
for i in 0..9 {
    sum += data[i] * kernel[i];
}

// After (fully unrolled):
sum += data[0] * kernel[0];
sum += data[1] * kernel[1];
sum += data[2] * kernel[2];
// ... 6 more lines

Benefits:
- No loop counter increment
- No branch for loop condition
- Better instruction-level parallelism (ILP)
- Compiler can optimize better
```

**Expected Improvement:** 2-3x speedup

### Architecture

**Optimization Techniques:**
- Full loop unrolling for small fixed-size kernels
- Manual register allocation hints
- `#[inline(always)]` for hot functions
- Constant propagation

**Key Functions:**
- `convolve_3x3_unrolled(image: &Image, kernel: &Kernel) -> Image` - Fully unrolled
- `convolve_5x5_unrolled(image: &Image, kernel: &Kernel) -> Image` - Unrolled 5×5
- `prefetch_register_block(...)` - Load next pixels into registers

**Unrolled 3×3 Pattern:**
```rust
#[inline(always)]
fn convolve_pixel_3x3(image: &Image, x: usize, y: usize, kernel: &[f32; 9]) -> (u8, u8, u8) {
    // Load all 9 pixels (should stay in registers)
    let p0 = image.get_pixel(x-1, y-1);
    let p1 = image.get_pixel(x,   y-1);
    let p2 = image.get_pixel(x+1, y-1);
    let p3 = image.get_pixel(x-1, y);
    let p4 = image.get_pixel(x,   y);
    let p5 = image.get_pixel(x+1, y);
    let p6 = image.get_pixel(x-1, y+1);
    let p7 = image.get_pixel(x,   y+1);
    let p8 = image.get_pixel(x+1, y+1);

    // Fully unrolled computation
    let r = p0.0 as f32 * kernel[0] +
            p1.0 as f32 * kernel[1] +
            p2.0 as f32 * kernel[2] +
            // ...
            p8.0 as f32 * kernel[8];

    // Same for g, b
    (r as u8, g as u8, b as u8)
}
```

### Checkpoint Tests

```rust
#[test]
fn test_unrolled_correctness() {
    let img = Image::new(100, 100);
    let kernel = Kernel::gaussian_blur(3, 1.0);

    let tiled = tiled_convolve(&img, &kernel);
    let unrolled = convolve_3x3_unrolled(&img, &kernel);

    // Results should match
    for y in 1..99 {
        for x in 1..99 {
            let (r1, g1, b1) = tiled.get_pixel(x, y);
            let (r2, g2, b2) = unrolled.get_pixel(x, y);

            assert!((r1 as i32 - r2 as i32).abs() <= 1);
        }
    }
}

#[test]
fn test_loop_unrolling_performance() {
    use std::time::Instant;

    let img = Image::new(1024, 1024);
    let kernel = Kernel::gaussian_blur(3, 1.0);

    // With loop
    let start = Instant::now();
    let _ = tiled_convolve(&img, &kernel);
    let loop_time = start.elapsed();

    // Unrolled
    let start = Instant::now();
    let _ = convolve_3x3_unrolled(&img, &kernel);
    let unroll_time = start.elapsed();

    println!("With loops: {:?}", loop_time);
    println!("Unrolled:   {:?}", unroll_time);
    println!("Speedup: {:.2}x", loop_time.as_secs_f64() / unroll_time.as_secs_f64());

    assert!(unroll_time < loop_time);
}

#[test]
fn test_inline_effectiveness() {
    // Test that inlining improves performance
    // Compare #[inline(always)] vs #[inline(never)]
}

#[test]
fn test_register_usage() {
    // This test is more conceptual - check assembly output
    // Use: cargo rustc --release -- --emit asm
    // Verify register usage in hot loops
}
```

### Starter Code

```rust
#[inline(always)]
fn load_3x3_neighborhood(
    image: &Image,
    x: usize,
    y: usize
) -> [(u8, u8, u8); 9] {
    // TODO: Load 9 pixels into array (encourages register allocation)
    //
    // [
    //     image.get_pixel(x-1, y-1),
    //     image.get_pixel(x,   y-1),
    //     image.get_pixel(x+1, y-1),
    //     image.get_pixel(x-1, y),
    //     image.get_pixel(x,   y),
    //     image.get_pixel(x+1, y),
    //     image.get_pixel(x-1, y+1),
    //     image.get_pixel(x,   y+1),
    //     image.get_pixel(x+1, y+1),
    // ]
    todo!()
}

pub fn convolve_3x3_unrolled(image: &Image, kernel: &Kernel) -> Image {
    // TODO: Fully unrolled 3×3 convolution
    //
    // assert_eq!(kernel.size, 3);
    //
    // let mut result = Image::new(image.width, image.height);
    //
    // // Convert kernel to fixed array (compiler can optimize better)
    // let k: [f32; 9] = [
    //     kernel.data[0], kernel.data[1], kernel.data[2],
    //     kernel.data[3], kernel.data[4], kernel.data[5],
    //     kernel.data[6], kernel.data[7], kernel.data[8],
    // ];
    //
    // for y in 1..image.height - 1 {
    //     for x in 1..image.width - 1 {
    //         // Load pixels
    //         let pixels = load_3x3_neighborhood(image, x, y);
    //
    //         // Fully unrolled multiplication
    //         let r = pixels[0].0 as f32 * k[0] +
    //                 pixels[1].0 as f32 * k[1] +
    //                 pixels[2].0 as f32 * k[2] +
    //                 pixels[3].0 as f32 * k[3] +
    //                 pixels[4].0 as f32 * k[4] +
    //                 pixels[5].0 as f32 * k[5] +
    //                 pixels[6].0 as f32 * k[6] +
    //                 pixels[7].0 as f32 * k[7] +
    //                 pixels[8].0 as f32 * k[8];
    //
    //         // Same for g, b channels
    //
    //         result.set_pixel(x, y, (r as u8, g as u8, b as u8));
    //     }
    // }
    //
    // result
    todo!()
}

pub fn convolve_5x5_unrolled(image: &Image, kernel: &Kernel) -> Image {
    // TODO: Unrolled 5×5 (25 operations)
    // More unrolling = more ILP but larger code size
    todo!()
}

#[inline(always)]
fn convolve_pixel_optimized(
    pixels: &[(u8, u8, u8); 9],
    kernel: &[f32; 9]
) -> (u8, u8, u8) {
    // TODO: Optimized pixel convolution
    // Keep everything in registers
    //
    // Unroll all 9 multiplications
    // Compiler should use FMA (fused multiply-add) instructions
    todo!()
}

// Compiler hints
#[cold]
#[inline(never)]
fn handle_edge_case(x: i32, y: i32, width: usize, height: usize) -> (u8, u8, u8) {
    // TODO: Mark edge cases as unlikely
    // #[cold] tells compiler this path is rare
    todo!()
}
```

---

## Milestone 5: Branch-Free Programming

### Introduction

**Why Milestone 4 Is Not Enough:**
Even with loop unrolling, we still have branches for:
1. Bounds checking (clipping at image edges)
2. Clamping values (0-255 range)
3. Edge case handling

**Branch Misprediction Cost:**
```
Pipeline depth: 15-20 stages
Mispredicted branch: Flush entire pipeline = 15-20 cycles wasted

For 1920×1080 image:
- 2,073,600 pixels
- ~10% at edges (need bounds check)
- ~200,000 branches
- 10% misprediction rate
- 20,000 mispredictions × 20 cycles = 400,000 cycles wasted
```

**What We're Improving:**
- Replace if/else with branchless alternatives
- Use arithmetic instead of conditionals
- Leverage CPU's conditional move (CMOV) instructions
- Pre-compute edge conditions

**Branchless Techniques:**
```rust
// Branchy clamp:
if value < 0.0 {
    0
} else if value > 255.0 {
    255
} else {
    value as u8
}

// Branchless clamp:
value.max(0.0).min(255.0) as u8

// Or using bit tricks:
let clamped = (value as i32) & !((value as i32) >> 31);
let clamped = clamped.min(255);
```

**Expected Improvement:** 1.5-2x speedup (depends on branch miss rate)

### Architecture

**Branchless Functions:**
- `clamp_branchless(value: f32) -> u8` - No branches for clamping
- `compute_with_padding(image: &PaddedImage, ...)` - Pre-padded image eliminates bounds checks
- `select_branchless(condition: bool, a: u8, b: u8) -> u8` - Conditional select without branch

**Padding Strategy:**
```
Instead of:
  if x < 0 || x >= width { edge_pixel } else { image[x] }

Pre-pad image:
  [EDGE|  IMAGE  |EDGE]
  Now always safe to access without checks!
```

**Role Each Plays:**
- Branchless clamp: Eliminate range checking branches
- Padding: Eliminate bounds checking branches
- Conditional moves: Hardware-level branch elimination
- Look-up tables: Replace complex conditionals

### Checkpoint Tests

```rust
#[test]
fn test_clamp_branchless() {
    assert_eq!(clamp_branchless(-10.0), 0);
    assert_eq!(clamp_branchless(100.0), 100);
    assert_eq!(clamp_branchless(300.0), 255);
}

#[test]
fn test_padded_image() {
    let img = Image::new(100, 100);
    let padded = create_padded_image(&img, 1);

    // Padded image should be larger
    assert_eq!(padded.width, 102);
    assert_eq!(padded.height, 102);

    // Can safely access edges without bounds checking
    let (r, g, b) = padded.get_pixel(0, 0);
    let (r, g, b) = padded.get_pixel(101, 101);
}

#[test]
fn test_branchless_performance() {
    use std::time::Instant;

    let img = Image::new(1920, 1080);
    let kernel = Kernel::gaussian_blur(3, 1.0);

    // With branches
    let start = Instant::now();
    let _ = convolve_3x3_unrolled(&img, &kernel);
    let branch_time = start.elapsed();

    // Branchless
    let start = Instant::now();
    let _ = convolve_branchless(&img, &kernel);
    let branchless_time = start.elapsed();

    println!("With branches: {:?}", branch_time);
    println!("Branchless:    {:?}", branchless_time);
    println!("Speedup: {:.2}x", branch_time.as_secs_f64() / branchless_time.as_secs_f64());

    assert!(branchless_time < branch_time);
}

#[test]
fn test_branch_statistics() {
    // Use perf stat to measure branch misses
    // perf stat -e branches,branch-misses ./program
}

#[test]
fn test_select_branchless() {
    assert_eq!(select_branchless(true, 10, 20), 10);
    assert_eq!(select_branchless(false, 10, 20), 20);
}
```

### Starter Code

```rust
#[inline(always)]
pub fn clamp_branchless(value: f32) -> u8 {
    // TODO: Branchless clamp to 0-255
    //
    // Option 1: Use min/max (compiler generates CMOV)
    // value.max(0.0).min(255.0) as u8
    //
    // Option 2: Bit tricks
    // let v = value as i32;
    // let clamped = v & !((v >> 31)); // Clamp to 0
    // (clamped.min(255)) as u8
    //
    // Option 3: Saturating cast (if available)
    todo!()
}

#[inline(always)]
pub fn select_branchless(condition: bool, a: u8, b: u8) -> u8 {
    // TODO: Branchless conditional select
    //
    // Compiler should generate CMOV instruction
    // if condition { a } else { b }
    //
    // Or manual:
    // let mask = -(condition as i8) as u8;
    // (a & mask) | (b & !mask)
    todo!()
}

pub struct PaddedImage {
    data: Vec<u8>,
    width: usize,
    height: usize,
    padding: usize,
}

pub fn create_padded_image(image: &Image, padding: usize) -> PaddedImage {
    // TODO: Create image with border padding
    //
    // Eliminates need for bounds checking!
    //
    // let padded_width = image.width + 2 * padding;
    // let padded_height = image.height + 2 * padding;
    // let mut data = vec![0u8; padded_width * padded_height * 3];
    //
    // // Copy image into center
    // for y in 0..image.height {
    //     for x in 0..image.width {
    //         let src_idx = (y * image.width + x) * 3;
    //         let dst_idx = ((y + padding) * padded_width + (x + padding)) * 3;
    //         data[dst_idx..dst_idx+3].copy_from_slice(&image.data[src_idx..src_idx+3]);
    //     }
    // }
    //
    // // Replicate edges for padding (or use mirror/wrap)
    // // ...
    //
    // PaddedImage {
    //     data,
    //     width: padded_width,
    //     height: padded_height,
    //     padding,
    // }
    todo!()
}

pub fn convolve_branchless(image: &Image, kernel: &Kernel) -> Image {
    // TODO: Convolution with no branches
    //
    // Strategy:
    // 1. Pre-pad image to eliminate bounds checks
    // 2. Use branchless clamp
    // 3. Fully unrolled loops (no loop branches)
    //
    // let padded = create_padded_image(image, kernel.size / 2);
    // let mut result = Image::new(image.width, image.height);
    //
    // // No bounds checking needed!
    // for y in 0..image.height {
    //     for x in 0..image.width {
    //         let r = convolve_pixel_branchless(&padded, x, y, kernel);
    //         result.set_pixel(x, y, r);
    //     }
    // }
    //
    // result
    todo!()
}

#[inline(always)]
fn convolve_pixel_branchless(
    image: &PaddedImage,
    x: usize,
    y: usize,
    kernel: &Kernel
) -> (u8, u8, u8) {
    // TODO: Pixel convolution with zero branches
    //
    // No bounds checks (pre-padded)
    // Branchless clamp
    // Fully unrolled loop
    todo!()
}
```

---

## Milestone 6: Assembly and SIMD Intrinsics

### Introduction

**Why Milestone 5 Is Not Enough:**
Even with all optimizations, the compiler may not generate optimal code. Modern CPUs have powerful SIMD instructions (SSE, AVX, AVX-512) that process 4-16 values simultaneously, but the compiler doesn't always use them optimally.

**SIMD Potential:**
```
Scalar: Process 1 pixel at a time
SSE:    Process 4 pixels at a time (4×f32)
AVX2:   Process 8 pixels at a time (8×f32)
AVX-512: Process 16 pixels at a time (16×f32)
```

**What We're Improving:**
- Use explicit SIMD intrinsics (AVX2)
- Hand-optimized assembly for critical kernels
- Vectorize convolution operations
- Fused multiply-add (FMA) instructions

**Expected Improvement:** 1.5-2x speedup (total 50-100x over naive!)

### Architecture

**Dependencies:**
```toml
[dependencies]
# For portable SIMD
packed_simd = "0.3"
```

**SIMD Structures:**
- Use `__m256` (AVX) or `f32x8` (portable_simd)
- Process 8 pixels in parallel

**Key Functions:**
- `convolve_avx2(image: &Image, kernel: &Kernel) -> Image` - AVX2 intrinsics
- `convolve_asm(image: &Image, kernel: &Kernel) -> Image` - Inline assembly (optional)

**AVX2 Pattern:**
```rust
unsafe {
    let pixel_vec = _mm256_loadu_ps(pixel_ptr);
    let kernel_vec = _mm256_set1_ps(kernel_value);
    let result = _mm256_fmadd_ps(pixel_vec, kernel_vec, accumulator);
}
```

### Checkpoint Tests

```rust
#[test]
fn test_simd_correctness() {
    let img = Image::new(256, 256);
    let kernel = Kernel::gaussian_blur(3, 1.0);

    let branchless = convolve_branchless(&img, &kernel);
    let simd = convolve_avx2(&img, &kernel);

    for y in 1..255 {
        for x in 1..255 {
            let (r1, g1, b1) = branchless.get_pixel(x, y);
            let (r2, g2, b2) = simd.get_pixel(x, y);

            assert!((r1 as i32 - r2 as i32).abs() <= 2);
        }
    }
}

#[test]
fn benchmark_final() {
    use std::time::Instant;

    let img = Image::new(3840, 2160); // 4K
    let kernel = Kernel::gaussian_blur(3, 1.0);

    println!("\n=== Final Benchmark (4K image: 3840×2160) ===\n");

    // Naive
    let start = Instant::now();
    let _ = naive_convolve(&img, &kernel);
    let naive_time = start.elapsed();
    let naive_mpx = (3840.0 * 2160.0) / (naive_time.as_secs_f64() * 1_000_000.0);
    println!("Naive:       {:?} ({:.2} Mpixels/sec)", naive_time, naive_mpx);

    // Stack
    let mut buffer = ImageBuffer::with_capacity(3840, 2160);
    let start = Instant::now();
    buffer.convolve_into(&img, &kernel);
    let stack_time = start.elapsed();
    let stack_mpx = (3840.0 * 2160.0) / (stack_time.as_secs_f64() * 1_000_000.0);
    println!("Stack:       {:?} ({:.2} Mpixels/sec, {:.1}x)",
        stack_time, stack_mpx, naive_time.as_secs_f64() / stack_time.as_secs_f64());

    // Tiled
    let start = Instant::now();
    let _ = tiled_convolve(&img, &kernel);
    let tiled_time = start.elapsed();
    let tiled_mpx = (3840.0 * 2160.0) / (tiled_time.as_secs_f64() * 1_000_000.0);
    println!("Tiled:       {:?} ({:.2} Mpixels/sec, {:.1}x)",
        tiled_time, tiled_mpx, naive_time.as_secs_f64() / tiled_time.as_secs_f64());

    // Unrolled
    let start = Instant::now();
    let _ = convolve_3x3_unrolled(&img, &kernel);
    let unroll_time = start.elapsed();
    let unroll_mpx = (3840.0 * 2160.0) / (unroll_time.as_secs_f64() * 1_000_000.0);
    println!("Unrolled:    {:?} ({:.2} Mpixels/sec, {:.1}x)",
        unroll_time, unroll_mpx, naive_time.as_secs_f64() / unroll_time.as_secs_f64());

    // Branchless
    let start = Instant::now();
    let _ = convolve_branchless(&img, &kernel);
    let branch_time = start.elapsed();
    let branch_mpx = (3840.0 * 2160.0) / (branch_time.as_secs_f64() * 1_000_000.0);
    println!("Branchless:  {:?} ({:.2} Mpixels/sec, {:.1}x)",
        branch_time, branch_mpx, naive_time.as_secs_f64() / branch_time.as_secs_f64());

    // SIMD
    let start = Instant::now();
    let _ = convolve_avx2(&img, &kernel);
    let simd_time = start.elapsed();
    let simd_mpx = (3840.0 * 2160.0) / (simd_time.as_secs_f64() * 1_000_000.0);
    println!("AVX2:        {:?} ({:.2} Mpixels/sec, {:.1}x)",
        simd_time, simd_mpx, naive_time.as_secs_f64() / simd_time.as_secs_f64());

    println!("\nTotal speedup: {:.1}x", naive_time.as_secs_f64() / simd_time.as_secs_f64());
}
```

### Starter Code

```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
pub fn convolve_avx2(image: &Image, kernel: &Kernel) -> Image {
    // TODO: AVX2-optimized convolution
    //
    // unsafe {
    //     let mut result = Image::new(image.width, image.height);
    //     let padded = create_padded_image(image, 1);
    //
    //     // Kernel as SIMD vector
    //     let k0 = _mm256_set1_ps(kernel.data[0]);
    //     let k1 = _mm256_set1_ps(kernel.data[1]);
    //     // ... etc
    //
    //     for y in 0..image.height {
    //         for x in (0..image.width).step_by(8) {
    //             // Load 8 pixels at once
    //             let pixels = _mm256_loadu_ps(
    //                 padded.data.as_ptr().add((y * padded.width + x) * 3) as *const f32
    //             );
    //
    //             // Vectorized multiply-add
    //             let mut sum = _mm256_setzero_ps();
    //             sum = _mm256_fmadd_ps(pixels, k0, sum);
    //             // ... accumulate all 9 kernel elements
    //
    //             // Store result
    //             _mm256_storeu_ps(
    //                 result.data.as_mut_ptr().add((y * image.width + x) * 3) as *mut f32,
    //                 sum
    //             );
    //         }
    //     }
    //
    //     result
    // }
    todo!()
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn convolve_8_pixels_avx2(
    pixels: &[u8],
    kernel: &[f32; 9],
) -> __m256 {
    // TODO: Convolve 8 pixels using AVX2
    //
    // 1. Load 8 RGB triplets
    // 2. Convert u8 to f32 (using _mm256_cvtepi32_ps)
    // 3. Multiply by kernel weights
    // 4. Horizontal sum
    // 5. Return result vector
    todo!()
}

// Optional: Inline assembly for ultimate control
#[cfg(target_arch = "x86_64")]
pub fn convolve_asm(image: &Image, kernel: &Kernel) -> Image {
    // TODO: Hand-written assembly for hot path
    //
    // use std::arch::asm;
    //
    // unsafe {
    //     let mut result: f32;
    //     asm!(
    //         "vmulps {result}, {pixel}, {kernel}",
    //         "vaddps {result}, {result}, {acc}",
    //         pixel = in(xmm_reg) pixel_vec,
    //         kernel = in(xmm_reg) kernel_vec,
    //         acc = in(xmm_reg) accumulator,
    //         result = out(xmm_reg) result,
    //     );
    // }
    todo!()
}
```

---

## Complete Working Example

```rust
use std::time::Instant;

// ============================================================================
// IMAGE STRUCTURE
// ============================================================================

#[derive(Debug, Clone)]
pub struct Image {
    data: Vec<u8>,
    width: usize,
    height: usize,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![0u8; width * height * 3],
            width,
            height,
        }
    }

    pub fn from_data(data: Vec<u8>, width: usize, height: usize) -> Self {
        assert_eq!(data.len(), width * height * 3);
        Self { data, width, height }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> (u8, u8, u8) {
        let idx = (y * self.width + x) * 3;
        (self.data[idx], self.data[idx + 1], self.data[idx + 2])
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        let idx = (y * self.width + x) * 3;
        self.data[idx] = rgb.0;
        self.data[idx + 1] = rgb.1;
        self.data[idx + 2] = rgb.2;
    }
}

// ============================================================================
// KERNEL
// ============================================================================

#[derive(Debug, Clone)]
pub struct Kernel {
    data: Vec<f32>,
    size: usize,
}

impl Kernel {
    pub fn gaussian_blur(size: usize, sigma: f32) -> Self {
        let mut data = vec![0.0; size * size];
        let center = (size / 2) as i32;

        for y in 0..size {
            for x in 0..size {
                let dx = x as i32 - center;
                let dy = y as i32 - center;
                let dist_sq = (dx * dx + dy * dy) as f32;
                data[y * size + x] = (-dist_sq / (2.0 * sigma * sigma)).exp();
            }
        }

        let sum: f32 = data.iter().sum();
        for val in data.iter_mut() {
            *val /= sum;
        }

        Self { data, size }
    }
}

// ============================================================================
// NAIVE IMPLEMENTATION
// ============================================================================

pub fn naive_convolve(image: &Image, kernel: &Kernel) -> Image {
    let mut result = Image::new(image.width, image.height);
    let offset = (kernel.size / 2) as i32;

    for y in 0..image.height {
        for x in 0..image.width {
            let mut r_sum = 0.0;
            let mut g_sum = 0.0;
            let mut b_sum = 0.0;

            for ky in 0..kernel.size {
                for kx in 0..kernel.size {
                    let img_x = x as i32 + kx as i32 - offset;
                    let img_y = y as i32 + ky as i32 - offset;

                    if img_x >= 0 && img_x < image.width as i32 &&
                       img_y >= 0 && img_y < image.height as i32 {

                        let (r, g, b) = image.get_pixel(img_x as usize, img_y as usize);
                        let weight = kernel.data[ky * kernel.size + kx];

                        r_sum += r as f32 * weight;
                        g_sum += g as f32 * weight;
                        b_sum += b as f32 * weight;
                    }
                }
            }

            let r = r_sum.max(0.0).min(255.0) as u8;
            let g = g_sum.max(0.0).min(255.0) as u8;
            let b = b_sum.max(0.0).min(255.0) as u8;

            result.set_pixel(x, y, (r, g, b));
        }
    }

    result
}

// ============================================================================
// OPTIMIZED IMPLEMENTATION (Combined)
// ============================================================================

const TILE_SIZE: usize = 64;

#[inline(always)]
fn clamp_branchless(value: f32) -> u8 {
    value.max(0.0).min(255.0) as u8
}

#[inline(always)]
fn load_3x3(image: &Image, x: usize, y: usize) -> [(u8, u8, u8); 9] {
    [
        image.get_pixel(x.wrapping_sub(1), y.wrapping_sub(1)),
        image.get_pixel(x, y.wrapping_sub(1)),
        image.get_pixel(x + 1, y.wrapping_sub(1)),
        image.get_pixel(x.wrapping_sub(1), y),
        image.get_pixel(x, y),
        image.get_pixel(x + 1, y),
        image.get_pixel(x.wrapping_sub(1), y + 1),
        image.get_pixel(x, y + 1),
        image.get_pixel(x + 1, y + 1),
    ]
}

pub fn optimized_convolve(image: &Image, kernel: &Kernel) -> Image {
    assert_eq!(kernel.size, 3);

    let mut result = Image::new(image.width, image.height);

    let k: [f32; 9] = [
        kernel.data[0], kernel.data[1], kernel.data[2],
        kernel.data[3], kernel.data[4], kernel.data[5],
        kernel.data[6], kernel.data[7], kernel.data[8],
    ];

    for tile_y in (1..image.height - 1).step_by(TILE_SIZE) {
        for tile_x in (1..image.width - 1).step_by(TILE_SIZE) {
            let end_y = (tile_y + TILE_SIZE).min(image.height - 1);
            let end_x = (tile_x + TILE_SIZE).min(image.width - 1);

            for y in tile_y..end_y {
                for x in tile_x..end_x {
                    let pixels = load_3x3(image, x, y);

                    let r = pixels[0].0 as f32 * k[0] +
                            pixels[1].0 as f32 * k[1] +
                            pixels[2].0 as f32 * k[2] +
                            pixels[3].0 as f32 * k[3] +
                            pixels[4].0 as f32 * k[4] +
                            pixels[5].0 as f32 * k[5] +
                            pixels[6].0 as f32 * k[6] +
                            pixels[7].0 as f32 * k[7] +
                            pixels[8].0 as f32 * k[8];

                    let g = pixels[0].1 as f32 * k[0] +
                            pixels[1].1 as f32 * k[1] +
                            pixels[2].1 as f32 * k[2] +
                            pixels[3].1 as f32 * k[3] +
                            pixels[4].1 as f32 * k[4] +
                            pixels[5].1 as f32 * k[5] +
                            pixels[6].1 as f32 * k[6] +
                            pixels[7].1 as f32 * k[7] +
                            pixels[8].1 as f32 * k[8];

                    let b = pixels[0].2 as f32 * k[0] +
                            pixels[1].2 as f32 * k[1] +
                            pixels[2].2 as f32 * k[2] +
                            pixels[3].2 as f32 * k[3] +
                            pixels[4].2 as f32 * k[4] +
                            pixels[5].2 as f32 * k[5] +
                            pixels[6].2 as f32 * k[6] +
                            pixels[7].2 as f32 * k[7] +
                            pixels[8].2 as f32 * k[8];

                    result.set_pixel(x, y, (
                        clamp_branchless(r),
                        clamp_branchless(g),
                        clamp_branchless(b),
                    ));
                }
            }
        }
    }

    result
}

// ============================================================================
// BENCHMARKING
// ============================================================================

fn main() {
    println!("=== CPU Optimization Benchmark ===\n");

    for &(width, height) in &[(640, 480), (1920, 1080), (3840, 2160)] {
        println!("Image size: {}×{}", width, height);

        let img = Image::new(width, height);
        let kernel = Kernel::gaussian_blur(3, 1.0);

        // Naive
        let start = Instant::now();
        let _ = naive_convolve(&img, &kernel);
        let naive_time = start.elapsed();
        let naive_mpx = (width * height) as f64 / (naive_time.as_secs_f64() * 1_000_000.0);
        println!("  Naive:     {:?} ({:.2} Mpixels/sec)", naive_time, naive_mpx);

        // Optimized
        let start = Instant::now();
        let _ = optimized_convolve(&img, &kernel);
        let opt_time = start.elapsed();
        let opt_mpx = (width * height) as f64 / (opt_time.as_secs_f64() * 1_000_000.0);
        println!("  Optimized: {:?} ({:.2} Mpixels/sec, {:.1}x speedup)",
            opt_time, opt_mpx, naive_time.as_secs_f64() / opt_time.as_secs_f64());

        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correctness() {
        let img = Image::new(100, 100);
        let kernel = Kernel::gaussian_blur(3, 1.0);

        let naive = naive_convolve(&img, &kernel);
        let optimized = optimized_convolve(&img, &kernel);

        for y in 1..99 {
            for x in 1..99 {
                let (r1, g1, b1) = naive.get_pixel(x, y);
                let (r2, g2, b2) = optimized.get_pixel(x, y);

                assert!((r1 as i32 - r2 as i32).abs() <= 1);
                assert!((g1 as i32 - g2 as i32).abs() <= 1);
                assert!((b1 as i32 - b2 as i32).abs() <= 1);
            }
        }
    }
}
```

This completes the comprehensive CPU optimization project covering heap/stack, cache, registers, branch prediction, and assembly!
