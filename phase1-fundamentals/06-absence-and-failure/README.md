# 06 — Absence and failure

Rust has no null and no exceptions. What it has instead are two ordinary enums
— `Option` and `Result` — and one operator that makes them pleasant to use.

You have been meeting `Option` since Phase 0, every time something handed back
`Some` or `None`. This module is where it stops being something you work
around and becomes the tool you reach for.

1. [`Option` and null safety](01-option-and-null-safety/README.md)
2. [`Option` combinators](02-option-combinators/README.md)
3. [`Result` and the question mark](03-result-and-question-mark/README.md)
4. [Panic versus `Result`](04-panic-vs-result/README.md)
5. [`From` and error conversion](05-from-and-error-conversion/README.md)
