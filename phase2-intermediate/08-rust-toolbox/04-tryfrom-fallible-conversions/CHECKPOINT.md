# Checkpoint

1. State the rule of thumb for choosing between `From` and `TryFrom`.
   Why is an `impl From` that panics on some inputs described as "a lie
   in the type system" — who gets hurt, and when?
2. You only wrote `impl TryFrom<u8> for Percentage`, yet the test
   `let p: Percentage = 42u8.try_into().unwrap();` compiles. Where does
   that `try_into` come from?
3. `300u64 as u8` and `u8::try_from(300u64)` produce completely
   different results. What exactly does each one produce, and *why* is
   the `as` result 44? In what (rare) situation would you genuinely
   want `as`-style truncation?
4. `Percentage`'s inner `u8` is private and only readable through
   `.value()`. Walk through what specific guarantee is lost the moment
   you change it to `pub struct Percentage(pub u8)` — even though every
   existing caller still compiles unchanged.
5. Contrast `fn is_valid_email(s: &str) -> bool` with
   `TryFrom<String> for EmailAddress` in terms of what a *downstream*
   function signature can promise. Which Python tools play the same role
   as the newtype here, and what can't they enforce that Rust can?
6. `ValidationError::InvalidEmail(String)` carries the rejected string
   by value. What does this cost, what does it buy the caller, and why
   is it natural here given that `try_from` took `String` by value in
   the first place?
