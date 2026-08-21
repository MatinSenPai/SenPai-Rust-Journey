# 02 — Ownership and memory

The heart of Rust. Every rule in this module is an answer to one question:
when does memory taken at run time get given back, and who decides?

Take this module slowly. If ownership lands, borrowing and lifetimes follow
behind it; if it does not, the rest of the language reads as arbitrary
strictness.

1. [Stack and heap](01-stack-and-heap/README.md)
2. [Move semantics](02-move-semantics/README.md)
3. [`Clone` and `Copy`](03-clone-and-copy/README.md)
4. [Ownership across functions](04-ownership-across-functions/README.md)
5. [`Drop` and RAII](05-drop-and-raii/README.md)
