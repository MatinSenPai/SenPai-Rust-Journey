# 1.6.4 — Panic versus `Result`

## At a glance

After this lesson you can:

- Decide, for a fresh failure, whether it should return `Result` or panic — based on whether the caller can reasonably be expected to handle it, not on taste.
- Write `assert!`, `assert_eq!`, `unreachable!()` and `debug_assert!` in the place each actually belongs, and say exactly what happens on the stack when a panic fires.
- Write an `.expect()` message that helps the person reading the log at 2am — not one that just restates that a value was missing.
- Separate the library, the application's startup, and a request handler as three different answers to this same question.

**Time:** ~55 minutes · **Prerequisites:** [1.6.3 — `Result` and the question mark](../03-result-and-question-mark/README.md)

---

## Why this matters

The last lesson put `Result` in your hands: an explicit way to say "this might fail, and here's why." But that isn't the whole story of failure in Rust. You've already seen the other half, way back in 1.1.2: a `u8` running past 255 panics in a debug build with a message and stops the program cold. In 1.1.6 you saw it again, more stubbornly — indexing a `Vec` out of bounds panics even in release. And in 1.2.5 a question was left open on purpose: "do destructors run if the program panics? This reaches forward to 1.6.4." Today that promise gets paid off.

So you now hold two genuinely different tools for "something went wrong," and this lesson isn't about learning new syntax — it's about **which one, when**. That's a real design decision you will make constantly from Phase 3 onward.

Python never draws this line for you. A `ValueError` from a user mistyping a field and an `IndexError` from your own code reaching past the end of a list both land in the same `except`. Rust separates them on purpose: one becomes part of a function's signature (`Result`) and forces you to deal with it; the other stops the program because something that was supposed to be impossible just happened. The compiler won't draw that line for you either — you draw it, and this lesson is about how.

---

## The concept

### The decision rule

Before any detail, the rule itself — because everything below is just this one sentence, worked out:

> **`Result` is for a failure the caller can reasonably be expected to handle. A panic is for a bug — a broken invariant, something that was supposed to be impossible.**

The practical question is "where did this value come from?" If it came from outside this program — something a person typed, a file, a network reply — being wrong is ordinary, expect it, return `Result`. If it came from inside this program — a data structure you built yourself, a precondition your own code was supposed to guarantee — being wrong means some part of this same codebase broke a promise, and that's a bug, not bad input.

Two functions side by side make the rule concrete:

```rust
fn parse_priority(input: &str) -> Result<u8, String> {
    let value: u8 = match input.trim().parse() {
        Ok(value) => value,
        Err(_) => return Err(format!("'{input}' is not a whole number")),
    };
    if !(1..=5).contains(&value) {
        return Err(format!("priority must be between 1 and 5, got {value}"));
    }
    Ok(value)
}
```

```text
parse_priority("3"): Ok(3)
parse_priority("9"): Err("priority must be between 1 and 5, got 9")
parse_priority("abc"): Err("'abc' is not a whole number")
```

`input` is text a person typed. It can be anything, and being invalid is the most ordinary thing in the world — so this returns `Result` and never panics.

```rust
fn checked_midpoint(sorted_ascending: &[i32]) -> i32 {
    assert!(
        !sorted_ascending.is_empty(),
        "checked_midpoint: caller must not pass an empty slice"
    );
    sorted_ascending[sorted_ascending.len() / 2]
}
```

```text
checked_midpoint(&[10, 20, 30, 40, 50]): 30
```

This one is different. `sorted_ascending` wasn't typed by a user — some other part of *this program* built it and handed it over. If it arrives empty, that isn't bad input; it means something in this program already broke its word. So it `assert!`s and panics instead of returning a `Result` nobody was expecting.

### A plain `panic!` — with an exact line number

`panic!` is the macro everything above is built on. An incomplete lookup table shows it plainly:

```rust
fn region_code(name: &str) -> u8 {
    match name {
        "ir" => 98,
        "us" => 1,
        "de" => 49,
        other => panic!(
            "region_code: no dialing code registered for {other:?} — this table is supposed \
             to cover every region this program deploys to"
        ),
    }
}
```

```text
ir -> 98
us -> 1

thread 'main' (2224) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\02-broken-invariant.rs:15:18:
region_code: no dialing code registered for "fr" — this table is supposed to cover every region this program deploys to
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

That table is supposed to cover every region this program deploys to — that's a rule *this codebase* owns, not something a caller typed. When the table and reality disagree, that's a bug, so it doesn't return a `Result` nobody would expect — it panics. Look at how precise the message is: the file, the exact line and column, and then exactly the text you gave `panic!`.

### `RUST_BACKTRACE=1` shows you where it came from

That last line of output above is a hint. Run the same program again with that variable set:

```text
thread 'main' (34352) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\02-broken-invariant.rs:15:18:
region_code: no dialing code registered for "fr" — this table is supposed to cover every region this program deploys to
stack backtrace:
   0: std::panicking::panic_handler
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library\std\src\panicking.rs:689
   1: core::panicking::panic_fmt
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library\core\src\panicking.rs:80
   2: 02_broken_invariant::region_code
             at .\phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\02-broken-invariant.rs:15
   3: 02_broken_invariant::main
             at .\phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\02-broken-invariant.rs:25
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.
```

(I trimmed one repetitive runtime-bootstrap frame off the end for readability; the rest is exactly what the terminal printed.) Frames 0 and 1 are always this same pair — the panic machinery itself. Frames 2 and 3 are the ones you actually care about: precisely which function panicked (`region_code`) and who called it (`main`, line 25). In a real program twenty calls deep, that's what points you to the actual path.

### `assert!` and `assert_eq!` — guards on a condition

`assert!(condition, "message")`, which you just saw in `checked_midpoint`, is nothing more than "panic if this condition is false." Its more common cousin is `assert_eq!`, which compares two values — with one useful difference in what it prints when it fails:

```rust
fn double(n: i32) -> i32 {
    n + n
}

assert_eq!(double(6), 11);
```

```text
thread 'main' (26600) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\03-assert-eq-mismatch.rs:19:5:
assertion `left == right` failed
  left: 12
 right: 11
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Here's the point: `assert!(double(6) == 11)` would only have said "these weren't equal" — you'd be left guessing which side was which. `assert_eq!` keeps both values and shows them side by side when it fails. `assert_ne!` also exists, for when you want to be sure two things are **different**.

### `unreachable!()` — when a branch truly must not be reached

From 1.5.4 you remember that a `match` has to cover every case. Sometimes a function lists every value its *type* allows in a `match`, but one of those arms should never actually run, because of a contract guaranteed somewhere else entirely:

```rust
fn priority_label(level: u8) -> &'static str {
    match level {
        1 | 2 => "low",
        3 => "normal",
        4 | 5 => "high",
        other => unreachable!(
            "priority_label: level {other} was never validated by parse_priority (must be 1..=5)"
        ),
    }
}
```

```text
thread 'tests::panics_when_the_caller_skips_validation' (30060) panicked at src\lib.rs:56:18:
internal error: entered unreachable code: priority_label: level 9 was never validated by parse_priority (must be 1..=5)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test tests::panics_when_the_caller_skips_validation - should panic ... ok
```

A `u8` holds thousands of values, but `priority_label` documents that it is only ever called with what `parse_priority` already validated — 1 through 5. The `other` arm has to exist because the compiler wants one arm per possible `u8` — but actually reaching it means the contract was bypassed somewhere. `unreachable!()` says exactly that: "as far as I know, this point never runs; if it did, there's a bug somewhere."

### `debug_assert!` — a check removed in release

The same `checked_midpoint` above, in the real `01-panic-or-result.rs` file, has one more line this excerpt left out:

```text
debug_assert!(
    sorted_ascending.windows(2).all(|pair| pair[0] <= pair[1]),
    "checked_midpoint: caller must pass an ascending slice"
);
```

