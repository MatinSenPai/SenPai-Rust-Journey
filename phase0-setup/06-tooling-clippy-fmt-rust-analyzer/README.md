# 06 — Tooling: clippy, fmt, rust-analyzer

## At a glance

After this lesson you can:

- Read a `clippy` warning, understand *why* it's objecting, and decide whether to take it.
- Say what `cargo fmt` and `cargo fmt -- --check` differ on and which one CI runs.
- Say what rust-analyzer is and why Rust is twice as hard to write when it isn't working.

**Time:** ~40 minutes · **Prerequisites:** [05 — Reading compiler errors](../05-reading-compiler-errors/README.md)

---

## Why this matters

The compiler tells you whether your code **works**. These tools tell you whether your code is **good**.

And unlike Python, where each team picks `black` or `yapf` or nothing and argues about line length, Rust has effectively one style and it's the same everywhere. That's a freedom in itself: you never debate formatting.

The bigger deal is `clippy`. It isn't just a linter; for someone new to Rust it's a **style tutor**. It keeps saying "what you wrote works, but the idiomatic way is this" — and explains why. If you read every warning in your first month instead of blindly accepting the fix, you'll learn idiomatic Rust faster than any book will teach you.

This lesson is unlike the others: **the code already works and its tests are already green.** Your job isn't to fix it, it's to improve it.

---

## The concept

### `rustfmt` — the style argument is over

```sh
cargo fmt              # reformat every file in place
cargo fmt -- --check   # change nothing; just report whether it's needed
```

The second is what CI runs: it exits non-zero if anything is unformatted, and prints the diff.

The closest analogy is `black`, with two differences:

- **It's part of the toolchain**, not a third-party package. `rustup component add rustfmt` and you're done.
- **It has a default style almost nobody changes.** `rustfmt.toml` exists, but you rarely see one in a real project.

Suggestion: turn on "format on save" in your editor and never think about it again.

### `clippy` — past "does it compile"

```sh
cargo clippy                      # lint
cargo clippy --all-targets        # lint tests and examples too
cargo clippy -- -D warnings       # treat every warning as an error (what CI does)
cargo clippy --fix                # apply the safe suggestions automatically
```

`clippy` has over 750 lints, grouped:

| Category | What it means | Default |
|---|---|---|
| `correctness` | almost certainly a bug | error |
| `suspicious` | probably a bug | warn |
| `style` | works, but isn't idiomatic | warn |
| `complexity` | could be written more simply | warn |
| `perf` | could be made faster | warn |
| `pedantic` | opinionated, sometimes useful | off |
| `nursery` | lints still in development | off |

Look at that `correctness` row: `clippy` isn't just taste. Some of its lints catch real bugs the compiler doesn't.

### Reading a `clippy` warning

The shape is exactly the compiler diagnostic you learned in lesson 05, plus two extra lines:

```text
warning: writing `&String` instead of `&str` involves a new object where a slice will do
 --> phase0-setup\06-tooling-clippy-fmt-rust-analyzer\src\lib.rs:9:28
  |
9 | pub fn is_empty_name(name: &String) -> bool {
  |                            ^^^^^^^
  |
  = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.97.0/index.html#ptr_arg
  = note: `#[warn(clippy::ptr_arg)]` on by default
help: change this to
  |
