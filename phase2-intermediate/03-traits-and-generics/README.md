# 03 — Traits and generics

Traits are how Rust does polymorphism without inheritance, and generics are
how it does it without paying for it at run time. This is the module where
"generic" stops meaning vague and starts meaning written once, working for
every type that qualifies — checked entirely at compile time.

1. [Defining and implementing traits](01-defining-and-implementing-traits/README.md)
2. [Generic functions and structs, bounds, `where`](02-generic-functions-and-structs/README.md)
3. [`From`, `Into`, `TryFrom`, `TryInto`](03-from-into-tryfrom/README.md)
4. [The standard derives, implemented by hand](04-standard-derives-by-hand/README.md)
5. [Associated types versus generic parameters](05-associated-types/README.md)
6. [Supertraits, blanket impls, the orphan rule and the newtype escape hatch](06-supertraits-blanket-impls-orphan-rule/README.md)
7. [Static versus dynamic dispatch, and object safety](07-static-vs-dynamic-dispatch/README.md)
