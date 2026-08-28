//! DELIBERATELY BROKEN — expected: a run-time panic, `called
//! `Result::unwrap()` on an `Err` value`. It compiles, and then it dies when
//! you run it.
//!
//!     cargo run -p p1-06-03-result-and-question-mark --example 05-unwrap-panics --features broken

fn safe_divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("cannot divide by zero".to_string())
    } else {
        Ok(a / b)
    }
}

fn main() {
    // `.unwrap()` on `Some`/`Ok` hands back the value. On `None`/`Err` it
    // panics — and for `Err`, the panic message includes whatever you put
    // inside it.
    let value = safe_divide(10.0, 0.0).unwrap();
    println!("{value}");
}
