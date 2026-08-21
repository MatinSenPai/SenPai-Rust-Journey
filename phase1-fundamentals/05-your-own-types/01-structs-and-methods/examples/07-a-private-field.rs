//! DELIBERATELY BROKEN — expected: E0616.
//!
//!     cargo run -p p1-05-01-structs-and-methods --example 07-a-private-field --features broken
//!
//! `pub` on the struct does not make its fields public. Everything inside
//! `catalog` can see them; nothing outside can. Modules get their own lesson
//! in Phase 2 — all you need here is that `mod catalog { ... }` draws a line
//! and privacy is measured against it.

mod catalog {
    pub struct Anime {
        pub title: String,
        episodes: u32,
        watched: u32,
    }

    impl Anime {
        pub fn new(title: String, episodes: u32) -> Self {
            Self {
                title,
                episodes,
                watched: 0,
            }
        }

        pub fn remaining(&self) -> u32 {
            self.episodes - self.watched
        }
    }
}

fn main() {
    let show = catalog::Anime::new(String::from("Frieren"), 28);

    // Fine: `title` is `pub`, and `remaining` is a `pub` method.
    println!("{} — {} to go", show.title, show.remaining());

    // Not fine: `episodes` was never made public.
    println!("episodes: {}", show.episodes);
}
