# 1.5.4 — `match` in depth

## At a glance

After this lesson you can:

- Read an `E0004` and say which variant the compiler found missing.
- Write a `match` that binds a variant's data in the same step that recognises it.
- Choose between a named arm, a guard, and `_` — and say what `_` takes away from you.
- Add a variant to an enum and get the complete list of places that must change, from the compiler.

**Time:** ~60 minutes · **Prerequisites:**
[1.5.3 — Enums as data](../03-enums-as-data/README.md) ·
[1.1.3 — Compound types and destructuring](../../01-foundations/03-compound-types-and-destructuring/README.md)

---

## Why this matters

The last lesson did half the job: with an enum, your *data* can no longer be wrong. A `Completed` carries a rating, a `Dropped` doesn't, and a value that is both "currently reading" and "rated" is not expressible at all.

But correct data isn't enough. The code that reads that data can still forget a case.

You've seen this in Python a hundred times:

```python
if status == "reading":
    ...
elif status == "finished":
    ...
else:
    pass          # and this "else" is where the bug lives
```

Tomorrow a `paused` status is added. This function compiles — there is nothing to compile — the tests stay green, and six weeks later someone reports that paused series don't show up on the dashboard at all. Your tool for finding every place that should have changed is `grep`.

Rust's `match` is not `if/elif` with nicer syntax. The difference is one word: **exhaustiveness**. The compiler proves that every possible case has an arm, and if one doesn't, the program is not built. Which means adding a variant turns every place that must change into a compile error — not into a ticket, later.

> Python 3.10 and later has `match` too, and with `enum.Enum` plus `typing.assert_never` you can build something like this check. But there it's opt-in, it happens at type-check time, and only if you actually run `mypy`. Here it is the compiler, and there's no switch to turn it off.

This lesson plus the last one is what people mean when they say "in Rust, invalid states are unrepresentable". Half of it was the enum. The other half is here.

---

## The concept

### `match` is an expression

The first thing to settle: `match` is not a statement, it is an **expression**. It produces a value, and that value can go anywhere a value goes — exactly like `if` in [1.1.4](../../01-foundations/04-functions-and-expressions/README.md).

```rust
let mood = match 3 {
    0 => "nothing on the shelf",
    1..=5 => "a manageable pile",
    _ => "too many open tabs",
};
```

```text
mood:  a manageable pile
```

That trailing `;` belongs to the `let`, not to the `match`. And because it's an expression it composes — here it is inside a `format!`, with no temporary variable in between:

```rust
println!(
    "check: {}",
    match band(7).len() {
        0 => "empty",
        1..=4 => "short",
        _ => "long enough",
    }
);
```

```text
check: short
```

One important rule falls straight out of that: **every arm must produce a value of the same type.** If one gives a `String` and another a `&str`, the program doesn't build — the same "both branches of an `if` agree" rule from 1.1.5, with more branches. The error is `E0308` and it's in the errors section.

### Every arm is a pattern

The left of each `=>` is a **pattern** — the same thing you wrote on the left of `let` in [1.1.3](../../01-foundations/03-compound-types-and-destructuring/README.md). That lesson said "this is the same machinery behind `match`, with a more interesting shape". Here are the shapes:

```rust
fn band(stars: u8) -> String {
    match stars {
        0 => "unrated".to_string(),
        1..=3 => "weak".to_string(),
        4..=6 => "watchable".to_string(),
        7 | 8 => "good".to_string(),
        9 | 10 => "top shelf".to_string(),
        _ => "not a score".to_string(),
    }
}
```

```text
  0  -> unrated
  2  -> weak
  5  -> watchable
  8  -> good
 10  -> top shelf
200  -> not a score
```

Four shapes in six lines:

| Pattern | Its name | Means |
|---|---|---|
| `0` | literal | exactly this value |
| `1..=3` | range pattern | 1 through 3, both ends included |
| `7 \| 8` | alternative | this or that, in one arm |
| `_` | wildcard | anything else, and I want no name for it |

