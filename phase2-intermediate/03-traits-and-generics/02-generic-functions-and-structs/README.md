# 2.3.2 — Generic functions and structs, bounds, `where`

## At a glance

After this lesson you can:

- Explain why `fn largest<T: PartialOrd>(list: &[T]) -> &T` refuses to compile without that `PartialOrd` bound, and read and fix the exact error yourself.
- Decide, for a generic struct, where a bound belongs — on the whole `impl` block or on just one method — and choose between writing a bound inline and writing it with `where` for a real signature.
- Say that `largest::<i32>` and `largest::<&str>` are two completely separate pieces of code once compiled, and why that is exactly what makes generics free at run time in Rust.

**Time:** ~65 minutes · **Prerequisites:** [2.3.1 — Defining and implementing traits](../01-defining-and-implementing-traits/README.md), and specifically [2.2.1 — Closures, `Fn`/`FnMut`/`FnOnce`, and `move`](../../02-iterators-and-closures/01-closures-and-fn-traits/README.md) for the `Fn` bound

---

## Why this matters

Since the very first lesson of Phase 1, you have been using generics every day — you just didn't have the name for it yet:

- [1.1.6](../../../phase1-fundamentals/01-foundations/06-vec-and-string-basics/README.md) — you used `Vec<i32>`, and `Vec<T>` never once needed a rewrite for any other element type. Always the same one definition.
- [1.6.1](../../../phase1-fundamentals/06-absence-and-failure/01-option-and-null-safety/README.md) — you saw `Option<T>`: `Option<u32>`, `Option<String>`, `Option<Box<T>>`. The same one definition again.
- [2.1.2](../../01-collections/02-hashmap-in-depth/README.md) — `HashMap<K, V>` took two of these parameters at once.
- [2.2.1](../../02-iterators-and-closures/01-closures-and-fn-traits/README.md) — you even wrote the signature `fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32`, and that lesson said outright: "generics are taught in full in module 2.3."

This is that moment. Today we open up `<T>` itself: why it is there, what promise it asks you to make, and how the compiler turns that promise into real code with no run-time cost at all.

Without generics, a function like "find the largest item in a list" would have to be written once for `u32`, once for `f64`, once for `String` — three nearly identical functions, differing only in type. That is exactly the kind of duplication programming languages exist to eliminate.

---

## The concept

### The problem generics solve

Say you need a function that finds the largest element in a slice. One version for `u32`, one for `f64`, one for `String` — the body of all three is word for word the same, only the type differs. In Python this problem does not really exist: Python checks argument types at *run time*, not at the time you write the code, so a single `def largest(items): ...` that uses `>` in its body works on anything that supports `>` — no extra work on your part. The price is that if `largest` is ever called with things that cannot be compared, Python only finds out the moment that line actually runs — not before, and that moment can be in the middle of a real user's request.

Rust checks types at compile time, so an ordinary signature like `fn largest(list: &[u32]) -> &u32` stays locked to `u32` forever. Rust's answer is the generic: write the function once, use a placeholder type instead of one fixed type, and let the compiler build the real versions for you:

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}
```

```rust
let episode_counts = [12, 24, 13, 64, 51];
println!("most episodes: {}", largest(&episode_counts));

