//! DELIBERATELY BROKEN — expected: E0502
//! Run `cargo run -p p1-03-03-borrow-scopes-and-nll \
//!   --example 06-borrowed-across-a-loop --features broken`.
//!
//! Read where `immutable borrow later used here` points. It is the line
//! ABOVE the one the error is reported on.

fn main() {
    let mut totals = vec![10, 20, 30];
    let view = &totals;

    for _ in 0..2 {
        println!("still {} items", view.len());
        totals.push(40);
    }
}
