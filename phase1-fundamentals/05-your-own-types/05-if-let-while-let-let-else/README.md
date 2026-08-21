# 1.5.5 — `if let`, `while let`, `let else`

## At a glance

After this lesson you can:

- Turn a `match` with one interesting arm into an `if let` — and say what you gave up doing it.
- Empty a collection with `while let Some(x) = ...` and say exactly where the loop stops.
- Bind a value or leave with `let ... else`, and explain why the `else` block must diverge.
- Look at real code and pick which of the four forms reads best — including when none of them do and a `match` is right.

**Time:** ~45 minutes · **Prerequisites:** [1.5.4 — `match` in depth](../04-match-in-depth/README.md) and [1.1.5 — Control flow](../../01-foundations/05-control-flow/README.md)

---

## Why this matters

The last lesson handed you `match` and, with it, a guarantee: the compiler counts the cases and gives you `E0004` when one is missing. That guarantee is valuable and you don't give it up for nothing.

But most real code has exactly **one** interesting case. The rest are "do nothing", and writing `None => {}` is a whole line that says nothing — worse, it *looks* like a decision you made, when you were only keeping the compiler happy.

And a second thing. In [1.1.5](../../01-foundations/05-control-flow/README.md) you met the guard clause: deal with the exceptional case at the top of the function and leave, so the rest of the function doesn't have to think about it. But that guard was on a **condition** — `if parts == 0`. What you actually want to guard on, most of the time, is a **pattern**: "if this is a `Some`, open it, otherwise leave." Until today you had no way to write that, and had to sink the whole function inside an `if let`.

This lesson fills all three gaps, and ends by giving you a rule for choosing.

---

## The concept

### `if let` — a `match` with one arm you care about

```rust
let rating: Option<u8> = Some(9);

match rating {
    Some(score) => println!("match:      rated {score}/10"),
    None => {}
}
```

```text
match:      rated 9/10
```

Read that second arm out loud: "and if there is no rating, do nothing." A whole line spent saying nothing. So Rust has a shorter form:

```rust
if let Some(score) = rating {
    println!("if let:     rated {score}/10");
}
```

```text
if let:     rated 9/10
```

`if let` means: if the value on the right matches the pattern on the left, run the block and bind whatever the pattern binds; otherwise carry straight on. **It is exactly that `match`, with the `_ => {}` arm you didn't write.**

Two things to know right away:

- The left of the `=` is a **pattern**, the same pattern machinery from [1.1.3](../../01-foundations/03-compound-types-and-destructuring/README.md). Anything legal in a `match` arm is legal here.
- The name the pattern binds — `score` here — is alive **only inside the block**. Outside the braces it doesn't exist, and one of this lesson's three errors is exactly that.

### `if let ... else` — the arm you deleted, brought back

When the other case has work to do, `else` brings that arm back:

```rust
let missing: Option<u8> = None;
if let Some(score) = missing {
    println!("with else:  rated {score}/10");
} else {
    println!("with else:  not rated yet");
}
```

```text
with else:  not rated yet
```

And because this is an `if`, it's an **expression** like every other `if` in Rust, so it's worth something:

```rust
let shown = if let Some(score) = rating { score } else { 0 };
println!("as a value: {shown}");
```

```text
as a value: 9
```

The three rules from [1.1.5](../../01-foundations/05-control-flow/README.md) still hold: both branches must produce the same type, and used as a value the `else` is compulsory.

### What you gave up: exhaustiveness

Here's the part nobody usually says out loud. `match` is checked for **exhaustiveness**; `if let` is not. The difference shows up the day you add a variant.

```rust
fn describe(status: &Status) -> String {
    match status {
        Status::Watching { episode } => format!("on episode {episode}"),
        Status::Completed { rating } => format!("finished, {rating}/10"),
        Status::Dropped => "dropped".to_string(),
    }
}
```

```rust
fn medal(status: &Status) -> String {
    if let Status::Completed { rating } = status {
        format!("{rating}/10")
    } else {
        "—".to_string()
    }
}
```

```text
Watching { episode: 7 }   on episode 7     —
Completed { rating: 9 }   finished, 9/10   9/10
Dropped                   dropped          —
```

Now add a fourth variant:

```rust
enum Status {
    Watching { episode: u32 },
    Completed { rating: u8 },
    Dropped,
    OnHold,
}
```