let titles = ["Frieren", "Bocchi the Rock!", "Made in Abyss"];
println!("alphabetically last: {}", largest(&titles));
```

```text
most episodes: 64
alphabetically last: Made in Abyss
```

One definition, two completely different types — numbers, then strings. Read `<T: PartialOrd>` as "for some unknown type `T`": the letter `T` itself is just a placeholder — Rust's convention is a single capital letter, standing for "whatever concrete type the caller uses."

### Why the bound is required

Now delete `: PartialOrd` from in front of `T` in that same signature and compile again. You get stuck on the line `if item > largest`: the compiler says the `>` operator cannot be used on type `T`. The full error, with its exact code (`E0369`), is in "Errors you will meet" — you can produce it yourself right now in `examples/04-missing-bound.rs`.

Here is the point: the compiler assumes nothing at all about `T` — not "some" assumptions and not others. For a completely unknown `T`, all you can do is exactly what *any* type can do: take it, hand it back, put it in a variable. Comparing, printing, cloning — none of that is free, unless you promise it with a bound. `T: PartialOrd` is exactly that promise: "any `T` this function is called with is guaranteed to support ordering comparisons." That one line both lets you use `>` inside the function body, and — just as important — rejects, right at the call site, any attempt to call `largest` with a type that does not make that promise. The bug moves from "something a user might trigger at run time" to "something that stops `cargo build` on your own machine right now."

This restriction — `T: PartialOrd` — is called a trait bound. From here on we just say "bound."

### Generic structs

Structs can take a type parameter too — the exact same `<T>` you saw on the function. Build a `Shelf<T>` — a shelf you put anything of one type onto:

```rust
struct Shelf<T> {
    items: Vec<T>,
}

impl<T> Shelf<T> {
    fn new() -> Self {
        Shelf { items: Vec::new() }
    }
    fn add(&mut self, item: T) {
        self.items.push(item);
    }
    fn len(&self) -> usize {
        self.items.len()
    }
}
```

```rust
let mut ratings: Shelf<u32> = Shelf::new();
ratings.add(7);
ratings.add(9);
ratings.add(6);
println!("ratings on shelf: {}", ratings.len());
```

```text
ratings on shelf: 3
```

`impl<T> Shelf<T>` carries no bound at all — these three methods work for *any* `T`, even a type that implements nothing whatsoever. `Shelf<u32>` and, soon, `Shelf<String>`, are both valid uses of this one same definition.

### A method with its own bound, versus a bound on the whole `impl` block

Now add a new capability: finding the largest item on the shelf. That needs one extra promise — being able to compare two `T`s with `>`. Rather than put that promise on the whole `impl` block, we put it on the method itself, with a `where` after the signature (shown separately here just to stay compact; in `examples/02-shelf.rs` every method lives in one `impl<T> Shelf<T>`):

```rust
impl<T> Shelf<T> {
    fn highest(&self) -> Option<&T>
    where
        T: PartialOrd,
    {
        let mut best = self.items.first()?;
        for item in &self.items {
            if item > best {
                best = item;
            }
        }
        Some(best)
    }
}
```

```rust
println!("highest rating: {:?}", ratings.highest());
```

```text
highest rating: Some(9)
```

There was another way: put the same bound on the whole `impl` block — `impl<T: PartialOrd> Shelf<T> { ... }` — instead of only on `highest`. With just these two methods (`new` and `highest`) in that block, the two notations give almost the same result. The difference shows up the moment you add a *different* method to that same block — one that has nothing to do with comparison, like `new` itself. If `new` also lived inside `impl<T: PartialOrd> Shelf<T>`, you could no longer even build an empty `Shelf<T>` for a type that has no `PartialOrd` — even though `new` never compares anything. `examples/05-impl-block-bound.rs` builds exactly this; compile it and read the `E0277` in "Errors you will meet".

That is the real difference between a bound on a method and a bound on a block: a method-level bound restricts only that method; a block-level bound restricts everything you write inside that block later too — even a brand-new method that has nothing to do with that bound.

### Trait bounds, two notations: inline and `where`

Everywhere we have written a bound so far, we wrote the same thing two ways. Here is the same `largest`, this time with `where`:

```rust
fn largest<T>(list: &[T]) -> &T
where
    T: PartialOrd,
{
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}
```

```rust
println!("{}", largest(&episode_counts));
```

```text
64
```

The same answer — because this is exactly the same function, just with its bound after the signature instead of inside `<>`. The full version, with both notations side by side and a `println!` for each, is in `examples/03-bounds-and-where.rs`.

When you have only one bound on one parameter, it does not matter which notation you pick — that is taste. `where` stops being optional the moment a method needs a bound on a parameter that was *already* introduced somewhere above — not here. Write a new method on `Shelf<T>`: find the first item matching a condition, and hand back an owned copy of it:

```rust
impl<T> Shelf<T> {
    fn find_and_clone<F>(&self, matches: F) -> Option<T>
    where
        T: Clone,
        F: Fn(&T) -> bool,
    {
        self.items.iter().find(|item| matches(item)).cloned()
    }
}
```

```rust
let found = ratings.find_and_clone(|&r| r >= 8);
println!("first rating >= 8: {found:?}");
```

```text
first rating >= 8: Some(9)
```

Here `where` really is not optional. `F` is newly introduced right on this method, so its bound could have been written inline: `<F: Fn(&T) -> bool>`. But `T` was already introduced earlier, on `impl<T> Shelf<T>` itself — here you are only adding an *extra* promise for this one method (`Clone`), and adding a bound to an already-introduced parameter has no inline form — `where` is the only way. And even if `T` had been introduced right here too, once two parameters (`T` and `F`) each carry their own bound, the inline notation gets crowded enough to be genuinely hard to read — everything on one line, before you even reach the parameter list. `where` spreads the same thing over several lines, one bound each, each readable on its own.

### Multiple bounds on one parameter

A parameter can carry more than one bound — joined with `+`:

```rust
use std::fmt::Display;

