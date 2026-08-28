# 2.7.5 — Benchmarking with `criterion`

## At a glance

After this lesson you can:

- Explain why a single `Instant::now()`/`.elapsed()` around a function is not benchmarking — and name its three real reasons: run-to-run noise, CPU frequency scaling, and a single sample with no statistics behind it.
- Build a real `criterion` benchmark (`criterion_group!`/`criterion_main!`), run `cargo bench`, and correctly read its mean-plus-confidence-interval output.
- Say exactly what `std::hint::black_box` stops from happening, and why every `criterion` benchmark needs it.
- Decide, for a real piece of code, when benchmarking is actually worth doing — and why "I'm worried about it right now" is not a good enough reason.

**Time:** ~55 minutes · **Prerequisites:** [2.7.4 — Property testing with `proptest`, snapshot testing with `insta`](../04-property-and-snapshot-testing/README.md)

---

## Why this matters

Every lesson in this module has, so far, answered one question: "is this code correct?" — exactly what `cargo test` is built entirely around. This lesson answers a different one: "is this code fast, and did the change I just made actually help, or did it just feel like it did?" — a question tests never answer, and a question a naive attempt to answer can actively mislead you about, not just fail to help with.

If you've worked with Python, you already know the shape of this problem. `timeit` doesn't time a snippet once — it runs it many times, precisely because a single `time.time()` is too noisy to trust. `criterion` solves the same problem, only more rigorously: not a raw average, but the full distribution — mean, confidence interval, and which samples were statistical "outliers" you shouldn't weigh too heavily. The bridge breaks here, though: you typically run `timeit` by hand, once; `criterion` becomes part of the development workflow — it saves its results and compares the next run against the last one.

And this is the last piece of this module's puzzle. [2.7.1](../01-modules-visibility-workspaces/README.md) gave code structure; [2.7.2](../02-unit-integration-doc-tests/README.md) gave you three kinds of tests to prove that structure was correct; [2.7.3](../03-test-doubles-in-rust/README.md) gave you doubles to make swappable dependencies testable; [2.7.4](../04-property-and-snapshot-testing/README.md) showed you how to test when the input space was too large or the output too complex. All of them answer one question: "does this actually work?" Today, for the first time, the question changes.

---

## The concept

### Why a stopwatch around one run fools you

Here's a function — a linear scan, one element at a time, to the end of the list:

```rust
pub fn contains_linear(haystack: &[u32], needle: u32) -> bool {
    haystack.iter().any(|&item| item == needle)
}
```

The obvious first instinct: wrap it in an `Instant` and see how long it takes.

```rust
let start = Instant::now();
let found = contains_linear(&haystack, 999_999);
println!("run 1: found={found}  elapsed={:?}", start.elapsed());
```

Call it three times in a row, in one run — exactly what `examples/01-naive-timing.rs` does. Its real output, on this machine, from two separate runs of the program:

```text
run 1: found=false  elapsed=2.3µs
run 2: found=false  elapsed=300ns
run 3: found=false  elapsed=600ns
```

```text
run 1: found=false  elapsed=1.8µs
run 2: found=false  elapsed=300ns
run 3: found=false  elapsed=300ns
```

On your machine these numbers will differ — that is exactly the point. Three things are happening at once:

- **No warmup.** `run 1` took several times longer than `run 2`/`run 3` every time, despite calling the exact same function on the exact same input. The first call pays a cost later calls don't — a cold cache, a cold branch predictor. A single measurement with no warmup bundles that cost into the result, even though it has nothing to do with the function's steady-state behavior inside a real, long-running program.
- **Run-to-run noise.** Even `run 3` differed between the two runs: 600 nanoseconds versus 300. The OS scheduler, whatever else is running on the machine, even memory layout — all of it leaves a mark on the number you see, and none of it is about your code.
- **CPU frequency scaling.** Modern CPUs raise and lower their clock speed moment to moment, depending on temperature and power draw (Turbo Boost and similar). The exact same code runs faster in a moment when the processor happens to be "boosted" than in a moment when it isn't — without a single byte of the code changing.

