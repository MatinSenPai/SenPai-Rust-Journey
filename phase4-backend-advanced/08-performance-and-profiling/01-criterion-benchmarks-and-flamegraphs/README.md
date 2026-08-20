# 08.1 — Criterion benchmarks and flamegraphs

Every lesson so far has answered one question: "is this correct" — does
the code produce the right output, handle the right edge cases, return the
right error. `cargo test` is built entirely around that question. This
lesson is about a different one: "is this *fast*, and did my change make
it faster or slower" — a question tests don't answer, and a question a
naive attempt to answer it can actively mislead you about.

## Why `std::time::Instant` in a `#[test]` is misleading

The obvious first instinct is:

```rust
let start = std::time::Instant::now();
my_function(input);
println!("{:?}", start.elapsed());
```

This runs, prints *a* number, and tells you almost nothing trustworthy:

- **One sample, no statistics.** A single timing is one data point pulled
  from a noisy process — OS scheduler jitter, whatever else is running on
  the machine, CPU frequency scaling — with no indication of how much that
  number varies run to run. Was this run typical, or an outlier?
- **No warmup.** The first call to a function pays costs later calls don't:
  cold CPU caches, a cold branch predictor, page faults for memory that
  hasn't been touched yet, and — for release builds compiled with certain
  optimizations, or in JIT'd languages generally — code that hasn't
  "warmed up" yet. A single untimed measurement bundles all of that into
  the result, even though none of it reflects the function's steady-state
  performance in a long-running process.
- **No comparison, no regression detection.** A single printed number
  answers "how long did this take just now," not "is this faster or
  slower than yesterday's version," which is almost always the question
  you actually care about when optimizing.

## What `criterion` does instead

`criterion` (already available as `criterion.workspace = true`, pinned to
`"0.5"` in the root `Cargo.toml`) is a statistically rigorous benchmarking
harness:

- **Warmup iterations** run first and are discarded, so the timed
  iterations reflect steady-state performance, not cold-start cost.
- **Many samples**, not one — criterion runs the benchmarked code
  repeatedly (the exact count is adaptive, based on how fast each
  iteration is) and reports a distribution: mean, standard deviation,
  confidence intervals — not a single number that might be an outlier.
- **Regression detection against a saved baseline.** `cargo bench` writes
  results to `target/criterion/`; run it again after a code change and
  criterion compares the new distribution to the previous one, reporting
  whether the change was a statistically significant improvement,
  regression, or noise — not just "the number went down," but "the number
  went down by more than measurement noise can explain."
- **HTML reports** (`target/criterion/report/index.html` after a local
  `cargo bench` run) with violin plots and time-series-over-runs charts,
  useful for spotting a slow drift across many small changes that no
  single before/after comparison would catch.

## Reading `src/lib.rs`

Two pairs, both fully implemented (no `todo!()` here — the exercise is the
benchmark, not these functions):

- **`fib_recursive` vs. `fib_iterative`** — the textbook naive recursive
  Fibonacci (`fib(n-1) + fib(n-2)`, exponential time, because `fib(2)` gets
  recomputed hundreds of thousands of times for larger `n`) against a
  linear-time loop that keeps only the last two values. Same output for
  every input (see `fib_iterative_matches_fib_recursive_across_a_range` in
  the test module), wildly different Big-O.
- **`concat_naive` vs. `concat_with_capacity`** — building one `String` out
  of a slice of words by repeated `push_str` into a `String::new()`
  (capacity 0, so the buffer reallocates and copies its contents forward
  every time it outgrows its current capacity) versus computing the exact
  final byte length up front and allocating it once with
  `String::with_capacity`. Ties back to Phase 1's `String`/`&str` lesson:
  preallocating is only possible because the total *owned* size can be
  computed cheaply before writing any of it.

Both pairs are deliberately concrete and easy to reason about by hand —
you can predict which variant of each pair will win before running
anything, which is the point: the benchmark should *confirm* your
reasoning about complexity and allocation with real numbers, not be your
only source of truth about which one is faster.

## Reading `benches/comparison.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn fibonacci_benchmark(c: &mut Criterion) {
    // your task
}