`..=` is an inclusive range — the same `..=` you used in loops in [1.1.5](../../01-foundations/05-control-flow/README.md). And that `_` here means precisely "11 through 255", because those are the `u8` values left over.

### The compiler counts the cases for you

Now the same thing over a four-variant enum, with one arm missing:

```rust
match progress {
    Progress::NotStarted => "not started".to_string(),
    Progress::Reading { chapter } => format!("chapter {chapter}"),
    Progress::Finished { rating } => format!("finished, {rating}/10"),
}
```

```text
error[E0004]: non-exhaustive patterns: `&Progress::Dropped { .. }` not covered
  --> examples\05-a-missing-arm.rs:15:11
   |
15 |     match progress {
   |           ^^^^^^^^ pattern `&Progress::Dropped { .. }` not covered
   |
note: `Progress` defined here
  --> examples\05-a-missing-arm.rs:7:6
   |
 7 | enum Progress {
   |      ^^^^^^^^
...
11 |     Dropped { at: u32 },
   |     ------- not covered
   = note: the matched value is of type `&Progress`
```

This is the most important error in the lesson, so read it line by line:

- `non-exhaustive patterns` — "your patterns don't cover everything". That's the official name of the thing we're after.
- `` `&Progress::Dropped { .. }` not covered `` — and it says **exactly which one**. You don't compare the enum against the match yourself; the compiler already did.
- ``note: `Progress` defined here`` with an arrow at line 11 — it shows you the missing variant's definition too. On a twelve-variant enum that's the difference between ten seconds and ten minutes.
- ``the matched value is of type `&Progress` `` — a reminder that you're matching on a reference, not on the value itself. We come back to that shortly.

The point: the compiler did not **run** the program to work this out. It compared the number of variants against the coverage of the arms and did a small proof. That's not possible in Python, where `status` could be any string at all and "every case" has no number.

### What exhaustiveness is for

Say three months later the product wants a series to be pausable. You add a variant — and change nothing else:

```rust
enum Progress {
    NotStarted,
    Reading { chapter: u32 },
    Finished { rating: u8 },
    Dropped { at: u32 },
    Paused { at: u32 },
}
```

```text
error[E0004]: non-exhaustive patterns: `&Progress::Paused { .. }` not covered
  --> examples\06-a-new-variant.rs:19:11
error[E0004]: non-exhaustive patterns: `&Progress::Paused { .. }` not covered
  --> examples\06-a-new-variant.rs:28:11
error[E0004]: non-exhaustive patterns: `&Progress::Paused { .. }` not covered
  --> examples\06-a-new-variant.rs:37:11
error: could not compile `p1-05-04-match-in-depth` (example "06-a-new-variant") due to 3 previous errors
```

That's each error's opening line plus the compiler's last word; every one of the three comes with the same detail you read above, and running the example shows them in full.

Three errors, three line numbers, three functions you have to make a decision about. **The compiler wrote that to-do list, not you.** And until all three are done, the program is not built.

This one capability changes how fast a large system can be changed. In Python that edit means grepping and hoping. Here it means running `cargo build` three times.

### `_` is a promise you may not want to make

There is a fourth function in `06-a-new-variant` and it is *not* in the error list, because its last arm is `_`. Here is the same shape, taken from `04-tuples-and-options` where it compiles and runs:

```rust
fn tag(progress: &Progress) -> String {
    match progress {
        Progress::Finished { rating } => format!("finished, {rating}/10"),
        _ => "not finished".to_string(),
    }
}
```

```text
not finished
not finished
finished, 9/10
```

`_` told the compiler "anything else is fine by me" — so the new variant quietly became `"not finished"` and you were never asked whether that was right. For a `u8` that's exactly what you want. For an enum it usually means you've bought tomorrow's bug today.

Working rule: **on an enum of your own, name the variants wherever you can.** If several variants really do behave the same, write `A | B | C` rather than `_` — then adding `D` still errors.

And if you want the tooling to watch it for you, clippy has an opt-in lint:

```text
warning: wildcard match will also match any future added variants
  --> examples\04-tuples-and-options.rs:50:9
   |
50 |         _ => "not finished".to_string(),
   |         ^ help: try: `Progress::NotStarted | Progress::Reading { .. }`
```

### Binding a variant's data

So far we've only recognised which variant it is. A pattern's second job is to pull the data out of it — in the same step:

```rust
fn describe(progress: &Progress) -> String {
    match progress {
        Progress::NotStarted => "not started".to_string(),
        Progress::Reading { chapter, of } => format!("chapter {chapter} of {of}"),
        Progress::Finished { rating } => format!("finished, {rating}/10"),
        Progress::Dropped { at, .. } => format!("dropped at chapter {at}"),
    }
}
```

```text
not started
chapter 12 of 40
finished, 9/10
finished, 6/10
dropped at chapter 3
```

`chapter` and `of` are new names, filled in from that variant's fields. You wrote no separate "is it Reading?" test and no `.chapter` afterwards. **One step, two jobs.**

And the `..` in the last arm means "the rest of this variant's fields, whatever they are". `Dropped` also has a `reason` field we didn't need here. `..` can stand on either side:

```rust
fn why_dropped(progress: &Progress) -> String {
    match progress {
        Progress::Dropped { reason, .. } => format!("gave up because the {reason}"),
        _ => "not dropped".to_string(),
    }
}
```

```text
gave up because the art changed
not dropped
```

### Patterns nest

A pattern goes exactly as deep as the data does. This is a struct containing an enum containing a literal:

```rust
match entry {
    Entry {
        title,
        progress: Progress::Finished { rating: 10 },
    } => format!("{title}: a perfect score"),
    Entry {
        title,
        progress: Progress::NotStarted,
    } => format!("{title}: untouched"),
    Entry { title, .. } => format!("{title}: somewhere in the middle"),
}
```

```text
Vinland Saga: a perfect score
Berserk: untouched
Vagabond: somewhere in the middle
```

The first arm says "an `Entry` whose `progress` is `Finished` whose `rating` is exactly 10 — and by the way give me its `title`". All of that is one pattern. Written with `if`s it's three nested conditions and a field access.

### `@` — test it and keep it

Sometimes you want a value to be *in a range* **and** you want the value itself. Write `9..=10` alone and you have the test but not the number. `@` gives you both:

```rust
fn shelf(progress: &Progress) -> String {
    match progress {
        Progress::Finished { rating: r @ 9..=10 } => format!("hall of fame ({r}/10)"),
        Progress::Finished { rating } => format!("read once ({rating}/10)"),
        other => format!("still open — {}", describe(other)),
    }
}
```

```text
still open — not started
still open — chapter 12 of 40
hall of fame (9/10)
read once (6/10)
still open — dropped at chapter 3
```

Read `r @ 9..=10` as "if it's in the range 9 to 10, call it `r`". And the last arm shows that a bare name is a pattern too: `other` takes anything and gives it a name — exactly like `_`, but with a name attached.

### Arms are tried in order

`match` tries the arms top to bottom and **the first one that matches wins**. The rest are not even looked at.

```senpai-visual
{"kind":"concept","labels":["value","arm 1","arm 2","first match wins"]}
```

Which makes order part of the meaning of the program. These two functions have identical arms and differ only in their order:

```rust
match (chapter, unread) {
    (0, n) => format!("never opened, {n} waiting"),
    (c, n) if n > 20 => format!("chapter {c}, {n} behind — good luck"),
    (c, n) => format!("chapter {c}, {n} to go"),
    (_, 0) => "all caught up".to_string(),
}
```

```text
all caught up
never opened, 7 waiting
chapter 12, 31 behind — good luck
chapter 12, 3 to go

never opened, 0 waiting
never opened, 7 waiting
chapter 12, 31 behind — good luck
chapter 12, 3 to go
```

