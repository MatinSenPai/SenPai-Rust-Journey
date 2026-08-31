# 2.10.1 — Pattern matching in depth

## At a glance

After this lesson you can:

- Explain why `match x { n if n >= 0 => .., n if n < 0 => .. }` fails to compile — even though the two guards obviously cover everything — and say exactly where that fact came from in 1.5.4.
- Write an or-pattern like `Ok(n) | Err(n)` whose two sides genuinely agree on type, and read and fix `E0308` yourself when they don't.
- Say why matching a `&LogEvent` gives a field `&String` instead of `String`, and when you still need to write `ref` by hand.
- Match a slice by its length — not `.len()` and indexing — with `[]`, `[x]`, `[first, .., last]`, and explain how the compiler proves those three cover every length.

**Time:** ~65 minutes · **Prerequisites:**
[1.5.4 — `match` in depth](../../../phase1-fundamentals/05-your-own-types/04-match-in-depth/README.md)

---

## Why this matters

[1.5.4](../../../phase1-fundamentals/05-your-own-types/04-match-in-depth/README.md) went halfway: a `match` that recognizes a variant, pulls its data out in the same step, and gets proven exhaustive by the compiler. But that same lesson left three things for this one: how matching through a reference produces new names that are themselves references (the "default binding mode," promised for exactly this lesson), how patterns work on slices too — "something you haven't seen yet" — and, under one combined "what comes back later" bullet, "the rest of the pattern language," which is where or-patterns and `@` bindings actually live.

This lesson finishes exactly those three, plus a fourth point that never even came up there: `ref`, for when the matched value is owned outright. None of these are new concepts — it's still the same `match` you've known since Phase 1 — just the corners where that simple mental model surprises you if you don't look closely.

Python 3.10+, as 1.5.4 said, has `match`/`case`, and borrowed most of its shape from languages like Rust. Or-patterns exist there too (`case 400 | 401 | 402:`), but with no range pattern over integers and nothing equivalent to `@` that both tests and keeps. Matching over lists (`case [first, *rest]:`) exists as well — but none of it is proven by a compiler; it only runs if you actually run `mypy`, and even that is optional. Here it's Rust's own compiler, on every build, no exceptions.

---

## The concept

### Quick recap: what you already have with `match`

A guard (an `if` on an arm), an or-pattern (`|`), an `@` binding, and nested destructuring — all four from 1.5.4. Put together, on the same `LogEvent` and `Severity` that `src/lib.rs` already has, they look like this:

```rust
fn headline(event: &LogEvent) -> Option<String> {
    match event {
        LogEvent::Request { status, path, .. } if *status >= 500 => {
            Some(format!("5xx {status} on {path}"))
        }
        LogEvent::Message { severity: Severity::Error, text } => Some(format!("error: {text}")),
        _ => None,
    }
}
```

```text
Some("5xx 503 on /api/jobs")
Some("error: db down")
None
```

The first arm has a guard (`if *status >= 500`); the second is nested destructuring — `severity: Severity::Error` is itself a sub-pattern, not a separate comparison. If any of these four feel shaky, stop here and go back to 1.5.4; everything from here builds on them without re-explaining.

### Or-patterns: every alternative binds the same type

`|` matches one arm against several shapes — even shapes from **different variants**, not just literals of one type. Here `status` from `Request` and `retry_after` from `RateLimited` are both `u16`, so binding them to one shared name is legal:

```rust
match event {
    LogEvent::Request { status: n } | LogEvent::RateLimited { retry_after: n } => *n,
}
```

```text
503
30
```

The rule: **the compiler checks every binding an or-pattern makes across all its alternatives — both the name and the type.** If one side gives `n: u16` and the other `n: u64`, the program does not build; the error is `E0308`, and you'll see it in "Errors you will meet." The same rule covers the name itself, not just the type — if one side binds `n` and the other `m`, the compiler says `E0408`, "this name is not in every alternative," because each arm must supply one fixed set of names with one fixed set of types, or the arm's body would not know which value it is working with.

### Default binding modes: matching through `&` borrows for free

