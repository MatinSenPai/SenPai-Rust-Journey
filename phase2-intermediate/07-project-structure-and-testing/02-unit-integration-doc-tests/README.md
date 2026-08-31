# 2.7.2 — Unit, integration, and doc tests

## At a glance

After this lesson you can:

- Name the pattern you've been writing non-stop since Phase 0, and say exactly *why* `#[cfg(test)] mod tests` can reach private items.
- Write an integration test in `tests/`, state in advance exactly which items it can and cannot reach, and read the real compiler error when you guess wrong.
- Turn an example inside a doc comment — including one that deliberately shows a panic or an `Err` path — into a real, running test, and say which of the three test kinds fits which situation.

**Time:** ~55 minutes · **Prerequisites:** [2.7.1 — Modules and visibility](../01-modules-visibility-workspaces/README.md)

---

## Why this matters

Go back to the very first lesson where you wrote a line of Rust in this course — [0.3 — Hello, Rust](../../../phase0-setup/03-hello-rust/README.md). Right there, at the bottom of `src/lib.rs`, was this block:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shouts() {
        assert_eq!(shout("here we go"), "HERE WE GO!");
    }
}
```

And that lesson said one precise sentence about it: "In Rust the small ones live in the same file as the code they test... That's not laziness — it lets a test reach private functions the outside world can't see" — and then added: "You'll write tests yourself in Phase 2."

From that day to right now, you've copied, edited, and skimmed past this exact block dozens of times in `src/lib.rs`, without a single lesson ever formally saying what it *is*. This lesson cashes that promise in: it names this block, and says *why* that `assert_eq!` above even works. Then it sets its two siblings next to it — a test that lives outside the crate, and a test that lives inside a comment — and shows you how all three add up to one complete picture.

There's a second thread here too. [2.7.1](../01-modules-visibility-workspaces/README.md) taught you the difference between `pub`, `pub(crate)`, and plain private — but there, the distinction stayed a little abstract, because all the code lived in one file. Here, exactly the same rules get a real, concrete consequence: which of your code can call a private function, and which of it can't even name it.

---

## The concept

### Unit tests: the same block you've been writing for months

First look at this — no `#[cfg(test)]`, just as an ordinary program:

```rust
fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

fn celsius_to_fahrenheit(c: f64) -> f64 {
    round1(c * 9.0 / 5.0 + 32.0)
}

fn main() {
    println!("{}", celsius_to_fahrenheit(0.0));
    println!("{}", round1(3.14159));
}
```

```text
32
3.1
```

Nothing strange here: `main` calls `round1` directly, even though `round1` has no `pub` at all. Why does it work? Because there's no such thing as "globally private" — privacy is always relative to a boundary, and that boundary (per [2.7.1](../01-modules-visibility-workspaces/README.md)) is the module and its descendants. `main` is written right here, in this same file — the same module. There's no wall for it to cross.

Now a **unit test** does exactly the same trick:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_correctly() {
        assert_eq!(round_to_one_decimal(1.24), 1.2);
    }
}
```

`#[cfg(test)]` means "only compile this module when building for `cargo test`" — your shipped binary never carries this code. `#[test]` marks one function as a test so `cargo test` finds and runs it. But the real point is elsewhere: this `mod tests` is a submodule of the same file, so exactly the rule above applies to it too — `use super::*;` brings in everything in `src/lib.rs`, private items included, into this module. A unit test isn't reaching into anything special; it's just using the same privilege every other bit of code in this file already has.

### Integration tests: the crate's real boundary

Any file you put under a top-level `tests/` directory, Cargo compiles differently: as its own **entirely separate crate**, depending on your library exactly the way an outside user would:

```rust
use p2_07_02_unit_integration_doc_tests::celsius_to_fahrenheit;

#[test]
fn converts_body_temperature() {
    let result = celsius_to_fahrenheit(37.0);
    assert!((result - 98.6).abs() < 0.5);
}
```

