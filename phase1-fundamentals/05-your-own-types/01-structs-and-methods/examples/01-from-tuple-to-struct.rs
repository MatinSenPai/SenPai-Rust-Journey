//! Four values that belong together — first as a tuple, then with names.
//!
//!     cargo run -p p1-05-01-structs-and-methods --example 01-from-tuple-to-struct

/// A show on the watch list: what it is called, how long it is, how far we
/// got, and whether it is a favourite.
struct Anime {
    title: String,
    episodes: u32,
    watched: u32,
    favourite: bool,
}

fn main() {
    // The tuple from 1.1.3. It compiles, and every read is a puzzle: which
    // of those two `u32`s was the length and which was the progress?
    let as_tuple = (String::from("Cowboy Bebop"), 26_u32, 26_u32, true);
    println!("tuple:   {} {} {}", as_tuple.0, as_tuple.1, as_tuple.3);

    // The same four values, with the names fixed by the type.
    let bebop = Anime {
        title: String::from("Cowboy Bebop"),
        episodes: 26,
        watched: 26,
        favourite: true,
    };
    println!(
        "struct:  {} {}/{} favourite={}",
        bebop.title, bebop.watched, bebop.episodes, bebop.favourite
    );

    // `mut` goes on the binding, not on a field. One `mut` unlocks the whole
    // struct; there is no way to make a single field mutable on its own.
    let mut frieren = Anime {
        title: String::from("Frieren"),
        episodes: 28,
        watched: 3,
        favourite: false,
    };
    frieren.watched += 1;
    frieren.favourite = true;
    println!(
        "changed: {} {}/{} favourite={}",
        frieren.title, frieren.watched, frieren.episodes, frieren.favourite
    );

    // Field init shorthand, inside `start_watching` below.
    let started = start_watching(String::from("Mushishi"), 26);
    println!(
        "started: {} {}/{}",
        started.title, started.watched, started.episodes
    );

    // Struct update syntax: "these fields, and every other one from that
    // value over there".
    let rewatch = Anime {
        watched: 0,
        favourite: true,
        ..started
    };
    println!(
        "rewatch: {} {}/{}",
        rewatch.title, rewatch.watched, rewatch.episodes
    );

    // `..started` MOVED the `String` out of `started`. The two `u32` fields
    // are `Copy`, so they were copied and are still readable — but `started`
    // as a whole value is gone.
    println!("left of started: episodes {}", started.episodes);
}

/// A series just added to the list: nothing watched, not yet a favourite.
///
/// `title` and `episodes` are written once each, not twice: when the variable
/// already carries the field's name, the field name alone is enough.
fn start_watching(title: String, episodes: u32) -> Anime {
    let watched = 0;
    Anime {
        title,
        episodes,
        watched,
        favourite: false,
    }
}
