# 06.1 — `Option` and null safety

## There is no `null` in Rust

Python (and most languages) let any variable be `None`/`null` unexpectedly
— a function typed to return a `User` can still hand you `None`, and
nothing forces you to check before using it. This is such a common source
of bugs that its inventor, Tony Hoare, has called the null reference his
"billion dollar mistake."

Rust's answer: **there is no null**. A value of type `User` is *always* a
real `User`. If a value might be absent, that's part of the type:

```rust
enum Option<T> {
    Some(T),
    None,
}
```

`Option<T>` is just a regular enum from the previous module (in fact, it's
defined in the standard library using exactly the tools you already know).
`find_user(id) -> Option<User>` tells you, in the type signature itself,
"this might not find anything" — and because `Option<User>` isn't a `User`,
the compiler won't let you call `.name()` on it directly. You're **forced**
to handle the `None` case somewhere, at compile time, before you can get to
the `User` inside.

## The main ways to work with `Option<T>`

```rust
let maybe_age: Option<u32> = Some(25);

maybe_age.is_some();                    // bool: true
maybe_age.is_none();                    // bool: false
maybe_age.unwrap();                     // 25, or PANICS if None — see next lesson
maybe_age.unwrap_or(0);                 // 25, or 0 if None
maybe_age.unwrap_or_default();          // 25, or 0 (u32's default) if None
maybe_age.map(|age| age + 1);           // Some(26) — transforms the inside, if present
if let Some(age) = maybe_age { /* ... */ }   // previous lesson
```

`.map()` is worth pausing on: it lets you transform the value *inside* an
`Option` without manually unwrapping and rewrapping it — `None` stays
`None`, `Some(x)` becomes `Some(f(x))`. This is the same shape you'll see
constantly on `Result` (next lesson) and on iterators (Phase 2).

## Your task

Implement the functions in `src/lib.rs` — a tiny "find a user by id"
lookup, entirely `Option`-based, no panics anywhere.

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`.