```text
error[E0004]: non-exhaustive patterns: `&Status::OnHold` not covered
  --> examples\04-the-silent-hole.rs:19:11
   |
19 |     match status {
   |           ^^^^^^ pattern `&Status::OnHold` not covered
   |
note: `Status` defined here
  --> examples\04-the-silent-hole.rs:7:6
   |
 7 | enum Status {
   |      ^^^^^^
...
11 |     OnHold,
   |     ------ not covered
```

`describe` stops compiling and puts its finger on the exact gap. `medal` **compiles silently**, runs, and answers `—` for `OnHold`. Maybe that's right and maybe it isn't — but nobody asked you.

> **Make this trade deliberately:** every time you turn a `match` into an `if let` you remove a safety net. If "all the other cases" really are one uniform group ("anything else means no"), it's a good trade. If you're just hiding the boring arms, it's a bad one.

### `while let` — keep going while the pattern matches

`Vec::pop` removes the last element and hands back `Some(...)`, then `None` once the Vec is empty. That is precisely the shape `while let` was made for:

```rust
let mut stack = vec![1, 2, 3];
while let Some(top) = stack.pop() {
    println!("popped:     {top}   (left: {})", stack.len());
}
```

```text
popped:     3   (left: 2)
popped:     2   (left: 1)
popped:     1   (left: 0)
```

Without `while let` you'd have written this:

```rust
loop {
    match stack.pop() {
        Some(top) => println!("popped:     {top}   (left: {})", stack.len()),
        None => break,
    }
}
```

```text
popped:     3   (left: 2)
popped:     2   (left: 1)
popped:     1   (left: 0)
```

The same output, at the cost of a `loop`, a `match` and a `break`, none of which say anything new. `while let` turns those five lines into one.

```senpai-visual
{"kind":"concept","labels":["pop()","Some(x)?","body","None","done"]}
```

The thing everybody misreads at first: the loop **ends the first time the pattern fails to match**. Nothing is skipped and retried; there is no hidden `continue` inside `while let`:

```rust
let mut countdown: u32 = 3;
while let Some(next) = countdown.checked_sub(1) {
    println!("countdown:  {next}");
    countdown = next;
}
```

```text
countdown:  2
countdown:  1
countdown:  0
```

At zero, `checked_sub(1)` answers `None` and that is the end of it. And because this is an ordinary loop, `break` and `continue` work in it exactly as they always did.

### `let ... else` — bind it, or leave

Now back to that guard. This function has to produce a number, but only one variant gives it a meaningful one:

```rust
fn gap_nested(entry: &Entry, latest: u32) -> u32 {
    if let Entry::Watching { episode } = entry {
        latest.saturating_sub(*episode)
    } else {
        0
    }
}
```

It works, but the function's real work has sunk a level — and it sinks another level for every guard you add. `let ... else` writes the same rule flat:

```rust
fn gap_flat(entry: &Entry, latest: u32) -> u32 {
    let Entry::Watching { episode } = entry else {
        return 0;
    };
    latest.saturating_sub(*episode)
}
```

```text
latest episode out: 12
Watching { episode: 7 }      nested 5   flat 5
Completed                    nested 0   flat 0
PlanToWatch                  nested 0   flat 0
```

The real difference isn't the shape, it's the **scope**: the name `let ... else` binds does not stay inside a block. It lands in the same scope an ordinary `let` would, so it's available from the next line to the end of the function. `if let` does not do that.

And now look at the family resemblance:

```rust
// 1.1.5 — a guard on a condition
if parts == 0 {
    return 0;
}

// 1.5.5 — a guard on a pattern
let Entry::Watching { episode } = entry else {
    return 0;
};
```

It's the same move: **dispose of the case that has no answer at the top, and leave.** The only difference is that the guard can now look at the *shape* of the data rather than only at a `bool` — and it unpacks what you needed while it's there.

### Why the `else` must diverge

If you write an `else` block that runs off its bottom, the compiler stops you. The full `E0308` is in the errors section; the sentence that matters is `expected !, found ()`.

The reason is one line of logic. `let Entry::Watching { episode } = entry else { ... };` has promised that after this line, `episode` exists. If the pattern doesn't match and the `else` block merely prints something and finishes, execution reaches the next line with `episode` holding nothing. So the only workable rule is that the `else` block **must never reach the next line**.

The name for that property is **diverging**, and you have already met its type in [1.1.4](../../01-foundations/04-functions-and-expressions/README.md): `!`, the never type.

