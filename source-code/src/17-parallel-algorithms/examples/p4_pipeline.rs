//! Pattern 4: Pipeline Parallelism
//!
//! Run with: cargo run --bin p4_pipeline

use rayon::prelude::*;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn simple_pipeline() {
    let (stage1_tx, stage1_rx) = mpsc::sync_channel(100);
    let (stage2_tx, stage2_rx) = mpsc::sync_channel(100);
    let (stage3_tx, stage3_rx) = mpsc::sync_channel(100);

    // Stage 1: Data generation
    let producer = thread::spawn(move || {
        for i in 0..1000 {
            stage1_tx.send(i).unwrap();
        }
    });

    // Stage 2: Transform (parallel)
    let stage2 = thread::spawn(move || {
        stage1_rx
            .into_iter()
            .par_bridge()
            .for_each_with(stage2_tx, |tx, item| {
                let transformed = item * 2;
                let _ = tx.send(transformed);
            });
    });

    // Stage 3: Filter (parallel)
    let stage3 = thread::spawn(move || {
        stage2_rx
            .into_iter()
            .par_bridge()
            .filter(|&x| x % 4 == 0)
            .for_each_with(stage3_tx, |tx, item| {
                let _ = tx.send(item);
            });
    });

    // Consumer
    let consumer = thread::spawn(move || {
        let sum: i32 = stage3_rx.into_iter().sum();
        sum
    });

    producer.join().unwrap();
    stage2.join().unwrap();
    stage3.join().unwrap();
    let result = consumer.join().unwrap();

    println!("Pipeline result: {}", result);
}

// Image processing pipeline
struct ImagePipeline;

impl ImagePipeline {
    fn process_batch(images: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        images
            .into_par_iter()
            .map(|img| Self::stage1_decode(img))
            .map(|img| Self::stage2_enhance(img))
            .map(|img| Self::stage3_compress(img))
            .collect()
    }

    fn stage1_decode(data: Vec<u8>) -> Vec<u8> {
        // Simulate decoding
        thread::sleep(Duration::from_micros(100));
        data
    }

    fn stage2_enhance(mut data: Vec<u8>) -> Vec<u8> {
        // Simulate enhancement
        for pixel in &mut data {
            *pixel = pixel.saturating_add(10);
        }
        data
    }

    fn stage3_compress(data: Vec<u8>) -> Vec<u8> {
        // Simulate compression
        thread::sleep(Duration::from_micros(50));
        data
    }
}

// Log processing pipeline
#[derive(Debug, Clone)]
struct RawLog(String);

#[derive(Debug, Clone)]
struct ParsedLog {
    timestamp: u64,
    level: String,
    message: String,
}

#[derive(Debug, Clone)]
struct EnrichedLog {
    log: ParsedLog,
    metadata: String,
}

struct LogPipeline;

impl LogPipeline {
    fn process(logs: Vec<RawLog>) -> Vec<EnrichedLog> {
        logs.into_par_iter()
            .filter_map(|raw| Self::parse(raw))
            .map(|parsed| Self::enrich(parsed))
            .filter(|enriched| enriched.log.level == "ERROR")
            .collect()
    }

    fn parse(raw: RawLog) -> Option<ParsedLog> {
        let parts: Vec<&str> = raw.0.split('|').collect();
        if parts.len() >= 3 {
            Some(ParsedLog {
                timestamp: parts[0].parse().ok()?,
                level: parts[1].to_string(),
                message: parts[2].to_string(),
            })
        } else {
            None
        }
    }

    fn enrich(log: ParsedLog) -> EnrichedLog {
        EnrichedLog {
            log: log.clone(),
            metadata: format!("enriched_{}", log.timestamp),
        }
    }
}

fn multi_stage_parallel() {
    let data: Vec<i32> = (0..10000).collect();

    // Stage 1: Light processing (high parallelism)
    let stage1: Vec<i32> = data
        .par_iter()
        .map(|&x| x + 1)
        .collect();

    // Stage 2: Heavy processing (moderate parallelism)
    let stage2: Vec<i32> = stage1
        .par_chunks(100) // Larger chunks for heavy work
        .flat_map(|chunk| {
            chunk.par_iter().map(|&x| {
                // Simulate heavy computation
                let mut result = x;
                for _ in 0..100 {
                    result = (result * 2) % 1000;
                }
                result
            })
        })
        .collect();

    // Stage 3: Aggregation
    let sum: i32 = stage2.par_iter().sum();

    println!("Multi-stage result: {}", sum);
}

// ETL pipeline
struct EtlPipeline;

impl EtlPipeline {
    fn run(input_files: Vec<String>) -> Vec<(String, usize)> {
        input_files
            .into_par_iter()
            // Extract: Read files in parallel
            .filter_map(|file| Self::extract(&file))
            // Transform: Process data in parallel
            .map(|data| Self::transform(data))
            // Load: Aggregate results
            .collect()
    }

    fn extract(file: &str) -> Option<Vec<String>> {
        // Simulate file reading
        Some(vec![format!("data_from_{}", file)])
    }

    fn transform(data: Vec<String>) -> (String, usize) {
        // Simulate transformation
        let processed = data
            .par_iter()
            .map(|s| s.to_uppercase())
            .collect::<Vec<_>>();

        ("transformed".to_string(), processed.len())
    }
}

fn main() {
    println!("=== Simple Pipeline ===\n");
    simple_pipeline();

    println!("\n=== Image Processing Pipeline ===\n");
    let images: Vec<Vec<u8>> = (0..100).map(|_| vec![128; 1000]).collect();
    let start = std::time::Instant::now();
    let processed = ImagePipeline::process_batch(images);
    println!("Processed {} images in {:?}", processed.len(), start.elapsed());

    println!("\n=== Log Processing Pipeline ===\n");
    let logs: Vec<RawLog> = (0..1000)
        .map(|i| RawLog(format!("{}|{}|message_{}", i, if i % 10 == 0 { "ERROR" } else { "INFO" }, i)))
        .collect();
    let errors = LogPipeline::process(logs);
    println!("Found {} errors", errors.len());

    println!("\n=== Multi-stage Parallel ===\n");
    multi_stage_parallel();

    println!("\n=== ETL Pipeline ===\n");
    let files: Vec<String> = (0..100).map(|i| format!("file_{}.csv", i)).collect();
    let results = EtlPipeline::run(files);
    println!("Processed {} files", results.len());
}
