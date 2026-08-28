# 2.5.4 — Designing an error taxonomy for a service

## At a glance

After this lesson you can:

- Build a nested error enum like `ServiceError` that keeps three real
  categories of failure — validation, not-found, internal — apart, using
  `thiserror` instead of a hand-written `Display` and `Error`.
- Keep an internal error's safe outward message (`Display`) separate from
  its real detail — reachable only through `source()` — without losing
  either one.
- Tell when a new category actually changes what a caller does, and when it
  only clutters the enum.

**Time:** ~50 minutes · **Prerequisites:**
[2.5.1 — Custom error types](../01-custom-error-types/README.md),
[2.5.2 — Source chains and `Box<dyn Error>`](../02-error-source-chains/README.md),
[2.5.3 — `thiserror` versus `anyhow`](../03-thiserror-and-anyhow/README.md)

---

## Why this matters

The last three lessons handed you three separate tools. 2.5.1 taught you to
stop returning a bare `String` and define an enum instead — one type per
real shape of failure, so a caller can `match` on it. 2.5.2 taught you not
to throw the original error away: chain back to the root cause through
`source()`, and box it in `Box<dyn Error>` when the exact cause isn't known
up front. 2.5.3 taught you not to hand-write that enum yourself —
`derive(thiserror::Error)` builds `Display` and `Error` for you — and to
reach for `anyhow` at the point a binary, not a library, is reporting an
error for good.

Three tools. None of them told you how to actually arrange them together
once a real piece of code — not a single-problem exercise — is in front of
you. Picture a small service: something that validates a caller's input,
looks something up by id, and occasionally touches the outside world (a
file, a network). Those are three genuinely different jobs, and they fail
in three genuinely different ways. Good design here isn't "one enum
variant per failure" or "one enum for everything" — it's a design
question: an **error taxonomy**, grouping the real failures under a small
number of categories that actually matter to a caller, instead of counting
them one by one.

Nothing new in the syntax below — everything you'll see is exactly what
2.5.1 through 2.5.3 already gave you. This lesson's job is to put them
together once, on one real problem, and that's what closes this module.

---

## The concept

### One flat error, three different reasons

Say a function can fail three different ways, and reports all of them
through one `Result<T, String>`:

```rust
fn add_entry_stringly(title: &str, rating: u8) -> Result<u64, String> {
    if title.trim().is_empty() {
        return Err("title must not be empty".to_string());
    }
    if rating > 10 {
        return Err(format!("rating {rating} is out of range 0..=10"));
    }
    Ok(0)
}
```

The only way a caller can react differently to each one is sniffing the
message text:

```rust
match add_entry_stringly("", 5) {
    Ok(id) => println!("added as {id}"),
    Err(msg) if msg.contains("empty") => {
        println!("caller reaction: ask again for a title ({msg})");
    }
    Err(msg) if msg.contains("out of range") => {
        println!("caller reaction: ask again for a rating ({msg})");
    }
    Err(msg) => println!("caller reaction: unknown failure: {msg}"),
}
```

```text
caller reaction: ask again for a title (title must not be empty)
```

That works today. But it's exactly the fragile string-sniffing 2.5.1
warned about — reword `"empty"` to `"blank"` tomorrow and this `match`
silently takes the wrong branch, with no compiler warning anywhere. An
enum turns that same mistake into a compile error instead — exactly what
`## Errors you will meet` shows you happening for real.

### Three natural categories: what the caller needs to react to differently

The organizing rule is not "give every distinct shape of failure its own
variant" — that's what 2.5.1 already taught you to do **inside** one
category. This lesson's question sits one level up: **how many genuinely
different reactions does a caller need?** For most services, the answer is
the same three:

- **Validation** — the caller's fault. Change the input, try again, the
  problem goes away.
- **Not found** — nobody's fault. A well-defined absence. The caller might
  create it, might pick something else — but that's a different reaction
  from validation.
- **Internal** — not the caller's fault at all. Nothing about their
  request fixes it. All they can do is see that it failed and maybe retry
  later.

```senpai-visual
{"kind":"concept","labels":["a failure happens","caller's fault: Validation","nothing there: NotFound","our fault: Internal","later: an HTTP status"]}
```

This three-way split is common enough that (later, once you build an HTTP
API — not today) it lands directly on three families of status code:
something like 400 for validation, 404 for not found, 500 for internal.
That mapping isn't this lesson's job. Today's job is getting the taxonomy
right on the Rust side.

### The nested shape: a `ServiceError` built with `thiserror`, not by hand

Each category earns a different shape, and `ServiceError` holds each one
differently:

```rust
#[derive(Debug, thiserror::Error)]
enum ValidationError {
    #[error("title must not be empty")]
    EmptyTitle,
    #[error("rating {rating} is out of range 0..=10")]
    RatingOutOfRange { rating: u8 },
}
```

```rust
#[derive(Debug, thiserror::Error)]
enum ServiceError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("no entry with id {id}")]
    NotFound { id: u64 },
    #[error("internal error")]
    Internal(#[from] std::io::Error),
}
```

Three variants, three different decisions:

- **`Validation`** wraps a **category-specific sub-error** — exactly what
  2.5.1 taught you: one variant per validation rule. `#[error(transparent)]`
  says "this variant's `Display` and `source()` are exactly
  `ValidationError`'s own, no extra wrapper text."
- **`NotFound`** carries plain data — an `id`. A category this small
  doesn't earn its own sub-error type.
- **`Internal`** wraps a real `std::io::Error`. `#[from]` does the same
  job you saw in 2.5.1: it lets `?` do the conversion on its own.

Run it:

```rust
let bad_title = ServiceError::from(ValidationError::EmptyTitle);
let bad_rating = ServiceError::from(ValidationError::RatingOutOfRange { rating: 15 });
let missing = ServiceError::NotFound { id: 7 };

println!("{bad_title}");
println!("{bad_rating}");
println!("{missing}");
```

```text
title must not be empty
rating 15 is out of range 0..=10
no entry with id 7
```

`ServiceError` itself knows nothing about *why* a validation failed —
that's `ValidationError`'s job. `ServiceError`'s only job is to say which
**category**. That split is exactly what keeps the outer enum small even
as the reasons inside one category grow.

### `?` doesn't guess a category — you decide it

```rust
fn validate(title: &str, rating: u8) -> Result<(), ValidationError> {
    if title.trim().is_empty() {
        return Err(ValidationError::EmptyTitle);
    }
    if rating > 10 {
        return Err(ValidationError::RatingOutOfRange { rating });
    }
    Ok(())
}

fn add_entry(title: &str, rating: u8) -> Result<(), ServiceError> {
    validate(title, rating)?;
    println!("stored: {title} ({rating}/10)");
    Ok(())
}
```

```rust
if let Err(err) = add_entry("Frieren", 10) {
    println!("unexpected: {err}");
}
if let Err(err) = add_entry("", 5) {
    println!("rejected: {err}");
}
if let Err(err) = add_entry("Bocchi", 15) {
    println!("rejected: {err}");
}
```

```text
stored: Frieren (10/10)
rejected: title must not be empty
rejected: rating 15 is out of range 0..=10
```

`validate` returns a `ValidationError`; `add_entry` needs a `ServiceError`.
`?` reconciles the two on its own only because `#[from]` on `Validation`
already wrote that conversion down. Without that wiring, `?` doesn't guess
anything — the compiler stops you and makes you pick a category on
purpose. `## Errors you will meet` shows you exactly that happening.

### Two audiences for one error: what you show, what you keep for yourself

This is the lesson's central point. `Internal`'s message is deliberately
generic:

```rust
#[derive(Debug, thiserror::Error)]
enum ServiceError {
    #[error("internal error")]
    Internal(#[from] std::io::Error),
}

fn restore_from_file(path: &str) -> Result<String, ServiceError> {
    Ok(std::fs::read_to_string(path)?)
}
```

```rust
use std::error::Error;

let err = restore_from_file("definitely/does/not/exist.txt").unwrap_err();
println!("shown to the caller: {err}");
println!("logged for debugging: {}", err.source().unwrap());
```

```text
shown to the caller: internal error
logged for debugging: The system cannot find the path specified. (os error 3)
```

`{err}` never shows the real `io::Error` text — only `"internal error"`.
That's deliberate: an external caller (a user, another team's service, a
public API response) has no business seeing your file path or
infrastructure details — they're neither safe nor useful to hand over. But
the detail isn't thrown away — the tool 2.5.2 gave you, `.source()`, is
still the way back to the root cause, for whoever is reading the logs.

```senpai-visual
{"kind":"result","labels":["io::Error (root cause)","ServiceError::Internal","Display: internal error","source(): the io::Error"]}
```

Compare that with `Validation`, where `#[error(transparent)]` deliberately
shows *all* of the detail — because that detail, which field, what value,
is exactly what the caller needs to fix their input. There's no one rule
that fits every category — "always hide" and "always show" are both
wrong. The decision is made per category: who sees this message, and is
the detail safe and useful to them or not.

### Where taxonomy granularity stops paying for itself

