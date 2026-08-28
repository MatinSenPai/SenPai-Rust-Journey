# 2.3.3 — `From`, `Into`, `TryFrom`, and `TryInto`

## At a glance

After this lesson you can:

- Say why `impl From<T> for U` means "this conversion can never fail," and why `i32::from(u8)` exists but `u8::from(i32)` does not.
- Choose, with a reason, between `as` (silent, quiet) and `try_into()` (honest, returns a `Result`) for a narrowing numeric conversion.
- Implement `TryFrom` for your own type, say why you get `TryInto` for free, and choose — with a reason — between `From` and `TryFrom` for a new type.

**Time:** ~60 minutes · **Prerequisites:**
[1.6.5 — `From` and error conversion](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.md),
[2.3.1 — Defining and implementing traits](../01-defining-and-implementing-traits/README.md)

---

## Why this matters

[1.6.5](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.md) gave you `From`, but scoped to one job: converting error types so `?` could do it automatically. That lesson even said, in passing, "`From` isn't only for errors" and pointed at `String::from` and `u64::from(u32)` — but that one paragraph was all the room it had. This lesson finishes the sentence: `From`/`Into` are a general-purpose pair for **any** conversion that can never fail, not just errors.

Not every conversion is like that, though. A backend handling a request constantly meets input that *might* be invalid: a star rating that should be 1 to 5 but the caller sent 9, a 64-bit number that has to fit a 32-bit database column but might not. `From` is a lie for this case: its signature promises "I always succeed," but the function underneath either has to panic or return a made-up value — neither is honest. Python doesn't separate these two cases in a function's signature at all: `int("12")` succeeds, `int("abc")` raises a `ValueError`, and neither fact is written into `int()`'s own signature — you have to read the docs. Rust carries the distinction in the return type itself: a `From`-based function always hands back the target type directly; a `TryFrom`-based one always hands back a `Result` — exactly what [1.6.1](../../../phase1-fundamentals/06-absence-and-failure/01-option-and-null-safety/README.md) said about `Option`: because the type is different, the compiler won't let you skip past the possibility of failure without facing it.

This lesson gives you the second tool: `TryFrom`, `From`'s fallible sibling, which makes the same promise plus an honest way to say no.

---

## The concept

### 1. `From` — you know it already, now for every always-succeeds conversion

1.6.5 showed you `impl From<A> for MyError` so `?` could convert errors, and the blanket impl `impl<T, U> Into<U> for T where U: From<T>` — the thing that makes `.into()` free. We are not rebuilding that mechanism here, just widening its scope. `From<T> for U` means: **every valid `T`, without exception, converts to a `U`.**

```rust
let score: u8 = 200;
let widened: i32 = i32::from(score);
println!("i32::from(u8): {widened}");
```

```text
i32::from(u8): 200
```

Every `u8` (0 to 255) fits inside an `i32` with nothing lost — no bit pattern goes missing. That's why the standard library gives you this `impl From<u8> for i32` for free; you never write it yourself. This kind of conversion has a name: **widening** — the target always has room for the source.

```rust
let delta: i32 = -12_000;
let as_float: f64 = f64::from(delta);
println!("f64::from(i32): {as_float}");
```

```text
f64::from(i32): -12000
```

A different pair, same idea: an `f64`'s mantissa has 52 bits — far more than the 32 an `i32` needs — so every `i32` also fits inside an `f64` without losing anything.

### 2. When the reverse direction can fail: `TryFrom`

Try the reverse: not every `i32` fits inside a `u8` — a `u8` only holds 0 to 255. This is called **narrowing**, and here `From` can no longer honestly exist. The standard library implements `TryFrom` instead:

```rust
pub trait TryFrom<T>: Sized {
    type Error;
    fn try_from(value: T) -> Result<Self, Self::Error>;
}
```

Same shape as `From`, with two differences: the method is called `try_from`, and instead of handing back `Self` directly it hands back a `Result<Self, Self::Error>`. That `type Error` is an **associated type** — a placeholder the trait declares that each implementor fills in exactly once; the full mechanism is [2.3.5](../05-associated-types/README.md), for now just know every `TryFrom` has to state what its own failure looks like. (And since this course targets the 2021 edition, `TryFrom`/`TryInto` are already in the prelude — nothing in this lesson ever needed `use std::convert::TryFrom;`.)

