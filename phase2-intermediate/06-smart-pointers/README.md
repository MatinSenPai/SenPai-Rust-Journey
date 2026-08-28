# 06 — Smart pointers and shared state

Ownership said one owner, always. This module is every sanctioned way
around that rule: `Box` for a value too large or too recursive to live on
the stack, `Rc`/`Arc` for state that is genuinely shared, and `RefCell` for
mutating something you were only handed a shared reference to.

1. [`Box` and heap allocation](01-box-and-heap-allocation/README.md)
2. [Recursive types and boxed trait objects](02-recursive-types-and-trait-objects/README.md)
3. [`Rc` and `Arc`](03-rc-and-arc/README.md)
4. [`Weak` and reference cycles](04-weak-and-reference-cycles/README.md)
5. [`RefCell`, `Cell`, and the run-time panic trade](05-refcell-and-interior-mutability/README.md)
