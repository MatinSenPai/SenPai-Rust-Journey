//! Five ways to work with a `Result` without writing `match` every time.
//!
//!     cargo run -p p1-06-03-result-and-question-mark --example 03-combinators

fn safe_divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("cannot divide by zero".to_string())
    } else {
        Ok(a / b)
    }
}

fn main() {
    // `.map()` transforms the `Ok` value and leaves `Err` alone — the same
    // shape `Option::map` had in the last lesson.
    println!("{:?}", safe_divide(10.0, 2.0).map(|v| v * 100.0));
    println!("{:?}", safe_divide(10.0, 0.0).map(|v| v * 100.0));

    // `.map_err()` is the mirror image: it transforms the `Err`, leaving
    // `Ok` alone. Useful for turning one error type into another.
    let renamed = safe_divide(10.0, 0.0).map_err(|e| format!("division failed: {e}"));
    println!("{renamed:?}");

    // `.and_then()` chains a second fallible step. If the first is `Err`,
    // the second never runs — its closure isn't even called.
    println!(
        "{:?}",
        safe_divide(10.0, 2.0).and_then(|v| safe_divide(v, 5.0))
    );
    println!(
        "{:?}",
        safe_divide(10.0, 0.0).and_then(|v| safe_divide(v, 5.0))
    );

    // `.unwrap_or()` gets a plain value out, with a fallback for `Err`.
    // No panic is possible.
    println!("{}", safe_divide(10.0, 2.0).unwrap_or(0.0));
    println!("{}", safe_divide(10.0, 0.0).unwrap_or(0.0));

    // `.ok()` throws the error away on purpose and gives you an `Option`.
    // Reach for it once you truly do not care why something failed.
    println!("{:?}", safe_divide(10.0, 2.0).ok());
    println!("{:?}", safe_divide(10.0, 0.0).ok());
}
