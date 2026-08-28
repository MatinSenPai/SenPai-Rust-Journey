# Solution — 2.1.4 Choosing a collection

```rust
use std::collections::{BTreeMap, HashSet, VecDeque};

pub fn unique_viewer_count(viewer_ids: &[u32]) -> usize {
    let mut seen: HashSet<u32> = HashSet::new();
    for id in viewer_ids {
        seen.insert(*id);
    }
    seen.len()
}

pub struct RecentEvents {
    capacity: usize,
    events: VecDeque<String>,
}

impl RecentEvents {
    pub fn new(capacity: usize) -> Self {
        RecentEvents {
            capacity,
            events: VecDeque::new(),
        }
    }

    pub fn record(&mut self, event: &str) {
        self.events.push_back(event.to_string());
        while self.events.len() > self.capacity {
            self.events.pop_front();
        }
    }

    pub fn oldest_to_newest(&self) -> Vec<String> {
        let mut out = Vec::new();
        for event in &self.events {
            out.push(event.clone());
        }
        out
    }
}

pub fn daily_report(entries: &[(u32, u32)]) -> String {
    let mut by_day: BTreeMap<u32, u32> = BTreeMap::new();
    for (day, count) in entries {
        by_day.insert(*day, *count);
    }

    let mut report = String::new();
    for (day, count) in &by_day {
        report.push_str(&format!("day {day}: {count}\n"));
    }
    report
}
```

All three follow exactly the reasoning `## The concept` built — no new method, just the same decision table, this time applied by you.

## `unique_viewer_count` — membership, not a value

```rust
let mut seen: HashSet<u32> = HashSet::new();
for id in viewer_ids {
    seen.insert(*id);
}
seen.len()
```

The real question this function asks is only "have I seen this ID before?" — not key-to-value, not order, not position. That is exactly Axis 1: when membership is all that matters, a `HashSet` answers it in `O(1)`. With a `Vec` you would have had to scan the whole seen-so-far list for every new ID — exactly what `01-lookup-by-key-vs-scan` showed you, except the question there was "what's its value?" and here it's "is it here at all?"

`.insert()` on a value that is already present does nothing and returns `false` (ignored here) — exactly the "each ID counted once" behavior the spec asked for, with no explicit `if already_seen` check needed.

## `RecentEvents` — both ends, cheaply

```rust
pub fn record(&mut self, event: &str) {
    self.events.push_back(event.to_string());
    while self.events.len() > self.capacity {
        self.events.pop_front();
    }
}
```

This is Axis 4, directly: every event is added at the **back** (`push_back`), and once capacity is exceeded, one comes off the **front** (`pop_front`) — exactly the pair of operations a `Vec` cannot do cheaply at its front (remember `06-vec-has-no-pop-front`), but `VecDeque` gives you both at `O(1)`. Building this with a `Vec` instead would have made `record` call `remove(0)` — shifting the entire buffer on every single event.

The `while` (not `if`) is deliberate: even if something changed `capacity` between two calls to `record`, the function still ends up correct. For `capacity == 0`, that same loop immediately pops back out whatever was just pushed — exactly "never holds anything."

`oldest_to_newest` returns an owned copy of the strings (not a reference into `self.events`) because its signature promised `Vec<String>`, not `&VecDeque<String>` — the same borrow-versus-ownership distinction Phase 1 taught you, here applied to a collection's contents.

## `daily_report` — a repeated key means replace, not add

```rust
let mut by_day: BTreeMap<u32, u32> = BTreeMap::new();
for (day, count) in entries {
    by_day.insert(*day, *count);
}
```

`BTreeMap::insert` **replaces** the old value with the new one when the key already exists — no summing, no error. That one line gives you the "the later entry wins" rule the spec asked for, for free.

```rust
let mut report = String::new();
for (day, count) in &by_day {
    report.push_str(&format!("day {day}: {count}\n"));
}
report
```

This is where choosing `BTreeMap` over `HashMap` really pays off: iterating `&by_day` is guaranteed to come out smallest-day-first — no need to collect the pairs and sort them yourself afterward, which is exactly what you would have had to do with a `HashMap`, and exactly what this module's first lesson said about `top_n`.

## What this lesson was really about

- **`unique_viewer_count`**: when the question is "is this a member?", not "what's its value?", a `HashSet` is exactly the right size — no less, no more.
- **`RecentEvents`**: when you need cheap push/pop at **both** ends, `VecDeque` is the only one of these six that gives you `O(1)` at both; `Vec` always makes one end `O(n)`.
- **`daily_report`**: when you need both key-based overwriting and sorted-by-key output at the end, `BTreeMap` gives you both in one structure — a `HashMap` would have made you collect and sort separately.
- Three functions, three different signatures, and **none of them named a collection.** The moment you made that call is the moment you actually learned this lesson.
