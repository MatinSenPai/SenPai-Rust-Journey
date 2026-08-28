//! `.filter()` keeps items a predicate approves of. `.filter_map()` does a
//! `.map()` and a `.filter()` in the same pass: run a closure that returns
//! `Option<U>`, keep the `Some`s, drop the `None`s.
//!
//!     cargo run -p p2-02-02-iterator-adapters --example 03-filter-and-filter-map

struct Show {
    title: String,
    completed: bool,
}

fn main() {
    let watchlist = vec![
        Show {
            title: "Frieren".to_string(),
            completed: true,
        },
        Show {
            title: "Bocchi the Rock!".to_string(),
            completed: false,
        },
        Show {
            title: "Mushoku Tensei".to_string(),
            completed: true,
        },
    ];

    for show in watchlist.iter().filter(|show| show.completed) {
        println!("completed: {}", show.title);
    }

    println!();

    // User-typed episode counts. Some of them are not valid numbers at all.
    let typed = vec!["12", "twelve", "24", "", "37"];

    // The naive way: `.map()` to `Result`, and now every item is wrapped
    // whether it parsed or not — you still have to unwrap each one.
    for parsed in typed.iter().map(|text| text.parse::<u32>()) {
        println!("map only: {parsed:?}");
    }

    println!();

    // `.filter_map()` runs the same parse, but a closure returning
    // `Option<U>` means the failures just vanish from the sequence.
    for episodes in typed.iter().filter_map(|text| text.parse::<u32>().ok()) {
        println!("filter_map: {episodes}");
    }
}
