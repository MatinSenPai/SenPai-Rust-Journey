# 2.1.1 — `Vec` in depth: capacity, `retain`, `drain`, `dedup`, `binary_search`

## At a glance

After this lesson you can:

- Explain why `.push()` is called "amortized O(1)" even though some calls do a full copy of the buffer — and predict, for a given len/capacity pair, whether the next push triggers a reallocation.
- Choose between `.retain()`, `.drain()`, `.swap_remove()` and `.remove()` for a given cleanup task, and say which ones preserve order and which are O(1) versus O(n).
- Fix a `.dedup()` that silently kept duplicates, and a `.binary_search()` that silently gave the wrong answer — both purely because the `Vec` wasn't sorted, with nobody pointing that out for you first.

**Time:** ~70 minutes · **Prerequisites:** [1.7.2 — Phase review](../../../phase1-fundamentals/07-putting-it-together/02-phase-review/README.md), and specifically [1.1.6 — `Vec` and `String` basics](../../../phase1-fundamentals/01-foundations/06-vec-and-string-basics/README.md)

---

## Why this matters

Up to now you've treated `Vec` like a bag: `push` something in, `pop` something out, look at it with `[]` or `.get()`, walk it with `for x in &v`. [1.1.6](../../../phase1-fundamentals/01-foundations/06-vec-and-string-basics/README.md) even gave you one glimpse past that — length versus capacity, and capacity jumping from 0 to 4 and then to 8. That was everything Phase 1 needed.

Real code — the kind you'll write for a backend in Phase 3 — keeps asking that bag for more. Imagine you're building the watch-history feature of an anime-tracking backend: a user logs the same episode twice (you need to remove the duplicate), you want to show only the unwatched ones (you need to filter), you want a top-rated leaderboard (you need to sort), and you want a fast lookup in a list you already keep sorted (you need to binary-search). None of that is exotic tooling — it is exactly the set of methods real Rust code reaches for constantly.

This lesson picks up exactly where Phase 1 stopped: not new syntax, but what is actually inside that bag, and the precise vocabulary for the things you do with it every day.

---

## The concept

### Capacity, on purpose: `with_capacity`, `.reserve()`, `.shrink_to_fit()`

[1.1.6](../../../phase1-fundamentals/01-foundations/06-vec-and-string-basics/README.md) showed you that **length** (how many are in it) and **capacity** (how much room is already reserved) are two different things, and you already met `Vec::with_capacity(n)`. Now it's time to control it on purpose:

```rust
let mut surprised: Vec<u32> = Vec::new();
let mut planned: Vec<u32> = Vec::with_capacity(8);
```

```text
Vec::new()           : len 0, capacity 0
Vec::with_capacity(8): len 0, capacity 8
```

`Vec::new()` reserves nothing. `Vec::with_capacity(8)` reserves 8 slots up front — still empty, just room waiting. Now fill both with 8 episodes:

```rust
for episode in 1..=8 {
    surprised.push(episode);
    planned.push(episode);
}
```

```text
surprised, after 8 pushes: len 8, capacity 8
planned,   after 8 pushes: len 8, capacity 8
```

No difference visible here — both needed exactly 8 slots and both have exactly 8. The difference shows up when you don't know exactly how many you'll need. If you have a rough estimate of how many more are coming, `.reserve(n)` grants that same room on a `Vec` that already exists:

```rust
planned.reserve(20);
```

```text
planned, after .reserve(20): len 8, capacity 28
```

`.reserve(20)` means "make sure at least 20 more fit" — not exactly 20, at least 20; the standard library is allowed to take a bit more if that's more efficient (here, 28 = 8 + 20). And the other direction — a `Vec` used to be big, is small now, and that spare capacity is just sitting there — `.shrink_to_fit()` returns capacity as close to length as it can:

```rust
planned.shrink_to_fit();
```

```text
planned, 2 left + .shrink_to_fit(): len 2, capacity 2
```

### Amortized growth: why `push` is amortized O(1) despite the occasional copy