You could go further: split `Internal` into `DiskError`,
`PermissionError`, `CorruptStateError`, and more. Why didn't we? Because no
caller — and nowhere else in this code — is ever going to react
differently depending on *which* internal thing happened. They all get the
same reaction: log it, maybe alert, maybe retry later. More variants there
would only mean more `match` arms that all do exactly the same thing.

The question to ask every time is: **is somewhere — a caller's code, or my
own — actually going to treat this one differently from its siblings in
the same category?** If not, it doesn't need its own variant — but its
detail shouldn't disappear either; it just doesn't need a name in the type
itself, exactly as `Internal` showed you.

The rule cuts the other way too: `ValidationError`'s two variants —
`EmptyTitle` and `RatingOutOfRange` — earned their separateness, because
upstream code (or a UI) genuinely wants to highlight a different field
depending on which one fired. And if `Internal` ever really needed to
represent more than one unrelated underlying type — not just `io::Error`
— the tool 2.5.2 gave you, boxing an error behind a `dyn Error`, is
exactly for that moment; not something to reach for ahead of an actual
need.

### A `ServiceError` in the binary caller's hands

2.5.3 gave you the rule: `thiserror` inside the library, `anyhow` at the
library/binary boundary. `ServiceError` doesn't change at all — it's the
same type you just built — but code calling it can put a human sentence on
top with `anyhow::Context`:

```rust
use anyhow::Context;

fn add_entry(title: &str) -> Result<(), ServiceError> {
    if title.trim().is_empty() {
        return Err(ValidationError::EmptyTitle.into());
    }
    Ok(())
}

fn seed_startup_data() -> anyhow::Result<()> {
    add_entry("").context("failed to seed the watchlist on startup")?;
    Ok(())
}
```

```rust
if let Err(err) = seed_startup_data() {
    println!("{err:?}");
}
```

```text
failed to seed the watchlist on startup

Caused by:
    title must not be empty
```

The same `ServiceError`, unmodified, answers to both audiences: code
inside the library that wants to `match` on categories, and a binary whose
`main` just wants to say *what* failed and *why*. The categories survive
for the first; the chain survives for the second.

---

## Hands on

```sh
cargo run -p p2-05-04-error-taxonomy-for-a-service --example 01-flat-error-cant-be-matched
cargo run -p p2-05-04-error-taxonomy-for-a-service --example 02-service-error-shape
cargo run -p p2-05-04-error-taxonomy-for-a-service --example 03-validation-flows-through-question-mark
cargo run -p p2-05-04-error-taxonomy-for-a-service --example 04-internal-error-display-vs-source
cargo run -p p2-05-04-error-taxonomy-for-a-service --example 05-caller-side-anyhow-context
```

Then the two broken ones:

```sh
cargo run -p p2-05-04-error-taxonomy-for-a-service --example 06-non-exhaustive-after-new-category --features broken
cargo run -p p2-05-04-error-taxonomy-for-a-service --example 07-question-mark-needs-a-category --features broken
```

Then try these:

1. In `02-service-error-shape.rs`, build a third `RatingOutOfRange` with a
   different number and print it — confirm that number shows up in the
   message.
2. In `04-internal-error-display-vs-source.rs`, point at a directory
   instead of a missing path (e.g. `"src"`). The real `io::Error` text
   coming out of `source()` changes — does the "shown to the caller" line
   change too? Why or why not?
3. In `05-caller-side-anyhow-context.rs`, print `{err:#}` instead of
   `{err:?}` — compare it with what you saw above from `{err}` and
   `{err:?}`.

---

## Errors you will meet

### `E0004` — a `match` that missed the new category

```text
error[E0004]: non-exhaustive patterns: `&ServiceError::Internal(_)` not covered
  --> phase2-intermediate\05-error-handling\04-error-taxonomy-for-a-service\examples\06-non-exhaustive-after-new-category.rs:26:11
   |
26 |     match err {
   |           ^^^ pattern `&ServiceError::Internal(_)` not covered
   |
note: `ServiceError` defined here
  --> phase2-intermediate\05-error-handling\04-error-taxonomy-for-a-service\examples\06-non-exhaustive-after-new-category.rs:16:6
   |
16 | enum ServiceError {
   |      ^^^^^^^^^^^^
...
22 |     Internal(#[from] std::io::Error),
   |     -------- not covered
   = note: the matched value is of type `&ServiceError`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
28 ~         ServiceError::NotFound { .. } => "not found",
29 ~         &ServiceError::Internal(_) => todo!(),
   |

For more information about this error, try `rustc --explain E0004`.
error: could not compile `p2-05-04-error-taxonomy-for-a-service` (example "06-non-exhaustive-after-new-category") due to 1 previous error
```

