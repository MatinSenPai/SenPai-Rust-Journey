# 1.5.3 — Enums as data

## At a glance

After this lesson you can:

- Declare an `enum` whose variants each carry different data — and write and build all three shapes of variant.
- Model a domain so that the invalid state is *unwritable*, instead of checking for it at run time.
- Say what `Option<T>` and `Result<T, E>` actually are, and predict an enum's size from its variants.

**Time:** ~50 minutes · **Prerequisites:**
[1.5.2 — Tuple structs and the newtype pattern](../02-tuple-structs-and-newtype/README.md) ·
[1.2.1 — Stack and heap](../../02-ownership-and-memory/01-stack-and-heap/README.md)

---

## Why this matters

In Django you write a model, keep its state in a `CharField`, and then add columns that only mean anything for *some* of those states:

```text
status  = "planned" | "watching" | "rated" | "dropped"
episode = null
score   = null
reason  = ""
```

Nothing stops `status="planned"` from sitting next to `score=9`. Nothing stops `"wathcing"` either. So every time you read that record you have to ask again: which columns are valid right now? — and the day you forget to ask, a bug is born.

Rust's `enum` attacks the same problem from a different side. Instead of *detecting* the wrong combination, it makes the wrong combination impossible to build.

And this is not the `enum` of C or Java. There, an enum is a list of numbered names and that is all. In Rust **each variant can carry its own data** — and that one difference promotes the enum from a minor convenience to the language's main modelling tool.

---

## The concept

### An enum is "exactly one of these"

```rust
#[derive(Debug)]
enum Medium {
    Anime,
    Manga,
    Webtoon,
}

let medium = Medium::Manga;
println!("{medium:?}");
```

```text
Manga
```

`Medium` is a new type, like the `struct` in 1.5.1 — with one essential difference. A `struct` has all of its fields *at once*; a `Medium` is exactly one of those three. Not none, not two, and never a fourth.

That is the simplest form of an **enum**, and each of those three names is a **variant**. A variant carrying no data is a **unit variant**. Up to here, this is the enum a C programmer already knows.

Note that you always name a variant through its type: `Medium::Manga`, never `Manga`. The enum's name is part of the path.

### A variant can carry data

```rust
#[derive(Debug)]
enum Episode {
    Numbered(u32),
    Special(String),
}

println!("{:?}", Episode::Numbered(12));
println!("{:?}", Episode::Special(String::from("OVA")));
```

```text
Numbered(12)
Special("OVA")
```

This is where Rust's enum leaves C's behind. `Numbered` carries a number and `Special` carries a string — **two completely different types, inside one type**. Both lines above produced an `Episode`.

That shape is a **tuple variant**, because its data is an unnamed tuple — the same thing you saw in the tuple structs of the last lesson.

And one detail worth keeping: `Episode::Numbered` is itself a function taking a `u32` and returning an `Episode`. That is why `Episode::Numbered(12)` looks like a function call — it is one.

### A variant can carry named data

```rust
#[derive(Debug)]
enum Entry {
    Planned,
    Watching(u32),
    Rated { score: u8 },
    Dropped { episode: u32, reason: String },
}

println!("{:?}", Entry::Rated { score: 9 });
```

```text
Rated { score: 9 }
```

`Rated` and `Dropped` have named fields, exactly like a `struct` body. These are **struct variants**, and when the data is more than one piece — or when the field name is half the explanation — they read better than a tuple.

But more important than any of the three names is this: **each variant chooses its own shape.** The `Entry` above has a unit variant, a tuple variant and two struct variants, and all four values have the same type:

```rust
let library = vec![
    Entry::Planned,
    Entry::Watching(7),
    Entry::Rated { score: 9 },
];
println!("{library:?}");
```

```text
[Planned, Watching(7), Rated { score: 9 }]
```

One `Vec<Entry>` holds all three, because all three are one type. A function taking an `Entry` accepts all four states, and **there is no fifth shape anyone could hand it**. This family of types is called a **sum type**: the set of possible values is the *sum* of the variants' values.

### Make the invalid state unwritable

Now write that Django model in Rust, but without an enum:

