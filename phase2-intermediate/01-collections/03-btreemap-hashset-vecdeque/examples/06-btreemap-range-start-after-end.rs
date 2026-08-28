//! DELIBERATELY BROKEN — expected: a run-time panic, "range start is greater
//! than range end in BTreeMap". It compiles cleanly — `.range()` type-checks
//! on any bounds of the right type — and then it dies when you run it.
//!
//! `.range()` takes the bounds in the order you write them, not sorted for
//! you. Swap the two years by accident — "2023 down to 2019" instead of
//! "2019 up to 2023" — and the start bound ends up greater than the end.
//!
//!     cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 06-btreemap-range-start-after-end --features broken

use std::collections::BTreeMap;

fn main() {
    let mut releases: BTreeMap<u32, &str> = BTreeMap::new();
    releases.insert(2019, "demon-slayer");
    releases.insert(2023, "frieren");

    for (year, title) in releases.range(2023..2019) {
        println!("{year}: {title}");
    }
}
