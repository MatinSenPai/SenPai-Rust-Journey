# 05 — Reading compiler errors

## At a glance

After this lesson you can:

- Take a full `rustc` diagnostic apart and say what each part is for: the error code, the file and position, the code frame, the labels, `note` and `help`.
- Get the complete explanation of any error code with `rustc --explain`, without reaching for a search engine.
- Say what `cargo check` and `cargo build` differ on, and why you run the first one while you're working.

**Time:** ~40 minutes · **Prerequisites:** [03 — Hello, Rust](../03-hello-rust/README.md) and [04 — Cargo basics](../04-cargo-basics/README.md)

---

## Why this matters

In your first month with Rust you will hit **far** more compile errors than you expect. That is everyone's experience, not a sign you're bad at this.

But there's a large gap between two people who each hit thirty errors a day:

- One sees the error, their eyes slide off a long block of English, they copy the red line into a search box.
- The other reads it, works out in ten seconds what the compiler is objecting to, and fixes it.

The difference is not talent. It's that the second person learned once **what this text is shaped like**.

Rust's compiler has arguably the best error messages of any mainstream language. It routinely suggests the correct name, underlines exactly the characters at fault, and often writes the corrected line for you. All of which is worth nothing if you don't read it.

So this is one of the highest-value lessons in the course: **the compiler is your primary teacher, and this lesson is about how to listen to it.**

---

## The concept

### Anatomy of a diagnostic

Here's a real error, exactly as `cargo` prints it. We'll take it apart:

```text
error[E0425]: cannot find function `total_itens` in this scope
  --> phase0-setup\05-reading-compiler-errors\examples\02-unknown-name.rs:7:27
   |
 7 |     println!("items: {}", total_itens(7, 3));
   |                           ^^^^^^^^^^^
...
10 | fn total_items(orders: u32, per_order: u32) -> u32 {
   | -------------------------------------------------- similarly named function `total_items` defined here
   |
help: a function with a similar name exists
   |
 7 -     println!("items: {}", total_itens(7, 3));
 7 +     println!("items: {}", total_items(7, 3));
   |

For more information about this error, try `rustc --explain E0425`.
```

Six parts, each doing a job:

**1. The header: `error[E0425]: cannot find function ...`**

`error` means the program will not compile. `E0425` is the **error code** — a stable identifier you can look up directly. The sentence after it is a one-line summary.

**2. The location: `--> file:line:column`**

