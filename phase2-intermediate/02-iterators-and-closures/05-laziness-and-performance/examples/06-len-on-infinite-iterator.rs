//! DELIBERATELY BROKEN — expected: E0599.
//!
//! `.len()` only exists on `ExactSizeIterator` — an iterator that knows its
//! exact remaining length up front. `std::iter::repeat(...)` never runs
//! out, so it has no length to report, and never implements that trait.
//!
//!     cargo run -p p2-02-05-laziness-and-performance --example 06-len-on-infinite-iterator --features broken

fn main() {
    let forever = std::iter::repeat(1);
    println!("{}", forever.len());
}
