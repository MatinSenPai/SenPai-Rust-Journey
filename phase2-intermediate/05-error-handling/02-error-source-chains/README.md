# 2.5.2 — Source chains and `Box<dyn Error>`

## At a glance

After this lesson you can:

- Implement `source()` for an error variant that wraps another error, using the trait's exact signature — `Option<&(dyn Error + 'static)>` — not whatever signature looks reasonable.
- Walk an error chain — from a top-level failure down to its root cause — with a loop, and print every link.
- Choose between a custom enum (2.5.1) and `Box<dyn Error>` for a function's return type, and say in one sentence what each one gives you and what it costs you.

**Time:** ~60 minutes · **Prerequisites:**
[2.5.1 — Custom error types and `std::error::Error`](../01-custom-error-types/README.md),
[2.3.7 — Static versus dynamic dispatch, and object safety](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md)

---

## Why this matters

2.5.1 had you build a custom error enum and implement `std::error::Error` on it — and left one method untouched: `source()`. Its default implementation just returns `None`, and the lesson said, in one sentence, that you'd need it once your error wraps another error. Today that sentence gets paid off in full.

Picture a real program reading a config file off disk. Where the failure actually happens is usually several layers below wherever a user or a log line sees it — a standard-library call failed underneath, or the OS said "file not found." If your top-level error ("couldn't load the config") hides that root cause, anyone who later needs to debug it — or write structured logs, or retry specifically on one kind of underlying failure — is working blind. `source()` solves exactly this: a way for an error to point at the lower-level error underneath it, so a chain forms from the top-level failure down to its root cause.

There's a second question 2.5.1 left open, too. Your own custom enum is great when a caller genuinely needs to distinguish between failure kinds — retry on a timeout, ask the user for a different value on a validation error. But a lot of the time — especially near the edge of a program: a `main`, a script, a top-level handler that just wants to log whatever went wrong and exit — the caller never wants to `match` at all. For that case, a custom enum is pure overhead. Today you meet `Box<dyn Error>` — the "any error" box, built for exactly that moment — along with its real cost.

---

## The concept

### Revisiting `source()`: for real this time

2.5.1 only told you this much: `std::error::Error` has one method, `source()`, with a default implementation that returns `None`, and you didn't have to override it. You hadn't seen its full signature yet:

```rust
fn source(&self) -> Option<&(dyn Error + 'static)>
```

Three pieces, each for a reason. An `Option`, because plenty of errors don't wrap anything at all — a `MissingField` *is* the root cause, not a layer over some lower error. A reference, because `source()` doesn't take ownership of the underlying error, it only points at it; `self` still owns it. And a trait object — `dyn Error` — because whatever `self` wraps could be any type at all; `source()` can't know the exact concrete type ahead of time, [the same type-erasure idea 2.3.7](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md) showed you on `dyn Summarize`.

That trailing `'static` is the one part that might catch you off guard. It doesn't mean the value lives forever — it means the type behind that trait object can't itself be holding a short-lived borrow; it has to own its own data. Almost every error type you write already qualifies — a `String`, an `io::Error` that isn't borrowing anything — so in practice this rarely limits you. It just has to be written explicitly, because Rust doesn't infer it here from a guess.

Now put this on a real enum. `ConfigError` moves one step past 2.5.1: this time the config comes from a real file on disk, so a new variant is needed — `Io`, for when the file itself doesn't even open:

```rust
enum ConfigError {
    Io(io::Error),
    MissingField(String),
    InvalidNumber { field: String, source: ParseIntError },
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            ConfigError::MissingField(_) => None,
            ConfigError::InvalidNumber { source, .. } => Some(source),
        }
    }
}
```

`Io` and `InvalidNumber` each wrap another error, so they return `Some`. `MissingField` is the end of the line itself — there's no lower error for it to point at — so `None`. That's exactly the distinction the next section walks down.

```senpai-visual
{"kind":"borrowing","labels":["ConfigError::Io owns the io::Error","source() returns &","caller borrows it, never takes ownership"]}
```

Run it and see:

```rust
let bad_path = ConfigError::Io(io::Error::new(io::ErrorKind::NotFound, "no such file"));
let missing = ConfigError::MissingField("name".to_string());
let invalid = ConfigError::InvalidNumber {
    field: "max_retries".to_string(),
    source: "not-a-number".parse::<u32>().unwrap_err(),
};

report("bad_path", &bad_path);
report("missing", &missing);
report("invalid", &invalid);
```

```text
bad_path: could not read config file: no such file
  source: Some(no such file)
missing: missing required field: name
  source: None
invalid: invalid number for field 'max_retries': invalid digit found in string
  source: Some(invalid digit found in string)
```

### Walking the chain: a loop that goes until `None`

Now that `source()` is real, you can start at the top-level error and walk down, layer by layer — a chain. The pattern is always the same: print the error, then keep printing whatever `source()` hands back, until it doesn't:

```rust
println!("error: {top}");
let mut cause = top.source();
while let Some(err) = cause {
    println!("caused by: {err}");
    cause = err.source();
}
```

Try it on a real error — not a hand-built `io::Error`, but one that actually comes out of trying to read a file that doesn't exist:

```text
error: could not read config file: The system cannot find the path specified. (os error 3)
caused by: The system cannot find the path specified. (os error 3)
```

The OS message shows up twice — once because `ConfigError::Io`'s own `Display` already embeds it via `{e}` in its one-line message, and once again because you're explicitly walking `source()` here too. That's expected, not duplication for nothing: `Display` and `source()` serve two different audiences. `Display` is for a human reading one line; `source()` is for code that wants to inspect or log each layer separately — a structured logger writing each layer into its own field instead of one flat string, for instance.

```senpai-visual
{"kind":"result","labels":["top-level ConfigError","source()","io::Error — root cause","source()","None — chain ends"]}
```

### `Box<dyn Error>`: an "any error" box

Say one function does two unrelated things: it reads a config file's size (which can fail with `ConfigError`) and it parses a retry count from a string (which can fail with `ParseIntError` — a completely unrelated type, straight from the standard library). If the function's signature is `Result<_, ConfigError>`, that second line simply doesn't compile — `?` has no idea how to turn a `ParseIntError` into a `ConfigError`, because you never wrote a `From` for it. The fix is writing a `From<ParseIntError> for ConfigError` — but repeat that for *every* combination of error types in *every* function, and your custom enum turns into a long list of `impl From` blocks that have nothing to do with your program's actual logic. The exact error this produces is in "Errors you will meet."

`Box<dyn Error>` removes this problem at the root. [2.3.7 showed you exactly this idea](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md) — a trait object with its concrete type erased — on `Summarize`; this is the same `dyn Trait`, the same type erasure, now applied to `Error`. The standard library wrote one `From<E>` that works for **any** `E` implementing `Error`, as long as your return type is `Box<dyn Error>` instead of a fixed enum. That means `?` just works on any error implementing `std::error::Error`, with no hand-written `impl From` at all:

```rust
fn file_size(path: &str) -> Result<u64, ConfigError> {
    let contents = std::fs::read_to_string(path)?;
    Ok(contents.len() as u64)
}

fn run(path: &str, retry_count_text: &str) -> Result<(u64, u32), Box<dyn Error>> {
    let size = file_size(path)?;
    let retries: u32 = retry_count_text.parse()?;
    Ok((size, retries))
}
```

```text
file was 21 bytes, retry count was 5
```

`ConfigError` and `ParseIntError` know nothing about each other — there's no `impl From` between them — and both converted straight into `Box<dyn Error>`. That's what `Box<dyn Error>` is really selling: **one** return type that covers every error type implementing `Error`.

`Box` itself is used only exactly as much as needed here, nothing more: a heap box that turns any error into a same-size pointer, so the return type stays fixed. The full story of `Box` — heap allocation, ownership, when to reach for it on its own — belongs to [2.6.1](../../06-smart-pointers/01-box-and-heap-allocation/README.md), the same way [2.3.7](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md) borrowed it early too, for this exact one role.

### The real cost: you can no longer `match`

That convenience isn't free. Once your return type is `Box<dyn Error>`, the caller no longer knows which concrete enum or struct is actually sitting behind the box — that's exactly what type erasure means. `match err { ConfigError::Io(_) => ..., ... }` on a `Box<dyn Error>` simply doesn't compile; the compiler checks the pattern's type against the thing you're matching on, and they're no longer the same. The exact error is in "Errors you will meet."

The only way out is `downcast_ref::<T>()`: a guess — "maybe this is a `ConfigError`" — that hands back the concrete type if you guessed right, and just `None` if you guessed wrong. Unlike `match`, it isn't exhaustive: the compiler never forces you to cover every possible case, and a type you didn't think to guess silently falls through:

```rust
fn describe(err: &(dyn Error + 'static)) -> &'static str {
    if err.downcast_ref::<ConfigError>().is_some() {
        "a ConfigError"
    } else if err.downcast_ref::<ParseIntError>().is_some() {
        "a ParseIntError"
    } else {
        "something else"
    }
}
```

```text
a: config problem: missing field: name (a ConfigError)
b: invalid digit found in string (a ParseIntError)
```

One technical note: `downcast_ref` is only defined on `dyn Error + 'static` — the same bound `source()` needed — so `describe`'s signature has to spell it out, not just write bare `dyn Error`. Same rule, twice.

```senpai-visual
{"kind":"concept","labels":["ConfigError","ParseIntError","Box<dyn Error>","print or log: works everywhere","match on variant: needs downcast_ref, one guess at a time"]}
```

### Custom enum, or `Box<dyn Error>`?

Neither is always right — the question is what the caller genuinely needs:

| Axis | A custom enum (2.5.1) wins when... | `Box<dyn Error>` wins when... |
|---|---|---|
| Caller behavior | it has to `match` and do something different per kind | it just wants to print, log, or bubble the error up |
| How many underlying error types | usually a handful, all known ahead of time | any number of unrelated types |
| Where in the program | the core logic layer, where real decisions get made | near the edge — `main`, a script, a top-level handler |
| The cost | one `impl From` per conversion | the concrete type is gone; only recoverable by guessing with `downcast_ref` |

The short version: default to the custom enum from 2.5.1 wherever the caller genuinely has to decide something. Save `Box<dyn Error>` for the spots where an error is only ever going to be seen, not handled. And a note for later: both routes — hand-writing `Display` and `impl From` for an enum, or living with an unnamed `Box<dyn Error>` — are exactly the repetitive work the crates `thiserror` and `anyhow` exist to automate; [2.5.3](../03-thiserror-and-anyhow/README.md) shows you how.

---

## Hands on

```sh
cargo run -p p2-05-02-error-source-chains --example 01-hand-written-source
cargo run -p p2-05-02-error-source-chains --example 02-walking-the-chain
cargo run -p p2-05-02-error-source-chains --example 03-box-dyn-error-unifies-return-types
cargo run -p p2-05-02-error-source-chains --example 04-the-cost-and-downcasting
```

Then the three broken ones:

```sh
cargo run -p p2-05-02-error-source-chains --example 05-source-wrong-lifetime-broken --features broken
cargo run -p p2-05-02-error-source-chains --example 06-cannot-convert-without-box-broken --features broken
cargo run -p p2-05-02-error-source-chains --example 07-cannot-match-a-boxed-error-broken --features broken
```

Then try these:

1. In `01-hand-written-source.rs`, add a new variant — say, `ConfigError::Empty` for a completely empty config file — and write its `source()`. Should it be `Some` or `None`?
2. In `02-walking-the-chain.rs`, instead of a nonexistent path, create a real file you aren't allowed to read (or find any other way to get a genuinely different real `io::Error`), and see how the printed message differs.
3. In `04-the-cost-and-downcasting.rs`, add a third error type that `describe` never guesses, and see what it prints.

---

## Errors you will meet

### Mismatched `impl` signature — no error code

```text
error: `impl` item signature doesn't match `trait` item signature
   --> phase2-intermediate\05-error-handling\02-error-source-chains\examples\05-source-wrong-lifetime-broken.rs:26:5
    |
 26 |     fn source(&self) -> Option<&dyn Error> {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ found `fn(&'1 ConfigError) -> Option<&'1 (dyn std::error::Error + '1)>`
    |
   ::: C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\error.rs:111:5
    |
111 |     fn source(&self) -> Option<&(dyn Error + 'static)> {
    |     -------------------------------------------------- expected `fn(&'1 ConfigError) -> Option<&'1 (dyn std::error::Error + 'static)>`
    |
    = note: expected signature `fn(&'1 ConfigError) -> Option<&'1 (dyn std::error::Error + 'static)>`
               found signature `fn(&'1 ConfigError) -> Option<&'1 (dyn std::error::Error + '1)>`
    = help: the lifetime requirements from the `impl` do not correspond to the requirements in the `trait`
    = help: verify the lifetime relationships in the `trait` and `impl` between the `self` argument, the other inputs and its output
```

**What the compiler is objecting to:** writing `Option<&dyn Error>` (without `+ 'static`) is a different signature from what the trait declared. Without `'static`, Rust applies the ordinary method-elision rule to that reference and ties it to `&self`'s own lifetime instead — not to `'static`. The compiler shows you exactly this, side by side: the signature your `impl` wrote versus the one the trait requires.

**The fix:** write exactly what the trait declared:

```rust
fn source(&self) -> Option<&(dyn Error + 'static)> {
    Some(&self.inner)
}
```

**Why this is the fix:** a trait method's signature isn't negotiable once you're implementing it — it has to match exactly (same elision, or the same explicit lifetime). The trait spelled out `'static` here, so the `impl` has to spell it out too; an "approximately the same" signature doesn't stand in for it.

### `E0271` — without a `From`, `?` cannot convert

```text
error[E0271]: type mismatch resolving `<u32 as FromStr>::Err == ConfigError`
  --> phase2-intermediate\05-error-handling\02-error-source-chains\examples\06-cannot-convert-without-box-broken.rs:26:27
   |
26 |     let value: u32 = text.parse()?;
   |                           ^^^^^ expected `ConfigError`, found `ParseIntError`

For more information about this error, try `rustc --explain E0271`.
```

**What the compiler is objecting to:** `text.parse()` fails with a `ParseIntError` on a bad input. The function returns `Result<u32, ConfigError>`, and `?` looks for `impl From<ParseIntError> for ConfigError` to convert the error. Since no such `impl` exists, the compiler lines up the two types and says they aren't the same.

**The fix:** either write an `impl From<ParseIntError> for ConfigError`, or — exactly this lesson's subject — make the return type `Box<dyn Error>` so that `impl` is never needed in the first place:

```rust
fn read_retries(text: &str) -> Result<u32, Box<dyn Error>> {
    let value: u32 = text.parse()?;
    Ok(value)
}
```

**Why this is the fix:** the standard library wrote `From<E> for Box<dyn Error>` once, for every `E: Error`. Change the return type and you stop having to write that conversion by hand for every new pair of types.

### `E0308` — a `Box<dyn Error>` can't be `match`ed like its original enum

```text
error[E0308]: mismatched types
  --> phase2-intermediate\05-error-handling\02-error-source-chains\examples\07-cannot-match-a-boxed-error-broken.rs:37:9
   |
36 |     match err {
   |           --- this expression has type `Box<dyn std::error::Error>`
37 |         ConfigError::Io(_) => println!("io"),
   |         ^^^^^^^^^^^^^^^^^^ expected `Box<dyn Error>`, found `ConfigError`
   |
   = note: expected struct `Box<dyn std::error::Error>`
                found enum `ConfigError`

error[E0308]: mismatched types
  --> phase2-intermediate\05-error-handling\02-error-source-chains\examples\07-cannot-match-a-boxed-error-broken.rs:38:9
   |
36 |     match err {
   |           --- this expression has type `Box<dyn std::error::Error>`
37 |         ConfigError::Io(_) => println!("io"),
38 |         ConfigError::MissingField(_) => println!("missing"),
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Box<dyn Error>`, found `ConfigError`
   |
   = note: expected struct `Box<dyn std::error::Error>`
                found enum `ConfigError`

For more information about this error, try `rustc --explain E0308`.
```

**What the compiler is objecting to:** `err` has type `Box<dyn Error>`, but both `match` arms wrote patterns of type `ConfigError`. A pattern has to share the type of whatever you're matching on; these two don't, exactly as "The real cost" said they wouldn't.

**The fix:** either don't erase to `Box<dyn Error>` in the first place (keep the return type `ConfigError` if the caller genuinely needs to `match`), or `downcast_ref::<ConfigError>()` first and work on what comes back:

```rust
if let Some(config_err) = err.downcast_ref::<ConfigError>() {
    match config_err {
        ConfigError::Io(_) => println!("io"),
        ConfigError::MissingField(_) => println!("missing"),
    }
}
```

**Why this is the fix:** `downcast_ref` hands back exactly what `match` needs — a real `&ConfigError`, not a `&Box<dyn Error>`. But notice the catch: this only works once you've already guessed the underlying type is `ConfigError`; unlike matching the enum directly, nothing here forces you to cover every type that might actually show up.

---

## Exercises

### Warm up

<details>
<summary>You write <code>impl Error for MyError</code> and don't override <code>source()</code>. What does <code>my_error.source()</code> return?</summary>

`None` — the trait's default implementation always does, exactly what 2.5.1 showed you.

</details>

<details>
<summary>Does this compile?</summary>

```rust
impl std::error::Error for MyError {
    fn source(&self) -> Option<&dyn std::error::Error> {
        None
    }
}
```

</details>

<details>
<summary>Answer</summary>

No. The signature doesn't match what the trait declared — the elided lifetime is tied to `&self`, not `'static`. Exactly the error you saw above.

</details>

<details>
<summary>A function returns <code>Result&lt;T, Box&lt;dyn Error&gt;&gt;</code>. Can the caller tell exactly which concrete error type happened, with no extra work?</summary>

No — the concrete type is erased. The only way is `downcast_ref::<T>()`, and that only works once you've already guessed what `T` is.

</details>

<details>
<summary><code>ConfigError::Io</code> wraps an <code>io::Error</code>; <code>ConfigError::MissingField</code> wraps nothing. What should each one's <code>source()</code> return?</summary>

`Io` should return `Some(&io_error)`; `MissingField` should return `None`, since it's the end of the chain itself — there's no lower error.

</details>

<details>
<summary>What does this print? (assume <code>err</code> is a <code>ConfigError::Io</code> whose own <code>io::Error</code> has no <code>source()</code> of its own)</summary>

```rust
let mut cause = err.source();
let mut count = 0;
while let Some(c) = cause {
    count += 1;
    cause = c.source();
}
println!("{count}");
```

</details>

<details>
<summary>Answer</summary>

```text
1
```

The loop runs once — on that same `io::Error` `err` wraps — and since that `io::Error` has no `source()` of its own, the loop stops right there.

</details>

### Repair

Fix all three broken examples — not with a syntax trick, but by changing the actual decision that broke each one:

1. `examples/05-source-wrong-lifetime-broken.rs` — make `source()`'s signature exactly what the trait requires.
2. `examples/06-cannot-convert-without-box-broken.rs` — change `read_retries`'s return type so `?` works without writing a new `impl From`.
3. `examples/07-cannot-match-a-boxed-error-broken.rs` — recover the concrete type with `downcast_ref` before `match`ing it.

### Implement

Five spots in `src/lib.rs` — `parse_config_str` is written in full already (that's 2.5.1's skill, not this lesson's); these five are today's:

```sh
cargo test -p p2-05-02-error-source-chains
```

- `impl From<std::io::Error> for ConfigError`
- `source()` — the trait's exact signature, `Some`/`None` per variant
- `load_config` — read, then parse
- `error_chain` — the error's own message, then each `source()` link until `None`
- `effective_max_retries` — two unrelated error types, one `Box<dyn Error>`

The exact specification for each — including exactly what each case should do — is in the doc comment above the function itself.

### Build

Design a small error type of your own that wraps another error — anything you like, say a "manifest loading" error wrapping a `std::env::VarError` or some other parse-style error you build yourself (no need to actually pull in a `serde`-style parser; a small hand-built error works fine). Write its `source()` correctly, trigger it for real at least once (not hand-built — make it actually happen), and print its chain using this lesson's pattern. In a comment, say which variant of your enum should return `None`, and why.

### Challenge (optional)

Take a `Box<dyn Error>` out of your `effective_max_retries` and `match` on both possible concrete types (`ConfigError` and `ParseIntError`) using `downcast_ref` — exactly `describe`'s pattern from "The real cost," now your own.

Then look a little further ahead. 2.5.1's custom enum needed a hand-written `impl Display` and a pile of `impl From` blocks; today's `Box<dyn Error>` dodged those but sacrificed the concrete type. Two crates — `thiserror` and `anyhow` — exist to fix exactly these two headaches: one automates the `Display`/`From` boilerplate for custom enums, the other makes working with `Box<dyn Error>` more ergonomic. Without installing anything, skim each one's `docs.rs` page and guess which one solves which problem — [2.5.3](../03-thiserror-and-anyhow/README.md) gives you the answer.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `source()` | the `Error` trait's method returning the wrapped-up underlying error: `Option<&(dyn Error + 'static)>` | any variant that wraps another error |
| Error chain | walking `source()` repeatedly, from a top-level error down to the root cause | structured logging, a full error report for debugging |
| `Box<dyn Error>` | a type-erased "any error" container; one return type for every error implementing `Error` | near the edge of a program, where an error is only ever going to be seen |
| Downcasting (`downcast_ref`) | recovering the concrete type from a `dyn Error + 'static`, once you've already guessed what it is | the only way out of `Box<dyn Error>`'s cost, not a substitute for `match` |

### What you now know

- `source()` has a fixed signature — `Option<&(dyn Error + 'static)>` — and something "approximately" that doesn't compile.
- A variant that wraps another error returns `Some`; a variant that is itself the root cause returns `None`.
- Walking `source()` until `None` builds a chain — the exact pattern behind every structured error-logging tool you've ever seen.
- `Box<dyn Error>` gathers every error type implementing `Error` behind one return type, with no `impl From` needed at all.
- That convenience isn't free: behind `Box<dyn Error>` you can no longer `match` — only `downcast_ref`, one guess at a time, never exhaustive.
- The choice between a custom enum and `Box<dyn Error>` comes down to whether the caller genuinely needs to decide something per error kind, or just needs to see it.

### What comes back later

- **`Box<T>` and heap allocation, in full** — this lesson only used `Box` as a way to give a return type one fixed size; the complete story — [2.6.1 — `Box` and heap allocation](../../06-smart-pointers/01-box-and-heap-allocation/README.md).
- **`thiserror` and `anyhow`** — this lesson's hand-written boilerplate (`impl Display`, `impl From`, choosing between an enum and `Box<dyn Error>`) is exactly what these two crates automate — [2.5.3 — `thiserror` versus `anyhow`](../03-thiserror-and-anyhow/README.md).
- **Designing a full error taxonomy for a service** — today was one small `ConfigError`; designing a whole service's error hierarchy, layer by layer — [2.5.4 — Designing an error taxonomy for a service](../04-error-taxonomy-for-a-service/README.md).
- **`Send`/`Sync` on `Box<dyn Error>`** — a bare `Box<dyn Error>` doesn't by itself promise it can cross a thread boundary; that needs `Box<dyn Error + Send + Sync>` — [2.8.4 — `Send` and `Sync`](../../08-concurrency/04-send-and-sync/README.md).

### Can you explain?

- Why isn't `Option<&dyn Error>` (without `'static`) a valid signature for `source()`?
- You have an enum with three variants, two of which wrap another error and one that doesn't. What should each one's `source()` return, and why?
- How does `Box<dyn Error>` let `?` move between completely unrelated error types, with no `impl From` at all?
- Why doesn't `match err { ConfigError::Io(_) => ... }` compile on a `Box<dyn Error>`, but it does on a bare `ConfigError`?
- Compare `downcast_ref` with `match`: what guarantee does each one give you that the other doesn't?
- For a real function you know — from a project, or from Phase 1 — say whether a custom enum or `Box<dyn Error>` would have suited it better, and why.

---

## Going further

- [`std::error::Error` documentation](https://doc.rust-lang.org/std/error/trait.Error.html) — `source()`'s full signature, from the standard library itself.
- [The `std::error` module](https://doc.rust-lang.org/std/error/index.html) — where `impl<E: Error> From<E> for Box<dyn Error>` is actually written.
- [The Rust Book, ch. 9 — Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html) — the broader view of this same subject, from the Rust team itself.
- [`Box<T>` documentation](https://doc.rust-lang.org/std/boxed/struct.Box.html) — the same tool [2.3.7](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md) used to size trait objects, today applied to `dyn Error`.
