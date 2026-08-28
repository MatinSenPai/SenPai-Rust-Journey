# 2.1.4 — Choosing a collection: a complexity table and when each wins

## At a glance

After this lesson you can:

- Look at a real access pattern — not a name — and say, without guessing, which of `Vec`, `HashMap`, `BTreeMap`, `HashSet`, `VecDeque`, and `BinaryHeap` fits it, and which ones flatly don't.
- Explain why a wrong choice still compiles, still runs, and still gives the right answer — and only quietly spends the `O(n)` that an `O(1)` would have given you for free.
- Defend your choice out loud: which axis decided it, and which axis breaks first if one requirement changes.

**Time:** ~70 minutes · **Prerequisites:**
[2.1.1 — `Vec` and `HashMap`](../01-vec-depth/README.md),
[2.1.2 — `HashMap` in depth: the `entry` API, hashers, `&str` lookup](../02-hashmap-in-depth/README.md),
[2.1.3 — `BTreeMap`, `HashSet`, `VecDeque`, `BinaryHeap`](../03-btreemap-hashset-vecdeque/README.md)

---

## Why this matters

The last three lessons handed you six tools, one at a time: `Vec`'s internal mechanics, the entry-API trick on `HashMap`, and then four more, each solving one specific problem. Every one of those lessons asked the same question: "how does this structure work?" This lesson asks a completely different one: **when real code is in front of you and nobody wrote the type's name for you, which one do you pick?**

Here's the dangerous part: picking wrong almost never produces a compiler error. Reach for a `Vec` instead of a `HashSet` to check membership, and the code compiles, runs, and gives the right answer — every check just walks the whole list linearly instead of taking one instant look. With ten items you never feel it. With ten million, your service falls over and there is no error on screen explaining why. This is exactly the question technical interviews ask with "which data structure?", and exactly what someone writes under a line in code review: "why is this a `Vec`?"

This lesson teaches no new syntax — every method you see below, you already know from the last three lessons. Its job is to put the six of them side by side and ask you to decide, once, with one glance — instead of guessing all over again every time.

---

## The concept

### One question, not six syntaxes

All six of these share one contract: they take a value, they give a value back. What separates them is which question they answer "cheaply" and which one they answer "expensively" — or don't answer at all. There are five axes you should ask through every time, and the order matters: each one usually rules out some of the options, until one is left.

```senpai-visual
{"kind":"concept","labels":["access pattern","key or index?","order matters?","where you add/remove","the right collection"]}
```

### Axis 1 — do you look something up by key, or just walk it?

If your question has the shape "what's the value for this key?" or "does this value exist at all?", `HashMap`/`HashSet` answer it in average `O(1)` — one hash, one direct jump to the right bucket.

```rust
use std::collections::HashMap;

let watch_counts: HashMap<&str, u32> = HashMap::from([
    ("Frieren", 12),
    ("Bocchi", 7),
]);

println!("{:?}", watch_counts.get("Frieren"));
println!("{:?}", watch_counts.get("Naruto"));
```

```text
Some(12)
None
```

A `Vec` has no concept of a "key" at all. Ask the same question of a `Vec<(&str, u32)>` and your only option is a manual loop that checks each entry one by one — exactly what you'd have written in Phase 1 for a plain list:

```rust
let shows = vec![("Frieren", 12), ("Bocchi", 7)];
let mut found = None;
for (title, count) in &shows {
    if *title == "Frieren" {
        found = Some(*count);
    }
}
println!("{found:?}");
```

```text
Some(12)
```

It gives the right answer — but to be sure "Naruto" isn't there, it has to look at every single one of ten thousand entries first. (If you truly have to stay with a `Vec`, keeping it sorted and using `.binary_search()` from 2.1.1 gets this down to `O(log n)` — but every insert now costs `O(n)` itself, since it has to open a gap. Neither `HashMap` nor `BTreeMap` makes that trade.)

### Axis 2 — do you look something up by index?

"What's the second element?" only means something for `Vec` and `VecDeque` — both answer indexing in `O(1)`, because both are a contiguous buffer under the hood.

