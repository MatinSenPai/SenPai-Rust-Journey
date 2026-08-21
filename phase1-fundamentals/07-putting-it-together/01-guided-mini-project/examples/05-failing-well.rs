//! Stage 5 — failing well. One operation that can fail two different ways,
//! an error enum to say which, and `?` converting one of them for free.
//!
//!     cargo run -p p1-07-01-guided-mini-project --example 05-failing-well

use std::num::ParseIntError;

const MAX_RATING: u8 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rating(u8);

impl Rating {
    fn new(value: u8) -> Rating {
        if value > MAX_RATING {
            Rating(MAX_RATING)
        } else {
            Rating(value)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum Status {
    Watching { episode: u32 },
    Finished { rating: Rating },
    Planned,
    Dropped { at: u32 },
}

#[derive(Debug, Clone, PartialEq)]
struct Entry {
    title: String,
    status: Status,
}

// Two ways to fail, named. `String` inside each is enough for now — a real
// service would carry a source error too, and that widening is 1.6.5's own
// lesson, not this one's.
#[derive(Debug, Clone, PartialEq, Eq)]
enum WatchlistError {
    NotFound(String),
    InvalidRating(String),
}

// This is the piece that makes `?` work below. `.parse::<u8>()` fails with
// a `ParseIntError`, and `rate_from_text`'s return type is
// `Result<(), WatchlistError>` — two different error types. `?` bridges
// them by calling exactly this conversion, automatically, every time it
// needs to.
impl From<ParseIntError> for WatchlistError {
    fn from(err: ParseIntError) -> WatchlistError {
        WatchlistError::InvalidRating(err.to_string())
    }
}

struct Watchlist {
    entries: Vec<Entry>,
}

impl Watchlist {
    fn new() -> Watchlist {
        Watchlist {
            entries: Vec::new(),
        }
    }

    fn add(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    // The one operation that can fail: the title might not be in the list.
    // Everything else this lesson built — add, find, titles — cannot fail,
    // and returns no `Result` at all.
    fn rate(&mut self, title: &str, rating: Rating) -> Result<(), WatchlistError> {
        for entry in &mut self.entries {
            if entry.title == title {
                entry.status = Status::Finished { rating };
                return Ok(());
            }
        }
        Err(WatchlistError::NotFound(title.to_string()))
    }

    // The realistic entry point: the rating arrives as text. `?` on the
    // `.parse()` call means "if this is `Err`, convert it with `From` and
    // return that `Err` right now" — no `match`, no early-return block.
    fn rate_from_text(&mut self, title: &str, rating_text: &str) -> Result<(), WatchlistError> {
        let raw: u8 = rating_text.trim().parse()?;
        self.rate(title, Rating::new(raw))
    }
}

fn main() {
    let mut list = Watchlist::new();
    list.add(Entry {
        title: String::from("Frieren"),
        status: Status::Planned,
    });

    println!("{:?}", list.rate("Frieren", Rating::new(9)));
    println!("{:?}", list.rate("Cowboy Bebop", Rating::new(7)));

    println!();
    println!("{:?}", list.rate_from_text("Frieren", "8"));
    println!("{:?}", list.rate_from_text("Frieren", "nine"));
    println!("{:?}", list.rate_from_text("Cowboy Bebop", "nine"));
}