Whenever `push` needs more room than the current capacity has, `Vec` allocates a new buffer — roughly **double** the size — copies every existing element into it, and frees the old one. That's exactly the 4-to-8 jump you saw in [1.1.6](../../../phase1-fundamentals/01-foundations/06-vec-and-string-basics/README.md). Watch where those jumps land over a longer list:

```rust
let mut episodes: Vec<u32> = Vec::new();
let mut last_capacity = episodes.capacity();
for episode in 1..=40 {
    episodes.push(episode);
    if episodes.capacity() != last_capacity {
        last_capacity = episodes.capacity();
    }
}
```

```text
len  1 -> capacity jumped  0 to  4
len  5 -> capacity jumped  4 to  8
len  9 -> capacity jumped  8 to 16
len 17 -> capacity jumped 16 to 32
len 33 -> capacity jumped 32 to 64
```

Out of 40 pushes, only five actually triggered a reallocation. The other 35 were a plain write into room that was already there.

```senpai-visual
{"kind":"concept","labels":["len == cap","allocate 2×","copy old data","free old buffer","push succeeds"]}
```

That is exactly what "amortized O(1)" means: not that *every* `push` is cheap — every fifth or ninth one does a real, expensive copy — but that the **total** work across all the pushes for `n` elements is proportional to `n`, not `n²`. You can measure it yourself: each jump copies roughly every existing element, and this program kept a running total:

```text
40 pushes total, only 60 element-moves happened during a regrow
```

Sixty element-moves for forty pushes — not forty (which would mean no copying at all), but nowhere near 40² = 1600 (which would mean every push re-copied everything). Reserve those same 40 slots up front with `Vec::with_capacity(40)`, and those sixty moves drop to zero — exactly what the previous section showed you.

### `.retain(|entry| ...)` — filtering in place

```rust
list.retain(|entry| !entry.watched);
```

```text
before: [Anime { title: "Frieren", watched: true }, Anime { title: "Bocchi the Rock!", watched: false }, Anime { title: "Mushoku Tensei", watched: true }, Anime { title: "Made in Abyss", watched: false }]
after:  [Anime { title: "Bocchi the Rock!", watched: false }, Anime { title: "Made in Abyss", watched: false }]
```

`.retain(...)` walks the whole `Vec` once and keeps only the elements the closure answers `true` for — everything else is dropped in place, without disturbing the order of whatever survives. That `|entry| !entry.watched` is a **closure** — a small, unnamed function written right where it's used. [2.2.1](../../02-iterators-and-closures/01-closures-and-fn-traits/README.md) teaches closures properly; for now just read it as "for each entry, answer: keep this one?"

### `.drain(range)` — takes it out AND hands it back

```rust
let mut just_watched: Vec<u32> = Vec::new();
for episode in queue.drain(0..4) {
    just_watched.push(episode);
}
```

```text
queue before: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
just watched: [1, 2, 3, 4]
queue after:  [5, 6, 7, 8, 9, 10]
```

`.drain(0..4)` pulls that range out of `queue` and hands it back to you as an iterator — one operation, both taking and getting. `queue` itself stays completely usable:

```rust
queue.push(11);
```

```text
still usable, pushed 11: [5, 6, 7, 8, 9, 10, 11]
```

`.drain(..)` — the full range — takes everything, much like `.clear()`, with one difference: `.clear()` throws away whatever it removes; `.drain(..)` hands it back to you as an iterator so you decide what to do with it. Both leave capacity untouched:

```text
drain(..) took:        [5, 6, 7, 8, 9, 10, 11]
queue after drain(..): [], capacity still 10 (was 10)
```

### `.dedup()`, `.dedup_by()`, `.dedup_by_key()` — neighbors only

```rust
ratings.dedup();
```

```text
adjacent duplicates only: [7, 7, 9, 9, 9, 5, 7]
after .dedup():           [7, 9, 5, 7]
```

