# 1.6.3 — `Result` and the question mark

## At a glance

After this lesson you can:

- Say exactly what `Result<T, E>` is, and build one by hand with `Ok` and `Err`.
- Read the real `#[must_use]` warning and say why ignoring a `Result` makes noise.
- Choose the right one of `.map`, `.map_err`, `.and_then`, `.unwrap_or`, and `.ok()` for a given situation.
- Replace a `match` on a `Result` with `?`, say exactly what `?` expands to, and write a `main` that returns `Result` itself.

**Time:** ~55 minutes · **Prerequisites:** [1.6.2 — `Option` combinators](../02-option-combinators/README.md)

---

## Why this matters

[1.1.4](../../01-foundations/04-functions-and-expressions/README.md) made a promise: early `return` has a shorthand, an operator that does the same thing invisibly. This is the lesson that keeps it.

There is an older debt too. Since Phase 0, every time you wrote `.parse()` — every time you turned a string into a number — you saw the result but never actually opened up its real type. This lesson opens that too.

Both debts come from the same place: `Result<T, E>`.

The last lesson closed out `Option`: one enum for "maybe there's nothing here." But plenty of failures want to say more than "there isn't one." When `.parse()` fails on `"abc"`, the useful news isn't "there was no number" — it's *why*: an invalid digit turned up. A `None` throws that reason away. Rust has a second enum that keeps it.

If you're coming from Python, the mental match is `try`/`except`: a function either returns a value or raises an exception, and the signature shows neither — you have to read the docs or the code to know what can fail. `Result<T, E>` folds that same possibility of failure into the return type itself: the moment you see a `Result` in a signature, you know it might fail, and the compiler forces you to look at that case before you can reach the value. The similarity ends there — Rust has no exceptions in the Python sense; the closest thing is `panic!`, and choosing between it and `Result` is the subject of [1.6.4](../04-panic-vs-result/README.md).

---

## The concept

### `Option` said "maybe nothing"; `Result` says "maybe, and here's why"

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

Same shape as `Option<T>` — an ordinary enum, exactly the kind you saw in [1.5.3](../../05-your-own-types/03-enums-as-data/README.md) — with one difference: instead of an empty arm (`None`), the second arm (`Err`) carries a value too. `T` is the type of what you get when things work; `E` is the type of the reason you get when they don't.

| | When it doesn't work | Error type |
|---|---|---|
| `Option<T>` | `None` — no information at all | none |
| `Result<T, E>` | `Err(E)` — a value with a reason | you choose it |

### Building one by hand: `Ok` and `Err`

```rust
fn safe_divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("cannot divide by zero".to_string())
    } else {
        Ok(a / b)
    }
}

let good = safe_divide(10.0, 2.0);
let bad = safe_divide(10.0, 0.0);
println!("good: {good:?}");
println!("bad:  {bad:?}");
```

```text
good: Ok(5.0)
bad:  Err("cannot divide by zero")
```

Here `E` is `String` — a simple choice to start with. The function has two ways out, and both are explicit: either a usable `f64`, or a sentence saying why not.

### Matching on it, and two small shortcuts

```rust
match safe_divide(20.0, 4.0) {
    Ok(value) => println!("matched ok:  {value}"),
    Err(reason) => println!("matched err: {reason}"),
}

println!("is_ok:  {}", good.is_ok());
println!("is_err: {}", bad.is_err());
```

```text
matched ok:  5
is_ok:  true
is_err: true
```

`match` on a `Result` follows the exact same rule from [1.5.4](../../05-your-own-types/04-match-in-depth/README.md): two variants, two arms, or it doesn't compile. `.is_ok()` and `.is_err()` are for when you only need the shape, not the value or the reason.

### Why ignoring a `Result` makes noise: `#[must_use]`

```rust
// no `let`, no `?`, nothing — the Result is thrown away
safe_divide(1.0, 0.0);
```

```text
warning: unused `Result` that must be used
  --> phase1-fundamentals\06-absence-and-failure\03-result-and-question-mark\examples\01-ok-err-and-must-use.rs:36:5
   |
36 |     safe_divide(1.0, 0.0);
   |     ^^^^^^^^^^^^^^^^^^^^^
   |
   = note: this `Result` may be an `Err` variant, which should be handled
   = note: `#[warn(unused_must_use)]` (part of `#[warn(unused)]`) on by default
help: use `let _ = ...` to ignore the resulting value
   |
