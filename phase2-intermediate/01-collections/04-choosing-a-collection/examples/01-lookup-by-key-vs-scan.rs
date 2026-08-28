//! Axis 1 — looking something up by a key versus walking a list by hand.
//!
//! Same question, same data, two collections. `watch_count_hashmap` does one
//! hash lookup. `watch_count_scan` cannot do that — a `Vec` has no idea what
//! a "key" is — so it walks every entry until it finds a match or runs out.
//!
//!     cargo run -p p2-01-04-choosing-a-collection --example 01-lookup-by-key-vs-scan

use std::collections::HashMap;

fn watch_count_hashmap(counts: &HashMap<&str, u32>, title: &str) -> Option<u32> {
    counts.get(title).copied()
}

fn watch_count_scan(counts: &[(&str, u32)], title: &str) -> Option<u32> {
    for (show, count) in counts {
        if *show == title {
            return Some(*count);
        }
    }
    None
}

fn main() {
    let by_map: HashMap<&str, u32> = HashMap::from([("Frieren", 12), ("Bocchi", 7)]);
    let by_list: Vec<(&str, u32)> = vec![("Frieren", 12), ("Bocchi", 7)];

    println!(
        "HashMap.get(\"Frieren\"): {:?}",
        watch_count_hashmap(&by_map, "Frieren")
    );
    println!(
        "HashMap.get(\"Naruto\"):  {:?}",
        watch_count_hashmap(&by_map, "Naruto")
    );
    println!();
    println!(
        "Vec scan(\"Frieren\"):    {:?}",
        watch_count_scan(&by_list, "Frieren")
    );
    println!(
        "Vec scan(\"Naruto\"):     {:?}",
        watch_count_scan(&by_list, "Naruto")
    );
}
