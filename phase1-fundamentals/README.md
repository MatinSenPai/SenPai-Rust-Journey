# Phase 1 — Core Fundamentals

This is the phase that actually makes Rust feel different from Python:
ownership, borrowing, and a type system that won't let you paper over a
missing value. Go slowly here — everything later (traits, async, the
capstone) leans on getting these mental models right.

1. **Variables, types & control flow**
   - [01 — Variables, mutability, shadowing](01-variables-types-control-flow/01-variables-mutability-shadowing/README.md)
   - [02 — Scalar and compound types](01-variables-types-control-flow/02-scalar-and-compound-types/README.md)
   - [03 — Conditionals, loops, match intro](01-variables-types-control-flow/03-conditionals-loops-match-intro/README.md)
2. **Ownership & memory**
   - [01 — Move semantics](02-ownership-and-memory/01-move-semantics/README.md)
   - [02 — Clone and Copy](02-ownership-and-memory/02-clone-and-copy/README.md)
   - [03 — Drop and RAII](02-ownership-and-memory/03-drop-and-raii/README.md)
3. **Borrowing & references**
   - [01 — Shared and mutable refs](03-borrowing-and-references/01-shared-and-mutable-refs/README.md)
   - [02 — Borrow checker rules](03-borrowing-and-references/02-borrow-checker-rules/README.md)
4. **Strings & slices**
   - [01 — `String` vs `&str`](04-strings-and-slices/01-string-vs-str/README.md)
   - [02 — Slices](04-strings-and-slices/02-slices/README.md)
5. **Structs, enums & pattern matching**
   - [01 — Structs and methods](05-structs-enums-pattern-matching/01-structs-and-methods/README.md)
   - [02 — Enums and match](05-structs-enums-pattern-matching/02-enums-and-match/README.md)
   - [03 — `if let` / `while let`](05-structs-enums-pattern-matching/03-if-let-while-let/README.md)
6. **`Option`, `Result` & error basics**
   - [01 — `Option` and null safety](06-option-result-error-basics/01-option-and-null-safety/README.md)
   - [02 — `Result` and the `?` operator](06-option-result-error-basics/02-result-and-question-mark/README.md)
   - [03 — `panic!` vs `Result`](06-option-result-error-basics/03-panic-vs-result/README.md)

**Motivational recall questions:** [Side-quest 1 — Anime Quote CLI](../side-quests/sq-01-anime-quote-cli/README.md)
— a small real program using everything above.

When Phase 1 is fully checked off in [`PROGRESS.md`](../PROGRESS.md), move on
to [Phase 2](../phase2-intermediate/README.md).
