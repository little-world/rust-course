# Chapter 22: Serialization Patterns - Programming Projects

## Project 2: Serialization Performance Benchmarking Tool

### Problem Statement

Build a command-line tool that benchmarks various serialization formats (JSON, Bincode, MessagePack) for performance. The tool will serialize and deserialize a sample dataset, measuring both the resulting data size and the time taken for each operation. The final output should be a clean, readable markdown table comparing the results.

### Use Cases

- **API Design**: Choosing between JSON (human-readable) and a binary format (performant) for a new API.
- **Data Storage**: Deciding on a format for caching, on-disk storage, or database fields where size and speed are critical.
- **Network Protocols**: Selecting an efficient format for client-server or service-to-service communication.
- **Game Development**: Storing game state or sending network packets where every byte and microsecond counts.

### Why It Matters

**Performance is a Feature**: The choice of serialization format has a massive impact on performance. A binary format like Bincode can be 10x faster and produce payloads that are 50% smaller than JSON. For high-throughput systems, this difference can translate to significant cost savings on bandwidth and CPU usage.

**Informed Trade-offs**: There is no single "best" format. JSON offers unparalleled readability and interoperability. Bincode offers maximum performance for Rust-to-Rust communication. MessagePack strikes a balance between performance and cross-language compatibility. This project teaches you how to *quantify* these trade-offs, enabling you to make informed decisions based on data, not just intuition.

**Benchmarking Skills**: Learning to use a professional benchmarking library like `criterion` is a vital skill for any performance-oriented programmer. It teaches you how to measure code accurately, accounting for statistical noise and providing reliable results.

Example comparison for a sample dataset:
| Format      | Size (bytes) | Serialization Time | Deserialization Time |
|-------------|--------------|--------------------|----------------------|
| JSON        | 250,000      | 500 µs             | 800 µs               |
| Bincode     | 120,000      | 50 µs              | 70 µs                |
| MessagePack | 140,000      | 80 µs              | 120 µs               |

---

## Milestone 1: Defining the Benchmark Data

### Introduction

Before we can benchmark anything, we need data to benchmark *with*. A good dataset should be representative of real-world complexity, containing a mix of data types. This milestone is about creating a rich data structure that will serve as the subject of our performance tests.

**Why Start Here**: The nature of the data significantly affects serialization performance. A text-heavy dataset will have different characteristics from a number-heavy one. A well-designed test case ensures our benchmark results are meaningful.

### Architecture

**Structs:**
- `BenchmarkData`: A complex struct designed to be our test subject.
  - **Fields**: A mix of `String`, `u64`, `f64`, `Vec<T>`, and `HashMap<K, V>`.

**Key Functions:**
- `fn generate_data(size: usize) -> BenchmarkData`: A function to create a `BenchmarkData` instance of a given complexity.

**Role Each Plays:**
- **`BenchmarkData`**: The "workload" for our serialization engines. Its structure will exercise different aspects of the serialization process.
- **`generate_data`**: Allows us to create test data of varying sizes, so we can see how each format scales.

### Checkpoint Tests

```rust
#[test]
fn test_data_generation() {
    let data = generate_data(100);

    assert_eq!(data.id, 1);
    assert!(!data.name.is_empty());
    assert_eq!(data.values.len(), 100);
    assert!(!data.metadata.is_empty());
}
```

### Starter Code

```rust
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// Add to Cargo.toml:
// serde = { version = "1.0", features = ["derive"] }

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BenchmarkData {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub timestamp: u64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
    pub active: bool,
}

pub fn generate_data(size: usize) -> BenchmarkData {
    // TODO: Create and return an instance of BenchmarkData.
    // - The `values` vector should have `size` elements.
    // - The `metadata` map should contain a few entries.
    // - Populate other fields with sample data.
    todo!("Implement generate_data")
}
```

**Implementation Hints:**
1.  Use a loop to populate the `values` vector. `(i as f64).sin()` can be a good source of varied float values.
2.  Use `metadata.insert(...)` to add a few key-value pairs.

---

## Milestone 2: Baseline with JSON

### Introduction

**Why Milestone 1 Isn't Enough**: We have data, but we haven't serialized it yet. We'll start with JSON, the most common text-based format, to establish a baseline for size and correctness.

**The Improvement**: This step provides our first data point. We'll implement a function to serialize our `BenchmarkData` to a JSON string and another to deserialize it, confirming that the process is "lossless" (what we put in is what we get out).

### Architecture

**Dependencies:**
- `serde_json = "1.0"`

**Key Functions:**
- `fn benchmark_json(data: &BenchmarkData) -> (usize, BenchmarkData)`: Serializes the data to JSON, measures the size of the output string, deserializes it back, and returns the size and the resulting data.

