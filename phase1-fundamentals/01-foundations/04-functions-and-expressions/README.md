# 1.1.4 — Functions and expressions

## At a glance

After this lesson you can:

- Write a function with typed parameters and a return type, and say why Rust never infers a parameter's type.
- Tell a statement from an expression, and say what a block is worth.
- Say what one extra semicolon does to a function's return type — and why that's the commonest beginner error there is.
- Explain why `todo!()` compiles inside a function that promised to return a number.

**Time:** ~40 minutes · **Prerequisites:** [1.1.3 — Compound types and destructuring](../03-compound-types-and-destructuring/README.md)

---

## Why this matters

You know how to write a function. This lesson isn't about `fn` syntax — that part takes ten minutes. It's about a deeper difference that will cost you months of arguing with the compiler if you skip past it.

In Python the language is split in half. One half is **statements** — things that do something: `x = 5`, `if`, `for`, `return`. The other half is **expressions** — things that are worth something: `5`, `x + 1`, `f()`. The two don't mix. That's why Python had to invent separate syntax so you could put a decision into a value:

```python
label = "even" if n % 2 == 0 else "odd"
```

That mid-line `if ... else` isn't a normal `if`. It's a second construct, added to the language purely because the real `if` was a statement and wasn't worth anything.

Rust barely has that split. **In Rust almost everything is an expression**: a block, a comparison, an `if`, a `match`. They're all worth something and they can all go into a `let`. Which means the language needs no second special construct, and the code you write comes out shorter and more direct.

The price is one rule you have to make automatic: **a semicolon turns an expression into a statement.** One missing or extra semicolon changes your function's return type. Recognising "expected `u32`, found `()`" on sight gets you tens of minutes of your life back.

---

## The concept

### Writing a function

```rust
fn area(width: u32, height: u32) -> u32 {
    width * height
}
```

There are four things in that signature and all four are required:

| Part | Meaning |
|---|---|
| `fn` | this is a function |
| `area` | its name, in `snake_case` |
| `(width: u32, height: u32)` | the parameters — **each explicitly typed** |
| `-> u32` | what it gives back |

**Rust never infers a parameter's type.** It infers constantly inside the body, and in `let` too, but not in the signature — deliberately. The signature is a contract between you and everyone who calls the function, and a contract the other side has to guess at isn't one. There's a good side effect too: when a function has a type error, the error stays inside that function instead of spreading through the whole file.

**Order doesn't matter.** This is perfectly fine:

```rust
fn main() {
    println!("{}", area(3, 4));
}

fn area(width: u32, height: u32) -> u32 {
    width * height
}
```

Rust reads the whole file before compiling any of it. Nothing has to be "defined above".

**No `->` means `()`.** A function that returns nothing actually returns the unit type from the last lesson:

```rust
fn announce(order_id: u32) {
    println!("order {order_id} received");
}
```

```text
           order 7 received
announce:  ()
```

### Statements versus expressions

This is the centre of the lesson, and the definition is one line:

> **A statement does something. An expression is worth something.**

```rust
let total = 3 * 4;
```

That whole line is a **statement**: it does something (makes a binding) and is itself worth nothing. But `3 * 4`, sitting on the right of it, is an **expression**: it's worth 12.

Expressions you've already met: literals, `a + b`, `a == b`, `readings[0]`, `sample.1`, `area(3, 4)`. All of them are worth something and all of them can go into a `let`.

### A block is an expression too

Here's where Rust parts company with Python:

```rust
let adjusted = {
    let base = 100;
    let bonus = 20;
    base + bonus
};
```

```text
adjusted:  120
```

Those braces make a **block**, and a block is an expression. Its value is the last expression inside it — the one without a semicolon.

Python has no equivalent. If you want to compute something in a few steps and put the result in a variable, you either leave the intermediate names lying around in the enclosing scope or you write a small function. Here, `base` and `bonus` don't exist at all outside those braces.

### And now the semicolon rule

The same block, with one extra semicolon on the last line:

```rust
let nothing = {
    let base = 100;
    let bonus = 20;
    base + bonus;
};
```

```text
nothing:   ()
```

Not `120`. `()`.

The semicolon turned that expression into a statement, the block had no expression left to be worth, and its value became "nothing". The compiler warns you when you run this, and the warning is the lesson:

