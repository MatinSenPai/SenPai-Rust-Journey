# 03 — Borrowing & References

Ownership and moves (previous module) explain *why* Rust needs this module:
constantly losing access to a value every time you pass it to a function
would make the language unworkable. Borrowing is the answer — temporary,
compiler-checked access to a value you don't own, with zero runtime cost.

1. [Shared and mutable refs](01-shared-and-mutable-refs/README.md)
2. [Borrow checker rules](02-borrow-checker-rules/README.md)