**Role Each Plays:**
- **`serde_json::to_string`**: The function that performs the serialization to a JSON string.
- **`serde_json::from_str`**: The function that performs deserialization from a JSON string.
- **`.len()`**: Used on the resulting string to measure the size in bytes.

### Checkpoint Tests

```rust
#[test]
fn test_json_roundtrip() {
    let original_data = generate_data(10);
    let (size, roundtrip_data) = benchmark_json(&original_data);

    assert!(size > 0, "JSON string should have a size");
    assert_eq!(original_data, roundtrip_data, "Data should be identical after JSON roundtrip");
}
```

### Starter Code

```rust
use serde_json;
// Assume BenchmarkData and generate_data from Milestone 1 are available

pub fn benchmark_json(data: &BenchmarkData) -> (usize, BenchmarkData) {
    // TODO: Serialize the data to a JSON string.
    let json_string = todo!();

    // TODO: Get the size of the string in bytes.
    let size = todo!();

    // TODO: Deserialize the string back into a BenchmarkData instance.
    let deserialized_data = todo!();
    
    (size, deserialized_data)
}
```

**Implementation Hints:**
1.  `serde_json::to_string(data).unwrap()`
2.  `json_string.len()`
3.  `serde_json::from_str(&json_string).unwrap()`

---

## Milestone 3: Comparing Size with Bincode

### Introduction

**Why Milestone 2 Isn't Enough**: JSON is readable but verbose. A primary reason to choose another format is to reduce data size. This milestone introduces Bincode, a compact binary format, to demonstrate this advantage.

**The Improvement**: We will add a new function to perform a roundtrip with Bincode and compare its output size to JSON's. This provides the first clear evidence of the size-performance trade-off.

### Architecture

**Dependencies:**
- `bincode = "1.3"`

**Key Functions:**
- `fn benchmark_bincode(data: &BenchmarkData) -> (usize, BenchmarkData)`: Performs a serialization/deserialization roundtrip with Bincode and returns the output size.

**Role Each Plays:**
- **`bincode::serialize`**: Serializes data into a `Vec<u8>`.
- **`bincode::deserialize`**: Deserializes data from a `&[u8]`.

### Checkpoint Tests

```rust
#[test]
fn test_bincode_roundtrip() {
    let original_data = generate_data(10);
    let (size, roundtrip_data) = benchmark_bincode(&original_data);

    assert!(size > 0, "Bincode output should have a size");
    assert_eq!(original_data, roundtrip_data, "Data should be identical after Bincode roundtrip");
}

#[test]
fn test_bincode_is_smaller_than_json() {
    let data = generate_data(100);
    let (json_size, _) = benchmark_json(&data);
    let (bincode_size, _) = benchmark_bincode(&data);

    println!("JSON size: {}, Bincode size: {}", json_size, bincode_size);
    assert!(bincode_size < json_size, "Bincode should be smaller than JSON");
}
```

### Starter Code

```rust
use bincode;
// Assume BenchmarkData and generate_data from Milestone 1 are available

pub fn benchmark_bincode(data: &BenchmarkData) -> (usize, BenchmarkData) {
    // TODO: Serialize the data to a byte vector using bincode.
    let encoded_data: Vec<u8> = todo!();

    // TODO: Get the size of the byte vector.
    let size = todo!();

    // TODO: Deserialize the byte vector back into a BenchmarkData instance.
    let deserialized_data = todo!();
    
    (size, deserialized_data)
}
```

**Implementation Hints:**
1.  `bincode::serialize(data).unwrap()`
2.  `encoded_data.len()`
3.  `bincode::deserialize(&encoded_data).unwrap()`

---

## Milestone 4: Adding MessagePack

### Introduction

**Why Milestone 3 Isn't Enough**: Bincode is fast and small, but it's a Rust-only format. We need a binary format that is also cross-language compatible. This milestone introduces MessagePack, which fills that role.

**The Improvement**: Adding MessagePack provides a third data point, illustrating a middle ground: better performance than JSON, but with broader language support than Bincode.

### Architecture

**Dependencies:**
- `rmp-serde = "1.1"` (rmp = Rust MessagePack)

**Key Functions:**
- `fn benchmark_msgpack(data: &BenchmarkData) -> (usize, BenchmarkData)`: The roundtrip function for MessagePack.

**Role Each Plays:**
- **`rmp_serde::to_vec`**: Serializes data into a MessagePack `Vec<u8>`.
- **`rmp_serde::from_slice`**: Deserializes data from a MessagePack `&[u8]`.

### Checkpoint Tests
```rust
#[test]
fn test_msgpack_roundtrip() {
    let original_data = generate_data(10);
    let (size, roundtrip_data) = benchmark_msgpack(&original_data);

    assert!(size > 0, "MessagePack output should have a size");
    assert_eq!(original_data, roundtrip_data, "Data should be identical after MessagePack roundtrip");
}
```

