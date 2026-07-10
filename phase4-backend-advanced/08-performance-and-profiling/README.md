# 08 — Performance & Profiling

Tests answer "is this correct." They don't answer "is this fast," and
`#[test]`-with-a-stopwatch is actively misleading — this module covers the
tools that answer that question properly: statistically rigorous
benchmarking with `criterion`, and where to look next (flamegraphs) once a
benchmark tells you *that* something is slow but not *why*.

1. [Criterion benchmarks & flamegraphs](01-criterion-benchmarks-and-flamegraphs/README.md)
