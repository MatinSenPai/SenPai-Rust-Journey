# Solution — Cargo basics

```rust
pub fn format_greeting(name: &str, times: u32) -> String {
    (0..times)
        .map(|_| format!("Hello, {name}!"))
        .collect::<Vec<_>>()
        .join("\n")
}
```

Why this shape, not a hand-rolled loop with a `String` you `push_str` into:

- `(0..times)` is a `Range`, which Rust lets you iterate directly — no
  manual index bookkeeping (`for i in 0..times`) needed here since we don't
  actually use `i`.
- `.map(|_| format!("Hello, {name}!"))` produces one greeting string per
  iteration. `format!` is doing the same job Python's f-strings do
  (`f"Hello, {name}!"`), just as a macro instead of built-in string syntax.
- `.collect::<Vec<_>>()` and `.join("\n")` is the idiomatic way to build a
  "N things, one per line" string in Rust — you'll see this `map` →
  `collect` → `join` pattern constantly once Phase 2 covers iterators
  properly. It's worth seeing it once now even before you fully understand
  why it works, so it looks familiar later instead of brand new.

An equally valid, more "beginner-explicit" version, if the iterator chain
above still looks like magic:

```rust
pub fn format_greeting(name: &str, times: u32) -> String {
    let mut result = String::new();
    for i in 0..times {
        if i > 0 {
            result.push('\n');
        }
        result.push_str(&format!("Hello, {name}!"));
    }
    result
}
```

Both compile, both pass every test. The iterator version is more idiomatic
Rust; the loop version is more obviously correct if you're still building
trust in the language. Neither is "wrong" — Phase 2 will make the iterator
version feel natural instead of clever.
