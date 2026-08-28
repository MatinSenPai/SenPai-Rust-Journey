//! Axis 2 — lookup by position. `Vec` and `VecDeque` have one; a `HashMap`
//! does not, because a key is not a position.
//!
//!     cargo run -p p2-01-04-choosing-a-collection --example 02-lookup-by-index

use std::collections::VecDeque;

fn main() {
    let lineup: Vec<&str> = vec!["Frieren", "Bocchi", "Naruto"];
    println!("lineup.first():  {:?}", lineup.first());
    println!("lineup.get(1):   {:?}", lineup.get(1));

    println!();

    let mut queue: VecDeque<&str> = VecDeque::new();
    queue.push_back("Frieren");
    queue.push_back("Bocchi");
    println!("queue[0]:        {}", queue[0]);
    println!("queue.get(1):    {:?}", queue.get(1));
}