```rust
#[derive(Debug)]
struct LooseEntry {
    status: String,
    episode: u32,
    score: u8,
    reason: String,
}
```

And build a nonsense value:

```rust
let nonsense = LooseEntry {
    status: String::from("planned"),
    episode: 40,
    score: 9,
    reason: String::from("too slow"),
};
println!("{nonsense:?}");
```

```text
LooseEntry { status: "planned", episode: 40, score: 9, reason: "too slow" }
```

It is *planned*, yet somehow on episode 40, scored 9, and carrying a reason for having been dropped. Three states at once. The compiler had no objection at all — and it shouldn't have, because every `String` is a valid `String` and every `u8` is a valid `u8`.

With `Entry`, there is no line you can write that produces that value. Not caught — **not expressible**. `Planned` has nowhere to put a score, and `Rated` has nowhere to put a reason.

> **Working rule:** if you have to look at one field to know which *other* fields are meaningful, that field is acting as a discriminant. There is an enum hiding there.

**The Python/Django bridge, and where it breaks:** Django's `choices` does exactly half of this — it closes the list of allowed tags. But `choices` only constrains the label; the `episode` and `score` columns stay outside it and can still hold any combination. Rust's enum ties the label and the data together in one move, and that is precisely where the analogy stops.

### An enum is a type, so it gets an `impl` block

Everything you learned about `impl` in 1.5.1 holds here. Associated functions, with no `self`:

```rust
impl Entry {
    fn new() -> Self {
        Entry::Planned
    }

    fn start(episode: u32) -> Self {
        Entry::Watching(episode)
    }
}
```

And methods that ask a question. To ask "which variant is this?", the `matches!` macro is enough:

```rust
impl Entry {
    fn is_watching(&self) -> bool {
        matches!(self, Entry::Watching(_))
    }

    fn is_done(&self) -> bool {
        matches!(self, Entry::Rated { .. } | Entry::Dropped { .. })
    }

    fn is_favourite(&self) -> bool {
        matches!(self, Entry::Rated { score } if *score >= 8)
    }
}
```

```text
Planned
    watching: false  done: false  favourite: false
Watching(7)
    watching: true  done: false  favourite: false
Rated { score: 9 }
    watching: false  done: true  favourite: true
Rated { score: 4 }
    watching: false  done: true  favourite: false
Dropped { episode: 3, reason: "too slow" }
    watching: false  done: true  favourite: false
```

Three patterns there: `_` means "carrying anything", `..` means "and whatever named fields it has", `|` means "or". And that trailing `if` looks at the *data* the variant carries rather than only at which variant it is — which is why `score: 9` is a favourite and `score: 4` is not.

`matches!` is deliberately feeble: it hands back `true` or `false` and never the data. The real tool for opening an enum up is `match`, and that is the whole of [1.5.4](../04-match-in-depth/README.md). Until then you build enums and ask about their shape; from there on you take them apart.

### `Option` and `Result` are just enums

Since 1.1.2 you have kept meeting things like `Some(3)` and `None`, and every time the answer was "later". Now is later, because you finally have the tool. This is the standard library's complete definition:

```rust
enum Option<T> {
    None,
    Some(T),
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

That is all of it. Nothing in the language was built for them; they are the same two things you could write yourself right now — a unit variant and a tuple variant. That `<T>` means "any type you like" and is called **generics** — the full treatment is in [Phase 2](../../../phase2-intermediate/03-generics-and-traits/01-generic-functions-and-structs/README.md); here it is enough that `Option<i32>` means "either nothing, or an `i32`".

And because these are variants, you can spell them out in full:

```rust
println!("{:?}", Option::Some(9_u8));
println!("{:?}", Option::<u8>::None);
println!("{:?}", Result::<u8, String>::Ok(9));
```

```text
Some(9)
None
Ok(9)
```

You write `Some`, `None`, `Ok` and `Err` bare only because the prelude imports them for you — not because they are keywords. And it is why `.last()` on a vector hands back an `Option`: "a look at the last element if there is one" and "a different shape if there isn't" are two variants of one enum. Working with them is [1.6.1](../../06-absence-and-failure/01-option-and-null-safety/README.md) and [1.6.3](../../06-absence-and-failure/03-result-and-question-mark/README.md).

### How much room an enum takes

An enum value has to hold two things: the variant's data, and *which* variant it is. That second one is a small number called the **discriminant**.

```senpai-visual
{"kind":"concept","labels":["discriminant","largest variant","alignment padding","size_of"]}
```

```rust
println!("Medium: {} bytes", size_of::<Medium>());
println!("Small:  {} bytes", size_of::<Small>()); // A(u8) | B(u8)
println!("Wide:   {} bytes", size_of::<Wide>()); // Tiny(u8) | Big(u64)
println!("Entry:  {} bytes", size_of::<Entry>());
```

```text
Medium:                1 bytes
Small:                 2 bytes
Wide:                  16 bytes
Entry:                 32 bytes
```

Read the rule off those four numbers: **an enum is as big as its largest variant, plus a discriminant, rounded up to its alignment.**

- `Medium` carries no data, so only the discriminant is left: 1 byte.
- `Small` carries one byte of data plus one byte of discriminant: 2 bytes.
- `Wide`'s largest variant is a `u64` — 8 bytes, alignment 8. The discriminant is one byte, but the whole thing must round to a multiple of 8: 16 bytes. A `Wide::Tiny(1)` takes 16 bytes too, because size is a property of the *type*, not the value.
- `Entry`'s largest variant is `Dropped`: a `String` (24 bytes) plus a `u32` (4) is 28, and alignment 8 rounds that to 32. The discriminant fitted into the four bytes of padding already going spare, so it came out free.

The practical consequence: a thousand-element `Vec<Entry>` takes a thousand times the size of the *largest* variant — even if every one of them is `Planned`. When one variant grows far bigger than the rest, that is where `Box` earns its place ([Phase 2](../../../phase2-intermediate/05-smart-pointers/01-box-and-heap-allocation/README.md)).

### The one that breaks the rule

```rust
println!("i32:              {} bytes", size_of::<i32>());
println!("Option<i32>:      {} bytes", size_of::<Option<i32>>());
println!("bool:             {} bytes", size_of::<bool>());
println!("Option<bool>:     {} bytes", size_of::<Option<bool>>());
println!("Box<i32>:         {} bytes", size_of::<Box<i32>>());
println!("Option<Box<i32>>: {} bytes", size_of::<Option<Box<i32>>>());
```

```text
i32:                   4 bytes
Option<i32>:           8 bytes
bool:                  1 bytes
Option<bool>:          1 bytes
Box<i32>:              8 bytes
Option<Box<i32>>:      8 bytes
```

`Option<i32>` obeyed the rule: four bytes of data, plus a discriminant, rounded to 8.

The other two didn't. `Option<bool>` and `bool` are both one byte; `Option<Box<i32>>` and `Box<i32>` are both eight. **The wrapper was free.**

The reason is that the compiler knows which bit patterns are *impossible* for a type, and borrows one of them for `None`. A `bool` is only ever `0` or `1`, so it has 254 unused patterns and `None` becomes one of them. A `Box` is never a null pointer, so `None` becomes zero. This is called the **niche optimisation**, and its pointer special case has historically been known as the null pointer optimisation.

The conclusion to take away: in Rust, null safety **costs no memory**. An `Option<Box<T>>` occupies exactly what a raw nullable pointer occupies in C — the difference is that the compiler will not let you use it without checking.

---

## Hands on

```sh
cargo run -p p1-05-03-enums-as-data --example 01-three-shapes
cargo run -p p1-05-03-enums-as-data --example 02-invalid-states
cargo run -p p1-05-03-enums-as-data --example 03-methods-on-enums
cargo run -p p1-05-03-enums-as-data --example 04-option-is-an-enum
cargo run -p p1-05-03-enums-as-data --example 05-what-an-enum-costs
```

Then the four broken ones:

```sh
cargo run -p p1-05-03-enums-as-data --example 06-wrong-variant-shape --features broken
cargo run -p p1-05-03-enums-as-data --example 07-no-such-field --features broken
cargo run -p p1-05-03-enums-as-data --example 08-variant-is-not-a-type --features broken
cargo run -p p1-05-03-enums-as-data --example 09-cannot-compare --features broken
```

Then try:

1. In `01-three-shapes`, add a fifth variant to `Entry` carrying a `String`, and build one. Did anything break?
2. In `05-what-an-enum-costs`, change `Big(u64)` to `Big(u32)`. What is `Wide` now, and why?
3. Still in `05-what-an-enum-costs`, print the size of `Option<Option<bool>>`. Guess first, then run it.

---

## Errors you will meet

### `E0533` — building a variant in the wrong shape

```text
error[E0533]: expected value, found struct variant `Entry::Rated`
  --> examples\06-wrong-variant-shape.rs:17:22
   |
