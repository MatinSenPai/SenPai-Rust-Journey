//! `cargo run -p p2-04-03-deref-asref-borrow --example 02-deref-coercion-chain`
//!
//! Deref coercion chains through more than one `Deref` impl at once. `&String`
//! coercing to `&str` is the familiar case from Phase 1; `&Watchlist` coercing
//! all the way to `&[String]` needs two hops (`Watchlist -> Vec<String>`, then
//! the standard library's own `Vec<T> -> [T]`), and the compiler does both
//! without being asked.

use std::ops::Deref;

struct Watchlist(Vec<String>);

impl Deref for Watchlist {
    type Target = Vec<String>;

    fn deref(&self) -> &Vec<String> {
        &self.0
    }
}

fn print_str(s: &str) {
    println!("str: {s}");
}

fn print_slice(items: &[String]) {
    println!("slice len: {}", items.len());
}

fn main() {
    let owned = String::from("hello");
    print_str(&owned);

    let list = Watchlist(vec!["Frieren".to_string(), "Bocchi the Rock".to_string()]);
    print_slice(&list);
}
