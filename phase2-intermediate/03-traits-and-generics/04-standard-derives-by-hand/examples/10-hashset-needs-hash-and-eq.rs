//! DELIBERATELY BROKEN — expected: E0599
//!
//! `Anime` here only has `#[derive(Debug)]` — no `Hash`, no `Eq`. A
//! `HashSet<Anime>` needs both to place a value and later recognize it
//! again. Same requirement as a `HashMap` key; this time on a `HashSet`.
//!
//!     cargo run -p p2-03-04-standard-derives-by-hand --example 10-hashset-needs-hash-and-eq --features broken

use std::collections::HashSet;

#[derive(Debug)]
struct Anime {
    title: String,
    episodes: u32,
}

fn main() {
    let mut seen: HashSet<Anime> = HashSet::new();
    seen.insert(Anime {
        title: "Frieren".to_string(),
        episodes: 28,
    });
    println!("{seen:?}");
}
