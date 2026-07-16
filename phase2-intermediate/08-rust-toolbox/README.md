# 08 — Rust toolbox

By this point you can write real Rust: collections, iterators, traits,
error types, smart pointers, async. This module is the toolbox drawer —
four smaller topics that don't each deserve a whole module but that you'll
reach for constantly in Phase 3+ and see on every page of other people's
crates: the pattern-matching features the early lessons skipped, your
first `macro_rules!` macros (and honest guidance on when *not* to write
one), Cargo features for compile-time configuration, and `TryFrom` for
conversions that can fail. Coming from Python: this is roughly the jump
from "I can write Django views" to "I understand `*args`, decorators,
`pip install package[extra]`, and `int(x)` raising `ValueError`" — the
idioms that make you fluent rather than merely functional. Each lesson is
deliberately short and self-contained; 1 → 4 is the natural order, but
nothing later depends on anything earlier.

1. [Pattern matching in depth](01-pattern-matching-depth/README.md)
2. [`macro_rules!` basics](02-macro-rules-basics/README.md)
3. [Cargo features](03-cargo-features/README.md)
4. [`TryFrom` and fallible conversions](04-tryfrom-fallible-conversions/README.md)
