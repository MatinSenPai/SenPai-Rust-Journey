# Solution

```rust
pub fn longest_word(text: &str) -> &str {
    text.split_whitespace()
        .max_by_key(|word| word.len())
        .unwrap_or("")
}
```

The interesting part isn't the iterator chain (Phase 2 formalizes these) —
it's the signature: `fn longest_word(text: &str) -> &str`. There's a hidden
relationship the compiler is enforcing here even though no lifetime is
written explicitly: the returned `&str` is only ever valid for as long as
`text` is valid, because it's a slice borrowed directly out of `text`,
never a new allocation. This works without you writing `<'a>` anywhere
because of **lifetime elision** — a function with exactly one reference
parameter and a reference return type gets its lifetime inferred
automatically ("the output borrows from the one input"). Phase 2's
lifetimes lesson makes this rule explicit; for now, just notice that the
compiler is quietly tracking it for you already.

`greet(&owned)` compiling where `greet` expects `&str` and `owned` is a
`String` is **deref coercion**: Rust automatically converts `&String` to
`&str` at a call site like this (via the `Deref` trait `String` implements
for `str`), because "I have a reference to an owned buffer, but you only
need a borrowed view into it" is always a safe, free conversion.