`.dedup()` only removes **neighboring** equal pairs — exactly one, not zero, survives from each consecutive run. That last `5` and `7` weren't neighbors with anything equal, so both stayed, even though a `7` had already shown up earlier.

If your notion of equal isn't simple — say, "same title, case doesn't matter" — `.dedup_by()` takes a custom comparison:

```rust
titles.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
```

```text
same show, case differs:  ["bocchi", "Bocchi", "frieren"]
after .dedup_by():        ["bocchi", "frieren"]
```

And if you want to compare by a **derived key** rather than the element itself, `.dedup_by_key()` pulls that key from each element and compares neighboring keys:

```rust
logged.dedup_by_key(|entry| entry.0.clone());
```

```text
logged twice by title:    [("Frieren", 9), ("Frieren", 10), ("Mushoku Tensei", 8)]
after .dedup_by_key():    [("Frieren", 9), ("Mushoku Tensei", 8)]
```

The key here was only the title; the rating was ignored. All three — `.dedup()`, `.dedup_by()`, `.dedup_by_key()` — always keep the **first** of each consecutive run, not the last; that's why the rating `9` survived here, not `10`.

**The part you need to hold onto:** all three only ever look at neighbors. If you want *every* duplicate gone — not just the ones that happen to sit next to each other — you have to `.sort()` first so equal elements become neighbors. Forget it, and the compiler says nothing. The full version of this trap is in "Errors you will meet".

### `.sort()` versus `.sort_unstable()`, `.sort_by()` versus `.sort_by_key()`

On types the compiler can fully order — numbers, text — `.sort()` needs no closure at all:

```rust
episode_counts.sort();
```

```text
plain .sort():           [1, 12, 12, 24, 64]
```

But sometimes you have to say "sort by what, exactly" yourself — for instance when the value is a floating-point number (you'll see in a moment why a plain `.sort()` on `f64` doesn't even compile), or when you want to sort by a field. `.sort_by()` takes a comparator:

```rust
stable.sort_by(|a, b| a.rating.total_cmp(&b.rating));
```

```text
stable   .sort_by():           ["A", "B", "C"]
```

Here two entries — `"A"` and `"B"` — both had a rating of 7.5, and `"A"` came first in the input. `.sort()`/`.sort_by()`/`.sort_by_key()` are all **stable**: when two elements are equal, their original relative order survives untouched — exactly why `"A"` is still ahead of `"B"`.

The `.sort_unstable_by()` version does the same job, with one less promise:

```rust
unstable.sort_unstable_by(|a, b| a.rating.total_cmp(&b.rating));
```

```text
unstable .sort_unstable_by():  ["A", "B", "C"]  (no promise, just what this run did)
```

Same answer! But that is a **coincidence**, not a guarantee — on a list this small, the internal algorithm often *looks* stable without ever promising to be. The standard library says plainly: it "may" reorder equal elements — not that it will. The real difference is speed: `.sort_unstable*` allocates no auxiliary buffer and is generally faster; `.sort`/`.sort_by`/`.sort_by_key` buy that stability guarantee with a bit of extra memory. Reach for the unstable version when the order of equal elements doesn't matter to you — or you never have any ties — and the stable one when it does.

### `.binary_search()` and `.binary_search_by()` — only when the `Vec` is already sorted

```rust
by_rating.binary_search_by(|entry| entry.rating.total_cmp(&target))
```

```text
rating 9: found "Frieren" at index 2
rating 8: not found, belongs at index 2
```

Given a list that is **already sorted** by whatever you're looking for, `.binary_search_by()` throws away half of what's left on every step — `O(log n)`, not a linear `O(n)` walk. The answer is `Ok(index)` if found, or `Err(insert_at)` if not — and that `insert_at` is not arbitrary: it's exactly where the value would have to go for the list to stay sorted. `target = 8` belongs between index 1 (rating 7.5) and index 2 (rating 9.0) — index 2, which is exactly what came back.

