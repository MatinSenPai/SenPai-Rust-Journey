# Solution — 2.2.3 Consuming and collecting, including `Result<Vec<_>, E>`

```rust
pub fn doubled_long_runs(shows: &[u32], min_episodes: u32) -> Vec<u32> {
    shows
        .iter()
        .filter(|&&count| count >= min_episodes)
        .map(|count| count * 2)
        .collect()
}

pub fn initials(titles: &[&str]) -> String {
    titles.iter().filter_map(|title| title.chars().next()).collect()
}

pub fn episode_lookup(entries: &[(String, u32)]) -> HashMap<String, u32> {
    entries.iter().cloned().collect()
}

pub fn all_genres(shows: &[Vec<String>]) -> HashSet<String> {
    shows.iter().flatten().cloned().collect()
}

pub fn parse_all(inputs: &[&str]) -> Result<Vec<i32>, String> {
    inputs
        .iter()
        .map(|s| s.parse::<i32>().map_err(|e| e.to_string()))
        .collect()
}
```

All five functions are exactly what the whole lesson has been pointing at: an adapter chain, ending in a `.collect()`. None of them has a `for` loop.

## `doubled_long_runs` — filter, then map, then collect into a `Vec`

```rust
shows
    .iter()
    .filter(|&&count| count >= min_episodes)
    .map(|count| count * 2)
    .collect()
```

`.filter()` is exactly what you know from [2.2.2](../../02-iterator-adapters/README.md): only the counts that are `>= min_episodes` get through. `.map()` doubles each one. `.collect()` infers its target from the function's own return type — `Vec<u32>` — so no turbofish and no separate type annotation were needed here; the function's signature already gave Rust that information.

## `initials` — `filter_map` to drop empty strings, collect into a `String`

```rust
titles.iter().filter_map(|title| title.chars().next()).collect()
```

`title.chars().next()` gives back an `Option<char>` — `None` only when `title` is empty. `.filter_map()` does exactly what its name says: it unwraps every `Some(c)` down to `c` and keeps it, and drops every `None` entirely — a `.map()` and a `.filter()` in one step. The result is an iterator of `char`, and because the function's return type is `String`, `.collect()` concatenates them rather than building a `Vec<char>`.

## `episode_lookup` — turn owned, then collect into a `HashMap`

```rust
entries.iter().cloned().collect()
```

`entries` is a `&[(String, u32)]`; `.iter()` over it gives `&(String, u32)`. Building a `HashMap<String, u32>` needs owned tuples, not references — so `.cloned()` turns each `&(String, u32)` into a fully owned `(String, u32)` (both `String` and `u32` are `Clone`). `.collect()` does the rest: each tuple becomes a key-value pair, and if a title repeats you get exactly the "last one wins" behavior from "The concept" — because `.collect()` on a `HashMap` is implemented exactly like a loop of `.insert()`s.

## `all_genres` — `flatten` to flatten a list-of-lists, collect into a `HashSet`

```rust
shows.iter().flatten().cloned().collect()
```

`shows` is a `&[Vec<String>]` — a list of lists. `.iter()` over it gives `&Vec<String>`; `.flatten()` opens each of those inner lists and lays all their `&String`s out one after another in a single flat iterator, with no hand-written nested loop required. `.cloned()` makes each one owned, and `.collect()` — because the return type is `HashSet<String>` — drops duplicates automatically, exactly the way "The concept" showed.

## `parse_all` — the same `Result<Vec<T>, E>`, written by you this time

```rust
inputs
    .iter()
    .map(|s| s.parse::<i32>().map_err(|e| e.to_string()))
    .collect()
```

`s.parse::<i32>()` gives back a `Result<i32, ParseIntError>`. `.map_err(|e| e.to_string())` — the same combinator you met in [1.6.3](../../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.md) — only changes the `Err` side, from `ParseIntError` to `String`, leaving the `Ok` side untouched. So the iterator reaching `.collect()` has `Result<i32, String>` items — exactly what the function's return type, `Result<Vec<i32>, String>`, needs. The rest is the lesson's centerpiece mechanism: the first `Err` short-circuits; if every item was `Ok`, you get back a `Vec` of every unwrapped number.

Compare this against [1.6.3](../../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.md)'s version — same signature, same spec, but that one was an explicit loop with `?`, and this one is three chained methods.

## What this lesson was really about

- `.collect()` is the general-purpose consumer; the function's return type (or a type annotation, or a turbofish) tells it exactly what to build — sometimes without you having to spell it out at all, if the function's signature already carries enough information.
- `.filter_map()` does a `.map()` and a `.filter()` in one step, when the transformation itself returns an `Option`.
- `.flatten()` turns a list-of-lists into one flat iterator, with no hand-written nested loop.
- `.cloned()` before `.collect()` is needed whenever the target wants owned values but all you have are references.
- `Result<Vec<T>, E>` from `.collect()` is exactly the manual loop-plus-`?` you wrote in Phase 1 — only now `.collect()` writes that loop for you.