The top block is the original and the bottom is the reordered one, on the same inputs. The first line changed: `(_, 0)` now sits below a catch-all arm and its turn never comes.

The compiler sees this and warns — `warning: unreachable pattern`, in full in the errors section. It's a warning rather than an error, but it nearly always means your program does not do what you think. Working rule: **most specific arm at the top, most general at the bottom.**

### Guards, for what a pattern cannot say

A pattern talks about *shape*. It cannot say "these two numbers are equal". For that you need a **guard**: an `if` that is checked after the pattern has already matched.

```rust
fn is_at_the_end(chapter: u32, of: u32) -> String {
    match (chapter, of) {
        (c, total) if c == total => "waiting for the next chapter".to_string(),
        (c, total) => format!("{} chapters left", total - c),
    }
}
```

```text
waiting for the next chapter
28 chapters left
```

Guards make arms far more selective, at one cost you need to know about: **the compiler does not count guarded arms towards exhaustiveness.** The two arms `s if s >= 8` and `s if s < 8` cover every possible `u8` between them and the compiler still refuses — because proving it would mean solving the arithmetic, which it does not do. `examples/07-guards-do-not-count.rs` is exactly that, and the error says so in as many words; it's in the errors section.

### Two values at once

A `match` can work on a tuple, and then you're testing two (or three) values in one place. This is where `match` genuinely beats `if/else`:

```rust
match (mine, theirs) {
    (Finished { rating: a }, Finished { rating: b }) if a == b => {
        "we gave it the same score".to_string()
    }
    (Finished { rating: a }, Finished { rating: b }) => {
        format!("{a}/10 against {b}/10")
    }
    (NotStarted, NotStarted) => "neither of us has started".to_string(),
    (Finished { .. }, _) | (_, Finished { .. }) => "one of us has finished".to_string(),
    (Reading { chapter: a }, Reading { chapter: b }) => {
        format!("chapter {a} against chapter {b}")
    }
    _ => "still going".to_string(),
}
```

```text
we gave it the same score
9/10 against 4/10
neither of us has started
one of us has finished
chapter 12 against chapter 40
still going
```

Look at the fourth arm: a `|` joining two nested patterns, meaning "either of them is `Finished`". The `if`-shaped version of this function is nine comparisons and an `else` nobody thought about.

### Matching through a reference

Every function so far has taken `&Progress` rather than `Progress`, because we don't want to take ownership ([1.3.1](../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md)). When you match on a reference, the names the pattern creates are references too:

```rust
fn chapters_read(progress: &Progress) -> u32 {
    match progress {
        Progress::NotStarted => 0,
        Progress::Reading { chapter } => *chapter,
        Progress::Finished { .. } => 0,
    }
}
```

```text
0
12
0
```

`chapter` here is a `&u32`, which is why getting the number out is `*chapter`. Rust makes this convenient — you don't have to write the pattern as `&Progress::Reading { .. }` — and the consequence is that a `*` is sometimes needed. Forget it and `E0308` tells you `expected u32, found &u32` and suggests exactly that `*`. The full rule (it's called "default binding modes") is in [Phase 2 — Pattern matching in depth](../../../phase2-intermediate/08-rust-toolbox/01-pattern-matching-depth/README.md).

### `Option` is an ordinary enum

The last lesson showed that `Option` is not magic:

```rust
match latest {
    None => "nothing published yet".to_string(),
    Some(n) if n == read => "caught up".to_string(),
    Some(n) if n > read => format!("{} behind", n - read),
    Some(_) => "ahead of the release".to_string(),
}
```

```text
nothing published yet
caught up
3 behind
ahead of the release
```

Two variants, so two shapes of arm — and the compiler counts them exactly the way it counts `Progress`'s. Delete the `None` arm and you get the same `E0004`. **"You have to handle `None`" is not a special rule about `Option`; it is ordinary exhaustiveness.**

The last arm is `Some(_)` rather than `Some(n)` because we don't want the number in that case. The shorter ways of writing this kind of thing (`unwrap_or`, `map` and friends) arrive in [1.6.2](../../06-absence-and-failure/02-option-combinators/README.md); learn this `match` first, because every one of them is built on it.

