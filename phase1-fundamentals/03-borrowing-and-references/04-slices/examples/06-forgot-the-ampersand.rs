//! DELIBERATELY BROKEN — expected: E0277.
//!
//!     cargo run -p p1-03-04-slices --example 06-forgot-the-ampersand --features broken
//!
//! The `&` in `&v[1..4]` is not decoration. Leaving it off asks for the
//! run of elements itself, which is not a thing a binding can hold.

fn main() {
    let readings = vec![10, 20, 30, 40, 50];

    let middle = readings[1..4];

    println!("middle: {middle:?}");
}
