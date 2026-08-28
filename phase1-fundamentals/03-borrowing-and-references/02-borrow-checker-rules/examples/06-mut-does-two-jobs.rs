//! DELIBERATELY BROKEN — expected: E0384 and E0596
//! One missing keyword, two different errors — because `mut` grants two
//! different permissions.
//!
//!     cargo run -p p1-03-02-borrow-checker-rules --example 06-mut-does-two-jobs --features broken

fn main() {
    // Job one: `mut` is what lets you assign to the binding again.
    let total = 0;
    total = total + 10;

    // Job two: `mut` is what lets anybody take a `&mut` to it.
    let scores = vec![10, 20];
    let writer = &mut scores;
    writer.push(30);

    println!("{total} {scores:?}");
}
