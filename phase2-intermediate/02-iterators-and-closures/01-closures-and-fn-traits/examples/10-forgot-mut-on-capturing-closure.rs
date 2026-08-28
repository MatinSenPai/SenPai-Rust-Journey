//! DELIBERATELY BROKEN -- expected: E0596
//!
//! `increment`'s body mutates `count`, so calling `increment` requires an
//! exclusive borrow of `count` -- which means the *binding* `increment`
//! itself has to be declared `mut`, the same way any other value you call
//! a mutating method through does.
//!
//!     cargo run -p p2-02-01-closures-and-fn-traits --example 10-forgot-mut-on-capturing-closure --features broken

fn main() {
    let mut count = 0;
    let increment = || {
        count += 1;
    };
    increment();
    println!("{count}");
}