### The two halves: invalid states are unrepresentable

Now you have both halves:

| | What it makes impossible |
|---|---|
| enum | building data that is an invalid combination |
| exhaustive `match` | writing code that forgets a state |

The first half isn't enough on its own: you can have a flawless enum and then write `_ => ()` in twenty places. The second half isn't enough either: an exhaustive `match` on a struct with contradictory fields saves nothing.

Together they are the sentence you keep hearing: **invalid states are unrepresentable.** Not because somebody was careful, but because the compiler won't have it.

One last note: very often only *one* arm is interesting and the rest do nothing. There's a shorter syntax for exactly that case and it's the next lesson — [1.5.5 — `if let`, `while let`, `let else`](../05-if-let-while-let-let-else/README.md). It teaches nothing new; it is this `match` with only one arm that matters.

---

## Hands on

```sh
cargo run -p p1-05-04-match-in-depth --example 01-match-is-an-expression
cargo run -p p1-05-04-match-in-depth --example 02-binding-out-of-variants
cargo run -p p1-05-04-match-in-depth --example 03-guards-and-order
cargo run -p p1-05-04-match-in-depth --example 04-tuples-and-options
```

`03` is built with a warning on purpose. Read the warning, then look at the output.

Then the four broken ones:

```sh
cargo run -p p1-05-04-match-in-depth --example 05-a-missing-arm --features broken
cargo run -p p1-05-04-match-in-depth --example 06-a-new-variant --features broken
cargo run -p p1-05-04-match-in-depth --example 07-guards-do-not-count --features broken
cargo run -p p1-05-04-match-in-depth --example 08-arms-disagree --features broken
```

Then try:

1. In `05-a-missing-arm`, replace the missing arm with `_ => "unknown".to_string()`. Does it compile? Now add a fifth variant to the enum. Does it still compile?
2. In `02-binding-out-of-variants`, change the `Dropped` arm of `describe` to `Progress::Dropped { at, reason }` and don't print `reason`. What does the compiler say?
3. In `03-guards-and-order`, move the `(_, 0)` arm of `queue_note_reordered` back to the top. Does the warning go, and what happens to the output?
4. In `04-tuples-and-options`, run this and read the opt-in warning:
   `cargo clippy -p p1-05-04-match-in-depth --example 04-tuples-and-options -- -W clippy::wildcard_enum_match_arm`

---

## Errors you will meet

### `E0004` — a variant you didn't handle

```text
error[E0004]: non-exhaustive patterns: `&Progress::Paused { .. }` not covered
  --> examples\06-a-new-variant.rs:19:11
   |
19 |     match progress {
   |           ^^^^^^^^ pattern `&Progress::Paused { .. }` not covered
   |
note: `Progress` defined here
  --> examples\06-a-new-variant.rs:10:6
   |
10 | enum Progress {
   |      ^^^^^^^^
...
15 |     Paused { at: u32 },
   |     ------ not covered
   = note: the matched value is of type `&Progress`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
23 ~         Progress::Dropped { at } => format!("dropped at chapter {at}"),
24 ~         &Progress::Paused { .. } => todo!(),
   |
```

**What the compiler is objecting to:** this `match` has to produce an answer for every possible `Progress`, and `Paused` has none. Because `match` is an expression, "no answer" means the program produces no value in that case — and that cannot be compiled.

**The fix:** add the arm, and decide what the right behaviour is:

```rust
Progress::Paused { at } => format!("paused at chapter {at}"),
```

**Why that's the fix:** the compiler's own suggestion is `todo!()`, and that's a good suggestion: it lets the build through and panics if you ever really reach that state. But don't leave `todo!()` as the answer — the question the error asked ("what does this state mean?") is still unanswered.

And recognise the temptation of `_ => ...`. It compiles, the error goes away, and you have just switched off the mechanism this very error demonstrated. If three functions errored, three decisions are needed — not one `_`.

