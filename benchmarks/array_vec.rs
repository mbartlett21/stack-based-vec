#![feature(array_map, min_const_generics)]

use criterion::{black_box, criterion_group, BenchmarkGroup, criterion_main, Criterion, BenchmarkId, measurement::Measurement};
use stack_based_vec::ArrayVec;

fn extend_from_copyable_slice<M, const N: usize>(group: &mut BenchmarkGroup<'_, M>)
where
    M: Measurement
{
    group.bench_with_input(BenchmarkId::new("ArrayVec::new", N), &N, |b, _| b.iter(|| {
        let mut v = ArrayVec::<usize, N>::new();
        let mut idx = 0;
        let array: [usize; N] = [(); N].map(|_| {
            idx += 1;
            idx
        });
        let _ = v.extend_from_copyable_slice(black_box(&array[..]));
    }));

    group.bench_with_input(BenchmarkId::new("Vec::new", N), &N, |b, _| b.iter(|| {
        let mut v: Vec<usize> = Vec::new();
        let mut idx = 0;
        let array: [usize; N] = [(); N].map(|_| {
            idx += 1;
            idx
        });
        v.extend(black_box(&array[..]));
    }));

    group.bench_with_input(BenchmarkId::new("Vec::with_capacity", N), &N, |b, _| b.iter(|| {
        let mut v: Vec<usize> = Vec::with_capacity(N);
        let mut idx = 0;
        let array: [usize; N] = [(); N].map(|_| {
            idx += 1;
            idx
        });
        v.extend(black_box(&array[..]));
    }));
}

fn push<M, const N: usize>(group: &mut BenchmarkGroup<'_, M>)
where
    M: Measurement
{
    group.bench_with_input(BenchmarkId::new("ArrayVec::new", N), &N, |b, _| b.iter(|| {
        let mut v = ArrayVec::<usize, N>::new();
        for elem in 0..N {
            let _ = v.push(black_box(elem));
        }
    }));

    group.bench_with_input(BenchmarkId::new("Vec::new", N), &N, |b, _| b.iter(|| {
        let mut v: Vec<usize> = Vec::new();
        for elem in 0..N {
            v.push(black_box(elem));
        }
    }));

    group.bench_with_input(BenchmarkId::new("Vec::with_capacity", N), &N, |b, _| b.iter(|| {
        let mut v: Vec<usize> = Vec::with_capacity(N);
        for elem in 0..N {
            v.push(black_box(elem));
        }
    }));
}

macro_rules! add_benchmark {
    ([$($n:expr),+], $f:ident, $group:expr) => {
        $(
            $f::<_, $n>(&mut $group);
        )+
    };
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("extend_from_copyable_slice");
    add_benchmark!([5000], extend_from_copyable_slice, group);
    group.finish();

    let mut group = c.benchmark_group("push");
    add_benchmark!([5000], push, group);
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);