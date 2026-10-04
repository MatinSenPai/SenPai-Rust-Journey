# Solution

```rust
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
        "the", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog", "while", "senpai",
        "reads", "rust", "documentation", "on", "a", "sunday", "afternoon", "in", "july",
    ];

    c.bench_function("concat_naive", |b| {
        b.iter(|| concat_naive(black_box(&words)))
    });
    c.bench_function("concat_with_capacity", |b| {
        b.iter(|| concat_with_capacity(black_box(&words)))
    });
}
```

## Why `black_box` wraps the *input*, not just the call

`fib_recursive(black_box(25))` wraps the literal `25`, not the whole call
expression. That's deliberate: if you instead wrote
`black_box(fib_recursive(25))`, the compiler is still free to see that
`25` is a compile-time constant and precompute `fib_recursive(25)` at
*compile* time, folding the entire call into a single constant load before
`black_box` ever sees it run — `black_box` can't stop an optimization that
already happened before its argument is evaluated. Wrapping the *input*
instead forces the compiler to treat `25` as a runtime-unknown value for
the whole rest of the expression, so `fib_recursive` actually has to run
with an argument it can't fold away. `words` in `string_concat_benchmark`
gets the same treatment for the same reason — `black_box(&words)` stops
the compiler from noticing `words` never changes across iterations and
hoisting/precomputing the concatenation once outside the timed loop.

## Why `n = 25`, not `n = 10` or `n = 40`

`fib_recursive` is O(2^n). At `n = 10` both variants finish so fast
(microseconds) that the timing noise floor — OS scheduler jitter, cache
effects unrelated to the algorithm — starts to dominate the actual signal,
making the comparison less trustworthy even with criterion's statistics.
At `n = 40`, `fib_recursive` alone would take the benchmark suite from
seconds to potentially minutes per run, painful for a "run this every time
you touch the code" workflow. `n = 25` (about 240,000 recursive calls) is
the middle ground: slow enough for `fib_recursive` that the difference from
`fib_iterative` is unmistakable in the report, fast enough that `cargo
bench` still finishes in a reasonable time.

## Reading the numbers once `cargo bench` runs

Expect `fib_iterative_25` to report a time in the tens-of-nanoseconds
range (a fixed 25 loop iterations, no allocation, no recursion overhead)
and `fib_recursive_25` to report something several orders of magnitude
larger (hundreds of microseconds to low milliseconds, machine-dependent) —
the exponential-vs-linear gap from `src/lib.rs`'s doc comments made
concrete. `concat_with_capacity` should report a smaller mean than
`concat_naive`, but the *gap* between them will be far less dramatic than
the Fibonacci pair — for a 20-word input, `concat_naive` triggers at most
a handful of reallocations (`String`'s growth strategy roughly doubles
capacity each time), not enough to swamp the runtime the way exponential
recursion does. That's a useful lesson on its own: preallocating is a real,
measurable win, but the *size* of a performance win depends enormously on
which cost you're avoiding — algorithmic complexity dwarfs a handful of
reallocations, which is exactly why `criterion`'s job is to give you real
numbers instead of leaving it to intuition.

## What the two `src/lib.rs` test functions were already telling you

`fib_iterative_matches_fib_recursive_across_a_range` and
`concat_naive_and_concat_with_capacity_produce_identical_output` establish
correctness — both variants in each pair agree on every output, for every
input tested. Benchmarking never replaces that: a faster function that
returns the wrong answer is a regression no matter what the flamegraph
says. Tests and benchmarks answer different questions and both need to
stay green; this lesson's `todo!()`s were only ever in the benchmark file
because the functions' *correctness* was never in question, only their
*speed*.
