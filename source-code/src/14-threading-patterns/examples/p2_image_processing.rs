//! Pattern 2: Thread Pools and Work Stealing
//! Parallel Image Processing
//!
//! Run with: cargo run --example p2_image_processing

use rayon::prelude::*;

struct Image {
    pixels: Vec<u8>,
    width: usize,
    height: usize,
}

impl Image {
    fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: vec![128; width * height], // Gray image
            width,
            height,
        }
    }

    fn apply_filter_parallel(&mut self, filter: impl Fn(u8) -> u8 + Sync) {
        self.pixels.par_iter_mut().for_each(|pixel| {
            *pixel = filter(*pixel);
        });
    }

    fn brighten(&mut self, amount: u8) {
        self.apply_filter_parallel(|p| p.saturating_add(amount));
    }

    fn invert(&mut self) {
        self.apply_filter_parallel(|p| 255 - p);
    }
}

fn main() {
    println!("=== Parallel Image Processing ===\n");

    let mut image = Image::new(1920, 1080); // Full HD image
    println!(
        "Image: {}x{} ({} pixels)",
        image.width,
        image.height,
        image.pixels.len()
    );
    println!("Original pixel[0]: {}", image.pixels[0]);

    image.brighten(50);
    println!("After brighten(50): {}", image.pixels[0]);

    image.invert();
    println!("After invert: {}", image.pixels[0]);

    println!("\n=== Key Points ===");
    println!("1. par_iter_mut() for parallel in-place modification");
    println!("2. Filter function must be Sync for parallel access");
    println!("3. Perfect for pixel-level operations (same op on each pixel)");
}
