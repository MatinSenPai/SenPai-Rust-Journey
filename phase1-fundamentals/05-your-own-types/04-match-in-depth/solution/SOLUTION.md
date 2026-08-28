# Solution — 1.5.4 `match` in depth

```rust
pub fn describe(progress: &Progress) -> String {
    match progress {
        Progress::NotStarted => "not started".to_string(),
        Progress::Reading { chapter, of } => format!("chapter {chapter} of {of}"),
        Progress::Finished { rating } => format!("finished, {rating}/10"),
        Progress::Dropped { at } => format!("dropped at chapter {at}"),
    }
}

pub fn band(stars: u8) -> String {
    match stars {
        0 => "unrated".to_string(),
        1..=3 => "weak".to_string(),
        4..=6 => "watchable".to_string(),
        7 | 8 => "good".to_string(),
        9 | 10 => "top shelf".to_string(),
        _ => "not a score".to_string(),
    }
}

pub fn shelf(progress: &Progress) -> String {
    match progress {
        Progress::Finished { rating: 9 | 10 } => "hall of fame".to_string(),
        Progress::Finished { .. } => "read".to_string(),
        Progress::Reading { chapter, of } if chapter == of => {
            "waiting for the next chapter".to_string()
        }
        Progress::Reading { .. } => "reading".to_string(),
        Progress::NotStarted => "the pile".to_string(),
        Progress::Dropped { .. } => "gone".to_string(),
    }
}

pub fn chapter_label(chapter: u32) -> String {
    match chapter {
        0 => "no chapters yet".to_string(),
        n @ 1..=9 => format!("early (chapter {n})"),
        n @ 10..=99 => format!("mid (chapter {n})"),
        n => format!("long runner (chapter {n})"),
    }
}

pub fn pair_verdict(mine: &Progress, theirs: &Progress) -> String {
    use Progress::{Finished, NotStarted};
    match (mine, theirs) {
        (Finished { rating: a }, Finished { rating: b }) if a == b => "we agree".to_string(),
        (Finished { .. }, Finished { .. }) => "we disagree".to_string(),
        (NotStarted, NotStarted) => "neither of us has started".to_string(),
        (Finished { .. }, _) | (_, Finished { .. }) => "one of us finished".to_string(),
        _ => "still reading".to_string(),
    }
}

pub fn release_note(latest: Option<u32>, read: u32) -> String {
    match latest {
        None => "nothing published yet".to_string(),
        Some(n) if n == read => "caught up".to_string(),
        Some(n) if n > read => format!("{} behind", n - read),
        Some(_) => "ahead of the release".to_string(),
    }
}
```

Six functions, and not one `if` among them. That isn't an accident.

## `describe` — the basic shape

```rust
Progress::Reading { chapter, of } => format!("chapter {chapter} of {of}"),
```

One arm per variant, each pulling out its own data. There is no `_` — and that is what makes this function safe to own: the day a fifth variant appears, this `match` errors and asks you what the answer is for that case.

Written as an `if let` or a chain of `if`s it would also work, and the tests would also be green. The difference only shows up on the day something changes.

A small point: `chapter` here is a `&u32`, because `progress` is a `&Progress`. `format!` follows the reference for you, so no `*` is needed here. Had you wanted to do arithmetic with it, one would have been.

## `band` — a range, an alternative, and a `_` that is right this time

```rust
1..=3 => "weak".to_string(),
7 | 8 => "good".to_string(),
_ => "not a score".to_string(),
```

Three tools in three lines: range, alternative, wildcard.

`_` is the right call here and was the wrong call in `describe`, and the difference is the type. A `u8` has 256 values and always will; writing `11..=255` would have been correct and would have added nothing. An enum is the opposite: how many variants it has is precisely the thing that changes tomorrow.

`9 | 10` could have been `9..=10`. For two values it makes no difference; for five, the range reads better.

## `shelf` — three tools in one function

```rust
Progress::Finished { rating: 9 | 10 } => "hall of fame".to_string(),
```

