# 2.3.7 — Static versus dynamic dispatch, and object safety

## At a glance

After this lesson you can:

- Tell apart `impl Trait` in argument position (just another spelling of a generic bound) from `impl Trait` in return position (a genuinely different tool that hides a concrete type), and explain why neither one ever needs a vtable.
- Choose, for a real function signature, between a generic parameter and `dyn Trait`, and say in one sentence what you gain and what you give up.
- Read an "is not dyn compatible" compiler error, name exactly which method is the blocker, and fix it.

**Time:** ~80 minutes · **Prerequisites:**
[2.2.1 — Closures, `Fn`/`FnMut`/`FnOnce`, and `move`](../../02-iterators-and-closures/01-closures-and-fn-traits/README.md),
[2.2.4 — Implementing `Iterator` and `IntoIterator` for your own type](../../02-iterators-and-closures/04-implementing-iterator/README.md),
[2.3.1 — Defining and implementing traits](../01-defining-and-implementing-traits/README.md),
[2.3.2 — Generic functions and structs, bounds, `where`](../02-generic-functions-and-structs/README.md)

---

## Why this matters

Ever since you met traits, every time you wrote `item.summary()` or called any other trait method, one thing was never said out loud: exactly *when* does the compiler decide which function that call targets — at compile time, or at run time? Until now it never mattered, because you always called through one specific concrete type. Today it matters, for two reasons.

First, [2.2.1](../../02-iterators-and-closures/01-closures-and-fn-traits/README.md) made you a promise and never said when it would come back for it. `make_adder` there returned an `impl Fn(i32) -> i32`, and the lesson dropped a note for later: "`impl Trait` in return position only promises a *single* concrete type; when the real type varies depending on a branch, you need `Box<dyn Fn(i32) -> i32>` instead — the subject of the trait-objects lesson." That trait-objects lesson is this one. Today that promise gets paid in full.

Second, you're about to run into code that needs to hold several *different* types — not one fixed type — in a single variable: a queue of different handlers, a plugin list, a collection of things that share nothing but implementing one trait. Generics run out here — not because they're broken, but because they were built for a different job entirely. And when you reach for `dyn Trait` to solve it, you sometimes hit a wall: the compiler says your trait "is not dyn compatible," followed by a paragraph that reads like a wall of text the first time. This lesson makes that paragraph make sense.

This lesson closes module 2.3. Every one of the six lessons before it — traits, generics, `From`/`TryFrom`, the standard derives, associated types, supertraits — comes together right here: the final question was never "how do I write a trait?" It's "once I have my trait, exactly how does the compiler use it?"

---

## The concept

### Recall: generics get monomorphized

[2.3.2](../02-generic-functions-and-structs/README.md) already showed you this: a generic function gets a separate compiled copy for every concrete type you actually use it with — **monomorphization**. Today all it takes is one trait and two types to see it again, this time with concrete proof:

```rust
trait Summarize {
    fn summary(&self) -> String;
}
// AnimeSeries and MangaVolume each have their own summary() —
// nothing new from 2.3.1.
```

A generic function, exactly the shape [2.3.2](../02-generic-functions-and-structs/README.md) taught:

```rust
fn announce<T: Summarize>(item: &T) -> String {
    format!("now: {}", item.summary())
}

println!("{}", announce(&anime));
println!("{}", announce(&manga));
```

```text
now: Trigun - 26 episodes
now: Blame! - 10 chapters
```

Now the proof: if the compiler really built two separate functions, they live at two separate addresses. Take the function's own address for each instantiation and compare:

```rust
let anime_fn = announce::<AnimeSeries> as usize;
let manga_fn = announce::<MangaVolume> as usize;
println!("same compiled function? {}", anime_fn == manga_fn);
```

```text
same compiled function? false
```

Two different addresses. `announce::<AnimeSeries>` and `announce::<MangaVolume>` are two genuinely separate functions — not one generic function deciding something at run time. The compiler knows, right at the call site, exactly which `summary` it's targeting, and can often inline the call directly there — zero extra overhead. This is called **static dispatch**: "static" because everything about the call is fixed and known at compile time.

```senpai-visual
{"kind":"concept","labels":["generic call site","T = AnimeSeries","T = MangaVolume","two compiled functions"]}
```

### Dynamic dispatch: one function, many types

