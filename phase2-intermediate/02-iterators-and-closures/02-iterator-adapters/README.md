# 2.2.2 — Iterator adapters

## At a glance

After this lesson you can:

- Explain that `Iterator` has exactly one required method — `next(&mut self)` — and that every adapter you meet in this lesson, from `map` to `flat_map`, is just a wrapper built around that one method.
- Pick the right adapter for a given need: transform with `map`, filter with `filter` or `filter_map`, bound with `take`/`skip` (or their `while` variants), or combine several iterators with `enumerate`/`zip`/`chain`.
- Say why building a chain of adapters does nothing by itself, and exactly what — a `for` loop, or `fold` itself — actually sets that chain in motion.

**Time:** ~80 minutes · **Prerequisites:** [2.2.1 — Closures and the `Fn` traits](../01-closures-and-fn-traits/README.md), and specifically [1.1.6 — `Vec` and `String` basics](../../../phase1-fundamentals/01-foundations/06-vec-and-string-basics/README.md)

---

## Why this matters

Up to now, every time you wanted to transform a list, filter it, or total something up, you reached for a manual loop:

- [1.1.5](../../../phase1-fundamentals/01-foundations/05-control-flow/README.md) wrote a manual loop that walked every element and kept the index of the first negative one.
- [2.1.1](../../01-collections/01-vec-depth/README.md) showed you `.retain(|entry| !entry.watched)` — a closure that answers "keep or go" for every element — without yet calling it "filtering."
- [2.1.2](../../01-collections/02-hashmap-in-depth/README.md) walked a `HashMap`'s pairs with `for (k, v) in map.iter()`.
- And [2.2.1](../01-closures-and-fn-traits/README.md) — the lesson right before this one — gave you the `|x| ...` vocabulary you'll see on every line from here on.

This lesson turns those scattered loops into one shared language. If you've written Python, where you'd reach for a list comprehension —

```python
uppercased = [title.upper() for title in titles if title.completed]
```

— here you reach for a **chain of iterator adapters** instead:

```rust
titles.iter().filter(|t| t.completed).map(|t| t.title.to_uppercase())
```

The similarity is real: both describe a transform and a condition over a list. But there is one essential difference, and this whole lesson turns on it: the Python line above runs eagerly, the instant it is written. The Rust line above — as you'll see all the way through this lesson — **does nothing at all**, until something actually consumes it. This is one of the few places where the Python bridge does real work and breaks at exactly that point.

---

## The concept

### Exactly one required method: `next(&mut self)`

Anything walkable in Rust satisfies, underneath everything else, one single contract: a method that, each time you call it, either hands back the next item wrapped in `Some`, or says "there is nothing left" with `None`. That contract is called **Iterator**, and this one method — in short, `next(&mut self) -> Option<Item>` — is the only thing that has to be implemented. Everything else you see in this lesson, from `map` to `fold`, is a layer built on top of that one method.

Call it by hand and see:

```rust
let scores = vec![10, 20, 30];
let mut by_hand = scores.iter();

println!("{:?}", by_hand.next());
println!("{:?}", by_hand.next());
println!("{:?}", by_hand.next());
println!("{:?}", by_hand.next());
```

```text
Some(10)
Some(20)
Some(30)
None
```

Notice `by_hand` had to be `mut`. Every call to `next()` advances the iterator's own internal position — it mutates the iterator itself — and the signature says exactly that: `&mut self`. Skip the `mut` and this is the first error this lesson gets you; the full version is in "Errors you will meet."

A `for` loop has no magic in it; it's exactly this loop, automated:

```rust
let mut manual = scores.iter();
loop {
    match manual.next() {
        Some(value) => println!("{value}"),
        None => break,
    }
}
```

```text
10
20
30
```

```senpai-visual
{"kind":"concept","labels":["v.iter()","next()","Some(item)","next()","None"]}
```

