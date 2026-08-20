# 01.1.2 — Scalar and compound types

Rust is **statically typed**: every value's type is known at compile time
(usually inferred, sometimes written explicitly), and it never changes.
Python's `x = 5; x = "hi"` is simply not something the same variable can do
in Rust (shadowing lets you *reuse a name* with a new type — the underlying
variable is still fixed-type, see the previous lesson).

## Scalars — one single value

| Type | What it is | Notes |
|---|---|---|
| `i8`..`i128`, `isize` | signed integers | `i32` is the default if unspecified |
| `u8`..`u128`, `usize` | unsigned integers | `usize` is what indexes/lengths use — sized to match the pointer width |
| `f32`, `f64` | floats | `f64` is the default |
| `bool` | `true`/`false` | same as Python |
| `char` | one Unicode scalar value | **4 bytes**, always — not the same thing as a byte, and not the same thing as "a letter" (`'🦀'` is one `char`) |

Rust's integers have fixed sizes and will **panic on overflow** in debug
builds (`u8` max is 255; `255u8 + 1` crashes rather than silently wrapping
or growing, unlike Python's arbitrary-precision `int`). This is a real,
deliberate difference from Python's "integers just work at any size" model
— sizing your integer type correctly is a real design decision in Rust.

## Compounds — bundles of values

- **Tuples** — fixed-size, can mix types: `let point: (f64, f64) = (1.0, 2.0);`
  access with `.0`, `.1`.
- **Arrays** — fixed-size, all-one-type, size is part of the type:
  `let nums: [i32; 3] = [1, 2, 3];` — `[i32; 3]` and `[i32; 4]` are
  genuinely different types. For a growable, one-type collection you want
  `Vec<T>` (Phase 2).

## Your task

Implement the functions in `src/lib.rs` — each exercises one of the types
above.

## Next

`cargo test -p p1-01-02-scalar-and-compound-types`, then the recall questions,
then `solution/SOLUTION.md`.