With one sample, you cannot tell these three apart. `run 1` was a real number, but the *wrong* number for the question you're actually asking: "in steady state, how long does this function usually take?"

### What `criterion` actually does differently

`criterion` calls a function hundreds or thousands of times, spends the first few seconds warming up and throws those samples away, then collects the real samples and computes statistics on them — not one number, but a **mean with a confidence interval**, plus flagging of samples that were statistical outliers.

This piece is from `benches/comparison.rs` — a file that already works right now; it is not an exercise:

```rust
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
```

`c.bench_function(name, |b| b.iter(|| ...))` is the heart of it: `name` shows up in the output and the report; the closure inside `b.iter` is what actually gets run and timed, over and over. `criterion_group!`/`criterion_main!` (at the bottom of the same file) generate `fn main` for you — you never write one by hand. And `Cargo.toml` has to say `harness = false`, because `criterion` brings its own harness instead of Rust's built-in, nightly-only one.

Run it — `cargo bench -p p2-07-05-benchmarking-with-criterion` — and this is its real output:

```text
Gnuplot not found, using plotters backend
Benchmarking contains_linear_1000
Benchmarking contains_linear_1000: Warming up for 3.0000 s
Benchmarking contains_linear_1000: Collecting 100 samples in estimated 5.0010 s (21M iterations)
Benchmarking contains_linear_1000: Analyzing
contains_linear_1000    time:   [240.95 ns 242.13 ns 243.40 ns]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
```

```text
Benchmarking contains_hashset_1000
Benchmarking contains_hashset_1000: Warming up for 3.0000 s
Benchmarking contains_hashset_1000: Collecting 100 samples in estimated 5.0000 s (746M iterations)
Benchmarking contains_hashset_1000: Analyzing
contains_hashset_1000   time:   [6.3456 ns 6.4187 ns 6.5034 ns]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
```

Your own numbers will differ — but not the shape. The three numbers inside `[...]` are the lower bound of the confidence interval, the mean, and the upper bound; when you want to quote one number, quote the middle one. `contains_linear_1000` here is 242.13 nanoseconds, `contains_hashset_1000` only 6.42 — roughly 38 times faster. This is exactly what [2.1.4](../../01-collections/04-choosing-a-collection/README.md) told you as theory: `HashMap`/`HashSet` answer membership in average `O(1)`; a `Vec` has to look at every element. Today you saw it as a number, not just heard it.

And take "Found N outliers" seriously: `criterion` doesn't hide these samples or quietly throw them away — it flags them, so you know how many of the hundred were unrelated to the function's steady-state behavior (a scheduler hiccup, a busy moment on the machine). That is exactly what a single `println!(elapsed)` never tells you.

### `black_box`: keeping the optimizer honest

Optimizing compilers are entitled to delete a computation whose result is never used anywhere, or to see that an input is constant and precompute the answer at compile time. Both are good things in real code — they make it faster. But inside a benchmark, that means you're measuring the time of *doing nothing*, not the time of the work you think you're measuring.

Run this trivial function two ways, in a loop of 100 million calls — `examples/02-why-black-box.rs`:

```rust
for _ in 0..ITERS {
    total += square(black_box(7));
}
```

```rust
for _ in 0..ITERS {
    total += square(7);
}
```

Its real output:

```text
with black_box:    48.1169ms  (total=4900000000)
without black_box: 100ns  (total=4900000000)
```

Same computation, same 100 million calls, the same final `total` — and the version without `black_box` "finished" close to half a million times faster. The compiler saw that `square(7)` is always the same 49, computed it once at compile time, and turned the loop into "add this one constant 100 million times" — which can itself collapse into a single multiplication. `std::hint::black_box(x)` refuses that: it hides the value of `x` from the optimizer without actually changing anything about it, so the compiler has to assume this value *could* be different every time — and the computation genuinely runs, every time.

```senpai-visual
{"kind":"concept","labels":["input wrapped in black_box","optimizer can't see through it","treated as unknown","real computation stays","measurement reflects real work"]}
```

