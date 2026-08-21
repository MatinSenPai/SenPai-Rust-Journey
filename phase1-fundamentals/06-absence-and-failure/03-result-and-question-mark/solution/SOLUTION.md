# Solution — 1.6.3 `Result` and the question mark

```rust
pub fn safe_divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("cannot divide by zero".to_string())
    } else {
        Ok(a / b)
    }
}

pub fn parse_quantity(s: &str) -> Result<u32, String> {
    s.parse::<u32>().map_err(|e| e.to_string())
}

pub fn chained_division(a: f64, b: f64, c: f64) -> Result<f64, String> {
    let step1 = safe_divide(a, b)?;
    let step2 = safe_divide(step1, c)?;
    Ok(step2)
}

pub fn half_or_zero(input: &str) -> f64 {
    input.parse::<f64>().map(|value| value / 2.0).unwrap_or(0.0)
}

pub fn parsed_and_divided(input: &str, divisor: f64) -> Option<f64> {
    input
        .parse::<f64>()
        .ok()
        .and_then(|value| safe_divide(value, divisor).ok())
}
```

Five functions, five facets of this lesson: building one by hand, `.parse()`, `?`, and two combinators that deliberately turn a `Result` into something simpler.

## `safe_divide` — exactly what you saw in "The concept"

```rust
if b == 0.0 {
    Err("cannot divide by zero".to_string())
} else {
    Ok(a / b)
}
```

Two ways out, both explicit. No `panic!`, no `None` throwing the reason away — just an ordinary `if` building one of the enum's two arms. This function is the base of the rest of the lesson; the other three call it.

## `parse_quantity` — `.parse()` plus `.map_err()`

```rust
s.parse::<u32>().map_err(|e| e.to_string())
```

`.parse::<u32>()` already gives back a `Result<u32, ParseIntError>` — half the work is done. The problem is the function's signature promised `Result<u32, String>`, not `Result<u32, ParseIntError>`. `.map_err()` fills exactly that gap: it leaves `Ok` alone, and on `Err` calls whatever function you hand it — here, `.to_string()`, which calls `Display` on the `ParseIntError` and returns that same familiar sentence.

Writing a `match` would have worked too:

```rust
match s.parse::<u32>() {
    Ok(n) => Ok(n),
    Err(e) => Err(e.to_string()),
}
```

But when what needs doing to `Ok` is "nothing" and what needs doing to `Err` is "call a function on it," `.map_err()` says the same sentence more briefly.

## `chained_division` — `?` twice

```rust
let step1 = safe_divide(a, b)?;
let step2 = safe_divide(step1, c)?;
Ok(step2)
```

Each `?` replaces a full four-line `match`. If `safe_divide(a, b)` fails, the function's second line never runs — that same `Err` returns immediately. The tests check both failure paths separately, to make sure this early exit genuinely works, not just that the function is correct on the happy path.

## `half_or_zero` — `.map()` and `.unwrap_or()` back to back

```rust
input.parse::<f64>().map(|value| value / 2.0).unwrap_or(0.0)
```

Three steps, one line: parse, halve it if that succeeded, give back `0.0` if anything along the way failed. This could have been written with `match` too, but when the chain of transformations is this short, the combinators say exactly what the function does — no more.

A subtle point: `.map()` comes before `.unwrap_or()`, not after. Swap them (`.unwrap_or(0.0).map(...)`) and it doesn't compile at all — `.unwrap_or()` on a `Result<f64, _>` hands back a plain `f64`, and `f64` has no method called `.map()`.

## `parsed_and_divided` — `.ok()` twice, with `.and_then()` between

```rust
input
    .parse::<f64>()
    .ok()
    .and_then(|value| safe_divide(value, divisor).ok())
```

Two fallible steps back to back here: parsing, then dividing. Both can fail, and by the end all that's wanted is `Some`/`None` — the exact reason for failure no longer matters.

The first `.ok()` turns `Result<f64, ParseIntError>` into `Option<f64>`. `.and_then()` only calls its closure when that's `Some`; the closure itself calls `safe_divide`, and the second `.ok()` turns its `Result<f64, String>` into an `Option<f64>` too. The result is exactly the type `.and_then()` expects its closure to return, so the `Option`s never nest.

## What this lesson was really about

- **`Result<T, E>` is an ordinary enum** with two arms, one of which carries a reason — the thing `Option` never had.
- **`?` compresses a `match`**, not something new. `chained_division` does exactly what could have been written by hand, just shorter.
- **Combinators are for when your behaviour on `Ok`/`Err` fits in one expression** — `.map`, `.map_err`, `.and_then`, `.unwrap_or`, `.ok()`. Once the behaviour gets more complex (nested conditions, multiple lines of logic), `match` is still more readable.
- **`.ok()` is a deliberate exit door**: when you're moving from the world of "why did it fail" to the world of "is there anything here at all," this is exactly the method that draws that line.
