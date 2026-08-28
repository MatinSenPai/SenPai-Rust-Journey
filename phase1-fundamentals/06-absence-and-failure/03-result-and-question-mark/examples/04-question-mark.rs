//! The `match` that `?` replaces, then `?` itself — and `main` returning a
//! `Result` too.
//!
//!     cargo run -p p1-06-03-result-and-question-mark --example 04-question-mark

fn safe_divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("cannot divide by zero".to_string())
    } else {
        Ok(a / b)
    }
}

// Written by hand: on `Err`, return that same `Err` right now.
fn chained_by_hand(a: f64, b: f64, c: f64) -> Result<f64, String> {
    let step1 = match safe_divide(a, b) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };
    let step2 = match safe_divide(step1, c) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };
    Ok(step2)
}

// The exact same function, written with `?`.
fn chained_with_question_mark(a: f64, b: f64, c: f64) -> Result<f64, String> {
    let step1 = safe_divide(a, b)?;
    let step2 = safe_divide(step1, c)?;
    Ok(step2)
}

fn main() -> Result<(), String> {
    println!("by hand,  ok: {:?}", chained_by_hand(100.0, 2.0, 5.0));
    println!("by hand,  err: {:?}", chained_by_hand(100.0, 0.0, 5.0));
    println!(
        "with `?`, ok: {:?}",
        chained_with_question_mark(100.0, 2.0, 5.0)
    );
    println!(
        "with `?`, err: {:?}",
        chained_with_question_mark(100.0, 2.0, 0.0)
    );

    // `main` can return a `Result` too, and `?` works inside it exactly as
    // it does anywhere else. This call fails on purpose — watch what
    // happens to the process, not just to a printed value.
    let result = chained_with_question_mark(100.0, 0.0, 5.0)?;
    println!("unreachable: {result}");
    Ok(())
}
