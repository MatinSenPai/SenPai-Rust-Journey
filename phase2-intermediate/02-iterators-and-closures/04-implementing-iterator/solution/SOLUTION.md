# Solution — 2.2.4 Implementing `Iterator` and `IntoIterator`

```rust
impl Iterator for Collatz {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let value = self.current?;
        self.current = if value == 1 {
            None
        } else if value % 2 == 0 {
            Some(value / 2)
        } else {
            Some(3 * value + 1)
        };
        Some(value)
    }
}

impl IntoIterator for EpisodeLog {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
```

Neither one needed a generic, a lifetime, or a brand-new custom trait — just the same two methods "The concept" showed you, this time on your own data.

## `Collatz::next` — one `?`, one `if`/`else`, one final `Some`

```rust
let value = self.current?;
self.current = if value == 1 {
    None
} else if value % 2 == 0 {
    Some(value / 2)
} else {
    Some(3 * value + 1)
};
Some(value)
```

The first line does two things at once: if `self.current` was already `None` (the sequence already finished), `?` returns `None` from the function right there — exactly the behavior the spec asked for ("if already finished, return None"). If it wasn't `None`, `value` becomes whatever was inside it.

Then the **next** state is computed and stored — not the return value. Catch this subtlety: the function is about to return `value` (what it just read), not whatever comes after it; it only stores what comes after for the *next* call. If `value` was exactly `1`, there is nothing left to continue with — `self.current` becomes `None`, and the following call stops right there. Otherwise, the even/odd rule is exactly what the doc comment specified.

The final line, `Some(value)`, is what turns this method from an ordinary function into a real `Iterator::next` — it always returns an `Option`, never the bare value.

## `EpisodeLog::into_iter` — handing back what you already have

```rust
self.0.into_iter()
```

That's it. `self`, taken by value, immediately calls `.into_iter()` on its inner field (`self.0`, a `Vec<String>`) — exactly what `Vec` already implements. All you had to do was say "whatever's needed, borrow it from what's already inside" — not build an iterator from scratch. `type IntoIter = std::vec::IntoIter<String>` was picked for exactly the same reason: it's exactly the type `Vec<String>::into_iter()` itself returns, with no extra conversion.

## What this lesson was really about

- **One method, everything free.** `Collatz` never wrote `.filter()`, `.count()`, or `.collect()` for itself — the `collatz_works_with_adapters_nobody_wrote_here` test proved exactly that.
- **`IntoIterator` is separate from `Iterator`.** `EpisodeLog` never implemented `Iterator` at all; it only said "here's how to become one" (handing back `std::vec::IntoIter`), and that alone was enough for `for title in log` to work.
- **An associated type is just a name.** `type Item = u64` on `Collatz` and `type Item = String` on `EpisodeLog` — neither one made `Iterator` or `IntoIterator` generic; each just said "mine is this."
