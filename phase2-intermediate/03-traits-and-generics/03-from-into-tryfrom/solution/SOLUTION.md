# Solution

```rust
impl From<Percentage> for f64 {
    fn from(value: Percentage) -> Self {
        f64::from(value.0) / 100.0
    }
}
```

This is the whole point of putting it next to `TryFrom` in the same file:
there is nothing here to reject. `value.0` is a `u8` that has already
been through `TryFrom<u8> for Percentage` once — that's the only door in
— so it is already known to be `0..=100`. Dividing a known-good number by
`100.0` cannot fail, cannot panic, cannot produce a value outside
`0.0..=1.0`. That is exactly what makes it a `From` and not a `TryFrom`:
the signature `fn from(value: Percentage) -> f64` has nowhere to put an
`Err` even if you wanted one. The inner `f64::from(value.0)` is 01's
widening conversion again — a `u8` always fits inside an `f64` — so this
one function quietly uses both traits this lesson teaches: `From` to
widen the `u8`, `From` again to describe the whole conversion.

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
the `From`/`Into` bridge 1.6.5 showed you.

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
directly — doesn't compile: `split_once` hands back `&str` slices
*borrowing `raw`*, the match scrutinee keeps that borrow alive for the
whole `match`, and you can't move `raw` while it's still borrowed
(E0505). So the work is split into two phases: first reduce the borrow
down to a plain `bool` (`matches!` with a guard does this in one
expression), then, once every borrow of `raw` has ended, move `raw` into
whichever side of the `Result` it belongs on. Handing the rejected
string back inside `InvalidEmail(raw)` costs nothing — we owned it
anyway — and saves the caller a clone if they want to log it or show it
back to whoever typed it. The validation itself is knowingly minimal:
`a@b` passes. That's the honest choice for a lesson — real email
validation is an RFC swamp, and "send a confirmation email" is the only
check that actually proves deliverability.

```rust
pub fn saturating_narrow(value: u64) -> u32 {
    value.try_into().unwrap_or(u32::MAX)
}
```

The one-liner is the payoff of the standard library already implementing
`TryFrom<u64> for u32`: the fallible cast arrives as a `Result`, so
*policy* becomes a combinator choice instead of a control-flow problem.
`?` would propagate the error, `.unwrap_or(u32::MAX)` clamps it,
`.unwrap()` would crash on it — one expression each, and every one of
them honest about what it does. Compare `value as u32`, which bakes in
the *silent-truncation* policy without looking like a decision at all:
`4_294_967_295u64 as u32` is `4294967295` (fits, no surprise), but
`4_294_967_296u64 as u32` — one more — is `0`. `try_into().unwrap_or(...)`
is the form you'll meet in real code, and it generalizes to every
narrowing pair the standard library defines, not just this one.