Question: in `LogEvent::Request { method, .. }`, when you're matching a `&LogEvent`, what type does `method` have? The answer is `&String`, not `String`:

```rust
fn method_of(event: &LogEvent) -> Option<&str> {
    match event {
        LogEvent::Request { method, .. } => Some(method.as_str()),
        _ => None,
    }
}
```

```text
Some("POST")
Request { method: "POST", path: "/login" }
```

If `method` were really `String`, this arm would have to move it out of something you've only borrowed — a compile error, since `event` isn't yours. Instead, Rust automatically binds every field inside a pattern matched through `&` by reference — no `ref`, nothing to write. That's also why the second line of output above still works: `event` was never actually consumed, only read. This behavior has a name — **default binding mode**, better known as "match ergonomics" — and it's exactly what 1.5.4 linked here to finish. Before this behavior existed (pre-Rust-2018) you had to write `ref method` explicitly; today it's automatic, and `ref` has effectively retired from that particular job.

```senpai-visual
{"kind":"borrowing","labels":["match &LogEvent { Request { method, .. } }","method binds as &String, automatically","event stays borrowed, nothing moves","the same match on an owned LogEvent","text would move out — ref is what stops that"]}
```

### `ref`: borrowing one field out of an owned value

Default binding modes only kick in when the value you're matching is itself a reference. If `event` is owned (`LogEvent`, not `&LogEvent`), matching it **moves** by default whatever field you bind:

```rust
match owned {
    LogEvent::Message { severity, ref text } => println!("[{severity}] {text}"),
    LogEvent::Request { .. } => {}
}
println!("{owned:?}");
```

```text
[2] disk 91% full
Message { severity: 2, text: "disk 91% full" }
```

`severity` is a `u8` (`Copy`), so binding it by value is fine. `text` is a `String` — bind it without `ref` and it would move out of `owned`, and the last line (`println!("{owned:?}")`) would stop compiling, because `owned` would no longer be whole. `ref text` says "only borrow this field, don't move it" — without borrowing or losing the rest of `owned` either. This is exactly where `ref` is still alive: **matching an owned value, when you want to borrow just one field of it, not move the whole thing.**

### Slice patterns: exhaustive by length

Patterns work on `&[T]` too, and this time "shape" means length:

```rust
fn describe(samples: &[u64]) -> String {
    match samples {
        [] => "no samples".to_string(),
        [only] => format!("1 sample: {only}ms"),
        [first, .., last] => format!("first {first}ms, last {last}ms"),
    }
}
```

```text
[]               -> no samples
[42]             -> 1 sample: 42ms
[5, 80, 9, 12]   -> first 5ms, last 12ms
[7, 3]           -> first 7ms, last 3ms
```

Three arms, three lengths: `[]` is exactly zero elements, `[only]` is exactly one, `[first, .., last]` is two or more (one is needed for `first`, one for `last`, and `..` absorbs whatever is left in between — even an empty middle, as in `[7, 3]`). The compiler counts these three the same way it counts an enum's variants: zero, one, two-or-more is a complete partition of every possible length, so this `match` is exhaustive — with no `_` anywhere.

```senpai-visual
{"kind":"concept","labels":["length 0: []","length 1: [x]","length 2 or more: [first, .., last]","together, every length is covered"]}
```

Compare that to a version written with `.len()` and indexing: `if samples.len() == 0 { .. } else if samples.len() == 1 { .. } else { samples[0]; samples[samples.len() - 1]; .. }`. Miss one of those branches, or get an index wrong, and the compiler would say nothing — you'd only find out at run time, via a panic or a wrong number. A slice pattern gives you that proof for free.

### `rest @ ..`: naming the middle

The third arm above threw away the middle of the slice with `..`. If you need that middle itself — not just the first and last — `@` does the same thing it does in ordinary patterns: instead of discarding, it names:

```rust
fn middle(samples: &[u64]) -> &[u64] {
    match samples {
        [_, rest @ .., _] => rest,
        _ => &[],
    }
}
```

```text
[]               -> middle []
[42]             -> middle []
[7, 3]           -> middle []
[5, 80, 9, 12]   -> middle [80, 9]
```

