//! Putting the axes together: "how many distinct genres did we see today?"
//! Only membership matters — not order, not a value attached to each genre,
//! not a position. That is exactly what a `HashSet` is for.
//!
//!     cargo run -p p2-01-04-choosing-a-collection --example 05-the-decision-in-action

use std::collections::HashSet;

fn main() {
    let genres_seen = ["isekai", "comedy", "isekai", "drama", "comedy"];

    // Reach for HashSet: no key/value pair needed, no order needed, just
    // "have we seen this one before?" — asked once per genre, O(1) each time.
    let mut distinct: HashSet<&str> = HashSet::new();
    for genre in genres_seen {
        distinct.insert(genre);
    }

    println!("genres logged today: {}", genres_seen.len());
    println!("distinct genres:     {}", distinct.len());
    println!("is \"drama\" among them? {}", distinct.contains("drama"));
    println!("is \"mecha\" among them? {}", distinct.contains("mecha"));
}
