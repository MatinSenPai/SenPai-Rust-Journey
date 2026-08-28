//! There is no `#[derive(Display)]`. A derive can mechanically read a
//! struct's fields and know what a debug dump should look like — field
//! names in braces. It cannot guess what a *human-facing* sentence should
//! look like; that is a decision only you can make, so `Display` is always
//! hand-written.
//!
//!     cargo run -p p2-03-04-standard-derives-by-hand --example 02-display-hand-written

use std::fmt;

#[derive(Debug)]
struct Anime {
    title: String,
    episodes: u32,
    score: u8,
}

impl fmt::Display for Anime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} — {} episodes, {}/100",
            self.title, self.episodes, self.score
        )
    }
}

fn main() {
    let a = Anime {
        title: "Frieren".to_string(),
        episodes: 28,
        score: 96,
    };
    println!("Display (for a viewer):    {a}");
    println!("Debug   (for a developer): {a:?}");
}
