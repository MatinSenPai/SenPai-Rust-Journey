# 2.3.5 — Associated types versus generic parameters

## At a glance

After this lesson you can:

- Explain why `type Item;` on `Iterator` does not make the trait itself generic, but `trait Converts<T>` does — and exactly what that difference changes.
- Say how many times a type can implement an associated-type trait versus a trait with a generic parameter — and prove that claim with the real compiler, not just a definition.
- Choose between an associated type and a generic parameter for a real trait design, and say exactly why the other shape would have broken.

**Time:** ~60 minutes · **Prerequisites:**
[2.3.1 — Defining and implementing traits](../01-defining-and-implementing-traits/README.md),
[2.3.2 — Generic functions and structs](../02-generic-functions-and-structs/README.md),
[2.2.4 — Implementing `Iterator` and `IntoIterator` for your own type](../../02-iterators-and-closures/04-implementing-iterator/README.md)

---

## Why this matters

The `Iterator` trait made a decision for you — without ever asking. In [2.2.4](../../02-iterators-and-closures/04-implementing-iterator/README.md) you passed right by it: "The associated type does not make `Iterator` itself generic; each implementer just says 'mine is this,' and that is enough." That exact decision is why `iter.next()` can always be called, everywhere, without ever writing an explicit type.

But once you design traits yourself — [2.3.1](../01-defining-and-implementing-traits/README.md) started you on that — this decision stops being free. You have to choose: write `type Target;`, or write `trait Foo<T>`. The compiler does not correct this choice for you if you get it wrong; it just builds a different design, with its own limits and its own costs. Getting it wrong here compiles and runs fine too, exactly like picking the wrong collection did in [2.1.4](../../01-collections/04-choosing-a-collection/README.md) — you just get stuck later: either a type that genuinely needed several answers gets locked to one, or every call to an otherwise simple method is forced to spell out its type by hand, even where there was only ever one sensible answer.

This lesson opens up exactly that decision — with a real compiler, not just a definition.

---

## The concept

### `Iterator::Item`: one type, forever

The `Iterator` trait has this shape — with no parameter at all on `trait Iterator` itself:

```rust
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

`type Item;` is a placeholder, not a value. Each implementation fills that placeholder exactly once. Build a countdown to show it:

```rust
struct Countdown {
    remaining: u8,
}
impl Iterator for Countdown {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.remaining == 0 {
            return None;
        }
        let value = self.remaining;
        self.remaining -= 1;
        Some(value)
    }
}
```

Now try everything [2.2](../../02-iterators-and-closures/README.md) taught you on it — without ever saying what `Item` is:

```rust
let full = Countdown { remaining: 5 }.count();
let odd = Countdown { remaining: 5 }.filter(|v| v % 2 == 1).count();
println!("full: {full}");
println!("odd: {odd}");
```

```text
full: 5
odd: 3
```

Neither in `Countdown`'s definition, nor in calling `.count()` or `.filter()`, did you ever have to write what `Item` is. The compiler already knows: because `Countdown` implements `Iterator` exactly once, its `Item` is exactly one thing — `u8` — always, everywhere. That "always, everywhere" is exactly the mechanism today opens up.

(Traits like `DoubleEndedIterator` build on top of `Iterator` itself — a completely different relationship from what today covers, and the subject of [2.3.6](../06-supertraits-blanket-impls-orphan-rule/README.md).)

### The same idea, as a trait of your own

`Iterator` made this decision for you. Now write two traits yourself that do almost the same job — "convert this value to some other type" — one with an associated type, one with a generic parameter:

```rust
trait ConvertsTo {
    type Target;
    fn convert(&self) -> Self::Target;
}

trait Converts<T> {
    fn convert(&self) -> T;
}
```

The method shape is nearly identical — both are `fn convert(&self) -> ...`. The difference is exactly this one spot: `Self::Target` (something the type itself decides, once) versus `T` (something the trait itself takes, on every `impl`). That one small difference builds two completely different behaviors.

### An associated type: one way, forever

Implement `ConvertsTo` for a real type — a Celsius temperature, that you want the Fahrenheit of:

```rust
struct Celsius(f64);
impl ConvertsTo for Celsius {
    type Target = f64;

