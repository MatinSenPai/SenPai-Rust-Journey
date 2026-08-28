//! `HashMap` cannot answer "everything between X and Y" without walking and
//! checking every single entry — there is no order to exploit. A `BTreeMap`
//! answers it directly with `.range()`, only visiting the entries that
//! qualify.
//!
//!     cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 02-btreemap-range-queries

use std::collections::BTreeMap;

fn main() {
    let mut releases: BTreeMap<u32, &str> = BTreeMap::new();
    releases.insert(2013, "attack-on-titan");
    releases.insert(2023, "frieren");
    releases.insert(2019, "demon-slayer");
    releases.insert(2022, "bocchi");
    releases.insert(2001, "spirited-away");

    println!("released 2019..=2023:");
    for (year, title) in releases.range(2019..=2023) {
        println!("  {year}: {title}");
    }

    // Half-open, exclusive-end works the same as a slice range: `..2019`
    // means "strictly before 2019".
    println!();
    println!("released before 2019:");
    for (year, title) in releases.range(..2019) {
        println!("  {year}: {title}");
    }
}