From here on, every time you see an adapter "take an item and do something," that one method above is the thing actually being called, underneath.

### Transform: `map`

`.map(closure)` hands back a new iterator that runs every item through the closure — the input and output types don't even have to match:

```rust
let titles = vec!["frieren".to_string(), "bocchi the rock!".to_string()];

for title in titles.iter().map(|t| t.to_uppercase()) {
    println!("{title}");
}
```

```text
FRIEREN
BOCCHI THE ROCK!
```

That went from `&String` to a bigger `String`. `.map()` has no opinion about types at all — it could just as easily go from `&String` to `usize`:

```rust
for length in titles.iter().map(|t| t.len()) {
    println!("length: {length}");
}
```

```text
length: 7
length: 16
```

### Filter: `filter` and `filter_map`

`.filter(predicate)` keeps only the items the closure answers `true` for:

```rust
for show in watchlist.iter().filter(|s| s.completed) {
    println!("completed: {}", show.title);
}
```

```text
completed: Frieren
completed: Mushoku Tensei
```

Now consider this: a list of user-typed episode counts that should become numbers — some of them aren't numbers at all. `.map()` alone leaves you with a `Result` per item, hit or miss:

```rust
let typed = vec!["12", "twelve", "24", "", "37"];
for parsed in typed.iter().map(|t| t.parse::<u32>()) {
    println!("{parsed:?}");
}
```

```text
Ok(12)
Err(ParseIntError { kind: InvalidDigit })
Ok(24)
Err(ParseIntError { kind: Empty })
Ok(37)
```

`.filter_map(closure)` does the same work — its closure also returns an `Option` — with one difference: every `None` silently drops out of the sequence, and what remains isn't wrapped in `Some` any more:

```rust
for n in typed.iter().filter_map(|t| t.parse::<u32>().ok()) {
    println!("{n}");
}
```

```text
12
24
37
```

`.filter_map()` is a `.map()` and a `.filter()` done in a single pass — not two separate ones.

### Bound: `take`/`take_while` and `skip`/`skip_while`

`.take(n)` and `.skip(n)` work by count — the first/remaining `n`:

```rust
let ratings = vec![9, 8, 9, 4, 7, 2];
for r in ratings.iter().take(3) {
    println!("take(3): {r}");
}
```

```text
take(3): 9
take(3): 8
take(3): 9
```

`.take_while(predicate)` and `.skip_while(predicate)` work by a condition instead of a count — and this is where you can watch laziness with your own eyes. The closure below announces every item it is asked about:

```rust
let announced = ratings.iter().take_while(|r| {
    println!("  checking {r}");
    **r >= 8
});
for r in announced {
    println!("kept: {r}");
}
```

```text
  checking 9
kept: 9
  checking 8
kept: 8
  checking 9
kept: 9
  checking 4
```

There is no further `checking` after `4` — not for `7`, not for `2`. `.take_while()` short-circuits the whole chain the moment the predicate fails; it never even bothers looking at the rest of the list. That is fundamentally different from `.filter()`, which always looks at everything. `.skip_while()` is its mirror image: it drops items while the predicate holds, then keeps everything from the first failure on — and it stops checking the predicate at all, right there:

```rust
for r in ratings.iter().skip_while(|r| **r >= 8) {
    println!("skip_while(>= 8): {r}");
}
```

```text
skip_while(>= 8): 4
skip_while(>= 8): 7
skip_while(>= 8): 2
```

### Combine several iterators: `enumerate`, `zip`, `chain`

`.enumerate()` pairs every item with its position (from 0):

```rust
for (i, title) in titles.iter().enumerate() {
    println!("{i} -> {title}");
}
```

```text
0 -> Frieren
1 -> Bocchi the Rock!
2 -> Mushoku Tensei
```

`.zip(other)` walks two iterators forward together, in pairs — and the instant either side runs out, the whole pairing stops, even if the other side still has items left:

