//! DELIBERATELY BROKEN — expected: E0502.
//!
//!     cargo run -p p1-03-04-slices --example 07-two-views-one-mutable --features broken
//!
//! A slice is a borrow, so the rule from 1.3.2 applies to it unchanged —
//! even when the two windows do not overlap.

fn main() {
    let mut readings = vec![10, 20, 30, 40, 50];

    let front = &readings[..2];
    let back = &mut readings[3..];

    back[0] = 0;
    println!("front: {front:?}");
    println!("back:  {back:?}");
}
