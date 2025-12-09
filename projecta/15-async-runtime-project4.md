# Chapter 15: Async Runtime Patterns - Programming Projects

## Project 4: Concurrent Image Processing Pipeline

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
// Cargo.toml:
// [dependencies]
// tokio = { version = "1.35", features = ["full"] }
// tokio-util = "0.7"
// image = "0.24"
// futures = "0.3"
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"

use tokio::fs;
use tokio::sync::{mpsc, Semaphore};
use tokio_util::sync::CancellationToken;
use image::{DynamicImage, GenericImageView, ImageFormat as ImgFormat};
use futures::stream::{self, StreamExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

// Image formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Gif,
    WebP,
    Bmp,
}

impl ImageFormat {
    fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|s| s.to_str())
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
                "png" => Some(ImageFormat::Png),
                "gif" => Some(ImageFormat::Gif),
                "webp" => Some(ImageFormat::WebP),
                "bmp" => Some(ImageFormat::Bmp),
                _ => None,
            })
    }

    fn to_image_format(&self) -> ImgFormat {
        match self {
            ImageFormat::Jpeg => ImgFormat::Jpeg,
            ImageFormat::Png => ImgFormat::Png,
            ImageFormat::Gif => ImgFormat::Gif,
            ImageFormat::WebP => ImgFormat::WebP,
            ImageFormat::Bmp => ImgFormat::Bmp,
        }
    }

    fn extension(&self) -> &str {
        match self {
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Png => "png",
            ImageFormat::Gif => "gif",
            ImageFormat::WebP => "webp",
            ImageFormat::Bmp => "bmp",
        }
    }
}

// Image file metadata
#[derive(Debug, Clone)]
pub struct ImageFile {
    pub path: PathBuf,
    pub filename: String,
    pub size_bytes: u64,
    pub format: ImageFormat,
}

// Transformations
#[derive(Debug, Clone)]
pub enum ImageTransform {
    Resize { width: u32, height: u32 },
    Thumbnail { max_size: u32 },
    Grayscale,
    Blur { sigma: f32 },
    Brighten { value: i32 },
}

impl ImageTransform {
    fn apply(&self, img: DynamicImage) -> DynamicImage {
        match self {
            ImageTransform::Resize { width, height } => {
                img.resize_exact(*width, *height, image::imageops::FilterType::Lanczos3)
            }
            ImageTransform::Thumbnail { max_size } => img.thumbnail(*max_size, *max_size),
            ImageTransform::Grayscale => img.grayscale(),
            ImageTransform::Blur { sigma } => img.blur(*sigma),
            ImageTransform::Brighten { value } => img.brighten(*value),
        }
    }
}

// Output variant
#[derive(Debug, Clone)]
pub struct OutputVariant {
    pub name: String,
    pub transforms: Vec<ImageTransform>,
    pub format: ImageFormat,
}

// Processing statistics
pub struct ProcessingStats {
    pub processed: AtomicUsize,
    pub failed: AtomicUsize,
    pub total_bytes: AtomicU64,
}

impl ProcessingStats {
    pub fn new() -> Self {
        Self {
            processed: AtomicUsize::new(0),
            failed: AtomicUsize::new(0),
            total_bytes: AtomicU64::new(0),
        }
    }

    pub fn report(&self) -> String {
        let processed = self.processed.load(Ordering::Relaxed);
        let failed = self.failed.load(Ordering::Relaxed);
        let bytes = self.total_bytes.load(Ordering::Relaxed);

        format!(
            "Processed: {}, Failed: {}, Total size: {:.2} MB",
            processed,
            failed,
            bytes as f64 / 1_000_000.0
        )
    }
}

// Progress tracking
#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub processed: usize,
    pub total: usize,
    pub current: String,
}

impl ProgressUpdate {
    pub fn format(&self) -> String {
        let percent = if self.total > 0 {
            (self.processed as f64 / self.total as f64) * 100.0
        } else {
            0.0
        };

        format!(
            "[{}/{}] {:.1}% - {}",
            self.processed, self.total, percent, self.current
        )
    }
}

// Main functions
pub async fn scan_directory(path: &Path) -> Result<Vec<ImageFile>, String> {
    let mut files = Vec::new();
    let mut entries = fs::read_dir(path)
        .await
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read entry: {}", e))?
    {
        let path = entry.path();

        if path.is_file() {
            if let Some(format) = ImageFormat::from_path(&path) {
                let metadata = entry
                    .metadata()
                    .await
                    .map_err(|e| format!("Failed to get metadata: {}", e))?;

                files.push(ImageFile {
                    path: path.clone(),
                    filename: path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                    size_bytes: metadata.len(),
                    format,
                });
            }
        }
    }

    Ok(files)
}

