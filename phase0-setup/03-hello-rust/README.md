# 03 — Hello, Rust

## At a glance

After this lesson you can:

- Write and run a complete Rust program, and say what every line does.
- Produce output with `println!`, dropping values into text with `{}`, with `{name}`, and with width control.
- Say why `src/lib.rs` and `src/main.rs` both exist and which one actually runs.

**Time:** ~45 minutes · **Prerequisites:** [02 — Installing Rust and toolchains](../02-installing-rust-and-toolchains/README.md)

---

## Why this matters

So far you've read *about* Rust. This is the first lesson where you *write* it.

The goal isn't to build anything impressive. It's to make the shape of the language concrete: what a function looks like, how you print something, where the return value lives. These repeat so often that they need to become thoughtless, so that when Phase 1 brings the genuinely hard ideas, none of your attention is going on syntax.

There's also one small thing here that many newcomers don't discover for months, and suffer for until they do: **in Rust the last expression of a function is its return value.** One stray `;` breaks that, and the error is baffling the first time. You'll meet it in this lesson.

---

## The concept

### `fn main` — where everything starts

The smallest complete Rust program:

```rust
fn main() {
    println!("Hello, Rust!");
}
```

```text
Hello, Rust!
```

- `fn` means "I'm defining a function". Python's `def`.
- `main` is a special name. When the program runs, this is what the operating system calls. Every executable binary has exactly one.
- `()` means it takes no arguments.
- `{ ... }` is the body. Rust uses braces, not indentation — so indentation is for your benefit, not the compiler's. (`cargo fmt` tidies it for you; lesson 06.)

### `println!` and that exclamation mark

The `!` isn't a typo. It means `println!` is a **macro**, not a function.

For now, know this much: a macro is something that expands into other code *before* compilation. That's why `println!` can take any number of arguments and check the format string at compile time — neither of which an ordinary Rust function can do.

You can see the practical consequence immediately: if the number of `{}` holes doesn't match the number of arguments, **it doesn't compile.** In Python, `"{} {}".format(1)` is a run-time `IndexError`. Here it's a compile error.

You'll write macros in Phase 2. Until then, read `!` as "this is a macro" and move on.

### Formatting: from `{}` to `{name}`

```rust
let language = "Rust";
let day = 1;

println!("Day {} of learning {}", day, language);
println!("Day {day} of learning {language}");
```

```text
Day 1 of learning Rust
Day 1 of learning Rust
```

Both do the same thing:

- **`{}`** is a hole filled by the next argument, in order.
- **`{name}`** takes a variable that's in scope directly. With four holes it's far more readable — and you can't get the order wrong.

Prefer `{name}` where you can. That's where the language is going.

There's width and alignment too:

```rust
println!("[{:>8}] right", "Rust");
println!("[{:<8}] left", "Rust");
println!("debug view: {:?}", "Rust");
```

```text
[    Rust] right
[Rust    ] left
debug view: "Rust"
```

`{:?}` asks for the **debug** view rather than the display view — which is why it shows the quotes. The difference is two separate traits, `Display` and `Debug`, which you'll implement in Phase 2. For now know this: **if `{}` doesn't work on something, try `{:?}`.** It's one of the most-used debugging tricks in Rust.

### A function with arguments and a return value

```rust
fn banner(title: &str, stars: usize) -> String {
    format!("{title} {}", "*".repeat(stars))
}
```

Three new things:

- **`title: &str`** — name, colon, type. Unlike Python this is not optional. Every parameter declares its type.
- **`-> String`** — the type of what the function returns. If it returns nothing, you leave this off entirely.
- **`format!`** — exactly like `println!` except it returns the string instead of printing it. Python's `f"..."`.

### The last expression is the return value

Read this bit carefully; it doesn't come from Python.

`banner`'s body has no `return`. The last **expression** is written without a `;`, and that is the return value.

```rust
fn double(n: u32) -> u32 {
    n * 2          // no semicolon: this is the return value
}
```

Add a `;` and the meaning changes:

```rust
fn double(n: u32) -> u32 {
    n * 2;         // with a semicolon: this is a statement, and returns nothing
}
```

The second one does **not** compile — and you'll see its exact error in "Errors you will meet" below. It's probably everyone's most common week-one mistake.

The `return` keyword does exist and does work, but in Rust it's for **early exit** from the middle of a function, not for the ordinary return value. The idiomatic style is the final expression without a semicolon.

The reasoning — that almost everything in Rust is an expression and has a value — is [1.1.4 — Functions and the expression language](../../phase1-fundamentals/01-foundations/04-functions-and-expressions/README.md). For now, just know the rule.

### `&str` and `String` — a first look

That signature had two different text types, and that's deliberate:

- **`&str`** — text you have **borrowed**. A view onto text that lives somewhere else. `"Rust"` written directly in your code is one.
- **`String`** — text you **own**. Growable, on the heap.

The rule of thumb that carries you to Phase 1: **take `&str` in, hand `String` out.**

