# 2.5.1 — Custom error types and `std::error::Error`

## At a glance

After this lesson you can:

- Design an error enum with one variant per distinct failure mode — not a single struct with a `message: String` — and say what that gives a caller that a `String` or a `Box<dyn Error>` doesn't.
- Implement `std::error::Error` for your own type, and state exactly which two traits it requires and which one method it provides itself.
- Pair a derived `Debug` with a hand-written `Display`, and write a `match` that responds differently depending on which variant failed.

**Time:** ~50 minutes · **Prerequisites:**
[2.3.4 — The standard derives, implemented by hand](../../03-traits-and-generics/04-standard-derives-by-hand/README.md) ·
[1.6.5 — `From` and error conversion](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.md)

---

## Why this matters

You've hit this exact wall twice since Phase 1, and both times the lesson took one step and stopped. In [1.6.3](../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.md) you started with `Result<T, String>` — the right call for a first meeting with `?`. The compiler there also suggested another option, `Result<(), Box<dyn std::error::Error>>`, and the lesson said the `Error` trait itself, and exactly how `Box<dyn Error>` works, was coming today. In [1.6.5](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.md) you went one step further: you wrote an error enum with several distinct variants (`ReadingError`), with an `impl From` that made `?` convert automatically. That enum had structure — but it still wasn't "a real Rust error," because neither `Display` nor `std::error::Error` was implemented on it; it only carried `#[derive(Debug)]`.

Today pays that off. First, see exactly why a `String` isn't enough on its own: a caller who receives a `String` can't do anything structured with it besides printing it or — worse — searching its text with `contains`, exactly the fragile string-sniffing Python does with `except Exception as e: if "not found" in str(e):`. A `Box<dyn Error>` is one step better — there's at least a real trait behind it — but it still erases the concrete type: without an extra downcast, the caller can't `match` on which variant of error actually happened. That downcasting, and walking a source chain, is entirely [2.5.2](../02-error-source-chains/README.md)'s subject; today stays with the simpler, more common case of a self-contained error — an enum that keeps its own structure with no box wrapped around it at all.

There's a second question too: why implement a standard trait called `Error` at all, when a plain enum with `#[derive(Debug)]` already prints just as well? Because "being an error" in Rust is an ecosystem-wide convention. [2.3.3](../../03-traits-and-generics/03-from-into-tryfrom/README.md) showed you an error type with nothing but a few derives on it — that was entirely enough for `TryFrom`, because that trait has no need for `std::error::Error`. What you get today isn't a compiler requirement; it's admission into a wider convention: libraries like `anyhow`, logging tools, and any function that asks for `Box<dyn Error>` all expect your error type to carry this trait.

---

## The concept

### Designing the error enum: one variant per failure mode

Say you're parsing a line of text like `"Frieren:96"` — a title and a score from 0 to 100. Three things can go wrong: the title can be empty, the score might not be a number at all, or the score might parse fine but be over 100. Instead of one struct with a single `message: String` field, you write an enum whose variants each stand for exactly one of these three:

```rust
#[derive(Debug, PartialEq)]
pub struct Review {
    pub title: String,
    pub score: u8,
}

#[derive(Debug)]
pub enum ReviewError {
    MissingTitle,
    InvalidScore(std::num::ParseIntError),
    ScoreOutOfRange(u8),
}

println!("{:?}", ReviewError::ScoreOutOfRange(150));
```

```text
ScoreOutOfRange(150)
```

