# 2.1.2 — `HashMap` in depth: the `entry` API, hashers, `&str` lookup

## At a glance

After this lesson you can:

- Explain how `HashMap` gives you average O(1) lookup and insert, and why you should never rely on the order you get back when you walk it.
- Choose between `.get()`, `.get_mut()`, and indexing directly (`map[key]`) for a look or a change, and say exactly which one panics on a missing key.
- Rewrite code that looks a key up twice — once to check, once to write — into a single lookup using the `entry` API (`or_insert`, `or_insert_with`, `and_modify`).
- Say why `HashMap` keys must be `Hash` and `Eq`, and get one of your own struct types ready to be a key.
- Look a `HashMap<String, V>` up with a `&str` literal, without building a fresh `String` just to ask the question.

**Time:** ~75 minutes · **Prerequisites:** [2.1.1 — `Vec` in depth](../01-vec-depth/README.md)

---

## Why this matters

Up to this point, whenever you needed to find something by an identifier, you had a `Vec` and a loop: walk it until an item's field matches what you're after. For ten items that costs nothing. For ten thousand — a user list, a cache, counting how often each word shows up in a long piece of text — every lookup means checking, on average, half the list one entry at a time. That cost has a name: **O(n)**, meaning the time a lookup takes grows with the size of the list.

`HashMap<K, V>` solves exactly that: hand it a key, get back a value, no linear walk — on average, in constant time (**O(1)**), whether the map holds ten keys or ten million. This is not a minor optimization; it is a type that nearly every real Rust program ends up needing — a web server's cache, a log processor's counters, configuration read from a file, anywhere the main question is "do you have this key?"

If you're coming from Python, none of this is new in spirit: `HashMap<K, V>` does what a `dict` does in Python, or a `Map` in JavaScript — hand it a key, get back a value. But there is one important difference this lesson pauses on, right here: both Python's `dict` (since 3.7) and JavaScript's `Map` remember insertion order — a formal guarantee of the language itself. Rust's `HashMap` makes no such promise, and this lesson is as much about *that difference* as it is about how to use the type — because if some part of your code, unknowingly, counts on a fixed order, it will show up one day, on a different machine, or after nothing more than a restart.

---

## The concept

### A map from key to value: `HashMap::new` and `.insert()`

You build a `HashMap<K, V>` the way you build a `Vec<T>` — except this time there are two types instead of one: the key's type and the value's type.

```rust
let mut watched: HashMap<String, u32> = HashMap::new();
watched.insert("Frieren".to_string(), 12);
watched.insert("Bocchi the Rock".to_string(), 12);
println!("tracked shows: {}", watched.len());

let previous = watched.insert("Frieren".to_string(), 13);
println!("previous count for Frieren: {previous:?}");
```

```text
tracked shows: 2
previous count for Frieren: Some(12)
```

`.insert()` does two things: it places the value under that key, and it hands back whatever value *used to* be there — wrapped in `Option`: `None` if the key was new, `Some(old)` if it just replaced one. The first time we wrote `"Frieren"`, that returned `None` (not printed here); the second time it returned `Some(12)` — exactly the previous value.

### `.get()` and `.get_mut()` — the same `Option` you already know from Phase 1

A look, or a change, with no risk of panicking on a missing key — `Option` does here exactly what it did in [1.6.1](../../../phase1-fundamentals/06-absence-and-failure/01-option-and-null-safety/README.md) for `Vec`, slices, and optional fields: `.get()` gives back an `Option<&V>`, `.get_mut()` an `Option<&mut V>`.

```rust
match watched.get("Frieren") {
    Some(count) => println!("Frieren: {count} episodes"),
    None => println!("Frieren: not tracked"),
}
match watched.get("Your Name") {
    Some(count) => println!("Your Name: {count} episodes"),
    None => println!("Your Name: not tracked"),
}

if let Some(count) = watched.get_mut("Frieren") {
    *count += 1;
}
println!("Frieren now: {:?}", watched.get("Frieren"));
```

```text
Frieren: 13 episodes
Your Name: not tracked
Frieren now: Some(14)
```

Nothing new about `Option` itself here — just the same tool, aimed at a new type. `match` and `if let` behave exactly the way they did on `Vec::get` or a slice's `.first()`.