Why Rust has two where Python has one, and what "borrowing" and "ownership" actually mean, is what the whole of Phase 1 is about. Here you're only seeing them, not learning them. If it doesn't fully land yet, that's correct — it isn't supposed to.

### The block at the bottom of `src/lib.rs`

Open `src/lib.rs` and scroll down. There's this:

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

Three pieces:

- **`#[cfg(test)]`** — "only compile what follows when running tests". Your shipped binary doesn't carry the tests.
- **`#[test]`** — marks one function as a test. `cargo test` finds every one of them and runs it.
- **`assert_eq!`** — another macro. If the two values differ, the test fails and prints both.

In Python your tests live in a separate `tests/` directory. In Rust the small ones live **in the same file as the code they test**, at the bottom. That's not laziness — it lets a test reach private functions the outside world can't see.

You'll write tests yourself in Phase 2. For now, know what you're looking at when `cargo test` reports a failure, and read the assertion it printed.

### `src/lib.rs` versus `src/main.rs`

This lesson has both, and the difference matters:

| File | What it is | How it runs / is tested |
|---|---|---|
| `src/lib.rs` | a **library**: functions other things — including tests — call | `cargo test` |
| `src/main.rs` | a **binary**: the program that actually runs | `cargo run` |

`main.rs` pulls in `lib.rs`'s functions with `use` and calls them. The pattern is:

```text
src/lib.rs   →  logic. Testable. Prints nothing.
src/main.rs  →  thin edge. Takes input, calls the logic, prints output.
```

This is the same split every backend phase is built on: logic you can test, separated from I/O you can't. With a three-line program it looks like pointless ceremony; when you have a real API in Phase 3, it's what makes your tests possible.

---

## Hands on

Three small programs. Run them, compare the output with what's written above, then change them.

> You're about to type `cargo` a lot without having been told what it is. That's deliberate — running a program should come before studying the tool that runs it. [04 — Cargo basics](../04-cargo-basics/README.md) is the very next lesson and explains every part of these commands. For now: `-p` names which crate to work on, and `--example` names which file in `examples/` to run.


```sh
cargo run -p p0-03-hello-rust --example 01-hello
cargo run -p p0-03-hello-rust --example 02-formatting
cargo run -p p0-03-hello-rust --example 03-a-function
```

Then try these and see what happens:

1. In `02-formatting`, delete one `{}` but keep its argument. What error do you get?
2. In the same file, change `{:>8}` to `{:>20}`. What changes?
3. In `03-a-function`, add a semicolon to the end of `banner`'s body. You now have the error the next section explains — read it yourself first.

---

## Errors you will meet

### `E0308` — one semicolon too many

`examples/04-wrong-return.rs` makes exactly this mistake:

```sh
cargo run -p p0-03-hello-rust --example 04-wrong-return --features broken
```

```text
error[E0308]: mismatched types
  --> phase0-setup\04-hello-rust\examples\04-wrong-return.rs:10:41
   |
10 | fn banner(title: &str, stars: usize) -> String {
   |    ------                               ^^^^^^ expected `String`, found `()`
   |    |
   |    implicitly returns `()` as its body has no tail or `return` expression
11 |     format!("{title} {}", "*".repeat(stars));
   |                                             - help: remove this semicolon to return this value
```

**What the compiler is objecting to:** the function promised a `String` and its body returns nothing. That `()` — pronounced "unit" — is the type of "no meaningful value"; the closest Python analogy is `None`, except this is a real type.

The `----` label on the function name says exactly why: *implicitly returns `()` as its body has no tail or `return` expression*.

**The fix:** remove the semicolon on line 11.

**Why that's the fix:** with the semicolon, that line is a **statement** — work happens and its value is thrown away. Without it, it's an **expression** and its value leaves the function. The compiler is confident enough to say so itself: *remove this semicolon to return this value*.

When you see this error, look first for a stray semicolon on a function's last line. Nine times out of ten that's it.

### A macro that doesn't exist

```sh
cargo run -p p0-03-hello-rust --example 05-unknown-macro --features broken
```

```text
error: cannot find macro `printn` in this scope
  --> phase0-setup\04-hello-rust\examples\05-unknown-macro.rs:7:5
   |
 7 |     printn!("Hello, Rust!");
   |     ^^^^^^
   |
  ::: C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\std\src\macros.rs:85:1
   |
85 | macro_rules! print {
   | ------------------ similarly named macro `print` defined here
   |
help: a macro with a similar name exists
   |
 7 -     printn!("Hello, Rust!");
 7 +     print!("Hello, Rust!");
   |
```

**Two interesting things here:**

**First: it has no error code.** Just `error:`, no `[E0433]`. Not every `rustc` error has one — so if `--explain` has nothing for something, you haven't made a mistake; that error simply has no code.

**Second: the `:::` points into the standard library.** The compiler went and looked in `library/std/src/macros.rs` and found `print!`. Labels can point at files outside your project — which becomes very useful later when an error comes from inside a third-party crate.

