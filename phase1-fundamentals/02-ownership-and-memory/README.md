# 02 — Ownership & Memory

The single biggest conceptual shift coming from Python. No garbage
collector, no manual `free()` — instead, the compiler tracks, at compile
time, exactly who is responsible for cleaning up every value, and refuses to
build code where that isn't unambiguous. Go slowly; this module is the
foundation everything else (borrowing, lifetimes, smart pointers, even async)
builds on.

1. [Move semantics](01-move-semantics/README.md)
2. [Clone and Copy](02-clone-and-copy/README.md)
3. [Drop and RAII](03-drop-and-raii/README.md)