| The `else` says | What it leaves | Its type |
|---|---|---|
| `return` or `return x` | the function | `!` |
| `break` | the loop | `!` |
| `continue` | this turn of the loop | `!` |
| `panic!("...")` or `todo!()` | the program | `!` |

There, `!` was a favour: because it never produces a value it could stand in for any type, which is how `todo!()` satisfies any signature. Here the same `!` is a **requirement** — the compiler says outright that `!` is what it expected. Same type, same meaning, this time doing the guarding.

### So which, when?

| You want | Write |
|---|---|
| every case handled | `match` |
| one case handled, the rest genuinely nothing | `if let` |
| one case handled, the rest sharing one answer | `if let ... else` |
| the value, or get out of here | `let ... else` |
| keep going while the pattern matches | `while let` |
| three shapes or more, all peers | `match`, always |

And the shape to recognise and not write:

```rust
if let Status::Watching { episode } = status {
    format!("on episode {episode}")
} else if let Status::Completed { rating } = status {
    format!("finished, {rating}/10")
} else {
    "dropped".to_string()
}
```

That's a `match` in a disguise — one that also left its safety net behind. Three peer shapes means write a `match` and let the compiler count.

> `let ... else` and the `?` operator solve overlapping problems: both write "if it isn't there, leave from here". `?` is specific to `Option` and `Result` and does the `return` for you; `let ... else` works with any pattern and you say how to leave. `?` gets its own lesson in [1.6.3](../../06-absence-and-failure/03-result-and-question-mark/README.md).

---

## Hands on

```sh
cargo run -p p1-05-05-if-let-while-let-let-else --example 01-one-arm-match
cargo run -p p1-05-05-if-let-while-let-let-else --example 02-while-let-drain
cargo run -p p1-05-05-if-let-while-let-let-else --example 03-let-else-guard
cargo run -p p1-05-05-if-let-while-let-let-else --example 04-the-silent-hole
```

Then the three broken ones:

```sh
cargo run -p p1-05-05-if-let-while-let-let-else --example 05-refutable-let --features broken
cargo run -p p1-05-05-if-let-while-let-let-else --example 06-else-must-diverge --features broken
cargo run -p p1-05-05-if-let-while-let-let-else --example 07-binding-escaped --features broken
```

Then try:

1. In `04-the-silent-hole`, add the `OnHold` variant and rebuild. Which function complained and which stayed quiet?
2. In `02-while-let-drain`, swap that `break` for a `continue`. What happens, and why?
3. In `03-let-else-guard`, replace the `return 0;` line with a `println!`. Read the error — it's the exact `E0308` in the next section.

---

## Errors you will meet

### `E0005` — a pattern that might not match, in a plain `let`

```text
error[E0005]: refutable pattern in local binding
 --> examples\05-refutable-let.rs:9:9
  |
9 |     let Some(score) = rating;
  |         ^^^^^^^^^^^ pattern `None` not covered
  |
  = note: `let` bindings require an "irrefutable pattern", like a `struct` or an `enum` with only one variant
  = note: for more information, visit https://doc.rust-lang.org/book/ch19-02-refutability.html
  = note: the matched value is of type `Option<u8>`
help: you might want to use `let...else` to handle the variant that isn't matched
  |
9 |     let Some(score) = rating else { todo!() };
  |                              ++++++++++++++++
```

**What the compiler is objecting to:** a plain `let` promises that after this line, the pattern's names exist. For that promise to be true every time, the pattern has to match **every time**. `Some(score)` doesn't; `None` is possible.

Here are two Rust words you'll keep meeting: a pattern that might not match is **refutable**, and one that always matches is **irrefutable**. A tuple, a struct, a plain name — all irrefutable, and all legal in a `let`. One specific variant of an enum is refutable, and isn't.

**The fix:** one of three, depending on what you actually meant:

```rust
if let Some(score) = rating {
    println!("score: {score}");
}
```

**Why that's the fix:** the error itself suggests another one, and it's right — look at the `else { todo!() }` in its message. It's saying "here's the hole, the decision is yours". If you want to leave the function when there's nothing there, `let ... else` is the exact answer; if you only want to skip, `if let`; and if both cases have work, `match`.

### `E0308` — the `else` block doesn't diverge

