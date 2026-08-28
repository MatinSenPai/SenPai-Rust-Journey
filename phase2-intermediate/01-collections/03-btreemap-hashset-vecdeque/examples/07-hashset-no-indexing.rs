//! DELIBERATELY BROKEN — expected: E0608.
//!
//! A `HashSet` has no concept of "position zero" — it isn't a sequence, so
//! there is nothing for `[0]` to mean. Indexing simply does not exist for
//! this type.
//!
//!     cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 07-hashset-no-indexing --features broken

use std::collections::HashSet;

fn main() {
    let genres: HashSet<&str> = HashSet::from(["action", "comedy", "isekai"]);
    let first = genres[0];
    println!("{first}");
}
