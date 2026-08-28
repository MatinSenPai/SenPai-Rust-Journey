//! DELIBERATELY BROKEN — expected: E0596
//!
//! `Watchlist` implements `Deref` but not `DerefMut`. Reading through it
//! (`.len()`) auto-derefs fine; a method that needs `&mut self` (`.push()`)
//! has no `&mut Vec<String>` to auto-deref to, because that step only exists
//! on `DerefMut`.
//!
//!     cargo run -p p2-04-03-deref-asref-borrow --example 07-derefmut-missing-broken --features broken

use std::ops::Deref;

struct Watchlist(Vec<String>);

impl Deref for Watchlist {
    type Target = Vec<String>;

    fn deref(&self) -> &Vec<String> {
        &self.0
    }
}

fn main() {
    let list = Watchlist(vec!["Frieren".to_string()]);
    list.push("Trigun".to_string());
    println!("{}", list.len());
}
