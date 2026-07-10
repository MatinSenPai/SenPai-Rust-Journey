//! Your task: fill in the two `todo!()`s below. Everything being
//! benchmarked (`fib_recursive`, `fib_iterative`, `concat_naive`,
//! `concat_with_capacity`) is already fully implemented in `src/lib.rs` —
//! this file's job is only to *measure* them correctly.
//!
//! `criterion::black_box` (re-exported here from `std::hint::black_box`)
//! matters more than it looks: without wrapping a benchmarked call's
//! input (and often its output) in `black_box`, an optimizing compiler
//! that can see the input is a constant and the result is never used is
//! entitled to prove the whole call has no observable effect and delete
//! it outright — you'd end up benchmarking an empty loop, not your
//! function. `black_box` is an opaque-to-the-optimizer identity function:
//! it forces the compiler to treat the value as unknown, so the real
//! computation can't be optimized away.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use p4_08_01_criterion_benchmarks_and_flamegraphs::{
    concat_naive, concat_with_capacity, fib_iterative, fib_recursive,
};

/// Compares `fib_recursive` and `fib_iterative` at the same input.
/// `n = 25` is deliberately small enough that `fib_iterative` still
/// finishes in nanoseconds, but large enough that `fib_recursive`'s
/// O(2^n) blowup is already clearly, measurably slower — big enough for
/// the difference to show up plainly in criterion's report without
/// making the whole benchmark suite slow to run.
fn fibonacci_benchmark(c: &mut Criterion) {
    todo!(
        "Register two benchmarks on c: \
         c.bench_function(\"fib_recursive_25\", |b| b.iter(|| fib_recursive(black_box(25)))); \
         then the same shape for fib_iterative named \"fib_iterative_25\". \
         black_box(25) stops the compiler from constant-folding the call away."
    )
}

/// Compares `concat_naive` and `concat_with_capacity` over the same
/// input slice of words.
fn string_concat_benchmark(c: &mut Criterion) {
    todo!(
        "Build a words: Vec<&str> of a few dozen short strings (any content), then \
         register c.bench_function(\"concat_naive\", |b| b.iter(|| concat_naive(black_box(&words)))) \
         and the same shape for concat_with_capacity named \"concat_with_capacity\". \
         black_box(&words) stops the compiler from hoisting/constant-folding the call."
    )
}

criterion_group!(benches, fibonacci_benchmark, string_concat_benchmark);
criterion_main!(benches);
