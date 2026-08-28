//! DELIBERATELY BROKEN — expected: E0599.
//!
//! Wanting sorted output is not the same as having it. `.sort()` belongs to
//! `Vec`/slices — a `HashMap` was never ordered to begin with, so there is
//! nothing on it for `.sort()` to do, and the method does not exist.
//!
//!     cargo run -p p2-01-04-choosing-a-collection --example 07-hashmap-has-no-sort --features broken

use std::collections::HashMap;

fn main() {
    let mut counts: HashMap<&str, u32> = HashMap::new();
    counts.insert("Frieren", 12);
    counts.insert("Bocchi", 7);
    counts.sort();
    println!("{counts:?}");
}