criterion_group!(benches, fibonacci_benchmark, string_concat_benchmark);
criterion_main!(benches);
```

- **`Cargo.toml`'s `[[bench]]` section** (`name = "comparison"`, `harness
  = false`) tells cargo two things: there's a benchmark target at
  `benches/comparison.rs`, and *don't* use Rust's built-in (nightly-only,
  much less capable) `#[bench]` harness for it — `harness = false` hands
  control of `main` entirely to criterion's own harness instead, which is
  what `criterion_main!` actually generates.
- **`criterion_group!`/`criterion_main!`** are macros that generate the
  `fn main()` for this benchmark binary — wiring your benchmark functions
  into criterion's runner, its CLI argument parsing (`cargo bench --
  --save-baseline my-branch`, etc.), and its report generation. You never
  write `fn main()` yourself in a criterion benchmark file.
- **`c.bench_function(name, |b| b.iter(|| ...))`** is the actual
  measurement call: `name` is how this benchmark shows up in output and
  the HTML report, and the closure passed to `b.iter` is what gets run
  repeatedly and timed. Whatever that inner closure returns is what
  criterion measures the cost of producing.
- **`black_box`** — see the doc comment at the top of the file. The short
  version: an optimizing compiler is entitled to delete code whose result
  is provably unused, or fold a call on a constant input into its
  precomputed answer at compile time. Either would silently turn "time
  spent computing `fib_recursive(25)`" into "time spent doing nothing" —
  `black_box` is an identity function the optimizer can't see through,
  forcing it to treat the wrapped value as genuinely unknown so the real
  work can't be optimized away.

## Flamegraphs: the next question after "this is slow"

A benchmark tells you *that* one implementation is slower than another and
by how much. It doesn't tell you *why* a single implementation is slow —
which function, which line, is actually eating the time. A **flamegraph**
answers that: it's a visualization of a program's call stack sampled
repeatedly while it runs, where each function's box width is proportional
to the total time samples landed inside it (directly or in something it
called), and boxes stack vertically by call depth — the function at the
bottom of a stack of boxes called everything above it. **Wide bars are
where the time is going**; a flamegraph with one enormous box somewhere in
the middle, dwarfing everything around it, is pointing directly at the
function worth optimizing next. A flat, evenly-distributed flamegraph with
no single wide bar tells you the opposite: there's no one hot spot to fix,
and further optimization needs to target the whole shape of the algorithm,
not one function.

`cargo flamegraph` (a separate cargo subcommand, `cargo install
flamegraph`, wrapping the Linux `perf` profiler or `dtrace` on macOS) is
the real tool for generating one against an actual binary — it runs your
program under a sampling profiler, then renders the collected stack
samples as an interactive SVG. This sandbox has neither `perf` nor a real
workload worth profiling for this lesson's tiny functions, so this section
is reading-only: understand what a flamegraph shows and when you'd reach
for one (after a benchmark or a production latency metric tells you
*something* is slow, when you need to find out *what*), rather than
generating one here. In a real investigation, the workflow is: notice a
regression (via criterion locally, or a latency metric in production),
reach for `cargo flamegraph` (or attach `perf` to a running process) to
find the specific hot function, fix it, then re-run the criterion
benchmark to confirm the fix actually moved the number.

## Your task

Open `benches/comparison.rs`. Fill in `fibonacci_benchmark` and
`string_concat_benchmark`: register a `c.bench_function(...)` call for
each of the four functions in `src/lib.rs`, using `black_box` on every
benchmarked input so the compiler can't optimize the call away.

`src/lib.rs` needs no changes — `fib_recursive`, `fib_iterative`,
`concat_naive`, and `concat_with_capacity` are already implemented and
already tested; read them to understand what you're about to benchmark.

## Next

`cargo test -p p4-08-01-criterion-benchmarks-and-flamegraphs` to confirm
the functions themselves are correct, then `cargo bench -p
p4-08-01-criterion-benchmarks-and-flamegraphs` once the two `todo!()`s are
filled in (this takes longer than a normal test run — criterion runs many
warmup and measured iterations on purpose). Then `solution/SOLUTION.md` — but only after a real attempt.
