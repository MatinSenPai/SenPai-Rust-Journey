# Checkpoint

1. `#[test]`-with-a-stopwatch (wrapping some code in `std::time::Instant::now()`
   / `.elapsed()` inside a `#[test]`) is a tempting shortcut for "is this
   fast." Name two concrete reasons a single such measurement is
   misleading, and explain what `criterion` does differently for each one.
2. `fib_recursive` and `fib_iterative` return identical results for every
   input (the test suite in `src/lib.rs` checks this directly) but have
   different Big-O complexity. Why does a benchmark comparing them at,
   say, `n = 25` matter *in addition to* — not instead of — the tests that
   already prove they agree?
3. What specifically does `criterion::black_box` prevent the compiler from
   doing, and what would you actually observe (not just "it'd be wrong" —
   what would the reported numbers look like) if you removed `black_box`
   from a benchmark of a function whose input is a compile-time constant?
4. `Cargo.toml` has `harness = false` on the `[[bench]]` entry for
   `comparison`. What is `harness = true` (the default) — Rust's built-in
   `#[bench]` benchmark harness — and why does criterion need `false`
   instead of layering on top of it?
5. A flamegraph's bars are stacked by call depth and sized by time spent.
   If you profiled a service and saw one extremely wide bar near the
   *bottom* of the stack (deep in a shared library, called from many
   different code paths above it), what would that tell you about where
   to focus an optimization effort, versus a wide bar near the *top* of
   the stack in your own application code?
6. `concat_with_capacity` computes `total_len` by summing
   `w.len() + 1` for every word before allocating anything. What would go
   wrong — silently or loudly — if that upfront sum were smaller than the
   string actually written in the loop that follows?
