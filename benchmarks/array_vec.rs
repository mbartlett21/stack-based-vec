#![allow(missing_docs)]
#![feature(array_map, min_const_generics)]

use criterion::{
    black_box, criterion_group, criterion_main, measurement::Measurement, BenchmarkGroup,
    BenchmarkId, Criterion,
};

macro_rules! add_benchmark_group {
    (
        $criterion:expr,
        $f:ident,
        $vec:expr,
        $vec_with_capacity:expr,
        $std_array_vec:expr,
    ) => {
        fn $f<M, const N: usize>(group: &mut BenchmarkGroup<'_, M>)
        where
            M: Measurement,
        {
            group.bench_with_input(BenchmarkId::new("Vec", N), &N, |b, _| {
                b.iter(|| {
                    let mut v: Vec<usize> = Vec::new();
                    $vec(&mut v);
                })
            });

            group.bench_with_input(BenchmarkId::new("Vec::with_capacity", N), &N, |b, _| {
                b.iter(|| {
                    let mut v: Vec<usize> = Vec::with_capacity(N);
                    $vec_with_capacity(&mut v);
                })
            });

            group.bench_with_input(BenchmarkId::new("std::ArrayVec", N), &N, |b, _| {
                b.iter(|| {
                    let mut v = stack_based_vec::ArrayVec::<usize, N>::new();
                    $std_array_vec(&mut v);
                })
            });
        }

        let mut group = $criterion.benchmark_group(stringify!($f));
        $f::<_, 99>(&mut group);
        $f::<_, 9999>(&mut group);
        group.finish();
    };
}

macro_rules! extend_vec_with_ascending_array {
    ($v:expr) => {
        let array = ascending_array::<N>();
        let _ = $v.extend(black_box(array.iter().copied()));
    };
}

fn criterion_benchmark(c: &mut Criterion) {
    add_benchmark_group!(
        c,
        clone,
        |v: &mut Vec<usize>| {
            extend_vec_with_ascending_array!(v);
            let _another = v.clone();
        },
        |v: &mut Vec<usize>| {
            extend_vec_with_ascending_array!(v);
            let _another = v.clone();
        },
        |v: &mut stack_based_vec::ArrayVec::<usize, N>| {
            extend_vec_with_ascending_array!(v);
            let _another = v.clone();
        },
    );

    add_benchmark_group!(
        c,
        extend,
        |v: &mut Vec<usize>| {
            extend_vec_with_ascending_array!(v);
        },
        |v: &mut Vec<usize>| {
            extend_vec_with_ascending_array!(v);
        },
        |v: &mut stack_based_vec::ArrayVec::<usize, N>| {
            extend_vec_with_ascending_array!(v);
        },
    );

    add_benchmark_group!(
        c,
        extend_from_copyable_slice,
        |v: &mut Vec<usize>| {
            let array = ascending_array::<N>();
            let _ = v.extend(black_box(&array[..]));
        },
        |v: &mut Vec<usize>| {
            let array = ascending_array::<N>();
            let _ = v.extend(black_box(&array[..]));
        },
        |v: &mut stack_based_vec::ArrayVec::<usize, N>| {
            let array = ascending_array::<N>();
            let _ = v.extend_from_copyable_slice(black_box(&array[..]));
        },
    );

    add_benchmark_group!(
        c,
        pop,
        |v: &mut Vec<usize>| {
            extend_vec_with_ascending_array!(v);
            for _ in 0..N {
                let _ = v.pop();
            }
        },
        |v: &mut Vec<usize>| {
            extend_vec_with_ascending_array!(v);
            for _ in 0..N {
                let _ = v.pop();
            }
        },
        |v: &mut stack_based_vec::ArrayVec::<usize, N>| {
            extend_vec_with_ascending_array!(v);
            for _ in 0..N {
                let _ = v.pop();
            }
        },
    );

    add_benchmark_group!(
        c,
        push,
        |v: &mut Vec<usize>| {
            for elem in 0..N {
                let _ = v.push(black_box(elem));
            }
        },
        |v: &mut Vec<usize>| {
            for elem in 0..N {
                let _ = v.push(black_box(elem));
            }
        },
        |v: &mut stack_based_vec::ArrayVec::<usize, N>| {
            for elem in 0..N {
                let _ = v.push(black_box(elem));
            }
        },
    );

    add_benchmark_group!(
        c,
        truncate,
        |v: &mut Vec<usize>| {
            extend_vec_with_ascending_array!(v);
            v.truncate(0);
        },
        |v: &mut Vec<usize>| {
            extend_vec_with_ascending_array!(v);
            v.truncate(0);
        },
        |v: &mut stack_based_vec::ArrayVec::<usize, N>| {
            extend_vec_with_ascending_array!(v);
            v.truncate(0);
        },
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
