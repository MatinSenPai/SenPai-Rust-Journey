# 2.7.4 — Property testing with `proptest`, snapshot testing with `insta`

## At a glance

After this lesson you can:

- Explain what a "property" is — a rule that must hold for *every* input, not just the handful you picked — and write a genuine `proptest!` block that checks one.
- Read a real, shrunk proptest failure and say exactly which minimal input breaks the property, and why.
- Capture a complex output as an insta snapshot, and correctly choose accept vs. reject when a later run's diff shows up.

**Time:** ~65 minutes · **Prerequisites:**
[2.7.3 — Test doubles in Rust](../03-test-doubles-in-rust/README.md)

---

## Why this matters

Every test you've written so far (2.7.1 through 2.7.3) had the same shape: you picked a handful of specific inputs, worked out the right answer for each by hand, and checked them with `assert_eq!`. That's completely fine for most code — and stays fine after this lesson too.

But it has a structural weak spot: you're the one choosing the inputs, and a bug loves exactly the input you didn't think to try. Test a round-trip function — something you format, then parse back; or encode, then decode — with three small positive numbers, and it goes green. Never tried a negative one? The bug lives right there, quietly, because your test never looked.

The second problem is unrelated: some outputs just aren't small. A multi-line report, a nested struct whose `Debug` output fills the screen — hand-writing `assert_eq!(output, "...")` for something like that is both tedious and fragile. Every small formatting tweak means rewriting that string by hand. Nobody actually does that; the test either gets deleted or updated blindly.

This lesson brings two new tools, one for each problem — **additions** to 2.7.2, not replacements for it:

- **proptest**: instead of a handful of hand-picked inputs, you describe a property — a rule that must hold for every input — and the library generates hundreds of inputs, including odd edge cases, trying to break it.
- **insta**: you capture a complex output once, review it yourself as the source of truth, and from then on every run diffs against that saved snapshot — failing loudly if anything changed without review.

Neither one replaces a plain test. For most code, a `#[test]` with a precise `assert_eq!` is exactly what you need — faster to write, faster to read. These two exist for those two specific situations: a genuine invariant that must hold across a wide space of inputs, and an output that's correct but too big or complex for a hand-written assert.

---

## The concept

### A contract that must hold for every input

Take this type — coordinates that can turn into a string and back:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn format(&self) -> String {
        format!("{},{}", self.x, self.y)
    }
}
```

```rust
impl Point {
    fn parse(s: &str) -> Option<Point> {
        let (x_str, y_str) = s.split_once(',')?;
        let x = x_str.parse().ok()?;
        let y = y_str.parse().ok()?;
        Some(Point { x, y })
    }
}
```

Now write a test exactly the way 2.7.2 taught you to:

```rust
#[test]
fn round_trips_a_hand_picked_point() {
    let p = Point { x: 3, y: 4 };
    assert_eq!(Point::parse(&p.format()), Some(p));
}
```

```text
running 1 test
test tests::round_trips_a_hand_picked_point ... ok
```

Green. But what did it actually prove? Only that the round trip works for `(3, 4)`. Billions of other possible `(x, y)` pairs — negatives, zero, `i32::MIN`, everything else — were never tried.

The real contract you want is: **for every `x` and every `y`, `parse(format(p))` must give back exactly `p`.** That's a **property** — not an example, a rule over the *entire* input space. That's exactly what proptest takes and tries to break:

```rust
proptest! {
    #[test]
    fn round_trip(x in any::<i32>(), y in any::<i32>()) {
        let original = Point { x, y };
        let parsed = Point::parse(&original.format());
        prop_assert_eq!(parsed, Some(original));
    }
}
```

```sh
cargo test -p p2-07-04-property-and-snapshot-testing --example 01-point-round-trip
```

```text
running 2 tests
test tests::round_trips_a_hand_picked_point ... ok
test tests::round_trip ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Both are green, but they did different work. `round_trip` is a function that takes `x` and `y` as parameters; `any::<i32>()` is a **strategy** — a description of where proptest should draw values from, here "any possible `i32`." By default proptest calls this function 256 times, each time with a fresh, random `x`/`y` — not once, like the hand-written test. `prop_assert_eq!` behaves like `assert_eq!`, except inside `proptest!` a failure also gets reported to proptest's shrinking engine — which is exactly what the next section shows.

