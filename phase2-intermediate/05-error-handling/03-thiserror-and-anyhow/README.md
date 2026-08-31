# 2.5.3 — `thiserror` versus `anyhow`, and the library/binary boundary

## At a glance

After this lesson you can:

- Explain exactly what `#[derive(thiserror::Error)]` generates, by setting it next to the `Display`/`Error` you hand-wrote in 2.5.1 and 2.5.2.
- Choose between `#[source]` and `#[from]` for a given variant, and say why `#[from]` refuses when the variant also needs an extra field, such as a line number.
- Choose between `thiserror` and `anyhow` for a given function, based on whether its caller genuinely needs to `match` on the kind of error — and attach a more readable message to an error with `anyhow::Context`, exactly as it propagates.

**Time:** ~75 minutes · **Prerequisites:**
[2.5.2 — Source chains and `Box<dyn Error>`](../02-error-source-chains/README.md), and specifically [2.5.1 — Custom error types](../01-custom-error-types/README.md) for the hand-written shape you see next to its macro version today

---

## Why this matters

2.5.1 had you build your own error `enum` and hand-write its `Display` — a `match` with one arm per variant — plus an `impl std::error::Error` (even an empty one) and an `impl From<...>` for the variant whose conversion was simple. 2.5.2 took it one step further: you overrode `source()` for real, so a caller could walk the error chain all the way down, and you saw `Box<dyn Error>` for when you just want to say "an error comes back here, it doesn't matter which exact type."

Now imagine doing this for the tenth time. The same four pieces — enum, `Display`'s `match`, `impl Error`, `impl From` — again, for a completely different error type. None of it is hard; the problem is that it's *mechanical*. You type the same shape every time, and every time there's a real chance you forget a `match` arm or phrase its message slightly differently. This is exactly the kind of work computers do better than people — and that is precisely what the `thiserror` crate generates: from a few attribute lines, it builds the same `Display`, `Error`, and (when you want it) `From` for you.

There's a second thread to this lesson too, from a completely different angle. 2.5.1 and 2.5.2 assumed your function's caller might want to `match` on the kind of error — that's why you built an `enum` with separate variants instead of a `String`. But that isn't always true. The `main` function, the top-level logic of a CLI tool, an HTTP handler that just wants to return a 500 and log the details — these usually have no caller at all who wants to branch on the kind of error. For them, building a precise `enum` is a cost with no payoff. The `anyhow` crate exists for exactly this spot.

Today you see both, and a rule that says where each one belongs.

---

## The concept

### A recap: the same shape, one more time, just to remember it

Before looking at the macro, let's build the same shape by hand once more — this time for a file of anime ratings, one `title,score` per line. The same three pieces you saw in 2.5.1, plus a real `source()` like 2.5.2:

```rust
#[derive(Debug)]
pub enum RatingsError {
    Io(std::io::Error),
    MissingScore {
        line: usize,
    },
    InvalidScore {
        line: usize,
        source: std::num::ParseIntError,
    },
}
```

```rust
impl fmt::Display for RatingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RatingsError::Io(source) => write!(f, "could not read ratings file: {source}"),
            RatingsError::MissingScore { line } => write!(f, "line {line}: missing score"),
            RatingsError::InvalidScore { line, source } => {
                write!(f, "line {line}: invalid score: {source}")
            }
        }
    }
}
```

```rust
impl std::error::Error for RatingsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RatingsError::Io(source) => Some(source),
            RatingsError::MissingScore { .. } => None,
            RatingsError::InvalidScore { source, .. } => Some(source),
        }
    }
}
```

```rust
impl From<std::io::Error> for RatingsError {
    fn from(source: std::io::Error) -> Self {
        RatingsError::Io(source)
    }
}
```

None of these four pieces is unfamiliar — this is exactly what 2.5.1 and 2.5.2 taught you. Run it and see:

```sh
cargo run -p p2-05-03-thiserror-and-anyhow --example 01-hand-written-error
```

```text
line 2: missing score
line 5: invalid score: invalid digit found in string
  caused by: invalid digit found in string
could not read ratings file: The system cannot find the file specified. (os error 2)
  caused by: The system cannot find the file specified. (os error 2)
```

### The same thing, behind `#[derive(thiserror::Error)]`

Now the same `enum`, the same messages, the same `source()`, the same `From` — but this time all of it comes from a handful of attributes:

```rust
#[derive(Debug, thiserror::Error)]
pub enum RatingsError {
    #[error("could not read ratings file: {0}")]
    Io(#[from] std::io::Error),

    #[error("line {line}: missing score")]
    MissingScore { line: usize },

    #[error("line {line}: invalid score: {source}")]
    InvalidScore {
        line: usize,
        #[source]
        source: std::num::ParseIntError,
    },
}
```

Three things are happening here:

- `#[error("...")]` on each variant is that variant's `Display` message — the interpolation syntax is exactly `format!`'s. `{0}` refers to a tuple variant's first (and here only) field; `{line}` and `{source}` refer to a struct variant's named fields.
- `#[source]` on a field marks it as what `Error::source()` should return — exactly the `match` you hand-wrote above.
- `#[from]` on the `Io` field both counts it as `#[source]` *and* generates `impl From<std::io::Error> for RatingsError` — the fourth block above, for free.

Run it and compare:

```sh
cargo run -p p2-05-03-thiserror-and-anyhow --example 02-thiserror-derive
```

```text
line 2: missing score
line 5: invalid score: invalid digit found in string
  caused by: invalid digit found in string
could not read ratings file: The system cannot find the file specified. (os error 2)
  caused by: The system cannot find the file specified. (os error 2)
```

Character for character, the same thing you got from the hand-written version above. `thiserror` hasn't changed what an error *is* — it's still the same `Display` + `Error`, the same two traits you met in 2.5.1 — it just means you no longer have to type them yourself.

### `#[source]` without `#[from]` — where `?` isn't enough

`#[from]` only works on a field that is the *entire* variant, because `From::from` only receives that one value and has to build the whole variant out of it. The `Io` variant above has exactly that shape; `InvalidScore` doesn't — it also needs a `line` that no `ParseIntError` could ever supply. The example below is the same `RatingsError`, this time with only these two variants — `MissingScore` isn't needed here:

```rust
fn read_len(path: &str) -> Result<usize, RatingsError> {
    let text = std::fs::read_to_string(path)?; // bare `?` — `#[from]` covers it
    Ok(text.len())
}

fn parse_score(line: usize, raw: &str) -> Result<u8, RatingsError> {
    raw.trim()
        .parse()
        .map_err(|source| RatingsError::InvalidScore { line, source })
}
```

```sh
cargo run -p p2-05-03-thiserror-and-anyhow --example 03-from-attribute
```

```text
read_len error: could not read ratings file: The system cannot find the file specified. (os error 2)
parse_score error: line 3: invalid score: invalid digit found in string
```

`read_len` gets away with a plain `?` because `#[from]` already generated the conversion it needs. `parse_score` can't — `line` has to come from somewhere only the function itself knows, not from `From::from`. This is the exact same boundary 2.5.1 and 2.5.2 each showed you by hand: a conversion can only be automatic when it needs nothing extra — no field name, no line number. `#[from]` is just the macro-powered version of that same boundary.

### `anyhow::Error` — one dynamic error type, for anything that's an `Error`

2.5.2 gave you `Box<dyn Error>`: a way to say "an error comes back here, it doesn't matter exactly which type" without writing out the concrete type either. `anyhow::Error` takes the same idea and adds two things you'll see below — a place to attach a message as the error propagates, and a way back to the concrete type if you ever need it.

