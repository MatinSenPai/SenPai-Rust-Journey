//! `&self`, `&mut self`, `self`, and an associated function.
//!
//!     cargo run -p p1-05-01-structs-and-methods --example 03-methods-and-self

#[derive(Debug, Clone)]
struct Anime {
    title: String,
    episodes: u32,
    watched: u32,
    favourite: bool,
}

impl Anime {
    /// An associated function: no `self`, called through the type. `new` is
    /// a convention — nothing in the language knows the name.
    fn new(title: String, episodes: u32) -> Self {
        Self {
            title,
            episodes,
            watched: 0,
            favourite: false,
        }
    }

    /// `&self` — a look. The caller keeps the value.
    fn remaining(&self) -> u32 {
        self.episodes - self.watched
    }

    /// `&mut self` — an exclusive look. The caller keeps the value and sees
    /// the change.
    fn watch_one(&mut self) {
        if self.watched < self.episodes {
            self.watched += 1;
        }
    }

    /// `self` — the value itself. The caller's value is gone afterwards.
    fn into_title(self) -> String {
        self.title
    }
}

fn main() {
    let mut show = Anime::new(String::from("Frieren"), 28);
    println!("remaining:  {}", show.remaining());
    println!("favourite:  {}", show.favourite);

    show.watch_one();
    show.watch_one();
    println!("watched:    {}/{}", show.watched, show.episodes);
    println!("remaining:  {}", show.remaining());

    // `show.remaining()` is sugar. Here is the same call written out, with
    // the `&` you never normally type.
    println!("same call:  {}", Anime::remaining(&show));

    // The `self` method eats the value. `show` is unusable after this line —
    // example 08 is what happens when you try anyway.
    let title = show.into_title();
    println!("title:      {title}");
}
