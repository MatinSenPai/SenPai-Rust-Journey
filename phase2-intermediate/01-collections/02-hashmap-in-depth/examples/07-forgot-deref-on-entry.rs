//! DELIBERATELY BROKEN — expected: E0368.
//!
//! `.or_insert(0)` hands back a `&mut u32` — a place, not a number. `+=`
//! needs a real `u32` on its left side, so the reference has to be followed
//! first, with `*`.
//!
//!     cargo run -p p2-01-02-hashmap-in-depth --example 07-forgot-deref-on-entry --features broken

use std::collections::HashMap;

fn main() {
    let seen = ["fish", "cat", "fish"];
    let mut counts: HashMap<&str, u32> = HashMap::new();

    for word in seen {
        counts.entry(word).or_insert(0) += 1;
    }

    println!("fish: {}", counts["fish"]);
}
