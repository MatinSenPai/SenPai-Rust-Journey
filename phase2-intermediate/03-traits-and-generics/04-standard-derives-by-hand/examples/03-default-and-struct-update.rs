//! `Default::default()` gives a "sensible starting value" for a type.
//! `#[derive(Default)]` needs every field's type to implement `Default`
//! itself. Struct-update syntax (`..Default::default()`) uses it to fill in
//! whatever fields you do not want to set by hand.
//!
//!     cargo run -p p2-03-04-standard-derives-by-hand --example 03-default-and-struct-update

#[derive(Debug, Default)]
struct Anime {
    title: String,
    episodes: u32,
    score: u8,
}

struct AnimeManual {
    title: String,
    episodes: u32,
    score: u8,
}
impl Default for AnimeManual {
    fn default() -> Self {
        AnimeManual {
            title: String::new(),
            episodes: 0,
            score: 0,
        }
    }
}

fn main() {
    let blank = Anime::default();
    println!("derived default:      {blank:?}");

    let manual = AnimeManual::default();
    println!(
        "hand-written default:  title={:?} episodes={} score={}",
        manual.title, manual.episodes, manual.score
    );

    let started = Anime {
        title: "Bocchi the Rock!".to_string(),
        ..Default::default()
    };
    println!("struct-update syntax:  {started:?}");
}