This compiles and passes — `celsius_to_fahrenheit` is public. But picture the same file with this line instead of that `use`:

```text
use p2_07_02_unit_integration_doc_tests::round_to_one_decimal;
```

It no longer compiles. Not a subtle bug, not a confusing message — an explicit error saying this function is private. You can try it yourself: `examples/02-private-item-is-unreachable.rs` has exactly this line; its full output is in "Errors you will meet".

Why did this fail to compile when the unit test above compiled just fine? Because this file is no longer "the same module." From the compiler's point of view, this file is an *outside* consumer of your library — exactly what [2.7.1](../01-modules-visibility-workspaces/README.md) said: `pub` means "visible to anyone who can see the module itself," and a different crate can never see a private item, no matter how tightly it depends on this one. The same holds for `pub(crate)`: what 2.7.1 called "visible anywhere in this same crate" is, from an integration test's point of view — whose crate has changed — every bit as invisible as plain private.

```senpai-visual
{"kind":"concept","labels":["src/lib.rs: one pub fn, one private fn","mod tests in the same file — same boundary, sees both","tests/*.rs — separate crate, sees pub only","/// doc comment — compiled separately, sees pub only"]}
```

This is exactly where an integration test earns its keep: somewhere a unit test structurally cannot go. If you ever change `celsius_to_fahrenheit` from `pub` back to private (or forget to re-export it from the crate root), every unit test in that same file keeps passing without complaint — because it has special access — but every integration test, and every real user of your crate, breaks that same instant. That's exactly the category of bug a unit test can never catch: an API that works from *inside* the crate but is either unreachable, or behaves differently, from outside.

### Doc tests: the comment that actually runs

```rust
/// Converts `c` from Celsius to Fahrenheit, rounded to one decimal place.
///
/// # Examples
///
/// ```
/// # use p2_07_02_unit_integration_doc_tests::celsius_to_fahrenheit;
/// assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
/// ```
pub fn celsius_to_fahrenheit(c: f64) -> f64 {
    // ...
}
```

You've seen this exact shape for months: a `///` comment, an `# Examples` heading, then a call and its answer. You saw exactly this in [0.3](../../../phase0-setup/03-hello-rust/README.md) — "`shout("here we go")` returns `"HERE WE GO!"`" — and in [1.5.2](../../../phase1-fundamentals/05-your-own-types/02-tuple-structs-and-newtype/README.md) you even saw a full fenced code block, tagged ` ```text `. Both were only ever there to *show* the shape of the output; neither one ever compiled. What's different about the block above is this: three bare backticks, with no tag at all — which defaults to Rust, not plain text. That one difference turns these few lines from a decorative example into code `cargo test` actually compiles and runs — a **doc test**.

Look at the line `# use ...;`. A `#` and a space, right at the start of the line. This line counts for compiling and running, but stays hidden from the *rendered* documentation — a trick for hiding setup noise (like this `use`) that a documentation reader doesn't need to see, but the compiler needs in order to build the example at all.

A doc test doesn't just show the optimistic path. It can explicitly promise a panic too — exactly the `#[should_panic(expected = "...")]` you wrote on a `#[test]` in [1.6.4](../../../phase1-fundamentals/06-absence-and-failure/04-panic-vs-result/README.md), this time spelled as a tag on the fence itself:

```rust
/// # Panics
///
/// Panics with the message `readings must not be empty` if `readings` is
/// empty.
///
/// ```should_panic
/// # use p2_07_02_unit_integration_doc_tests::average_celsius;
/// average_celsius(&[]);
/// ```
```

One difference: `should_panic` only asks "did it panic at all?" — unlike `#[should_panic(expected = "...")]`, it has no way to check the *text* of the message. That's why the `# Panics` section above spells out the exact message in words; the fence only proves the panic really happens.

And it can document an `Err` path just as well:

```rust
/// # Errors
///
/// Returns `Err` with the message `not a valid number: "<input>"` if
/// `input` cannot be parsed as a floating-point number.
///
/// ```
/// # use p2_07_02_unit_integration_doc_tests::parse_celsius;
/// assert_eq!(
///     parse_celsius("hot"),
///     Err("not a valid number: \"hot\"".to_string())
/// );
/// ```
```

No special tag needed — a plain `assert_eq!` on a `Result` is enough. The `# Examples`, `# Panics`, and `# Errors` headings are convention (the standard library uses the very same ones), not something the compiler recognizes; what actually compiles and runs is only the code fences underneath them.

Once you write all three functions in `src/lib.rs` and run `cargo test`, these five doc tests — two for `average_celsius`, two for `parse_celsius`, one for `celsius_to_fahrenheit` — run exactly like this:

```text
   Doc-tests p2_07_02_unit_integration_doc_tests_solution

running 5 tests
test src\lib.rs - average_celsius (line 35) ... ok
test src\lib.rs - average_celsius (line 24) ... ok
test src\lib.rs - celsius_to_fahrenheit (line 11) ... ok
test src\lib.rs - parse_celsius (line 49) ... ok
test src\lib.rs - parse_celsius (line 60) ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
```

(This particular run is from the fully-implemented reference — notice the `_solution` suffix on the crate name. You'll see this same set of five tests, with your own crate's name, once you finish the "Exercises" section — though doc tests run in parallel, so the order they print in can vary between runs; the five names and line numbers are what to check, not their sequence.) This is a guarantee a plain comment never gives you: a hand-written example sitting in prose can silently go stale the moment the function's real behavior changes; a doc test stops the build cold, that same instant. You'll see a real capture of exactly that failure in "Errors you will meet".

### Three tools, three questions

| Test kind | Where it lives | What it can see | What question it answers |
|---|---|---|---|
| Unit | `#[cfg(test)] mod tests`, bottom of the same file | Everything in the crate, private items included | "Does this internal piece work correctly?" |
| Integration | `tests/*.rs`, a separate crate | Only the `pub` API | "Does this crate work correctly from outside?" |
| Doc test | Inside a `///` comment, compiled separately | Only the `pub` API | "Is the example in the documentation still correct?" |

Reach for a unit test for internals and edge cases — exactly where access to a private helper like `round_to_one_decimal` makes the assertion itself simpler to write. Reach for an integration test for the crate's public *contract* — a question that only makes sense from a real consumer's point of view. And reach for a doc test not as some third, separate test you write on purpose, but as a guarantee attached to something you're writing *anyway*: an example a human reader sees. None of the three compete with each other; each one covers a different layer of the same crate.

---

## Hands on

First, the working version:

```sh
cargo run -p p2-07-02-unit-integration-doc-tests --example 01-preview-the-domain
```

```text
0C  -> 32F
37C -> 98.6F
round1(3.14159) called directly: 3.1
```

This file is a standalone, fully working copy of what you're about to build in `src/lib.rs` — a local copy, not a `use` from this lesson's own crate (which is why its names are a little different). Exactly what you saw in "The concept": `main` calls the private-looking `round1` directly, because both live in this one file.

Now two deliberately broken ones:

```sh
cargo run -p p2-07-02-unit-integration-doc-tests --example 02-private-item-is-unreachable --features broken
cargo run -p p2-07-02-unit-integration-doc-tests --example 03-average-of-nothing-panics --features broken
```

The first doesn't compile at all; the second compiles and panics on empty input. Both outputs, verbatim, are in "Errors you will meet".

Then try these:

1. In `01-preview-the-domain.rs`, add a new private function and call it from `main`. Does it compile? Why isn't that even a real question?
2. In `02-private-item-is-unreachable.rs`, change only the imported name from `round_to_one_decimal` to `celsius_to_fahrenheit` (leave the rest of the line alone). Why is that one change enough to make it compile?
3. In `03-average-of-nothing-panics.rs`, add `if readings.is_empty() { return; }` before the second call to `average`. Is that exactly what the real `average_celsius` should do, or is there a difference? (Look back at "# Panics" in "The concept".)

