//! `.flat_map()` — map each item to its OWN small iterator, then flatten
//! all of those into one stream. It is `.map()` and "un-nest the result"
//! done in a single adapter.
//!
//!     cargo run -p p2-02-02-iterator-adapters --example 08-flat-map

struct Show {
    title: String,
    genres: Vec<String>,
}

fn main() {
    let watchlist = vec![
        Show {
            title: "Frieren".to_string(),
            genres: vec!["fantasy".to_string(), "adventure".to_string()],
        },
        Show {
            title: "Bocchi the Rock!".to_string(),
            genres: vec!["comedy".to_string(), "music".to_string()],
        },
    ];

    for show in watchlist.iter() {
        println!("{}: {:?}", show.title, show.genres);
    }

    println!();

    // Plain `.map()` leaves you with an iterator of `Vec<String>` — a nested
    // shape, one list of genres per show. You would need a second loop just
    // to get at the genres themselves.
    for genre_list in watchlist.iter().map(|show| &show.genres) {
        println!("map only, still nested: {genre_list:?}");
    }

    println!();

    // `.flat_map()` maps each show to its genre iterator AND flattens the
    // result in the same step — one flat stream of genres, no nesting left,
    // and no record left of which show each genre came from.
    for genre in watchlist.iter().flat_map(|show| show.genres.iter()) {
        println!("flat_map: {genre}");
    }

    println!();

    // It composes exactly like every other adapter.
    for genre in watchlist
        .iter()
        .flat_map(|show| show.genres.iter())
        .filter(|genre| genre.len() > 5)
    {
        println!("flat_map + filter: {genre}");
    }
}
