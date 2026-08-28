//! `.map()` — transform every item, one at a time, lazily.
//!
//!     cargo run -p p2-02-02-iterator-adapters --example 02-map

fn main() {
    let titles = vec!["frieren".to_string(), "bocchi the rock!".to_string()];

    let shouting = titles.iter().map(|title| title.to_uppercase());
    // `shouting` has not touched a single character yet — it is a `Map`
    // value describing "when someone asks, uppercase the next title."

    for title in shouting {
        println!("{title}");
    }

    println!();

    // `.map()` does not care what shape it gets in or hands back — here it
    // goes from `&String` in to a completely different type, `usize`, out.
    for length in titles.iter().map(|title| title.len()) {
        println!("length: {length}");
    }
}