This is the same trade you saw with overflow in 1.1.2. Checking the ordering isn't free — it means walking the whole slice. `assert!` always runs, even in `--release`, because an empty slice is a cheap, always-worth-checking rule. `debug_assert!` only **exists** in a debug build — in release it isn't skipped or silenced, it isn't compiled in at all. For expensive checks that only exist to catch bugs during development, that's exactly what you want.

### When a panic happens, `Drop` still runs

This is the surprising part of the lesson, and 1.2.5 pointed straight at it. Build three `Guard`s, then panic partway through:

```rust
struct Guard {
    name: &'static str,
}

impl Guard {
    fn new(name: &'static str) -> Guard {
        println!("open  {name}");
        Guard { name }
    }
}
```

```rust
impl Drop for Guard {
    fn drop(&mut self) {
        println!("close {}", self.name);
    }
}

fn main() {
    let _a = Guard::new("a");
    let _b = Guard::new("b");
    let _c = Guard::new("c");
    println!("all three open, about to fail partway through setup");
    panic!("simulated failure while building the fourth resource");
}
```

```text
open  a
open  b
open  c
all three open, about to fail partway through setup

thread 'main' (18256) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\06-drop-during-unwind.rs:36:5:
simulated failure while building the fourth resource
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
close c
close b
close a
```

The panic message sits in the middle of the output — and then three `close` lines, in exactly the same reversed order 1.2.5 showed for an ordinary scope exit. The panic didn't leave any of those three `Guard`s behind.

The mechanism has a name: **unwinding**. Rust's default behaviour is that a panic, instead of killing the process on the spot, walks back out of the function it happened in — like a forced `return` from every function still on the stack, one at a time — and at each step runs the destructor of anything still in scope. That's why `_a`, `_b` and `_c` got closed even though they never reached the end of `main`.

```senpai-visual
{"kind":"concept","labels":["build a","build b","build c","panic!","unwinding","drop c, b, a"]}
```

### `panic = "abort"` — opting out of unwinding

Unwinding isn't free: it has to know what to clean up at every step, and that costs time and binary size. Rust offers a second option, set in `Cargo.toml`:

```toml
[profile.release]
panic = "abort"
```

With this set, a panic no longer walks back up — the whole process stops right there. No unwinding, no `Drop`. The binary gets smaller and faster, and it's a common choice for programs that were never going to recover from a panic anyway — very constrained targets, or code called from another language like C, where unwinding can't cross that boundary safely. This workspace doesn't set it, but you can see the exact same behaviour without touching a single file, using only an environment variable:

```text
open  a
open  b
open  c
all three open, about to fail partway through setup

thread 'main' (4780) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\06-drop-during-unwind.rs:36:5:
simulated failure while building the fourth resource
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Same program, same three `Guard`s — but not one `close` line printed this time. With `CARGO_PROFILE_DEV_PANIC=abort` the process stopped exactly at the panic line, with no walk back up the stack and no destructor called. The OS still closes files and sockets on its own when a process dies; what actually gets lost is your program's *own* cleanup logic — writing a final log record, releasing an application-level lock, deleting a temp file. Which is why `panic = "abort"` isn't a free choice — it's a trade.

### `.unwrap()` and `.expect()` — "I am asserting this cannot fail"

From 1.6.1 you know `.unwrap()` panics on a `None`. Look closer at exactly what it gives you:

```rust
fn find_config_path<'a>(candidates: &[&'a str], wanted: &str) -> Option<&'a str> {
    candidates.iter().find(|&&c| c == wanted).copied()
}

let path = find_config_path(&["dev.toml", "staging.toml"], "prod.toml").unwrap();
```

```text
looking for prod.toml among ["dev.toml", "staging.toml"]

thread 'main' (24568) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\04-unwrap-panics.rs:18:59:
called `Option::unwrap()` on a `None` value
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