```rust
let ratings = vec![9, 8]; // titles has three, ratings only two
for (title, r) in titles.iter().zip(ratings.iter()) {
    println!("{title} rated {r}");
}
```

```text
Frieren rated 9
Bocchi the Rock! rated 8
```

"Mushoku Tensei" never shows up — no error, no panic, it is just quietly left out because it had no partner. `.chain(other)` does something entirely different: it doesn't pair anything up, it just starts the second iterator the moment the first runs dry — one walk-through after another, with no new list built to hold both:

```rust
let already_watched = vec!["Frieren", "AOT"];
let plan_to_watch = vec!["Bocchi the Rock!", "Chainsaw Man"];
for title in already_watched.iter().chain(plan_to_watch.iter()) {
    println!("{title}");
}
```

```text
Frieren
AOT
Bocchi the Rock!
Chainsaw Man
```

### Reorder: `rev` — and why not everyone gets to

`.rev()` walks the same items back to front. But it only works on iterators that know both where the front is and where the back is — that is, alongside `next()`, they also have `next_back()`. That ability is called **`DoubleEndedIterator`**, and not every iterator has it. On a slice, both ends are well defined:

```rust
let recently_added = vec!["Frieren", "Bocchi the Rock!", "Mushoku Tensei"];
let mut both_ends = recently_added.iter();
println!("{:?}", both_ends.next());
println!("{:?}", both_ends.next_back());
println!("{:?}", both_ends.next());
```

```text
Some("Frieren")
Some("Mushoku Tensei")
Some("Bocchi the Rock!")
```

That is exactly what "double-ended" means: pull from one side, pull from the other side, and they meet in the middle. `.rev()` just packages that same idea as an adapter:

```rust
for title in recently_added.iter().rev() {
    println!("{title}");
}
```

```text
Mushoku Tensei
Bocchi the Rock!
Frieren
```

But a `HashMap` has no "front" or "back" — its iteration order isn't even guaranteed, let alone its two ends. Its iterator isn't `DoubleEndedIterator`, and `.rev()` simply doesn't compile on it; the full story is in "Errors you will meet."

### `fold` — the do-everything tool

Every adapter above handed back a new iterator. `.fold(initial, |acc, item| ...)` is different: it takes a starting value, runs that value through every item, and at the end hands back that one value — not an iterator:

```rust
let episodes = vec![28, 12, 24];
let total = episodes.iter().fold(0, |acc, n| acc + n);
println!("{total}");
```

```text
64
```

That is exactly the manual loop you'd otherwise write:

```rust
let mut total = 0;
for n in episodes.iter() {
    total += n;
}
println!("{total}");
```

```text
64
```

If you strip away every other part of this lesson, `.fold()` is the one to keep: every adapter, every consuming method you meet later, is doing this same thing underneath — holding a value, updating it with each item. `.fold()` just puts that exact pattern into a method, explicit and in your own hands. And that value doesn't have to be a number — it can be a `Vec` you build yourself:

```rust
let titles = vec!["frieren", "bocchi the rock!"];
let shouted: Vec<String> = titles.iter().fold(Vec::new(), |mut acc, t| {
    acc.push(t.to_uppercase());
    acc
});
println!("{shouted:?}");
```

```text
["FRIEREN", "BOCCHI THE ROCK!"]
```

This is exactly what you need in "Exercises" — since you don't have `.collect()` yet (that's [2.2.3](../03-consuming-and-collecting/README.md)), `.fold()` and a plain `for` loop are the only two ways today to pull a `Vec` out of a chain.

### Flatten: `flat_map`

Say every item is itself a small list. Reach for `.map()` and the result is a nested iterator — a `Vec` sitting inside each item:

```rust
for genres in watchlist.iter().map(|s| &s.genres) {
    println!("{genres:?}");
}
```

```text
["fantasy", "adventure"]
["comedy", "music"]
```

You still haven't reached the individual genres — just a list of lists. `.flat_map(closure)` does exactly those two steps — map, then flatten the result — as a single adapter:

