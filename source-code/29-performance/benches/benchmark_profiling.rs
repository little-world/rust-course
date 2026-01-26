// Criterion benchmark for profiling demonstration

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn validate(s: &str) -> bool {
    s.len() > 10 && s.chars().all(|c| c.is_alphanumeric())
}

fn transform(s: &str) -> String {
    s.to_uppercase()
}

fn process_data(items: &[String]) -> Vec<String> {
    items.iter()
        .filter(|s| validate(s))
        .map(|s| transform(s))
        .collect()
}

fn benchmark(c: &mut Criterion) {
    let data: Vec<String> = (0..1000)
        .map(|i| format!("item_number_{}", i))
        .collect();

    c.bench_function("process_data", |b| {
        b.iter(|| process_data(black_box(&data)))
    });

    c.bench_function("validate", |b| {
        b.iter(|| {
            for item in &data {
                black_box(validate(black_box(item)));
            }
        })
    });

    c.bench_function("transform", |b| {
        b.iter(|| {
            for item in &data {
                black_box(transform(black_box(item)));
            }
        })
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