There is a third way too: index directly, `watched["Frieren"]`. It works — but unlike `.get()`, if the key is missing you panic, with the message `no entry found for key`. Until you're sure the key is there, `.get()` is the safer choice.

### Keys must be `Hash` and `Eq`

To know where to put a key and how to find it again later, `HashMap` needs two things from the key's type: the `Hash` trait (turn the key into a hash number) and the `Eq` trait (tell two keys apart with exact equality, not an approximation). `String`, `&str`, integers, and `char` already have both. For a struct of your own, both are one `#[derive]` away:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Coord {
    row: i32,
    col: i32,
}

let mut board: HashMap<Coord, char> = HashMap::new();
board.insert(Coord { row: 1, col: 2 }, 'O');
println!("{:?}", board.get(&Coord { row: 1, col: 2 }));
```

```text
Some('O')
```

Two separately-built `Coord`s with the same fields are `==` — that's `PartialEq`/`Eq` — and land in the same internal bucket — that's `Hash`. Without either one, `.get()` has no way to know that the key you're handing it now is the same key you placed earlier.

(One sentence on the hashing algorithm itself, since you'll wonder: `HashMap`'s default hasher, SipHash, is chosen to resist a hostile caller who deliberately crafts keys that all collide — not to be the fastest hash possible. Faster hashers exist for hot paths; that is a swap, not a rewrite, and not this lesson's detour.)

### Iteration order is never guaranteed

This is the moment that surprises you if you're coming from Python's `dict` or JavaScript's `Map`. We insert six keys, in this order: `zeta`, `alpha`, `mu`, `beta`, `quill`, `delta`. Then, twice in a row, in the same run, we walk the map:

```rust
let mut ranks: HashMap<&str, u32> = HashMap::new();
ranks.insert("zeta", 1);
ranks.insert("alpha", 2);
ranks.insert("mu", 3);
ranks.insert("beta", 4);
ranks.insert("quill", 5);
ranks.insert("delta", 6);

for (name, _) in &ranks {
    print!("{name} ");
}
```

First run:

```text
zeta mu delta alpha beta quill
```

The exact same program — not one character changed — run again:

```text
mu quill alpha zeta delta beta
```

Two things to notice. First: the order is not insertion order at all — `alpha` was the second key we placed, but it came first in neither run. Second, and more important: **the order differed between the two runs themselves** — same program, same code, same keys, two completely different outputs. (Run the file yourself a few times in a row and you will see the same thing.)

This is not accidental — it's deliberate. Every time the program runs, Rust picks a fresh random key for the hasher — the same SipHash named above — so nobody can prepare keys in advance that all land in the same bucket and slow the map down (a real attack, called *hash-flooding*). The cost is that you can never, even in an otherwise fully deterministic program, rely on `HashMap`'s iteration order. If order matters to you — display to a user, a test that wants a fixed answer, anything a human or an `assert_eq!` is going to read — you have to write an explicit rule *yourself*. This lesson's `most_common` exercise asks you to do exactly that.

### The `entry` API: `or_insert` and `or_insert_with`

Say you want to count how often each word repeats in a list. The first approach that comes to mind: check whether the key exists, then either bump it or create it with 1:

```rust
if counts.contains_key(word) {
    let count = counts.get_mut(word).unwrap();
    *count += 1;
} else {
    counts.insert(word, 1);
}
```

That works, but it looks up every word **twice** — once for `.contains_key()`, once for `.get_mut()` or `.insert()`. The `entry` API does the same job with a single lookup: `.entry(key)` hands back that key's slot — whether it is already filled or not — and `.or_insert(default)` says "if it's empty, put this here," handing back a `&mut V` that points straight at that slot either way:

```rust
let seen = ["fish", "cat", "fish", "dog", "fish", "cat"];
let mut counts: HashMap<&str, u32> = HashMap::new();

for word in seen {
    *counts.entry(word).or_insert(0) += 1;
}

