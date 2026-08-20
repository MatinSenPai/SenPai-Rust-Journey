# 03.1 — Shared and mutable references

## Borrowing instead of moving

```rust
fn calculate_length(s: &String) -> usize {
    s.len()
} // s (the reference) goes out of scope here — but it doesn't own the
  // String, so nothing is dropped. The String itself is unaffected.

let s1 = String::from("hello");
let len = calculate_length(&s1);
println!("{s1} is {len} long"); // s1 is still valid!
```

`&s1` creates a **reference** to `s1` — you're lending temporary access,
not handing over ownership. `calculate_length` receives `&String`, reads
through it, and when the reference goes out of scope, nothing is dropped
(there's nothing for it to own). This is how you pass values around without
constantly losing access to them via moves.

## Shared references: `&T` — read-only, but you can have many at once

```rust
let s = String::from("hello");
let r1 = &s;
let r2 = &s;
println!("{r1} and {r2} and {s}"); // all three valid simultaneously — fine
```

Any number of shared (`&T`) references can coexist, because none of them
can modify the data — many readers, no writers, is always safe.

## Mutable references: `&mut T` — exclusive, and there can be only one

```rust
fn add_exclamation(s: &mut String) {
    s.push_str("!");
}

let mut s = String::from("hello");
add_exclamation(&mut s);
println!("{s}"); // "hello!"
```

To modify a value through a reference, you need `&mut T`, and the variable
being borrowed must itself be declared `mut`. The critical rule, enforced
entirely at compile time with zero runtime cost:

> **While a mutable reference to a value exists, no other reference to
> that value — shared or mutable — may exist at the same time.**

This is often phrased "aliasing XOR mutability": either many read-only
aliases, or exactly one mutable one, never both. It's the same underlying
principle that gives Rust its headline guarantee (no data races), just
enforced at compile time on a single thread here — Phase 2's concurrency
module is where you'll see this exact rule prevent actual multi-threaded
bugs.

## Your task

Implement the functions in `src/lib.rs`. None of them need `.clone()` —
that's the point of this lesson.

## Next

`solution/SOLUTION.md` — but only after a real attempt.
