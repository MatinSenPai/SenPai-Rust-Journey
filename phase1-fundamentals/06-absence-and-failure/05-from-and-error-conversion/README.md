# 1.6.5 — `From` and error conversion

## At a glance

After this lesson you can:

- Say exactly what `?` does to an `Err` before it returns early — it doesn't just return the value, it calls `From::from` on it.
- Write `impl From<A> for MyError` for your own error type, and say why writing that one thing also gets you `.into()` for free.
- Tell the difference between a place where type inference can work out `.into()`'s target and a place where it can't.
- Recognize an `E0277` from `?` and know the fix is the missing `From` impl, not another `.map_err()`.

**Time:** ~50 minutes · **Prerequisites:** [1.6.4 — Panic versus `Result`](../04-panic-vs-result/README.md) · [1.6.3 — `Result` and the question mark](../03-result-and-question-mark/README.md)

---

## Why this matters

The last lesson introduced `?` like this: "if it's `Err`, return that `Err` from the current function right now." That explanation is complete as long as every function you call returns the same error type you return.

The trouble starts when one function has to do three different things in a row, each of which fails its own way: turn a string into an integer (its error is `ParseIntError`), turn another string into a float (its error is `ParseFloatError`), validate a label against a small rule (its error is whatever you designed). Your function returns exactly one `Result<T, E>` — one `E`, not three. Slap a bare `?` on all three without thinking, and the compiler gives you a message that, at first glance, looks unrelated to anything you actually did.

The usual fix is to convert each one by hand. It works, but you write the same conversion three times, and the next function with three more fallible calls asks for the same repetition all over again.

Here's what this lesson opens up: **`?` was already doing that exact conversion — you just hadn't given it a name yet.**

---

## The concept

### A quick recall of `?` — and the question it hasn't answered

[1.6.3](../03-result-and-question-mark/README.md) taught you that `expr?` means: if it's `Ok(value)`, unwrap to `value` and keep going; if it's `Err(e)`, return `Err(e)` from the current function right now. That explanation leaves a question open: what if `e`'s error type isn't the same as the current function's error type? The compiler can't put two different types into one `Result`.

This lesson answers it.

### The manual way: one `match` and two `.map_err()`

Say you're reading a line like `"12,36.6,C"` — an id, a value, a unit — and parsing all three separately. Three fallible calls, three different error types:

```rust
let id = match id_str.parse::<u32>() {
    Ok(id) => id,
    Err(err) => return Err(ReadingError::BadId(err)),
};
```

That's exactly what you already know from [1.6.3](../03-result-and-question-mark/README.md): a `match` on a `Result`, with an early return in the `Err` arm. It works, but you'd write the same four lines again for every other fallible call.

It can be shortened. `Result` has a method called **`.map_err()`**: an `Ok` passes through untouched; an `Err(e)` gets run through whatever function you give it, and the result becomes the new `Err`. Same conversion, one line:

```rust
let value = value_str.parse::<f64>().map_err(ReadingError::BadValue)?;
let unit = parse_unit(unit_str).map_err(ReadingError::BadUnit)?;

Ok((id, value, unit))
```

`ReadingError::BadValue` here is the variant itself — Rust lets you call any data-carrying variant like a function, so it's enough on its own for `.map_err()`.

Run it and see:

```sh
cargo run -p p1-06-05-from-and-error-conversion --example 02-error-conversion-by-hand
```

```text
"12,36.6,C" -> Ok((12, 36.6, Celsius))
"x,36.6,C" -> Err(BadId(ParseIntError { kind: InvalidDigit }))
"12,hot,C" -> Err(BadValue(ParseFloatError { kind: Invalid }))
"12,36.6,K" -> Err(BadUnit("K"))
```

It works, and the answers are right — but write three more functions like this one and you retype the same three conversion lines three more times.

### The secret `?` was hiding

Read this carefully, because it's the whole lesson. `expr?` isn't just "return `Err` if it's `Err`." What it actually means is:

```rust
// what `expr?` means, conceptually:
match expr {
    Ok(value) => value,
    Err(error) => return Err(From::from(error)),
}
```

Look at that last line. Before `?` returns the error, it calls `From::from` on it. So if you state, once, how to turn a `ParseIntError` into a `ReadingError`, `?` uses exactly that at the call site — with nothing extra written there at all.

That statement is an **`impl From<A> for MyError`**.

### Writing `From<A> for MyError`

