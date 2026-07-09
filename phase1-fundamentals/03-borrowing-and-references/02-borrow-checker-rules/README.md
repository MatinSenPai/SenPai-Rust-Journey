# 03.2 — Borrow checker rules

The previous lesson used borrows correctly. This one is about what happens
when you don't — reading real compiler errors is a skill, and it's worth
deliberately practicing on broken code before it happens to you by accident.

## The two rules, stated precisely

At any given point in the code, for any given value, you may have:
- any number of shared references (`&T`), **or**
- exactly one mutable reference (`&mut T`),

but never both kinds at once. Also: **a reference must never outlive the
value it points to** (no "dangling references" — Rust simply won't compile
code where this could happen, full stop).

## Classic broken example #1: mutable + shared at once

```rust
let mut s = String::from("hello");
let r1 = &s;
let r2 = &mut s;      // error: cannot borrow `s` as mutable because it's
                       // also borrowed as immutable
println!("{r1}");
```

## Classic broken example #2: two mutable borrows at once

```rust
let mut s = String::from("hello");
let r1 = &mut s;
let r2 = &mut s;       // error: cannot borrow `s` as mutable more than once
println!("{r1} {r2}");
```

## Classic broken example #3: dangling reference

```rust
fn dangle() -> &String {
    let s = String::from("hello");
    &s  // error: `s` is dropped at the end of this function — the
        // reference you're trying to return would point at freed memory
}
```

This one is the sharpest contrast with a language like C: this exact bug
(a "dangling pointer") is a classic source of real security vulnerabilities
and crashes in C/C++. Rust doesn't detect it at runtime and doesn't just
document it as undefined behavior — it refuses to compile it, ever.

## Your task

`src/lib.rs` contains three functions with the *fixed* versions of bugs like
the ones above already implemented (your job is to fill in the `todo!()`
bodies correctly, not to break anything). At the bottom, there's a
commented-out block reproducing example #1 for you to uncomment and observe
directly.

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`. Borrowing done — next up:
strings and slices.
