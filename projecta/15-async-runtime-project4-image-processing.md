
# Concurrent Image Processing Pipeline

### Problem Statement

Build a concurrent image processing pipeline that processes images from a directory (or multiple directories) with various transformations: resizing, format conversion, applying filters (grayscale, blur, brightness), generating thumbnails, and watermarking. The system must process images concurrently using async I/O, handle different image formats, provide progress tracking, support cancellation, and gracefully handle errors (corrupted images, disk full, etc.).

### Use Cases

- **Thumbnail generation for photo galleries** - Generate multiple sizes for responsive web images
- **Batch photo editing** - Apply filters/corrections to hundreds of photos at once
- **E-commerce product images** - Resize, watermark, optimize images for web display
- **Social media platforms** - Process user uploads (compress, resize, detect inappropriate content)
- **Medical imaging** - Batch process X-rays, MRIs with normalization and enhancement
- **Real estate listings** - Watermark, resize, optimize property photos
- **Photography studios** - Batch export RAW photos to multiple formats
- **Content delivery networks** - Generate image variants for different devices

### Why It Matters

**I/O vs CPU Bound**: Image processing is CPU-intensive (pixel operations), but loading/saving is I/O-bound. Async I/O overlaps disk reads with processing. Example: Loading 100 images at 50ms each = 5s sequential. With async I/O + CPU processing overlap, total time approaches max(load_time, process_time) instead of sum.

**Concurrency Wins**: Sequential processing of 1000 images at 200ms each = 200 seconds (3.3 minutes). With 8-core CPU and thread pool: 1000 ÷ 8 × 200ms = 25 seconds (8x faster). Proper concurrency saturates CPU cores.

**Memory Management**: Loading all 1000 images (5MB each) = 5GB RAM. Streaming pipeline processes batches (e.g., 10 at a time) = 50MB RAM. Memory efficiency enables processing datasets larger than RAM.

**Backpressure**: Fast image loading can overwhelm slow processing. Without backpressure, queue grows unbounded → OOM. Bounded channels naturally slow down readers when processors are busy.

Example performance numbers:
```
Sequential (1 core):        1000 images × 200ms = 200s
Parallel (8 cores):         1000 images ÷ 8 × 200ms = 25s (8x speedup)
With async I/O overlap:     ~20s (I/O happens during CPU work)

Memory usage:
  Load all:                 1000 × 5MB = 5GB
  Stream batches (10):      10 × 5MB = 50MB (100x reduction)
```

---

## Key Concepts Explained

This project requires understanding of Image Processing and Rust concepts for concurrent I/O, CPU-bound processing, and pipeline architectures. These concepts enable building high-throughput image processing systems that efficiently utilize both CPU and I/O resources.

### Convolution Algorithms in Image Processing

**What is Convolution?** A mathematical operation that applies a small matrix (kernel) to every pixel in an image, producing effects like blur, sharpen, edge detection, and more.

#### The Mathematics of Convolution

**Core Concept**: Slide a small matrix (kernel) over the image, multiply overlapping values, sum them, and replace the center pixel.

**Visual Example**:

```
Original Image (grayscale values):
┌────────────────────┐
│ 10  20  30  40  50 │
│ 15  25  35  45  55 │
│ 20  30  40  50  60 │
│ 25  35  45  55  65 │
│ 30  40  50  60  70 │
└────────────────────┘

3×3 Kernel (simple blur):
┌──────────┐
│ 1  1  1  │  ← Each value is 1/9
│ 1  1  1  │  ← (sum = 1, maintains brightness)
│ 1  1  1  │
└──────────┘
Divide by 9 after sum

Process:
1. Place kernel over top-left 3×3 region
2. Multiply each kernel value by corresponding pixel
3. Sum all products
4. Divide by 9 (normalization)
5. This becomes the new center pixel value
6. Slide kernel one pixel right, repeat
```

**Step-by-Step Calculation**:

```rust
// Position: Center at pixel [1,1] (value 25)
// Surrounding region:
// 10  20  30
// 15  25  35
// 20  30  40

// Apply kernel:
new_value = (
    10*1 + 20*1 + 30*1 +
    15*1 + 25*1 + 35*1 +
    20*1 + 30*1 + 40*1
) / 9

new_value = (10 + 20 + 30 + 15 + 25 + 35 + 20 + 30 + 40) / 9
new_value = 225 / 9 = 25

// Result: Pixel at [1,1] becomes 25 (averaged with neighbors)
```

**Algorithm**:

```rust
fn convolve(image: &Image, kernel: &Kernel) -> Image {
    let mut output = Image::new(image.width, image.height);

    // For each pixel in image (excluding borders)
    for y in 1..(image.height - 1) {
        for x in 1..(image.width - 1) {
            let mut sum = 0.0;

            // Apply kernel
            for ky in 0..kernel.height {
                for kx in 0..kernel.width {
                    let pixel_x = x + kx - kernel.width / 2;
                    let pixel_y = y + ky - kernel.height / 2;

                    let pixel_value = image.get_pixel(pixel_x, pixel_y);
                    let kernel_value = kernel.get(kx, ky);

                    sum += pixel_value * kernel_value;
                }
            }

            output.set_pixel(x, y, sum);
        }
    }

    output
}
```

---

#### Common Convolution Kernels

**1. Box Blur (Average)**

```
Kernel:
┌───────────┐
│ 1  1  1  │
│ 1  1  1  │  × 1/9
│ 1  1  1  │
└───────────┘

Effect: Smooth blur, reduces noise
Use case: Fast blur, noise reduction
Cost: O(n) per pixel (9 operations for 3×3)
```

**2. Gaussian Blur (Weighted Average)**

```
Kernel:
┌───────────┐
│ 1  2  1  │
│ 2  4  2  │  × 1/16
│ 1  2  1  │
└───────────┘

Effect: Natural blur, preserves edges better than box blur
Use case: Photo editing, anti-aliasing
Cost: O(n) per pixel, same as box blur but better quality
```

**3. Sharpen**

```
Kernel:
┌────────────┐
│  0  -1   0 │
│ -1   5  -1 │
│  0  -1   0 │
└────────────┘

Effect: Enhances edges, increases contrast
Use case: Make blurry photos crisper
How it works: Emphasizes difference between pixel and neighbors
```

**4. Edge Detection (Sobel X)**

```
Kernel:
┌────────────┐
│ -1   0   1 │
│ -2   0   2 │
│ -1   0   1 │
└────────────┘

Effect: Detects vertical edges
Use case: Computer vision, object detection
Result: High values where brightness changes horizontally
```

**5. Edge Detection (Sobel Y)**

```
Kernel:
┌────────────┐
│ -1  -2  -1 │
│  0   0   0 │
│  1   2   1 │
└────────────┘

Effect: Detects horizontal edges
Use case: Computer vision, object detection
Combined with Sobel X: magnitude = sqrt(x² + y²)
```

**6. Emboss**

```
Kernel:
┌────────────┐
│ -2  -1   0 │
│ -1   1   1 │
│  0   1   2 │
└────────────┘

Effect: 3D-like raised effect
Use case: Artistic filters
```

**7. Identity (No Change)**

```
Kernel:
┌────────────┐
│  0   0   0 │
│  0   1   0 │
│  0   0   0 │
└────────────┘

Effect: Original image unchanged
Use case: Testing, baseline
```

---

#### Visual Example: Blur in Action

```
Original 5×5 image:
┌─────────────────────┐
│  0   0   0   0   0  │
│  0   0   0   0   0  │
│  0   0  255  0   0  │  ← Single bright pixel
│  0   0   0   0   0  │
│  0   0   0   0   0  │
└─────────────────────┘

After 3×3 Box Blur:
┌─────────────────────┐
│  0   0   0   0   0  │
│  0  28  28  28   0  │  ← Blur spread out
│  0  28  28  28   0  │
│  0  28  28  28   0  │
│  0   0   0   0   0  │
└─────────────────────┘

Calculation for pixel [1,1]:
= (0 + 0 + 0 + 0 + 255 + 0 + 0 + 0 + 0) / 9
= 255 / 9 ≈ 28

Effect: Single bright pixel "blurs" into neighboring pixels
```

---

#### Performance Characteristics

**Computational Cost**:

```rust
// For an N×N image with K×K kernel:
// Total operations = N² × K²

// Example: 1920×1080 image (2MP) with 3×3 kernel
let pixels = 1920 * 1080;           // 2,073,600 pixels
let ops_per_pixel = 3 * 3;          // 9 multiply-adds
let total_ops = pixels * ops_per_pixel;  // 18,662,400 operations

// For RGB image (3 channels):
let rgb_ops = total_ops * 3;       // 55,987,200 operations

// At 1 GFLOP/s (1 billion ops/sec):
let time_ms = rgb_ops / 1_000_000; // ~56ms per image

// For 1000 images:
// Sequential: 1000 × 56ms = 56 seconds
// Parallel (8 cores): 56s / 8 = 7 seconds
```

**Why Convolution is CPU-Intensive**:

1. **Nested loops**: O(N² × K²) complexity
   ```rust
   for y in image.height {         // N iterations
       for x in image.width {      // N iterations
           for ky in kernel.height { // K iterations
               for kx in kernel.width { // K iterations
                   // Multiply-add operation
               }
           }
       }
   }
   // Total: N × N × K × K operations
   ```

2. **Memory access patterns**: Poor cache locality
   ```
   Kernel slides across image:
   Row 0: [████████████████] Sequential reads (cache-friendly)
   Row 1: [████████████████] Sequential reads
   Row 2: [████████████████] Sequential reads

   But accessing neighboring pixels requires jumping in memory:
   Pixel [100, 100] → Pixel [100, 101]: +1 byte (good)
   Pixel [100, 100] → Pixel [101, 100]: +width bytes (cache miss!)
   ```

3. **Floating-point operations**: Multiply-add is expensive
   ```
   Integer multiply: ~1 cycle
   Float multiply: ~3-5 cycles
   Division/normalization: ~10-20 cycles
   ```

**Optimization Strategies**:

**1. Separable Filters** (Huge Speedup for Gaussian/Box Blur):

```rust
// Standard 2D convolution: O(N² × K²)
fn convolve_2d(image: &Image, kernel: &[[f32; 3]; 3]) { /* ... */ }

// Separable convolution: O(N² × K) - much faster!
// Gaussian blur can be split into horizontal then vertical pass

// Horizontal pass:
let kernel_h = [1.0, 2.0, 1.0];  // 1×3 kernel
for y in 0..height {
    for x in 0..width {
        temp[y][x] = image[y][x-1] * 1.0 + image[y][x] * 2.0 + image[y][x+1] * 1.0;
    }
}

// Vertical pass:
let kernel_v = [1.0, 2.0, 1.0];  // 3×1 kernel
for y in 0..height {
    for x in 0..width {
        output[y][x] = temp[y-1][x] * 1.0 + temp[y][x] * 2.0 + temp[y+1][x] * 1.0;
    }
}

// Speedup: 3×3 kernel = 9 ops → 3+3 = 6 ops (33% faster)
//          5×5 kernel = 25 ops → 5+5 = 10 ops (60% faster!)
```

**2. SIMD (Single Instruction Multiple Data)**:

```rust
// Without SIMD: Process 1 pixel at a time
for x in 0..width {
    output[x] = input[x] * kernel[0] + input[x+1] * kernel[1] + input[x+2] * kernel[2];
}
// Throughput: 1 pixel per iteration

// With SIMD (AVX2): Process 8 pixels at once
use std::arch::x86_64::*;

for x in (0..width).step_by(8) {
    let v0 = _mm256_loadu_ps(&input[x]);      // Load 8 pixels
    let v1 = _mm256_loadu_ps(&input[x+1]);
    let v2 = _mm256_loadu_ps(&input[x+2]);

    let k0 = _mm256_set1_ps(kernel[0]);       // Broadcast kernel value
    let k1 = _mm256_set1_ps(kernel[1]);
    let k2 = _mm256_set1_ps(kernel[2]);

    let r0 = _mm256_mul_ps(v0, k0);           // Multiply 8 pixels
    let r1 = _mm256_mul_ps(v1, k1);
    let r2 = _mm256_mul_ps(v2, k2);

    let sum = _mm256_add_ps(_mm256_add_ps(r0, r1), r2);  // Add 8 pixels
    _mm256_storeu_ps(&mut output[x], sum);    // Store 8 pixels
}
// Throughput: 8 pixels per iteration (8x faster!)
```

**3. Parallel Processing with rayon**:

```rust
use rayon::prelude::*;

// Process each row in parallel
let output: Vec<Vec<f32>> = (0..height)
    .into_par_iter()  // Parallel iterator
    .map(|y| {
        let mut row = vec![0.0; width];
        for x in 0..width {
            row[x] = convolve_pixel(&image, kernel, x, y);
        }
        row
    })
    .collect();

// With 8 cores: ~8x speedup
```

**Combined Optimizations**:

```
Baseline: 1920×1080 RGB image, 5×5 Gaussian blur
Sequential:                  200ms
+ Separable filter:          120ms (1.7x faster)
+ SIMD (AVX2):                15ms (13x faster)
+ Parallel (8 cores):          2ms (100x faster total!)
```

---

#### Real-World Implementation Example