`From` is a standard trait with one method: `from`. For every source error `parse_reading` might see, you write one `impl From<that error> for ReadingError`:

```rust
impl From<ParseIntError> for ReadingError {
    fn from(err: ParseIntError) -> Self {
        ReadingError::BadId(err)
    }
}

impl From<ParseFloatError> for ReadingError {
    fn from(err: ParseFloatError) -> Self {
        ReadingError::BadValue(err)
    }
}
```

And for the third one — `UnknownUnit`, an error type we defined ourselves, not from the standard library — exactly the same shape: one more three-line `impl`. `From` doesn't care where the source error came from; it only needs to know one way to build a `ReadingError` out of it.

Now the body of the function is this — three bare `?`s, no `.map_err()` anywhere:

```rust
let id = id_str.parse::<u32>()?;
let value = value_str.parse::<f64>()?;
let unit = parse_unit(unit_str)?;

Ok((id, value, unit))
```

```sh
cargo run -p p1-06-05-from-and-error-conversion --example 03-error-conversion-with-from
```

```text
"12,36.6,C" -> Ok((12, 36.6, Celsius))
"x,36.6,C" -> Err(BadId(ParseIntError { kind: InvalidDigit }))
"12,hot,C" -> Err(BadValue(ParseFloatError { kind: Invalid }))
"12,36.6,K" -> Err(BadUnit(UnknownUnit("K")))
```

Same inputs, same answers. The only difference is that you wrote all three conversions once, at the top of the file — not every time a new function meets the same three errors. Write a fourth function that also sees a `ParseIntError`, and the `impl` you already wrote works for it too; nothing gets rewritten.

### You get `Into` for free

The standard library has a second trait called `Into`, but nowhere in this lesson have we written `impl Into` even once. The reason is a blanket impl the standard library writes once, for everyone:

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

Don't worry about the syntax — writing something shaped like that is [Phase 2](../../../phase2-intermediate/03-generics-and-traits/02-defining-and-implementing-traits/README.md)'s job. Just read what it says: for any types `T` and `U`, if `U: From<T>` exists, then `T: Into<U>` exists automatically too, and its `into` just calls `U::from(self)`. **Every `From` you write brings a free `Into` with it.** The reverse is technically true too, but pointless — because this blanket impl is already everywhere, nobody writes `impl Into` directly.

The result is `.into()` — the same conversion, read from the destination's side:

```rust
let box_weight = Kilograms(5.0);
let in_grams: Grams = box_weight.into();
```

```text
.into() (typed let): 5000
```

We never wrote `impl Into<Grams> for Kilograms` anywhere. We only wrote `impl From<Kilograms> for Grams`, and `.into()` came along with it.

### `.into()` — when type inference can work it out

`.into()` needs to know its target, and it gets that from **where the value is going to land** — not from `.into()` itself:

```rust
fn print_grams(amount: Grams) {
    println!("as grams:   {}", amount.0);
}

print_grams(Kilograms(2.0).into());

let in_pounds: Pounds = Kilograms(5.0).into();
```

```text
as grams:   2000
.into() (as Pounds): 11.0231
```

Same `Kilograms`, two different targets. The first time, `print_grams`'s signature says the target is `Grams`; the second time, the `let`'s written type says `Pounds`. The compiler reads the target from places like these: a `let`'s explicit type, the parameter type of a function the value is passed into, the return type of a function the value is returned from.

When none of those pin it down — when several `From<Kilograms>` impls exist and nothing says which one you mean — the compiler can't guess. You'll see that error in the next section.

### `From` is for non-error conversions too

`From` isn't only for errors. You've been calling this exact method since Phase 0, without its name:

```rust
let name = String::from("Matin");
let wide: u64 = u64::from(42_u32);
```

```text
String::from: Matin
u64::from:  42
```

`String::from("Matin")` is precisely `<String as From<&str>>::from` — converting a `&str` into a `String` that owns its own buffer. `u64::from(42_u32)` is an `impl From<u32> for u64` the standard library writes, because every `u32` fits inside a `u64`; the conversion can never lose data.

And that same rule explains why `i32::from(x: i64)` **does not exist**. An `i64` might be larger than anything that fits in an `i32`; `From` promises "this never fails," and this conversion can't keep that promise. You have two tools for this direction instead: `as` (which you saw in [1.1.2](../../01-foundations/02-scalar-types-and-overflow/README.md) — it truncates silently, no error), or the **`TryFrom`** family, which makes exactly that same promise but wrapped in a `Result`: "I'll convert you if you fit; if you don't, here's an `Err`." You'll see `TryFrom` in full in [Phase 2](../../../phase2-intermediate/08-rust-toolbox/04-tryfrom-fallible-conversions/README.md); for now, just know it's the fallible sibling of the `From` you learned today.