### When the contract genuinely breaks: shrinking

Give `format` a real bug:

```rust
fn format(&self) -> String {
    // BUG: unsigned_abs() throws away the sign of `x`.
    format!("{},{}", self.x.unsigned_abs(), self.y)
}
```

```sh
cargo run -p p2-07-04-property-and-snapshot-testing --example 03-broken-sign-drop --features broken
```

```text
Point { x: -1, y: 0 } -> "1,0" -> Some(Point { x: 1, y: 0 }) — the sign of x is just gone
```

Now run the same `proptest!` against this version:

```sh
cargo test -p p2-07-04-property-and-snapshot-testing --example 03-broken-sign-drop --features broken
```

```text
running 1 test
test tests::round_trip ... FAILED

failures:

---- tests::round_trip stdout ----

thread 'tests::round_trip' (28720) panicked at phase2-intermediate\07-project-structure-and-testing\04-property-and-snapshot-testing\examples\03-broken-sign-drop.rs:38:5:
Test failed: assertion failed: `(left == right)` 
  left: `Some(Point { x: 1, y: 0 })`,
 right: `Some(Point { x: -1, y: 0 })` at phase2-intermediate\07-project-structure-and-testing\04-property-and-snapshot-testing\examples\03-broken-sign-drop.rs:47.
minimal failing input: x = -1, y = 0
	successes: 0
	local rejects: 0
	global rejects: 0

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::round_trip

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

proptest broke on roughly 50% of the `i32` space (every negative number), so it found a failing case almost immediately; the exact `successes` count above differs every time you run it, because the search is random. The interesting part happens next. proptest doesn't keep that original failing value (maybe some big, unreadable number like `-1992831647`). It shrinks it — over and over, cutting it toward zero — retrying each time: "does this smaller one still break the property?" — until it can't shrink any further. The result is always the same here: `x = -1, y = 0`. Not because that was the first thing tried, but because it's the **smallest** input that still shows the bug.

```senpai-visual
{"kind":"concept","labels":["generate many random inputs","one input fails","shrink toward the simplest case","still fails: shrink again","minimal failing case reported"]}
```

That's exactly what separates proptest from "just try one random input": it never leaves you staring at one big, messy failure. It always hands you the smallest, most readable case that still reproduces the same bug — exactly what you need to actually fix it.

(By default, proptest also saves this failing input to a file next to the source, so the next run tries it first — a genuinely useful feature. The example above turns that off on purpose, since this file would otherwise be regenerated every time you run this lesson's demo, rather than something you'd want checked into the lesson itself.)

`any::<i32>()` isn't the only strategy. proptest has one for collections too (`prop::collection::vec(any::<bool>(), 0..16)` builds a `Vec<bool>` between 0 and 15 items long), for tuples, and for your own types via `.prop_map(...)`. You'll meet one of these in the exercises below, behind an already-written test — you don't need to write it yourself, just know it's there.

### An output not worth hand-writing an assert for

Now the second problem. This function builds a multi-line report:

```rust
fn watch_digest(entries: &[Entry]) -> String {
    let mut out = String::new();
    for e in entries {
        let rating = match e.rating {
            Some(r) => format!("{r}/10"),
            None => "unrated".to_string(),
        };
        out.push_str(&format!(
            "{} - {} episodes - {}\n",
            e.title, e.episodes_watched, rating
        ));
    }
    out
}
```

You could hand-write its result into an `assert_eq!`, but this isn't a three-character string — real reports run to dozens of lines, and every new field you add means rewriting that string by hand again. insta does this for you: it captures the output once, you confirm it's correct as the source of truth, and from then on insta only diffs:

```rust
#[test]
fn digest_snapshot() {
    insta::assert_snapshot!(watch_digest(&sample_entries()));
}
```

The very first time this test runs, there's no snapshot to compare against — insta fails, on purpose, and writes a "pending" file:

```text
running 1 test
stored new snapshot M:\SenPai-Rust-Journey\phase2-intermediate\07-project-structure-and-testing\04-property-and-snapshot-testing\examples\snapshots\02_watch_digest_snapshot__tests__digest_snapshot.snap.new
test tests::digest_snapshot ... FAILED

