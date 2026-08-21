//! DELIBERATELY BROKEN — expected: E0308.
//!
//!     cargo run -p p1-01-05-control-flow --example 04-truthiness --features broken

fn main() {
    let remaining = 0;

    // Works in Python and in C. Rust has no truthiness: a condition is a
    // `bool` or it is an error.
    if remaining {
        println!("still some left");
    }
}