**Take its one hard requirement seriously:** the `Vec` must already be sorted, exactly by whatever your comparator measures. Nothing in the signature enforces that, nothing is written down anywhere — and breaking it doesn't panic. You just get the wrong answer. The full story is in "Errors you will meet".

### `.swap_remove()` versus `.remove()` — O(1) that reorders, versus O(n) that preserves order

```rust
let removed = fast.swap_remove(1);
```

```text
before:              ["Frieren", "Bocchi", "Mushoku Tensei", "Made in Abyss"]
swap_remove(1) took: "Bocchi"
left, order changed: ["Frieren", "Made in Abyss", "Mushoku Tensei"]
```

`.swap_remove(i)` takes out the element at index `i`, moves the **last** element into its place, and shrinks the length by one — one write, one shrink, `O(1)` no matter how big the `Vec` is. The price: the order is no longer what it was.

```rust
let removed = ordered.remove(1);
```

```text
before:              ["Frieren", "Bocchi", "Mushoku Tensei", "Made in Abyss"]
remove(1) took:      "Bocchi"
left, order intact:  ["Frieren", "Mushoku Tensei", "Made in Abyss"]
```

`.remove(i)` shifts every element after `i` back by one — order stays exactly as it was, but the price is `O(n)`: everything after `i` moves. For an unordered playback queue or a disposable cache, `.swap_remove()` is cheaper for free. For a sorted playlist a user genuinely looks at in order, `.remove()` is the only correct choice.

---

## Hands on

```sh
cargo run -p p2-01-01-vec-depth --example 01-capacity-and-reserve
cargo run -p p2-01-01-vec-depth --example 02-amortized-growth
cargo run -p p2-01-01-vec-depth --example 03-retain
cargo run -p p2-01-01-vec-depth --example 04-drain
cargo run -p p2-01-01-vec-depth --example 05-dedup-family
cargo run -p p2-01-01-vec-depth --example 06-dedup-without-sort-trap
cargo run -p p2-01-01-vec-depth --example 07-sort-family
cargo run -p p2-01-01-vec-depth --example 08-binary-search
cargo run -p p2-01-01-vec-depth --example 09-binary-search-unsorted-trap
cargo run -p p2-01-01-vec-depth --example 10-swap-remove-vs-remove
```

Then the two broken ones:

```sh
cargo run -p p2-01-01-vec-depth --example 11-sort-ord-not-satisfied --features broken
cargo run -p p2-01-01-vec-depth --example 12-remove-out-of-bounds --features broken
```

Then try:

1. In `02-amortized-growth`, change `40` to `100`. How many more capacity jumps do you see, and how many elements get moved this time?
2. In `05-dedup-family`, change the `logged` line so three "Frieren" entries land in a row instead of two. How many does `.dedup_by_key()` keep?
3. In `10-swap-remove-vs-remove`, try `.swap_remove(3)` on the four-item list — the last index. Does anything move? Why does that make sense?

---

## Errors you will meet

### No error at all — `.dedup()` on an unsorted `Vec` silently keeps duplicates

```rust
let mut logged = vec![
    "Frieren".to_string(),
    "Bocchi".to_string(),
    "Frieren".to_string(),
];
logged.dedup();
```

```text
before:              ["Frieren", "Bocchi", "Frieren"]
after .dedup():      ["Frieren", "Bocchi", "Frieren"]
(still 3 entries — "Frieren" was never adjacent to itself)
```

**What the compiler is objecting to:** nothing. This code compiles, runs, and finishes without any error at all. The problem is that the answer is *wrong*: there are two "Frieren" entries in the list, but `.dedup()` would only have removed one of them if they were neighbors — here they weren't, so both stayed.

**The fix:** sort before you `.dedup()`:

```rust
sorted_first.sort();
sorted_first.dedup();
```

```text
sort() then dedup(): ["Bocchi", "Frieren"]
```

**Why that's the fix:** `.sort()` brings every "Frieren", wherever it was in the list, next to every other "Frieren" — they become neighbors, and now `.dedup()` can actually see them. **This category of bug is the most dangerous kind there is:** no red squiggle, no panic, nothing a shallow test would catch — just a number, or a list, that's a little wrong, somewhere in production.