17 |     println!("{:?}", Entry::Rated(9));
   |                      ^^^^^^^^^^^^ not a value
   |
help: you might have meant to create a new value of the struct
   |
17 -     println!("{:?}", Entry::Rated(9));
17 +     println!("{:?}", Entry::Rated { score: /* value */ });
   |
```

**What the compiler is objecting to:** `Rated` is a struct variant, so `Entry::Rated` on its own is not a value — it is a shape waiting for braces and a field name. `Entry::Rated(9)` is calling it like a function, and no such function exists.

**The fix:** `Entry::Rated { score: 9 }`.

**Why that's the fix:** you build a variant in exactly the shape you declared it. Tuple variants take parentheses, struct variants take braces, unit variants take nothing. The mistake in the other direction has its own error: `Entry::Watching { episode: 7 }` gives `E0559`, with the message "`Entry::Watching` is a tuple variant, use the appropriate syntax".

### `E0609` — no such field on this type

```text
error[E0609]: no field `score` on type `Entry`
  --> examples\07-no-such-field.rs:19:33
   |
19 |     println!("score: {}", entry.score);
   |                                 ^^^^^ unknown field
```

**What the compiler is objecting to:** `entry` has type `Entry`, and `Entry` has no fields. `score` is a field of **one of its variants**. From the variable's type alone the compiler cannot know which variant this value is — and if it is `Planned`, there is no `score` to read.

**The fix:** none yet. This error is precisely the door that [1.5.4](../04-match-in-depth/README.md) opens: `match` first asks which variant it is, and then, *inside that arm*, hands you its data.

**Why that's the fix:** this is the bargain the whole lesson is about. The `LooseEntry` model let you read `score` at any moment, at the price of it sometimes being meaningless. The enum forbids the unconditional read so that every read is valid. And if all you want right now is to *know* which variant it is, `matches!` will tell you.

### `E0573` — a variant is not a type

```text
error[E0573]: expected type, found variant `Entry::Rated`
  --> examples\08-variant-is-not-a-type.rs:14:16
   |
