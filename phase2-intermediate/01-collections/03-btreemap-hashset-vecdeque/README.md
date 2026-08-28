# 2.1.3 — `BTreeMap`, `HashSet`, `VecDeque`, `BinaryHeap`

## At a glance

After this lesson you can:

- Choose between `BTreeMap` and `HashMap` for a real piece of code, and defend the choice with a specific complexity trade-off — not a guess.
- Reach for `.range()` on a `BTreeMap`, the set-algebra methods on a `HashSet`, both ends of a `VecDeque`, and `.pop()` on a `BinaryHeap` for exactly the job each one is built for.
- Turn a max-heap into a min-heap with `Reverse<T>`, and read and fix the four errors these four collections invite most often.

**Time:** ~90 minutes · **Prerequisites:**
[2.1.2 — `HashMap` in depth: the `entry` API, hashers, `&str` lookup](../02-hashmap-in-depth/README.md)

---

## Why this matters

So far you have two shapes of data: `Vec` (a sequence) and `HashMap` (unordered key lookup, average `O(1)`). Those two cover a lot of ground — but not everything, and where they fall short, the gap doesn't show up as a compiler error. It shows up as a silent bug.

Three concrete cases: if a report or a test needs to show the exact same order every time, `HashMap` never promises that — [2.1.2](../02-hashmap-in-depth/README.md) already told you why. If you're building a queue that needs to both add to the back and occasionally remove from the front, `Vec` does one of those two jobs in `O(n)` — with no error at all, just a program that gets slower as the queue grows. And if you keep asking "what's the most important thing right now?" — a support queue, a job scheduler — re-sorting the whole list every time a new item arrives is wasted work.

This lesson gives you four tools that solve exactly those three problems — each with a specific trade-off, never a free lunch. Choosing correctly between them is the skill this lesson teaches, and the next lesson ([2.1.4](../04-choosing-a-collection/README.md)) turns it into a full decision table.

---

## The concept

### `BTreeMap<K, V>` — the same shape as `HashMap`, sorted

`BTreeMap` takes almost the same API as `HashMap` — `.insert()`, `.get()`, even the entry API you learned in [2.1.2](../02-hashmap-in-depth/README.md) works exactly the same way — with one difference: it always keeps its keys sorted. That means iterating a `BTreeMap` always yields entries by key, ascending. Put the same three inserts into a `HashMap`, then into a `BTreeMap`:

```rust
use std::collections::{BTreeMap, HashMap};

let mut counts: HashMap<String, u32> = HashMap::new();
counts.insert("naruto".to_string(), 4);
counts.insert("bleach".to_string(), 2);
counts.insert("frieren".to_string(), 9);

let mut sorted: BTreeMap<String, u32> = BTreeMap::new();
for (title, count) in counts {
    sorted.insert(title, count);
}
for (title, count) in &sorted {
    println!("{title}: {count}");
}
```

```text
bleach: 2
frieren: 9
naruto: 4
```

Notice the order the entries went into `counts` never mattered — whether `naruto` went in first or last, iterating `sorted` always gives you exactly these three lines. And the entry API you already know still works, unchanged:

```rust
*sorted.entry("bocchi".to_string()).or_insert(0) += 1;
*sorted.entry("bleach".to_string()).or_insert(0) += 1;
for (title, count) in &sorted {
    println!("{title}: {count}");
}
```

```text
bleach: 3
bocchi: 1
frieren: 9
naruto: 4
```

That determinism isn't free. `HashMap` finds a key by hashing it straight to a bucket — average `O(1)`. `BTreeMap` walks a tree, level by level — `O(log n)`. For a few hundred or a few thousand entries you will rarely feel the difference; it's still a real one, which is why the rule of thumb is: reach for `HashMap` by default, and switch to `BTreeMap` only when you specifically need sorted iteration — a report, a test, or a range query, which is next.

### `.range()` — range queries, free once you're sorted

Because `BTreeMap` keeps its keys sorted, it answers a question `HashMap` can never answer cheaply: "give me everything whose key is between X and Y." With `HashMap` you'd have to check every single entry — `O(n)`. With `BTreeMap`, `.range()` goes straight to that slice of the tree:

```rust
let mut releases: BTreeMap<u32, &str> = BTreeMap::new();
releases.insert(2013, "attack-on-titan");
releases.insert(2023, "frieren");
releases.insert(2019, "demon-slayer");
releases.insert(2022, "bocchi");
releases.insert(2001, "spirited-away");

for (year, title) in releases.range(2019..=2023) {
    println!("{year}: {title}");
}
```