```rust
for genre in watchlist.iter().flat_map(|s| s.genres.iter()) {
    println!("{genre}");
}
```

```text
fantasy
adventure
comedy
music
```

One flat stream, with no record left of which genre belonged to which show — and, like any other adapter, you can keep going from here: `.flat_map(...).filter(...)` is exactly as natural as everything you've seen so far.

### The point to hold onto: none of this runs early

Every adapter you saw today — `map`, `filter`, `take`, `enumerate`, `rev`, `flat_map` — only builds a new iterator that *describes* its job. None of them, by itself, touches a single item:

```rust
println!("building the pipeline...");
let pipeline = shows
    .iter()
    .map(|t| {
        println!("  map saw:    {t}");
        t.to_uppercase()
    })
    .filter(|t| {
        println!("  filter saw: {t}");
        t.len() > 8
    });
println!("pipeline built — nothing printed above.");
```

```text
building the pipeline...
pipeline built — nothing printed above.
```

Exactly as claimed: between those two lines, no `map saw`, no `filter saw`. Only once a `for` (or a consuming method, like `fold` above) actually calls `next()` does the pipeline start moving — and even then, one item at a time, not all at once:

```rust
for title in pipeline {
    println!("  got:        {title}");
}
```

```text
  map saw:    Frieren
  filter saw: FRIEREN
  map saw:    Bocchi the Rock!
  filter saw: BOCCHI THE ROCK!
  got:        BOCCHI THE ROCK!
  map saw:    Mushoku Tensei
  filter saw: MUSHOKU TENSEI
  got:        MUSHOKU TENSEI
```

The compiler agrees. Write that same chain as a bare statement — no `let`, nothing to catch what it builds — and you get this warning:

```rust
shows.iter().map(|t| t.to_uppercase()).filter(|t| t.len() > 8);
```

```text
warning: unused `Filter` that must be used
  = note: iterators are lazy and do nothing unless consumed
```

Rust names the *outermost* adapter — `Filter` is the last one applied, wrapping the `Map` inside it, so that's the type the warning is about. (Binding it to a `let` you never read, like `pipeline` above, gets a different, plainer warning instead — `unused variable` — because the value technically *was* used, just never read afterward.)

```senpai-visual
{"kind":"concept","labels":["map()","filter()","iterator, not run yet","for → next()","output"]}
```

This laziness isn't just a quirk — it's a deliberate performance decision: because nothing runs before it has to, Rust never has to build an intermediate `Vec` between each step; the whole chain, however long, runs element by element in a single pass. Exactly why that matters for performance, and how much, is the subject of [2.2.5](../05-laziness-and-performance/README.md); one paragraph is enough for now.

---

## Hands on

```sh
cargo run -p p2-02-02-iterator-adapters --example 01-next-is-the-whole-trait
cargo run -p p2-02-02-iterator-adapters --example 02-map
cargo run -p p2-02-02-iterator-adapters --example 03-filter-and-filter-map
cargo run -p p2-02-02-iterator-adapters --example 04-take-skip-and-while-variants
cargo run -p p2-02-02-iterator-adapters --example 05-enumerate-zip-chain
cargo run -p p2-02-02-iterator-adapters --example 06-rev
cargo run -p p2-02-02-iterator-adapters --example 07-fold
cargo run -p p2-02-02-iterator-adapters --example 08-flat-map
cargo run -p p2-02-02-iterator-adapters --example 09-the-pipeline-is-just-a-plan
```

Then the two broken ones:

```sh
cargo run -p p2-02-02-iterator-adapters --example 10-next-needs-mut --features broken
cargo run -p p2-02-02-iterator-adapters --example 11-rev-needs-double-ended --features broken
```

Then try these:

