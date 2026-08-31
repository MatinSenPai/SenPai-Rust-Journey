//! A real, runnable criterion benchmark. This file is not an exercise —
//! everything here already works. Run it yourself:
//!
//!     cargo bench -p p2-07-05-benchmarking-with-criterion
//!
//! The "Build" exercise asks you to add a third benchmark function to this
//! same file, once `sum_of_squares_loop`/`sum_of_squares_iter` exist.

use std::collections::HashSet;
use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use p2_07_05_benchmarking_with_criterion::{contains_hashset, contains_linear};

/// Deliberately trivial — multiply by itself — so its only job is
/// demonstrating what `black_box` protects against below. Not a serious
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

criterion_group!(benches, lookup_benchmark, black_box_benchmark);
criterion_main!(benches);
