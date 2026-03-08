use criterion::{criterion_group, criterion_main, Criterion};

fn gossip_benchmark(c: &mut Criterion) {
    c.bench_function("gossip_noop", |b| b.iter(|| 1 + 1));
}

criterion_group!(benches, gossip_benchmark);
criterion_main!(benches);
