//! DELIBERATELY BROKEN — expected: E0277.
//!
//! Implementing `IntoIterator for WatchList` does not also implement it for
//! `&WatchList` — that is a separate `impl`, on a separate type, that this
//! lesson never wrote. `for title in &queue` needs exactly that `impl` and
//! does not have it.
//!
//!     cargo run -p p2-02-04-implementing-iterator --example 09-reference-not-into-iterator --features broken

struct WatchList(Vec<String>);

impl IntoIterator for WatchList {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

fn main() {
    let queue = WatchList(vec!["Frieren".to_string()]);

    for title in &queue {
        println!("now watching: {title}");
    }
}
