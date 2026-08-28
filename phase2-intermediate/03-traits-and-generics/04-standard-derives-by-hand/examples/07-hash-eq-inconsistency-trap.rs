//! This compiles. It runs without panicking. Its answer is still wrong.
//!
//! `PartialEq` below only looks at `title` — two entries for the same show
//! count as equal no matter how many episodes are logged. `Hash` was written
//! sloppily: it also feeds in `episodes_watched`. The two impls now
//! disagree, and `HashSet` trusts `Hash` first.
//!
//!     cargo run -p p2-03-04-standard-derives-by-hand --example 07-hash-eq-inconsistency-trap

use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
struct Anime {
    title: String,
    episodes_watched: u32,
}

impl PartialEq for Anime {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
    }
}
impl Eq for Anime {}

impl Hash for Anime {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.episodes_watched.hash(state); // bug: eq() never looks at this
    }
}

fn main() {
    let mut in_progress: HashSet<Anime> = HashSet::new();
    in_progress.insert(Anime {
        title: "Frieren".to_string(),
        episodes_watched: 5,
    });

    let same_show_more_progress = Anime {
        title: "Frieren".to_string(),
        episodes_watched: 12,
    };

    let first = in_progress.iter().next().unwrap();
    println!(
        "== says these are the same show: {}",
        first == &same_show_more_progress
    );
    println!(
        ".contains() finds it:            {}",
        in_progress.contains(&same_show_more_progress)
    );

    in_progress.insert(same_show_more_progress);
    println!(
        "set length after the \"duplicate\" insert: {}",
        in_progress.len()
    );

    // HashSet iteration order is not guaranteed (2.1.2 covered why), so sort
    // before printing to keep this example's output reproducible.
    let mut entries: Vec<&Anime> = in_progress.iter().collect();
    entries.sort_by_key(|a| a.episodes_watched);
    for a in entries {
        println!("  {a:?}");
    }
}