14 | fn show(entry: Entry::Rated) {
   |                ^^^^^^^^^^^^ not a type
   |
help: try using the variant's enum
   |
14 - fn show(entry: Entry::Rated) {
14 + fn show(entry: crate::Entry) {
   |
```

**What the compiler is objecting to:** the type is `Entry`. `Entry::Rated` is one of that type's *values*. You cannot put a value where a type goes, any more than you could write `fn f(x: 5)`.

**The fix:** `fn show(entry: Entry)`, exactly as the compiler suggested.

**Why that's the fix:** this mistake usually comes from a reasonable wish — "I want a function that only accepts `Rated`". Rust doesn't give you that, and deliberately so. If you genuinely need the narrower type, make it separately — a `struct Rated { score: u8 }` — and have the variant carry it: `Rated(Rated)`. Then you have both the enum and the narrow type.

### `E0369` — comparing without `PartialEq`

```text
error[E0369]: binary operation `==` cannot be applied to type `Medium`
  --> examples\09-cannot-compare.rs:19:15
   |
19 |     if chosen == Medium::Manga {
   |        ------ ^^ ------------- Medium
   |        |
   |        Medium
   |
note: an implementation of `PartialEq` might be missing for `Medium`
  --> examples\09-cannot-compare.rs:8:1
   |
 8 | enum Medium {
   | ^^^^^^^^^^^ must implement `PartialEq`
help: consider annotating `Medium` with `#[derive(PartialEq)]`
   |
 8 + #[derive(PartialEq)]
 9 | enum Medium {
   |
```

**What the compiler is objecting to:** `#[derive(Debug)]` bought you the ability to be printed. Equality is a separate ability, your type doesn't have it, so `==` means nothing here.

**The fix:** `#[derive(Debug, PartialEq)]`.

**Why that's the fix:** it is 1.2.3's `derive` again, on a different ability. On an enum, `PartialEq` means "the same variant, carrying equal data" — so `Rated { score: 9 } == Rated { score: 9 }` holds and `== Rated { score: 8 }` does not. If you only care whether two values are the same *variant* and not what they carry, `matches!` answers that and needs no `PartialEq` at all.

---

## Exercises

### Warm up

<details>
<summary>The difference between a <code>struct</code> and an <code>enum</code>, in one sentence?</summary>

A `struct` has all of its fields at once; an `enum` is exactly one of its variants.

</details>

<details>
<summary>What are the three shapes of variant, and how is each one built?</summary>

Unit (`Planned`, nothing at all), tuple (`Watching(7)`, parentheses), struct (`Rated { score: 9 }`, braces and field names).

</details>

<details>
<summary>Why doesn't <code>let x: Entry::Rated = ...</code> compile?</summary>

Because `Entry::Rated` isn't a type — it is one of the values of the type `Entry`. That is `E0573`.

</details>

<details>
<summary>What exactly is <code>Option&lt;T&gt;</code>?</summary>

An enum with two variants: `None`, which is a unit variant, and `Some(T)`, which is a tuple variant. Nothing in the language is special about it and you could have written it yourself.

</details>

<details>
<summary>How many bytes is an enum with variants <code>A(u8)</code> and <code>B(u64)</code>? Why?</summary>

16. The largest variant is 8 bytes with alignment 8, and the discriminant pushes the whole thing up to the next multiple of 8.

</details>

<details>
<summary>Why is <code>size_of::&lt;Option&lt;Box&lt;i32&gt;&gt;&gt;()</code> equal to <code>size_of::&lt;Box&lt;i32&gt;&gt;()</code>?</summary>

Because a `Box` is never null, so the all-zero bit pattern is unused and the compiler takes it for `None`. That is the niche optimisation.

</details>

### Repair

Fix all four broken examples:

1. `examples/06-wrong-variant-shape.rs` — then try `Entry::Watching { episode: 7 }` as well, to see the mirror-image error.
2. `examples/07-no-such-field.rs` — without `match`, rewrite it so the program compiles and prints something meaningful. (Hint: `matches!` and `{:?}`.)
3. `examples/08-variant-is-not-a-type.rs` — take the compiler's own suggestion.
4. `examples/09-cannot-compare.rs` — then say what `PartialEq` would compare if `Medium` had a variant carrying data.

### Implement

Three functions and two methods in `src/lib.rs`:

```sh
cargo test -p p1-05-03-enums-as-data
```

None of them needs `match`. The first three only *build* the right variant; the last two use `matches!` to ask a question — one about the variant's shape, one about the data the variant carries.

### Build

Design an enum for a domain of your own: **the outcome of a background job** in a service.

It must have at least one unit variant, one tuple variant and one struct variant — say "queued", "running with a percentage", "failed with a code and a message". Then:

1. Add `#[derive(Debug)]`, build one of each variant, and print them.
2. Before running it, guess what `size_of` says about your type — then print it. If you guessed wrong, say which variant set the size.
3. Write an `is_terminal(&self) -> bool` method with `matches!` that says whether this is a final state.
4. Write a paragraph: if you had modelled this as a `struct` with a `status: String` field, exactly which meaningless value would have become buildable?

### Challenge (optional)

**Part one.** Guess these, then print them: `size_of::<Option<u8>>()`, `size_of::<Option<char>>()`, `size_of::<Option<&i32>>()`, `size_of::<Option<Option<bool>>>()`. Which got a niche and which didn't?

**Part two.** Write a recursive enum — a list where each link carries the next one:

```rust
enum Chain {
    End,
    Link(u32, Chain),
}
```

Read the `E0072` you get. The compiler names the fix itself; apply it, and explain why the size becomes finite with that change. (This leads into [Phase 2 — `Box` and heap allocation](../../../phase2-intermediate/05-smart-pointers/01-box-and-heap-allocation/README.md).)

**Part three.** An enum with no data can choose its own discriminant numbers and be cast to one:

```rust
#[derive(Debug)]
enum Rank {
    Bronze = 1,
    Silver = 2,
    Gold = 3,
}
```

Print `Rank::Silver as i32`. Now try the same thing with `Entry` and read the error — why is this only possible for an enum with no data?

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| enum | a type that is exactly one of several shapes | modelling state |
| variant | one of those shapes | `Entry::Rated` |
| unit variant | a variant carrying no data | `Planned` |
| tuple variant | unnamed data, in parentheses | `Watching(7)` |
| struct variant | named data, in braces | `Rated { score: 9 }` |
| sum type | possible values = the sum of the variants' | why the invalid state is unwritable |
| `matches!` | "which variant is this?" as a `bool` | a simple question, without `match` |
| discriminant | the number saying which variant | why an enum has a byte extra |
| niche optimisation | using an impossible bit pattern as the tag | `Option<Box<T>>` is free |
| `E0533` | building a variant in the wrong shape | parentheses instead of braces |
| `E0609` | a variant's field can't be read off the type | the door `match` opens |
| `E0573` | a variant is not a type | function signatures |
| `E0369` | `==` without `PartialEq` | `#[derive(Debug, PartialEq)]` |

### What you now know

- An enum is exactly one of its variants, and the compiler knows which.
- Each variant has its own shape — unit, tuple or struct — all inside one enum.
- A variant is built in the shape it was declared, always prefixed with the enum's name.
- An enum is a type, so it takes `impl` blocks and `derive`.
- If one field's meaning depends on another field, there is an enum hiding there.
- `Option<T>` and `Result<T, E>` are ordinary enums, not language magic.
- An enum's size = the largest variant + a discriminant, rounded to alignment.
- The niche optimisation sometimes makes the discriminant entirely free.

### What comes back later

- **Opening an enum up and taking its data out** — [1.5.4 — `match` in depth](../04-match-in-depth/README.md)
- **When you only care about one variant** — [1.5.5 — `if let`, `while let`, `let else`](../05-if-let-while-let-let-else/README.md)
- **`Option` and the absence of null** — [1.6.1 — `Option` and null safety](../../06-absence-and-failure/01-option-and-null-safety/README.md)
- **`Result` and the `?` operator** — [1.6.3 — `Result` and `?`](../../06-absence-and-failure/03-result-and-question-mark/README.md)
- **That `<T>` you saw in `Option`'s definition** — [Phase 2 — Generics](../../../phase2-intermediate/03-generics-and-traits/01-generic-functions-and-structs/README.md)
- **Recursive variants and oversized enums** — [Phase 2 — `Box` and heap allocation](../../../phase2-intermediate/05-smart-pointers/01-box-and-heap-allocation/README.md)
- **Deeper patterns: guards, bindings, nesting** — [Phase 2 — Pattern matching in depth](../../../phase2-intermediate/08-rust-toolbox/01-pattern-matching-depth/README.md)

### Can you explain?

- State the difference between a Rust `enum` and a C `enum` in one sentence.
- Name the three shapes of variant and build one of each.
- Why doesn't `entry.score` compile, when `Entry::Rated` really does have a `score`?
- Define `Option<T>` from memory.
- Where does an enum's size come from?
- Name one place in your own code where a string field is acting as a discriminant.

---

## Going further

- [The Rust Book — Chapter 6: Enums](https://doc.rust-lang.org/book/ch06-00-enums.html) — the same ground, officially.
- [`std::option::Option`](https://doc.rust-lang.org/std/option/enum.Option.html) — click "Source" and read the real definition. It really is those two lines.
- [The Rust Reference — Type layout](https://doc.rust-lang.org/reference/type-layout.html#enum-layout) — the discriminant and niche rules, stated formally.
- [Making Illegal States Unrepresentable](https://blog.janestreet.com/effective-ml-video/) — the same idea from the OCaml world, which is where the phrase came from.