9 - pub fn is_empty_name(name: &String) -> bool {
9 + pub fn is_empty_name(name: &str) -> bool {
  |
```

Two lines you don't get from compiler errors:

- **`= help: for further information visit ...#ptr_arg`** — a link to that lint's explanation page, which covers the *why* with examples. Read it; that's where the learning is.
- **`= note: #[warn(clippy::ptr_arg)] on by default`** — the lint's **name**. You need it if you ever want to silence it.

### When `clippy` is wrong

Sometimes it's right in general but wrong for your case. Then you silence it:

```rust
#[allow(clippy::needless_range_loop)]
fn sum_verbose(nums: &[i32]) -> i32 {
    // ...
}
```

**A rule to hold yourself to: every `#[allow(...)]` gets a reason, written down, right next to it.** An unexplained `allow` means "I couldn't be bothered", and six months later nobody — probably including you — knows whether it's still needed.

This lesson's `examples/01-before-and-after.rs` has one, with its reason: that function is deliberately un-idiomatic, because showing what clippy dislikes is its entire job.

### `rust-analyzer` — the compiler, inside your editor

rust-analyzer is a **language server**: it analyses your code in the background as you type and reports to your editor.

What it gives you:

- **Errors as you type**, before you run `cargo check`.
- **Inlay type hints** — the types of variables you didn't annotate, shown next to them. Excellent for learning: you see what the compiler inferred.
- **Jump to definition** — click any function from any crate and read its source.
- **Quick fixes** — add a missing `use`, generate a missing function, fill in a `match`.

If your editor isn't showing type hints, you're working twice as hard as you need to. Setup is in [`docs/setup-guide.md`](../../docs/setup-guide.md).

**Something many people don't know:** rust-analyzer and `cargo clippy` are separate. By default rust-analyzer runs `cargo check`, not clippy. In VS Code you can change that:

```json
"rust-analyzer.check.command": "clippy"
```

From then on you see clippy's warnings as you type. For learning, it's one of the highest-value settings you can turn on.

### In CI

This repo's `.github/workflows/ci.yml` runs:

```sh
cargo fmt --all -- --check
cargo clippy ... -- -D warnings
```

That `-D warnings` means "treat every warning as an error", so CI goes red on any of them.

**But only on finished code.** Lesson skeletons are deliberately unfinished: `todo!()` bodies leave parameters unused, and clippy is right to warn — but those warnings are the lesson's guidance, not a CI failure. So CI lints only `course-ui`, `lesson-lint` and the capstone crates strictly.

---

## Hands on

First see what "before and after" looks like:

```sh
cargo run -p p0-06-tooling-clippy-fmt --example 01-before-and-after
```

```text
verbose: 14
idiomatic: 14
```

Identical result, two styles. Now to the real code.

**1. First prove nothing is broken:**

```sh
cargo test -p p0-06-tooling-clippy-fmt
```

```text
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**2. Now run `clippy`:**

```sh
cargo clippy -p p0-06-tooling-clippy-fmt
```

You'll get seven warnings. **Read all of them** — not just the suggestions, the explanations.

**3. Break the formatting on purpose.** Take a function and mangle its spacing and indentation. Then:

```sh
cargo fmt -- --check      # watch it complain without changing anything
cargo fmt                 # now let it fix things
cargo fmt -- --check      # and it goes quiet
```

---

## Errors you will meet

These are warnings, not errors — the code compiles and the tests pass. But each one teaches you something.

### `clippy::ptr_arg` — `&String` where `&str` would do

```text
warning: writing `&String` instead of `&str` involves a new object where a slice will do
 --> src/lib.rs:9:28
  |
9 | pub fn is_empty_name(name: &String) -> bool {
  |                            ^^^^^^^
  |
help: change this to
  |
9 - pub fn is_empty_name(name: &String) -> bool {
9 + pub fn is_empty_name(name: &str) -> bool {
```

**Why it matters:** a function taking `&String` accepts only a `String`. A function taking `&str` accepts a `String` **and** a literal **and** a slice of a larger string — none of which copy anything. You were being restrictive for no gain.

It's the rule of thumb from lesson 03: **take `&str`, hand back `String`.** Clippy is drilling it into you.

### `clippy::len_zero` — `is_empty()` instead of `len() == 0`

```text
warning: length comparison to zero
  --> src/lib.rs:10:5
   |
10 |     name.len() == 0
   |     ^^^^^^^^^^^^^^^ help: using `is_empty` is clearer and more explicit: `name.is_empty()`
```

**Why it matters:** it reads better, and for some types `is_empty()` is cheaper than computing the whole length. A `style` lint that's also a `perf` one.

### `clippy::needless_return` — a `return` you don't need

```text
warning: unneeded `return` statement
  --> src/lib.rs:15:5
   |
15 |     return x * 2;
   |     ^^^^^^^^^^^^
   |
help: remove `return`
   |
15 -     return x * 2;
15 +     x * 2
```

**Why it matters:** the rule from lesson 03 — the last expression is the return value. Keep `return` for early exit, not for ordinary returns. Clippy is helping you unlearn a Python habit.

### `clippy::needless_range_loop` — looping over indices by hand

```text
warning: the loop variable `i` is only used to index `nums`
  --> src/lib.rs:36:14
   |
36 |     for i in 0..nums.len() {
   |              ^^^^^^^^^^^^^
   |
help: consider using an iterator
   |
36 -     for i in 0..nums.len() {
36 +     for <item> in &nums {
```

**Why it matters:** manual indexing is the classic habit carried over from other languages, and off-by-one errors come from exactly there. Iterating directly removes that whole category — and in Rust it also removes a bounds check.

That `<item>` in the suggestion is a placeholder; put a meaningful name in.

### `clippy::manual_unwrap_or_default` — a `match` that has a method

```text
warning: match can be simplified with `.unwrap_or_default()`
  --> src/lib.rs:49:5
   |
49 | /     match opt {
50 | |         Some(x) => x,
51 | |         None => 0,
52 | |     }
   | |_____^ help: replace it with: `opt.unwrap_or_default()`
```

**Why it matters:** `Option` has dozens of ready-made methods, and almost any `match` you write on one probably has a method already. You'll meet them properly in [Phase 1 — Option](../../phase1-fundamentals/06-option-result-error-basics/01-option-and-null-safety/README.md). For now, know that clippy knows them and will teach them to you.

---

## Exercises

### Warm up

<details>
<summary>What's the difference between <code>cargo fmt</code> and <code>cargo fmt -- --check</code>?</summary>

The first reformats files in place. The second changes nothing and only reports whether it's needed — exiting non-zero if so. CI runs the second.

</details>

<details>
<summary>Is <code>clippy</code> just taste?</summary>

No. The `correctness` category catches real bugs the compiler doesn't, and defaults to error rather than warning. `style` and `complexity` are more opinion, but they're the collective opinion of the whole ecosystem.

</details>

<details>
<summary>You get a warning and you're sure it's wrong for your case. What do you do?</summary>

Add `#[allow(clippy::the_lint_name)]` **together with a comment saying why**. The lint's name is in the warning's `note:` line. An unexplained `allow` is technical debt.

</details>

<details>
<summary>Does rust-analyzer run clippy by default?</summary>

No, it runs `cargo check`. To see clippy warnings as you type, set `rust-analyzer.check.command` to `"clippy"`.

</details>

### Repair

Clear all seven warnings in `src/lib.rs` until this is quiet:

```sh
cargo clippy -p p0-06-tooling-clippy-fmt
```

**Two rules:**

1. `cargo test -p p0-06-tooling-clippy-fmt` stays green throughout. Behaviour doesn't change, only style.
2. For each change, say **why** clippy is right before you make it. If you can't, open the `help:` link and read.

Note: `cargo clippy --fix -p p0-06-tooling-clippy-fmt` can apply five of them automatically. **Do them by hand first**, then compare against `--fix` on a copy if you like. The goal here is learning, not a green output.

### Implement

Once clippy is quiet, change `is_empty_name` so that all three of these work:

```rust
is_empty_name("");
is_empty_name(&String::new());
is_empty_name(&some_longer_string[0..0]);
```

If you changed the signature correctly, all three work with no further changes. If not, the compiler error tells you exactly why — that `E0308` you read about in lesson 05.

### Build

Break the formatting deliberately in three different files, then run:

```sh
cargo fmt --all -- --check
```

Read the output: it's exactly what you'd see on CI after committing something unformatted. Learn to read that diff — `-` is what you wrote, `+` is what rustfmt wants.

Then turn on format-on-save in your editor and never think about it again.

### Challenge (optional)

Run `clippy` in strict mode:

```sh
cargo clippy -p p0-06-tooling-clippy-fmt -- -W clippy::pedantic
```

You'll get many more warnings, and some are annoying. Read three of them and for each decide: would you keep this on in a real project?

There's no right answer. The exercise is learning to **negotiate** with a linter rather than either obeying it completely or ignoring it completely.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `rustfmt` | the standard formatter | `cargo fmt`, before every commit |
| `cargo fmt -- --check` | report only, don't change | CI |
| `clippy` | 750+ lints for style, complexity, performance and correctness | every day |
| lint category | `correctness` through `pedantic` | deciding whether to take a warning seriously |
| `#[allow(clippy::x)]` | silence one lint — always with a written reason | genuinely exceptional cases |
| `-D warnings` | treat every warning as an error | CI |
| language server (rust-analyzer) | in-editor analysis, as you type | continuously |
| inlay type hints | showing inferred types | learning the type system |

### What you now know

- Rust has one standard style and `cargo fmt` applies it.
- `clippy` isn't only taste; the `correctness` category catches bugs.
- Every clippy warning carries its lint name and a link to its explanation.
- `#[allow(...)]` without a written reason is technical debt.
- rust-analyzer can be pointed at clippy, and it's worth doing.
- CI lints only finished code strictly, not lesson skeletons.

### What comes back later

- **The git workflow and when CI actually runs** — [07 — Git and repo workflow](../07-git-and-repo-workflow/README.md)
- **The `Option` methods clippy keeps suggesting** — [Phase 1 — Option](../../phase1-fundamentals/06-option-result-error-basics/01-option-and-null-safety/README.md)
- **Iterators, and why `.iter().sum()` beats an index loop** — [Phase 2 — Iterators](../../phase2-intermediate/02-iterators-and-closures/02-iterator-adapters/README.md)
- **Workspace-level lints and CI that means it** — [Phase 4 — Deployment and operations](../../phase4-backend-advanced/07-deployment-and-operations/README.md)

### Can you explain?

- What's the difference between `cargo fmt` and `cargo fmt -- --check`, and which runs in CI?
- Name two clippy lint categories and how they differ.
- Where do you find a lint's name from its warning?
- When is writing `#[allow(...)]` right, and what must go with it?
- Why is taking `&str` better than taking `&String`?
- What is rust-analyzer, and why doesn't it run clippy by default?

---

## Going further

- [Clippy lint list](https://rust-lang.github.io/rust-clippy/master/) — searchable. Every lint with a good and bad example.
- [rust-analyzer manual](https://rust-analyzer.github.io/manual.html) — everything it can do. Read the "Assists" section; there are things in there you probably don't know exist.
- [`rustfmt` configuration](https://rust-lang.github.io/rustfmt/) — all the options you'll almost never need, but it's good to know they're there.
