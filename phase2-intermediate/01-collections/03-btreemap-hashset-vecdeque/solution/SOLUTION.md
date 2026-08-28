# Solution — 2.1.3 `BTreeMap`, `HashSet`, `VecDeque`, `BinaryHeap`

```rust
pub fn sorted_by_title(counts: HashMap<String, u32>) -> BTreeMap<String, u32> {
    let mut sorted = BTreeMap::new();
    for (title, count) in counts {
        sorted.insert(title, count);
    }
    sorted
}
```

There's no `.sort()` here, and none was needed. `counts` is walked with an ordinary `for` loop — in whatever order `HashMap` happens to hand entries over — and each pair is placed into a fresh `BTreeMap` with `.insert()`. Sortedness is a property of *where the data ends up*, not the order it arrived in: `BTreeMap` re-finds the correct spot for a key on every `insert()`, no matter what order the keys showed up in.

```rust
pub fn shows_in_year_range(
    releases: &BTreeMap<u32, String>,
    start: u32,
    end: u32,
) -> Vec<String> {
    if start > end {
        return Vec::new();
    }
    let mut titles = Vec::new();
    for (_year, title) in releases.range(start..=end) {
        titles.push(title.clone());
    }
    titles
}
```

The first guard is exactly what "Errors you will meet" showed: calling `.range(start..=end)` without this check, with `start` greater than `end`, panics. Here, instead of panicking, the spec says to return an empty `Vec` — so that's exactly what happens, before `.range()` is ever reached. After the guard, `.range(start..=end)` goes straight to that slice of the tree; `title.clone()` is needed because `releases` is only borrowed, and `titles` has to own its own strings.

```rust
pub fn shared_genres(a: &HashSet<String>, b: &HashSet<String>) -> HashSet<String> {
    let mut shared = HashSet::new();
    for genre in a.intersection(b) {
        shared.insert(genre.clone());
    }
    shared
}

pub fn exclusive_genres(a: &HashSet<String>, b: &HashSet<String>) -> HashSet<String> {
    let mut exclusive = HashSet::new();
    for genre in a.symmetric_difference(b) {
        exclusive.insert(genre.clone());
    }
    exclusive
}
```

Both functions share a shape: the set-algebra method (`.intersection()` or `.symmetric_difference()`) hands back an iterator of `&String`s borrowed from `a`; since the signature promises an *owned* `HashSet<String>`, each member is cloned into a fresh `HashSet`. No `.collect()` anywhere — a `for` loop and an `.insert()` do exactly what `.cloned().collect()` would have, just spelled out.

```rust
pub struct WatchQueue {
    queue: VecDeque<String>,
}

impl WatchQueue {
    pub fn new() -> Self {
        Self { queue: VecDeque::new() }
    }

    pub fn enqueue(&mut self, title: String) {
        self.queue.push_back(title);
    }

    pub fn watch_next(&mut self) -> Option<String> {
        self.queue.pop_front()
    }

    pub fn watch_next_priority(&mut self, title: String) {
        self.queue.push_front(title);
    }
    // len and is_empty just forward to self.queue.len() and self.queue.is_empty().
}
```

`WatchQueue` is a thin wrapper around `VecDeque` — every method just calls the matching `VecDeque` method. The interesting decision was the type, not the code: `enqueue` goes to `push_back` (the back of the queue, `O(1)`) and `watch_next_priority` goes to `push_front` (the front, also `O(1)`) — if this wrapped a `Vec` instead, `watch_next_priority` would need `.insert(0, title)`, which is `O(n)`.

```rust
pub struct WatchPriorityQueue {
    queue: BinaryHeap<(u32, String)>,
}

impl WatchPriorityQueue {
    pub fn new() -> Self {
        Self { queue: BinaryHeap::new() }
    }

    pub fn add(&mut self, priority: u32, title: String) {
        self.queue.push((priority, title));
    }

    pub fn watch_highest_priority(&mut self) -> Option<String> {
        self.queue.pop().map(|(_priority, title)| title)
    }
}
```

`add` just `push`es each pair with no extra logic — tuples already compare lexicographically, `priority` first, `title` only to break a tie. `watch_highest_priority` pops the maximum and uses `.map()` on the resulting `Option` (not on an iterator — this is the `Option` combinator from [1.6.2](../../../../phase1-fundamentals/06-absence-and-failure/02-option-combinators/README.md)) to keep only `title`, discarding `priority`.

## What this lesson was really about

- **Returning a `BTreeMap` instead of a `HashMap` buys sorted iteration for free, no matter what order you inserted in** — `sorted_by_title` proved that with zero `.sort()` calls.
- **The guard goes before `.range()`, never after** — the same "handle the exceptional case first" shape you already know from [1.1.5](../../../../phase1-fundamentals/01-foundations/05-control-flow/README.md).
- **Set-algebra methods hand back an iterator, not a fresh `HashSet`** — a `for` loop and an `.insert()`, no `.collect()`, build exactly the same result.
- **Choosing `VecDeque` over `Vec` was a type decision, not a code-complexity one** — the same four lines, now `O(1)` at both ends.
- **Tuples turn a `BinaryHeap` into a priority queue with zero extra comparison logic** — because Rust already knows how to compare two tuples.
