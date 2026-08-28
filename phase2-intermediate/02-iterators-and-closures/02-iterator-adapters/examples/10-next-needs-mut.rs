//! DELIBERATELY BROKEN — expected: E0596
//!
//! `next(&mut self)` needs a mutable iterator to call it on — each call has
//! to move the iterator's internal position forward. `it` here is never
//! declared `mut`.
//!
//!     cargo run -p p2-02-02-iterator-adapters --example 10-next-needs-mut --features broken

fn main() {
    let scores = vec![10, 20, 30];
    let it = scores.iter();
    println!("{:?}", it.next());
}
