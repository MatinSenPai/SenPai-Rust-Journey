# 03 — Generics & traits

Python's duck typing means "does this object support the operation I want"
gets answered *while the program is running*, on whatever object happens to
show up. Rust answers the same question at compile time instead: generics
let you write one function or struct that works across many types, while
the compiler checks and even generates specialized code for each type you
actually use; traits let you define the contract ("must support this
operation") that makes that possible. This module builds the toolkit —
generic functions/structs, your own traits with default methods, and the
static-vs-dynamic dispatch choice — that the rest of the course (error
types, smart pointers, and eventually `axum` handlers in Phase 3) is built
on top of.

1. [Generic functions and structs](01-generic-functions-and-structs/README.md)
2. [Defining and implementing traits](02-defining-and-implementing-traits/README.md)
3. [Trait objects vs. static dispatch](03-trait-objects-vs-static-dispatch/README.md)
