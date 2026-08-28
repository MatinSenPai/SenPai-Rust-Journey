//! DELIBERATELY BROKEN — expected: E0599.
//!
//! A `BinaryHeap` only ever gives up its current largest value, through
//! `pop()`. It was never built to find and remove one arbitrary value from
//! the middle, the way `HashSet` can — so there is no `.remove()`
//! to reach for here at all.
//!
//!     cargo run -p p2-01-04-choosing-a-collection --example 08-binaryheap-has-no-remove --features broken

use std::collections::BinaryHeap;

fn main() {
    let mut scores: BinaryHeap<(u32, &str)> = BinaryHeap::new();
    scores.push((10, "Yui"));
    scores.push((25, "Matin"));
    scores.remove(&(10, "Yui"));
    println!("{:?}", scores.peek());
}
