# 01.1.1 — Variables, mutability, shadowing

## Immutable by default

```rust
let x = 5;
x = 6; // compile error: cannot assign twice to immutable variable `x`
```

In Python, every variable is reassignable by default. In Rust, `let x = 5`
makes `x` **immutable** — the compiler will refuse to build a program that
tries to reassign it. This isn't a limitation you'll fight forever; it's a
default that makes reading code easier (`x` written once means `x` really is
that value everywhere below), and you opt out explicitly the moment you need
to:

```rust
let mut x = 5;
x = 6; // fine — `mut` opted in
```

## Shadowing — not the same as mutation

```rust
let x = 5;
let x = x * 2;      // a brand new `x`, allowed to have a different type even
let x = x.to_string();
```

This looks like mutation but isn't: each `let x = ...` creates a **new**
variable that happens to reuse the name `x`, hiding ("shadowing") the
previous one. Consequences:
- You can change the *type* between shadows (`i32` → `String` above) —
  `mut` never lets you do that; a `mut` variable's type is fixed forever.
- The old value isn't dropped or mutated in place, it's just no longer
  reachable by that name (still cleaned up per Phase 1's next module on
  ownership).

Use `mut` when you're genuinely updating the same value over time (a
counter, an accumulator). Use shadowing when you're transforming a value
through a pipeline of steps and don't want to invent a new name for each
stage (`raw_input` → `trimmed` → `parsed`, or just reuse `input` three
times via shadowing).

## `const`

```rust
const MAX_RETRIES: u32 = 3;
```

Always immutable (no `mut const`), must have an explicit type, and must be
set to a value the compiler can compute at compile time (no calling a
function whose result isn't known until runtime). Closest Python analogy:
a module-level `MAX_RETRIES = 3` you promise never to reassign, except Rust
actually enforces the promise.

## Your task

Implement both functions in `src/lib.rs`.

## Next

`cargo test -p p1-01-01-variables-mutability-shadowing`, then
`solution/SOLUTION.md` — but only after a real attempt.
