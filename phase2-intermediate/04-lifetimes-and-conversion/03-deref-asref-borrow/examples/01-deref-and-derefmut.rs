//! `cargo run -p p2-04-03-deref-asref-borrow --example 01-deref-and-derefmut`
//!
//! A newtype wrapper around `Vec<String>` that behaves like its inner `Vec`
//! at method-call sites and through the `*` operator, once `Deref` and
//! `DerefMut` are implemented for it.

use std::ops::{Deref, DerefMut};

struct Watchlist(Vec<String>);

impl Deref for Watchlist {
    type Target = Vec<String>;

    fn deref(&self) -> &Vec<String> {
        &self.0
    }
}

impl DerefMut for Watchlist {
    fn deref_mut(&mut self) -> &mut Vec<String> {
        &mut self.0
    }
}

fn main() {
    let mut list = Watchlist(vec!["Frieren".to_string(), "Bocchi the Rock".to_string()]);

    println!("count: {}", list.len());
    list.push("Trigun".to_string());
    println!("first: {}", list[0]);
    println!("via *: {}", (*list).len());

    let inner: &Vec<String> = &list;
    println!("coerced: {inner:?}");
}