fn announce<T: Display + Clone>(item: T) -> (String, T) {
    let headline = format!("now airing: {item}");
    (headline, item.clone())
}
```

```rust
let (headline, kept) = announce(String::from("Frieren"));
println!("{headline}");
println!("kept a copy: {kept}");
```

```text
now airing: Frieren
kept a copy: Frieren
```

`Display` is needed for `{item}` inside `format!`; `Clone` for `item.clone()`. These are two entirely independent promises — one about printing, one about duplicating — both, at once, about the same `T`. Drop either one, and the part of the body that leaned on that promise stops compiling: `examples/06-missing-one-of-two-bounds.rs` drops exactly `Clone` and gets `E0599` — you'll see it in "Errors you will meet".

### Monomorphization

Go back to the two `largest` calls from the start of this lesson — one on `[i32]`, one on `[&str]`. One definition, two completely different types. But that does not mean there is one "generic" version of `largest` sitting around at run time, checking on every call what type it has been handed this time — which is genuinely what Python does. Rust does this at *compile* time: for every concrete type `largest` is actually called with, it generates one entirely separate, entirely concrete version — exactly as if you had hand-written both functions yourself:

```rust
fn largest_i32(list: &[i32]) -> &i32 {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}
```

```rust
println!("{}", largest_i32(&episode_counts));
```

```text
64
```

The same `64` you saw earlier — because this really is what the compiler builds for you behind the scenes. A second version, with `T` replaced by the string type, is already sitting somewhere in the very same binary right now. Generating one completely separate version of a generic function or struct, per type actually used, at compile time, is called monomorphization.

```senpai-visual
{"kind":"concept","labels":["generic largest source","instantiated for i32","instantiated for string","two separate binary functions","zero run-time type check"]}
```

This is the mechanical reason behind the "zero-cost" claim about generics — not a marketing line, a fact about compilation: by the time the program starts running, there is no unresolved `T` left anywhere; every call site is already tied to one fully concrete function. The type check Python performs on every single call simply does not exist here at all — not made cheaper, absent from the start.

There is another way to choose behaviour based on a trait: instead of the compiler deciding, at compile time, which version runs, that decision can be pushed all the way to run time — with a trait object (`dyn Trait`). That carries a small run-time cost, against the complete zero of generics; that whole trade-off is the subject of [2.3.7](../07-static-vs-dynamic-dispatch/README.md).

---

## Hands on

```sh
cargo run -p p2-03-02-generic-functions-and-structs --example 01-largest
cargo run -p p2-03-02-generic-functions-and-structs --example 02-shelf
cargo run -p p2-03-02-generic-functions-and-structs --example 03-bounds-and-where
```

Then the three broken ones:

```sh
cargo run -p p2-03-02-generic-functions-and-structs --example 04-missing-bound --features broken
cargo run -p p2-03-02-generic-functions-and-structs --example 05-impl-block-bound --features broken
cargo run -p p2-03-02-generic-functions-and-structs --example 06-missing-one-of-two-bounds --features broken
```

Then try these:

1. In `01-largest.rs`, add an array of `f64` too and call `largest` on it — without changing the function definition at all.
2. In `02-shelf.rs`, change `find_and_clone`'s predicate to something none of the ratings satisfy (e.g. `r > 100`). Guess the output before running, then check.
3. In `03-bounds-and-where.rs`, add a third function, `announce_where`, that writes `announce`'s signature with `where` instead of inline `Display + Clone`. Does it accept exactly the same calls?

---

## Errors you will meet

### `E0369` — you cannot compare two `T`s with `>`

```text
error[E0369]: binary operation `>` cannot be applied to type `&T`
  --> phase2-intermediate\03-traits-and-generics\02-generic-functions-and-structs\examples\04-missing-bound.rs:12:17
   |
