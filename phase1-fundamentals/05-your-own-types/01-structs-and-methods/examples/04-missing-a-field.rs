//! DELIBERATELY BROKEN — expected: E0063.
//!
//!     cargo run -p p1-05-01-structs-and-methods --example 04-missing-a-field --features broken
//!
//! A struct literal names every field or it is not a value of that type.

struct Anime {
    title: String,
    episodes: u32,
    watched: u32,
    favourite: bool,
}

fn main() {
    let show = Anime {
        title: String::from("Frieren"),
        episodes: 28,
    };

    println!("{} {}/{}", show.title, show.watched, show.episodes);
}