    fn convert(&self) -> f64 {
        self.0 * 9.0 / 5.0 + 32.0
    }
}
```

```rust
let boiling = Celsius(100.0);
println!("{}", boiling.convert());
```

```text
212
```

It works, and that's all it took. Now suppose you also want to convert `Celsius` into a descriptive string — a second `impl ConvertsTo` for the same `Celsius`, with `type Target = String;`. It does not compile — `E0119`; "Errors you will meet" has the exact message.

Why? Because `trait ConvertsTo` itself takes no parameter. As far as the compiler is concerned there is exactly one possible pair: "trait `ConvertsTo`, for type `Celsius`." One pair, one `impl`, one `Target`. Forever.

### A generic parameter: many ways, one per `T`

Now do the same thing with `Converts<T>` — and this time actually implement it twice:

```rust
impl Converts<f64> for Celsius {
    fn convert(&self) -> f64 {
        self.0 * 9.0 / 5.0 + 32.0
    }
}
impl Converts<String> for Celsius {
    fn convert(&self) -> String {
        format!("{:.1}C", self.0)
    }
}
```

```rust
let fahrenheit: f64 = boiling.convert();
let label: String = boiling.convert();
println!("{fahrenheit}");
println!("{label}");
```

```text
212
100.0C
```

This time it compiled — the same method name, the same type, implemented twice. Why does this one get to, when the last section didn't?

Because `Converts<T>` itself takes a parameter. `Converts<f64>` and `Converts<String>` are two completely different things as far as the compiler is concerned — two different pairs: "`Converts<f64>`, for `Celsius`" is one, "`Converts<String>`, for `Celsius`" is another. Each has its own pair, so each gets its own `impl`.

If that claim is true, it also makes a precise prediction: two `impl Converts<f64> for Celsius` — the same `T`, twice — should hit exactly the last section's problem, because now you genuinely have a duplicate pair. Try it — you get `E0119`, this time with `Converts<f64>` in the message instead of `ConvertsTo`. "Errors you will meet" has the exact message.

> **Remember this?** You've already met a generic trait once before, without its name attached: [1.6.5](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.md), `From<T>`. `String` implements `From<&str>`, and `From<char>`, and several others — all of them at once, on that same one type, for exactly this reason.

### Where's the cost: ambiguity at the call site

Look back at the first section: `Countdown { remaining: 5 }.count()` never said what `Item` is, and it compiled. Now try the same thing with `Converts<T>` — with no type written anywhere:

```rust
let result = boiling.convert();
```

It doesn't compile. `E0283` — the compiler says both `Converts<f64>` and `Converts<String>` exist, and nothing here says which one you want. "Errors you will meet" has the exact message; the compiler itself even suggests a fix.

This is exactly the cost of the "many ways" side: anywhere more than one `impl` exists, the call site has to say which one it wants. Two ways to say it:

```rust
let by_annotation: f64 = boiling.convert();
let by_qualified = <Celsius as Converts<String>>::convert(&boiling);
println!("{by_annotation}");
println!("{by_qualified}");
```

```text
212
100.0C
```

The first way — an explicit type on the binding — is what the previous section already used. The second — fully qualified syntax — spells out exactly which trait, with which `T`, for which type. Both tell the compiler the same thing: which pair you mean.

### Which one? One question, one table

All of this comes down to one question:

```senpai-visual
{"kind":"concept","labels":["does this type always have one answer?","yes → associated type","no, it has several?","does the caller choose?","yes → generic parameter"]}
```

| | Associated type (`type Target;`) | Generic parameter (`Converts<T>`) |
|---|---|---|
| How many times can one type implement it? | Exactly once | Once per distinct `T` |
| Does the call site ever need an explicit type? | Never | Whenever more than one `impl` exists |
| Standard library example | `Iterator::Item` | `From<T>` / `Into<T>` |
| When to reach for it | This type always, forever, has exactly one correct answer | This type genuinely has more than one answer, and the caller should choose |

The rule is exactly that short: if your type *always*, *forever*, has exactly one correct answer, reach for an associated type — the call site never has to write a type. If it genuinely has more than one correct answer and the caller should choose, reach for a generic parameter — and accept that the call site will sometimes have to say which one it wants.

This table answers the question for when the compiler already knows, at compile time, exactly which type it's dealing with. There's a different question for when you only find that out at run time — that one is the next lesson's subject.

---

## Hands on

```sh
cargo run -p p2-03-05-associated-types --example 01-one-way-associated-type
cargo run -p p2-03-05-associated-types --example 02-many-ways-generic-parameter
cargo run -p p2-03-05-associated-types --example 03-iterator-item-is-fixed
cargo run -p p2-03-05-associated-types --example 04-two-ways-to-disambiguate
```

Then the four broken ones:

```sh
cargo run -p p2-03-05-associated-types --example 05-two-impls-conflict --features broken
cargo run -p p2-03-05-associated-types --example 06-ambiguous-convert-call --features broken
cargo run -p p2-03-05-associated-types --example 07-mismatched-target-type --features broken
cargo run -p p2-03-05-associated-types --example 08-same-target-twice-conflicts --features broken
```

Then try these:

1. In `02-many-ways-generic-parameter.rs`, add a third `impl Converts<bool>` for `Celsius` (for example: "is it above the boiling point?"). How many lines of the rest of the file actually have to change for it to keep compiling?
2. In `04-two-ways-to-disambiguate.rs`, write `by_annotation` with fully qualified syntax too, instead of a type on the binding. Does the output change?
3. In `03-iterator-item-is-fixed.rs`, add a second `impl Iterator for Countdown` that sets `Item` to `i32`. What does the compiler say, and which of the next section's errors is it?

---

## Errors you will meet

### `E0119` — two conflicting implementations of `ConvertsTo`

```rust
struct Celsius(f64);
impl ConvertsTo for Celsius {
    type Target = f64;
    fn convert(&self) -> f64 {
        self.0 * 9.0 / 5.0 + 32.0
    }
}
impl ConvertsTo for Celsius {
    type Target = String;
    fn convert(&self) -> String {
        format!("{:.1}C", self.0)
    }
}
```

```text
error[E0119]: conflicting implementations of trait `ConvertsTo` for type `Celsius`
  --> phase2-intermediate\03-traits-and-generics\05-associated-types\examples\05-two-impls-conflict.rs:24:1
   |
