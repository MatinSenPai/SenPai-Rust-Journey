# 04.1 — `String` vs `&str`

You've already used both throughout Phase 0-1 without a full explanation —
this lesson gives you one.

## Two types, one job

- **`String`** — an **owned**, growable, heap-allocated buffer of UTF-8
  text. You own it; when it goes out of scope, it's dropped (Phase 1's
  ownership modules apply to it directly, since it's not `Copy`).
- **`&str`** ("string slice") — a **borrowed** view into UTF-8 text living
  somewhere else: inside a `String`, or a string literal baked directly into
  the compiled binary (`"hello"` is a `&'static str` — a reference valid for
  the whole program, since it's part of the binary itself, not allocated at
  runtime).

Every `String` can be borrowed as a `&str` (that's what `&my_string` or
`my_string.as_str()` gives you). A `&str` cannot become a `String` without
an actual allocation and copy (`.to_string()` or `.to_owned()`).

## Why two types, when Python just has `str`?

Python's `str` is always, implicitly, heap-allocated and reference-counted
— you never think about where the memory lives because the runtime handles
it uniformly. Rust makes that choice *visible* and *explicit*, because it
matters for performance and API design:

```rust
fn greet(name: &str) -> String {   // borrows in, owns out
    format!("Hello, {name}!")
}
```

`greet` takes `&str` because it only needs to *read* the name — accepting
`&str` means it works whether the caller has a `String`, a string literal,
or a slice of a larger string, with zero copying. It returns `String`
because the greeting is brand new data that has to live somewhere, and the
function can't return a reference to something it created locally (that's
the "dangling reference" trap from the borrow-checker lesson) — it must
hand over ownership.

**Rule of thumb**: function parameters that only read text should almost
always be `&str`, not `&String` (you tightened exactly this in Phase 0's
tooling lesson via the `ptr_arg` clippy warning). Return `String` when
you're producing new, owned text.

## UTF-8, and why there's no `s[3]`

Rust `String`s are UTF-8 encoded, and characters can take 1-4 bytes (recall
the `char`/`len_utf8` lesson). Because of this, Rust deliberately does
**not** support indexing a `String` by character position (`s[3]` doesn't
compile) — a byte index into multi-byte UTF-8 text can land in the *middle*
of a character, which would be nonsense. Instead you index by byte range
into a slice (next lesson) or iterate `.chars()` explicitly.

## Your task

Implement the functions in `src/lib.rs`.

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`.