```rust
fn parse_score(raw: &str) -> Result<u8, std::num::ParseIntError> {
    raw.trim().parse()
}

fn run() -> anyhow::Result<()> {
    let score = parse_score("87")?;
    println!("parsed score: {score}");
    let score = parse_score("oops")?; // ParseIntError -> anyhow::Error, no From needed
    println!("parsed score: {score}");
    Ok(())
}
```

`anyhow::Result<T>` is just shorthand for `Result<T, anyhow::Error>`. The important part here: `parse_score` doesn't return `RatingsError` at all — it's a bare standard-library `std::num::ParseIntError`, a type this crate neither owns nor wrote a `From` for. And yet `?` inside a function returning `anyhow::Result<T>` converts it without any extra code, because any type implementing `std::error::Error` is accepted just like that:

```sh
cargo run -p p2-05-03-thiserror-and-anyhow --example 04-anyhow-basics
```

```text
parsed score: 87
Display : invalid digit found in string
Debug   : invalid digit found in string
```

Here `{}` and `{:?}` show the same thing, because there's still only one layer of error — no `.context(...)` has touched it yet. That's exactly what the next subsection changes.

### `anyhow::Context` — attaching a message right as the error propagates

```rust
use anyhow::Context;

fn load_score(raw: &str) -> anyhow::Result<u8> {
    parse_score(raw).context("failed to load score from config")
}
```

`.context("...")` puts a more human-readable message on the error — without discarding the original. `{}` shows only that new message; `{:?}` shows the whole chain:

```sh
cargo run -p p2-05-03-thiserror-and-anyhow --example 05-anyhow-context
```

```text
Display : failed to load score from config
Debug   : failed to load score from config

Caused by:
    invalid digit found in string
```

Compare this to the previous subsection's output: the same `ParseIntError`, but this time `{:?}` also has a "`Caused by:`" section, because now there really is a chain — the new message on top, the original error underneath. `.with_context(|| ...)` is the lazy version of the same thing: when building the message itself costs something (say, you have to `format!` a value), you only pay that cost when the error path is actually taken.

```senpai-visual
{"kind":"result","labels":["parse_score(raw)?","Err: invalid digit","context: failed to load score","Debug: message + Caused by chain"]}
```

### The library/binary boundary

Now put both pieces side by side. A library-shaped function — one that *might* have a caller who wants to branch on the kind of error — returns the precise `RatingsError`. A function with no caller of its own — because it's the end of the line — collapses everything into `anyhow::Result`. This time `RatingsError` only has two variants, `MissingScore` and `InvalidScore` — the only two this example actually needs:

```rust
fn parse_line(line: usize, text: &str) -> Result<(String, u8), RatingsError> {
    let (title, score) = text
        .split_once(',')
        .ok_or(RatingsError::MissingScore { line })?;
    let score = score
        .trim()
        .parse()
        .map_err(|source| RatingsError::InvalidScore { line, source })?;
    Ok((title.to_string(), score))
}
```

And its binary side, built on that same function:

```rust
fn load_ratings(input: &str) -> anyhow::Result<Vec<(String, u8)>> {
    input
        .lines()
        .enumerate()
        .map(|(index, line)| parse_line(index + 1, line))
        .collect::<Result<Vec<_>, RatingsError>>()
        .context("failed to load ratings")
}
```

`load_ratings` uses exactly the short-circuiting trick you saw in 2.2.3 with `.collect::<Result<Vec<_>, _>>()`: the first `Err` becomes the whole result, and nothing after it is even touched. Run it:

```sh
cargo run -p p2-05-03-thiserror-and-anyhow --example 06-library-vs-binary-boundary
```

```text
Ok([("Frieren", 96), ("Bocchi the Rock!", 90)])
Display : failed to load ratings
Debug   : failed to load ratings

Caused by:
    0: line 2: invalid score: invalid digit found in string
    1: invalid digit found in string
```