You've already seen `black_box`'s core trick, even though it wasn't taught then: [1.1.3](../../../phase1-fundamentals/01-foundations/03-compound-types-and-destructuring/README.md) used the same function so the compiler couldn't see a constant `5` and refuse, at compile time, to build an out-of-bounds index at all. Same function, same general power — "hide anything from the optimizer's view" — used today for a different reason: not stopping a compile-time rejection, but stopping work from being deleted at run time. `black_box` has no inherent connection to benchmarking at all; it just happens to be the place you need it most often.

Now watch the same thing inside `benches/comparison.rs` itself, this time on a real `Criterion::bench_function`:

```rust
c.bench_function("square_with_black_box", |b| b.iter(|| square(black_box(7))));
c.bench_function("square_without_black_box", |b| b.iter(|| square(7)));
```

```text
Benchmarking square_with_black_box
Benchmarking square_with_black_box: Warming up for 3.0000 s
Benchmarking square_with_black_box: Collecting 100 samples in estimated 5.0000 s (10B iterations)
Benchmarking square_with_black_box: Analyzing
square_with_black_box   time:   [490.91 ps 494.27 ps 498.09 ps]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
```

```text
Benchmarking square_without_black_box
Benchmarking square_without_black_box: Warming up for 3.0000 s
Benchmarking square_without_black_box: Collecting 100 samples in estimated 5.0000 s (21B iterations)
Benchmarking square_without_black_box: Analyzing
square_without_black_box
                        time:   [231.67 ps 234.23 ps 237.67 ps]
Found 12 outliers among 100 measurements (12.00%)
  9 (9.00%) high mild
  3 (3.00%) high severe
```