Now do the same job with a different spelling — instead of `T: Summarize`, write `dyn Summarize`:

```rust
fn announce_dyn(item: &dyn Summarize) -> String {
    format!("now: {}", item.summary())
}

println!("{}", announce_dyn(&anime));
println!("{}", announce_dyn(&manga));
```

```text
now: Trigun - 26 episodes
now: Blame! - 10 chapters
```

Same output — but this time the function's signature has no `<T>` at all. There's nothing to instantiate, so there's nothing to monomorphize either: `announce_dyn` is **one** compiled function, exactly as written, and that one function just answered both types above.

What does that mean? `&dyn Summarize` is a **trait object**: a reference to a value of *some* concrete type that implements `Summarize`, with that concrete type erased — the compiler only knows "this implements `Summarize`," not which struct it actually is. For `.summary()` to still work correctly, Rust keeps a second thing alongside that reference: a **vtable** (virtual method table) — a small table of function pointers, one per trait method, pointing at whichever concrete type's implementation this particular value actually has.

You already know this shape — "one reference, plus a second pointer": [1.3.4](../../../phase1-fundamentals/03-borrowing-and-references/04-slices/README.md) called it a **fat pointer** — a slice is exactly this too, just with a length as the second word instead of a vtable. `&dyn Summarize` is a fat pointer as well: one word to the real data, one word to the vtable.

Calling `.summary()` on a `dyn Summarize` means: at run time, read the vtable pointer stored alongside the data, look up the `summary` function pointer in that table, then jump to it. One extra hop — small, but real — compared to a direct call the compiler already knew the target of. This is called **dynamic dispatch**.

```senpai-visual
{"kind":"concept","labels":["call through &dyn Trait","read the vtable pointer","find summary in the table","jump to the real function"]}
```

### Heterogeneity: the thing a generic can never do

The type erasure that looked like a cost above is exactly what buys you a new capability. Put a `Box<dyn Summarize>` — a boxed `AnimeSeries` and a boxed `MangaVolume` — into one `Vec`:

```rust
let lineup: Vec<Box<dyn Summarize>> = vec![
    Box::new(anime),
    Box::new(manga),
];
for item in &lineup {
    println!("{}", item.summary());
}
```

```text
Trigun - 26 episodes
Blame! - 10 chapters
```

One `Vec`, one element type (`Box<dyn Summarize>`), two genuinely different structs underneath. This is the thing no `Vec<T>` with a single fixed generic `T` can ever do — `T` is chosen exactly once, for the whole `Vec`, and here two are needed.

`Box` here is said only exactly as much as is needed, nothing more: a heap box that turns whatever it holds into a same-size pointer, that's it. `AnimeSeries` and `MangaVolume` are different sizes; `Box<dyn Summarize>` is a fixed size for either one (a fat pointer), so a `Vec` can line them up side by side. The full story of `Box` — heap allocation, ownership, when to reach for it on its own — belongs to [2.6.1](../../06-smart-pointers/01-box-and-heap-allocation/README.md); today only this one role of it is needed.

Without `Box`, this idea does not compile at all — the reason is in "Errors you will meet."

### `impl Trait` in argument position: just another spelling of the same bound

[2.2.1](../../02-iterators-and-closures/01-closures-and-fn-traits/README.md) made exactly this same move, on closures: `apply_twice<F: Fn(i32) -> i32>(f: F, x: i32)` and `apply_twice_v2(f: impl Fn(i32) -> i32, x: i32)` — one identical signature, spelled two ways. The same move works on `Summarize` too:

```rust
fn describe<T: Summarize>(item: &T) -> String {
    item.summary()
}
fn describe_v2(item: &impl Summarize) -> String {
    item.summary()
}
```

Call `describe` and `describe_v2` with `&AnimeSeries` — both get a monomorphized copy with `T = AnimeSeries`, both are static dispatch, and the compiler generates the same code for either one. `impl Trait` in argument position **is** the generic bound, exactly, just a shorter spelling that drops the name `T`. Nothing new here — only confirmation of what [2.2.1](../../02-iterators-and-closures/01-closures-and-fn-traits/README.md) already showed you.

### `impl Trait` in return position: hiding the type

Return position is a different story. [2.2.1](../../02-iterators-and-closures/01-closures-and-fn-traits/README.md) wrote `make_adder` for exactly this reason:

