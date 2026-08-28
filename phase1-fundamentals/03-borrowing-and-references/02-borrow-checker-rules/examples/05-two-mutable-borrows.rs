//! DELIBERATELY BROKEN — expected: E0499
//! Swapping the ends of a Vec, written the way it reads in English.
//!
//!     cargo run -p p1-03-02-borrow-checker-rules --example 05-two-mutable-borrows --features broken

fn main() {
    let mut scores = vec![10, 20, 30];

    // Two exclusive borrows of the same Vec, both alive at the same time.
    // "Exclusive" is the whole point: the second one cannot exist.
    let front = &mut scores[0];
    let back = &mut scores[2];

    let keep = *front;
    *front = *back;
    *back = keep;

    println!("{scores:?}");
}
