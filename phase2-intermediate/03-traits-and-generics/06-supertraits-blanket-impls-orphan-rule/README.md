# 2.3.6 — Supertraits, blanket impls, the orphan rule and the newtype escape hatch

## At a glance

After this lesson you can:

- Define a supertrait — a trait that forces a type to already implement another trait before it can implement this one — and explain why that isn't "inheritance."
- Read the `where` clause of the `Into` blanket impl you saw back in 1.6.5, this time understanding exactly what it says, and write a small blanket impl of your own.
- State the orphan rule from memory, explain why it exists, and work around it by wrapping a foreign type in your own newtype — the same pattern you used for type safety in 1.5.2.
- Read and fix `E0277` on a missing supertrait impl, and `E0117` on an orphan-rule violation, on your own.

**Time:** ~60 minutes · **Prerequisites:**
[2.3.1 — Defining and implementing traits](../01-defining-and-implementing-traits/README.md), and specifically
[2.3.2 — Generic functions and structs](../02-generic-functions-and-structs/README.md),
[1.5.2 — Tuple structs and the newtype pattern](../../../phase1-fundamentals/05-your-own-types/02-tuple-structs-and-newtype/README.md), and
[1.6.5 — `From` and error conversion](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.md)

---

## Why this matters

You have two unfinished promises from earlier.

The first is from [1.6.5](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.md). You saw this code there:

```rust
impl<T, U> Into<U> for T
where
    U: From<T>,
{
    fn into(self) -> U {
        U::from(self)
    }
}
```

and the lesson said: "Don't worry about the syntax — writing something like this is Phase 2's job. Just read it." That day, you only read it. Today you find out exactly what this syntax says, and you write one of these yourself.

The second is from [1.5.2](../../../phase1-fundamentals/05-your-own-types/02-tuple-structs-and-newtype/README.md). There, you built `AccountId(u64)` and `Rial(i64)` so the compiler wouldn't let an account id sit where a money amount belonged. That lesson said a newtype builds a fresh type the compiler recognizes. What it didn't say: sometimes you reach for a newtype not for type safety, but because you have **no other choice**.

Say you want to print a `Vec<i32>` your own way — `println!("{}", numbers)`, not `{:?}`. That means `impl std::fmt::Display for Vec<i32>`. Write it and compile:

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
```

`Vec` isn't yours, and neither is `Display`. The standard library wrote both. The compiler refuses this with a rule called the **orphan rule**, and it refuses for a good reason — one you'll understand by the end of this lesson. And the way around it, interestingly, is the same tool you learned in 1.5.2, put to a completely different job.

---

## The concept

### Supertraits: a trait that leans on another trait

A small trait for "anything with a name":

```rust
trait Named {
    fn name(&self) -> String;
}
```

And now a second trait that wants to say hello — but to do that, it needs to know the other side has a name:

```rust
trait Greet: Named {
    fn greet(&self) -> String {
        format!("Hello, I'm {}!", self.name())
    }
}
```

Look at `trait Greet: Named`. This is called a **supertrait**: `Named` is `Greet`'s supertrait, and this line means "no type can implement `Greet` unless it has already implemented `Named`." `greet`'s default body is already calling `self.name()` — a method `Greet` itself never defined; `Named` did. The compiler accepts this call because that same `: Named` line already guaranteed that whatever `Self` reaches this body also has a `name()`.

Now two types that both implement the pair:

```rust
struct Villager {
    name: String,
}

