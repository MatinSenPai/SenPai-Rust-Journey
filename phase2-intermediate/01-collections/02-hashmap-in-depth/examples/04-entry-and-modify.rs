//! The idiomatic combo: `.and_modify()` runs a closure only when the key
//! already exists; `.or_insert()` supplies the value for when it does not.
//! Together they are the word-count pattern in one expression, and this is
//! the shape you will reach for most.
//!
//!     cargo run -p p2-01-02-hashmap-in-depth --example 04-entry-and-modify

use std::collections::HashMap;

fn word_counts(text: &str) -> HashMap<&str, u32> {
    let mut counts: HashMap<&str, u32> = HashMap::new();
    for word in text.split_whitespace() {
        counts
            .entry(word)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }
    counts
}

fn main() {
    let review = "great show great cast good story great animation good pacing";
    let counts = word_counts(review);

    println!("great: {}", counts["great"]);
    println!("good:  {}", counts["good"]);
    println!("show:  {}", counts["show"]);
    println!("cast:  {}", counts["cast"]);

    // `.and_modify()` alone changes nothing for a key that was never there —
    // there is nothing to modify yet. `.or_insert()` is what actually places
    // a first value, which is why the pattern needs both, in this order.
    let mut fresh: HashMap<&str, u32> = HashMap::new();
    fresh
        .entry("new")
        .and_modify(|count| *count += 1)
        .or_insert(1);
    println!("new:   {}", fresh["new"]);
}
