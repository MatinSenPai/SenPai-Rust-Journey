//! One type, several traits, working together: `Debug`/`Clone`/`Default`/
//! `PartialEq`/`Eq`/`Hash` derived (once you know what each expands to,
//! stacking them is not scary), `Ord`/`PartialOrd` hand-written so a
//! `BinaryHeap` prioritizes by `score` instead of by title.
//!
//!     cargo run -p p2-03-04-standard-derives-by-hand --example 08-all-together

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
struct Anime {
    title: String,
    episodes: u32,
    score: u8,
}

impl PartialOrd for Anime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Anime {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

fn main() {
    let mut queue: BinaryHeap<Anime> = BinaryHeap::new();
    queue.push(Anime {
        title: "Frieren".into(),
        episodes: 28,
        score: 96,
    });
    queue.push(Anime {
        title: "Bocchi the Rock!".into(),
        episodes: 12,
        score: 90,
    });
    queue.push(Anime {
        title: "Mushoku Tensei".into(),
        episodes: 23,
        score: 88,
    });

    println!("watch next: {:?}", queue.pop());
    println!("then:       {:?}", queue.pop());

    let mut library: HashSet<Anime> = HashSet::new();
    library.insert(Anime {
        title: "Frieren".into(),
        episodes: 28,
        score: 96,
    });
    let inserted_again = library.insert(Anime {
        title: "Frieren".into(),
        episodes: 28,
        score: 96,
    });
    println!("duplicate insert returned: {inserted_again}");
    println!("library size:              {}", library.len());

    println!("Default::default():        {:?}", Anime::default());
}
