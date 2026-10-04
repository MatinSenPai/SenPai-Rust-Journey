# Module 9 — Performance

Measure first. This module covers profiling and flamegraphs, the allocation and hot-path habits that matter in Rust, the async-specific traps that quietly cap throughput, and load testing to find the limit your service really has.

1. [01 — Profiling, benchmarks and flamegraphs](01-criterion-benchmarks-and-flamegraphs/README.md)
   — Criterion for micro-benchmarks and flamegraphs for finding where time goes.
2. [02 — Allocation awareness and hot-path Rust](02-allocation-awareness-and-hot-paths/README.md)
   — Where allocations hide and how to take them out of the loop that matters.
3. [03 — Async performance traps](03-async-performance-traps/README.md)
   — Blocking the runtime, unbounded spawn, and other quiet throughput caps.
4. [04 — Load testing and finding your real limits](04-load-testing/README.md)
   — Generating realistic load and reading what the results actually say.
