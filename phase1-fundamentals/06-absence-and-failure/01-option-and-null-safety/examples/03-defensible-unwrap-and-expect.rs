//! `.unwrap()` and `.expect("...")` both take the value out of a `Some` and
//! panic on `None`. Neither is "wrong" — they're the right tool exactly when
//! *you* can see something the type checker can't.
//!
//!     cargo run -p p1-06-01-option-and-null-safety --example 03-defensible-unwrap-and-expect

fn main() {
    // Defensible #1: you built the `Some` two lines above, with your own
    // hands. There is nothing to check — you already know.
    let picked: Option<i32> = Some(7);
    let value = picked.unwrap();
    println!("unwrap on a value you just wrote: {value}");

    // Defensible #2: `.last()` returns `Option<&T>` because a Vec *might* be
    // empty — but this one just had three items pushed onto it, on the line
    // right above. `.expect("...")` takes that proof and writes it down, so
    // the next reader (including future you) doesn't have to reconstruct it.
    let mut scores = Vec::new();
    scores.push(10);
    scores.push(20);
    scores.push(30);
    let last = scores.last().expect("scores just had three items pushed");
    println!("expect on a Vec you just filled: {last}");

    // The message is not decoration. If this invariant ever breaks — someone
    // adds an early return above the pushes, say — the panic names exactly
    // which assumption stopped holding, at the moment it stopped holding.
    println!();
    println!("the rule: unwrap/expect are for a None the *type* allows");
    println!("but the *code around it* rules out. write down why in expect().");
}