### Starter Code
```rust
use rmp_serde;
// Assume BenchmarkData is available

pub fn benchmark_msgpack(data: &BenchmarkData) -> (usize, BenchmarkData) {
    // TODO: Serialize the data using rmp_serde.
    let encoded_data: Vec<u8> = todo!();

    let size = encoded_data.len();

    // TODO: Deserialize the data using rmp_serde.
    let deserialized_data = todo!();
    
    (size, deserialized_data)
}
```
---

## Milestone 5: Measuring Time with Criterion

### Introduction

**Why Previous Milestones Aren't Enough**: We've only measured size. The other critical factor is *speed*. This milestone introduces the `criterion` benchmarking harness to accurately measure the time it takes to serialize and deserialize for each format.

**The Improvement**: We move from simple size comparison to rigorous performance measurement. `criterion` runs our code many times to get statistically significant results, giving us reliable data on which format is fastest.

### Architecture

**Setup:**
- Create a `benches` directory in your project root.
- Create a file `benches/serialization_benchmark.rs`.
- Add `[[bench]]` configuration to `Cargo.toml`.

**Dependencies:**
- `criterion = "0.4"`

**Key Functions:**
- `fn serialization_benchmark(c: &mut Criterion)`: The main benchmark function that `criterion` will run.
- `c.bench_function(...)` and `c.benchmark_group(...)`: Criterion's API for defining and organizing benchmarks.

### Checkpoint (How to Run)

Run the benchmark with:
```sh
cargo bench
```
Criterion will produce a detailed report in `target/criterion/report/index.html`.

### Starter Code (`benches/serialization_benchmark.rs`)

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
// Import your project's functions and data structures
use your_project_name::{generate_data, BenchmarkData};
use your_project_name::{benchmark_json, benchmark_bincode, benchmark_msgpack};

fn bench_formats(c: &mut Criterion) {
    let data = generate_data(100);
    let mut group = c.benchmark_group("Serialization Comparison (size=100)");

    // --- Benchmark Serialization Speed ---
    
    // TODO: Benchmark JSON serialization speed.
    // Use `group.bench_function("JSON serialize", |b| b.iter(|| ...));`
    // The code inside `iter` is what gets measured.
    
    // TODO: Benchmark Bincode serialization speed.
    
    // TODO: Benchmark MessagePack serialization speed.
    
    // --- Benchmark Deserialization Speed ---
    let json_str = serde_json::to_string(&data).unwrap();
    let bincode_vec = bincode::serialize(&data).unwrap();
    let msgpack_vec = rmp_serde::to_vec(&data).unwrap();
    
    // TODO: Benchmark JSON deserialization speed.
    
    // TODO: Benchmark Bincode deserialization speed.

    // TODO: Benchmark MessagePack deserialization speed.

    group.finish();
}

criterion_group!(benches, bench_formats);
criterion_main!(benches);
```

**Implementation Hints:**
1.  Wrap the function call you're benchmarking in `black_box(...)` to prevent the compiler from optimizing it away. Example: `b.iter(|| serde_json::to_string(black_box(&data)))`.
2.  For deserialization benchmarks, prepare the serialized data *outside* the `iter` loop.

---

## Milestone 6: Generating a Markdown Report

### Introduction

**Why Milestone 5 Isn't Enough**: The `criterion` HTML report is excellent for detailed analysis, but a simple, shareable summary is often more useful. This final milestone focuses on creating a program that runs all the benchmarks and prints a clean markdown table to the console.

**The Improvement**: This makes the benchmark results easy to copy, paste, and share in documentation, pull requests, or articles. It's the final step in presenting your findings clearly.

### Architecture

**Key Functions:**
- `fn main()`: The main entry point of a new binary (`src/main.rs`) that will orchestrate the benchmarks and print the report.

### Checkpoint (How to Run)
```sh
cargo run
```

### Starter Code (`src/main.rs`)
```rust
use std::time::Instant;

fn main() {
    let data = generate_data(1000); // Use a larger dataset for the report

    println!("# Serialization Benchmark Report (Dataset Size: 1000)");
    println!("| Format      | Size (bytes) | Serialization Time (µs) | Deserialization Time (µs) |");
    println!("|-------------|--------------|---------------------------|-----------------------------|");

    // --- JSON ---
    let (json_size, _) = benchmark_json(&data);
    let ser_start = Instant::now();
    let json_str = serde_json::to_string(&data).unwrap();
    let ser_time = ser_start.elapsed().as_micros();
    
    let de_start = Instant::now();
    let _ = serde_json::from_str::<BenchmarkData>(&json_str).unwrap();
    let de_time = de_start.elapsed().as_micros();

    println!("| JSON        | {:<12} | {:<25} | {:<27} |", json_size, ser_time, de_time);
    
    // TODO: Repeat the process for Bincode.

    // TODO: Repeat the process for MessagePack.
}
```
---

## Complete Working Example

Here is a complete project structure and the code to achieve the final result.

**`Cargo.toml`**
```toml
[package]
name = "serialization_benchmark"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"
rmp-serde = "1.1"