`InvalidScore` keeps the underlying [`ParseIntError`](https://doc.rust-lang.org/std/num/struct.ParseIntError.html) itself — no information is thrown away. `ScoreOutOfRange` carries the actual value that got rejected, not just a message saying "a number was rejected." This is exactly what [1.6.5](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.md)'s rule already said — one variant per failure mode — today just takes one more step: making the enum itself "a real Rust error."

The parsing function, written the manual way for now (no `?` yet — the `From`-based version 1.6.5 taught you is a few sections away):

```sh
cargo run -p p2-05-01-custom-error-types --example 01-designing-the-error-enum
```

```text
"Frieren:96" -> Ok(Review { title: "Frieren", score: 96 })
":90" -> Err(MissingTitle)
"Bocchi:oops" -> Err(InvalidScore(ParseIntError { kind: InvalidDigit }))
"Bocchi:150" -> Err(ScoreOutOfRange(150))
```

Three failures, one enum. A caller can say exactly which one happened — not by searching a string, with an ordinary `match`.

### `Debug` for free, `Display` by hand

[2.3.4](../../03-traits-and-generics/04-standard-derives-by-hand/README.md) already taught you this split: `Debug` (`{:?}`) is mechanical and almost always derived — exactly what you saw above. `Display` (`{}`) is a human decision and is never derived. Nothing changes about that split here; the only difference is that the type getting a `Display` this time is an error:

```rust
impl std::fmt::Display for ReviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewError::MissingTitle => write!(f, "review is missing a title"),
            ReviewError::InvalidScore(source) => write!(f, "invalid score: {source}"),
            ReviewError::ScoreOutOfRange(score) => {
                write!(f, "score {score} is out of range (must be 0-100)")
            }
        }
    }
}
```

```sh
cargo run -p p2-05-01-custom-error-types --example 02-debug-and-display
```

```text
Display: review is missing a title
Debug:   MissingTitle
Display: invalid score: invalid digit found in string
Debug:   InvalidScore(ParseIntError { kind: InvalidDigit })
Display: score 150 is out of range (must be 0-100)
Debug:   ScoreOutOfRange(150)
```

The `InvalidScore` arm is worth a second look: `{source}` folds `ParseIntError`'s own `Display` text ("invalid digit found in string") straight into your own message. No information from the underlying failure is lost, not even at the text-message level.

### `std::error::Error`: two supertraits, and one method

You saw the name `std::error::Error` in [1.6.3](../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.md); now here's the trait itself. Its real declaration — straight from Rust's own standard library, minus a couple of deprecated methods nobody writes anymore — is this:

```rust
pub trait Error: Debug + Display {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
```

`Error: Debug + Display` means `Error` has two **supertraits** — exactly the term [2.3.6](../../03-traits-and-generics/06-supertraits-blanket-impls-orphan-rule/README.md) gave you: no type can implement `Error` unless it already has `Debug` and `Display`. `ReviewError` above already has both — one derived, one hand-written. So implementing `Error` becomes this:

```rust
impl std::error::Error for ReviewError {}
```

```sh
cargo run -p p2-05-01-custom-error-types --example 03-implementing-the-error-trait
```

```text
err.source(): None
```

A completely empty body. Not because you cut a corner — because everything it needed was already supplied elsewhere: the two supertraits above, and the one method `Error` actually adds, `source()`, which already has a default body that returns `None`. `source()` exists for chaining errors together — "what other error caused this one?" — and for a root-cause error like `ReviewError` (one that isn't itself caused by anything else), `None` is exactly the right answer. Actually chaining — overriding `source()` when an error wraps another one — is [2.5.2](../02-error-source-chains/README.md)'s subject.

### `From` does exactly what 1.6.5 taught

[1.6.5](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.md) showed you that `?` calls `From::from` on an `Err` before returning it. That same mechanism, with no changes at all, works here:

```rust
impl From<std::num::ParseIntError> for ReviewError {
    fn from(source: std::num::ParseIntError) -> Self {
        ReviewError::InvalidScore(source)
    }
}
```

With this `impl`, `parse_review` no longer needs the manual `match` from above; only one line changes — `let score: u8 = score_str.parse()?;`, a bare `?`, no `.map_err(...)` anywhere:

```sh
cargo run -p p2-05-01-custom-error-types --example 04-from-lets-question-mark-convert
```

```text
"Frieren:96" -> Ok(Review { title: "Frieren", score: 96 })
":90" -> Err(MissingTitle)
"Bocchi:oops" -> Err(InvalidScore(ParseIntError { kind: InvalidDigit }))
"Bocchi:150" -> Err(ScoreOutOfRange(150))
```

Same answers, same example as 01. Notice `MissingTitle` never comes through `From` — it's built directly, right where you detect it. 1.6.5 already said this: not every variant needs an `impl From`, only the ones that arrive by converting a foreign error.

### A structured response: `match`, then act differently

This is the real payoff of the whole lesson. A `String` or a `Box<dyn Error>` can only be printed. An enum can be `match`ed:

```rust
fn guidance(err: &ReviewError) -> &'static str {
    match err {
        ReviewError::MissingTitle => "ask them to add a title",
        ReviewError::InvalidScore(_) => "ask them to type digits only",
        ReviewError::ScoreOutOfRange(_) => "ask them for a score between 0 and 100",
    }
}
```

```sh
cargo run -p p2-05-01-custom-error-types --example 05-matching-to-respond-differently
```

```text
MissingTitle -> ask them to add a title
InvalidScore(ParseIntError { kind: InvalidDigit }) -> ask them to type digits only
ScoreOutOfRange(150) -> ask them for a score between 0 and 100
```

Three completely different responses, from one `match`. This is exactly what a `String` could never give you — not without forcing the caller to search its text with `contains`.

```senpai-visual
{"kind":"result","labels":["parse_review(line)","Err(MissingTitle)","Err(InvalidScore)","Err(ScoreOutOfRange)","a different response per kind"]}
```

---

## Hands on

```sh
cargo run -p p2-05-01-custom-error-types --example 01-designing-the-error-enum
cargo run -p p2-05-01-custom-error-types --example 02-debug-and-display
cargo run -p p2-05-01-custom-error-types --example 03-implementing-the-error-trait
cargo run -p p2-05-01-custom-error-types --example 04-from-lets-question-mark-convert
cargo run -p p2-05-01-custom-error-types --example 05-matching-to-respond-differently
```

Then the two broken ones:

```sh
cargo run -p p2-05-01-custom-error-types --example 06-error-needs-debug-and-display --features broken
cargo run -p p2-05-01-custom-error-types --example 07-source-needs-the-trait-in-scope --features broken
```

Then try:

1. In `01-designing-the-error-enum`, add a fourth input that uses `;` instead of `:` (for example `"Frieren;96"`). What `Err` do you get, and why is it exactly that one, not a new variant?
2. In `03-implementing-the-error-trait`, add a `println!("{}", err);` too. How is printing it different from printing `err.source()`?
3. In `05-matching-to-respond-differently`, add a fourth variant to `ReviewError` (for example `DuplicateTitle`) and see exactly where the compiler objects inside `guidance`.

---

## Errors you will meet

### `E0277` — `Error` without `Debug` and without `Display`

```text
error[E0277]: `ReviewError` doesn't implement `std::fmt::Display`
  --> phase2-intermediate\05-error-handling\01-custom-error-types\examples\06-error-needs-debug-and-display.rs:9:28
   |
 9 | impl std::error::Error for ReviewError {}
   |                            ^^^^^^^^^^^ unsatisfied trait bound
   |
help: the trait `std::fmt::Display` is not implemented for `ReviewError`
  --> phase2-intermediate\05-error-handling\01-custom-error-types\examples\06-error-needs-debug-and-display.rs:5:1
   |
 5 | pub struct ReviewError;
   | ^^^^^^^^^^^^^^^^^^^^^^
note: required by a bound in `std::error::Error`
  --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\error.rs:59:26
   |
59 | pub trait Error: Debug + Display {
   |                          ^^^^^^^ required by this bound in `Error`

error[E0277]: `ReviewError` doesn't implement `Debug`
  --> phase2-intermediate\05-error-handling\01-custom-error-types\examples\06-error-needs-debug-and-display.rs:9:28
   |
 9 | impl std::error::Error for ReviewError {}
   |                            ^^^^^^^^^^^ the trait `Debug` is not implemented for `ReviewError`
   |
   = note: add `#[derive(Debug)]` to `ReviewError` or manually `impl Debug for ReviewError`
note: required by a bound in `std::error::Error`
  --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\error.rs:59:18
   |
59 | pub trait Error: Debug + Display {
   |                  ^^^^^ required by this bound in `Error`
help: consider annotating `ReviewError` with `#[derive(Debug)]`
   |
 5 + #[derive(Debug)]
 6 | pub struct ReviewError;
   |

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p2-05-01-custom-error-types` (example "06-error-needs-debug-and-display") due to 2 previous errors
```

**What the compiler is objecting to:** `examples/06-error-needs-debug-and-display.rs` has a `ReviewError` with nothing on it — no `#[derive(Debug)]`, no `impl Display`. The line `impl std::error::Error for ReviewError {}` wants exactly the two supertraits "The concept" showed you, and the compiler reports both violations separately — one for `Display`, one for `Debug` — both pointing at the same `pub trait Error: Debug + Display` line inside the standard library itself.

**The fix:** actually supply both supertraits:

```rust
#[derive(Debug)]
pub struct ReviewError;

impl std::fmt::Display for ReviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "review is missing a title")
    }
}
```

**Why that's the fix:** the moment `Debug` and `Display` are both genuinely implemented, that same empty `impl std::error::Error for ReviewError {}`, with no further change, compiles — exactly what you saw with the three-variant `ReviewError` in "The concept."

### `E0599` — `source()` without the trait in scope

```text
error[E0599]: no method named `source` found for struct `ReviewError` in the current scope
   --> phase2-intermediate\05-error-handling\01-custom-error-types\examples\07-source-needs-the-trait-in-scope.rs:21:26
    |
  9 | pub struct ReviewError;
    | ---------------------- method `source` not found for this struct
...
 21 |     println!("{:?}", err.source());
    |                          ^^^^^^ method not found in `ReviewError`
    |
   ::: C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\error.rs:111:8
    |
111 |     fn source(&self) -> Option<&(dyn Error + 'static)> {
    |        ------ the method is available for `ReviewError` here
    |
    = help: items from traits can only be used if the trait is in scope
help: trait `Error` which provides `source` is implemented but not in scope; perhaps you want to import it
    |
  8 + use std::error::Error;
    |

For more information about this error, try `rustc --explain E0599`.
error: could not compile `p2-05-01-custom-error-types` (example "07-source-needs-the-trait-in-scope") due to 1 previous error
```

**What the compiler is objecting to:** `examples/07-source-needs-the-trait-in-scope.rs` genuinely has `impl std::error::Error for ReviewError {}` — `source()` really does exist on this type. The problem is elsewhere: nowhere in this file is there a `use std::error::Error;`. `source()` is a trait method, not an inherent method on the type, and calling a trait method needs the trait itself in scope — exactly what the compiler's own message says: "items from traits can only be used if the trait is in scope."

**The fix:** add the `use` line:

```rust
use std::error::Error;
```

**Why that's the fix:** with that `use`, `Error` is in scope, and `err.source()` finds exactly the method that was already implemented on `ReviewError` this whole time — the compiler just didn't know where to look for it until now. This trap isn't specific to `source()`: any method that comes from a trait, rather than from the type itself, follows the same rule.

---

## Exercises

### Warm up

<details>
<summary>An error enum has three variants: <code>MissingTitle</code>, <code>InvalidScore(ParseIntError)</code>, <code>ScoreOutOfRange(u8)</code>. You give it only <code>#[derive(Debug)]</code>, no <code>Display</code>. Does <code>impl std::error::Error for X {}</code> compile?</summary>

No. `Error: Debug + Display` needs both supertraits. `Debug` is there, `Display` isn't — that's `E0277`, exactly like the second half of example 06.

</details>

<details>
<summary>A type has both <code>Debug</code> and <code>Display</code>, and you've written <code>impl std::error::Error for X {}</code>. Do you also need to write <code>source()</code> for it to compile?</summary>

No. `source()` is a default method — its body already returns `None`. A completely empty body is enough for `impl Error`.

</details>

<details>
<summary>Why is <code>MissingTitle</code> never built through an <code>impl From</code>?</summary>

Because it doesn't come from a foreign error — you detect it directly inside `parse_review` (`title.is_empty()`) and build it directly. `From` is only needed for converting a different error.

</details>

<details>
<summary>A function bounds a type parameter with <code>E: std::error::Error</code>. Can you pass it a <code>String</code>?</summary>

No. The problem is that the standard library never implements `Error` for `String`. A type having `Debug`+`Display` doesn't automatically mean its `Error` is implemented too — `impl Error for X` always has to be written explicitly.

</details>

<details>
<summary>An error enum has <code>impl Error</code>, but you never wrote <code>use std::error::Error;</code> anywhere. Does <code>format!("{}", err)</code> work?</summary>

Yes. `{}` uses `Display`, and the format macros always know `Display`/`Debug` without needing them `use`d anywhere. The "trait not in scope" problem only shows up when you call a trait method with `.` — like `err.source()`.

</details>

### Repair

Fix both broken examples:

1. Fix `examples/06-error-needs-debug-and-display.rs` so it compiles — add `#[derive(Debug)]` and write a real `impl Display`, without touching `impl std::error::Error for ReviewError {}`.
2. Fix `examples/07-source-needs-the-trait-in-scope.rs` with a `use std::error::Error;`.

### Implement

Three things in `src/lib.rs` — a `Display`, an `impl From`, and the parsing function:

```sh
cargo test -p p2-05-01-custom-error-types
```

`impl std::error::Error for EntryError {}` is already there, completely empty — exactly what you saw in "The concept" once `Debug` (derived) and `Display` (what you write) both exist. Implement the other three exactly to the doc comment above each one — the `Display` text format, the order the checks run in, and the numeric cap are all stated precisely there.

### Build

Design an error enum for a domain of your choosing — a discount code, an email address, a phone number, whatever you like — with at least two distinct variants. Give it `#[derive(Debug)]`, a hand-written `Display`, and an empty `impl std::error::Error`. Then write a function that takes one of these variants and behaves genuinely differently depending on which one it is — not just a different message.

### Challenge (optional)

**Part one.** Take the enum from "Build." Add a new variant that itself carries a different error enum inside it (for example, a parse error from some sub-field). Keep your `impl Error` as the same empty body it's always been — meaning `source()` still has its default and still returns `None`, even though the new variant genuinely carries another error inside it. Is that a lie?

**Part two.** (This one looks ahead.) Open the documentation for [`std::error::Error::source`](https://doc.rust-lang.org/std/error/trait.Error.html#tymethod.source) and read its code example — two types named `SuperError` and `SuperErrorSideKick`. Guess exactly what `source()` returns on `SuperError`, then check your answer in [2.5.2](../02-error-source-chains/README.md).

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Error enum | One variant per distinct failure mode, not a `String` | Any function that can fail in more than one way |
| `std::error::Error` | The standard trait for "this is a real Rust error" | Any error type meant to compose with the rest of the ecosystem |
| Supertrait (here) | `Error: Debug + Display` — both must already exist | Why `impl Error for X {}` is sometimes this small |
| `source()` | `Error`'s only own method, defaulting to `None` | A root-cause error that isn't itself caused by anything else |
| Hand-written `Display` + derived `Debug` | The same split 2.3.4 taught, applied to an error | The user's message versus the programmer's dump |

### What you now know

- An error enum has one variant per distinct failure mode; a caller can `match` to say exactly which one happened, not just read it.
- `std::error::Error` needs two supertraits — `Debug` and `Display` — and adds exactly one method of its own, `source()`, defaulting to `None`.
- For a root-cause error, `impl std::error::Error for X {}` can be completely empty, as long as `Debug` and `Display` both already exist.
- A `String` has no structure to `match` on; a `Box<dyn Error>` has the same problem without a downcast.
- `From` does exactly what [1.6.5](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.md) taught — not every variant needs one, only the ones arriving from a foreign error.
- `source()` is a trait method; calling it with `.` needs `use std::error::Error;`, even when the `impl` already exists.

### What comes back later

- **Source chains: overriding `source()`, and `Box<dyn Error>` for holding mismatched error types together** — [2.5.2 — Error source chains](../02-error-source-chains/README.md)

### Can you explain?

- Why does an error enum give a caller more than a `Result<T, String>` does?
- Exactly which two traits does `std::error::Error` require, and why?
- Why can `impl std::error::Error for X {}` be completely empty?
- What does `source()` default to, and when is that default exactly right?
- Why doesn't `err.source()` compile without `use std::error::Error;`, while `format!("{}", err)` works fine without it?

---

## Going further

- [The Rust Book — Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html) — the same ground, officially.
- [`std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html) — the trait's full documentation, including the `source()` example the challenge points at.
- [`std::fmt::Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html) and [`std::fmt::Debug`](https://doc.rust-lang.org/std/fmt/trait.Debug.html) — documentation for those same two supertraits.
