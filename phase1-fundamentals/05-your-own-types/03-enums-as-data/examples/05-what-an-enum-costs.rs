//! How big is an enum? Big enough for its largest variant, plus a tag —
//! rounded up for alignment. `size_of` shows all of it.
//!
//!     cargo run -p p1-05-03-enums-as-data --example 05-what-an-enum-costs

use std::mem::size_of;

// These types exist to be measured, never to be built, so the unconstructed
// variants are expected.
#[allow(dead_code)]
mod shapes {
    /// Three names, no data. One byte is enough to say which.
    pub enum Medium {
        Anime,
        Manga,
        Webtoon,
    }

    /// Two variants, both carrying one byte.
    pub enum Small {
        A(u8),
        B(u8),
    }

    /// One tiny variant and one wide one. The wide one sets the size.
    pub enum Wide {
        Tiny(u8),
        Big(u64),
    }

    /// The lesson's running example. Its widest variant carries a `u32` and
    /// a `String`.
    pub enum Entry {
        Planned,
        Watching(u32),
        Rated { score: u8 },
        Dropped { episode: u32, reason: String },
    }
}

use shapes::{Entry, Medium, Small, Wide};

fn main() {
    println!("Medium:                {} bytes", size_of::<Medium>());
    println!("Small:                 {} bytes", size_of::<Small>());
    println!("u8:                    {} bytes", size_of::<u8>());

    println!();
    println!("Wide:                  {} bytes", size_of::<Wide>());
    println!("u64:                   {} bytes", size_of::<u64>());

    println!();
    println!("String:                {} bytes", size_of::<String>());
    println!("u32:                   {} bytes", size_of::<u32>());
    println!("Entry:                 {} bytes", size_of::<Entry>());

    // The standard library's enums measure the same way.
    println!();
    println!("i32:                   {} bytes", size_of::<i32>());
    println!("Option<i32>:           {} bytes", size_of::<Option<i32>>());
    println!("bool:                  {} bytes", size_of::<bool>());
    println!("Option<bool>:          {} bytes", size_of::<Option<bool>>());
    println!(
        "Result<i32, String>:   {} bytes",
        size_of::<Result<i32, String>>()
    );

    // And then this one, which does not follow the rule.
    println!();
    println!("Box<i32>:              {} bytes", size_of::<Box<i32>>());
    println!(
        "Option<Box<i32>>:      {} bytes",
        size_of::<Option<Box<i32>>>()
    );
}