Now the same conversion, reversed:

```rust
let fits: Result<u8, _> = u8::try_from(200i32);
let overflow: Result<u8, _> = u8::try_from(300i32);
let negative: Result<u8, _> = u8::try_from(-1i32);

println!("u8::try_from(200i32): {fits:?}");
println!("u8::try_from(300i32): {overflow:?}");
println!("u8::try_from(-1i32):  {negative:?}");
```

```text
u8::try_from(200i32): Ok(200)
u8::try_from(300i32): Err(TryFromIntError(PosOverflow))
u8::try_from(-1i32):  Err(TryFromIntError(NegOverflow))
```

`200` fits: `Ok(200)`. `300` and `-1` both don't — but even the failure carries information: `PosOverflow` says "too big," `NegOverflow` says "too small." Exactly the same `i32`/`u8` pair from the last section: one direction (`i32::from(u8)`) can never fail, the other (`u8::try_from(i32)`) can — same two types, just the direction flipped.

```senpai-visual
{"kind":"result","labels":["u8 try_from i32","fits in 0 to 255?","Ok: u8","Err: TryFromIntError"]}
```

### 3. `as` versus `try_into`: one is silent, one is honest

[1.1.2](../../../phase1-fundamentals/01-foundations/02-scalar-types-and-overflow/README.md) showed you the `as` operator. Now that you have `TryFrom`, it's worth putting the two side by side:

```rust
let big: i32 = 300;

let truncated = big as u8;
println!("300i32 as u8:      {truncated}");

let honest: Result<u8, _> = big.try_into();
println!("300i32.try_into(): {honest:?}");
```

```text
300i32 as u8:      44
300i32.try_into(): Err(TryFromIntError(PosOverflow))
```

`as` never fails; it just takes an `i32`'s 32 bits and keeps the lowest 8 — the rest is thrown away. 300 in binary is `100101100`; only the last 8 bits (`00101100`, which is 44) survive. `try_into()` is the same `try_from`, read from the destination's side — exactly the type inference [1.6.5](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.md) showed you for `.into()`, now for the fallible pair: the written type `Result<u8, _>` on `let honest` is what tells the compiler which `try_from` to call.

The danger is exactly here: `256i32 as u8` also becomes `0` — not an error, not a panic, just a perfectly reasonable-looking number that is completely wrong.

```rust
let wrapped = 256i32 as u8;
println!("256i32 as u8: {wrapped}");
```

```text
256i32 as u8: 0
```

**Rule:** don't reach for `as` as your default for a numeric conversion; save it for the places where truncating bits is genuinely what you want. Everywhere else, `try_into()` is what honestly tells you whether the conversion worked.

### 4. Implementing `TryFrom` for your own type: the validated newtype

Say you're taking a star rating — a number from 1 to 5 — and you don't want a raw, never-checked `u8` passed around your codebase. Wrap it in a newtype (which [1.5.2](../../../phase1-fundamentals/05-your-own-types/02-tuple-structs-and-newtype/README.md) showed you), except this time the constructor *is* the validation:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rating(u8);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RatingError {
    OutOfRange(u8),
}
```

```rust
impl TryFrom<u8> for Rating {
    type Error = RatingError;

