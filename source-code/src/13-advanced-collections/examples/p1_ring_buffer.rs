//! Pattern 1: VecDeque and Ring Buffers
//! Ring Buffer for Real-Time Data
//!
//! Run with: cargo run --example p1_ring_buffer

use std::collections::VecDeque;

struct RingBuffer<T> {
    buffer: VecDeque<T>,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn push(&mut self, item: T) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front(); // Remove oldest
        }
        self.buffer.push_back(item);
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.buffer.get(index)
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        self.buffer.iter()
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }

    fn is_full(&self) -> bool {
        self.buffer.len() >= self.capacity
    }

    fn clear(&mut self) {
        self.buffer.clear();
    }

    fn as_slice(&self) -> (&[T], &[T]) {
        self.buffer.as_slices()
    }
}

// Specialized: Sliding window statistics
struct SlidingWindowStats {
    buffer: RingBuffer<f64>,
}

impl SlidingWindowStats {
    fn new(window_size: usize) -> Self {
        Self {
            buffer: RingBuffer::new(window_size),
        }
    }

    fn add(&mut self, value: f64) {
        self.buffer.push(value);
    }

    fn mean(&self) -> Option<f64> {
        if self.buffer.len() == 0 {
            return None;
        }

        let sum: f64 = self.buffer.iter().sum();
        Some(sum / self.buffer.len() as f64)
    }

    fn min(&self) -> Option<f64> {
        self.buffer.iter().copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
    }

    fn max(&self) -> Option<f64> {
        self.buffer.iter().copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    fn variance(&self) -> Option<f64> {
        if self.buffer.len() < 2 {
            return None;
        }

        let mean = self.mean()?;
        let sum_squared_diff: f64 = self.buffer
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum();

        Some(sum_squared_diff / self.buffer.len() as f64)
    }

    fn std_dev(&self) -> Option<f64> {
        self.variance().map(|v| v.sqrt())
    }
}

// Real-world example: Audio sample buffer
struct AudioBuffer {
    samples: RingBuffer<f32>,
    sample_rate: u32,
}

impl AudioBuffer {
    fn new(duration_seconds: f32, sample_rate: u32) -> Self {
        let capacity = (duration_seconds * sample_rate as f32) as usize;
        Self {
            samples: RingBuffer::new(capacity),
            sample_rate,
        }
    }

    fn add_sample(&mut self, sample: f32) {
        self.samples.push(sample);
    }

    fn add_samples(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.add_sample(sample);
        }
    }

    fn rms(&self) -> f32 {
        if self.samples.len() == 0 {
            return 0.0;
        }

        let sum_squares: f32 = self.samples.iter().map(|&s| s * s).sum();
        (sum_squares / self.samples.len() as f32).sqrt()
    }

    fn peak(&self) -> f32 {
        self.samples
            .iter()
            .map(|&s| s.abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    fn zero_crossing_rate(&self) -> f32 {
        if self.samples.len() < 2 {
            return 0.0;
        }

        let mut crossings = 0;
        let samples: Vec<_> = self.samples.iter().copied().collect();

        for i in 0..samples.len() - 1 {
            if (samples[i] >= 0.0 && samples[i + 1] < 0.0)
                || (samples[i] < 0.0 && samples[i + 1] >= 0.0)
            {
                crossings += 1;
            }
        }

        crossings as f32 / (samples.len() - 1) as f32
    }
}

// Example usage
fn main() {
    println!("=== Sliding Window Stats ===\n");

    let mut stats = SlidingWindowStats::new(5);

    for value in [10.0, 20.0, 15.0, 25.0, 30.0, 18.0, 22.0] {
        stats.add(value);
        println!("Added {}: mean={:.2}, std_dev={:.2}",
                 value,
                 stats.mean().unwrap_or(0.0),
                 stats.std_dev().unwrap_or(0.0));
    }

    println!("\n=== Audio Buffer ===\n");

    let mut audio = AudioBuffer::new(0.1, 44100); // 100ms buffer at 44.1kHz

    // Simulate sine wave
    for i in 0..4410 {
        let t = i as f32 / 44100.0;
        // 440 Hz sine wave
        let sample = (2.0 * std::f32::consts::PI * 440.0 * t).sin();
        audio.add_sample(sample * 0.5); // 50% amplitude
    }

    println!("RMS: {:.4}", audio.rms());
    println!("Peak: {:.4}", audio.peak());
    println!("Zero crossing rate: {:.4}", audio.zero_crossing_rate());

    println!("\n=== Key Points ===");
    println!("1. Ring buffer overwrites oldest data when full");
    println!("2. Fixed memory usage regardless of data stream");
    println!("3. Ideal for sensor data, audio, and logging");
}