```text
error[E0308]: `else` clause of `let...else` does not diverge
  --> examples\06-else-must-diverge.rs:10:35
   |
10 |       let Some(score) = rating else {
   |  ___________________________________^
11 | |         println!("no rating yet");
12 | |     };
   | |_____^ expected `!`, found `()`
   |
   = note:   expected type `!`
           found unit type `()`
   = help: try adding a diverging expression, such as `return` or `panic!(..)`
   = help: ...or use `match` instead of `let...else`
```

**What the compiler is objecting to:** that `else` block finishes, and execution falls out of the bottom of it. The next line wants `score`, and `score` was never bound. So Rust says the block's type has to be `!` and what it found was `()`.

**The fix:**

```rust
let Some(score) = rating else {
    println!("no rating yet");
    return;
};
```

**Why that's the fix:** `return` has type `!`, so the block no longer "finishes" and the `let`'s promise holds. Notice that printing isn't forbidden — the `else` block may do whatever it likes, as long as it **leaves at the end**.

And read that line twice: `expected !, found ()`. That's the never type from [1.1.4](../../01-foundations/04-functions-and-expressions/README.md). There, `!` was a permission that satisfied any signature; here it's an obligation. The compiler's second help is worth reading too — "or use `match` instead" — because in a `match` every arm produces its own value and no such promise was ever made.

### `E0425` — the binding stayed inside the block

```text
error[E0425]: cannot find value `score` in this scope
  --> examples\07-binding-escaped.rs:15:25
   |
15 |     println!("outside: {score}");
   |                         ^^^^^ not found in this scope
```

**What the compiler is objecting to:** `score` was bound by an `if let`, and that name is alive only inside that `if let`'s block. Outside the braces there is no such name — not empty, not there at all.

**The fix:** if you genuinely need it after the block, then `if let` was the wrong tool:

```rust
let Some(score) = rating else { return };
println!("outside: {score}");
```

**Why that's the fix:** this is precisely what separates the two forms. `if let` is a block and its name stays inside the block; `let ... else` is a `let` and its name lands in the current scope. When you hit this error, the right question isn't "how do I get the name out?" but "which of these two did I want?".

---

## Exercises

### Warm up

What does this print?

```rust
let mut stack = vec![1, 2, 3];
while let Some(top) = stack.pop() {
    if top == 2 {
        continue;
    }
    println!("{top}");
}
```

<details>
<summary>Answer</summary>

```text
3
1
```

`continue` only skips the rest of this turn; the loop carries on because `stack.pop()` still answers `Some`. The only thing that ends the loop is the pattern failing to match.

</details>

<details>
<summary><code>if let</code> against <code>match</code>: which guarantee do you lose?</summary>

Exhaustiveness. `match` catches a missing case with `E0004`; `if let` drops a new variant silently into the `else`.

</details>

<details>
<summary>Why doesn't <code>let Some(x) = v;</code> compile when <code>let (a, b) = pair;</code> does?</summary>

Because `(a, b)` is irrefutable — a two-element tuple always has that shape. `Some(x)` is refutable, because `None` is possible. A plain `let` accepts only irrefutable patterns.

</details>

<details>
<summary>What type must the <code>else</code> block of a <code>let ... else</code> have, and why?</summary>

`!`, the never type. Because if execution came out of the bottom of that block, the next line would want a name that was never bound. So the block has to leave: `return`, `break`, `continue` or a panic.

</details>

<details>
<summary>Exactly when does <code>while let Some(x) = v.pop()</code> stop?</summary>

The first time `pop()` hands back something that doesn't match `Some(x)` — that is, `None`. Not a turn early, not a turn late, and no element is skipped.

</details>

<details>
<summary>How long does an <code>if let</code> binding live? And a <code>let ... else</code> one?</summary>

The first, to the end of the `if let` block. The second, to the end of the scope it was written in — like any other `let`. That difference is what makes `let ... else` the right tool for a guard.

</details>

### Repair

Fix all three broken examples:

1. `examples/05-refutable-let.rs` **two** ways: once with `if let`, once with `let ... else`. Then say which one matches that program's intent.
2. `examples/06-else-must-diverge.rs`, keeping the "no rating yet" message.
3. `examples/07-binding-escaped.rs` **two** ways: once by moving the `println!` inside the block, once by turning the `if let` into a `let ... else`. Which one changes something more than the layout?

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-05-05-if-let-while-let-let-else
```

Each wants one of today's forms. Once the tests are green, rewrite one of them as a `match` and put the two side by side: which reads better, and is your answer the same for all five?

### Build

Write a `pub fn shelf_report(entries: Vec<Entry>) -> String` producing a one-line summary: how many are being watched, how many are finished, and the highest rating seen — in a format you choose and state in the function's doc comment.

Then write it twice: once with a `match` inside the loop, once with a handful of `if let`s. Which came out shorter? And which one tells you about it when you add a new variant to `Entry`?

### Challenge (optional)

**Part one.** What does this print? Guess, then run it:

```rust
let mut queue = vec![Some(1), None, Some(3)];
while let Some(item) = queue.pop() {
    println!("{item:?}");
}
```

That outer `Some` belongs to `pop()`, not to the element. Now rewrite it so it prints only the numbers and skips the empty ones — and still reaches the end of the queue.

**Part two.** Run this and read its warning in full:

```rust
let n = 5;
if let x = n {
    println!("irrefutable: {x}");
}
```

The compiler says this `if let` is useless. Explain why using the vocabulary of `E0005` — and say what happens if you write the same thing as a `while let`.

**Part three.** (This one reaches forward.) In the 2024 edition you can chain several conditions and patterns with `&&` inside one `if let`; they're called let-chains. This repository is on the 2021 edition, so it won't compile here — but find and read the release note, and say which of today's nested `if let`s it would have removed.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `if let` | a `match` with one arm | when only one case has work |
| `if let ... else` | the same, with the second arm | when the other case has work too |
| `while let` | while the pattern keeps matching | draining a collection |
| `let ... else` | bind it, or leave | a guard on a pattern |
| refutable | a pattern that might not match | `match`, `if let`, `let ... else` |
| irrefutable | a pattern that always matches | the only kind a plain `let` allows |
| diverging | execution does not come out of here | the `else` of a `let ... else` |
| `E0005` | a refutable pattern in a plain `let` | the error suggests `let...else` itself |
| `E0308` in `let...else` | expected `!`, found `()` | the `else` block has to leave |

### What you now know

- `if let` is exactly a `match` with the "do nothing" arm you didn't write.
- `if let`, like `if`, is an expression and can produce a value.
- Turning a `match` into an `if let` costs you exhaustiveness, and that trade should be deliberate.
- `while let` ends the first time the pattern fails, not a turn later.
- `let ... else` binds into the current scope; `if let` binds inside its block.
- A `let ... else` block must diverge, and its type is `!` — the never type.
- An irrefutable pattern always matches, and only that kind is allowed in a plain `let`.

### What comes back later

- **`Option`, and everything you can do with it** — [1.6 — Absence and failure](../../06-absence-and-failure/README.md)
- **The methods that replace many of these `if let`s** — [1.6.2 — `Option` combinators](../../06-absence-and-failure/02-option-combinators/README.md)
- **The `?` operator, `let ... else`'s close relative** — [1.6.3 — `Result` and `?`](../../06-absence-and-failure/03-result-and-question-mark/README.md)
- **Panicking, as one of the ways to diverge** — [1.6.4 — Panic vs `Result`](../../06-absence-and-failure/04-panic-vs-result/README.md)
- **Advanced patterns and the other places patterns appear** — [Phase 2 — Pattern matching in depth](../../../phase2-intermediate/08-rust-toolbox/01-pattern-matching-depth/README.md)

### Can you explain?

- Translate an `if let` into a `match`, arm by arm.
- What do you lose when you turn a `match` into an `if let`? Give an example you'd actually run into.
- When does `while let Some(x) = v.pop()` stop, and why exactly there?
- Why must the `else` block of a `let ... else` diverge? Answer using the never type.
- What's the difference in scope between a name bound by `if let` and one bound by `let ... else`?
- What do "refutable" and "irrefutable" mean, and which is allowed in a plain `let`?
- Name a situation where a `match` beats all three of these, and say why.

---

## Going further

- [The Rust Book — Concise control flow with `if let`](https://doc.rust-lang.org/book/ch06-03-if-let.html) — the same ground, officially.
- [The Rust Reference — `if` expressions](https://doc.rust-lang.org/reference/expressions/if-expr.html) — the exact rules, including things we didn't say here.
- [The Rust 1.65 release notes](https://blog.rust-lang.org/2022/11/03/Rust-1.65.0.html) — where `let ... else` arrived, with the reasoning behind it. Short, and worth reading.
- [`std::vec::Vec::pop`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.pop) — the method today's entire `while let` was riding on.