```text
warning: unused arithmetic operation that must be used
  --> examples\02-expressions.rs:24:9
   |
24 |         base + bonus;
   |         ^^^^^^^^^^^^ the arithmetic operation produces a value
```

"Produces a value" — and you threw it away.

### The tail expression is the return value

A function body is a block, so exactly the same rule applies:

```rust
fn with_tail_expression(n: u32) -> u32 {
    n * 3
}

fn with_return(n: u32) -> u32 {
    return n * 3;
}
```

```text
implicit:  30
explicit:  30
```

The same function twice. They compile to the same machine code. **The first form is the Rust idiom** and Rust programmers read it faster.

So what's `return` for? **Leaving early** — when you find out halfway through that the answer is settled and you don't want to run the rest. That needs a condition, and conditions are the next lesson, so leave `return` alone until then.

> **Commit this one, because you're going to see it a lot:** if the compiler says "expected `u32`, found `()`", look first for an extra semicolon on your function's last line. Nine times in ten that's it.

### The never type — and why `todo!()` compiles

You've been staring at this for three lessons:

```rust
pub fn clamped_add(a: u8, b: u8) -> u8 {
    todo!("add them in the way that clamps at the maximum")
}
```

That function promised a `u8`. Its body has no `u8` anywhere in it. Why does it compile?

Because `todo!()` has a type of its own: `!`, **the never type**. The type of an expression that produces no value because it doesn't finish. `todo!()` panics; execution never comes out the other side.

And that's where the trick falls into place: if an expression never produces a value, then "the value it produces" can be **any type at all** without telling a lie. So `!` fits wherever a type is wanted — `u8`, `String`, anything.

You've met, or will meet, the whole family:

| Expression | For |
|---|---|
| `todo!()` | haven't written it yet |
| `unimplemented!()` | not going to write it |
| `panic!()` | things have gone wrong here |
| `unreachable!()` | execution can't get here |
| `std::process::exit(1)` | the program ends here |

All five have type `!`. That's why every exercise stub in this course compiles without a line of real code in it. **`todo!()` isn't a placeholder; it's a promise the compiler believes.**

### Operators are expressions too

Easy to skim past, and the consequence is large:

```rust
let year = 2024;
let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
```

```text
2024 leap: true
```

`year % 4 == 0` is a `bool` in exactly the sense that `2 + 2` is an `i32`. A three-part rule in one line, **with no `if` in it at all**.

`&&` and `||` **short-circuit**: if the left side settles the answer, the right side is never evaluated. It doesn't matter here. It matters a great deal when the right side is an expensive call, or one that might panic.

### Naming

Functions and variables in `snake_case`, constants in `SCREAMING_SNAKE_CASE`, types in `PascalCase`. You don't have to memorise it — `cargo clippy`, which you set up in [0.6](../../../phase0-setup/06-tooling-clippy-fmt-rust-analyzer/README.md), reminds you every time.

---

## Hands on

```sh
cargo run -p p1-01-04-functions-and-expressions --example 01-functions
cargo run -p p1-01-04-functions-and-expressions --example 02-expressions
cargo run -p p1-01-04-functions-and-expressions --example 03-the-never-type
```

`02-expressions` produces a warning on purpose. **Read it** — it's the semicolon rule, explained by the compiler itself.

Then look at the two broken ones:

```sh
cargo run -p p1-01-04-functions-and-expressions --example 04-stray-semicolon --features broken
cargo run -p p1-01-04-functions-and-expressions --example 05-missing-argument --features broken
```

Then try:

1. In `01-functions`, delete `-> u32` from `area`. What error do you get, and why does it make sense?
2. In `02-expressions`, add a semicolon to the `base + bonus` line of the first block. What prints now?
3. In `03-the-never-type`, uncomment the last line of `main` and run it. Is the message you see one you wrote?

---

## Errors you will meet

### `E0308` — one extra semicolon

The commonest Rust error for newcomers, by a distance.

```text
error[E0308]: mismatched types
  --> examples\04-stray-semicolon.rs:11:23
   |
11 | fn tripled(n: u32) -> u32 {
   |    -------            ^^^ expected `u32`, found `()`
   |    |
   |    implicitly returns `()` as its body has no tail or `return` expression
12 |     n * 3;
   |          - help: remove this semicolon to return this value
```

