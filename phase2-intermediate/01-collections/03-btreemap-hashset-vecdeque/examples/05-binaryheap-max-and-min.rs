//! `BinaryHeap::pop()` always returns the current maximum, in O(log n) —
//! never insertion order, never sorted order. `Reverse<T>` flips "biggest
//! wins" into "smallest wins", turning the same max-heap into a min-heap
//! with no other code changes.
//!
//!     cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 05-binaryheap-max-and-min

use std::cmp::Reverse;
use std::collections::BinaryHeap;

fn main() {
    let mut ratings: BinaryHeap<u32> = BinaryHeap::new();
    ratings.push(7);
    ratings.push(2);
    ratings.push(9);
    ratings.push(4);

    // peek() looks at the max without removing it.
    println!("peek: {:?}", ratings.peek());

    let mut pop_order: Vec<u32> = Vec::new();
    while let Some(top) = ratings.pop() {
        pop_order.push(top);
    }
    println!("pop order: {pop_order:?}");

    // Tuples order lexicographically — compare the first element, and only
    // look at the second to break a tie. That is enough to build a priority
    // queue of (priority, title) pairs with no custom code at all.
    println!();
    let mut up_next: BinaryHeap<(u32, &str)> = BinaryHeap::new();
    up_next.push((2, "Bocchi"));
    up_next.push((5, "Frieren"));
    up_next.push((1, "Bleach"));
    while let Some((priority, title)) = up_next.pop() {
        println!("priority {priority}: {title}");
    }

    // Same heap, wrapped in Reverse: now the *smallest* priority pops first.
    println!();
    let mut low_first: BinaryHeap<Reverse<(u32, &str)>> = BinaryHeap::new();
    low_first.push(Reverse((2, "Bocchi")));
    low_first.push(Reverse((5, "Frieren")));
    low_first.push(Reverse((1, "Bleach")));
    while let Some(Reverse((priority, title))) = low_first.pop() {
        println!("priority {priority}: {title}");
    }
}
