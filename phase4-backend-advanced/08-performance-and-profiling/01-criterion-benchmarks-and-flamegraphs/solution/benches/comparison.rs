use criterion::{black_box, criterion_group, criterion_main, Criterion};
use p4_08_01_criterion_benchmarks_and_flamegraphs_solution::{
    concat_naive, concat_with_capacity, fib_iterative, fib_recursive,
};

fn fibonacci_benchmark(c: &mut Criterion) {
    c.bench_function("fib_recursive_25", |b| {
        b.iter(|| fib_recursive(black_box(25)))
    });
    c.bench_function("fib_iterative_25", |b| {
        b.iter(|| fib_iterative(black_box(25)))
    });
}

fn string_concat_benchmark(c: &mut Criterion) {
    let words: Vec<&str> = vec![
        "the",
        "quick",
        "brown",
        "fox",
        "jumps",
        "over",
        "the",
        "lazy",
        "dog",
        "while",
        "senpai",
        "reads",
        "rust",
        "documentation",
        "on",
        "a",
        "sunday",
        "afternoon",
        "in",
        "july",
    ];

    c.bench_function("concat_naive", |b| {
        b.iter(|| concat_naive(black_box(&words)))
    });
    c.bench_function("concat_with_capacity", |b| {
        b.iter(|| concat_with_capacity(black_box(&words)))
    });
}

criterion_group!(benches, fibonacci_benchmark, string_concat_benchmark);
criterion_main!(benches);