```rust
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}
```

You can never write a closure's real return type — it has no name at all. A generic doesn't help here, because the return type isn't chosen by the caller; the function itself has to decide. `impl Trait` in return position says exactly this: "some concrete type, I won't say exactly which." And the important part: this is still **static dispatch** — there is only one concrete type, the compiler knows it, only the caller isn't allowed to know its name.

The same idea, on something you already know — the Fibonacci iterator from [2.2.4](../../02-iterators-and-closures/04-implementing-iterator/README.md):

```rust
fn fibonacci() -> impl Iterator<Item = u64> {
    Fibonacci { current: 0, next: 1 }
}

let first_eight: Vec<u64> = fibonacci().take(8).collect();
println!("{first_eight:?}");
```

```text
[0, 1, 1, 2, 3, 5, 8, 13]
```

The return type says "some `Iterator` of `u64`," not "`Fibonacci`." The caller never learns the real type — and never needs to; everything [2.2.4](../../02-iterators-and-closures/04-implementing-iterator/README.md) gave you for free on `Fibonacci` (`.take()`, `.filter()`, `.collect()`) still works for free here, because whatever it really is, it genuinely is an `Iterator`. The only thing that changed is you no longer have to write `Fibonacci`'s name into your function's public signature — useful when that type is an implementation detail you don't want to commit to.

### When `impl Trait` runs out

[2.2.1](../../02-iterators-and-closures/01-closures-and-fn-traits/README.md) left exactly this note for later: "`impl Trait` in return position only promises a *single* concrete type; when the real type varies depending on a branch, you need `Box<dyn Fn(i32) -> i32>` instead." See exactly where it breaks. Two closures, one condition:

```rust
fn make_adjuster(bonus: bool, n: i32) -> impl Fn(i32) -> i32 {
    if bonus {
        move |x| x + n
    } else {
        move |x| x * n
    }
}
```

```text
error[E0308]: `if` and `else` have incompatible types
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\10-branch-mismatch-broken.rs:14:9
   |
11 | /     if bonus {
12 | |         move |x| x + n
   | |         --------------
   | |         |
   | |         the expected closure
   | |         expected because of this
13 | |     } else {
14 | |         move |x| x * n
   | |         ^^^^^^^^^^^^^^ expected closure, found a different closure
15 | |     }
   | |_____- `if` and `else` have incompatible types
   |
   = note: expected closure `{closure@phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\10-branch-mismatch-broken.rs:12:9: 12:17}`
              found closure `{closure@phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\10-branch-mismatch-broken.rs:14:9: 14:17}`
   = note: no two closures, even if identical, have the same type
   = help: consider boxing your closure and/or using it as a trait object
help: if you change the return type to expect trait objects, box the returned expressions
   |
12 ~         Box::new(move |x| x + n)
13 |     } else {
14 ~         Box::new(move |x| x * n)
   |
```

The compiler's message is precise: "no two closures, even if identical, have the same type" — every closure has its own type, written nowhere, and `impl Fn(i32) -> i32` can only name **one** of them, not "either of these two, depending on the condition." The fix is exactly what [2.2.1](../../02-iterators-and-closures/01-closures-and-fn-traits/README.md) promised — `Box<dyn Fn(i32) -> i32>`:

```rust
fn make_adjuster(bonus: bool, n: i32) -> Box<dyn Fn(i32) -> i32> {
    if bonus {
        Box::new(move |x| x + n)
    } else {
        Box::new(move |x| x * n)
    }
}

println!("{}", make_adjuster(true, 5)(1));
println!("{}", make_adjuster(false, 5)(1));
```

```text
6
5
```

This is the same dynamic dispatch from the previous section, just on `Fn` instead of `Summarize`: both branches now return a shared type — `Box<dyn Fn(i32) -> i32>` — so it doesn't matter which real closure ends up underneath. The general rule: `impl Trait` works whenever there genuinely is one concrete type; the moment the real type depends on a branch, a loop, or an input, you need `dyn Trait`.

### Object safety: why not every trait becomes `dyn`

Every trait we've seen so far turned into `dyn` without a fight. That's not always true. Consider a trait that returns a fresh copy of the type itself — it looks perfectly ordinary:

```rust
trait Spinoff {
    fn spinoff(&self) -> Self;
}
```

Now a function taking `&dyn Spinoff`:

```text
error[E0038]: the trait `Spinoff` is not dyn compatible
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\08-not-object-safe-self-by-value-broken.rs:26:21
   |
26 | fn announce(_item: &dyn Spinoff) {}
   |                     ^^^^^^^^^^^ `Spinoff` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\08-not-object-safe-self-by-value-broken.rs:11:26
   |
10 | trait Spinoff {
   |       ------- this trait is not dyn compatible...
11 |     fn spinoff(&self) -> Self;
   |                          ^^^^ ...because method `spinoff` references the `Self` type in its return type
   = help: consider moving `spinoff` to another trait
   = help: only type `AnimeSeries` implements `Spinoff`; consider using it directly instead.
```

**Object safety** is exactly this: the condition a trait must meet to become `dyn Trait`. Your own compiler, in this very message, gives it a newer name too — "dyn compatible" — the same idea, just the more official name in newer Rust versions. You'll see both names; they're the same thing.

The reason traces back to that same vtable. A `dyn Spinoff` has erased the real type underneath — the compiler only knows "this implements `Spinoff`," not how much room it takes. `spinoff(&self) -> Self` asks the vtable to return a value **by value** — meaning reserve room for it — but nobody knows how big that erased type is. A slot in a table cannot promise a size it doesn't know.

That's not the only way to break it. A generic method causes exactly the same problem, from a different angle:

```rust
trait Rated {
    fn rating_as<T: From<u8>>(&self) -> T;
}
```

```text
error[E0038]: the trait `Rated` is not dyn compatible
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\09-not-object-safe-generic-method-broken.rs:24:21
   |
24 | fn announce(_item: &dyn Rated) {}
   |                     ^^^^^^^^^ `Rated` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\09-not-object-safe-generic-method-broken.rs:11:8
   |
10 | trait Rated {
   |       ----- this trait is not dyn compatible...
11 |     fn rating_as<T: From<u8>>(&self) -> T;
   |        ^^^^^^^^^ ...because method `rating_as` has generic type parameters
   = help: consider moving `rating_as` to another trait
   = help: only type `AnimeSeries` implements `Rated`; consider using it directly instead.
```

A vtable is **one** fixed table, built once, before any caller has ever chosen a `T`. A generic method needs a separate slot for every possible `T` — a count that isn't known ahead of time, maybe unbounded. A table that doesn't know how many rows it has isn't a table.

Per this lesson, these two are the most common real-world reasons object safety breaks — not the full list of rules. If you genuinely need one of these two shapes but still want the rest of the trait to stay `dyn`-able, the "Challenge" below shows a real escape hatch.

```senpai-visual
{"kind":"concept","labels":["dyn Trait call","method returns Self by value","erased Self has no size","not dyn compatible"]}
```

### Decision table: generic, or `dyn`?

Everything above, in one place:

| Axis | Generic / `impl Trait` wins when... | `dyn Trait` wins when... |
|---|---|---|
| Collection shape | every element really is one fixed type | you genuinely need a mix of types in one variable or collection |
| Speed | the call is hot and must inline | one vtable hop next to the rest of the work is negligible |
| Binary size | few concrete types are ever actually used | many concrete types would each get their own full copy |
| The trait itself | always available | only if the trait is object safe — no generic methods, no `Self` by value |

A shorter rule, for when you forget the table: **default to generics.** Reach for `dyn Trait` specifically when you genuinely need heterogeneity, or binary size from heavy monomorphization has actually become a problem. Phase 3 shows you this exact trade-off again: `axum`'s router and its handler registries lean on `dyn Trait` for precisely this reason — a router has to hold many *different* handler types in one collection, exactly the "heterogeneity" problem above.

---

## Hands on

```sh
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 01-static-two-compiled-functions
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 02-dynamic-one-function-many-types
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 03-heterogeneous-vec-box-dyn
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 04-fibonacci-hidden-behind-impl-trait
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 05-branch-fixed-with-box-dyn-fn
```

Then the five broken ones:

```sh
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 06-vec-dyn-no-box-broken --features broken
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 07-missing-dyn-keyword-broken --features broken
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 08-not-object-safe-self-by-value-broken --features broken
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 09-not-object-safe-generic-method-broken --features broken
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 10-branch-mismatch-broken --features broken
```

