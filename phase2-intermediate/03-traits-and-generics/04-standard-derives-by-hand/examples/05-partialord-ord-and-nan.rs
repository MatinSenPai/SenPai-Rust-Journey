//! `#[derive(PartialOrd, Ord)]` compares fields in declaration order — title
//! first, then episodes, then score — the same way tuples compare. A
//! hand-written `Ord` can pick a more useful order instead.
//!
//!     cargo run -p p2-03-04-standard-derives-by-hand --example 05-partialord-ord-and-nan

use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct AnimeByFields {
    title: String,
    episodes: u32,
    score: u8,
}

#[derive(Debug)]
struct AnimeByScore {
    title: String,
    episodes: u32,
    score: u8,
}
impl PartialEq for AnimeByScore {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}
impl Eq for AnimeByScore {}
impl PartialOrd for AnimeByScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for AnimeByScore {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

fn main() {
    let mut by_fields = vec![
        AnimeByFields {
            title: "Frieren".into(),
            episodes: 28,
            score: 96,
        },
        AnimeByFields {
            title: "Bocchi".into(),
            episodes: 12,
            score: 90,
        },
    ];
    by_fields.sort();
    println!("derived Ord (title, then episodes, then score):");
    for a in &by_fields {
        println!("  {a:?}");
    }

    let mut by_score = vec![
        AnimeByScore {
            title: "Frieren".into(),
            episodes: 28,
            score: 96,
        },
        AnimeByScore {
            title: "Bocchi".into(),
            episodes: 12,
            score: 98,
        },
    ];
    by_score.sort();
    println!("hand-written Ord (score only):");
    for a in &by_score {
        println!("  {a:?}");
    }

    let nan = f64::NAN;
    println!("1.0.partial_cmp(&NAN): {:?}", 1.0_f64.partial_cmp(&nan));
    println!("NAN.partial_cmp(&NAN): {:?}", nan.partial_cmp(&nan));
}