    fn try_from(raw: u8) -> Result<Self, Self::Error> {
        if (1..=5).contains(&raw) {
            Ok(Rating(raw))
        } else {
            Err(RatingError::OutOfRange(raw))
        }
    }
}
```

```rust
let five_star = Rating::try_from(5);
let zero_star = Rating::try_from(0);
println!("Rating::try_from(5): {five_star:?}");
println!("Rating::try_from(0): {zero_star:?}");
```

```text
Rating::try_from(5): Ok(Rating(5))
Rating::try_from(0): Err(OutOfRange(0))
```

`Rating`'s field is private — `Rating(u8)`, not a `pub` field. That means the only way to build a `Rating` is through `TryFrom::try_from`; no code, anywhere in the program, can construct a `Rating(9)` without going through that `if`. The consequence: the moment you're holding a `Rating`, that fact alone proves the value was checked — you never need to look at it twice.

This is what you'd usually reach for a checker function or a pydantic validator for in Python: check the value at the edge. The difference is that pydantic's check is something you *have* to call everywhere, and mypy has no way to remind you — miss one path and an invalid value slips through unnoticed. Here, because the field is private, forgetting isn't even possible: the compiler guarantees it, not the author's discipline.

### 5. `TryInto` arrives for free too

Exactly what [1.6.5](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.md) showed for `Into` happens here too — this time for the fallible pair. The standard library writes this once, for everyone:

```rust
impl<T, U> TryInto<U> for T
where
    U: TryFrom<T>,
{
    type Error = U::Error;
    fn try_into(self) -> Result<U, U::Error> {
        U::try_from(self)
    }
}
```

We never wrote `impl TryInto` anywhere — only `impl TryFrom<u8> for Rating` (section 4) — and `TryInto` came free:

```rust
let from_call: Result<Rating, RatingError> = Rating::try_from(4);
let from_method: Result<Rating, RatingError> = 4u8.try_into();
println!("Rating::try_from(4): {from_call:?}");
println!("4u8.try_into():      {from_method:?}");
```

```text
Rating::try_from(4): Ok(Rating(4))
4u8.try_into():      Ok(Rating(4))
```

`?` does exactly what you learned in [1.6.3](../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.md) — on a `Result<Rating, RatingError>` it behaves exactly as it does on any other `Result`:

```rust
fn build(raw: u8) -> Result<Rating, RatingError> {
    let rating: Rating = raw.try_into()?;
    Ok(rating)
}
```

```text
build(3): Ok(Rating(3))
build(7): Err(OutOfRange(7))
```

### 6. The choice rule: when `From`, when `TryFrom`

The rule is one sentence: **if the conversion always succeeds for every valid input, write `From`; if even one input has to be rejected — validation, a range check, parsing — write `TryFrom`.** An `impl From` that panics on "bad" input is a lie in the type system: its signature promises "always," and it doesn't deliver.

The direction of a conversion carries the same rule, even on a single type. Section 4 was the incoming direction — a raw `u8` that might be invalid. The reverse direction is different:

```rust
impl From<Rating> for u8 {
    fn from(value: Rating) -> Self {
        value.0
    }
}
```

Every `Rating` that exists has already gone through that `if` in section 4 — there is nothing left to reject. Pulling the `u8` back out cannot fail, so it's `From`, not `TryFrom`:

```rust
let rating = Rating::try_from(4).unwrap();
let raw: u8 = rating.into();
println!("Rating::try_from(4).unwrap().into(): {raw}");
```

```text
Rating::try_from(4).unwrap().into(): 4
```

```senpai-visual
{"kind":"concept","labels":["every input can succeed?","yes: impl From","no: impl TryFrom","Result Self Error"]}
```

A side note, for later: both `impl`s above were on `Rating`, a type we defined ourselves. Rust has a rule about which side of an `impl From<X> for Y` has to belong to you before you're even allowed to write it — the full rule is [2.3.6](../06-supertraits-blanket-impls-orphan-rule/README.md).

---

## Hands on

```sh
cargo run -p p2-03-03-from-into-tryfrom --example 01-from-is-still-infallible
cargo run -p p2-03-03-from-into-tryfrom --example 02-numeric-narrowing-tryfrom
cargo run -p p2-03-03-from-into-tryfrom --example 03-as-vs-try-into
cargo run -p p2-03-03-from-into-tryfrom --example 04-tryfrom-for-your-own-type
cargo run -p p2-03-03-from-into-tryfrom --example 05-tryinto-for-free
cargo run -p p2-03-03-from-into-tryfrom --example 06-choosing-from-or-tryfrom
```

Then the three broken ones:

```sh
cargo run -p p2-03-03-from-into-tryfrom --example 07-result-not-a-value --features broken
cargo run -p p2-03-03-from-into-tryfrom --example 08-narrow-panics-on-overflow --features broken
cargo run -p p2-03-03-from-into-tryfrom --example 09-missing-error-type --features broken
```

Then try:

1. In `02-numeric-narrowing-tryfrom`, also print `u8::try_from(255i32)` and `u8::try_from(256i32)`. Exactly where is the line between `Ok` and `Err`?
2. In `03-as-vs-try-into`, try the same conversion to `i8` instead of `u8` (`300i32 as i8`). Do you still see 44? Why or why not?
3. In `06-choosing-from-or-tryfrom`, write a function `average(ratings: &[Rating]) -> f64` that averages several `Rating`s — using the same `From<Rating> for u8` written there.

---

## Errors you will meet

### `E0308` — a `try_into` result is not a value, it's a `Result`

`examples/07-result-not-a-value.rs` makes exactly the mistake section 2 warned about: `try_into()` returns a `Result<Rating, RatingError>`, not a bare `Rating` — but this code is written as if it were `From`:

```text
error[E0308]: mismatched types
  --> phase2-intermediate\03-traits-and-generics\03-from-into-tryfrom\examples\07-result-not-a-value.rs:29:26
   |