Then try these:

1. In `01-static-two-compiled-functions.rs`, add a third type (say, `LightNovel`), implement `Summarize` for it, and add `announce::<LightNovel>`'s address to the comparison. Are all three addresses different?
2. In `03-heterogeneous-vec-box-dyn.rs`, instead of `for item in &lineup`, sum every `summary()`'s length with `.iter().map(...).sum()` — the exact thing `total_summary_length_dyn` in `src/lib.rs` asks of you.
3. In `05-branch-fixed-with-box-dyn-fn.rs`, add a third branch (say, a subtracting closure) and check whether `Box<dyn Fn(i32) -> i32>` is still enough with an `if`/`else if`/`else` — why must it be?

---

## Errors you will meet

### `E0782` — a bare trait, with no `dyn`

```text
error[E0782]: expected a type, found a trait
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\07-missing-dyn-keyword-broken.rs:23:20
   |
23 | fn announce(item: &Summarize) -> String {
   |                    ^^^^^^^^^
   |
help: use a new generic type parameter, constrained by `Summarize`
   |
23 - fn announce(item: &Summarize) -> String {
23 + fn announce<T: Summarize>(item: &T) -> String {
   |
help: you can also use an opaque type, but users won't be able to specify the type parameter when calling the `fn`, having to rely exclusively on type inference
   |
23 | fn announce(item: &impl Summarize) -> String {
   |                    ++++
help: alternatively, use a trait object to accept any type that implements `Summarize`, accessing its methods at runtime using dynamic dispatch
   |
23 | fn announce(item: &dyn Summarize) -> String {
   |                    +++

For more information about this error, try `rustc --explain E0782`.
```

**What the compiler is objecting to:** `&Summarize` reads like "a reference to the trait itself" — but a trait is not a type, it's a contract types can satisfy. Since Rust 2021, the language forces you to say exactly what you meant: a generic `T`, an `impl Summarize`, or a `dyn Summarize`.

**The fix:** pick one of the compiler's own three suggestions; for dynamic dispatch, `&dyn Summarize`.

**Why this is the fix:** this error is a summary of the whole lesson by itself — the same three options "The concept" showed you, this time in the compiler's own words. None of them is "more correct"; the choice depends on whether a fixed type is enough or you need heterogeneity.

### `E0277` — a `Vec<dyn Summarize>` has no known size