impl Named for Villager {
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Greet for Villager {}
```

`impl Greet for Villager {}` has an empty body — it just picks up the default. Run it:

```sh
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 01-supertraits-basics
```

```text
Hello, I'm Rin!
Hello, I'm Old Tom of the Blacksmith!
```

The second line comes from a second type in the same file — `Merchant`, whose `name()` builds something fancier (a name plus a shop). `Greet` doesn't care where `name()` comes from; it only cares that it exists.

### This isn't inheritance — it's a dependency

Here is where it's tempting to think `Greet: Named` is the same thing you know from class inheritance in Python or Java. It isn't, and the difference matters.

In object-oriented inheritance, a subclass gets things from its parent **for free** — fields, implemented methods, all of it, without rewriting any of it. Nothing here is free. `impl Greet for Villager {}` doesn't hand `Villager` any of `Named`'s fields or behavior. You still have to write `impl Named for Villager` separately — exactly as you did above — and if you don't, "Errors you will meet" shows you exactly what happens.

A supertrait is only a **condition**: "you may not implement `Greet` until you've implemented `Named`." A trait-level dependency, not a gift handed down from above.

### Supertrait methods are reachable from a subtrait bound too

A practical fact that follows straight from the definition above: a function generic over `T: Greet` can call `Named`'s methods on that same `T` too — without ever writing `T: Greet + Named`:

```rust
fn print_intro<T: Greet>(entity: &T) {
    println!("{}", entity.greet());
    println!("(bare name: {})", entity.name());
}
```

```sh
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 02-supertraits-through-the-bound
```

```text
Hello, I'm Rin!
(bare name: Rin)
```

`print_intro` only wrote `T: Greet`, yet it also called `entity.name()` — a `Named` method. The compiler didn't object, because `trait Greet: Named` already proved: any `T` that satisfies `Greet` has already satisfied `Named` too. Writing `+ Named` would have been redundant.

### Supertraits you were already relying on, unknowingly

This pattern is everywhere in the standard library. The same `+` you saw in [2.3.2](../02-generic-functions-and-structs/README.md) for combining several bounds on a generic parameter (`T: Display + Clone`) is exactly the `+` that joins multiple supertraits:

```text
pub trait Eq: PartialEq<Self> {}
pub trait Ord: Eq + PartialOrd<Self> { /* ... */ }
pub trait Copy: Clone {}
```

`Copy` has one supertrait: `Clone`. That's why every type that derives `Copy` must also derive `Clone` — exactly what you saw in [1.2.3](../../../phase1-fundamentals/02-ownership-and-memory/03-clone-and-copy/README.md), without it being called a "supertrait" there. `Ord` has two supertraits, joined with `+` — the same syntax you know from generic bounds, here applied to `Self` itself.

### The blanket impl, revisited — now you can read the `where`

Go back to what was quoted at the top of this lesson, from 1.6.5:

```rust
impl<T, U> Into<U> for T
where
    U: From<T>,
{
    fn into(self) -> U {
        U::from(self)
    }
}
```

You now know from [2.3.2](../02-generic-functions-and-structs/README.md) what `<T>` and `where` mean. Read it again, word by word this time: "for **every** pair of types `T` and `U`, if `U: From<T>` holds, then `T: Into<U>`." The key word is "every" — not some specific `T` and `U` the library picked, but all of them, at once. This is called a **blanket impl**: an `impl` that runs for every type satisfying a bound, instead of one single concrete type.

The result is the one you already saw in 1.6.5: **nobody writes `impl Into` by hand**, because this one `impl` — right above — already turns every `From` into an `Into`.

### A blanket impl of your own

Now it's your turn to build one. Go back to `Named`/`Greet`. Every type that wanted `Greet` had to write an empty `impl Greet for X {}` line too — even though its body is always the same default. With ten types, you'd write that same empty line ten times.

Instead, write this once:

```rust
impl<T: Named> Greet for T {}
```

That's it. Now **every** type that implements `Named` automatically has `Greet` too — without ever writing `impl Greet for Villager {}` or `impl Greet for Merchant {}` anywhere:

```rust
struct Villager {
    name: String,
}

impl Named for Villager {
    fn name(&self) -> String {
        self.name.clone()
    }
}
```

```sh
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 03-blanket-impl-of-your-own
```

```text
Hello, I'm Rin!
Hello, I'm Old Tom of the Blacksmith!
```

The exact same output as the first example — same two types, same `name()`s — but this time the phrase `impl Greet for` never appears again anywhere in the file. `impl<T: Named> Greet for T {}` did exactly what the `Into`-from-`From` blanket impl did: you wrote one `impl`, and it worked for every type at once.

(A side note for later: now that `Greet` comes from a blanket impl, no type can write its own custom `greet` anymore — that `impl` would conflict with this one. You'll meet this in this lesson's warm-up.)

### The orphan rule, and its two legal shapes

Back to what failed in "Why this matters": `impl Display for Vec<i32>`. Now for the precise rule:

> **You may implement a trait for a type only if at least one of the two — the trait itself, or the type itself — is defined in your own crate.**

The two legal shapes, both in one file:

```rust
trait Summarized {
    fn summarize(&self) -> String;
}

// Foreign type (`Vec` is std's), local trait (`Summarized` is ours).
impl Summarized for Vec<i32> {
    fn summarize(&self) -> String {
        let total: i32 = self.iter().sum();
        format!("{} numbers, sum {}", self.len(), total)
    }
}
```

```rust
struct Point {
    x: i32,
    y: i32,
}

// Local type (`Point` is ours), foreign trait (`Display` is std's).
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
```

```sh
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 04-orphan-rule-legal-forms
```

```text
4 numbers, sum 10
(3, 4)
```

Both compile, because both have exactly one local side. `impl Display for Vec<i32>` — the one that failed above — has no local side at all: neither `Display` is yours, nor `Vec`. That's exactly the dividing line.

```senpai-visual
{"kind":"concept","labels":["Villager has only Greet","compiler also wants Named","missing it: E0277","Named written for Villager","now Greet compiles too"]}
```

### Why this rule exists

Suppose the orphan rule didn't exist. Two completely unrelated libraries — say `crate_weather` and `crate_finance` — both decide to write `impl std::fmt::Display for Vec<i32>`, each to its own taste (one separates with commas, the other with spaces). Now your program depends on both and writes `println!("{}", my_vec)` somewhere. Which `impl` should run? Neither the compiler knows, nor you, nor even either library's author — because neither knew the other existed.

The orphan rule makes this situation impossible from the start. An `impl Trait for Type` is only legal where at least one of `Trait` or `Type` is defined in that same crate — meaning there is always exactly one clear owner of that `impl`: either the trait's author, or the type's author. Two unrelated crates can never both own the same combination, so this collision simply can't happen. (The precise rule for generic types has more nuance; the one sentence above is what you need day to day.)

### The newtype escape hatch: a local wrapper around a foreign type

So what if you genuinely want to print a `Vec<i32>` with `Display`? The same pattern you learned in [1.5.2](../../../phase1-fundamentals/05-your-own-types/02-tuple-structs-and-newtype/README.md), this time for a second job: wrap `Vec<i32>` in your own local tuple struct.

```rust
struct Numbers(Vec<i32>);

impl fmt::Display for Numbers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total: i32 = self.0.iter().sum();
        write!(f, "{} numbers, sum {}", self.0.len(), total)
    }
}
```

```sh
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 05-newtype-escape-hatch
```

```text
4 numbers, sum 10
```

`Numbers` is a `Vec<i32>`, just with a thin local layer around it. That layer is enough: `Numbers` is defined in your own crate, so `impl Display for Numbers` fully satisfies the orphan rule — even though what's actually inside it is the exact same foreign `Vec<i32>`.

In [1.5.2](../../../phase1-fundamentals/05-your-own-types/02-tuple-structs-and-newtype/README.md), you built a newtype so the compiler wouldn't mix up an `AccountId` with a `Rial` — **type safety**. Here, the newtype is doing something entirely different: not separating two similar-looking things, but making a foreign type **local** so the orphan rule allows it. Same tool, second job — and the reason a lot of real Rust code reaches for a newtype in the first place.

There's a cost: `Numbers` inherits none of `Vec`'s methods. `numbers.push(5)` doesn't compile; you'd write `numbers.0.push(5)`, or add your own forwarding method. There's a way to pass those methods through automatically — `Deref` — and you'll see it in full in [2.4.3](../../04-lifetimes-and-conversion/03-deref-asref-borrow/README.md).

```senpai-visual
{"kind":"concept","labels":["Display for Vec directly","both sides are foreign","that means E0117","wrap it in Numbers","now Display compiles"]}
```

---

## Hands on

```sh
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 01-supertraits-basics
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 02-supertraits-through-the-bound
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 03-blanket-impl-of-your-own
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 04-orphan-rule-legal-forms
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 05-newtype-escape-hatch
```

Then the two broken ones:

```sh
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 06-missing-supertrait-impl --features broken
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 07-orphan-rule-violation --features broken
```

Then try these:

1. In `01-supertraits-basics`, add a third type — say `Guard` — that implements `Named` and then `Greet` (without overriding `greet`). Does it get the same default?
2. In `03-blanket-impl-of-your-own`, try also adding a hand-written `impl Greet for Villager { ... }` alongside the blanket impl `impl<T: Named> Greet for T {}` that's already there. What error do you get? Keep the error code — this lesson's warm-up asks exactly this.
3. In `04-orphan-rule-legal-forms`, add a second `impl Summarized for Vec<i32>`, right under the first. What error do you get, and why has nothing to do with the orphan rule?

---

## Errors you will meet

### `E0277` — a missing supertrait implementation

`examples/06-missing-supertrait-impl.rs` has an `impl Greet for Villager {}` where `Villager` never implements `Named` anywhere:

```text
error[E0277]: the trait bound `Villager: Named` is not satisfied
  --> phase2-intermediate\03-traits-and-generics\06-supertraits-blanket-impls-orphan-rule\examples\06-missing-supertrait-impl.rs:20:16
   |
