//! Pattern 5: SIMD Operations
//! Example: Image Processing Pipeline
//!
//! Run with: cargo run --example p5_image_processing

fn main() {
    println!("=== Image Processing with SIMD-Friendly Patterns ===\n");

    // RGB to Grayscale conversion
    println!("=== RGB to Grayscale ===\n");

    fn grayscale(rgb_data: &[u8], output: &mut [u8]) {
        assert_eq!(rgb_data.len() % 3, 0);
        assert_eq!(output.len(), rgb_data.len() / 3);

        for (i, pixel) in rgb_data.chunks_exact(3).enumerate() {
            let r = pixel[0] as u32;
            let g = pixel[1] as u32;
            let b = pixel[2] as u32;

            // Weighted average for luminance (ITU-R BT.601)
            // 0.299*R + 0.587*G + 0.114*B
            // Using integer math: (77*R + 150*G + 29*B) >> 8
            let gray = ((r * 77 + g * 150 + b * 29) >> 8) as u8;
            output[i] = gray;
        }
    }

    // Create a sample 4x4 RGB image
    let rgb_image: Vec<u8> = vec![
        255, 0, 0,    // Red
        0, 255, 0,    // Green
        0, 0, 255,    // Blue
        255, 255, 0,  // Yellow
        255, 0, 255,  // Magenta
        0, 255, 255,  // Cyan
        255, 255, 255,// White
        0, 0, 0,      // Black
        128, 128, 128,// Gray
        255, 128, 0,  // Orange
        128, 0, 255,  // Purple
        0, 128, 64,   // Dark teal
        192, 192, 192,// Light gray
        64, 64, 64,   // Dark gray
        255, 200, 200,// Light pink
        200, 255, 200,// Light green
    ];

    let mut grayscale_image = vec![0u8; rgb_image.len() / 3];
    grayscale(&rgb_image, &mut grayscale_image);

    println!("RGB image ({} pixels):", rgb_image.len() / 3);
    for (i, pixel) in rgb_image.chunks_exact(3).enumerate() {
        println!("  Pixel {}: RGB({:3}, {:3}, {:3}) -> Gray({})",
            i, pixel[0], pixel[1], pixel[2], grayscale_image[i]);
    }

    // Brightness adjustment
    println!("\n=== Brightness Adjustment ===\n");

    fn adjust_brightness(data: &mut [u8], delta: i16) {
        for value in data {
            let new_val = (*value as i16 + delta).clamp(0, 255) as u8;
            *value = new_val;
        }
    }

    let mut image = vec![100u8, 150, 200, 50, 250, 30];
    println!("Before: {:?}", image);

    adjust_brightness(&mut image, 30);
    println!("After +30 brightness: {:?}", image);

    adjust_brightness(&mut image, -50);
    println!("After -50 brightness: {:?}", image);

    // Contrast adjustment
    println!("\n=== Contrast Adjustment ===\n");

    fn adjust_contrast(data: &mut [u8], factor: f32) {
        for value in data {
            let centered = *value as f32 - 128.0;
            let adjusted = (centered * factor + 128.0).clamp(0.0, 255.0) as u8;
            *value = adjusted;
        }
    }

    let mut image = vec![50u8, 100, 128, 156, 200];
    println!("Before: {:?}", image);

    let mut high_contrast = image.clone();
    adjust_contrast(&mut high_contrast, 1.5);
    println!("High contrast (1.5x): {:?}", high_contrast);

    let mut low_contrast = image.clone();
    adjust_contrast(&mut low_contrast, 0.5);
    println!("Low contrast (0.5x): {:?}", low_contrast);

    // Image inversion
    println!("\n=== Image Inversion ===\n");

    fn invert(data: &mut [u8]) {
        for value in data {
            *value = 255 - *value;
        }
    }

    let mut image = vec![0u8, 64, 128, 192, 255];
    println!("Before: {:?}", image);

    invert(&mut image);
    println!("After inversion: {:?}", image);

    // Box blur (3x3)
    println!("\n=== Simple Box Blur (1D) ===\n");

    fn blur_1d(data: &[u8]) -> Vec<u8> {
        if data.len() < 3 {
            return data.to_vec();
        }

        let mut result = Vec::with_capacity(data.len());

        // Handle first element
        result.push(((data[0] as u16 + data[1] as u16) / 2) as u8);

        // Middle elements
        for window in data.windows(3) {
            let avg = (window[0] as u16 + window[1] as u16 + window[2] as u16) / 3;
            result.push(avg as u8);
        }

        // Handle last element
        result.push(((data[data.len()-2] as u16 + data[data.len()-1] as u16) / 2) as u8);

        result
    }

    let image = vec![10u8, 20, 100, 200, 50, 30];
    println!("Before blur: {:?}", image);

    let blurred = blur_1d(&image);
    println!("After blur:  {:?}", blurred);

    // Threshold/Binarization
    println!("\n=== Threshold (Binarization) ===\n");

    fn threshold(data: &mut [u8], thresh: u8) {
        for value in data {
            *value = if *value >= thresh { 255 } else { 0 };
        }
    }

    let mut image = vec![50u8, 100, 127, 128, 150, 200, 250];
    println!("Before (threshold=128): {:?}", image);

    threshold(&mut image, 128);
    println!("After:                  {:?}", image);

    // Performance test
    println!("\n=== Performance Test ===\n");

    let width = 1920;
    let height = 1080;
    let rgb_size = width * height * 3;

    let rgb_image: Vec<u8> = (0..rgb_size as u32).map(|i| (i % 256) as u8).collect();
    let mut gray_image = vec![0u8; width * height];

    let start = std::time::Instant::now();
    grayscale(&rgb_image, &mut gray_image);
    let time = start.elapsed();

    println!("Converting {}x{} RGB to grayscale:", width, height);
    println!("  Input size:  {} bytes", rgb_image.len());
    println!("  Output size: {} bytes", gray_image.len());
    println!("  Time:        {:?}", time);
    println!("  Throughput:  {:.1} MB/s", rgb_image.len() as f64 / time.as_secs_f64() / 1_000_000.0);

    println!("\n=== Key Points ===");
    println!("1. Process pixels in chunks of 3/4 (RGB/RGBA)");
    println!("2. Use integer math (>> 8) instead of division");
    println!("3. Clamp values to [0, 255] range");
    println!("4. chunks_exact ensures uniform processing");
    println!("5. These patterns auto-vectorize well");
}
