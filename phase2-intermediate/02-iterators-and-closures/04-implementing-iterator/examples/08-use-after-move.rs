//! DELIBERATELY BROKEN — expected: E0382.
//!
//! `for title in queue` calls `WatchList`'s by-value `into_iter(self)` —
//! the same move Phase 1 already taught you to expect from `for x in v`.
//! `queue` is gone once the loop is done with it.
//!
//!     cargo run -p p2-02-04-implementing-iterator --example 08-use-after-move --features broken

#[derive(Debug)]
struct WatchList(Vec<String>);

impl IntoIterator for WatchList {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

fn main() {
    let queue = WatchList(vec!["Frieren".to_string(), "Bocchi the Rock!".to_string()]);

    for title in queue {
        println!("now watching: {title}");
    }

    println!("queue again: {queue:?}");
}
