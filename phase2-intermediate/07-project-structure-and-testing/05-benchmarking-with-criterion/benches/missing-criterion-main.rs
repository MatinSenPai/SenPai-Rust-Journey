//! DELIBERATELY BROKEN — expected: E0601
//! Run `cargo build -p p2-07-05-benchmarking-with-criterion --bench missing-criterion-main --features broken`
//!
//! `criterion_group!` only registers which functions to run — it does not
//! generate `fn main`. That is `criterion_main!`'s only job, and it is easy
//! to type the first macro, see it "work" in your editor, and forget the
//! second.

use criterion::{criterion_group, Criterion};
use p2_07_05_benchmarking_with_criterion::contains_linear;

fn lookup_benchmark(c: &mut Criterion) {
    let haystack: Vec<u32> = (0..1_000).collect();
    c.bench_function("contains_linear_1000", |b| {
        b.iter(|| contains_linear(&haystack, 999_999))
    });
}

criterion_group!(benches, lookup_benchmark);