`7:27` means line 7, column 27. Most editors will jump straight there if you click it. (On Windows the path separator is `\`, on Linux and macOS `/` — that's the only difference.)

**3. The code frame: the lines shown between `|` markers**

That's your own code. Line numbers on the left. `...` means some lines were skipped to keep it readable.

**4. The labels: `^^^^` and `----`**

This is the most important part and the one most often skipped:

- `^^^^` underlines **where the problem is**. Here, exactly under `total_itens`.
- `----` underlines **something related to the problem**, with an explanation attached. Here, the definition of `total_items` with "similarly named function defined here".

So the compiler isn't saying "something is broken somewhere". It's saying "*this* is broken, and *that* is probably what you meant".

**5. `help:` and the `-` / `+` block**

That's a suggested fix, written as a diff: the `-` line is what you wrote, the `+` line is what the compiler proposes. It is, quite literally, writing the correct code for you.

**6. The last line: `try rustc --explain E0425`**

How to get the full explanation. That's the next section.

### Three levels: `error`, `warning`, `note`

| Level | What it means | Does it compile? |
|---|---|---|
| `error` | the compiler cannot continue | no |
| `warning` | it compiles, but it's probably not what you meant | yes |
| `note` / `help` | supporting text attached to an `error` or `warning` | — |

Take warnings seriously. "Unused variable" usually means either you wrote a line for nothing or you misspelled a name. The compiler is politely telling you something is off.

`note` and `help` never appear alone; they're always attached to an `error` or `warning` and are part of the same diagnostic. Read them too — the actual answer is often in there.

### Error codes and `rustc --explain`

Every error code has a full explanation page, installed on your own machine. No internet required:

```sh
rustc --explain E0425
```

You get several paragraphs with a broken example, a fixed example, and why. For errors you're meeting for the first time — especially the ownership errors in Phase 1 — this is a far better starting point than the first search result, which may describe a version of the language from three years ago.

Build the habit now: **new error, `--explain` first.**

### `cargo check` versus `cargo build`

```sh
cargo check    # just check it. Don't produce a binary.
cargo build    # check it and produce a binary.
```

`cargo check` does all of the compiler's analysis — the same type checking, the same borrow checking, the same errors — and skips only the final step of generating machine code. That final step is the expensive part.

In practice that means `cargo check` is several times faster than `cargo build` and gives you **exactly the same errors**.

So your loop becomes:

```sh
cargo check    # ten times a minute, while writing
cargo test     # when you think it's right
cargo build    # when you actually want a binary
```

If nothing else sticks from this lesson, let it be reaching for `check` instead of `build` while you work. Your feedback loop gets several times faster.

### When several errors arrive together

Rust reports every error it finds, not just the first. Sometimes that's twenty of them, and it's alarming.

**Always read the first one first.** Errors cascade: one real mistake can produce nineteen more that are all consequences of it. A single wrong type name breaks every place that type is used.

The routine: fix the first error, run `cargo check` again, see what's left. Very often all twenty go at once.

### When the error points somewhere else

Sometimes — not always, but sometimes — the compiler flags a line where the real problem isn't. The usual cause is **type inference**.

Rust works out the types of many things for you. If you supply a wrong type on line 10, the compiler may not notice the contradiction until line 40, and it will underline line 40. That `^^^^` marks where the contradiction *became apparent*, not necessarily where it was *created*.

The symptom is recognisable: you stare at the flagged line and can't find anything wrong with it. When that happens, read the `note:` lines — they usually point at the real place — and work backwards to where that value was created.

You won't fully understand this yet, and you don't need to. Just know it exists, so you don't think you're going mad the first time it happens.

---

## Hands on

First run a program that works, so you know what "it worked" looks like:

```sh
cargo run -p p0-05-reading-compiler-errors --example 01-tour
```

```text
orders: 7
items:  21
```

Now four deliberately broken programs. Each has exactly one mistake:

```sh
cargo run -p p0-05-reading-compiler-errors --example 02-unknown-name  --features broken
cargo run -p p0-05-reading-compiler-errors --example 03-wrong-type    --features broken
cargo run -p p0-05-reading-compiler-errors --example 04-wrong-arity   --features broken
cargo run -p p0-05-reading-compiler-errors --example 05-no-such-method --features broken
```

**Why does `--features broken` exist?** Because `cargo test` compiles examples by default, and these four deliberately don't compile — without the flag the whole repository would be red for everyone. Putting them behind a feature makes running them a deliberate act. Cargo features get taught properly in Phase 2; this is just the first place the course needed one.

**Before you read the next section**, run all four and, for each, say out loud or on paper:

1. What is the error code?
2. What is `^^^^` under?
3. What do the `----` labels point at?
4. Did the compiler suggest a fix?

Then read the next section and compare. The order matters — read the explanation first and you're only confirming it; work it out first and you're actually practising.

---

## Errors you will meet

### `E0425` — a name that doesn't exist

```text
error[E0425]: cannot find function `total_itens` in this scope
  --> phase0-setup\05-reading-compiler-errors\examples\02-unknown-name.rs:7:27
   |
 7 |     println!("items: {}", total_itens(7, 3));
   |                           ^^^^^^^^^^^
...
10 | fn total_items(orders: u32, per_order: u32) -> u32 {
   | -------------------------------------------------- similarly named function `total_items` defined here
   |
help: a function with a similar name exists
   |
 7 -     println!("items: {}", total_itens(7, 3));
 7 +     println!("items: {}", total_items(7, 3));
   |
```

**What the compiler is objecting to:** you referred to a name that doesn't exist in this scope. So it's either a typo, a missing `use`, or something you genuinely haven't defined yet.

**The fix:** change `total_itens` to `total_items`. The compiler has written it out for you.

**Why that's the fix:** the compiler holds every name in scope and searched them for the closest match. That `----` label on the function definition is the match it found. This is Python's `NameError`, moved to before the program runs and with the correct name attached.

### `E0308` — the wrong type

```text
error[E0308]: mismatched types
  --> phase0-setup\05-reading-compiler-errors\examples\03-wrong-type.rs:7:39
   |
 7 |     println!("items: {}", total_items("7", 3));
   |                           ----------- ^^^ expected `u32`, found `&str`
   |                           |
   |                           arguments to this function are incorrect
   |
note: function defined here
  --> phase0-setup\05-reading-compiler-errors\examples\03-wrong-type.rs:10:4
   |
10 | fn total_items(orders: u32, per_order: u32) -> u32 {
   |    ^^^^^^^^^^^ -----------
```

**What the compiler is objecting to:** read `expected ... found ...` and always read it in that order — the compiler expected `u32` and got `&str`. You passed the string `"7"` where a number belongs.

**The fix:** change `"7"` to `7`.

**Why that's the fix:** in Rust the string `"7"` and the number `7` are entirely different types, and there is no automatic conversion between them. Python doesn't give you 21 from `"7" * 3` either — it gives `"777"`. The difference is that Rust stops it before the program runs instead of quietly producing something strange.

Notice `note: function defined here` too: the compiler shows you the definition so you don't have to go looking.

`E0308` is the error you'll hit most in Phase 1. Memorise its shape: **what was expected, what was found.**

### `E0061` — the wrong number of arguments

```text
error[E0061]: this function takes 2 arguments but 1 argument was supplied
  --> phase0-setup\05-reading-compiler-errors\examples\04-wrong-arity.rs:7:27
   |
 7 |     println!("items: {}", total_items(7));
   |                           ^^^^^^^^^^^--- argument #2 of type `u32` is missing
   |
note: function defined here
  --> phase0-setup\05-reading-compiler-errors\examples\04-wrong-arity.rs:10:4
   |
10 | fn total_items(orders: u32, per_order: u32) -> u32 {
   |    ^^^^^^^^^^^              --------------
help: provide the argument
   |
 7 |     println!("items: {}", total_items(7, /* u32 */));
   |                                        +++++++++++
```

**What the compiler is objecting to:** the function wants two arguments and you gave one. It even names which one is missing: argument #2, of type `u32`.

**The fix:** supply the second argument, e.g. `total_items(7, 3)`.

**Why that's the fix:** a function signature in Rust is a contract. There are no default parameter values (unlike Python's `def f(a, b=3)`), so every call must supply exactly that many arguments. The `/* u32 */` in the suggestion is a placeholder — you put the real value there.

### `E0599` — no such method

```text
error[E0599]: no method named `lenght` found for reference `&str` in the current scope
 --> phase0-setup\05-reading-compiler-errors\examples\05-no-such-method.rs:8:34
  |
8 |     println!("length: {}", title.lenght());
  |                                  ^^^^^^
  |
help: there is a method `len` with a similar name
  |
8 -     println!("length: {}", title.lenght());
8 +     println!("length: {}", title.len());
  |
```

**What the compiler is objecting to:** you called a method named `lenght` on a value of type `&str`, and no such method exists on that type.

**The fix:** change `lenght` to `len`.

**Why that's the fix:** the phrase `found for reference &str` is the important part — the compiler is telling you which *specific type* it searched. That sentence becomes the key to much harder errors later: when it says a method doesn't exist on a type, the first question is "what type is this value actually?", not "why is this method missing?".

---

## Exercises

### Warm up

Answer from the error text alone, without running anything.

<details>
<summary>In <code>error[E0308]: mismatched types</code> reading <code>expected `u32`, found `&str`</code> — which one is the thing you wrote?</summary>

`&str` — the `found` one. Always: `expected` is what the compiler wanted, `found` is what you gave it. Read them the wrong way round and you'll go looking for the problem in the wrong direction every time.

</details>

<details>
<summary>In the code frame, what's the difference between <code>^^^^</code> and <code>----</code>?</summary>

`^^^^` is under where the problem is. `----` is under something related that helps you understand it — the definition of the function, say, or where a variable was first bound.

</details>

<details>
<summary>You got twenty errors. Which do you fix first, and why?</summary>

The first one. Errors cascade: one real mistake can produce nineteen consequences. Fix the first, run <code>cargo check</code> again, then see what's left — usually far fewer.

</details>

<details>
<summary><code>cargo check</code> is faster than <code>cargo build</code>. What does it skip, and does it find fewer errors?</summary>

It skips generating machine code. It finds exactly the same errors — all the type and borrow checking still runs. It just doesn't hand you a binary.

</details>

### Repair

Actually fix the four broken examples above. Edit the files in `examples/` until all four run:

```sh
cargo run -p p0-05-reading-compiler-errors --example 02-unknown-name --features broken
```

After fixing, all four should run without error. For each one, **say what's wrong before you type anything** — that's the real exercise, not the typing.

### Implement

Complete the three functions in `src/lib.rs`. Each is fully specified in its own doc comment; you shouldn't need to open the test file.

```sh
cargo test -p p0-05-reading-compiler-errors
```

I've deliberately not told you how to write them. If you get stuck, read the error message — that's what this lesson is about.

### Build

Write a broken example of your own. Create `examples/06-mine.rs` containing **exactly one** compile error, and register it behind the `broken` feature in `Cargo.toml` like the others.

Then predict its error code, run it, and see whether you were right.

It's harder than it sounds: many breakages produce more than one error.

### Challenge (optional)

Run `rustc --explain E0382`. That's the "use of moved value" error — the heart of Rust's ownership system, which you haven't studied yet.

You won't fully understand it, and that's fine. What I want you to do is look at the **shape** of the explanation: broken example, fixed example, reason. When you genuinely hit this error in [Phase 1 — Move semantics](../../phase1-fundamentals/02-ownership-and-memory/01-move-semantics/README.md), this page is the first place you'll go.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| diagnostic | a complete compiler message with all its parts | every time something doesn't compile |
| error code (`E0308`) | the stable identifier for a kind of error | the argument to `rustc --explain` |
| label (`^^^^` / `----`) | "the problem is here" / "this is related" | finding the actual location |
| `expected` / `found` | what the compiler wanted / what you supplied | almost every type error |
| `cargo check` | the same checks, without building a binary | your fast writing loop |
| cascading errors | errors that are consequences of one mistake | when you get twenty at once |

### What you now know

- A `rustc` diagnostic has six parts, each doing a specific job.
- `^^^^` is the problem and `----` is the relevant context.
- `expected` is what the compiler wanted, `found` is what you gave it — in that order.
- `rustc --explain` has the full explanation on your own machine.
- `cargo check` gives the same errors much faster.
- You fix the first error first.

### What comes back later

- **`clippy`** — a linter that goes past "does it compile" to "is this idiomatic": [06 — Tooling](../06-tooling-clippy-fmt-rust-analyzer/README.md)
- **`E0308` on real types** — when you meet Rust's type system properly: [Phase 1 — Foundations](../../phase1-fundamentals/01-variables-types-control-flow/README.md)
- **`E0382` and the ownership error family** — Rust's most feared errors, each with its own lesson: [Phase 1 — Ownership and memory](../../phase1-fundamentals/02-ownership-and-memory/README.md)
- **Bytes versus characters** (which you met in `title_len`): [Phase 1 — UTF-8, bytes, chars](../../phase1-fundamentals/04-strings-and-slices/01-string-vs-str/README.md)
- **Cargo features** (that `--features broken`): [Phase 2 — Toolbox](../../phase2-intermediate/08-rust-toolbox/03-cargo-features/README.md)

### Can you explain?

- Name the six parts of a compiler diagnostic and what each does.
- What's the difference between `^^^^` and `----`?
- In `expected u32, found &str`, which one is your code?
- What does `cargo check` skip, and why does it still catch every error?
- You have twenty errors. What's your next move?
- Where do you get the full explanation of an error code with no internet?

---

## Going further

- [Rust Error Index](https://doc.rust-lang.org/error_codes/error-index.html) — what `--explain` shows you, on the web.
- [`rustc` book — Error codes](https://doc.rust-lang.org/rustc/error-codes.html) — background on how the codes work.
- [Shape of errors to come](https://blog.rust-lang.org/2016/08/10/Shape-of-errors-to-come.html) — the Rust team's post on why they designed this format. A little old, but it explains the reasoning well.