```text
2019: demon-slayer
2022: bocchi
2023: frieren
```

`.range()` takes the same range syntax you already know from slices — `2019..=2023` means both ends included, `..2019` means "everything less than 2019," and so on. One thing is on you: making sure the start bound is not greater than the end bound. "Errors you will meet" shows exactly what happens if you don't.

### `HashSet<T>` and `BTreeSet<T>` — membership, no value

`HashSet<T>` is conceptually a `HashMap<T, ()>`: it only asks "is this value present?", with no extra data attached to it.

```rust
use std::collections::HashSet;

let mut watched: HashSet<&str> = HashSet::new();
watched.insert("frieren");
watched.insert("bocchi");
println!("watched frieren? {}", watched.contains("frieren"));
println!("watched naruto?  {}", watched.contains("naruto"));
```

```text
watched frieren? true
watched naruto?  false
```

If you've used Python's `set`, this already feels familiar — with one difference: `HashSet` in Rust has no index or position at all. `set[0]` is meaningless in Python too, but Python just raises `TypeError`; Rust's compiler won't even let that code exist — you'll see why in "Errors you will meet."

And the same relationship that held between `HashMap` and `BTreeMap` holds between `HashSet` and `BTreeSet`: the same membership operations, always sorted this time.

```rust
use std::collections::BTreeSet;

let sorted_genres: BTreeSet<&str> = BTreeSet::from(["isekai", "action", "comedy", "drama"]);
for genre in &sorted_genres {
    println!("{genre}");
}
```

```text
action
comedy
drama
isekai
```

### Set algebra: intersection, union, difference

Where `HashSet` really earns its keep is the same set algebra you know from high-school math, or from Python's `set`:

| Python | Rust |
|---|---|
| `a & b` | `a.intersection(&b)` |
| `a \| b` | `a.union(&b)` |
| `a - b` | `a.difference(&b)` |
| `a ^ b` | `a.symmetric_difference(&b)` |

The one place the analogy breaks: in Python, `a & b` gives you a new `set`. In Rust, all four of these methods give you an *iterator* of borrowed references, not a new `HashSet`. If you only want to look at the result or loop over it, that's already enough:

```rust
let action: HashSet<&str> = HashSet::from(["frieren", "bleach", "naruto"]);
let comedy: HashSet<&str> = HashSet::from(["bocchi", "bleach"]);

for title in action.intersection(&comedy) {
    println!("{title}");
}
```

```text
bleach
```

`.union()`, `.difference()` and `.symmetric_difference()` all work the same way — each hands back an iterator you can loop over. If you want an owned `HashSet` built from the result, you build a fresh empty one yourself and insert a clone of each member into it — exactly the pattern you'll use in "Exercises."

### `VecDeque<T>` — a ring buffer, fast at both ends

From [2.1.1](../01-vec-depth/README.md) you know `Vec` is fast at the back — `O(1)` for `.push()` and `.pop()` — but slow at the front: `.insert(0, x)` or `.remove(0)` has to shift every other element over by one, `O(n)`. That's because `Vec` is one contiguous block of memory; clearing room at the start means pushing everything after it along.

`VecDeque<T>` ("double-ended queue") solves this with a different underlying shape: a **ring buffer**. Instead of always starting at slot zero, it keeps two pointers — one for the front, one for the back — that can sit anywhere in the buffer and wrap around to the start once they reach the end. The result: adding or removing from *either* end is `O(1)`.

```senpai-visual
{"kind":"queue","labels":["push_front — O(1)","ring buffer","push_back — O(1)","front","back"]}
```

```rust
use std::collections::VecDeque;

let mut queue: VecDeque<&str> = VecDeque::new();
queue.push_back("Frieren");
queue.push_back("Bocchi");
queue.push_front("Bleach");
println!("{queue:?}");

let next = queue.pop_front();
println!("{next:?}");
```

```text
["Bleach", "Frieren", "Bocchi"]
Some("Bleach")
```

If you know Python, this is exactly `collections.deque` — same idea, same `.append()`/`.appendleft()`/`.pop()`/`.popleft()`, just different names. Rule of thumb: if you only ever work at one end, `Vec` is simpler and just as fast; if you need both ends — a queue, a sliding window, an "undo" history — reach for `VecDeque`.

### `BinaryHeap<T>` — a priority queue