**What the compiler is objecting to:** the signature promised a `u32`. The body ends in a statement, so the whole block is worth `()`.

**The fix:** delete the semicolon.

**Why that's the fix:** notice how well written this error is. It separates three things: where the promise was made (`-> u32`), why it broke (`implicitly returns ()`), and which character is at fault. The error's *location* is the signature, but the *help* is on the semicolon — and the help always points at the real place. Get in the habit of reading errors to the end; that's what [0.5](../../../phase0-setup/05-reading-compiler-errors/README.md) was about.

### A statement where an expression was wanted

This one has no example file in the lesson: rustfmt cannot format a file it cannot parse, so committing it would break `cargo fmt --check`. Paste it into a scratch file to see it yourself.

```text
error: expected expression, found `let` statement
 --> statement-as-value.rs:4:14
  |
4 |     let x = (let y = 6);
  |              ^^^
  |
  = note: only supported directly in conditions of `if` and `while` expressions
```

**What the compiler is objecting to:** the right of a `let` has to be an expression, and `let y = 6` is a statement. A statement isn't worth anything, so there's nothing to bind to `x`.

**The fix:** make it two lines.

**Why that's the fix:** this is one of the few places Rust kept the statement/expression split, and it's on purpose: `let x = y = 6` compiles in C and almost always means the programmer wanted `==`. That trailing note — "only supported directly in conditions of `if` and `while`" — is pointing at `if let`, which arrives in [1.5.5](../../05-your-own-types/05-if-let-while-let-let-else/README.md).

### `E0061` — the argument count doesn't match

```text
error[E0061]: this function takes 2 arguments but 1 argument was supplied
  --> examples\05-missing-argument.rs:8:20
   |
 8 |     println!("{}", area(3));
   |                    ^^^^--- argument #2 of type `u32` is missing
   |
note: function defined here
  --> examples\05-missing-argument.rs:11:4
   |
11 | fn area(width: u32, height: u32) -> u32 {
   |    ^^^^             -----------
help: provide the argument
   |
 8 |     println!("{}", area(3, /* u32 */));
   |                          +++++++++++
```

**What the compiler is objecting to:** the signature wants two arguments and one arrived. Rust has no default arguments.

**The fix:** supply the second argument.

**Why that's the fix:** the `note: function defined here` takes you straight to the definition and underlines *the specific parameter* you left out. If it's someone else's function, that note saves you the jump.

### An untyped parameter

```text
error: expected one of `:`, `@`, or `|`, found `)`
 --> untyped-parameter.rs:5:12
  |
5 | fn double(n) -> u32 {
  |            ^ expected one of `:`, `@`, or `|`
  |
help: if this is a parameter name, give it a type
  |
5 | fn double(n: TypeName) -> u32 {
  |            ++++++++++
```

**What the compiler is objecting to:** there's no such thing as an untyped parameter in Rust. This isn't even a type error — the code can't be parsed past this point.

**The fix:** write the type.

**Why that's the fix:** look at that `TypeName` — the compiler put in an invented type because it genuinely has no idea what you meant. Unlike `let`, which infers from the value, a signature has nothing to infer from.

---

## Exercises

### Warm up

<details>
<summary>What's the difference between a statement and an expression, in one sentence?</summary>

A statement does something and is worth nothing; an expression is worth something. `let x = 5;` is a statement and `5` is an expression.

</details>

<details>
<summary>What is <code>let a = { 1 + 1 };</code>? And <code>let b = { 1 + 1; };</code>?</summary>

`a` is `2`, an `i32`. `b` is `()`, because the semicolon turned the expression into a statement and left the block with nothing to be worth.

</details>

<details>
<summary>Why doesn't Rust infer parameter types when it infers <code>let</code> types happily?</summary>

Because the signature is the function's public contract. If it were inferred, changing one line inside the body could silently change the signature and break every caller. Inside the body there's no such risk.

</details>

<details>
<summary>What does <code>fn f() { }</code> return?</summary>

`()` — the unit type. No `-> ...` means exactly that, and an empty body is worth exactly that.

</details>

<details>
<summary>Why does <code>todo!()</code> compile inside a function declared <code>-> u8</code>?</summary>

