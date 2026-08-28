//! DELIBERATELY BROKEN — expected: E0277
//!
//! `HashMap<String, V>::get<Q>` needs `String: Borrow<Q>`. The standard
//! library only gives `String` a `Borrow<str>` (and the trivial
//! `Borrow<String>`) — never a `Borrow<i32>` — so asking for an entry with a
//! `&5` does not typecheck, regardless of what the map holds.
//!
//!     cargo run -p p2-04-03-deref-asref-borrow --example 09-hashmap-get-wrong-query-broken --features broken

use std::collections::HashMap;

fn main() {
    let ratings: HashMap<String, u8> = HashMap::new();
    println!("{:?}", ratings.get(&5));
}
