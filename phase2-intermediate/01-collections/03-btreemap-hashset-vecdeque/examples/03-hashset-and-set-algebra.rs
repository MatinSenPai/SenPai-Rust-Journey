//! `HashSet<T>` is a `HashMap<T, ()>` in spirit: membership only, no value
//! attached. It gives you the same set algebra Python's `set` does. When you
//! need that same membership test but sorted, `BTreeSet<T>` is the
//! `BTreeMap`-shaped version — same trade-off as the map pair.
//!
//!     cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 03-hashset-and-set-algebra

use std::collections::{BTreeSet, HashSet};

fn main() {
    let mut watched: HashSet<&str> = HashSet::new();
    watched.insert("frieren");
    watched.insert("bocchi");
    println!("watched frieren? {}", watched.contains("frieren"));
    println!("watched naruto?  {}", watched.contains("naruto"));

    let action: HashSet<&str> = HashSet::from(["frieren", "bleach", "naruto"]);
    let comedy: HashSet<&str> = HashSet::from(["bocchi", "bleach"]);

    // Every set-algebra method hands back an *iterator* of borrowed
    // references, not a new HashSet — so a plain `for` loop is all you need
    // to use the result; no `.collect()` required. HashSet has no ordering
    // promise (same rule as HashMap from 2.1.2), so each result below is
    // gathered into a Vec and sorted purely so it prints in a fixed order —
    // the set itself stays unordered.
    println!();
    let mut both: Vec<&&str> = Vec::new();
    for title in action.intersection(&comedy) {
        both.push(title);
    }
    both.sort();
    println!("in both action and comedy: {both:?}");

    let mut action_only: Vec<&&str> = Vec::new();
    for title in action.difference(&comedy) {
        action_only.push(title);
    }
    action_only.sort();
    println!("in action but not comedy: {action_only:?}");

    let mut either_only: Vec<&&str> = Vec::new();
    for title in action.symmetric_difference(&comedy) {
        either_only.push(title);
    }
    either_only.sort();
    println!("in exactly one of the two: {either_only:?}");

    // BTreeSet: identical membership API, sorted iteration instead of
    // unordered — the same reason to reach for it as BTreeMap.
    let sorted_genres: BTreeSet<&str> = BTreeSet::from(["isekai", "action", "comedy", "drama"]);
    println!();
    println!("BTreeSet, always alphabetical:");
    for genre in &sorted_genres {
        println!("  {genre}");
    }
}