### The design rule for an error enum

The pattern you saw in `ReadingError` repeats everywhere:

```rust
enum AppError {
    Parse(ParseIntError),   // has a From impl — ? uses it
    Validation(MyOwnError), // this one too — same mechanism
    NotFound,                // built directly; nothing to convert from
}
```

The rule: **one variant per failure mode, and one `impl From` per foreign error that arrives through `?`.** Not every variant needs to arrive via `From` either — `NotFound` gets built directly wherever you detect it, because it's already your own error type.

One last thing to know: if two different calls return exactly the same source error type (say, parsing an hour *and* parsing a minute, both `ParseIntError`), you can only have one `impl From<ParseIntError>`, and it sends both to the same variant — `?` can't tell you which one it was. Where that distinction matters, `.map_err()` is still the right tool, because it lets you pick the variant at the call site. `From`/`?` is for when the source error is enough on its own; `.map_err()` is for when the call site needs to be part of the answer too.

---

## Hands on

```sh
cargo run -p p1-06-05-from-and-error-conversion --example 01-from-and-into-basics
cargo run -p p1-06-05-from-and-error-conversion --example 02-error-conversion-by-hand
cargo run -p p1-06-05-from-and-error-conversion --example 03-error-conversion-with-from
```

Then the two broken ones:

```sh
cargo run -p p1-06-05-from-and-error-conversion --example 04-ambiguous-into --features broken
cargo run -p p1-06-05-from-and-error-conversion --example 05-question-mark-without-from --features broken
```

Then try:

1. In `02-error-conversion-by-hand`, add a new input line without three fields (for example `"12,36.6"`). What `Err` do you get, and why?
2. In `03-error-conversion-with-from`, comment out the `impl From` blocks and see exactly which `?` the compiler objects to.
3. In `01-from-and-into-basics`, add a third target type (for example `Ounces`) with your own `impl From<Kilograms> for Ounces`, and write a new `.into()` for it with an explicit type.

---

## Errors you will meet

### `E0277` — `?` can't convert the error

This is the error the whole lesson is built around. `examples/05-question-mark-without-from.rs` has a function returning `Result<u32, ReadingError>` that uses `?` directly on `.parse::<u32>()`, which produces `ParseIntError` — with no `impl From<ParseIntError> for ReadingError` anywhere:

```text
error[E0277]: `?` couldn't convert the error to `ReadingError`
  --> examples\05-question-mark-without-from.rs:12:32
   |
 8 | fn parse_id(raw: &str) -> Result<u32, ReadingError> {
   |                           ------------------------- expected `ReadingError` because of this
...
12 |     let id = raw.parse::<u32>()?;
   |                  --------------^ the trait `From<ParseIntError>` is not implemented for `ReadingError`
   |                  |
   |                  this can't be annotated with `?` because it has type `Result<_, ParseIntError>`
   |
note: `ReadingError` needs to implement `From<ParseIntError>`
  --> examples\05-question-mark-without-from.rs:6:1
   |
 6 | struct ReadingError;
   | ^^^^^^^^^^^^^^^^^^^
   = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait
```

**What the compiler is objecting to:** the last line of the message says it outright — "the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait." That's the exact secret "The concept" unpacked, coming straight out of `rustc`'s own mouth. `?` saw that `raw.parse::<u32>()` is a `Result<u32, ParseIntError>`, tried to convert its error into `ReadingError` so it could return the `Err` from the function, and found no way to do it.

**The fix:** exactly the same three lines from "Writing `From<A> for MyError`":

```rust
impl From<ParseIntError> for ReadingError {
    fn from(err: ParseIntError) -> Self {
        ReadingError::BadId(err)
    }
}
```

**Why that's the fix:** the compiler's note says "`ReadingError` needs to implement `From<ParseIntError>`" — almost literally what to write. It's tempting to silence this with a one-off `.map_err(|_| ReadingError)`; that's fine if it's genuinely a one-off, but if this same function has three more calls producing the same error, you're writing exactly the repetition this lesson exists to remove.

### `E0282` — `.into()` doesn't know its target