29 |     let rating: Rating = 4u8.try_into();
   |                 ------   ^^^^^^^^^^^^^^ expected `Rating`, found `Result<_, _>`
   |                 |
   |                 expected due to this
   |
   = note: expected struct `Rating`
                found enum `Result<_, _>`
help: consider using `Result::expect` to unwrap the `Result<_, _>` value, panicking if the value is a `Result::Err`
   |
29 |     let rating: Rating = 4u8.try_into().expect("REASON");
   |                                        +++++++++++++++++
```

**What the compiler is objecting to:** the message is exact — "`Rating` expected, `Result<_, _>` found." `let rating: Rating` wants exactly `Rating`; `4u8.try_into()` gives back a `Result`, not `Rating` directly — precisely the difference that separates `TryFrom` from `From`.

**The fix:** treat the `Result` like any other `Result` — `match`, `?`, or (when you're genuinely certain the answer is `Ok`) `.unwrap()`:

```rust
let rating: Rating = 4u8.try_into().unwrap();
```

**Why that's the fix:** the compiler's own suggestion (`.expect("REASON")`) also works, but it's more honest to know this is a deliberate decision, not a bug fix: either you genuinely believe `Err` can't happen here (and write `.expect()` saying why), or you need to answer both cases.

### Panic — `.unwrap()` on an overflow `Err`

`examples/08-narrow-panics-on-overflow.rs` compiles — `.unwrap()` type-checks on any `Result<T, E>` — and then dies at exactly the point where `300` doesn't fit inside a `u8`:

```text
thread 'main' (416) panicked at phase2-intermediate\03-traits-and-generics\03-from-into-tryfrom\examples\08-narrow-panics-on-overflow.rs:10:38:
called `Result::unwrap()` on an `Err` value: TryFromIntError(PosOverflow)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**What the compiler is objecting to:** this isn't even a compiler error — the program built and ran, and panicked the moment `.unwrap()` saw an `Err`. The same `PosOverflow` section 2 showed you, this time stopping the program instead of quietly printing.

**The fix:** either make sure the value fits, or give the `Err` case an honest fallback path — exactly what `saturating_narrow` does in the exercises:

```rust
let narrowed = u8::try_from(300i32).unwrap_or(u8::MAX);
```

**Why that's the fix:** `.unwrap()` is a bet that the answer is always `Ok`; when that bet is wrong, the program stops entirely instead of handing back a wrong answer — often the right call, but not always what you want. `.unwrap_or(...)` (which [1.6.2](../../../phase1-fundamentals/06-absence-and-failure/02-option-combinators/README.md) taught you) gives an explicit third path: no panic, no lie, an explicit fallback value.

### `E0046` — `TryFrom` needs two things, not one

`examples/09-missing-error-type.rs` writes the `try_from` method but leaves out `type Error` — exactly the second item section 2 showed the trait declares:

```text
error[E0046]: not all trait items implemented, missing: `Error`
  --> phase2-intermediate\03-traits-and-generics\03-from-into-tryfrom\examples\09-missing-error-type.rs:16:1
   |
16 | impl TryFrom<u8> for Rating {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `Error` in implementation
   |
   = help: implement the missing item: `type Error = /* Type */;`
```