1. In `04-take-skip-and-while-variants`, change `ratings` so the very first item is below `8`. How many does `take_while` print?
2. In `05-enumerate-zip-chain`, add a third item to `ratings` so its length matches `titles`. Does "Mushoku Tensei" show up in the `zip` output now?
3. In `09-the-pipeline-is-just-a-plan`, swap the order of `.map()` and `.filter()`. How does the order of the `map saw`/`filter saw` lines in the output change?

---

## Errors you will meet

### `E0596` — cannot call `next` on an immutable iterator

```text
error[E0596]: cannot borrow `it` as mutable, as it is not declared as mutable
  --> phase2-intermediate\02-iterators-and-closures\02-iterator-adapters\examples\10-next-needs-mut.rs:12:22
   |
12 |     println!("{:?}", it.next());
   |                      ^^ cannot borrow as mutable
   |
help: consider changing this to be mutable
   |
11 |     let mut it = scores.iter();
   |         +++

For more information about this error, try `rustc --explain E0596`.
```

**What the compiler is actually complaining about:** look again at `next`'s signature — `fn next(&mut self) -> Option<Self::Item>`. Every call to it advances the iterator's internal position, meaning it mutates `it` itself. `it` here was declared with a plain `let`, not `let mut` — it is immutable, and the compiler won't let you call a method that needs `&mut self` on an immutable variable.

**The fix:** exactly what the compiler itself suggests:

```rust
let mut it = scores.iter();
println!("{:?}", it.next());
```

**Why this is the fix:** `it` can genuinely change now, so the `&mut self` that `next` needs can be borrowed from it. Worth noticing: a `for` loop never hits this error, because it makes the iterator it builds `mut` for you, automatically, behind the scenes — you only see this when you call `next()` by hand yourself.

### `E0277` — `.rev()` on an iterator that isn't `DoubleEndedIterator`

```text
error[E0277]: the trait bound `std::collections::hash_map::Iter<'_, &str, u32>: DoubleEndedIterator` is not satisfied
    --> phase2-intermediate\02-iterators-and-closures\02-iterator-adapters\examples\11-rev-needs-double-ended.rs:16:35
     |
  16 |     let _reversed = scores.iter().rev();
     |                                   ^^^ the trait `DoubleEndedIterator` is not implemented for `std::collections::hash_map::Iter<'_, &str, u32>`
     |
note: required by a bound in `rev`
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\iter\traits\iterator.rs:3445:23
     |
3443 |     fn rev(self) -> Rev<Self>
     |        --- required by a bound in this associated function
3444 |     where
3445 |         Self: Sized + DoubleEndedIterator,
     |                       ^^^^^^^^^^^^^^^^^^^ required by this bound in `Iterator::rev`

