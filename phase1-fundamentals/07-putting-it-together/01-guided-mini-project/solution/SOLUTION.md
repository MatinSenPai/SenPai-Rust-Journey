# Solution — 1.7.1 A guided mini-project

```rust
pub fn add(&mut self, entry: Entry) {
    self.entries.push(entry);
}

pub fn find(&self, title: &str) -> Option<&Entry> {
    for entry in &self.entries {
        if entry.title == title {
            return Some(entry);
        }
    }
    None
}

pub fn titles(&self) -> Vec<&str> {
    let mut out = Vec::with_capacity(self.entries.len());
    for entry in &self.entries {
        out.push(entry.title.as_str());
    }
    out
}

pub fn rate(&mut self, title: &str, rating: Rating) -> Result<(), WatchlistError> {
    for entry in &mut self.entries {
        if entry.title == title {
            entry.status = Status::Finished { rating };
            return Ok(());
        }
    }
    Err(WatchlistError::NotFound(title.to_string()))
}

pub fn rate_from_text(&mut self, title: &str, rating_text: &str) -> Result<(), WatchlistError> {
    let raw: u8 = rating_text.trim().parse()?;
    self.rate(title, Rating::new(raw))
}
```

Five methods, five decisions — none of them arbitrary. The types (`Rating`, `Status`, `Entry`, `WatchlistError`) were already fully given; what's here is just filling in the bodies of these five methods, each exactly as its doc comment said it should behave.

## `add` — ownership, because something has to hold on to it

```rust
self.entries.push(entry);
```

One line. There's nothing clever here, and there shouldn't be — `add` is supposed to keep `entry` forever, and `push` on a `Vec<Entry>` does exactly that. The signature had already done the real work: `entry: Entry`, not `&Entry`, means it stops belonging to the caller the moment it enters this function.

Write this with `entry: &Entry` instead and it simply wouldn't compile — `self.entries.push(entry)` needs an owned `Entry`, not a borrowed look at one. The signature closed off that path before a single line of the body existed.

## `find` — a hand-written loop, because iterators haven't arrived yet

```rust
for entry in &self.entries {
    if entry.title == title {
        return Some(entry);
    }
}
None
```

This is exactly what Phase 2 writes as `self.entries.iter().find(|e| e.title == title)` in one line — but this lesson isn't there yet, so a plain loop with an early `return` does the same job.

The subtler detail is in `&self.entries`: the loop walks over **borrows**, not the elements themselves. `entry` is `&Entry` on every turn, so `Some(entry)` builds exactly the `Option<&Entry>` the signature promised — with no extra `&` written by hand. Loop over `self.entries` without the `&` instead, and the very first turn would take ownership of the whole `Vec`, and the compiler would stop you with an ownership error — the same 1.2.4 rule that a function shouldn't consume something it only needs to read.

## `titles` — a `Vec<&str>`, built with `with_capacity`

```rust
let mut out = Vec::with_capacity(self.entries.len());
for entry in &self.entries {
    out.push(entry.title.as_str());
}
out
```

`with_capacity` wasn't required — `Vec::new()` would have worked too — but since the exact element count is already known (`self.entries.len()`), it's a free allocation that avoids any reallocating partway through the loop; the same capacity argument from 1.2.3.

`entry.title.as_str()` turns a `&String` into a `&str` — no copy, no allocation, just another look at the same buffer. That one line of the loop says exactly what's being built: a vector of looks, not copies.

## `rate` — the one method with a `Result`

```rust
for entry in &mut self.entries {
    if entry.title == title {
        entry.status = Status::Finished { rating };
        return Ok(());
    }
}
Err(WatchlistError::NotFound(title.to_string()))
```

The loop here is over `&mut self.entries`, not `&self.entries` — because this time `entry.status` is actually being changed, and changing something needs an exclusive reference, the same 1.3.1 rule.

The more important detail is where `Err` sits: outside the loop, after every element has been checked with no match. `title.to_string()` is the only place this function allocates, and it only allocates when it's actually needed — if the title is found, the function has already returned with `Ok(())` before that line is ever reached.

## `rate_from_text` — where `?` does two jobs at once

```rust
let raw: u8 = rating_text.trim().parse()?;
self.rate(title, Rating::new(raw))
```

Two lines, and each one is a decision.

The first line: `.trim()` strips the stray whitespace that a real piece of user input almost always has. `.parse::<u8>()` turns the string into a number and returns `Result<u8, ParseIntError>`. `?` on that means: if it's `Ok`, take the value and keep going; if it's `Err`, convert that `ParseIntError` using the `From<ParseIntError> for WatchlistError` impl (written earlier in the same file), and return from `rate_from_text` right now with that error. One symbol, two jobs: unwrapping the value and converting the error's type.

The second line builds nothing new — it just hands the work off to `rate`. Because `rate` already returns exactly the `Result<(), WatchlistError>` that `rate_from_text` is supposed to return too, there's no need for another `?` or `match`; the same value, as it is, is the function's last expression.

And the order here matters: `parse` runs before any lookup happens. If the text is invalid, the function returns with `InvalidRating` before `self.rate` is ever called — even if `title` isn't in the list either. The test `rate_from_text_reports_bad_text_before_checking_the_title` checks exactly that.

## What this lesson was really about

- **A method's signature makes the ownership decision before its body does.** An owned `Entry` in `add` means "I'm keeping this"; a `&Entry` coming out of `find` means "I'm only showing you". When the signature is right, the body usually writes itself.
- **A data-carrying `enum`, paired with an exhaustive `match` over it, together make an invalid state unwritable and forgetting a state a compile error.**
- **String padding and string slicing are two entirely separate rules.** One counts characters, the other counts bytes — and that difference is exactly where a Persian title exposes a silent bug.
- **`Result` is written only for an operation that can genuinely fail.** `add`, `find`, `titles` never fail, and are simpler and more honest for having no `Result` at all.
- **`?` is more than an early `return` — it also calls a `From`.** That one symbol converted a `ParseIntError` into a `WatchlistError` without a `match` written by hand anywhere.
- And most of all: none of these five methods was a new idea. The last thirty lessons were all here already — just, this time, together.