36 |     let _ = safe_divide(1.0, 0.0);
   |     +++++++
```

The standard library's `Result` is marked with an attribute called **must-use (`#[must_use]`)**. It means: "if you don't put this value somewhere, at least let the compiler tell you once — you may have just silently dropped an `Err`." This is a **warning**, not an error: the code above compiles and runs, it just says its piece. The way to silence it deliberately is exactly what the compiler's own help suggests: `let _ = ...`, which says "I saw it, and I'm throwing it away on purpose."

### `.parse()` finally explained

```rust
let good: Result<u32, _> = "42".parse::<u32>();
let letters: Result<u32, _> = "abc".parse::<u32>();
let negative: Result<u32, _> = "-5".parse::<u32>();
println!("good:     {good:?}");
println!("letters:  {letters:?}");
println!("negative: {negative:?}");
```

```text
good:     Ok(42)
letters:  Err(ParseIntError { kind: InvalidDigit })
negative: Err(ParseIntError { kind: InvalidDigit })
```

Now the real type can be said out loud: `.parse::<u32>()` returns a `Result<u32, ParseIntError>`, not an `Option<u32>`. That `::<u32>` you've been writing since Phase 0 — the **turbofish** — tells `.parse()` which `T` to aim for, because the method itself is generic and nothing else pins that down. `u32` doesn't accept negative numbers, so `"-5"` fails exactly the way `"abc"` does — same error variant, same message.

```rust
if let Err(e) = "abc".parse::<u32>() {
    println!("message: {e}");
}
```

```text
message: invalid digit found in string
```

`ParseIntError` implements `Display`, so `{e}` (not `{e:?}`) gives the same sentence `.to_string()` would. You'll see this again a few lines down, in the combinators.

### `Result`'s combinators: five ways around `match`

The same combinators you saw on `Option` in [1.6.2](../02-option-combinators/README.md) exist on `Result` too — with one difference: some of them reach into `Err` as well.

```rust
println!("{:?}", safe_divide(10.0, 2.0).map(|v| v * 100.0));
println!("{:?}", safe_divide(10.0, 0.0).map(|v| v * 100.0));

let renamed = safe_divide(10.0, 0.0).map_err(|e| format!("division failed: {e}"));
println!("{renamed:?}");
```

```text
Ok(500.0)
Err("cannot divide by zero")
Err("division failed: cannot divide by zero")
```

`.map()` works on `Ok` and leaves `Err` untouched — the exact shape `Option::map` had. `.map_err()` is its mirror image: it works on `Err` and leaves `Ok` untouched. When you want to change the error type (say, from `ParseIntError` to `String`, exactly like a few lines above), this is the tool.

```rust
println!(
    "{:?}",
    safe_divide(10.0, 2.0).and_then(|v| safe_divide(v, 5.0))
);
println!(
    "{:?}",
    safe_divide(10.0, 0.0).and_then(|v| safe_divide(v, 5.0))
);

println!("{}", safe_divide(10.0, 2.0).unwrap_or(0.0));
println!("{}", safe_divide(10.0, 0.0).unwrap_or(0.0));

println!("{:?}", safe_divide(10.0, 2.0).ok());
println!("{:?}", safe_divide(10.0, 0.0).ok());
```

```text
Ok(1.0)
Err("cannot divide by zero")
5
0
Some(5.0)
None
```

`.and_then()` chains a second fallible step; if the first was `Err`, the second step's closure is never even called. `.unwrap_or()` pulls out a plain value, with a fallback for `Err` — no panic possible. And `.ok()` deliberately throws the error reason away and hands back an `Option`; reach for it once you genuinely no longer care why something failed.

### The `?` operator — a hidden early return

This is what [1.1.4](../../01-foundations/04-functions-and-expressions/README.md) was pointing at. By hand, first:

```rust
fn chained_by_hand(a: f64, b: f64, c: f64) -> Result<f64, String> {
    let step1 = match safe_divide(a, b) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };
    let step2 = match safe_divide(step1, c) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };
    Ok(step2)
}
```

Every step needs a `match`: on `Ok`, pull the value out and keep going; on `Err`, return right now from the function with that same `Err`. Now the same function, word for word the same logic, with `?`:

```rust
fn chained_with_question_mark(a: f64, b: f64, c: f64) -> Result<f64, String> {
    let step1 = safe_divide(a, b)?;
    let step2 = safe_divide(step1, c)?;
    Ok(step2)
}
```