```rust
let lineup: Vec<&str> = vec!["Frieren", "Bocchi", "Naruto"];
println!("first:   {:?}", lineup.first());
println!("index 1: {:?}", lineup.get(1));
```

```text
first:   Some("Frieren")
index 1: Some("Bocchi")
```

None of the other four have a "second" at all — a `HashMap` or `BTreeMap` has a key, not a position; a `HashSet` has members; a `BinaryHeap` only knows "the largest." This axis is usually the fastest way to cut the field: if you genuinely need a position, only two of the six are still standing.

### Axis 3 — what order does iteration come out in?

You know from 2.1.2 that `HashMap` makes no promise about order at all. `BTreeMap` is the exact opposite: iteration always comes out sorted by key, ascending, and **free** — with no collecting or sorting on your part:

```rust
use std::collections::BTreeMap;

let mut by_day: BTreeMap<u32, u32> = BTreeMap::new();
by_day.insert(3, 40);
by_day.insert(1, 12);
by_day.insert(2, 25);
for (day, count) in &by_day {
    println!("day {day}: {count}");
}
```

```text
day 1: 12
day 2: 25
day 3: 40
```

`BinaryHeap` is a strange third case: the only thing it guarantees is that *whatever you pop next* is the largest value still inside it. That's the whole job description of a **priority queue** — a structure that cheaply gives you only "the current largest (or smallest)," and promises nothing about anything else — and it's exactly what `BinaryHeap` is. Plain iteration (`for x in &heap`) has no order at all — it's the heap's internal tree order, not sorted order:

```rust
use std::collections::BinaryHeap;

let ratings: BinaryHeap<u32> = BinaryHeap::from([3, 1, 4, 1, 5, 9, 2, 6]);
println!("peek: {:?}", ratings.peek());
let mut still_sorted = true;
let mut previous = u32::MAX;
for value in &ratings {
    still_sorted &= *value <= previous;
    previous = *value;
}
println!("iter() order was sorted: {still_sorted}");
```

```text
peek: Some(9)
iter() order was sorted: false
```

`peek()` correctly says 9 is the largest. But plain iteration is not sorted — this is the exact trap almost everyone falls into the first time: it's called a "heap," so they assume iterating it is sorted too. It isn't. Only successive `pop()`s are.

### Axis 4 — where do you add and remove?

One end, both ends, or only "the largest"? Three completely different answers:

```rust
use std::collections::VecDeque;

let mut recent: VecDeque<&str> = VecDeque::new();
recent.push_back("ep1 released");
recent.push_back("ep2 released");
recent.push_front("server maintenance");
println!("{recent:?}");
println!("oldest: {:?}", recent.pop_front());
```

```text
["server maintenance", "ep1 released", "ep2 released"]
oldest: Some("server maintenance")
```