`[_, rest @ .., _]` reads as "a first element I don't care about, a last element I don't care about, and whatever is between them, name it `rest`." Because this pattern needs at least two elements (one for each `_`), it simply doesn't match a slice of length zero or one, so the `_` arm answers with an empty slice; for exactly two elements, `rest` also ends up an empty slice (there's nothing between two adjacent elements) — no panic, no error, just a slice of length zero.

---

## Hands on

```sh
cargo run -p p2-10-01-pattern-matching-depth --example 01-or-pattern-type-consistency
cargo run -p p2-10-01-pattern-matching-depth --example 02-binding-modes-and-ref
cargo run -p p2-10-01-pattern-matching-depth --example 03-slice-patterns
```

(Each run also shows four `unused variable` warnings first — from the four still-`todo!()` functions in `src/lib.rs` you complete below, in "Exercises." Ignore those; the output beneath them is what matters.)

Then the three broken ones:

```sh
cargo run -p p2-10-01-pattern-matching-depth --example 04-or-pattern-mismatch-broken --features broken
cargo run -p p2-10-01-pattern-matching-depth --example 05-forgot-ref-broken --features broken
cargo run -p p2-10-01-pattern-matching-depth --example 06-slice-missing-arm-broken --features broken
```

Then try these:

1. In `02-binding-modes-and-ref`, remove `ref` from the `LogEvent::Message` arm. Which error do you get, and how does it compare to `05`?
2. In `01-or-pattern-type-consistency`, change `retry_after: n` to `retry_after: m`. What happens, and why does the error mention both `n` and `m`?
3. In `03-slice-patterns`, delete the `[only]` arm from `describe`. Compare the error to `06`.

---

## Errors you will meet

### `E0308` — the two sides of an or-pattern disagree on type

```text
error[E0308]: mismatched types
  --> phase2-intermediate\10-rust-toolbox\01-pattern-matching-depth\examples\04-or-pattern-mismatch-broken.rs:16:81
   |
15 |     match event {
   |           ----- this expression has type `LogEvent`
16 |         LogEvent::Request { status: code } | LogEvent::Heartbeat { uptime_secs: code } => {
   |                                     ---- first introduced with type `u16` here  ^^^^ expected `u16`, found `u64`
   |
   = note: in the same arm, a binding must have the same type in all alternatives

For more information about this error, try `rustc --explain E0308`.
```

**What the compiler is objecting to:** the arm binds one shared name, `code`, across both alternatives of the or-pattern. The first alternative (`status`) fixed its type as `u16`; the second (`uptime_secs`) gives a `u64`. The compiler can't decide which type `code` is inside the arm's body, so it rejects the match.

**The fix:** split the two alternatives into two separate arms, each keeping its own type:

```rust
match event {
    LogEvent::Request { status } => status.to_string(),
    LogEvent::Heartbeat { uptime_secs } => uptime_secs.to_string(),
}
```

**Why this is the fix:** no name has to be shared between two different types anymore; each arm writes its own body against its field's real type. An or-pattern earns its keep when several shapes genuinely produce one same-typed binding — not whenever two alternatives merely look alike.

### `E0382` — borrow of a partially moved value

```text
error[E0382]: borrow of partially moved value: `event`
  --> phase2-intermediate\10-rust-toolbox\01-pattern-matching-depth\examples\05-forgot-ref-broken.rs:24:16
   |
21 |         LogEvent::Message { text, .. } => println!("{text}"),
   |                             ---- value partially moved here
...
24 |     println!("{event:?}");
   |                ^^^^^ value borrowed here after partial move
   |
   = note: partial move occurs because value has type `String`, which does not implement the `Copy` trait
help: borrow this binding in the pattern to avoid moving the value
   |
21 |         LogEvent::Message { ref text, .. } => println!("{text}"),
   |                             +++

For more information about this error, try `rustc --explain E0382`.
```

**What the compiler is objecting to:** `event` is owned, not a reference, so matching it against `text` without `ref` genuinely moves `text` out of `event`. After that, `event` is no longer whole — one of its fields is gone — and the later line that wants all of `event` (`{event:?}`) has nothing to show for it.

