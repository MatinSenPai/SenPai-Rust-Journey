//! Axis 3 — what order does iteration come out in? `BTreeMap` always sorts by
//! key. `BinaryHeap` guarantees only that the *next pop* is the largest value
//! left — plain iteration over it is not sorted at all, which surprises
//! almost everyone the first time they see it.
//!
//!     cargo run -p p2-01-04-choosing-a-collection --example 03-iteration-order

use std::collections::{BTreeMap, BinaryHeap};

fn main() {
    let mut by_day: BTreeMap<u32, u32> = BTreeMap::new();
    by_day.insert(3, 40);
    by_day.insert(1, 12);
    by_day.insert(2, 25);
    for (day, count) in &by_day {
        println!("day {day}: {count}");
    }

    println!();

    let ratings: BinaryHeap<u32> = BinaryHeap::from([3, 1, 4, 1, 5, 9, 2, 6]);
    println!("peek:        {:?}", ratings.peek());
    let mut still_sorted = true;
    let mut previous = u32::MAX;
    for value in &ratings {
        still_sorted &= *value <= previous;
        previous = *value;
    }
    println!("iter() order was sorted: {still_sorted}");
}