`Vec` is only cheap at its **back** — `.push()`/`.pop()` in amortized `O(1)`; its front is `O(n)` because everyone else has to shift over by one (2.1.3 showed you exactly this with an `E0599` when you called `.pop_front()` — that method simply doesn't exist). `VecDeque` gives both ends amortized `O(1)`, because underneath it's a ring buffer, not a plain buffer. `BinaryHeap` has no "ends" at all — only `push` (wherever it needs to go, `O(log n)`) and pop-the-largest (also `O(log n)`), plus a free `peek()` for just looking.

`HashMap`/`BTreeMap`/`HashSet` have no "ends" either — inserting and removing happen by **key**, not by position.

### Axis 5 — memory overhead

This one can't be shown with a `println!`, but it's just as real: `Vec` and `VecDeque` hold only their own elements, back to back, with no extra bytes per element. `BinaryHeap` is the same — underneath, it *is* a `Vec`.

`HashMap`/`HashSet` deliberately never fill up: to keep lookup at `O(1)`, they always keep some slots empty (the load factor) — meaning the memory reserved is always more than the memory actually used. `BTreeMap` carries a different kind of overhead: each tree node holds several entries and several pointers — less waste than a hash table, but still more than a raw array. If memory is genuinely tight, this axis matters too — but it's usually the last one to change the decision, not the first.

### The decision table

All five axes, in one place:

| Collection | Ordered iteration? | Lookup by key? | Lookup by index? | push/pop front | push/pop back | Memory overhead | Reach for this when... |
|---|---|---|---|---|---|---|---|
| `Vec<T>` | insertion order | no (linear, unless sorted + `binary_search`) | `O(1)` | `O(n)` | `O(1)` amortized | lowest — contiguous | order is exactly insertion order; you mostly work by index or at the back |
| `VecDeque<T>` | insertion order | no | `O(1)` | `O(1)` amortized | `O(1)` amortized | close to `Vec` | you need cheap add/remove at **both** ends — a queue, a sliding window, "last N" |
| `HashMap<K, V>` | none (randomized per run) | `O(1)` average | no | key-addressed, not end-addressed | key-addressed, not end-addressed | higher — deliberate empty slots | you look things up by key and don't care about order |
| `BTreeMap<K, V>` | always sorted by key | `O(log n)` | no | key-addressed | key-addressed | moderate — node overhead | you need both sorted-by-key order and range queries, and `O(log n)` is fine |
| `HashSet<T>` | none | `O(1)` average (membership only) | no | member-addressed | member-addressed | like `HashMap` | you only care whether something is present, not a value attached to it |
| `BinaryHeap<T>` | no guarantee (only `pop` gives the largest) | no | no | none | `push` anywhere `O(log n)`; largest-`pop` `O(log n)`; `peek` `O(1)` | like `Vec` | you repeatedly need "the current largest," not arbitrary lookup |

### A real decision, step by step

Let's walk this table through a real question: "how many distinct genres were seen today?"

- Axis 1: the question has the shape "has this genre been seen before?" — a membership check, not a value. `HashMap` is out (no *value* is needed per key), `HashSet` stays.
- Axis 2: no index is needed — `Vec`/`VecDeque` are out too.
- Axis 3: the order of the genres doesn't matter — `BTreeMap` wouldn't add anything either, just extra cost.
- Axis 4: neither front, nor back, nor "the largest" matters — `BinaryHeap` is out too.

Only `HashSet` is left:

```rust
use std::collections::HashSet;

let genres_seen = ["isekai", "comedy", "isekai", "drama", "comedy"];

let mut distinct: HashSet<&str> = HashSet::new();
for genre in genres_seen {
    distinct.insert(genre);
}

println!("genres logged today: {}", genres_seen.len());
println!("distinct genres:     {}", distinct.len());
```

```text
genres logged today: 5
distinct genres:     3
```

Those four lines of reasoning are exactly what you're about to write yourself in the exercises below — just for three different scenarios.

---

## Hands on

```sh
cargo run -p p2-01-04-choosing-a-collection --example 01-lookup-by-key-vs-scan
cargo run -p p2-01-04-choosing-a-collection --example 02-lookup-by-index
cargo run -p p2-01-04-choosing-a-collection --example 03-iteration-order
cargo run -p p2-01-04-choosing-a-collection --example 04-ends-and-priority
cargo run -p p2-01-04-choosing-a-collection --example 05-the-decision-in-action
```

Then the three broken ones:

```sh
cargo run -p p2-01-04-choosing-a-collection --example 06-hashmap-is-not-positional --features broken
cargo run -p p2-01-04-choosing-a-collection --example 07-hashmap-has-no-sort --features broken
cargo run -p p2-01-04-choosing-a-collection --example 08-binaryheap-has-no-remove --features broken
```

Then try this:

1. In `03-iteration-order.rs`, pop that same `BinaryHeap` down to empty with `while let Some(top) = ratings.pop() { ... }` and print it. Is that order sorted? Why is this one sorted when plain iteration wasn't?
2. In `04-ends-and-priority.rs`, instead of `push_back`/`push_front` on `recent`, build a plain `Vec` and try the same sequence with `.insert(0, ...)`. For a hundred thousand events, which one actually gets slower?
3. In `05-the-decision-in-action.rs`, suppose that alongside the distinct-genre count you now also need to say which genre repeated the most. Walk the decision table again — is `HashSet` still enough?

---

## Errors you will meet

### `E0308` — indexing a `HashMap` with a number

```text
error[E0308]: mismatched types
  --> phase2-intermediate\01-collections\04-choosing-a-collection\examples\06-hashmap-is-not-positional.rs:14:24
   |
14 |     let first = counts[0];
   |                        ^ expected `&_`, found integer
   |
   = note: expected reference `&_`
                   found type `{integer}`
help: consider borrowing here
   |
14 |     let first = counts[&0];
   |                        +

For more information about this error, try `rustc --explain E0308`.
error: could not compile `p2-01-04-choosing-a-collection` (example "06-hashmap-is-not-positional") due to 1 previous error
```

**What the compiler is actually objecting to:** `counts[0]` reads like "give me the first entry" — the `Vec` habit. But `Index` for `HashMap<K, V>` is defined over `&Q`, not over `K` itself — meaning whatever you put inside the brackets has to already be a *reference to a key*, not a bare number. That's exactly what the compiler says: it expected `&_`, and got a plain integer.

**The fix:** index with an actual key, or better, use `.get(...)`, which returns an `Option` instead of panicking:

```rust
let first = counts.get("Frieren");
```

**Why this is the fix:** the compiler's own suggestion (`counts[&0]`) only solves half the problem — the key type is still `String`, not `i32`, so that same line would fail again with a different type error. The real problem was never that you forgot a `&`; it's that "position zero" doesn't mean anything for a `HashMap` at all. The only real fix is asking with an actual key.

### `E0599` — a `HashMap` has no `.sort()`

```text
error[E0599]: no method named `sort` found for struct `HashMap<K, V, S, A>` in the current scope
  --> phase2-intermediate\01-collections\04-choosing-a-collection\examples\07-hashmap-has-no-sort.rs:15:12
   |
15 |     counts.sort();
   |            ^^^^ method not found in `HashMap<&str, u32>`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `p2-01-04-choosing-a-collection` (example "07-hashmap-has-no-sort") due to 1 previous error
```

**What the compiler is actually objecting to:** `.sort()` belongs to `Vec`/slices, not `HashMap`. This is wanting order after already choosing a structure that never promised any — Axis 3, from the wrong direction.

**The fix:** either choose `BTreeMap` from the start (order comes free), or, if you genuinely need `HashMap`, explicitly collect the pairs into a `Vec` and sort them yourself:

```rust
let mut by_count: Vec<(&str, u32)> = Vec::new();
for (title, count) in counts {
    by_count.push((title, count));
}
by_count.sort_by_key(|(_, count)| *count);
```

**Why this is the fix:** you cannot sort an unordered structure in place — order has to come from somewhere. Either you pick an already-sorted structure up front (`BTreeMap`, order for free), or you deliberately accept one extra step (collect, then `.sort_by_key`) to arrive at that same order. Phase 2.2 writes this same collecting step in one line; this manual loop does the same job today.

### `E0599` — a `BinaryHeap` has no `.remove()`

```text
error[E0599]: no method named `remove` found for struct `BinaryHeap<T, A>` in the current scope
  --> phase2-intermediate\01-collections\04-choosing-a-collection\examples\08-binaryheap-has-no-remove.rs:16:12
   |
16 |     scores.remove(&(10, "Yui"));
   |            ^^^^^^ method not found in `BinaryHeap<(u32, &str)>`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `p2-01-04-choosing-a-collection` (example "08-binaryheap-has-no-remove") due to 1 previous error
```

**What the compiler is actually objecting to:** `HashSet` can find and remove an arbitrary member with `.remove(&value)` — it's reasonable to assume `BinaryHeap` has the same. It doesn't. `BinaryHeap` is optimized for exactly one question: "what's the largest?" — finding one arbitrary, *non*-largest value somewhere in the middle of the tree is something it was never built to do.

**The fix:** if you genuinely need arbitrary removal, `BinaryHeap` was not the right choice — go back to Axis 4: something that needs both "give me the largest" and "remove whichever one I want" needs a different structure, or a combination of two.

**Why this is the fix:** this error isn't a code mistake — it's a decision mistake. `BinaryHeap` was built for exactly this trade: in exchange for making every other operation impossible, it keeps "give me the largest" as cheap as it can possibly be.

---

## Exercises

### Warm up

<details>
<summary>You only need to know "have I seen this ID before?" — not its value, not its order. Which of the six, and why?</summary>

`HashSet`. The question is pure membership — no attached value, no order needed — exactly Axis 1, in its simplest form.

</details>

<details>
<summary>You're building a live leaderboard: scores update constantly, and you need to say who's currently in the lead right now. Which one tempts you, and what exactly does it not give you for free?</summary>

`BinaryHeap` is tempting because "give me the largest" is literally its job. But updating one specific person's score (not adding a fresh entry) is not free on a `BinaryHeap` — there's no direct way to "find this item and change its value." That exact tension is what the Build section is about.

</details>

<details>
<summary>What does this print?</summary>

```rust
use std::collections::BTreeMap;
let mut m = BTreeMap::new();
m.insert(5, "five");
m.insert(1, "one");
m.insert(3, "three");
for (k, _) in &m {
    print!("{k} ");
}
```

</details>

<details>
<summary>Answer</summary>

```text
1 3 5
```

`BTreeMap` always iterates ascending by key — it doesn't care what order `.insert()` was called in.

</details>

<details>
<summary>Does this compile?</summary>

```rust
let mut v: Vec<u32> = vec![1, 2, 3];
let first = v.pop_front();
```

</details>

<details>
<summary>Answer</summary>

No. `Vec` has no method called `pop_front` at all — `E0599`. If you genuinely need cheap removal from the front, you want `VecDeque`.

</details>

<details>
<summary>What does this print?</summary>

```rust
use std::collections::BinaryHeap;
let mut h = BinaryHeap::from([2, 8, 5]);
println!("{:?}", h.pop());
println!("{:?}", h.pop());
```

</details>

<details>
<summary>Answer</summary>

```text
Some(8)
Some(5)
```

Each `pop()` returns the largest value still left — not insertion order, not the tree's internal order.

</details>

<details>
<summary>Both <code>Vec&lt;T&gt;</code> and <code>VecDeque&lt;T&gt;</code> are amortized <code>O(1)</code> at the back. So when would you ever actually reach for <code>VecDeque</code>?</summary>

When you also need the **front** to be cheap, not just the back — a queue, a sliding window, "the last N events." `Vec`'s front is `O(n)`; `VecDeque`'s is amortized `O(1)` at both ends. If you only ever work at the back, `Vec` is simpler and enough.

</details>

### Repair

Fix all three broken examples — not with a syntax trick, but by changing the decision:

1. Fix `examples/06-hashmap-is-not-positional.rs` so it compiles, by asking with an actual key instead of a number.
2. Fix `examples/07-hashmap-has-no-sort.rs` **two** ways: once by keeping `counts` as a `HashMap` and explicitly collecting/sorting the pairs, once by changing the type itself to `BTreeMap` from the start. Which one ends up with less code?
3. Rewrite `examples/08-binaryheap-has-no-remove.rs` so it actually achieves its goal — removing "Yui"'s score — without `.remove()`. (Hint: what does `BinaryHeap` actually give you for free? Popping everything and keeping everything except the one you don't want is one way; is there a better one?)

