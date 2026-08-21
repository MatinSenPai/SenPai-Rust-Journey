//! DELIBERATELY BROKEN — expected: E0369
//!
//!     cargo run -p p1-05-03-enums-as-data --example 09-cannot-compare --features broken
//!
//! `#[derive(Debug)]` buys you printing. Comparing is a separate ability.

#[derive(Debug)]
enum Medium {
    Anime,
    Manga,
    Webtoon,
}

fn main() {
    let chosen = Medium::Manga;
    println!("chosen: {chosen:?}");
    println!("others: {:?} {:?}", Medium::Anime, Medium::Webtoon);

    if chosen == Medium::Manga {
        println!("it is manga");
    }
}