That message could come from a hundred different places in a codebase, and it always looks exactly the same. `.unwrap()` means "I'm asserting this is never `None`/`Err`" — and when that assertion turns out to be wrong, this generic sentence is all you get.

`.expect()` makes the same assertion, but lets you say *why* you believed it couldn't fail:

```rust
let path = find_config_path(&["dev.toml", "staging.toml"], "prod.toml")
    .expect("prod.toml must be listed in `candidates` — deploy config is incomplete");
```

```text
looking for prod.toml among ["dev.toml", "staging.toml"]

thread 'main' (5572) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\05-expect-with-a-good-message.rs:19:10:
prod.toml must be listed in `candidates` — deploy config is incomplete
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Compare the two: the first tells you there was a `None` — which you already knew, that's what `None` means. The second tells you exactly which assumption broke, and what that assumption was. The rule is one sentence: **the message should name the invariant you believed, not describe the failure.** `.expect("no path found")` is no better than `.unwrap()` — it just restates, in your own words, what the compiler already gave you for free. `.expect("prod.toml must be listed in candidates — deploy config is incomplete")` says something else entirely: it tells whoever reads this at 2am exactly where to look.

### `#[should_panic]` — a test that expects a panic

When panicking is exactly the behaviour you want, the test has to assert that too — instead of confusing a failing test with a broken program:

```rust
#[test]
#[should_panic(expected = "must not pass an empty slice")]
fn panics_on_an_empty_slice() {
    checked_midpoint(&[]);
}
```

```text
running 9 tests
test tests::finds_the_last_digit ... ok
test tests::labels_every_valid_priority ... ok
test tests::finds_the_midpoint ... ok
test tests::panics_on_an_empty_slice - should panic ... ok
test tests::panics_when_the_caller_skips_validation - should panic ... ok
test tests::panics_on_empty_values - should panic ... ok
test tests::rejects_out_of_range_priorities_without_panicking ... ok
test tests::parses_valid_priorities ... ok
test tests::rejects_unparsable_priorities_without_panicking ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`expected = "..."` only needs a substring, not the whole message — enough to prove this was the *right* panic, not some unrelated one that happened to fire at the same spot. Without `expected`, `#[should_panic]` only asks "did it panic at all?" — good enough, but less precise.

### The real boundaries: library, startup, request handler

The rule above ("can the caller reasonably handle this?") gives a different answer depending on where you're standing in a system:

| Where | What's right | Why |
|---|---|---|
| A library — a function you hand to others | Return `Result` | The decision belongs to the caller, not you; they might retry, fall back to a default, or panic themselves |
| Application startup — reading config in `main` | Panicking (with a message-carrying `.expect()`) is fine | If the config is wrong, continuing is meaningless, and there's no caller left inside the program to "handle" it |
| A request handler — a function re-run for every user | Never panic on user input | One bad input shouldn't take the whole service or loop down with it; return `Result` and respond |

File that third row away as a principle for now — [2.1 in Phase 3](../../../phase3-backend-foundations/02-axum-and-rest-api-design/01-routing-handlers-extractors/README.md) shows you exactly what it means once you're writing an HTTP handler called hundreds of times a second, where one `.unwrap()` on a user's JSON body can take the whole thread down with it.

### `catch_unwind` — you now know the name, not today

There is a fourth tool: `std::panic::catch_unwind` can catch a panic right where it happens and hand back a value instead — without bringing the whole program down. This lesson stops short of it for two reasons. First, using it correctly (keeping a thread pool alive when one of its jobs panics, say) needs `Arc`, threads, and `Send`/`Sync`, none of which you have yet. Second, even once you do, `catch_unwind` isn't a replacement for `Result` as input validation — it's built for boundaries (a thread pool, a plugin), not for the everyday decisions this lesson has been about. Just know it exists; you'll meet its name again where it actually belongs.

---

## Hands on

```sh
cargo run -p p1-06-04-panic-vs-result --example 01-panic-or-result
```

Then the five that panic:

```sh
cargo run -p p1-06-04-panic-vs-result --example 02-broken-invariant --features broken
cargo run -p p1-06-04-panic-vs-result --example 03-assert-eq-mismatch --features broken
cargo run -p p1-06-04-panic-vs-result --example 04-unwrap-panics --features broken
cargo run -p p1-06-04-panic-vs-result --example 05-expect-with-a-good-message --features broken
cargo run -p p1-06-04-panic-vs-result --example 06-drop-during-unwind --features broken
```

Then see the same plain panic with a backtrace:

```sh
RUST_BACKTRACE=1 cargo run -p p1-06-04-panic-vs-result --example 02-broken-invariant --features broken
```

And then `06-drop-during-unwind` once more, this time without unwinding:

```sh
# PowerShell
$env:CARGO_PROFILE_DEV_PANIC='abort'; cargo run -p p1-06-04-panic-vs-result --example 06-drop-during-unwind --features broken
# bash
CARGO_PROFILE_DEV_PANIC=abort cargo run -p p1-06-04-panic-vs-result --example 06-drop-during-unwind --features broken
```

Then try:

1. In `01-panic-or-result`, give `checked_midpoint` a descending slice (e.g. `&[5, 3, 1]`). Why does it panic in debug but not in `--release`?
2. In `02-broken-invariant`, add `"fr" => 33` to the table. Is that genuinely the right fix, or did it just move the panic one step further out? Write a sentence on whether this table should really be exhaustive at all.
3. In `06-drop-during-unwind`, build a fourth `Guard` and move the panic after it. How does the `close` order change?

---

## Errors you will meet

### A plain panic — a message and an exact line

```text
thread 'main' (2224) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\02-broken-invariant.rs:15:18:
region_code: no dialing code registered for "fr" — this table is supposed to cover every region this program deploys to
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**What happened:** `region_code` wanted every region in its table, and `"fr"` wasn't there. This is not a compile error — the program built completely; the problem only appeared at run time, when the table and the real input met.

**The fix:** either add `"fr"` to the table (if every region genuinely must be there), or, if the table is allowed to be incomplete by design, change the signature to `Option<u8>` and stop panicking altogether.

**Why that's the fix:** the choice between those two depends on a fact outside the code — is this table really meant to cover everything? If yes, the panic is correct and the table has the bug. If no, the panic was wrong from the start; it was never really a true invariant.

### `assert_eq!` failing — the real left/right diff

```text
thread 'main' (26600) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\03-assert-eq-mismatch.rs:19:5:
assertion `left == right` failed
  left: 12
 right: 11
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**What happened:** `assert_eq!(double(6), 11)` was written, but `double(6)` is actually 12. `left` and `right` are exactly what you handed the macro, in that order — not "correct" and "wrong."

**The fix:** work out which side is actually wrong. If `double` is supposed to turn 6 into 12 (which it does), the expectation written in `assert_eq!` was wrong — correct it to 12.

**Why that's the fix:** a failing `assert_eq!` has two suspects, not one: the function itself, or the expectation written about it. The message only says "these weren't equal" — deciding which one was right is your job, and here `double` is innocent.

### A bare `.unwrap()` versus a good `.expect()`

```text
thread 'main' (24568) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\04-unwrap-panics.rs:18:59:
called `Option::unwrap()` on a `None` value
```

**What happened:** `find_config_path` found nothing for `"prod.toml"` and returned `None`; `.unwrap()` panicked on it. The message only says there was a `None` — exactly what `Option` itself already told you.

**The fix:**

```text
.expect("prod.toml must be listed in `candidates` — deploy config is incomplete")
```

which gives this panic instead:

```text
thread 'main' (5572) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\05-expect-with-a-good-message.rs:19:10:
prod.toml must be listed in `candidates` — deploy config is incomplete
```

