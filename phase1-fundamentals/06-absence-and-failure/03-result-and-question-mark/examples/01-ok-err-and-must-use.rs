//! `Result<T, E>` is `Option`'s sibling: instead of "maybe nothing here", it
//! says "maybe, and here is why not."
//!
//!     cargo run -p p1-06-03-result-and-question-mark --example 01-ok-err-and-must-use

fn safe_divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("cannot divide by zero".to_string())
    } else {
        Ok(a / b)
    }
}

fn main() {
    // Two ways out of the same function: a value, or a reason it failed.
    let good = safe_divide(10.0, 2.0);
    let bad = safe_divide(10.0, 0.0);
    println!("good: {good:?}");
    println!("bad:  {bad:?}");

    // `match` handles both arms, exhaustively, the same as any enum.
    match safe_divide(20.0, 4.0) {
        Ok(value) => println!("matched ok:  {value}"),
        Err(reason) => println!("matched err: {reason}"),
    }

    // `.is_ok()` / `.is_err()`, for when you only need the shape, not the
    // value or the reason.
    println!("is_ok:  {}", good.is_ok());
    println!("is_err: {}", bad.is_err());

    // A `Result` is marked `#[must_use]`. The next line throws one away —
    // no `let`, no `?`, nothing — and rustc will not stay quiet about it.
    // This is deliberate, to show you the real warning; it is not an error,
    // so the build still succeeds.
    safe_divide(1.0, 0.0);
}
