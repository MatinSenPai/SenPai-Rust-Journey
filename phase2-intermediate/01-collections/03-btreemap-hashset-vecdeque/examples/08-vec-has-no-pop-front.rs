//! DELIBERATELY BROKEN — expected: E0599.
//!
//! `Vec` only ever grew a front-removal method in your head, not in the
//! standard library — `.remove(0)` exists, but it's O(n), and `.pop_front()`
//! was never one of `Vec`'s methods to begin with. This is the exact wall
//! that `VecDeque` exists to remove.
//!
//!     cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 08-vec-has-no-pop-front --features broken

fn main() {
    let mut queue: Vec<&str> = vec!["Frieren", "Bocchi"];
    let next = queue.pop_front();
    println!("{next:?}");
}
