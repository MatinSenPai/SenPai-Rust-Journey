//! DELIBERATELY BROKEN — expected: E0277.
//!
//!     cargo run -p p1-05-01-structs-and-methods --example 06-printing-without-debug --features broken
//!
//! `{:?}` is not free. Somebody has to say how the type prints.

struct Anime {
    title: String,
    episodes: u32,
    watched: u32,
}

fn main() {
    let show = Anime {
        title: String::from("Frieren"),
        episodes: 28,
        watched: 3,
    };

    println!("{show:?}");
}
