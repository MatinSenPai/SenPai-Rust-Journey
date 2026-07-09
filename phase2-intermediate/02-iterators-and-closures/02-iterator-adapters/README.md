# 02.2 — Iterator adapters

This is the single biggest day-to-day habit shift coming from Python. Where
you'd reach for a list comprehension, idiomatic Rust reaches for a chain of
**iterator adapters**.

## Three ways to iterate, tied to ownership (Phase 1, module 02)

```rust
let titles = vec!["Frieren".to_string(), "Mob Psycho".to_string()];

for t in titles.iter() {}      // t: &String  — borrows each element, `titles` still usable after
for t in titles.iter_mut() {}  // t: &mut String — mutably borrows, needs `let mut titles`
for t in titles.into_iter() {} // t: String  — consumes `titles`; it's gone afterward
```

`for t in &titles` is shorthand for `.iter()`, and `for t in titles`
(no `&`) is shorthand for `.into_iter()`. This is exactly the borrow-vs-move
distinction from Phase 1: reach for `.iter()` by default (you almost always
just want to *look at* the data), and only use `.into_iter()` when you
genuinely want to hand ownership of every element away and are done with
the original collection.

## Adapters are lazy — nothing runs until you consume

```rust
let doubled = vec![1, 2, 3].iter().map(|n| n * 2); // nothing has happened yet
let doubled: Vec<i32> = doubled.collect();          // *now* the map runs
```

`.map()`, `.filter()`, `.enumerate()`, `.zip()`, `.take()`, `.skip()`, and
`.take_while()` all return a new iterator that does nothing by itself —
they just describe a pipeline. Only a **consuming** method — `.collect()`,
`.sum()`, `.fold()`, `.count()`, `.for_each()` — actually pulls values
through that pipeline and runs the work. This is a real, meaningful
difference from Python:

```python
doubled = [n * 2 for n in [1, 2, 3]]  # runs immediately, right here, eagerly
```

A Python list comprehension executes the instant you write it. A Rust
iterator chain is closer to a *plan* you build up first — which means you
can freely chain `.filter()` after `.map()` after `.enumerate()` without
Rust ever materializing an intermediate `Vec` at each step; the whole
pipeline runs element-by-element in a single pass only once you consume it.

## `.fold()` — the general-purpose "reduce to one value" tool

```rust
let total = vec![1, 2, 3].iter().fold(0, |acc, n| acc + n); // 6
```

`.fold(initial, |accumulator, item| ...)` is directly equivalent to a
manual loop:

```rust
let mut total = 0;
for n in vec![1, 2, 3].iter() {
    total = total + n;
}
```

`.sum()` is really just `.fold(0, |acc, n| acc + n)` with a friendlier
name for the extremely common case; reach for `.fold()` whenever you need
to combine every item into one result but the combining logic isn't a
simple sum/product/count.

## `.collect()` needs to know what to build

`.collect()` is generic over `FromIterator` — it can build a `Vec`, a
`HashMap`, a `String`, and more from the same iterator, so Rust needs a
type annotation to know which one you mean:

```rust
let v: Vec<i32> = (1..5).collect();          // annotate the binding, or...
let v = (1..5).collect::<Vec<i32>>();        // ...use the turbofish `::<>`
```

Forgetting both gives a compile error ("type annotations needed") — this
is normal and not a sign anything is wrong; just tell Rust what you're
building.

## Loop vs. chain, side by side

Here's `total_episodes` (see `src/lib.rs`) written both ways, to make the
point that an iterator chain isn't magic — it's the same loop, expressed
more declaratively:

```rust
// Manual loop with an accumulator
fn total_episodes_loop(shows: &[(String, u32)]) -> u32 {
    let mut total = 0;
    for (_, episodes) in shows {
        total += episodes;
    }
    total
}

// Iterator chain
fn total_episodes_iter(shows: &[(String, u32)]) -> u32 {
    shows.iter().map(|(_, episodes)| episodes).sum()
}
```

Both produce identical results by doing identical work. The chain version
just names the two steps ("map each show to its episode count," "sum
them") instead of spelling out the bookkeeping (declare `total`, mutate it,
return it) — once you're fluent in reading adapters, the chain communicates
*intent* faster than the loop does, without being any less explicit about
what actually happens.

## Your task

Implement the five functions in `src/lib.rs`.

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`.
