// Criterion benchmark comparing sum implementations

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn sum_loop(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data {
        sum += x;
    }
    sum
}

fn sum_iter(data: &[i32]) -> i32 {
    data.iter().sum()
}

fn sum_fold(data: &[i32]) -> i32 {
    data.iter().fold(0, |acc, &x| acc + x)
}

fn benchmark_alternatives(c: &mut Criterion) {
    let data: Vec<i32> = (0..10000).collect();

    c.bench_function("sum_loop", |b| {
        b.iter(|| {
            let mut sum = 0;
            for &x in black_box(&data) {
                sum += black_box(x);
            }
            black_box(sum)
        })
    });

    c.bench_function("sum_iter", |b| {
        b.iter(|| {
            black_box(&data).iter().sum::<i32>()
        })
    });

    c.bench_function("sum_fold", |b| {
        b.iter(|| {
            black_box(&data).iter().fold(0, |acc, &x| acc + x)
        })
    });
}

criterion_group!(benches, benchmark_alternatives);
criterion_main!(benches);
