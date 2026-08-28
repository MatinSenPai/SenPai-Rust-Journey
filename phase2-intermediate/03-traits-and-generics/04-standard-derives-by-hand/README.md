# 2.3.4 — The standard derives, implemented by hand

## At a glance

After this lesson you can:

- Explain exactly what `#[derive(Debug)]` generates — by writing the same thing twice, once with `f.debug_struct(...).field(...).finish()` and once with a raw `write!` — and say why `#[derive(Display)]` does not exist at all.
- For a type of your own, build `Default`, `PartialEq`/`Eq`, and `PartialOrd`/`Ord` both with `#[derive]` and by hand, and say precisely when those two paths land on the same result and when they don't.
- Explain why `f64` can never be `Eq` or `Ord`, and write a hand-rolled `Hash` that stays consistent with your `Eq` — because if it doesn't, `HashMap`/`HashSet` breaks silently.

**Time:** ~100 minutes · **Prerequisites:**
[2.3.3 — `From`, `Into`, `TryFrom`, `TryInto`](../03-from-into-tryfrom/README.md)

---

## Why this matters

Three times this phase, you have walked straight into this exact wall — and three times the lesson said the same sentence and moved on.

In [2.1.1](../../01-collections/01-vec-depth/README.md), `ratings.sort()` on a `Vec<f64>` refused to compile at all; the compiler said `f64` doesn't implement `Ord` (`E0277`). In [2.1.3](../../01-collections/03-btreemap-hashset-vecdeque/README.md), the same story got one notch more serious: `BinaryHeap<f64>` never even got off the ground (`E0599`) — a priority queue has to be able to compare any two elements, and `f64` won't make that promise. In [2.2.3](../../02-iterators-and-closures/03-consuming-and-collecting/README.md), it was `.max()` on an iterator of `f64` — same `Ord`, same `E0277`. And in [2.1.2](../../01-collections/02-hashmap-in-depth/README.md), a `struct Coord` carrying nothing but `#[derive(Debug)]` hit a related wall on `HashMap::insert`: `Coord: Eq` and `Coord: Hash` were both unsatisfied (`E0599`) — and the fix was a `#[derive(Eq, Hash, PartialEq)]` the compiler itself suggested, without you ever really knowing what those three words promise together.

Every one of those four times, the lesson's answer was an honest deferral, not an explanation: "2.3.4 covers this properly." Today is the day that deferral gets paid off — and the `NaN`-breaks-`Ord` story, this time, isn't a new fact. It's the answer to a question you've now asked four times.

There's a quieter, more everyday thread running under this one too. Since [1.5.1](../../../phase1-fundamentals/05-your-own-types/01-structs-and-methods/README.md), you've put `#[derive(Debug)]` above nearly every struct you've written, without once asking what that single line actually generates. If the compiler hands you that implementation for free, why learn to write it by hand? Because some of the six are never that obvious (`Display` is never derived; you'll see why), because sometimes you want different behavior than the derive's default (`Ord` by one field, not all of them), and because when a `HashSet` starts silently keeping duplicates, the only way to understand why is to know exactly what that `derive` promised in the first place.

---

## The concept

### The `#[derive(Debug)]` you've typed a hundred times — today you write it

One small type to work with:

```rust
#[derive(Debug)]
struct Anime {
    title: String,
    episodes: u32,
    score: u8,
}

let a = Anime { title: "Frieren".to_string(), episodes: 28, score: 96 };
println!("{a:?}");
```

```text
Anime { title: "Frieren", episodes: 28, score: 96 }
```

You've seen this a hundred times. What you haven't seen is that `#[derive(Debug)]` writes exactly this, as an `impl fmt::Debug`, itself. With the same ergonomic builder the derive uses under its own hood — `f.debug_struct(name).field(...).finish()` — you can build the identical thing by hand:

```rust
use std::fmt;

impl fmt::Debug for Anime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Anime")
            .field("title", &self.title)
            .field("episodes", &self.episodes)
            .field("score", &self.score)
            .finish()
    }
}
```

