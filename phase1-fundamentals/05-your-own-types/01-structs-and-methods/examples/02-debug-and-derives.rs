//! What `#[derive(Debug, Clone, PartialEq)]` writes for you.
//!
//!     cargo run -p p1-05-01-structs-and-methods --example 02-debug-and-derives

/// `derive` means "write the obvious implementation for me", field by field.
#[derive(Debug, Clone, PartialEq)]
struct Anime {
    title: String,
    episodes: u32,
    watched: u32,
    favourite: bool,
}

fn main() {
    let bebop = Anime {
        title: String::from("Cowboy Bebop"),
        episodes: 26,
        watched: 26,
        favourite: true,
    };

    // `{:?}` — one line. For a log.
    println!("{bebop:?}");

    println!();

    // `{:#?}` — the same information, one field per line. For a human.
    println!("{bebop:#?}");

    // `Clone` clones every field. The `String` gets a second buffer; the
    // three small fields are copied. Four fields, one allocation.
    let copy = bebop.clone();

    // `PartialEq` compares every field. Two separate values, equal contents.
    println!();
    println!("bebop == copy:   {}", bebop == copy);

    let mut edited = bebop.clone();
    edited.watched = 1;
    println!("bebop == edited: {}", bebop == edited);
    println!(
        "they differ at:  watched {} vs {}",
        bebop.watched, edited.watched
    );
}
