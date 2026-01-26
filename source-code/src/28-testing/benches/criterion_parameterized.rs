// Pattern 8: Criterion Benchmarking - Parameterized Benchmarks
// Demonstrates benchmarking across multiple input sizes.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn sort_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort");

    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let data: Vec<i32> = (0..size).rev().collect();
            b.iter(|| {
                let mut d = data.clone();
                d.sort();
                black_box(d);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, sort_benchmark);
criterion_main!(benches);
