//! DELIBERATELY BROKEN — expected: E0283
//!
//! `.collect()` can build many different types from this same iterator.
//! With no turbofish and no type annotation on `evens`, the compiler has no
//! way to pick one.
//!
//!     cargo run -p p2-02-03-consuming-and-collecting --example 07-collect-ambiguous-type --features broken

fn main() {
    let evens = (1..10).filter(|n| n % 2 == 0).collect();
    println!("{evens:?}");
}