**What the compiler is objecting to:** `TryFrom` has two members, an associated type (`Error`) and a method (`try_from`) — writing only one of them leaves the `impl` incomplete, exactly like [2.3.1](../01-defining-and-implementing-traits/README.md) showed you every method without a default body is mandatory.

**The fix:** add the missing line:

```rust
impl TryFrom<u8> for Rating {
    type Error = RatingError;
    // ...
}
```

**Why that's the fix:** the error message itself suggests exactly this — "implement `type Error = /* Type */;`". When you're hand-writing `TryFrom` on a brand-new type from scratch (not filling in a given skeleton), this line is as much a habit as `try_from` itself — don't forget it.

---

## Exercises

### Warm up

<details>
<summary>What does this print?</summary>

```rust
let x: Result<u8, _> = u8::try_from(-5i32);
println!("{x:?}");
```

</details>

<details>
<summary>Answer</summary>

```text
Err(TryFromIntError(NegOverflow))
```

`-5` is negative and no `u8` is negative, so it overflows from below — the same `NegOverflow` section 2 showed you.

</details>

<details>
<summary>Does <code>let x: u8 = 10i32.into();</code> compile?</summary>

Write down your answer before reading on.

</details>

<details>
<summary>Answer</summary>

No. You get `E0277`: "`u8: From<i32>` is not satisfied." No `impl From<i32> for u8` exists — an `i32` might not fit inside a `u8`, and `From` promises "always." You need `.try_into()`.

</details>

<details>
<summary>What does <code>256i32 as u8</code> become?</summary>

Write down your answer.

</details>

<details>
<summary>Answer</summary>

`0`. `as` only keeps the lowest 8 bits, and 256 is all zero in those 8 bits — not an error, not a panic, just a wrong number.

</details>

<details>
<summary>True or false: any type used as a <code>TryFrom</code>'s <code>Error</code> must implement <code>std::error::Error</code>.</summary>

Write down your answer.

</details>

<details>
<summary>Answer</summary>

False. `RatingError` in this very lesson only has `#[derive(Debug, Clone, PartialEq, Eq)]`, and that's entirely enough for `TryFrom`. `std::error::Error` belongs to [2.5.1](../../05-error-handling/01-custom-error-types/README.md), further ahead.

</details>

<details>
<summary>If <code>Rating</code>'s field were <code>pub</code> instead of private, would every existing <code>Rating</code> still be guaranteed valid?</summary>

Write down your answer.

</details>

<details>
<summary>Answer</summary>

No. Anyone could build `Rating(9)` directly and skip `TryFrom::try_from` entirely. The guarantee depends entirely on the field being private.

</details>

### Repair

Fix all three broken examples:

1. Fix `examples/07-result-not-a-value.rs` **two** ways: once by adding `.unwrap()`, once by changing `let rating`'s type to `Result<Rating, RatingError>`.
2. Fix `examples/08-narrow-panics-on-overflow.rs` so it no longer panics — with `.unwrap_or(u8::MAX)` or a `match` that answers both cases.
3. Fix `examples/09-missing-error-type.rs` by adding the line `type Error = RatingError;`.

### Implement

Four things in `src/lib.rs`:

```sh
cargo test -p p2-03-03-from-into-tryfrom
```

`impl From<Percentage> for f64` and `impl TryFrom<u8> for Percentage` both work on the same type, two different directions — exactly section 6's pattern. `impl TryFrom<String> for EmailAddress` is the same validated-newtype pattern from section 4, this time on a `String`. `saturating_narrow` is the same `try_into` with an explicit path for the overflow case you saw in "Errors you will meet," this time between `u64` and `u32`.

Implement all four exactly to the doc comment above each one — don't guess at anything.

### Build

Write a validated newtype for a domain of your choosing — a two-letter country code, a course grade (`0..=20`), a network port number (`1..=65535`). Write a `TryFrom` to build it from the raw type, and a `From` to hand the inner value back out — exactly the pair `Rating` showed you in this lesson.