failures:

---- tests::digest_snapshot stdout ----
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ Snapshot Summary ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Snapshot file: phase2-intermediate\07-project-structure-and-testing\04-property-and-snapshot-testing\examples\snapshots\02_watch_digest_snapshot__tests__digest_snapshot.snap
Snapshot: digest_snapshot
Source: M:\SenPai-Rust-Journey:55
───────────────────────────────────────────────────────────────────────────────
Expression: watch_digest(&sample_entries())
───────────────────────────────────────────────────────────────────────────────
+new results
────────────┬──────────────────────────────────────────────────────────────────
          1 │+Frieren - 12 episodes - 9/10
          2 │+Bocchi the Rock! - 12 episodes - 10/10
          3 │+Made in Abyss - 3 episodes - unrated
────────────┴──────────────────────────────────────────────────────────────────
To update snapshots run `cargo insta review`
Stopped on the first failure. Run `cargo insta test` to run all snapshots.

thread 'tests::digest_snapshot' (14796) panicked at C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\insta-1.48.0\src\runtime.rs:719:13:
snapshot assertion for 'digest_snapshot' failed in line 55
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::digest_snapshot

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
```

(The full path next to `Source:` will look different on your own computer — it's always just wherever you ran this command from.) Notice: this isn't a compiler error, it's a *deliberate* test-time failure. insta wrote the actual content to a file right next to it, with a `.snap.new` extension:

```text
---
source: phase2-intermediate/07-project-structure-and-testing/04-property-and-snapshot-testing/examples/02-watch-digest-snapshot.rs
assertion_line: 55
expression: watch_digest(&sample_entries())
---
Frieren - 12 episodes - 9/10
Bocchi the Rock! - 12 episodes - 10/10
Made in Abyss - 3 episodes - unrated
```

This is exactly what "capture it, then review it" means: insta never decides on its own whether this output is correct. It only says "here's what got produced; you look at it."

### Review, accept, reject: insta's workflow

After looking at the diff above, you (the author) decide this output is correct. There are two ways to **accept** it: rename `.snap.new` to `.snap` by hand, or run the real command — which, if `cargo-insta` is installed (`cargo install cargo-insta`), does that same rename for you and also strips the `assertion_line` field, since line numbers drift but the text itself doesn't:

```sh
cargo insta accept
```

```text
insta review finished
accepted:
  phase2-intermediate/07-project-structure-and-testing/04-property-and-snapshot-testing/examples/02-watch-digest-snapshot.rs (digest_snapshot.snap)
```

From this point on, `.snap` is the source of truth. Every future `cargo test` run is just a string comparison, with no panic — until something actually changes:

```senpai-visual
{"kind":"concept","labels":["first run: no snapshot yet","insta fails, writes a pending snapshot","you review the diff by hand","accept: it becomes the truth","reject: the pending one is discarded"]}
```

Now say a teammate, without telling anyone, tweaks the report's wording:

```rust
// Someone shortened "episodes" to "eps" here without telling anyone.
// The committed snapshot still says "episodes".
out.push_str(&format!(
    "{} - {} eps - {}\n",
    e.title, e.episodes_watched, rating
));
```

```sh
cargo test -p p2-07-04-property-and-snapshot-testing --example 04-broken-stale-snapshot --features broken
```

This is the middle of the output — the diff itself. Its complete transcript, panic message and all, is in "Errors you will meet":

```text
-old snapshot
+new results
────────────┬──────────────────────────────────────────────────────────────────
    1       │-Frieren - 12 episodes - 9/10
    2       │-Bocchi the Rock! - 12 episodes - 10/10
    3       │-Made in Abyss - 3 episodes - unrated
          1 │+Frieren - 12 eps - 9/10
          2 │+Bocchi the Rock! - 12 eps - 10/10
          3 │+Made in Abyss - 3 eps - unrated
