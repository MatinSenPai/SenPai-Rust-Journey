# Solution — 2.1.2 `HashMap` in depth

```rust
pub fn get_or_zero(counts: &HashMap<String, u32>, key: &str) -> u32 {
    counts.get(key).copied().unwrap_or(0)
}

pub fn word_counts(text: &str) -> HashMap<String, u32> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for word in text.split_whitespace() {
        counts
            .entry(word.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }
    counts
}

pub fn group_by_first_letter(words: &[&str]) -> HashMap<char, Vec<String>> {
    let mut groups: HashMap<char, Vec<String>> = HashMap::new();
    for word in words {
        if let Some(letter) = word.chars().next() {
            groups
                .entry(letter)
                .or_insert_with(Vec::new)
                .push(word.to_string());
        }
    }
    groups
}

pub fn bump_or_init(counts: &mut HashMap<String, i32>, key: &str, amount: i32) {
    counts
        .entry(key.to_string())
        .and_modify(|value| *value += amount)
        .or_insert(amount);
}

pub fn most_common(counts: &HashMap<String, u32>) -> Option<(String, u32)> {
    let mut best: Option<(&String, &u32)> = None;
    for (key, value) in counts {
        let replace = match best {
            None => true,
            Some((best_key, best_value)) => {
                value > best_value || (value == best_value && key < best_key)
            }
        };
        if replace {
            best = Some((key, value));
        }
    }
    best.map(|(key, value)| (key.clone(), *value))
}
```

All five functions use nothing but `HashMap` and the `entry` API — no `BTreeMap`, no `HashSet`, no `VecDeque`. Those are [2.1.3](../../03-btreemap-hashset-vecdeque/README.md).

## `get_or_zero` — an `Option` you already know

```rust
counts.get(key).copied().unwrap_or(0)
```

`.get(key)` hands back an `Option<&u32>` — exactly what Phase 1 handed you over and over. `.copied()` turns that `Option<&u32>` into an `Option<u32>` (cheap, since `u32` is `Copy`), and `.unwrap_or(0)` is the same [1.6.2](../../../../phase1-fundamentals/06-absence-and-failure/02-option-combinators/README.md) combinator, aimed here at whatever `HashMap::get` handed back. Nothing new is happening in this function; it just proves `.get()` on a `HashMap` behaves exactly like `.get()` on a `Vec` or `.first()` on a slice: all three hand back an `Option`, and all three open with the same tools.

## `word_counts` — the `and_modify` + `or_insert` combo

```rust
for word in text.split_whitespace() {
    counts
        .entry(word.to_string())
        .and_modify(|count| *count += 1)
        .or_insert(1);
}
```

The same combo as example 04, now over real text instead of a fixed list. `.entry(word.to_string())` is one lookup. If the key is already there, `.and_modify(...)` bumps its value; if not, `.or_insert(1)` builds it with a starting value of 1. The two arms never both run — exactly what their names promise.

The one subtlety: `word`, coming out of `.split_whitespace()`, is a `&str` — a slice of `text` itself. But the map's key has to be a `String`, because the map has to own its keys, and `text` might not outlive the function. `word.to_string()` is exactly that: an owned copy of the slice.

## `group_by_first_letter` — `or_insert_with` for a default that is actually expensive

```rust
if let Some(letter) = word.chars().next() {
    groups
        .entry(letter)
        .or_insert_with(Vec::new)
        .push(word.to_string());
}
```

`word.chars().next()` gives an `Option<char>` — `None` only when `word` is the empty string — and `if let` skips exactly that one case without anything panicking. `.or_insert_with(Vec::new)`, not `.or_insert(Vec::new())`, matters here: with the second form, a fresh `Vec` gets built on *every* iteration — new key or not — only to be thrown away immediately whenever the key already existed. With `n` words and `k` distinct keys, that is `n` allocations instead of `k`.

Order survives because the loop walks `words` in input order and only ever `.push`es — nothing here sorts or reverses anything.

## `bump_or_init` — the same pattern, mutating in place this time

```rust
counts
    .entry(key.to_string())
    .and_modify(|value| *value += amount)
    .or_insert(amount);
```

The difference from `word_counts` is that this one returns nothing — `counts` is a `&mut HashMap<...>`, and the function works on that same map directly. `amount` can be negative too (it is `i32`, not `u32`), so this is not quite "counting" — it is "adjusting a value," with the same one entry pattern doing the work either way.

## `most_common` — why this function needed a tie-break rule at all

```rust
let mut best: Option<(&String, &u32)> = None;
for (key, value) in counts {
    let replace = match best {
        None => true,
        Some((best_key, best_value)) => {
            value > best_value || (value == best_value && key < best_key)
        }
    };
    if replace {
        best = Some((key, value));
    }
}
best.map(|(key, value)| (key.clone(), *value))
```

This is where you actually put this chapter's central lesson into practice: **walking `for (key, value) in counts` has no guaranteed order.** If this simply crowned the first value it happened to reach as the "winner," `most_common`'s answer on a tied map could differ between two runs of the exact same program — precisely what example 02 (`02-no-guaranteed-order`) showed you. Instead, the `replace` condition states an explicit rule: a strictly higher value always wins; on a tie, the alphabetically earlier key wins. That rule has nothing to do with iteration order, so the answer is the same every time, on every machine.

`best` works on references (`&String`, `&u32`) to avoid cloning on every step of the loop; only at the very end, once, does `.map(...)` take ownership of the final answer (`key.clone()`, `*value`).

## What this lesson was really about

- The `entry` API turns a lookup into one lookup — whether through `.or_insert()`, `.or_insert_with()`, or `.and_modify().or_insert()`.
- Reach for `.or_insert_with(f)` instead of `.or_insert(...)` when the default value is actually work (like `Vec::new`) — otherwise you repeat that work on every single lookup, hit or miss.
- A `HashMap`'s key has to own itself once the map outlives the function that built it; `.to_string()` on a borrowed `&str` is exactly that.
- `HashMap` iteration has no guaranteed order — so wherever the correct answer depends on "which one comes first," you write an explicit rule yourself rather than leaning on iteration order to decide it for you.
