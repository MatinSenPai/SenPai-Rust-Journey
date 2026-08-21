# Solution — 1.1.2 Scalar types and overflow

```rust
pub fn clamped_add(a: u8, b: u8) -> u8 {
    a.saturating_add(b)
}

pub fn wrapped_add(a: u8, b: u8) -> u8 {
    a.wrapping_add(b)
}

pub fn is_close(a: f64, b: f64) -> bool {
    (a - b).abs() < TOLERANCE
}

pub fn to_fahrenheit(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0
}

pub const TOLERANCE: f64 = 1e-9;
```

## The two additions

Both are one method call, and that's the lesson: **Rust already has the behaviour you want, named after what it does.** If you wrote a manual `if a > 255 - b { 255 } else { a + b }`, it's correct and it's what you'd write in a language without these methods — but the named version says its intent, can't be subtly wrong, and compiles to the same machine code.

Worth noticing that `a + b` would have passed neither test reliably. In debug it panics on the overflow cases; in release it wraps, which would accidentally pass `wrapped_add` and fail `clamped_add`. **A test that passes only in release is a bug you haven't found yet** — one reason `cargo test` uses a debug build by default.

## `is_close` — why `.abs()` and not a comparison

```rust
(a - b).abs() < TOLERANCE
```

The test insists that `is_close(0.3, 0.1 + 0.2)` works as well as `is_close(0.1 + 0.2, 0.3)`. If you wrote `a - b < TOLERANCE` without `.abs()`, one direction gives a negative number, which is always less than the tolerance — so your function would return `true` for `is_close(1.0, 1000.0)`.

That's a real bug and a very easy one to ship. `.abs()` is what makes the comparison about *distance* rather than *order*.

### On `TOLERANCE` versus `f64::EPSILON`

`f64::EPSILON` is roughly `2.2e-16` — the smallest difference that's meaningful *for values near 1.0*. It's the wrong tolerance for numbers in the millions, where the gap between representable values is far larger than epsilon, and also wrong for numbers near zero, where it's needlessly strict.

`1e-9` is a blunt fixed tolerance. It works fine for this lesson and for most everyday code. Knowing that "the right tolerance depends on the magnitude of the numbers" is the part to carry forward; production numerical code uses relative comparison for exactly this reason.

## `to_fahrenheit` — why `9.0` and not `9`

```rust
celsius * 9.0 / 5.0 + 32.0
```

Write `celsius * 9 / 5` and it doesn't compile: `celsius` is an `f64` and `9` is an integer, and Rust performs no automatic conversion between them. You get `E0308`.

This is the same strictness as `"7"` versus `7` from Phase 0, applied to numbers. It's occasionally annoying and it's why `9.0 / 5.0` is spelled out.

There's a real bug hiding behind that annoyance. In a language that does convert automatically, `celsius * 9 / 5` with integer division silently truncates: 37°C becomes 98°F instead of 98.6°F. Rust refuses to compile the ambiguous version, so that bug can't reach you.

## What this lesson was really about

Not these four functions. It was the habit:

- **Overflow is a decision.** `a + b` is you declining to make it.
- **Float equality is a decision.** `==` is you getting it wrong.
- **Numeric types don't convert themselves**, and that strictness is buying you something.

The exercises are small so that the decisions are visible. In Phase 3, when you're computing pagination offsets and currency totals inside a web service, they'll be exactly the same decisions with real consequences.
