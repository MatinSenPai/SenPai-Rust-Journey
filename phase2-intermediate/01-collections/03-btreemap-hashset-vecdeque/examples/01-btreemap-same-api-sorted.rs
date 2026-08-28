//! `BTreeMap` takes almost the same calls as `HashMap` — `.insert()`, `.get()`,
//! even `.entry()` — but keeps its keys sorted, so iterating it always comes
//! out in ascending key order. No `.sort()` step, ever.
//!
//!     cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 01-btreemap-same-api-sorted

use std::collections::{BTreeMap, HashMap};

fn main() {
    let mut counts: HashMap<String, u32> = HashMap::new();
    counts.insert("naruto".to_string(), 4);
    counts.insert("bleach".to_string(), 2);
    counts.insert("frieren".to_string(), 9);

    // Same three inserts, into a BTreeMap instead. The method names do not
    // change; what changes is what you get back when you walk the result.
    let mut sorted: BTreeMap<String, u32> = BTreeMap::new();
    for (title, count) in counts {
        sorted.insert(title, count);
    }

    println!("BTreeMap, walked in insertion-independent order:");
    for (title, count) in &sorted {
        println!("  {title}: {count}");
    }

    // The entry API you already know from HashMap works identically here —
    // same `.or_insert()`, same one-lookup increment.
    *sorted.entry("bocchi".to_string()).or_insert(0) += 1;
    *sorted.entry("bleach".to_string()).or_insert(0) += 1;

    println!();
    println!("after two entry() increments:");
    for (title, count) in &sorted {
        println!("  {title}: {count}");
    }
}
