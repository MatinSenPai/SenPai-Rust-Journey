# Solution — 04 Cargo basics

```rust
pub fn format_greeting(name: &str, times: u32) -> String {
    let one = format!("Hello, {name}!");
    vec![one; times as usize].join("\n")
}

pub fn pick_encouragement() -> String {
    let lines = [
        "Keep going.",
        "One todo!() at a time.",
        "You've got this.",
    ];
    lines[rand::random::<usize>() % lines.len()].to_string()
}
```

## `format_greeting` — there are several right answers

Three shapes people reach for:

```rust
// build the string once, clone it n times, join
let one = format!("Hello, {name}!");
vec![one; times as usize].join("\n")

// build a vector by iterating
(0..times).map(|_| format!("Hello, {name}!")).collect::<Vec<_>>().join("\n")

// push into a string by hand
let mut out = String::new();
for i in 0..times {
    if i > 0 { out.push('\n'); }
    out.push_str(&format!("Hello, {name}!"));
}
out
```

All three pass. If you wrote the third, don't feel bad about it — it's the most obvious one coming from Python, and it's the one clippy would nudge you away from in lesson 06.

The first is preferred here for a specific reason: it calls `format!` **once**. The second calls it `times` times, formatting the same string over and over. On three greetings nobody cares. On thirty thousand it's the difference between one allocation-and-clone loop and thirty thousand format operations.

Noticing that is worth more than the syntax. `format!` is not free.

The `times as usize` is a cast — `vec![value; n]` wants a `usize` for the count and you have a `u32`. Casts have edge cases (Phase 1 covers them); this one is safe because a `u32` always fits in a `usize` on any platform this course targets.

## `pick_encouragement` — the point was the dependency

The logic is deliberately trivial. What mattered was:

1. `rand` had to be in `Cargo.toml` for `rand::random` to resolve at all.
2. You had to find out how to call it — ideally through `cargo doc --open` rather than a web search.

`rand::random::<usize>()` needs the turbofish `::<usize>` because `random` is generic over what it produces, and nothing here tells the compiler which type you want. When you write `let x: u8 = rand::random();` the annotation supplies it instead. Both work; generics are Phase 2.

`% lines.len()` maps any number onto a valid index. It's very slightly biased towards the earlier entries — for picking an encouraging line, entirely irrelevant. For anything where fairness matters (shuffling a deck, sampling), use `rand`'s own helpers, which handle it correctly.

## What this lesson was really about

Not these two functions. It was:

- Reading `Cargo.toml` and knowing what each section does.
- `cargo add` and `cargo tree`, so a dependency isn't magic.
- Knowing `Cargo.lock` exists and when to commit it.
- `cargo check` versus `cargo build`, and why you reach for the first.

If `cargo new` and `cargo add` in a scratch directory felt routine by the end, that's the outcome that matters.
