//! `.fold(initial, |accumulator, item| ...)` — carry a running value through
//! every item and hand back whatever it became. It is the general-purpose
//! "reduce everything to one value" primitive.
//!
//!     cargo run -p p2-02-02-iterator-adapters --example 07-fold

fn main() {
    let episodes = vec![28, 12, 24];

    let total = episodes.iter().fold(0, |acc, n| acc + n);
    println!("total episodes: {total}");

    // The manual loop `.fold()` is standing in for, so the equivalence is
    // not just a claim:
    let mut total_by_hand = 0;
    for n in episodes.iter() {
        total_by_hand += n;
    }
    println!("same, by hand:  {total_by_hand}");

    println!();

    // `.fold()` is not limited to numbers — the accumulator can be
    // anything, including a `Vec` you build up one push at a time. This is
    // how you turn a pipeline into a `Vec` before `.collect()` exists.
    let titles = vec!["frieren", "bocchi the rock!"];
    let shouted: Vec<String> = titles.iter().fold(Vec::new(), |mut acc, title| {
        acc.push(title.to_uppercase());
        acc
    });
    println!("built with fold: {shouted:?}");
}
