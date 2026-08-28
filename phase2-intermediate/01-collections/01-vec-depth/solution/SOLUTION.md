# Solution — 2.1.1 `Vec` in depth

```rust
pub fn retain_unwatched(entries: &mut Vec<WatchEntry>) {
    entries.retain(|entry| !entry.watched);
}

pub fn drain_first_n(entries: &mut Vec<WatchEntry>, n: usize) -> Vec<WatchEntry> {
    let n = n.min(entries.len());
    let mut removed = Vec::new();
    for item in entries.drain(0..n) {
        removed.push(item);
    }
    removed
}

pub fn dedup_adjacent_titles(entries: &mut Vec<WatchEntry>) {
    entries.dedup_by_key(|entry| entry.title.clone());
}

pub fn sorted_by_rating(entries: Vec<WatchEntry>) -> Vec<WatchEntry> {
    let mut entries = entries;
    entries.sort_by(|a, b| a.rating.total_cmp(&b.rating));
    entries
}

pub fn find_by_rating(entries: &[WatchEntry], target: f64) -> Option<usize> {
    entries
        .binary_search_by(|entry| entry.rating.total_cmp(&target))
        .ok()
}
```

None of these five functions needed complicated closures, generics or a `HashMap` — just the same five methods from "The concept", now aimed at your own data.

## `retain_unwatched` — one line, exactly what example 03 showed

```rust
entries.retain(|entry| !entry.watched);
```

`.retain()` already does "only elements that pass this condition survive"; your job was just writing the condition. Notice the closure takes `!entry.watched`, not `entry.watched` — because `.retain()` asks "does this stay?", not "does this go?".

## `drain_first_n` — `.min()` before `.drain()`, no `.collect()`

```rust
let n = n.min(entries.len());
let mut removed = Vec::new();
for item in entries.drain(0..n) {
    removed.push(item);
}
removed
```

`n.min(entries.len())` gives exactly the promised guarantee — "if `n` is past the end, everything is removed" — without it, `entries.drain(0..n)` would panic on an `n` larger than the length (an invalid range). After that, instead of `.drain(0..n).collect()` (which Phase 2.2 hasn't given you yet), a plain loop pushes each removed item — the same job, with the tools you have so far.

## `dedup_adjacent_titles` — `dedup_by_key`, not `dedup_by`

```rust
entries.dedup_by_key(|entry| entry.title.clone());
```

Because the equality criterion is a **key derived** from the element — just `title`, not the whole `WatchEntry` — `.dedup_by_key()` wants exactly that: a closure returning the key, not a manual comparison. The `.clone()` is necessary because the closure has to hand back an owned `String`, not a reference to a field that might be removed the very same moment. And because `dedup_by_key` always keeps the **first** of each neighboring run, the rating and `watched` status also come from that first entry — exactly what the test checks.

## `sorted_by_rating` — `sort_by` with `total_cmp`, not `sort_unstable_by`

```rust
let mut entries = entries;
entries.sort_by(|a, b| a.rating.total_cmp(&b.rating));
entries
```

Two choices here were deliberate. First, `f64` isn't `Ord`, so a plain `.sort()` wouldn't even compile; `total_cmp` gives a real `Ordering`, even for `NaN`. Second — and this is the one that matters more — the specification explicitly asked for equal ratings to keep their original relative order. `.sort_by()` **guarantees** that; `.sort_unstable_by()` doesn't. That single word ("unstable") is exactly what would fail `sorted_by_rating_orders_ascending_and_keeps_ties_stable`.

## `find_by_rating` — `binary_search_by` plus `.ok()`

```rust
entries
    .binary_search_by(|entry| entry.rating.total_cmp(&target))
    .ok()
```

`binary_search_by` gives back a `Result<usize, usize>`: `Ok(index)` if found, `Err(insert_at)` if not. The function's spec only wanted `Option<usize>` — the same answer, minus the insertion point when nothing matched. `.ok()` does exactly that conversion: `Ok(x)` becomes `Some(x)`, and `Err(_)` becomes `None`, without spelling it out with an explicit `match`.

## What this lesson was really about

- **Today's methods replace manual loops, they aren't new tools.** `.retain()`, `.drain()`, `.dedup_by_key()` — each does what a `for` with an `if` used to do, just with a name, and with guarantees you don't have to re-derive by hand every time.
- **`.sort_by()` and `.sort_unstable_by()` agree, except when ties matter.** Where they do — exactly like `sorted_by_rating` — the word "stable" is the entire distance between correct and wrong.
- **`binary_search_by` carries a silent contract.** Nowhere does the code say "`entries` must be sorted" — that's the caller's responsibility, always, and the signature does nothing to remind you.
