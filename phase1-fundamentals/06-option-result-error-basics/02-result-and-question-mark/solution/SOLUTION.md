# Solution

```rust
pub fn chained_division(a: f64, b: f64, c: f64) -> Result<f64, String> {
    let step1 = safe_divide(a, b)?;
    let step2 = safe_divide(step1, c)?;
    Ok(step2)
}
```

The `match`-based equivalent of just the first `?`:

```rust
let step1 = match safe_divide(a, b) {
    Ok(v) => v,
    Err(e) => return Err(e),
};
```

Five lines to say what `safe_divide(a, b)?` says in one — and that pattern
(`Ok(v) => v, Err(e) => return Err(e)`) is *always* what `?` expands to
(roughly), which is exactly why it exists as sugar: the "propagate the
error unchanged, otherwise keep going with the success value" case is so
common that spelling it out every time would be pure noise.

```rust
pub fn parse_positive(s: &str) -> Result<u32, String> {
    s.parse::<u32>().map_err(|e| e.to_string())
}
```

`.parse::<u32>()` returns `Result<u32, ParseIntError>` — a *different*
error type than this function's declared `Result<u32, String>`. `?` only
propagates an error type that already matches (or can convert via the
`From` trait, which `String` and `ParseIntError` don't automatically do
here) — so `.map_err(|e| e.to_string())` converts the error type
explicitly first, turning `Result<u32, ParseIntError>` into
`Result<u32, String>`, which now matches the function's signature and
could even be used with `?` on a later line if needed.

On recall question 4: `Result::Err` carries a *value* (`E`) describing
what went wrong; `Option::None` carries nothing at all, structurally, by
design — `None` means "absent," not "absent, and here's why." If you find
yourself wanting to explain *why* something is missing, that's usually a
sign the function should return `Result`, not `Option`.
