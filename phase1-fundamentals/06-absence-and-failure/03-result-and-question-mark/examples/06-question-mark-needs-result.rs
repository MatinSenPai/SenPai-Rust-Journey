//! DELIBERATELY BROKEN — expected: E0277.
//!
//!     cargo run -p p1-06-03-result-and-question-mark --example 06-question-mark-needs-result --features broken

fn safe_divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("cannot divide by zero".to_string())
    } else {
        Ok(a / b)
    }
}

fn main() {
    // `main` here returns `()`, not a `Result` — so there is nowhere for
    // an early `Err` to go.
    let value = safe_divide(10.0, 0.0)?;
    println!("{value}");
}