For more information about this error, try `rustc --explain E0277`.
```

**What the compiler is actually complaining about:** `rev` itself carries this bound — `Self: Sized + DoubleEndedIterator` — and `scores.iter()` here has the type `std::collections::hash_map::Iter`, which doesn't implement that trait. The reason isn't technical, it's logical: `HashMap` makes no promise at all about iteration order, so it has neither a "front" nor a "back" for `.rev()` to work from.

**The fix:** if you genuinely need a reversible order, gather the keys or values into a `Vec` first (with a plain `for`, no `.collect()`), and walk that `Vec` instead:

```rust
let mut titles: Vec<&str> = Vec::new();
for title in scores.keys() {
    titles.push(title);
}
for title in titles.iter().rev() {
    println!("{title}");
}
```

**Why this is the fix:** a `Vec` has a real, stable order — the one you pushed it into — so its iterator is `DoubleEndedIterator` and `.rev()` makes sense on it. Don't reach for this trick by default: if you're just walking a `HashMap` with a `for` and don't care about order, plain `.iter()` is all you need.

---

## Exercises

### Warm up

<details>
<summary>You have a <code>Vec&lt;i32&gt;</code> with three items, and you call <code>.iter().next()</code> twice, separately (each time on a fresh iterator). What does each call print?</summary>

Write down your answer before reading on.

</details>

<details>
<summary>Answer</summary>

Both print `Some(<the first item>)` — because each time you built a **brand-new** `.iter()`, not continued the previous one.

</details>

<details>
<summary>Does <code>let v = vec![1, 2, 3]; let it = v.iter(); it.next();</code> compile?</summary>

Write down your answer before reading "Errors you will meet."

</details>

<details>
<summary>Answer</summary>

No — `E0596`. `next(&mut self)` needs a `mut` iterator, and `it` was declared with a plain `let`.

</details>

<details>
<summary>How many items does <code>vec![1, 2, 8, 3, 4].iter().take_while(|n| **n &lt; 5)</code> return?</summary>

Write down your answer — and remember how `take_while` actually works.

</details>

<details>
<summary>Answer</summary>

Just two: `1` and `2`. The moment it reaches `8` (which fails the predicate), `take_while` stops — it doesn't even look at `3` and `4`, even though they themselves are below 5.

</details>

<details>
<summary>How many pairs does <code>vec![1, 2, 3].iter().zip(vec!["a", "b"].iter())</code> produce?</summary>

Write down your answer.

</details>

<details>
<summary>Answer</summary>

Just two: `(1, "a")` and `(2, "b")`. `.zip()` stops the instant the shorter side runs out; `3` never gets paired.

</details>

<details>
<summary>You have a <code>HashMap&lt;&amp;str, u32&gt;</code>. Does <code>map.iter().rev()</code> compile?</summary>

Write down your answer before reading "Errors you will meet."

</details>

<details>
<summary>Answer</summary>

No — `E0277`. `HashMap`'s iterator has no guaranteed order, so it isn't `DoubleEndedIterator`, and `.rev()` isn't defined on it.

</details>

<details>
<summary>What does <code>vec![1, 2, 3, 4].iter().fold(1, |acc, n| acc * n)</code> return?</summary>

Work it out by hand.

</details>

<details>
<summary>Answer</summary>

```text
24
```

Starting from `acc = 1`: `1×1=1`, then `1×2=2`, then `2×3=6`, then `6×4=24` — the running product of every item.

</details>

### Repair

Fix both broken examples:

1. Fix `examples/10-next-needs-mut.rs` so it compiles — by changing only how `it` is declared, nothing else.
2. Fix `examples/11-rev-needs-double-ended.rs` so you can genuinely print the `HashMap`'s pairs in reverse — by gathering them into a `Vec` (with a plain `for`) before calling `.rev()`.

### Implement

Five functions in `src/lib.rs`, over a `struct Show { title: String, episodes: u32, completed: bool }`:

```sh
cargo test -p p2-02-02-iterator-adapters
```

No `.collect()` — that's [2.2.3](../03-consuming-and-collecting/README.md). To build each `Vec`, use a plain `for` loop or `.fold()` — exactly the two ways you saw in "The concept." Each function's doc comment states exactly what it returns; don't guess.

### Build

Write a `pub fn watchlist_summary(shows: &[Show]) -> String` that produces a one-line summary of a watchlist — how many shows, how many are `completed`, what the total `episodes` adds up to — in a format you choose and state exactly in the doc comment.

Then write a second function, `pub fn titles_and_ratings(titles: &[String], ratings: &[u32]) -> Vec<String>`, that pairs each title with its matching rating (say, as `"Frieren - 9"`) — with `.zip()`, not manual indexing. If the lengths differ, state in the doc comment exactly how many items end up in the result, and why.

### Challenge (optional)

Write a `pub fn newest_and_oldest(shows: &[Show], n: usize) -> Vec<String>` that puts the first `n` shows (newest, in their original order) together with the last `n` shows (oldest — also in their own original order, not reversed) — using only `.take()`, `.rev()` and `.chain()`, no manual indexing. (Hint: "the last `n`, but in original order" is exactly what a `.rev()`, a `.take(n)`, and another `.rev()`, chained together, give you.) If `n` is bigger than half the list, what do you decide to do? State it in the doc comment.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `next(&mut self)` | `Iterator`'s only required method; the next item, or `None` | Everything in this lesson is built on this |
| iterator adapter | A method that returns a new iterator and does nothing by itself | `.map()`, `.filter()`, `.take()`, … |
| `.map()` | Transform every item, one at a time | Changing a type or a value |
| `.filter()` / `.filter_map()` | Keep the approved items / map + filter in one pass | Dropping unwanted items, or parsing something that might fail |
| `.take()`/`.take_while()`, `.skip()`/`.skip_while()` | Bound the pipeline, by count or by condition | Taking just the first items, or dropping the first items |
| `.enumerate()`/`.zip()`/`.chain()` | Pair with position, pair up two iterators, run one after another | Ranking, combining two parallel lists, joining two lists |
| `.rev()` | Walk from last to first; needs `DoubleEndedIterator` | Only when the iterator knows where "the back" is |
| `.fold()` | The general-purpose tool for reducing to one value | Summing, building a `Vec` by hand, any other combination |
| `.flat_map()` | Map, then flatten the result, in one step | Turning each item into several items |
| laziness | An adapter does nothing by itself | Until something — a `for`, or a consuming method — consumes it |

### What you now know

- `Iterator` has exactly one required method — `next(&mut self) -> Option<Item>` — and every adapter, and every `for` loop, is underneath just calling that one method repeatedly.
- `.map()` transforms; `.filter()` keeps or drops; `.filter_map()` does both in one pass and silently drops the `None`s.
- `.take`/`.skip` work by count, `.take_while`/`.skip_while` work by condition — and the `while` versions short-circuit the moment the answer turns negative.
- `.enumerate()` adds position, `.zip()` stops at the shorter side, `.chain()` runs two iterators back to back without building a new list.
- `.rev()` only works on `DoubleEndedIterator`s — an iterator with both `next()` and `next_back()`; `HashMap` doesn't have this.
- `.fold()` can reduce any combination down to a single value, including building a `Vec` by hand.
- None of this runs before it has to — an adapter only describes a pipeline; only something that actually calls `next()` (a `for` loop, or a consuming method) sets it moving.

### What comes back later

- **`.collect()` and the family of consuming methods — `.sum()`, `.count()`, `.for_each()`, and `Result<Vec<_>, E>`** — [2.2.3 — Consuming and collecting](../03-consuming-and-collecting/README.md)
- **Implementing `Iterator` and `IntoIterator` for your own type** — [2.2.4](../04-implementing-iterator/README.md)
- **Why laziness matters for performance, and exactly how much** — [2.2.5 — Laziness and performance](../05-laziness-and-performance/README.md)

### Can you explain?

- Why do we describe `Iterator` as having "one required method," when there are this many other methods (`.map()`, `.filter()`, `.fold()`, …) on it?
- What does `.filter_map()` do that `.map()` alone cannot?
- Why can `.take_while()` avoid even looking at the rest of a long list, while `.filter()` cannot?
- What does `.zip()` do with two iterators of different lengths?
- Why does `.rev()` work on a `Vec` but not on a `HashMap`?
- If you build a `.map().filter()` chain and never consume it, what exactly happens?

---

## Going further

- [`std::iter::Iterator` documentation](https://doc.rust-lang.org/std/iter/trait.Iterator.html) — the full method list; today you only saw part of it.
- [The Rust Book — Processing a Series of Items with Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html) — the same ground, official, with more detail on how the trait is defined.
- [`DoubleEndedIterator` documentation](https://doc.rust-lang.org/std/iter/trait.DoubleEndedIterator.html) — the trait `.rev()` relies on.
- [the `itertools` crate](https://docs.rs/itertools) — for the day these standard adapters aren't enough, this crate adds dozens more; outside this lesson, just good to know it exists.