This time the gap is only about 2×, not half a million. That's real too, not a mistake. The reason is that `Bencher::iter` itself, without you asking, already wraps your closure's *output* in `black_box` (you can see this yourself in criterion's docs, linked in "Going further"), so the whole call can't be deleted for "the result was never used." What's left for you to protect is the *input* — exactly where `square_without_black_box` is still a touch faster: the compiler can't delete the call outright (criterion already stopped that), but it can still see the input is always `7` and precompute the multiplication early. (Aside: `criterion` also ships its own `black_box` — this lesson deliberately uses `std::hint::black_box` directly, the same one [1.1.3](../../../phase1-fundamentals/01-foundations/03-compound-types-and-destructuring/README.md) already showed you.)

### When benchmarking is actually worth doing

Today you saw two comparisons: one differed by close to 38×, the other by only about 2×. Before running either, you couldn't have been sure which was which — and that is exactly why benchmarking is worth doing: it replaces a guess with an exact answer. But this example has a more important point buried in it too: neither function was benchmarked because we *thought* it was slow — they were benchmarked because we were, at that exact moment, teaching this lesson.

In real code, the order has to be reversed. Donald Knuth wrote, in 1974:

> "We should forget about small efficiencies, say about 97% of the time: premature optimization is the root of all evil. Yet we should not pass up our opportunities in that critical 3%."
>
> — Donald Knuth, *Structured Programming with go to Statements*, 1974

Meaning: most of a program's code is not on the hot path at all — the database, the network, a user waiting to click something, are hundreds of times slower than any difference you'll find between two implementations of your own function. Benchmarking code that was never the bottleneck is time you could have spent on something that actually mattered — and worse, it sacrifices readability for a speed nobody will ever feel.

The right order: first, use a profiler — or even a real production latency metric — to find out which function is actually eating time. Then, only for that one, test your hypothesis with `criterion`. Benchmarking gives an exact answer to "which one is faster"; it does not answer "which one actually matters" — you answer that question somewhere else, first.

---

## Hands on

```sh
cargo run --release -p p2-07-05-benchmarking-with-criterion --example 01-naive-timing
cargo run --release -p p2-07-05-benchmarking-with-criterion --example 02-why-black-box
cargo bench -p p2-07-05-benchmarking-with-criterion
```

Then the three broken ones:

```sh
cargo build -p p2-07-05-benchmarking-with-criterion --example 03-nightly-bench-attribute --features broken
cargo build -p p2-07-05-benchmarking-with-criterion --example 04-forgot-black-box-import --features broken
cargo build -p p2-07-05-benchmarking-with-criterion --bench missing-criterion-main --features broken
```

Then try these:

1. In `01-naive-timing`, change `haystack`'s size from 1000 to 100_000 and run it a few more times. Are the three numbers in each run noisier relative to each other, or less?
2. In `02-why-black-box`, cut `ITERS` from 100_000_000 to 1_000_000. Is the "without black_box" number still this dramatic? Why do you think that is?
3. After `cargo bench`, you also have an HTML report: `target/criterion/report/index.html` (relative to the repo root). Open it and compare `contains_linear_1000`'s distribution chart against `contains_hashset_1000`'s.

---

## Errors you will meet

### `E0554` — Rust's built-in harness needs nightly

```text
error[E0554]: `#![feature]` may not be used on the stable release channel
 --> phase2-intermediate\07-project-structure-and-testing\05-benchmarking-with-criterion\examples\03-nightly-bench-attribute.rs:9:1
  |
9 | #![feature(test)]
  | ^^^^^^^^^^^^^^^^^
```

**What the compiler is actually objecting to:** Rust itself has a `#[bench]` attribute — older than `criterion` — but it's locked behind an unstable feature (`test`) only available on the nightly channel. `examples/03-nightly-bench-attribute.rs` tried to go that route; the stable compiler this repository builds with won't even let the file compile.

**The fix:** instead of `#![feature(test)]`/`extern crate test`/`#[bench]`, either write a simple `Instant`-based measurement (model it on `01-naive-timing`), or reach for `criterion` — which is exactly what this lesson is about.

**Why this is the fix:** `criterion` exists precisely because the built-in harness isn't available on stable. An ordinary crate (`criterion = "0.5"` in `[dev-dependencies]`) with its own harness (`harness = false`) sidesteps the restriction entirely, and is more statistically rigorous besides.

### `E0425` — `black_box` without an import

```text
error[E0425]: cannot find function `black_box` in this scope
 --> phase2-intermediate\07-project-structure-and-testing\05-benchmarking-with-criterion\examples\04-forgot-black-box-import.rs:9:18
  |
9 |     let hidden = black_box(7);
  |                  ^^^^^^^^^ not found in this scope
  |
help: consider importing this function
  |
8 + use std::hint::black_box;
  |
```

**What the compiler is actually objecting to:** `black_box` isn't in the prelude — like anything else in `std`, it has to be imported explicitly. This file calls it without `use std::hint::black_box;`.

**The fix:** add the line the compiler itself suggests: `use std::hint::black_box;`.

**Why this is the fix:** the compiler's own help message says exactly this, and it's correct. This is the single most common typo you'll hit writing a criterion benchmark from memory.

### `E0601` — `criterion_group!` without `criterion_main!`

```text
error[E0601]: `main` function not found in crate `missing_criterion_main`
  --> phase2-intermediate\07-project-structure-and-testing\05-benchmarking-with-criterion\benches\missing-criterion-main.rs:19:45
   |
19 | criterion_group!(benches, lookup_benchmark);
   |                                             ^ consider adding a `main` function to `phase2-intermediate\07-project-structure-and-testing\05-benchmarking-with-criterion\benches\missing-criterion-main.rs`
```

**What the compiler is actually objecting to:** `criterion_group!` only registers which functions belong to this bench — it doesn't run anything and it doesn't generate `fn main`. Generating `fn main` is `criterion_main!`'s only job. This file has the first macro and forgot the second — and a binary crate with no `main`, exactly like any other binary crate, does not compile.

**The fix:** add one line: `criterion_main!(benches);`.

**Why this is the fix:** look at `benches/comparison.rs` — that one extra macro, in that exact spot, is what generates the real `fn main`. Without it, `criterion_group!` is just a list nobody ever calls.

---

## Exercises

### Warm up

<details>
<summary>You run this code three times in a row and see three different numbers. Which one should you believe?</summary>

```rust
let start = Instant::now();
let result = do_work();
println!("{:?}", start.elapsed());
```

</details>

<details>
<summary>Answer</summary>

None of them, alone. A single sample can't separate run-to-run noise, CPU frequency scaling, or the cost of not being warmed up from the function's actual behavior. For a trustworthy number, you need many runs and statistics — exactly `criterion`'s job.

</details>

<details>
<summary>Two loops, each calling <code>square(x)</code> 100 million times — one with <code>black_box(7)</code>, the other with raw <code>7</code>. Which one "finishes" faster, and why?</summary>

The one without `black_box`. The compiler sees the input is always `7`, computes the answer once at compile time, and turns the loop into a constant — not into actually multiplying 100 million times.

</details>

<details>
<summary>Does this file compile, with no other changes?</summary>

```rust
use criterion::{criterion_group, Criterion};

fn my_benchmark(c: &mut Criterion) {
    c.bench_function("x", |b| b.iter(|| 1 + 1));
}

criterion_group!(benches, my_benchmark);
```

</details>

<details>
<summary>Answer</summary>

No. `criterion_group!` alone doesn't generate `fn main`; this file is exactly what you saw as `E0601` in "Errors you will meet". It's missing a line: `criterion_main!(benches);`.

</details>

<details>
<summary><code>contains_linear_1000 time: [240.95 ns 242.13 ns 243.40 ns]</code> — what do these three numbers mean, and which one do you quote when you tell someone "this function takes about 242 nanoseconds"?</summary>

The lower bound, the mean, and the upper bound of the confidence interval. The middle one — 242.13 nanoseconds — is what you quote; the two on either side tell you how much to trust that mean, instead of it being a single number you have to believe blindly.

</details>

### Repair

Fix all three broken examples:

1. Rewrite `examples/03-nightly-bench-attribute.rs` so it compiles on stable — either with a simple `Instant` measurement, or by turning it into a real criterion bench.
2. Fix `examples/04-forgot-black-box-import.rs` by adding `use std::hint::black_box;`.
3. Fix `benches/missing-criterion-main.rs` by adding `criterion_main!(benches);`, then run `cargo bench -p p2-07-05-benchmarking-with-criterion --bench missing-criterion-main --features broken` and confirm it actually works.

### Implement

Two functions in `src/lib.rs` — `sum_of_squares_loop` and `sum_of_squares_iter` — both must return the sum of the squares of every integer from `1` to `n` (`0` for `n == 0`; for example, `n = 3` gives `1*1 + 2*2 + 3*3 = 14`). Write one with a `for` loop, the other with an iterator chain — no loop.

```sh
cargo test -p p2-07-05-benchmarking-with-criterion
```

### Build

Open `benches/comparison.rs` and add a third benchmark function that compares `sum_of_squares_loop` and `sum_of_squares_iter` — same `n` for both, large enough that a difference, if there is one, would actually show up. Add its name to `criterion_group!`, run `cargo bench`, and look at the numbers. [2.2.5](../../02-iterators-and-closures/05-laziness-and-performance/README.md) proved by counting calls that an iterator chain does exactly as much work as a hand-written loop, no more. Today, with real time, see how well that claim actually holds up.

### Challenge (optional)

**Part one.** `criterion` can save a result and compare against it later: `cargo bench -p p2-07-05-benchmarking-with-criterion -- --save-baseline before`. Now deliberately slow something down in `lookup_benchmark` — growing `haystack` from 1,000 elements to 100,000 works — and run it again: `cargo bench -p p2-07-05-benchmarking-with-criterion -- --baseline before`. What does `criterion` report about the change?

**Part two.** (This one looks ahead.) A benchmark tells you *which* implementation is slower and *by how much* — it doesn't tell you *why* a particular implementation is slow, when it itself calls several functions. The tool that answers exactly that — a flamegraph, showing which function the time actually went into — belongs to [Phase 4 — Criterion benchmarks and flamegraphs](../../../phase4-backend-advanced/08-performance-and-profiling/01-criterion-benchmarks-and-flamegraphs/README.md).

---

## Wrapping up

This lesson closes the module. A quick look back at the path you walked: [2.7.1](../01-modules-visibility-workspaces/README.md) gave code structure and visibility; [2.7.2](../02-unit-integration-doc-tests/README.md) separated three built-in kinds of tests; [2.7.3](../03-test-doubles-in-rust/README.md) built doubles for swappable dependencies; [2.7.4](../04-property-and-snapshot-testing/README.md) showed you how to test when the input space was too large or the output too complex. Today, the last step: once you know code works and its performance genuinely matters, `criterion` answers "which one is faster" with statistics, not a guess.

| Term | What it means | Where you'll use it |
|---|---|---|
| Statistical benchmarking | Running a function many times, reporting a mean plus confidence interval, instead of a single sample | Comparing two implementations |
| `criterion` | The benchmarking crate that does this for you | `[dev-dependencies]`, `cargo bench` |
| `criterion_group!` / `criterion_main!` | Macros that register benchmark functions and generate `fn main` | Every `benches/*.rs` file |
| `harness = false` | Tells cargo not to use the built-in test harness for this target | `[[bench]]` in `Cargo.toml` |
| `black_box` | Hides a value from the compiler's optimizer | Every benchmark's inputs |
| Outlier flagging | Samples that are statistically out of line with the rest | Reading `cargo bench` output |
| Benchmark after profiling | Only once you know which function is actually slow | Deciding when to benchmark at all |

### What you now know

- Why a single `Instant::now()`/`.elapsed()` is noisy enough that you can't trust it — and its three specific reasons.
- How `criterion` solves that with warmup, many samples, and statistics (mean, confidence interval, outliers).
- How to write a real benchmark: `[[bench]]` with `harness = false` in `Cargo.toml`, `criterion_group!`/`criterion_main!` in the file.
- What `std::hint::black_box` stops from happening, and why without it a benchmark can measure the time of *doing nothing* instead of the work you think you're measuring.
- Why benchmarking before profiling — on code you don't yet know is a bottleneck — is wasted time, not caution.

### What comes back later

- **Flamegraphs — understanding *why* a specific function is slow, not just that it is** — [Phase 4 — Criterion benchmarks and flamegraphs](../../../phase4-backend-advanced/08-performance-and-profiling/01-criterion-benchmarks-and-flamegraphs/README.md)
- **Comparing against a saved baseline, for automatic regression detection** — the same Phase 4 lesson, deeper than what you saw in today's "Challenge".

### Can you explain?

- Why does running a raw `Instant::now()`/`.elapsed()` three times give three different numbers, and which one should you believe?
- What does `criterion` actually do that a `println!(elapsed)` doesn't?
- Explain `black_box` in your own words — what does it stop, and why does its name have nothing to do with "benchmark"?
- Why did the criterion version of the `square` comparison only differ by 2×, while the raw-loop version of that same comparison differed by close to half a million times?
- Why does benchmarking have to come *after* profiling, not before it?

---

## Going further

- [The `criterion.rs` book](https://bheisler.github.io/criterion.rs/book/index.html) — the official guide, including the `--save-baseline`/`--baseline` section you saw in "Challenge".
- [`std::hint::black_box`](https://doc.rust-lang.org/std/hint/fn.black_box.html) — the official docs; explains why this function makes no formal guarantee, only a best-effort request to the compiler.
- [`Bencher::iter` in criterion's docs](https://docs.rs/criterion/latest/criterion/struct.Bencher.html#method.iter) — where you can see for yourself that your closure's output is already wrapped in `black_box`.
- Donald Knuth, *Structured Programming with go to Statements*, ACM Computing Surveys 6:4 (December 1974), pp. 261–301 — [free PDF](https://pic.plover.com/knuth-GOTO.pdf).
