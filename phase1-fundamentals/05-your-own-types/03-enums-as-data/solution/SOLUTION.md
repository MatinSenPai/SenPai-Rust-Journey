# Solution — 1.5.3 Enums as data

```rust
pub fn from_episode(watched: u32) -> Entry {
    if watched == 0 {
        Entry::Planned
    } else {
        Entry::Watching(watched)
    }
}

pub fn rate(score: u8) -> Entry {
    let score = if score > 10 { 10 } else { score };
    Entry::Rated { score }
}

pub fn drop_at(episode: u32, reason: String) -> Entry {
    let reason = if reason.is_empty() {
        String::from("no reason given")
    } else {
        reason
    };
    Entry::Dropped { episode, reason }
}

impl Entry {
    pub fn is_watching(&self) -> bool {
        matches!(self, Entry::Watching(_))
    }

    pub fn is_favourite(&self) -> bool {
        matches!(self, Entry::Rated { score } if *score >= 8)
    }
}
```

Not one `match` between them. Three of these only *build* a variant, and two ask a one-word question about one.

## `from_episode` — the enum is the return type

```rust
if watched == 0 {
    Entry::Planned
} else {
    Entry::Watching(watched)
}
```

Both arms of the `if` produce an `Entry`, and that is the only reason this compiles as one expression: `if` is an expression, and both branches must have the same type. `Entry::Planned` and `Entry::Watching(7)` do — which is the whole point of an enum.

Try to imagine writing this without one. You would return a tuple `(bool, u32)`, or a struct with a `started` flag, and every caller would have to remember that `episode` is meaningless when `started` is false. The enum makes the caller's job smaller by making yours more precise.

Note there is no `return`. This is the tail expression from [1.1.4](../../../01-foundations/04-functions-and-expressions/README.md), and it is the ordinary way to write a two-branch function in Rust.

## `rate` — clamping, and the field shorthand

```rust
let score = if score > 10 { 10 } else { score };
Entry::Rated { score }
```

Two things worth naming.

The first is shadowing, from [1.1.1](../../../01-foundations/01-variables-mutability-shadowing/README.md): the second `score` is a new binding with the same name, so nothing has to be `mut` and the clamped value is the only one visible from that point on. `score.min(10)` does the same job in fewer characters and is what you would write in real code.

The second is `Entry::Rated { score }` rather than `Entry::Rated { score: score }`. That is the **field init shorthand**, exactly as in a struct literal — a struct variant is built with struct-literal syntax, so everything you know about struct literals applies unchanged.

And notice what the clamp is *for*. A `u8` allows 0 to 255; the domain allows 0 to 10. The type is wider than the truth, so something has to narrow it. Here that is a clamp; the sharper answer — a type that cannot hold 200 in the first place — is the newtype pattern from [1.5.2](../../02-tuple-structs-and-newtype/README.md).

## `drop_at` — moving a `String` into a variant

```rust
let reason = if reason.is_empty() {
    String::from("no reason given")
} else {
    reason
};
Entry::Dropped { episode, reason }
```

The `else` branch is a **move**, not a clone. `reason` was passed in by value, nobody else needs it, and it goes straight into the variant — so building an `Entry::Dropped` costs no allocation at all. This is [1.2.2](../../../02-ownership-and-memory/02-move-semantics/README.md) doing quiet work: a variant holding a `String` owns that `String`, and dropping the `Entry` drops it.

Writing `reason.clone()` there would also pass the tests and would allocate a second buffer for nothing. Being able to see that is what [1.2.3](../../../02-ownership-and-memory/03-clone-and-copy/README.md) was for.

`{ episode, reason }` is the field shorthand again, twice.

## `is_watching` — the shape only

```rust
matches!(self, Entry::Watching(_))
```

`matches!` takes a value and a pattern and gives back `true` or `false`. The `_` is the wildcard you have been using since [1.1.3](../../../01-foundations/03-compound-types-and-destructuring/README.md) — "there is something here, and I do not care what". So this is true for `Watching(0)` and `Watching(9000)` alike.

The alternative without `matches!` needs `match`, and that is [1.5.4](../../04-match-in-depth/README.md):

```rust
match self {
    Entry::Watching(_) => true,
    _ => false,
}
```

`matches!` is literally a macro that expands to that. clippy will even rewrite the long form into the short one for you, under the `match_like_matches_macro` lint.

## `is_favourite` — the data, not just the shape

```rust
matches!(self, Entry::Rated { score } if *score >= 8)
```

This one is the point of the exercise, because it is the first time you look at what a variant *carries*.

`Entry::Rated { score }` binds the name `score` to the field. The trailing `if` is a **guard**: the pattern has to match *and* the guard has to be true. So `Rated { score: 8 }` is a favourite, `Rated { score: 7 }` is not, and `Planned` never gets as far as the guard.

The `*` in `*score` is there because `self` is a `&Entry`, so matching through the reference binds `score` as a `&u8`. Dereferencing gives the `u8` back. If that felt like a detail you had to guess at, it is — Rust calls the rule "default binding modes", and it is exactly the kind of thing [1.5.4](../../04-match-in-depth/README.md) slows down for. `score >= &8` compiles just as well.

This is also the honest answer to "why can't I just write `entry.score`?" — you can get at the data, but only after establishing which variant you are holding. The guard establishes it and hands you the field in one line.

## What this lesson was really about

- **A variant carries data, and each variant's data is its own shape.** That is the difference between Rust's enum and C's, and it is why an enum can model a whole state machine.
- **The invalid state is not caught — it is not expressible.** No `Planned` can hold a score, so no code has to check for one.
- **The enum's name is part of the path, and the shape is part of the construction.** `Entry::Rated { score: 9 }`, never `Rated(9)`.
- **`Option` and `Result` are enums you could have written.** From here on, `Some` and `None` are variants, not vocabulary.
- **An enum costs its largest variant plus a discriminant.** And often not even that, thanks to the niche optimisation.
- **`matches!` asks the shape; `match` gets the data.** You now have half the tool, and [1.5.4](../../04-match-in-depth/README.md) is the other half.
