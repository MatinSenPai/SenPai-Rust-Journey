//! DELIBERATELY BROKEN — expected: E0599.
//!
//! `BinaryHeap<T>` has to compare elements to know which one is biggest, so
//! `T` must implement `Ord` — a *total* order, where every pair of values
//! has a defined answer to "which is bigger?". `f64` cannot promise that
//! (`NaN` is comparable to nothing, not even itself), so it only implements
//! the weaker `PartialOrd`, and `BinaryHeap<f64>` never gets off the ground.
//!
//!     cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 09-binaryheap-needs-ord --features broken

use std::collections::BinaryHeap;

fn main() {
    let mut ratings: BinaryHeap<f64> = BinaryHeap::new();
    ratings.push(8.5);
    println!("{:?}", ratings.peek());
}
