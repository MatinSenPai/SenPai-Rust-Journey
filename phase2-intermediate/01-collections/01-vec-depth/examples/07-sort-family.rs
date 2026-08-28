//! `.sort()` only needs a type it can put in one total order — plain
//! numbers and text qualify, so no closure required. `.sort_by()` and
//! `.sort_unstable_by()` take that comparison as an argument instead, for
//! everything else (like `f64`, which has no total order of its own).
//!
//!     cargo run -p p2-01-01-vec-depth --example 07-sort-family

#[derive(Debug, Clone)]
struct Anime {
    title: String,
    rating: f64,
}

fn titles(entries: &[Anime]) -> Vec<&str> {
    let mut out = Vec::new();
    for entry in entries {
        out.push(entry.title.as_str());
    }
    out
}

fn main() {
    let mut episode_counts = vec![24, 12, 64, 12, 1];
    episode_counts.sort();
    println!("plain .sort():           {episode_counts:?}");

    // Two entries tie at 7.5. "B" was logged after "A" in the input.
    let by_input_order = vec![
        Anime {
            title: "A".to_string(),
            rating: 7.5,
        },
        Anime {
            title: "C".to_string(),
            rating: 9.0,
        },
        Anime {
            title: "B".to_string(),
            rating: 7.5,
        },
    ];

    let mut stable = by_input_order.clone();
    stable.sort_by(|a, b| a.rating.total_cmp(&b.rating));
    println!();
    println!("stable   .sort_by():           {:?}", titles(&stable));

    let mut unstable = by_input_order.clone();
    unstable.sort_unstable_by(|a, b| a.rating.total_cmp(&b.rating));
    println!(
        "unstable .sort_unstable_by():  {:?}  (no promise, just what this run did)",
        titles(&unstable)
    );
}