`expr?` is exactly the `match` above, compressed into one character: **if `expr` is `Ok(v)`, replace the whole expression with `v` and keep going; if it's `Err(e)`, return from the current function right now with `Err(e)`.** Nothing new is happening — you just no longer have to write each step by hand.

```senpai-visual
{"kind":"result","labels":["expr?","Ok(v)","continue","Err(e) → return"]}
```

```rust
println!("by hand,  ok: {:?}", chained_by_hand(100.0, 2.0, 5.0));
println!("by hand,  err: {:?}", chained_by_hand(100.0, 0.0, 5.0));
println!(
    "with `?`, ok: {:?}",
    chained_with_question_mark(100.0, 2.0, 5.0)
);
println!(
    "with `?`, err: {:?}",
    chained_with_question_mark(100.0, 2.0, 0.0)
);
```

```text
by hand,  ok: Ok(10.0)
by hand,  err: Err("cannot divide by zero")
with `?`, ok: Ok(10.0)
with `?`, err: Err("cannot divide by zero")
```

Two functions, same inputs, same output. Only the amount of code differs.

Two things before moving on: `?` does exactly the same thing on `Option<T>` (returning `None` early instead of `Err`) — and when the error types on either side of `?` don't match, `?` tries to convert between them using the `From` trait. Everywhere here uses one error type (`String`), so that conversion never had to happen. `From` itself, and that automatic conversion, is the subject of [1.6.5](../05-from-and-error-conversion/README.md).

### `main` can return `Result` too

```rust
fn main() -> Result<(), String> {
    let result = chained_with_question_mark(100.0, 0.0, 5.0)?;
    println!("unreachable: {result}");
    Ok(())
}
```

```text
Error: "cannot divide by zero"
```

`?` only works inside functions that themselves return `Result` (or `Option`) — and that includes `main`. Here `main`'s signature has been changed to `Result<(), String>`, so `?` is allowed inside it. Because `chained_with_question_mark` returns `Err` this time, `?` immediately returns from `main` too with that same `Err` — the `println!("unreachable: ...")` line never runs.

When `main` returns `Err`, Rust prints it with `{:?}` (which is why the message is quoted) and the process ends with exit code 1, not 0. This is exactly what a command-line program needs to report failure to whatever shell called it. If `chained_with_question_mark` had succeeded, `main` would have finished quietly with `Ok(())` and exit code 0.

---

## Hands on

```sh
cargo run -p p1-06-03-result-and-question-mark --example 01-ok-err-and-must-use
cargo run -p p1-06-03-result-and-question-mark --example 02-parse
cargo run -p p1-06-03-result-and-question-mark --example 03-combinators
cargo run -p p1-06-03-result-and-question-mark --example 04-question-mark
```

The last one ends with exit code 1 — that's deliberate; it's the exact point you just read about.

Then the two broken ones:

```sh
cargo run -p p1-06-03-result-and-question-mark --example 05-unwrap-panics --features broken
cargo run -p p1-06-03-result-and-question-mark --example 06-question-mark-needs-result --features broken
```

Then try:

1. In `01-ok-err-and-must-use`, change the last line to `let _ = safe_divide(1.0, 0.0);`. Does the warning go away? Why?
2. In `03-combinators`, swap `.map_err()` and `.map()` (that is, call `.map_err()` on the `Ok` case too). What gets printed?
3. In `04-question-mark`, change the arguments in the last call to `chained_with_question_mark` so both divisions succeed. What do the output and exit code become now?

---

## Errors you will meet

### Panic — `called `Result::unwrap()` on an `Err` value`

