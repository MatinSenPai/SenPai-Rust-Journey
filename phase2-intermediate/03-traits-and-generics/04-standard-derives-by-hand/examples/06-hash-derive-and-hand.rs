//! `Hash` is what lets a type sit inside a `HashMap`/`HashSet` key.
//! `#[derive(Hash)]` feeds every field into the hasher in order; a
//! hand-written `Hash` can choose to hash fewer fields — as long as it stays
//! consistent with `Eq` (the next example shows what happens when it does
//! not).
//!
//!     cargo run -p p2-03-04-standard-derives-by-hand --example 06-hash-derive-and-hand

use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq, Eq, Hash)]
struct AnimeDerived {
    title: String,
    episodes: u32,
}

#[derive(Debug)]
struct AnimeManual {
    title: String,
    episodes: u32,
}
impl PartialEq for AnimeManual {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title && self.episodes == other.episodes
    }
}
impl Eq for AnimeManual {}
impl Hash for AnimeManual {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.episodes.hash(state);
    }
}

fn main() {
    let mut derived_set: HashSet<AnimeDerived> = HashSet::new();
    derived_set.insert(AnimeDerived {
        title: "Frieren".into(),
        episodes: 28,
    });
    let dup = derived_set.insert(AnimeDerived {
        title: "Frieren".into(),
        episodes: 28,
    });
    println!("derived Hash — inserting the same value again returns: {dup}");
    println!("set size: {}", derived_set.len());

    let mut manual_set: HashSet<AnimeManual> = HashSet::new();
    manual_set.insert(AnimeManual {
        title: "Frieren".into(),
        episodes: 28,
    });
    let dup2 = manual_set.insert(AnimeManual {
        title: "Frieren".into(),
        episodes: 28,
    });
    println!("hand-written Hash — inserting the same value again returns: {dup2}");
    println!("set size: {}", manual_set.len());
}