**The fix:** the compiler already suggests it — `ref` in front of `text`:

```rust
LogEvent::Message { ref text, .. } => println!("{text}"),
```

**Why this is the fix:** `ref text` only takes a reference to `text`, not its ownership. `event` stays whole, because nothing was actually moved out of it — exactly what a `&LogEvent` gets for free from default binding modes; here, because `event` itself isn't a reference, you have to ask for it explicitly.

### `E0004` — non-exhaustive slice pattern

```text
error[E0004]: non-exhaustive patterns: `&[_]` not covered
  --> phase2-intermediate\10-rust-toolbox\01-pattern-matching-depth\examples\06-slice-missing-arm-broken.rs:10:11
   |
10 |     match samples {
   |           ^^^^^^^ pattern `&[_]` not covered
   |
   = note: the matched value is of type `&[u64]`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
12 ~         [first, .., last] => format!("first {first}ms, last {last}ms"),
13 ~         &[_] => todo!(),
   |

For more information about this error, try `rustc --explain E0004`.
```

**What the compiler is objecting to:** two arms are left — one for length zero (`[]`), one for length two-or-more (`[first, .., last]`) — and there's a complete gap between them: length exactly one. `&[_]` means exactly that — a one-element slice, whatever that one element is.

**The fix:** bring back the missing arm:

```rust
[only] => format!("1 sample: {only}ms"),
```

**Why this is the fix:** now three arms — zero, one, two-or-more — are exactly a partition of every possible slice length, and the compiler proves it. An `if`/`else` chain with `.len()` and indexing had this same gap and stayed silent about it — it would panic or misreport, only at run time, and only if a test happened to reach that length.

---

## Exercises

### Warm up

<details>
<summary>Does this compile?</summary>

```rust
fn classify(n: i32) -> &'static str {
    match n {
        n if n >= 0 => "non-negative",
        n if n < 0 => "negative",
    }
}
```

</details>

<details>
<summary>Answer</summary>

No. Even though these two guards together genuinely cover every `i32`, the compiler doesn't count guarded arms toward exhaustiveness — the same rule you saw in 1.5.4. Fixing it needs an unguarded arm, such as a trailing `_ =>`.

</details>

<details>
<summary>Does this compile? (assume <code>r: Result&lt;i32, String&gt;</code>)</summary>

```rust
match r {
    Ok(n) | Err(n) => println!("{n}"),
}
```

</details>

<details>
<summary>Answer</summary>

No — `E0308`. `Ok` of a `Result<i32, String>` gives an `i32`, its `Err` gives a `String`; binding both to one shared name, `n`, asks it to be two different types at once.

</details>

<details>
<summary>In <code>LogEvent::Request { method, .. }</code>, when matching a <code>&LogEvent</code>, what type does <code>method</code> have?</summary>

</details>

<details>
<summary>Answer</summary>

`&String`, not `String`. Default binding modes give every field bound inside a pattern matched through `&` a reference automatically.

</details>

<details>
<summary>With no <code>_</code> at all, what's the minimum number of arms to exhaustively match a <code>&[T]</code>, and what are they?</summary>

</details>

<details>
<summary>Answer</summary>

Three: `[]` (length zero), `[x]` (length one), `[first, .., last]` (length two or more). Together these three are a complete partition of every possible length.

</details>

### Repair

Fix all three broken examples:

1. Fix `examples/04-or-pattern-mismatch-broken.rs` by splitting the two alternatives into two independent arms, each keeping its own field's real type.
2. Fix `examples/05-forgot-ref-broken.rs` by adding `ref` in front of `text`.
3. Fix `examples/06-slice-missing-arm-broken.rs` by adding the missing arm for length one.

### Implement

Four functions in `src/lib.rs` — each one forces one of the things above:

- `describe_status` — or-pattern, range pattern, `@` binding, arm order.
- `noteworthy` — a guard + nested destructuring of an enum inside a struct.
- `method_of` — default binding mode: `method` arrives as `&String`.
- `summarize_samples` — slice pattern, exhaustive over lengths 0 / 1 / 2+.

Each has a doc comment stating exactly the strings the tests expect — you don't need to open the test file.