```text
thread 'main' (17828) panicked at phase1-fundamentals\06-absence-and-failure\03-result-and-question-mark\examples\05-unwrap-panics.rs:19:40:
called `Result::unwrap()` on an `Err` value: "cannot divide by zero"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**What the compiler (really, the runtime) is objecting to:** `.unwrap()` on `Ok` hands back the value; on `Err` it panics. The message is exactly what you put inside `Err(...)` — that same `"cannot divide by zero"` written in `safe_divide`, now sitting inside the panic itself.

**The fix:** use `match`, `?`, or one of the combinators above instead of `.unwrap()` — whichever actually handles the `Err` path instead of assuming it never happens.

**Why that's the fix:** `.unwrap()` says "I'm sure this is `Ok`; if it isn't, stop everything." That judgment is sometimes right (your own test, a throwaway example) and sometimes wrong (user input). Telling the two apart is the subject of [1.6.4](../04-panic-vs-result/README.md).

### `E0277` — `?` outside a function that returns `Result`

```text
error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option` (or another type that implements `FromResidual`)
  --> phase1-fundamentals\06-absence-and-failure\03-result-and-question-mark\examples\06-question-mark-needs-result.rs:16:39
   |
13 | fn main() {
   | --------- this function should return `Result` or `Option` to accept `?`
...
16 |     let value = safe_divide(10.0, 0.0)?;
   |                                       ^ cannot use the `?` operator in a function that returns `()`
   |
help: consider adding return type
   |
13 ~ fn main() -> Result<(), Box<dyn std::error::Error>> {
14 |     // `main` here returns `()`, not a `Result` — so there is nowhere for
...
17 |     println!("{value}");
18 +     Ok(())
   |

For more information about this error, try `rustc --explain E0277`.
```

**What the compiler is objecting to:** `?` needs somewhere to put an `Err` — a `Result` or `Option` return type on the same function. Here `main`'s return type is `()`, so there's nowhere for that `Err` to go, and the compiler says exactly that: "this function should return `Result` or `Option` to accept `?`".

**The fix:** change `main`'s signature to `Result<(), E>`, as you saw in "The concept."

**Why that's the fix:** the compiler itself suggests another option too: `Result<(), Box<dyn std::error::Error>>`. **`Box<dyn Error>`** is a catch-all error type — anything implementing the standard `Error` trait fits inside it, so you no longer need every error in the program to be the same type. For a small program or a `main`, this is a common quick-and-dirty choice. All you needed here was the name; the `Error` trait itself, and exactly how `Box<dyn Error>` works, come up in [Custom error types, Phase 2](../../../phase2-intermediate/04-error-handling-and-lifetimes/01-custom-error-types/README.md).

---

## Exercises

### Warm up

<details>
<summary>What does <code>Result&lt;f64, String&gt;</code> mean, in one sentence?</summary>

Either a usable `f64` (`Ok`), or a `String` saying why not (`Err`) — exactly one of the two, never neither and never both.

</details>

<details>
<summary>Does this compile?

```rust
fn area(width: u32) -> Result<u32, String> {
    width * width
}
```
</summary>

No. The tail expression has type `u32`, but the signature promised `Result<u32, String>`. The same error you saw in [1.1.4](../../01-foundations/04-functions-and-expressions/README.md) ("expected `Result<u32, String>`, found `u32`"), this time with an enum instead of a plain number. You need `Ok(width * width)`.

</details>

<details>
<summary>What does <code>"7".parse::&lt;u32&gt;()</code> print with <code>{:?}</code>?</summary>

`Ok(7)`.

</details>

<details>
<summary>

```rust
fn h(n: i32) -> Result<i32, String> {
    if n < 0 {
        return Err("negative".to_string());
    }
    Ok(n * 2)
}
println!("{:?}", h(-3));
```

What does this print?
</summary>

`Err("negative")`. `n < 0`, so it returns early with `return`; the `Ok(n * 2)` line never runs.

</details>

<details>
<summary>Does <code>let _ = safe_divide(1.0, 0.0);</code> also produce the <code>unused_must_use</code> warning?</summary>

No. `let _ = ...` is exactly what the compiler's own help suggests for silencing the warning deliberately — it says "I saw it, and I'm throwing it away on purpose," not "I forgot about it."

</details>

### Repair

Fix `examples/06-question-mark-needs-result.rs` **two** ways:

1. By changing `main`'s signature to `Result<(), String>` — the way you saw in "The concept."
2. Without touching `main`'s signature at all — handle the `Result` some other way (`match`, or one of the combinators) so `?` is no longer needed.

