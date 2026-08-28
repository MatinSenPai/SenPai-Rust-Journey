# Solution — 2.4.2 Lifetimes in structs and methods

## `whole`

```rust
pub fn whole(source: &'a str) -> Self {
    Excerpt { text: source }
}
```

No processing at all — `source` becomes `text` directly. The signature was already given: `source: &'a str`, explicit, because this is an associated function, not a method — there's no `&self` here for rule 3 to work on.

## `text`

```rust
pub fn text(&self) -> &str {
    self.text
}
```

No `'a` anywhere in the signature. Elision rule 3 — as "The concept" explained — gives the elided output lifetime the lifetime of the `&self` borrow itself, and since `self.text` (which is `&'a str`) is always valid for at least as long as that borrow, shortening it is safe.

## `word_count`

```rust
pub fn word_count(&self) -> usize {
    self.text.split_whitespace().count()
}
```

`.split_whitespace()` is exactly what the doc comment asked for: splitting on whitespace without producing extra empty pieces when several spaces appear in a row. `.count()` gives the number of pieces.

## `total_words`

```rust
pub fn total_words(excerpts: &[Excerpt<'_>]) -> usize {
    excerpts.iter().map(Excerpt::word_count).sum()
}
```

A plain walk: map each `Excerpt` to its `word_count()`, then sum them. The input slice's lifetime is elided right there with `'_` — this function only reads the data inside each `Excerpt`, never carries a reference back out of it, so there's nothing that needs to be named explicitly.
