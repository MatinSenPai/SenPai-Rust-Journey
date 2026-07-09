# 04.2 — Slices

`&str` is actually a special case of a more general idea: a **slice** is a
borrowed view into a *contiguous run* of elements inside some larger
collection — no allocation, no copying, just a pointer + a length.

## String slices, precisely

```rust
let s = String::from("hello world");
let hello: &str = &s[0..5];   // bytes 0 through 4
let world: &str = &s[6..11];
let whole: &str = &s[..];     // the whole thing
```

`&s[0..5]` is a **range index** into `s`'s underlying bytes. This is the
byte-range indexing the previous lesson said `String` doesn't support via a
single `s[3]` — ranges are allowed (and required) specifically because a
range boundary is something *you* choose, and Rust will panic at runtime
if you pick a boundary that lands in the middle of a multi-byte character
(`s[0..1]` on a string starting with `'🦀'` panics — a real, sharp edge
worth knowing about explicitly, not silently getting corrupted data).

## Array/Vec slices — the same idea, generalized

```rust
let nums = [1, 2, 3, 4, 5];
let middle: &[i32] = &nums[1..4]; // [2, 3, 4]
```

`&[i32]` is a slice type: a borrowed view into any run of `i32`s, whether
that run lives inside a fixed-size array, a `Vec<i32>` (Phase 2), or
another slice. Functions that only need to *read* a sequence should almost
always take `&[T]` rather than `&Vec<T>` — the exact same reasoning as
preferring `&str` over `&String` (and yes, clippy's `ptr_arg` lint flags
this one too).

## Your task

Implement the functions in `src/lib.rs`.

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`. That closes out Strings &
Slices — next up: structs, enums, and pattern matching, where Rust's type
system starts doing real work for you.