```rust
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb};
use rayon::prelude::*;

#[derive(Clone)]
struct Kernel {
    data: Vec<Vec<f32>>,
    width: usize,
    height: usize,
    divisor: f32,  // Normalization factor
}

impl Kernel {
    fn box_blur() -> Self {
        Self {
            data: vec![
                vec![1.0, 1.0, 1.0],
                vec![1.0, 1.0, 1.0],
                vec![1.0, 1.0, 1.0],
            ],
            width: 3,
            height: 3,
            divisor: 9.0,
        }
    }

    fn gaussian_blur() -> Self {
        Self {
            data: vec![
                vec![1.0, 2.0, 1.0],
                vec![2.0, 4.0, 2.0],
                vec![1.0, 2.0, 1.0],
            ],
            width: 3,
            height: 3,
            divisor: 16.0,
        }
    }

    fn sharpen() -> Self {
        Self {
            data: vec![
                vec![ 0.0, -1.0,  0.0],
                vec![-1.0,  5.0, -1.0],
                vec![ 0.0, -1.0,  0.0],
            ],
            width: 3,
            height: 3,
            divisor: 1.0,
        }
    }

    fn edge_detect() -> Self {
        Self {
            data: vec![
                vec![-1.0, -1.0, -1.0],
                vec![-1.0,  8.0, -1.0],
                vec![-1.0, -1.0, -1.0],
            ],
            width: 3,
            height: 3,
            divisor: 1.0,
        }
    }
}

fn apply_kernel(image: &DynamicImage, kernel: &Kernel) -> DynamicImage {
    let (width, height) = image.dimensions();
    let rgb_image = image.to_rgb8();

    // Process in parallel by row
    let output_data: Vec<Vec<Rgb<u8>>> = (1..(height - 1))
        .into_par_iter()
        .map(|y| {
            let mut row = Vec::with_capacity(width as usize);

            for x in 1..(width - 1) {
                let mut r_sum = 0.0;
                let mut g_sum = 0.0;
                let mut b_sum = 0.0;

                // Apply kernel
                for ky in 0..kernel.height {
                    for kx in 0..kernel.width {
                        let px = x + kx as u32 - kernel.width as u32 / 2;
                        let py = y + ky as u32 - kernel.height as u32 / 2;

                        let pixel = rgb_image.get_pixel(px, py);
                        let k_val = kernel.data[ky][kx];

                        r_sum += pixel[0] as f32 * k_val;
                        g_sum += pixel[1] as f32 * k_val;
                        b_sum += pixel[2] as f32 * k_val;
                    }
                }

                // Normalize and clamp
                let r = (r_sum / kernel.divisor).clamp(0.0, 255.0) as u8;
                let g = (g_sum / kernel.divisor).clamp(0.0, 255.0) as u8;
                let b = (b_sum / kernel.divisor).clamp(0.0, 255.0) as u8;

                row.push(Rgb([r, g, b]));
            }

            row
        })
        .collect();

    // Reconstruct image from rows
    let mut output = ImageBuffer::new(width, height);
    for (y, row) in output_data.iter().enumerate() {
        for (x, pixel) in row.iter().enumerate() {
            output.put_pixel(x as u32 + 1, y as u32 + 1, *pixel);
        }
    }

    DynamicImage::ImageRgb8(output)
}

// Usage in pipeline:
async fn process_with_filter(image: DynamicImage, filter: &str) -> DynamicImage {
    let kernel = match filter {
        "blur" => Kernel::gaussian_blur(),
        "sharpen" => Kernel::sharpen(),
        "edge" => Kernel::edge_detect(),
        _ => Kernel::box_blur(),
    };

    // This is CPU-bound, runs on thread pool automatically
    tokio::task::spawn_blocking(move || {
        apply_kernel(&image, &kernel)
    })
    .await
    .unwrap()
}
```

---

#### Why This Matters for Concurrent Pipelines

**1. CPU Saturation**:
- Convolution uses 100% CPU per core
- Without parallelism: 1 core at 100%, 7 cores idle
- With rayon: All 8 cores at 100% → 8x throughput

**2. Memory Bandwidth**:
- 1920×1080 RGB = 6.2 MB per image
- Loading from memory: ~10 GB/s bandwidth
- Processing: ~600 images/second per core (theoretical)
- Bottleneck shifts from CPU to memory bandwidth at scale

**3. Pipeline Balance**:
```
[Load 10ms I/O] → [Decode 20ms CPU] → [Convolve 50ms CPU] → [Encode 30ms CPU] → [Save 10ms I/O]

Convolution is the slowest stage!
- Need to batch or parallelize this stage
- 8 parallel convolvers can match throughput of other stages
```

**4. Caching Effects**:
- Small kernels (3×3): Good cache locality, ~2ms per image
- Large kernels (15×15): Cache misses, ~15ms per image
- Separable filters: Better cache usage, 2-3x faster

Understanding convolution is critical for optimizing image processing pipelines—it's where most CPU time is spent!

---


### Async I/O vs Blocking I/O

**The Fundamental Difference**: Blocking I/O wastes CPU time waiting. Async I/O allows other work while waiting for disk/network.

**Blocking I/O (std::fs)**:

```rust
use std::fs;

// Thread blocks here for entire read duration
let data = fs::read("/path/to/image.jpg")?;  // 10ms disk read
// CPU does NOTHING for 10ms

// Reading 100 images sequentially:
for path in image_paths {
    let data = fs::read(path)?;  // 10ms each
    process(data);
}
// Total: 100 × 10ms = 1000ms of blocked CPU time
```

**Async I/O (tokio::fs)**:

```rust
use tokio::fs;

// Initiates read, immediately returns a Future
let data = fs::read("/path/to/image.jpg").await;  // Suspends, CPU free
// CPU can do other work while disk reads

// Reading 100 images concurrently:
let futures: Vec<_> = image_paths
    .iter()
    .map(|path| fs::read(path))
    .collect();

let results = futures::future::join_all(futures).await;
// All reads happen in parallel (limited by disk/OS)
// Total: ~10-50ms (disk parallelism limits)
```

**How tokio::fs Works Internally**:

```
tokio::fs::read(path).await
  ↓
1. Creates Future representing the read
2. Registers with OS (epoll/kqueue/IOCP)
3. Returns control to runtime (CPU free)
4. OS performs read in background
5. OS notifies runtime when ready
6. Runtime polls future again
7. Returns data
```

**Performance Comparison**:

```
100 images, 10ms disk latency each:

Blocking (std::fs):
  Thread 1: [████████████████████████████] 1000ms
  CPU utilization: 0% (waiting on I/O)

Async (tokio::fs):
  Reads: [████] 50ms (limited by disk parallelism)
  CPU can process other tasks during reads
  CPU utilization: Can approach 100% with proper pipelining
```

**When to Use Each**:

| Aspect | std::fs (Blocking) | tokio::fs (Async) |
|--------|-------------------|-------------------|
| **Use case** | Single file, sync context | Many files, async context |
| **CPU efficiency** | Poor (blocks) | Good (overlaps I/O) |
| **Complexity** | Simple | More complex |
| **Thread usage** | 1 thread = 1 I/O op | 1 thread = many I/O ops |
| **Throughput** | Low | High |

---

### CPU-Bound vs I/O-Bound Work

Understanding the difference is critical for optimal concurrency.

**I/O-Bound Work**: Limited by disk/network speed, not CPU.

```rust
// I/O-bound: Waiting for disk
async fn load_image(path: &Path) -> Result<Vec<u8>, Error> {
    tokio::fs::read(path).await  // CPU mostly idle
}

// Best concurrency: async/await (overlaps waiting)
let futures = paths.iter().map(load_image);
let images = join_all(futures).await;  // Efficient!
```

**CPU-Bound Work**: Limited by CPU speed, not I/O.

```rust
// CPU-bound: Heavy computation
fn resize_image(image: DynamicImage, size: u32) -> DynamicImage {
    // Processes millions of pixels - pure CPU work
    image.resize(size, size, FilterType::Lanczos3)  // CPU at 100%
}

// Best concurrency: thread pool (parallel CPU work)
use rayon::prelude::*;
let resized: Vec<_> = images
    .par_iter()  // Parallel iterator
    .map(|img| resize_image(img.clone(), 800))
    .collect();  // Uses all CPU cores
```

**Hybrid Workload: Image Processing Pipeline**

Image processing combines both:
1. **Load** (I/O-bound): Read from disk
2. **Decode** (CPU-bound): Decompress JPEG/PNG
3. **Process** (CPU-bound): Resize, filter, transform
4. **Encode** (CPU-bound): Compress to output format
5. **Save** (I/O-bound): Write to disk

**Wrong Approach** (Sequential):
```rust
// Total time = sum of all stages
for path in paths {
    let data = load(path).await;        // 10ms I/O
    let img = decode(data);             // 20ms CPU
    let processed = resize(img);        // 50ms CPU
    let encoded = encode(processed);    // 30ms CPU
    save(encoded).await;                // 10ms I/O
}
// Per image: 120ms
// 100 images: 12,000ms = 12 seconds
```

**Right Approach** (Pipeline):
```rust
// Stages run in parallel, overlap I/O and CPU

[Load] → [Decode] → [Process] → [Encode] → [Save]
  ↓         ↓          ↓           ↓          ↓
Image1   Image2     Image3      Image4     Image5

// Throughput: limited by slowest stage (50ms processing)
// 100 images: ~5 seconds (2.4x faster!)
```

---


### Channels and Backpressure

Channels connect pipeline stages, but unbounded queues cause memory explosion.

**The Problem: Unbounded Queues**

```rust
// BAD: Unbounded channel
let (tx, rx) = mpsc::unbounded_channel();

// Fast loader
tokio::spawn(async move {
    for path in 10000 paths {
        let data = load(path).await;  // Fast: 1ms each
        tx.send(data).unwrap();
    }
});

// Slow processor
tokio::spawn(async move {
    while let Some(data) = rx.recv().await {
        process(data);  // Slow: 100ms each
    }
});

// Queue grows: 10,000 images × 5MB = 50GB in memory!
// OOM crash or swapping → system unusable
```

**The Solution: Bounded Channels (Backpressure)**

```rust
// GOOD: Bounded channel with capacity
let (tx, rx) = mpsc::channel(10);  // Max 10 images in queue

// Fast loader
tokio::spawn(async move {
    for path in 10000_paths {
        let data = load(path).await;
        tx.send(data).await;  // Blocks when queue full!
        // Loader slows down to match processor speed
    }
});

// Slow processor
tokio::spawn(async move {
    while let Some(data) = rx.recv().await {
        process(data);  // Slow: 100ms
    }
});

// Queue stays at 10 images × 5MB = 50MB (constant!)
// Loader naturally throttles when processor can't keep up
```

**Backpressure Flow Control**:

```
Without backpressure:
Loader: ████████████████████████ (fast, unbounded)
Queue:  [1][2][3][4]...[9999][10000] (grows forever)
Processor: ████ (slow, overwhelmed)

With backpressure (capacity=10):
Loader: ████░░░░████░░░░████ (blocks when queue full)
Queue:  [1][2]...[10] (bounded, 10 max)
Processor: ████████████████ (steady throughput)

Loader adapts to processor speed automatically!
```

**Choosing Channel Capacity**:

```rust
// Too small (capacity=1): Excessive blocking, poor throughput
// Too large (capacity=10000): No backpressure, memory issues
// Sweet spot: 2-10x processing time / load time

// Example calculation:
// Load time: 10ms
// Process time: 100ms
// Ratio: 100/10 = 10
// Good capacity: 10-20 images

let (tx, rx) = mpsc::channel(15);  // Balances memory and throughput
```

---

### Streaming and Batching

**Streaming**: Process items one-by-one as they arrive.

```rust
// Stream processing
async fn process_stream(mut rx: Receiver<Image>) {
    while let Some(image) = rx.recv().await {
        let result = process_one(image);  // Process immediately
        save(result).await;
    }
}

// Pros:
// - Constant memory (one item at a time)
// - Low latency (start immediately)
// - Simple pipeline

// Cons:
// - Can't amortize costs
// - No batch optimizations
```

**Batching**: Collect N items, process together.

```rust
// Batch processing
async fn process_batches(mut rx: Receiver<Image>) {
    let mut batch = Vec::new();

    while let Some(image) = rx.recv().await {
        batch.push(image);

        if batch.len() >= 10 {
            // Process batch in parallel
            let results: Vec<_> = batch
                .par_iter()  // rayon parallel iterator
                .map(|img| process_one(img))
                .collect();

            save_all(results).await;
            batch.clear();
        }
    }

    // Don't forget remaining items
    if !batch.is_empty() {
        let results: Vec<_> = batch.par_iter().map(process_one).collect();
        save_all(results).await;
    }
}

// Pros:
// - Parallel processing (use all cores)
// - Amortized I/O costs (batch writes)
// - Better CPU utilization

// Cons:
// - Higher memory (batch in memory)
// - Higher latency (wait for batch)
```

**Adaptive Batching**:

```rust
use tokio::time::{timeout, Duration};

async fn process_adaptive_batches(mut rx: Receiver<Image>) {
    let mut batch = Vec::new();
    const MAX_BATCH: usize = 20;
    const MAX_WAIT: Duration = Duration::from_millis(100);

    loop {
        // Collect up to MAX_BATCH items or wait MAX_WAIT
        let deadline = tokio::time::Instant::now() + MAX_WAIT;

        while batch.len() < MAX_BATCH {
            match timeout_at(deadline, rx.recv()).await {
                Ok(Some(image)) => batch.push(image),
                Ok(None) => break,  // Channel closed
                Err(_) => break,    // Timeout - process what we have
            }
        }

        if batch.is_empty() {
            break;  // No more items
        }

        // Process batch
        process_and_save_batch(&batch).await;
        batch.clear();
    }
}

// Adaptive: batches under high load, streams under low load
```

---

### Thread Pools for CPU-Bound Work

Async is great for I/O, but CPU-bound work needs real parallelism.

**The Problem with async for CPU**:

```rust
// This WON'T use multiple cores:
let futures: Vec<_> = images
    .iter()
    .map(|img| async { resize_image(img) })  // CPU work
    .collect();

join_all(futures).await;

// All run on same thread! No parallelism.
// CPU: [██████████░░░░░░░░░░] (1 core at 100%, 7 idle)
```