### Implement

Three functions/structs in `src/lib.rs`, each a different axis:

```sh
cargo test -p p2-01-04-choosing-a-collection
```

None of the signatures name a collection at all — that decision is yours, exactly the way you justified it in Warm up:

- `unique_viewer_count` — the same Axis-1 question you saw in "A real decision," this time with viewer IDs instead of genres.
- `RecentEvents` — a tracker that always holds only the most recent `capacity` events, oldest dropped first. Axis 4: which of the six is cheap at both ends?
- `daily_report` — a report that both has to replace a repeated key with its latest value, and come out sorted by day at the end. Which one gives you both in a single structure?

The exact specification — including the literal string format `daily_report` returns — is in the doc comment above each function.

### Build

A small `Leaderboard` for whatever show is currently airing: viewers submit a score (0–100) under their name; at any moment you need to say "who's in the lead right now?" It should support at least:

- Recording a fresh score for a name (if that name already scored, replace it, don't add to it).
- Reporting the current leader's name and score.

None of the six gives you both of these for free — `HashMap` gives `O(1)` update-by-name but `O(n)` finding the leader; `BinaryHeap` gives `O(1)` finding the leader (that's exactly what `.peek()` is for) but has no update-by-name at all (remember the Errors section). Pick one — or combine two — and write one comment saying why, and exactly what you'd have given up with the other choice.

### Challenge (optional)

Now make it harder: viewers can cancel their subscription (their score is wiped entirely), and instead of just the leader, you now need to report the top three — every time someone asks, not just once at the end.

Try the `Leaderboard` you wrote in Build against these two new demands. Does the same choice still hold up? If you need both an arbitrary removal (`remove_by_name`) and a "top three" report to be cheap, which of the six — or which combination — wins now? A sentence or two is enough; you don't need to write the full code.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| decision axis | a question about access pattern that narrows the field | every time you're choosing between collections |
| priority queue | a structure that cheaply gives only "the current largest/smallest" | exactly what `BinaryHeap` is |
| ring buffer | a buffer that wraps around instead of shifting elements | what makes `VecDeque` `O(1)` at both ends |
| load factor | the ratio of filled slots to total slots in a hash table | why `HashMap`/`HashSet` always carry extra overhead |

### What you now know

- Five axes — key, index, iteration order, which ends are cheap, memory overhead — and which of the six answers each one cheaply.
- A wrong choice almost never produces a compiler error; it just adds invisible complexity you don't feel until your data grows.
- `BinaryHeap` states its trade plainly: in exchange for making every other operation impossible, it keeps "give me the largest" as cheap as it can be.
- No collection wins all five axes at once — sometimes none of the six gives you both requirements for free, and then you either trade something off or combine two structures.
- None of this lesson's three functions named a collection in their signature — that's exactly where the decision was genuinely yours.

### What comes back later

- **Iterator adapters and `.collect()`** — every manual loop you wrote in this lesson shrinks to one chained expression there — [Phase 2 — Iterator adapters](../../02-iterators-and-closures/02-iterator-adapters/README.md) and [Consuming and collecting](../../02-iterators-and-closures/03-consuming-and-collecting/README.md).
- **`Ord`/`Hash`/`PartialOrd` for your own type** — every example in this lesson used numbers and tuples; putting a struct you designed yourself inside a `BinaryHeap`, or using it as a `HashMap`/`BTreeMap` key, needs these — [Phase 2 — Standard derives by hand](../../03-traits-and-generics/04-standard-derives-by-hand/README.md).
- **Using any of these six from more than one thread at once** — none of them is safe for that on its own — [Phase 2 — Threads, `Mutex`, and `Arc`](../../08-concurrency/01-threads-mutex-arc/README.md).
- **Actually measuring performance, not just reasoning from theoretical complexity** — this lesson only ever argued from Big-O; measuring real speed on real data is a separate skill — [Phase 2 — Benchmarking with criterion](../../07-project-structure-and-testing/05-benchmarking-with-criterion/README.md).

### Can you explain?

- For a hypothetical scenario, walk through the five axes one at a time and say which options each one rules out.
- Why does *choosing the wrong collection* almost never produce a compiler error, and what danger does that harmlessness hide?
- Why does `BinaryHeap` have neither `.remove()` nor indexing, and what does that absence buy?
- Say why none of the three "Implement" exercise signatures named a collection, and that this was deliberate.
- For the "Build" scenario, explain why none of the six gave you both behaviors for free on its own.

---

## Going further

- [The Rust Book — Chapter 8 (Collections)](https://doc.rust-lang.org/book/ch08-00-common-collections.html) — this same table, from the Rust team itself.
- [`std::collections` — the official choosing guide](https://doc.rust-lang.org/std/collections/index.html) — an official decision table on exactly this topic; it reads oddly familiar after this lesson.
- [`std::collections::BinaryHeap`](https://doc.rust-lang.org/std/collections/struct.BinaryHeap.html) — its full method list, complexity included.
- [Big-O cheat sheet](https://www.bigocheatsheet.com/) — the same complexity table you saw in this lesson, for far more data structures than these six.