────────────┴──────────────────────────────────────────────────────────────────
To update snapshots run `cargo insta review`
```

insta never knows whether that change was intentional or a bug — it only knows the output differs from what you last approved, and that's exactly why it fails loudly. The decision is yours: if "eps" really was the change you wanted, `cargo insta accept` makes it the new `.snap`; if it was a mistake, `cargo insta reject` (or just deleting the `.snap.new` file) throws the change away and you revert the code. This exact same mechanism, with no difference at all, is what catches a genuine regression too — insta doesn't distinguish "intentional" from "bug," it only shows you the difference.

### When each, and when neither

The practical rule is short:

- You have a **genuine invariant** that must hold across a wide space of inputs (a round trip, an ordering, a total, a bound) → proptest.
- The output is **correct but big or complex** — a report, a nested struct, some type's `Debug` output — and hand-writing an assert for it costs more time than it saves → insta.
- Everything else — most code — is the same `#[test]` with an `assert_eq!` that 2.7.2 taught you. These two tools were added; nothing was replaced.

Picking the wrong tool has a real cost too: a proptest property on a function with no meaningful invariant just slows you down without finding anything new; a snapshot of a three-word output just adds an unnecessary layer between you and a plain `assert_eq!`.

---

## Hands on

```sh
cargo run -p p2-07-04-property-and-snapshot-testing --example 01-point-round-trip
cargo test -p p2-07-04-property-and-snapshot-testing --example 01-point-round-trip
cargo run -p p2-07-04-property-and-snapshot-testing --example 02-watch-digest-snapshot
cargo test -p p2-07-04-property-and-snapshot-testing --example 02-watch-digest-snapshot
```

Then the two broken ones:

```sh
cargo test -p p2-07-04-property-and-snapshot-testing --example 03-broken-sign-drop --features broken
cargo test -p p2-07-04-property-and-snapshot-testing --example 04-broken-stale-snapshot --features broken
```

Then try:

1. In `02-watch-digest-snapshot`, delete `examples/snapshots/02_watch_digest_snapshot__tests__digest_snapshot.snap` and run `cargo test --example 02-watch-digest-snapshot` again. Do you see the same "first run" failure from above? Now accept the `.snap.new` yourself by renaming it, and run it again.
2. In `01-point-round-trip`, add a third line to `proptest!`: `#![proptest_config(ProptestConfig { cases: 2000, ..ProptestConfig::default() })]` (above `fn round_trip` itself). How much does the run time change?
3. In `03-broken-sign-drop`, run that same test command two or three times in a row. Which part of the output changes each time, and which part always stays the same? Why?

---

## Errors you will meet

### A proptest failure — the minimal counterexample

```text
running 1 test
test tests::round_trip ... FAILED

failures:

---- tests::round_trip stdout ----

thread 'tests::round_trip' (28720) panicked at phase2-intermediate\07-project-structure-and-testing\04-property-and-snapshot-testing\examples\03-broken-sign-drop.rs:38:5:
Test failed: assertion failed: `(left == right)` 
  left: `Some(Point { x: 1, y: 0 })`,
 right: `Some(Point { x: -1, y: 0 })` at phase2-intermediate\07-project-structure-and-testing\04-property-and-snapshot-testing\examples\03-broken-sign-drop.rs:47.
minimal failing input: x = -1, y = 0
	successes: 0
	local rejects: 0
	global rejects: 0

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::round_trip

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**What the compiler is objecting to:** this isn't even a compiler error — the program compiled, proptest tried hundreds of `x`/`y` pairs, and on this one (after shrinking: `x = -1, y = 0`), `prop_assert_eq!` failed. `left` is what the code actually returned; `right` is what was expected.

**The fix:** drop `.unsigned_abs()` — the sign of `x` needs to stay in the formatted text:

```rust
fn format(&self) -> String {
    format!("{},{}", self.x, self.y)
}
```

**Why that's the fix:** with this version, `format` never loses information — every `Point` has exactly one string representation, and `parse` gives that exact one back, for every `x` and `y`, not just the positive ones.

### A run-time panic — insta snapshot mismatch

```text
running 1 test
stored new snapshot M:\SenPai-Rust-Journey\phase2-intermediate\07-project-structure-and-testing\04-property-and-snapshot-testing\examples\snapshots\04_broken_stale_snapshot__tests__digest_snapshot.snap.new
test tests::digest_snapshot ... FAILED

failures:

---- tests::digest_snapshot stdout ----
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ Snapshot Summary ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Snapshot file: phase2-intermediate\07-project-structure-and-testing\04-property-and-snapshot-testing\examples\snapshots\04_broken_stale_snapshot__tests__digest_snapshot.snap
Snapshot: digest_snapshot
Source: M:\SenPai-Rust-Journey:60
───────────────────────────────────────────────────────────────────────────────
Expression: watch_digest(&sample_entries())
───────────────────────────────────────────────────────────────────────────────
-old snapshot
+new results
────────────┬──────────────────────────────────────────────────────────────────
    1       │-Frieren - 12 episodes - 9/10
    2       │-Bocchi the Rock! - 12 episodes - 10/10
    3       │-Made in Abyss - 3 episodes - unrated
          1 │+Frieren - 12 eps - 9/10
          2 │+Bocchi the Rock! - 12 eps - 10/10
          3 │+Made in Abyss - 3 eps - unrated
────────────┴──────────────────────────────────────────────────────────────────
To update snapshots run `cargo insta review`
Stopped on the first failure. Run `cargo insta test` to run all snapshots.

thread 'tests::digest_snapshot' (20648) panicked at C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\insta-1.48.0\src\runtime.rs:719:13:
snapshot assertion for 'digest_snapshot' failed in line 60
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::digest_snapshot

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
```

**What the compiler is objecting to:** again, not a compiler error. `watch_digest` ran fine and produced a perfectly valid output; it just doesn't match the `.snap` you already approved. insta reports that exactly the way an `assert_eq!` would — it fails, it doesn't stay quiet.

**The fix:** per the product spec, "eps" was a mistake; revert the code so it matches the existing `.snap` again:

```rust
out.push_str(&format!(
    "{} - {} episodes - {}\n",
    e.title, e.episodes_watched, rating
));
```

**Why that's the fix:** if "eps" really had been the intended change, the right fix would have been refreshing `.snap` with `cargo insta accept`, not reverting the code. Here it's the other way around: the code went wrong, and the already-approved `.snap` is exactly what should stay — that's the entire point of having a committed source of truth.

---

## Exercises

### Warm up

<details>
<summary>The transcript above says "minimal failing input: x = -1, y = 0" and below it "successes: 0". What does that number mean?</summary>

It means no other random input succeeded before the failing one was found — proptest found a negative case almost immediately (among the very first inputs it tried). Run it again and this number usually differs; the `minimal failing input` usually stays the same, though, because shrinking is what decides it, not the initial search.

</details>

<details>
<summary>True or false: <code>.snap</code> files should be committed to git, but <code>.snap.new</code> files should not.</summary>

True. `.snap` is the approved source of truth — the one every other developer needs too. `.snap.new` is just a pending proposal, specific to that one moment on your own machine; it shouldn't be committed.

</details>

<details>
<summary>You have a function with five green hand-written tests from 2.7.2. You add a proptest test for the same function. Should you delete those five?</summary>

No. Those five are specific, readable examples, each one a named, permanent check on its own value — `cargo test`'s output shows exactly which one passed or broke. A passing proptest run never shows you which 256 values it tried; only a failure ever surfaces one specific input. The five hand-picked tests are what let you see, by name, that a particular known case still works after a change. proptest is additive, not a replacement.

</details>

<details>
<summary>Does this compile? If so, does it pass or fail?</summary>

```rust
proptest! {
    #[test]
    fn trivial(x in any::<i32>()) {
        prop_assert_eq!(x, x);
    }
}
```

</details>

<details>
<summary>Answer</summary>

It compiles and it passes — across all 256 random cases. `x == x` is always true for any `i32` (reflexivity, from 2.3.4). This is itself worth noticing: proptest is only as good as the property you give it — a no-op property like this one, no matter how many cases it tries, will never find a single bug.

</details>

### Repair

Fix both broken examples:

1. Fix `examples/03-broken-sign-drop.rs` so the round-trip property holds for **every** `i32`, not just the positive ones.
2. Fix `examples/04-broken-stale-snapshot.rs`. Per the product spec, "eps" was an unwanted change — revert the code so it matches the existing `.snap` again, without touching the `.snap` itself. (If it had been the other way around — "eps" was intentional — which command would you run to refresh `.snap` instead?)

### Implement

Three functions/types in `src/lib.rs`:

```sh
cargo test -p p2-07-04-property-and-snapshot-testing
```

Implement `encode_flags`/`decode_flags` exactly to the doc comment above each — their hidden test is a round-trip property (proptest), not one specific input. Implement `checklist` to its doc comment too — its hidden test is an already-approved snapshot in `src/snapshots/`; your output format has to match exactly what the doc comment specifies.

### Build

Write one small function from a domain of your choosing, and decide which tool actually fits it — a round-trip invariant (proptest), or an output too complex for a hand-written assert (insta). Pick only one of the two, the way this lesson described what each is for, and write one genuine test of that kind for it.

### Challenge (optional)

**Part one.** On the `flags_round_trip` test (in `src/lib.rs`), change the maximum `Vec<bool>` length from `0..16` to `0..500`, and add a `cases: 5000` field to the `ProptestConfig` that's already there (alongside the `failure_persistence: None` it already has). Time the run before and after. Does anything break?

**Part two.** (This one looks ahead.) proptest hunts for the smallest input that produces the *wrong* answer. A completely different question is: once you've picked one representative input, how *fast* does your code handle it — measured rigorously, not just eyeballed with a stopwatch? That question belongs to [2.7.5 — Benchmarking with `criterion`](../05-benchmarking-with-criterion/README.md).

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Property | A rule that must hold for every input, not just a few examples | A genuine invariant — round trip, ordering, total |
| `proptest!` | A macro turning a parameterized function into a test | Writing the property itself |
| Strategy | A description of where proptest draws values from, like `any::<i32>()` | Defining the space of test inputs |
| Shrinking | Automatically reducing a failing input to the smallest still-failing case | Making a failure easy to read |
| Snapshot | A recorded output every future run compares against | An output that's correct but too big/complex for a hand-written assert |
| `.snap` / `.snap.new` | The approved snapshot / a pending, unreviewed one | The first is committed, the second isn't |
| `cargo insta accept` / `reject` | Approving or discarding a pending snapshot | Replaces renaming the file by hand |

### What you now know

- A property is a rule that must hold for *every* input, not just a few you picked; `proptest!` runs that same test function hundreds of times, with fresh random inputs each time.
- When proptest finds a failure, it shrinks it down to the smallest still-failing case — you always see a readable example, never a big random number.
- insta captures a complex output once and stores it as the source of truth (`.snap`); every later run diffs against it.
- A pending snapshot (`.snap.new`) never automatically replaces the approved one — you have to decide: accept (`cargo insta accept`) or reject (`cargo insta reject`).
- Both tools are additions to 2.7.2's toolbox, not replacements for it; most code is still best tested with a `#[test]` and an `assert_eq!`.