### `E0277` — you can't `.sort()` a `Vec<f64>` directly

```text
error[E0277]: the trait bound `f64: Ord` is not satisfied
   --> phase2-intermediate\01-collections\01-vec-depth\examples\11-sort-ord-not-satisfied.rs:12:13
    |
 12 |     ratings.sort();
    |             ^^^^ the trait `Ord` is not implemented for `f64`
    |
    = help: the following other types implement trait `Ord`:
              i128
              i16
              i32
              i64
              i8
              isize
              u128
              u16
            and 4 others
note: required by a bound in `slice::<impl [T]>::sort`
   --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\alloc\src\slice.rs:133:12
    |
131 |     pub fn sort(&mut self)
    |            ---- required by a bound in this associated function
132 |     where
133 |         T: Ord,
    |            ^^^ required by this bound in `slice::<impl [T]>::sort`

For more information about this error, try `rustc --explain E0277`.
```

**What the compiler is objecting to:** a bare `.sort()` only works for types with a total, unambiguous order — for any two values, there is always exactly one answer to "which is smaller?" Integers have that. `f64` doesn't: because of `NaN` ("not a number" — the result of something like 0.0 ÷ 0.0), which cannot be compared to any other number, not even itself. That total, unambiguous order has an official name — a trait called `Ord` — and this error says `f64` doesn't make that promise. ([2.3.4](../../03-traits-and-generics/04-standard-derives-by-hand/README.md) covers `Ord` and its family properly; this one paragraph is all you need today.)

**The fix:** give it an explicit comparison:

```rust
ratings.sort_by(|a, b| a.total_cmp(b));
```

**Why that's the fix:** `.sort_by()` doesn't wait around for `Ord` — you write the comparison yourself. `f64::total_cmp` exists for exactly this: a total order over *every* `f64` value, `NaN` included, for the cases where sorting matters more than the mathematical debate over whether `NaN` is really a number.

### `.binary_search()` on an unsorted `Vec` — not found, even though it was there

```rust
let ratings: Vec<f64> = vec![9.0, 6.5, 9.5, 7.5];
ratings.binary_search_by(|value| value.total_cmp(&7.5))
```

```text
ratings (unsorted): [9.0, 6.5, 9.5, 7.5]
binary_search for 7.5: not found
a real, linear search:  Some(3)
```

**What the compiler is objecting to:** again, nothing. `binary_search_by` can never check whether the list is actually sorted — that check would itself be a linear walk, defeating the entire point of `O(log n)`. So it doesn't claim to check; it just assumes. Here the assumption was wrong: `7.5` really is in the list, right at index 3 — but the algorithm, assuming sorted order, looked in places that index never reaches.

**The fix:** either sort first, or, when you can't be sure it's sorted, write a plain linear search — a loop that genuinely looks at every element.

**Why that's the fix:** a linear search always gives the right answer, sorted or not — it's just slower, `O(n)` instead of `O(log n)`. Only reach for `binary_search_by` when you can genuinely finish this sentence: "this `Vec` is sorted, right now, by exactly this field" — not "I think it's sorted."

### The `.remove()` panic — index out of range

