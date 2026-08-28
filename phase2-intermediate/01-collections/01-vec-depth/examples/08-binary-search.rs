//! `.binary_search()` and `.binary_search_by()` only work correctly on a
//! `Vec` that is ALREADY sorted by whatever you're searching on. Given
//! that, they find a match in O(log n) instead of walking every element.
//!
//!     cargo run -p p2-01-01-vec-depth --example 08-binary-search

struct Anime {
    title: String,
    rating: f64,
}

fn anime(title: &str, rating: f64) -> Anime {
    Anime {
        title: title.to_string(),
        rating,
    }
}

fn look_up(by_rating: &[Anime], target: f64) {
    match by_rating.binary_search_by(|entry| entry.rating.total_cmp(&target)) {
        Ok(index) => println!(
            "rating {target}: found {:?} at index {index}",
            by_rating[index].title
        ),
        Err(insert_at) => println!("rating {target}: not found, belongs at index {insert_at}"),
    }
}

fn main() {
    // Sorted by rating, ascending — binary_search's one hard requirement.
    let by_rating = vec![
        anime("Made in Abyss", 6.5),
        anime("Bocchi the Rock!", 7.5),
        anime("Frieren", 9.0),
        anime("Mushoku Tensei", 9.5),
    ];

    look_up(&by_rating, 9.0);
    look_up(&by_rating, 8.0);
}