[dev-dependencies]
criterion = "0.4"

[[bench]]
name = "serialization_benchmark"
harness = false
```

**`src/lib.rs`**
```rust
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BenchmarkData {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub timestamp: u64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
    pub active: bool,
}

pub fn generate_data(size: usize) -> BenchmarkData {
    let mut values = Vec::with_capacity(size);
    for i in 0..size {
        values.push((i as f64).sin() * 100.0);
    }

    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "benchmark".to_string());
    metadata.insert("version".to_string(), "1.0".to_string());

    BenchmarkData {
        id: 1,
        name: "Sample Data".to_string(),
        description: "A sample dataset for benchmarking serialization formats.".to_string(),
        timestamp: 1678886400,
        values,
        metadata,
        active: true,
    }
}
```

**`src/main.rs` (for the report)**
```rust
use serialization_benchmark::{generate_data, BenchmarkData};
use std::time::Instant;

fn main() {
    let data = generate_data(1000);

    println!("# Serialization Benchmark Report (Dataset Size: 1000)");
    println!("| Format      | Size (bytes) | Serialization Time (µs) | Deserialization Time (µs) |");
    println!("|-------------|--------------|---------------------------|-----------------------------|");

    // --- JSON ---
    let ser_start = Instant::now();
    let json_str = serde_json::to_string(&data).unwrap();
    let ser_time = ser_start.elapsed().as_micros();
    let json_size = json_str.len();
    
    let de_start = Instant::now();
    let _ = serde_json::from_str::<BenchmarkData>(&json_str).unwrap();
    let de_time = de_start.elapsed().as_micros();
    println!("| JSON        | {:<12} | {:<25} | {:<27} |", json_size, ser_time, de_time);
    
    // --- Bincode ---
    let ser_start = Instant::now();
    let bincode_vec = bincode::serialize(&data).unwrap();
    let ser_time = ser_start.elapsed().as_micros();
    let bincode_size = bincode_vec.len();

    let de_start = Instant::now();
    let _ = bincode::deserialize::<BenchmarkData>(&bincode_vec).unwrap();
    let de_time = de_start.elapsed().as_micros();
    println!("| Bincode     | {:<12} | {:<25} | {:<27} |", bincode_size, ser_time, de_time);

    // --- MessagePack ---
    let ser_start = Instant::now();
    let msgpack_vec = rmp_serde::to_vec(&data).unwrap();
    let ser_time = ser_start.elapsed().as_micros();
    let msgpack_size = msgpack_vec.len();

    let de_start = Instant::now();
    let _ = rmp_serde::from_slice::<BenchmarkData>(&msgpack_vec).unwrap();
    let de_time = de_start.elapsed().as_micros();
    println!("| MessagePack | {:<12} | {:<25} | {:<27} |", msgpack_size, ser_time, de_time);
}
```

**`benches/serialization_benchmark.rs` (for `cargo bench`)**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serialization_benchmark::{generate_data, BenchmarkData};

fn bench_formats(c: &mut Criterion) {
    let data = generate_data(100);
    let mut group = c.benchmark_group("Serialization Comparison (size=100)");

    // --- Serialization ---
    group.bench_function("JSON serialize", |b| b.iter(|| serde_json::to_string(black_box(&data))));
    group.bench_function("Bincode serialize", |b| b.iter(|| bincode::serialize(black_box(&data))));
    group.bench_function("MessagePack serialize", |b| b.iter(|| rmp_serde::to_vec(black_box(&data))));

    // --- Deserialization ---
    let json_str = serde_json::to_string(&data).unwrap();
    let bincode_vec = bincode::serialize(&data).unwrap();
    let msgpack_vec = rmp_serde::to_vec(&data).unwrap();

    group.bench_function("JSON deserialize", |b| b.iter(|| serde_json::from_str::<BenchmarkData>(black_box(&json_str))));
    group.bench_function("Bincode deserialize", |b| b.iter(|| bincode::deserialize::<BenchmarkData>(black_box(&bincode_vec))));
    group.bench_function("MessagePack deserialize", |b| b.iter(|| rmp_serde::from_slice::<BenchmarkData>(black_box(&msgpack_vec))));
    
    group.finish();
}

criterion_group!(benches, bench_formats);
criterion_main!(benches);
```
This project will give students a powerful, data-driven understanding of why serialization format choice is a critical engineering decision.
