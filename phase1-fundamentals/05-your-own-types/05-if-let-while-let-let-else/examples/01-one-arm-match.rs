//! `if let` is a `match` with one arm you actually care about.
//!
//!     cargo run -p p1-05-05-if-let-while-let-let-else --example 01-one-arm-match

fn main() {
    let rating: Option<u8> = Some(9);

    // The full match. Read the second arm out loud: "and if there is no
    // rating, do nothing." That is a whole line spent saying nothing.
    match rating {
        Some(score) => println!("match:      rated {score}/10"),
        None => {}
    }

    // The same decision, with the arm that does nothing deleted.
    if let Some(score) = rating {
        println!("if let:     rated {score}/10");
    }

    // The `else` is that arm brought back, for when it has work to do.
    let missing: Option<u8> = None;
    if let Some(score) = missing {
        println!("with else:  rated {score}/10");
    } else {
        println!("with else:  not rated yet");
    }

    // And because `if let` is an `if`, it is an expression like any other.
    let shown = if let Some(score) = rating { score } else { 0 };
    println!("as a value: {shown}");

    // The binding only exists inside the block. This is the line that catches
    // people, and it is `07-binding-escaped.rs`.
    println!("outside:    `score` does not exist out here");
}