**The fix:** change `printn!` to `println!`. (Note the compiler suggested `print!`, not `println!` — and they differ: `print!` doesn't start a new line. The compiler suggests the nearest name, not necessarily the one you meant. Read suggestions; don't accept them blindly.)

---

## Exercises

### Warm up

<details>
<summary>Does this compile? <code>println!("{} and {}", "a");</code></summary>

No. Two `{}` holes, one argument. Because `println!` is a macro, the format string is checked at compile time. The Python equivalent would be a run-time error.

</details>

<details>
<summary>What does this function return?<br><code>fn f(n: u32) -> u32 { n + 1 }</code></summary>

`n + 1`. The last expression without a semicolon is the return value. No `return` needed.

</details>

<details>
<summary>And this one?<br><code>fn f(n: u32) -> u32 { n + 1; }</code></summary>

Nothing — which is why it doesn't compile. The semicolon turns the expression into a statement, so the function returns `()` when it promised `u32`. The error is `E0308`.

</details>

<details>
<summary>Which does <code>cargo run</code> execute: <code>src/lib.rs</code> or <code>src/main.rs</code>?</summary>

`src/main.rs` — that's where `fn main` is. `src/lib.rs` holds functions and is exercised by `cargo test`; it doesn't run on its own.

</details>

### Repair

Fix the two broken examples until they run:

```sh
cargo run -p p0-03-hello-rust --example 04-wrong-return --features broken
cargo run -p p0-03-hello-rust --example 05-unknown-macro --features broken
```

For each, **say what's wrong before you touch the code**.

### Implement

Three functions in `src/lib.rs`. Each one's exact output is written in its own doc comment:

```sh
cargo test -p p0-03-hello-rust
```

Then check the real program works too:

```sh
cargo run -p p0-03-hello-rust
```

### Build

Change `src/main.rs` to print a small progress bar for this course — say two lessons out of Phase 0's seven.

Then push further: print several lines that together make a tidy report. Use width control (`{:>8}`) so the columns line up. The point is to get comfortable with `println!`, because it's your main debugging tool from here on.

### Challenge (optional)

`format!` builds a string and `println!` prints one. There's a third: `eprintln!`, which writes to **standard error** rather than standard output.

Try it. Then run this and see what happens:

```sh
cargo run -p p0-03-hello-rust > out.txt
```

Which lines went into `out.txt` and which stayed on screen? That separation is what lets you keep logs apart from output in Phase 4.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `fn main` | the program's entry point | every executable binary |
| macro (`println!`) | expands into code before compilation; the `!` marks it | printing, `format!`, `vec!`, and much later |
| `{}` and `{name}` | holes in a format string | all output and every error message |
| `{:?}` | the debug view instead of the display view | when `{}` doesn't work on a type |
| expression vs statement | with a semicolon the value is discarded, without it it's returned | every function you write |
| `()` (unit) | the type of "no meaningful value" | the `E0308` return-type error |
| `&str` / `String` | borrowed text / owned text | everywhere; Phase 1 explains it fully |
| `lib.rs` / `main.rs` | testable logic / the runnable edge | the shape of every serious project |

### What you now know

- A Rust program starts at `fn main`.
- `println!` is a macro and its format string is checked at compile time.
- `{name}` is the preferred formatting style, and `{:?}` is what you reach for when `{}` won't do.
- The last expression without a semicolon is a function's return value.
- You take `&str` in and hand `String` out.
- `lib.rs` is logic and `main.rs` is the edge.

### What comes back later

- **Reading compiler errors** — that `E0308` above, taken apart properly: [05 — Reading compiler errors](../05-reading-compiler-errors/README.md)
- **Expressions versus statements, and why `if` has a value too** — [Phase 1 — Control flow](../../phase1-fundamentals/01-foundations/04-functions-and-expressions/README.md)
- **`&str` versus `String` for real** — [Phase 1 — Strings and slices](../../phase1-fundamentals/04-text-and-strings/02-utf8-bytes-chars-graphemes/README.md)
- **Implementing `Display` and `Debug` yourself** — [Phase 2 — Generics and traits](../../phase2-intermediate/03-generics-and-traits/README.md)
- **Writing your own macro** — [Phase 2 — Macros](../../phase2-intermediate/08-rust-toolbox/02-macro-rules-basics/README.md)

### Can you explain?

- What's special about `fn main`?
- What does the `!` in `println!` mean, and name one practical consequence.
- Between `println!("{}", x)` and `println!("{x}")`, which do you prefer and why?
- How does a function return a value, and what does one stray semicolon do to it?
- What is `()`?
- Why does this lesson have both a `lib.rs` and a `main.rs`?

---

## Going further

- [The Rust Book — Hello, World!](https://doc.rust-lang.org/book/ch01-02-hello-world.html) — the same ground in the official book's words.
- [`std::fmt`](https://doc.rust-lang.org/std/fmt/) — everything you can put inside `{}`. It's a reference, not a read-through; but when you want a number printed to two decimal places, the answer is here.
- [Rust by Example — Formatted print](https://doc.rust-lang.org/rust-by-example/hello/print.html) — short runnable examples.
