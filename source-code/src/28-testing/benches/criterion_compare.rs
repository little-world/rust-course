// Pattern 8: Criterion Benchmarking - Comparing Implementations
// Demonstrates comparing multiple implementations of the same function.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn sum_loop(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data {
        sum += x;
    }
    sum
}

fn sum_iterator(data: &[i32]) -> i32 {
    data.iter().sum()
}

fn sum_fold(data: &[i32]) -> i32 {
    data.iter().fold(0, |acc, &x| acc + x)
}

fn benchmark_sum_implementations(c: &mut Criterion) {
    let mut group = c.benchmark_group("sum_implementations");

    let data: Vec<i32> = (0..1000).collect();

    group.bench_with_input(BenchmarkId::new("loop", data.len()), &data, |b, data| {
        b.iter(|| sum_loop(black_box(data)))
    });

    group.bench_with_input(BenchmarkId::new("iterator", data.len()), &data, |b, data| {
        b.iter(|| sum_iterator(black_box(data)))
    });

    group.bench_with_input(BenchmarkId::new("fold", data.len()), &data, |b, data| {
        b.iter(|| sum_fold(black_box(data)))
    });

    group.finish();
}

criterion_group!(benches, benchmark_sum_implementations);
criterion_main!(benches);
