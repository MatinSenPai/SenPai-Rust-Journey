//! One type, several shapes. Every kind of variant an enum can have, in one
//! place.
//!
//!     cargo run -p p1-05-03-enums-as-data --example 01-three-shapes

// `{:?}` does read every field below, but dead-code analysis deliberately
// ignores a derived `Debug` — so without this the run is buried in warnings
// about data that is in fact used.
#![allow(dead_code)]

/// Names and nothing else — *unit variants*. This is the enum a C or Java
/// programmer already knows.
#[derive(Debug)]
enum Medium {
    Anime,
    Manga,
    Webtoon,
}

/// A variant may carry data with no field names — a *tuple variant*. The
/// two variants carry different types, and that is allowed.
#[derive(Debug)]
enum Episode {
    Numbered(u32),
    Special(String),
}

/// A variant may carry named data — a *struct variant*. And all three kinds
/// can share one enum, because every variant picks its own shape.
#[derive(Debug)]
enum Entry {
    Planned,
    Watching(u32),
    Rated { score: u8 },
    Dropped { episode: u32, reason: String },
}

fn main() {
    println!("anime:    {:?}", Medium::Anime);
    println!("manga:    {:?}", Medium::Manga);
    println!("webtoon:  {:?}", Medium::Webtoon);

    println!();
    println!("numbered: {:?}", Episode::Numbered(12));
    println!("special:  {:?}", Episode::Special(String::from("OVA")));

    println!();
    println!("planned:  {:?}", Entry::Planned);
    println!("watching: {:?}", Entry::Watching(7));
    println!("rated:    {:?}", Entry::Rated { score: 9 });
    println!(
        "dropped:  {:?}",
        Entry::Dropped {
            episode: 3,
            reason: String::from("too slow"),
        }
    );

    // Every one of those five lines produced a value of one single type, so
    // they go in one Vec together. There is no sixth shape an `Entry` could
    // be, and the compiler knows it.
    let library = vec![
        Entry::Planned,
        Entry::Watching(7),
        Entry::Rated { score: 9 },
    ];
    println!();
    println!("library:  {library:?}");
}
