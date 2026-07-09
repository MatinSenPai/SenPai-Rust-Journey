# Solution

```rust
pub fn first_n_completed<'a>(shows: &'a [(String, bool)], n: usize) -> Vec<&'a str> {
    shows
        .iter()
        .filter(|(_, done)| *done)
        .map(|(title, _)| title.as_str())
        .take(n)
        .collect()
}
```

The order of `.filter()` then `.take(n)` matters and answers checkpoint
question 5. `.take(n)` doesn't know anything about "completed" — it just
stops the pipeline after `n` items have flowed through *whatever comes
before it*. If `.take(n)` ran first (`shows.iter().take(n).filter(...)`),
you'd grab the first `n` shows *in the original list*, completed or not,
and then filter *those* down — you could easily end up with fewer than `n`
results even when plenty of completed shows exist later in the list.
Putting `.filter()` first means only completed shows ever reach `.take()`,
so `.take(n)` correctly means "the first `n` shows that passed the
filter." This is also a nice illustration of laziness in action: because
adapters are lazy, `.take(n)` can short-circuit the whole chain the moment
it has `n` items — it never even asks `.filter()` to look at the remaining
shows, unlike a Python approach that (naively) filters the entire list
first and slices afterward.

```rust
pub fn total_episodes(shows: &[(String, u32)]) -> u32 {
    shows.iter().map(|(_, episodes)| episodes).sum()
}
```

`.map(|(_, episodes)| episodes)` destructures each `&(String, u32)` tuple
reference, throwing away the title and keeping only a reference to the
episode count, then `.sum()` (checkpoint question 3) adds them all up.
`.sum()` is `Iterator`'s dedicated "add everything together" consumer —
reach for `.fold()` instead the moment your combining step is anything
other than addition/multiplication/counting: for example, building up a
running maximum with tie-breaking logic, concatenating strings with a
custom separator rule, or accumulating into a struct instead of a single
number. `.fold(initial, |acc, item| ...)` can express all of those;
`.sum()` can only express "add."

On checkpoint question 2: `vec![1, 2, 3].iter().map(|n| n * 2)` by itself
does **nothing** — it produces a `Map` iterator value that has not been
run yet, and if you never consume it (no `.collect()`, no `.sum()`, no
`for` loop over it), the multiplication never happens at all. In fact, an
unused iterator like this typically triggers a compiler/clippy warning
("unused `Map` that must be used") precisely because leaving one dangling
is almost always a mistake. Rust designs iterators this way so that a long
chain of `.filter().map().enumerate()` compiles down to a single tight
loop over the source data with no intermediate `Vec`s allocated at each
step — laziness here is a genuine performance decision, not just an
API quirk.
