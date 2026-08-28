# 04.1 — Custom error types

## Why `String` errors stop scaling

Back in Phase 1 (`06-option-result-error-basics`), every fallible function
returned `Result<T, String>`. That was the right call for a first exposure
to `?` — but it has a real cost, and this lesson is about paying it off.

Say a caller receives a `String` error. What can they *do* with it besides
print it or log it? Nothing, structurally. If your code needs to react
differently depending on *what kind* of failure happened — retry on a
timeout, ask the user for a different value on a validation error, crash
loudly on a bug — a `String` gives the caller nothing to `match` on. They'd
have to inspect the text itself (`if msg.contains("missing field")`), which
is exactly the kind of fragile, typo-prone string-sniffing Python code does
with `except Exception as e: if "not found" in str(e):`. Rust's whole pitch
is that the compiler catches this class of mistake for you — but only if
your error type carries structured information instead of prose.

The fix: define your own error **enum**, one variant per distinct failure
kind. Callers can then `match` on it exhaustively, and the compiler will
yell at them if a new variant shows up and they forgot to handle it.

## What makes something "a proper Rust error"

Nothing magic — any type can go in a `Result`'s `E` slot. But by
*convention*, a well-behaved Rust error type implements two traits:

- **`std::fmt::Display`** — produces the human-readable message shown to an
  end user via `{}` (e.g. in a `println!` or when an `anyhow`-wrapped error
  gets printed at the top of `main`). You write this by hand: a `match` over
  your variants, one arm per message.
- **`std::error::Error`** — a marker trait (from Rust's *standard* library,
  not a crate) that says "this type is an error, and can compose with other
  errors." It requires `Display` and `Debug` already be implemented, and it
  has one method, `source()`, with a default implementation that returns
  `None` — you don't need to override it for this exercise (you would if
  your error *wraps* another error and you want callers to be able to walk
  the chain). Implementing `Error` is what lets your type be boxed as
  `Box<dyn std::error::Error>`, used with `?` inside functions that return
  that trait object, and — next lesson — wrapped by `anyhow::Error`.

Note that `Display` and `Debug` are two *separate, non-overlapping* traits,
even though they look similar:

- `Debug` (`{:?}`) is for **developers** — a mechanical dump of the value's
  structure, and you almost always get it for free with
  `#[derive(Debug)]`.
- `Display` (`{}`) is for **end users** — a message you write by hand,
  because only you know what's actually useful to show someone.

`std::error::Error` requires both, so even once you hand-write `Display`,
you still need `#[derive(Debug)]` on top — the derive doesn't give you
`Display`, and your manual `impl Display` doesn't give you `Debug`.

## `From` — what makes `?` convert error types automatically

```rust
impl From<std::num::ParseIntError> for ConfigError {
    fn from(source: std::num::ParseIntError) -> Self {
        ConfigError::MissingField(format!("bad number: {source}"))
    }
}
```

When you write `some_result?` inside a function returning
`Result<T, ConfigError>`, and `some_result` is actually a
`Result<_, ParseIntError>`, the compiler doesn't just refuse to compile —
it looks for `impl From<ParseIntError> for ConfigError` and, if one exists,
calls it automatically to convert the error before returning. That's the
whole mechanism behind `?` working "across" error types; it's not magic,
it's one trait lookup. You saw the manual version of this in Phase 1
(`.map_err(|e| e.to_string())`) — `From` is what lets you skip the
`.map_err` and just write `?`.

This lesson's `ConfigError::InvalidNumber` variant needs both a field
*name* and the underlying parse error, so a blanket `From` impl can't
supply the field name — you'll write `.map_err(...)` explicitly for that
one. But `MissingField`'s conversion is simple enough to go through `From`
and get triggered implicitly by `?`. Seeing both side by side in the same
function is the point: `?` isn't always available, only when a matching
`From` exists.

## Your task

Implement `ConfigError` (`Display` + `std::error::Error`), the `From` impl,
and `parse_config` in `src/lib.rs`. The format being parsed is tiny —
`key=value` pairs, one per line, like:

```
name=OnePieceTracker
max_retries=3
```

## Next

`solution/SOLUTION.md` — but only after a real attempt.
