//! Pattern 1: Par Bridge Examples
//!
//! Run with: cargo run --bin p1_par_bridge

use rayon::prelude::*;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::fs;
use std::path::PathBuf;

fn par_bridge_basic() {
    let iter = (0..1000).filter(|x| x % 2 == 0);

    // Bridge to parallel
    let sum: i32 = iter.par_bridge().map(|x| x * x).sum();
    println!("Sum: {}", sum);
}

fn par_bridge_from_channel() {
    let (tx, rx) = mpsc::channel();

    // Producer thread
    thread::spawn(move || {
        for i in 0..1000 {
            tx.send(i).unwrap();
            thread::sleep(Duration::from_micros(10));
        }
    });

    // Parallel processing of channel items
    let sum: i32 = rx
        .into_iter()
        .par_bridge()
        .map(|x| {
            // Expensive computation
            thread::sleep(Duration::from_micros(100));
            x * x
        })
        .sum();

    println!("Channel sum: {}", sum);
}

fn find_large_files_parallel(root: &str, min_size: u64) -> Vec<(PathBuf, u64)> {
    fn visit_dirs(path: PathBuf) -> Box<dyn Iterator<Item = PathBuf> + Send> {
        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(_) => return Box::new(std::iter::empty()),
        };

        let iter = entries.filter_map(|e| e.ok()).flat_map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(path)
            } else {
                Box::new(std::iter::once(path))
            }
        });

        Box::new(iter)
    }

    visit_dirs(PathBuf::from(root))
        .par_bridge()
        .filter_map(|path| {
            let metadata = fs::metadata(&path).ok()?;
            let size = metadata.len();
            if size >= min_size {
                Some((path, size))
            } else {
                None
            }
        })
        .collect()
}

struct DatabaseIterator {
    current: usize,
    total: usize,
}

impl Iterator for DatabaseIterator {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.total {
            let value = self.current as i32;
            self.current += 1;
            // Simulate database fetch delay
            thread::sleep(Duration::from_micros(10));
            Some(value)
        } else {
            None
        }
    }
}

fn process_database_results() {
    let db_iter = DatabaseIterator {
        current: 0,
        total: 100, // Reduced for faster demo
    };

    // Process results in parallel as they arrive
    let sum: i32 = db_iter
        .par_bridge()
        .map(|x| x * 2)
        .sum();

    println!("Database result sum: {}", sum);
}

fn process_network_stream() {
    let (tx, rx) = mpsc::channel();

    // Simulate network packets arriving
    thread::spawn(move || {
        for i in 0..100 {
            let packet = format!("packet_{}", i);
            tx.send(packet).unwrap();
            thread::sleep(Duration::from_millis(5));
        }
    });

    // Process packets in parallel
    let processed: Vec<String> = rx
        .into_iter()
        .par_bridge()
        .map(|packet| {
            // Expensive processing (e.g., parsing, validation)
            thread::sleep(Duration::from_millis(10));
            format!("processed_{}", packet)
        })
        .collect();

    println!("Processed {} packets", processed.len());
}

fn main() {
    println!("=== Par Bridge Basic ===\n");
    par_bridge_basic();

    println!("\n=== Par Bridge from Channel ===\n");
    par_bridge_from_channel();

    println!("\n=== File System Traversal ===\n");
    let large_files = find_large_files_parallel(".", 1000);
    println!("Found {} files >= 1000 bytes", large_files.len());

    println!("\n=== Database Results ===\n");
    process_database_results();

    println!("\n=== Network Stream ===\n");
    process_network_stream();
}