**The Solution: rayon (Work-Stealing Thread Pool)**:

```rust
use rayon::prelude::*;

let resized: Vec<_> = images
    .par_iter()  // Parallel iterator
    .map(|img| resize_image(img))  // CPU work
    .collect();

// Uses all cores automatically!
// CPU: [██████████][██████████][██████████][██████████]
//      Core 1      Core 2      Core 3      Core 4
```

**How rayon Works**:

```
Work-stealing algorithm:

Thread 1: [Task1][Task2][Task3]     ← Busy
Thread 2: [Task4]                   ← Done early, steals from Thread 1
Thread 3: [Task5][Task6]            ← Busy
Thread 4: []                        ← Idle, steals from others

Result: Balanced load across all cores
```

**Combining async I/O + rayon CPU**:

```rust
// Perfect hybrid: async I/O + parallel CPU
async fn process_pipeline(paths: Vec<PathBuf>) {
    // Stage 1: Async load (I/O-bound)
    let futures = paths.iter().map(|path| tokio::fs::read(path));
    let file_data = join_all(futures).await;

    // Stage 2: Parallel decode (CPU-bound)
    let images: Vec<_> = file_data
        .par_iter()
        .filter_map(|data| image::load_from_memory(data).ok())
        .collect();

    // Stage 3: Parallel resize (CPU-bound)
    let resized: Vec<_> = images
        .par_iter()
        .map(|img| img.resize(800, 800, FilterType::Lanczos3))
        .collect();

    // Stage 4: Parallel encode (CPU-bound)
    let encoded: Vec<_> = resized
        .par_iter()
        .map(|img| encode_jpeg(img, 85))
        .collect();

    // Stage 5: Async save (I/O-bound)
    let save_futures = encoded.iter().zip(&paths).map(|(data, path)| {
        tokio::fs::write(path, data)
    });
    join_all(save_futures).await;
}

// Result: Maximum CPU and I/O utilization!
```

---

### Progress Tracking with Atomics

**The Requirement**: Show progress without slowing down the pipeline.

**Wrong Approach: Mutex/RwLock**:

```rust
// BAD: Lock contention slows pipeline
let progress = Arc::new(Mutex::new(0));

// Every worker contends for lock
for _ in 0..1000 {
    let progress = Arc::clone(&progress);
    tokio::spawn(async move {
        process_image().await;
        *progress.lock().unwrap() += 1;  // Lock! Contention!
    });
}

// With 100 workers, lock becomes bottleneck
// Throughput drops 10-50%
```

**Right Approach: Atomic Counters**:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

// GOOD: Lock-free atomic increment
let progress = Arc::new(AtomicUsize::new(0));
let total = 1000;

for _ in 0..total {
    let progress = Arc::clone(&progress);
    tokio::spawn(async move {
        process_image().await;
        progress.fetch_add(1, Ordering::Relaxed);  // Lock-free!
    });
}