println!("fish: {}", counts["fish"]);
println!("cat:  {}", counts["cat"]);
println!("dog:  {}", counts["dog"]);
```

```text
fish: 3
cat:  2
dog:  1
```

`*counts.entry(word).or_insert(0) += 1` is one line, one lookup, no branching. The first time a word shows up, `.or_insert(0)` places a `0` and hands back a `&mut` to it; `+= 1` makes it 1. Every later time, `.or_insert(0)` does nothing — the key is already there — and hands back a `&mut` to the existing value instead.

```senpai-visual
{"kind":"concept","labels":["entry(key)","slot found?","or_insert: fill it","&mut V either way"]}
```

If your default value isn't free — say you need to build a fresh `Vec` — reach for `.or_insert_with(f)`: `f` only runs when the key was actually missing.

```rust
let mut first_letters: HashMap<char, Vec<&str>> = HashMap::new();
for word in seen {
    let letter = word.chars().next().unwrap();
    first_letters.entry(letter).or_insert_with(Vec::new).push(word);
}

println!("starting with 'f': {:?}", first_letters[&'f']);
```

```text
starting with 'f': ["fish", "fish", "fish"]
```

The difference from `.or_insert(Vec::new())` is subtle but real: with that form, a fresh `Vec` is built — and immediately thrown away — on *every* call, hit or miss. `.or_insert_with(Vec::new)` keeps that construction for the one moment it is actually needed.

### `and_modify` — when the update itself is conditional

The fuller pattern, and the one you'll reach for most: `.and_modify(f)` calls the closure `f` *only* when the key already exists; `.or_insert(default)` supplies the starting value for when it doesn't. (That closure is the same thing you met in [1.6.2](../../../phase1-fundamentals/06-absence-and-failure/02-option-combinators/README.md) — an inline function passed as a value; the full treatment, with `Fn`/`FnMut`/`FnOnce`, comes later.) Together, the two are the whole word-count pattern:

```rust
let review = "great show great cast good story great animation good pacing";
let mut counts: HashMap<&str, u32> = HashMap::new();

for word in review.split_whitespace() {
    counts
        .entry(word)
        .and_modify(|count| *count += 1)
        .or_insert(1);
}

println!("great: {}", counts["great"]);
println!("good:  {}", counts["good"]);
```

```text
great: 3
good:  2
```

`.and_modify()` on its own never creates a new key — there's nothing to "modify" yet. `.or_insert()` is what actually places the first value. That's why the pattern needs both, in this order.

### Looking up with `&str` on a map whose key is `String`

This one usually surprises people: a `HashMap<String, V>` owns its keys, but *finding* one does not require you to own a `String` too.

```rust
let mut ratings: HashMap<String, u8> = HashMap::new();
ratings.insert(String::from("Frieren"), 10);
ratings.insert(String::from("Bocchi the Rock"), 9);

println!("Frieren: {:?}", ratings.get("Frieren"));

let name = String::from("Bocchi the Rock");
println!("{name}: {:?}", ratings.get(&name));
println!("still own it: {name}");
```

```text
Frieren: Some(10)
Bocchi the Rock: Some(9)
still own it: Bocchi the Rock
```

`"Frieren"` here is a plain `&str` literal — no `.to_string()`, no fresh `String` built just for one comparison and thrown away immediately after. `name`, a real `String`, is only ever borrowed (`&name`) and is still yours afterward — the same rule you know from [1.3.1](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.md).

This isn't a special case carved out in `HashMap`'s code — a more general rule sits behind it, the `Borrow` trait, which says a `String` and a `&str` can count as "the same thing" for exactly this kind of comparison. For now, just trust that it works; the mechanism itself — and why this is a good deal more specific than ordinary auto-deref — belongs to [Phase 2.4](../../04-lifetimes-and-conversion/03-deref-asref-borrow/README.md).

---

## Hands on

```sh
cargo run -p p2-01-02-hashmap-in-depth --example 01-new-insert-get
cargo run -p p2-01-02-hashmap-in-depth --example 02-no-guaranteed-order
cargo run -p p2-01-02-hashmap-in-depth --example 03-entry-or-insert
cargo run -p p2-01-02-hashmap-in-depth --example 04-entry-and-modify
cargo run -p p2-01-02-hashmap-in-depth --example 05-custom-key-hash-eq
cargo run -p p2-01-02-hashmap-in-depth --example 06-str-lookup-without-allocating
```

Then the three broken ones:

```sh
cargo run -p p2-01-02-hashmap-in-depth --example 07-forgot-deref-on-entry --features broken
cargo run -p p2-01-02-hashmap-in-depth --example 08-key-missing-hash-eq --features broken
cargo run -p p2-01-02-hashmap-in-depth --example 09-double-borrow-get-then-insert --features broken
```

Then try these:

1. Run `02-no-guaranteed-order` ten times in a row. How many different orders do you see? Did any of them come out exactly as insertion order (`zeta, alpha, mu, beta, quill, delta`)?
2. In `03-entry-or-insert`, add one new word (say `"bird"`) to the `seen` array, once. What does `counts["bird"]` come out to?
3. In `05-custom-key-hash-eq`, change `Coord`'s `#[derive(...)]` so it only has `Hash` (no `Eq`, no `PartialEq`). Compile it — which error code do you see, and how does its message differ from the original?

