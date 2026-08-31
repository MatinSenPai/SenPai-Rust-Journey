//! Reference solution's benchmark file — includes the Build exercise's
//! third benchmark group, comparing `sum_of_squares_loop` against
//! `sum_of_squares_iter`.

use std::collections::HashSet;
use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use p2_07_05_benchmarking_with_criterion_solution::{
    contains_hashset, contains_linear, sum_of_squares_iter, sum_of_squares_loop,
};

/// Deliberately trivial — multiply by itself — so its only job is
/// demonstrating what `black_box` protects against. Not a serious
/// benchmark subject on its own.
fn square(x: u64) -> u64 {
    x * x
}

fn lookup_benchmark(c: &mut Criterion) {
    let haystack: Vec<u32> = (0..1_000).collect();
    let set: HashSet<u32> = haystack.iter().copied().collect();
    let needle = 999_999;

    c.bench_function("contains_linear_1000", |b| {
        b.iter(|| contains_linear(black_box(&haystack), black_box(needle)))
    });
    c.bench_function("contains_hashset_1000", |b| {
        b.iter(|| contains_hashset(black_box(&set), black_box(needle)))
    });
}

fn black_box_benchmark(c: &mut Criterion) {
    c.bench_function("square_with_black_box", |b| b.iter(|| square(black_box(7))));
    c.bench_function("square_without_black_box", |b| b.iter(|| square(7)));
}

/// The Build exercise: compare the loop and iterator versions of
/// `sum_of_squares` at the same, large-enough `n`.
fn sum_of_squares_benchmark(c: &mut Criterion) {
    let n = 10_000u32;

    c.bench_function("sum_of_squares_loop_10000", |b| {
        b.iter(|| sum_of_squares_loop(black_box(n)))
    });
    c.bench_function("sum_of_squares_iter_10000", |b| {
        b.iter(|| sum_of_squares_iter(black_box(n)))
    });
}

criterion_group!(
    benches,
    lookup_benchmark,
    black_box_benchmark,
    sum_of_squares_benchmark
);
criterion_main!(benches);
