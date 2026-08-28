//! DELIBERATELY BROKEN — expected: E0499
//! Run `cargo run -p p1-03-03-borrow-scopes-and-nll \
//!   --example 07-two-mutable-borrows --features broken`.
//!
//! Delete the `first.len()` line and it compiles. Work out why before you do.

fn main() {
    let mut scores = vec![90, 80];

    let first = &mut scores;
    first.push(70);

    let second = &mut scores;
    second.push(60);

    println!("{}", first.len());
    println!("{}", second.len());
}
