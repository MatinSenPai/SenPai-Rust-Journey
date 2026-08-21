//! `.parse()` — used since Phase 0, explained here at last.
//!
//!     cargo run -p p1-06-03-result-and-question-mark --example 02-parse

fn main() {
    // The turbofish tells `.parse()` which type to aim for. It has to —
    // `Result<T, E>` is generic, and nothing else pins down `T`.
    let good: Result<u32, _> = "42".parse::<u32>();
    println!("good:     {good:?}");

    // Failure is not `None`. It is `Err`, carrying a real `ParseIntError`
    // that says exactly what went wrong.
    let letters: Result<u32, _> = "abc".parse::<u32>();
    println!("letters:  {letters:?}");

    // `u32` cannot be negative, so parsing a negative number fails the same
    // way as parsing letters would — same error variant, same message.
    let negative: Result<u32, _> = "-5".parse::<u32>();
    println!("negative: {negative:?}");

    let empty: Result<u32, _> = "".parse::<u32>();
    println!("empty:    {empty:?}");

    // A `ParseIntError` implements `Display`, so `.to_string()` turns it
    // into the same words you saw inside the `Err(..)` above.
    if let Err(e) = "abc".parse::<u32>() {
        println!("message:  {e}");
    }

    // `match` handles both arms without ever calling `.unwrap()`.
    match "7".parse::<u32>() {
        Ok(n) => println!("parsed {n}, doubled is {}", n * 2),
        Err(e) => println!("could not parse: {e}"),
    }
}
