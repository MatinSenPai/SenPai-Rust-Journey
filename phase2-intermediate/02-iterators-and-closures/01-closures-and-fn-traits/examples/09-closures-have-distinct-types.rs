//! DELIBERATELY BROKEN -- expected: E0308
//!
//! `add_one` and `add_two` have the exact same signature -- both are
//! `Fn(i32) -> i32` -- but each closure literal still gets its own,
//! distinct, compiler-generated type. `which` is typed as "whatever
//! `add_one` is", and `add_two` is a different type, even though nothing
//! about the two closures looks different from the outside.
//!
//!     cargo run -p p2-02-01-closures-and-fn-traits --example 09-closures-have-distinct-types --features broken

fn main() {
    let add_one = |x: i32| x + 1;
    let add_two = |x: i32| x + 2;
    let mut which = add_one;
    which = add_two;
    println!("{}", which(5));
}
