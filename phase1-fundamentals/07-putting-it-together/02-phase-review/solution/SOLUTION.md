# Solution — 1.7.2 Phase review

```rust
pub fn grade_letter(score: u32) -> char {
    if score >= 90 {
        'A'
    } else if score >= 80 {
        'B'
    } else if score >= 70 {
        'C'
    } else if score >= 60 {
        'D'
    } else {
        'F'
    }
}

pub fn merge_unique(base: Vec<String>, extra: &[String]) -> Vec<String> {
    let mut merged = base;
    for item in extra {
        if !merged.contains(item) {
            merged.push(item.clone());
        }
    }
    merged
}

pub fn interior(values: &[i32]) -> &[i32] {
    if values.len() < 2 {
        &[]
    } else {
        &values[1..values.len() - 1]
    }
}

pub fn shorten(text: &str, max_chars: usize) -> &str {
    let mut end = text.len();
    let mut count = 0;
    for (index, _) in text.char_indices() {
        if count == max_chars {
            end = index;
            break;
        }
        count += 1;
    }
    &text[..end]
}

pub enum Status {
    Pending,
    Shipped { tracking: String },
    Cancelled { reason: String },
}

pub fn describe_status(status: &Status) -> String {
    match status {
        Status::Pending => "pending".to_string(),
        Status::Shipped { tracking } => format!("shipped, tracking {tracking}"),
        Status::Cancelled { reason } => format!("cancelled: {reason}"),
    }
}

pub fn safe_average(values: &[i32]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut total: i64 = 0;
    for value in values {
        total += *value as i64;
    }
    Some(total as f64 / values.len() as f64)
}
```

Six functions, one per module of Phase 1. Each gets its own section here — not
because the code is complicated, but because the value of this exercise is in
naming which *decision* sits behind each one.

## `grade_letter` — a decision that is a value

```rust
if score >= 90 {
    'A'
} else if score >= 80 {
    'B'
} ...
```

No `let mut letter`, no `return`. The whole `if`/`else if`/`else` chain is one
expression, and its value is what the function hands back. That is exactly
what [1.1.5](../../../01-foundations/05-control-flow/README.md) opened with:
in Rust, control flow doesn't just shape the program, it produces values.

Order matters too: the branches are checked top to bottom, so writing
`score >= 60` before `score >= 90` would turn every score into `'D'`. The
first condition that holds wins.

## `merge_unique` — ownership, not just a signature

```rust
let mut merged = base;
for item in extra {
    if !merged.contains(item) {
        merged.push(item.clone());
    }
}
merged
```

`base: Vec<String>` is taken with no `&` — the function owns it, the exact
distinction [1.2.4](../../../02-ownership-and-memory/04-ownership-across-functions/README.md)
drew between `fn f(x: String)` and `fn f(x: &str)`. Ownership is the right
call here because the function hands back this very `Vec`, extended; there's
nothing to "borrow and give back."

`extra: &[String]` is the opposite: only ever read, so a borrow is enough —
[1.3.1](../../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md).

The finer point: `.clone()` sits inside the `if` alone. Strings already in
`merged` are never cloned — they're already there, nobody needs to allocate
them again. Duplicates from `extra` are never cloned either — they never make
it past the check. The only thing that gets cloned is exactly the thing about
to be inserted. That's
[1.2.3](../../../02-ownership-and-memory/03-clone-and-copy/README.md): a
clone should be visible in the code, and only where it's actually needed.

## `interior` — a look, not a copy

```rust
&values[1..values.len() - 1]
```

No new `Vec` gets built. `&values[1..values.len() - 1]` is a borrowed look at
part of memory that already exists — exactly
[1.3.4](../../../03-borrowing-and-references/04-slices/README.md)'s
definition of a slice: two words, start and length, no ownership.

The `values.len() < 2` guard has to come before the subtraction, because
`values.len() - 1` underflows when the length is zero — and since that's a
`usize` (which can never go negative), the subtraction panics instead of
quietly producing a negative number. That's the same
[overflow](../../../01-foundations/02-scalar-types-and-overflow/README.md)
trap [1.1.2](../../../01-foundations/02-scalar-types-and-overflow/README.md)
warned about, showing up again here.

## `shorten` — count scalars, not bytes

```rust
for (index, _) in text.char_indices() {
    if count == max_chars {
        end = index;
        break;
    }
    count += 1;
}
&text[..end]
```

`text.char_indices()` hands back every Unicode scalar together with its byte
offset. The loop counts up to the `max_chars`-th character, keeps *that*
byte offset, and the final slice lands on that boundary — not on byte
`max_chars`.

This is exactly what
[1.4.2](../../../04-text-and-strings/02-utf8-bytes-chars-graphemes/README.md)
showed with "سلام": four letters, eight bytes. If this function had written
`&text[..max_chars]` directly, `shorten("سلام", 2)` would have become
`&text[..2]` — landing right in the middle of the letter «س» and panicking,
exactly what this lesson's [Repair](../README.md#repair) exercise is built on.

If `max_chars` is larger than the character count, the loop never `break`s
and `end` stays at its initial `text.len()` — the whole string comes back.

## `describe_status` — forced completeness

```rust
match status {
    Status::Pending => "pending".to_string(),
    Status::Shipped { tracking } => format!("shipped, tracking {tracking}"),
    Status::Cancelled { reason } => format!("cancelled: {reason}"),
}
```

Three arms for three shapes of `Status` — no fewer. Leave one out and this
`match` simply wouldn't compile; the compiler would name the exact arm that's
missing. That "forced completeness" is what
[1.5.4](../../../05-your-own-types/04-match-in-depth/README.md) called
`match`'s biggest gift: an enum that grows a fourth shape tomorrow turns every
incomplete `match` into a compile error, not a silent bug.

The patterns `{ tracking }` and `{ reason }` recognise the shape and grab its
data in the same step — the exact lesson
[1.5.3](../../../05-your-own-types/03-enums-as-data/README.md) taught with
data-carrying enums.

## `safe_average` — absence instead of an accident

```rust
if values.is_empty() {
    return None;
}
...
Some(total as f64 / values.len() as f64)
```

Before any arithmetic happens, the "there are no numbers" case is separated
out and named explicitly: `None`. Without that check, `values.len() as f64`
would be zero, and dividing a float by zero doesn't panic — it produces
`NaN`, a value that spreads silently through a program and turns up a
thousand lines later. `Option` makes that impossible: the caller has to
consider `None` before it can use the average at all. That's exactly what
[1.6.1](../../../06-absence-and-failure/01-option-and-null-safety/README.md)
proposed instead of a sentinel like `-1.0` or `0.0`.

## What this lesson was really about

- Six functions, six different decisions — and each one traces back to one
  specific Phase 1 lesson, not to "Rust" in general.
- If one of these was hard, that difficulty is telling you exactly which
  chapter to reopen — that's the entire point of this exercise.
- None of these functions needed anything outside Phase 1: no `HashMap`, no
  closures, no traits of your own. Everything came from these thirty-one
  lessons.