This time the chain has two layers under the context message, not one: `RatingsError::InvalidScore`'s own message (layer zero), then the `ParseIntError` it was already holding as its `source()` (layer one). `anyhow` finds this chain on its own, through `Error::source()` — the same method you learned in 2.5.2 and overrode again above.

**The rule:** a library exposes a precise, matchable error type, because it doesn't know whether its caller needs to branch on it. A binary — the outermost layer, actually running, with no caller of its own — can reach for `anyhow` instead, because there's no one downstream left who wants to `match` on the kind of error.

```senpai-visual
{"kind":"concept","labels":["fallible fn in a library","caller may need to match?","yes: thiserror enum","no further caller: main()","binary: anyhow::Error"]}
```

### When you break this rule

This isn't an absolute rule, it's a good default. Where it genuinely breaks is a "binary" that is, in effect, also a library. A CLI tool whose `main.rs` is thin but whose real logic lives in a `lib.rs` that other crates also `use`; or a service whose errors eventually become an HTTP body, and the other end of that line — not Rust code, but the API's client — genuinely needs to tell a "404" from a "500" apart. Here the right question is no longer "is this a binary or a library?" but "will *something*, whatever it is, later want to branch on this error?" — and if the answer is yes, choose `thiserror` even inside a `main.rs`. 2.5.4 continues exactly here: designing an error taxonomy for a real service, where this decision is no longer made once and for all.

---

## Hands on

```sh
cargo run -p p2-05-03-thiserror-and-anyhow --example 01-hand-written-error
cargo run -p p2-05-03-thiserror-and-anyhow --example 02-thiserror-derive
cargo run -p p2-05-03-thiserror-and-anyhow --example 03-from-attribute
cargo run -p p2-05-03-thiserror-and-anyhow --example 04-anyhow-basics
cargo run -p p2-05-03-thiserror-and-anyhow --example 05-anyhow-context
cargo run -p p2-05-03-thiserror-and-anyhow --example 06-library-vs-binary-boundary
```

Then the three broken ones:

```sh
cargo run -p p2-05-03-thiserror-and-anyhow --example 07-derive-error-needs-debug --features broken
cargo run -p p2-05-03-thiserror-and-anyhow --example 08-context-needs-trait-import --features broken
cargo run -p p2-05-03-thiserror-and-anyhow --example 09-from-needs-single-field --features broken
```

Then try these:

1. In `01-hand-written-error.rs` and `02-thiserror-derive.rs`, add a new variant (say `EmptyFile`) — once in the hand-written version, once in the `derive` version. How many lines did each one cost you?
2. In `03-from-attribute.rs`, remove `#[from]` from `Io` and try to keep `read_len`'s bare `?`. What error do you get, and why is it exactly that one?
3. In `06-library-vs-binary-boundary.rs`, change the `bad` input so the *first* line is the broken one instead of the second. Does the line number in the message find itself correctly?

---

## Errors you will meet

### `E0277` — `#[derive(thiserror::Error)]` without `#[derive(Debug)]`

```text
error[E0277]: `RatingsError` doesn't implement `Debug`
  --> phase2-intermediate\05-error-handling\03-thiserror-and-anyhow\examples\07-derive-error-needs-debug.rs:11:10
   |
10 | #[derive(thiserror::Error)]
   |          ---------------- in this derive macro expansion
11 | pub enum RatingsError {
   |          ^^^^^^^^^^^^ the trait `Debug` is not implemented for `RatingsError`
   |
   = note: add `#[derive(Debug)]` to `RatingsError` or manually `impl Debug for RatingsError`
note: required by a bound in `std::error::Error`
  --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\error.rs:59:18
   |
