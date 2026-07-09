# 01.1.3 — Conditionals, loops, match intro

## `if` is an expression

```rust
let status = if score >= 90 { "A" } else if score >= 80 { "B" } else { "F" };
```

No parens needed around the condition, and — unlike Python's `if`/`else`
statements — Rust's `if` produces a value directly, as long as every branch
returns the same type. No ternary operator needed; this *is* Rust's
ternary.

## Three kinds of loops

- `loop { ... }` — loops forever until an explicit `break`. Unusually,
  `break value;` can hand a value back out of the loop itself:
  `let x = loop { break 5; };` sets `x` to `5`.
- `while condition { ... }` — same as Python.
- `for item in iterable { ... }` — same spirit as Python's `for`, but always
  iterates over something implementing `IntoIterator` (ranges `0..5`,
  arrays, slices, `Vec`s once you meet them in Phase 2). There's no
  C-style `for (i = 0; i < n; i++)` in Rust at all — `for i in 0..n` is how
  you get that.

## `match` — a preview

```rust
let description = match n {
    0 => "zero",
    1 | 2 => "one or two",
    3..=9 => "single digit, at least three",
    _ => "big",
};
```

`match` is Rust's pattern-matching control flow — closer to a supercharged
`switch` than Python's `match` statement, and it's **exhaustive**: the
compiler refuses to build if you haven't handled every possible case (the
`_` above is the catch-all, same role as Python's `case _:`). This lesson
only uses it on plain integers; Phase 1's structs/enums module is where
`match` really earns its keep.

## Your task

Implement the functions in `src/lib.rs`.

## Checkpoint

`cargo test -p p1-01-03-conditionals-loops-match-intro`, then
`CHECKPOINT.md`, then `solution/SOLUTION.md`. This closes out module 1 —
next up: ownership.