If you left a variant out from the start you get the same error with one arm missing — `examples/05-a-missing-arm.rs` is the smallest version of it.

### `E0004` — when the guards fool you

```text
error[E0004]: non-exhaustive patterns: `0_u8..=u8::MAX` not covered
  --> examples\07-guards-do-not-count.rs:9:11
   |
 9 |     match stars {
   |           ^^^^^ pattern `0_u8..=u8::MAX` not covered
   |
   = note: the matched value is of type `u8`
   = note: match arms with guards don't count towards exhaustivity
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
11 ~         s if s < 8 => format!("the rest ({s}/10)"),
12 ~         0_u8..=u8::MAX => todo!(),
   |
```

**What the compiler is objecting to:** the line `match arms with guards don't count towards exhaustivity` is the whole story. Two guarded arms do cover every `u8` between them — but working that out means solving `s >= 8` against `s < 8`, and the compiler doesn't do that. Instead it treats every guarded arm as "might match, might not".

**The fix:** make the last arm guard-free:

```rust
s => format!("the rest ({s}/10)"),
```

**Why that's the fix:** now there is one unconditional arm that definitely matches, so the coverage is complete. That's a good style rule in its own right: finish a chain of guards with an unguarded arm — the same thing you do with `else` at the end of a chain of `if`s.

### `E0308` — the arms are different types

```text
error[E0308]: `match` arms have incompatible types
  --> examples\08-arms-disagree.rs:11:18
   |
 9 | /     match stars {
10 | |         0..=4 => "weak".to_string(),
   | |                  ------------------ this is found to be of type `String`
11 | |         5..=7 => "watchable",
   | |                  ^^^^^^^^^^^ expected `String`, found `&str`
12 | |         _ => "good".to_string(),
13 | |     }
   | |_____- `match` arms have incompatible types
   |
help: try using a conversion method
   |
11 |         5..=7 => "watchable".to_string(),
   |                             ++++++++++++
```

**What the compiler is objecting to:** `match` is an expression, and an expression has one type. The first arm fixed that type as `String` (`this is found to be of type String`) and the next arm handed over a `&str`.

**The fix:** exactly what it says:

```rust
5..=7 => "watchable".to_string(),
```

**Why that's the fix:** `&str` and `String` are two different types ([1.4.1](../../04-text-and-strings/01-string-vs-str/README.md)) and Rust doesn't convert between them behind your back. The other repair is to have every arm produce a `&str` and convert once on the outside — which, where it's possible, is cheaper, because it allocates nothing.

### `warning: unreachable pattern` — an arm whose turn never comes

```text
warning: unreachable pattern
  --> examples\03-guards-and-order.rs:26:9
   |
25 |         (c, n) => format!("chapter {c}, {n} to go"),
   |         ------ matches any value
26 |         (_, 0) => "all caught up".to_string(),
   |         ^^^^^^ no value can reach this
   |
   = note: `#[warn(unreachable_patterns)]` (part of `#[warn(unused)]`) on by default
```

**What the compiler is objecting to:** the arm on line 25 takes any value at all, so the arm on line 26 is never tried. It writes both halves of the explanation: `matches any value` on the culprit, `no value can reach this` on the victim.

**The fix:** move the specific arm above the general one. Or, if it really is redundant, delete it.

**Why that's the fix:** this is a warning, not an error — the program builds and runs — but it nearly always means the behaviour isn't what you think. In that example "all caught up" is never printed; "never opened, 0 waiting" comes out instead. Read compiler warnings like errors; they're just less urgent.

---

## Exercises

### Warm up

<details>
<summary>What does this print? <code>match 5 { 1..=5 => "a", 5..=10 => "b", _ => "c" }</code></summary>

`a`. Both of the first two arms match 5, and arms are tried top to bottom: the first match wins. Compile it and you get a warning — but a different one, `overlapping_range_endpoints`, pointing out that the two ranges share the 5. Not `unreachable pattern`: the second arm is still reachable for 6 through 10.

</details>

<details>
<summary>You have a four-variant enum and your <code>match</code> has three arms. What happens?</summary>

`E0004`, naming the variant you didn't cover and pointing at its definition. The program is not built.

</details>

<details>
<summary>Why aren't <code>s if s >= 8</code> and <code>s if s < 8</code> exhaustive together?</summary>

Because the compiler doesn't count guarded arms. Proving the coverage would mean solving the arithmetic, and it doesn't do that. The error says so: `match arms with guards don't count towards exhaustivity`.

