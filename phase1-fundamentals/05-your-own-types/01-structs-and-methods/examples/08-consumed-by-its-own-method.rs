//! DELIBERATELY BROKEN — expected: E0382.
//!
//!     cargo run -p p1-05-01-structs-and-methods --example 08-consumed-by-its-own-method --features broken
//!
//! A method taking `self` is a move, and the move is invisible at the call
//! site. This is 1.2.2 arriving on a type you wrote yourself.

struct Anime {
    title: String,
    episodes: u32,
}

impl Anime {
    fn into_title(self) -> String {
        self.title
    }
}

fn main() {
    let show = Anime {
        title: String::from("Frieren"),
        episodes: 28,
    };

    let title = show.into_title();

    println!("{title} has {} episodes", show.episodes);
}