59 | pub trait Error: Debug + Display {
   |                  ^^^^^ required by this bound in `Error`
   = note: this error originates in the derive macro `thiserror::Error` (in Nightly builds, run with -Z macro-backtrace for more info)
help: consider annotating `RatingsError` with `#[derive(Debug)]`
   |
11 + #[derive(Debug)]
12 | pub enum RatingsError {
   |

For more information about this error, try `rustc --explain E0277`.
```

**What the compiler is actually complaining about:** `#[derive(thiserror::Error)]` is building `impl std::error::Error for RatingsError`, but the trait itself declares `Error: Debug + Display` — anything that is `Error` has to already be `Debug`. 2.5.1 showed you the same fact: `Error` needs both of its supertraits, and writing one never gives you the other for free; `thiserror` hasn't changed either side of that rule — it only builds `Display` for you, not `Debug`.

**The fix:** exactly what the compiler suggests:

```rust
#[derive(Debug, thiserror::Error)]
pub enum RatingsError {
    #[error("line {0}: missing score")]
    MissingScore(usize),
}
```

**Why this is the fix:** `#[derive(Debug)]` is a mechanical, free implementation — exactly what you saw in 2.3.4. Now both halves of `Error`'s bound are satisfied, and `thiserror` can finish `impl Error` without complaint.

### `E0599` — `.context(...)` without `use anyhow::Context;`

```text
error[E0599]: no method named `context` found for enum `Result<T, E>` in the current scope
   --> phase2-intermediate\05-error-handling\03-thiserror-and-anyhow\examples\08-context-needs-trait-import.rs:15:22
    |
 15 |     parse_score(raw).context("failed to load score")
    |                      ^^^^^^^
    |
   ::: C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\anyhow-1.0.103\src\lib.rs:618:8
    |
618 |     fn context<C>(self, context: C) -> Result<T, Error>
    |        ------- the method is available for `Result<u8, ParseIntError>` here
    |
    = help: items from traits can only be used if the trait is in scope
help: trait `Context` which provides `context` is implemented but not in scope; perhaps you want to import it
    |
 10 + use anyhow::Context;
    |
help: there is a method `with_context` with a similar name
    |
 15 |     parse_score(raw).with_context("failed to load score")
    |                      +++++

For more information about this error, try `rustc --explain E0599`.
```

**What the compiler is actually complaining about:** `.context()` isn't defined on `Result` itself — it's a method added to `Result<T, E>` by the `anyhow::Context` trait, only when that trait is in scope. Without `use anyhow::Context;`, the method genuinely doesn't exist as far as the compiler can see.

**The fix:** add the import:

```rust
use anyhow::Context;
```

**Why this is the fix:** that's the compiler's first suggestion. Its second one — swap to `.with_context()` — doesn't actually help on its own: `with_context` lives on the exact same `Context` trait as `context`, so without the `use` it fails with the identical E0599, trait-not-in-scope error, all over again. (Once the trait genuinely is in scope, `.with_context()` also wants a closure — `F: FnOnce() -> C` — not a bare string, so `"failed to load score"` would then need to become `|| "failed to load score"` too — but that is a separate fact about the method's signature, not what breaks it here.) The real fix is always the first one: bring the right trait into scope.

### `thiserror`'s macro error — `#[from]` on a variant with an extra field

```text
error: deriving From requires no fields other than source and backtrace
  --> phase2-intermediate\05-error-handling\03-thiserror-and-anyhow\examples\09-from-needs-single-field.rs:16:9
   |
16 |         #[from]
   |         ^^^^^^^
```

**What the compiler is actually complaining about:** this one has no `E` code — it's `thiserror`'s own macro message, raised while expanding `#[derive]`, not by `rustc` itself. `#[from]` promises to build `impl From<ParseIntError> for RatingsError`; but that function only ever receives the one `ParseIntError` value and has to build the *entire* variant out of it. `InvalidScore` also needs a `line: usize` that no `ParseIntError` could ever supply — so this promise can't honestly be kept.

**The fix:** drop `#[from]`, keep `#[source]`, and write the conversion by hand exactly as "The concept" showed:

```rust
InvalidScore {
    line: usize,
    #[source]
    source: std::num::ParseIntError,
},
```

then at the call site:

```rust
.map_err(|source| RatingsError::InvalidScore { line, source })?
```

**Why this is the fix:** `#[source]` still wires up `Error::source()` for you — you still have the chain. You just can't use a bare, automatic `?` anymore, because no automatic conversion *can* exist when an extra field is required. This is exactly the boundary "`#[source]` without `#[from]`" showed you.

---

## Exercises

### Warm up

<details>
<summary>For <code>#[error("missing required field: {0}")] MissingField(String)</code>, what exact string does <code>MissingField("name".to_string()).to_string()</code> return?</summary>

Write down your answer before looking.

</details>

<details>
<summary>Answer</summary>

```text
missing required field: name
```

`{0}` refers to that first (and only) field of the tuple variant — exactly the way `format!` already worked.

</details>

<details>
<summary>Does <code>#[derive(thiserror::Error)] enum Foo { #[error("bad")] Bad }</code> — without <code>#[derive(Debug)]</code> — compile?</summary>

Write down your answer — what did `std::error::Error` declare as a precondition?

</details>

<details>
<summary>Answer</summary>

No. `std::error::Error: Debug + Display` — and `#[derive(thiserror::Error)]` only builds `Display`. The error code is `E0277`.

</details>

<details>
<summary>You have a variant with <code>Io(#[from] std::io::Error)</code>. Does a bare <code>?</code> on a <code>Result&lt;T, std::io::Error&gt;</code>, inside a function returning <code>Result&lt;T, Foo&gt;</code>, compile?</summary>

Write down your answer — what does `#[from]` actually generate?

</details>

<details>
<summary>Answer</summary>

Yes. `#[from]` builds an `impl From<std::io::Error> for Foo`, and that alone is enough for `?` to run the conversion automatically.

</details>

<details>
<summary>True or false: <code>anyhow::Error</code> can only wrap error types defined in the same crate.</summary>

Write down your answer.

</details>

<details>
<summary>Answer</summary>

False. `anyhow::Error` accepts any type that implements `std::error::Error` — even a completely foreign type like `std::num::ParseIntError`, exactly as `04-anyhow-basics` showed.

</details>

<details>
<summary>A library function returns a validation error its caller might want to <code>match</code> on. The binary that calls that library has no caller of its own. Which one returns <code>thiserror</code>, and which returns <code>anyhow</code>?</summary>

Write down your answer — state the library/binary boundary rule in your own words.

</details>

<details>
<summary>Answer</summary>

The library returns `thiserror` (a precise enum), because its caller might need to branch on it. The binary returns `anyhow::Result`, because nothing downstream of it wants to `match`.

</details>

### Repair

Fix all three broken examples:

1. Fix `examples/07-derive-error-needs-debug.rs` so it compiles — without changing any `#[error("...")]`.
2. Fix `examples/08-context-needs-trait-import.rs` so it compiles. Don't try the compiler's second suggestion (`.with_context()`) — predict why it won't work first, then confirm by running it.
3. Fix `examples/09-from-needs-single-field.rs` so it compiles, without removing `line` from the `InvalidScore` variant.

### Implement

Two functions in `src/lib.rs`, on the `WatchNoteError` type, already fully written — because filling in `#[error("...")]` isn't a job for `todo!()`, it's an attribute, not executable code:

```sh
cargo test -p p2-05-03-thiserror-and-anyhow
```

`parse_watch_note()` parses one `"<episode>:<note>"` line; `load_watch_notes()` calls that same function on every line of a multi-line input and collects the result into `anyhow::Result`. Each function's doc comment states exactly what its output and error message should be — don't guess.

### Build

Add a new variant to `WatchNoteError`, for another kind of failure this format could have (say, a blank note after `:`, or episode number zero). Decide the `#[error("...")]` message yourself, and whether it needs `#[source]` or `#[from]` at all — most failures don't. Change `parse_watch_note()` so it actually returns your new variant, and write a comment explaining why you made that call.

### Challenge (optional)

Change one of `WatchNoteError`'s variants (or the one you wrote in "Build") so that, instead of wrapping a standard-library error like `ParseIntError`, it wraps a *different error type of your own* with `#[from]` — a small new `enum`, two or three variants, just for this. Then check whether `anyhow::Context` still shows the chain correctly three layers down. This is exactly the problem 2.5.4 solves in full: combining several library error types into one service-wide taxonomy.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `thiserror` | A derive macro that builds `Display`, `Error`, and (with `#[from]`) `From` from attributes | A library's error type, instead of writing it by hand |
| `#[error("...")]` | The `Display` message, per variant | The same interpolation syntax as `format!`, right on the variant |
| `#[source]` | Marks this field as what `Error::source()` should return | Error chains (2.5.2) |
| `#[from]` | `#[source]` plus an automatic `From` | Only when the field is the entire variant |
| `anyhow::Error` | One dynamic error type, for anything that's `std::error::Error` | The outermost layer, where no one matches on the kind |
| `.context(...)` / `.with_context(...)` | A more human-readable message on an error, without discarding the original | An error as it propagates through several layers |
| Library/binary boundary | A library is precise and matchable; a binary collapses it with `anyhow` | Deciding each function's return type |

### What you now know

- `#[derive(thiserror::Error)]` builds the same `Display` and `Error` you hand-wrote in 2.5.1/2.5.2 — no more, no less — and still needs your own `#[derive(Debug)]`.
- `#[error("...")]` has the same syntax as `format!`; `#[source]` wires up `source()`; `#[from]` also builds a `From` — only when the field is the entire variant.
- `anyhow::Error` accepts any type that is `std::error::Error`, even completely foreign ones; `?` inside a function returning `anyhow::Result<T>` always performs that conversion for free.
- `.context(...)` puts a new message on the chain without discarding the original error; `{}` shows only the new message, `{:?}` the whole chain.
- A library shows a precise, matchable error type because it doesn't know whether its caller needs to branch; a binary — the last layer, with no caller of its own — can collapse it into `anyhow`.
- This rule isn't absolute: wherever *something*, even inside that same binary, might later want to branch on the error, the right answer is still `thiserror`.

### What comes back later

- **Designing an error taxonomy for a real service, when several library error types have to live side by side** — [2.5.4 — Designing an error taxonomy for a service](../04-error-taxonomy-for-a-service/README.md)
- **This exact rule, on a real Axum service whose errors have to turn into consistent HTTP bodies** — [3.7.1 — Consistent error envelopes](../../../phase3-backend-foundations/08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)

### Can you explain?

- What exactly does `#[derive(thiserror::Error)]` build for you, and what does it not?
- Why doesn't `#[from]` compile on a variant that also has an extra field, such as `line`?
- Why does `?` inside a function returning `anyhow::Result<T>` work even for a completely foreign error type?
- What exactly does `.context(...)` change, so that `{}` and `{:?}` differ on the resulting error?
- Explain the library/binary boundary rule with a real example from this lesson — then give an example of where you'd break it.

---

## Going further

- [`thiserror` documentation](https://docs.rs/thiserror/latest/thiserror/) — the full list of attributes, including `#[error(transparent)]`, which we didn't see today.
- [`anyhow` documentation](https://docs.rs/anyhow/latest/anyhow/) — especially the comparison with `thiserror`, from the author of both crates.
- [`anyhow::Context` trait documentation](https://docs.rs/anyhow/latest/anyhow/trait.Context.html) — the exact signatures of `.context()` and `.with_context()`.
- [The Rust CLI book — handling errors](https://rust-cli.github.io/book/tutorial/errors.html) — this same library/binary boundary rule, from the angle of a real CLI tool.
