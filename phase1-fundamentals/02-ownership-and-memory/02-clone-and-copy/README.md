# 02.2 — Clone and Copy

## Why `i32` didn't move in the last lesson

```rust
let x = 5;
let y = x;
println!("{x}"); // fine! no error
```

`i32` implements the **`Copy`** trait. Types marked `Copy` are duplicated
bit-for-bit on assignment instead of moved — cheap, fixed-size,
stack-only data (all the integer/float types, `bool`, `char`, and tuples/
arrays of `Copy` types) implement it. After `let y = x`, both `x` and `y`
are fully independent, valid `i32`s. This is the closest Rust gets to
Python's "everything is just a reference to the same cheap-to-copy
immutable int" behavior — except explicit and opt-in per type, not
universal.

`String`, `Vec<T>`, and most types that own heap data are **not** `Copy` —
duplicating them silently on every assignment would be a hidden, unbounded
performance cost (copying a 10MB `String` just because you wrote `let s2 =
s1`). Rust makes that cost visible: it's a move by default, and if you
genuinely want an independent duplicate, you ask for it explicitly.

## `Clone` — the explicit, possibly-expensive duplicate

```rust
let s1 = String::from("hello");
let s2 = s1.clone();       // s1 is still valid — this is a real, independent copy
println!("{s1} {s2}");     // both usable
```

`.clone()` is how you opt into "yes, I actually want a second, independent
copy of this heap data, I understand it's not free." Every `Copy` type is
also `Clone` (clone-ing an `i32` is just... copying it), but not every
`Clone` type is `Copy` — `Clone` is the general-purpose "duplicate this"
mechanism, `Copy` is the special case where duplicating is so cheap it can
happen implicitly.

## The rule of thumb

- Need an independent, owned duplicate of heap data? `.clone()` — and know
  that it has a real runtime cost proportional to the data size.
- Working with small, fixed-size data (numbers, booleans, chars, simple
  tuples)? It's probably already `Copy` — you don't need to think about it.
- About to reach for `.clone()` just to work around a move error you don't
  fully understand? Pause — often what you actually want is a **borrow**
  (`&`), covered next module, which needs zero copying at all.

## Your task

Implement the functions in `src/lib.rs`.

## Next

`solution/SOLUTION.md` — but only after a real attempt.