```text
thread 'main' (21236) panicked at phase2-intermediate\01-collections\01-vec-depth\examples\12-remove-out-of-bounds.rs:11:22:
removal index (is 5) should be < len (is 3)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**What the compiler is objecting to:** this isn't a compiler error either — the program built and ran, and panicked exactly where `.remove(5)` was called on a three-element `Vec`. Unlike `.get()`, which politely returns `None` for an invalid index, neither `.remove()` nor `.swap_remove()` returns an `Option` — an invalid index here means a bug in the caller's code, not a normal case to handle.

**The fix:** check the index against the current length before calling:

```rust
if index < queue.len() {
    let gone = queue.remove(index);
    println!("removed: {gone}");
} else {
    println!("no such index");
}
```

**Why that's the fix:** the panic message already says exactly what was violated — "removal index (5) should be less than len (3)" — and the fix checks that same contract before breaking it. If the index comes from somewhere external (user input, a network request), always write that check first; if you built the index yourself right above and you're sure it's valid, the panic is exactly the thing you want — a bug caught early, not late.

---

## Exercises

### Warm up

<details>
<summary>Build a <code>Vec::with_capacity(5)</code> and push three times. What are its <code>len</code> and <code>capacity</code>?</summary>

```rust
let mut v: Vec<i32> = Vec::with_capacity(5);
v.push(1);
v.push(2);
v.push(3);
```

</details>

<details>
<summary>Answer</summary>

```text
len 3, capacity 5
```

Three pushes haven't reached those five reserved slots yet, so no reallocation happened and capacity is still 5.

</details>

<details>
<summary>Does <code>let v: Vec&lt;f64&gt; = vec![3.0, 1.0]; v.sort();</code> compile?</summary>

Decide before reading "Errors you will meet".

</details>

<details>
<summary>Answer</summary>

No — `E0277`, because `f64` doesn't implement `Ord` (thanks to `NaN`). You'd need `.sort_by()` with an explicit comparison like `total_cmp`.

</details>

<details>
<summary>What survives <code>vec![5, 5, 3, 3, 5].dedup()</code>?</summary>

Count each consecutive run separately.

</details>

<details>
<summary>Answer</summary>

```text
[5, 3, 5]
```

Three consecutive runs: `(5, 5)`, `(3, 3)`, `(5)`. Only the first of each survives — and those three aren't neighbors with each other, so all three stay.

</details>

<details>
<summary>What exactly does <code>binary_search_by</code>'s <code>Err</code> hold when nothing is found?</summary>

Decide before reading on.

</details>

<details>
<summary>Answer</summary>

The index where the value would have to sit for the list to stay sorted — an insertion point, not just "not found".

</details>

<details>
<summary>What order is left after <code>["a", "b", "c"].swap_remove(0)</code>?</summary>

Decide before reading on.

</details>

<details>
<summary>Answer</summary>

```text
["c", "b"]
```

Index 0 is taken out, and the last element (`"c"`) moves into its place.

</details>

### Repair

Fix both broken examples:

1. Fix `examples/11-sort-ord-not-satisfied.rs` so it compiles — with `.sort_by()` or `.sort_unstable_by()` and an explicit comparison, not by changing the data to integers.
2. Fix `examples/12-remove-out-of-bounds.rs` so it no longer panics — add a length check before `.remove()`, and print something sensible for an invalid index.

And these two traps too — even though they neither panic nor get rejected, they're just as real:

3. Write a version of `examples/06-dedup-without-sort-trap.rs` (or just explain it on paper) that genuinely removes *every* duplicate, not just neighboring ones.
4. Change `examples/09-binary-search-unsorted-trap.rs` so it always gives the correct answer — either by sorting the list before searching, or by replacing `binary_search_by` with a linear search.

### Implement

Five functions in `src/lib.rs`, each built around one of this lesson's methods:

```sh
cargo test -p p2-01-01-vec-depth
```

All five work on a `struct WatchEntry { title: String, rating: f64, watched: bool }` — one row of watch history. Each function's doc comment states exactly what it returns; don't guess.

### Build

Write a `pub fn remove_by_title_fast(entries: &mut Vec<WatchEntry>, title: &str) -> Option<WatchEntry>` that finds and removes the first entry matching `title` — using whichever of `.swap_remove()` or `.remove()` you think is right, and say in its doc comment why you picked that one and when the other choice would be better.

### Challenge (optional)

Change `examples/02-amortized-growth.rs` so that, instead of just printing the jumps, it computes the **total element-moves** for a few different final lengths — say 40, 400, 4000. Print the ratio of "moves to final length" for all three. Does that ratio grow as `n` grows, or does it stay near the same small number? That hand-run experiment is exactly what "amortized O(1)" is claiming.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Capacity | how much room is reserved, apart from length | deciding between `with_capacity`/`reserve` |
| Amortized growth | capacity doubling, which keeps the total cost of `n` pushes proportional to `n` | understanding why `push` is usually cheap |
| `.retain()` | in-place filter; only elements the closure approves survive | cleaning up a list by a condition |
| `.drain()` | removes a range and hands it back as an iterator | splitting a list in two without an extra copy |
| `.dedup_by_key()` | removes *neighboring* duplicates by a derived key | deduplication, always after `.sort()` if you want every duplicate gone |
| Stable / unstable | a stable sort promises equal elements keep their relative order; unstable doesn't | choosing between `.sort_by()` and `.sort_unstable_by()` |
| `.binary_search_by()` | `O(log n)` search on an already-sorted `Vec` | fast lookup, only when the sorted precondition genuinely holds |
| `.swap_remove()` / `.remove()` | O(1) that reorders, versus O(n) that preserves order | choosing based on whether order matters to you |

### What you now know

- Length and capacity are two different things; `.reserve()` and `.shrink_to_fit()` give you the same control over an existing `Vec` that `with_capacity` gave a new one.
- `push` is amortized O(1) because capacity doubles each time; the total work across `n` pushes is proportional to `n`, not `n²` — something you measured yourself by counting element-moves.
- `.retain()`, `.drain()`, `.dedup_by_key()`, `.sort_by()` and `.binary_search_by()` are everyday vocabulary for work you used to write with a manual loop.
- `.dedup()` and its family only see neighbors; to remove *every* duplicate, `.sort()` first — forgetting it doesn't error, it just gives the wrong answer.
- `.binary_search_by()` never checks its precondition; on an unsorted `Vec`, its answer isn't trustworthy, but it won't panic either.
- `.swap_remove()` is fast but breaks order; `.remove()` preserves order but is slower — and both panic on an invalid index.

### What comes back later

- **Closures, `Fn`/`FnMut`/`FnOnce`** — [2.2.1 — Closures and the `Fn` traits](../../02-iterators-and-closures/01-closures-and-fn-traits/README.md)
- **`Ord`, `PartialOrd` and the standard-derive family** — [2.3.4 — Standard derives, by hand](../../03-traits-and-generics/04-standard-derives-by-hand/README.md)
- **Iterator adapters and `.collect()`** — [2.2.2 — Iterator adapters](../../02-iterators-and-closures/02-iterator-adapters/README.md) and [2.2.3 — Consuming and collecting](../../02-iterators-and-closures/03-consuming-and-collecting/README.md)
- **`HashMap` and the `entry` API** — [2.1.2 — `HashMap` in depth](../02-hashmap-in-depth/README.md)
- **When to reach for a different collection than `Vec`** — [2.1.4 — Choosing a collection](../04-choosing-a-collection/README.md)

### Can you explain?

- Why is `push` called "amortized O(1)" even though some of its calls do a full buffer copy?
- What's the difference between `.retain()` and `.drain()`? Give a real example where each is the right tool.
- Why can `.dedup()` on an unsorted `Vec` keep duplicates around without any error at all?
- Why doesn't `.binary_search()` panic on an unsorted `Vec`, and yet its answer still can't be trusted?
- For an unordered playback queue and a sorted playlist, which one wants `.swap_remove()` and which wants `.remove()`, and why?

---

## Going further

- [`Vec` docs — Capacity and reallocation](https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation) — the official version of what you saw in "amortized growth".
- [`slice` docs](https://doc.rust-lang.org/std/primitive.slice.html) — most of what you used today — `sort`, `sort_by`, `binary_search`, `dedup` — is really defined on `[T]`, not on `Vec<T>` itself; `Vec` just gives you automatic access to it.
- [The Rustonomicon — building `Vec` from scratch](https://doc.rust-lang.org/nomicon/vec/vec.html) — for when you're curious how the allocation and growth are actually written, by hand.