---

## Errors you will meet

### `E0368` — you can't `+=` on a `&mut u32`

```text
error[E0368]: binary assignment operation `+=` cannot be applied to type `&mut u32`
  --> phase2-intermediate\01-collections\02-hashmap-in-depth\examples\07-forgot-deref-on-entry.rs:16:9
   |
16 |         counts.entry(word).or_insert(0) += 1;
   |         -------------------------------^^^^^
   |         |
   |         cannot use `+=` on type `&mut u32`
   |
help: `+=` can be used on `u32` if you dereference the left-hand side
   |
16 |         *counts.entry(word).or_insert(0) += 1;
   |         +

For more information about this error, try `rustc --explain E0368`.
```

**What the compiler is actually objecting to:** `.or_insert(0)` returns a `&mut u32` — the address of where the value lives, not the number itself. `+=` isn't defined on a reference; it's defined on the underlying type.

**The fix:** take the compiler's own suggestion — a `*` in front of the whole expression:

```rust
*counts.entry(word).or_insert(0) += 1;
```

**Why this is the fix:** `*` follows the reference to the actual `u32` on the other end of it — the same dereference [1.3.1](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.md) taught you. `+= 1` then changes exactly the slot `entry` found — not some passing copy.

### `E0599` — a key with no `Hash` and no `Eq`

```text
error[E0599]: the method `insert` exists for struct `HashMap<Coord, char>`, but its trait bounds were not satisfied
  --> phase2-intermediate\01-collections\02-hashmap-in-depth\examples\08-key-missing-hash-eq.rs:19:11
   |
12 | struct Coord {
   | ------------ doesn't satisfy `Coord: Eq` or `Coord: Hash`
...
19 |     board.insert(Coord { row: 0, col: 0 }, 'X');
   |           ^^^^^^
   |
   = note: the following trait bounds were not satisfied:
           `Coord: Eq`
           `Coord: Hash`
help: consider annotating `Coord` with `#[derive(Eq, Hash, PartialEq)]`
   |
