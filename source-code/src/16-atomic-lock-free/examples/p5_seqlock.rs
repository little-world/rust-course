//! Pattern 5: Seqlock Pattern
//!
//! Run with: cargo run --example p5_seqlock

use std::sync::atomic::{AtomicUsize, Ordering};
use std::cell::UnsafeCell;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct SeqLock<T> {
    seq: AtomicUsize,
    data: UnsafeCell<T>,
}

impl<T: Copy> SeqLock<T> {
    pub fn new(data: T) -> Self {
        Self {
            seq: AtomicUsize::new(0),
            data: UnsafeCell::new(data),
        }
    }

    pub fn read(&self) -> T {
        loop {
            // Read sequence number (even = not writing)
            let seq1 = self.seq.load(Ordering::Acquire);

            if seq1 % 2 == 1 {
                // Writer is active, spin
                std::hint::spin_loop();
                continue;
            }

            // Read data
            let data = unsafe { *self.data.get() };

            // Verify sequence hasn't changed
            std::sync::atomic::fence(Ordering::Acquire);
            let seq2 = self.seq.load(Ordering::Acquire);

            if seq1 == seq2 {
                return data;
            }

            // Sequence changed during read, retry
        }
    }

    pub fn write(&self, data: T) {
        // Increment sequence (makes it odd = writing)
        let seq = self.seq.fetch_add(1, Ordering::Acquire);
        debug_assert!(seq % 2 == 0, "Concurrent writes detected");

        // Write data
        unsafe {
            *self.data.get() = data;
        }

        // Increment again (makes it even = readable)
        self.seq.fetch_add(1, Ordering::Release);
    }

    pub fn try_read(&self) -> Option<T> {
        let seq1 = self.seq.load(Ordering::Acquire);

        if seq1 % 2 == 1 {
            return None; // Writer active
        }

        let data = unsafe { *self.data.get() };

        std::sync::atomic::fence(Ordering::Acquire);
        let seq2 = self.seq.load(Ordering::Acquire);

        if seq1 == seq2 {
            Some(data)
        } else {
            None // Data changed
        }
    }
}

unsafe impl<T: Copy + Send> Send for SeqLock<T> {}
unsafe impl<T: Copy + Send> Sync for SeqLock<T> {}

// Coordinates with seqlock
#[derive(Copy, Clone, Debug)]
struct Coordinates {
    x: f64,
    y: f64,
    z: f64,
}

fn seqlock_coordinates_example() {
    let position = Arc::new(SeqLock::new(Coordinates {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }));

    // Writer thread (updates position)
    let writer_pos = Arc::clone(&position);
    let writer = thread::spawn(move || {
        for i in 0..100 {
            let coords = Coordinates {
                x: i as f64,
                y: (i * 2) as f64,
                z: (i * 3) as f64,
            };
            writer_pos.write(coords);
            thread::sleep(Duration::from_millis(10));
        }
    });

    // Reader threads (read position frequently)
    let mut readers = vec![];
    for id in 0..5 {
        let reader_pos = Arc::clone(&position);
        readers.push(thread::spawn(move || {
            for _ in 0..1000 {
                let coords = reader_pos.read();
                if id == 0 && coords.x as usize % 10 == 0 {
                    println!("Reader {}: {:?}", id, coords);
                }
            }
        }));
    }

    writer.join().unwrap();
    for reader in readers {
        reader.join().unwrap();
    }
}

// Statistics snapshot
#[derive(Copy, Clone, Debug)]
struct Stats {
    count: u64,
    sum: u64,
    min: u64,
    max: u64,
}

impl Stats {
    fn new() -> Self {
        Self {
            count: 0,
            sum: 0,
            min: u64::MAX,
            max: 0,
        }
    }

    fn add(&mut self, value: u64) {
        self.count += 1;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    fn average(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum as f64 / self.count as f64
        }
    }
}

fn seqlock_stats_example() {
    let stats = Arc::new(SeqLock::new(Stats::new()));

    // Writer thread
    let writer_stats = Arc::clone(&stats);
    let writer = thread::spawn(move || {
        for i in 0..1000 {
            let mut current = writer_stats.read();
            current.add(i);
            writer_stats.write(current);
        }
    });

    // Reader threads (monitor stats)
    let mut readers = vec![];
    for id in 0..3 {
        let reader_stats = Arc::clone(&stats);
        readers.push(thread::spawn(move || {
            for _ in 0..100 {
                thread::sleep(Duration::from_millis(10));
                let snapshot = reader_stats.read();
                if id == 0 {
                    println!(
                        "Stats - Count: {}, Avg: {:.2}, Min: {}, Max: {}",
                        snapshot.count,
                        snapshot.average(),
                        snapshot.min,
                        snapshot.max
                    );
                }
            }
        }));
    }

    writer.join().unwrap();
    for reader in readers {
        reader.join().unwrap();
    }
}

// Versioned seqlock (track writes)
pub struct VersionedSeqLock<T> {
    seqlock: SeqLock<T>,
}

impl<T: Copy> VersionedSeqLock<T> {
    pub fn new(data: T) -> Self {
        Self {
            seqlock: SeqLock::new(data),
        }
    }

    pub fn read_with_version(&self) -> (T, usize) {
        let seq1 = self.seqlock.seq.load(Ordering::Acquire);
        let data = self.seqlock.read();
        let version = seq1 / 2;
        (data, version)
    }

    pub fn write(&self, data: T) {
        self.seqlock.write(data);
    }

    pub fn version(&self) -> usize {
        self.seqlock.seq.load(Ordering::Acquire) / 2
    }
}

fn main() {
    println!("=== Seqlock Coordinates ===\n");
    seqlock_coordinates_example();

    println!("\n=== Seqlock Statistics ===\n");
    seqlock_stats_example();

    println!("\n=== Versioned Seqlock ===\n");

    let data = VersionedSeqLock::new(0u64);

    for i in 0..5 {
        data.write(i * 10);
        let (value, version) = data.read_with_version();
        println!("Value: {}, Version: {}", value, version);
    }
}