Then fix `examples/05-unwrap-panics.rs` so it no longer panics, without changing what `safe_divide` does — have it print a friendly message instead of crashing.

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-06-03-result-and-question-mark
```

One builds `Ok`/`Err` by hand. One leans on `.parse()`. One uses `?` to pass a failure through two steps. The last two deliberately turn a `Result` into a plain value or an `Option` — exactly where the reason for failure stops mattering.

### Build

Write a `pub fn parse_all(inputs: &[&str]) -> Result<Vec<i32>, String>`: parse every string in `inputs` as an `i32`, in order. If all of them succeed, return `Ok` with all the numbers, in the same order. If one fails, return `Err` right then, holding that parse error's own text (`.to_string()`), without trying the rest.

Then write a sentence on why `?` inside a loop gives you exactly this behaviour for free.

### Challenge (optional)

**Part one.** `?` works on `Option<T>` too, not just `Result`. Write this, run it, and see whether your guess was right:

```rust
fn first_char(s: &str) -> Option<char> {
    let c = s.chars().next()?;
    Some(c.to_ascii_uppercase())
}
```

Try it with an empty string and an ordinary one.

**Part two.** Change `chained_with_question_mark`'s error type from `String` to `&'static str`. Does it compile? If you don't also change `safe_divide` to the same type, what error do you get? (This is exactly where [1.6.5](../05-from-and-error-conversion/README.md) becomes necessary.)

**Part three.** Change `main`'s signature in `04-question-mark.rs` to `Result<(), Box<dyn std::error::Error>>` — the same thing the compiler suggested in the errors section. Does it compile? What do you need to add for it to compile?

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `Result<T, E>` | either `Ok(T)`, or `Err(E)` with a reason | anywhere failure needs an explanation |
| `#[must_use]` | ignoring it warns, doesn't error | `let _ = ...` to silence it on purpose |
| `.parse::<T>()` | `Result<T, ParseIntError>` (or similar) | string-to-number, since Phase 0 |
| `.map` / `.map_err` | transform `Ok` / transform `Err` | reshaping without a `match` |
| `.and_then` | chain a second fallible step | when step two needs step one's value |
| `.unwrap_or` / `.ok()` | plain value with a fallback / to `Option` | once the error's reason stops mattering |
| `?` | the `match` above, compressed to one character | only inside a function returning `Result`/`Option` |
| `main() -> Result<...>` | ties the process exit code to `Err` | command-line programs |

### What you now know

- `Result<T, E>` says "either success, with this value; or failure, with this reason" — and the reason is what `Option` never had.
- `Result` is marked `#[must_use]`; ignoring one warns, it doesn't error.
- `.parse::<T>()` gives a `Result<T, E>`; the turbofish says which `T`.
- `.map`, `.map_err`, `.and_then`, `.unwrap_or`, and `.ok()` are five different ways around a `match` on a `Result`.
- `expr?` means: on `Ok(v)`, continue with `v`; on `Err(e)`, return from the function right now with `Err(e)`.
- `?` only works inside a function that itself returns `Result` or `Option` — and that includes `main`.

### What comes back later

- **When to panic and when to return `Result`** — [1.6.4 — Panic versus `Result`](../04-panic-vs-result/README.md)
- **Automatic error-type conversion with `From`, what `?` does behind the scenes** — [1.6.5 — `From` and error conversion](../05-from-and-error-conversion/README.md)
- **Custom error types implementing the `Error` trait, and `Box<dyn Error>`** — [Phase 2 — Custom error types](../../../phase2-intermediate/04-error-handling-and-lifetimes/01-custom-error-types/README.md)
- **`thiserror` and `anyhow`, for when writing errors by hand gets old** — [Phase 2 — `thiserror` and `anyhow`](../../../phase2-intermediate/04-error-handling-and-lifetimes/02-thiserror-and-anyhow/README.md)
- **This same `.map` shape, this time on iterators** — [Phase 2 — Closures and `Fn` traits](../../../phase2-intermediate/02-iterators-and-closures/01-closures-and-fn-traits/README.md)

### Can you explain?

- What's the difference between `Option<T>` and `Result<T, E>`, in one sentence?
- What does `#[must_use]` guarantee, and why is it a warning rather than an error?
- What does `.parse::<u32>()` return on a negative number, and why?
- `expr?` is exactly equivalent to which `match`?
- Why doesn't `?` compile inside `fn main() { ... }` with no return type?
- When `main` returns `Err`, what's the process's exit code?

---

## Going further

- [The Rust Book — `Result` and early return from errors](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html) — the same ground, officially, with a full section on `?`.
- [`std::result::Result`](https://doc.rust-lang.org/std/result/enum.Result.html) — the full list of combinators; far more than the five you saw here.
- [`std::ops::Try` and `?`](https://doc.rust-lang.org/std/ops/trait.Try.html) — for when you want to know exactly what more general thing `?` implements.