`f: &mut fmt::Formatter` is the buffer the output text gets written into; `fmt::Result` means "writing either succeeded or it didn't" — the same `Result` you know from [1.6.3](../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.md). `debug_struct("Anime")` says "this is a struct named Anime," each `.field(...)` adds one field, and `.finish()` closes it out. Line for line, the result matches what the derive produced.

That's not the only way, though. You can build the same text with a raw `write!`, no builder at all:

```rust
impl fmt::Debug for Anime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Anime {{ title: {:?}, episodes: {:?}, score: {:?} }}",
            self.title, self.episodes, self.score
        )
    }
}
```

(Two braces in a row — `{{` and `}}` — mean "print a literal brace," not a placeholder. You need it here because the struct's own `{` and `}` would otherwise look like more `format!` placeholders.)

Both give the same answer for `{:?}`. The difference shows up the moment you ask for the alternate form, `{:#?}`:

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 01-debug-two-ways
```

```text
compact — derived: Derived { title: "Frieren", episodes: 28, score: 96 }
compact — builder: Builder { title: "Frieren", episodes: 28, score: 96 }
compact — raw:     Raw { title: "Frieren", episodes: 28, score: 96 }

alternate — derived:
Derived {
    title: "Frieren",
    episodes: 28,
    score: 96,
}
alternate — builder:
Builder {
    title: "Frieren",
    episodes: 28,
    score: 96,
}
alternate — raw:
Raw { title: "Frieren", episodes: 28, score: 96 }
```

The `builder` version indents itself across several lines under `{:#?}`. The `raw` version doesn't — the same compact single line, even though you explicitly asked for the alternate form. `debug_struct` gives you that support for free, because it already knows whether `f.alternate()` was requested; a hand-rolled `write!` has no idea unless you check that yourself. This is exactly what makes the builder the right choice for a real struct — not just less typing.

### `Display` — why there is no `#[derive]` for it

`{:?}` asks "what does the programmer's shape look like?" — the question [1.4.3](../../../phase1-fundamentals/04-text-and-strings/03-building-and-transforming-strings/README.md) introduced. `{}` asks something else: "what does the *user's* shape look like?" — and the compiler can never guess that one. For `Debug`, the answer is mechanical: field name, colon, value, the same shape for every type. For `Display`, the answer is a human decision: how should `Anime` read to a viewer? The title alone? The title plus the score? Because of that, `Display` is **never** derived — for every type that actually needs one, you write it by hand:

```rust
impl fmt::Display for Anime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} — {} episodes, {}/100", self.title, self.episodes, self.score)
    }
}
```

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 02-display-hand-written
```

```text
Display (for a viewer):    Frieren — 28 episodes, 96/100
Debug   (for a developer): Anime { title: "Frieren", episodes: 28, score: 96 }
```

Same value, two completely different sentences — and that is exactly the point. `Debug` and `Display` are two separate traits because they serve two separate audiences: one is always built mechanically from the fields, the other is always built from a decision.

### `Default` — a sensible starting point

`Default` answers "what is a sensible starting value for this type?" `#[derive(Default)]` builds that answer from the fields — as long as *every* field is itself `Default` (an empty `String`, zero for `u32`/`u8`):

```rust
#[derive(Debug, Default)]
struct Anime {
    title: String,
    episodes: u32,
    score: u8,
}

let blank = Anime::default();
println!("{blank:?}");
```

```text
Anime { title: "", episodes: 0, score: 0 }
```

By hand it's just as direct — you spell out each field yourself:

```rust
impl Default for AnimeManual {
    fn default() -> Self {
        AnimeManual { title: String::new(), episodes: 0, score: 0 }
    }
}
```

Where `Default` earns its keep is **struct update syntax** — the same `..other` from [1.5.1](../../../phase1-fundamentals/05-your-own-types/01-structs-and-methods/README.md), except this time the right-hand side isn't a value you built, it's `Default::default()` itself:

```rust
let started = Anime {
    title: "Bocchi the Rock!".to_string(),
    ..Default::default()
};
```

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 03-default-and-struct-update
```

```text
derived default:      Anime { title: "", episodes: 0, score: 0 }
hand-written default:  title="" episodes=0 score=0
struct-update syntax:  Anime { title: "Bocchi the Rock!", episodes: 0, score: 0 }
```

You spelled out `title`; `Default::default()` filled in `episodes` and `score`. For a struct with several fields where most of them usually want the same starting value, this combination is exactly what saves you from repeating yourself.

### `PartialEq` and `Eq` — structural equality, and the extra promise `Eq` makes

`#[derive(PartialEq)]` builds an `==` that compares field by field — every field has to match for the whole value to match:

```rust
#[derive(Debug, PartialEq, Eq)]
struct Anime {
    title: String,
    episodes: u32,
    score: u8,
}

let a = Anime { title: "Frieren".to_string(), episodes: 28, score: 96 };
let b = Anime { title: "Frieren".to_string(), episodes: 28, score: 96 };
let c = Anime { title: "Frieren".to_string(), episodes: 28, score: 95 };
println!("a == b: {}", a == b);
println!("a == c: {}", a == c);
```

```text
a == b: true
a == c: false
```

Writing it by hand is nothing surprising — an `eq` function spelling out the same comparison:

```rust
impl PartialEq for Anime {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title && self.episodes == other.episodes && self.score == other.score
    }
}
```

Now the question that matters: `#[derive(PartialEq, Eq)]` above added **two** traits, not one. Look at `impl Eq for Anime {}` — its body is empty. `Eq` needs no new function at all; it is only a **marker trait**, and the extra thing it promises is **reflexivity** — that every value always equals itself, `x == x`, no exceptions. `PartialEq` does not promise that; it only says that when two values *are* equal, that equality behaves the ordinary way (symmetric, transitive) — not that *every* value is guaranteed to equal itself.

This is exactly where `f64` falls out of step:

```rust
let nan = f64::NAN;
println!("f64::NAN == f64::NAN: {}", nan == nan);
println!("1.0_f64 == 1.0_f64:   {}", 1.0_f64 == 1.0_f64);
```

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 04-partialeq-eq-and-nan
```

```text
a == b (every field matches): true
a == c (score differs):       false
f64::NAN == f64::NAN:         false
1.0_f64 == 1.0_f64:           true
```

(The first two lines come from the file's earlier `a`/`b`/`c` comparisons — the same ordinary `PartialEq` you already saw work correctly above. The point is entirely in the last two.)

`NaN` ("not a number" — the result of something like `0.0 / 0.0`) is not equal to any value, including itself — that rule comes from the IEEE 754 standard, not from Rust. So `f64` is not reflexive, and the standard library honestly does not implement `Eq` for it — only `PartialEq`. If you try to slap `#[derive(Eq)]` on a struct with an `f64` field yourself, the compiler tells you exactly this with an error; the full transcript is in "Errors you will meet."

### `PartialOrd` and `Ord` — the same story, one level up

`#[derive(PartialOrd, Ord)]` compares fields in **declaration order** — exactly like comparing tuples: the first field first, and only if that ties does it move to the next:

```rust
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct AnimeByFields {
    title: String,
    episodes: u32,
    score: u8,
}

let mut by_fields = vec![
    AnimeByFields { title: "Frieren".into(), episodes: 28, score: 96 },
    AnimeByFields { title: "Bocchi".into(), episodes: 12, score: 90 },
];
by_fields.sort();
```

```text
derived Ord (title, then episodes, then score):
  AnimeByFields { title: "Bocchi", episodes: 12, score: 90 }
  AnimeByFields { title: "Frieren", episodes: 28, score: 96 }
```

`"Bocchi"` sorts before `"Frieren"` because `title` is the first field — not because its score is lower. If you want a more meaningful order — say, "by score" — you write `cmp` yourself:

```rust
impl Ord for AnimeByScore {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}
```

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 05-partialord-ord-and-nan
```

```text
hand-written Ord (score only):
  AnimeByScore { title: "Frieren", episodes: 28, score: 96 }
  AnimeByScore { title: "Bocchi", episodes: 12, score: 98 }
```

(You still have to write `PartialOrd::partial_cmp`, but once you have `cmp`, it's always the same one line: `Some(self.cmp(other))` — never anything else.)

Now, the answer to those three walls. `Ord` makes exactly the same kind of extra promise that `Eq` made, one level up: a **total order** — for *any* two values, there is always a definite answer to "which is smaller?" `PartialOrd` does not promise that; its `partial_cmp` is allowed to return `None` — "these two values simply aren't comparable":

```rust
let nan = f64::NAN;
println!("1.0.partial_cmp(&NAN): {:?}", 1.0_f64.partial_cmp(&nan));
println!("NAN.partial_cmp(&NAN): {:?}", nan.partial_cmp(&nan));
```

```text
1.0.partial_cmp(&NAN): None
NAN.partial_cmp(&NAN): None
```

Not `Some(Less)`, not `Some(Equal)`, not `Some(Greater)` — `None`. `NaN` isn't comparable to `1.0`, and it isn't comparable to itself either. `cmp`'s signature has no room for "not comparable" at all — `fn cmp(&self, other: &Self) -> Ordering`, no `Option`, always a definite answer. `f64` cannot honestly fill that signature, so `Ord` is never implemented for it. This is the exact sentence standing behind `ratings.sort()` in [2.1.1](../../01-collections/01-vec-depth/README.md), behind `BinaryHeap<f64>` in [2.1.3](../../01-collections/03-btreemap-hashset-vecdeque/README.md), and behind `.max()` in [2.2.3](../../02-iterators-and-closures/03-consuming-and-collecting/README.md). It isn't a fact to memorize anymore — it's something you can now derive yourself, straight from `cmp`'s own signature.

### `Hash` — why `HashMap`/`HashSet` keys need to be `Hash`

[2.1.2](../../01-collections/02-hashmap-in-depth/README.md) said `HashMap` keys need to be `Hash` and `Eq`, without saying exactly how `Hash` works. Here's the answer: `Hash` feeds a value into a `Hasher` object, field by field, in order — the same object that eventually produces a hash number. `#[derive(Hash)]` writes exactly that, automatically:

```rust
#[derive(Debug, PartialEq, Eq, Hash)]
struct AnimeDerived {
    title: String,
    episodes: u32,
}
```

By hand it's just a few `.hash(state)` calls, in order:

```rust
impl Hash for AnimeManual {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.episodes.hash(state);
    }
}
```

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 06-hash-derive-and-hand
```

```text
derived Hash — inserting the same value again returns: false
set size: 1
hand-written Hash — inserting the same value again returns: false
set size: 1
```

Both behave identically: `HashSet::insert` returns `false` when a value is already present, and the size doesn't change — exactly the contract [2.1.2](../../01-collections/02-hashmap-in-depth/README.md) showed you for `.insert()`.

And `f64`? It's not just missing `Ord` — it's missing `Hash` too. You can see exactly why:

```rust
let z = 0.0_f64;
let neg_z = -0.0_f64;
println!("0.0 == -0.0:      {}", z == neg_z);
println!("0.0.to_bits():    {}", z.to_bits());
println!("(-0.0).to_bits(): {}", neg_z.to_bits());
```

```text
0.0 == -0.0:      true
0.0.to_bits():    0
(-0.0).to_bits(): 9223372036854775808
```

`0.0` and `-0.0` are equal under `==`, but their bit patterns are completely different. A naive `Hash` that just worked off the raw bits would give two *equal* values two *different* hashes — exactly the thing you're about to see is a disaster.

```senpai-visual
{"kind":"concept","labels":["insert(value)","hash(value) → bucket number","inside the bucket: Eq compares","equal → replaced, unequal → added"]}
```

That's the exact path a `HashMap`/`HashSet` walks every time: first `hash(value)` decides which bucket, then — only inside that one bucket — `Eq` compares to see whether this is genuinely the same value that was already there. Now the rule that path rests on can be stated precisely: **two values that are equal (by `Eq`) must hash equal too.** If they don't, the "inside the bucket: `Eq` compares" step never even runs — because the first step already sent the two values to two different buckets.

You can see this with a deliberately inconsistent `Hash`, not just take it on faith:

```rust
impl PartialEq for Anime {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title // title only; episodes_watched doesn't matter
    }
}
impl Hash for Anime {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.episodes_watched.hash(state); // bug: eq() never looks at this
    }
}
```

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 07-hash-eq-inconsistency-trap
```