```text
error[E0277]: the size for values of type `(dyn Summarize + 'static)` cannot be known at compilation time
   --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\06-vec-dyn-no-box-broken.rs:14:29
    |
 14 | fn total_summaries(_lineup: &Vec<dyn Summarize>) {}
    |                             ^^^^^^^^^^^^^^^^^^^ doesn't have a size known at compile-time
    |
    = help: the trait `Sized` is not implemented for `(dyn Summarize + 'static)`
note: required by an implicit `Sized` bound in `Vec`
   --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\alloc\src\vec\mod.rs:438:16
    |
438 | pub struct Vec<T, #[unstable(feature = "allocator_api", issue = "32838")] A: Allocator = Global> {
    |                ^ required by the implicit `Sized` requirement on this type parameter in `Vec`

For more information about this error, try `rustc --explain E0277`.
```

**What the compiler is objecting to:** a `Vec`'s backing array stores its elements inline, back to back, and needs to know each element's exact byte size up front. `dyn Summarize` alone doesn't say that: `AnimeSeries` and `MangaVolume` are different sizes, and "some type implementing `Summarize`" could be any size at all.

**The fix:** make the element type `Box<dyn Summarize>`, not bare `dyn Summarize`:

```rust
fn total_summaries(lineup: &Vec<Box<dyn Summarize>>) -> usize {
    lineup.len()
}
```

**Why this is the fix:** a `Box`, regardless of what it holds, is always a fixed size — one pointer, to a spot on the heap. `Box<dyn Summarize>` is a perfectly ordinary, sized element type from the `Vec`'s point of view; it just doesn't know what's on the other end of each pointer. That's exactly why `dyn Trait` almost always shows up behind a pointer (`Box<dyn T>`, `&dyn T`, `Rc<dyn T>`), never bare.

### `E0308` — `if`/`else` stop matching under `impl Trait`

```text
error[E0308]: `if` and `else` have incompatible types
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\10-branch-mismatch-broken.rs:14:9
   |
11 | /     if bonus {
12 | |         move |x| x + n
   | |         --------------
   | |         |
   | |         the expected closure
   | |         expected because of this
13 | |     } else {
14 | |         move |x| x * n
   | |         ^^^^^^^^^^^^^^ expected closure, found a different closure
15 | |     }
   | |_____- `if` and `else` have incompatible types
   |
   = note: expected closure `{closure@phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\10-branch-mismatch-broken.rs:12:9: 12:17}`
              found closure `{closure@phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\10-branch-mismatch-broken.rs:14:9: 14:17}`
   = note: no two closures, even if identical, have the same type
   = help: consider boxing your closure and/or using it as a trait object
help: you could change the return type to be a boxed trait object
   |
10 - fn make_adjuster(bonus: bool, n: i32) -> impl Fn(i32) -> i32 {
10 + fn make_adjuster(bonus: bool, n: i32) -> Box<dyn Fn(i32) -> i32> {
   |
help: if you change the return type to expect trait objects, box the returned expressions
   |
12 ~         Box::new(move |x| x + n)
13 |     } else {
14 ~         Box::new(move |x| x * n)
   |

For more information about this error, try `rustc --explain E0308`.
```

**What the compiler is objecting to:** two closures, even with an identical signature, are two completely different types — each has its own unnamed type. `impl Fn(i32) -> i32` promises "one fixed concrete type," but this function returns two different types depending on `bonus`.

**The fix:** make the return type `Box<dyn Fn(i32) -> i32>` and box both closures — exactly what the compiler itself suggests.

**Why this is the fix:** `Box<dyn Fn(i32) -> i32>` is a **single** type both closures can convert to (type erasure, exactly like `Box<dyn Summarize>`). Now both `if` branches genuinely return the same type, and deciding which real closure it was is pushed from compile time to run time.

### `E0038` — a trait that returns `Self` by value is not "dyn compatible"

```text
error[E0038]: the trait `Spinoff` is not dyn compatible
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\08-not-object-safe-self-by-value-broken.rs:26:21
   |
26 | fn announce(_item: &dyn Spinoff) {}
   |                     ^^^^^^^^^^^ `Spinoff` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\08-not-object-safe-self-by-value-broken.rs:11:26
   |
10 | trait Spinoff {
   |       ------- this trait is not dyn compatible...
11 |     fn spinoff(&self) -> Self;
   |                          ^^^^ ...because method `spinoff` references the `Self` type in its return type
   = help: consider moving `spinoff` to another trait
   = help: only type `AnimeSeries` implements `Spinoff`; consider using it directly instead.

For more information about this error, try `rustc --explain E0038`.
```

**What the compiler is objecting to:** its own message is precise — `spinoff` uses the `Self` type in its return type. Behind `&dyn Spinoff`, `Self` is erased and its size is unknown; a vtable cannot promise a return value of unknown size.

**The fix:** either move this method out of the trait that's meant to become `dyn` (the compiler's own suggestion), or return a concrete or boxed type instead of `Self` — for instance, `Box<dyn Spinoff>`.

**Why this is the fix:** the problem isn't the idea of "build a fresh copy of myself" — it's that "myself," behind `dyn`, no longer has a known size. Returning a `Box<dyn Spinoff>` fixes it, because `Box` — exactly like in "Heterogeneity" — is always a fixed size.

### `E0038` — a generic method breaks "dyn compatible" too

```text
error[E0038]: the trait `Rated` is not dyn compatible
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\09-not-object-safe-generic-method-broken.rs:24:21
   |
24 | fn announce(_item: &dyn Rated) {}
   |                     ^^^^^^^^^ `Rated` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\09-not-object-safe-generic-method-broken.rs:11:8
   |
10 | trait Rated {
   |       ----- this trait is not dyn compatible...
11 |     fn rating_as<T: From<u8>>(&self) -> T;
   |        ^^^^^^^^^ ...because method `rating_as` has generic type parameters
   = help: consider moving `rating_as` to another trait
   = help: only type `AnimeSeries` implements `Rated`; consider using it directly instead.

For more information about this error, try `rustc --explain E0038`.
```

**What the compiler is objecting to:** this time the reason isn't "references the Self type" — it's "has generic type parameters." `rating_as::<T>` needs a separate row in the vtable for every possible `T`, but the vtable is built once, before any caller has chosen a `T`.

**The fix:** move this method out of the trait, or make it non-generic (a fixed return type instead of `T`).

**Why this is the fix:** a vtable cannot have an unknown number of slots. The only way `Rated` stays `dyn`-able is for every one of its methods to have exactly one fixed signature — not an unbounded family of signatures, one per `T`.

---

## Exercises

### Warm up

<details>
<summary>You call a generic function <code>f&lt;T: Trait&gt;</code> twice, with two different concrete types. How many versions of <code>f</code> does the compiler build?</summary>

Two — one per `T`. This is monomorphization: every generic instantiation gets its own compiled copy.

</details>

<details>
<summary>Does this compile?</summary>

```rust
trait Summarize {
    fn summary(&self) -> String;
}
fn take_it(x: &Summarize) -> String {
    x.summary()
}
```

</details>

<details>
<summary>Answer</summary>

No — `E0782`. `&Summarize` is not a reference to the trait itself; you have to say what you mean: `&dyn Summarize` for dynamic dispatch, or `&impl Summarize` (or a generic `T`) for static dispatch.

</details>

<details>
<summary>Does this compile?</summary>

```rust
trait Summarize {
    fn summary(&self) -> String;
}
let items: Vec<dyn Summarize> = Vec::new();
```

</details>

<details>
<summary>Answer</summary>

No — `E0277`. `dyn Summarize` has no fixed size, and `Vec` needs to know each element's size up front. It has to be `Vec<Box<dyn Summarize>>`.

</details>

<details>
<summary>A trait has this method: <code>fn make(&self) -> Self;</code>. Can this trait become a <code>dyn Trait</code>? Why or why not?</summary>

No. `make` returns `Self` by value; behind `dyn`, `Self` is erased and its size is unknown, so a vtable cannot reserve room for it. This is exactly what the `E0038` error on `Spinoff` showed you.

</details>

<details>
<summary>What does this print?</summary>

```rust
fn seq() -> impl Iterator<Item = u64> {
    Fibonacci { current: 0, next: 1 }
}
println!("{:?}", seq().take(3).collect::<Vec<_>>());
```

</details>

<details>
<summary>Answer</summary>

```text
[0, 1, 1]
```

The first three values of the same sequence [2.2.4](../../02-iterators-and-closures/04-implementing-iterator/README.md) built — `impl Trait` in return position only hides the type's name, it doesn't change its behavior.

</details>

### Repair

Fix all five broken examples — not with a syntax trick, but by changing the actual decision:

1. `examples/07-missing-dyn-keyword-broken.rs` — fix it by adding `dyn` (pick one of the compiler's three suggestions and say why that one).
2. `examples/06-vec-dyn-no-box-broken.rs` — fix it by changing the element type to `Box<dyn Summarize>`.
3. `examples/10-branch-mismatch-broken.rs` — fix it by changing the return type to `Box<dyn Fn(i32) -> i32>` and boxing both closures.
4. `examples/08-not-object-safe-self-by-value-broken.rs` — fix it so `Spinoff` still means the same thing without `Self` by value — either move the method, or change what it returns.
5. `examples/09-not-object-safe-generic-method-broken.rs` — fix it so `Rated` no longer has a generic method.

### Implement

Four functions in `src/lib.rs`, each one piece of this lesson:

```sh
cargo test -p p2-03-07-static-vs-dynamic-dispatch
```

- `total_summary_length_generic` — static dispatch, over a fixed `T`.
- `total_summary_length_dyn` — the same total, over trait objects.
- `lineup` — build a two-element `Vec<Box<dyn Summarize>>`.
- `fibonacci` — return a fresh `Fibonacci` behind `impl Iterator<Item = u64>`.

The exact specification — including the precise output format of each — is in the doc comment above each function.

### Build

A brand-new trait of your own design, with at least two different types implementing it — anything you like (a `Playable` trait for a few kinds of media, a `Notify` trait for a few kinds of message, whatever). Write **both** a generic function and a `dyn`-based function for it, both working over the same two types. In a comment, say which one you would actually reach for in real code and why — and exactly what you would lose by picking the other one.

### Challenge (optional)

Reopen the `Spinoff` trait from "Errors you will meet." This time, instead of moving `spinoff` to another trait, write `fn spinoff(&self) -> Self where Self: Sized;` on that same method — one extra bound, on just this one method. Compile it, and try `&dyn Spinoff` again. Does it compile this time? Which methods are still callable through `&dyn Spinoff`, and which aren't? In a sentence or two, say exactly what promise `where Self: Sized` on a single method makes to the compiler that lets the rest of the trait stay `dyn`-able.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Static dispatch | deciding which function a call targets, entirely at compile time | generics, `impl Trait` in argument or return position |
| Dynamic dispatch | the same decision, at run time, through a vtable | `&dyn Trait`, `Box<dyn Trait>` |
| Trait object | a value of some type implementing a trait, with the concrete type erased | anywhere you need heterogeneity |
| vtable | the table of function pointers stored alongside a trait object's data | what dynamic dispatch passes through on every call |
| Object safety (dyn compatibility) | the condition a trait must meet to become `dyn Trait` | a generic method or a `Self`-by-value return breaks it |
| `impl Trait` | in argument position: a generic bound, spelled differently. In return position: hiding one fixed concrete type | both are static dispatch |

### What you now know

- A generic function gets monomorphized — one compiled copy per concrete type; a function built on `dyn Trait` has only one copy and decides, at run time, off the vtable, which implementation to call.
- `Box<dyn Trait>` inside a `Vec` does what no `Vec<T>` with a fixed `T` can: hold several genuinely different concrete types in one collection.
- `impl Trait` in argument position is exactly the generic bound; in return position it's a different tool that hides one fixed concrete type — and the moment that type depends on a branch, it's no longer enough.
- Object safety (or "dyn compatibility," its more official name in the compiler's own output) is the condition a trait must meet to become `dyn Trait`; a generic method and a `Self`-by-value return are the two common ways to break it.
- Default to generics; reach for `dyn Trait` when you genuinely need a heterogeneous collection, or binary size from heavy monomorphization has become a real concern.

### What comes back later

- **`Box<T>` and heap allocation, in full** — this lesson only used `Box` as a way to give a trait object a fixed size; the complete story — [2.6.1 — `Box` and heap allocation](../../06-smart-pointers/01-box-and-heap-allocation/README.md).
- **`Rc`/`Arc` for shared ownership** — the next step once a single owner (even a `Box<dyn Trait>` one) isn't enough — [2.6.3 — `Rc` and `Arc`](../../06-smart-pointers/03-rc-and-arc/README.md).
- **`Send`/`Sync` bounds on trait objects** — the extra bound a `dyn Trait` needs to cross a thread boundary — [2.8.4 — `Send` and `Sync`](../../08-concurrency/04-send-and-sync/README.md).
- **`async` methods inside a trait** — why `async fn` in a trait wrestled with this exact object-safety wall for years, and how it's handled today — [2.9.4 — Async traits and `spawn_blocking`](../../09-async-in-practice/04-async-traits-and-blocking/README.md).

### Can you explain?

- Why does a generic function get a separate compiled copy per concrete type, while a function built on `&dyn Trait` has only one?
- Why does `Vec<dyn Summarize>` fail to compile but `Vec<Box<dyn Summarize>>` succeed?
- Why is `impl Trait` in argument position "just another spelling," but in return position a genuinely different tool?
- A `Self`-by-value return and a generic method both break object safety — but by two different routes. Explain each in your own words.
- For the "Build" scenario, explain why both a generic and a `dyn Trait` version worked over the same two types, and why you still picked only one.

---

## Going further

- [The Rust Book, ch. 18.2 — Using Trait Objects](https://doc.rust-lang.org/book/ch18-02-trait-objects.html) — the same subject, from the Rust team itself.
- [The Rust Reference — Dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility) — the full list of object-safety rules; this lesson showed only its two most common ones.
- [`std::boxed::Box`](https://doc.rust-lang.org/std/boxed/struct.Box.html) — the official documentation for the exact tool you used today to give trait objects a fixed size.
- [RFC 1522 — Conservative `impl Trait`](https://rust-lang.github.io/rfcs/1522-conservative-impl-trait.html) — the original proposal for `impl Trait`, for anyone who likes seeing where a feature came from.
