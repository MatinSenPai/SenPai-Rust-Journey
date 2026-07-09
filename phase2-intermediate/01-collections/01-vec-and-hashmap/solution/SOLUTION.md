# Solution

```rust
pub fn word_frequency(text: &str) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for word in text.split_whitespace() {
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    counts
}
```

The interesting line is `*counts.entry(word.to_lowercase()).or_insert(0) += 1;`.
`.entry(key)` returns an `Entry` enum — a handle to "the slot for this key,"
whether it currently exists or not. `.or_insert(0)` resolves that handle:
if the key was missing, it inserts `0` and returns a `&mut usize` pointing
at the freshly-inserted `0`; if the key already existed, it returns a
`&mut usize` pointing at the *existing* value, untouched. Either way you get
back a mutable reference to the count, and `*... += 1` writes through that
reference to increment it. All of this — check, maybe-insert, hand back a
mutable handle — is **one** call into the `HashMap`'s internals, not two,
which is exactly the answer to checkpoint question 2: the naive
`if map.contains_key(&word) { *map.get_mut(&word).unwrap() += 1 } else {
map.insert(word, 1) }` does the key lookup *twice* on the "already exists"
path (once for `contains_key`, once for `get_mut`), where the entry API
does it once regardless of which branch you end up on.

```rust
pub fn top_n(freqs: &HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
    let mut pairs: Vec<(String, usize)> =
        freqs.iter().map(|(word, count)| (word.clone(), *count)).collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    pairs.truncate(n);
    pairs
}
```

On checkpoint question 4: dropping the `.then_with(|| a.0.cmp(&b.0))`
tie-breaker and sorting only by count would make the *relative order of
tied entries* depend on whatever order `.iter()` happened to yield them in
— which, per `HashMap`'s unordered-iteration guarantee (or rather, lack of
one), can differ between runs of the same program. Concretely: the test
`top_n_sorts_by_count_descending_then_alphabetically` asserts `"b"` comes
before `"c"` (both count `5`) — without the tie-breaker, that assertion
would pass or fail nondeterministically depending on the process's hash
seed for that run, which is exactly the kind of flaky test you never want
to ship.