```sh
cargo test -p p2-10-01-pattern-matching-depth
```

### Build

Write a new function in `src/lib.rs`:

```rust
pub fn middle_samples(samples: &[u64]) -> &[u64]
```

It should return the elements strictly between the first and the last — the same middle that `[first, .., last]` in `summarize_samples` never names on its own. More precisely:

- For a slice with zero or one element, there is no "between"; the result is an empty slice.
- For a slice with exactly two elements the result is also empty — there's nothing between two adjacent elements.
- For a slice with three or more elements, the result is every element strictly between the first and the last, in order.

For instance, `middle_samples(&[5, 80, 9, 12])` should give `&[80, 9]`. Write it with `rest @ ..`, not manual indexing. Add a small test for it too.

### Challenge (optional)

This part is optional — skip it if you're short on time.

Write a new function in `src/lib.rs`:

```rust
pub fn first_alert(events: &[LogEvent]) -> Option<String>
```

If the **first** element of `events` is a `Request` with `status >= 500`, return exactly `format!("first event: server error {status} on {path}")` using that first event's `status` and `path`. For every other case (an empty slice, a first element that isn't a `Request`, or a `Request` with `status < 500`), return `None`. Write it as a **single** `match` on `events` — one arm that matches both the slice shape (only the first element matters, whatever the rest are) and the nested `Request` shape at once, plus a guard for `status >= 500` — not `events.first()` followed by a separate `match`.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| default binding mode | matching a pattern through `&` binds every field you bind by reference automatically | anywhere you match `&LogEvent` or the like |
| `ref` | an explicit by-reference binding, for when the matched value is owned outright | borrowing one field without moving the rest of the value |
| slice pattern | matching `&[T]` by shape — `[]`, `[x]`, `[first, .., last]` — exhaustive over lengths 0 / 1 / 2+ | anywhere you'd be tempted to reach for `.len()` and indexing |
| `rest @ ..` | naming the very middle `..` alone would discard | when you need the middle elements themselves, not just first and last |

### What you now know

- Why two guards that together look exhaustive still don't make a `match` exhaustive — and where 1.5.4 showed you this with a real transcript.
- Why every alternative of an or-pattern must bind the same names to the same types, and which violation `E0308` catches versus `E0408`.
- Why matching a `&LogEvent` gives every field you bind a reference — with nothing written for it — and the name for that behavior: default binding mode.
- When you still need to write `ref` by hand: matching an owned value, when you only want to borrow one field of it.
- How the compiler proves a slice pattern is exhaustive by counting length — `[]` / `[x]` / `[first, .., last]` for 0 / 1 / 2+ — and how `rest @ ..` names that middle itself.

### What comes back later

- **`matches!` and the rest of declarative macros** — this lesson only went deeper on `match` itself; writing a macro of your own belongs to [2.10.2 — `macro_rules!` basics](../02-macro-rules-basics/README.md).
- **Pattern matching on real backend code** — parsing JSON payloads, branching on error kinds, modeling a request's state — from [Phase 3 — Backend foundations](../../../phase3-backend-foundations/README.md) onward, exactly the things you saw today become routine.

### Can you explain?

- Why don't `n if n >= 0` and `n if n < 0` together make a `match` exhaustive?
- What exactly must an or-pattern keep identical across its alternatives? What error do you get if it doesn't?
- What does "default binding mode" mean, and why is `ref` almost never needed today — except one specific case?
- How does the compiler prove that `[]`/`[x]`/`[first, .., last]` cover every length? Explain it in your own words, without looking back up.

---

## Going further

- [The Rust Book — Patterns and Matching](https://doc.rust-lang.org/book/ch19-00-patterns.html) — the full chapter on patterns, including everything you saw here.
- [The Rust Reference — Patterns](https://doc.rust-lang.org/reference/patterns.html) — the exact list of every pattern form, including default binding modes.
- [`rustc --explain E0308`](https://doc.rust-lang.org/error_codes/E0308.html) — the official explanation of this error.
- [`rustc --explain E0382`](https://doc.rust-lang.org/error_codes/E0382.html) — the official explanation of the partial-move error.
