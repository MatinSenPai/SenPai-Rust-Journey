# Solution — 1.5.5 `if let`, `while let`, `let else`

```rust
pub fn watching_line(entry: &Entry) -> String {
    if let Entry::Watching { episode } = entry {
        format!("on episode {episode}")
    } else {
        "not watching".to_string()
    }
}

pub fn ratings_only(entries: Vec<Entry>) -> Vec<u8> {
    let mut out = Vec::new();
    for entry in entries {
        if let Entry::Completed { rating } = entry {
            out.push(rating);
        }
    }
    out
}

pub fn pop_all(stack: Vec<i32>) -> Vec<i32> {
    let mut stack = stack;
    let mut out = Vec::with_capacity(stack.len());
    while let Some(value) = stack.pop() {
        out.push(value);
    }
    out
}

pub fn episode_gap(entry: &Entry, latest_episode: u32) -> u32 {
    let Entry::Watching { episode } = entry else {
        return 0;
    };
    latest_episode.saturating_sub(*episode)
}

pub fn pop_until_negative(stack: Vec<i32>) -> Vec<i32> {
    let mut stack = stack;
    let mut out = Vec::new();
    while let Some(value) = stack.pop() {
        if value < 0 {
            break;
        }
        out.push(value);
    }
    out
}
```

Five functions, four forms. Not one of them needed a `match` — but every one of them *could* have been a `match`, and that's what makes the interesting question interesting.

## `watching_line` — `if let ... else`, with nothing else going on

```rust
if let Entry::Watching { episode } = entry {
    format!("on episode {episode}")
} else {
    "not watching".to_string()
}
```

Two cases, both with work, and only one of them unpacking data. That's exactly where `if let ... else` beats a `match`: one arm wants the data and the rest share a single answer.

Notice that `entry` is a `&Entry` and the pattern is `Entry::Watching { episode }` — no `&` anywhere. The compiler works out that you're looking through a reference and binds `episode` as a reference too. `format!` doesn't care, because it only reads.

The `match` version is this:

```rust
match entry {
    Entry::Watching { episode } => format!("on episode {episode}"),
    _ => "not watching".to_string(),
}
```

Almost the same size — and if you delete that `_`, the compiler makes you write out all three remaining variants. That's the tell: **the `_` is what `if let` writes for you implicitly.** Wherever you'd have been happy with a `_` in the `match` version, `if let` is just as safe. Wherever you wouldn't, `if let` is hiding something from you.

## `ratings_only` — `if let` with no `else`, inside a loop

```rust
for entry in entries {
    if let Entry::Completed { rating } = entry {
        out.push(rating);
    }
}
```

Here the other cases genuinely have nothing to do, so there's no `else` — writing one would leave an empty block.

`entries` is consumed (`Vec<Entry>`, not `&Vec<Entry>`), so `rating` is a real `u8` rather than a reference and goes straight into the answer. Had the signature been `&[Entry]` you'd have needed `*rating` — which is the distinction from [1.3.1](../../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md), not a new rule.

## `pop_all` — why `while let` and not `for`

```rust
while let Some(value) = stack.pop() {
    out.push(value);
}
```

A `for` can't do this job, because `for` walks something that already has a shape; here each turn comes from a **function call** that might not answer. That's the difference between the two: `for` walks a collection, `while let` repeats a step that can fail.

`Vec::with_capacity(stack.len())` is free to write and saves the output from growing in stages, because the final length is known up front.

And look at `let mut stack = stack;`: the value arrives owned but the signature doesn't say `mut`. A fresh `let` with the same name makes it mutable — that's shadowing, from [1.1.1](../../../01-foundations/01-variables-mutability-shadowing/README.md). You could equally write `mut stack: Vec<i32>` in the signature; the behaviour is identical, but `mut` in a signature means nothing to the caller, so it's better kept inside.

## `episode_gap` — a guard on a pattern

```rust
let Entry::Watching { episode } = entry else {
    return 0;
};
latest_episode.saturating_sub(*episode)
```

This is the shortest of the five and also the one with the most to say. The function's real work is one line, and it sits at the function's top level — not inside an `if let`, not inside a `match` arm.

Three things:

- `return 0;` diverges, which is what makes the `else` block legal. Write a bare `0` instead and you get `E0308`: the block would finish and reach the next line.
- `episode` is available *after* that line. With `if let` it wouldn't be, and everything would have had to move inside the block.
- `*episode` because `entry` is a reference, so `episode` is a `&u32`, and `saturating_sub` wants a `u32`.

And `saturating_sub` rather than `-` is deliberate: if somebody is on episode 14 and the latest aired is 12, plain subtraction on a `u32` panics. That's `saturating_` from [1.1.2](../../../01-foundations/02-scalar-types-and-overflow/README.md), and the doc comment said outright that it never wraps.

## `pop_until_negative` — `while let` plus a `break`

```rust
while let Some(value) = stack.pop() {
    if value < 0 {
        break;
    }
    out.push(value);
}
```

Two different reasons for the loop to end, and it matters that you don't confuse them:

- **the pattern failed** — the Vec is empty. `while let` handles that itself.
- **`break`** — a negative number came out. That's your decision and has nothing to do with the pattern.

The `break` comes before the `push`, because the negative number itself must not appear in the answer. And zero isn't negative, so `value < 0` is right and `value <= 0` is wrong — the `vec![5, 0, 6]` case is there to catch exactly that.

## What this lesson was really about

- **`if let` is a `match` with the `_ => {}` arm left unwritten.** If a `_` would have satisfied you in the `match` version, `if let` is safe.
- **`while let` repeats a step that can fail**, and the first failure ends it.
- **`let ... else` binds into the current scope**, which is what makes it right for a guard.
- **The `else` block has to leave.** Its type is `!`, the never type.
- **Every step down from `match` leaves exhaustiveness behind.** Sometimes that's a good trade. It should always be a deliberate one.
