//! `Vec` is O(1) at the back and O(n) at the front (`insert(0, _)` and
//! `remove(0)` both have to shift every other element over). `VecDeque` is
//! O(1) at *both* ends — it is a ring buffer, not a plain contiguous array.
//!
//!     cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 04-vecdeque-both-ends

use std::collections::VecDeque;

fn main() {
    let mut queue: VecDeque<&str> = VecDeque::new();
    queue.push_back("Frieren"); // joins at the back, the normal way — O(1)
    queue.push_back("Bocchi");
    queue.push_front("Bleach"); // jumps to the front — O(1), no shifting
    println!("queue: {queue:?}");

    let next = queue.pop_front();
    println!("pop_front(): {next:?}");
    println!("queue now: {queue:?}");

    queue.push_front("Naruto");
    println!("queue now: {queue:?}");
    println!("len: {}", queue.len());
}
