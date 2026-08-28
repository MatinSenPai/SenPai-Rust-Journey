# 01.1 — `Vec` and `HashMap`

You already know `Vec<T>` a little from Phase 1 (it's Rust's growable array,
roughly Python's `list`). This lesson drills two things: a couple of `Vec`
methods you probably haven't needed yet, and `HashMap<K, V>` — Rust's `dict`
equivalent, with one very idiomatic trick (the **entry API**) that doesn't
have a clean Python parallel until you reach for `collections.Counter` or
`dict.setdefault`.

The running example for this whole module: analyzing a list of anime/show
watch data.

## `Vec`: sorting and deduping

```rust
let mut nums = vec![3, 1, 4, 1, 5, 9, 2, 6];
nums.sort();                 // [1, 1, 2, 3, 4, 5, 6, 9] — sorts in place
nums.dedup();                // [1, 2, 3, 4, 5, 6, 9]    — removes CONSECUTIVE duplicates only
```

`.dedup()` only removes duplicates that are *adjacent* — that's why you
almost always `.sort()` immediately before it (so equal elements become
neighbors). If you need to dedupe by some derived key rather than the
element itself, use `.sort_by_key(|x| ...)`:

```rust
let mut shows = vec![("Frieren", 2023), ("Bocchi", 2022), ("Frieren", 2023)];
shows.sort_by_key(|(title, _year)| title.clone());
```

Python's `sorted(list, key=...)` returns a *new* list; `.sort_by_key` sorts
`nums` itself, in place, and returns `()` — another instance of Rust
preferring an explicit, mutating method over an allocating one when you
don't need the original order preserved.

## `HashMap`: the entry API

Coming from Python, counting word occurrences probably looks like this:

```python
counts = {}
for word in words:
    if word in counts:
        counts[word] += 1
    else:
        counts[word] = 1
# or: counts = collections.Counter(words)
```

A direct Rust translation of the `if/else` version works, but does the
lookup *twice* (once for `.contains_key`, once for the `[]`/`.get_mut`
that follows). Rust's `HashMap` instead gives you the **entry API**, which
does one lookup and hands you a mutable "slot" you can either fill or
update in a single expression:

```rust
use std::collections::HashMap;

let mut counts: HashMap<String, usize> = HashMap::new();
for word in words {
    *counts.entry(word).or_insert(0) += 1;
}
```

`.entry(word)` returns an `Entry` — "the slot for this key, whether or not
it exists yet." `.or_insert(0)` says "if empty, put `0` there," and either
way hands back a `&mut usize` pointing at the count, which `*... += 1`
increments in place. One lookup, one line, no branch. `.or_insert_with(||
...)` is the lazy version — use it when the default value is expensive to
construct (e.g. `.or_insert_with(Vec::new)`), since `or_insert`'s argument
is always evaluated eagerly even when the key already exists.

## `HashMap` iteration order is NOT guaranteed

This is the single biggest surprise coming from Python 3.7+, where a plain
`dict` remembers insertion order and iterating it is deterministic. Rust's
`HashMap` makes **no ordering promise at all** — the order you get from
`.iter()`, `.keys()`, or a `for (k, v) in &map` can differ between runs of
the *same program*, because it's randomized per-process (a deliberate
security measure against hash-flooding attacks). If you need a
deterministic order — for display, for tests, for anything a human or an
assertion will check — you must **collect and sort explicitly**. That's
exactly why `top_n` below can't just take the first `n` entries it happens
to iterate: it has to sort the collected pairs itself, with an explicit
tie-breaker, or two runs of your program could show a different "top 5"
for entries with equal counts. (Lesson 2 introduces `BTreeMap`, which keeps
keys sorted for you automatically — at a small performance cost.)

## Your task

Implement the three functions in `src/lib.rs`:

- `word_frequency` — count word occurrences using the entry API.
- `top_n` — return the top-`n` (word, count) pairs, sorted by count
  descending, then alphabetically for ties.
- `dedupe_preserve_order` — remove duplicates while keeping first-seen
  order (notice why `.sort().dedup()` alone would be wrong here: it would
  silently reorder the input).

## Next

`solution/SOLUTION.md` — but only after a real attempt.
