# Solution — 2.5.1 Custom error types and `std::error::Error`

```rust
impl std::fmt::Display for EntryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryError::BlankName => write!(f, "entry is missing a name"),
            EntryError::BadScore(source) => write!(f, "invalid score: {source}"),
            EntryError::ScoreTooHigh(score) => {
                write!(f, "score {score} is above the maximum of 9999")
            }
        }
    }
}

impl From<ParseIntError> for EntryError {
    fn from(source: ParseIntError) -> Self {
        EntryError::BadScore(source)
    }
}

pub fn parse_entry(line: &str) -> Result<LeaderboardEntry, EntryError> {
    let (name, score_str) = line.split_once(':').unwrap_or((line, ""));
    let name = name.trim();
    if name.is_empty() {
        return Err(EntryError::BlankName);
    }
    let score: u32 = score_str.trim().parse()?;
    if score > 9999 {
        return Err(EntryError::ScoreTooHigh(score));
    }
    Ok(LeaderboardEntry {
        name: name.to_string(),
        score,
    })
}
```

None of this needed anything beyond what "The concept" showed you — the same three pieces, aimed at a different domain.

## `Display` — three arms, one per variant

```rust
match self {
    EntryError::BlankName => write!(f, "entry is missing a name"),
    EntryError::BadScore(source) => write!(f, "invalid score: {source}"),
    EntryError::ScoreTooHigh(score) => {
        write!(f, "score {score} is above the maximum of 9999")
    }
}
```

The spec stated all three strings exactly. `BadScore`'s arm folds the wrapped `ParseIntError`'s own `Display` text straight in via `{source}` — the same trick `ReviewError` used in the lesson body, so no information from the underlying parse failure gets lost even in the message a person reads.

## `impl std::error::Error for EntryError {}` — given, and why it needed nothing

This one was already in the skeleton, untouched. `Debug` (derived at the top of the file) and `Display` (just written) are `Error`'s two supertraits, and both already exist by the time this line is reached — so there's nothing left for the body to do. `source()` keeps its default, returning `None`; `EntryError` is a root-cause error, never itself caused by another error.

## `From<ParseIntError> for EntryError` — one line, one variant

```rust
EntryError::BadScore(source)
```

The same shape 1.6.5 taught: wrap the foreign error in the one variant that means "this specific kind of foreign failure." Nothing else to decide — `BadScore` only ever means one thing.

## `parse_entry` — three checks, in the order the spec stated

```rust
let (name, score_str) = line.split_once(':').unwrap_or((line, ""));
let name = name.trim();
if name.is_empty() {
    return Err(EntryError::BlankName);
}
let score: u32 = score_str.trim().parse()?;
if score > 9999 {
    return Err(EntryError::ScoreTooHigh(score));
}
```

`.split_once(':').unwrap_or((line, ""))` is what makes a line with no `:` behave as "empty score half," exactly as the doc comment promised — no separate branch needed for it. `score_str.trim().parse()?` is where the `From` impl above actually gets used: a bare `?`, no `.map_err(...)`. The `> 9999` check runs only after a successful parse, which is why `ScoreTooHigh` always carries a real, already-valid `u32` rather than something that merely looked like one.

## What this lesson was really about

- **The trait's shape decides how small an `impl` can be.** `Error: Debug + Display` plus one defaulted method means a root-cause error's `impl Error` is often empty — the work already happened in `Debug`/`Display`, which you'd write anyway.
- **A `String` and a `Box<dyn Error>` both erase structure; an enum doesn't.** `EntryError` is `match`able. Neither of the alternatives is, at least not without extra work — a `Box<dyn Error>`'s extra work is exactly [2.5.2](../02-error-source-chains/README.md)'s subject.
- **Not every variant needs an `impl From`.** `BlankName` and `ScoreTooHigh` are both built directly, right where you already have the information; only `BadScore`, which wraps a foreign type, benefits from one.
