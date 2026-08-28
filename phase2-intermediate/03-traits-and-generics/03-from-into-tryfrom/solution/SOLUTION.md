# Solution

```rust
impl TryFrom<u8> for Percentage {
    type Error = ValidationError;

    fn try_from(raw: u8) -> Result<Self, Self::Error> {
        if raw <= 100 {
            Ok(Percentage(raw))
        } else {
            Err(ValidationError::PercentageOutOfRange(raw))
        }
    }
}
```

Nothing clever, deliberately — the value of this impl is *where* it
lives, not how it's written. Because `Percentage`'s field is private,
this `if` is the only place in the universe where a `Percentage` can be
born, so every `Percentage` anywhere in the program has passed it. Make
the field `pub` and that global guarantee evaporates without a single
caller changing: `Percentage(255)` becomes constructible anywhere, and
"is it really 0–100?" goes back to being a code-review question instead
of a type-system fact. Note also what the test
`implementing_try_from_provides_try_into_for_free` demonstrates: we
never wrote `TryInto` anywhere. The standard library has a blanket
`impl<T, U> TryInto<U> for T where U: TryFrom<T>`, the exact mirror of
the `From`/`Into` bridge you've relied on since the error-handling
module.

```rust
impl TryFrom<String> for EmailAddress {
    type Error = ValidationError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        let valid = matches!(
            raw.split_once('@'),
            Some((local, domain))
                if !local.is_empty() && !domain.is_empty() && !domain.contains('@')
        );

        if valid {
            Ok(EmailAddress(raw))
        } else {
            Err(ValidationError::InvalidEmail(raw))
        }
    }
}
```

The shape of this function is dictated by the borrow checker, and it's
worth understanding why. The "obvious" version — one `match` on
`raw.split_once('@')` whose success arm returns `Ok(EmailAddress(raw))`
— doesn't compile: `split_once` hands back `&str` slices *borrowing
`raw`*, the match scrutinee keeps that borrow alive for the whole
`match`, and you can't move `raw` while it's borrowed (E0505). So we
split the work into two phases: first reduce the borrow to a plain
`bool` (using `matches!` with a guard — pattern matching from lesson
08.1 earning its keep), then, with all borrows dead, move `raw` into
whichever side of the `Result` it belongs. Handing the rejected string
back inside `InvalidEmail(raw)` costs nothing (we owned it anyway) and
saves the caller a clone when they want to log it — the design note
from the README made concrete. The validation itself is knowingly
minimal: `a@b` passes. That's the honest choice for a lesson — real
email validation is an RFC swamp, and "send a confirmation mail" is the
only check that actually proves deliverability.

```rust
pub fn saturating_narrow(value: u64) -> u32 {
    value.try_into().unwrap_or(u32::MAX)
}
```

The one-liner is the payoff of std implementing `TryFrom<u64> for u32`:
the fallible cast is already a `Result`, so *policy* becomes a
combinator choice. `?` would propagate, `unwrap_or(u32::MAX)` clamps,
`unwrap` would crash — one expression each, all explicit. Compare `value
as u32`, which encodes the *silent-truncation* policy without looking
like a decision at all (`u64::MAX as u32` is `4294967295`… but `(u32::MAX
as u64 + 1) as u32` is `0`). Since Rust 1.79 there's also
`u32::try_from(value).unwrap_or(u32::MAX)` spelled as a saturating cast
in some codebases via `num` crates or manual `min` — but the
`try_into().unwrap_or(...)` form is the idiom you'll actually meet in
the wild, and the one that generalizes to every narrowing pair.