`BinaryHeap<T>` answers a completely different question: not "sort this," not "is this a member," but "what matters most right now?" Every time you call `.pop()`, it hands back the current *maximum* — not insertion order, not sorted order — in `O(log n)`. If you only need the maximum once, `.iter().max()` on a `Vec` works fine too; `BinaryHeap` earns its keep when you keep adding new items and keep taking the current maximum, without re-sorting the whole collection each time.

```rust
use std::collections::BinaryHeap;

let mut ratings: BinaryHeap<u32> = BinaryHeap::new();
ratings.push(7);
ratings.push(2);
ratings.push(9);
ratings.push(4);

println!("{:?}", ratings.pop());
println!("{:?}", ratings.pop());
```

```text
Some(9)
Some(7)
```

It's called a "heap," the same word from [1.2.1](../../../phase1-fundamentals/02-ownership-and-memory/01-stack-and-heap/README.md) — but it means something different here: there it was a region of memory, here it's the shape of a data structure (a complete binary tree, stored inside a plain array). Two unrelated meanings for one word; Rust itself carries the same overload.

Where `BinaryHeap` earns its name is a priority queue. Tuples compare lexicographically — the first element first, and only the second if the first ties — which is already enough to build a priority queue out of `(priority, title)` pairs, with no extra code at all:

```rust
let mut up_next: BinaryHeap<(u32, &str)> = BinaryHeap::new();
up_next.push((2, "Bocchi"));
up_next.push((5, "Frieren"));
up_next.push((1, "Bleach"));

while let Some((priority, title)) = up_next.pop() {
    println!("priority {priority}: {title}");
}
```

```text
priority 5: Frieren
priority 2: Bocchi
priority 1: Bleach
```

If you know Python, you know the `heapq` module — same idea, with one difference worth stating right here: `heapq` defaults to a min-heap; `BinaryHeap` in Rust defaults to a max-heap. Forget that one fact, and your program's logic silently inverts — with no error at all.

### `Reverse<T>` — the same heap, flipped

For the times you genuinely need a min-heap, you don't have to write comparison logic yourself. `std::cmp::Reverse` is a one-field wrapper that simply flips the result of a comparison — smaller becomes bigger, as far as `BinaryHeap` is concerned:

```rust
use std::cmp::Reverse;

let mut low_first: BinaryHeap<Reverse<u32>> = BinaryHeap::new();
low_first.push(Reverse(7));
low_first.push(Reverse(2));
low_first.push(Reverse(9));

println!("{:?}", low_first.pop());
```

```text
Some(Reverse(2))
```

Same `BinaryHeap`, same `.push()`/`.pop()`, no new "minimum" logic was written — only what's inside changed. That's exactly what you'll reach for in "Challenge."

---

## Hands on

Five working examples:

```sh
cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 01-btreemap-same-api-sorted
cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 02-btreemap-range-queries
cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 03-hashset-and-set-algebra
cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 04-vecdeque-both-ends
cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 05-binaryheap-max-and-min
```

Then the four broken ones:

```sh
cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 06-btreemap-range-start-after-end --features broken
cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 07-hashset-no-indexing --features broken
cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 08-vec-has-no-pop-front --features broken
cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 09-binaryheap-needs-ord --features broken
```

Then try these:

1. In `01-btreemap-same-api-sorted`, add a fourth title to `counts` (the `HashMap` version) starting with the letter "a". Where does it show up in `sorted`'s output?
2. In `03-hashset-and-set-algebra`, change `comedy` so it shares nothing with `action`. What does `.intersection()` print now?
3. In `05-binaryheap-max-and-min`, add a fifth rating to `ratings` bigger than all the others, right before the first `.pop()`. Is the `.peek()` printed earlier still correct?

---

## Errors you will meet

### `.range()` panic — when the start is greater than the end

```text
thread 'main' (25344) panicked at C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\alloc\src\collections\btree\search.rs:121:21:
range start is greater than range end in BTreeMap
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(The number in parentheses after `'main'` is the thread id and changes every run; the rest of the message is fixed.)

**What the compiler is actually complaining about:** this isn't even a compiler error — the code compiles fine, because `.range()` type-checks on any bounds of the right type. The problem shows up at run time instead: `.range()` takes the bounds exactly as you write them, it does not sort them for you. If the start bound is greater than the end bound — say, writing "2023 down to 2019" instead of "2019 up to 2023" — the result is a panic.

**The fix:** check yourself, before calling `.range()`, that the start isn't greater than the end:

```rust
let (start, end) = (2023, 2019);
if start > end {
    println!("no results — start is after end");
} else {
    for (year, title) in releases.range(start..=end) {
        println!("{year}: {title}");
    }
}
```

```text
no results — start is after end
```

**Why this is the fix:** the same guard you saw in [1.6.1](../../../phase1-fundamentals/06-absence-and-failure/01-option-and-null-safety/README.md) — handle the exceptional case first, so the rest of the code doesn't have to think about it. `.range()` can't add this guard itself, because it has no way to know what an "empty range" should mean to you — maybe you want a panic, maybe an empty list, maybe you want the bounds flipped automatically. That decision is yours, and it has to happen before `.range()` runs.

### `E0608` — you cannot index into a `HashSet`

```text
error[E0608]: cannot index into a value of type `HashSet<&str>`
  --> phase2-intermediate\01-collections\03-btreemap-hashset-vecdeque\examples\07-hashset-no-indexing.rs:13:23
   |