First: `|` can appear **inside** a pattern, not only between two whole arms. This says "a `Finished` whose `rating` is 9 or 10".

```rust
Progress::Reading { chapter, of } if chapter == of => {
    "waiting for the next chapter".to_string()
}
```

Second: this is the one place a guard is genuinely required. No pattern can say "these two fields are equal" — a pattern talks about shape, not about relationships between values. The comparison `chapter == of` runs on two `&u32`s and does the right thing, because Rust turns equality of references into equality of what they point at.

Third: order. The `Finished { rating: 9 | 10 }` arm has to sit above `Finished { .. }`. Swap them and the tests break and the compiler gives you `unreachable pattern` — the same thing you saw in example `03`, this time on your own code.

And again, no `_`. Six arms for four variants, because two of the variants have two behaviours.

## `chapter_label` — exactly what `@` is for

```rust
n @ 1..=9 => format!("early (chapter {n})"),
```

Without `@` you'd have to say the same thing twice: once to test the range and once to get the number. With `@` both happen in one pattern.

The last arm is a bare `n` with no range — a pattern meaning "anything, and call it `n`". Writing `100..=u32::MAX` would also have been correct, but `n` says the same thing more shortly. Note that `_` **could not** have done the job here, because we needed the number itself.

## `pair_verdict` — four cases out of sixteen, in one `match`

```rust
(Finished { rating: a }, Finished { rating: b }) if a == b => "we agree".to_string(),
(Finished { .. }, Finished { .. }) => "we disagree".to_string(),
```

Those first two arms are a common shape: **the specific one with a guard, then the same pattern without one.** Because arms are tried in order, the second only gets its turn when the first one's guard failed — that is, "both `Finished` and the ratings differ", without ever writing `a != b`.

```rust
(Finished { .. }, _) | (_, Finished { .. }) => "one of us finished".to_string(),
```

This arm means "exactly one of them is `Finished`" — but only because it comes after the two "both `Finished`" arms. Move it up and it swallows the "both" case too, and the first two tests break. Order here is part of the logic, not a matter of taste.

The `use Progress::{Finished, NotStarted};` inside the function is only for readability. With the full `Progress::Finished { .. }` names it is exactly the same match, three times as long.

## `release_note` — `Option` needs nothing new

```rust
None => "nothing published yet".to_string(),
Some(n) if n == read => "caught up".to_string(),
Some(n) if n > read => format!("{} behind", n - read),
Some(_) => "ahead of the release".to_string(),
```

Four arms for an enum with two variants — because two of the cases carry guards. And the usual rule applies: finish the chain with an unguarded arm. Write the last one as `Some(n) if n < read` and you get `E0004`, because the compiler doesn't count guarded arms.

The last arm is `Some(_)` rather than `Some(n)` because that case doesn't want the number. Write `Some(n)` and the compiler warns about an unused variable and suggests `_n`. `_` is the more direct way of saying it.

And `n - read` is safe here only because the arm above it guaranteed `n > read`. Reorder them and that same subtraction underflows in the `n < read` case and panics in a debug build — the behaviour from [1.1.2](../../../01-foundations/02-scalar-types-and-overflow/README.md). Guards here aren't only sorting cases; they're making the code after them safe.

## What this lesson was really about

- **Exhaustiveness isn't a feature, it's leverage.** It pays out on the day you add a variant and the compiler writes the to-do list.
- **A `_` on an enum breaks that leverage.** Five of these six functions have no `_`, deliberately. `band` is the exception because it matches a `u8`, not an enum.
- **One arm both tests and extracts, in a single step.** Nowhere in this file is there an `if` followed by a field access.
- **Order carries meaning.** In `shelf` and in `pair_verdict`, swapping two arms changes the logic, and the compiler only sometimes warns.
- **Guards say what a pattern cannot** — and in exchange, they don't count towards the compiler's coverage.

The next lesson ([1.5.5](../../05-if-let-while-let-let-else/README.md)) shortens all of this for the case where only one arm matters to you. It's nothing new; it's this, with less typing.
