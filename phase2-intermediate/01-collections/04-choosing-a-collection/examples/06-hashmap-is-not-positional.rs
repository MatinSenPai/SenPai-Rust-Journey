//! DELIBERATELY BROKEN — expected: E0308.
//!
//! `counts[0]` reads like "give me the first entry," the `Vec` habit. A
//! `HashMap<String, u32>` has no first entry — indexing it takes a reference
//! to an actual key, not a position.
//!
//!     cargo run -p p2-01-04-choosing-a-collection --example 06-hashmap-is-not-positional --features broken

use std::collections::HashMap;

fn main() {
    let mut counts: HashMap<String, u32> = HashMap::new();
    counts.insert("Frieren".to_string(), 12);
    let first = counts[0];
    println!("{first}");
}