</details>

<details>
<summary>What's the difference between <code>_ =></code> and <code>other =></code>?</summary>

Nothing, in terms of what they match: both take everything left over. The difference is that `other` keeps the value under a name so you can use it and `_` doesn't — which also means `_` takes no ownership.

</details>

<details>
<summary>In <code>n @ 1..=9</code>, what does <code>n</code> hold?</summary>

The value itself, given it was between 1 and 9. `@` checks the condition and binds the checked value to that name.

</details>

<details>
<summary>You add a variant to the enum. Which of these two functions errors?</summary>

```rust
fn a(p: &Progress) -> bool {
    match p {
        Progress::Finished { .. } => true,
        Progress::NotStarted | Progress::Reading { .. } | Progress::Dropped { .. } => false,
    }
}

fn b(p: &Progress) -> bool {
    match p {
        Progress::Finished { .. } => true,
        _ => false,
    }
}
```

Only `a`. `b` quietly returns `false` and never asks you whether that's right — which is exactly why `a` is the better piece of writing, longer though it is.

</details>

### Repair

Fix all four broken examples, and for each one say what you changed and why:

1. `examples/05-a-missing-arm.rs` — once by adding the missing arm, once with `_`. Then say which one you'd write in a real program.
2. `examples/06-a-new-variant.rs` — all three errors. Decide for each function what `Paused` should do; the three answers are not the same. Then look at what the fourth function (`is_finished`) now returns and say whether it's right.
3. `examples/07-guards-do-not-count.rs` — without deleting any guard.
4. `examples/08-arms-disagree.rs` — two ways: once by making every arm a `String`, once by keeping every arm a `&str`.

Then change `examples/03-guards-and-order.rs` so that the `unreachable pattern` warning goes away and both functions produce the same output.

### Implement

Six functions in `src/lib.rs`:

```sh
cargo test -p p1-05-04-match-in-depth
```

Each one wants a different piece of the pattern language: plain arms, ranges and alternatives, a guard, an `@` binding, a tuple of two values, and an `Option`. None of them needs an `if` — if you write one that isn't a guard, you have probably taken the hard road.

The exact specification is in the doc comments. The table above each function is everything you need to pass its tests; you should not have to open the test file.

### Build

Add a `Paused { at: u32 }` variant to the `Progress` enum in `src/lib.rs` and change nothing else. Then:

1. Run `cargo test -p p1-05-04-match-in-depth` and **count** the errors.
2. For each one, decide what `Paused` should do and write it. What should `describe` say? Which shelf does `shelf` put it on?
3. Which functions did **not** error? For each, say why, and say whether their current behaviour on `Paused` is right.
4. Write a test for the new behaviour.

This exercise is the thing you do once a week in a real system. Note down how long it took.

### Challenge (optional)

**Part one.** Patterns work on slices too — something you haven't seen yet:

```rust
fn run_of(chapters: &[u32]) -> String {
    match chapters {
        [] => "nothing yet".to_string(),
        [only] => format!("just chapter {only}"),
        [first, .., last] => format!("chapters {first} to {last}"),
    }
}
```

Run it. Then delete the `[only]` arm and see what the compiler says. Then try `[first, second, ..]` as well.

**Part two.** When all you want is "is it this variant?" and not the data inside it, there's a macro:

```rust
fn is_finished(progress: &Progress) -> bool {
    matches!(progress, Progress::Finished { .. })
}
```

