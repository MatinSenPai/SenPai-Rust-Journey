//! An enum is a type, so it gets an `impl` block like any other type.
//!
//!     cargo run -p p1-05-03-enums-as-data --example 03-methods-on-enums

// `{:?}` does read every field below, but dead-code analysis deliberately
// ignores a derived `Debug` — so without this the run is buried in warnings
// about data that is in fact used.
#![allow(dead_code)]

#[derive(Debug)]
enum Entry {
    Planned,
    Watching(u32),
    Rated { score: u8 },
    Dropped { episode: u32, reason: String },
}

impl Entry {
    /// An associated function — no `self`, called through the type. This is
    /// the state a brand-new entry starts in.
    fn new() -> Self {
        Entry::Planned
    }

    /// Another one. Naming the constructor is often nicer than writing the
    /// variant out at every call site.
    fn start(episode: u32) -> Self {
        Entry::Watching(episode)
    }

    /// `matches!` answers one question: is this value that shape? The `_`
    /// says "carrying anything".
    fn is_watching(&self) -> bool {
        matches!(self, Entry::Watching(_))
    }

    /// `..` says "and whatever named fields it has". The `|` means "or".
    fn is_done(&self) -> bool {
        matches!(self, Entry::Rated { .. } | Entry::Dropped { .. })
    }

    /// A trailing `if` looks at the data the variant carries, not just at
    /// which variant it is.
    fn is_favourite(&self) -> bool {
        matches!(self, Entry::Rated { score } if *score >= 8)
    }
}

fn main() {
    let fresh = Entry::new();
    let midway = Entry::start(7);
    let loved = Entry::Rated { score: 9 };
    let meh = Entry::Rated { score: 4 };
    let gone = Entry::Dropped {
        episode: 3,
        reason: String::from("too slow"),
    };

    for entry in [&fresh, &midway, &loved, &meh, &gone] {
        println!(
            "{:?}\n    watching: {}  done: {}  favourite: {}",
            entry,
            entry.is_watching(),
            entry.is_done(),
            entry.is_favourite()
        );
    }
}
