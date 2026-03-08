use criterion::{criterion_group, criterion_main, Criterion};

fn bloom_filter_benchmark(c: &mut Criterion) {
    c.bench_function("bloom_noop", |b| b.iter(|| 2 + 2));
}

criterion_group!(benches, bloom_filter_benchmark);
criterion_main!(benches);
