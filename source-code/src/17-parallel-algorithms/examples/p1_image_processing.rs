//! Pattern 1: Image Processing with Rayon
//!
//! Run with: cargo run --bin p1_image_processing

use rayon::prelude::*;

struct Image {
    pixels: Vec<u8>,
    width: usize,
    height: usize,
}

impl Image {
    fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: vec![128; width * height * 3], // RGB
            width,
            height,
        }
    }

    fn apply_filter_parallel(&mut self, filter: fn(u8) -> u8) {
        self.pixels.par_iter_mut().for_each(|pixel| {
            *pixel = filter(*pixel);
        });
    }

    fn grayscale_parallel(&mut self) {
        self.pixels
            .par_chunks_mut(3)
            .for_each(|rgb| {
                let gray = ((rgb[0] as u32 + rgb[1] as u32 + rgb[2] as u32) / 3) as u8;
                rgb[0] = gray;
                rgb[1] = gray;
                rgb[2] = gray;
            });
    }

    fn brightness_parallel(&mut self, delta: i16) {
        self.pixels.par_iter_mut().for_each(|pixel| {
            *pixel = (*pixel as i16 + delta).clamp(0, 255) as u8;
        });
    }
}

fn main() {
    println!("=== Image Processing ===\n");

    let mut img = Image::new(1920, 1080);
    println!("Created {}x{} image with {} pixels", img.width, img.height, img.pixels.len() / 3);

    let start = std::time::Instant::now();
    img.grayscale_parallel();
    println!("Grayscale applied in {:?}", start.elapsed());

    let start = std::time::Instant::now();
    img.brightness_parallel(10);
    println!("Brightness adjusted in {:?}", start.elapsed());

    println!("First few pixels: {:?}", &img.pixels[0..9]);
}
