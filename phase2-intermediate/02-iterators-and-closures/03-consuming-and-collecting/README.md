# 2.2.3 — Consuming and collecting, including `Result<Vec<_>, E>`

## At a glance

After this lesson you can:

- Explain why `.collect()` needs to be told in advance what it's building, and tell it with either a type annotation or the turbofish.
- Collect the same kind of iterator into a `Vec`, a `String`, a `HashMap`, or a `HashSet` — whichever the situation calls for.
- Choose the right consumer for a given situation: `.sum()`, `.count()`, `.min()`/`.max()`, `.find()`, `.any()`/`.all()`, or `.last()`.
- Collect an iterator of `Result`s straight into a `Result<Vec<T>, E>`, and explain why the first `Err` stops the whole collection right there.

**Time:** ~70 minutes · **Prerequisites:** [2.2.2 — Iterator adapters](../02-iterator-adapters/README.md)

---

## Why this matters

The previous lesson, [2.2.2](../02-iterator-adapters/README.md), ended on a line you might have skimmed past: iterator adapters — `.map()`, `.filter()`, `.enumerate()`, and the rest — are lazy, and none of them do anything until something actually consumes them. That "something" is this whole lesson.

There's an older debt, too. [1.7.2](../../../phase1-fundamentals/07-putting-it-together/02-phase-review/README.md) — the Phase 1 review — put a line of `.filter()` and `.collect()` over a range in front of you and said, flatly, "these come from Phase 2, module 2.2, and you haven't formally learned them yet." Now you have.

And there's a quieter question. In [1.6.3](../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.md), the "Build" exercise asked you to write `pub fn parse_all(inputs: &[&str]) -> Result<Vec<i32>, String>`: parse every string by hand, in a loop, with `?` — and if one failed, return right there with that same error, without trying the rest. If you actually wrote that exercise, you probably thought, at some point, "this loop feels long for such a common pattern — there has to be a shorter way." There was. It's here.

If you're coming from Python: you'd usually build a list with `list(...)` or `[... for ...]`, a dict with `dict(...)` or `{...: ... for ...}`, and a set with `set(...)` — each target has its own function or syntax, and that name or syntax alone tells you what's being built. Rust does all of this with one method: `.collect()`. The method name by itself gives no hint of the target — that's exactly what the first part of this lesson dwells on. And for "process everything, but stop at the first failure," Python usually needs an explicit loop wrapped in `try`/`except`; in Rust the same job is one `.collect()` — the centerpiece of this lesson.

---

## The concept

### Consumers: what actually pulls the pipeline

In [2.2.2](../02-iterator-adapters/README.md) you saw that `.filter()`, `.map()`, and the rest — the ones we call adapters — do nothing by themselves; they only describe a pipeline. What actually pulls that pipeline through and does the work is a **consuming adapter**. `.collect()` is the most famous one, but it's only one of several — the same iterator can be handed to several different consumers, depending on what you actually need at the end:

```rust
let numbers = vec![1, 2, 3, 4, 5, 6];

let evens: Vec<i32> = numbers.iter().filter(|&&n| n % 2 == 0).copied().collect();
let even_count = numbers.iter().filter(|&&n| n % 2 == 0).count();
let even_sum: i32 = numbers.iter().filter(|&&n| n % 2 == 0).sum();

println!("evens: {evens:?}");
println!("count: {even_count}");
println!("sum:   {even_sum}");
```

```text
evens: [2, 4, 6]
count: 3
sum:   12
```

The same `.filter()` is written three times because each time a different consuming method is going to be called on it — once an iterator has been consumed it's spent; you can't reuse the same value for the next method. The rest of this lesson is about which consumer to reach for, and when.

### `.collect()` needs to know what it's building

Among all the consumers, `.collect()` has one fundamental difference from the rest: `.sum()`, `.count()`, `.find()`, and similar methods each produce exactly one kind of output — `.count()` always gives back a `usize`, no matter which iterator you call it on. But `.collect()` can build several completely different things from the same iterator, depending on what you ask for: a `Vec`, a `HashMap`, a `String`, even — as you'll see by the end of this lesson — a `Result`. Because of that, the compiler can't guess the target just from seeing `.collect()`; you have to tell it, explicitly, one of two ways:

```rust
let long_runs: Vec<i32> = episodes.iter().filter(|&&n| n >= 12).copied().collect();
println!("long_runs (annotated binding): {long_runs:?}");

let long_runs_turbofish = episodes
    .iter()
    .filter(|&&n| n >= 12)
    .copied()
    .collect::<Vec<i32>>();
println!("long_runs (turbofish):        {long_runs_turbofish:?}");
```

```text
long_runs (annotated binding): [12, 24, 50, 13]
long_runs (turbofish):        [12, 24, 50, 13]
```

(With `episodes = [12, 24, 6, 50, 13]`.) The first form writes the type on the binding, and Rust works backwards from there to figure out what `.collect()` must build. The second form — the same turbofish you know from [1.6.3](../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.md) — writes that same type right at the call site instead, for when the value is used immediately and there's no separate `let` to hang a type off of. You can even write only part of the type: `collect::<Vec<_>>()` also works, because Rust can read the element type off `episodes` itself; it only needs to know which container — `Vec`, not something else. Leave out both, and the compiler gets stuck — that's one of the entries in "Errors you will meet."

This ability of `.collect()` to build several different things from the same iterator isn't magic — a standard trait called `FromIterator` sits behind it. Any type that wants to be a target of `.collect()` has to implement `FromIterator`; `Vec`, `String`, `HashMap`, `HashSet`, and — the centerpiece of this lesson — `Result` itself, all do. You'll see the first four right now.

### Collecting into a `String`

When each element is a `char` or a `&str`, `.collect()` can build a `String` directly — with nothing inserted between pieces, just plain concatenation:

```rust
let letters = ['R', 'u', 's', 't'];
let word: String = letters.into_iter().collect();
println!("chars -> String:   {word}");

let parts = ["Sen", "pai"];
let shout: String = parts.into_iter().collect();
println!("&str parts -> String: {shout}");
```

```text
chars -> String:   Rust
&str parts -> String: Senpai
```

If you also want a separator between pieces — a space between words, say — that isn't `.collect()`'s job anymore; you want `.join(" ")` on the slice itself, not on the iterator. `.collect()` only concatenates.

### Collecting into a `HashMap`

When each element is a `(key, value)` tuple, `.collect()` can build a `HashMap<K, V>` — the same thing you did in [2.1.2](../../01-collections/02-hashmap-in-depth/README.md) with `HashMap::new()` and a loop of `.insert()`s, this time in one line:

```rust
let entries = [("Frieren", 28), ("Bocchi the Rock", 12), ("K-On!", 13)];

let episodes: HashMap<&str, u32> = entries.into_iter().collect();
println!("Frieren episodes:  {:?}", episodes.get("Frieren"));
println!("total shows:       {}", episodes.len());
```

```text
Frieren episodes:  Some(28)
total shows:       3
```

If a key repeats, you get exactly the behavior you'd get from calling `.insert()` that many times in a row: the value from whichever pair held that key *last* wins — the rest are silently overwritten.

### Collecting into a `HashSet`

When you only want to know "which distinct values showed up here," not how many times each one repeated, `HashSet<T>` does exactly what `Vec<T>` did — with one difference: it quietly drops a repeat instead of keeping it:

```rust
let tags = ["comedy", "drama", "comedy", "slice of life", "drama", "comedy"];
println!("tags seen (with repeats): {}", tags.len());

let unique: HashSet<&str> = tags.into_iter().collect();
println!("unique tags:               {}", unique.len());
println!("contains \"drama\":          {}", unique.contains("drama"));
```

```text
tags seen (with repeats): 6
unique tags:               3
contains "drama":          true
```

You didn't write any dedup logic by hand — the same `Hash`/`Eq` traits that [2.1.2](../../01-collections/02-hashmap-in-depth/README.md) explained for `HashMap` keys are doing the same work here, behind the scenes.