Put it in place of the `match` version and run `cargo clippy`. What did clippy say about the old version? And — the more important question — does this version error when a new variant is added?

**Part three.** Run this over the whole lesson:

```sh
cargo clippy -p p1-05-04-match-in-depth --all-targets -- -W clippy::wildcard_enum_match_arm
```

How many warnings do you get? For each, decide whether `_` was the right call there. (This lint is off by default. After the exercise, say why you think it is.)

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| pattern | the shape the compiler matches a value against | `let`, `match`, function arguments |
| arm | one `pattern => expression` in a `match` | every `match` |
| exhaustiveness | the proof that every case has an arm | what produces `E0004` |
| `E0004` | a case was left without an arm | it names the missing variant |
| range pattern | `1..=9` | numbers and characters |
| alternative | `7 \| 8` in one arm | several cases, one behaviour |
| guard | an `if` on an already-matched arm | conditions a pattern can't state |
| `@` | test it and keep it: `n @ 1..=9` | when you want the condition and the value |
| `_` | wildcard, no name | fine on numbers, careful on your own enums |
| `..` | "the rest of the fields" | variants with many fields |
| unreachable arm | an arm whose turn never comes | a warning, and nearly always a bug |

### What you now know

- `match` is an expression: it produces a value, and every arm must agree on the type.
- The left of each `=>` is the same pattern language as `let`, with more interesting shapes.
- The compiler proves every case has an arm, and gives `E0004` with the missing variant's name when one doesn't.
- Adding a variant turns every place that must change into a compile error — except where you wrote `_`.
- One arm both recognises a variant and pulls its data out, in a single step.
- Arms are tried top to bottom and the first match wins.
- Guards don't count towards exhaustiveness, so finish a chain of them with an unguarded arm.
- `Option` is an ordinary enum and all of the same rules apply to it.

### What comes back later

- **The short syntax for "only one arm matters to me"** — [1.5.5 — `if let`, `while let`, `let else`](../05-if-let-while-let-let-else/README.md)
- **`Option`, and what you can do with it without a `match`** — [1.6.1 — `Option` and null safety](../../06-absence-and-failure/01-option-and-null-safety/README.md) and [1.6.2](../../06-absence-and-failure/02-option-combinators/README.md)
- **`Result` and `?`, which do this same matching over errors** — [1.6.3](../../06-absence-and-failure/03-result-and-question-mark/README.md)
- **Default binding modes, `ref`, slice patterns, and the rest of the pattern language** — [Phase 2 — Pattern matching in depth](../../../phase2-intermediate/08-rust-toolbox/01-pattern-matching-depth/README.md)
- **`matches!` and macros** — [Phase 2 — `macro_rules!`](../../../phase2-intermediate/08-rust-toolbox/02-macro-rules-basics/README.md)

### Can you explain?

- Why is `match` an expression, and what does that force to be true of the arms?
- What does "exhaustiveness" mean, and how does the compiler prove it?
- What exactly happens when you add a variant to an enum?
- Why is `_` usually a bad choice on an enum of your own?
- Why are two guarded arms that cover everything between them still not exhaustive?
- What difference does the order of the arms make, and when does the compiler warn about it?
- Why does matching on an `Option` need nothing new?

---

## Going further

- [The Rust Book — `match`](https://doc.rust-lang.org/book/ch06-02-match.html) — the same ground, officially.
- [The Rust Book — Patterns and matching](https://doc.rust-lang.org/book/ch19-00-patterns.html) — the full patterns chapter, including the things only mentioned here.
- [`rustc --explain E0004`](https://doc.rust-lang.org/error_codes/E0004.html) — the official write-up of this exact error. You can get it in the terminal too.
- [The Rust Reference — Patterns](https://doc.rust-lang.org/reference/patterns.html) — the precise list of every pattern form there is.
- [`clippy::wildcard_enum_match_arm`](https://rust-lang.github.io/rust-clippy/master/#wildcard_enum_match_arm) — the lint that finds dangerous `_`s.
