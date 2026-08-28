//! DELIBERATELY BROKEN — expected: E0106
//!
//! Two `&str` parameters, one `&str` return value. Elision rule 1 gives `x`
//! and `y` two separate, unrelated lifetimes, and rule 2 only fires when
//! there is exactly one input lifetime — so nothing tells the compiler
//! which parameter (or neither) the return value actually borrows from.
//!
//!     cargo run -p p2-04-01-lifetime-basics-and-elision --example 05-two-inputs-one-output --features broken

fn longest(x: &str, y: &str) -> &str {
    if x.len() >= y.len() {
        x
    } else {
        y
    }
}

fn main() {
    let a = String::from("senpai");
    let b = String::from("kouhai teaches back");
    println!("longest: {}", longest(&a, &b));
}
