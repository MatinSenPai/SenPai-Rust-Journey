//! DELIBERATELY BROKEN — expected: E0308
//!
//! `AsRef<str>` promises `fn as_ref(&self) -> &str`. `Watchlist` wraps a
//! `Vec<String>`, and `&self.0` is a `&Vec<String>`, not a `&str` — the
//! promised return type and the actual one do not match.
//!
//!     cargo run -p p2-04-03-deref-asref-borrow --example 08-asref-wrong-return-type-broken --features broken

struct Watchlist(Vec<String>);

impl AsRef<str> for Watchlist {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

fn main() {
    let list = Watchlist(vec!["Frieren".to_string()]);
    println!("{}", list.as_ref());
}
