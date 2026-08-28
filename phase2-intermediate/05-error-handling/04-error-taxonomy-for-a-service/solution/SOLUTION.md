# Solution — 2.5.4 Designing an error taxonomy for a service

```rust
fn validate(title: &str, rating: u8) -> Result<(), ValidationError> {
    if title.trim().is_empty() {
        return Err(ValidationError::EmptyTitle);
    }
    if rating > 10 {
        return Err(ValidationError::RatingOutOfRange { rating });
    }
    Ok(())
}

impl WatchlistStore {
    pub fn new() -> Self {
        WatchlistStore {
            entries: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add_entry(&mut self, title: &str, rating: u8) -> Result<u64, ServiceError> {
        validate(title, rating)?;
        let id = self.next_id;
        self.next_id += 1;
        self.entries.insert(
            id,
            Entry {
                title: title.to_string(),
                rating,
            },
        );
        Ok(id)
    }

    pub fn rating_of(&self, id: u64) -> Result<u8, ServiceError> {
        self.entries
            .get(&id)
            .map(|entry| entry.rating)
            .ok_or(ServiceError::NotFound { id })
    }

    pub fn restore_from_file(&mut self, path: &str) -> Result<usize, ServiceError> {
        let contents = std::fs::read_to_string(path)?;
        let mut added = 0;
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Some((title, rating_text)) = line.split_once('=') else {
                continue;
            };
            let Ok(rating) = rating_text.trim().parse::<u8>() else {
                continue;
            };

            let id = self.next_id;
            self.next_id += 1;
            self.entries.insert(
                id,
                Entry {
                    title: title.trim().to_string(),
                    rating,
                },
            );
            added += 1;
        }
        Ok(added)
    }
}
```

All five follow exactly the shape `ServiceError` and `ValidationError`
already promised — no new method, just the logic that produces and
consumes them.

## `validate` — doesn't pick a category, only decides inside one

```rust
fn validate(title: &str, rating: u8) -> Result<(), ValidationError> {
    if title.trim().is_empty() {
        return Err(ValidationError::EmptyTitle);
    }
    if rating > 10 {
        return Err(ValidationError::RatingOutOfRange { rating });
    }
    Ok(())
}
```

The title is checked first: if both are wrong, the answer is always
`EmptyTitle` — exactly what the spec in the doc comment said. `validate`
has no idea where it will be used later; it just returns a
`ValidationError`. The decision of how to turn that into a `ServiceError`
lives outside this function — exactly the category split `## The concept`
walked through.

## `add_entry` — `?` calls a category that was already written down

```rust
pub fn add_entry(&mut self, title: &str, rating: u8) -> Result<u64, ServiceError> {
    validate(title, rating)?;
    let id = self.next_id;
    self.next_id += 1;
    self.entries.insert(id, Entry { title: title.to_string(), rating });
    Ok(id)
}
```

`validate(title, rating)?` converts a `ValidationError`, if there is one,
into `ServiceError::Validation` on its own — because `#[from]` on that
variant already wrote the conversion down. After that, the id bookkeeping
is plain: read `next_id`'s current value, then increment it. That order —
read first, then bump — is exactly why the first entry that ever succeeds
gets `0`.

## `rating_of` — plain data, no sub-error

```rust
pub fn rating_of(&self, id: u64) -> Result<u8, ServiceError> {
    self.entries
        .get(&id)
        .map(|entry| entry.rating)
        .ok_or(ServiceError::NotFound { id })
}
```

`.get(&id)` gives an `Option<&Entry>`; `.map(...)` pulls the rating out of
it if there is one; `.ok_or(...)` turns that same `Option` into a
`Result`, with `ServiceError::NotFound { id }` when nothing was there.
Notice the exact `id` the caller asked about comes straight back in the
error — not a generic message — because that detail is exactly what's
safe and useful for the "not found" category.

## `restore_from_file` — exactly one way to fail

```rust
pub fn restore_from_file(&mut self, path: &str) -> Result<usize, ServiceError> {
    let contents = std::fs::read_to_string(path)?;
    let mut added = 0;
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((title, rating_text)) = line.split_once('=') else {
            continue;
        };
        let Ok(rating) = rating_text.trim().parse::<u8>() else {
            continue;
        };
        // ... insert and count
    }
    Ok(added)
}
```

The `?` on `read_to_string` is exactly the one place a real `io::Error`
can happen, and `#[from]` on `Internal` converts it automatically. Nothing
after that line can return an error at all — a malformed line or an
unparseable rating is simply skipped with `continue`, never an `Err`.
That's deliberate: the spec said this function has "exactly one way to
fail," and that way is reading the file itself, never what's inside it.

## What this lesson was really about

- **`validate`**: the logic inside one category, kept separate from the
  decision of what that category is.
- **`add_entry`**: `?` only picks a category on its own once you've
  already said how — `#[from]` is that saying-how.
- **`rating_of`**: a small category can be just data; its detail (here,
  `id`) goes out unguarded the moment it's safe and useful.
- **`restore_from_file`**: "this function has exactly one way to fail" is
  not a fact that comes for free — it was built by deliberately skipping
  malformed lines instead of erroring on them.
- None of these four functions reinvented the taxonomy — they only wrote
  the logic that correctly fills the taxonomy already designed for them.
