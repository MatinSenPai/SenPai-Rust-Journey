# Solution

```rust
pub(crate) fn normalize_rating(raw: u8) -> u8 {
    raw.min(10)
}
```

The interesting detail here is what's *missing*: no lower-bound clamp.
`u8` is an unsigned 8-bit integer — it cannot represent a negative number at
all, so `raw` is already guaranteed `>= 0` by the type system itself. This
is a small but real example of a broader Rust habit: pick the narrowest
type that makes an invalid state unrepresentable, and you get to delete a
whole class of validation code. In a language without unsigned integer
types you'd have to write (and test!) the lower-bound check too.

```rust
pub fn public_rating_band(&self) -> &'static str {
    match self.internal_rating {
        0..=3 => "low",
        4..=7 => "medium",
        _ => "high",
    }
}
```

`0..=3` is an inclusive range pattern inside a `match` — it reads almost
exactly like the English "0 through 3." Because `internal_rating` is a
`u8` clamped to `0..=10` by construction, the three arms `0..=3`, `4..=7`,
and `_` (catch-all, covering `8..=10`) are exhaustive; the compiler would
refuse to compile a `match` on an integer type that didn't cover every
possible value, which is exactly why the catch-all `_` arm is there instead
of spelling out `8..=10` explicitly (both work — `_` is just shorter once
you've covered the ranges you care about).

## On checkpoint question 1

If an external crate depended on this one and wrote
`my_crate::Anime::new("X", 5).internal_rating`, it would fail **at compile
time**, not run time — with an error like "field `internal_rating` of
struct `Anime` is private." `pub(crate)` items don't exist at all from the
outside crate's point of view; there's no runtime check to bypass, no
panic to catch, because the compiler for the *external* crate never even
generates code that references the field — it simply can't name it. This
is the same category of guarantee as ownership and borrowing: enforced
before the program ever runs, not defended against while it's running.

## On checkpoint question 3

`pub use catalog::Anime;` re-exports the *name* `Anime` at a new path; the
type is still physically defined inside `catalog`. The reason a real
library does this instead of making `catalog` itself `pub` is API
stability and ergonomics: if `catalog` were `pub`, every caller would have
to write `my_crate::catalog::Anime`, and — worse — you'd be committing to
that exact module path as part of your public API forever. If you later
split `catalog` into `catalog::series` and `catalog::studio` submodules
(exactly as the lesson's intro example mentions), every external caller's
code breaks. With a re-export, you can reorganize the internal module tree
freely — the one line `pub use catalog::series::Anime;` absorbs the change,
and every external caller's `my_crate::Anime` keeps compiling untouched.
