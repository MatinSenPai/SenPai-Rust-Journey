# 02 — Iterators & closures

This is the module where Rust starts feeling less like "C with extra
compiler complaints" and more like a language with its own idioms.
Closures — anonymous functions that capture their environment — are the
building block; the `Iterator` trait's adapter methods (`.map()`,
`.filter()`, `.fold()`, and friends) are where they pay off, replacing the
Python list-comprehension habit with lazy, composable chains that the
compiler can optimize into a single tight loop. Expect this module to
change how you write nearly every function from here on.

1. [Closures and `Fn` traits](01-closures-and-fn-traits/README.md)
2. [Iterator adapters](02-iterator-adapters/README.md)
