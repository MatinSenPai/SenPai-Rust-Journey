# 08.4 — `TryFrom` and fallible conversions

You met `From`/`Into` back in the error-handling module: `From` is the
trait for conversions that **cannot fail** — `String::from("hi")`,
`i64::from(some_u32)`, your error types absorbing `io::Error`. But most
interesting conversions *can* fail: not every `u8` is a valid percentage,
not every `String` is an email address, not every `u64` fits in a `u32`.
Rust's answer is `TryFrom` — identical shape, but returning a `Result`:

```rust
impl TryFrom<u8> for Percentage {
    type Error = ValidationError;
    fn try_from(raw: u8) -> Result<Self, Self::Error> { ... }
}
```

The associated `Error` type means every conversion names *its own*
failure type — no one-size-fits-all exception. And just like
`From` auto-provides `Into`, implementing `TryFrom<u8> for Percentage`
gives you the mirror `TryInto` for free: `let p: Percentage =
42u8.try_into()?`. Implement `TryFrom`, use whichever direction reads
better. The rule of thumb for choosing between the two traits: if you
can convert **every** input value, implement `From`; if even one input
must be rejected, `TryFrom` — an `impl From` that panics on "bad" input
is a lie in the type system.

## Numeric narrowing: where Python can't even see the problem

Python has one `int`, arbitrary precision — `2**64` is just another
integer, and "doesn't fit" isn't a thing. Rust's fixed-width integers
make narrowing a real decision, and gives you two very different tools:

```rust
let big: u64 = 300;
let a = big as u8;                       // 44 — silently truncates bits!
let b: Result<u8, _> = big.try_into();   // Err(TryFromIntError) — honest
```

`as` never fails; it just chops (300 mod 256 = 44). That's occasionally
what you want, but as a *default* it's a bug factory. The standard
library implements `TryFrom` between all integer pairs where the
conversion can fail, so `big.try_into()` gives you a `Result` and makes
the "didn't fit" case a value you must handle — `?` it upward,
`.unwrap_or(...)` it to a fallback, or `match` on it. This lesson's
`saturating_narrow` picks the fallback route: clamp to `u32::MAX` rather
than truncate or crash.

## The validated-newtype pattern ("parse, don't validate")

The Python reflex is a checker function — `def is_valid_email(s): ...` —
called at the boundary, after which everyone downstream *hopes* the check
happened. Django forms and pydantic push the same idea further: validate
at the edge, pass plain `str`s around inside. The Rust idiom goes one
step beyond: wrap the value in a **newtype whose constructor is the
validation**:

```rust
pub struct EmailAddress(String);   // field is PRIVATE

impl TryFrom<String> for EmailAddress { ... }   // the only way in
```

Because the field is private, the *only* way to obtain an `EmailAddress`
is through `try_from`, so holding one **is proof it passed validation**.
A function signature `fn notify(to: &EmailAddress)` can't even be called
with an unvalidated string — the "did anyone check this?" question is
answered by the type checker, not by code review. That's the slogan
"parse, don't validate": a `bool`-returning checker produces knowledge
that evaporates immediately; a parse into a richer type produces
knowledge that sticks to the value. (This is pydantic's philosophy, but
enforced at compile time and with zero runtime cost after construction —
a `Percentage(u8)` is exactly one byte.)

Two mechanical notes for the exercises. First, give the newtype an
explicit read-only accessor (`.value()`, `.as_str()`) instead of making
the field `pub` — a `pub` field would let anyone bypass the invariant
with `email.0 = "garbage".into()`. Second, on the error path, hand the
rejected input back inside the error (`InvalidEmail(String)`): the caller
gave you ownership, and returning it lets them log or reuse it without a
clone.

## Your task

In `src/lib.rs` (the `ValidationError` enum and its `Display` impl are
already written — read them first):

- `TryFrom<u8> for Percentage` — accept `0..=100`, reject the rest with
  `PercentageOutOfRange`.
- `TryFrom<String> for EmailAddress` — deliberately minimal validation:
  exactly one `@`, non-empty on both sides. (Real email validation is a
  swamp; resist the regex.)
- `saturating_narrow` — `u64` → `u32` via `try_into`, clamping overflow
  to `u32::MAX`.

## Next

`solution/SOLUTION.md` — but only after a real attempt.