```text
error[E0282]: type annotations needed
  --> examples\04-ambiguous-into.rs:24:9
   |
24 |     let result = Kilograms(5.0).into();
   |         ^^^^^^
25 |     println!("{}", result.0);
   |                    ------ type must be known at this point
   |
help: consider giving `result` an explicit type
   |
24 |     let result: /* Type */ = Kilograms(5.0).into();
   |               ++++++++++++
```

**What the compiler is objecting to:** `examples/04-ambiguous-into.rs` has both `impl From<Kilograms> for Grams` and `impl From<Kilograms> for Pounds`. When you write `Kilograms(5.0).into()` and drop it into `let result` with no explicit type, nothing says which one you mean. `result.0` doesn't help either, since both types have one `f64` field with that name.

**The fix:** the same two options from "`.into()` — when type inference can work it out" — put an explicit type on the `let` (`let result: Grams = ...`), or hand the value somewhere its type is already known (a function parameter, or a function's return value).

**Why that's the fix:** the compiler's message says "type annotations needed" — not "no conversion exists," just "I don't know which one." That's what separates this from the `E0277` above: there, no `From` existed at all; here, **too many** exist and the compiler can't choose between them.

---

## Exercises

### Warm up

<details>
<summary>What exactly does <code>expr?</code> return when <code>expr</code> is <code>Err(e)</code>?</summary>

`Err(From::from(e))`, not plain `Err(e)`. `?` calls `From::from` on `e` before returning, converting it to the current function's own error type.

</details>

<details>
<summary>If you write <code>impl From&lt;ParseIntError&gt; for AppError</code>, do you also need to write <code>impl Into&lt;AppError&gt; for ParseIntError</code> to make <code>.into()</code> work?</summary>

No. A blanket impl in the standard library turns every `From<T> for U` into an `Into<U> for T` automatically. Writing `impl Into` by hand is possible but never necessary.

</details>

<details>
<summary>Does this compile: <code>let x: i32 = i32::from(5_i64);</code></summary>

No, that's an `E0277`. There's no `impl From<i64> for i32`, because an `i64` can be larger than anything an `i32` can hold, and `From` promises "this never fails."

</details>

<details>
<summary>If both <code>Grams</code> and <code>Pounds</code> implement <code>From&lt;Kilograms&gt;</code>, does <code>let x = Kilograms(1.0).into();</code> (no explicit type anywhere) compile?</summary>

No, that's an `E0282` — "type annotations needed". Two possible targets exist and nothing says which one.

</details>

<details>
<summary>True or false: every variant of an error enum needs an <code>impl From</code>.</summary>

False. Only the ones that arrive by converting a foreign error need one. A variant you construct directly yourself (like a simple validation error) needs no `From` at all.

</details>

<details>
<summary>If two different calls in one function both return <code>ParseIntError</code>, can <code>?</code> tell you which one failed?</summary>

No, not with a single `impl From<ParseIntError>` — both route to the same variant. If that distinction matters, use `.map_err()` at each call site instead of a bare `?`.

</details>

### Repair

Fix `examples/04-ambiguous-into.rs` **two** ways:

1. By giving `let result` an explicit type (`Grams` or `Pounds`).
2. By writing the explicit `Grams::from(...)` or `Pounds::from(...)` form instead of `.into()`.

Then fix `examples/05-question-mark-without-from.rs` two ways: once by writing `impl From<ParseIntError> for ReadingError` so the bare `?` works, once with no `From` at all — using `.map_err(...)` that builds a `ReadingError` directly.

### Implement

Four things in `src/lib.rs` — two functions and two `impl From` blocks:

```sh
cargo test -p p1-06-05-from-and-error-conversion
```

`parse_color` and `impl From<ParseIntError> for ColorError` work together: one builds the conversion, the other uses a bare `?` to call it — exactly the pattern from "The concept". `all_fahrenheit` and `impl From<Celsius> for Fahrenheit` have the same relationship, this time for a conversion that can't fail.

Implement all four exactly to the doc comment above each one — the input format, the darkness rule, and the temperature formula are all stated precisely there.

### Build

Write an error enum and a parsing function for a domain of your choosing — a fraction (`"3/4"`), a date (`"2024-01-15"`), a coordinate (`"12.5,45.0"`). Have at least two different fallible calls, each with a different error type, each with its own `impl From`.

Then count: how many lines is your final version, using bare `?`? How many lines would the same function be if you wrote `.map_err()` at every call site instead?

### Challenge (optional)

**Part one.** Write a function `parse_time(s: &str) -> Result<u32, TimeError>` that turns a `"HH:MM"` string into a total minute count, with `TimeError` having a single variant — `BadNumber(ParseIntError)` — and one `impl From<ParseIntError>` that both the hour parse and the minute parse go through. Then try `parse_time("aa:30")` and `parse_time("12:bb")`. Both fail, both produce `TimeError::BadNumber(_)`. Can you tell from the `Err` alone which one it was — the hour or the minute? If that distinction mattered, exactly what would you have to change?

**Part two.** Suppose you want to write `Grams::try_from(user_input: f64)` that returns an `Err` for a negative value. Why can't this be expressed with `From`, but can with `TryFrom`? Write one sentence — you don't need to write the code; that's [Phase 2](../../../phase2-intermediate/08-rust-toolbox/04-tryfrom-fallible-conversions/README.md).

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `From<A> for B` | a way to convert `A` to `B` that never fails | write it once, use it everywhere |
| `Into<B> for A` | the free result of writing `From<A> for B` | `.into()` — never implemented by hand |
| `.into()` | the same conversion, read from the target's side | when the target is known from elsewhere |
| `?` and `From` | `?` calls `From::from` on the error, then returns | the secret |
| `.map_err()` | converting the error by hand, at the call site | when the call site itself needs to be in the answer |
| `E0277` (here) | `?` found no matching `From` | write the missing `impl From` |
| `E0282` | `.into()` is stuck between several targets | give it an explicit type |
| `TryFrom` | the fallible sibling of `From` | [Phase 2](../../../phase2-intermediate/08-rust-toolbox/04-tryfrom-fallible-conversions/README.md) |

### What you now know

- `expr?` on an `Err(e)` returns `Err(From::from(e))` — not plain `Err(e)`.
- Writing `impl From<A> for B` gets you `.into()` from `A` to `B` for free; nobody writes `impl Into` by hand.
- `.into()` gets its target from wherever the value lands; if nothing says where, that's `E0282`.
- `From` isn't only for errors — `String::from` and `u64::from` are the same trait.
- A narrowing conversion like `i32::from(x: i64)` doesn't exist because it might lose data; `as` or `TryFrom` fill that role instead.
- An error enum has one variant per failure mode, and one `impl From` per foreign error that arrives through `?` — not necessarily for every variant.

### What comes back later

- **`TryFrom` and `TryInto`, the fallible sibling of `From`** — [Phase 2 — `TryFrom` and fallible conversions](../../../phase2-intermediate/08-rust-toolbox/04-tryfrom-fallible-conversions/README.md)
- **Defining your own trait, and implementing it for your own type** — [Phase 2 — Defining and implementing traits](../../../phase2-intermediate/03-generics-and-traits/02-defining-and-implementing-traits/README.md)
- **`Display` and `std::error::Error`, for a properly real error type** — [Phase 2 — Custom error types](../../../phase2-intermediate/04-error-handling-and-lifetimes/01-custom-error-types/README.md)
- **`thiserror` and `anyhow`, for when you'd rather not write this boilerplate yourself** — [Phase 2 — `thiserror` and `anyhow`](../../../phase2-intermediate/04-error-handling-and-lifetimes/02-thiserror-and-anyhow/README.md)

### Can you explain?

- When `expr?` meets an `Err(e)`, what exactly does it return?
- Why is knowing how to write `impl From<A> for B` enough to make `.into()` work too?
- Where can the compiler read `.into()`'s target from? When can't it?
- Why doesn't `i32::from(x: i64)` exist? What do you use instead?
- Why doesn't every variant of an error enum need an `impl From`?
- If two calls return the same source error type, what's the limit of `?` and `From`, and what do you write instead?

---

## Going further

- [The Rust Book — A Shortcut for Propagating Errors: the `?` Operator](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator) — the same ground, officially.
- [`std::convert::From`](https://doc.rust-lang.org/std/convert/trait.From.html) — the trait's full documentation, including the list of conversions the standard library itself implements.
- [`std::convert::Into`](https://doc.rust-lang.org/std/convert/trait.Into.html) — where the blanket impl is defined.
- [`clippy::from_over_into`](https://rust-lang.github.io/rust-clippy/master/#from_over_into) — the lint that flags you for writing `impl Into` directly instead of `From`; now you know why.
