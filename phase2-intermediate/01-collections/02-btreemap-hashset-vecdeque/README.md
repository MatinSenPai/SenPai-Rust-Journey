# 01.2 — `BTreeMap`, `HashSet`, `VecDeque`

Lesson 1 covered `Vec` and `HashMap`. This lesson rounds out the collection
toolbox with three more shapes, still using the same watch-data domain:
ordered maps, sets, and a queue that's fast at both ends.

## `BTreeMap<K, V>` — like `HashMap`, but sorted

`BTreeMap` has almost the same API as `HashMap` (`.insert`, `.get`, the
entry API all work the same way), but it keeps its keys in sorted order
internally, which means iterating a `BTreeMap` always yields entries by key,
ascending — no explicit sort needed, unlike the `top_n` function from
lesson 1. That determinism isn't free: lookups and inserts are `O(log n)`
(walking a tree) instead of `HashMap`'s average `O(1)` (hashing straight to
a bucket). Rule of thumb: reach for `HashMap` by default; switch to
`BTreeMap` when you specifically need sorted iteration (e.g. printing a
report, or a "range query" like "all shows released between 2020 and
2023," which `BTreeMap::range` supports and `HashMap` cannot).

```rust
use std::collections::{BTreeMap, HashMap};

let counts: HashMap<String, u32> = HashMap::from([
    ("naruto".to_string(), 4),
    ("bleach".to_string(), 2),
]);
let sorted: BTreeMap<String, u32> = counts.into_iter().collect();
// iterating `sorted` now always yields "bleach" before "naruto"
```

## `HashSet<T>` — set operations you already know from Python

If you've used Django's ORM or plain Python `set`s, this one will feel
familiar: `HashSet<T>` stores unique values with no associated data (think
`HashMap<T, ()>` under the hood, conceptually), and gives you the same set
algebra Python does:

| Python | Rust |
|---|---|
| `a & b` | `a.intersection(&b)` |
| `a \| b` | `a.union(&b)` |
| `a - b` | `a.difference(&b)` |
| `a ^ b` | `a.symmetric_difference(&b)` |

The one wrinkle: Rust's set methods return an *iterator* of borrowed
references, not a new `HashSet` directly — so you typically `.collect()`
the result if you want an owned set back:

```rust
use std::collections::HashSet;

let a: HashSet<&str> = HashSet::from(["action", "comedy", "isekai"]);
let b: HashSet<&str> = HashSet::from(["comedy", "drama"]);

let shared: HashSet<&str> = a.intersection(&b).copied().collect();
// shared == {"comedy"}
```

## `VecDeque<T>` — fast at both ends

`Vec` is fast to push/pop at the **end** (`O(1)`), but pushing or removing
at the **front** is `O(n)` — `insert(0, x)` or `remove(0)` has to shift
every other element over by one to make room. If you need to add or remove
from *either* end efficiently — a "watch queue" where you both add shows to
the back and occasionally jump one to the front — `VecDeque<T>` ("double-
ended queue") gives you `O(1)` at both ends:

```rust
use std::collections::VecDeque;

let mut queue: VecDeque<&str> = VecDeque::new();
queue.push_back("Frieren");   // add to the back — O(1)
queue.push_front("Bocchi");   // jump to the front — O(1)
let next = queue.pop_front(); // Some("Bocchi") — O(1)
```

## Your task

Implement the functions and the `WatchQueue` struct in `src/lib.rs`:

- `sorted_by_key` — convert a `HashMap` into a `BTreeMap` (sorted iteration
  "for free").
- `shared_genres` — intersection of two genre sets.
- `exclusive_genres` — symmetric difference: genres in exactly one of the
  two sets, not both.
- `WatchQueue` — a small wrapper around `VecDeque<String>` modeling an
  "up next" queue: `enqueue` (push to the back), `watch_next` (pop from the
  front), and `watch_next_priority` (jump a title to the front of the
  line).

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`.