20 | impl Greet for Villager {}
   |                ^^^^^^^^ unsatisfied trait bound
   |
help: the trait `Named` is not implemented for `Villager`
  --> phase2-intermediate\03-traits-and-generics\06-supertraits-blanket-impls-orphan-rule\examples\06-missing-supertrait-impl.rs:15:1
   |
15 | struct Villager {
   | ^^^^^^^^^^^^^^^
help: this trait has no implementations, consider adding one
  --> phase2-intermediate\03-traits-and-generics\06-supertraits-blanket-impls-orphan-rule\examples\06-missing-supertrait-impl.rs:5:1
   |
 5 | trait Named {
   | ^^^^^^^^^^^
note: required by a bound in `Greet`
  --> phase2-intermediate\03-traits-and-generics\06-supertraits-blanket-impls-orphan-rule\examples\06-missing-supertrait-impl.rs:9:14
   |
 9 | trait Greet: Named {
   |              ^^^^^ required by this bound in `Greet`
```

The same file also produces a second `E0277` — at `rin.greet()` in `main`, not at the `impl` line:

```text
error[E0277]: the trait bound `Villager: Named` is not satisfied
  --> phase2-intermediate\03-traits-and-generics\06-supertraits-blanket-impls-orphan-rule\examples\06-missing-supertrait-impl.rs:26:24
   |
26 |     println!("{}", rin.greet());
   |                        ^^^^^ unsatisfied trait bound
   |
help: the trait `Named` is not implemented for `Villager`
  --> phase2-intermediate\03-traits-and-generics\06-supertraits-blanket-impls-orphan-rule\examples\06-missing-supertrait-impl.rs:15:1
   |
15 | struct Villager {
   | ^^^^^^^^^^^^^^^
help: this trait has no implementations, consider adding one
  --> phase2-intermediate\03-traits-and-generics\06-supertraits-blanket-impls-orphan-rule\examples\06-missing-supertrait-impl.rs:5:1
   |
 5 | trait Named {
   | ^^^^^^^^^^^
note: required by a bound in `Greet::greet`
  --> phase2-intermediate\03-traits-and-generics\06-supertraits-blanket-impls-orphan-rule\examples\06-missing-supertrait-impl.rs:9:14
   |
 9 | trait Greet: Named {
   |              ^^^^^ required by this bound in `Greet::greet`
10 |     fn greet(&self) -> String {
   |        ----- required by a bound in this associated function
```

**What the compiler is actually complaining about:** the first message is exactly the sentence you read in "The concept" — `` the trait bound `Villager: Named` is not satisfied ``. `trait Greet: Named` made a promise: "whatever `Self` reaches here also has `Named`." `Villager` never kept that promise, so even the line `impl Greet for Villager {}` — before any method is ever called — is rejected. The second message shows the same problem again, this time at the `greet()` call site.

**The fix:** write the `impl Named` you skipped:

```rust
impl Named for Villager {
    fn name(&self) -> String {
        self.name.clone()
    }
}
```

**Why this is the fix:** the compiler's own help line says "the trait `Named` is not implemented for `Villager`" — literally what you need to write. With that `impl` in place, both `impl Greet for Villager {}` and `rin.greet()` compile — because both errors came from the same root cause.

### `E0117` — an orphan-rule violation

`examples/07-orphan-rule-violation.rs` tries to implement `Display` — a foreign trait — directly for `Vec<i32>` — a foreign type:

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
 --> phase2-intermediate\03-traits-and-generics\06-supertraits-blanket-impls-orphan-rule\examples\07-orphan-rule-violation.rs:9:1
  |
9 | impl fmt::Display for Vec<i32> {
  | ^^^^^^^^^^^^^^^^^^^^^^--------
  |                       |
  |                       `Vec` is not defined in the current crate
  |
  = note: impl doesn't have any local type before any uncovered type parameters
  = note: for more information see https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules
  = note: define and implement a trait or new type instead
```

**What the compiler is actually complaining about:** the compiler's own last line gives away the fix: "define and implement a trait or new type instead" — define your own trait, or a new type. Right under the underlined span, the compiler points straight at `Vec` and says it's "not defined in the current crate" — exactly what "The concept" already told you: neither `Display` nor `Vec` belong to this crate.

**The fix:** one of the two shapes you saw in "The concept" — make the trait local, or make the type local:

```rust
struct Numbers(Vec<i32>);

impl fmt::Display for Numbers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total: i32 = self.0.iter().sum();
        write!(f, "{} numbers, sum {}", self.0.len(), total)
    }
}
```

**Why this is the fix:** `Numbers` is defined right here in this crate, so one side of `impl fmt::Display for Numbers` is local — exactly what the orphan rule wanted. If you'd rather make the trait local instead of wrapping the type, there was another way too: instead of `Display`, write your own trait (`trait Summarized { ... }`) and implement it directly for `Vec<i32>` — exactly what `04-orphan-rule-legal-forms.rs` showed. Which one you pick depends on which side is actually yours: if you want `Display`'s standard behavior (the one `println!("{}", ...)` recognizes everywhere), the newtype is right; if you only need one method of your own, a fresh local trait is simpler.

---

## Exercises

### Warm up

<details>
<summary>Does this compile?</summary>

```rust
trait A {}
trait B: A {}

struct S;
impl B for S {}
```

</details>

<details>
<summary>Answer</summary>

No, `E0277`. `B: A` means `S` must have already implemented `A` before `impl B for S` — and there's no `impl A for S` here.

</details>

<details>
<summary>If you write <code>impl Greet for Cat {}</code>, does that automatically make <code>Cat</code> implement <code>Named</code> too?</summary>

</details>

<details>
<summary>Answer</summary>

No. You still have to write `impl Named for Cat` yourself, separately. A supertrait only checks that you did; it doesn't do it for you.

</details>

<details>
<summary>With <code>impl&lt;T: Named&gt; Greet for T {}</code> already in scope, does this also compile?</summary>

```rust
impl Greet for Villager {
    fn greet(&self) -> String {
        "Meow!".to_string()
    }
}
```

</details>

<details>
<summary>Answer</summary>

No, `E0119` — "conflicting implementations of trait `Greet` for type `Villager`". The blanket impl already gave `Villager` a `Greet` (because it has `Named`); this second `impl` claims the same combination again. A type can only implement a trait once — the same rule that sits behind the orphan rule itself, this time between two `impl`s in the same crate.

</details>

<details>
<summary>Does this compile?</summary>

```rust
struct MyVec(Vec<i32>);

impl std::fmt::Display for MyVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} items", self.0.len())
    }
}
```

</details>

<details>
<summary>Answer</summary>

Yes. `Display` is foreign, but `MyVec` is local — exactly one side is enough.

</details>

<details>
<summary>Does this compile?</summary>

```rust
impl std::fmt::Display for Vec<i32> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} items", self.len())
    }
}
```

</details>

<details>
<summary>Answer</summary>

No, `E0117`. Neither side is local: not `Display`, not `Vec`.

</details>

### Repair

Fix `examples/06-missing-supertrait-impl.rs` by writing `impl Named for Villager`.

Then fix `examples/07-orphan-rule-violation.rs` **two** ways:

1. Wrap `Vec<i32>` in a local `struct Numbers(Vec<i32>);` and implement `Display` for `Numbers`.
2. Without any newtype at all — instead of `Display`, write your own trait (say `trait Summarized { fn summarize(&self) -> String; }`) and implement it directly for `Vec<i32>`.

Then write one sentence: which side of the orphan rule does each fix make local — the trait, or the type?

### Implement

Four things in `src/lib.rs`:

```sh
cargo test -p p2-03-06-supertraits-blanket-impls-orphan-rule
```

`Discounted` is a supertrait of `Priced` — complete its default body. `amount_saved` is a generic function that only asks for `T: Discounted` but needs methods from both traits. `Cart` is a newtype around `Vec<Pen>` — `total_cents` is an ordinary method, and `Display for Cart` is where the whole lesson meets: `Cart` is the only reason this `impl` compiles at all.

Implement all four exactly to the doc comment above each — the discount formula, and `Cart`'s exact text format, are both spelled out right there.

### Build

Define a first trait with one required method (pick any domain — games, cooking, anything). Build a second trait that requires the first as a supertrait, with a default method that uses the first trait's method. Then, instead of writing a separate `impl SecondTrait for Type {}` for every type, write a single blanket impl that hands it, for free, to every type that has the first trait. Try it with at least two different types.

### Challenge (optional)

**Part one.** Suppose you want a list holding both `Book`s and `Pen`s side by side, calling `.discounted_cents()` on each — without knowing in advance which is which. `Vec<T>` can't do this, because `T` has to be one single concrete type. There's a way — something called a **trait object** — that makes exactly this possible, at the cost of a different compile-time decision. That's the subject of the next lesson: [2.3.7 — Trait objects vs static dispatch](../07-static-vs-dynamic-dispatch/README.md).

**Part two.** (This looks ahead of today, and we know it.) Look up the real definition of `std::cmp::Ord` in the standard docs. How many supertraits does it have? Why does it need both `Eq` and `PartialOrd` — what's the difference between them that makes one insufficient on its own?

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Supertrait | `trait B: A` — implementing `B` requires already implementing `A` | trait hierarchies, like `Ord: Eq + PartialOrd` |
| Blanket impl | `impl<T: Bound> Trait for T` — one `impl`, for every type with that bound | `impl<T, U> Into<U> for T where U: From<T>` |
| Orphan rule | trait or type — at least one must be local | prevents two unrelated crates from colliding |
| Newtype escape hatch | a local wrapper around a foreign type, to make that side local | `impl Display for Vec<T>`, indirectly |
| `E0277` (here) | a required supertrait was never implemented | write the missing `impl` |
| `E0117` | neither the trait nor the type is local | build a newtype, or write your own trait |

### What you now know

- `trait B: A` means no type can implement `B` unless it has already implemented `A` — a dependency, not inheritance; nothing is transferred for free.
- A generic function that only asks for `T: B` can still call `A`'s methods on that same `T`.
- `impl<T: Bound> Trait for T` implements a trait once for every qualifying type — exactly what the standard library did for `Into`.
- The orphan rule: `impl Trait for Type` is only legal if `Trait` or `Type` is local — so two unrelated crates can never own the same `impl`.
- Wrapping a foreign type in a local tuple struct makes it local for the orphan rule's purposes — the same newtype pattern from 1.5.2, this time to work around a rule, not just for type safety.

### What comes back later

- **Trait objects, for holding several different types behind one shared trait** — [2.3.7 — Trait objects vs static dispatch](../07-static-vs-dynamic-dispatch/README.md)
- **`Deref`, so a newtype can pass its inner type's methods through for free too** — [2.4.3 — `Deref`, `AsRef` and `Borrow`](../../04-lifetimes-and-conversion/03-deref-asref-borrow/README.md)

### Can you explain?

- What's the difference between a supertrait and object-oriented inheritance?
- Why can a function that only writes `T: Discounted` still call `.price_cents()` on that same `T`?
- Explain the `Into` blanket impl out loud, word by word — as if talking to someone who just learned what `where` means.
- What exactly does the orphan rule forbid? Why does it exist?
- What problem did newtype solve in 1.5.2, and what different problem does it solve today?

---

## Going further

- [The Rust Book — Using supertraits](https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#using-supertraits-to-require-one-traits-functionality-within-another-trait) — the same material, from the official source.
- [The Rust Reference — Orphan rules](https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules) — the precise, official definition of the rule, the same one the `E0117` message links to.
- [`std::convert::Into`](https://doc.rust-lang.org/std/convert/trait.Into.html) — where today's blanket impl is actually defined.
- [Rust API Guidelines — Newtype types](https://rust-lang.github.io/api-guidelines/type-safety.html#newtypes-provide-static-distinctions-c-newtype) — the same guide 1.5.2 introduced, for the type-safety use of newtypes. The orphan-rule escape hatch this lesson adds is a second job the same pattern happens to be good at, not something that page itself discusses.
