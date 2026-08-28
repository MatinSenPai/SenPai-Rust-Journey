//! `.retain(|item| ...)` walks the `Vec` once and keeps only the elements
//! where the closure answers `true` — everything else is dropped, in place,
//! with the survivors keeping their original order.
//!
//!     cargo run -p p2-01-01-vec-depth --example 03-retain

#[derive(Debug)]
struct Anime {
    title: String,
    watched: bool,
}

fn anime(title: &str, watched: bool) -> Anime {
    Anime {
        title: title.to_string(),
        watched,
    }
}

fn main() {
    let mut list = vec![
        anime("Frieren", true),
        anime("Bocchi the Rock!", false),
        anime("Mushoku Tensei", true),
        anime("Made in Abyss", false),
    ];
    println!("before: {list:?}");

    // `|entry| ...` is a closure — a tiny inline function. Phase 2's
    // closures module (2.2.1) explains it properly; for now, read it as
    // "for each entry, answer: keep this one?"
    list.retain(|entry| !entry.watched);

    println!("after:  {list:?}");
}