### Other consumers: a quick tour

Not every consumer builds a fresh collection. When the final output is a number, a `bool`, or just one item — not a new container — one of these is a more direct match than `.collect()`:

```rust
let ratings = [7, 9, 5, 10, 6];

println!("sum:              {}", ratings.iter().sum::<i32>());
println!("count:            {}", ratings.iter().filter(|&&r| r >= 7).count());
println!("min:              {:?}", ratings.iter().min());
println!("max:              {:?}", ratings.iter().max());
println!("first below 6:    {:?}", ratings.iter().find(|&&r| r < 6));
println!("all at least 5:   {}", ratings.iter().all(|&r| r >= 5));
println!("any perfect 10:   {}", ratings.iter().any(|&r| r == 10));
println!("last:             {:?}", ratings.iter().last());
```

```text
sum:              37
count:            3
min:              Some(5)
max:              Some(10)
first below 6:    Some(5)
all at least 5:   true
any perfect 10:   true
last:             Some(6)
```

Something worth noticing about `.find()`: it returns the *first* item that satisfies the condition, not the smallest or largest one — the same job you did by hand with a loop in [1.1.5](../../../phase1-fundamentals/01-foundations/05-control-flow/README.md) (`index_of_first_negative`), now a method call. `.min()` and `.max()` both return `Option<&T>`, because an empty iterator has neither a minimum nor a maximum — the same `Option` rule you already know from Phase 1.

### The centerpiece: collecting `Result`s into a `Result<Vec<T>, E>`

Every iterator you've collected so far has had "healthy" items — numbers, strings, tuples. But there's a very common pattern: you have a list of strings, you want to parse each one, and if they all succeed you want a `Vec` of the results — but if even one fails, the whole thing should fail, with that first error. That's exactly what you wrote by hand in [1.6.3](../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.md)'s "Build" exercise — a loop, a `?`, an early return on the first `Err`.

Now the same job, in one line:

```rust
let all_good = ["1", "2", "3"];
let parsed: Result<Vec<i32>, _> = all_good.iter().map(|s| s.parse::<i32>()).collect();
println!("all valid:   {parsed:?}");
```

```text
all valid:   Ok([1, 2, 3])
```

Each `s.parse::<i32>()` gives back a `Result<i32, ParseIntError>` — so the iterator reaching `.collect()` has `Result` items, not `i32` ones. And yet the type asked for is `Result<Vec<i32>, _>` — a `Result` wrapped around a `Vec`, not a `Vec` of `Result`s. This isn't magic: `Result<T, E>` itself, exactly like `Vec` and `HashMap`, implements the `FromIterator` trait — an implementation that does precisely what you wrote by hand in 1.6.3: it reads each item; if it's `Ok`, it keeps the value and moves on; the moment it hits the first `Err`, it stops right there and that `Err` becomes the whole answer. If every item was `Ok` all the way through, you get back a single `Ok` wrapping a `Vec` of every unwrapped value.

The practical effect is **short-circuiting** — and you can watch it happen, not just take it on faith:

```rust
let has_a_bad_one = ["1", "x", "3"];
let parsed: Result<Vec<i32>, _> = has_a_bad_one
    .iter()
    .map(|s| {
        println!("  parsing {s:?}...");
        s.parse::<i32>()
    })
    .collect();
println!("one invalid: {parsed:?}");
```

```text
  parsing "1"...
  parsing "x"...
one invalid: Err(ParseIntError { kind: InvalidDigit })
```

Look at the `println!` inside the closure: `"3"` is never parsed. `.collect()` stopped pulling from the iterator the instant `"x"` produced an `Err` — because `.map()` is lazy (as you remember from [2.2.2](../02-iterator-adapters/README.md)), its closure never runs until something calls `.next()`, and `.collect()` never calls `.next()` again after it has seen a first `Err`.

```senpai-visual
{"kind":"result","labels":["[\"1\", \"x\", \"3\"]","\"1\" -> Ok","\"x\" -> Err","stop immediately","output: Err(e)"]}
```