12 |         if item > largest {
   |            ---- ^ ------- &T
   |            |
   |            &T
   |
help: consider restricting type parameter `T` with trait `PartialOrd`
   |
 9 | fn largest<T: std::cmp::PartialOrd>(list: &[T]) -> &T {
   |             ++++++++++++++++++++++

For more information about this error, try `rustc --explain E0369`.
```

**What the compiler is actually complaining about:** `T` is completely unknown inside this function — it could be any type. The compiler assumes nothing about it, so when it reaches `item > largest`, it has no idea whether `>` even means anything for `T`. Notice the error is on `&T`, not `T` — because `item` and `largest` are both references (`list` has type `&[T]`) — but that detail has nothing to do with the actual problem: the problem is the missing bound, not the reference.

**The fix:** exactly what the compiler itself suggested — bring back the `PartialOrd` bound:

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}
```

**Why this is the fix:** `PartialOrd` is exactly the trait that defines `<`, `>`, `<=`, and `>=`. Adding it tells the compiler that any `T` this function is called with promises to be comparable — and then `item > largest` means something.

### `E0277` — a bound on the `impl` block reaches every method inside it

```text
error[E0277]: can't compare `MangaVolume` with `MangaVolume`
  --> phase2-intermediate\03-traits-and-generics\02-generic-functions-and-structs\examples\05-impl-block-bound.rs:25:38
   |
25 |     let _shelf: Shelf<MangaVolume> = Shelf::new();
   |                                      ^^^^^^^^^^^^ no implementation for `MangaVolume < MangaVolume` and `MangaVolume > MangaVolume`
   |
   = help: the trait `PartialOrd` is not implemented for `MangaVolume`
note: required by a bound in `Shelf::<T>::new`
  --> phase2-intermediate\03-traits-and-generics\02-generic-functions-and-structs\examples\05-impl-block-bound.rs:14:9
   |
14 | impl<T: PartialOrd> Shelf<T> {
   |         ^^^^^^^^^^ required by this bound in `Shelf::<T>::new`
15 |     fn new() -> Self {
   |        --- required by a bound in this associated function
help: consider annotating `MangaVolume` with `#[derive(PartialOrd)]`
   |
20 + #[derive(PartialOrd)]
21 | struct MangaVolume {
   |

For more information about this error, try `rustc --explain E0277`.
```

**What the compiler is actually complaining about:** `new` never compares anything — it just builds an empty `Vec`. But because `new` is written inside the same `impl<T: PartialOrd> Shelf<T>` block, the compiler demands the same promise from `T` that a comparing method needs, even for this method that has nothing to do with comparison. `MangaVolume` never made that promise (it does not implement `PartialOrd`), so even `Shelf::new()` itself fails to compile for it.

**The fix:** take the bound off the whole block, and put it only on the method that genuinely needs it:

```rust
impl<T> Shelf<T> {
    fn new() -> Self {
        Shelf { items: Vec::new() }
    }
}
```

**Why this is the fix:** now `new` works for any `T` — as it always should have — and only the method that actually compares things carries its own bound, separately. The compiler agrees: with this change, the same program compiles with `MangaVolume`.

### `E0599` — drop one of two bounds, and exactly that one goes missing

```text
error[E0599]: no method named `clone` found for type parameter `T` in the current scope
  --> phase2-intermediate\03-traits-and-generics\02-generic-functions-and-structs\examples\06-missing-one-of-two-bounds.rs:13:21
   |
11 | fn announce<T: Display>(item: T) -> (String, T) {
   |             - method `clone` not found for this type parameter
12 |     let headline = format!("now airing: {item}");
13 |     (headline, item.clone())
   |                     ^^^^^ method not found in `T`
   |
   = help: items from traits can only be used if the type parameter is bounded by the trait
help: the following trait defines an item `clone`, perhaps you need to restrict type parameter `T` with it:
   |
11 | fn announce<T: Display + Clone>(item: T) -> (String, T) {
   |                        +++++++

For more information about this error, try `rustc --explain E0599`.
```

**What the compiler is actually complaining about:** `T: Display` only promises to be printable; `.clone()` is an entirely different promise (`Clone`), and nobody made that one. The compiler names exactly which trait defines this method and suggests adding it.

**The fix:** bring the second bound back:

```rust
fn announce<T: Display + Clone>(item: T) -> (String, T) {
    let headline = format!("now airing: {item}");
    (headline, item.clone())
}
```

**Why this is the fix:** every line of the body needs its own promise — `format!` needs `Display`, `.clone()` needs `Clone`. Two bounds on one parameter means both promises are required at once; dropping either one takes exactly that one away from the body.

---

## Exercises

### Warm up

<details>
<summary>Does this compile?</summary>

```rust
fn first<T>(list: &[T]) -> &T {
    &list[0]
}
```

</details>

<details>
<summary>Answer</summary>

Yes. This function does nothing with `T` besides holding it and handing it back — no comparing, no printing, no cloning. So it needs no bound at all; `T` can be anything.

</details>

<details>
<summary>Does this compile?</summary>

```rust
fn largest<T>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}
```

</details>

<details>
<summary>Answer</summary>

No. `item > largest` needs the `PartialOrd` bound, which is missing here — the same `E0369` you saw in "Errors you will meet".

</details>

<details>
<summary>Do <code>f</code> and <code>g</code> accept exactly the same set of types?</summary>

```rust
fn f<T: Clone>(x: T) -> T {
    x.clone()
}

fn g<T>(x: T) -> T
where
    T: Clone,
{
    x.clone()
}
```

</details>

<details>
<summary>Answer</summary>

Yes, exactly. `where` is just a different notation for the same bound — the same contract, written two ways. Any type `f` accepts, `g` accepts too, and the other way around.

</details>

<details>
<summary>Does this compile, even though <code>Pair&lt;T&gt;</code> carries no bound?</summary>

```rust
struct Pair<T> {
    a: T,
    b: T,
}

impl<T> Pair<T> {
    fn describe(&self) -> &str {
        "a pair"
    }
}
```

</details>

<details>
<summary>Answer</summary>

Yes. `describe` does nothing with `a` or `b` — it just returns a fixed string — so it needs no promise from `T` at all.

</details>

### Repair

Fix all three broken examples:

1. Fix `examples/04-missing-bound.rs` so it compiles — bring back the bound that is actually needed.
2. Fix `examples/05-impl-block-bound.rs` so `Shelf<MangaVolume>` can be built — without changing `MangaVolume` itself. (Hint: where should the bound go?)
3. Fix `examples/06-missing-one-of-two-bounds.rs` so `.clone()` works again.

### Implement

Four signatures in `src/lib.rs`:

```sh
cargo test -p p2-03-02-generic-functions-and-structs
```

### Build

Write `pub fn describe_all<T: Display>(items: &[T]) -> String` that joins every item's `Display` output into one string — in a format you choose, documented in the function's doc comment.

Then write a second version, `describe_matching`, that only includes items for which a `predicate` returns `true` — the full function signature, and what bound `predicate` needs, is your choice this time.

### Challenge (optional)

**Part one.** Extend the `Pair<T>` you built in Implement with a new method, `swap(&mut self)`, that swaps its two values — without adding any bound at all. Why does this method need no bound whatsoever?

**Part two.** (This one looks ahead.) Suppose you want a `Vec` that can hold both `AnimeSeries` and `MangaVolume` side by side, as long as both have a shared method like `summarize()`. What you learned today — a `Vec<T>` with one generic `T` — cannot do that: every `Vec<T>` holds exactly one concrete `T` at a time, no matter what bound you give it. Write down why that restriction exists, and guess, in one sentence, what solution Rust has for it — check your answer in [2.3.7](../07-static-vs-dynamic-dispatch/README.md).

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Generic | Code written once, parameterized over a type | Any function or struct that must work over more than one type |
| Trait bound | The promise a generic parameter makes: "whatever type goes here has this trait" | `<T: PartialOrd>`, `<T: Display + Clone>` |
| `where` | The same bound, written after the signature instead of inside `<>` | When several bounds on several parameters pile up, or a bound is needed on an already-introduced parameter |
| Monomorphization | Generating one entirely separate copy of a generic function/struct, per type actually used, at compile time | Why generics are free at run time |

### What you now know

- You can read the signature `fn largest<T: PartialOrd>(list: &[T]) -> &T` and say why that bound is there.
- You know why the compiler gives `E0369` without the bound, and you can fix that error yourself.
- You can write a generic struct with a bound-free `impl` block, and add a method with a bound of its own.
- You choose, deliberately, between putting a bound on the whole `impl` block and on just one method — and you know what the first choice costs.
- You write a bound both inline and with `where`, and you know when the second is not just more readable but the only option.
- You put more than one bound on a parameter with `+`.
- You can say exactly what monomorphization does, and when — and why that is exactly what makes generics free in Rust.

### What comes back later

- **Writing `Display`/`Debug` for your own type, and deriving `PartialOrd`/`Ord`/`Hash` for your own type** — [2.3.4 — The standard derives, implemented by hand](../04-standard-derives-by-hand/README.md)
- **Associated types, versus generic parameters** — [2.3.5 — Associated types versus generic parameters](../05-associated-types/README.md)
- **`dyn Trait` and dynamic dispatch, against exactly what you saw today** — [2.3.7 — Static versus dynamic dispatch](../07-static-vs-dynamic-dispatch/README.md)
- **The full story of the lifetime on that `&T` `largest` returns** — [2.4.1 — Lifetime basics and elision](../../04-lifetimes-and-conversion/01-lifetime-basics-and-elision/README.md)

### Can you explain?

- Why doesn't `fn largest<T>(list: &[T]) -> &T` compile without a bound? Which exact line of the body gets stuck?
- What is the difference between putting a bound on the whole `impl` block and putting it on just one method? Give your own example.
- What is the difference between `<T: Display + Clone>` and `where T: Display + Clone`?
- Why does `matches_count` (in `src/lib.rs`) need no bound on `T` at all?
- Exactly when does monomorphization happen — at compile time or at run time? And why is that exactly what makes generics "free"?
- How is `dyn Trait` different from a generic? (One sentence is enough — 2.3.7 has the details.)

---

## Going further

- [The Rust Book — Generic Types, Traits, and Lifetimes](https://doc.rust-lang.org/book/ch10-00-generics.html) — the same ground, official and more complete.
- [The official Rust blog — "Abstraction without overhead: traits in Rust"](https://blog.rust-lang.org/2015/05/11/traits.html) — the same "free" claim, from the Rust team itself.
- [`std::cmp::PartialOrd`](https://doc.rust-lang.org/std/cmp/trait.PartialOrd.html) — the trait you bounded against all lesson long; the full list of what it promises.
- [The Rust Reference — Generic parameters](https://doc.rust-lang.org/reference/items/generics.html) — the precise, official write-up of everything you saw today.
