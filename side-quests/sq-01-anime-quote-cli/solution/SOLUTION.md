# Solution

```rust
pub fn find_by_anime<'a>(quotes: &'a [Quote], query: &str) -> Vec<&'a Quote> {
    let query = query.to_lowercase();
    quotes.iter().filter(|q| q.anime.to_lowercase().contains(&query)).collect()
}
```

`query.to_lowercase()` is computed once, outside the `.filter()` closure —
lowercasing it once per call instead of once per quote is both faster and
avoids allocating a fresh lowercase `String` on every iteration. The `<'a>`
lifetime says: every `&Quote` inside the returned `Vec` borrows from
`quotes`, and none of them borrow from `query` — which is exactly what's
true here (`query`'s lowercase copy is only used for comparison, never
stored).

```rust
pub fn format_quote(q: &Quote) -> String {
    format!("\"{}\" — {}, {}", q.text, q.character, q.anime)
}
```

Straightforward `format!` — note the escaped `\"` to get literal quote
marks into the output string.

```rust
pub fn pick_random(quotes: &[Quote]) -> Option<&Quote> {
    if quotes.is_empty() {
        return None;
    }
    let idx = rand::thread_rng().gen_range(0..quotes.len());
    quotes.get(idx)
}
```

`rand::thread_rng()` gets a random number generator seeded per-thread
(you've used this exact pattern already, back in
`phase0-setup/03-cargo-basics`). `.get(idx)` (rather than `quotes[idx]`)
returns `Option<&Quote>` instead of panicking on an out-of-bounds index —
belt-and-suspenders here since the empty check above already guarantees
`idx` is in range, but it means this function can never panic no matter
what changes around it later, which is a good habit for code that isn't
purely internal.
