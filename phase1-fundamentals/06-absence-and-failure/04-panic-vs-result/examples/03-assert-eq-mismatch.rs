//! DELIBERATELY BROKEN — expected: a panic at run time (this one compiles)
//!
//! `assert_eq!` is a guard clause for an invariant, same family as `panic!`.
//! Here the expectation itself is simply wrong — watch what the failure
//! looks like.
//!
//!     cargo run -p p1-06-04-panic-vs-result --example 03-assert-eq-mismatch --features broken

fn double(n: i32) -> i32 {
    n + n
}

fn main() {
    println!("double(2) == {}", double(2));
    assert_eq!(double(2), 4);
    println!("double(5) == {}", double(5));
    assert_eq!(double(5), 10);
    println!("checking double(6)...");
    assert_eq!(double(6), 11);
}
