# Solution — 2.2.2 Iterator adapters

```rust
pub fn uppercase_titles(shows: &[Show]) -> Vec<String> {
    shows.iter().fold(Vec::new(), |mut acc, show| {
        acc.push(show.title.to_uppercase());
        acc
    })
}

pub fn completed_titles(shows: &[Show]) -> Vec<&str> {
    let mut out = Vec::new();
    for show in shows.iter().filter(|show| show.completed) {
        out.push(show.title.as_str());
    }
    out
}

pub fn total_episodes_watched(shows: &[Show]) -> u32 {
    shows
        .iter()
        .filter(|show| show.completed)
        .fold(0, |acc, show| acc + show.episodes)
}

pub fn first_n_in_progress(shows: &[Show], n: usize) -> Vec<&str> {
    let mut out = Vec::new();
    for show in shows.iter().filter(|show| !show.completed).take(n) {
        out.push(show.title.as_str());
    }
    out
}

pub fn ranked_titles(shows: &[Show]) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (index, show) in shows.iter().enumerate() {
        out.push((index + 1, show.title.clone()));
    }
    out
}
```

None of these five needed `.collect()` — every one of them built the `Vec` it wanted by hand, either with `.fold()` or with a plain `for`. Both are shown here on purpose; either one could swap for the other.

## `uppercase_titles` — `.fold()` instead of a `for`

```rust
shows.iter().fold(Vec::new(), |mut acc, show| {
    acc.push(show.title.to_uppercase());
    acc
})
```

This is exactly the pattern from "The concept": the starting value is an empty `Vec::new()`, every item adds one `push` to it, and the closure hands back that same `acc` — now with one more in it. A `for` loop ("build an empty `Vec`, `push` into it each time, return it at the end") would have been just as correct; `.fold()` was chosen here purely so you can see with your own eyes how directly that pattern turns into a single adapter.

## `completed_titles` and `first_n_in_progress` — why `&str`, not `String`

```rust
out.push(show.title.as_str());
```

Both specifications said "borrowed, not cloned" — and `.as_str()` gives exactly that: a `&str` pointing at the very same `String` living inside `shows`, without copying a single byte. Both signatures show this on their face too: `shows: &[Show]` comes in, `Vec<&str>` goes out — no explicit lifetime was needed, nothing else either; the same elision rule from Phase 1 quietly did its job here too.

## `first_n_in_progress` — why `.filter()` has to come before `.take()`

```rust
shows.iter().filter(|show| !show.completed).take(n)
```

This order is not an accident. `.take(n)` knows nothing about "in progress" — it only says "stop after `n` items have come through whatever was in front of me in the pipeline." Swap the order (`shows.iter().take(n).filter(...)`) and you'd grab the first `n` shows of the **whole original list** — completed or not — and only then filter those down; you could easily end up with fewer than `n` results even when the list had plenty of in-progress shows further along. The `first_n_in_progress_stops_at_n_matches` test checks exactly this — and it's the same question that came up in "Warm up" and in the `.take_while()` discussion: a bounding adapter only ever looks at what reaches it through the pipeline, never at the original source.

## `total_episodes_watched` — `.filter()` then `.fold()`, not an `if` inside the closure

```rust
shows
    .iter()
    .filter(|show| show.completed)
    .fold(0, |acc, show| acc + show.episodes)
```

This could have been written as a single `.fold()` too — an `if show.completed { acc + show.episodes } else { acc }` inside its closure — and it would land on the same answer. But the version above names each step separately: "keep only these," "now sum them" — exactly what you saw separately in the "Filter" section and the "fold" section, just chained back to back here.

## `ranked_titles` — `enumerate` plus `+ 1`

```rust
for (index, show) in shows.iter().enumerate() {
    out.push((index + 1, show.title.clone()));
}
```

`.enumerate()` counts from `0`, but the spec asked for a rank starting at `1` — hence the `+ 1`, the only difference from a raw index. `.clone()` was genuinely necessary here, unlike the previous two functions: the return type is `Vec<(usize, String)>`, not `Vec<(usize, &str)>` — the result has to own its strings, not just borrow them.

## What this lesson was really about

- **No adapter builds anything by itself.** Each of the five functions above chained several adapters together, but it was the `for` loop or the `.fold()` itself that actually built the resulting `Vec` — the same central point from "The concept," now in your own code.
- **Adapter order matters.** `.filter()` before `.take()` gives a different answer than `.take()` before `.filter()` — because each one only sees what made it past the adapter in front of it.
- **Borrowing is cheaper than cloning, when it's enough.** `completed_titles` and `first_n_in_progress` never copied a single string; only `ranked_titles`, because its return type demanded ownership, needed `.clone()`.