12 + #[derive(Eq, Hash, PartialEq)]
13 | struct Coord {
   |

For more information about this error, try `rustc --explain E0599`.
```

**What the compiler is actually objecting to:** `.insert()` on a `HashMap<K, V>` only exists at all when `K: Hash + Eq` — because to place a key, the map has to be able to both hash it and later tell it apart with exact equality. `Coord` here only has `#[derive(Debug)]`; no `Hash`, no `Eq`. The compiler isn't saying "this type is bad" — it's saying this method, on *this particular* `HashMap<Coord, char>`, doesn't exist, because its bounds aren't met.

**The fix:** exactly what the compiler suggests:

```rust
#[derive(Debug, Eq, Hash, PartialEq)]
struct Coord {
    row: i32,
    col: i32,
}
```

**Why this is the fix:** `derive` builds these three traits from `Coord`'s own fields — because `i32` is itself `Hash` and `Eq`, `Coord` can be too, just by asking for it. (`Eq` makes no sense without `PartialEq`; the two always travel together.)

### `E0502` — a live immutable borrow, in the way of a mutable one

```text
error[E0502]: cannot borrow `scores` as mutable because it is also borrowed as immutable
  --> phase2-intermediate\01-collections\02-hashmap-in-depth\examples\09-double-borrow-get-then-insert.rs:17:9
   |
16 |     if let Some(current) = scores.get("alice") {
   |                            ------ immutable borrow occurs here
17 |         scores.insert("bob".to_string(), *current);
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
18 |         println!("alice's score is still {current}");
   |                                           ------- immutable borrow later used here

For more information about this error, try `rustc --explain E0502`.
```

**What the compiler is actually objecting to:** `current` is a `&i32` borrowed from `scores.get("alice")`. Its last use is the `println!` on the next line — so the immutable borrow stays alive until then. But the line in between, `scores.insert(...)`, needs a full mutable borrow of `scores`. A live immutable borrow plus a mutable borrow at the same time — exactly the alias rule [1.3.2](../../../phase1-fundamentals/03-borrowing-and-references/02-borrow-checker-rules/README.md) taught you, just this time the borrow comes out of a `HashMap` instead of a plain variable.

```senpai-visual
{"kind":"borrowing","labels":["get(\"alice\") -> &i32","borrow alive until println!","insert() needs &mut scores","collision"]}
```

**The fix:** pull out the value you need, once, *before* the mutable borrow:

```rust
let alice_score = *scores.get("alice").unwrap();
scores.insert("bob".to_string(), alice_score);
println!("alice's score is still {alice_score}");
```

**Why this is the fix:** `alice_score` is an `i32` — a copy, not a reference — so no borrow of `scores` is left standing. By the time that line finishes, `scores` is completely free, and `.insert()` can take its own mutable borrow without colliding with anything. This is exactly why the `entry` API exists: it does a lookup-and-decide in one step, without ever forcing you to hold a value aside by hand.

---

## Exercises

### Warm up

<details>
<summary>What does this print?</summary>

```rust
let mut m: HashMap<&str, i32> = HashMap::new();
m.insert("a", 1);
let old = m.insert("a", 2);
println!("{old:?}");
```

</details>

<details>
<summary>Answer</summary>

```text
Some(1)
```

`.insert()` always returns that key's *previous* value, wrapped in `Option`. The key was new the first time, but we only printed the second insert; by then the previous value was `1`.

</details>

<details>
<summary>Does this compile? If so, what happens when it runs?</summary>

```rust
let scores: HashMap<&str, i32> = HashMap::new();
println!("{}", scores["missing"]);
```

</details>

<details>
<summary>Answer</summary>

Yes, it compiles — the types are fine. But it panics at run time, with the message `no entry found for key`. Direct indexing (`[]`) assumes the key is there; when you aren't sure, `.get()` is what you want.

</details>

<details>
<summary>Does this compile?</summary>

```rust
let mut counts: HashMap<&str, u32> = HashMap::new();
counts.entry("a").or_insert(0) += 1;
```

</details>

<details>
<summary>Answer</summary>

No. `.or_insert(0)` gives back a `&mut u32`, not a `u32`; `+=` has to land on the number itself, behind a `*`. The error code is `E0368` — the full story is in "Errors you will meet".

</details>

<details>
<summary>Does this compile?</summary>

```rust
#[derive(Debug)]
struct Tag(String);

let mut seen: HashMap<Tag, u32> = HashMap::new();
seen.insert(Tag("x".to_string()), 1);
```

</details>

<details>
<summary>Answer</summary>

No. `Tag` only has `Debug`, not `Hash` and `Eq`. `.insert()` on `HashMap<Tag, u32>` doesn't exist at all without those two traits. The error code is `E0599`.

</details>

<details>
<summary>Does this compile?</summary>

```rust
let mut scores: HashMap<String, i32> = HashMap::new();
scores.insert("alice".to_string(), 10);

if let Some(current) = scores.get("alice") {
    scores.insert("bob".to_string(), *current);
    println!("{current}");
}
```

</details>

<details>
<summary>Answer</summary>

No. `current` is an immutable borrow of `scores` that stays alive until the `println!`; `.insert()` in between needs a mutable borrow. The error code is `E0502`.

</details>

### Repair

Fix all three broken examples:

1. `examples/07-forgot-deref-on-entry.rs` — add a `*` in the right place so `+=` lands on the number itself, not the reference.
2. `examples/08-key-missing-hash-eq.rs` — complete `Coord`'s `#[derive(...)]` so it also has `Hash` and `Eq` (and `PartialEq`).
3. `examples/09-double-borrow-get-then-insert.rs` — pull the value you need out into a separate variable (an owned `i32`, not a reference) before `.insert()`, so no borrow of `scores` is still open by the time `.insert()` is called.

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p2-01-02-hashmap-in-depth
```

No `BTreeMap`, `HashSet`, or `VecDeque` — those are [2.1.3](../03-btreemap-hashset-vecdeque/README.md). `HashMap` and the `entry` API are enough for all five.

Read `most_common` carefully: its spec asks for a tie-breaking rule (the alphabetically earlier key wins) — precisely because you read "iteration order is never guaranteed" a few pages back. Without that rule, your function might look correct on your own machine and still give a different answer between two runs.

### Build

Write a `pub fn` that counts or groups something of your own choosing — genres in an anime watchlist, HTTP status codes from a log, ingredients across a handful of recipes, anything — using the `entry` API. Pick the exact input and output shape yourself, write it down in the function's doc comment, then add at least two tests.

### Challenge (optional)

Write a `pub fn merge_counts(a: &HashMap<String, u32>, b: &HashMap<String, u32>) -> HashMap<String, u32>` that merges two count maps: every key present in either one shows up in the result, and a key present in both has its values added together. Try starting from a clone of `a` and looping only over `b` with `entry` + `and_modify` + `or_insert` — not building a map from scratch and walking both by hand.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `HashMap<K, V>` | a key-to-value map with average O(1) lookup | anywhere "do you have this key?" is the main question |
| the `entry` API | one lookup, instead of lookup-check-lookup | counting, grouping, conditional updates |
| `.or_insert(v)` | eagerly builds the default value | when the default is free (a plain number) |
| `.or_insert_with(f)` | builds the default only if actually needed | when the default is work (like `Vec::new`) |
| `.and_modify(f)` | calls `f` only if the key already existed | "change it if it's there, create it if not" |
| the `Hash` / `Eq` traits | what makes a type usable as a key | any struct or enum you want as a `HashMap` key |
| hasher | something that turns a key into a hash number | default: SipHash, resistant to hash-flooding |

### What you now know

- `HashMap<K, V>` does lookup and insert in average constant time; the cost is that it guarantees nothing about iteration order — not insertion order, not even a stable order between two runs of the same program.
- `.get()`/`.get_mut()` return the same `Option<&V>`/`Option<&mut V>` you already know from Phase 1; direct indexing (`[]`) panics on a missing key.
- The `entry` API — `or_insert`, `or_insert_with`, `and_modify` — turns a double lookup into a single one; reach for `.or_insert_with(f)` only when the default is actually work.
- `HashMap` keys must be `Hash` and `Eq`; for a struct of your own, both are one `#[derive]` away.
- A `HashMap<String, V>` can be looked up with a `&str`, without building a fresh `String` just for that one lookup.

### What comes back later

- **`BTreeMap`, `HashSet`, `VecDeque`** — [2.1.3](../03-btreemap-hashset-vecdeque/README.md)
- **Which collection to choose, and when** — [2.1.4 — Choosing a collection](../04-choosing-a-collection/README.md)
- **Closures, `Fn`/`FnMut`/`FnOnce`, and capturing the environment** — [2.2.1 — Closures and the Fn traits](../../02-iterators-and-closures/01-closures-and-fn-traits/README.md)
- **The `Borrow` trait mechanism — why `.get("literal")` works on `HashMap<String, V>`** — [2.4.3 — Deref, AsRef, Borrow](../../04-lifetimes-and-conversion/03-deref-asref-borrow/README.md)

### Can you explain?

- Why does `HashMap` guarantee nothing about iteration order, and what problem does that lack of a guarantee actually solve?
- Explain the difference between `.or_insert()` and `.or_insert_with()` with an example where the difference actually matters.
- Why does `.and_modify()` on its own never create a new key?
- Picture a type of your own that you want to use as a `HashMap` key. Which two traits does it need, and why each one?
- Why does `.get("literal")` work on a `HashMap<String, V>` without allocating a fresh `String`?

---

## Going further

- [The Rust Book — Storing Keys with Associated Values in Hash Maps](https://doc.rust-lang.org/book/ch08-03-hash-maps.html) — the same ground, from the Rust team itself.
- [`std::collections::HashMap` docs](https://doc.rust-lang.org/std/collections/struct.HashMap.html) — the full method list.
- [`std::collections::hash_map::Entry` docs](https://doc.rust-lang.org/std/collections/hash_map/enum.Entry.html) — every method the `entry` API has, not just the three you saw today.
- [The original SipHash paper](https://www.aumasson.jp/siphash/siphash.pdf) — for when you're curious why this particular hasher was chosen.