```text
== says these are the same show: true
.contains() finds it:            false
set length after the "duplicate" insert: 2
  Anime { title: "Frieren", episodes_watched: 5 }
  Anime { title: "Frieren", episodes_watched: 12 }
```

`==` says these two values are the same entry — correctly, because `eq()` only checks `title`. But `.contains()` can't find it, and `.insert()` accepts the second one too, because their hashes — which also mix in `episodes_watched` — differ, so they never even reach the `Eq` comparison step. No error, no panic, no warning — just a `HashSet` with 2 entries that should have been 1.

**The rule to hold onto:** whatever your `Eq` ignores, your `Hash` has to ignore too — no more, no less. The compiler never checks this; it can't, because deciding which fields belong to a value's identity is a decision only you can make.

### Seeing them together — one type that has several

Now that you know exactly what each one promises, stacking several of them onto one `#[derive(...)]` line isn't intimidating anymore:

```rust
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
struct Anime {
    title: String,
    episodes: u32,
    score: u8,
}

impl Ord for Anime {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score) // more meaningful than field order
    }
}
```

That's five derives plus one hand-written `Ord` — precisely because "which one is bigger" for an anime means score, not wherever its title happens to fall alphabetically. Now the same type, in two completely different data structures:

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 08-all-together
```

```text
watch next: Some(Anime { title: "Frieren", episodes: 28, score: 96 })
then:       Some(Anime { title: "Bocchi the Rock!", episodes: 12, score: 90 })
duplicate insert returned: false
library size:              1
Default::default():        Anime { title: "", episodes: 0, score: 0 }
```

`BinaryHeap<Anime>` uses your hand-written `Ord` to always hand back the highest score first. `HashSet<Anime>` uses the derived `Hash`/`Eq` to reject a duplicate. `Default` gives a blank starting point. Three completely separate traits, three jobs that only belong to each one, on a single type.

One subtlety worth knowing before you reach for `BTreeSet`/`BTreeMap`: those two decide "are these the same value?" using only `Ord`, never `Eq`. If your `Ord` — like the one above — only looks at `score`, then two *different* animes with the *same* score get treated as one entry inside a `BTreeSet`, even though `==` says they're unequal. The same disease you just saw in `Hash`, wearing `Ord`'s clothes this time.

---

## Hands on

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 01-debug-two-ways
cargo run -p p2-03-04-standard-derives-by-hand --example 02-display-hand-written
cargo run -p p2-03-04-standard-derives-by-hand --example 03-default-and-struct-update
cargo run -p p2-03-04-standard-derives-by-hand --example 04-partialeq-eq-and-nan
cargo run -p p2-03-04-standard-derives-by-hand --example 05-partialord-ord-and-nan
cargo run -p p2-03-04-standard-derives-by-hand --example 06-hash-derive-and-hand
cargo run -p p2-03-04-standard-derives-by-hand --example 07-hash-eq-inconsistency-trap
cargo run -p p2-03-04-standard-derives-by-hand --example 08-all-together
```

