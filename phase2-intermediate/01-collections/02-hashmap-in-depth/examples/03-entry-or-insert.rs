//! The `entry` API turns "look up, then decide" into a single lookup instead
//! of two. `.or_insert(default)` fills a missing key eagerly; `.or_insert_with(f)`
//! only calls `f` when the key was actually missing.
//!
//!     cargo run -p p2-01-02-hashmap-in-depth --example 03-entry-or-insert

use std::collections::HashMap;

fn main() {
    let seen = ["fish", "cat", "fish", "dog", "fish", "cat"];
    let mut counts: HashMap<&str, u32> = HashMap::new();

    for word in seen {
        // `.entry(word)` is one lookup: "the slot for this key, whether it
        // exists yet or not." `.or_insert(0)` fills that slot with 0 the
        // first time, and either way hands back a `&mut u32` aimed straight
        // at the count, which `*... += 1` bumps in place.
        *counts.entry(word).or_insert(0) += 1;
    }
    println!("fish: {}", counts["fish"]);
    println!("cat:  {}", counts["cat"]);
    println!("dog:  {}", counts["dog"]);

    // `.or_insert_with(f)` is the lazy twin: `f` runs only if the key was
    // missing. Reach for it when the default is real work — like allocating
    // a `Vec` — because a plain `.or_insert(Vec::new())` would build (and
    // immediately throw away) a `Vec` on *every* call, hit or miss.
    let mut first_letters: HashMap<char, Vec<&str>> = HashMap::new();
    for word in seen {
        let letter = word.chars().next().unwrap();
        first_letters
            .entry(letter)
            .or_insert_with(Vec::new)
            .push(word);
    }
    println!("starting with 'f': {:?}", first_letters[&'f']);
    println!("starting with 'c': {:?}", first_letters[&'c']);
    println!("starting with 'd': {:?}", first_letters[&'d']);
}