### Challenge (optional)

**Part one.** Write `pub fn saturating_narrow_to_i8(value: i32) -> i8`. Mind the negative side too — `i32::MIN` should clamp to `i8::MIN`, not just the positive side to `i8::MAX`.

**Part two.** Suppose you want to write `impl From<Rating> for u8` — exactly what section 6 wrote — except this time `Rating` isn't your own crate's type, it comes from an external crate, and `u8` has always belonged to the standard library, not to you either. Can you still write this `impl`? Guess why or why not, in one sentence — you don't need the code; the full answer is [2.3.6](../06-supertraits-blanket-impls-orphan-rule/README.md).

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `From<T> for U` | an always-succeeds conversion from `T` to `U` (recall from 1.6.5) | your own types, widening numeric conversions |
| `TryFrom<T> for U` | a conversion that can fail; `try_from` returns a `Result<U, Self::Error>` | validation, parsing, narrowing numeric conversions |
| `TryInto` | the free result of writing `TryFrom` | `.try_into()` — never implemented by hand |
| `as` | a silent numeric conversion; drops the extra bits without a word | only where truncation is genuinely what you want |
| `TryFromIntError` | the standard error for narrowing numeric conversions | `PosOverflow` / `NegOverflow` |
| `E0046` | an `impl` that left out a trait's mandatory member | forgetting `type Error = ...;` |

### What you now know

- `From<T> for U` promises "never fails"; `TryFrom<T> for U` gives the same conversion wrapped in a `Result<U, Self::Error>`.
- `TryFrom` and `TryInto` come free with each other — exactly like the `From`/`Into` pair 1.6.5 showed you.
- `as` never fails because it never actually checks — it just cuts bits; `try_into()` reports the same conversion honestly.
- A newtype with a private field and a `TryFrom` constructor guarantees every instance that exists has been validated — you never need to check it again.
- The choice rule: if every valid input always succeeds, `From`; if even one has to be rejected, `TryFrom` — even on a single type, its two directions can have two different answers.

### What comes back later

- **Hand-written `Display`, so your error prints a readable message instead of just `{:?}`** — [2.3.4 — Standard derives, by hand](../04-standard-derives-by-hand/README.md)
- **Associated types in full** — [2.3.5 — Associated types](../05-associated-types/README.md)
- **The orphan rule — why `impl From<Rating> for u8` was allowed but not every combination is** — [2.3.6 — Supertraits, blanket impls, the orphan rule](../06-supertraits-blanket-impls-orphan-rule/README.md)
- **`std::error::Error`, for a genuinely complete error type** — [2.5.1 — Custom error types](../../05-error-handling/01-custom-error-types/README.md)
- **`thiserror` and `anyhow`, for when you'd rather not write this boilerplate yourself** — [2.5.3 — `thiserror` and `anyhow`](../../05-error-handling/03-thiserror-and-anyhow/README.md)

### Can you explain?

- Why does `i32::from(u8)` exist but not `u8::from(i32)`?
- Where does `try_into()` read its target from? The same question 1.6.5 asked about `.into()`.
- Why does `as` never panic, even when the answer is completely wrong?
- How does the `Rating` newtype guarantee every instance is valid, without anywhere else in the code checking it again?
- For a single type, why can one direction of a conversion be `From` and the other `TryFrom`?
- A `TryFrom` that left out `type Error` — exactly when, and with what error, does it tell you?

---

## Going further

- [`std::convert::From` docs](https://doc.rust-lang.org/std/convert/trait.From.html) — the full list of widening numeric conversions the standard library implements itself.
- [`std::convert::TryFrom` docs](https://doc.rust-lang.org/std/convert/trait.TryFrom.html) — the same, for the fallible pair.
- [The Rust Edition Guide — prelude additions in 2021](https://doc.rust-lang.org/edition-guide/rust-2021/prelude.html) — why nothing in this lesson ever needed `use std::convert::TryFrom;`.
- [The `clippy::cast_possible_truncation` lint](https://rust-lang.github.io/rust-clippy/master/#cast_possible_truncation) — the exact mistake this lint targets: an `as` that can silently lose data.
