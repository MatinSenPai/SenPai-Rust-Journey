//! DELIBERATELY BROKEN — expected: E0369
//! Run `cargo run -p p1-05-02-tuple-structs-and-newtype --example
//! 05-adding-two-newtypes --features broken` and read the error.
//!
//! A newtype does not inherit the arithmetic of the type it wraps.

#[derive(Debug)]
struct Rial(i64);

fn main() {
    let price = Rial(250_000);
    let fee = Rial(7_500);

    let total = price + fee;
    println!("{total:?}");
}