### What comes back later

- **Measuring performance, not just correctness** — [2.7.5 — Benchmarking with `criterion`](../05-benchmarking-with-criterion/README.md)

### Can you explain?

- What's the difference between an "example" and a "property"? Give a real property for a function you've written recently.
- What does proptest do when it finds a failure that a hand-written test never does?
- Why does the very first run of `insta::assert_snapshot!` fail — even when the code has no bug at all?
- What's the difference between `.snap` and `.snap.new`, and which one gets committed?
- A teammate says "why did this test fail? I didn't even touch the main logic!" — what does insta see that they didn't?
- Which is the better fit, proptest or insta, for each of these: (a) a function that adds two numbers, (b) a Base64 decoding function, (c) a function that builds a multi-line HTML fragment?

---

## Going further

- [proptest documentation](https://docs.rs/proptest/latest/proptest/) — the full list of strategies, including `prop_oneof!` for enums and `.prop_map()`/`.prop_flat_map()` for your own types.
- [The proptest book](https://proptest-rs.github.io/proptest/intro.html) — why shrinking works, and how to write a custom strategy.
- [insta documentation](https://insta.rs/docs/) — the full `cargo insta review` workflow, format settings (YAML/JSON/Debug), and CI integration.
- [`cargo-fuzz`](https://rust-fuzz.github.io/book/) — a neighboring idea, not the same one: instead of checking one specific property, it just hunts for any input that panics or crashes the program. Not covered by this course; start here if you're curious.
