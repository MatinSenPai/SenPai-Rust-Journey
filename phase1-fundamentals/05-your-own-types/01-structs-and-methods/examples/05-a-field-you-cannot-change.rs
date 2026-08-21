//! DELIBERATELY BROKEN — expected: E0594.
//!
//!     cargo run -p p1-05-01-structs-and-methods --example 05-a-field-you-cannot-change --features broken
//!
//! `mut` belongs to the binding, not to the field. Two errors here, one
//! cause: `show` was never declared mutable.

struct Anime {
    title: String,
    episodes: u32,
    watched: u32,
}

impl Anime {
    fn watch_one(&mut self) {
        self.watched += 1;
    }
}

fn main() {
    let show = Anime {
        title: String::from("Frieren"),
        episodes: 28,
        watched: 3,
    };

    show.watched += 1;
    show.watch_one();

    println!("{} {}/{}", show.title, show.watched, show.episodes);
}
