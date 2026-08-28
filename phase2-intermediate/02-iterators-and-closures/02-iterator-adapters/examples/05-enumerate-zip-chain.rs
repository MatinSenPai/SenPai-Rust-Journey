//! Three ways to combine iterators: `.enumerate()` pairs items with their
//! position, `.zip()` pairs two iterators up, `.chain()` runs one after
//! another.
//!
//!     cargo run -p p2-02-02-iterator-adapters --example 05-enumerate-zip-chain

fn main() {
    let titles = vec!["Frieren", "Bocchi the Rock!", "Mushoku Tensei"];

    for (index, title) in titles.iter().enumerate() {
        println!("enumerate: {index} -> {title}");
    }

    println!();

    let ratings = vec![9, 8];
    // `titles` has 3 items, `ratings` only 2 — `.zip()` stops the instant
    // EITHER side runs out. "Mushoku Tensei" never appears below.
    for (title, rating) in titles.iter().zip(ratings.iter()) {
        println!("zip: {title} rated {rating}");
    }

    println!();

    let already_watched = vec!["Frieren", "AOT"];
    let plan_to_watch = vec!["Bocchi the Rock!", "Chainsaw Man"];
    // `.chain()` needs both sides to yield the same item type — here,
    // `&&str` from each `Vec<&str>` — and just runs the second after the
    // first is exhausted. No new collection is built to hold both.
    for title in already_watched.iter().chain(plan_to_watch.iter()) {
        println!("chain: {title}");
    }
}