13 |     let first = genres[0];
   |                       ^^^

For more information about this error, try `rustc --explain E0608`.
```

**What the compiler is actually complaining about:** `[0]` means "whatever is at position zero," and that only makes sense for things that have positions — a `Vec`, an array, a slice. A `HashSet` is not a sequence; its members don't have positions, they just are or aren't present. Rust never wrote an indexing implementation for `HashSet`, because there was nothing meaningful to write.

**The fix:** the question you actually want to ask is usually "is this a member?", not "what's member zero?":

```rust
println!("{}", genres.contains("comedy"));
```

```text
true
```

**Why this is the fix:** `.contains()` is exactly the question a set can answer, and it's almost always what you actually wanted from a `HashSet` in the first place. If you genuinely — rarely — just need *some* member, `.iter().next()` does that; but which member you get is unspecified, because `HashSet` makes no ordering promise at all — the same rule [2.1.2](../02-hashmap-in-depth/README.md) stated for `HashMap`.

### `E0599` — `Vec` never had a `pop_front`

```text
error[E0599]: no method named `pop_front` found for struct `Vec<&str>` in the current scope
  --> phase2-intermediate\01-collections\03-btreemap-hashset-vecdeque\examples\08-vec-has-no-pop-front.rs:12:22
   |
12 |     let next = queue.pop_front();
   |                      ^^^^^^^^^ method not found in `Vec<&str>`

For more information about this error, try `rustc --explain E0599`.
```

**What the compiler is actually complaining about:** `Vec` never defined a method called `pop_front`. `.remove(0)` exists — but it's `O(n)`, because it has to shift every remaining element over by one. `pop_front` doesn't exist at all, because the thing it would mean — remove from the front in `O(1)` — isn't something `Vec`'s underlying shape (one contiguous block, always starting at slot zero) can deliver.

**The fix:** change the type, not the method name:

```rust
let mut queue: VecDeque<&str> = vec!["Frieren", "Bocchi"].into();
let next = queue.pop_front();
println!("{next:?}");
```

```text
Some("Frieren")
```

**Why this is the fix:** this is precisely the wall `VecDeque` exists to remove. `.into()` converts a `Vec` into a `VecDeque` (moving the elements into a fresh ring buffer), and from that point on both `pop_front` and `push_front` are available in `O(1)`.

### `E0599` — `BinaryHeap<f64>` never gets off the ground

```text
error[E0599]: the method `push` exists for struct `BinaryHeap<f64>`, but its trait bounds were not satisfied
  --> phase2-intermediate\01-collections\03-btreemap-hashset-vecdeque\examples\09-binaryheap-needs-ord.rs:15:13
   |
15 |     ratings.push(8.5);
   |             ^^^^ method cannot be called on `BinaryHeap<f64>` due to unsatisfied trait bounds
   |
   = note: the following trait bounds were not satisfied:
           `f64: Ord`

For more information about this error, try `rustc --explain E0599`.
```

**What the compiler is actually complaining about:** `BinaryHeap<T>` has to compare any two elements to find the maximum, so `T` must be `Ord` — able to give a definite answer to "which is bigger?" for *every* pair of values. `f64` cannot make that promise, because `NaN` cannot be compared to any number, not even itself. That's why `f64` only implements the weaker `PartialOrd` (a comparison that might not have an answer), not `Ord`, and `BinaryHeap<f64>` never gets off the ground.

**The fix:** use a fully ordered type instead — for example, scale the rating and keep it as an integer:

```rust
let mut ratings: BinaryHeap<u32> = BinaryHeap::new();
ratings.push(85); // 8.5 out of 10, scaled by 10 to stay an integer
println!("{:?}", ratings.peek());
```

```text
Some(85)
```

**Why this is the fix:** `u32` can always compare any two values, no exceptions — exactly what `BinaryHeap` needs. It isn't the only fix (fully-ordered float wrapper crates exist too), but it's the simplest one, and it needs no new tooling at all.

---

## Exercises

### Warm up

<details>
<summary>What does this print?</summary>

```rust
use std::collections::BTreeMap;