If you wrote 1.6.3, this is exactly the quiet question that might have crossed your mind while writing that hand-rolled loop — "there has to be a shorter way" — and now you know why it works, not just that it does.

---

## Hands on

```sh
cargo run -p p2-02-03-consuming-and-collecting --example 01-collect-into-vec
cargo run -p p2-02-03-consuming-and-collecting --example 02-collect-into-string
cargo run -p p2-02-03-consuming-and-collecting --example 03-collect-into-hashmap
cargo run -p p2-02-03-consuming-and-collecting --example 04-collect-into-hashset
cargo run -p p2-02-03-consuming-and-collecting --example 05-other-consumers
cargo run -p p2-02-03-consuming-and-collecting --example 06-collecting-results
```

Then the three broken ones:

```sh
cargo run -p p2-02-03-consuming-and-collecting --example 07-collect-ambiguous-type --features broken
cargo run -p p2-02-03-consuming-and-collecting --example 08-collect-result-into-vec-directly --features broken
cargo run -p p2-02-03-consuming-and-collecting --example 09-max-on-floats --features broken
```

Then try these:

1. In `03-collect-into-hashmap`, add a fourth tuple — `("Frieren", 99)` — to the end of `entries`. What does `episodes.get("Frieren")` come out to now?
2. In `04-collect-into-hashset`, change only `unique`'s type from `HashSet<&str>` to `Vec<&str>` (leave `.collect()` itself untouched). Does `unique.len()` now equal `tags.len()` or not? Why?
3. In `06-collecting-results`, change the order of `has_a_bad_one` to `["1", "3", "x"]` — put the bad string last. Which strings do you now see printed as "parsing ..."?

---

## Errors you will meet

### `E0283` — the compiler doesn't know what `.collect()` should build

```text
error[E0283]: type annotations needed
    --> phase2-intermediate\02-iterators-and-closures\03-consuming-and-collecting\examples\07-collect-ambiguous-type.rs:10:9
     |
  10 |     let evens = (1..10).filter(|n| n % 2 == 0).collect();
     |         ^^^^^                                  ------- type must be known at this point
     |
     = note: the type must implement `FromIterator<i32>`
note: required by a bound in `collect`
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\iter\traits\iterator.rs:2077:19
     |
2077 |     fn collect<B: FromIterator<Self::Item>>(self) -> B
     |                   ^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Iterator::collect`
help: consider giving `evens` an explicit type
     |
  10 |     let evens: Vec<_> = (1..10).filter(|n| n % 2 == 0).collect();
     |              ++++++++

For more information about this error, try `rustc --explain E0283`.
```

**What the compiler is actually objecting to:** the `= note` line says exactly what "The concept" already told you: the type of `evens` has to implement `FromIterator<i32>` — but no type is "the" default for that; it could be `Vec<i32>`, `HashSet<i32>`, or anything else implementing the same trait. The compiler can't guess, and says so.

**The fix:** either of the two ways you saw in "The concept":

```rust
let evens: Vec<i32> = (1..10).filter(|n| n % 2 == 0).collect();
```

```text
[2, 4, 6, 8]
```

**Why this is the fix:** `evens`'s type is now explicit, so Rust knows exactly which `FromIterator` implementation to call. The turbofish form — `.collect::<Vec<i32>>()` — would do the same job; only where you write the type differs.

### `E0277` — a `Vec<i32>` can't be built from an iterator of `Result`s

```text
error[E0277]: a value of type `Vec<i32>` cannot be built from an iterator over elements of type `Result<i32, ParseIntError>`
    --> phase2-intermediate\02-iterators-and-closures\03-consuming-and-collecting\examples\08-collect-result-into-vec-directly.rs:13:68
     |
  13 |     let parsed: Vec<i32> = inputs.iter().map(|s| s.parse::<i32>()).collect();
     |                                                                    ^^^^^^^ value of type `Vec<i32>` cannot be built from `std::iter::Iterator<Item=Result<i32, ParseIntError>>`
     |
