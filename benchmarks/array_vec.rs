#![feature(array_map, min_const_generics)]

use criterion::{black_box, criterion_group, BenchmarkGroup, criterion_main, Criterion, BenchmarkId, measurement::Measurement};

macro_rules! add_benchmark_group {
    (
        $criterion:expr,
        $f:ident,
        [$($n:expr),+],
        $static_vec:expr,
        $vec:expr,
        $vec_with_capacity:expr,
        $std_array_vec:expr,
        $tinyvec_array_vec:expr
    ) => {
        fn $f<M, const N: usize>(group: &mut BenchmarkGroup<'_, M>)
        where
            M: Measurement
        {
            group.bench_with_input(BenchmarkId::new("StaticVec", N), &N, |b, _| b.iter(|| {
                let mut v = staticvec::StaticVec::<usize, N>::new();
                $static_vec(&mut v);
            }));
            
            group.bench_with_input(BenchmarkId::new("Vec", N), &N, |b, _| b.iter(|| {
                let mut v: Vec<usize> = Vec::new();
                $vec(&mut v);
            }));
            
            group.bench_with_input(BenchmarkId::new("Vec::with_capacity", N), &N, |b, _| b.iter(|| {
                let mut v: Vec<usize> = Vec::with_capacity(N);
                $vec_with_capacity(&mut v);
            }));
            
            group.bench_with_input(BenchmarkId::new("std::ArrayVec", N), &N, |b, _| b.iter(|| {
                let mut v = stack_based_vec::ArrayVec::<usize, N>::new();
                $std_array_vec(&mut v);
            }));
            
            group.bench_with_input(BenchmarkId::new("tinyvec::ArrayVec", N), &N, |b, _| b.iter(|| {
                let mut v = tinyvec::ArrayVec::<[usize; N]>::new();
                $tinyvec_array_vec(&mut v);
            }));
        }

        let mut group = $criterion.benchmark_group(stringify!($f));
        $( $f::<_, $n>(&mut group); )+
        group.finish();
    };
}

fn criterion_benchmark(c: &mut Criterion) {
    add_benchmark_group!(
        c,
        extend_from_copyable_slice,
        [99, 9999],
        |v: &mut staticvec::StaticVec::<usize, N>| {
            let array = ascending_array::<N>();
            let _ = v.extend_from_slice(black_box(&array[..]));
        },
        |v: &mut Vec<usize>| {
            let array = ascending_array::<N>();
            v.extend(black_box(&array[..]));
        },
        |v: &mut Vec<usize>| {
            let array = ascending_array::<N>();
            v.extend(black_box(&array[..]));
        },
        |v: &mut stack_based_vec::ArrayVec::<usize, N>| {
            let array = ascending_array::<N>();
            let _ = v.extend_from_copyable_slice(black_box(&array[..]));
        },
        |v: &mut tinyvec::ArrayVec::<[usize; N]>| {
            let array = ascending_array::<N>();
            let _ = v.extend_from_slice(black_box(&array[..]));
        }
    );

    add_benchmark_group!(
        c,
        push,
        [99, 9999],
        |v: &mut staticvec::StaticVec::<usize, N>| {
            for elem in 0..N {
                let _ = v.try_push(black_box(elem));
            }
        },
        |v: &mut Vec<usize>| {
            for elem in 0..N {
                v.push(black_box(elem));
            }
        },
        |v: &mut Vec<usize>| {
            for elem in 0..N {
                v.push(black_box(elem));
            }
        },
        |v: &mut stack_based_vec::ArrayVec::<usize, N>| {
            for elem in 0..N {
                let _ = v.push(black_box(elem));
            }
        },
        |v: &mut tinyvec::ArrayVec::<[usize; N]>| {
            for elem in 0..N {
                let _ = v.try_push(black_box(elem));
            }
        }
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

#[inline]
fn ascending_array<const N: usize>() -> [usize; N] {
    let mut idx = 0;
    [(); N].map(|_| {
        idx += 1;
        idx
    })
}
