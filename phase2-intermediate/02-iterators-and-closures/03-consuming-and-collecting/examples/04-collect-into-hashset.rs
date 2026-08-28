//! `HashSet<T>` implements `FromIterator<T>` the same way `Vec<T>` does —
//! the only difference is what the target type does with a repeat: a `Vec`
//! keeps it, a `HashSet` quietly drops it.
//!
//!     cargo run -p p2-02-03-consuming-and-collecting --example 04-collect-into-hashset

use std::collections::HashSet;

fn main() {
    let tags = [
        "comedy",
        "drama",
        "comedy",
        "slice of life",
        "drama",
        "comedy",
    ];
    println!("tags seen (with repeats): {}", tags.len());

    let unique: HashSet<&str> = tags.into_iter().collect();
    println!("unique tags:               {}", unique.len());
    println!("contains \"drama\":          {}", unique.contains("drama"));
    println!("contains \"action\":         {}", unique.contains("action"));

    // Same trick, now on the far end of an adapter chain: every distinct
    // word length that shows up across a list of titles.
    let titles = ["Frieren", "K-On!", "Mob", "Nana"];
    let distinct_lengths: HashSet<usize> = titles.iter().map(|t| t.len()).collect();
    println!("distinct title lengths:    {}", distinct_lengths.len());
}