**Why that's the fix:** the panic still happens — this fix doesn't make the failure go away, because the actual problem (`prod.toml` missing) is somewhere else entirely. What gets fixed is the honesty of the message: whoever sees this line in a log no longer has to guess "which `None`?" — the message names, right there, exactly which assumption broke.

---

## Exercises

### Warm up

<details>
<summary>A function parses text a user typed and it might be invalid. <code>Result</code> or panic?</summary>

`Result`. User input is external to the program; being invalid is ordinary, not a bug.

</details>

<details>
<summary>An internal function assumes its argument is never empty, because the one place that calls it always fills it first. What if it arrives empty anyway?</summary>

Panic — with `assert!` or `.expect()`. Arriving empty here means some other part of this same program already broke its promise; that's a bug, not bad input.

</details>

<details>
<summary>A <code>debug_assert!</code> with a false condition sits in the code. Does <code>cargo run</code> panic? Does <code>cargo run --release</code>?</summary>

Yes in debug. No in `--release` — because `debug_assert!` isn't skipped or silenced there, it isn't compiled in at all.

</details>

<details>
<summary>What does <code>.expect("some message")</code> print on a <code>Some(x)</code>?</summary>

Nothing, and it doesn't panic either. `.expect()` only fires when the value is genuinely absent — on `Some`/`Ok` it just hands back the value inside.

</details>

<details>
<summary>Three <code>Guard</code>s named <code>a</code>, <code>b</code>, <code>c</code> open, then you panic. What order do the <code>close</code>s print in?</summary>

`c`, then `b`, then `a` — the same reversed order 1.2.5 always showed, because unwinding follows the exact same rule.

</details>

<details>
<summary>True or false: <code>catch_unwind</code> is the right way to handle invalid user input.</summary>

False. `catch_unwind` is built for boundaries (a thread pool, a plugin), not everyday validation. For user input, the answer is still `Result`.

</details>

### Repair

Fix `examples/03-assert-eq-mismatch.rs` — but first work out which side is actually wrong: `double` itself, or the number written in `assert_eq!`? (Write one sentence on how you knew.)

Fix `examples/04-unwrap-panics.rs` so that, if it panics at all, the message actually helps — exactly what you saw in `05-expect-with-a-good-message.rs`. Then write a second version that doesn't panic at all: use `.unwrap_or(...)` and fall back to a default path. Which one is really right for this function, and does the answer depend on who calls `find_config_path`?

Fix `examples/02-broken-invariant.rs` two ways: once by completing the table, once by changing the signature to `Option<u8>` so it never panics at all. Which one you'd actually pick depends on whether this table is really meant to cover everything — and that isn't something the code itself can tell you.

### Implement

Four functions in `src/lib.rs`:

```sh
cargo test -p p1-06-04-panic-vs-result
```

Two of them (`parse_priority`, `checked_midpoint`) you already know from above. The other two are new: `priority_label` should `unreachable!()` exactly like the example above, and `last_digit_of` should `.expect()` with a message that names the invariant, not the failure.

### Build

Write a `pub fn` (name and signature your choice) that reads a required setting out of a `HashMap<String, String>` — like reading an environment variable at startup. If the key is missing, panic with `.expect()` and a message naming exactly which key and why it's required.

Then write a second function beside it that parses a user-submitted field (an email, a phone number — your choice) and returns `Result`.

Then write one sentence: why are these two, despite both being "look up some text that might not be there," fundamentally different in kind?

### Challenge (optional)

**Part one.** Read the [`std::panic::catch_unwind`](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html) documentation. Wrap a call to `checked_midpoint(&[])` in it and print what comes back. This reaches forward — real usage only makes sense once you've seen threads in [Phase 2](../../../phase2-intermediate/07-concurrency-and-async/01-threads-mutex-arc/README.md), but you can see the shape of what it returns today.

**Part two.** Try `06-drop-during-unwind` with `CARGO_PROFILE_DEV_PANIC=abort` on `--release` too. Does anything differ from the aborted debug build?