let mut map: BTreeMap<&str, u32> = BTreeMap::new();
map.insert("naruto", 4);
map.insert("bleach", 2);
map.insert("frieren", 9);

for (title, count) in &map {
    println!("{title}: {count}");
}
```

</details>

<details>
<summary>Answer</summary>

```text
bleach: 2
frieren: 9
naruto: 4
```

Insertion order never mattered; a `BTreeMap` is always walked by key, alphabetically.

</details>

<details>
<summary>What does this print?</summary>

```rust
use std::collections::VecDeque;

let mut q: VecDeque<i32> = VecDeque::new();
q.push_back(1);
q.push_front(2);
q.push_back(3);
q.push_front(4);
println!("{q:?}");
```

</details>

<details>
<summary>Answer</summary>

```text
[4, 2, 1, 3]
```

Each `push_front` joins the front of whatever exists so far, each `push_back` joins the back. Trace it: `[1]` → `[2, 1]` → `[2, 1, 3]` → `[4, 2, 1, 3]`.

</details>

<details>
<summary>What does this print?</summary>

```rust
use std::collections::BinaryHeap;

let mut heap: BinaryHeap<i32> = BinaryHeap::new();
heap.push(3);
heap.push(8);
heap.push(1);
println!("{:?}", heap.pop());
```

</details>

<details>
<summary>Answer</summary>

```text
Some(8)
```

`.pop()` always returns the current maximum — not the last thing pushed, and not the first thing pushed.

</details>

<details>
<summary>What does <code>a.intersection(&b)</code> on two <code>HashSet&lt;String&gt;</code> values return — a fresh <code>HashSet</code>?</summary>

No. It returns an iterator of references borrowed from `a`, not a new set. If you need an owned `HashSet<String>`, you build an empty one yourself and insert a clone of each member into it.

</details>

<details>
<summary>Does <code>BinaryHeap&lt;f64&gt;</code> compile?</summary>

No. `BinaryHeap<T>` needs `T` to implement `Ord` — every pair of values must be definitely comparable. `f64` only implements `PartialOrd` because of `NaN`, not `Ord`, so `BinaryHeap<f64>` fails to compile entirely.

</details>

### Repair

Fix all four broken examples:

1. Fix `examples/06-btreemap-range-start-after-end.rs` so it no longer panics — add a guard that checks the start isn't greater than the end before calling `.range()`.
2. Fix `examples/07-hashset-no-indexing.rs` so it compiles — use `.contains()` instead of `[0]`.
3. Fix `examples/08-vec-has-no-pop-front.rs` so it compiles — change `queue`'s type from `Vec` to `VecDeque`.
4. Fix `examples/09-binaryheap-needs-ord.rs` so it compiles — without losing what the "rating" actually means (hint: keep the meaning, just change the type).

### Implement

Six pieces in `src/lib.rs` — two functions for `BTreeMap`, two for `HashSet`, and two structs for `VecDeque` and `BinaryHeap`:

```sh
cargo test -p p2-01-03-btreemap-hashset-vecdeque
```

You never need `.collect()` anywhere — a plain `for` loop that pushes into a fresh collection is always enough. Read `shows_in_year_range`'s doc comment carefully: it states exactly what to return when `start` is greater than `end` — the same guard you just saw in "Errors you will meet," this time written by you.

### Build

Extend `WatchQueue` (or build a fresh small struct) so that `enqueue` silently does nothing if the title is already waiting in the queue — without scanning the whole queue every time. Hint: keep a `HashSet<String>` alongside the `VecDeque<String>` so the "is it already there?" check is `O(1)` instead of `O(n)`. The exact API is up to you — this one is open, there's no test for it.

### Challenge (optional)

This one reaches past what this lesson needs — only for when you're curious how `BinaryHeap` gets used for something more serious than an example.

Say you have a large `BTreeMap<u32, String>` of year to title, and you only want the `k` *most recent* titles — without sorting the whole thing. Keep a `BinaryHeap<Reverse<(u32, String)>>` that never holds more than `k` members: add each new entry, and if the size goes past `k`, take the maximum of this min-heap — meaning the *smallest* remaining year — and discard it. At the end you're left with exactly the `k` most recent, without ever fully sorting the rest.

(If you did this same exercise with an iterator adapter, it would be a few lines shorter — exactly what [2.2](../../02-iterators-and-closures/README.md) teaches you. Not yet, though; write it with a loop and an `if` for now.)

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `BTreeMap<K, V>` | Like `HashMap`, but keys stay sorted — `O(log n)` instead of average `O(1)` | Reports, tests, anywhere iteration order must be deterministic |
| `.range()` | Every entry of a `BTreeMap` whose key falls in a range | A "between X and Y" query without scanning everything |
| `HashSet<T>` / `BTreeSet<T>` | Membership with no value — like `HashMap<T, ()>`; the second is always sorted | A fast "is this present?" check, with no extra data |
| Intersection / union / difference / symmetric difference | `.intersection()`, `.union()`, `.difference()`, `.symmetric_difference()` — each returns an iterator | Comparing two sets without a manual loop |
| `VecDeque<T>` | A ring buffer; `O(1)` at both ends, unlike `Vec`, which is only fast at the back | A queue, an undo history, a sliding window |
| Priority queue | A structure that always hands back "what matters most right now" without a full sort | `BinaryHeap` is exactly this |
| `BinaryHeap<T>` | `.pop()` always returns the maximum, in `O(log n)`; plain iteration over it is not sorted | Scheduling, a support queue, "what's next?" |
| `Reverse<T>` | Flips the result of a comparison | Turning that same `BinaryHeap` into a min-heap |

### What you now know

- `BTreeMap` has the same API as `HashMap` — entry API included — trading `O(log n)` for always-sorted iteration.
- `.range()` runs a range query straight against the relevant slice of the tree; you have to make sure the start bound isn't past the end bound yourself.
- `HashSet<T>` is membership with no value, and its algebra methods return iterators, not a fresh `HashSet`.
- `Vec` is `O(1)` at the back and `O(n)` at the front; `VecDeque` is `O(1)` at both ends because it's a ring buffer, not one contiguous block.
- `BinaryHeap::pop()` always returns the current maximum, in `O(log n)`, with no full sort required; it needs a type that implements `Ord`, not just `PartialOrd`.
- `Reverse<T>` turns the same `BinaryHeap` into a min-heap, with no new logic of your own.
- Picking the wrong collection isn't a compiler error — it's a silent performance or ordering bug.

### What comes back later

- **A full decision table across every collection** — [2.1.4 — Choosing a collection](../04-choosing-a-collection/README.md)
- **`.collect()` and iterator adapters, for building a `HashSet`/`BTreeMap` in one line** — [2.2 — Iterators and closures](../../02-iterators-and-closures/README.md)
- **Implementing `Ord`/`PartialOrd`/`Hash` on your own types, so they can go into a `BinaryHeap` or `HashSet` too** — [2.3.4 — Standard derives, by hand](../../03-traits-and-generics/04-standard-derives-by-hand/README.md)
- **Writing a function that works over any `Ord` type, not just `u32`** — [2.3.2 — Generic functions and structs](../../03-traits-and-generics/02-generic-functions-and-structs/README.md)

### Can you explain?

- Why is `BTreeMap` slower than `HashMap`, and exactly what do you get in exchange for that slowness?
- What question does `.range()` answer that `HashMap` can never answer cheaply at all?
- Why doesn't `a.intersection(&b)` give you a fresh `HashSet`, and what do you do if you need one?
- Why is `.remove(0)` on a `Vec` slow, and exactly how does `VecDeque` make the same operation cheap?
- What's the difference between Python's `heapq` and Rust's `BinaryHeap` on "which end is the default"?
- Why doesn't `BinaryHeap<f64>` compile, when `BinaryHeap<u32>` does?

---

## Going further

- [`BTreeMap` docs](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html) and [`HashSet` docs](https://doc.rust-lang.org/std/collections/struct.HashSet.html) — the full method list for each.
- [`VecDeque` docs](https://doc.rust-lang.org/std/collections/struct.VecDeque.html) and [`BinaryHeap` docs](https://doc.rust-lang.org/std/collections/struct.BinaryHeap.html) — same.
- [The Rust Book — common collections](https://doc.rust-lang.org/book/ch08-00-common-collections.html) — these four and their neighbors, from the Rust team itself.
- [`std::cmp::Reverse` docs](https://doc.rust-lang.org/std/cmp/struct.Reverse.html) — small, but useful everywhere from `sort_by_key` to `BinaryHeap`.
