# Solution — 1.6.1 `Option` and null safety

```rust
pub fn index_of_first_negative(readings: [i32; 6]) -> Option<usize> {
    for position in 0..readings.len() {
        if readings[position] < 0 {
            return Some(position);
        }
    }
    None
}

pub fn safe_average(total: i32, count: u32) -> Option<f64> {
    if count == 0 {
        return None;
    }
    Some(total as f64 / count as f64)
}

pub fn nickname_len(nickname: &Option<String>) -> Option<usize> {
    match nickname.as_ref() {
        Some(name) => Some(name.len()),
        None => None,
    }
}

pub fn first_word_upper(words: &[String]) -> Option<String> {
    match words.first() {
        Some(word) => Some(word.to_uppercase()),
        None => None,
    }
}

pub struct Profile {
    pub nickname: Option<String>,
}

pub fn greeting(profile: &Profile) -> String {
    match &profile.nickname {
        Some(name) => format!("Hey, {name}!"),
        None => "Hey, stranger!".to_string(),
    }
}
```

All five are written with only `match` and `if let` — no `.map()`, `.and_then()`, `.unwrap_or()` or `.ok_or()`. Those are [1.6.2](../../02-option-combinators/README.md).

## `index_of_first_negative` — the same function, the right signature

```rust
for position in 0..readings.len() {
    if readings[position] < 0 {
        return Some(position);
    }
}
None
```

The body is almost word-for-word what you wrote in [1.1.5](../../../01-foundations/05-control-flow/README.md) — the same loop, the same early `return`. The only thing that changed is the return type: `readings.len()` gave way to `None`, and `position` is now wrapped in a `Some`.

The difference looks small until you write the caller. With the old version, `readings[index_of_first_negative(r)]` compiled — and if nothing was negative, it panicked on `readings[6]`. With this version, that same line doesn't compile at all; you have to write a `match` or `if let` first. The bug moved from something that *could* happen to something that *cannot* — the compiler stands between you and the mistake.

## `safe_average` — guard, then divide

```rust
if count == 0 {
    return None;
}
Some(total as f64 / count as f64)
```

A guard clause, from [1.1.5](../../../01-foundations/05-control-flow/README.md): dispose of the exceptional case up front, and the rest of the function doesn't have to think about it. After that guard, `count` is guaranteed nonzero, so the division is safe.

One subtlety: the guard runs **before** the `as f64` conversion. Reorder it and it still works — dividing by a floating-point zero doesn't panic in Rust, it produces `inf` or `NaN` — but that's an `f64`, not the `None` the signature promised; the test catches exactly that.

## `nickname_len` — borrow, don't own

```rust
match nickname.as_ref() {
    Some(name) => Some(name.len()),
    None => None,
}
```

The parameter is already a reference (`&Option<String>`), so `nickname.as_ref()` produces an `Option<&String>` to match on — without taking anything out of that reference. Writing `match nickname { ... }` directly, on the reference itself without `.as_ref()`, would also have worked, because matching on a reference auto-borrows — but `.as_ref()` spells out the same intent by name, and it's the word the rest of this lesson and the wider Rust ecosystem uses.

The test calls it twice in a row to make sure nothing moved:

```rust
assert_eq!(nickname_len(&missing), None);
assert_eq!(nickname_len(&missing), None);
```

Had the signature been `nickname: Option<String>` (no `&`), the second call wouldn't have compiled at all — exactly the `E0382` you saw in "Errors you will meet".

## `first_word_upper` — an `Option<&T>` that was already a look

```rust
match words.first() {
    Some(word) => Some(word.to_uppercase()),
    None => None,
}
```

`.first()` already hands back an `Option<&String>` — no `.as_ref()` needed, because you never owned `words` in the first place. Inside the `Some` arm, `word` is a `&String`; `.to_uppercase()` on it builds a fresh, owned `String`, so what you return is independent of the input parameter.

## `greeting` — `Option` in a field, the exact two strings

```rust
match &profile.nickname {
    Some(name) => format!("Hey, {name}!"),
    None => "Hey, stranger!".to_string(),
}
```

`&profile.nickname` borrows the field rather than pulling it out of `Profile` — the same pattern as `nickname_len`, this time leaning on match ergonomics' automatic borrow instead of spelling out `.as_ref()`. Both ways get you to the same place; which you write is taste, as long as you remember why it works.

The format is exactly what the doc comment promised: `"Hey, {nickname}!"` or `"Hey, stranger!"` — not close to it, exactly it.

## What this lesson was really about

- **A different return type means accidental misuse is not possible.** Write `index_of_first_negative` with `Option<usize>`, and the compiler itself forces the caller to face "not found" before doing anything else.
- **`match` on a reference auto-borrows; `.as_ref()` does the same thing by name.** Both stop you from moving an `Option` you only meant to look at.
- **The `Option<&T>` that `.first()` hands back is already borrowed** — no `.as_ref()` needed when you never owned it to begin with.
- **None of these five needed `.unwrap()`/`.expect()`** — because none of them had to assume something was definitely `Some`; every one answered both cases.
- **`match` and `if let` are enough for anything shaped "a value, or none".** Combinators ([1.6.2](../../02-option-combinators/README.md)) write the same thing shorter, not a different thing.
