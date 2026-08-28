//! `HashMap<K, V>` implements `FromIterator<(K, V)>` — hand `.collect()` an
//! iterator of key/value tuples and it builds the whole map in one call, no
//! `HashMap::new()` plus a loop of `.insert()`s required.
//!
//!     cargo run -p p2-02-03-consuming-and-collecting --example 03-collect-into-hashmap

use std::collections::HashMap;

fn main() {
    let entries = [("Frieren", 28), ("Bocchi the Rock", 12), ("K-On!", 13)];

    let episodes: HashMap<&str, u32> = entries.into_iter().collect();
    println!("Frieren episodes:  {:?}", episodes.get("Frieren"));
    println!("total shows:       {}", episodes.len());

    // The adapter stage still runs first — this builds the (title, title
    // length) pairs, then `.collect()` turns that iterator into a map.
    let name_lengths: HashMap<&str, usize> = entries
        .iter()
        .map(|(title, _)| (*title, title.len()))
        .collect();
    println!("\"K-On!\" title length: {:?}", name_lengths.get("K-On!"));

    // A repeated key behaves exactly like calling `.insert()` that many
    // times in order: the last value written for that key wins.
    let last_write_wins: HashMap<&str, u32> = [("Frieren", 1), ("Frieren", 2), ("Frieren", 3)]
        .into_iter()
        .collect();
    println!(
        "last value for a repeated key: {:?}",
        last_write_wins.get("Frieren")
    );
}