**Part three.** Write a `#[should_panic(expected = "...")]` test for `priority_label(0)` without looking at `unreachable!()`'s exact message — guess from what you wrote yourself, then run it and see if you guessed right.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `panic!` | immediately stops the current thread with a message | a rule this program itself broke |
| `assert!` / `assert_eq!` / `assert_ne!` | panic if a condition/equality/inequality doesn't hold | guarding a precondition |
| `unreachable!()` | panic on a branch that a contract says can never run | a `match` exhaustive by type but not by meaning |
| `debug_assert!` | like `assert!`, but not compiled in at all in release | expensive checks, development only |
| unwinding | walking back out of the stack on a panic, running every `Drop` on the way | Rust's default panic behaviour |
| `panic = "abort"` | the process stops on the spot; no unwinding, no `Drop` | smaller binary, no cleanup |
| `RUST_BACKTRACE=1` | shows the call path that led to the panic | finding where the fault really is |
| `.expect("...")` | like `.unwrap()`, but the message names the broken promise | always over `.unwrap()` in real code |
| `#[should_panic]` | a test that only passes if it panics | when panicking is itself the correct behaviour |
| `catch_unwind` | catches a panic at a boundary instead of letting it crash everything | thread pools, plugins — not everyday validation |

### What you now know

- `Result` is for a failure the caller can reasonably handle; a panic is for a broken rule inside the program itself.
- `assert!`/`assert_eq!`/`unreachable!()` guard preconditions and impossible branches; `debug_assert!` does the same but is absent from release builds.
- A panic unwinds the stack by default and runs every `Drop` it passes — the same reversed order as 1.2.5.
- `panic = "abort"` skips that unwinding; the process stops immediately, with no `Drop`.
- `.expect()` should name the promise you believed was true, not describe the failure.
- `#[should_panic(expected = "...")]` builds a test that tells the right panic apart from any other.
- A library returns `Result`, application startup may panic, a request handler must never panic on user input.

### What comes back later

- **Automatic error conversion with `From` and `?`** — [1.6.5 — `From` and error conversion](../05-from-and-error-conversion/README.md)
- **A real error type of your own, instead of `String`** — [Phase 2 — Custom error types](../../../phase2-intermediate/04-error-handling-and-lifetimes/01-custom-error-types/README.md)
- **`thiserror` and `anyhow`, the real industry tools for what we just did with `String`** — [Phase 2 — `thiserror` and `anyhow`](../../../phase2-intermediate/04-error-handling-and-lifetimes/02-thiserror-and-anyhow/README.md)
- **A panic in a separate thread, and why it poisons a `Mutex`** — [Phase 2 — Threads, `Mutex` and `Arc`](../../../phase2-intermediate/07-concurrency-and-async/01-threads-mutex-arc/README.md)
- **An HTTP handler that must never panic on user input** — [Phase 3 — Routing and handlers with Axum](../../../phase3-backend-foundations/02-axum-and-rest-api-design/01-routing-handlers-extractors/README.md)
- **Consistent error formatting for an API response** — [Phase 3 — Consistent error envelopes](../../../phase3-backend-foundations/07-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)

### Can you explain?

- State the decision rule between `Result` and panic in one sentence.
- What's the difference between `assert!` and `debug_assert!`, and which one survives `--release`?
- When a panic happens, what exactly happens to values still in scope?
- What does `panic = "abort"` change?
- How do you tell a bad `.expect()` message from a good one?
- Why must a request handler never panic on user input, even though the exact same code panicking in `main` was fine?

---

## Going further

- [The Rust Book — Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html) — the same ground, officially, with a full chapter on when to panic and when to return `Result`.
- [`std::panic`](https://doc.rust-lang.org/std/panic/index.html) — the module documentation, where `catch_unwind` and `PanicHookInfo` live.
- [`clippy::unwrap_used` and `clippy::expect_used`](https://rust-lang.github.io/rust-clippy/master/#unwrap_used) — restriction lints you turn on in library/backend code so a forgotten `.unwrap()` doesn't even compile.
