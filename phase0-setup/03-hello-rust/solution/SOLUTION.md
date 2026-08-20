# Solution — 03 Hello, Rust

```rust
pub fn describe_journey(language: &str, day: u32) -> String {
    format!("Day {day} of learning {language}: still compiling.")
}

pub fn shout(line: &str) -> String {
    format!("{}!", line.to_uppercase())
}

pub fn progress_bar(done: usize, total: usize) -> String {
    format!(
        "[{}{}] {done}/{total}",
        "#".repeat(done),
        ".".repeat(total - done)
    )
}
```

## `describe_journey`

One `format!`, no `return`, no semicolon on the last line. If yours has a `return`, it works — but `cargo clippy` will tell you to drop it in lesson 06, so you may as well form the habit now.

Note the two styles mixed in one string: `{day}` and `{language}` name their variables directly, while `{}` in the others is filled positionally. You can mix them freely. Naming is better whenever the value is already in a variable.

## `shout` — why `{}` and not `{line}`

```rust
format!("{}!", line.to_uppercase())
```

You can't write `format!("{line.to_uppercase()}!")`. The `{name}` form takes a **variable name**, not an expression. If you want a method call in there, either use a positional `{}` with the expression as an argument, or bind it first:

```rust
let shouted = line.to_uppercase();
format!("{shouted}!")
```

Both are fine. The first is shorter; the second reads better when the expression is long.

`to_uppercase` returns a new `String` rather than modifying `line` — it can't modify it, because `line` is a `&str`, borrowed text you don't own. That's the ownership system showing through, and Phase 1 makes it explicit.

## `progress_bar` — `repeat` beats a loop

```rust
"#".repeat(done)
```

You may have written a loop instead. That works, but `repeat` says what it means in one call, and it allocates once instead of growing a string repeatedly.

The subtraction is the part worth thinking about:

```rust
".".repeat(total - done)
```

If `done` were ever greater than `total`, `total - done` would underflow — and on `usize` that panics in debug and wraps to an enormous number in release. The doc comment says `done` is never greater than `total`, so this is fine *given the contract*.

That's a real pattern worth noticing: **a doc comment stating a precondition is doing load-bearing work.** In Phase 1 you'll learn how to make the type system enforce such things instead of trusting a comment.

## What the exercises were really testing

Not `format!`. They were testing whether you internalised:

1. The last expression is the return value.
2. `&str` in, `String` out.
3. When `{name}` works and when you need `{}`.

If all three now feel automatic, this lesson did its job.