// Monitor progress
tokio::spawn(async move {
    loop {
        let current = progress.load(Ordering::Relaxed);
        println!("Progress: {}/{} ({:.1}%)",
            current, total, (current as f64 / total as f64) * 100.0);

        if current >= total {
            break;
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
});

// No contention, minimal overhead (~2ns per update)
```

**Multi-Metric Progress**:

```rust
struct ProgressMetrics {
    total: AtomicUsize,
    completed: AtomicUsize,
    failed: AtomicUsize,
    bytes_processed: AtomicUsize,
}

impl ProgressMetrics {
    fn new(total: usize) -> Self {
        Self {
            total: AtomicUsize::new(total),
            completed: AtomicUsize::new(0),
            failed: AtomicUsize::new(0),
            bytes_processed: AtomicUsize::new(0),
        }
    }

    fn record_success(&self, bytes: usize) {
        self.completed.fetch_add(1, Ordering::Relaxed);
        self.bytes_processed.fetch_add(bytes, Ordering::Relaxed);
    }

    fn record_failure(&self) {
        self.failed.fetch_add(1, Ordering::Relaxed);
    }

    fn report(&self) -> String {
        let total = self.total.load(Ordering::Relaxed);
        let completed = self.completed.load(Ordering::Relaxed);
        let failed = self.failed.load(Ordering::Relaxed);
        let bytes = self.bytes_processed.load(Ordering::Relaxed);

        format!(
            "Completed: {}/{}, Failed: {}, Processed: {:.2} MB",
            completed, total, failed, bytes as f64 / 1_000_000.0
        )
    }
}
```

---

### Error Handling in Pipelines

**The Challenge**: One corrupted image shouldn't crash entire pipeline.

**Strategy 1: Fail Fast**:

```rust
// Stop on first error
async fn process_all_strict(paths: Vec<PathBuf>) -> Result<Vec<Image>, Error> {
    let mut results = Vec::new();

    for path in paths {
        let image = load_and_process(&path).await?;  // Propagates error
        results.push(image);
    }

    Ok(results)
}

// Use when: Data integrity critical, can't tolerate partial results
```

**Strategy 2: Collect Errors**:

```rust
// Process all, return both successes and errors
async fn process_all_resilient(paths: Vec<PathBuf>)
    -> (Vec<Image>, Vec<(PathBuf, Error)>)
{
    let mut successes = Vec::new();
    let mut errors = Vec::new();

    for path in paths {
        match load_and_process(&path).await {
            Ok(image) => successes.push(image),
            Err(e) => errors.push((path, e)),
        }
    }

    (successes, errors)
}

// Use when: Partial results acceptable, want to know what failed
```

**Strategy 3: Retry with Circuit Breaker**:

```rust
struct CircuitBreaker {
    failures: AtomicUsize,
    max_failures: usize,
}

impl CircuitBreaker {
    fn is_open(&self) -> bool {
        self.failures.load(Ordering::Relaxed) >= self.max_failures
    }

    fn record_failure(&self) {
        self.failures.fetch_add(1, Ordering::Relaxed);
    }
}

async fn process_with_circuit_breaker(
    paths: Vec<PathBuf>,
    circuit: &CircuitBreaker,
) -> Vec<Result<Image, Error>> {
    let mut results = Vec::new();

    for path in paths {
        if circuit.is_open() {
            results.push(Err(Error::CircuitOpen));
            continue;
        }

        match retry_with_backoff(|| load_and_process(&path), 3).await {
            Ok(img) => results.push(Ok(img)),
            Err(e) => {
                circuit.record_failure();
                results.push(Err(e));
            }
        }
    }

    results
}

// Use when: Transient failures expected, want to prevent cascade failures
```

**Pipeline Error Handling**:

```rust
async fn resilient_pipeline(paths: Vec<PathBuf>) -> PipelineResult {
    let metrics = Arc::new(ProgressMetrics::new(paths.len()));
    let (tx, mut rx) = mpsc::channel(20);

    // Producer: Load images
    let loader_metrics = Arc::clone(&metrics);
    tokio::spawn(async move {
        for path in paths {
            match load_image(&path).await {
                Ok(img) => {
                    if tx.send(Ok(img)).await.is_err() {
                        break;  // Receiver dropped
                    }
                }
                Err(e) => {
                    loader_metrics.record_failure();
                    eprintln!("Failed to load {:?}: {}", path, e);
                    // Continue with other images
                }
            }
        }
    });

    // Consumer: Process images
    let processor_metrics = Arc::clone(&metrics);
    tokio::spawn(async move {
        while let Some(result) = rx.recv().await {
            match result {
                Ok(img) => {
                    match process_image(img).await {
                        Ok(size) => processor_metrics.record_success(size),
                        Err(e) => {
                            processor_metrics.record_failure();
                            eprintln!("Processing failed: {}", e);
                        }
                    }
                }
                Err(_) => {}  // Already logged
            }
        }
    });

    metrics
}

// Errors are logged but don't stop the pipeline
```

---

### Connection to This Project

Now that you understand the core concepts, here's how they map to the milestones:

**Milestone 1: Async Image Loading**
- **Concepts Used**: tokio::fs async I/O, PathBuf, directory traversal
- **Why**: Overlap disk I/O for multiple images, don't block CPU while waiting
- **Key Insight**: `tokio::fs::read_dir()` + `join_all()` loads many images concurrently

**Milestone 2: Image Decoding and Processing**
- **Concepts Used**: image crate, DynamicImage, CPU-bound work
- **Why**: Decode/resize are CPU-intensive, need real parallelism
- **Key Insight**: Use rayon for parallel decoding/processing across all cores

**Milestone 3: Pipeline with Channels**
- **Concepts Used**: mpsc bounded channels, backpressure, producer-consumer
- **Why**: Stream images through stages without loading all into memory
- **Key Insight**: Bounded channels naturally throttle fast stages to match slow ones

**Milestone 4: Batched Processing**
- **Concepts Used**: Batching, rayon par_iter, adaptive timeouts
- **Why**: Process multiple images in parallel for better CPU utilization
- **Key Insight**: Batch size trades off latency vs throughput

**Milestone 5: Progress Tracking**
- **Concepts Used**: AtomicUsize, lock-free counters, periodic reporting
- **Why**: Monitor pipeline without slowing it down
- **Key Insight**: Atomics avoid lock contention that would bottleneck high-throughput pipeline

**Milestone 6: Error Handling and Resilience**
- **Concepts Used**: Result propagation, partial results, circuit breakers
- **Why**: One bad image shouldn't crash entire batch
- **Key Insight**: Collect errors separately, continue processing good images

**Putting It All Together**:

The complete pipeline combines all concepts:
1. **Async I/O** loads images without blocking
2. **Channels** connect stages with backpressure
3. **Thread pools** parallelize CPU-bound work
4. **Batching** optimizes throughput
5. **Atomics** track progress without locks
6. **Error handling** ensures resilience

This architecture achieves:
- **High throughput**: Saturates both CPU and I/O
- **Bounded memory**: Processes datasets larger than RAM
- **Observability**: Real-time progress updates
- **Resilience**: Gracefully handles errors

Each milestone builds incrementally toward a production-ready image processing system.

---

## Milestone 1: Async Image Loading from Directory

### Introduction

Before processing images, you need to load them efficiently. This milestone teaches async file I/O, directory traversal, and handling different image formats.

**Why Start Here**: Blocking I/O (std::fs) blocks the entire thread. With 100 images on a slow disk (10ms each), that's 1 second of wasted CPU time. Async I/O (tokio::fs) allows other work while waiting for disk.

### Architecture

**Structs:**
- `ImageFile` - Represents an image file
  - **Field** `path: PathBuf` - File system path
  - **Field** `filename: String` - Just the filename
  - **Field** `size_bytes: u64` - File size
  - **Field** `format: ImageFormat` - Detected format (JPEG, PNG, etc.)

- `ImageFormat` - Supported image formats
  - **Variant** `Jpeg`, `Png`, `Gif`, `WebP`, `Bmp`, `Unknown`

- `ImageData` - Loaded image with metadata
  - **Field** `file: ImageFile` - Source file info
  - **Field** `data: Vec<u8>` - Raw image bytes
  - **Field** `width: u32` - Image width in pixels
  - **Field** `height: u32` - Image height in pixels

**Key Functions:**
- `async fn scan_directory(path: &Path) -> Result<Vec<ImageFile>, String>` - Finds all images in directory
- `async fn load_image(file: &ImageFile) -> Result<ImageData, String>` - Loads image into memory
- `fn detect_format(path: &Path) -> ImageFormat` - Determines format from extension
- `async fn get_image_dimensions(data: &[u8]) -> Result<(u32, u32), String>` - Reads width/height

**Role Each Plays:**
- **tokio::fs**: Async file operations (read_dir, read)
- **PathBuf**: Cross-platform file path handling
- **image crate**: Decoding/encoding various image formats
- **DynamicImage**: In-memory representation supporting multiple formats

### Checkpoint Tests

```rust
#[tokio::test]
async fn test_detect_format() {
    use std::path::Path;

    assert_eq!(detect_format(Path::new("photo.jpg")), ImageFormat::Jpeg);
    assert_eq!(detect_format(Path::new("IMAGE.JPEG")), ImageFormat::Jpeg);
    assert_eq!(detect_format(Path::new("icon.png")), ImageFormat::Png);
    assert_eq!(detect_format(Path::new("anim.gif")), ImageFormat::Gif);
    assert_eq!(detect_format(Path::new("photo.webp")), ImageFormat::WebP);
    assert_eq!(detect_format(Path::new("unknown.txt")), ImageFormat::Unknown);
}

#[tokio::test]
async fn test_scan_directory() {
    // Create test directory with sample images
    tokio::fs::create_dir_all("test_images").await.unwrap();

    // Create dummy image files for testing
    tokio::fs::write("test_images/test1.jpg", b"fake jpg data").await.unwrap();
    tokio::fs::write("test_images/test2.png", b"fake png data").await.unwrap();
    tokio::fs::write("test_images/readme.txt", b"not an image").await.unwrap();

    let images = scan_directory(Path::new("test_images")).await.unwrap();

    // Should find 2 images, skip txt file
    assert_eq!(images.len(), 2);
    assert!(images.iter().any(|img| img.filename == "test1.jpg"));
    assert!(images.iter().any(|img| img.filename == "test2.png"));

    // Cleanup
    tokio::fs::remove_dir_all("test_images").await.unwrap();
}

#[tokio::test]
async fn test_load_image() {
    // Create a real test image (1x1 red pixel PNG)
    let png_data: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // Width=1, Height=1
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
        0xDE, // IHDR end
        0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, // IDAT chunk
        0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00,
        0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB4,
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND
        0xAE, 0x42, 0x60, 0x82,
    ];

    tokio::fs::create_dir_all("test_images").await.unwrap();
    tokio::fs::write("test_images/test.png", &png_data).await.unwrap();

    let file = ImageFile {
        path: PathBuf::from("test_images/test.png"),
        filename: "test.png".to_string(),
        size_bytes: png_data.len() as u64,
        format: ImageFormat::Png,
    };

    let loaded = load_image(&file).await.unwrap();

    assert_eq!(loaded.width, 1);
    assert_eq!(loaded.height, 1);
    assert!(loaded.data.len() > 0);

    // Cleanup
    tokio::fs::remove_dir_all("test_images").await.unwrap();
}

#[tokio::test]
async fn test_concurrent_scanning() {
    // Test scanning multiple directories concurrently
    tokio::fs::create_dir_all("test_dir1").await.unwrap();
    tokio::fs::create_dir_all("test_dir2").await.unwrap();

    tokio::fs::write("test_dir1/a.jpg", b"data").await.unwrap();
    tokio::fs::write("test_dir2/b.png", b"data").await.unwrap();

    let (result1, result2) = tokio::join!(
        scan_directory(Path::new("test_dir1")),
        scan_directory(Path::new("test_dir2"))
    );

    assert_eq!(result1.unwrap().len(), 1);
    assert_eq!(result2.unwrap().len(), 1);

    // Cleanup
    tokio::fs::remove_dir_all("test_dir1").await.unwrap();
    tokio::fs::remove_dir_all("test_dir2").await.unwrap();
}
```

### Starter Code

```rust
use tokio::fs;
use std::path::{Path, PathBuf};
use image::{self, DynamicImage, ImageFormat as ImgFormat, GenericImageView};

#[derive(Debug, Clone, PartialEq)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Gif,
    WebP,
    Bmp,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ImageFile {
    pub path: PathBuf,
    pub filename: String,
    pub size_bytes: u64,
    pub format: ImageFormat,
}

#[derive(Debug)]
pub struct ImageData {
    pub file: ImageFile,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub fn detect_format(path: &Path) -> ImageFormat {
    // TODO: Get file extension
    // TODO: Match extension to ImageFormat
    // Hint: path.extension().and_then(|s| s.to_str())

    todo!("Implement format detection")
}

pub async fn scan_directory(path: &Path) -> Result<Vec<ImageFile>, String> {
    // TODO: Read directory entries with tokio::fs::read_dir
    // TODO: Filter for files (not directories)
    // TODO: Check if extension is an image format
    // TODO: Get file metadata (size)
    // TODO: Build ImageFile structs

    todo!("Implement directory scanning")
}

pub async fn load_image(file: &ImageFile) -> Result<ImageData, String> {
    // TODO: Read file bytes with tokio::fs::read
    // TODO: Use image::load_from_memory to decode
    // TODO: Get dimensions with img.dimensions()
    // TODO: Return ImageData

    todo!("Implement image loading")
}

pub async fn get_image_dimensions(data: &[u8]) -> Result<(u32, u32), String> {
    // TODO: Load image from bytes
    // TODO: Return (width, height)

    todo!("Implement dimension reading")
}

#[tokio::main]
async fn main() {
    println!("=== Image Directory Scanner ===\n");

    let directory = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".".to_string());

    println!("Scanning directory: {}", directory);

    match scan_directory(Path::new(&directory)).await {
        Ok(images) => {
            println!("Found {} images:", images.len());
            for img in images.iter().take(10) {
                println!(
                    "  {} - {} ({} bytes)",
                    img.filename,
                    format!("{:?}", img.format),
                    img.size_bytes
                );
            }

            if images.len() > 10 {
                println!("  ... and {} more", images.len() - 10);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

**Implementation Hints:**
1. Extension matching: `match ext.to_lowercase().as_str() { "jpg" | "jpeg" => ImageFormat::Jpeg, ... }`
2. Read directory: `let mut entries = fs::read_dir(path).await?;`
3. Iterate entries: `while let Some(entry) = entries.next_entry().await? { ... }`
4. Get metadata: `entry.metadata().await?.len()`
5. Load image: `image::load_from_memory(&data).map_err(|e| e.to_string())?`

---

## Milestone 2: Basic Image Transformations

### Introduction

**Why Milestone 1 Isn't Enough**: Loading images is just the first step. Real applications need transformations—resize for thumbnails, grayscale for previews, rotation for mobile photos.

**The Improvement**: Implement common image operations using the `image` crate. These are CPU-bound operations that will benefit from parallelism in later milestones.

**Optimization**: Image operations are pixel-parallel. A 1000×1000 image has 1M pixels—each can be processed independently. Later we'll parallelize, but first we need the operations.

### Architecture

**Structs:**
- `ImageTransform` - Enum of available transformations
  - **Variant** `Resize { width: u32, height: u32 }` - Scale to exact dimensions
  - **Variant** `Thumbnail { max_size: u32 }` - Fit within square, maintain aspect ratio
  - **Variant** `Grayscale` - Convert to black and white
  - **Variant** `Blur { sigma: f32 }` - Gaussian blur
  - **Variant** `Brighten { value: i32 }` - Adjust brightness (+/- 0-255)
  - **Variant** `Rotate90`, `Rotate180`, `Rotate270` - Rotation
  - **Variant** `FlipHorizontal`, `FlipVertical` - Mirroring

- `ProcessedImage` - Result of transformation
  - **Field** `original: ImageFile` - Source file
  - **Field** `image: DynamicImage` - Processed image
  - **Field** `transforms: Vec<ImageTransform>` - Applied transformations

**Key Functions:**
- `fn apply_transform(img: DynamicImage, transform: &ImageTransform) -> DynamicImage` - Applies single transformation
- `fn apply_transforms(img: DynamicImage, transforms: &[ImageTransform]) -> DynamicImage` - Chains multiple transforms
- `async fn process_image(data: ImageData, transforms: Vec<ImageTransform>) -> ProcessedImage` - Full processing pipeline

**Role Each Plays:**
- **DynamicImage**: In-memory image supporting various pixel formats
- **Transform chain**: Composable operations (resize then grayscale then blur)
- **image crate methods**: resize_exact, grayscale, blur, brighten, rotate90, fliph, flipv

### Checkpoint Tests

```rust
#[test]
fn test_apply_resize() {
    use image::{DynamicImage, RgbaImage};

    let img = DynamicImage::ImageRgba8(RgbaImage::new(100, 100));

    let transform = ImageTransform::Resize {
        width: 50,
        height: 50,
    };

    let result = apply_transform(img, &transform);

    assert_eq!(result.width(), 50);
    assert_eq!(result.height(), 50);
}

#[test]
fn test_apply_thumbnail() {
    use image::{DynamicImage, RgbaImage};

    let img = DynamicImage::ImageRgba8(RgbaImage::new(200, 100));

    let transform = ImageTransform::Thumbnail { max_size: 50 };

    let result = apply_transform(img, &transform);

    // Should fit within 50x50, maintaining aspect ratio
    assert!(result.width() <= 50);
    assert!(result.height() <= 50);
    // Aspect ratio should be preserved (2:1)
    assert_eq!(result.width(), 50);
    assert_eq!(result.height(), 25);
}

#[test]
fn test_apply_grayscale() {
    use image::{DynamicImage, Rgb, RgbImage};

    let mut img = RgbImage::new(10, 10);
    // Set a red pixel
    img.put_pixel(5, 5, Rgb([255, 0, 0]));

    let colored = DynamicImage::ImageRgb8(img);
    let gray = apply_transform(colored, &ImageTransform::Grayscale);

    // Grayscale images should have R=G=B for each pixel
    let pixel = gray.as_rgb8().unwrap().get_pixel(5, 5);
    assert_eq!(pixel[0], pixel[1]);
    assert_eq!(pixel[1], pixel[2]);
}

#[test]
fn test_apply_multiple_transforms() {
    use image::{DynamicImage, RgbaImage};

    let img = DynamicImage::ImageRgba8(RgbaImage::new(100, 100));

    let transforms = vec![
        ImageTransform::Resize {
            width: 50,
            height: 50,
        },
        ImageTransform::Grayscale,
        ImageTransform::Rotate90,
    ];

    let result = apply_transforms(img, &transforms);

    // After rotate90, dimensions swap
    assert_eq!(result.width(), 50);
    assert_eq!(result.height(), 50);
}

#[tokio::test]
async fn test_process_image() {
    // Create simple test image data
    let img = image::RgbaImage::new(100, 100);
    let dynamic = DynamicImage::ImageRgba8(img);
    let mut buffer = Vec::new();

    dynamic
        .write_to(&mut std::io::Cursor::new(&mut buffer), ImgFormat::Png)
        .unwrap();

    let file = ImageFile {
        path: PathBuf::from("test.png"),
        filename: "test.png".to_string(),
        size_bytes: buffer.len() as u64,
        format: ImageFormat::Png,
    };

    let data = ImageData {
        file: file.clone(),
        data: buffer,
        width: 100,
        height: 100,
    };

    let transforms = vec![ImageTransform::Thumbnail { max_size: 32 }];

    let processed = process_image(data, transforms).await;

    assert!(processed.image.width() <= 32);
    assert!(processed.image.height() <= 32);
}
```

### Starter Code

```rust
use image::{DynamicImage, imageops, ImageBuffer};

#[derive(Debug, Clone)]
pub enum ImageTransform {
    Resize { width: u32, height: u32 },
    Thumbnail { max_size: u32 },
    Grayscale,
    Blur { sigma: f32 },
    Brighten { value: i32 },
    Rotate90,
    Rotate180,
    Rotate270,
    FlipHorizontal,
    FlipVertical,
}

#[derive(Debug)]
pub struct ProcessedImage {
    pub original: ImageFile,
    pub image: DynamicImage,
    pub transforms: Vec<ImageTransform>,
}

pub fn apply_transform(img: DynamicImage, transform: &ImageTransform) -> DynamicImage {
    // TODO: Match on transform variant
    // TODO: Apply corresponding image operation
    // Hints:
    //   - Resize: img.resize_exact(w, h, FilterType::Lanczos3)
    //   - Thumbnail: img.thumbnail(max, max)
    //   - Grayscale: img.grayscale()
    //   - Blur: img.blur(sigma)
    //   - Brighten: img.brighten(value)
    //   - Rotate: img.rotate90(), rotate180(), rotate270()
    //   - Flip: img.fliph(), flipv()

    todo!("Implement transform application")
}

pub fn apply_transforms(mut img: DynamicImage, transforms: &[ImageTransform]) -> DynamicImage {
    // TODO: Fold over transforms, applying each one
    // Hint: for transform in transforms { img = apply_transform(img, transform); }

    todo!("Implement transform chain")
}

pub async fn process_image(
    data: ImageData,
    transforms: Vec<ImageTransform>,
) -> ProcessedImage {
    // TODO: Load DynamicImage from data.data bytes
    // TODO: Apply all transforms
    // TODO: Return ProcessedImage

    todo!("Implement image processing")
}

#[tokio::main]
async fn main() {
    use std::io::Cursor;

    // Example: Load and process an image
    let images = scan_directory(Path::new("./sample_images")).await.unwrap();

    if let Some(first) = images.first() {
        println!("Processing: {}", first.filename);

        let data = load_image(first).await.unwrap();
        println!("Loaded: {}x{}", data.width, data.height);

        let transforms = vec![
            ImageTransform::Thumbnail { max_size: 200 },
            ImageTransform::Grayscale,
        ];

        let processed = process_image(data, transforms).await;

        println!(
            "Processed: {}x{}",
            processed.image.width(),
            processed.image.height()
        );

        // Save result
        processed
            .image
            .save("output_thumbnail.png")
            .expect("Failed to save");
        println!("Saved to output_thumbnail.png");
    }
}
```

**Implementation Hints:**
1. For resize: `img.resize_exact(width, height, image::imageops::FilterType::Lanczos3)`
2. For thumbnail: `img.thumbnail(max_size, max_size)` (maintains aspect ratio)
3. Most operations return new DynamicImage, enabling chaining
4. Load from bytes: `image::load_from_memory(&data.data)?`
5. Save: `img.save(path)?` or `img.write_to(&mut writer, format)?`

---

## Milestone 3: Concurrent Image Processing with Worker Pool

### Introduction

**Why Milestone 2 Isn't Enough**: Processing images sequentially is slow. Each image takes 100-500ms depending on size and operations. With 1000 images, that's 2-8 minutes.

**The Improvement**: Create a worker pool using `tokio::spawn` to process multiple images concurrently. Use CPU threads for processing (tokio's multi-threaded runtime or rayon).

**Optimization (Parallelism)**: Image processing is CPU-bound and parallelizable. An 8-core CPU can process 8 images simultaneously, achieving ~8x speedup. The key is saturating all cores without overwhelming RAM.

### Architecture

**Structs:**
- `ProcessingConfig` - Configuration for processing pipeline
  - **Field** `worker_count: usize` - Concurrent workers
  - **Field** `batch_size: usize` - Images per batch
  - **Field** `transforms: Vec<ImageTransform>` - Operations to apply

- `ProcessingResult` - Result of processing one image
  - **Field** `original_path: PathBuf` - Source file
  - **Field** `success: bool` - Whether processing succeeded
  - **Field** `output_path: Option<PathBuf>` - Where result was saved
  - **Field** `error: Option<String>` - Error message if failed
  - **Field** `duration: Duration` - Processing time

- `ProcessingStats` - Aggregate statistics
  - **Field** `total_processed: AtomicUsize` - Images completed
  - **Field** `total_failed: AtomicUsize` - Images failed
  - **Field** `total_duration: AtomicU64` - Sum of processing times (ms)

**Key Functions:**
- `async fn process_image_worker(data: ImageData, transforms: Vec<ImageTransform>, output_dir: PathBuf) -> ProcessingResult` - Worker function
- `async fn process_directory_concurrent(input_dir: &Path, output_dir: &Path, config: ProcessingConfig) -> ProcessingStats` - Main pipeline
- `async fn save_processed_image(img: &DynamicImage, path: &Path, format: ImageFormat) -> Result<(), String>` - Async save

**Role Each Plays:**
- **tokio::spawn**: Spawns async task onto runtime
- **Semaphore**: Limits concurrent workers (prevents spawning 1000 tasks at once)
- **mpsc channel**: Distributes work to workers, collects results
- **Arc<ProcessingStats>**: Shared statistics across workers

### Checkpoint Tests

```rust
#[tokio::test]
async fn test_concurrent_processing() {
    use std::sync::Arc;

    // Create test images
    tokio::fs::create_dir_all("test_input").await.unwrap();
    tokio::fs::create_dir_all("test_output").await.unwrap();

    // Create 10 small test images
    for i in 0..10 {
        let img = image::RgbaImage::new(50, 50);
        let dynamic = DynamicImage::ImageRgba8(img);
        dynamic
            .save(format!("test_input/img{}.png", i))
            .unwrap();
    }

    let config = ProcessingConfig {
        worker_count: 4,
        batch_size: 5,
        transforms: vec![ImageTransform::Thumbnail { max_size: 32 }],
    };

    let start = std::time::Instant::now();
    let stats = process_directory_concurrent(
        Path::new("test_input"),
        Path::new("test_output"),
        config,
    )
    .await;
    let elapsed = start.elapsed();

    assert_eq!(stats.total_processed.load(Ordering::Relaxed), 10);
    assert_eq!(stats.total_failed.load(Ordering::Relaxed), 0);

    println!("Processed 10 images in {:?}", elapsed);

    // Cleanup
    tokio::fs::remove_dir_all("test_input").await.unwrap();
    tokio::fs::remove_dir_all("test_output").await.unwrap();
}

#[tokio::test]
async fn test_worker_pool_performance() {
    // Create test setup
    tokio::fs::create_dir_all("perf_input").await.unwrap();
    tokio::fs::create_dir_all("perf_output").await.unwrap();

    for i in 0..20 {
        let img = image::RgbaImage::new(100, 100);
        DynamicImage::ImageRgba8(img)
            .save(format!("perf_input/img{}.png", i))
            .unwrap();
    }

    // Test with different worker counts
    for workers in [1, 2, 4, 8] {
        let config = ProcessingConfig {
            worker_count: workers,
            batch_size: 10,
            transforms: vec![
                ImageTransform::Resize {
                    width: 50,
                    height: 50,
                },
                ImageTransform::Grayscale,
            ],
        };

        let start = std::time::Instant::now();
        let stats = process_directory_concurrent(
            Path::new("perf_input"),
            Path::new("perf_output"),
            config,
        )
        .await;
        let elapsed = start.elapsed();

        println!("{} workers: {:?}", workers, elapsed);
    }

    // Cleanup
    tokio::fs::remove_dir_all("perf_input").await.unwrap();
    tokio::fs::remove_dir_all("perf_output").await.unwrap();
}

#[tokio::test]
async fn test_error_handling() {
    tokio::fs::create_dir_all("error_input").await.unwrap();
    tokio::fs::create_dir_all("error_output").await.unwrap();

    // Create a corrupted "image"
    tokio::fs::write("error_input/corrupt.jpg", b"not an image")
        .await
        .unwrap();

    let config = ProcessingConfig {
        worker_count: 2,
        batch_size: 5,
        transforms: vec![ImageTransform::Grayscale],
    };

    let stats = process_directory_concurrent(
        Path::new("error_input"),
        Path::new("error_output"),
        config,
    )
    .await;

    // Should fail gracefully
    assert_eq!(stats.total_failed.load(Ordering::Relaxed), 1);

    // Cleanup
    tokio::fs::remove_dir_all("error_input").await.unwrap();
    tokio::fs::remove_dir_all("error_output").await.unwrap();
}
```

### Starter Code

```rust
use tokio::sync::{mpsc, Semaphore};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use std::time::Duration;

#[derive(Clone)]
pub struct ProcessingConfig {
    pub worker_count: usize,
    pub batch_size: usize,
    pub transforms: Vec<ImageTransform>,
}

#[derive(Debug)]
pub struct ProcessingResult {
    pub original_path: PathBuf,
    pub success: bool,
    pub output_path: Option<PathBuf>,
    pub error: Option<String>,
    pub duration: Duration,
}

pub struct ProcessingStats {
    pub total_processed: AtomicUsize,
    pub total_failed: AtomicUsize,
    pub total_duration: AtomicU64,
}

impl ProcessingStats {
    pub fn new() -> Self {
        Self {
            total_processed: AtomicUsize::new(0),
            total_failed: AtomicUsize::new(0),
            total_duration: AtomicU64::new(0),
        }
    }

    pub fn get_report(&self) -> String {
        let processed = self.total_processed.load(Ordering::Relaxed);
        let failed = self.total_failed.load(Ordering::Relaxed);
        let duration_ms = self.total_duration.load(Ordering::Relaxed);

        let avg_ms = if processed > 0 {
            duration_ms / processed as u64
        } else {
            0
        };

        format!(
            "Processed: {}, Failed: {}, Avg time: {}ms",
            processed, failed, avg_ms
        )
    }
}

pub async fn save_processed_image(
    img: &DynamicImage,
    path: &Path,
    format: ImageFormat,
) -> Result<(), String> {
    // TODO: Convert ImageFormat to image::ImageFormat
    // TODO: Encode image to bytes in memory
    // TODO: Write bytes to file with tokio::fs::write
    // Hint: Use std::io::Cursor for in-memory buffer

    todo!("Implement async image save")
}

pub async fn process_image_worker(
    data: ImageData,
    transforms: Vec<ImageTransform>,
    output_dir: PathBuf,
) -> ProcessingResult {
    // TODO: Record start time
    // TODO: Load DynamicImage from data
    // TODO: Apply transforms
    // TODO: Generate output path (output_dir + filename)
    // TODO: Save processed image
    // TODO: Record duration and return result

    todo!("Implement image worker")
}

pub async fn process_directory_concurrent(
    input_dir: &Path,
    output_dir: &Path,
    config: ProcessingConfig,
) -> ProcessingStats {
    // TODO: Create output directory if it doesn't exist
    // TODO: Scan input directory for images
    // TODO: Create shared stats
    // TODO: Create semaphore to limit concurrent workers
    // TODO: Spawn workers for each image
    // TODO: Collect results and update stats
    // TODO: Return final stats

    todo!("Implement concurrent processing pipeline")
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    let input_dir = args.get(1).map(|s| s.as_str()).unwrap_or("./input");
    let output_dir = args.get(2).map(|s| s.as_str()).unwrap_or("./output");

    println!("Processing images from {} to {}", input_dir, output_dir);

    let config = ProcessingConfig {
        worker_count: 8,
        batch_size: 10,
        transforms: vec![
            ImageTransform::Thumbnail { max_size: 800 },
            ImageTransform::Brighten { value: 10 },
        ],
    };

    let start = std::time::Instant::now();

    let stats = process_directory_concurrent(
        Path::new(input_dir),
        Path::new(output_dir),
        config,
    )
    .await;

    let elapsed = start.elapsed();

    println!("\n=== Processing Complete ===");
    println!("{}", stats.get_report());
    println!("Total time: {:?}", elapsed);
}
```

**Implementation Hints:**
1. Create semaphore: `let sem = Arc::new(Semaphore::new(worker_count));`
2. Acquire permit: `let permit = sem.acquire().await.unwrap();`
3. Spawn worker: `tokio::spawn(async move { ... })`
4. Save image: encode to `Vec<u8>` then `tokio::fs::write`
5. Use `futures::future::join_all` to wait for all workers

---

## Milestone 4: Progress Tracking and Cancellation

### Introduction

**Why Milestone 3 Isn't Enough**: Long-running batch processing needs progress visibility. Users want to know "50/1000 images processed (5%)". Also need ability to cancel (user error, wrong directory).

**The Improvement**: Add progress tracking with channels, implement cancellation with `tokio::sync::watch` or `CancellationToken`.

**Optimization (User Experience)**: Progress feedback makes slow operations feel faster. Users tolerate 5-minute processing if they see steady progress. Without feedback, they assume it's frozen.

### Architecture

**Structs:**
- `ProgressUpdate` - Progress event
  - **Field** `processed: usize` - Images completed so far
  - **Field** `total: usize` - Total images to process
  - **Field** `current_file: Option<String>` - File being processed
  - **Field** `stage: ProcessingStage` - Current stage

- `ProcessingStage` - Pipeline stage
  - **Variant** `Scanning`, `Loading`, `Processing`, `Saving`, `Complete`

- `CancellationToken` - Shared cancellation signal
  - **Method** `cancel()` - Signals cancellation
  - **Method** `is_cancelled() -> bool` - Checks if cancelled

**Key Functions:**
- `async fn process_with_progress(config: ProcessingConfig, progress_tx: mpsc::Sender<ProgressUpdate>) -> ProcessingStats` - Processing with updates
- `async fn monitor_progress(mut progress_rx: mpsc::Receiver<ProgressUpdate>)` - Display progress
- `async fn process_with_cancellation(config: ProcessingConfig, cancel_token: CancellationToken) -> ProcessingStats` - Cancellable processing

**Role Each Plays:**
- **mpsc channel**: Stream of progress updates
- **watch channel**: Broadcast cancellation signal to all workers
- **Progress bar**: Visual feedback (can use indicatif crate)

### Checkpoint Tests

```rust
#[tokio::test]
async fn test_progress_tracking() {
    let (progress_tx, mut progress_rx) = mpsc::channel(100);

    // Create test images
    tokio::fs::create_dir_all("progress_input").await.unwrap();
    tokio::fs::create_dir_all("progress_output").await.unwrap();

    for i in 0..5 {
        let img = image::RgbaImage::new(50, 50);
        DynamicImage::ImageRgba8(img)
            .save(format!("progress_input/img{}.png", i))
            .unwrap();
    }

    let config = ProcessingConfig {
        worker_count: 2,
        batch_size: 5,
        transforms: vec![ImageTransform::Thumbnail { max_size: 32 }],
    };

    // Spawn processor
    tokio::spawn(async move {
        process_with_progress(
            Path::new("progress_input"),
            Path::new("progress_output"),
            config,
            progress_tx,
        )
        .await;
    });

    // Collect progress updates
    let mut updates = Vec::new();
    while let Some(update) = progress_rx.recv().await {
        updates.push(update);
        if updates.last().unwrap().processed == 5 {
            break;
        }
    }

    assert!(updates.len() >= 5);
    assert_eq!(updates.last().unwrap().processed, 5);

    // Cleanup
    tokio::fs::remove_dir_all("progress_input").await.unwrap();
    tokio::fs::remove_dir_all("progress_output").await.unwrap();
}

#[tokio::test]
async fn test_cancellation() {
    use tokio_util::sync::CancellationToken;

    tokio::fs::create_dir_all("cancel_input").await.unwrap();
    tokio::fs::create_dir_all("cancel_output").await.unwrap();

    // Create many images
    for i in 0..100 {
        let img = image::RgbaImage::new(50, 50);
        DynamicImage::ImageRgba8(img)
            .save(format!("cancel_input/img{}.png", i))
            .unwrap();
    }

    let config = ProcessingConfig {
        worker_count: 2,
        batch_size: 10,
        transforms: vec![
            ImageTransform::Resize {
                width: 100,
                height: 100,
            },
            ImageTransform::Blur { sigma: 2.0 },
        ],
    };

    let cancel_token = CancellationToken::new();
    let cancel_clone = cancel_token.clone();

    // Cancel after 500ms
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        cancel_clone.cancel();
        println!("Cancellation requested");
    });

    let start = std::time::Instant::now();
    let stats = process_with_cancellation(
        Path::new("cancel_input"),
        Path::new("cancel_output"),
        config,
        cancel_token,
    )
    .await;
    let elapsed = start.elapsed();

    let processed = stats.total_processed.load(Ordering::Relaxed);

    // Should have processed fewer than 100 images
    assert!(processed < 100);
    println!(
        "Processed {} images before cancellation in {:?}",
        processed, elapsed
    );

    // Cleanup
    tokio::fs::remove_dir_all("cancel_input").await.unwrap();
    tokio::fs::remove_dir_all("cancel_output").await.unwrap();
}

#[test]
fn test_progress_update() {
    let update = ProgressUpdate {
        processed: 50,
        total: 100,
        current_file: Some("image.jpg".to_string()),
        stage: ProcessingStage::Processing,
    };

    let progress = update.progress_percentage();
    assert_eq!(progress, 50.0);
}
```

### Starter Code

```rust
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub enum ProcessingStage {
    Scanning,
    Loading,
    Processing,
    Saving,
    Complete,
}

#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub processed: usize,
    pub total: usize,
    pub current_file: Option<String>,
    pub stage: ProcessingStage,
}

impl ProgressUpdate {
    pub fn progress_percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.processed as f64 / self.total as f64) * 100.0
        }
    }

    pub fn format(&self) -> String {
        // TODO: Format progress update nicely
        // Example: "[Processing] 50/100 (50.0%) - image.jpg"

        todo!("Implement progress formatting")
    }
}

pub async fn monitor_progress(mut progress_rx: mpsc::Receiver<ProgressUpdate>) {
    // TODO: Loop receiving progress updates
    // TODO: Print each update
    // TODO: Exit when channel closes

    todo!("Implement progress monitor")
}

pub async fn process_with_progress(
    input_dir: &Path,
    output_dir: &Path,
    config: ProcessingConfig,
    progress_tx: mpsc::Sender<ProgressUpdate>,
) -> ProcessingStats {
    // TODO: Scan directory
    // TODO: Send scanning update
    // TODO: For each image:
    //   - Send processing update
    //   - Process image
    //   - Send completion update
    // TODO: Send final complete update

    todo!("Implement processing with progress")
}

pub async fn process_with_cancellation(
    input_dir: &Path,
    output_dir: &Path,
    config: ProcessingConfig,
    cancel_token: CancellationToken,
) -> ProcessingStats {
    // TODO: Before each image, check cancel_token.is_cancelled()
    // TODO: If cancelled, stop processing and return partial stats
    // TODO: Otherwise continue processing

    todo!("Implement cancellable processing")
}

#[tokio::main]
async fn main() {
    use tokio_util::sync::CancellationToken;

    let (progress_tx, progress_rx) = mpsc::channel(100);
    let cancel_token = CancellationToken::new();
    let cancel_clone = cancel_token.clone();

    // Spawn progress monitor
    tokio::spawn(monitor_progress(progress_rx));

    // Handle Ctrl+C
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        println!("\nCancelling...");
        cancel_clone.cancel();
    });

    let config = ProcessingConfig {
        worker_count: 8,
        batch_size: 20,
        transforms: vec![
            ImageTransform::Thumbnail { max_size: 1024 },
            ImageTransform::Brighten { value: 5 },
        ],
    };

    let stats = process_with_cancellation(
        Path::new("./input"),
        Path::new("./output"),
        config,
        cancel_token,
    )
    .await;

    println!("\n{}", stats.get_report());
}
```

**Implementation Hints:**
1. Send update: `progress_tx.send(ProgressUpdate { ... }).await.ok();`
2. Check cancellation: `if cancel_token.is_cancelled() { return stats; }`
3. Progress formatting: `format!("[{:?}] {}/{} ({:.1}%) - {}", stage, processed, total, percent, file)`
4. Use `tokio::select!` to race processing with cancellation
5. For visual progress bar: use `indicatif` crate

---

## Milestone 5: Batch Output Formats and Watermarking

### Introduction

**Why Milestone 4 Isn't Enough**: Real applications need multiple output variants (original, thumbnail, webp version) and watermarks for copyright protection.

**The Improvement**: Generate multiple outputs per input image, add text/image watermarking, support various output formats.

**Optimization (Storage)**: WebP format is 25-35% smaller than JPEG at same quality. For 1000 images at 5MB each → 3.5GB vs 5GB (1.5GB saved). Automated format conversion optimizes storage costs.

### Architecture

**Structs:**
- `OutputVariant` - Defines an output version
  - **Field** `name: String` - Variant name (e.g., "thumbnail", "web", "print")
  - **Field** `transforms: Vec<ImageTransform>` - Transformations to apply
  - **Field** `format: ImageFormat` - Output format
  - **Field** `quality: u8` - Compression quality (1-100)

- `WatermarkConfig` - Watermark settings
  - **Field** `text: Option<String>` - Text watermark
  - **Field** `image_path: Option<PathBuf>` - Image watermark (logo)
  - **Field** `position: WatermarkPosition` - Placement
  - **Field** `opacity: f32` - Transparency (0.0-1.0)

- `WatermarkPosition` - Where to place watermark
  - **Variant** `TopLeft`, `TopRight`, `BottomLeft`, `BottomRight`, `Center`

**Key Functions:**
- `fn apply_text_watermark(img: &mut DynamicImage, text: &str, position: WatermarkPosition)` - Adds text
- `fn apply_image_watermark(img: &mut DynamicImage, watermark: &DynamicImage, position: WatermarkPosition, opacity: f32)` - Overlays image
- `async fn process_with_variants(data: ImageData, variants: Vec<OutputVariant>) -> Vec<ProcessingResult>` - Generates multiple outputs

**Role Each Plays:**
- **imageproc crate**: Text rendering on images
- **image::overlay**: Compositing images
- **Format conversion**: Encoding to different formats with quality settings

### Checkpoint Tests

```rust
#[test]
fn test_watermark_positions() {
    let img = DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));

    let (x, y) = calculate_watermark_position(&img, 20, 10, WatermarkPosition::TopLeft);
    assert_eq!(x, 0);
    assert_eq!(y, 0);

    let (x, y) = calculate_watermark_position(&img, 20, 10, WatermarkPosition::BottomRight);
    assert_eq!(x, 80); // 100 - 20
    assert_eq!(y, 90); // 100 - 10

    let (x, y) = calculate_watermark_position(&img, 20, 10, WatermarkPosition::Center);
    assert_eq!(x, 40); // (100 - 20) / 2
    assert_eq!(y, 45); // (100 - 10) / 2
}

#[tokio::test]
async fn test_multiple_output_variants() {
    tokio::fs::create_dir_all("variant_input").await.unwrap();
    tokio::fs::create_dir_all("variant_output").await.unwrap();

    // Create test image
    let img = image::RgbaImage::new(200, 200);
    DynamicImage::ImageRgba8(img)
        .save("variant_input/test.png")
        .unwrap();

    let file = ImageFile {
        path: PathBuf::from("variant_input/test.png"),
        filename: "test.png".to_string(),
        size_bytes: 1000,
        format: ImageFormat::Png,
    };

    let data = load_image(&file).await.unwrap();

    let variants = vec![
        OutputVariant {
            name: "thumbnail".to_string(),
            transforms: vec![ImageTransform::Thumbnail { max_size: 64 }],
            format: ImageFormat::Jpeg,
            quality: 80,
        },
        OutputVariant {
            name: "web".to_string(),
            transforms: vec![ImageTransform::Resize {
                width: 800,
                height: 800,
            }],
            format: ImageFormat::WebP,
            quality: 85,
        },
    ];

    let results = process_with_variants(data, variants, Path::new("variant_output")).await;

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.success));

    // Check outputs exist
    assert!(Path::new("variant_output/test_thumbnail.jpg").exists());
    assert!(Path::new("variant_output/test_web.webp").exists());

    // Cleanup
    tokio::fs::remove_dir_all("variant_input").await.unwrap();
    tokio::fs::remove_dir_all("variant_output").await.unwrap();
}

#[test]
fn test_watermark_application() {
    let mut img = DynamicImage::ImageRgba8(image::RgbaImage::new(200, 200));

    let config = WatermarkConfig {
        text: Some("Copyright 2024".to_string()),
        image_path: None,
        position: WatermarkPosition::BottomRight,
        opacity: 0.5,
    };

    apply_watermark(&mut img, &config);

    // Image should be modified (hard to verify text, but dimensions unchanged)
    assert_eq!(img.width(), 200);
    assert_eq!(img.height(), 200);
}

#[tokio::test]
async fn test_format_conversion() {
    let img = DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));

    // Test JPEG encoding
    let jpeg_bytes = encode_image(&img, ImageFormat::Jpeg, 90).unwrap();
    assert!(jpeg_bytes.len() > 0);

    // Test PNG encoding
    let png_bytes = encode_image(&img, ImageFormat::Png, 100).unwrap();
    assert!(png_bytes.len() > 0);

    // PNG should be larger than JPEG for simple image
    assert!(png_bytes.len() > jpeg_bytes.len());
}
```

### Starter Code

```rust
use image::{Rgba, DynamicImage};
use imageproc::drawing::{draw_text_mut};
use rusttype::{Font, Scale};

#[derive(Debug, Clone)]
pub struct OutputVariant {
    pub name: String,
    pub transforms: Vec<ImageTransform>,
    pub format: ImageFormat,
    pub quality: u8,
}

#[derive(Debug, Clone)]
pub enum WatermarkPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

#[derive(Debug, Clone)]
pub struct WatermarkConfig {
    pub text: Option<String>,
    pub image_path: Option<PathBuf>,
    pub position: WatermarkPosition,
    pub opacity: f32,
}

pub fn calculate_watermark_position(
    img: &DynamicImage,
    watermark_width: u32,
    watermark_height: u32,
    position: WatermarkPosition,
) -> (u32, u32) {
    // TODO: Calculate x, y coordinates based on position
    // TODO: Account for watermark size to keep it within bounds

    todo!("Implement position calculation")
}

pub fn apply_text_watermark(
    img: &mut DynamicImage,
    text: &str,
    position: WatermarkPosition,
) {
    // TODO: Load font (use default or embedded font)
    // TODO: Calculate text position
    // TODO: Draw text on image
    // Hint: Use imageproc::drawing::draw_text_mut

    todo!("Implement text watermark")
}

pub fn apply_image_watermark(
    img: &mut DynamicImage,
    watermark: &DynamicImage,
    position: WatermarkPosition,
    opacity: f32,
) {
    // TODO: Calculate watermark position
    // TODO: Blend watermark onto image with opacity
    // Hint: Use image::imageops::overlay

    todo!("Implement image watermark")
}

pub fn apply_watermark(img: &mut DynamicImage, config: &WatermarkConfig) {
    // TODO: If text watermark, apply it
    // TODO: If image watermark, load and apply it

    todo!("Implement watermark application")
}

pub fn encode_image(
    img: &DynamicImage,
    format: ImageFormat,
    quality: u8,
) -> Result<Vec<u8>, String> {
    // TODO: Convert ImageFormat to image::ImageOutputFormat
    // TODO: Encode image to bytes with quality setting
    // TODO: Return bytes

    todo!("Implement image encoding")
}

pub async fn process_with_variants(
    data: ImageData,
    variants: Vec<OutputVariant>,
    output_dir: &Path,
) -> Vec<ProcessingResult> {
    // TODO: Load image
    // TODO: For each variant:
    //   - Apply transforms
    //   - Encode to specified format
    //   - Save with variant name suffix
    // TODO: Return results for all variants

    todo!("Implement multi-variant processing")
}

#[tokio::main]
async fn main() {
    let variants = vec![
        OutputVariant {
            name: "thumbnail".to_string(),
            transforms: vec![ImageTransform::Thumbnail { max_size: 256 }],
            format: ImageFormat::Jpeg,
            quality: 85,
        },
        OutputVariant {
            name: "web".to_string(),
            transforms: vec![
                ImageTransform::Resize {
                    width: 1920,
                    height: 1080,
                },
                ImageTransform::Brighten { value: 5 },
            ],
            format: ImageFormat::WebP,
            quality: 90,
        },
        OutputVariant {
            name: "print".to_string(),
            transforms: vec![],
            format: ImageFormat::Png,
            quality: 100,
        },
    ];

    let watermark = WatermarkConfig {
        text: Some("© 2024 YourCompany".to_string()),
        image_path: None,
        position: WatermarkPosition::BottomRight,
        opacity: 0.6,
    };

    println!("Processing with {} variants...", variants.len());

    // Process directory with variants
    let images = scan_directory(Path::new("./input")).await.unwrap();

    for image_file in images {
        let data = load_image(&image_file).await.unwrap();
        let results = process_with_variants(data, variants.clone(), Path::new("./output")).await;

        println!(
            "Processed {} -> {} variants",
            image_file.filename,
            results.iter().filter(|r| r.success).count()
        );
    }
}
```

**Implementation Hints:**
1. Position calculation: `match position { TopLeft => (0, 0), BottomRight => (img.width() - w, img.height() - h), ... }`
2. Text rendering: Need `rusttype` crate for font, `imageproc` for drawing
3. Image overlay: `image::imageops::overlay(base, watermark, x as i64, y as i64);`
4. Encoding: `img.write_to(&mut Cursor::new(&mut buf), ImageOutputFormat::Jpeg(quality))`
5. Output filename: `format!("{}_{}.{}", stem, variant.name, ext)`

---

## Milestone 6: Memory-Efficient Streaming and Error Recovery

### Introduction

**Why Milestone 5 Isn't Enough**: Processing thousands of images can exhaust memory. Also, one corrupted image shouldn't crash the entire pipeline. Need streaming processing and resilient error handling.

**The Improvement**: Implement streaming pipeline (bounded channels), process batches to limit memory, skip/log errors instead of failing, add retry logic for transient I/O errors.

**Optimization (Memory)**: Unbounded processing loads all images: 10,000 × 5MB = 50GB. Streaming with 10-image buffer: 10 × 5MB = 50MB (1000x reduction). Critical for large datasets.

### Architecture

**Structs:**
- `PipelineConfig` - Complete pipeline configuration
  - **Field** `input_dirs: Vec<PathBuf>` - Source directories
  - **Field** `output_dir: PathBuf` - Destination
  - **Field** `worker_count: usize` - Parallel workers
  - **Field** `buffer_size: usize` - Max images in memory
  - **Field** `retry_attempts: u32` - Retries for I/O errors
  - **Field** `variants: Vec<OutputVariant>` - Output versions
  - **Field** `watermark: Option<WatermarkConfig>` - Optional watermark

- `ErrorRecoveryStrategy` - How to handle errors
  - **Variant** `Skip` - Log and continue
  - **Variant** `Retry { attempts: u32 }` - Retry then skip
  - **Variant** `Fail` - Stop processing

- `ProcessingLog` - Execution log
  - **Field** `successful: Vec<PathBuf>` - Completed files
  - **Field** `failed: Vec<(PathBuf, String)>` - Failed files with errors
  - **Field** `skipped: Vec<PathBuf>` - Skipped files

**Key Functions:**
- `async fn streaming_pipeline(config: PipelineConfig, progress_tx: mpsc::Sender<ProgressUpdate>) -> ProcessingLog` - Main streaming processor
- `async fn load_with_retry(file: &ImageFile, attempts: u32) -> Result<ImageData, String>` - Resilient loading
- `async fn save_log(log: &ProcessingLog, path: &Path) -> Result<(), String>` - Persist processing log

**Role Each Plays:**
- **Bounded channels**: Limit in-flight images (backpressure)
- **Stream processing**: Load → Process → Save pipeline
- **Error recovery**: Catch errors, log, continue
- **Processing log**: Record successes/failures for resume capability

### Checkpoint Tests

```rust
#[tokio::test]
async fn test_streaming_memory_usage() {
    // Create many images
    tokio::fs::create_dir_all("stream_input").await.unwrap();
    tokio::fs::create_dir_all("stream_output").await.unwrap();

    for i in 0..100 {
        let img = image::RgbaImage::new(100, 100);
        DynamicImage::ImageRgba8(img)
            .save(format!("stream_input/img{}.png", i))
            .unwrap();
    }

    let config = PipelineConfig {
        input_dirs: vec![PathBuf::from("stream_input")],
        output_dir: PathBuf::from("stream_output"),
        worker_count: 4,
        buffer_size: 5, // Small buffer
        retry_attempts: 2,
        variants: vec![OutputVariant {
            name: "output".to_string(),
            transforms: vec![ImageTransform::Thumbnail { max_size: 50 }],
            format: ImageFormat::Jpeg,
            quality: 80,
        }],
        watermark: None,
    };

    let (progress_tx, _progress_rx) = mpsc::channel(10);

    let log = streaming_pipeline(config, progress_tx).await;

    assert_eq!(log.successful.len(), 100);
    assert_eq!(log.failed.len(), 0);

    // Cleanup
    tokio::fs::remove_dir_all("stream_input").await.unwrap();
    tokio::fs::remove_dir_all("stream_output").await.unwrap();
}

#[tokio::test]
async fn test_error_recovery() {
    tokio::fs::create_dir_all("error_input").await.unwrap();
    tokio::fs::create_dir_all("error_output").await.unwrap();

    // Mix of valid and invalid images
    for i in 0..5 {
        let img = image::RgbaImage::new(50, 50);
        DynamicImage::ImageRgba8(img)
            .save(format!("error_input/good{}.png", i))
            .unwrap();
    }

    // Corrupted images
    tokio::fs::write("error_input/bad1.jpg", b"corrupted")
        .await
        .unwrap();
    tokio::fs::write("error_input/bad2.png", b"not an image")
        .await
        .unwrap();

    let config = PipelineConfig {
        input_dirs: vec![PathBuf::from("error_input")],
        output_dir: PathBuf::from("error_output"),
        worker_count: 2,
        buffer_size: 10,
        retry_attempts: 1,
        variants: vec![OutputVariant {
            name: "out".to_string(),
            transforms: vec![],
            format: ImageFormat::Png,
            quality: 100,
        }],
        watermark: None,
    };

    let (progress_tx, _) = mpsc::channel(10);
    let log = streaming_pipeline(config, progress_tx).await;

    assert_eq!(log.successful.len(), 5);
    assert_eq!(log.failed.len(), 2);

    // Cleanup
    tokio::fs::remove_dir_all("error_input").await.unwrap();
    tokio::fs::remove_dir_all("error_output").await.unwrap();
}

#[tokio::test]
async fn test_load_with_retry() {
    // Create image that might have transient I/O errors
    tokio::fs::create_dir_all("retry_test").await.unwrap();

    let img = image::RgbaImage::new(50, 50);
    DynamicImage::ImageRgba8(img)
        .save("retry_test/test.png")
        .unwrap();

    let file = ImageFile {
        path: PathBuf::from("retry_test/test.png"),
        filename: "test.png".to_string(),
        size_bytes: 1000,
        format: ImageFormat::Png,
    };

    // Should succeed
    let result = load_with_retry(&file, 3).await;
    assert!(result.is_ok());

    // Cleanup
    tokio::fs::remove_dir_all("retry_test").await.unwrap();
}

#[tokio::test]
async fn test_processing_log() {
    let log = ProcessingLog {
        successful: vec![PathBuf::from("a.jpg"), PathBuf::from("b.png")],
        failed: vec![(PathBuf::from("c.jpg"), "corrupted".to_string())],
        skipped: vec![PathBuf::from("d.gif")],
    };

    save_log(&log, Path::new("test_log.json")).await.unwrap();

    let loaded = load_log(Path::new("test_log.json")).await.unwrap();

    assert_eq!(loaded.successful.len(), 2);
    assert_eq!(loaded.failed.len(), 1);
    assert_eq!(loaded.skipped.len(), 1);

    // Cleanup
    tokio::fs::remove_file("test_log.json").await.unwrap();
}
```

### Starter Code

```rust
use serde::{Serialize, Deserialize};

#[derive(Clone)]
pub struct PipelineConfig {
    pub input_dirs: Vec<PathBuf>,
    pub output_dir: PathBuf,
    pub worker_count: usize,
    pub buffer_size: usize,
    pub retry_attempts: u32,
    pub variants: Vec<OutputVariant>,
    pub watermark: Option<WatermarkConfig>,
}

#[derive(Debug, Clone)]
pub enum ErrorRecoveryStrategy {
    Skip,
    Retry { attempts: u32 },
    Fail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingLog {
    pub successful: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, String)>,
    pub skipped: Vec<PathBuf>,
}

impl ProcessingLog {
    pub fn new() -> Self {
        Self {
            successful: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "Successful: {}, Failed: {}, Skipped: {}",
            self.successful.len(),
            self.failed.len(),
            self.skipped.len()
        )
    }
}

pub async fn load_with_retry(
    file: &ImageFile,
    attempts: u32,
) -> Result<ImageData, String> {
    // TODO: Try loading image
    // TODO: On failure, retry with exponential backoff
    // TODO: After max attempts, return error

    todo!("Implement retry logic")
}

pub async fn save_log(log: &ProcessingLog, path: &Path) -> Result<(), String> {
    // TODO: Serialize log to JSON
    // TODO: Write to file
    // Hint: Use serde_json

    todo!("Implement log saving")
}

pub async fn load_log(path: &Path) -> Result<ProcessingLog, String> {
    // TODO: Read file
    // TODO: Deserialize JSON

    todo!("Implement log loading")
}

pub async fn streaming_pipeline(
    config: PipelineConfig,
    progress_tx: mpsc::Sender<ProgressUpdate>,
) -> ProcessingLog {
    // TODO: Create bounded channels for streaming
    // TODO: Spawn scanner (produces ImageFiles)
    // TODO: Spawn loaders (load images with retry)
    // TODO: Spawn processors (apply transforms)
    // TODO: Spawn savers (write results)
    // TODO: Collect results into log
    // TODO: Handle errors gracefully

    todo!("Implement streaming pipeline")
}

#[tokio::main]
async fn main() {
    let config = PipelineConfig {
        input_dirs: vec![
            PathBuf::from("./photos"),
            PathBuf::from("./images"),
        ],
        output_dir: PathBuf::from("./processed"),
        worker_count: 8,
        buffer_size: 20,
        retry_attempts: 3,
        variants: vec![
            OutputVariant {
                name: "web".to_string(),
                transforms: vec![ImageTransform::Thumbnail { max_size: 1920 }],
                format: ImageFormat::WebP,
                quality: 85,
            },
            OutputVariant {
                name: "thumb".to_string(),
                transforms: vec![ImageTransform::Thumbnail { max_size: 256 }],
                format: ImageFormat::Jpeg,
                quality: 80,
            },
        ],
        watermark: Some(WatermarkConfig {
            text: Some("© 2024".to_string()),
            image_path: None,
            position: WatermarkPosition::BottomRight,
            opacity: 0.5,
        }),
    };

    let (progress_tx, mut progress_rx) = mpsc::channel(100);

    // Monitor progress
    tokio::spawn(async move {
        while let Some(update) = progress_rx.recv().await {
            println!("{}", update.format());
        }
    });

    println!("Starting streaming pipeline...");
    let log = streaming_pipeline(config, progress_tx).await;

    println!("\n=== Processing Complete ===");
    println!("{}", log.summary());

    // Save log
    save_log(&log, Path::new("processing_log.json"))
        .await
        .unwrap();
    println!("Log saved to processing_log.json");
}
```

**Implementation Hints:**
1. Bounded channels: `mpsc::channel(buffer_size)` naturally limits memory
2. Pipeline stages: Scanner → Loader → Processor → Saver
3. Error handling: `match result { Ok(_) => log.successful.push(...), Err(e) => log.failed.push(...) }`
4. Retry: `for attempt in 1..=attempts { ... sleep(Duration::from_millis(100 * 2u64.pow(attempt))).await; }`
5. Use `futures::stream::StreamExt` for stream combinators

---

## Complete Working Example

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

// =============================================================================
// Milestone 1: Basic Atomic Counter
// =============================================================================

pub struct AtomicCounter {
    count: AtomicUsize,
}

impl AtomicCounter {
    pub fn new() -> Self {
        Self {
            count: AtomicUsize::new(0),
        }
    }

    pub fn increment(&self) {
        self.count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn add(&self, value: usize) {
        self.count.fetch_add(value, Ordering::SeqCst);
    }

    pub fn get(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }

    pub fn reset(&self) -> usize {
        self.count.swap(0, Ordering::SeqCst)
    }
}

// =============================================================================
// Milestone 2: Multiple Metric Types with Relaxed Ordering
// =============================================================================

pub struct MetricsCollector {
    requests: AtomicUsize,
    errors: AtomicUsize,
    bytes_sent: AtomicUsize,
    active_connections: AtomicUsize,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            requests: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
            bytes_sent: AtomicUsize::new(0),
            active_connections: AtomicUsize::new(0),
        }
    }

    pub fn record_request(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_bytes(&self, bytes: usize) {
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn connection_opened(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn connection_closed(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            requests: self.requests.load(Ordering::Acquire),
            errors: self.errors.load(Ordering::Acquire),
            bytes_sent: self.bytes_sent.load(Ordering::Acquire),
            active_connections: self.active_connections.load(Ordering::Acquire),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub requests: usize,
    pub errors: usize,
    pub bytes_sent: usize,
    pub active_connections: usize,
}

impl MetricsSnapshot {
    pub fn error_rate(&self) -> f64 {
        if self.requests == 0 {
            0.0
        } else {
            self.errors as f64 / self.requests as f64
        }
    }
}

// =============================================================================
// Milestone 3: Histogram with Lock-Free Buckets
// =============================================================================

pub struct AtomicHistogram<const N: usize> {
    buckets: [AtomicUsize; N],
    bucket_boundaries: [u64; N],
}

impl<const N: usize> AtomicHistogram<N> {
    pub fn new(boundaries: [u64; N]) -> Self {
        Self {
            buckets: std::array::from_fn(|_| AtomicUsize::new(0)),
            bucket_boundaries: boundaries,
        }
    }

    pub fn record(&self, value_us: u64) {
        let bucket_idx = self.find_bucket(value_us);
        self.buckets[bucket_idx].fetch_add(1, Ordering::Relaxed);
    }

    fn find_bucket(&self, value: u64) -> usize {
        match self.bucket_boundaries.binary_search(&value) {
            Ok(idx) => idx,
            Err(idx) => idx.min(N - 1),
        }
    }

    pub fn snapshot(&self) -> HistogramSnapshot {
        HistogramSnapshot {
            buckets: self
                .buckets
                .iter()
                .map(|bucket| bucket.load(Ordering::Acquire))
                .collect(),
            boundaries: self.bucket_boundaries.to_vec(),
        }
    }
}

pub struct HistogramSnapshot {
    pub buckets: Vec<usize>,
    pub boundaries: Vec<u64>,
}

impl HistogramSnapshot {
    pub fn total(&self) -> usize {
        self.buckets.iter().sum()
    }

    pub fn percentile(&self, p: f64) -> u64 {
        let total = self.total();
        if total == 0 {
            return 0;
        }
        let mut target = (total as f64 * p).ceil() as usize;
        if target == 0 {
            target = 1;
        }
        let mut accumulated = 0;
        for (idx, count) in self.buckets.iter().enumerate() {
            accumulated += count;
            if accumulated >= target {
                if idx == 0 {
                    return self.boundaries[0];
                } else {
                    return self.boundaries[idx - 1];
                }
            }
        }
        *self.boundaries.last().unwrap_or(&0)
    }

    pub fn mean(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            return 0.0;
        }

        let mut sum = 0.0;
        for (idx, count) in self.buckets.iter().enumerate() {
            if *count == 0 {
                continue;
            }
            let lower = if idx == 0 { 0 } else { self.boundaries[idx - 1] };
            let upper = self.boundaries[idx];
            let midpoint = (lower + upper) as f64 / 2.0;
            sum += midpoint * (*count as f64);
        }

        sum / total as f64
    }
}

// =============================================================================
// Milestone 4: Compare-and-Swap for Atomic Max/Min
// =============================================================================

pub struct AtomicMinMax {
    min: AtomicU64,
    max: AtomicU64,
}

impl AtomicMinMax {
    pub fn new() -> Self {
        Self {
            min: AtomicU64::new(u64::MAX),
            max: AtomicU64::new(0),
        }
    }

    pub fn update(&self, value: u64) {
        let mut current_min = self.min.load(Ordering::Relaxed);
        loop {
            if value >= current_min {
                break;
            }
            match self
                .min
                .compare_exchange_weak(current_min, value, Ordering::Relaxed, Ordering::Relaxed)
            {
                Ok(_) => break,
                Err(actual) => current_min = actual,
            }
        }

        let mut current_max = self.max.load(Ordering::Relaxed);
        loop {
            if value <= current_max {
                break;
            }
            match self
                .max
                .compare_exchange_weak(current_max, value, Ordering::Relaxed, Ordering::Relaxed)
            {
                Ok(_) => break,
                Err(actual) => current_max = actual,
            }
        }
    }

    pub fn get_min(&self) -> u64 {
        self.min.load(Ordering::Acquire)
    }

    pub fn get_max(&self) -> u64 {
        self.max.load(Ordering::Acquire)
    }

    pub fn reset(&self) {
        self.min.store(u64::MAX, Ordering::Release);
        self.max.store(0, Ordering::Release);
    }
}

// =============================================================================
// Milestone 5: Full Metrics System with Periodic Export
// =============================================================================

pub struct MetricsRegistry {
    collectors: HashMap<String, Arc<MetricsCollector>>,
    histograms: HashMap<String, Arc<AtomicHistogram<8>>>,
    export_interval: Duration,
    running: Arc<AtomicBool>,
}

impl MetricsRegistry {
    pub fn new(interval: Duration) -> Self {
        Self {
            collectors: HashMap::new(),
            histograms: HashMap::new(),
            export_interval: interval,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn register_collector(&mut self, name: String) -> Arc<MetricsCollector> {
        let collector = Arc::new(MetricsCollector::new());
        self.collectors.insert(name, Arc::clone(&collector));
        collector
    }

    pub fn register_histogram(
        &mut self,
        name: String,
        boundaries: [u64; 8],
    ) -> Arc<AtomicHistogram<8>> {
        let histogram = Arc::new(AtomicHistogram::new(boundaries));
        self.histograms.insert(name, Arc::clone(&histogram));
        histogram
    }

    pub fn start_export_thread<F>(&self, callback: F)
    where
        F: Fn(FullSnapshot) + Send + 'static,
    {
        self.running.store(true, Ordering::SeqCst);
        let collectors = self.collectors.clone();
        let histograms = self.histograms.clone();
        let interval = self.export_interval;
        let running = Arc::clone(&self.running);

        thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                thread::sleep(interval);

                let snapshot = FullSnapshot {
                    timestamp: SystemTime::now(),
                    metrics: collectors
                        .iter()
                        .map(|(name, collector)| (name.clone(), collector.snapshot()))
                        .collect(),
                    histograms: histograms
                        .iter()
                        .map(|(name, histogram)| (name.clone(), histogram.snapshot()))
                        .collect(),
                };

                callback(snapshot);
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn snapshot_all(&self) -> FullSnapshot {
        FullSnapshot {
            timestamp: SystemTime::now(),
            metrics: self
                .collectors
                .iter()
                .map(|(name, collector)| (name.clone(), collector.snapshot()))
                .collect(),
            histograms: self
                .histograms
                .iter()
                .map(|(name, histogram)| (name.clone(), histogram.snapshot()))
                .collect(),
        }
    }
}

pub struct FullSnapshot {
    pub timestamp: SystemTime,
    pub metrics: HashMap<String, MetricsSnapshot>,
    pub histograms: HashMap<String, HistogramSnapshot>,
}

impl FullSnapshot {
    pub fn to_prometheus_format(&self) -> String {
        let mut output = String::new();

        for (name, snapshot) in &self.metrics {
            output.push_str(&format!("# TYPE {}_requests counter\n", name));
            output.push_str(&format!("{}_requests {}\n", name, snapshot.requests));
            output.push_str(&format!("# TYPE {}_errors counter\n", name));
            output.push_str(&format!("{}_errors {}\n", name, snapshot.errors));
            output.push_str(&format!("# TYPE {}_bytes_sent counter\n", name));
            output.push_str(&format!("{}_bytes_sent {}\n", name, snapshot.bytes_sent));
            output.push_str(&format!("# TYPE {}_active_connections gauge\n", name));
            output.push_str(&format!(
                "{}_active_connections {}\n",
                name, snapshot.active_connections
            ));
        }

        for (name, histogram) in &self.histograms {
            output.push_str(&format!("# TYPE {}_latency histogram\n", name));
            for (idx, count) in histogram.buckets.iter().enumerate() {
                output.push_str(&format!(
                    "{}_latency_bucket{{le=\"{}\"}} {}\n",
                    name, histogram.boundaries[idx], count
                ));
            }
            output.push_str(&format!(
                "{}_latency_count {}\n",
                name,
                histogram.total()
            ));
        }

        output
    }
}

// =============================================================================
// Milestone 6: Memory Ordering Optimization and Benchmarking
// =============================================================================

#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::time::Instant;

    fn benchmark_operation<F>(name: &str, iterations: usize, mut op: F)
    where
        F: FnMut(),
    {
        let start = Instant::now();
        for _ in 0..iterations {
            op();
        }
        let elapsed = start.elapsed();
        let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
        let ns_per_op = elapsed.as_nanos() as f64 / iterations as f64;
        println!(
            "{}: {:.0} ops/sec ({:.2} ns/op)",
            name, ops_per_sec, ns_per_op
        );
    }

    #[test]
    fn compare_orderings() {
        const ITERATIONS: usize = 100_000;

        let seq_counter = AtomicUsize::new(0);
        benchmark_operation("SeqCst", ITERATIONS, || {
            seq_counter.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(seq_counter.load(Ordering::SeqCst), ITERATIONS);

        let acqrel_counter = AtomicUsize::new(0);
        benchmark_operation("AcqRel", ITERATIONS, || {
            acqrel_counter.fetch_add(1, Ordering::AcqRel);
        });
        assert_eq!(acqrel_counter.load(Ordering::SeqCst), ITERATIONS);

        let relaxed_counter = AtomicUsize::new(0);
        benchmark_operation("Relaxed", ITERATIONS, || {
            relaxed_counter.fetch_add(1, Ordering::Relaxed);
        });
        assert_eq!(relaxed_counter.load(Ordering::SeqCst), ITERATIONS);
    }
}

pub mod ordering_docs {
    pub const GUIDELINES: &str = "Use Relaxed for independent counters, Acquire loads for \
snapshots/export, and SeqCst for control flags like shutdown signals.";
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rand::random;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    // ----- Milestone 1 -------------------------------------------------------

    #[test]
    fn test_counter_increment() {
        let counter = AtomicCounter::new();
        assert_eq!(counter.get(), 0);

        counter.increment();
        assert_eq!(counter.get(), 1);

        counter.add(5);
        assert_eq!(counter.get(), 6);
    }

    #[test]
    fn test_counter_reset() {
        let counter = AtomicCounter::new();
        counter.add(42);

        let old_value = counter.reset();
        assert_eq!(old_value, 42);
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_concurrent_increments() {
        let counter = Arc::new(AtomicCounter::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let counter_clone = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                for _ in 0..1000 {
                    counter_clone.increment();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.get(), 10_000);
    }

    // ----- Milestone 2 -------------------------------------------------------

    #[test]
    fn test_multiple_metrics() {
        let metrics = MetricsCollector::new();

        metrics.record_request();
        metrics.record_request();
        metrics.record_error();
        metrics.record_bytes(1024);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.requests, 2);
        assert_eq!(snapshot.errors, 1);
        assert_eq!(snapshot.bytes_sent, 1024);
        assert_eq!(snapshot.error_rate(), 0.5);
    }

    #[test]
    fn test_gauge_operations() {
        let metrics = MetricsCollector::new();

        metrics.connection_opened();
        metrics.connection_opened();
        assert_eq!(metrics.snapshot().active_connections, 2);

        metrics.connection_closed();
        assert_eq!(metrics.snapshot().active_connections, 1);
    }

    #[test]
    fn test_concurrent_mixed_operations() {
        let metrics = Arc::new(MetricsCollector::new());
        let mut handles = vec![];

        for _ in 0..5 {
            let m = Arc::clone(&metrics);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    m.record_request();
                    if random::<bool>() {
                        m.record_error();
                    }
                    m.record_bytes(256);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.requests, 500);
        assert_eq!(snapshot.bytes_sent, 500 * 256);
    }

    // ----- Milestone 3 -------------------------------------------------------

    #[test]
    fn test_histogram_basic() {
        let hist = AtomicHistogram::new([10_000, 50_000, 100_000, 500_000, u64::MAX]);

        hist.record(5_000);
        hist.record(25_000);
        hist.record(75_000);

        let snapshot = hist.snapshot();
        assert_eq!(snapshot.buckets[0], 1);
        assert_eq!(snapshot.buckets[1], 1);
        assert_eq!(snapshot.buckets[2], 1);
        assert_eq!(snapshot.total(), 3);
    }

    #[test]
    fn test_percentile_calculation() {
        let hist = AtomicHistogram::new([10_000, 50_000, 100_000, 500_000, u64::MAX]);

        for _ in 0..50 {
            hist.record(5_000);
        }
        for _ in 0..30 {
            hist.record(25_000);
        }
        for _ in 0..20 {
            hist.record(75_000);
        }

        let snapshot = hist.snapshot();
        assert!(snapshot.percentile(0.5) <= 10_000);
        let p90 = snapshot.percentile(0.9);
        assert!(p90 > 10_000 && p90 <= 50_000);
    }

    #[test]
    fn test_concurrent_histogram() {
        let hist = Arc::new(AtomicHistogram::new([
            10_000,
            50_000,
            100_000,
            500_000,
            u64::MAX,
        ]));
        let mut handles = vec![];

        for thread_id in 0..10 {
            let h = Arc::clone(&hist);
            let handle = thread::spawn(move || {
                for i in 0..100 {
                    let value = (thread_id * 1000 + i * 100) as u64;
                    h.record(value);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(hist.snapshot().total(), 1000);
    }

    // ----- Milestone 4 -------------------------------------------------------

    #[test]
    fn test_minmax_basic() {
        let minmax = AtomicMinMax::new();

        minmax.update(100);
        assert_eq!(minmax.get_min(), 100);
        assert_eq!(minmax.get_max(), 100);

        minmax.update(50);
        assert_eq!(minmax.get_min(), 50);
        assert_eq!(minmax.get_max(), 100);

        minmax.update(150);
        assert_eq!(minmax.get_min(), 50);
        assert_eq!(minmax.get_max(), 150);
    }

    #[test]
    fn test_concurrent_minmax() {
        let minmax = Arc::new(AtomicMinMax::new());
        let mut handles = vec![];

        for thread_id in 0..10 {
            let mm = Arc::clone(&minmax);
            let handle = thread::spawn(move || {
                for i in 0..100 {
                    let value = (thread_id * 100 + i) as u64;
                    mm.update(value);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(minmax.get_min(), 0);
        assert_eq!(minmax.get_max(), 999);
    }

    #[test]
    fn test_reset() {
        let minmax = AtomicMinMax::new();
        minmax.update(50);
        minmax.update(150);

        minmax.reset();
        assert_eq!(minmax.get_min(), u64::MAX);
        assert_eq!(minmax.get_max(), 0);
    }

    // ----- Milestone 5 -------------------------------------------------------

    #[test]
    fn test_registry_registration() {
        let mut registry = MetricsRegistry::new(Duration::from_secs(10));

        let collector1 = registry.register_collector("http".to_string());
        let collector2 = registry.register_collector("db".to_string());

        collector1.record_request();
        collector2.record_request();
        collector2.record_request();

        let snapshot = registry.snapshot_all();
        assert_eq!(snapshot.metrics["http"].requests, 1);
        assert_eq!(snapshot.metrics["db"].requests, 2);
    }

    #[test]
    fn test_periodic_export() {
        let mut registry = MetricsRegistry::new(Duration::from_millis(100));
        let collector = registry.register_collector("test".to_string());

        let export_count = Arc::new(Mutex::new(0));
        let count_clone = Arc::clone(&export_count);

        registry.start_export_thread(move |_snapshot| {
            *count_clone.lock().unwrap() += 1;
        });

        for _ in 0..10 {
            collector.record_request();
            thread::sleep(Duration::from_millis(50));
        }

        registry.stop();
        assert!(*export_count.lock().unwrap() >= 1);
    }

    #[test]
    fn test_prometheus_format() {
        let mut registry = MetricsRegistry::new(Duration::from_secs(60));
        let collector = registry.register_collector("http".to_string());

        collector.record_request();
        collector.record_request();
        collector.record_error();
        collector.record_bytes(1024);

        let snapshot = registry.snapshot_all();
        let prom = snapshot.to_prometheus_format();

        assert!(prom.contains("http_requests 2"));
        assert!(prom.contains("http_errors 1"));
        assert!(prom.contains("http_bytes_sent 1024"));
    }

    // ----- Milestone 6 -------------------------------------------------------

    #[test]
    fn benchmark_counter_increment_relaxed() {
        let counter = AtomicCounter::new();
        let start = Instant::now();

        for _ in 0..1_000_000 {
            counter.increment();
        }

        let elapsed = start.elapsed();
        println!("1M increments (Relaxed impl): {:?}", elapsed);
        assert_eq!(counter.get(), 1_000_000);
    }

    #[test]
    fn benchmark_concurrent_throughput() {
        let metrics = Arc::new(MetricsCollector::new());
        let start = Instant::now();

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let m = Arc::clone(&metrics);
                thread::spawn(move || {
                    for _ in 0..250_000 {
                        m.record_request();
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let elapsed = start.elapsed();
        let ops_per_sec = 1_000_000.0 / elapsed.as_secs_f64();

        println!("Throughput: {:.0} ops/sec", ops_per_sec);
        assert_eq!(metrics.snapshot().requests, 1_000_000);
    }

    #[test]
    fn verify_snapshot_consistency() {
        let metrics = Arc::new(MetricsCollector::new());

        let m1 = Arc::clone(&metrics);
        let writer = thread::spawn(move || {
            for i in 0..1000 {
                m1.record_request();
                m1.record_bytes(i);
            }
        });

        let m2 = Arc::clone(&metrics);
        let reader = thread::spawn(move || {
            for _ in 0..100 {
                let snap = m2.snapshot();
                assert!(snap.bytes_sent <= snap.requests * 1000);
            }
        });

        writer.join().unwrap();
        reader.join().unwrap();
    }
}

```