16 | impl ConvertsTo for Celsius {
   | --------------------------- first implementation here
...
24 | impl ConvertsTo for Celsius {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^ conflicting implementation for `Celsius`

For more information about this error, try `rustc --explain E0119`.
```

**What the compiler is objecting to:** `ConvertsTo` takes no parameter on the trait itself. As far as the compiler is concerned there is exactly one possible pair — "`ConvertsTo`, for `Celsius`" — and you're answering for that same pair twice: once `Target = f64`, once `Target = String`. The compiler cannot tell which one is right, so it accepts neither.

**The fix:** keep one of the two `impl` blocks — or, if you genuinely need both conversions, go back to "A generic parameter: many ways, one per `T`": `Converts<T>` was built for exactly this.

**Why this is the fix:** an associated type promises "one fixed answer." Keeping two `impl` blocks breaks that promise. Either keep the promise (delete one), or don't make that promise in the first place (reach for the generic parameter).

### `E0283` — `.convert()` doesn't know which `impl` you want

```rust
let boiling = Celsius(100.0);
let result = boiling.convert();
println!("{result}");
```

```text
error[E0283]: type annotations needed
  --> phase2-intermediate\03-traits-and-generics\05-associated-types\examples\06-ambiguous-convert-call.rs:28:9
   |
28 |     let result = boiling.convert();
   |         ^^^^^^           ------- type must be known at this point
   |
note: multiple `impl`s satisfying `Celsius: Converts<_>` found
  --> phase2-intermediate\03-traits-and-generics\05-associated-types\examples\06-ambiguous-convert-call.rs:14:1
   |
14 | impl Converts<f64> for Celsius {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
20 | impl Converts<String> for Celsius {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
help: consider giving `result` an explicit type
   |
28 |     let result: /* Type */ = boiling.convert();
   |               ++++++++++++

For more information about this error, try `rustc --explain E0283`.
```

**What the compiler is objecting to:** the message says it directly — "multiple `impl`s satisfying `Celsius: Converts<_>` found." Both `Converts<f64>` and `Converts<String>` exist, and `result` has no type at all. The compiler cannot guess which `convert` you mean.

**The fix:** say which one you want:

```rust
let result: f64 = boiling.convert();
```

**Why this is the fix:** the compiler's own suggestion ("consider giving `result` an explicit type") is exactly this. This is the price you pay for a generic parameter: anywhere more than one `impl` exists, the caller has to choose — exactly what "Where's the cost" said in "The concept."

### `E0308` — the body doesn't match the declared `Target`

```rust
impl ConvertsTo for Celsius {
    type Target = f64;
    fn convert(&self) -> Self::Target {
        format!("{:.1}C", self.0)
    }
}
```

```text
error[E0308]: mismatched types
  --> phase2-intermediate\03-traits-and-generics\05-associated-types\examples\07-mismatched-target-type.rs:19:9
   |
18 |     fn convert(&self) -> Self::Target {
   |                          ------------ expected `f64` because of return type
19 |         format!("{:.1}C", self.0)
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^ expected `f64`, found `String`

For more information about this error, try `rustc --explain E0308`.
```

**What the compiler is objecting to:** `type Target = f64;` is a promise, not a comment. The moment the compiler sees `Self::Target`, it substitutes `f64` and holds the body to that exact standard. The body returns a `String` — the promise is broken.

**The fix:** either make the body match `f64`:

```rust
fn convert(&self) -> Self::Target {
    self.0 * 9.0 / 5.0 + 32.0
}
```

or, if you genuinely want a string, change `Target` itself: `type Target = String;`.

**Why this is the fix:** an associated type is decided exactly once — but that one decision applies everywhere, including the method's own signature, not just the type's name.

### `E0119` — again, and this time more precisely

```rust
impl Converts<f64> for Celsius {
    fn convert(&self) -> f64 {
        self.0 * 9.0 / 5.0 + 32.0
    }
}
impl Converts<f64> for Celsius {
    fn convert(&self) -> f64 {
        self.0 + 273.15
    }
}
```

```text
error[E0119]: conflicting implementations of trait `Converts<f64>` for type `Celsius`
  --> phase2-intermediate\03-traits-and-generics\05-associated-types\examples\08-same-target-twice-conflicts.rs:21:1
   |
15 | impl Converts<f64> for Celsius {
   | ------------------------------ first implementation here
...
21 | impl Converts<f64> for Celsius {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ conflicting implementation for `Celsius`

For more information about this error, try `rustc --explain E0119`.
```

**What the compiler is objecting to:** this time the message says `Converts<f64>`, not `ConvertsTo`. It's exactly the same mechanism as the first error — a duplicate pair — but now on the pair "`Converts<f64>`, for `Celsius`." These were not two different implementations; they were the same one, twice.

**The fix:** delete one of the two `impl Converts<f64>` blocks — or, if you genuinely want two different conversions to `f64` (Fahrenheit, Kelvin), those are no longer "the same `T`" as far as usage goes, so you're forced to give them separate names, for instance as two ordinary methods instead of two `impl`s of one trait.

**Why this is the fix:** this error proves "The concept" section's claim: the restriction is on the *(trait, type) pair*, not on the trait's name. `Converts<f64>` is one pair, `Converts<String>` is another — and each pair, exactly once.

---

## Exercises

### Warm up

<details>
<summary>For <code>impl Iterator for Countdown { type Item = u8; ... }</code>, can you also write a second <code>impl</code> with <code>type Item = i32;</code>?</summary>

Write your answer down before reading on.

</details>

<details>
<summary>Answer</summary>

No. `Iterator` takes no parameter on itself, so there is only one pair — "`Iterator`, for `Countdown`" — `E0119`.

</details>

<details>
<summary>Consider a hypothetical trait <code>trait Holds&lt;T&gt; { fn get(&self) -> T; }</code>. Can one type implement both <code>Holds&lt;i32&gt;</code> and <code>Holds&lt;String&gt;</code>, at the same time?</summary>

Write your answer down before reading on.

</details>

<details>
<summary>Answer</summary>

Yes. `Holds<i32>` and `Holds<String>` are two different pairs as far as the compiler is concerned, so both `impl`s are valid at once.

</details>

<details>
<summary>Does this compile?</summary>

```rust
trait Labels<T> {
    fn label(&self) -> T;
}
struct Item;
impl Labels<i32> for Item {
    fn label(&self) -> i32 {
        1
    }
}
impl Labels<bool> for Item {
    fn label(&self) -> bool {
        true
    }
}
let y = Item.label();
```

</details>

<details>
<summary>Answer</summary>

No — `E0283`. Both `Labels<i32>` and `Labels<bool>` exist, and `y` has no type at all.

</details>

<details>
<summary>True or false: calling <code>iter.next()</code> on a custom iterator of your own never needs an explicit type.</summary>

Write your answer down before reading on.

</details>

<details>
<summary>Answer</summary>

True. `Item` is fixed exactly once, in the `impl Iterator` definition — not at the call site. There is always exactly one answer, so there is never any ambiguity.

</details>

<details>
<summary>True or false: an associated type makes the trait itself generic.</summary>

Write your answer down before reading on.

</details>

<details>
<summary>Answer</summary>

False — exactly the opposite. An associated type is what keeps the trait from being generic. `trait Iterator` has no `<T>` on itself at all; `type Item;` only says each implementer substitutes something for it, once.

</details>

### Repair

Fix all four broken examples:

1. `examples/05-two-impls-conflict.rs` — delete one of the two `impl` blocks. Which one do you keep? Say why in a comment.
2. `examples/06-ambiguous-convert-call.rs` — say which conversion you want, with an explicit type or fully qualified syntax.
3. `examples/07-mismatched-target-type.rs` — either match the body to `Target`, or change `Target` itself. Try both; which one makes more sense when the type is named `Celsius` and `Target = f64`?
4. `examples/08-same-target-twice-conflicts.rs` — delete one of the two `impl Converts<f64>` blocks, or, if you genuinely want both conversions, give them separate names so they're no longer a repeat of the same `impl`.

### Implement

Two traits, already fully written, in `src/lib.rs` — your job isn't to design them, it's to write bodies that stay true to what each shape already promises:

```sh
cargo test -p p2-03-05-associated-types
```

`Measures` has an associated type — you implement it for `Rectangle` exactly once, and `measure()` never needs a type written anywhere. `DescribesAs<T>` has a generic parameter — you implement it for `Rectangle` twice, once for `u32` (the perimeter), once for `String` (a label). Each doc comment states exactly what it returns; don't guess at anything.

### Build

Build a `Track` (a song title, and its length in seconds). Two parts of your program need a "now playing" label from the same `Track`, at the same time: one, for a progress bar, wants just the raw number of seconds (`u32`); one, for display, wants a readable string like `"Title — 3:45"`.

Design a trait yourself — an associated type or a generic parameter, your choice — and implement it for `Track`. The exact string format is up to you; just say, in a comment above the trait, in a sentence or two, why the other shape wouldn't have worked here.

### Challenge (optional)

Look up the standard `std::ops::Add` trait. Its real signature is roughly: `pub trait Add<Rhs = Self> { type Output; fn add(self, rhs: Rhs) -> Self::Output; }` — it has both a generic parameter (`Rhs`) and an associated type (`Output`), both at once, on one trait.

Build a `Money` type (say, cents, as `i64`) and implement `Add` for it so two `Money` values can be added with `+`.

Then, just by thinking it through — you don't need to write the code: why does `Output` need to be an associated type, not a generic parameter? And why is `Rhs` the opposite — a generic parameter, not an associated type? (Hint: which of the two genuinely needs to have more than one correct answer?)

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| associated type (`type Target;`) | a type the implementer decides, once | when a type always has exactly one correct answer |
| generic trait parameter (`trait Foo<T>`) | a parameter written on the trait itself, not a function or struct | when a type genuinely has more than one correct answer |
| fully qualified syntax (`<Type as Trait<T>>::method()`) | telling the compiler exactly which trait and `T` you mean | resolving ambiguity between several `impl`s |
| `E0119` | two `impl`s for one (trait, type) pair | always a design decision gone wrong, never a typo |

### What you now know

- `type Item;` (and any associated type) does not make the trait itself generic; it only says each implementer substitutes something for it, once — which is exactly why its call site never needs a type written anywhere.
- A generic parameter on the trait itself (`trait Foo<T>`) does make the trait generic — `Foo<A>` and `Foo<B>` are different pairs, so one type can implement both at once.
- The "only one `impl`" restriction is on the *(trait, type) pair*, not on the trait's name — implementing `Converts<f64>` twice hits exactly the same error as `ConvertsTo` did.
- That freedom isn't free: anywhere more than one `impl` exists, the call site has to say which one it wants — with an explicit type or with fully qualified syntax.
- The choice between the two comes down to one question: does this type always have exactly one correct answer (associated type), or does it genuinely have several, with the caller choosing (generic parameter)?

### What comes back later

- **Traits that build on other traits (like `DoubleEndedIterator` building on `Iterator`)** — [2.3.6 — Supertraits, blanket impls, the orphan rule](../06-supertraits-blanket-impls-orphan-rule/README.md)
- **When you only know the exact type at run time, not at compile time** — [2.3.7 — Static vs. dynamic dispatch](../07-static-vs-dynamic-dispatch/README.md)

### Can you explain?

- Why doesn't `type Item;` make `Iterator` itself generic, while `trait Converts<T>` does make a trait generic?
- Why do two `impl ConvertsTo for Celsius` blocks conflict, but `impl Converts<f64> for Celsius` and `impl Converts<String> for Celsius` don't?
- Why do two `impl Converts<f64> for Celsius` blocks (the same `T`, twice) conflict too?
- Name the two ways to resolve ambiguity at a call site with more than one applicable `impl`.
- For a brand-new trait you're designing yourself, how do you decide between an associated type and a generic parameter?

---

## Going further

- [The Rust Book — Advanced traits, associated types](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html) — explains exactly this contrast, with the same `Iterator` example.
- [`Iterator` trait documentation](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
- [`std::ops::Add` trait documentation](https://doc.rust-lang.org/std/ops/trait.Add.html) — the same trait from "Challenge": an associated type and a generic parameter, together.
- [The Rust Reference — Associated items](https://doc.rust-lang.org/reference/items/associated-items.html) — the more formal, precise definition.
