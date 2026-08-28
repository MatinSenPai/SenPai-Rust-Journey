//! Stage 1 — the data. An enum whose variants carry different things, a
//! struct that names an entry, and a newtype that cannot be built wrong.
//!
//!     cargo run -p p1-07-01-guided-mini-project --example 01-the-data

const MAX_RATING: u8 = 10;

// The newtype pattern from 1.5.2: one field, a validating constructor, an
// accessor. Nothing outside this file can build a `Rating` that holds more
// than `MAX_RATING`, because the field is private and `new` is the only
// door in.
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

    fn value(self) -> u8 {
        self.0
    }
}

// Four variants, three different shapes — unit, tuple-with-a-field, and
// struct-with-a-field — exactly the enum from 1.5.3, renamed for this
// domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

fn main() {
    let bebop = Entry {
        title: String::from("Cowboy Bebop"),
        status: Status::Watching { episode: 9 },
    };
    let frieren = Entry {
        title: String::from("Frieren"),
        status: Status::Finished {
            rating: Rating::new(9),
        },
    };
    let aot = Entry {
        title: String::from("حمله به تایتان"),
        status: Status::Dropped { at: 12 },
    };
    let bocchi = Entry {
        title: String::from("Bocchi the Rock!"),
        status: Status::Planned,
    };

    println!("{bebop:?}");
    println!("{frieren:?}");
    println!("{aot:?}");
    println!("{bocchi:?}");

    // The validation boundary: nothing you pass Rating::new can escape the
    // clamp, because there is no other way to build one.
    println!();
    println!("Rating::new(9):   {}", Rating::new(9).value());
    println!("Rating::new(255): {}", Rating::new(255).value());
}
