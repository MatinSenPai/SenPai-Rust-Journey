# Solution — 2.4.3 `Deref`, `AsRef`, `Borrow`, `ToOwned`

```rust
use std::borrow::Borrow;
use std::ops::{Deref, DerefMut};

pub struct Playlist(pub Vec<String>);

impl Deref for Playlist {
    type Target = Vec<String>;
    fn deref(&self) -> &Vec<String> {
        &self.0
    }
}

impl DerefMut for Playlist {
    fn deref_mut(&mut self) -> &mut Vec<String> {
        &mut self.0
    }
}

pub struct DisplayName(pub String);

impl AsRef<str> for DisplayName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Handle(pub String);

impl Borrow<str> for Handle {
    fn borrow(&self) -> &str {
        &self.0
    }
}

pub fn longest_word_owned(text: &str) -> String {
    text.split_whitespace()
        .max_by_key(|word| word.chars().count())
        .unwrap_or("")
        .to_owned()
}
```

## `Playlist` — `Deref` and `DerefMut` both hand back the same field

```rust
impl Deref for Playlist {
    type Target = Vec<String>;
    fn deref(&self) -> &Vec<String> {
        &self.0
    }
}
impl DerefMut for Playlist {
    fn deref_mut(&mut self) -> &mut Vec<String> {
        &mut self.0
    }
}
```

Both methods do the least possible: hand back a reference to the field already there. `deref` borrows `self` and returns `&self.0`; `deref_mut` borrows `self` mutably and returns `&mut self.0`. Nothing is copied, built, or transformed — the wrapper is genuinely just a name for the `Vec<String>` inside it. That is exactly what makes `list.len()`, `list.push(...)`, and `list[1]` all compile without `Playlist` defining a single one of those methods itself: method lookup walks `Playlist`, does not find them, follows `Deref`/`DerefMut` to `Vec<String>`, and finds them there.

## `DisplayName` — one line, because `AsRef<str>`'s contract is exactly `Deref`'s

```rust
impl AsRef<str> for DisplayName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
```

Same shape as `Playlist`'s `deref`, aimed at a different trait: hand back a reference to the field, in the exact type the trait promises (`&str`, not `&String` — `&String` coerces to `&str` here, so this compiles either way you write it, but `&self.0` already reads as `&str` once the field is a `String`). Any function written as `fn f(name: impl AsRef<str>)` now accepts a `DisplayName` for free, alongside every `&str` and `String` it already accepted.

## `Handle` — `Borrow<str>` has to agree with the `derive`d line above it

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Handle(pub String);

impl Borrow<str> for Handle {
    fn borrow(&self) -> &str {
        &self.0
    }
}
```

The `derive` builds `Handle`'s `Hash` and `Eq` by hashing and comparing its one field, `String`, using `String`'s own `Hash`/`Eq`. `borrow()` hands back that exact same field, unmodified — so `Handle`'s hash and a lookup key's hash are computed from the same bytes either way, and the contract holds. This is the boring, correct version of what `examples/05-borrow-contract-violation.rs` broke on purpose: nothing here transforms the string before hashing it one way and before comparing it another.

`handle_looks_up_by_str_in_hashmap` is the payoff: insert a `Handle`, look it up with a plain `&str` literal, get it back.

## `longest_word_owned` — `.max_by_key()` plus one `.to_owned()`

```rust
pub fn longest_word_owned(text: &str) -> String {
    text.split_whitespace()
        .max_by_key(|word| word.chars().count())
        .unwrap_or("")
        .to_owned()
}
```

`.split_whitespace()` walks `text`'s words as borrowed `&str` slices, all borrowing from `text` itself. `.max_by_key(|word| word.chars().count())` picks the one with the most characters — and on a tie, `Iterator::max_by_key` keeps the *last* maximum it sees, which is why `"cat dog owl bee"` (four three-character words) returns `"bee"`, not `"cat"`. `.unwrap_or("")` handles the no-words case without a `match`. The `&str` that comes out of all that is still borrowed from `text` — it cannot outlive the argument it came from — so the final `.to_owned()` is what actually produces the `String` the signature promises: an independent copy, good for as long as its caller wants to keep it.

## What this lesson was really about

- `Deref`/`DerefMut` are what auto-deref and deref coercion actually call — not magic, one trait with one method each (`deref`, `deref_mut`), implemented once by `Box`, `String`, `Vec`, and now `Playlist`.
- `AsRef<T>` and `Deref` end up looking similar on a newtype this simple, but they answer different questions: `Deref` says "treat me as *the* thing I wrap, everywhere"; `AsRef<T>` says "let a function ask for a cheap `&T` view, on purpose, at one call site."
- `Borrow<T>`'s signature is identical to `AsRef<T>`'s — the difference is a promise the compiler never checks for you: `Hash`, `Eq`, and `Ord` must agree between the borrowed form and the owned one. `Handle` keeps that promise by never transforming the string; `CiKey` in the examples broke it on purpose, and a key that provably exists (`len() == 1`) became unreachable through `.get()` under any spelling.
- `ToOwned` exists because `Clone` cannot: `str::clone(&self) -> Self` would have to return an unsized `str` by value, which cannot compile. `ToOwned::to_owned(&self) -> Self::Owned` lets the owned type be something else entirely — `String` for `str`, `Vec<T>` for `[T]`.
