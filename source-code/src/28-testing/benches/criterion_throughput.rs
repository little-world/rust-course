// Pattern 8: Criterion Benchmarking - Throughput Measurement
// Demonstrates measuring throughput in bytes per second.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

fn parse_numbers(data: &str) -> Vec<i32> {
    data.lines()
        .filter_map(|line| line.parse().ok())
        .collect()
}

fn throughput_benchmark(c: &mut Criterion) {
    let data = (0..10000).map(|i| i.to_string()).collect::<Vec<_>>().join("\n");
    let data_bytes = data.len();

    let mut group = c.benchmark_group("parse_throughput");
    group.throughput(Throughput::Bytes(data_bytes as u64));

    group.bench_function("parse", |b| {
        b.iter(|| parse_numbers(black_box(&data)))
    });

    group.finish();
}

criterion_group!(benches, throughput_benchmark);
criterion_main!(benches);