---

## Errors you will meet

Both examples below run against this same crate, which — since it's still `todo!()` — also prints a handful of unrelated "unused variable" and "never used" warnings alongside them. Ignore those; they disappear the moment you implement the functions. The outputs below show only the part that matters.

### `E0603` — a private function is unreachable from outside the crate

```text
error[E0603]: function `round_to_one_decimal` is private
  --> phase2-intermediate\07-project-structure-and-testing\02-unit-integration-doc-tests\examples\02-private-item-is-unreachable.rs:9:42
   |
 9 | use p2_07_02_unit_integration_doc_tests::round_to_one_decimal;
   |                                          ^^^^^^^^^^^^^^^^^^^^ private function
   |
note: the function `round_to_one_decimal` is defined here
  --> phase2-intermediate\07-project-structure-and-testing\02-unit-integration-doc-tests\src\lib.rs:10:1
   |
10 | fn round_to_one_decimal(x: f64) -> f64 {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

For more information about this error, try `rustc --explain E0603`.
error: could not compile `p2-07-02-unit-integration-doc-tests` (example "02-private-item-is-unreachable") due to 1 previous error
```

**What the compiler is actually complaining about:** `round_to_one_decimal` exists, but carries no `pub`. This file — exactly like any `tests/*.rs` file — is, from the compiler's point of view, an entirely separate crate that depends on the lesson's library, not part of that crate itself. The compiler even tells you where the function is defined (that `note:`), but that's informational, not permission.

**The fix:** import something that actually is `pub` instead:

```rust
use p2_07_02_unit_integration_doc_tests::celsius_to_fahrenheit;
```

**Why this is the fix:** `round_to_one_decimal` was never meant to be part of this crate's public contract — it's an implementation detail `celsius_to_fahrenheit` hides behind itself. Forcing it `pub` just so one outside file can call it is the same trap [2.7.1](../01-modules-visibility-workspaces/README.md) covered — picking a wider visibility than any real caller needs — just showing up here for a test's convenience instead of another module's.

### A run-time panic — the average of nothing

```text
20

thread 'main' (32392) panicked at phase2-intermediate\07-project-structure-and-testing\02-unit-integration-doc-tests\examples\03-average-of-nothing-panics.rs:10:5:
readings must not be empty
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**What the compiler is actually complaining about:** this isn't even a compiler error — the program built, ran, printed the first `average(&[10.0, 20.0, 30.0])` without trouble (`20`), and panicked exactly where it called `average(&[])`. The `assert!` inside the function checks its input before doing any arithmetic at all.

**The fix:** make sure the input isn't empty before you call it — either with an explicit check, or by changing the return type to something that can also express "no answer" (`Option`, for instance).

**Why this is the fix:** this is exactly the panic `should_panic` on `average_celsius` in "The concept" promised — not a bug, but an explicit contract: "the average of nothing doesn't mean anything, so this function panics on it." When a contract is stated explicitly and documented, it's the caller's job to honor it, not the function's job to soften its behavior.

### A doc test failure — when the example stops matching reality

There's no separate example file for this one in this lesson: a doc example that stays wrong forever can't be part of this lesson's committed code — `cargo test` would keep it red permanently. Instead, this output is captured from a completely separate scratch crate: the same `celsius_to_fahrenheit` function, with a deliberate mistake in the example itself (`celsius_to_fahrenheit(100.0)` written as `300.0`, not `212.0`):

```text
   Doc-tests doctest_drift_scratch

running 1 test
test src\lib.rs - celsius_to_fahrenheit (line 5) ... FAILED

failures:

---- src\lib.rs - celsius_to_fahrenheit (line 5) stdout ----
Test executable failed (exit code: 101).

stderr:

thread 'main' (32764) panicked at src\lib.rs:7:1:
assertion `left == right` failed
  left: 212.0
 right: 300.0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace



failures:
    src\lib.rs - celsius_to_fahrenheit (line 5)

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s

error: doctest failed, to rerun pass `--doc`
```

**What the compiler is actually complaining about:** the function itself worked fine — it returned `212.0`. The problem was somewhere else: the *example* in the comment was written wrong. `assertion left == right failed` says exactly that: what the function actually returned (`left: 212.0`) doesn't match what the example claimed (`right: 300.0`).

**The fix:** fix the example, not the function.

**Why this is the fix:** this is exactly the moment this lesson promised from its very first section: a doc test, unlike a plain comment, never lets an example fall behind real behavior. A plain comment would sit there quietly, lying; this one turns the whole `cargo test` run red until you fix it — right then, not six months later when a user has already trusted that same example.

---

## Exercises

### Warm up

<details>
<summary>Does this compile?</summary>

```rust
fn helper() -> i32 {
    42
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calls_helper() {
        assert_eq!(helper(), 42);
    }
}
```

</details>

<details>
<summary>Answer</summary>

Yes. `helper` has no `pub` at all, but `mod tests` is a submodule of this same file — the same privacy boundary. `use super::*;` brings in everything in that file, `helper` included.

</details>

<details>
<summary>Call that same <code>helper</code> from a <code>tests/foo.rs</code> file with <code>use my_crate::helper;</code> — does it compile?</summary>

No. `tests/*.rs` is its own entirely separate crate, and `helper` was never made `pub`; the compiler says exactly this function is private — not "not found," but "private."

</details>

<details>
<summary>Is this a doc test?</summary>

```rust
/// `add_one(4)` is `5`.
pub fn add_one(n: i32) -> i32 {
    n + 1
}
```

</details>

<details>
<summary>Answer</summary>

No. There's no code fence (three backticks) anywhere — this is just plain prose inside a comment, exactly the shape you've seen since 0.3 and through most of this course. `cargo test` never compiles or runs it; if `add_one` ever broke, this line would quietly keep lying.

</details>

<details>
<summary>What does this print when <code>cargo test</code> runs it?</summary>

```rust
/// ```
/// # fn double(n: i32) -> i32 { n * 2 }
/// assert_eq!(double(4), 8);
/// ```
```

</details>

<details>
<summary>Answer</summary>

Nothing, unless it fails. A passing `assert_eq!` prints nothing; it just passes the test, and `cargo test` shows one summary line at the end.

</details>

<details>
<summary>What happens to this test if its function doesn't panic?</summary>

```rust
#[test]
#[should_panic]
fn expects_a_panic() {
    // a body that does not panic
}
```

</details>

<details>
<summary>Answer</summary>

It FAILs. `#[should_panic]` wants the opposite of the usual case: if the function finishes without panicking, that quiet, ordinary finish is itself the test failure.

</details>

### Repair

Fix both broken examples:

1. Fix `examples/02-private-item-is-unreachable.rs` so it compiles — **without** making `round_to_one_decimal` public in `src/lib.rs`. (Hint: whatever this file should actually be testing was already public.)
2. Fix `examples/03-average-of-nothing-panics.rs` so it no longer panics — **without** removing the `assert!` from `average`. Replace the second call with a non-empty input, or add a check before it that prints something else when the input is empty.

### Implement

Four functions in `src/lib.rs` — each fully specified in its own doc comment, and every example inside that comment is a real test too:

```sh
cargo test -p p2-07-02-unit-integration-doc-tests
```

Until you've written all four, the `mod tests` at the bottom, `tests/public_api.rs`, and the doc tests inside the comments themselves will all three fail. That's expected — it's exactly why this lesson leaned so hard on "three kinds of test, one goal."

### Build

Add one new `pub` function of your own to this crate — anything you like (for instance `pub fn format_reading(c: f64) -> String`, turning a Celsius reading into a string like `"36.6C"`) — with a full doc comment, including an `# Examples` section that actually compiles and runs. Write it a unit test in `src/lib.rs` *and* an integration test in `tests/public_api.rs`.

Then run this experiment: deliberately break one of the assertions in that doc example (a wrong number, say), run `cargo test`, and read the failure carefully — exactly the shape you saw in "Errors you will meet", this time on your own function. Then put it back.

### Challenge (optional)

Run each of the three sections in isolation and see the difference:

```sh
cargo test -p p2-07-02-unit-integration-doc-tests --lib
cargo test -p p2-07-02-unit-integration-doc-tests --test public_api
cargo test -p p2-07-02-unit-integration-doc-tests --doc
```

Then mark one of `src/lib.rs`'s tests `#[ignore]` (with a comment next to it saying why), and run only that one with:

```sh
cargo test -p p2-07-02-unit-integration-doc-tests -- --ignored
```

This is exactly the problem [2.7.4](../04-property-and-snapshot-testing/README.md) and [2.7.5](../05-benchmarking-with-criterion/README.md) solve properly with tools built for it: tests that shouldn't run on every ordinary pass.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Unit test | A `#[test]` compiled as part of the crate itself; reaches private items too | Internals, edge cases |
| Integration test | A file in `tests/`, a separate crate; sees only `pub` | "Does the public API work from outside?" |
| Doc test | A code fence inside `///`; compiled and run for real | Documentation that can never quietly go stale |
| Hidden line (`# `) | Compiles inside the fence, hidden from rendered docs | Hiding setup noise |
| `should_panic` | A fence that only passes if it panics; doesn't check the message | Documenting a panic explicitly |
| `E0603` | "This function is private" | Reaching for a private item from outside the crate |

### What you now know

- `#[cfg(test)] mod tests` at the bottom of a file is a unit test: it compiles as part of the crate itself, so it reaches that same file's private items too — exactly what you've been writing since Phase 0.
- Every file under `tests/` is its own entirely separate crate that sees only the library's `pub` API — exactly like an outside user, not a special internal case.
- `pub(crate)`, like plain private, is invisible from an integration test's point of view; only genuine `pub` crosses that boundary.
- A three-backtick code fence inside `///` — untagged, or tagged `rust` — is compiled and run by `cargo test`; that same fence tagged `text` stays purely illustrative.
- Lines starting with `# ` inside that fence count for running but stay hidden from the rendered documentation.
- `should_panic` documents a promised panic; unlike `#[should_panic(expected = "...")]`, it doesn't check the message text.
- Unit tests for internals, integration tests for the public contract, doc tests so the examples inside your comments never quietly go stale.

### What comes back later

- **Test doubles, and why you rarely need a mocking framework** — [2.7.3 — Test doubles in Rust](../03-test-doubles-in-rust/README.md)
- **Property testing with `proptest`, snapshot testing with `insta`** — [2.7.4](../04-property-and-snapshot-testing/README.md)
- **Measuring performance with `criterion`** — [2.7.5](../05-benchmarking-with-criterion/README.md)
- **A real error type instead of a bare `String`** — [2.5.1 — Custom error types](../../05-error-handling/01-custom-error-types/README.md) and [2.5.3 — `thiserror` and `anyhow`](../../05-error-handling/03-thiserror-and-anyhow/README.md)

### Can you explain?

- Why can a unit test call a private function, but an integration test can't?
- How do you tell a doc example apart from a purely illustrative code block?
- What does the `# ` line inside a doc test fence do, and why is it needed?
- What does `should_panic` check, and what does it not check?
- Name a real bug an integration test catches that a unit test structurally cannot.

---

## Going further

- [The Rust Book — Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html) — this same ground, official.
- [The Rust Book — Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html) — unit versus integration, in full.
- [The rustdoc book — Documentation tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html) — every fence tag (`ignore`, `no_run`, `compile_fail`, and the rest), for when you're curious how deep this goes.
