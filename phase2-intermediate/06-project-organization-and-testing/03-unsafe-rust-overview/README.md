# 06.3 — `unsafe` Rust overview

A careful introduction, not a scary one. Most Rust code — everything you've
written in this repo so far — never needs `unsafe` at all. This lesson
demystifies what it actually unlocks, so it stops being a mysterious
red flag and becomes a specific, bounded tool you understand.

## `unsafe` doesn't turn off the borrow checker

This is the single most common misconception. Inside an `unsafe` block,
ownership, borrowing, and lifetimes are all **still fully enforced** by the
compiler. `unsafe` unlocks exactly five additional operations, and nothing
else:

1. Dereferencing a **raw pointer** (`*const T` / `*mut T`).
2. Calling an `unsafe fn` (including C functions via FFI).
3. Accessing or modifying a mutable `static` variable.
4. Implementing an `unsafe trait`.
5. Accessing a field of a `union`.

That's the entire list. Everything else about Rust's rules still applies
inside `unsafe { ... }` — you haven't left safe Rust's world, you've just
been handed a small set of extra keys, along with full personal
responsibility for using them correctly. The compiler stops checking the
specific invariant each of these five operations relies on; it does not
stop checking anything else.

## Raw pointers: `*const T` and `*mut T`

Unlike `&T`/`&mut T`, raw pointers:
- Can be null, dangling, or unaligned — the compiler doesn't verify any of
  that when you *create* one (only when you *dereference* it, and only in
  the sense that dereferencing invalid memory is undefined behavior you're
  now responsible for avoiding).
- Aren't subject to the "aliasing XOR mutability" rule — you can have
  multiple `*mut T` pointers to the same data simultaneously. This is
  exactly why raw pointers exist: some data structures are genuinely safe
  to build, but impossible to express through the borrow checker's rules as
  written.
- Can only be *dereferenced* inside `unsafe` — creating one is always safe;
  reading through one is where the risk (and the `unsafe` keyword) lives.

## The exercise: `split_at_mut`

The classic, canonical example of "safe Rust genuinely cannot express
this, even though it's obviously safe in practice": splitting one mutable
slice into two independent mutable halves.

```rust
fn split_at_mut_naive<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    (&mut slice[..mid], &mut slice[mid..])  // does not compile!
}
```

This fails to compile — two `&mut` borrows of `slice` at once, and the
borrow checker has no way to see that `[..mid]` and `[mid..]` provably
never overlap; it only sees "two mutable borrows of the same variable."
You, the programmer, know they're disjoint. Raw pointers are how you tell
the compiler "trust me here" — narrowly, for this one operation — without
giving up safety everywhere else.

## Your task

Implement `split_at_mut_demo` in `src/lib.rs` using raw pointers — a
simplified version of `std::slice::split_at_mut` (a real function in the
standard library, implemented exactly this way).

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`. That's Project Organization &
Testing done — on to Concurrency & Async, the last module of Phase 2.