Then the two broken ones:

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 09-derive-eq-needs-field-eq --features broken
cargo run -p p2-03-04-standard-derives-by-hand --example 10-hashset-needs-hash-and-eq --features broken
```

Then try these:

1. In `01-debug-two-ways`, add a fourth field to all three structs. The `builder` version's output under `{:#?}` picks it up automatically; what about the `raw` version?
2. In `05-partialord-ord-and-nan`, swap the field order of `AnimeByFields` (put `episodes` first). Does the sorted order change? Is that exactly what you'd expect?
3. In `07-hash-eq-inconsistency-trap`, delete the line `self.episodes_watched.hash(state);`. What does `set length after the "duplicate" insert` print now?

---

## Errors you will meet

### No error at all — a `Hash` that disagrees with `Eq` silently breaks a `HashSet`

The full code is `07-hash-eq-inconsistency-trap` above. **What the compiler is objecting to:** nothing. `impl PartialEq` and `impl Hash` are each valid on their own; no rule in the language says the two have to agree about a field — that agreement is only a contract, not something `rustc` can check. The program compiles, runs, and finishes with no error and no panic; it's just wrong: `set length after the "duplicate" insert` should have been `1`, and it was `2`.

**The fix:** give `Hash` exactly the fields `Eq` looks at — not one more:

```rust
impl Hash for Anime {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.hash(state); // only what eq() also checks
    }
}
```

**Why this is the fix:** now two values `eq()` considers equal also land on the same hash, so they land in the same bucket, and `Eq` finally gets the chance to compare them. **This is the most dangerous kind of defect there is:** no red text, no panic, nothing a shallow test would catch — just a `HashSet` that counts something as two when it should have counted it as one.

### `E0277` — `#[derive(Eq)]` on a struct with an `f64` field

```text
error[E0277]: the trait bound `f64: Eq` is not satisfied
   --> phase2-intermediate\03-traits-and-generics\04-standard-derives-by-hand\examples\09-derive-eq-needs-field-eq.rs:12:5
    |
 10 | #[derive(Debug, PartialEq, Eq)]
    |                            -- in this derive macro expansion
 11 | struct Rating {
 12 |     value: f64,
    |     ^^^^^^^^^^ the trait `Eq` is not implemented for `f64`
    |
    = help: the following other types implement trait `Eq`:
              i128
              i16
              i32
              i64
              i8
              isize
              u128
              u16
            and 4 others
note: required by a bound in `std::cmp::AssertParamIsEq`
   --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\cmp.rs:380:31
    |
380 | pub struct AssertParamIsEq<T: Eq + PointeeSized> {
    |                               ^^ required by this bound in `AssertParamIsEq`

For more information about this error, try `rustc --explain E0277`.
```

**What the compiler is objecting to:** `#[derive(Eq)]` is only valid when *every* field of the struct is itself `Eq` — because you can't promise reflexivity when one of your fields (here, `value: f64`) doesn't promise it either. This is the same `NaN` fact you saw in "The concept," except this time it isn't hiding behind a bare `f64: Ord` the way it did in [2.1.1](../../01-collections/01-vec-depth/README.md) and [2.1.3](../../01-collections/03-btreemap-hashset-vecdeque/README.md) — it's behind a struct of your own.

**The fix:** either drop `Eq` (`PartialEq` alone is fine if you genuinely don't need the reflexivity promise), or change the field's type — for instance, multiply the score by ten and keep it as an integer, the same trick [2.1.1](../../01-collections/01-vec-depth/README.md) used for `.sort()`.

**Why this is the fix:** `#[derive(PartialEq)]` alone makes no promise about reflexivity, so no field is required to be `Eq` — exactly how `f64` itself behaves. And an integer, unlike `f64`, is always equal to itself; it gets `Eq` for free.

### `E0599` — `HashSet<Anime>` without `Hash` and `Eq`

```text
error[E0599]: the method `insert` exists for struct `HashSet<Anime>`, but its trait bounds were not satisfied
  --> phase2-intermediate\03-traits-and-generics\04-standard-derives-by-hand\examples\10-hashset-needs-hash-and-eq.rs:19:10
   |
12 | struct Anime {
   | ------------ doesn't satisfy `Anime: Eq` or `Anime: Hash`
...
19 |     seen.insert(Anime {
   |     -----^^^^^^
   |
   = note: the following trait bounds were not satisfied:
           `Anime: Eq`
           `Anime: Hash`
help: consider annotating `Anime` with `#[derive(Eq, Hash, PartialEq)]`
   |
12 + #[derive(Eq, Hash, PartialEq)]
13 | struct Anime {
   |

For more information about this error, try `rustc --explain E0599`.
```

**What the compiler is objecting to:** this is the same wall you saw behind `HashMap::insert` in [2.1.2](../../01-collections/02-hashmap-in-depth/README.md), this time behind a `HashSet`. For `.insert()` to know where to place a value and later recognize it again, it needs both traits — `Hash` for the bucket, `Eq` for comparing within the bucket. `Anime` here only has `#[derive(Debug)]`; it has neither.

**The fix:** exactly what the compiler suggests:

```rust
#[derive(Debug, Eq, Hash, PartialEq)]
struct Anime {
    title: String,
    episodes: u32,
}
```

**Why this is the fix:** `String` and `u32` are both `Hash` and `Eq` themselves, so `derive` can build both traits straight from the fields — exactly the same mechanism you already wrote by hand in "The concept."

---

## Exercises

### Warm up

<details>
<summary>You have a struct with <code>#[derive(Debug)]</code> and fields <code>id: u32</code> and <code>name: String</code>. For <code>Item { id: 7, name: "x".to_string() }</code>, what exactly does <code>format!("{item:?}")</code> give?</summary>

Write down your answer before checking.

</details>

<details>
<summary>Answer</summary>

```text
Item { id: 7, name: "x" }
```

Type name, brace, then each field as `name: value` separated by commas — exactly what `f.debug_struct("Item").field(...)...` builds.

</details>

<details>
<summary>Does <code>#[derive(Debug, PartialEq, Eq)] struct Meters(f64);</code> compile?</summary>

Write down your answer — what did you already see about `f64`?

</details>

<details>
<summary>Answer</summary>

No. `f64` does not implement `Eq` (because `NaN != NaN` breaks reflexivity), and `#[derive(Eq)]` requires *every* field to be `Eq` itself. The error code is `E0277`.

</details>

<details>
<summary>What does <code>f64::NAN.partial_cmp(&f64::NAN)</code> return?</summary>

Write down your answer.

</details>

<details>
<summary>Answer</summary>

`None` — not `Some(Equal)`. `NaN` isn't comparable even to itself; `partial_cmp` returns an `Option` for exactly this "not comparable" case.

</details>

<details>
<summary>A struct derives <code>PartialOrd, Ord, ...</code> with fields <code>(a: u32, b: u32)</code> in that order. For two values that differ only in <code>b</code> — one with <code>b: 1</code>, the other with <code>b: 9</code> — which one comes first under <code>.sort()</code>?</summary>

Write down your answer — what order does `derive` compare fields in?

</details>

<details>
<summary>Answer</summary>

The one with `b: 1`. Since `a` is the same on both, the comparison falls through to the second field, `b` — field comparison always follows declaration order.

</details>

<details>
<summary>A type has an <code>impl PartialEq</code> that only checks field <code>x</code>, but a <code>#[derive(Hash)]</code> on it hashes both <code>x</code> and <code>y</code>. Does this compile?</summary>

Write down your answer — is there anything the compiler could actually check here?

</details>

<details>
<summary>Answer</summary>

Yes, it compiles — no error at all. The problem shows up at runtime: two values `==` considers equal can get different hashes, and a `HashSet`/`HashMap` built from this type quietly starts failing to recognize duplicates.

</details>

### Repair

Fix both broken examples:

1. Fix `examples/09-derive-eq-needs-field-eq.rs` so it compiles — without losing what `value` means (hint: change its type, don't just drop `Eq` unless you genuinely don't need it).
2. Fix `examples/10-hashset-needs-hash-and-eq.rs` so it compiles — exactly what the compiler itself suggests.

### Implement

One `struct Track` in `src/lib.rs`, and six traits — one at a time, each a `todo!()`:

```sh
cargo test -p p2-03-04-standard-derives-by-hand
```

`impl Eq for Track {}` and `impl PartialOrd for Track` are already written — both are always that same one line, once you have a real `PartialEq`/`Ord`. Each function's doc comment states exactly what its output should be; don't guess, especially the rule that "your `Hash` sees exactly the fields your `Eq` sees."

### Build

Design a small struct of your own (two or three fields, any domain you like) and implement at least **three** of today's six traits by hand — not derived. At least one of those three has to be `Ord` or `Hash`. In the struct's doc comment, say which three you picked and why a plain `#[derive]` would *not* have been enough for at least one of them.

### Challenge (optional)

[2.1.3](../../01-collections/03-btreemap-hashset-vecdeque/README.md)'s optional challenge showed you how to find the top `k` items without fully sorting the list, using a `BinaryHeap<Reverse<(u32, String)>>` that never grows past `k` entries. Apply the same technique to your `Track` (or the struct you wrote for "Build"): write a function that returns the `k` shortest tracks, using a `BinaryHeap` that never holds more than `k` entries — without ever sorting the whole list.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `f.debug_struct(...)` | The ergonomic builder for `Debug` output; supports `{:#?}` for free | A hand-written `impl Debug` for a real struct |
| `Default` | A sensible starting value; `Default::default()` | Struct update syntax (`..Default::default()`) |
| Reflexivity | Every value always equals itself, `x == x` | Why `f64` has only `PartialEq`, never `Eq` |
| Total order | For any two values, there is always exactly one answer to "which is smaller?" | Why `f64` has only `PartialOrd`, never `Ord` |
| `NaN` | "Not a number"; not comparable or equal to any value, including itself | The root cause behind `f64` missing `Eq`/`Ord`/`Hash` |
| `Hash` trait | Feeds a value into a `Hasher`, field by field | Any type that wants to be a `HashMap`/`HashSet` key |
| `Hash`/`Eq` consistency | Values equal by `Eq` must also hash equal | The compiler never sees a violation; a `HashSet` breaks silently instead |

### What you now know

- `#[derive(Debug)]` is exactly what `f.debug_struct(...).field(...).finish()` also builds; a raw `write!` gives the same compact output but doesn't support `{:#?}`.
- `Display` is never derived, because "for the user" is a human decision, not something mechanical you can read off the fields.
- `Default` gives a sensible starting value; `#[derive(Default)]` requires every field to itself be `Default`, and struct update syntax leans on it.
- `Eq` adds exactly one extra promise on top of `PartialEq` — reflexivity — and `Ord` carries that same promise one level up over `PartialOrd` — a total order. `f64` can't honestly give either, because of `NaN`.
- `Hash` feeds a value into a `Hasher` field by field; it has to see exactly the fields your `Eq` sees — no more, no less — or `HashMap`/`HashSet` starts giving silently wrong answers with no error at all.
- `f64` doesn't even have `Hash`: `0.0 == -0.0`, yet their bit patterns differ — exactly the disease a `Hash` inconsistent with `Eq` causes.

### What comes back later

- **Implementing a trait for a type you don't own, and the orphan rule** — [2.3.6 — Supertraits, blanket impls, and the orphan rule](../06-supertraits-blanket-impls-orphan-rule/README.md)
- **Something that works identically across several different types, without knowing the exact type at compile time** — [2.3.7 — Static vs. dynamic dispatch](../07-static-vs-dynamic-dispatch/README.md)
- **`Display` on error types, for your own error messages** — [2.5.1 — Custom error types](../../05-error-handling/01-custom-error-types/README.md)

### Can you explain?

- Why does a hand-written `impl Debug` using raw `write!` give the same thing under `{:#?}` that it gave under `{:?}`, while the `debug_struct` version doesn't?
- Why does `#[derive(Display)]` never exist?
- What exact extra promise does `Eq` make over `PartialEq`, and why can't `f64` make it?
- What exact extra promise does `Ord` make over `PartialOrd`?
- Why doesn't `f64` itself have `Hash` either — not just `Ord`?
- If your `Hash` disagrees with your `Eq`, exactly where does a `HashSet` break, and why do you never get a panic?

---

## Going further

- [The Rust Book — Appendix C: Derivable Traits](https://doc.rust-lang.org/book/appendix-03-derivable-traits.html) — the full list of standard traits `#[derive]` understands, from the Rust team itself.
- [`std::fmt` documentation](https://doc.rust-lang.org/std/fmt/index.html) — every format macro and the exact signatures of `Debug`/`Display`.
- [`std::hash::Hash` documentation](https://doc.rust-lang.org/std/hash/trait.Hash.html) — including the line that states, in so many words: "`Eq` and `Hash` must agree."
- [`f64::total_cmp` documentation](https://doc.rust-lang.org/std/primitive.f64.html#method.total_cmp) — the exact tool [2.1.1](../../01-collections/01-vec-depth/README.md) used to work around this same problem.