help: the trait `FromIterator<Result<i32, ParseIntError>>` is not implemented for `Vec<i32>`
      but trait `FromIterator<i32>` is implemented for it
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\alloc\src\vec\mod.rs:3923:1
     |
3923 | impl<T> FromIterator<T> for Vec<T> {
     | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     = help: for that trait implementation, expected `i32`, found `Result<i32, ParseIntError>`
note: the method call chain might not have had the expected associated types
    --> phase2-intermediate\02-iterators-and-closures\03-consuming-and-collecting\examples\08-collect-result-into-vec-directly.rs:13:42
     |
  12 |     let inputs = ["1", "2", "x"];
     |                  --------------- this expression has type `[&str; 3]`
  13 |     let parsed: Vec<i32> = inputs.iter().map(|s| s.parse::<i32>()).collect();
     |                                   ------ ^^^^^^^^^^^^^^^^^^^^^^^^^ `Iterator::Item` changed to `Result<i32, ParseIntError>` here
     |                                   |
     |                                   `Iterator::Item` is `&&str` here
note: required by a bound in `collect`
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\iter\traits\iterator.rs:2077:19
     |
2077 |     fn collect<B: FromIterator<Self::Item>>(self) -> B
     |                   ^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Iterator::collect`

For more information about this error, try `rustc --explain E0277`.
```

**What the compiler is actually objecting to:** this is longer than the errors you've seen so far, but the headline is the very first line: a `Vec<i32>` cannot be built from an iterator whose items are `Result<i32, ParseIntError>`. `s.parse::<i32>()` returns a `Result`, not a raw `i32`; so after `.map()`, the iterator's items are `Result<i32, ParseIntError>`. The first `help` says exactly that: `Vec<i32>` only implements `FromIterator<i32>` — not `FromIterator<Result<i32, ParseIntError>>`.

**The fix:** instead of asking for `Vec<i32>`, ask for what this iterator can actually build — a `Result` wrapped around a `Vec`:

```rust
let parsed: Result<Vec<i32>, _> = inputs.iter().map(|s| s.parse::<i32>()).collect();
```

```text
Err(ParseIntError { kind: InvalidDigit })
```

(With `inputs = ["1", "2", "x"]`.)

**Why this is the fix:** the requested type now matches the iterator's item type exactly — `Result<Vec<i32>, ParseIntError>` implements the trait that's actually needed: `FromIterator<Result<i32, ParseIntError>>`. This is exactly the mechanism from "The concept," seen this time from the error side.

### `E0277` — `.max()` needs a total order `f64` doesn't promise

```text
error[E0277]: the trait bound `{float}: Ord` is not satisfied
    --> phase2-intermediate\02-iterators-and-closures\03-consuming-and-collecting\examples\09-max-on-floats.rs:11:33
     |
  11 |     let biggest = values.iter().max();
     |                                 ^^^ the trait `Ord` is not implemented for `{float}`
     |
     = help: the following other types implement trait `Ord`:
               i128
               i16
               i32
               i64
               i8
               isize
               u128
               u16
             and 4 others
     = note: required for `&{float}` to implement `Ord`
note: required by a bound in `std::iter::Iterator::max`
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\iter\traits\iterator.rs:3253:21
     |
3250 |     fn max(self) -> Option<Self::Item>
     |        --- required by a bound in this associated function
...
3253 |         Self::Item: Ord,
     |                     ^^^ required by this bound in `Iterator::max`

For more information about this error, try `rustc --explain E0277`.
```

**What the compiler is actually objecting to:** `.max()` (with no argument) only works for types that have one unambiguous, total order — for any two values, there's always exactly one answer to "which is bigger?". That guarantee has an official name, a trait called `Ord`, and `f64` doesn't implement it — because of `NaN` ("not a number"), which cannot be compared to any other number, not even itself. (`Ord` and its family belong entirely to [2.3.4](../../03-traits-and-generics/04-standard-derives-by-hand/README.md); one paragraph today is enough.)

**The fix:** give it an explicit comparison instead of waiting on `Ord`:

```rust
let values: Vec<f64> = vec![1.0, 5.5, 2.3];
let biggest = values.iter().max_by(|a, b| a.total_cmp(b));
println!("{biggest:?}");
```

```text
Some(5.5)
```

**Why this is the fix:** `.max_by()` has you write the comparison yourself, so it no longer waits on `Ord`. `f64::total_cmp` was built for exactly this — a total order over every `f64` value, `NaN` included. (This is the same `total_cmp` you saw in [2.1.1](../../01-collections/01-vec-depth/README.md) for `.sort_by()` — same tool, this time for `.max_by()`.)

---

## Exercises

### Warm up

<details>
<summary>Does this compile? If so, what does it print?

```rust
let v = vec![1, 2, 3];
let doubled = v.iter().map(|n| n * 2);
println!("{doubled:?}");
```
</summary>

```text
Map { iter: Iter([1, 2, 3]) }
```

Yes, it compiles — `doubled` is only a description of a pipeline, the iterator's own internal `Map` struct, with none of the doubled numbers inside it. `{:?}` shows that struct, not whatever you'd get from actually consuming it. This is exactly the laziness [2.2.2](../02-iterator-adapters/README.md) showed you — even printing it doesn't run `.map()`.

</details>

<details>
<summary>Does this compile?

```rust
let names = vec!["a", "b"];
let joined = names.into_iter().collect();
println!("{joined}");
```
</summary>

No. `.collect()` doesn't know whether to build a `String`, a `Vec<&str>`, or something else — the error code is `E0283`; the full story is in "Errors you will meet".

</details>

<details>
<summary>What does this print?

```rust
let scores = [3, 7, 2, 9];
println!("{:?}", scores.iter().find(|&&s| s > 5));
```
</summary>

```text
Some(7)
```

`.find()` returns the first item satisfying the condition, not the largest one — `7` comes before `9`.

</details>

<details>
<summary>What does this print?

```rust
let unique: std::collections::HashSet<char> = "hello".chars().collect();
println!("{}", unique.len());
```
</summary>

```text
4
```

`"hello"` has five characters, but `'l'` repeats twice; a `HashSet` only keeps the distinct values: `h`, `e`, `l`, `o`.

</details>

<details>
<summary>Does this compile?

```rust
let values = vec![1.0, 5.5, 2.3];
let biggest = values.iter().max();
```
</summary>

No. `f64` doesn't implement `Ord` (because of `NaN`), and `.max()` with no argument requires `Ord`. The error code is `E0277`; the full story is in "Errors you will meet".

</details>

<details>
<summary>What does this print?

```rust
let inputs = ["10", "abc", "30"];
let result: Result<Vec<i32>, _> = inputs.iter().map(|s| s.parse::<i32>()).collect();
match result {
    Ok(values) => println!("ok: {values:?}"),
    Err(_) => println!("err"),
}
```
</summary>

```text
err
```

`"abc"` fails to parse, so the whole `.collect()` returns an `Err` — even though `"10"` before it and `"30"` after it are both valid.

</details>

### Repair

Fix all three broken examples:

1. `examples/07-collect-ambiguous-type.rs` — add a type annotation or turbofish so `.collect()` knows what to build.
2. `examples/08-collect-result-into-vec-directly.rs` — change `parsed`'s type from `Vec<i32>` to `Result<Vec<i32>, _>`.
3. `examples/09-max-on-floats.rs` — make `values`'s type explicitly `Vec<f64>`, and replace `.max()` with `.max_by(|a, b| a.total_cmp(b))`.

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p2-02-03-consuming-and-collecting
```

Every one of them can be written as a single `.collect()` (plus whatever adapter it needs) — no manual loop, no `HashMap::new()` plus a loop of `.insert()`s. The last one, `parse_all`, is the exact function you wrote in [1.6.3](../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.md) with a loop and `?` — same signature, same spec; write it as a one-liner this time.

### Build

Write a `pub fn` that works on something of your own choosing — scores from an anime watchlist, lines from a log file, anything — and uses both of these: (1) `.collect()` into one of today's target types, and (2) at least one of the other consumers (`.sum()`, `.count()`, `.min()`/`.max()`, `.find()`, `.any()`/`.all()`, `.last()`). Pick the exact input and output shape yourself, write it down in the function's doc comment, then add at least two tests.

### Challenge (optional)

**Part one.** Search the Rust standard docs for `FromIterator`. Does `Option<T>` implement it too? Guess what `collect::<Option<Vec<T>>>()` returns for an iterator of `Option`s with a `None` somewhere in the middle — then write a small function and try it to check your guess.

**Part two.** Now try `collect::<Result<HashSet<i32>, String>>()` on an iterator of `Result`s. Does the same short-circuiting work for a different target type? Why or why not?

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| consuming adapter | a method that pulls the pipeline through and produces a final, non-iterator result | `.collect()`, `.sum()`, `.count()`, and the rest |
| `.collect()` | the general-purpose consumer; from the same iterator it can build several different types | anywhere the final output is a `Vec`, `String`, `HashMap`, `HashSet`, or `Result` |
| the `FromIterator` trait | the contract `.collect()` is generic over | any type that is a target of `.collect()` has to implement it |
| turbofish / type annotation | the two ways to tell `.collect()` what to build | whenever the target isn't inferable on its own |
| `Result<Vec<T>, E>` from `.collect()` | collecting an iterator of `Result`s straight into one `Result` | replaces a manual loop plus `?` |
| short-circuiting | stopping as soon as the final answer is already known | the first `Err` in collecting `Result`s |

### What you now know

- `.collect()` is the general-purpose consumer; because it can build several different types from the same iterator, you have to tell it what you mean with a turbofish or a type annotation.
- The same iterator can be collected into a `Vec`, into a `String` (from `char` or `&str`, by plain concatenation), into a `HashMap` (from key-value tuples, last write wins), or into a `HashSet` (with automatic deduplication).
- `.sum()`, `.count()`, `.min()`/`.max()`, `.find()`, `.any()`/`.all()`, and `.last()` are more direct consumers, for when the final output is a number, a `bool`, or one item — not a fresh collection.
- `Result<T, E>`, like `Vec` and `HashMap`, implements the `FromIterator` trait; an iterator of `Result`s can be collected straight into a `Result<Vec<T>, E>`.
- That collection short-circuits: the first `Err` stops it right there and becomes the whole answer; if every item was `Ok`, you get back a single `Ok` holding a `Vec` of every unwrapped value.

### What comes back later

- **Implementing `Iterator` and `IntoIterator` for your own type** — [2.2.4](../04-implementing-iterator/README.md)
- **Laziness and iterator chain performance, in depth** — [2.2.5](../05-laziness-and-performance/README.md)
- **The `Ord`/`PartialOrd` traits and the standard derive family (the one behind the `f64` error today)** — [2.3.4](../../03-traits-and-generics/04-standard-derives-by-hand/README.md)

### Can you explain?

- Why does `.collect()` need a turbofish or a type annotation, when `.sum()` or `.count()` need neither?
- Name three different types you could collect a `(word, count)` iterator into, and for each one say what `.collect()` builds differently.
- Explain collecting an iterator of `Result`s using the word "short-circuiting" — exactly when does it stop, and what does it return?
- Why doesn't `.max()` compile on an iterator of `f64`, but it does on an iterator of `i32`?
- When a key repeats while collecting into a `HashMap`, which value survives?

---

## Going further

- [The Rust Book — Processing a Series of Items with Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html) — the same ground, the consuming-adapters section.
- [`std::iter::FromIterator` docs](https://doc.rust-lang.org/std/iter/trait.FromIterator.html) — the list of standard types that implement it.
- [`std::result::Result` docs](https://doc.rust-lang.org/std/result/enum.Result.html) — where you can see `Result`'s own `FromIterator` implementation, straight from the standard library.
- [`std::iter::Iterator` docs](https://doc.rust-lang.org/std/iter/trait.Iterator.html) — the full list of consumers; far more than the nine you saw today.
