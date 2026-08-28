//! Axis 4 — where you push and pop. `VecDeque` is cheap at both ends.
//! `BinaryHeap` has no "ends" at all — only "give me the current largest."
//!
//!     cargo run -p p2-01-04-choosing-a-collection --example 04-ends-and-priority

use std::collections::{BinaryHeap, VecDeque};

fn main() {
    let mut recent: VecDeque<&str> = VecDeque::new();
    recent.push_back("ep1 released");
    recent.push_back("ep2 released");
    recent.push_front("server maintenance");
    println!("{recent:?}");
    println!("oldest: {:?}", recent.pop_front());
    println!("{recent:?}");

    println!();

    let mut pending: BinaryHeap<u32> = BinaryHeap::new();
    pending.push(240);
    pending.push(90);
    pending.push(500);
    println!("next up: {:?}", pending.pop());
    println!("next up: {:?}", pending.pop());
    println!("still waiting: {:?}", pending.peek());
}
