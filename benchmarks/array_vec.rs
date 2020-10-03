use criterion::{black_box, criterion_group, criterion_main, Criterion};
use stack_based_vec::ArrayVec;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Something", |b| b.iter(|| {
        let mut vec = ArrayVec::<i32, 64>::new();
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);