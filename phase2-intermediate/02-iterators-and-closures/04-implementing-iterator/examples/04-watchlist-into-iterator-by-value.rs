//! `IntoIterator` is the trait a `for` loop actually calls. Implement it for
//! `WatchList` and `for title in queue` becomes legal — without it, that
//! line would not compile at all.
//!
//!     cargo run -p p2-02-04-implementing-iterator --example 04-watchlist-into-iterator-by-value

struct WatchList(Vec<String>);

impl IntoIterator for WatchList {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

fn main() {
    let queue = WatchList(vec![
        "Frieren".to_string(),
        "Bocchi the Rock!".to_string(),
        "Mushoku Tensei".to_string(),
    ]);

    for title in queue {
        println!("now watching: {title}");
    }
}