**What the compiler is objecting to:** `describe` was written back when
`ServiceError` had only two categories. `Internal` was added to the
taxonomy later, and this `match` was never updated. This is exactly what a
taxonomy-as-enum buys you: adding a category turns every place that forgot
to update into a compile error, not a silent gap that only shows itself
the moment a real `Internal` shows up.

**The fix:** add a real arm for `Internal` too:

```rust
fn describe(err: &ServiceError) -> &'static str {
    match err {
        ServiceError::Validation(_) => "bad input",
        ServiceError::NotFound { .. } => "not found",
        ServiceError::Internal(_) => "internal error",
    }
}
```

**Why this is the fix:** the compiler's own suggestion (a `todo!()` arm)
gets you compiling, but it's telling you that you still owe this category
a real decision — exactly what a taxonomy is supposed to force. A
catch-all `_ => "unknown"` also compiles, but reopens the exact hole this
lesson is about: a new category, added later, quietly disappears into a
generic branch with nothing telling you it happened.

### `E0277` — `?` can't guess a category that doesn't exist yet

```text
error[E0277]: `?` couldn't convert the error to `ServiceError`
  --> phase2-intermediate\05-error-handling\04-error-taxonomy-for-a-service\examples\07-question-mark-needs-a-category.rs:25:49
   |
24 | fn restore_from_file(path: &str) -> Result<String, ServiceError> {
   |                                     ---------------------------- expected `ServiceError` because of this
25 |     let contents = std::fs::read_to_string(path)?;
   |                    -----------------------------^ the trait `From<std::io::Error>` is not implemented for `ServiceError`
   |                    |
   |                    this can't be annotated with `?` because it has type `Result<_, std::io::Error>`
   |
note: `ServiceError` needs to implement `From<std::io::Error>`
  --> phase2-intermediate\05-error-handling\04-error-taxonomy-for-a-service\examples\07-question-mark-needs-a-category.rs:17:1
   |
17 | enum ServiceError {
   | ^^^^^^^^^^^^^^^^^
   = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait
help: the trait `From<std::io::Error>` is not implemented for `ServiceError`
      but trait `From<ValidationError>` is implemented for it
  --> phase2-intermediate\05-error-handling\04-error-taxonomy-for-a-service\examples\07-question-mark-needs-a-category.rs:19:18
   |
19 |     Validation(#[from] ValidationError),
   |                  ^^^^
   = help: for that trait implementation, expected `ValidationError`, found `std::io::Error`

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p2-05-04-error-taxonomy-for-a-service` (example "07-question-mark-needs-a-category") due to 1 previous error
```

**What the compiler is objecting to:** this draft of `ServiceError` doesn't
have an `Internal` category yet — only `Validation` and `NotFound`. `?` on
an `io::Error` needs to convert it to `ServiceError`, but there's no
`From<std::io::Error>` for the compiler to call. The message even names
what *does* exist: `From<ValidationError>` — one category already wired
up, this one not.

**The fix:** add the third category this draft is missing:

```rust
#[error("internal error")]
Internal(#[from] std::io::Error),
```

**Why this is the fix:** `?` never invents a category — it only calls a
`From` that has already been written. That limit isn't a flaw; it's
exactly the decision point this lesson is about: whenever code is about to
raise a new kind of failure, a human decides, once, which category it
belongs to — the compiler doesn't guess that for you, it only makes sure
you didn't forget to.

---

## Exercises

### Warm up

<details>
<summary>What does this print?</summary>

```rust
let err = ServiceError::NotFound { id: 42 };
println!("{err}");
```

</details>

<details>
<summary>Answer</summary>

```text
no entry with id 42
```

`#[error("no entry with id {id}")]` puts the `id` field straight into the
message — `NotFound` carries plain data, not a sub-error.

</details>

<details>
<summary>What does this print?</summary>

```rust
let err = ServiceError::Internal(
    std::io::Error::new(std::io::ErrorKind::Other, "disk full"),
);
println!("{err}");
```

</details>

<details>
<summary>Answer</summary>

```text
internal error
```

Not `"disk full"`. `Internal`'s message is a string literal, not a format
that interpolates its field — it always gives that exact generic text, no
matter what's inside.

</details>

<details>
<summary>Does this compile?</summary>

```rust
fn describe(err: &ServiceError) -> &'static str {
    match err {
        ServiceError::Validation(_) => "bad input",
        ServiceError::NotFound { .. } => "not found",
    }
}
```

(`ServiceError` is the same three-category enum you built in `## The
concept`, `Internal` included.)

</details>

<details>
<summary>Answer</summary>

No. `E0004` — the `match` doesn't cover `Internal`.

</details>

<details>
<summary>Does this compile? (Assume this draft of <code>ServiceError</code> only has <code>Validation</code> and <code>NotFound</code> — no <code>Internal</code> yet)</summary>

```rust
fn restore_from_file(path: &str) -> Result<String, ServiceError> {
    let contents = std::fs::read_to_string(path)?;
    Ok(contents)
}
```

</details>

<details>
<summary>Answer</summary>

No. `E0277` — there is no `From<std::io::Error>` for this `ServiceError`,
so `?` cannot perform the conversion.

</details>

### Repair

Fix both broken examples:

1. Fix `examples/06-non-exhaustive-after-new-category.rs` so it compiles —
   by adding a real arm for `Internal`, not a `_` stand-in.
2. Fix `examples/07-question-mark-needs-a-category.rs` so it compiles — by
   adding the missing `Internal` category to `ServiceError`, wired with
   `#[from]`.

### Implement

Five functions/methods in `src/lib.rs`:

```sh
cargo test -p p2-05-04-error-taxonomy-for-a-service
```

`ValidationError` and `ServiceError` are already fully written — their
shape is exactly what `## The concept` walked through. What's left is the
logic that produces and consumes them: `validate`, `WatchlistStore::new`,
`add_entry`, `rating_of`, `restore_from_file`. Each one's exact
specification — including where ids start counting and what happens to a
malformed backup line — is in the doc comment above it.

### Build

Add a new method to `WatchlistStore`:

```rust
pub fn rename(&mut self, id: u64, new_title: &str) -> Result<(), ServiceError>
```

This can fail two different ways. Neither one needs a fourth category —
your job is to work out which of the three existing categories each one
belongs to, and write the signature with that reasoning.

### Challenge (optional)

This one looks ahead — to Phase 3, where you'll build a real HTTP API. In
a comment, write down which HTTP status code (400, 404, or 500) fits each
of `ServiceError`'s three categories, and one sentence for why. Then look
at
[3.7.1 — Consistent error envelopes](../../../phase3-backend-foundations/07-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)
and see whether it uses this exact same three-way split.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Error taxonomy | Grouping real failures under a small number of categories that matter to a caller | Designing any real error enum |
| Category | A group of failures the caller should react to the same way | The first decision in a taxonomy |
| `#[error(transparent)]` | Hands `Display` and `source()` straight through to the wrapped error | When a category says nothing beyond its sub-error |
| Safe `Display` vs. full `source()` | One message for the external caller, one for the logs | The `Internal` category |

### What you now know

- An error taxonomy groups failures by how many different ways a caller
  reacts, not by counting every way something can go wrong.
- Validation, not found, and internal are a natural three-way split that
  recurs across most services.
- An outer enum can wrap a sub-error (`Validation`), carry plain data
  (`NotFound`), or wrap an outside type like `io::Error` (`Internal`) —
  all three built with `thiserror`, not by hand.
- `Display` and `source()` have two separate audiences; you can keep one
  deliberately generic and the other fully specific, category by category.
- More variants is not automatically better design. The real question is
  whether anywhere actually treats that variant differently.
- The same `ServiceError`, unchanged, answers both a caller matching on
  categories and a binary putting a human sentence on top with
  `anyhow::Context`.

### What comes back later

- **Mapping these three categories onto HTTP status codes, behind one
  shared `impl IntoResponse`** —
  [Phase 3 — Consistent error envelopes](../../../phase3-backend-foundations/07-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)

### Can you explain?

- Why is a taxonomy organized around "how does the caller react", not
  "how many ways can this fail"?
- Why does `Internal` have a generic message while `Validation` doesn't?
  What would go wrong if you swapped them?
- What does `.source()` give you that `Display` didn't already say?
- What test do you run to decide whether a new category is actually needed
  or just clutters the enum?
- Why does adding a new category to `ServiceError` turn every incomplete
  `match` into a compile error instead of a silent bug?

---

## Going further

- [`thiserror` documentation](https://docs.rs/thiserror/latest/thiserror/)
  — including `#[error(transparent)]`, what `Validation` used above.
- [`anyhow` documentation](https://docs.rs/anyhow/latest/anyhow/) —
  `Context`, and exactly how `{:?}` differs from `{}`.
- [`std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html)
  — the official definition of `source()`.
- [The Rust API Guidelines — error types are meaningful and well-behaved](https://rust-lang.github.io/api-guidelines/interoperability.html#error-types-are-meaningful-and-well-behaved-c-good-err)
  — the same principles, as a formal rule for any crate you publish.
