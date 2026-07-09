# Solution

```rust
pub fn celsius_to_fahrenheit(c: f64) -> f64 {
    round_to_one_decimal(c * 9.0 / 5.0 + 32.0)
}

fn round_to_one_decimal(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}
```

Nothing subtle in the implementation — the interesting part of this lesson
is where each test *lives*, not the math. Three genuinely different test
surfaces exercise this same small crate:

- `src/lib.rs`'s `#[cfg(test)] mod tests` (unit tests) call
  `round_to_one_decimal` **directly**, because unit tests compile as part
  of this crate and privacy doesn't apply to code inside the same crate.
- `tests/conversion_test.rs` (an integration test) only imports
  `celsius_to_fahrenheit` — it has no way to even *name*
  `round_to_one_decimal`, because from that file's perspective, this crate
  is an external dependency, and `round_to_one_decimal` was never `pub`.
- The `/// # Examples` block on `celsius_to_fahrenheit` is a doc test —
  compiled and run by `cargo test`, and would fail the whole test suite the
  moment the function's actual behavior drifted from what the doc comment
  promises. This is what makes documentation "load-bearing" in Rust rather
  than aspirational prose that quietly rots.

On checkpoint question 4: integration tests catch a category of bug unit
tests structurally can't — a public API that *works* when called from
inside the crate (where private helpers, internal invariants, and
convenient shortcuts are all still reachable) but breaks, or was never
actually exposed correctly, when used the way a real external consumer
would use it. A classic example: forgetting to mark something `pub`, or
forgetting to re-export it from the crate root — every unit test still
passes (they don't need the item to be public), but an integration test
(and therefore any real downstream user) can't reach it at all.
