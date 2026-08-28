# Phase 1 — Core Fundamentals

This is the phase that makes Rust feel different from Python: ownership,
borrowing, and a type system that will not let you paper over a missing value.
Go slowly here. Everything later — traits, async, the capstone — leans on
getting these mental models right.

Thirty-one lessons in seven modules, in the order they are meant to be read.
Each one names its prerequisite, and each one says which later lesson finishes
the parts it only starts.

## 1. [Foundations](01-foundations/README.md)

The syntax layer, and the small decisions Rust makes you take for yourself.

1. [Variables, mutability and shadowing](01-foundations/01-variables-mutability-shadowing/README.md)
2. [Scalar types and overflow](01-foundations/02-scalar-types-and-overflow/README.md)
3. [Compound types and destructuring](01-foundations/03-compound-types-and-destructuring/README.md)
4. [Functions and expressions](01-foundations/04-functions-and-expressions/README.md)
5. [Control flow](01-foundations/05-control-flow/README.md)
6. [`Vec` and `String` basics](01-foundations/06-vec-and-string-basics/README.md)

## 2. [Ownership and memory](02-ownership-and-memory/README.md)

The heart of the language, and the reason for everything above it.

1. [Stack and heap](02-ownership-and-memory/01-stack-and-heap/README.md)
2. [Move semantics](02-ownership-and-memory/02-move-semantics/README.md)
3. [`Clone` and `Copy`](02-ownership-and-memory/03-clone-and-copy/README.md)
4. [Ownership across functions](02-ownership-and-memory/04-ownership-across-functions/README.md)
5. [`Drop` and RAII](02-ownership-and-memory/05-drop-and-raii/README.md)

## 3. [Borrowing and references](03-borrowing-and-references/README.md)

How to use a value without taking it.

1. [Shared and mutable references](03-borrowing-and-references/01-shared-and-mutable-refs/README.md)
2. [The rules of the borrow checker](03-borrowing-and-references/02-borrow-checker-rules/README.md)
3. [Borrow scopes and NLL](03-borrowing-and-references/03-borrow-scopes-and-nll/README.md)
4. [Slices](03-borrowing-and-references/04-slices/README.md)

## 4. [Text and strings](04-text-and-strings/README.md)

Where Rust is stricter than you expect, and where that strictness earns its
keep for anyone working in Persian.

1. [`String` versus `&str`](04-text-and-strings/01-string-vs-str/README.md)
2. [UTF-8: bytes, chars, graphemes](04-text-and-strings/02-utf8-bytes-chars-graphemes/README.md)
3. [Building and transforming strings](04-text-and-strings/03-building-and-transforming-strings/README.md)
4. [Slicing text safely](04-text-and-strings/04-slicing-text-safely/README.md)

## 5. [Your own types](05-your-own-types/README.md)

Making the compiler understand your problem, not just your data.

1. [Structs and methods](05-your-own-types/01-structs-and-methods/README.md)
2. [Tuple structs and the newtype pattern](05-your-own-types/02-tuple-structs-and-newtype/README.md)
3. [Enums as data](05-your-own-types/03-enums-as-data/README.md)
4. [`match` in depth](05-your-own-types/04-match-in-depth/README.md)
5. [`if let`, `while let`, `let else`](05-your-own-types/05-if-let-while-let-let-else/README.md)

## 6. [Absence and failure](06-absence-and-failure/README.md)

No null, no exceptions. Two enums and a question mark instead.

1. [`Option` and null safety](06-absence-and-failure/01-option-and-null-safety/README.md)
2. [`Option` combinators](06-absence-and-failure/02-option-combinators/README.md)
3. [`Result` and the question mark](06-absence-and-failure/03-result-and-question-mark/README.md)
4. [Panic versus `Result`](06-absence-and-failure/04-panic-vs-result/README.md)
5. [`From` and error conversion](06-absence-and-failure/05-from-and-error-conversion/README.md)

## 7. [Putting it together](07-putting-it-together/README.md)

1. [Guided mini-project](07-putting-it-together/01-guided-mini-project/README.md)
2. [Phase review](07-putting-it-together/02-phase-review/README.md)

---

**A real program using all of it:** [Side-quest 1 — Anime Quote CLI](../side-quests/sq-01-anime-quote-cli/README.md)

When Phase 1 is checked off in [`PROGRESS.md`](../PROGRESS.md), move on to
[Phase 2](../phase2-intermediate/README.md).
