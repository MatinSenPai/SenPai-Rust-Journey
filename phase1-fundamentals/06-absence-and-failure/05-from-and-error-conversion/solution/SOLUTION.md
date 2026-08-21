# Solution — 1.6.5 `From` and error conversion

```rust
use std::num::ParseIntError;

#[derive(Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, PartialEq)]
pub enum ColorError {
    BadChannel(ParseIntError),
    TooDark,
}

impl From<ParseIntError> for ColorError {
    fn from(err: ParseIntError) -> Self {
        ColorError::BadChannel(err)
    }
}

pub fn parse_color(s: &str) -> Result<Color, ColorError> {
    let mut parts = s.split(',');
    let r_str = parts.next().unwrap_or("");
    let g_str = parts.next().unwrap_or("");
    let b_str = parts.next().unwrap_or("");

    let r = r_str.parse::<u8>()?;
    let g = g_str.parse::<u8>()?;
    let b = b_str.parse::<u8>()?;

    if (r as u32) + (g as u32) + (b as u32) < 30 {
        return Err(ColorError::TooDark);
    }

    Ok(Color { r, g, b })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Celsius(pub f64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fahrenheit(pub f64);

impl From<Celsius> for Fahrenheit {
    fn from(value: Celsius) -> Self {
        Fahrenheit(value.0 * 9.0 / 5.0 + 32.0)
    }
}

pub fn all_fahrenheit(values: Vec<Celsius>) -> Vec<Fahrenheit> {
    let mut out = Vec::with_capacity(values.len());
    for value in values {
        out.push(value.into());
    }
    out
}
```

Two of these wrote one `impl From`. The other two used that same impl — one through `?` on an error, one through `.into()` on a conversion that can't fail.

## `impl From<ParseIntError> for ColorError` — the secret itself

```rust
impl From<ParseIntError> for ColorError {
    fn from(err: ParseIntError) -> Self {
        ColorError::BadChannel(err)
    }
}
```

Three lines. All it says is "if you see a `ParseIntError`, put it inside `BadChannel`." This function is never called anywhere in this file directly — `?` finds it and calls it exactly when it needs to, precisely what you saw in "The concept".

## `parse_color` — three bare `?`s, thanks to that one `impl`

```rust
let r = r_str.parse::<u8>()?;
let g = g_str.parse::<u8>()?;
let b = b_str.parse::<u8>()?;
```

Three calls to `.parse::<u8>()`, all three producing `ParseIntError`, all three converted to `ColorError` by that same one `impl From` above. Without it, these same three lines would each need a `match` or a `.map_err(ColorError::BadChannel)` — exactly the difference you saw in "The manual way".

Missing fields are handled for free too: `s.split(',')` on `"1,2"` yields only two pieces; `parts.next().unwrap_or("")` leaves the third as an empty string, and `"".parse::<u8>()` itself fails with `IntErrorKind::Empty`. No separate path for "a field is missing" was needed — the same "not a valid number" path already covered it.

The darkness rule comes after all three `?`s, not before:

```rust
if (r as u32) + (g as u32) + (b as u32) < 30 {
    return Err(ColorError::TooDark);
}
```

All three numbers have to be valid before their sum means anything. Summing three `u8`s can reach up to 765, which doesn't fit in a `u8` itself, so each one is cast to `u32` before adding — [1.1.2](../../../01-foundations/02-scalar-types-and-overflow/README.md) — and `TooDark` is built directly, with no `From` at all, because this error isn't converted from anything; this function is the one that detects it.

## `impl From<Celsius> for Fahrenheit` — the same trait, no error involved

```rust
impl From<Celsius> for Fahrenheit {
    fn from(value: Celsius) -> Self {
        Fahrenheit(value.0 * 9.0 / 5.0 + 32.0)
    }
}
```

`From` promises "always succeeds"; this conversion always does — there is no floating-point value this formula fails on. There's no `Result` in the signature either, because there's nothing to report.

## `all_fahrenheit` — `.into()` instead of rewriting the formula

```rust
let mut out = Vec::with_capacity(values.len());
for value in values {
    out.push(value.into());
}
out
```

`value.into()` is exactly `Fahrenheit::from(value)`, just read from the target's side. The compiler gets the target from `all_fahrenheit`'s own signature: the return type is `Vec<Fahrenheit>`, so every `push` must produce a `Fahrenheit`, so `.into()` knows which `From` to call — the one written a few lines up.

Writing `Fahrenheit(value.0 * 9.0 / 5.0 + 32.0)` again, right here, would also pass the tests — but then the formula would live in two places. The `all_fahrenheit_matches_the_same_conversion_used_alone` test checks exactly that: the result of `all_fahrenheit` has to match calling that same `From` directly. `Vec::with_capacity(values.len())` is a callback to [1.2.3](../../../02-ownership-and-memory/03-clone-and-copy/README.md) too: the length is already known, so there's no reason to let the Vec reallocate as it grows.

## What this lesson was really about

- **`?` calls `From::from` on the error, then returns the converted `Err`.** That one sentence completes `?`'s whole behavior.
- **Write one `impl From` and it works everywhere** — behind `?` on an error, and behind `.into()` on a conversion that can't fail.
- **Nobody writes `Into` by hand.** The standard library's blanket impl builds one from every `From`.
- **Not every variant comes from `From`.** `TooDark` is built directly, because its error isn't converted from anything.
- **A `.into()` has to get its target from somewhere** — a return type, a parameter type, or an explicit `let` type. If none of them say, that's `E0282`.
