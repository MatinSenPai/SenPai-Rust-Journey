# 06.2 — `Result` and the `?` operator

## `Option` for absence, `Result` for failure with a reason

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

Where `Option<T>` says "there might be nothing here," `Result<T, E>` says
"this might fail, and here's *why*." You've already used it indirectly:
`"42".parse::<i32>()` returns `Result<i32, ParseIntError>`, not just
`Option<i32>`, because when parsing fails, "the string wasn't a valid
number" is more useful to the caller than a bare `None`.

```rust
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("cannot divide by zero".to_string())
    } else {
        Ok(a / b)
    }
}
```

## Handling a `Result`

```rust
match divide(10.0, 2.0) {
    Ok(value) => println!("Result: {value}"),
    Err(e) => println!("Error: {e}"),
}

divide(10.0, 0.0).unwrap();          // panics with the Err's message
divide(10.0, 0.0).unwrap_or(0.0);    // 0.0, no panic
```

## The `?` operator — propagate errors without a `match` at every step

```rust
fn compute(a: f64, b: f64, c: f64) -> Result<f64, String> {
    let step1 = divide(a, b)?;   // if divide returns Err, this function
    let step2 = divide(step1, c)?; // immediately returns that Err too
    Ok(step2)
}
```

`expr?` is shorthand for "if this is `Ok(value)`, unwrap to `value` and
keep going; if it's `Err(e)`, return `Err(e)` from the *current* function
right now." This is what makes Rust's error handling ergonomic instead of
a `match` pyramid at every fallible call — you'll use `?` constantly
starting in Phase 3, where nearly every database call and HTTP handler
returns a `Result`. The catch: `?` can only be used inside a function whose
own return type is `Result` (or `Option`) with a compatible error type —
you'll hit a compile error immediately if you try it inside `fn main()`
without adjusting `main`'s signature (try it in the recall questions).

## Your task

Implement the functions in `src/lib.rs`.

## Next

`solution/SOLUTION.md` — but only after a real attempt.
