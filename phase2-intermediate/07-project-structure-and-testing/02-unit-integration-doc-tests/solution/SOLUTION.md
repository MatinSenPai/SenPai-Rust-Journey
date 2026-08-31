# Solution — 2.7.2 Unit, integration, and doc tests

```rust
fn round_to_one_decimal(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

pub fn celsius_to_fahrenheit(c: f64) -> f64 {
    round_to_one_decimal(c * 9.0 / 5.0 + 32.0)
}

pub fn average_celsius(readings: &[f64]) -> f64 {
    assert!(!readings.is_empty(), "readings must not be empty");
    let sum: f64 = readings.iter().sum();
    round_to_one_decimal(sum / readings.len() as f64)
}

pub fn parse_celsius(input: &str) -> Result<f64, String> {
    input
        .parse::<f64>()
        .map_err(|_| format!("not a valid number: {input:?}"))
}
```

Nothing subtle in the arithmetic — the interesting part of this lesson is where each test *lives*, not the math. Three genuinely different test surfaces exercise these same four small functions.

## `round_to_one_decimal` — private, and only a unit test reaches it

The `#[cfg(test)] mod tests` block at the bottom of `src/lib.rs` calls this function **directly**, because unit tests compile as part of this crate and privacy doesn't apply to code inside a shared crate. `tests/public_api.rs` has no way to even *name* this function — from that file's point of view, this crate is an external dependency, and `round_to_one_decimal` was never `pub`. `examples/02-private-item-is-unreachable.rs` makes exactly this attempt and fails with `E0603`.

## `average_celsius` — an explicit contract, documented in two places

The implementation starts with an `assert!`, not with arithmetic — exactly what the comment above the function, under `# Panics`, already spelled out in words. The panic message (`"readings must not be empty"`) is the exact same string that both the `#[should_panic(expected = "...")]` unit test checks, and the `should_panic` fence inside the comment demonstrates (which only proves the panic happens, not that it checks the message). Three places — the comment, the unit test, the doc test — restate one contract from three different angles.

## `parse_celsius` — a `Result`, with both paths documented and tested

`.map_err(|_| ...)` discards the real `ParseFloatError` and replaces it with a message of our own — the exact same message written in the comment's `# Errors` section, and checked with `assert_eq!` in the doc test right beneath it. (In a real crate, you'd probably reach for your own error type instead of a bare `String` — [2.5.1](../../../05-error-handling/01-custom-error-types/README.md) teaches exactly that; this lesson chose simplicity on purpose so the focus stays on the tests themselves.)

## What this lesson was really about

- **The unit test** (`mod tests` at the bottom of `src/lib.rs`) reaches all four functions, private `round_to_one_decimal` included — because it compiles as part of the crate itself.
- **The integration test** (`tests/public_api.rs`) only sees the three `pub` functions — exactly what a real user of this crate sees, and nothing more.
- **The doc tests** (five code fences, inside the comments of those same three `pub` functions) are actually compiled and run by `cargo test`; if any one function's behavior ever drifts from what its comment promises, this is exactly where and when the build breaks.

All three together, over a twenty-line crate, prove exactly what this lesson promised from its first section: these three kinds of test aren't competing with each other — each one covers a different layer of the same crate.