pub async fn load_image(file: &ImageFile) -> Result<DynamicImage, String> {
    let bytes = fs::read(&file.path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    image::load_from_memory(&bytes).map_err(|e| format!("Failed to decode image: {}", e))
}

pub async fn process_image(
    img: DynamicImage,
    transforms: &[ImageTransform],
) -> DynamicImage {
    let mut result = img;

    for transform in transforms {
        result = tokio::task::spawn_blocking({
            let t = transform.clone();
            let img = result.clone();
            move || t.apply(img)
        })
        .await
        .unwrap();
    }

    result
}

pub async fn save_image(
    img: &DynamicImage,
    path: &Path,
    format: ImageFormat,
) -> Result<(), String> {
    let img_clone = img.clone();
    let path_clone = path.to_path_buf();
    let fmt = format.to_image_format();

    tokio::task::spawn_blocking(move || {
        img_clone
            .save_with_format(&path_clone, fmt)
            .map_err(|e| format!("Failed to save: {}", e))
    })
    .await
    .unwrap()
}

pub async fn process_directory(
    input_dir: &Path,
    output_dir: &Path,
    variants: Vec<OutputVariant>,
    max_concurrent: usize,
    progress_tx: mpsc::Sender<ProgressUpdate>,
) -> ProcessingStats {
    fs::create_dir_all(output_dir).await.ok();

    let files = scan_directory(input_dir).await.unwrap_or_default();
    let total = files.len();
    let stats = Arc::new(ProcessingStats::new());
    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    let tasks: Vec<_> = files
        .into_iter()
        .enumerate()
        .map(|(idx, file)| {
            let stats = Arc::clone(&stats);
            let sem = Arc::clone(&semaphore);
            let variants = variants.clone();
            let output_dir = output_dir.to_path_buf();
            let progress_tx = progress_tx.clone();

            tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                // Send progress
                let _ = progress_tx
                    .send(ProgressUpdate {
                        processed: idx,
                        total,
                        current: file.filename.clone(),
                    })
                    .await;

                // Load image
                let img = match load_image(&file).await {
                    Ok(img) => img,
                    Err(_) => {
                        stats.failed.fetch_add(1, Ordering::Relaxed);
                        return;
                    }
                };

                // Process each variant
                for variant in variants {
                    let processed = process_image(img.clone(), &variant.transforms).await;

                    let output_path = output_dir.join(format!(
                        "{}_{}.{}",
                        file.path.file_stem().unwrap().to_string_lossy(),
                        variant.name,
                        variant.format.extension()
                    ));

                    if save_image(&processed, &output_path, variant.format)
                        .await
                        .is_ok()
                    {
                        stats.processed.fetch_add(1, Ordering::Relaxed);
                        stats
                            .total_bytes
                            .fetch_add(file.size_bytes, Ordering::Relaxed);
                    } else {
                        stats.failed.fetch_add(1, Ordering::Relaxed);
                    }
                }
            })
        })
        .collect();

    for task in tasks {
        task.await.ok();
    }

    Arc::try_unwrap(stats).unwrap_or_else(|arc| (*arc).clone())
}

// Clone implementation for ProcessingStats
impl Clone for ProcessingStats {
    fn clone(&self) -> Self {
        Self {
            processed: AtomicUsize::new(self.processed.load(Ordering::Relaxed)),
            failed: AtomicUsize::new(self.failed.load(Ordering::Relaxed)),
            total_bytes: AtomicU64::new(self.total_bytes.load(Ordering::Relaxed)),
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Concurrent Image Processing Pipeline ===\n");

    let input_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./input".to_string());
    let output_dir = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "./output".to_string());

    let variants = vec![
        OutputVariant {
            name: "thumbnail".to_string(),
            transforms: vec![ImageTransform::Thumbnail { max_size: 256 }],
            format: ImageFormat::Jpeg,
        },
        OutputVariant {
            name: "web".to_string(),
            transforms: vec![
                ImageTransform::Thumbnail { max_size: 1920 },
                ImageTransform::Brighten { value: 5 },
            ],
            format: ImageFormat::WebP,
        },
    ];

    let (progress_tx, mut progress_rx) = mpsc::channel(100);

    // Spawn progress monitor
    tokio::spawn(async move {
        while let Some(update) = progress_rx.recv().await {
            println!("{}", update.format());
        }
    });

    let start = Instant::now();

    let stats = process_directory(
        Path::new(&input_dir),
        Path::new(&output_dir),
        variants,
        8, // concurrent workers
        progress_tx,
    )
    .await;

    let elapsed = start.elapsed();

    println!("\n=== Complete ===");
    println!("{}", stats.report());
    println!("Time: {:.2}s", elapsed.as_secs_f64());
}
```

This complete implementation provides a production-ready concurrent image processor with:
1. **Async directory scanning** - Finds images efficiently
2. **Concurrent processing** - Parallel transformation on multiple cores
3. **Multiple output variants** - Generate thumbnails, web versions, etc.
4. **Progress tracking** - Real-time feedback
5. **Error handling** - Gracefully skip corrupted images
6. **Memory efficiency** - Bounded concurrency prevents OOM

Perfect for batch photo processing, thumbnail generation, and image optimization workflows!