Because its type is `!`, the never type: it produces no value ever, so it can stand in for any type without telling a lie.

</details>

<details>
<summary>What's the commonest cause of "expected <code>u32</code>, found <code>()</code>"?</summary>

An extra semicolon on the function's last line. Look there first.

</details>

### Repair

Fix `examples/04-stray-semicolon.rs`. It's one character.

Then break it a different way on purpose: put the semicolon back and add `return` instead. Does it compile? Why?

Then delete `-> u32` from `tripled` entirely but keep the semicolon. Now it compiles — and that's the interesting part: code that raises no error isn't necessarily the code you meant.

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-01-04-functions-and-expressions
```

All five can be written with no `return` and no condition. If you catch yourself reaching for `if`, look at `is_leap_year` again — a decision is an expression too.

### Build

Write two functions:

```rust
pub fn bmi(weight_kg: f64, height_cm: f64) -> f64
pub fn bmi_is_healthy(weight_kg: f64, height_cm: f64) -> bool
```

BMI is weight divided by the square of the height **in metres**. `bmi_is_healthy` is true when the BMI is between 18.5 and 25 — and it must call `bmi` rather than repeating the formula.

Then write a sentence on why the second function shouldn't repeat the formula. The answer isn't just "duplication is bad"; it's something specific about these two functions.

### Challenge (optional)

**Part one.** Without running it, write down the type and value of each:

```rust
let a = { 1 + 1 };
let b = { 1 + 1; };
let c = { let d = 5; };
let e = ();
let f = { { { 7 } } };
```

Then run it and see how many you got. Which of them share a type?

**Part two.** Compile this:

```rust
fn stop() -> ! {
    todo!("this function never returns")
}
```

Then answer: why is `-> !` allowed when `fn stop() -> ! { 5 }` isn't? And what happens if you swap the `todo!()` for `println!("bye")`?

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| signature | name, typed parameters, return type | every function you write |
| statement | does something, worth nothing | `let x = 5;` |
| expression | worth something | `3 * 4`, `a == b`, a block |
| block expression | `{ ... }`, worth its last expression | grouping a few steps |
| tail expression | the body's last expression, no semicolon | the return value |
| `return` | explicit exit | early exit only |
| `()` | the unit type, "no value" | no `->` |
| `!` | the never type | `todo!`, `panic!`, `unreachable!` |
| short-circuit | `&&` and `||` skip the right side | chained checks |

### What you now know

- Parameters are always typed, and the order functions appear in a file doesn't matter.
- A block is an expression, worth its last expression.
- A semicolon turns an expression into a statement, and makes the block worth `()`.
- The tail expression is the return value; `return` is for leaving early.
- `!` is the type with no values, which is why it fits anywhere.
- `&&` and `||` produce values and short-circuit — a decision doesn't require an `if`.

### What comes back later

- **`if` and `match`, which are expressions too** — [1.1.5 — Control flow](../05-control-flow/README.md)
- **Early `return`, as it's actually used** — [1.1.5 — Control flow](../05-control-flow/README.md)
- **`?`, which is a hidden early return** — [1.6.3 — `Result` and the question mark](../../06-absence-and-failure/03-result-and-question-mark/README.md)
- **`!` where it really earns its keep** — [1.6.4 — Panic versus `Result`](../../06-absence-and-failure/04-panic-vs-result/README.md)
- **Functions that are themselves values** — [Phase 2 — Iterators and closures](../../../phase2-intermediate/02-iterators-and-closures/README.md)

### Can you explain?

- What separates a statement from an expression, and which one is `let x = 5;`?
- What is a block worth? And if its last line has a semicolon?
- Why does Rust infer `let` types but not parameter types?
- What is `!`, and why does `todo!()` satisfy any signature?
- What does "expected `u32`, found `()`" mean, and where do you look first?
- How do you write a three-part rule without an `if`?

---

## Going further

- [The Rust Book — Functions](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html) — the same ground, officially.
- [The Rust Reference — Expressions](https://doc.rust-lang.org/reference/expressions.html) — the complete list of everything that's an expression in Rust. It's long, and its being long is the point.
- [The Rust Reference — The never type](https://doc.rust-lang.org/reference/types/never.html) — short, and comprehensible now that you've done this lesson.
